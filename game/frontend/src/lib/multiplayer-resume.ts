// Regex helpers for the multiplayer resume handshake. The engine's MatchLog
// JSON contains u64 Zobrist hashes that exceed JS's safe-integer range, so we
// cannot round-trip the log through JSON.parse without lossy bigint coercion.
// Both helpers therefore extract the raw decimal digit-string directly from
// the JSON text and leave string comparison to the caller.
//
// Pinned to MatchLog JSON v1 (see core_engine/src/session.rs). If the
// `match-log` serde representation changes — renamed fields, restructured
// nesting, base64 zobrist encoding — update these regexes AND the pinning
// tests in `multiplayer-resume.test.ts`.

import { getEngine } from "./engine";

// MatchLog ply_no is 1-indexed (core_engine/src/session.rs:360); plyCount=0 → start_zobrist.
export function extractPostZobristForPly(
  matchLogJson: string,
  n: number,
): string | null {
  const plyRe = new RegExp(`"ply_no"\\s*:\\s*${n}\\b`);
  const m = plyRe.exec(matchLogJson);
  if (!m) return null;
  const tail = matchLogJson.slice(m.index);
  const zRe = /"post_zobrist"\s*:\s*(\d+)/;
  const z = zRe.exec(tail);
  return z ? z[1] : null;
}

export function extractStartZobrist(matchLogJson: string): string | null {
  const m = /"start_zobrist"\s*:\s*(\d+)/.exec(matchLogJson);
  return m ? m[1] : null;
}

/** Compute the (plyCount, zobrist) pair to send in a resume-request from a
 *  persisted MatchLog. Returns `{plyCount: 0, zobrist: "0"}` when the log
 *  shape is unexpected so the host can still respond (with start_zobrist
 *  comparison) or reject cleanly. */
export function extractResumeStateFromLog(
  logJson: string,
): { plyCount: number; zobrist: string } {
  const tpMatch = /"total_plies"\s*:\s*(\d+)/.exec(logJson);
  const plyCount = tpMatch ? parseInt(tpMatch[1], 10) : 0;
  if (plyCount === 0) {
    const sz = extractStartZobrist(logJson);
    return { plyCount: 0, zobrist: sz ?? "0" };
  }
  const pz = extractPostZobristForPly(logJson, plyCount);
  return { plyCount, zobrist: pz ?? "0" };
}

/** Rebuild an engine Snapshot JSON ({ start_fen, actions, config }) from a
 *  persisted MatchLog. Used by the host's Rejoin flow to re-enter /match/
 *  at the state it had when it left — the engine doesn't expose a replay-from-
 *  log API, but `restoreFromSnapshot` accepts the same shape we build here,
 *  and replays every action through `try_apply` on load.
 *
 *  Returns null if the log can't be parsed or required fields are missing.
 *  Walks the plies array via JSON.parse — the Zobrists in the log overflow
 *  Number precision but `action.raw` is u32 and survives the round-trip. */
export function snapshotJsonFromMatchLog(matchLogJson: string): string | null {
  try {
    const log = JSON.parse(matchLogJson) as {
      start_fen?: string;
      config?: unknown;
      plies?: Array<{ action?: { raw?: number } }>;
    };
    if (typeof log.start_fen !== "string") return null;
    if (log.config === undefined) return null;
    const actions: number[] = [];
    for (const ply of log.plies ?? []) {
      const raw = ply.action?.raw;
      if (typeof raw !== "number" || !Number.isInteger(raw) || raw < 0) {
        return null;
      }
      actions.push(raw);
    }
    return JSON.stringify({
      start_fen: log.start_fen,
      actions,
      config: log.config,
    });
  } catch {
    return null;
  }
}

/** Restore an engine from the given MatchLog and ask whether the resulting
 *  position is still in `Phase::Draft`. Used by the lobby's rejoin flow to
 *  decide whether to route resume traffic to /draft/ (mid-draft) or /match/
 *  (post-draft / play-phase). Returns false if the log can't be parsed —
 *  the caller treats that as "route to /match/" which is the safe fallback.
 *
 *  Cost: one WASM engine boot + a snapshot replay. ~50ms when WASM is
 *  cached (typical for a user who just left a match). The lobby fires
 *  this lazily, only when the user clicks Rejoin, so it doesn't add to
 *  page-load time. */
export async function logIsMidDraft(matchLogJson: string): Promise<boolean> {
  const snap = snapshotJsonFromMatchLog(matchLogJson);
  if (!snap) return false;
  try {
    const e = await getEngine();
    await e.restoreFromSnapshot(snap);
    const view = await e.positionView();
    // Phase::Draft = 2 (see core_engine/src/wrapper_api.rs:84 mapping).
    return view.currentPhase === 2;
  } catch {
    return false;
  }
}
