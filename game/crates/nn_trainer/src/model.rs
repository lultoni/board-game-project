//! Configurable MLP for the position rater.
//!
//! Topology is a startup config (`MlpConfig::hidden_sizes`), not a hard-coded
//! constant. Each training run picks its shape; the gauntlet decides which
//! shape wins. Per ADR-007, autograd + optimisers come from `burn`; we own
//! only the module definition and the input encoding.
//!
//! ## Default shape (v1)
//!
//! `input → 256 → 64 → 32 → 1`, ReLU between hidden layers, no activation on
//! the output (the rater is a scalar in P1-POV i32/f32 units, can be negative).
//!
//! ## Forward pass
//!
//! `forward(x: Tensor<B, 2>) -> Tensor<B, 2>` — batched. Shape
//! `(batch, INPUT_DIM)` → `(batch, 1)`. Inference path uses batch=1.

use burn::{
    config::Config,
    module::Module,
    nn::{Linear, LinearConfig, Relu},
    tensor::{backend::Backend, Tensor},
};

use crate::encoding::INPUT_DIM;

/// Configuration for an MLP rater. `hidden_sizes` is the list of hidden-layer
/// widths; the final layer always maps to a single scalar.
#[derive(Config, Debug)]
pub struct MlpConfig {
    /// Defaults to `INPUT_DIM`. Overridable for ablation experiments (e.g.
    /// stripped-down encoders), but every production rater uses INPUT_DIM.
    #[config(default = "INPUT_DIM")]
    pub input_dim: usize,

    /// Hidden-layer widths, in order. Default is the v1 shape
    /// `[256, 64, 32]` per session-37 decision.
    #[config(default = "vec![256, 64, 32]")]
    pub hidden_sizes: Vec<usize>,
}

/// Multi-layer perceptron with N hidden layers + ReLU + scalar output.
///
/// The hidden-layer count is encoded by the `Vec<Linear>` length, so the
/// same struct serialises cleanly for any topology. Burn's `Module` derive
/// handles parameter enumeration, save/load, and autograd through the Vec.
#[derive(Module, Debug)]
pub struct Mlp<B: Backend> {
    /// All linear layers, including the final scalar projection. Length is
    /// `hidden_sizes.len() + 1`.
    layers: Vec<Linear<B>>,
    activation: Relu,
}

impl MlpConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> Mlp<B> {
        assert!(!self.hidden_sizes.is_empty(),
            "MlpConfig::hidden_sizes must have at least one hidden layer");

        let mut layers = Vec::with_capacity(self.hidden_sizes.len() + 1);
        let mut prev = self.input_dim;
        for &h in &self.hidden_sizes {
            layers.push(LinearConfig::new(prev, h).init(device));
            prev = h;
        }
        // Output: scalar.
        layers.push(LinearConfig::new(prev, 1).init(device));

        Mlp { layers, activation: Relu::new() }
    }
}

impl<B: Backend> Mlp<B> {
    /// `(batch, input_dim)` → `(batch, 1)`. ReLU between hidden layers,
    /// linear output.
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let mut x = x;
        let last_idx = self.layers.len() - 1;
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(x);
            if i != last_idx {
                x = self.activation.forward(x);
            }
        }
        x
    }

    /// Per-layer summary statistics. One `LayerStats` per `Linear`, in
    /// forward order — last entry is the scalar output projection. Used by
    /// the Training Observatory's Network Inspector to surface mean/std/
    /// min/max/NaN-count per layer without leaking raw weights through the
    /// Tauri boundary.
    pub fn weight_stats(&self) -> Vec<LayerStats> {
        self.layers
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                let weight: Vec<f32> = layer.weight.val().into_data().to_vec().unwrap_or_default();
                let bias: Vec<f32> = layer
                    .bias
                    .as_ref()
                    .map(|b| b.val().into_data().to_vec().unwrap_or_default())
                    .unwrap_or_default();
                let mut all = weight;
                all.extend(bias);
                LayerStats::from_values(format!("linear_{i}"), &all)
            })
            .collect()
    }
}

/// Summary statistics for one parameter tensor (concatenated weight + bias
/// for a `Linear` layer). Plain `f32`/`u32` so it can be serialised without
/// dragging burn types through the Tauri boundary.
#[derive(Clone, Debug)]
pub struct LayerStats {
    pub layer: String,
    pub mean: f32,
    pub std: f32,
    pub min: f32,
    pub max: f32,
    pub nan_count: u32,
}

impl LayerStats {
    pub fn from_values(layer: String, values: &[f32]) -> Self {
        if values.is_empty() {
            return Self { layer, mean: 0.0, std: 0.0, min: 0.0, max: 0.0, nan_count: 0 };
        }
        let mut nan_count: u32 = 0;
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        let mut sum = 0.0_f64;
        let mut count = 0_u64;
        for &v in values {
            if v.is_nan() {
                nan_count += 1;
                continue;
            }
            if v < min { min = v; }
            if v > max { max = v; }
            sum += v as f64;
            count += 1;
        }
        if count == 0 {
            return Self { layer, mean: 0.0, std: 0.0, min: 0.0, max: 0.0, nan_count };
        }
        let mean = (sum / count as f64) as f32;
        let mut var_sum = 0.0_f64;
        for &v in values {
            if v.is_nan() { continue; }
            let d = (v - mean) as f64;
            var_sum += d * d;
        }
        let std = (var_sum / count as f64).sqrt() as f32;
        Self { layer, mean, std, min, max, nan_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;
    use burn::tensor::TensorData;

    type B = NdArray<f32>;

    #[test]
    fn default_config_has_v1_shape() {
        let cfg = MlpConfig::new();
        assert_eq!(cfg.input_dim, INPUT_DIM);
        assert_eq!(cfg.hidden_sizes, vec![256, 64, 32]);
    }

    #[test]
    fn forward_pass_on_zero_input_returns_finite() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);

        let zeros = vec![0.0_f32; INPUT_DIM];
        let data = TensorData::new(zeros, [1, INPUT_DIM]);
        let input: Tensor<B, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);

        let v: Vec<f32> = out.into_data().to_vec().unwrap();
        assert_eq!(v.len(), 1);
        assert!(v[0].is_finite(), "expected finite output, got {}", v[0]);
    }

    #[test]
    fn custom_topology_initialises() {
        let device = Default::default();
        let cfg = MlpConfig::new()
            .with_input_dim(10)
            .with_hidden_sizes(vec![4]);
        let model: Mlp<B> = cfg.init(&device);
        // 1 hidden + 1 output = 2 layers.
        assert_eq!(model.layers.len(), 2);

        let data = TensorData::new(vec![0.0_f32; 10], [1, 10]);
        let input: Tensor<B, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        assert_eq!(out.dims(), [1, 1]);
    }

    #[test]
    fn weight_stats_returns_one_entry_per_layer() {
        let device = Default::default();
        let cfg = MlpConfig::new()
            .with_input_dim(10)
            .with_hidden_sizes(vec![4, 3]);
        let model: Mlp<B> = cfg.init(&device);

        let stats = model.weight_stats();
        // hidden_sizes.len() + 1 = 3 layers.
        assert_eq!(stats.len(), 3);
        for s in &stats {
            assert!(s.layer.starts_with("linear_"));
            assert!(s.min.is_finite());
            assert!(s.max.is_finite());
            assert!(s.mean.is_finite());
            assert!(s.std >= 0.0);
            assert_eq!(s.nan_count, 0);
        }
    }

    #[test]
    fn layer_stats_handles_nan_and_empty() {
        let s = LayerStats::from_values("empty".into(), &[]);
        assert_eq!(s.mean, 0.0);
        assert_eq!(s.std, 0.0);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 0.0);
        assert_eq!(s.nan_count, 0);

        let s = LayerStats::from_values("mixed".into(), &[1.0, 2.0, f32::NAN, 3.0]);
        // Mean of finite values = 2.0; std non-negative.
        assert!((s.mean - 2.0).abs() < 1e-6);
        assert_eq!(s.nan_count, 1);
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 3.0);
    }
}
