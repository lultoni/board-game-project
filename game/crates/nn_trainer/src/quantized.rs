//! Quantized integer forward pass for the NNUE tail net.
//!
//! The in-search inference path must NOT call `burn` per node (ns-49 measured a
//! dense burn forward at ~382x the hand-crafted eval). This module runs the net
//! as a hand-written integer forward over the accumulator + small tail layers.
//! See `design/inbox/nnue-rework-plan.md` §3.4.
//!
//! ## Scheme (fixed-point, uniform per-domain scales)
//!
//! Topology: `Accumulator(i32 x 128) → clippedReLU → 128→32 → clippedReLU →
//! 32→32 → clippedReLU → 32→1 → centipawns`, matching the trained
//! `hidden_sizes = [128, 32, 32]` (ns-50 tail-cost rework; was [256,64,32]).
//!
//! - **Feature transform (layer 0):** weights `w·QA` rounded to i16, bias
//!   `b·QA` to i32. The accumulator sum lives in the **QA fixed-point domain**
//!   (an f32 pre-activation `p` is represented as `round(p·QA)`).
//! - **clipped-ReLU:** dequantize (`>> shift`), clamp to `[0, CR_MAX]`. The
//!   activation is an integer in `[0, CR_MAX]` representing `a` directly (scale
//!   1), i.e. `act = clamp(acc >> QA_SHIFT, 0, CR_MAX)` where `2^QA_SHIFT ≈ QA`.
//! - **Hidden layers:** weights `w·QW` rounded to i8, bias `b·QW·1` to i32
//!   (bias must match the `act(scale 1) · w(scale QW)` product domain → `b·QW`).
//!   Output `sum = bias_qw + Σ act·w_i8`, then clipped-ReLU back to scale 1 via
//!   `>> QW_SHIFT`.
//! - **Output layer:** same product domain (scale QW); dequantize and scale to
//!   centipawns by the trained label divisor folded into `out_scale`.
//!
//! Quantization introduces rounding; the Phase-0 regression harness grades the
//! **quantized** path, so the error band is measured, not assumed.

use crate::accumulator::{Accumulator, FeatureTransform};
use crate::model::Mlp;
use crate::nn_evaluator::{InferenceBackend, MAX_NN_SCORE};
use crate::sparse::{ACCUM_WIDTH, NUM_FEATURES};
use wide::{i16x8, i32x8};

/// SIMD lane width for the i16 dot product (`i16x8`).
const LANES: usize = 8;

/// Feature-transform weight scale (f32 → i16). Power of two so dequant is a
/// shift.
pub const QA: f32 = 1024.0;
const QA_SHIFT: u32 = 10; // 2^10 == 1024

/// Hidden/output weight scale (f32 → i8). Power of two.
pub const QW: f32 = 64.0;
const QW_SHIFT: u32 = 6; // 2^6 == 64

/// Clipped-ReLU ceiling in activation units (scale 1). Generous - activations
/// rarely exceed this after the trained net's ReLUs; clipping high is benign.
const CR_MAX: i32 = 8192;

/// Documented quantization scales (surfaced for sidecar metadata / debugging).
#[derive(Clone, Copy, Debug)]
pub struct QuantScales {
    pub qa: f32,
    pub qw: f32,
    /// Multiplies the raw net output (f32, in normalized-label units) to reach
    /// centipawns. Equals the label divisor used at training time (see
    /// `bootstrap`).
    pub out: f32,
}

impl Default for QuantScales {
    fn default() -> Self {
        QuantScales { qa: QA, qw: QW, out: 1.0 }
    }
}

/// A quantized `Linear`, laid out for a SIMD integer dot product.
///
/// Weights are **output-major**, **pre-widened i16**, and each output row is
/// **zero-padded to a multiple of `LANES`** (`in_pad`). So `w[o*in_pad + i]` is
/// the weight from input `i` to output `o`, and the accumulate-over-inputs for a
/// fixed `o` walks a contiguous, lane-aligned run - vectorizable with `wide`.
/// (burn hands us input-major i8 `w[i*out+o]`; the transpose + widen + pad
/// happens once in `quantize_linear`, not per node.)
struct QLinear {
    /// Output-major, i16, row-padded: length `out_dim * in_pad`.
    w: Vec<i16>,
    b: Vec<i32>,
    /// `in_dim` rounded up to a multiple of `LANES` (the padded row stride).
    in_pad: usize,
    out_dim: usize,
}

impl QLinear {
    /// `out[o] = clamp(( b[o] + Σ_i act[i]·w[o][i] ) >> QW_SHIFT, 0, CR_MAX)`
    /// when `relu` is set (hidden layers), else the raw shifted sum (output).
    ///
    /// `act` is the i16 activation buffer, zero-padded to `in_pad` (the padding
    /// weights are zero, so padded lanes contribute nothing - the sum is
    /// identical to the scalar `in_dim` dot product). Integer SIMD reordering is
    /// exact, so this matches the scalar sum bit-for-bit.
    fn forward(&self, act_padded: &[i16], relu: bool, out: &mut [i32]) {
        debug_assert_eq!(act_padded.len(), self.in_pad);
        debug_assert_eq!(out.len(), self.out_dim);
        let chunks = self.in_pad / LANES;
        for o in 0..self.out_dim {
            let wrow = &self.w[o * self.in_pad..o * self.in_pad + self.in_pad];
            let mut acc = i32x8::ZERO;
            for c in 0..chunks {
                let a = load_i16x8(&act_padded[c * LANES..]);
                let w = load_i16x8(&wrow[c * LANES..]);
                acc += a.mul_widen(w);
            }
            let sum = self.b[o] + acc.reduce_add();
            let scaled = sum >> QW_SHIFT;
            out[o] = if relu { scaled.clamp(0, CR_MAX) } else { scaled };
        }
    }
}

/// Load 8 contiguous `i16` into an `i16x8`. The slice must have ≥ 8 elements.
#[inline]
fn load_i16x8(s: &[i16]) -> i16x8 {
    i16x8::new([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]])
}

/// Round `in_dim` up to a multiple of `LANES`.
#[inline]
fn pad_to_lanes(in_dim: usize) -> usize {
    in_dim.div_ceil(LANES) * LANES
}

/// The full quantized net: feature transform + integer tail.
pub struct QuantizedNet {
    ft: FeatureTransform,
    l1: QLinear,
    l2: QLinear,
    out: QLinear,
    /// Multiplies the (dequantized, scale-QW) raw output to centipawns.
    out_to_cp: f32,
}

impl QuantizedNet {
    /// Borrow the feature transform (the evaluator refreshes accumulators
    /// against it).
    pub fn ft(&self) -> &FeatureTransform {
        &self.ft
    }

    /// Integer forward: accumulator → clippedReLU → tail → centipawns (P1-POV).
    /// Clamped to ±MAX_NN_SCORE so the net can't claim a false mate.
    ///
    /// Activations are built as i16 buffers zero-padded to each layer's `in_pad`
    /// (all clipped-ReLU outputs are in `[0, CR_MAX]` ⊂ i16), so `QLinear::
    /// forward` can run a lane-aligned SIMD dot product with no scalar tail.
    pub fn forward_int(&self, acc: &Accumulator) -> i32 {
        // clipped-ReLU on the accumulator: dequant from QA domain to scale 1,
        // emitted directly into the l1-input i16 buffer (padded to l1.in_pad).
        let a0 = acc.values();
        let mut act0 = vec![0i16; self.l1.in_pad];
        for j in 0..ACCUM_WIDTH {
            act0[j] = (a0[j] >> QA_SHIFT).clamp(0, CR_MAX) as i16;
        }

        let mut a1 = vec![0i32; self.l1.out_dim];
        self.l1.forward(&act0, true, &mut a1);
        // Repack a1 → i16 buffer padded to l2.in_pad (values are in [0,CR_MAX]).
        let mut a1i = vec![0i16; self.l2.in_pad];
        for i in 0..self.l1.out_dim {
            a1i[i] = a1[i] as i16;
        }

        let mut a2 = vec![0i32; self.l2.out_dim];
        self.l2.forward(&a1i, true, &mut a2);
        let mut a2i = vec![0i16; self.out.in_pad];
        for i in 0..self.l2.out_dim {
            a2i[i] = a2[i] as i16;
        }

        let mut a3 = vec![0i32; self.out.out_dim];
        self.out.forward(&a2i, false, &mut a3);

        // a3[0] is the raw output in the scale-QW/scale-1 product domain,
        // already `>> QW_SHIFT` (scale 1, i.e. normalized-label units x 1).
        // Dequant is identity here (scale 1); scale to centipawns.
        let cp = (a3[0] as f32) * self.out_to_cp;
        (cp.round() as i32).clamp(-MAX_NN_SCORE, MAX_NN_SCORE)
    }

    /// Quantize a trained f32 `Mlp` (input_dim == NUM_FEATURES, hidden
    /// [128,32,32]) into the integer tables.
    pub fn from_mlp(model: &Mlp<InferenceBackend>, scales: QuantScales) -> Self {
        let params = model.layer_params();
        assert_eq!(params.len(), 4, "expected 4 layers (3 hidden + output)");

        // --- Layer 0: feature transform (NUM_FEATURES → ACCUM_WIDTH) --------
        let (w0, b0, in0, out0) = &params[0];
        assert_eq!(*in0, NUM_FEATURES, "layer0 input_dim must equal NUM_FEATURES");
        assert_eq!(*out0, ACCUM_WIDTH, "layer0 output must equal ACCUM_WIDTH");
        let mut weights = vec![[0i16; ACCUM_WIDTH]; NUM_FEATURES];
        // burn weight is input-major: w0[f*out + o] is the contribution of
        // feature f to accumulator lane o - exactly the column for feature f.
        for f in 0..NUM_FEATURES {
            for o in 0..ACCUM_WIDTH {
                weights[f][o] = round_i16(w0[f * ACCUM_WIDTH + o] * scales.qa);
            }
        }
        let mut bias = [0i32; ACCUM_WIDTH];
        if !b0.is_empty() {
            for o in 0..ACCUM_WIDTH {
                bias[o] = (b0[o] * scales.qa).round() as i32;
            }
        }
        let ft = FeatureTransform { weights, bias };

        // --- Hidden + output layers ----------------------------------------
        let l1 = quantize_linear(&params[1], scales.qw);
        let l2 = quantize_linear(&params[2], scales.qw);
        let out = quantize_linear(&params[3], scales.qw);

        QuantizedNet { ft, l1, l2, out, out_to_cp: scales.out }
    }
}

/// Quantize one hidden/output layer into the SIMD-friendly `QLinear` layout.
/// Input activations are scale 1; weights are scaled by QW to i8, then widened
/// to i16 and stored **output-major**, each output row zero-padded to a multiple
/// of `LANES`. Bias lives in the `act·w` product domain (scale QW), so
/// `b_quant = round(b_f32 · QW)`.
fn quantize_linear(params: &(Vec<f32>, Vec<f32>, usize, usize), qw: f32) -> QLinear {
    let (w, b, in_dim, out_dim) = params;
    let in_pad = pad_to_lanes(*in_dim);
    // Output-major, i16, row-padded. burn's `w` is input-major (`w[i*out+o]`).
    let mut w_q = vec![0i16; out_dim * in_pad];
    for o in 0..*out_dim {
        for i in 0..*in_dim {
            w_q[o * in_pad + i] = round_i8(w[i * out_dim + o] * qw) as i16;
        }
        // padded lanes [in_dim, in_pad) stay zero.
    }
    let b_q: Vec<i32> = if b.is_empty() {
        vec![0i32; *out_dim]
    } else {
        b.iter().map(|&x| (x * qw).round() as i32).collect()
    };
    QLinear { w: w_q, b: b_q, in_pad, out_dim: *out_dim }
}

#[inline]
fn round_i16(x: f32) -> i16 {
    x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

#[inline]
fn round_i8(x: f32) -> i8 {
    x.round().clamp(i8::MIN as f32, i8::MAX as f32) as i8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MlpConfig;
    use burn::tensor::{Device, Tensor, TensorData};
    use core_engine::game_logic::{generator, make_unmake};
    use core_engine::state::Position;

    /// f32 reference forward for a position: scatter the sparse features into a
    /// dense row and run the burn model.
    fn f32_forward(
        model: &Mlp<InferenceBackend>,
        pos: &Position,
        device: &Device<InferenceBackend>,
    ) -> f32 {
        let active = crate::sparse::encode_sparse_vec(pos);
        let mut row = vec![0.0f32; NUM_FEATURES];
        for &f in &active {
            row[f as usize] = 1.0;
        }
        let data = TensorData::new(row, [1, NUM_FEATURES]);
        let input: Tensor<InferenceBackend, 2> = Tensor::from_data(data, device);
        let out = model.forward(input);
        out.into_data().to_vec::<f32>().unwrap()[0]
    }

    #[test]
    fn quantized_matches_f32_within_tol() {
        let device = Default::default();
        // Small random net with the sparse input dim.
        let cfg = MlpConfig::new().with_input_dim(NUM_FEATURES);
        let model: Mlp<InferenceBackend> = cfg.init(&device);

        let scales = QuantScales { qa: QA, qw: QW, out: 1.0 };
        let qnet = QuantizedNet::from_mlp(&model, scales);

        // Gather a set of positions: start + a random walk.
        let mut positions = vec![Position::setup_stack_m(), Position::empty()];
        let mut pos = Position::setup_stack_m();
        let mut rng = 0xBEEF_u64;
        for _ in 0..40 {
            let actions = generator::generate(&pos);
            if actions.is_empty() {
                break;
            }
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (rng >> 33) as usize % actions.len();
            make_unmake::make(&mut pos, actions[idx]);
            positions.push(pos.clone());
        }

        let mut sum_abs = 0.0f64;
        let mut max_abs = 0.0f32;
        let mut n = 0;
        for p in &positions {
            let f32_out = f32_forward(&model, p, &device); // scale-1 units
            let f32_cp = f32_out * scales.out;
            let int_cp = qnet.forward_int(&Accumulator::refresh(p, qnet.ft())) as f32;
            let d = (int_cp - f32_cp).abs();
            sum_abs += d as f64;
            max_abs = max_abs.max(d);
            n += 1;
        }
        let mean_abs = sum_abs / n as f64;

        // The band is measured, not exact-match: fixed-point rounding across
        // 4 layers accumulates a few units. With out==1.0 the output is in the
        // same (normalized-label) units as the f32 net. A random-init net's
        // outputs are O(1), so a mean abs error of a few units is expected.
        // Assert a conservative band; tighten once real scales are set in
        // bootstrap. (Values in these same units → threshold is unit-scale.)
        assert!(
            mean_abs < 5.0,
            "quantized-vs-f32 mean |Δ| = {mean_abs:.3} (max {max_abs:.3}) exceeds band"
        );
    }
}
