// Resume-handshake helpers. The current MP wire (`multiplayer-protocol-v2.ts`)
// sends zobrists as the live `PositionView.zobrist` bigint, so we no longer
// need to re-parse them out of persisted MatchLog JSON.
//
// What remains: rebuilding an engine Snapshot JSON from a MatchLog (for the
// host's Rejoin flow), and a cheap mid-draft check used by the lobby to
// route Rejoin to /draft/ vs /match/.

/** Rebuild an engine Snapshot JSON ({ start_fen, actions, config }) from a
 *  persisted MatchLog. Used by the host's Rejoin flow to re-enter /match/
 *  at the state it had when it left — the engine doesn't expose a replay-from-
 *  log API, but `restoreFromSnapshot` accepts the same shape we build here,
 *  and replays every action through `try_apply` on load.
 *
 *  Returns null if the log can't be parsed or required fields are missing.
 *  Walks the plies array via JSON.parse — Zobrists in the log overflow Number
 *  precision but we don't read them here; `action.raw` is u32 and survives
 *  the round-trip. */
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
