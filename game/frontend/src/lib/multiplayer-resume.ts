// Regex helpers for the multiplayer resume handshake. The engine's MatchLog
// JSON contains u64 Zobrist hashes that exceed JS's safe-integer range, so we
// cannot round-trip the log through JSON.parse without lossy bigint coercion.
// `extractStartZobrist` / `extractPostZobristForPly` therefore extract the raw
// decimal digit-string directly from the JSON text and leave string comparison
// to the caller.
//
// Pinned to MatchLog JSON v1 (see core_engine/src/session.rs). If the
// `match-log` serde representation changes — renamed fields, restructured
// nesting, base64 zobrist encoding — update these regexes AND the pinning
// tests in `multiplayer-resume.test.ts`.

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

/** Decide whether a persisted MatchLog represents a mid-draft state without
 *  booting the engine. The draft phase consumes the first 12 actions (six per
 *  side, two picks each), so any log with fewer than 12 plies is still in
 *  Phase::Draft. Used by the lobby's host-side Rejoin to route to /draft/ vs
 *  /match/ without spinning up WASM just for the phase check.
 *
 *  Returns false on parse failure — the lobby treats that as "route to /match/",
 *  which is the safe fallback for any malformed log. */
export function logIsMidDraftCheap(matchLogJson: string): boolean {
  try {
    const log = JSON.parse(matchLogJson) as { plies?: unknown[] };
    return Array.isArray(log.plies) && log.plies.length < 12;
  } catch {
    return false;
  }
}
