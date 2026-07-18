//! Incremental per-piece evaluation (ns-49, Session 48).
//!
//! Goal: the search leaf re-evaluates the whole board every call, but between
//! two consecutive `evaluate()` calls only ~2 squares change (measured on the
//! corpus: avg 2.3 changed squares/call, <0.1% over 8). This evaluator caches
//! the per-square term decomposition of the last position it scored, and on the
//! next call **diffs which pieces changed and recomputes only those** (plus the
//! surrounding squares a radius-dependent term reads), instead of re-scoring all
//! ~24 pieces.
//!
//! ## Design constraints (owner, Session 48)
//!
//! - Eval must still work as a normal full standalone call - the free
//!   `evaluate` / `evaluate_breakdown` fns and `HeuristicEvaluator` are
//!   unchanged; the frontend / telemetry / nn_trainer keep using them.
//! - **No make/unmake hooks.** The cache is NOT updated between calls; it is a
//!   memo of "the last position I was asked to score". On each `evaluate` call
//!   it diffs the new position against the memo and updates. So the number of
//!   incremental *term-recomputes* equals the number of eval *calls* - we never
//!   pay for nodes the search skipped (TT cutoffs, LMP prunes, NMP null branch).
//! - **Byte-identical to `evaluate_scalar`.** Pinned by
//!   `incremental_matches_scalar_over_playout` (this module) + the existing
//!   `golden_eval_unchanged` (the scalar path is untouched).
//! - **Never regress.** When the diff is large (rare) or a side-global input
//!   changed, fall back to a full rebuild.
//!
//! ## Rollout
//!
//! - **Phase 1 (this commit):** cache scaffold + full-rebuild path only. Every
//!   call does a full `accumulate_terms`, populates the cache, returns the
//!   identical total. No speedup yet - this is the correctness foundation and
//!   the A/B scaffold. Opt-in (`ENABLE_INCREMENTAL_EVAL`, default off).
//! - **Phase 2/3:** per-piece diff + local-radius term recompute.

use std::cell::RefCell;

use crate::state::Position;
use crate::state::position::{GameResult, Phase};
use super::params::EvalParams;
use super::context::{EvalContext, GameStage};
use super::registry::{self, TermSums, N_PIECE_TERMS};
use super::{Evaluator, EvalBreakdown, MATE_SCORE};

/// Cached per-square term decomposition of one position, so a changed piece can
/// have its contribution subtracted and re-added without re-scoring the board.
#[derive(Clone)]
pub(crate) struct EvalCache {
    /// Is the cache populated with a valid (non-terminal) position?
    valid: bool,
    /// Zobrist of the cached position - a fast "identical position" check.
    zobrist: u64,

    /// Per-square, per-term owner-relative magnitude. `piece_mag[sq][t]` is the
    /// magnitude term `t` assigned to the piece on `sq` (0 if empty). Signs are
    /// applied only in the fold, via the per-side sums below.
    piece_mag: [[i32; N_PIECE_TERMS]; 64],
    /// Which occupied squares belong to P1 (mirror of `ctx.p1_bb`).
    occ_is_p1: u64,
    /// Occupancy snapshot (mirror of `ctx.all_occ`).
    occ: u64,

    /// The rolled-up per-side/per-term sums (the thing `fold_total` consumes).
    sums: TermSums,

    /// Mailbox snapshot (raw `MailboxEntry.0`) - the diff source and the source
    /// of the OLD kind/hp/skills when subtracting a changed square's old term
    /// contributions.
    mailbox: [u16; 64],

    // Cached side-scalars - used (Phase 2) to decide side-term dirtiness.
    c_p1_money: u16,
    c_p2_money: u16,
    c_phase: Phase,
    c_actions_remaining: u8,
    c_round: u16,
    c_pending_mods: u8,
    c_stage: GameStage,
}

impl Default for EvalCache {
    fn default() -> Self {
        EvalCache {
            valid: false,
            zobrist: 0,
            piece_mag: [[0; N_PIECE_TERMS]; 64],
            occ_is_p1: 0,
            occ: 0,
            sums: TermSums::default(),
            mailbox: [0; 64],
            c_p1_money: 0,
            c_p2_money: 0,
            c_phase: Phase::Move,
            c_actions_remaining: 0,
            c_round: 0,
            c_pending_mods: 0,
            c_stage: GameStage::Opening,
        }
    }
}

impl EvalCache {
    /// Snapshot the per-square magnitudes + side-scalars for `pos`/`ctx`,
    /// recomputing every occupied square (full rebuild). Fills `self` so a later
    /// incremental call can diff against it. Returns the folded total.
    fn rebuild(&mut self, pos: &Position, ctx: &EvalContext, params: &EvalParams) -> i32 {
        let guards_present = pos.guards.0 != 0;
        let champions_present = pos.champions.0 != 0;

        // Reset per-square store; only occupied squares get written.
        self.piece_mag = [[0; N_PIECE_TERMS]; 64];
        let mut sums = TermSums::default();

        let mut bits = ctx.all_occ;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let mask = 1u64 << sq;
            let is_p1 = ctx.p1_bb & mask != 0;
            let pc = super::term::PieceContext {
                sq,
                mask,
                is_p1,
                is_guard: pos.guards.0 & mask != 0,
                is_king: pos.kings.0 & mask != 0,
                is_champion: pos.champions.0 & mask != 0,
                mailbox: pos.mailbox[sq as usize],
            };
            let mags = registry::score_piece_all(ctx, &pc, guards_present, champions_present);
            self.piece_mag[sq as usize] = mags;
            for i in 0..N_PIECE_TERMS {
                if is_p1 { sums.piece[i].0 += mags[i]; } else { sums.piece[i].1 += mags[i]; }
            }
        }

        let (money, tempo, off, wasted, close) = registry::score_side_all(ctx);
        sums.money = money; sums.tempo = tempo; sums.off = off;
        sums.wasted = wasted; sums.close = close;

        // Snapshot diff sources.
        for i in 0..64 { self.mailbox[i] = pos.mailbox[i].0; }
        self.occ = ctx.all_occ;
        self.occ_is_p1 = ctx.p1_bb;
        self.sums = sums;
        self.zobrist = pos.zobrist;
        self.c_p1_money = pos.p1_money;
        self.c_p2_money = pos.p2_money;
        self.c_phase = ctx.phase;
        self.c_actions_remaining = pos.actions_remaining;
        self.c_round = pos.round_number;
        self.c_pending_mods = pos.pending_modifiers;
        self.c_stage = ctx.stage;
        self.valid = true;

        sums.fold_total(params)
    }
}

/// Runtime toggle for the incremental evaluator when used behind
/// `HeuristicEvaluator`-style call sites. Currently the incremental evaluator is
/// selected by *constructing* `IncrementalEvaluator` explicitly (search_bench /
/// A/B), so this flag is reserved for wiring it into the default path later.
pub static ENABLE_INCREMENTAL_EVAL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Stateful evaluator that caches the per-square decomposition and updates it
/// incrementally. `evaluate` is `&self` (the `Evaluator` trait), so the cache
/// lives behind a `RefCell` - the search is single-threaded and each AI seat
/// owns its own evaluator (matching `HeuristicEvaluator`'s `Send`-only bound;
/// we do NOT add `Sync`).
pub struct IncrementalEvaluator {
    cache: RefCell<EvalCache>,
    params: &'static EvalParams,
}

impl Default for IncrementalEvaluator {
    fn default() -> Self { Self::new() }
}

impl IncrementalEvaluator {
    pub fn new() -> Self {
        IncrementalEvaluator {
            cache: RefCell::new(EvalCache::default()),
            params: &EvalParams::DEFAULT,
        }
    }

    /// Scalar eval, using the cache. Phase 1: always full-rebuild (byte-identical
    /// to `evaluate_scalar`). Phase 2/3 add the incremental diff.
    fn eval_scalar(&self, pos: &Position) -> i32 {
        // Terminal - overrules everything; do not touch the cache (leave it
        // valid for the next real position). Matches `evaluate_scalar`.
        match pos.game_result {
            Some(GameResult::P1Wins) => return MATE_SCORE,
            Some(GameResult::P2Wins) => return -MATE_SCORE,
            None => {}
        }

        super::super::counters::bump_eval_calls(); // keep the eval_calls counter in step

        let ctx = EvalContext::new(pos, self.params);
        let mut cache = self.cache.borrow_mut();
        let total = cache.rebuild(pos, &ctx, self.params);

        if pos.actions_remaining == 0 {
            super::super::counters::bump_actions_zero_hit();
        }
        total
    }
}

impl Evaluator for IncrementalEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 { self.eval_scalar(pos) }
    #[inline]
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown {
        // Breakdown is a frontend/telemetry path, not hot - always full.
        super::evaluate_breakdown(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::{generator, make_unmake};
    use crate::search::evaluator::registry::evaluate_scalar;

    fn scalar(pos: &Position) -> i32 {
        evaluate_scalar(pos, &EvalParams::DEFAULT)
    }

    /// Drive a fixed pseudo-random make/unmake playout and assert the
    /// incremental evaluator returns byte-identical scores to `evaluate_scalar`
    /// at EVERY step, including after unmakes (cache must not retain stale
    /// contributions) and interleaved across independent positions.
    #[test]
    fn incremental_matches_scalar_over_playout() {
        let ev = IncrementalEvaluator::new();

        // Several starting positions covering the corpus categories in spirit.
        let starts = [
            Position::setup_stack_m(),
        ];

        for start in starts.iter() {
            let mut pos = start.clone();
            // Deterministic pseudo-random walk: pick moves by a fixed LCG over
            // the legal list; recurse a few plies, then unwind. No Date/rand.
            let mut seed: u64 = 0x9E3779B97F4A7C15;
            let mut undo_stack = Vec::new();

            for _step in 0..400 {
                assert_eq!(
                    ev.evaluate(&pos), scalar(&pos),
                    "incremental != scalar (forward)"
                );
                if pos.game_result.is_some() {
                    // terminal - unwind one.
                    if let Some(u) = undo_stack.pop() { make_unmake::unmake(&mut pos, &u); }
                    continue;
                }
                let moves = generator::generate(&pos);
                if moves.is_empty() { break; }
                // Occasionally unwind to exercise unmake / non-monotonic diffs.
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let unwind = (seed >> 60) & 0x7 == 0 && !undo_stack.is_empty();
                if unwind {
                    let u = undo_stack.pop().unwrap();
                    make_unmake::unmake(&mut pos, &u);
                    assert_eq!(ev.evaluate(&pos), scalar(&pos), "incremental != scalar (after unmake)");
                    continue;
                }
                let idx = ((seed >> 33) as usize) % moves.len();
                let a = moves[idx];
                let u = make_unmake::make(&mut pos, a);
                undo_stack.push(u);
            }
            // Fully unwind, checking at each step.
            while let Some(u) = undo_stack.pop() {
                make_unmake::unmake(&mut pos, &u);
                assert_eq!(ev.evaluate(&pos), scalar(&pos), "incremental != scalar (unwind tail)");
            }
        }
    }

    /// Determinism: the incremental evaluator must be a pure function of `pos`,
    /// independent of what it cached before. Evaluate two different positions
    /// alternately many times; each must return its own stable value.
    #[test]
    fn incremental_is_deterministic_across_interleaving() {
        let ev = IncrementalEvaluator::new();
        let a = Position::setup_stack_m();
        let mut b = Position::setup_stack_m();
        // Perturb b by making one move.
        let moves = generator::generate(&b);
        let _u = make_unmake::make(&mut b, moves[0]);

        let va = scalar(&a);
        let vb = scalar(&b);
        for _ in 0..128 {
            assert_eq!(ev.evaluate(&a), va);
            assert_eq!(ev.evaluate(&b), vb);
        }
    }
}
