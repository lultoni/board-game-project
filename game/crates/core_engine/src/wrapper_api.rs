//! Flat target-agnostic engine façade for `wasm_wrapper` and `tauri_wrapper`.
//!
//! Both wrappers translate from their respective boundary (wasm-bindgen /
//! tauri::command) into calls on this surface. Keeping the wrappers thin
//! ensures they stay in lockstep - drift means a divergent UI.
//!
//! # Conventions
//!
//! - **Hot path** (called per frame / per AI step) returns flat primitives,
//!   `#[repr(C)]` structs, or `&[u16; 64]` slices - never JSON.
//! - **Cold path** (save/load, log export) returns owned `String` JSON.
//! - `Match` lifetime is managed by the wrapper (one per Engine on wasm; a
//!   handle table on Tauri). This module is stateless beyond the `&mut Match`
//!   it receives.

use crate::game_logic::action::Action;
use crate::game_logic::draft::{draft_state, DraftState};
use crate::game_logic::generator;
use crate::game_logic::skills::{validate_loadout, DraftError, SideLoadout};
use crate::search::alpha_beta::SearchResult;
use crate::session::{ApplyError, AiError, Config, Match, Snapshot, SnapshotError};
use crate::state::position::{GameResult, Phase, Player};
use crate::telemetry::{MatchResult, SearchMeta};

/// Compact, flat view of the current position. Designed to cross the
/// wasm-bindgen / Tauri boundary cheaply. The renderer combines this with
/// `position_mailbox` to draw the board.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PositionView {
    /// `[p1_pieces, p2_pieces, kings, champions, guards]` - five u64 bitboards.
    pub bitboards:         [u64; 5],
    /// 0 = P1, 1 = P2.
    pub to_move:           u8,
    /// 0 = Move, 1 = Skill.
    pub current_phase:     u8,
    pub actions_remaining: u8,
    pub round_number:      u16,
    pub p1_money:          u16,
    pub p2_money:          u16,
    /// `modifier_bits::FOCUS | CHARGE` packed.
    pub pending_modifiers: u8,
    /// 0 = ongoing, 1 = P1Wins, 2 = P2Wins.
    pub game_result:       u8,
    pub zobrist:           u64,
    /// Bitboard of squares whose piece has already used its Move action this
    /// Move-Phase (final square, not src). Zero outside the Move-Phase. Lets the
    /// UI grey out already-moved pieces on LOAD (resume/snapshot/preview), not
    /// just incrementally as they move this session.
    pub moved_this_phase:  u64,
}

/// Per-step delta returned by `try_apply` / `step_ai`. Small on purpose -
/// the worker posts this back to the UI thread on every move.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct StepResult {
    /// The Action that was applied (raw `Action(u32)` bits). 0 if no action
    /// was applied (e.g. `step_ai` on a terminal position).
    pub applied_action: u32,
    /// AI search score (centipawn-style). Meaningless for human plies.
    pub score:          i32,
    /// AI search depth completed. 0 for human plies.
    pub depth:          u8,
    /// AI search nodes visited. 0 for human plies.
    pub nodes:          u64,
    /// Wall-clock ms the AI thought. 0 for human plies (caller fills in).
    pub thought_ms:     u32,
    /// 0 = ongoing, 1 = P1Wins, 2 = P2Wins.
    pub game_result:    u8,
}

// --- Position reads (hot path) ---------------------------------------------

/// Snapshot the current position into a flat `PositionView`. Cheap (no
/// allocation). Call once per frame.
#[inline]
pub fn position_view(m: &Match) -> PositionView {
    let p = m.position();
    PositionView {
        bitboards: [
            p.p1_pieces.0,
            p.p2_pieces.0,
            p.kings.0,
            p.champions.0,
            p.guards.0,
        ],
        to_move:           match p.to_move { Player::P1 => 0, Player::P2 => 1 },
        current_phase:     match p.current_phase {
            Phase::Move  => 0,
            Phase::Skill => 1,
            Phase::Draft => 2,
        },
        actions_remaining: p.actions_remaining,
        round_number:      p.round_number,
        p1_money:          p.p1_money,
        p2_money:          p.p2_money,
        pending_modifiers: p.pending_modifiers,
        game_result:       encode_game_result(p.game_result),
        zobrist:           p.zobrist,
        moved_this_phase:  p.moved_this_phase.0,
    }
}

/// Borrow the 64-entry packed mailbox. Each `u16` packs HP/armor/combo/skill1/
/// skill2 - see `MailboxEntry`. Wrappers expose this as a zero-copy typed-array
/// view (wasm) or raw bytes (Tauri).
#[inline]
pub fn position_mailbox(m: &Match) -> &[u16; 64] {
    // Safe: MailboxEntry is `#[repr(transparent)]` over u16, so the array
    // shares layout with `[u16; 64]`.
    let mb = &m.position().mailbox;
    unsafe { &*(mb as *const [crate::state::MailboxEntry; 64] as *const [u16; 64]) }
}

/// Debug-only FEN string. Not on the hot path.
#[inline]
pub fn position_fen(m: &Match) -> String {
    m.position().to_fen()
}

/// Snapshot the engine's `pending_bodyguard` slot. `None` in the common
/// case; `Some(...)` only between the attacker's tentative Move-Attack and
/// the defender's `BodyguardChoice`. The renderer uses this to drive the
/// chooser overlay; clients drive the engine via `BodyguardChoice` actions
/// - there is no side-channel.
#[inline]
pub fn pending_bodyguard(m: &Match) -> Option<crate::state::position::PendingBodyguard> {
    m.position().pending_bodyguard
}

// --- Play (hot path) -------------------------------------------------------

/// Fill `buf` with the legal `Action` bits for the current position. Clears
/// the buffer first; the wrapper owns the allocation so it can be reused
/// across calls.
#[inline]
pub fn legal_actions_into(m: &Match, buf: &mut Vec<u32>) {
    buf.clear();
    if m.game_result().is_some() { return; }
    for a in generator::generate(m.position()) {
        buf.push(a.0);
    }
}

/// Apply a human action. Wrapper supplies `applied_at_unix_ms` (the wall
/// clock - `Date.now()` on wasm, `SystemTime` on Tauri). Returns a
/// `StepResult` with `applied_action`, `game_result`, and zeros for AI
/// fields.
pub fn try_apply(
    m: &mut Match,
    raw_action: u32,
    applied_at_unix_ms: u64,
) -> Result<StepResult, ApplyError> {
    try_apply_with_thought(m, raw_action, /*thought_ms=*/ 0, applied_at_unix_ms)
}

/// Like `try_apply` but records `thought_ms` (human decision time) into the
/// ply's telemetry. The wrapper computes `thought_ms` from a start-of-turn
/// timestamp it tracks; the engine has no clock of its own. `StepResult`
/// echoes `thought_ms` back so the frontend can display it immediately.
pub fn try_apply_with_thought(
    m: &mut Match,
    raw_action: u32,
    thought_ms: u32,
    applied_at_unix_ms: u64,
) -> Result<StepResult, ApplyError> {
    let a = Action(raw_action);
    m.try_apply_timed(a, thought_ms, applied_at_unix_ms, /*ai=*/ None)?;
    Ok(StepResult {
        applied_action: raw_action,
        score:          0,
        depth:          0,
        nodes:          0,
        thought_ms,
        game_result:    encode_game_result(m.game_result()),
    })
}

/// Run the AI for the side-to-move under its configured budget, apply the
/// chosen action, and return a flat `StepResult`. Wrapper supplies
/// `applied_at_unix_ms`. Wall time is measured here via `crate::time::now_ms`
/// (which on wasm imports `engine_now_ms` from the host).
pub fn step_ai(m: &mut Match, applied_at_unix_ms: u64) -> Result<StepResult, AiError> {
    step_ai_with_cb(m, applied_at_unix_ms, None)
}

/// Like `step_ai` but calls `on_depth(depth, score)` after each completed
/// iterative-deepening iteration so callers can stream progress.
pub fn step_ai_with_cb(
    m: &mut Match,
    applied_at_unix_ms: u64,
    on_depth: Option<&dyn Fn(u8, i32)>,
) -> Result<StepResult, AiError> {
    let t0 = crate::time::now_ms();
    let r: SearchResult = m.request_ai_move_with_cb(on_depth)?;
    let thought_ms = crate::time::now_ms()
        .saturating_sub(t0)
        .min(u32::MAX as u64) as u32;

    let applied = if let Some(a) = r.best {
        let meta = SearchMeta::from_search(r.depth, r.nodes, r.score);
        // alpha-beta returning an illegal action would be a bug in search,
        // not a runtime error - propagate as None so the UI can surface it.
        match m.try_apply_timed(a, thought_ms, applied_at_unix_ms, Some(meta)) {
            Ok(()) => a.0,
            Err(_) => 0,
        }
    } else {
        0
    };

    Ok(StepResult {
        applied_action: applied,
        score:          r.score,
        depth:          r.depth,
        nodes:          r.nodes,
        thought_ms,
        game_result:    encode_game_result(m.game_result()),
    })
}

/// Inspector variant: runs the AI search for whoever is to move regardless
/// of seat kind (Human vs Human positions included), returning the best
/// action without applying it. The seat-restricted variant was removed as
/// dead surface - the match route uses `step_ai` (which applies atomically),
/// and the inspector wants the unrestricted form.
pub fn request_ai_move_forced(m: &mut Match) -> Result<StepResult, AiError> {
    let t0 = crate::time::now_ms();
    let r: SearchResult = m.request_ai_move_forced()?;
    let thought_ms = crate::time::now_ms()
        .saturating_sub(t0)
        .min(u32::MAX as u64) as u32;

    Ok(StepResult {
        applied_action: r.best.map(|a| a.0).unwrap_or(0),
        score:          r.score,
        depth:          r.depth,
        nodes:          r.nodes,
        thought_ms,
        game_result:    encode_game_result(m.game_result()),
    })
}

/// Iterative-deepening helper for the inspector: runs ID up to `max_depth`
/// with no time bound. Caller drives the loop, polling cancellation
/// between calls. The shared transposition table makes successive depths
/// progressively cheaper.
pub fn request_ai_move_at_depth(m: &mut Match, max_depth: u8) -> Result<StepResult, AiError> {
    let t0 = crate::time::now_ms();
    let r: SearchResult = m.request_ai_move_at_depth(max_depth)?;
    let thought_ms = crate::time::now_ms()
        .saturating_sub(t0)
        .min(u32::MAX as u64) as u32;

    Ok(StepResult {
        applied_action: r.best.map(|a| a.0).unwrap_or(0),
        score:          r.score,
        depth:          r.depth,
        nodes:          r.nodes,
        thought_ms,
        game_result:    encode_game_result(m.game_result()),
    })
}

// --- Telemetry / persistence (cold path) -----------------------------------

/// JSON of the `MatchLog` if `auto_log` is on, else `None`.
#[inline]
pub fn match_log_json(m: &Match) -> Option<String> {
    m.match_log().map(crate::telemetry::to_json)
}

/// JSON of the most recently recorded `PlyRecord`. `None` when `auto_log` is
/// off or no plies have been recorded yet. Used by the frontend telemetry
/// persistence layer to write per-ply records incrementally without
/// re-serialising the entire match log on every move (avoids O(n²) work).
#[inline]
pub fn latest_ply_json(m: &Match) -> Option<String> {
    m.match_log()
        .and_then(|log| log.plies.last())
        .map(crate::telemetry::to_json)
}

/// Stamp the final result + close out the log. `result_byte`:
/// 0 = P1Win, 1 = P2Win, 2 = Draw, 3 = Aborted. No-op if `auto_log` is off.
pub fn finalise_log(m: &mut Match, now_unix_ms: u64, result_byte: u8) {
    let result = match result_byte {
        0 => MatchResult::P1Win,
        1 => MatchResult::P2Win,
        2 => MatchResult::Draw,
        _ => MatchResult::Aborted,
    };
    m.finalise_log(now_unix_ms, result);
}

/// Part B (Change 5): run a time-bounded background search on the current
/// position and patch the last recorded ply's `background_eval`. Called off
/// the apply hot path (the Tauri layer runs it on a worker thread after
/// `try_apply` returns). `budget_ms` is a wall-clock budget — iterative
/// deepening climbs to whatever depth the position allows in that window
/// (0 ⇒ shallow fixed-depth fallback). No-op when not logging / no ply /
/// game over / draft.
pub fn annotate_last_ply_background_eval(m: &mut Match, budget_ms: u64) {
    m.annotate_last_ply_with_background_eval(budget_ms);
}

/// Serialise a save-game snapshot.
#[inline]
pub fn snapshot_json(m: &Match) -> String {
    crate::telemetry::to_json(&m.to_snapshot())
}

/// Number of plies recorded in the match log so far, or 0 when the match
/// isn't logging. Used by the background AIvAI producer to publish a
/// "known ply count" ceiling the frontend log-player advances toward.
#[inline]
pub fn log_ply_count(m: &Match) -> usize {
    m.match_log().map_or(0, |l| l.plies.len())
}

/// Map the match's current terminal state to the `result_byte` convention
/// `finalise_log` expects (0 = P1Win, 1 = P2Win, 2 = Draw, 3 = Aborted).
/// A live (non-terminal) position maps to `Aborted` — the background
/// producer uses this when it stops on a leave/abort or a no-move wedge
/// rather than a natural win.
#[inline]
pub fn finalise_result_byte(m: &Match) -> u8 {
    match m.game_result() {
        Some(GameResult::P1Wins) => 0,
        Some(GameResult::P2Wins) => 1,
        None                     => 3, // no draws in this ruleset; live → aborted
    }
}

/// Reconstruct a `Match` from a snapshot JSON. Wrapper supplies the wall
/// clock (engine has no `SystemTime` on wasm).
pub fn from_snapshot_json(s: &str, now_unix_ms: u64) -> Result<Match, SnapshotErrorOrParse> {
    let snap: Snapshot = crate::telemetry::from_json(s)
        .map_err(SnapshotErrorOrParse::Parse)?;
    Match::from_snapshot_with_clock(snap, now_unix_ms)
        .map_err(SnapshotErrorOrParse::Snapshot)
}

/// Error union for `from_snapshot_json` - wrappers flatten this to a string.
#[derive(Debug)]
pub enum SnapshotErrorOrParse {
    Parse(serde_json::Error),
    Snapshot(SnapshotError),
}

// --- Draft constructors / state (L8) --------------------------------------

/// Build a fresh `Match` in `Phase::Draft`. The wrapper owns the wall-clock.
#[inline]
pub fn new_match_with_draft(config: Config, now_unix_ms: u64) -> Match {
    Match::new_with_draft(config, now_unix_ms)
}

/// Build a fresh `Match` that bypasses draft, with loadouts already applied.
/// Both loadouts are validated; returns the first `DraftError` encountered.
pub fn new_match_with_loadouts(
    config: Config,
    p1: &SideLoadout,
    p2: &SideLoadout,
    now_unix_ms: u64,
) -> Result<Match, DraftError> {
    validate_loadout(p1)?;
    validate_loadout(p2)?;
    Ok(Match::new_with_loadouts(config, p1, p2, now_unix_ms))
}

/// Parse a JSON-encoded `SideLoadout` (a 6-tuple array of `[skill1, skill2]`
/// pairs) into the engine's typed representation. Wrappers expose this so
/// the frontend can send `[[6,7],[1,9],...]` over the bridge.
pub fn parse_side_loadout_json(s: &str) -> Result<SideLoadout, serde_json::Error> {
    let arr: [[u8; 2]; 6] = crate::telemetry::from_json(s)?;
    Ok([
        (arr[0][0], arr[0][1]),
        (arr[1][0], arr[1][1]),
        (arr[2][0], arr[2][1]),
        (arr[3][0], arr[3][1]),
        (arr[4][0], arr[4][1]),
        (arr[5][0], arr[5][1]),
    ])
}

/// Snapshot of the draft (turn number, side-to-pick, used-slots bitmap).
/// Returned as a flat, owned struct so wrappers can copy it across the
/// boundary directly.
#[inline]
pub fn current_draft_state(m: &Match) -> DraftState {
    draft_state(m.position())
}

/// Dynamic eval breakdown of the current position, using the Match's installed
/// evaluator. `per_piece` selects the per-piece decomposition (for the square
/// hover card) vs aggregate-only (for the eval bar / term list).
pub fn eval_report(m: &Match, per_piece: bool) -> crate::search::evaluator::EvalReport {
    eval_report_with(m, m.evaluator_for_to_move(), per_piece)
}

/// Like [`eval_report`] but scores with a caller-supplied evaluator instead of
/// the Match's installed one — the UI-eval path uses this to render the panel
/// under a user-chosen evaluator (`settings.uiEvaluator`) without disturbing the
/// AI seats' evaluator on the Match.
pub fn eval_report_with(
    m: &Match,
    evaluator: &(dyn crate::search::evaluator::Evaluator + Send),
    per_piece: bool,
) -> crate::search::evaluator::EvalReport {
    use crate::search::evaluator::BreakdownDetail;
    let detail = if per_piece { BreakdownDetail::PerPiece } else { BreakdownDetail::Aggregate };
    evaluator.evaluate_report(m.position(), detail)
}

/// Whether the AI (whose seat is `ai_seat`: 0 = P1, 1 = P2) should accept
/// a draw offer from the human. Accepts when the AI is not clearly winning —
/// defined as its seat-relative score ≤ 100 centipawns.
///
/// The decision uses the AI's *searched* evaluation at its configured budget
/// (the same search it plays with), NOT a 1-ply static heuristic: a static
/// read badly misjudges tactical positions (a hanging piece, a forced mate),
/// so the AI would agree to draws it's actually winning or refuse ones it's
/// losing. `request_ai_move_forced` searches from the current position for
/// whoever is to move (typically the human, who just offered), so the returned
/// `score` is from the side-to-move's perspective; we rotate it to P1-relative
/// and then to the AI's seat.
pub fn evaluate_draw_offer(m: &mut Match, ai_seat: u8) -> bool {
    // Side-to-move-relative score from a real search. Fall back to the static
    // eval only if the search can't run (e.g. game already over / draft).
    let stm_score = match m.request_ai_move_forced() {
        Ok(r) => r.score,
        Err(_) => crate::search::evaluator::evaluate(m.position()),
    };
    // Rotate score to P1-relative (positive = P1 advantage), then to AI-relative.
    let p1_relative = if m.position().to_move == crate::state::position::Player::P1 {
        stm_score
    } else {
        -stm_score
    };
    let ai_relative = if ai_seat == 0 { p1_relative } else { -p1_relative };
    ai_relative <= 100
}

impl core::fmt::Display for SnapshotErrorOrParse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e)    => write!(f, "snapshot parse error: {e}"),
            Self::Snapshot(e) => write!(f, "snapshot restore error: {e:?}"),
        }
    }
}

// --- Helpers ---------------------------------------------------------------

#[inline]
fn encode_game_result(g: Option<GameResult>) -> u8 {
    match g {
        None                       => 0,
        Some(GameResult::P1Wins)   => 1,
        Some(GameResult::P2Wins)   => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Config;

    fn fresh_match() -> Match {
        Match::new(Config::local_aivai())
    }

    fn fresh_logging_match() -> Match {
        let mut cfg = Config::local_aivai();
        cfg.auto_log = true;
        Match::new_with_clock(cfg, 1_700_000_000_000)
    }

    #[test]
    fn log_ply_count_tracks_applied_plies() {
        let mut m = fresh_logging_match();
        assert_eq!(log_ply_count(&m), 0, "fresh log has no plies");
        // Apply the AI's first move; the log grows by one.
        step_ai(&mut m, 1_700_000_000_001).unwrap();
        assert_eq!(log_ply_count(&m), 1, "one ply recorded after a step");
    }

    #[test]
    fn log_ply_count_zero_when_not_logging() {
        let mut m = fresh_match(); // auto_log off
        step_ai(&mut m, 1).unwrap();
        assert_eq!(log_ply_count(&m), 0, "no log → ply count 0");
    }

    #[test]
    fn finalise_result_byte_live_position_is_aborted() {
        let m = fresh_logging_match();
        // A fresh, ongoing game has no result → maps to Aborted (3).
        assert_eq!(finalise_result_byte(&m), 3);
    }

    #[test]
    fn evaluate_draw_offer_runs_without_panic() {
        // The decision path must run a search and return cleanly for both seats
        // from a fresh position (search is make/unmake balanced, so the second
        // call sees the same position).
        let mut m = fresh_match();
        let _ = evaluate_draw_offer(&mut m, 0);
        let _ = evaluate_draw_offer(&mut m, 1);
    }

    #[test]
    fn evaluate_draw_offer_uses_searched_eval_not_static() {
        // Regression: the old code decided from a 1-ply static heuristic; the
        // fix runs the AI's actual search. From the start position the searched
        // eval sees the side-to-move tempo advantage (~86cp here) that the
        // static eval (~30cp) understates — so the two paths disagree, proving
        // the search is being used. We assert the two scores differ rather than
        // pinning exact magnitudes (which would be brittle to eval tuning).
        let mut m = fresh_match();
        let searched_stm = m.request_ai_move_forced().unwrap().score;
        let static_p1 = crate::search::evaluator::evaluate(m.position());
        assert_ne!(
            searched_stm, static_p1,
            "draw decision must use the searched eval, which differs from the static one",
        );
    }

    #[test]
    fn evaluate_draw_offer_is_seat_symmetric_in_sign() {
        // The two seats must see opposite standings from the same position: if
        // P1 declines (thinks it's winning), P2 accepts, and vice-versa. This
        // pins the seat sign-rotation regardless of the eval magnitude.
        let mut m = fresh_match();
        let p1_accepts = evaluate_draw_offer(&mut m, 0);
        let p2_accepts = evaluate_draw_offer(&mut m, 1);
        // Not both declining: at most one side can be "clearly winning".
        assert!(
            p1_accepts || p2_accepts,
            "at least one seat must accept — both sides can't be clearly winning",
        );
    }

    #[test]
    fn position_view_matches_position_fields() {
        let m = fresh_match();
        let v = position_view(&m);
        let p = m.position();
        assert_eq!(v.bitboards[0], p.p1_pieces.0);
        assert_eq!(v.bitboards[1], p.p2_pieces.0);
        assert_eq!(v.bitboards[2], p.kings.0);
        assert_eq!(v.bitboards[3], p.champions.0);
        assert_eq!(v.bitboards[4], p.guards.0);
        assert_eq!(v.to_move, 0);
        assert_eq!(v.current_phase, 0); // Move
        assert_eq!(v.actions_remaining, p.actions_remaining);
        assert_eq!(v.round_number, p.round_number);
        assert_eq!(v.p1_money, p.p1_money);
        assert_eq!(v.p2_money, p.p2_money);
        assert_eq!(v.game_result, 0); // ongoing
        assert_eq!(v.zobrist, p.zobrist);
    }

    #[test]
    fn position_mailbox_aliases_engine_mailbox() {
        let m = fresh_match();
        let view = position_mailbox(&m);
        let truth = &m.position().mailbox;
        for sq in 0..64 {
            assert_eq!(view[sq], truth[sq].0,
                       "mailbox view diverges at sq {sq}");
        }
    }

    #[test]
    fn legal_actions_into_reuses_buffer_capacity() {
        let m = fresh_match();
        let mut buf: Vec<u32> = Vec::with_capacity(256);
        let cap0 = buf.capacity();
        legal_actions_into(&m, &mut buf);
        assert!(!buf.is_empty(), "Stack-M start has legal moves");
        assert!(buf.capacity() >= cap0, "capacity must not shrink");
        // All bits decode to legal actions.
        let truth = generator::generate(m.position());
        let truth_bits: Vec<u32> = truth.iter().map(|a| a.0).collect();
        assert_eq!(buf, truth_bits);
    }

    #[test]
    fn legal_actions_into_clears_prior_contents() {
        let m = fresh_match();
        let mut buf: Vec<u32> = vec![0xDEAD_BEEF, 0xCAFE_F00D, 42];
        legal_actions_into(&m, &mut buf);
        assert!(!buf.contains(&0xDEAD_BEEF));
        assert!(!buf.contains(&0xCAFE_F00D));
    }

    #[test]
    fn try_apply_records_step_result_and_terminal() {
        let mut m = fresh_match();
        let mut buf: Vec<u32> = Vec::new();
        legal_actions_into(&m, &mut buf);
        let raw = buf[0];
        let r = try_apply(&mut m, raw, /*now_unix_ms=*/ 1_700_000_000_000).unwrap();
        assert_eq!(r.applied_action, raw);
        assert_eq!(r.depth, 0);
        assert_eq!(r.nodes, 0);
        assert_eq!(r.thought_ms, 0);
        assert_eq!(r.game_result, 0); // ongoing
    }

    #[test]
    fn try_apply_with_thought_records_thought_ms() {
        // With auto_log on, the recorded ply must carry the caller-supplied
        // thought_ms (human decision time), not the hardcoded 0 that plain
        // `try_apply` uses.
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        let mut buf: Vec<u32> = Vec::new();
        legal_actions_into(&m, &mut buf);
        let raw = buf[0];
        let r = try_apply_with_thought(&mut m, raw, /*thought_ms=*/ 4200, 1_700_000_000_000).unwrap();
        assert_eq!(r.thought_ms, 4200, "StepResult echoes thought_ms");
        let ply = m.match_log().unwrap().plies.last().unwrap();
        assert_eq!(ply.thought_ms, 4200, "PlyRecord captures human think-time");
        assert!(ply.ai.is_none(), "human ply has no SearchMeta");
    }

    #[test]
    fn try_apply_rejects_illegal_action() {
        let mut m = fresh_match();
        // Action(0) is `default()` - unlikely to be legal at the start.
        let r = try_apply(&mut m, 0, 0);
        assert!(matches!(r, Err(ApplyError::IllegalAction)));
    }

    #[test]
    fn step_ai_applies_an_action_and_advances_state() {
        let mut m = fresh_match();
        let z0 = m.position().zobrist;
        let r = step_ai(&mut m, /*now_unix_ms=*/ 1_700_000_000_000).unwrap();
        assert!(r.applied_action != 0, "AI must produce a move on a fresh board");
        assert_ne!(m.position().zobrist, z0, "position must advance");
    }

    #[test]
    fn position_view_round_trip_via_bitboards_and_mailbox() {
        // Drive a few plies, then verify position_view + position_mailbox
        // together still reflect the engine state byte-for-byte.
        let mut m = fresh_match();
        for _ in 0..5 {
            if m.game_result().is_some() { break; }
            step_ai(&mut m, 0).unwrap();
        }
        let v = position_view(&m);
        let mb = position_mailbox(&m);
        let p = m.position();
        assert_eq!(v.bitboards[0], p.p1_pieces.0);
        assert_eq!(v.bitboards[4], p.guards.0);
        assert_eq!(v.zobrist, p.zobrist);
        // moved_this_phase must mirror the engine bitboard so the UI can grey
        // out already-moved pieces on load (P2-E).
        assert_eq!(v.moved_this_phase, p.moved_this_phase.0);
        for sq in 0..64 {
            assert_eq!(mb[sq], p.mailbox[sq].0);
        }
    }

    #[test]
    fn snapshot_json_round_trips() {
        let mut m = fresh_match();
        step_ai(&mut m, 0).unwrap();
        step_ai(&mut m, 0).unwrap();
        let s = snapshot_json(&m);
        let m2 = from_snapshot_json(&s, 0).expect("snapshot must restore");
        assert_eq!(m.position().zobrist, m2.position().zobrist);
    }

    #[test]
    fn match_log_json_returns_none_when_logging_off() {
        let m = fresh_match();
        assert!(match_log_json(&m).is_none());
    }

    #[test]
    fn match_log_json_returns_some_when_logging_on() {
        let mut cfg = Config::local_aivai();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        step_ai(&mut m, 1_700_000_000_000).unwrap();
        let log = match_log_json(&m).expect("auto_log on → log present");
        assert!(log.contains("plies"));
    }

    #[test]
    fn latest_ply_json_none_when_logging_off() {
        let m = fresh_match();
        assert!(latest_ply_json(&m).is_none());
    }

    #[test]
    fn latest_ply_json_none_before_first_ply() {
        let mut cfg = Config::local_aivai();
        cfg.auto_log = true;
        let m = Match::new(cfg);
        assert!(latest_ply_json(&m).is_none());
    }

    #[test]
    fn latest_ply_json_returns_last_ply_only() {
        let mut cfg = Config::local_aivai();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        step_ai(&mut m, 1_700_000_000_000).unwrap();
        step_ai(&mut m, 1_700_000_000_500).unwrap();
        let first = latest_ply_json(&m).expect("two plies recorded");
        // Confirm it's a single PlyRecord (not the whole log) by parsing back.
        let parsed: crate::telemetry::PlyRecord =
            crate::telemetry::from_json(&first).expect("single PlyRecord");
        assert_eq!(parsed.ply_no, m.match_log().unwrap().plies.last().unwrap().ply_no);
        assert_eq!(parsed.ply_no, 2);
    }
}
