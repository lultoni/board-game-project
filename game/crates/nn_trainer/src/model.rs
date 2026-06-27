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
}
