//! Flat target-agnostic engine façade for `wasm_wrapper` and `tauri_wrapper`.
//!
//! Both wrappers translate from their respective boundary (wasm-bindgen /
//! tauri::command) into calls on this surface. Keeping the wrappers thin
//! ensures they stay in lockstep — drift means a divergent UI.
//!
//! # Conventions
//!
//! - **Hot path** (called per frame / per AI step) returns flat primitives,
//!   `#[repr(C)]` structs, or `&[u16; 64]` slices — never JSON.
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
    /// `[p1_pieces, p2_pieces, kings, champions, guards]` — five u64 bitboards.
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
}

/// Per-step delta returned by `try_apply` / `step_ai`. Small on purpose —
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
    }
}

/// Borrow the 64-entry packed mailbox. Each `u16` packs HP/armor/combo/skill1/
/// skill2 — see `MailboxEntry`. Wrappers expose this as a zero-copy typed-array
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
/// clock — `Date.now()` on wasm, `SystemTime` on Tauri). Returns a
/// `StepResult` with `applied_action`, `game_result`, and zeros for AI
/// fields.
pub fn try_apply(
    m: &mut Match,
    raw_action: u32,
    applied_at_unix_ms: u64,
) -> Result<StepResult, ApplyError> {
    let a = Action(raw_action);
    m.try_apply_timed(a, /*thought_ms=*/ 0, applied_at_unix_ms, /*ai=*/ None)?;
    Ok(StepResult {
        applied_action: raw_action,
        score:          0,
        depth:          0,
        nodes:          0,
        thought_ms:     0,
        game_result:    encode_game_result(m.game_result()),
    })
}

/// Run the AI for the side-to-move under its configured budget, apply the
/// chosen action, and return a flat `StepResult`. Wrapper supplies
/// `applied_at_unix_ms`. Wall time is measured here via `crate::time::now_ms`
/// (which on wasm imports `engine_now_ms` from the host).
pub fn step_ai(m: &mut Match, applied_at_unix_ms: u64) -> Result<StepResult, AiError> {
    let t0 = crate::time::now_ms();
    let r: SearchResult = m.request_ai_move()?;
    let thought_ms = crate::time::now_ms()
        .saturating_sub(t0)
        .min(u32::MAX as u64) as u32;

    let applied = if let Some(a) = r.best {
        let meta = SearchMeta::from_search(r.depth, r.nodes, r.score);
        // alpha-beta returning an illegal action would be a bug in search,
        // not a runtime error — propagate as None so the UI can surface it.
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

/// Run the AI search for the side-to-move and return the best move it found,
/// **without applying it**. Lets inspector / debugger UIs ask "what would
/// the AI play here?" without mutating state. Honours the same per-seat
/// `AiBudget` as `step_ai`.
pub fn request_ai_move(m: &mut Match) -> Result<StepResult, AiError> {
    let t0 = crate::time::now_ms();
    let r: SearchResult = m.request_ai_move()?;
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

/// Inspector variant of `request_ai_move`: runs the search for whoever is
/// to move regardless of seat kind (Human vs Human positions included),
/// returning the best action without applying it.
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

/// Serialise a save-game snapshot.
#[inline]
pub fn snapshot_json(m: &Match) -> String {
    crate::telemetry::to_json(&m.to_snapshot())
}

/// Reconstruct a `Match` from a snapshot JSON. Wrapper supplies the wall
/// clock (engine has no `SystemTime` on wasm).
pub fn from_snapshot_json(s: &str, now_unix_ms: u64) -> Result<Match, SnapshotErrorOrParse> {
    let snap: Snapshot = crate::telemetry::from_json(s)
        .map_err(SnapshotErrorOrParse::Parse)?;
    Match::from_snapshot_with_clock(snap, now_unix_ms)
        .map_err(SnapshotErrorOrParse::Snapshot)
}

/// Error union for `from_snapshot_json` — wrappers flatten this to a string.
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
    fn try_apply_rejects_illegal_action() {
        let mut m = fresh_match();
        // Action(0) is `default()` — unlikely to be legal at the start.
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
