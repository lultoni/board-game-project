//! `core_engine::Evaluator` impl backed by a trained MLP rater.
//!
//! Wraps a loaded `Mlp<NdArray<f32>>` plus the position encoder; on every
//! `evaluate()` call:
//!
//! 1. Terminal check first. `game_result == Some(_)` returns ±MATE_SCORE
//!    so the search keeps preferring fast mates regardless of what the NN
//!    says about the position. (Plan §4: NN never overrules terminals.)
//! 2. Encode the position via `encoding::encode_position`.
//! 3. Run a `forward()` pass with autograd-disabled `NdArray<f32>`.
//! 4. Scale the unit-range scalar back to centipawn-scale i32 to match
//!    `HeuristicEvaluator`'s sign convention and magnitude.
//!
//! ## Scale
//!
//! Training labels live in {−1, +1} (P1 wins / P2 wins). A converged rater
//! will return values in roughly [-1, +1] for non-terminal positions; the
//! `EVAL_SCALE` factor maps that to centipawn-scale.
//! `EVAL_SCALE = 3 * CHAMPION_VALUE = 3000` is a starting heuristic: a
//! "definitely winning" position (label +1) gets a score comparable to a
//! 3-Champion material lead. Final tuning happens against the gauntlet.
//!
//! ## What this does NOT provide
//!
//! `evaluate_breakdown` returns a single-bucket EvalBreakdown — the NN
//! doesn't decompose its score into material/HP/skills/etc. Callers that
//! want a real breakdown should fall back to `HeuristicEvaluator`.

use crate::encoding::{encode_position, INPUT_DIM};
use crate::model::Mlp;

use burn::backend::NdArray;
use burn::tensor::{Device, Tensor, TensorData};
use core_engine::search::evaluator::{EvalBreakdown, Evaluator, MATE_SCORE};
use core_engine::state::Position;
use core_engine::state::position::GameResult;

/// Inference backend. No autograd — eval is read-only and burn's autograd
/// wrapper carries a non-trivial cost per forward call.
pub type InferenceBackend = NdArray<f32>;

/// Centipawn-scale magnitude for `forward_output == 1.0`. See module docs.
pub const EVAL_SCALE: f32 = 3000.0;

/// Maximum non-terminal score. The NN must never overrule a mate, so we
/// clamp the scaled output strictly below `MATE_SCORE`. The gap leaves room
/// for ordering ties between near-mate heuristic positions and adjacent NN
/// scores.
pub const MAX_NN_SCORE: i32 = MATE_SCORE - 1;

/// MLP-backed evaluator. Holds the loaded model + the inference device.
/// Cheap to clone (model is `Module`-cloneable, device is `Copy`).
pub struct NnEvaluator {
    model: Mlp<InferenceBackend>,
    device: Device<InferenceBackend>,
}

impl NnEvaluator {
    /// Wrap an inference-mode `Mlp` (no autograd). The caller is expected to
    /// have stripped autograd via `into_inference` after training.
    pub fn new(model: Mlp<InferenceBackend>) -> Self {
        let device = Default::default();
        Self { model, device }
    }

    /// Single forward pass. Returns the raw scalar from the model — bench /
    /// debug only. Production callers go through `Evaluator::evaluate`.
    pub fn forward_raw(&self, pos: &Position) -> f32 {
        let features = encode_position(pos);
        debug_assert_eq!(features.len(), INPUT_DIM);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, &self.device);
        let out = self.model.forward(input);
        out.into_data().to_vec::<f32>().unwrap()[0]
    }

    /// Load a rater from `<dir>/raters/<rater_id>` and run a forward pass on
    /// `pos`. Convenience wrapper that hides the burn-side plumbing from
    /// callers (the Tauri command surface, primarily). Returns the raw NN
    /// output scalar.
    pub fn evaluate_fen_at_stem(
        stem: &std::path::Path,
        pos: &Position,
    ) -> Result<f32, crate::persistence::PersistenceError> {
        let device = Default::default();
        let (model, _meta) = crate::persistence::load_rater::<InferenceBackend>(stem, &device)?;
        let features = encode_position(pos);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        Ok(out.into_data().to_vec::<f32>().unwrap()[0])
    }
}

/// Convert an MLP forward output (unit-scale) to a centipawn-scale i32 with
/// the same sign convention as `HeuristicEvaluator`.
///
/// Clamped to `[-MAX_NN_SCORE, +MAX_NN_SCORE]` so the NN can never report a
/// false mate. Non-finite outputs (NaN / ±∞ from a poisoned rater) collapse
/// to 0 — a "no information" signal that lets the search fall back on move
/// ordering rather than propagating garbage scores.
#[inline]
fn nn_output_to_centipawns(raw: f32) -> i32 {
    if !raw.is_finite() {
        return 0;
    }
    let scaled = raw * EVAL_SCALE;
    let clamped = scaled.clamp(-(MAX_NN_SCORE as f32), MAX_NN_SCORE as f32);
    clamped.round() as i32
}

impl Evaluator for NnEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        match pos.game_result {
            Some(GameResult::P1Wins) => return MATE_SCORE,
            Some(GameResult::P2Wins) => return -MATE_SCORE,
            None => {}
        }
        nn_output_to_centipawns(self.forward_raw(pos))
    }

    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown {
        // The NN doesn't decompose; fold everything into `material_p1` if
        // positive or `material_p2` if negative so `total - (mat_p1 - mat_p2)
        // - hp - …` stays zero. Callers that want true per-bucket data have
        // to wire a `HeuristicEvaluator` instead.
        let total = self.evaluate(pos);
        let mut b = EvalBreakdown::default();
        b.total = total;
        if total >= 0 { b.material_p1 = total; }
        else          { b.material_p2 = -total; }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MlpConfig;
    use core_engine::state::position::GameResult;

    fn fresh_evaluator() -> NnEvaluator {
        let device = Default::default();
        let model: Mlp<InferenceBackend> = MlpConfig::new().init(&device);
        NnEvaluator::new(model)
    }

    #[test]
    fn terminal_p1_overrules_nn() {
        let eval = fresh_evaluator();
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(eval.evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_overrules_nn() {
        let eval = fresh_evaluator();
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(eval.evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn non_terminal_returns_finite_in_range() {
        let eval = fresh_evaluator();
        let pos = Position::setup_stack_m();
        let s = eval.evaluate(&pos);
        assert!(s.abs() <= MAX_NN_SCORE,
            "non-terminal NN score must stay below MATE_SCORE: got {}", s);
    }

    #[test]
    fn output_to_centipawns_clamps_extreme_values() {
        // EVAL_SCALE = 3000, MAX_NN_SCORE = MATE_SCORE - 1. Saturation
        // requires roughly |raw| >= MAX_NN_SCORE / EVAL_SCALE.
        let saturate = (MAX_NN_SCORE as f32 / EVAL_SCALE) + 1.0;
        assert_eq!(nn_output_to_centipawns(saturate), MAX_NN_SCORE);
        assert_eq!(nn_output_to_centipawns(-saturate), -MAX_NN_SCORE);
        assert_eq!(nn_output_to_centipawns(f32::NAN), 0);
        assert_eq!(nn_output_to_centipawns(f32::INFINITY), 0);
        assert_eq!(nn_output_to_centipawns(f32::NEG_INFINITY), 0);
        assert_eq!(nn_output_to_centipawns(0.0), 0);
        // Sub-saturation values scale linearly.
        assert_eq!(nn_output_to_centipawns(1.0), EVAL_SCALE as i32);
        assert_eq!(nn_output_to_centipawns(-1.0), -(EVAL_SCALE as i32));
    }

    #[test]
    fn breakdown_total_matches_evaluate() {
        let eval = fresh_evaluator();
        let pos = Position::setup_stack_m();
        let total = eval.evaluate(&pos);
        let b = eval.evaluate_breakdown(&pos);
        assert_eq!(b.total, total);
        // Sign-correctness: total is folded into one of the material buckets.
        if total >= 0 {
            assert_eq!(b.material_p1 as i32, total);
            assert_eq!(b.material_p2, 0);
        } else {
            assert_eq!(b.material_p2 as i32, -total);
            assert_eq!(b.material_p1, 0);
        }
    }

    #[test]
    fn evaluator_works_through_dyn_trait() {
        // Confirm dyn-dispatch compatibility — search code calls through
        // `&dyn Evaluator`, so a runtime trait object must work.
        let eval = fresh_evaluator();
        let dyn_eval: &dyn Evaluator = &eval;
        let pos = Position::setup_stack_m();
        let s = dyn_eval.evaluate(&pos);
        assert!(s.abs() <= MAX_NN_SCORE);
    }
}
