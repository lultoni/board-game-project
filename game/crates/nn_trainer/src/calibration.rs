//! Centipawn-scale calibration for a trained NN rater.
//!
//! A converged rater returns roughly {−1, +1} for "P2 winning" / "P1 winning"
//! positions, but the search compares those scores against a heuristic
//! evaluator working in centipawns. Multiplying the raw NN output by
//! `DEFAULT_EVAL_SCALE = 3000` is a reasonable starting heuristic - but a
//! *fitted* scale makes the NN's preferences land in the same magnitude as
//! the heuristic's, which keeps move ordering and alpha-beta windows sensible
//! when the two evaluators ever sit side-by-side (e.g. as separate seats in
//! the gauntlet, or when callers blend their outputs).
//!
//! ## Method: slope-only OLS
//!
//! We fit a single scalar `k` minimising
//!   Σ (k·nn_raw_i − heuristic_cp_i)²
//! over a calibration probe set. The closed form is
//!   k = Σ(x_i · y_i) / Σ(x_i²)
//! where `x_i = nn_raw_i` and `y_i = heuristic_cp_i`. No intercept - the NN
//! is sign-symmetric around 0 by construction (`{−1,+1}` labels averaged to
//! 0), and adding an offset would distort the symmetry we rely on for
//! `evaluate(pos) == −evaluate(mirror(pos))`-style invariants in the search.
//!
//! ## Probe set
//!
//! The caller picks the positions. The orchestrator passes a hold-out slice
//! from `generate_corpus` (a small fraction the trainer didn't see); ad-hoc
//! callers can build whatever probe set they like. Calibration is *cheap* -
//! one forward pass + one heuristic evaluation per probe.
//!
//! ## Edge cases
//!
//! - Empty probe set → returns `None`.
//! - All-zero NN outputs → returns `None` (avoids division by zero; the
//!   un-calibrated `DEFAULT_EVAL_SCALE` fallback is the right answer here).
//! - Non-finite ratios → returns `None`.
//!
//! When this returns `None` the caller should leave `eval_scale = 0.0` in
//! the sidecar, which `NnEvaluator::with_scale` interprets as "use the
//! default."

use crate::nn_evaluator::InferenceBackend;
use crate::model::Mlp;
use crate::encoding::{encode_position, INPUT_DIM};

use burn::tensor::{Tensor, TensorData};
use core_engine::search::evaluator::Evaluator;
use core_engine::state::Position;

/// Run the slope-only OLS fit over `probes`, returning the fitted scale or
/// `None` if the probe set is degenerate. `model` is the trained rater (in
/// inference form, no autograd); `heuristic` is the centipawn-scale reference
/// (typically `HeuristicEvaluator`).
pub fn calibrate_rater<E: Evaluator>(
    model: &Mlp<InferenceBackend>,
    heuristic: &E,
    probes: &[Position],
) -> Option<f32> {
    if probes.is_empty() {
        return None;
    }
    let device: burn::tensor::Device<InferenceBackend> = Default::default();

    let mut sum_xy = 0.0_f64;
    let mut sum_xx = 0.0_f64;

    for pos in probes {
        let features = encode_position(pos);
        debug_assert_eq!(features.len(), INPUT_DIM);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        let raw: f32 = out.into_data().to_vec::<f32>().ok()
            .and_then(|v| v.first().copied())
            .unwrap_or(f32::NAN);
        if !raw.is_finite() { continue; }

        let cp = heuristic.evaluate(pos);
        // The heuristic returns ±MATE_SCORE on terminal positions; those
        // dominate the fit and aren't what we're trying to scale (the NN
        // never sees them - terminals short-circuit in `NnEvaluator`).
        // Drop them.
        if cp.unsigned_abs() > 100_000 { continue; }

        let x = raw as f64;
        let y = cp as f64;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    if sum_xx <= 0.0 { return None; }
    let k = (sum_xy / sum_xx) as f32;
    if !k.is_finite() || k == 0.0 { return None; }
    Some(k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::TrainingBackend;
    use crate::model::MlpConfig;
    use crate::train::into_inference;
    use core_engine::search::evaluator::HeuristicEvaluator;

    #[test]
    fn empty_probes_returns_none() {
        let device = Default::default();
        let train_model: Mlp<TrainingBackend> = MlpConfig::new().init(&device);
        let model = into_inference(train_model);
        let h = HeuristicEvaluator;
        assert!(calibrate_rater(&model, &h, &[]).is_none());
    }

    #[test]
    fn single_setup_position_produces_some_value_or_none() {
        // A fresh untrained rater might output anything; we just verify the
        // function doesn't panic and returns a finite scale or a clean None.
        let device = Default::default();
        let train_model: Mlp<TrainingBackend> = MlpConfig::new().init(&device);
        let model = into_inference(train_model);
        let h = HeuristicEvaluator;
        let probes = vec![Position::setup_stack_m()];
        match calibrate_rater(&model, &h, &probes) {
            Some(k) => assert!(k.is_finite() && k != 0.0,
                "fitted scale must be finite and non-zero, got {}", k),
            None => {
                // Acceptable - untrained rater might output exactly 0 here.
            }
        }
    }

    #[test]
    fn slope_only_fit_recovers_known_scale_on_synthetic_data() {
        // Test the math directly: if NN raw = y / k, then slope-only OLS
        // recovers k exactly (up to floating-point noise).
        //
        // We construct a synthetic two-evaluator scenario where the
        // "heuristic" is just `target_k * forward_raw` for some fixed
        // target_k. The fit should find target_k.
        //
        // Implementation note: we can't easily mock the NN model output, but
        // we *can* test the math by inlining the OLS:
        let xs = [0.1_f32, 0.3, -0.2, 0.5, -0.4];
        let target_k = 2500.0_f32;
        let ys: Vec<f32> = xs.iter().map(|x| x * target_k).collect();

        let sum_xy: f64 = xs.iter().zip(ys.iter())
            .map(|(&x, &y)| x as f64 * y as f64).sum();
        let sum_xx: f64 = xs.iter().map(|&x| (x as f64) * (x as f64)).sum();
        let k = (sum_xy / sum_xx) as f32;
        assert!((k - target_k).abs() < 1e-2,
            "slope-only OLS must recover synthetic scale: got {} expected {}",
            k, target_k);
    }
}
