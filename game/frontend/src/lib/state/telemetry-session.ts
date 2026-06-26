// Telemetry session lifecycle helpers. Extracted from match-store so they
// can be unit-tested without booting the Svelte compiler (runes).
//
// These helpers mutate `match.telemetryMatchId` and read `match.mode`
// through the passed reference. Failures are swallowed: telemetry must
// never block gameplay. If IDB is unavailable (e.g. private browsing in
// some Safari modes), the match still plays — it just doesn't get logged.

import type { EngineClient } from "../engine/types";
import { getTelemetryStore, type EndReason } from "../storage";
import type { MatchMode } from "./match-store.svelte";

/** Subset of MatchState the telemetry helpers read/write. The Svelte $state
 *  store satisfies this. Tests can pass a plain object. */
export interface TelemetryCarrier {
  telemetryMatchId: string | null;
}

let telemetryDisabledForSession = false;

function logFail(stage: string, e: unknown): void {
  // eslint-disable-next-line no-console
  console.warn(`[telemetry] ${stage} failed:`, e);
}

/** Starts a telemetry session for the given mode. No-op (returns null) for
 *  sandbox / replay / idle. Sets `carrier.telemetryMatchId` on success. */
export async function startTelemetrySession(
  carrier: TelemetryCarrier,
  mode: MatchMode,
  opts: { multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null } = {},
): Promise<string | null> {
  if (mode === "sandbox" || mode === "replay" || mode === "idle") return null;
  // L7c authoritative-host model: only the host owns an IDB row per match.
  // Joiner uses the `joined_codes` store instead (in lobby) — no `matches`
  // row, so the duplicate-recent-sessions-card bug can't recur.
  if (opts.multiplayerRole === "joiner") {
    carrier.telemetryMatchId = null;
    return null;
  }
  telemetryDisabledForSession = false;
  try {
    const store = getTelemetryStore();
    const id = await store.startMatch({
      mode,
      multiplayerCode: opts.multiplayerCode ?? null,
      multiplayerRole: opts.multiplayerRole ?? null,
    });
    carrier.telemetryMatchId = id;
    return id;
  } catch (e) {
    logFail("startTelemetrySession", e);
    telemetryDisabledForSession = true;
    carrier.telemetryMatchId = null;
    return null;
  }
}

/** Reads the engine's latest PlyRecord and appends it to the active
 *  telemetry session. Safe to call after every apply; cheap no-op if no
 *  session is active. Also checkpoints the engine's consolidated MatchLog
 *  to the match row so a tab close immediately after this ply produces a
 *  resume snapshot reflecting every applied move, not just the snapshot
 *  taken at the previous disconnect. */
export async function recordPly(carrier: TelemetryCarrier, eng: EngineClient): Promise<void> {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  try {
    const plyJson = await eng.latestPlyJson();
    if (!plyJson) return;
    const plyNo = extractPlyNo(plyJson);
    if (plyNo === null) {
      // Engine returned a record without a parseable ply_no — surface loudly
      // rather than silently dropping. The library row will be incomplete but
      // a console.warn lets us notice serialiser drift in dev.
      console.warn(
        "[telemetry] recordPly: latestPlyJson missing ply_no; dropping entry.",
        plyJson.slice(0, 200),
      );
      return;
    }
    const store = getTelemetryStore();
    await store.appendPly(carrier.telemetryMatchId, plyJson, plyNo);
    // Refresh the consolidated MatchLog on the matches row too. Without
    // this, `markNetworkLost` is the only writer of `matchLogJson` — and it
    // is a one-shot transition that no-ops after the first flip, so any
    // plies applied AFTER the row turned `mid-match-network-lost` would be
    // dropped from the resume snapshot.
    try {
      const logJson = await eng.matchLogJson();
      if (logJson) await store.checkpointMatchLog(carrier.telemetryMatchId, logJson);
    } catch {
      // Engine without auto_log on, or a transient failure — appendPly
      // already succeeded, so the per-ply log is preserved. Skip silently.
    }
  } catch (e) {
    logFail("recordPly", e);
  }
}

export function extractPlyNo(plyJson: string): number | null {
  // Engine emits `"ply_no": <u32>` as the first field of PlyRecord. Pull
  // it with a regex rather than JSON.parse to keep this hot path light.
  // Fallback: parse if regex misses (e.g. serialiser changes field order).
  const m = plyJson.match(/"ply_no"\s*:\s*(\d+)/);
  if (m) return Number(m[1]);
  try {
    const obj = JSON.parse(plyJson) as { ply_no?: number };
    return typeof obj.ply_no === "number" ? obj.ply_no : null;
  } catch {
    return null;
  }
}

/** Pulls the full match log from the engine and writes the consolidated
 *  record. Caller is responsible for having called eng.finaliseLog(...)
 *  beforehand (the engine stamps result/final_fen there). */
export async function finalizeTelemetrySession(
  carrier: TelemetryCarrier,
  eng: EngineClient,
  endReason: EndReason,
  resultByte: 0 | 1 | 2 | 3,
): Promise<void> {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  const id = carrier.telemetryMatchId;
  carrier.telemetryMatchId = null;
  try {
    const logJson = await eng.matchLogJson();
    if (!logJson) return;
    let totalPlies = 0;
    let totalWallMs = 0;
    try {
      const parsed = JSON.parse(logJson) as {
        total_plies?: number;
        total_wall_ms?: number;
        plies?: unknown[];
      };
      // Prefer engine-reported totals; if `total_plies` is absent, fall back
      // to the length of the `plies` array (still better than 0). `total_wall_ms`
      // has no safe fallback — leave at 0 if absent.
      totalPlies = parsed.total_plies ?? (Array.isArray(parsed.plies) ? parsed.plies.length : 0);
      totalWallMs = parsed.total_wall_ms ?? 0;
    } catch (parseErr) {
      // The library reads the consolidated log directly; finalising with an
      // unparseable string would corrupt the row. Surface and bail so the
      // session stays in-progress and the user can export the raw blob via
      // the inspector instead.
      logFail("finalizeTelemetrySession.parseLog", parseErr);
      carrier.telemetryMatchId = id; // restore so a retry path is possible
      return;
    }
    const store = getTelemetryStore();
    await store.finalizeMatch(id, logJson, endReason, resultByte, totalPlies, totalWallMs);
  } catch (e) {
    logFail("finalizeTelemetrySession", e);
  }
}

/** Marks an in-progress telemetry session as abandoned. Called on /match/
 *  teardown when no natural end was observed. Per-ply records are kept.
 *  If `eng` is provided, the engine's current MatchLog view is captured so
 *  the library can still open the abandoned match in the inspector. */
export async function abandonTelemetrySession(
  carrier: TelemetryCarrier,
  eng?: EngineClient,
): Promise<void> {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  const id = carrier.telemetryMatchId;
  carrier.telemetryMatchId = null;
  try {
    let partial: string | undefined;
    if (eng) {
      try {
        partial = (await eng.matchLogJson()) ?? undefined;
      } catch {
        // Engine in a bad state — store the abandonment marker without a log.
      }
    }
    const store = getTelemetryStore();
    await store.markAbandoned(id, partial);
  } catch (e) {
    logFail("abandonTelemetrySession", e);
  }
}

/** Synchronous-entry variant of `networkLostTelemetrySession` for the
 *  `pagehide` event path. Skips the `await eng.matchLogJson()` round-trip —
 *  `recordPly` already checkpoints the consolidated MatchLog after every
 *  applied ply, so the matches row's `matchLogJson` is up-to-date when we
 *  enter pagehide. The remaining IDB write is fired without `await`; the
 *  browser may discard it on tab close, and we accept that loss (the row
 *  stays `in-progress` until the next session sweeps it). Returns nothing
 *  because no caller awaits — pagehide handlers can't block teardown. */
export function networkLostTelemetrySessionSync(carrier: TelemetryCarrier): void {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  const id = carrier.telemetryMatchId;
  carrier.telemetryMatchId = null;
  try {
    const store = getTelemetryStore();
    void store.markNetworkLost(id).catch((e) => logFail("networkLostTelemetrySessionSync", e));
  } catch (e) {
    logFail("networkLostTelemetrySessionSync", e);
  }
}

/** Synchronous-entry variant of `abandonTelemetrySession` for the `pagehide`
 *  event path. See `networkLostTelemetrySessionSync` for the rationale. */
export function abandonTelemetrySessionSync(carrier: TelemetryCarrier): void {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  const id = carrier.telemetryMatchId;
  carrier.telemetryMatchId = null;
  try {
    const store = getTelemetryStore();
    void store.markAbandoned(id).catch((e) => logFail("abandonTelemetrySessionSync", e));
  } catch (e) {
    logFail("abandonTelemetrySessionSync", e);
  }
}

/** Marks an in-progress multiplayer telemetry session as network-lost.
 *  Used by /match/ when the user leaves the route during a multiplayer match
 *  (tab close, navigation away). Mirrors `abandonTelemetrySession`'s
 *  swallow-errors policy and partial-log capture. The lobby's recent-sessions
 *  card list reads rows in this state. */
export async function networkLostTelemetrySession(
  carrier: TelemetryCarrier,
  eng?: EngineClient,
): Promise<void> {
  if (!carrier.telemetryMatchId || telemetryDisabledForSession) return;
  const id = carrier.telemetryMatchId;
  carrier.telemetryMatchId = null;
  try {
    let partial: string | undefined;
    if (eng) {
      try {
        partial = (await eng.matchLogJson()) ?? undefined;
      } catch {
        // Engine in a bad state — store the marker without a log.
      }
    }
    const store = getTelemetryStore();
    await store.markNetworkLost(id, partial);
  } catch (e) {
    logFail("networkLostTelemetrySession", e);
  }
}
