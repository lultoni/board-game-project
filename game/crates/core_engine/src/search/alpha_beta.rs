//! Slice 10 - Straight alpha-beta + iterative deepening + TT integration.
//!
//! Score convention: **absolute, P1's POV** (positive = P1 advantage). Side
//! to move branches on `pos.to_move`: P1 maximises, P2 minimises. This is
//! straight alpha-beta, not negamax - the evaluator already returns absolute
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
//! Mask-bit every 1024 nodes - one `time::now_ms()` call per ~1024 visits.
//! On expiry: `aborted = true`, return 0. Every caller checks the flag
//! after the recursive call and propagates without storing TT garbage.
//! `time_limit_ms == 0` ⇒ no deadline, max_depth is the sole bound.
//!
//! Clock source: `crate::time::now_ms()` is monotonic ms since a fixed
//! origin. On native it wraps `Instant`; on wasm32 it imports `engine_now_ms`
//! from the host (supplied by `wasm_wrapper`). See `crate::time`.

use crate::time::now_ms;

use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

/// Runtime kill-switch for the quiescence search at depth-0 boundary. Default
/// `false` (QS enabled). The AI-vs-AI evaluation harness toggles this per side
/// to grade QS vs non-QS play strength. Production callers must not flip it.
pub static DISABLE_QS: AtomicBool = AtomicBool::new(false);

/// Runtime toggle for null-move pruning. Default `true`. Validated in the
/// Session 41 Phase B sweep on corpus v2: -9.4% depth-6 nodes, +0.27 plies at
/// 1s, +18.6% NPS geom-mean, zero regressions. The search_bench harness can
/// flip this off for A/B comparison.
///
/// Null move = `Action::EndPhase` applied during `Phase::Skill`, which flips
/// the side to move (via `turn_manager::end_turn`). Only fires during Skill
/// phase - EndPhase during Move phase just transitions to the same player's
/// Skill phase, so it isn't a real null.
pub static ENABLE_NMP: AtomicBool = AtomicBool::new(true);

/// Runtime toggle for Principal Variation Search (Phase 2, Session 48). Default
/// `true`. First move at a node is searched with the full `[alpha, beta]`
/// window; siblings are probed with a null window and re-searched full only on
/// a raise. Paired with LMR (below) - the null-window probe is the structure
/// LMR's reduced-depth re-search rides on (catalogue §5). Session 36 rejected
/// PVS standalone (nothing to save atop strong ordering); it earns its keep once
/// LMR is reducing the re-search depth.
pub static ENABLE_PVS: AtomicBool = AtomicBool::new(true);

/// Runtime toggle for Late Move Reductions (Phase 2, Session 48). Default
/// `true`. Late, quiet, non-PV, non-TT, non-killer moves are searched at reduced
/// depth; a beat-the-bound result triggers a full-depth re-search. Aimed at the
/// EBF-10-12 Skill-phase tails. Never reduces: PV-window nodes' first move,
/// in-check positions, loud/King-threatening actions, or moves below
/// `LMR_MIN_IDX`.
pub static ENABLE_LMR: AtomicBool = AtomicBool::new(true);

/// Runtime toggle for Late Move Pruning / move-count pruning (Phase 3, Session
/// 48). Default `true`. At shallow depth and away from the PV, skip quiet moves
/// whose ordering index exceeds `lmp_threshold(depth)` ENTIRELY (no reduced
/// search - more aggressive than LMR). Directly attacks the EBF-12 skill-phase
/// tails that LMR's re-searches couldn't contain (opening-with-skills-03,
/// midgame-move-03). Session 36 accepted a `{depth1→16}` config standalone; this
/// re-grades atop fast-eval + LMR/PVS. Never prunes: first moves, loud/
/// King-threatening actions, in-check nodes, PV (full-window) nodes.
pub static ENABLE_LMP: AtomicBool = AtomicBool::new(true);

/// R = 2 reduction for null-move search. Depth on the null branch is
/// `depth - 1 - R`.
const NMP_R: i32 = 2;
/// Zugzwang guard: skip NMP when total piece count is under this threshold.
/// In sparse endgames a "pass" can be strictly better than any real move,
/// which is exactly the situation NMP fails on.
const NMP_MIN_PIECES: u32 = 6;

/// LMR: don't reduce the first `LMR_MIN_IDX` moves at a node (the TT-move +
/// early killers/history-ordered moves are the likely PV / cut moves).
const LMR_MIN_IDX: usize = 3;
/// LMR: only reduce at `depth >= LMR_MIN_DEPTH` (reducing shallow nodes buys
/// nothing - the child is already near a leaf).
const LMR_MIN_DEPTH: i32 = 3;

/// Late-move reduction amount: `R = base + ln(depth)·ln(idx)/divisor`, floored
/// at 1, capped so the reduced child keeps `depth >= 1`. Integer approximation
/// of the classic LMR curve (catalogue §2). `idx` is the 0-based move index.
#[inline]
fn lmr_reduction(depth: i32, idx: usize) -> i32 {
    let ld = (depth as f32).ln();
    let li = (idx as f32).ln();
    let r = 0.75 + ld * li / 2.25;
    (r as i32).max(1)
}

/// LMP: move-count threshold by depth. At `depth <= LMP_MAX_DEPTH`, quiet moves
/// with ordering index `>= lmp_threshold(depth)` are pruned outright. Grows with
/// depth (deeper nodes keep more moves). `None` = no pruning at this depth.
/// Schedule `{1→6, 2→9, 3→13, 4→18, 5→24}` - extends through depth 5 because the
/// EBF-12 offenders (opening-with-skills-03, midgame-move-03) spend most of
/// their nodes in the depth-4/5 interior, which a depth≤3 schedule never
/// touches. More aggressive than Session 36's `{1→16}`, but it now sits atop
/// fast-eval + QS + LMR/PVS, which absorb the leaf-set churn that made LMP
/// regress standalone in Session 36.
const LMP_MAX_DEPTH: i32 = 5;
#[inline]
fn lmp_threshold(depth: i32) -> Option<usize> {
    match depth {
        1 => Some(6),
        2 => Some(9),
        3 => Some(13),
        4 => Some(18),
        5 => Some(24),
        _ => None,
    }
}

use super::evaluator::{AccHandle, Evaluator, HeuristicEvaluator, MATE_SCORE};
use super::transposition::{BoundFlag, Entry, TranspositionTable};
use super::counters;
use crate::game_logic::action::{Action, ActionKind};
use crate::game_logic::{generator, make_unmake};
use crate::state::Position;
use crate::state::position::{GameResult, Phase, Player};

const MAX_PLY: i32 = 128;
const MATE_THRESHOLD: i32 = MATE_SCORE - MAX_PLY;
pub(super) const INF: i32 = MATE_SCORE + 1;
pub(super) const TIME_CHECK_MASK: u64 = 0x3FF;

// --- Move-ordering tables (killers + history) ---
//
// **Killers.** Two slots per (ply, phase). Phase is part of the index because
// the Move-phase and Skill-phase action sets are disjoint - a Skill-phase
// killer is meaningless during the Move phase that follows. Indexed
// `[ply][phase_idx][slot]`. `Action(0)` is the empty sentinel.
//
// **History.** Indexed `[side][action_kind][from][to]`, incremented by
// `depth*depth` on every beta-cutoff. `EndPhase` and `EndTurn` accrue history
// at `(from, to) = (0, 0)` - the catalogue explicitly allows EndPhase to
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

pub(super) struct OrderingTables {
    killers: [[[Action; KILLERS_PER_PLY]; PHASES]; MAX_PLY as usize],
    history: [[[[i32; 64]; 64]; KIND_COUNT]; SIDES],
}

impl OrderingTables {
    fn new() -> Box<Self> {
        // Box-allocate - ~128 KB total, too big for the stack frame.
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
        // index is still well-defined (in-range u8 → u8) - we just never
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
        // Skip DraftTurn / BodyguardChoice - those tags collide with regular
        // bit layouts and recording them would corrupt history slots.
        if a.is_draft_turn() || a.is_bodyguard_choice() { return; }
        let kind = a.kind();
        let from = a.src() as usize;
        let to   = a.target() as usize;
        let bonus = depth * depth;
        // Saturating add - histories are i32 and a long search could
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

/// Ply-prefer band for non-mate scores. When a leaf returns an eval whose
/// magnitude is ≥ SIGNIFICANT_SCORE, we shrink it by 1 per ply so the search
/// prefers reaching the same winning-or-losing eval sooner (and stalling on
/// bad ones later). Mirrors the mate-scoring convention but at a lower
/// threshold - orthogonal to `is_mate`.
///
/// The leaf-side adjustment is CLAMPED so no ply-adjusted score can cross
/// under SIGNIFICANT_SCORE. That keeps the TT round-trip clean: the "in the
/// significant band" test is identical at leaf and TT, so scores that were
/// never adjusted (raw magnitude < 500) never accidentally re-inflate on
/// retrieval.
const SIGNIFICANT_SCORE: i32 = 500;

/// Applied at leaf returns (`search` depth-0 → eval / QS boundary, `quiesce`
/// stand-pat and horizon-cap returns). Mate scores are already ply-adjusted
/// at the terminal return; guard against double-adjustment.
///
/// For positive `s ≥ SIGNIFICANT_SCORE`, returns `max(s - ply, SIGNIFICANT_SCORE)`.
/// For negative `s ≤ -SIGNIFICANT_SCORE`, returns `min(s + ply, -SIGNIFICANT_SCORE)`.
/// The clamp preserves band membership so TT's `is_significant` check agrees.
#[inline]
pub(super) fn adjust_for_ply(s: i32, ply: i32) -> i32 {
    if is_mate(s) { return s; }
    if s >=  SIGNIFICANT_SCORE { return (s - ply).max( SIGNIFICANT_SCORE); }
    if s <= -SIGNIFICANT_SCORE { return (s + ply).min(-SIGNIFICANT_SCORE); }
    s
}

#[inline]
fn score_to_tt(s: i32, ply: i32) -> i32 {
    if      s >  MATE_THRESHOLD     { s + ply }
    else if s < -MATE_THRESHOLD     { s - ply }
    else if s >=  SIGNIFICANT_SCORE { s + ply }
    else if s <= -SIGNIFICANT_SCORE { s - ply }
    else                            { s }
}

#[inline]
fn score_from_tt(s: i32, ply: i32) -> i32 {
    if      s >  MATE_THRESHOLD     { s - ply }
    else if s < -MATE_THRESHOLD     { s + ply }
    else if s >=  SIGNIFICANT_SCORE { s - ply }
    else if s <= -SIGNIFICANT_SCORE { s + ply }
    else                            { s }
}

pub(super) struct SearchCtx<'a> {
    pub(super) tt:        &'a mut TranspositionTable,
    pub(super) ord:       &'a mut OrderingTables,
    pub(super) evaluator: &'a dyn Evaluator,
    /// Absolute deadline in `time::now_ms()` units. `None` disables the
    /// time check (max_depth is the sole bound).
    pub(super) deadline: Option<u64>,
    pub(super) nodes:    u64,
    pub(super) aborted:  bool,
    /// Incremental-eval accumulator stack, top = current node's state. Empty
    /// (never touched) iff `!evaluator.uses_accumulator()` - so the default
    /// `HeuristicEvaluator` pays nothing. Invariant: while `search`/`quiesce`
    /// are between a `make` and its matching `unmake`, `acc_stack.last()`
    /// reflects the CURRENT (post-make) `pos`; save/restore keeps it balanced.
    pub(super) acc_stack: Vec<AccHandle>,
}

fn search(pos: &mut Position, depth: i32, ply: i32,
          mut alpha: i32, mut beta: i32, can_null: bool,
          ctx: &mut SearchCtx) -> i32 {
    ctx.nodes += 1;
    counters::bump_ab_nodes();

    // Whether to thread the incremental accumulator at this node. One vtable
    // read per node; when false, every `*_acc` hook below is skipped.
    let inc = ctx.evaluator.uses_accumulator();

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

    if depth <= 0 {
        if DISABLE_QS.load(AtomicOrdering::Relaxed) {
            let s = if inc {
                ctx.evaluator.eval_acc(ctx.acc_stack.last().unwrap(), pos)
            } else {
                ctx.evaluator.evaluate(pos)
            };
            return adjust_for_ply(s, ply);
        }
        return super::quiescence::quiesce(pos, alpha, beta, ply, 0, ctx);
    }

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

    // Null-move pruning. Assume the side to move passes (Action::EndPhase in
    // Skill phase, which flips STM). If even a "free" pass by the opponent
    // still keeps our position at/above beta (or at/below alpha for P2), the
    // real best move is at least as good - cutoff without generating.
    //
    // Guards:
    //   - Skill phase only (Move-phase EndPhase doesn't flip STM).
    //   - actions_remaining >= 1 (EndPhase is only legal with actions left).
    //   - No pending bodyguard (else make() will refuse EndPhase).
    //   - depth >= 3 so the reduced null search still runs at depth >= 1.
    //   - ply > 0 (never null at root - we need a real best move).
    //   - can_null: never do two nulls in a row (avoid infinite reduction).
    //   - Piece count >= NMP_MIN_PIECES to sidestep zugzwang in sparse endgames.
    if ENABLE_NMP.load(AtomicOrdering::Relaxed)
        && can_null
        && ply > 0
        && depth >= 3
        && pos.current_phase == Phase::Skill
        && pos.actions_remaining >= 1
        && pos.pending_bodyguard.is_none()
        && (pos.p1_pieces.0 | pos.p2_pieces.0).count_ones() >= NMP_MIN_PIECES
    {
        let null = Action::encode(0, 0, ActionKind::EndPhase, 0, 0);
        let undo = make_unmake::make(pos, null);
        // Save/restore the accumulator across the null move (STM flips; the
        // global re-encode in `apply` picks that up).
        let saved = if inc { Some(ctx.evaluator.clone_acc(ctx.acc_stack.last().unwrap())) } else { None };
        if inc { ctx.evaluator.push_acc(ctx.acc_stack.last_mut().unwrap(), &undo, pos); }
        // Null-window search around the appropriate bound. Since scores are
        // absolute P1-POV, the pruning condition depends on which side we're
        // trying to fail-high against: P1 (max) fails high vs beta, P2 (min)
        // fails low vs alpha. `maximising_before_null` is the ORIGINAL side
        // (the one we're pruning for); after make(), pos.to_move has flipped.
        let maximising_before_null = pos.to_move == Player::P2; // flipped by make
        let reduced = depth - 1 - NMP_R;
        let s = if maximising_before_null {
            // Original side was P1: probe with null window [beta-1, beta].
            search(pos, reduced, ply + 1, beta - 1, beta, false, ctx)
        } else {
            // Original side was P2: probe with null window [alpha, alpha+1].
            search(pos, reduced, ply + 1, alpha, alpha + 1, false, ctx)
        };
        make_unmake::unmake(pos, &undo);
        // Restore BEFORE the abort check - the parent must never read a stale
        // (post-null) accumulator.
        if inc { *ctx.acc_stack.last_mut().unwrap() = saved.unwrap(); }
        if ctx.aborted { return 0; }
        if maximising_before_null {
            // P1's turn: if the opponent-pass score already fails high, cut.
            // Avoid returning mate scores from a null branch (unverified).
            if s >= beta && !is_mate(s) { return s; }
        } else {
            if s <= alpha && !is_mate(s) { return s; }
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
            // Skip the sort below depth 3: the A/B sweep (2026-07-05, corpus
            // v2) confirmed sort-at-d≥1 loses 10% NPS and 0.3 plies at 1s;
            // sort-at-d≥2 is essentially flat but never improves depth. The
            // TT-move swap above is the only ordering work that pays for
            // itself at low depth - first-move cutoffs dominate cutoff rate.
            //
            // Skipping the full sort in favour of TT-move + killer-promote
            // only (B4) blew up skill-phase-full nodes by +55% - the
            // history-based tail ordering is load-bearing.
            moves[start..].sort_by_key(|a| -ctx.ord.score(*a, side, ply, phase));
        }
    }

    let maximising = pos.to_move == Player::P1;
    let mut best_score  = if maximising { -INF } else { INF };
    let mut best_action = Action::default();

    // PVS/LMR gates (read once per node).
    let pvs_on = ENABLE_PVS.load(AtomicOrdering::Relaxed);
    let lmr_on = ENABLE_LMR.load(AtomicOrdering::Relaxed);
    let lmp_on = ENABLE_LMP.load(AtomicOrdering::Relaxed);
    // In-check gate for LMR (never reduce when our own King is threatened -
    // those lines are tactically forced). Reuses the QS bitboard fast path.
    // `side` is the side to move at this node (captured above before the loop).
    let node_in_check = super::quiescence::is_king_threatened(pos, side);
    // LMP is only safe away from the PV (a null-window node) and out of check.
    // Never at a mate-boundary window (guarded via alpha/beta below).
    let is_pv_node = beta - alpha > 1;
    let lmp_thresh = if lmp_on && !is_pv_node && !node_in_check && depth <= LMP_MAX_DEPTH {
        lmp_threshold(depth)
    } else {
        None
    };

    for (idx, a) in moves.into_iter().enumerate() {
        // LMR eligibility (computed before make(): `is_loud` inspects only the
        // action, and `node_in_check` is a property of the parent node).
        let is_first = idx == 0;

        // LMP: at shallow non-PV nodes, prune late quiet moves outright. The
        // move list is already ordered (TT-move + killers + history), so a high
        // index is a low-value quiet tail move. Never prune loud/King-threatening
        // actions or near a mate boundary.
        if let Some(t) = lmp_thresh {
            if !is_first
                && idx >= t
                && !is_mate(alpha) && !is_mate(beta)
                && !super::quiescence::is_loud(a, pos)
            {
                continue;
            }
        }

        let reduce = lmr_on
            && !is_first
            && depth >= LMR_MIN_DEPTH
            && idx >= LMR_MIN_IDX
            && !node_in_check
            && !super::quiescence::is_loud(a, pos);
        let r = if reduce { lmr_reduction(depth, idx).min(depth - 1) } else { 0 };

        let undo = make_unmake::make(pos, a);
        // Save the pre-make accumulator, then advance once - all PVS/LMR
        // re-searches below run with `pos` fixed post-make, so a single
        // push_acc covers them; restore on unmake.
        let saved = if inc { Some(ctx.evaluator.clone_acc(ctx.acc_stack.last().unwrap())) } else { None };
        if inc { ctx.evaluator.push_acc(ctx.acc_stack.last_mut().unwrap(), &undo, pos); }
        let s = if is_first || !pvs_on {
            // First move (or PVS off): full window, full depth.
            search(pos, depth - 1, ply + 1, alpha, beta, true, ctx)
        } else if maximising {
            // Null-window probe [alpha, alpha+1] at (optionally reduced) depth.
            let mut s = search(pos, depth - 1 - r, ply + 1, alpha, alpha + 1, true, ctx);
            // LMR re-search at full depth if the reduced probe beat alpha.
            if r > 0 && !ctx.aborted && s > alpha {
                s = search(pos, depth - 1, ply + 1, alpha, alpha + 1, true, ctx);
            }
            // PVS full-window re-search if the probe raised alpha inside the window.
            if !ctx.aborted && s > alpha && s < beta {
                s = search(pos, depth - 1, ply + 1, alpha, beta, true, ctx);
            }
            s
        } else {
            // Minimising: null-window probe [beta-1, beta].
            let mut s = search(pos, depth - 1 - r, ply + 1, beta - 1, beta, true, ctx);
            if r > 0 && !ctx.aborted && s < beta {
                s = search(pos, depth - 1, ply + 1, beta - 1, beta, true, ctx);
            }
            if !ctx.aborted && s < beta && s > alpha {
                s = search(pos, depth - 1, ply + 1, alpha, beta, true, ctx);
            }
            s
        };

        make_unmake::unmake(pos, &undo);
        // Restore BEFORE the abort check (parent must not read a stale acc).
        if inc { *ctx.acc_stack.last_mut().unwrap() = saved.unwrap(); }
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
            // is out of range (defensive - we already bound at MAX_PLY in
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
    find_best_with_evaluator(pos, tt, time_limit_ms, max_depth, &HeuristicEvaluator, None)
}

/// `find_best` with an explicit evaluator. Intended for the NN-rater training
/// loop and for A/B experiments comparing rater versions. Production callers
/// use `find_best` and get the hand-coded heuristic.
///
/// `on_depth` is called after each completed iterative-deepening iteration
/// with `(depth, score)`. Pass `None` for the default (no-op) behaviour.
pub fn find_best_with_evaluator(pos: &mut Position, tt: &mut TranspositionTable,
                                time_limit_ms: u64, max_depth: u8,
                                evaluator: &dyn Evaluator,
                                on_depth: Option<&dyn Fn(u8, i32)>) -> SearchResult {
    tt.new_search();
    let deadline = if time_limit_ms == 0 {
        None
    } else {
        Some(now_ms().saturating_add(time_limit_ms))
    };

    // Forced-move short-circuit. When the position has exactly one legal
    // action (common: EndPhase-only when actions_remaining==0 and no skills
    // are castable), skip the whole tree - the caller will just apply it.
    // Score is the static eval so telemetry / UI show something meaningful.
    // nodes=1: we did examine one node (the root) to determine it was forced.
    //
    // `fallback_move` also seeds `best` below: it guarantees `find_best` returns
    // a LEGAL move on any non-terminal position even if the search is aborted by
    // the clock before iteration 1 completes (ns-49 bug: a tight/loaded time
    // budget on an expensive wide-open endgame aborted during depth 1, leaving
    // `best = default() = None`, and the UI reported "AI returned no move").
    let mut fallback_move = Action::default();
    if pos.game_result.is_none() {
        let root_moves = generator::generate(pos);
        if root_moves.len() == 1 {
            return SearchResult {
                best:  Some(root_moves[0]),
                score: evaluator.evaluate(pos),
                depth: max_depth.max(1),
                nodes: 1,
            };
        }
        if let Some(&m) = root_moves.first() { fallback_move = m; }
    }

    // Seed with the fallback legal move + static eval, so an abort before any
    // completed iteration still returns something the caller can legally apply.
    let mut best = SearchResult {
        best: if fallback_move.0 == 0 { None } else { Some(fallback_move) },
        score: if pos.game_result.is_none() { evaluator.evaluate(pos) } else { 0 },
        depth: 0,
        nodes: 0,
    };
    let mut total_nodes: u64 = 0;

    // Killers + history persist across iterative-deepening iterations within
    // this single `find_best` call. Allocated once on the heap.
    let mut ord = OrderingTables::new();

    for d in 1..=max_depth.max(1) {
        // Root accumulator: one full refresh per ID iteration when the
        // evaluator maintains one (negligible vs the iteration's node count);
        // empty Vec (zero cost) for the default heuristic.
        let acc_stack = if evaluator.uses_accumulator() {
            vec![evaluator.fresh_acc(pos)]
        } else {
            Vec::new()
        };
        let mut ctx = SearchCtx { tt, ord: &mut ord, evaluator, deadline, nodes: 0, aborted: false, acc_stack };
        let score = search(pos, d as i32, 0, -INF, INF, true, &mut ctx);
        total_nodes += ctx.nodes;

        if ctx.aborted {
            // Aborted mid-iteration: keep the best from the last COMPLETED depth
            // (or the seeded fallback if none completed). Record nodes examined.
            best.nodes = total_nodes;
            break;
        }

        let root_move = tt.probe(pos.zobrist).map(|e| e.best_move).unwrap_or_default();
        best = SearchResult {
            // Never regress to None: if this completed iteration somehow lacks a
            // root TT move, keep the prior best move rather than nulling it.
            best:  if root_move.0 != 0 { Some(root_move) } else { best.best },
            score,
            depth: d,
            nodes: total_nodes,
        };

        if let Some(cb) = on_depth { cb(d, score); }

        if is_mate(score) { break; }
        if let Some(d_) = deadline { if now_ms() >= d_ { break; } }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::evaluator::evaluate;
    use crate::game_logic::action::ActionKind;
    use crate::game_logic::skills::Skill;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{GameResult, Phase, Player};

    /// Local copy of the place helper used in `evaluator.rs::tests` -
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
        TranspositionTable::with_capacity_pow2(12) // 4096 slots - ample for tests
    }

    #[test]
    fn mate_score_helpers_roundtrip() {
        // Mate band and neutral band round-trip cleanly for any ply.
        for &s in &[0_i32, 100, -100, MATE_SCORE - 3, -MATE_SCORE + 3, MATE_SCORE - 1, -MATE_SCORE + 1] {
            for &p in &[0_i32, 5, 17] {
                assert_eq!(score_from_tt(score_to_tt(s, p), p), s,
                    "roundtrip failed for s={} p={}", s, p);
            }
        }
        // Significant band (±500 upward). Values ≥ SIGNIFICANT_SCORE are stored
        // ply-invariant and re-adjusted on retrieval - same shape as mate.
        for &s in &[SIGNIFICANT_SCORE, SIGNIFICANT_SCORE + 3, 1000, -SIGNIFICANT_SCORE, -SIGNIFICANT_SCORE - 3, -1000] {
            for &p in &[0_i32, 5, 17] {
                assert_eq!(score_from_tt(score_to_tt(s, p), p), s,
                    "significant-band roundtrip failed for s={} p={}", s, p);
            }
        }
        assert_eq!(score_to_tt(42, 5), 42);
        assert_eq!(score_from_tt(42, 5), 42);
        // Boundary: 499 is neutral, never adjusted.
        assert_eq!(score_to_tt(499, 5), 499);
        assert_eq!(score_from_tt(499, 5), 499);
    }

    #[test]
    fn adjust_for_ply_clamps_at_band_boundary() {
        // Positive band: 505 at ply 10 would go to 495 (below band) - clamped to 500.
        assert_eq!(adjust_for_ply(505, 10), SIGNIFICANT_SCORE);
        // 600 at ply 10 stays inside the band → 590.
        assert_eq!(adjust_for_ply(600, 10), 590);
        // Negative band mirror.
        assert_eq!(adjust_for_ply(-505, 10), -SIGNIFICANT_SCORE);
        assert_eq!(adjust_for_ply(-600, 10), -590);
        // Neutral band unchanged.
        assert_eq!(adjust_for_ply(100, 10), 100);
        assert_eq!(adjust_for_ply(-100, 10), -100);
        // Mate untouched (guarded by is_mate).
        assert_eq!(adjust_for_ply(MATE_SCORE - 5, 3), MATE_SCORE - 5);
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

    /// P1 Champion adjacent to lone P2 King (HP=1 - one Move-Attack kills it).
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
        // Quiet symmetric position - no forced mate, ID runs to max_depth.
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
        if depth <= 0 { return adjust_for_ply(evaluate(pos), ply); }
        let moves = generator::generate(pos);
        if moves.is_empty() { return adjust_for_ply(evaluate(pos), ply); }
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
        // entries - so the second search probes a populated table.
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

    /// ns-49 regression: a near-lost, wide-open endgame where the time-limited
    /// AI returned NO move despite legal moves existing. Root cause: an
    /// expensive depth-1 got aborted by the clock before completing, so
    /// `find_best` returned `SearchResult::default()` (best=None) and the UI
    /// reported "AI returned no move - pausing". `find_best` must ALWAYS return
    /// a legal move on a non-terminal position, even if aborted before iteration
    /// 1 completes. FEN reported by Elias.
    #[test]
    fn find_best_returns_move_even_when_clock_aborts_depth_1() {
        let fen = "8/3C[2/0/0/1/10]4/8/k[2/2/0/6/9]1C[2/2/0/6/10]5/8/8/1C[2/2/0/1/6]4K[2/2/0/6/9]1/8 P2 M 2 6 18 0 33 0x0";
        let mut pos = crate::state::fen::from_fen(fen).expect("valid fen");
        let moves = crate::game_logic::generator::generate(&pos);
        assert!(!moves.is_empty(), "position must have legal moves");
        assert!(pos.game_result.is_none(), "position is not terminal");

        // Full depth-limited search returns the true best.
        let mut tt = TranspositionTable::with_capacity_mb(16);
        assert!(find_best(&mut pos, &mut tt, 0, 6).best.is_some());

        // Tight time limits force an abort during (or before) depth 1 on this
        // expensive position - the search must still return a legal move.
        for tl in [1u64, 2, 5, 10] {
            let mut tt2 = TranspositionTable::with_capacity_mb(16);
            let mut p2 = crate::state::fen::from_fen(fen).unwrap();
            let r = find_best(&mut p2, &mut tt2, tl, 64);
            assert!(r.best.is_some(),
                    "AI returned no move under {}ms time limit (has {} legal moves)", tl, moves.len());
            // And the returned move must be one the generator considers legal.
            assert!(moves.contains(&r.best.unwrap()),
                    "returned move under {}ms not in the legal set", tl);
        }
    }
}
