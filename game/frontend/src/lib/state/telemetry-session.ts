// Telemetry session lifecycle helpers. Extracted from match-store so they
// can be unit-tested without booting the Svelte compiler (runes).
//
// These helpers mutate `match.telemetryMatchId` and read `match.mode`
// through the passed reference. Failures are swallowed: telemetry must
// never block gameplay. If IDB is unavailable (e.g. private browsing in
// some Safari modes), the match still plays - it just doesn't get logged.
//
// The module exports `createTelemetrySession()` - a factory that closes
// over the `disabledForSession` flag. The flag flips on the FIRST IDB
// failure observed within a session and stays set until the next
// `startTelemetrySession`. Keeping it per-instance (not module-scoped)
// means tests in the same process don't bleed state through it, and a
// future multi-session future (spectator? side-by-side analysis?) gets
// the right isolation for free.

import type { EngineClient } from "../engine";
import { getTelemetryStore, type EndReason } from "../storage";
import type { MatchMode } from "./match-store.svelte";

/** Subset of MatchState the telemetry helpers read/write. The Svelte $state
 *  store satisfies this. Tests can pass a plain object. */
export interface TelemetryCarrier {
  telemetryMatchId: string | null;
}

export interface TelemetrySession {
  startTelemetrySession(
    carrier: TelemetryCarrier,
    mode: MatchMode,
    opts?: { multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null },
  ): Promise<string | null>;
  recordPly(carrier: TelemetryCarrier, eng: EngineClient): Promise<void>;
  finalizeTelemetrySession(
    carrier: TelemetryCarrier,
    eng: EngineClient,
    endReason: EndReason,
    resultByte: 0 | 1 | 2 | 3,
    logJsonOverride?: string | null,
  ): Promise<void>;
  abandonTelemetrySession(carrier: TelemetryCarrier, eng?: EngineClient, logJsonOverride?: string | null): Promise<void>;
  networkLostTelemetrySession(carrier: TelemetryCarrier, eng?: EngineClient): Promise<void>;
  abandonTelemetrySessionSync(carrier: TelemetryCarrier): void;
  networkLostTelemetrySessionSync(carrier: TelemetryCarrier): void;
  /** Test seam: clears the disabled-for-session latch without starting a new
   *  session. Production callers should never need this - startMatch resets
   *  the flag on its own. */
  reset(): void;
}

function logFail(stage: string, e: unknown): void {
  // eslint-disable-next-line no-console
  console.warn(`[telemetry] ${stage} failed:`, e);
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

export function createTelemetrySession(): TelemetrySession {
  let disabled = false;

  async function startTelemetrySession(
    carrier: TelemetryCarrier,
    mode: MatchMode,
    opts: { multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null } = {},
  ): Promise<string | null> {
    if (mode === "sandbox" || mode === "replay" || mode === "idle") return null;
    disabled = false;
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
      disabled = true;
      carrier.telemetryMatchId = null;
      return null;
    }
  }

  async function recordPly(carrier: TelemetryCarrier, eng: EngineClient): Promise<void> {
    if (!carrier.telemetryMatchId || disabled) return;
    try {
      const plyJson = await eng.latestPlyJson();
      if (!plyJson) return;
      const plyNo = extractPlyNo(plyJson);
      if (plyNo === null) {
        // Engine returned a record without a parseable ply_no - surface loudly
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
      // this, `markNetworkLost` is the only writer of `matchLogJson` - and it
      // is a one-shot transition that no-ops after the first flip, so any
      // plies applied AFTER the row turned `mid-match-network-lost` would be
      // dropped from the resume snapshot.
      try {
        const logJson = await eng.matchLogJson();
        if (logJson) await store.checkpointMatchLog(carrier.telemetryMatchId, logJson);
      } catch {
        // Engine without auto_log on, or a transient failure - appendPly
        // already succeeded, so the per-ply log is preserved. Skip silently.
      }
    } catch (e) {
      logFail("recordPly", e);
    }
  }

  async function finalizeTelemetrySession(
    carrier: TelemetryCarrier,
    eng: EngineClient,
    endReason: EndReason,
    resultByte: 0 | 1 | 2 | 3,
    logJsonOverride?: string | null,
  ): Promise<void> {
    if (!carrier.telemetryMatchId || disabled) return;
    const id = carrier.telemetryMatchId;
    carrier.telemetryMatchId = null;
    try {
      // AIvAI (Change 6) passes the background PRODUCER's log here — that is
      // the authoritative game, not the frontend view engine's replay. When
      // omitted, read the given engine's own log (HvH / HvAI / MP).
      const logJson = logJsonOverride !== undefined ? logJsonOverride : await eng.matchLogJson();
      if (!logJson) return;
      let totalPlies = 0;
      let totalWallMs = 0;
      try {
        const parsed = JSON.parse(logJson) as {
          total_plies?: number;
          total_wall_ms?: number;
          plies?: unknown[];
        };
        totalPlies = parsed.total_plies ?? (Array.isArray(parsed.plies) ? parsed.plies.length : 0);
        totalWallMs = parsed.total_wall_ms ?? 0;
      } catch (parseErr) {
        // The library reads the consolidated log directly; finalising with an
        // unparseable string would corrupt the row. Surface and bail so the
        // session stays in-progress and the user can export the raw blob via
        // the inspector instead.
        logFail("finalizeTelemetrySession.parseLog", parseErr);
        carrier.telemetryMatchId = id;
        return;
      }
      const store = getTelemetryStore();
      await store.finalizeMatch(id, logJson, endReason, resultByte, totalPlies, totalWallMs);
    } catch (e) {
      logFail("finalizeTelemetrySession", e);
    }
  }

  async function abandonTelemetrySession(
    carrier: TelemetryCarrier,
    eng?: EngineClient,
    logJsonOverride?: string | null,
  ): Promise<void> {
    if (!carrier.telemetryMatchId || disabled) return;
    const id = carrier.telemetryMatchId;
    carrier.telemetryMatchId = null;
    try {
      let partial: string | undefined;
      // AIvAI passes the producer's (post-abort, finalised) log here so the
      // abandoned row's log length equals exactly what the producer computed.
      if (logJsonOverride !== undefined) {
        partial = logJsonOverride ?? undefined;
      } else if (eng) {
        try {
          partial = (await eng.matchLogJson()) ?? undefined;
        } catch {
          // Engine in a bad state - store the abandonment marker without a log.
        }
      }
      const store = getTelemetryStore();
      await store.markAbandoned(id, partial);
    } catch (e) {
      logFail("abandonTelemetrySession", e);
    }
  }

  async function networkLostTelemetrySession(
    carrier: TelemetryCarrier,
    eng?: EngineClient,
  ): Promise<void> {
    if (!carrier.telemetryMatchId || disabled) return;
    const id = carrier.telemetryMatchId;
    carrier.telemetryMatchId = null;
    try {
      let partial: string | undefined;
      if (eng) {
        try {
          partial = (await eng.matchLogJson()) ?? undefined;
        } catch {
          // Engine in a bad state - store the marker without a log.
        }
      }
      const store = getTelemetryStore();
      await store.markNetworkLost(id, partial);
    } catch (e) {
      logFail("networkLostTelemetrySession", e);
    }
  }

  function networkLostTelemetrySessionSync(carrier: TelemetryCarrier): void {
    if (!carrier.telemetryMatchId || disabled) return;
    const id = carrier.telemetryMatchId;
    carrier.telemetryMatchId = null;
    try {
      const store = getTelemetryStore();
      void store.markNetworkLost(id).catch((e) => logFail("networkLostTelemetrySessionSync", e));
    } catch (e) {
      logFail("networkLostTelemetrySessionSync", e);
    }
  }

  function abandonTelemetrySessionSync(carrier: TelemetryCarrier): void {
    if (!carrier.telemetryMatchId || disabled) return;
    const id = carrier.telemetryMatchId;
    carrier.telemetryMatchId = null;
    try {
      const store = getTelemetryStore();
      void store.markAbandoned(id).catch((e) => logFail("abandonTelemetrySessionSync", e));
    } catch (e) {
      logFail("abandonTelemetrySessionSync", e);
    }
  }

  function reset(): void {
    disabled = false;
  }

  return {
    startTelemetrySession,
    recordPly,
    finalizeTelemetrySession,
    abandonTelemetrySession,
    networkLostTelemetrySession,
    abandonTelemetrySessionSync,
    networkLostTelemetrySessionSync,
    reset,
  };
}
