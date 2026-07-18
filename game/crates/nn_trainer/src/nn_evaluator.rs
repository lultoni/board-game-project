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
//! will return values in roughly [-1, +1] for non-terminal positions; a
//! per-rater scale factor maps that to centipawn-scale. The factor is fitted
//! by `crate::calibration` (slope-only OLS against the heuristic) and stored
//! in the sidecar (`RaterMetadata::eval_scale`). Un-calibrated raters fall
//! back to `DEFAULT_EVAL_SCALE = 3 * CHAMPION_VALUE = 3000`, a starting
//! heuristic where a "definitely winning" position (label +1) gets a score
//! comparable to a 3-Champion material lead.
//!
//! ## What this does NOT provide
//!
//! `evaluate_breakdown` returns a single-bucket EvalBreakdown - the NN
//! doesn't decompose its score into material/HP/skills/etc. Callers that
//! want a real breakdown should fall back to `HeuristicEvaluator`.

use crate::encoding::{encode_position, INPUT_DIM};
use crate::model::Mlp;

use burn::tensor::{Device, Tensor, TensorData};
use core_engine::search::evaluator::{EvalBreakdown, Evaluator, MATE_SCORE};
use core_engine::state::Position;
use core_engine::state::position::GameResult;

/// Inference backend. No autograd - eval is read-only and burn's autograd
/// wrapper carries a non-trivial cost per forward call. The concrete type is
/// selected at build time via the `backend-*` Cargo feature (see
/// `crate::backend`).
pub use crate::backend::InferenceBackend;

/// Fallback centipawn-scale magnitude for `forward_output == 1.0` when a
/// rater has not yet been calibrated (sidecar `eval_scale == 0.0`). See
/// module docs and `crate::calibration`.
pub const DEFAULT_EVAL_SCALE: f32 = 3000.0;

/// Maximum non-terminal score. The NN must never overrule a mate, so we
/// clamp the scaled output strictly below `MATE_SCORE`. The gap leaves room
/// for ordering ties between near-mate heuristic positions and adjacent NN
/// scores.
pub const MAX_NN_SCORE: i32 = MATE_SCORE - 1;

/// MLP-backed evaluator. Holds the loaded model + the inference device +
/// the centipawn-scale conversion factor fitted by the calibration pass.
/// Cheap to clone (model is `Module`-cloneable, device is `Copy`).
pub struct NnEvaluator {
    model: Mlp<InferenceBackend>,
    device: Device<InferenceBackend>,
    /// Multiplier applied to the raw NN output before clamping to the
    /// centipawn range. `DEFAULT_EVAL_SCALE` for un-calibrated raters,
    /// otherwise the slope-only OLS fit from `crate::calibration`.
    scale: f32,
}

impl NnEvaluator {
    /// Wrap an inference-mode `Mlp` (no autograd). The caller is expected to
    /// have stripped autograd via `into_inference` after training. Uses
    /// `DEFAULT_EVAL_SCALE`; callers with a calibrated scale should use
    /// `with_scale`.
    pub fn new(model: Mlp<InferenceBackend>) -> Self {
        Self::with_scale(model, DEFAULT_EVAL_SCALE)
    }

    /// Wrap an inference-mode `Mlp` with an explicit centipawn-scale factor.
    /// A `scale` of `0.0` (the sentinel meaning "not yet calibrated" in the
    /// sidecar) falls back to `DEFAULT_EVAL_SCALE`. Non-finite values fall
    /// back too - a poisoned scale shouldn't take the evaluator down with it.
    pub fn with_scale(model: Mlp<InferenceBackend>, scale: f32) -> Self {
        let device = Default::default();
        let scale = if scale.is_finite() && scale != 0.0 {
            scale
        } else {
            DEFAULT_EVAL_SCALE
        };
        Self { model, device, scale }
    }

    /// The active centipawn-scale factor. Exposed for telemetry / inspector
    /// surfaces that want to display the fitted value.
    pub fn scale(&self) -> f32 { self.scale }

    /// Load a rater from disk and wrap it in an `NnEvaluator` with the
    /// calibrated scale from its sidecar (or `DEFAULT_EVAL_SCALE` when the
    /// sidecar's `eval_scale == 0.0`). Convenience for the Tauri layer so it
    /// doesn't have to name burn's `Device` type directly.
    pub fn load_from_stem(
        stem: &std::path::Path,
    ) -> Result<Self, crate::persistence::PersistenceError> {
        let device: Device<InferenceBackend> = Default::default();
        let (model, meta) = crate::persistence::load_rater::<InferenceBackend>(stem, &device)?;
        Ok(Self::with_scale(model, meta.eval_scale))
    }

    /// Single forward pass. Returns the raw scalar from the model - bench /
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
    /// output scalar and the calibrated centipawn-scale factor from the
    /// sidecar (or `DEFAULT_EVAL_SCALE` when un-calibrated).
    pub fn evaluate_fen_at_stem(
        stem: &std::path::Path,
        pos: &Position,
    ) -> Result<(f32, f32), crate::persistence::PersistenceError> {
        let device = Default::default();
        let (model, meta) = crate::persistence::load_rater::<InferenceBackend>(stem, &device)?;
        let features = encode_position(pos);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        let raw = out.into_data().to_vec::<f32>().unwrap()[0];
        let scale = if meta.eval_scale.is_finite() && meta.eval_scale != 0.0 {
            meta.eval_scale
        } else {
            DEFAULT_EVAL_SCALE
        };
        Ok((raw, scale))
    }

    /// Inspect a rater: load from disk, run a forward pass on `pos`, and
    /// collect per-layer weight stats. Used by the Training Observatory's
    /// Network Inspector via a single Tauri call so the panel doesn't have
    /// to round-trip twice. Returns `(raw_output, scale, weight_stats)`.
    pub fn inspect_fen_at_stem(
        stem: &std::path::Path,
        pos: &Position,
    ) -> Result<(f32, f32, Vec<crate::model::LayerStats>), crate::persistence::PersistenceError> {
        let device = Default::default();
        let (model, meta) = crate::persistence::load_rater::<InferenceBackend>(stem, &device)?;
        let features = encode_position(pos);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        let scalar = out.into_data().to_vec::<f32>().unwrap()[0];
        let stats = model.weight_stats();
        let scale = if meta.eval_scale.is_finite() && meta.eval_scale != 0.0 {
            meta.eval_scale
        } else {
            DEFAULT_EVAL_SCALE
        };
        Ok((scalar, scale, stats))
    }
}

/// Convert an MLP forward output (unit-scale) to a centipawn-scale i32 with
/// the same sign convention as `HeuristicEvaluator`.
///
/// Clamped to `[-MAX_NN_SCORE, +MAX_NN_SCORE]` so the NN can never report a
/// false mate. Non-finite outputs (NaN / ±∞ from a poisoned rater) collapse
/// to 0 - a "no information" signal that lets the search fall back on move
/// ordering rather than propagating garbage scores.
#[inline]
fn nn_output_to_centipawns(raw: f32, scale: f32) -> i32 {
    if !raw.is_finite() {
        return 0;
    }
    let scaled = raw * scale;
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
        nn_output_to_centipawns(self.forward_raw(pos), self.scale)
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
        // DEFAULT_EVAL_SCALE = 3000, MAX_NN_SCORE = MATE_SCORE - 1. Saturation
        // requires roughly |raw| >= MAX_NN_SCORE / DEFAULT_EVAL_SCALE.
        let scale = DEFAULT_EVAL_SCALE;
        let saturate = (MAX_NN_SCORE as f32 / scale) + 1.0;
        assert_eq!(nn_output_to_centipawns(saturate, scale), MAX_NN_SCORE);
        assert_eq!(nn_output_to_centipawns(-saturate, scale), -MAX_NN_SCORE);
        assert_eq!(nn_output_to_centipawns(f32::NAN, scale), 0);
        assert_eq!(nn_output_to_centipawns(f32::INFINITY, scale), 0);
        assert_eq!(nn_output_to_centipawns(f32::NEG_INFINITY, scale), 0);
        assert_eq!(nn_output_to_centipawns(0.0, scale), 0);
        // Sub-saturation values scale linearly.
        assert_eq!(nn_output_to_centipawns(1.0, scale), scale as i32);
        assert_eq!(nn_output_to_centipawns(-1.0, scale), -(scale as i32));
    }

    #[test]
    fn with_scale_falls_back_on_zero_or_nonfinite() {
        let device = Default::default();
        let model: Mlp<InferenceBackend> = MlpConfig::new().init(&device);
        // Cloning would require Mlp: Clone - instead just rebuild the model.
        let mk = || -> Mlp<InferenceBackend> { MlpConfig::new().init(&device) };

        let zero = NnEvaluator::with_scale(mk(), 0.0);
        assert_eq!(zero.scale(), DEFAULT_EVAL_SCALE);

        let nan = NnEvaluator::with_scale(mk(), f32::NAN);
        assert_eq!(nan.scale(), DEFAULT_EVAL_SCALE);

        let inf = NnEvaluator::with_scale(mk(), f32::INFINITY);
        assert_eq!(inf.scale(), DEFAULT_EVAL_SCALE);

        let custom = NnEvaluator::with_scale(mk(), 1234.5);
        assert!((custom.scale() - 1234.5).abs() < 1e-6);

        let _ = model;  // silence "unused" - we built mk() instead
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
        // Confirm dyn-dispatch compatibility - search code calls through
        // `&dyn Evaluator`, so a runtime trait object must work.
        let eval = fresh_evaluator();
        let dyn_eval: &dyn Evaluator = &eval;
        let pos = Position::setup_stack_m();
        let s = dyn_eval.evaluate(&pos);
        assert!(s.abs() <= MAX_NN_SCORE);
    }
}
