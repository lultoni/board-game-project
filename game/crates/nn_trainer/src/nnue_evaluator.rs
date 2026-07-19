//! `NnueEvaluator` - the search-time `Evaluator` backed by the quantized NNUE
//! net (integer forward over an accumulator).
//!
//! Two paths, both proven bit-identical (the `accumulator` golden test pins
//! `apply == refresh`):
//! - **`evaluate(pos)` - refresh-per-call.** A pure function of `pos`: rebuild
//!   the accumulator, run the integer forward. Used at the root and as the
//!   scratch-path fallback / correctness oracle.
//! - **`*_acc` seam - incremental (ns-50 Phase-1 wiring).** The search owns an
//!   `AccHandle` stack, `fresh_acc`s the root, `push_acc`s it forward on each
//!   `make`, reads it at leaves via `eval_acc`, and save/restores on `unmake`.
//!   This is the in-search path the plan's §4.6 speed gate measures.
//!
//! Additive - `NnEvaluator` (the dense burn evaluator) stays untouched.

use core_engine::search::evaluator::{AccHandle, EvalBreakdown, Evaluator, MATE_SCORE};
use core_engine::state::position::GameResult;
use core_engine::state::Position;
use core_engine::game_logic::action::Undo;

use crate::accumulator::Accumulator;
use crate::model::Mlp;
use crate::nn_evaluator::InferenceBackend;
use crate::quantized::{QuantScales, QuantizedNet};

/// Evaluator wrapping a quantized NNUE net. Terminals bypass the net entirely
/// (±MATE_SCORE via the existing convention), preserving mate-distance math.
pub struct NnueEvaluator {
    net: QuantizedNet,
}

impl NnueEvaluator {
    pub fn new(net: QuantizedNet) -> Self {
        NnueEvaluator { net }
    }

    /// Quantize a trained f32 model and wrap it.
    pub fn from_mlp(model: &Mlp<InferenceBackend>, scales: QuantScales) -> Self {
        NnueEvaluator { net: QuantizedNet::from_mlp(model, scales) }
    }

    /// Load a persisted **NNUE** (sparse-topology) rater from disk and wrap it.
    /// Reconstructs the f32 `Mlp` from the blob + sidecar, then quantizes with
    /// the standard scales (same convention as `bootstrap::bootstrap`, so
    /// `forward_int` yields centipawns). Used by the in-game AI load path for
    /// raters whose `model_config.input_dim == NUM_FEATURES` (the dense
    /// `NnEvaluator::load_from_stem` would mismatch on those). The sidecar's
    /// `eval_scale` is not used - the quantized integer forward already emits
    /// centipawns directly via `LABEL_DIVISOR`.
    pub fn load_from_stem(
        stem: &std::path::Path,
    ) -> Result<Self, crate::persistence::PersistenceError> {
        let device: burn::tensor::Device<InferenceBackend> = Default::default();
        let (model, _meta) = crate::persistence::load_rater::<InferenceBackend>(stem, &device)?;
        let scales = QuantScales {
            qa: crate::quantized::QA,
            qw: crate::quantized::QW,
            out: crate::bootstrap::LABEL_DIVISOR,
        };
        Ok(Self::from_mlp(&model, scales))
    }

    /// Borrow the underlying quantized net.
    pub fn net(&self) -> &QuantizedNet {
        &self.net
    }
}

impl Evaluator for NnueEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        match pos.game_result {
            Some(GameResult::P1Wins) => return MATE_SCORE,
            Some(GameResult::P2Wins) => return -MATE_SCORE,
            None => {}
        }
        let acc = Accumulator::refresh(pos, self.net.ft());
        self.net.forward_int(&acc)
    }

    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown {
        // The NN doesn't decompose; fold into material_p1/p2 so the breakdown's
        // internal consistency (total == mat_p1 - mat_p2 - …) holds. Mirrors
        // `NnEvaluator::evaluate_breakdown`.
        let total = self.evaluate(pos);
        let mut b = EvalBreakdown::default();
        b.total = total;
        if total >= 0 {
            b.material_p1 = total;
        } else {
            b.material_p2 = -total;
        }
        b
    }

    // --- incremental accumulator seam (ns-50 Phase-1 wiring) ---------------
    //
    // The `AccHandle` boxes a concrete `Accumulator`. Downcasts here are
    // infallible (this evaluator built the box), but every path falls back to
    // the refresh scratch path on `None`/mismatch - never `unwrap()` - so a
    // mis-wired search can only be slower, never wrong.

    #[inline]
    fn uses_accumulator(&self) -> bool { true }

    #[inline]
    fn fresh_acc(&self, pos: &Position) -> AccHandle {
        AccHandle::new(Accumulator::refresh(pos, self.net.ft()))
    }

    #[inline]
    fn clone_acc(&self, h: &AccHandle) -> AccHandle {
        match h.downcast_ref::<Accumulator>() {
            Some(acc) => AccHandle::new(acc.clone()),
            None => AccHandle::none(),
        }
    }

    #[inline]
    fn push_acc(&self, h: &mut AccHandle, undo: &Undo, pos: &Position) {
        if let Some(acc) = h.downcast_mut::<Accumulator>() {
            acc.apply(undo, pos, self.net.ft());
        }
    }

    #[inline]
    fn eval_acc(&self, h: &AccHandle, pos: &Position) -> i32 {
        // Terminal short-circuit MUST match `evaluate` exactly (the incremental
        // leaf read has to be bit-identical to refresh-per-call).
        match pos.game_result {
            Some(GameResult::P1Wins) => return MATE_SCORE,
            Some(GameResult::P2Wins) => return -MATE_SCORE,
            None => {}
        }
        match h.downcast_ref::<Accumulator>() {
            Some(acc) => self.net.forward_int(acc),
            None => self.evaluate(pos), // fallback: scratch path, still correct.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MlpConfig;
    use crate::sparse::NUM_FEATURES;
    use core_engine::state::fen::from_fen;

    fn fresh_evaluator() -> NnueEvaluator {
        let device = Default::default();
        let model: Mlp<InferenceBackend> = MlpConfig::new().with_input_dim(NUM_FEATURES).init(&device);
        NnueEvaluator::from_mlp(&model, QuantScales::default())
    }

    #[test]
    fn terminal_p1_win_overrules() {
        let eval = fresh_evaluator();
        // A position where P2's king is captured → P1 wins. Build via a FEN
        // with only a P1 king on the board (P2 king absent = P1Wins by the
        // king-capture rule). Simpler: from a start position, assert non-mate;
        // terminals are exercised through the game_result branch directly.
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(eval.evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_win_overrules() {
        let eval = fresh_evaluator();
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(eval.evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn non_terminal_in_range() {
        let eval = fresh_evaluator();
        let pos = Position::setup_stack_m();
        let s = eval.evaluate(&pos);
        assert!(s.abs() < MATE_SCORE, "non-terminal must not claim a mate score");
    }

    #[test]
    fn works_through_dyn_trait() {
        let eval = fresh_evaluator();
        let dyn_eval: &dyn Evaluator = &eval;
        let pos = Position::setup_stack_m();
        let s = dyn_eval.evaluate(&pos);
        assert!(s.abs() < MATE_SCORE);
        // breakdown total agrees with evaluate.
        let b = dyn_eval.evaluate_breakdown(&pos);
        assert_eq!(b.total, s);
    }

    #[test]
    fn evaluate_matches_forward_int_refresh() {
        let eval = fresh_evaluator();
        // A non-terminal mid-game position from the corpus (or the start).
        let pos = from_fen("1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCC1 P1 M 2 6 6 0 1 0x0")
            .unwrap_or_else(|_| Position::setup_stack_m());
        assert!(pos.game_result.is_none());
        let expected = eval
            .net()
            .forward_int(&Accumulator::refresh(&pos, eval.net().ft()));
        assert_eq!(eval.evaluate(&pos), expected);
    }

    /// Milestone speed diagnostic: measure per-node inference cost three ways -
    /// hand-crafted `evaluate`, NNUE refresh-per-call (Phase-0 conservative
    /// bound), and NNUE incremental `apply` (the Phase-1 in-search path). Prints
    /// the ratios so the milestone verdict is grounded in a measurement, not an
    /// estimate. `#[ignore]` - timing noise makes it a diagnostic, not a gate;
    /// the real gate is the search-sweep NPS ratio.
    #[test]
    #[ignore = "timing diagnostic; run explicitly with --nocapture"]
    fn inference_cost_refresh_vs_incremental() {
        use core_engine::game_logic::{generator, make_unmake};
        use core_engine::search::evaluator::evaluate;
        use std::time::Instant;

        let eval = fresh_evaluator();
        let ft = eval.net().ft();
        let net = eval.net();

        // Build a realistic mid-game position via a short random walk.
        let mut pos = Position::setup_stack_m();
        let mut rng = 0x51EDu64;
        for _ in 0..12 {
            let acts = generator::generate(&pos);
            if acts.is_empty() { break; }
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            make_unmake::make(&mut pos, acts[(rng >> 33) as usize % acts.len()]);
        }

        const N: u32 = 200_000;

        // 1) hand-crafted eval.
        let t0 = Instant::now();
        let mut sink = 0i64;
        for _ in 0..N { sink += evaluate(&pos) as i64; }
        let hand_ns = t0.elapsed().as_nanos() as f64 / N as f64;

        // 2) NNUE refresh-per-call (full rebuild + integer forward).
        let t0 = Instant::now();
        for _ in 0..N { sink += net.forward_int(&Accumulator::refresh(&pos, ft)) as i64; }
        let refresh_ns = t0.elapsed().as_nanos() as f64 / N as f64;

        // 3) NNUE incremental inference cost: apply(undo) + forward_int - the
        // eval work the search adds per node on top of make/unmake (which every
        // evaluator pays equally, so it's excluded here). Pre-make once; time
        // clone+apply+forward against the fixed undo.
        let acts = generator::generate(&pos);
        let action = acts[0];
        let base = Accumulator::refresh(&pos, ft);
        let undo = make_unmake::make(&mut pos, action); // pos now post-make; undo fixed
        let t0 = Instant::now();
        for _ in 0..N {
            let mut acc = base.clone();
            acc.apply(&undo, &pos, ft);
            sink += net.forward_int(&acc) as i64;
        }
        let incr_ns = t0.elapsed().as_nanos() as f64 / N as f64;
        make_unmake::unmake(&mut pos, &undo);

        eprintln!("sink={sink}");
        eprintln!("hand-crafted eval : {hand_ns:8.1} ns/call");
        eprintln!("NNUE refresh      : {refresh_ns:8.1} ns/call  ({:.1}x hand)", refresh_ns / hand_ns);
        eprintln!("NNUE incremental  : {incr_ns:8.1} ns/call  ({:.1}x hand)", incr_ns / hand_ns);
    }

    // --- Phase-1 wiring gate: incremental-in-search == refresh-per-call ------

    /// Wraps an `NnueEvaluator` but leaves `uses_accumulator()` at the default
    /// `false`, forcing the search down the refresh-per-call `evaluate` path.
    /// The equivalence baseline: `apply == refresh` is proven, so the seam-on
    /// search must produce bit-identical results to this.
    struct RefreshOnlyNnue(NnueEvaluator);

    impl Evaluator for RefreshOnlyNnue {
        fn evaluate(&self, pos: &Position) -> i32 { self.0.evaluate(pos) }
        fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown { self.0.evaluate_breakdown(pos) }
        // All *_acc methods stay default → uses_accumulator() == false.
    }

    /// A few non-terminal positions: the start + a short seeded walk + a couple
    /// of corpus FENs (missing file is fine).
    fn gate_positions() -> Vec<Position> {
        use core_engine::game_logic::{generator, make_unmake};
        let mut out = vec![Position::setup_stack_m()];
        let mut pos = Position::setup_stack_m();
        let mut rng = 0xC0FFEEu64;
        for _ in 0..8 {
            let acts = generator::generate(&pos);
            if acts.is_empty() { break; }
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            make_unmake::make(&mut pos, acts[(rng >> 33) as usize % acts.len()]);
            if pos.game_result.is_none() { out.push(pos.clone()); }
        }
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../bench/corpus/corpus.txt");
        if let Ok(text) = std::fs::read_to_string(corpus) {
            for line in text.lines().take(6) {
                let fen = line.split(',').next_back().map(str::trim).unwrap_or("");
                if let Ok(p) = from_fen(fen) {
                    if p.game_result.is_none() { out.push(p); }
                }
            }
        }
        out
    }

    /// THE Phase-1 wiring gate: the search using the incremental accumulator
    /// (seam-on `NnueEvaluator`) must return BIT-IDENTICAL `SearchResult`
    /// { best, score, nodes } to the refresh-per-call path, across positions x
    /// depths. `nodes` equality is the strongest signal that ordering / TT /
    /// pruning are untouched - the whole point is "wiring must not change
    /// outputs." Also asserts determinism (seam-on twice agrees).
    #[test]
    fn search_incremental_matches_refresh_per_call() {
        use core_engine::search::alpha_beta::find_best_with_evaluator;
        use core_engine::search::transposition::TranspositionTable;

        let device = Default::default();
        let model: Mlp<InferenceBackend> =
            MlpConfig::new().with_input_dim(NUM_FEATURES).init(&device);
        // Same weights on both paths - quantize once, share via clone_from.
        let seam = NnueEvaluator::from_mlp(&model, QuantScales::default());
        let refresh = RefreshOnlyNnue(NnueEvaluator::from_mlp(&model, QuantScales::default()));

        let run = |ev: &dyn Evaluator, pos: &Position, depth: u8| {
            let mut p = pos.clone();
            let mut tt = TranspositionTable::with_capacity_pow2(14);
            find_best_with_evaluator(&mut p, &mut tt, 0, depth, ev, None)
        };

        for (pi, pos) in gate_positions().iter().enumerate() {
            for &depth in &[2u8, 4, 6] {
                let a = run(&seam, pos, depth);
                let b = run(&refresh, pos, depth);
                assert_eq!(
                    (a.best, a.score, a.nodes),
                    (b.best, b.score, b.nodes),
                    "seam-on != refresh-per-call at pos {pi} depth {depth}: \
                     seam={a:?} refresh={b:?}"
                );
                // Determinism: seam-on twice is identical.
                let a2 = run(&seam, pos, depth);
                assert_eq!(
                    (a.best, a.score, a.nodes),
                    (a2.best, a2.score, a2.nodes),
                    "seam-on nondeterministic at pos {pi} depth {depth}"
                );
            }
        }
    }
}
