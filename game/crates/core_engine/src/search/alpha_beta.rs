//! Slice 10 — Straight alpha-beta + iterative deepening + TT integration.
//!
//! Score convention: **absolute, P1's POV** (positive = P1 advantage). Side
//! to move branches on `pos.to_move`: P1 maximises, P2 minimises. This is
//! straight alpha-beta, not negamax — the evaluator already returns absolute
//! scores with terminals at ±MATE_SCORE, so flipping signs at every leaf
//! and inverting mate-score-with-ply math at every frame buys nothing.
//! Keeping the score frame-invariant makes the mate-distance math (the
//! highest-risk piece) easier to reason about and test.
//!
//! # Mate-score handling
//!
//! Terminals return `MATE_SCORE - ply` (P1 wins) or `-MATE_SCORE + ply`
//! (P2 wins), so shorter mates score higher in magnitude. The TT stores
//! the score in this node's frame; on probe we restore it to the caller's
//! frame. `score_to_tt` and `score_from_tt` are the only place this
//! arithmetic lives. Pinned by `mate_score_helpers_roundtrip`.
//!
//! # Time check
//!
//! Mask-bit every 1024 nodes — one `time::now_ms()` call per ~1024 visits.
//! On expiry: `aborted = true`, return 0. Every caller checks the flag
//! after the recursive call and propagates without storing TT garbage.
//! `time_limit_ms == 0` ⇒ no deadline, max_depth is the sole bound.
//!
//! Clock source: `crate::time::now_ms()` is monotonic ms since a fixed
//! origin. On native it wraps `Instant`; on wasm32 it imports `engine_now_ms`
//! from the host (supplied by `wasm_wrapper`). See `crate::time`.

use crate::time::now_ms;

use super::evaluator::{evaluate, MATE_SCORE};
use super::transposition::{BoundFlag, Entry, TranspositionTable};
use crate::game_logic::action::{Action, ActionKind};
use crate::game_logic::{generator, make_unmake};
use crate::state::Position;
use crate::state::position::{GameResult, Phase, Player};

const MAX_PLY: i32 = 128;
const MATE_THRESHOLD: i32 = MATE_SCORE - MAX_PLY;
const INF: i32 = MATE_SCORE + 1;
const TIME_CHECK_MASK: u64 = 0x3FF;

// --- Move-ordering tables (killers + history) ---
//
// **Killers.** Two slots per (ply, phase). Phase is part of the index because
// the Move-phase and Skill-phase action sets are disjoint — a Skill-phase
// killer is meaningless during the Move phase that follows. Indexed
// `[ply][phase_idx][slot]`. `Action(0)` is the empty sentinel.
//
// **History.** Indexed `[side][action_kind][from][to]`, incremented by
// `depth*depth` on every beta-cutoff. `EndPhase` and `EndTurn` accrue history
// at `(from, to) = (0, 0)` — the catalogue explicitly allows EndPhase to
// participate in ordering, and our move generator emits at most one
// EndPhase/EndTurn per phase so the slot collision is fine.
//
// Cleared (zeroed) at the start of every top-level `find_best` call. Surviving
// killers / history within a single iterative-deepening run is by design:
// the cumulative score over deepening iterations is what makes history useful.
const PHASES: usize = 3;        // Phase::{Draft, Move, Skill}
const KIND_COUNT: usize = 4;    // ActionKind::{Move, Skill, EndPhase, EndTurn}
const KILLERS_PER_PLY: usize = 2;
const SIDES: usize = 2;

struct OrderingTables {
    killers: [[[Action; KILLERS_PER_PLY]; PHASES]; MAX_PLY as usize],
    history: [[[[i32; 64]; 64]; KIND_COUNT]; SIDES],
}

impl OrderingTables {
    fn new() -> Box<Self> {
        // Box-allocate — ~128 KB total, too big for the stack frame.
        Box::new(OrderingTables {
            killers: [[[Action::default(); KILLERS_PER_PLY]; PHASES]; MAX_PLY as usize],
            history: [[[[0_i32; 64]; 64]; KIND_COUNT]; SIDES],
        })
    }

    #[inline]
    fn phase_idx(phase: Phase) -> usize {
        match phase {
            Phase::Draft => 0,
            Phase::Move  => 1,
            Phase::Skill => 2,
        }
    }

    #[inline]
    fn side_idx(p: Player) -> usize {
        match p { Player::P1 => 0, Player::P2 => 1 }
    }

    #[inline]
    fn kind_idx(k: ActionKind) -> usize { k as usize }

    /// Score a regular (non-Draft, non-BodyguardChoice) action for ordering.
    /// Higher = try earlier. TT-move handling lives outside (it's swap-to-0
    /// before we score at all).
    #[inline]
    fn score(&self, a: Action, side: Player, ply: i32, phase: Phase) -> i32 {
        // DraftTurn / BodyguardChoice fall through to the regular path here.
        // Their src/target/kind accessors return garbage but the resulting
        // index is still well-defined (in-range u8 → u8) — we just never
        // record cutoffs for them so their history score stays 0, and they
        // never end up in killer slots either.
        let k1 = self.killers[ply as usize][Self::phase_idx(phase)][0];
        let k2 = self.killers[ply as usize][Self::phase_idx(phase)][1];
        if a == k1 { return 1_000_000; }
        if a == k2 { return    900_000; }
        let kind = a.kind();
        let from = a.src() as usize;
        let to   = a.target() as usize;
        self.history[Self::side_idx(side)][Self::kind_idx(kind)][from][to]
    }

    /// Record a beta-cutoff. Bumps history and slides the move into the
    /// killer slot if it wasn't already killer1.
    #[inline]
    fn record_cutoff(&mut self, a: Action, side: Player, depth: i32, ply: i32, phase: Phase) {
        // Skip DraftTurn / BodyguardChoice — those tags collide with regular
        // bit layouts and recording them would corrupt history slots.
        if a.is_draft_turn() || a.is_bodyguard_choice() { return; }
        let kind = a.kind();
        let from = a.src() as usize;
        let to   = a.target() as usize;
        let bonus = depth * depth;
        // Saturating add — histories are i32 and a long search could
        // theoretically overflow; saturating keeps ordering stable past that.
        let cell = &mut self.history[Self::side_idx(side)][Self::kind_idx(kind)][from][to];
        *cell = cell.saturating_add(bonus);

        let slot = &mut self.killers[ply as usize][Self::phase_idx(phase)];
        if slot[0] != a {
            slot[1] = slot[0];
            slot[0] = a;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchResult {
    pub best:  Option<Action>,
    pub score: i32,
    pub depth: u8,
    /// Accumulated across iterative-deepening iterations.
    pub nodes: u64,
}

#[inline]
fn is_mate(s: i32) -> bool { s.abs() > MATE_THRESHOLD }

#[inline]
fn score_to_tt(s: i32, ply: i32) -> i32 {
    if      s >  MATE_THRESHOLD { s + ply }
    else if s < -MATE_THRESHOLD { s - ply }
    else                         { s }
}

#[inline]
fn score_from_tt(s: i32, ply: i32) -> i32 {
    if      s >  MATE_THRESHOLD { s - ply }
    else if s < -MATE_THRESHOLD { s + ply }
    else                         { s }
}

struct SearchCtx<'a> {
    tt:       &'a mut TranspositionTable,
    ord:      &'a mut OrderingTables,
    /// Absolute deadline in `time::now_ms()` units. `None` disables the
    /// time check (max_depth is the sole bound).
    deadline: Option<u64>,
    nodes:    u64,
    aborted:  bool,
}

fn search(pos: &mut Position, depth: i32, ply: i32,
          mut alpha: i32, mut beta: i32, ctx: &mut SearchCtx) -> i32 {
    ctx.nodes += 1;

    if ctx.nodes & TIME_CHECK_MASK == 0 {
        if let Some(d) = ctx.deadline {
            if now_ms() >= d { ctx.aborted = true; return 0; }
        }
    }

    if let Some(r) = pos.game_result {
        return match r {
            GameResult::P1Wins =>  MATE_SCORE - ply,
            GameResult::P2Wins => -MATE_SCORE + ply,
        };
    }

    if depth <= 0 { return evaluate(pos); }

    let key = pos.zobrist;
    let alpha_orig = alpha;
    let beta_orig  = beta;
    // `Action::default() == Action(0)` doubles as "no TT entry" and "no
    // chosen move yet". The two roles are disambiguated by context.
    let mut tt_move = Action::default();
    if let Some(e) = ctx.tt.probe(key) {
        tt_move = e.best_move;
        if (e.depth as i32) >= depth {
            let s = score_from_tt(e.score, ply);
            match e.flag {
                BoundFlag::Exact => return s,
                BoundFlag::Lower => if s >= beta  { return s; } else if s > alpha { alpha = s; },
                BoundFlag::Upper => if s <= alpha { return s; } else if s < beta  { beta  = s; },
            }
            if alpha >= beta { return s; }
        }
    }

    let mut moves = generator::generate(pos);
    debug_assert!(!moves.is_empty(), "non-terminal position with no legal actions");

    // Move ordering: TT-move first (if present), then killers + history score
    // for the remainder. We swap the TT-move to slot 0 *first*, then sort the
    // remainder by score so the TT-move is preserved at the front.
    let tt_at_front = if tt_move.0 != 0 {
        if let Some(i) = moves.iter().position(|m| *m == tt_move) {
            moves.swap(0, i);
            true
        } else { false }
    } else { false };
    let side = pos.to_move;
    let phase = pos.current_phase;
    if ply < MAX_PLY && depth >= 3 {
        let start = if tt_at_front { 1 } else { 0 };
        if moves.len() - start > 1 {
            // Sort descending by ordering score. Stable sort keeps original
            // (generator) order as the tiebreaker.
            //
            // Skip the sort below depth 3: at low depth the per-node sort
            // overhead dominates the cutoff savings (the TT-move is already
            // at slot 0 from the swap above, which is the only thing that
            // matters when the first move usually causes a cutoff anyway).
            moves[start..].sort_by_key(|a| -ctx.ord.score(*a, side, ply, phase));
        }
    }

    let maximising = pos.to_move == Player::P1;
    let mut best_score  = if maximising { -INF } else { INF };
    let mut best_action = Action::default();

    for a in moves {
        let undo = make_unmake::make(pos, a);
        let s = search(pos, depth - 1, ply + 1, alpha, beta, ctx);
        make_unmake::unmake(pos, &undo);
        if ctx.aborted { return 0; }

        if maximising {
            if s > best_score { best_score = s; best_action = a; }
            if best_score > alpha { alpha = best_score; }
        } else {
            if s < best_score { best_score = s; best_action = a; }
            if best_score < beta  { beta  = best_score; }
        }
        if alpha >= beta {
            // Record killer / history bump on the cutoff move. Skip if ply
            // is out of range (defensive — we already bound at MAX_PLY in
            // the ordering read above).
            if ply < MAX_PLY {
                ctx.ord.record_cutoff(a, side, depth, ply, phase);
            }
            break;
        }
    }

    let flag = if maximising {
        if      best_score <= alpha_orig { BoundFlag::Upper }
        else if best_score >= beta_orig  { BoundFlag::Lower }
        else                              { BoundFlag::Exact }
    } else {
        if      best_score >= beta_orig  { BoundFlag::Lower }
        else if best_score <= alpha_orig { BoundFlag::Upper }
        else                              { BoundFlag::Exact }
    };
    let mut entry = Entry::default();
    entry.key       = key;
    entry.score     = score_to_tt(best_score, ply);
    entry.best_move = best_action;
    entry.depth     = depth as u8;
    entry.flag      = flag;
    // `generation` is overwritten by `store` to the TT's current generation.
    ctx.tt.store(entry);

    best_score
}

/// Iterative-deepening alpha-beta. Searches from depth 1 upward until
/// either `time_limit_ms` elapses or `max_depth` is reached. Returns the
/// most recent *completed* iteration's result; partial iterations are
/// discarded so a half-explored deeper search can't return worse advice
/// than a fully-explored shallower one.
///
/// `time_limit_ms == 0` disables the time check entirely; `max_depth`
/// becomes the sole bound. `max_depth == 0` is coerced to 1 (we always
/// complete at least one ply unless the position is already terminal).
pub fn find_best(pos: &mut Position, tt: &mut TranspositionTable,
                 time_limit_ms: u64, max_depth: u8) -> SearchResult {
    tt.new_search();
    let deadline = if time_limit_ms == 0 {
        None
    } else {
        Some(now_ms().saturating_add(time_limit_ms))
    };

    let mut best = SearchResult::default();
    let mut total_nodes: u64 = 0;

    // Killers + history persist across iterative-deepening iterations within
    // this single `find_best` call. Allocated once on the heap.
    let mut ord = OrderingTables::new();

    for d in 1..=max_depth.max(1) {
        let mut ctx = SearchCtx { tt, ord: &mut ord, deadline, nodes: 0, aborted: false };
        let score = search(pos, d as i32, 0, -INF, INF, &mut ctx);
        total_nodes += ctx.nodes;

        if ctx.aborted { break; }

        let root_move = tt.probe(pos.zobrist).map(|e| e.best_move).unwrap_or_default();
        best = SearchResult {
            best:  if root_move.0 == 0 { None } else { Some(root_move) },
            score,
            depth: d,
            nodes: total_nodes,
        };

        if is_mate(score) { break; }
        if let Some(d_) = deadline { if now_ms() >= d_ { break; } }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::action::ActionKind;
    use crate::game_logic::skills::Skill;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{GameResult, Phase, Player};

    /// Local copy of the place helper used in `evaluator.rs::tests` —
    /// `make_unmake::tests::place` is `pub(super)`-scoped and not reachable.
    fn place(p: &mut Position, sq: u8, player: Player, kind: u8, entry: MailboxEntry) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        match kind {
            0 => p.kings     = p.kings     | bit,
            1 => p.champions = p.champions | bit,
            _ => p.guards    = p.guards    | bit,
        }
        p.mailbox[sq as usize] = entry;
    }

    fn fresh_tt() -> TranspositionTable {
        TranspositionTable::with_capacity_pow2(12) // 4096 slots — ample for tests
    }

    #[test]
    fn mate_score_helpers_roundtrip() {
        for &s in &[0_i32, 100, -100, MATE_SCORE - 3, -MATE_SCORE + 3, MATE_SCORE - 1, -MATE_SCORE + 1] {
            for &p in &[0_i32, 5, 17] {
                assert_eq!(score_from_tt(score_to_tt(s, p), p), s,
                    "roundtrip failed for s={} p={}", s, p);
            }
        }
        assert_eq!(score_to_tt(42, 5), 42);
        assert_eq!(score_from_tt(42, 5), 42);
    }

    #[test]
    fn find_best_returns_action_at_depth_1() {
        let mut pos = Position::setup_stack_m();
        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 1);
        assert!(r.best.is_some(), "depth-1 search must pick an action");
        assert_eq!(r.depth, 1);
        assert!(r.nodes > 0);
    }

    /// Build a minimal Move-Phase position where P1 has one Champion and one
    /// option that captures a P2 Guard. With each side having only one piece
    /// (no Bodyguards), the Move-Attack is the highest-static-eval move.
    fn p1_can_capture_guard_position() -> Position {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 35, Player::P2, 2, MailboxEntry::default().with_hp(1));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        pos
    }

    #[test]
    fn depth_1_picks_capturing_move_for_p1() {
        let mut pos = p1_can_capture_guard_position();
        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 1);
        let a = r.best.expect("must pick an action");
        assert_eq!(a.kind(), ActionKind::Move);
        assert_eq!(a.target(), 35);
        assert_eq!(a.src(), 27);
        assert!(r.score > 0, "expected P1-positive score, got {}", r.score);
    }

    /// P1 Champion adjacent to lone P2 King (HP=1 — one Move-Attack kills it).
    /// No P2 Guards anywhere means no Bodyguard redirect is possible.
    fn p1_mate_in_1_position() -> Position {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        // P2 King at HP=1: a single Move-Attack drops it to 0 and ends the game.
        place(&mut pos, 36, Player::P2, 0, MailboxEntry::default().with_hp(1));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        pos
    }

    #[test]
    fn mate_in_1_p1() {
        let mut pos = p1_mate_in_1_position();
        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 4);
        assert!(is_mate(r.score), "expected mate score, got {}", r.score);
        assert!(r.score > 0, "P1 mate must be positive, got {}", r.score);
        assert!(r.best.is_some(), "must return the capturing move");
    }

    fn p2_mate_in_1_position() -> Position {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P2, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 36, Player::P1, 0, MailboxEntry::default().with_hp(1));
        pos.to_move = Player::P2;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        pos
    }

    #[test]
    fn mate_in_1_p2() {
        let mut pos = p2_mate_in_1_position();
        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 4);
        assert!(is_mate(r.score), "expected mate score, got {}", r.score);
        assert!(r.score < 0, "P2 mate must be negative, got {}", r.score);
    }

    #[test]
    fn mate_score_distance_invariant() {
        // Same mate-in-1 position, two different max_depth values. The mate
        // distance is intrinsic to the position; deepening must not drift it.
        let mut pos1 = p1_mate_in_1_position();
        let mut tt1 = fresh_tt();
        let r2 = find_best(&mut pos1, &mut tt1, 0, 2);

        let mut pos2 = p1_mate_in_1_position();
        let mut tt2 = fresh_tt();
        let r4 = find_best(&mut pos2, &mut tt2, 0, 4);

        assert!(is_mate(r2.score));
        assert!(is_mate(r4.score));
        assert_eq!(r2.score, r4.score,
            "mate-in-1 score must be stable across max_depth; got {} vs {}",
            r2.score, r4.score);
    }

    #[test]
    fn iterative_deepening_reaches_max_depth() {
        // Quiet symmetric position — no forced mate, ID runs to max_depth.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 0, MailboxEntry::default().with_hp(2));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);

        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 10_000, 3);
        assert_eq!(r.depth, 3, "ID must reach max_depth in quiet symmetric position");
    }

    /// Brute-force minimax for cross-checking alpha-beta. Uses the same
    /// terminal-distance encoding the production search does so the
    /// comparison is apples-to-apples.
    fn minimax(pos: &mut Position, depth: i32, ply: i32) -> i32 {
        if let Some(r) = pos.game_result {
            return match r {
                GameResult::P1Wins =>  MATE_SCORE - ply,
                GameResult::P2Wins => -MATE_SCORE + ply,
            };
        }
        if depth <= 0 { return evaluate(pos); }
        let moves = generator::generate(pos);
        if moves.is_empty() { return evaluate(pos); }
        let maximising = pos.to_move == Player::P1;
        let mut best = if maximising { -INF } else { INF };
        for a in moves {
            let undo = make_unmake::make(pos, a);
            let s = minimax(pos, depth - 1, ply + 1);
            make_unmake::unmake(pos, &undo);
            if maximising { if s > best { best = s; } }
            else          { if s < best { best = s; } }
        }
        best
    }

    #[test]
    fn alpha_beta_matches_minimax_for_small_tree() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 35, Player::P2, 2, MailboxEntry::default().with_hp(1));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);

        let mut pos_clone = pos.clone();
        let manual = minimax(&mut pos_clone, 2, 0);

        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 2);
        assert_eq!(r.score, manual,
            "alpha-beta score {} disagreed with minimax {}", r.score, manual);
    }

    #[test]
    fn unmake_perfectly_restores_position() {
        let mut pos = Position::setup_stack_m();
        let zobrist_before   = pos.zobrist;
        let p1_pieces_before = pos.p1_pieces.0;
        let p2_pieces_before = pos.p2_pieces.0;
        let to_move_before   = pos.to_move;

        let mut tt = fresh_tt();
        let _ = find_best(&mut pos, &mut tt, 0, 2);

        assert_eq!(pos.zobrist, zobrist_before, "zobrist drifted across search");
        assert_eq!(pos.p1_pieces.0, p1_pieces_before);
        assert_eq!(pos.p2_pieces.0, p2_pieces_before);
        assert_eq!(pos.to_move, to_move_before);
    }

    #[test]
    fn tt_records_hits_on_second_search() {
        // `find_best` calls `new_search()` which resets stats but leaves
        // entries — so the second search probes a populated table.
        let mut pos = Position::setup_stack_m();
        let mut tt = fresh_tt();
        let _ = find_best(&mut pos, &mut tt, 0, 2);
        let _ = find_best(&mut pos, &mut tt, 0, 2);
        assert!(tt.stats().hits > 0,
            "second search should hit the populated TT; got {:?}", tt.stats());
    }

    #[test]
    fn no_best_move_when_immediately_terminal() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 3);
        // Search returns at the terminal check before reaching the store
        // branch, so the TT has no root entry and `best` is None.
        assert!(r.best.is_none());
        assert!(is_mate(r.score));
        assert!(r.score > 0);
    }

    #[test]
    fn skill_phase_does_not_panic() {
        // Smoke test: a Skill-Phase position with a real skill loadout.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 27, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 35, Player::P2, 2, MailboxEntry::default().with_hp(1));
        place(&mut pos, 63, Player::P2, 0, MailboxEntry::default().with_hp(2));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Skill;
        pos.actions_remaining = 2;
        pos.p1_money = 6;
        pos.p2_money = 6;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);

        let mut tt = fresh_tt();
        let r = find_best(&mut pos, &mut tt, 0, 2);
        assert!(r.best.is_some());
        assert_eq!(r.depth, 2);
    }

    #[test]
    fn ordering_tables_record_cutoffs() {
        // Sanity: record_cutoff bumps history and seeds killer slots.
        let mut ord = OrderingTables::new();
        let a1 = Action::encode(/*src*/ 10, /*tgt*/ 20, ActionKind::Move, 0, 0);
        let a2 = Action::encode(/*src*/ 11, /*tgt*/ 21, ActionKind::Move, 0, 0);
        // First cutoff: a1 becomes killer1, history[+1].
        ord.record_cutoff(a1, Player::P1, /*depth*/ 4, /*ply*/ 3, Phase::Move);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][0], a1);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][1], Action::default());
        assert_eq!(ord.history[OrderingTables::side_idx(Player::P1)]
                              [OrderingTables::kind_idx(ActionKind::Move)][10][20], 16);
        // Second cutoff (different move): a2 slides into killer1, a1 to killer2.
        ord.record_cutoff(a2, Player::P1, /*depth*/ 3, /*ply*/ 3, Phase::Move);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][0], a2);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][1], a1);
        // Repeating a2 must NOT push a2 into the killer2 slot (no self-displacement).
        ord.record_cutoff(a2, Player::P1, /*depth*/ 2, /*ply*/ 3, Phase::Move);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][0], a2);
        assert_eq!(ord.killers[3][OrderingTables::phase_idx(Phase::Move)][1], a1);
        // Draft / BG actions must not corrupt the table.
        let dr = Action::encode_draft_turn(1, 0, 0, 2, 1, 1);
        let bg = Action::encode_bodyguard_choice(2);
        let h_before = ord.history[OrderingTables::side_idx(Player::P1)]
                                  [OrderingTables::kind_idx(ActionKind::Move)][10][20];
        ord.record_cutoff(dr, Player::P1, 5, 3, Phase::Move);
        ord.record_cutoff(bg, Player::P1, 5, 3, Phase::Move);
        let h_after = ord.history[OrderingTables::side_idx(Player::P1)]
                                 [OrderingTables::kind_idx(ActionKind::Move)][10][20];
        assert_eq!(h_before, h_after, "Draft/BG cutoffs must not touch history");
    }
}
