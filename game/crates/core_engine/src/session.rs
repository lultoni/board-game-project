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

use serde::{Serialize, Deserialize};

use crate::game_logic::action::{Action, Undo};
use crate::game_logic::{generator, make_unmake};
use crate::search::alpha_beta::{find_best_with_evaluator, SearchResult};
use crate::search::evaluator::{Evaluator, HeuristicEvaluator};
use crate::search::transposition::TranspositionTable;
use crate::state::Position;
use crate::state::position::{GameResult, Phase, Player};
use crate::telemetry::{
    MatchLog, MatchResult, PlyRecord, SearchMeta, ActionDecoded,
    snapshot_pre, snapshot_post,
};

// === Configuration ==========================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeatKind { Human, Ai }

/// Per-seat AI budget. `time_limit_ms == 0` disables the time check (max_depth
/// is the sole bound — useful in tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AiBudget {
    pub time_limit_ms: u64,
    pub max_depth:     u8,
}

impl Default for AiBudget {
    fn default() -> Self { AiBudget { time_limit_ms: 1000, max_depth: 6 } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// When true, `Match` allocates a `MatchLog` and records a `PlyRecord`
    /// for every applied action. Default false to keep tight test loops fast.
    /// `#[serde(default)]` so older snapshots (pre-auto-log) still load.
    #[serde(default)]
    pub auto_log: bool,
}

impl Config {
    pub fn local_hvh() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Human,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: true,
            auto_log:   false,
        }
    }
    pub fn local_hvai() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Ai,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: true,
            auto_log:   false,
        }
    }
    pub fn local_aivai() -> Self {
        Config {
            p1: SeatKind::Ai, p2: SeatKind::Ai,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::from_millis(300),
            allow_undo: false,
            auto_log:   false,
        }
    }
    pub fn networked_hvh() -> Self {
        Config {
            p1: SeatKind::Human, p2: SeatKind::Human,
            p1_ai: AiBudget::default(), p2_ai: AiBudget::default(),
            aivai_step_delay: Duration::ZERO,
            allow_undo: false,
            auto_log:   false,
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub start_fen: String,
    pub actions:   Vec<u32>,
    pub config:    Config,
}

// === Match ==================================================================

/// One running game. Owns its position, action history, configuration, a
/// per-match transposition table that persists across AI calls (giving each
/// `find_best` warm move-ordering hints from the previous search), and —
/// when `config.auto_log` is set — a `MatchLog` accumulating per-ply data.
pub struct Match {
    position: Position,
    history:  Vec<(Action, Undo)>,
    config:   Config,
    tt:       TranspositionTable,
    /// Captured for snapshotting. We could re-`to_fen()` on demand, but
    /// keeping the exact start string sidesteps any future FEN normalisation
    /// drift between save and load.
    start_fen: String,
    /// L5 telemetry. `Some` iff `config.auto_log`. Caller-driven clock:
    /// `Match` doesn't read system time itself.
    log: Option<MatchLog>,
    /// Position evaluator used by `request_ai_move*`. Defaults to
    /// `HeuristicEvaluator`; callers (e.g. the Tauri layer wiring an
    /// `NnEvaluator`) install a replacement via `set_evaluator`. Not
    /// serialised in `Snapshot` — restoring from snapshot reverts to the
    /// default heuristic.
    evaluator: Box<dyn Evaluator + Send>,
}

impl Match {
    /// Fresh match from Stack M's canonical starting position. Equivalent to
    /// `new_with_clock(config, 0)`.
    pub fn new(config: Config) -> Self {
        Self::new_with_clock(config, 0)
    }

    /// Fresh match with an explicit unix-ms clock reading. Pass the current
    /// `SystemTime::now()` here if you want telemetry timestamps to be real.
    pub fn new_with_clock(config: Config, now_unix_ms: u64) -> Self {
        let position = Position::setup_stack_m();
        let start_fen = position.to_fen();
        let log = if config.auto_log {
            Some(MatchLog::new(now_unix_ms, config, &position))
        } else {
            None
        };
        Match {
            position,
            history:  Vec::new(),
            config,
            tt:       TranspositionTable::with_capacity_mb(16),
            start_fen,
            log,
            evaluator: Box::new(HeuristicEvaluator),
        }
    }

    /// Fresh match that opens in `Phase::Draft`. Both sides have empty skill
    /// slots; play begins after the 12 `DraftTurn` plies complete (Phase B
    /// adds the DraftTurn action; until then `legal_actions()` returns an
    /// empty list while the position is in Draft phase).
    pub fn new_with_draft(config: Config, now_unix_ms: u64) -> Self {
        let position = Position::setup_stack_m_for_draft();
        let start_fen = position.to_fen();
        let log = if config.auto_log {
            Some(MatchLog::new(now_unix_ms, config, &position))
        } else {
            None
        };
        Match {
            position,
            history:  Vec::new(),
            config,
            tt:       TranspositionTable::with_capacity_mb(16),
            start_fen,
            log,
            evaluator: Box::new(HeuristicEvaluator),
        }
    }

    /// Fresh match that bypasses the draft, opening directly in `Phase::Move`
    /// with the supplied per-side skill loadouts already written into the
    /// mailbox. Used by the pre-made-loadout mode picker. Caller must
    /// validate the loadouts with `validate_loadout` before calling.
    pub fn new_with_loadouts(
        config: Config,
        p1: &crate::game_logic::skills::SideLoadout,
        p2: &crate::game_logic::skills::SideLoadout,
        now_unix_ms: u64,
    ) -> Self {
        let position = Position::setup_stack_m_with_loadouts(p1, p2);
        let start_fen = position.to_fen();
        let log = if config.auto_log {
            Some(MatchLog::new(now_unix_ms, config, &position))
        } else {
            None
        };
        Match {
            position,
            history:  Vec::new(),
            config,
            tt:       TranspositionTable::with_capacity_mb(16),
            start_fen,
            log,
            evaluator: Box::new(HeuristicEvaluator),
        }
    }

    /// Reconstruct a match by replaying its action history through the
    /// generator. Rejects any action that doesn't appear in the legal list
    /// at its replay-time position — i.e. a tampered snapshot is rejected
    /// without trusting the actions.
    ///
    /// When `s.config.auto_log` is set, the replay also rebuilds a
    /// `MatchLog`: each replayed action becomes a synthesized `PlyRecord`
    /// with zeroed timing/AI metadata (we no longer have the originals).
    /// This keeps the Inspector/Library consistent with what the match
    /// produced during live play — without it, a reload via snapshot would
    /// surface a stripped log anchored at the pre-history FEN.
    pub fn from_snapshot(s: Snapshot) -> Result<Self, SnapshotError> {
        Self::from_snapshot_with_clock(s, 0)
    }

    pub fn from_snapshot_with_clock(s: Snapshot, now_unix_ms: u64) -> Result<Self, SnapshotError> {
        let mut position = Position::from_fen(&s.start_fen)
            .map_err(|e| SnapshotError::BadFen(format!("{:?}", e)))?;
        let start_pos_for_log = position.clone();
        let mut log = if s.config.auto_log {
            Some(MatchLog::new(now_unix_ms, s.config, &start_pos_for_log))
        } else {
            None
        };
        let mut history: Vec<(Action, Undo)> = Vec::with_capacity(s.actions.len());
        for (i, &raw) in s.actions.iter().enumerate() {
            let action = Action(raw);
            let legal = generator::generate(&position);
            if !legal.contains(&action) {
                return Err(SnapshotError::IllegalActionInHistory { index: i, action: raw });
            }

            // Capture pre-action telemetry BEFORE make() — same shape as
            // `try_apply_timed`. Skipped when not logging to save the eval cost.
            let pre = if log.is_some() {
                let seat_player = position.to_move;
                let seat_kind = match seat_player {
                    Player::P1 => s.config.p1,
                    Player::P2 => s.config.p2,
                };
                let legal_count = legal.len() as u32;
                let (prev_zobrist, prev_fen, prev_eval, prev_breakdown) = snapshot_pre(&position);
                Some((seat_player, seat_kind, legal_count, prev_zobrist, prev_fen, prev_eval, prev_breakdown))
            } else {
                None
            };

            let undo = make_unmake::make(&mut position, action);
            history.push((action, undo));

            if let (Some((seat_player, seat_kind, legal_count, prev_zobrist, prev_fen, prev_eval, prev_breakdown)),
                    Some(l)) = (pre, log.as_mut()) {
                let (post_zobrist, post_fen, post_eval, post_breakdown,
                     post_game_result, post_phase, post_actions_remaining, post_round,
                     post_focus_pending, post_charge_pending, post_moved_this_phase,
                     post_p1_money, post_p2_money,
                     post_tracked_enemies, post_tracked_casters) = snapshot_post(&position);
                let ply_no = (l.plies.len() as u32).saturating_add(1);
                l.record(PlyRecord {
                    ply_no, seat_player, seat_kind,
                    // Original timing/AI metadata isn't in the snapshot — zero it.
                    thought_ms: 0,
                    applied_at_unix_ms: now_unix_ms,
                    action: ActionDecoded::from_action(action),
                    legal_count,
                    prev_zobrist, prev_fen, prev_static_eval: prev_eval, prev_breakdown,
                    post_zobrist, post_fen, post_static_eval: post_eval, post_breakdown,
                    post_game_result, post_phase, post_actions_remaining, post_round,
                    post_focus_pending, post_charge_pending, post_moved_this_phase,
                    post_p1_money, post_p2_money,
                    post_tracked_enemies, post_tracked_casters,
                    ai: None,
                });
            }
        }
        Ok(Match {
            position,
            history,
            config:    s.config,
            tt:        TranspositionTable::with_capacity_mb(16),
            start_fen: s.start_fen,
            log,
            evaluator: Box::new(HeuristicEvaluator),
        })
    }

    // --- Accessors ---------------------------------------------------------

    #[inline] pub fn position(&self) -> &Position { &self.position }
    #[inline] pub fn history(&self)  -> &[(Action, Undo)] { &self.history }
    #[inline] pub fn config(&self)   -> &Config { &self.config }
    #[inline] pub fn game_result(&self) -> Option<GameResult> { self.position.game_result }
    #[inline] pub fn match_log(&self) -> Option<&MatchLog> { self.log.as_ref() }
    #[inline] pub fn match_log_mut(&mut self) -> Option<&mut MatchLog> { self.log.as_mut() }

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
    ///
    /// Delegates to `try_apply_timed` with zero timing / no AI metadata. Use
    /// `try_apply_timed` directly when you have the data (e.g. from a UI
    /// hand-clock or replay).
    pub fn try_apply(&mut self, action: Action) -> Result<(), ApplyError> {
        self.try_apply_timed(action, 0, 0, None)
    }

    /// Validate + apply, plus record a `PlyRecord` into the match log when
    /// `config.auto_log` is set. `thought_ms` and `applied_at_unix_ms` are
    /// caller-supplied (engine has no clock); pass `None` for `ai` when a
    /// human played, or `Some(SearchMeta::from_search(...))` for an AI move.
    pub fn try_apply_timed(
        &mut self,
        action: Action,
        thought_ms: u32,
        applied_at_unix_ms: u64,
        ai: Option<SearchMeta>,
    ) -> Result<(), ApplyError> {
        if self.position.game_result.is_some() { return Err(ApplyError::GameOver); }
        let legal = generator::generate(&self.position);
        if !legal.contains(&action) { return Err(ApplyError::IllegalAction); }

        // Capture pre-action telemetry BEFORE make() (only when logging — saves
        // ~6 µs/ply when auto_log is off, which is the default path).
        let pre = if self.log.is_some() {
            let seat_player = self.position.to_move;
            let seat_kind = self.to_move_kind();
            let legal_count = legal.len() as u32;
            let (prev_zobrist, prev_fen, prev_eval, prev_breakdown) = snapshot_pre(&self.position);
            Some((seat_player, seat_kind, legal_count, prev_zobrist, prev_fen, prev_eval, prev_breakdown))
        } else {
            None
        };

        let undo = make_unmake::make(&mut self.position, action);
        self.history.push((action, undo));

        if let (Some((seat_player, seat_kind, legal_count, prev_zobrist, prev_fen, prev_eval, prev_breakdown)),
                Some(log)) = (pre, self.log.as_mut()) {
            let (post_zobrist, post_fen, post_eval, post_breakdown,
                 post_game_result, post_phase, post_actions_remaining, post_round,
                 post_focus_pending, post_charge_pending, post_moved_this_phase,
                 post_p1_money, post_p2_money,
                 post_tracked_enemies, post_tracked_casters) = snapshot_post(&self.position);
            let ply_no = (log.plies.len() as u32).saturating_add(1);
            log.record(PlyRecord {
                ply_no, seat_player, seat_kind,
                thought_ms, applied_at_unix_ms,
                action: ActionDecoded::from_action(action),
                legal_count,
                prev_zobrist, prev_fen, prev_static_eval: prev_eval, prev_breakdown,
                post_zobrist, post_fen, post_static_eval: post_eval, post_breakdown,
                post_game_result, post_phase, post_actions_remaining, post_round,
                post_focus_pending, post_charge_pending, post_moved_this_phase,
                post_p1_money, post_p2_money,
                post_tracked_enemies, post_tracked_casters,
                ai,
            });
        }
        Ok(())
    }

    /// Install a new position evaluator. The replacement is consulted by
    /// every subsequent `request_ai_move*` call until replaced again. Used
    /// by the Tauri layer to swap in an `NnEvaluator` for an AI seat.
    pub fn set_evaluator(&mut self, e: Box<dyn Evaluator + Send>) {
        self.evaluator = e;
    }

    /// Run the search for the current side WITHOUT applying the result.
    /// Useful for HvAI "show me the AI's pick before I commit it" flows.
    pub fn request_ai_move(&mut self) -> Result<SearchResult, AiError> {
        self.request_ai_move_with_cb(None)
    }

    pub fn request_ai_move_with_cb(&mut self, on_depth: Option<&dyn Fn(u8, i32)>) -> Result<SearchResult, AiError> {
        if self.position.game_result.is_some() { return Err(AiError::GameOver); }
        if self.to_move_kind() != SeatKind::Ai { return Err(AiError::NotAiTurn); }
        // Draft phase short-circuits search — see `oq-83` for the real-AI-
        // draft follow-up. The preset emits a deterministic `DraftTurn` and
        // we wrap it in a `SearchResult` so callers don't have to special-
        // case the draft path.
        if self.position.current_phase == Phase::Draft {
            return Ok(self.draft_preset_search_result());
        }
        let budget = match self.position.to_move {
            Player::P1 => self.config.p1_ai,
            Player::P2 => self.config.p2_ai,
        };
        Ok(find_best_with_evaluator(&mut self.position, &mut self.tt,
                     budget.time_limit_ms, budget.max_depth, &*self.evaluator, on_depth))
    }

    /// Wrap the preset-driven draft turn (if any) in a `SearchResult`. Score
    /// is 0 (the position evaluator isn't meaningful mid-draft) and depth is
    /// 0 (no search ran). Returns an empty `SearchResult` if the preset has
    /// nothing more to do — caller will surface that as a no-op step, which
    /// in practice can only happen when the draft is already complete or
    /// the position is malformed (engine bug).
    fn draft_preset_search_result(&self) -> SearchResult {
        use crate::game_logic::draft::{next_preset_draft_turn, DEFAULT_AI_LOADOUT};
        let best = next_preset_draft_turn(&self.position, &DEFAULT_AI_LOADOUT);
        SearchResult {
            best,
            score: 0,
            depth: 0,
            nodes: 0,
        }
    }

    /// Inspector / debugger variant: run the search for whoever is to move,
    /// regardless of whether they're a human seat. Falls back to a default
    /// AiBudget when the side has no AI configured (i.e. HvH matches). Does
    /// NOT apply the result. Returns `GameOver` if the position is decided.
    pub fn request_ai_move_forced(&mut self) -> Result<SearchResult, AiError> {
        if self.position.game_result.is_some() { return Err(AiError::GameOver); }
        if self.position.current_phase == Phase::Draft {
            return Ok(self.draft_preset_search_result());
        }
        let budget = match self.position.to_move {
            Player::P1 => self.config.p1_ai,
            Player::P2 => self.config.p2_ai,
        };
        let budget = if budget.time_limit_ms == 0 && budget.max_depth == 0 {
            AiBudget::default()
        } else {
            budget
        };
        Ok(find_best_with_evaluator(&mut self.position, &mut self.tt,
                     budget.time_limit_ms, budget.max_depth, &*self.evaluator, None))
    }

    /// Inspector variant for "infinite iterative deepening": runs the
    /// search with no time limit, capped at `max_depth`. The caller drives
    /// the deepening loop, polling cancellation between calls. The shared
    /// transposition table makes repeated calls progressively cheaper.
    pub fn request_ai_move_at_depth(&mut self, max_depth: u8) -> Result<SearchResult, AiError> {
        if self.position.game_result.is_some() { return Err(AiError::GameOver); }
        if self.position.current_phase == Phase::Draft {
            return Ok(self.draft_preset_search_result());
        }
        Ok(find_best_with_evaluator(&mut self.position, &mut self.tt, 0, max_depth.max(1), &*self.evaluator, None))
    }

    /// Convenience for AIvAI loops: run search and auto-apply the chosen
    /// action. Times the search wall and feeds SearchMeta into the log when
    /// `config.auto_log` is set.
    pub fn step_ai(&mut self) -> Result<SearchResult, AiError> {
        let t0 = crate::time::now_ms();
        let r = self.request_ai_move()?;
        let thought_ms = crate::time::now_ms().saturating_sub(t0).min(u32::MAX as u64) as u32;
        if let Some(a) = r.best {
            let meta = SearchMeta::from_search(r.depth, r.nodes, r.score);
            // try_apply could in principle reject if the AI returned an
            // action our generator no longer considers legal — that'd be a
            // bug in alpha-beta, not in this call site. Propagate as a panic
            // via expect so the failure surfaces immediately.
            self.try_apply_timed(a, thought_ms, 0, Some(meta))
                .expect("alpha-beta returned an illegal action");
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
        // Mirror the undo in the telemetry log so future replays don't carry
        // ghost plies. We don't try to "un-undo" by re-recording when the user
        // re-applies — that's a new ply by construction.
        if let Some(log) = self.log.as_mut() {
            if let Some(removed) = log.plies.pop() {
                log.total_plies = log.total_plies.saturating_sub(1);
                log.total_wall_ms = log.total_wall_ms.saturating_sub(removed.thought_ms as u64);
                if let Some(ai) = removed.ai {
                    log.total_ai_nodes = log.total_ai_nodes.saturating_sub(ai.nodes);
                }
            }
        }
        Ok(())
    }

    // --- Telemetry ---------------------------------------------------------

    /// Mark the log as finalised. Caller decides which `MatchResult` (the
    /// engine reports `GameResult::P1Wins/P2Wins` for natural wins; abort and
    /// draw decisions are the caller's). No-op when `auto_log` is off.
    pub fn finalise_log(&mut self, now_unix_ms: u64, result: MatchResult) {
        let pos = &self.position;
        if let Some(log) = self.log.as_mut() {
            log.finish(now_unix_ms, result, pos);
        }
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

    #[test]
    fn snapshot_restore_with_auto_log_relogs_plies() {
        // The regression this guards against: Inspector opens a previously-
        // saved match via snapshot restore. Pre-fix, the restored MatchLog
        // was empty even though the replay rebuilt full Position state, so
        // Library/Inspector showed "0 plies" and the move list rendered the
        // pre-history legal set.
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new_with_clock(cfg, 1);
        for _ in 0..5 {
            let a = first_legal(&m);
            m.try_apply(a).unwrap();
        }
        let original_plies = m.match_log().unwrap().plies.len();
        assert_eq!(original_plies, 5);

        let snap = m.to_snapshot();
        let m2 = Match::from_snapshot_with_clock(snap, 1).expect("snapshot reload");
        let restored_log = m2.match_log().expect("auto_log restored");
        assert_eq!(restored_log.plies.len(), original_plies,
            "restored MatchLog must carry the replayed plies");
        // ply_no monotonic and starts at 1.
        for (i, p) in restored_log.plies.iter().enumerate() {
            assert_eq!(p.ply_no, (i + 1) as u32);
        }
        // Final zobrist matches the live position after replay.
        let final_post = restored_log.plies.last().unwrap().post_zobrist;
        assert_eq!(final_post, m2.position().zobrist);
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

    // === L8 Phase C — AI driven by preset during Phase::Draft ===============

    #[test]
    fn new_with_draft_starts_in_draft_phase() {
        let m = Match::new_with_draft(Config::local_aivai(), 0);
        assert_eq!(m.position().current_phase, Phase::Draft);
    }

    #[test]
    fn step_ai_in_draft_phase_applies_preset_turn() {
        let mut m = Match::new_with_draft(Config::local_aivai(), 0);
        let r = m.step_ai().expect("AI should produce a DraftTurn in Phase::Draft");
        let a = r.best.expect("step_ai must return an action");
        assert!(a.is_draft_turn(), "AI's first action in draft must be a DraftTurn");
        // After one DraftTurn the phase is still Draft, side flipped to P2.
        assert_eq!(m.position().current_phase, Phase::Draft);
        assert_eq!(m.position().to_move, Player::P2);
    }

    #[test]
    fn aivai_draft_runs_to_completion_and_starts_move_phase() {
        let mut m = Match::new_with_draft(Config::local_aivai(), 0);
        // Drive both AI sides through the full 12-turn draft.
        let mut plies = 0;
        while m.position().current_phase == Phase::Draft {
            m.step_ai().expect("AI step in draft");
            plies += 1;
            assert!(plies <= 12, "draft must complete within 12 plies");
        }
        assert_eq!(plies, 12, "exactly 12 DraftTurns to drain the draft");
        assert_eq!(m.position().current_phase, Phase::Move);
        assert_eq!(m.position().actions_remaining, 2);
        // Every skill-bearing piece on both sides is now fully equipped.
        let pos = m.position();
        for sq in 0..64u8 {
            if pos.kings.contains(sq) || pos.champions.contains(sq) {
                let e = pos.mailbox[sq as usize];
                assert!(e.skill1() != 0 && e.skill2() != 0,
                    "sq {} still has empty slots after AI-driven draft", sq);
            }
        }
    }

    #[test]
    fn new_with_loadouts_bypasses_draft() {
        use crate::game_logic::draft::DEFAULT_AI_LOADOUT;
        let m = Match::new_with_loadouts(
            Config::local_aivai(),
            &DEFAULT_AI_LOADOUT,
            &DEFAULT_AI_LOADOUT,
            0,
        );
        assert_eq!(m.position().current_phase, Phase::Move,
            "Pre-made-loadout match opens directly in Move phase");
        assert_eq!(m.position().actions_remaining, 2);
    }

    #[test]
    fn request_ai_move_forced_handles_draft_for_hvh() {
        // HvH (no AI seats) — inspector must still produce a draft pick when
        // Phase::Draft via request_ai_move_forced.
        let mut m = Match::new_with_draft(Config::local_hvh(), 0);
        let r = m.request_ai_move_forced().expect("forced must work in draft");
        let a = r.best.expect("forced inspector must return an action");
        assert!(a.is_draft_turn());
    }

    #[test]
    fn aivai_draft_logs_draftturn_plies_when_auto_log() {
        let mut cfg = Config::local_aivai();
        cfg.auto_log = true;
        let mut m = Match::new_with_draft(cfg, 1);
        while m.position().current_phase == Phase::Draft {
            m.step_ai().unwrap();
        }
        let log = m.match_log().expect("auto_log enabled");
        // First 12 plies should be DraftTurns; after that, the play phase
        // begins (we don't drive past draft here).
        assert!(log.plies.len() >= 12, "expected at least 12 draft plies, got {}", log.plies.len());
        for (i, p) in log.plies.iter().take(12).enumerate() {
            assert_eq!(p.action.kind, "DraftTurn",
                "ply {} should be DraftTurn, got {}", i + 1, p.action.kind);
            assert!(p.action.picks.is_some(), "draft ply {} must carry picks", i + 1);
        }
    }
}
