//! Layer 4 — Session & Match Manager.
//!
//! `Match` holds the live `Position`, action history, configuration (which
//! seats are Human vs AI, AI search budgets), and a per-match transposition
//! table that persists across consecutive AI calls.
//!
//! ## Play modes
//!
//! All four modes from ADR-005 share the same `Match` API; they differ only
//! in `Config`:
//!
//! - **Local HvH**       — both seats Human, `allow_undo: true`. UI passes
//!                          actions for whichever side is to move.
//! - **Local HvAI**      — one Human + one Ai. UI calls `try_apply` for the
//!                          human's turn; for the AI's turn it calls
//!                          `step_ai` (or `request_ai_move` then `try_apply`
//!                          to show the move first).
//! - **Local AIvAI**     — both Ai. Caller loops `step_ai` and sleeps for
//!                          `config.aivai_step_delay` between moves.
//! - **Networked HvH**   — both Human, `allow_undo: false`. Each peer runs
//!                          its own `Match`; after every local `try_apply`,
//!                          the caller sends an `ApplyEvent` over the
//!                          configured `NetworkTransport`. Each peer also
//!                          re-validates incoming events via its own
//!                          generator (per ADR-005's L7 local-validation rule).
//!
//! ## Pure
//!
//! No I/O. No threads. No async. `request_ai_move` blocks. The frontend wraps
//! it in a Web Worker (web) or background thread (Tauri).
//!
//! ## Network transport
//!
//! `Match` does NOT own a `NetworkTransport`. The caller glues the two:
//!
//! ```ignore
//! // After a successful local try_apply:
//! transport.send(ApplyEvent { action, zobrist_after: m.position().zobrist });
//!
//! // Each tick of the run loop:
//! while let Some(ev) = transport.poll() {
//!     m.try_apply(ev.action).expect("peer sent illegal action");
//!     debug_assert_eq!(m.position().zobrist, ev.zobrist_after, "desync");
//! }
//! ```
//!
//! This keeps the transport orthogonal to `Match` — trivially mockable in
//! tests, and L7's real PeerJS implementation drops in without changing L4.

use std::time::Duration;

use crate::game_logic::action::{Action, Undo};
use crate::game_logic::{generator, make_unmake};
use crate::search::alpha_beta::{find_best, SearchResult};
use crate::search::transposition::TranspositionTable;
use crate::state::Position;
use crate::state::position::{GameResult, Player};

// === Configuration ==========================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatKind { Human, Ai }

/// Per-seat AI budget. `time_limit_ms == 0` disables the time check (max_depth
/// is the sole bound — useful in tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiBudget {
    pub time_limit_ms: u64,
    pub max_depth:     u8,
}

impl Default for AiBudget {
    fn default() -> Self { AiBudget { time_limit_ms: 1000, max_depth: 6 } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub p1:      SeatKind,
    pub p2:      SeatKind,
    pub p1_ai:   AiBudget,
    pub p2_ai:   AiBudget,
    /// Hint to AIvAI loop drivers — `Match` itself never sleeps.
    pub aivai_step_delay: Duration,
    /// Local-HvH allows take-backs; networked HvH does not (history would
    /// drift between peers).
    pub allow_undo: bool,
}

impl Config {
    pub fn local_hvh() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Human,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: true,
        }
    }
    pub fn local_hvai() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Ai,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: true,
        }
    }
    pub fn local_aivai() -> Self {
        Config {
            p1: SeatKind::Ai, p2: SeatKind::Ai,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::from_millis(300),
            allow_undo: false,
        }
    }
    pub fn networked_hvh() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Human,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: false,
        }
    }
}

// === Errors =================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyError { IllegalAction, GameOver }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoError  { NoHistory, NotAllowed, GameOver }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiError    { NotAiTurn, GameOver }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    BadFen(String),
    IllegalActionInHistory { index: usize, action: u32 },
}

// === Network transport (L7 hook) ============================================

/// Event a peer broadcasts after locally applying an action. Carries the
/// post-apply Zobrist hash so the receiving peer can detect desync (per
/// ADR-005 Layer 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyEvent {
    pub action:        Action,
    pub zobrist_after: u64,
}

/// L4 ↔ L7 contract. `Match` never calls these — the run-loop glues them.
pub trait NetworkTransport {
    fn send(&mut self, event: ApplyEvent);
    fn poll(&mut self) -> Option<ApplyEvent>;
}

/// No-op transport for local-only matches and tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalTransport;

impl NetworkTransport for LocalTransport {
    fn send(&mut self, _: ApplyEvent) {}
    fn poll(&mut self) -> Option<ApplyEvent> { None }
}

// === Snapshot (save / load) =================================================

/// Compact, replayable game state: the starting FEN plus the full action
/// list. `from_snapshot` replays every action through `try_apply`, so a
/// tampered snapshot fails fast with `IllegalActionInHistory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub start_fen: String,
    pub actions:   Vec<u32>,
    pub config:    Config,
}

// === Match ==================================================================

/// One running game. Owns its position, action history, configuration, and a
/// per-match transposition table that persists across AI calls (giving each
/// `find_best` warm move-ordering hints from the previous search).
pub struct Match {
    position: Position,
    history:  Vec<(Action, Undo)>,
    config:   Config,
    tt:       TranspositionTable,
    /// Captured for snapshotting. We could re-`to_fen()` on demand, but
    /// keeping the exact start string sidesteps any future FEN normalisation
    /// drift between save and load.
    start_fen: String,
}

impl Match {
    /// Fresh match from Stack M's canonical starting position.
    pub fn new(config: Config) -> Self {
        let position = Position::setup_stack_m();
        let start_fen = position.to_fen();
        Match {
            position,
            history:  Vec::new(),
            config,
            tt:       TranspositionTable::with_capacity_mb(16),
            start_fen,
        }
    }

    /// Reconstruct a match by replaying its action history through the
    /// generator. Rejects any action that doesn't appear in the legal list
    /// at its replay-time position — i.e. a tampered snapshot is rejected
    /// without trusting the actions.
    pub fn from_snapshot(s: Snapshot) -> Result<Self, SnapshotError> {
        let mut position = Position::from_fen(&s.start_fen)
            .map_err(|e| SnapshotError::BadFen(format!("{:?}", e)))?;
        let mut history: Vec<(Action, Undo)> = Vec::with_capacity(s.actions.len());
        for (i, &raw) in s.actions.iter().enumerate() {
            let action = Action(raw);
            let legal = generator::generate(&position);
            if !legal.contains(&action) {
                return Err(SnapshotError::IllegalActionInHistory { index: i, action: raw });
            }
            let undo = make_unmake::make(&mut position, action);
            history.push((action, undo));
        }
        Ok(Match {
            position,
            history,
            config:    s.config,
            tt:        TranspositionTable::with_capacity_mb(16),
            start_fen: s.start_fen,
        })
    }

    // --- Accessors ---------------------------------------------------------

    #[inline] pub fn position(&self) -> &Position { &self.position }
    #[inline] pub fn history(&self)  -> &[(Action, Undo)] { &self.history }
    #[inline] pub fn config(&self)   -> &Config { &self.config }
    #[inline] pub fn game_result(&self) -> Option<GameResult> { self.position.game_result }

    /// Returns the seat-kind of the player whose turn it is.
    #[inline]
    pub fn to_move_kind(&self) -> SeatKind {
        match self.position.to_move {
            Player::P1 => self.config.p1,
            Player::P2 => self.config.p2,
        }
    }

    /// Legal actions for the current position. Empty when the game is over.
    pub fn legal_actions(&self) -> Vec<Action> {
        if self.position.game_result.is_some() { Vec::new() }
        else { generator::generate(&self.position) }
    }

    // --- Mutation ----------------------------------------------------------

    /// Validate `action` against `generator::generate`, then apply via
    /// `make_unmake::make`. On rejection the position is unchanged.
    pub fn try_apply(&mut self, action: Action) -> Result<(), ApplyError> {
        if self.position.game_result.is_some() { return Err(ApplyError::GameOver); }
        let legal = generator::generate(&self.position);
        if !legal.contains(&action) { return Err(ApplyError::IllegalAction); }
        let undo = make_unmake::make(&mut self.position, action);
        self.history.push((action, undo));
        Ok(())
    }

    /// Run the search for the current side WITHOUT applying the result.
    /// Useful for HvAI "show me the AI's pick before I commit it" flows.
    pub fn request_ai_move(&mut self) -> Result<SearchResult, AiError> {
        if self.position.game_result.is_some() { return Err(AiError::GameOver); }
        if self.to_move_kind() != SeatKind::Ai { return Err(AiError::NotAiTurn); }
        let budget = match self.position.to_move {
            Player::P1 => self.config.p1_ai,
            Player::P2 => self.config.p2_ai,
        };
        Ok(find_best(&mut self.position, &mut self.tt,
                     budget.time_limit_ms, budget.max_depth))
    }

    /// Convenience for AIvAI loops: run search and auto-apply the chosen
    /// action. If the search returned no move (shouldn't happen at a
    /// non-terminal node), the call is a no-op and the result still carries
    /// the score for inspection.
    pub fn step_ai(&mut self) -> Result<SearchResult, AiError> {
        let r = self.request_ai_move()?;
        if let Some(a) = r.best {
            // try_apply could in principle reject if the AI returned an
            // action our generator no longer considers legal — that'd be a
            // bug in alpha-beta, not in this call site. Propagate as a panic
            // via expect so the failure surfaces immediately.
            self.try_apply(a).expect("alpha-beta returned an illegal action");
        }
        Ok(r)
    }

    /// Pop the last applied action and reverse it. Gated by `config.allow_undo`.
    pub fn undo_last(&mut self) -> Result<(), UndoError> {
        if !self.config.allow_undo            { return Err(UndoError::NotAllowed); }
        // GameOver intentionally NOT a hard stop here — undoing the terminal
        // move (a King capture) is a legitimate take-back. `make_unmake::unmake`
        // restores `game_result` to its pre-move value.
        let (_, undo) = self.history.pop().ok_or(UndoError::NoHistory)?;
        make_unmake::unmake(&mut self.position, &undo);
        Ok(())
    }

    // --- Snapshot ----------------------------------------------------------

    pub fn to_snapshot(&self) -> Snapshot {
        Snapshot {
            start_fen: self.start_fen.clone(),
            actions:   self.history.iter().map(|(a, _)| a.0).collect(),
            config:    self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::action::ActionKind;
    use crate::state::position::Phase;

    fn first_legal(m: &Match) -> Action {
        *m.legal_actions().first().expect("expected at least one legal action")
    }

    // --- Construction ------------------------------------------------------

    #[test]
    fn new_match_starts_at_stack_m() {
        let m = Match::new(Config::local_hvh());
        let p = m.position();
        assert_eq!(p.to_move, Player::P1);
        assert_eq!(p.current_phase, Phase::Move);
        assert_eq!(p.actions_remaining, 2);
        assert_eq!(p.p1_money, 6);
        assert_eq!(p.p2_money, 6);
        assert_eq!(p.round_number, 1);
        assert!(m.history().is_empty());
        assert!(p.game_result.is_none());
    }

    // --- Apply pipeline ----------------------------------------------------

    #[test]
    fn try_apply_legal_succeeds() {
        let mut m = Match::new(Config::local_hvh());
        let a = first_legal(&m);
        let res = m.try_apply(a);
        assert!(res.is_ok());
        assert_eq!(m.history().len(), 1);
        assert_eq!(m.history()[0].0, a);
    }

    #[test]
    fn try_apply_illegal_rejected_state_unchanged() {
        let mut m = Match::new(Config::local_hvh());
        let zob_before = m.position().zobrist;
        // Action(0) is `Action::default()` — the "no action" sentinel; the
        // generator never emits it, so it's a reliable illegal-action probe.
        let bogus = Action::default();
        let res = m.try_apply(bogus);
        assert_eq!(res, Err(ApplyError::IllegalAction));
        assert!(m.history().is_empty());
        assert_eq!(m.position().zobrist, zob_before);
    }

    #[test]
    fn to_move_kind_tracks_seat() {
        // HvAI: P1 is human, P2 is AI. Confirm to_move_kind flips with turn.
        let mut m = Match::new(Config::local_hvai());
        assert_eq!(m.to_move_kind(), SeatKind::Human);
        // Walk forward until to_move flips. Repeatedly apply the first legal
        // action — a bounded loop because Move-Phase actions_remaining=2 and
        // EndPhase / EndTurn are always emitted by the generator when needed.
        for _ in 0..32 {
            let a = first_legal(&m);
            m.try_apply(a).unwrap();
            if m.position().to_move == Player::P2 { break; }
        }
        assert_eq!(m.position().to_move, Player::P2);
        assert_eq!(m.to_move_kind(), SeatKind::Ai);
    }

    // --- Undo --------------------------------------------------------------

    #[test]
    fn undo_round_trip() {
        let mut m = Match::new(Config::local_hvh());
        let zob_start = m.position().zobrist;
        let p1_before = m.position().p1_pieces.0;
        let p2_before = m.position().p2_pieces.0;

        // Apply a handful of legal moves (whichever they happen to be).
        for _ in 0..6 {
            let a = first_legal(&m);
            m.try_apply(a).unwrap();
        }
        assert_eq!(m.history().len(), 6);
        for _ in 0..6 {
            m.undo_last().unwrap();
        }
        assert!(m.history().is_empty());
        assert_eq!(m.position().zobrist, zob_start);
        assert_eq!(m.position().p1_pieces.0, p1_before);
        assert_eq!(m.position().p2_pieces.0, p2_before);
    }

    #[test]
    fn undo_rejected_when_not_allowed() {
        let mut m = Match::new(Config::networked_hvh());
        let a = first_legal(&m);
        m.try_apply(a).unwrap();
        assert_eq!(m.undo_last(), Err(UndoError::NotAllowed));
        // History intact — caller must accept the action as final.
        assert_eq!(m.history().len(), 1);
    }

    #[test]
    fn undo_rejected_when_empty() {
        let mut m = Match::new(Config::local_hvh());
        assert_eq!(m.undo_last(), Err(UndoError::NoHistory));
    }

    // --- AI driver ---------------------------------------------------------

    #[test]
    fn request_ai_move_returns_legal_action() {
        let mut m = Match::new(Config::local_aivai());
        m.config.p1_ai = AiBudget { time_limit_ms: 0, max_depth: 2 }; // fast
        let r = m.request_ai_move().expect("AI move available");
        let a = r.best.expect("non-terminal: must pick something");
        assert!(m.legal_actions().contains(&a),
            "AI returned action {:?} not in legal set", a);
    }

    #[test]
    fn request_ai_move_errors_on_human_turn() {
        let mut m = Match::new(Config::local_hvh());
        assert_eq!(m.request_ai_move().unwrap_err(), AiError::NotAiTurn);
    }

    #[test]
    fn step_ai_advances_position() {
        let mut m = Match::new(Config::local_aivai());
        m.config.p1_ai = AiBudget { time_limit_ms: 0, max_depth: 2 };
        let r = m.step_ai().expect("step ok");
        assert!(r.best.is_some());
        assert_eq!(m.history().len(), 1);
    }

    #[test]
    fn aivai_terminates_within_budget() {
        // Shallow-depth random-ish play; should still terminate.
        let mut m = Match::new(Config::local_aivai());
        m.config.p1_ai = AiBudget { time_limit_ms: 0, max_depth: 2 };
        m.config.p2_ai = AiBudget { time_limit_ms: 0, max_depth: 2 };
        const MAX_PLIES: usize = 5_000;
        for _ in 0..MAX_PLIES {
            if m.game_result().is_some() { break; }
            m.step_ai().expect("ai step");
        }
        assert!(m.game_result().is_some(),
            "AIvAI did not terminate within {} plies", MAX_PLIES);
    }

    // --- Snapshot ----------------------------------------------------------

    #[test]
    fn snapshot_roundtrip() {
        let mut m = Match::new(Config::local_hvh());
        for _ in 0..5 {
            let a = first_legal(&m);
            m.try_apply(a).unwrap();
        }
        let zob_before = m.position().zobrist;
        let len_before = m.history().len();
        let cfg_before = *m.config();

        let snap = m.to_snapshot();
        let m2 = Match::from_snapshot(snap).expect("snapshot reload");
        assert_eq!(m2.position().zobrist, zob_before);
        assert_eq!(m2.history().len(), len_before);
        assert_eq!(*m2.config(), cfg_before);
    }

    #[test]
    fn snapshot_rejects_illegal_history() {
        let m = Match::new(Config::local_hvh());
        let mut snap = m.to_snapshot();
        // Splice in Action::default() — never emitted by generate(), so
        // replay rejects at index 0.
        snap.actions.push(Action::default().0);
        match Match::from_snapshot(snap) {
            Err(SnapshotError::IllegalActionInHistory { index, .. }) => assert_eq!(index, 0),
            Err(other) => panic!("expected IllegalActionInHistory, got {:?}", other),
            Ok(_) => panic!("expected error, snapshot accepted"),
        }
    }

    // --- Terminal handling -------------------------------------------------

    #[test]
    fn game_over_blocks_further_applies() {
        let mut m = Match::new(Config::local_hvh());
        // Directly mark the game as over (skipping a full mate sequence
        // here — terminal-handling is what's under test, not mate search).
        m.position.game_result = Some(GameResult::P1Wins);
        let a = Action::default(); // any action; should be rejected before legality check
        assert_eq!(m.try_apply(a), Err(ApplyError::GameOver));
        assert!(m.legal_actions().is_empty());
    }

    // --- Transport ---------------------------------------------------------

    #[test]
    fn local_transport_is_inert() {
        let mut t = LocalTransport;
        assert!(t.poll().is_none());
        t.send(ApplyEvent { action: Action::default(), zobrist_after: 0 });
        assert!(t.poll().is_none());
    }

    #[test]
    fn apply_event_matches_zobrist_after_local_apply() {
        // The L7 contract: ApplyEvent.zobrist_after equals position().zobrist
        // taken immediately after the local try_apply.
        let mut m = Match::new(Config::local_hvh());
        let a = first_legal(&m);
        m.try_apply(a).unwrap();
        let ev = ApplyEvent { action: a, zobrist_after: m.position().zobrist };
        assert_eq!(ev.zobrist_after, m.position().zobrist);
    }

    // --- Action shape sanity ----------------------------------------------

    /// Without this, the test above for "first legal action" might silently
    /// be testing only EndPhase, which would still pass but would be a thin
    /// signal. Confirm Stack-M start has real Move-kind actions available.
    #[test]
    fn stack_m_start_has_move_actions() {
        let m = Match::new(Config::local_hvh());
        let legals = m.legal_actions();
        let move_count = legals.iter().filter(|a| a.kind() == ActionKind::Move).count();
        assert!(move_count > 0, "Stack M start should emit Move actions");
    }
}
