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
 *  booting the engine. Used by the lobby's Rejoin flow to route to /draft/ vs
 *  /match/ without spinning up WASM just for the phase check.
 *
 *  Two signals combined:
 *
 *   1. `start_fen` phase field (3rd space-separated token, single char
 *      `M`|`S`|`D`). This tells us how the engine was booted:
 *        - `D` → `new_with_draft` (drafted mode).
 *        - `M`|`S` → `new_with_loadouts` (preMade). `/draft/` is skipped by
 *          setup on both peers; the match log's first ply is a game ply,
 *          not a `DraftTurn`.
 *
 *   2. Ply kinds. During drafted mode's draft phase, plies are
 *      `action.kind === "DraftTurn"`; game plies are
 *      `"Move" | "Skill" | "EndPhase" | "EndTurn" | "BodyguardChoice"`.
 *
 *  Rule:
 *    - preMade (start_fen phase !== 'D') → **never** mid-draft. Return false
 *      even with zero plies (the game just hasn't opened yet).
 *    - drafted (start_fen phase === 'D') → mid-draft iff every ply so far is
 *      a `DraftTurn` (or there are none yet). Any non-DraftTurn ply flips it.
 *
 *  Historical note: the older heuristic was `plies.length < 12`, which was
 *  wrong for preMade — a preMade match with < 12 game plies had length < 12
 *  and got routed to /draft/. /draft/'s stale-entry guard bounced back to
 *  /match/, and the intermediate teardown killed the fresh rejoin WS.
 *  The intermediate fix (kind-only) also broke on empty preMade logs.
 *
 *  Returns false on parse failure — the lobby treats that as "route to
 *  /match/", the safe fallback for any malformed log. */
export function logIsMidDraftCheap(matchLogJson: string): boolean {
  try {
    const log = JSON.parse(matchLogJson) as {
      start_fen?: string;
      plies?: Array<{ action?: { kind?: string } }>;
    };
    if (typeof log.start_fen !== "string") return false;
    // FEN grammar: `<board> <to_move> <phase> …` — phase is the 3rd token.
    const phase = log.start_fen.split(" ")[2];
    if (phase !== "D") return false; // preMade or already past draft.
    if (!Array.isArray(log.plies)) return false;
    for (const ply of log.plies) {
      const kind = ply.action?.kind;
      if (typeof kind === "string" && kind !== "DraftTurn") {
        return false;
      }
    }
    return true;
  } catch {
    return false;
  }
}
