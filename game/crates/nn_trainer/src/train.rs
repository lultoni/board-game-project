//! Training loop: corpus → trained `Mlp` weights.
//!
//! The pipeline is:
//!   1. `batch_to_tensors` - fold a slice of `LabelledPosition` through
//!      `encode_position` into `(inputs: Tensor<B,2>, labels: Tensor<B,1>)`.
//!   2. `train_step` - one forward + MSE + backward + optimiser step. Returns
//!      the updated model and the scalar loss for the batch.
//!   3. `train` - minibatch loop over a fixed corpus for N epochs.
//!
//! The autodiff backend (`B: AutodiffBackend`) is what makes `loss.backward()`
//! work. At inference time we use the bare backend (`NdArray<f32>`) - the
//! autograd graph is training-only overhead.
//!
//! Loss: mean squared error between the forward output (unbounded scalar) and
//! the game-outcome label (in {-1, +1}). Plain MSE is the right starting
//! choice; once the gauntlet has data we can experiment with Huber etc.

use crate::encoding::{encode_position, INPUT_DIM};
use crate::model::Mlp;
use crate::selfplay::LabelledPosition;
use crate::sparse::{encode_sparse, NUM_FEATURES};

use burn::module::AutodiffModule;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{ElementConversion, Tensor, TensorData};

/// Hyperparameters for one training run.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrainingConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub epochs: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        // Conservative starting point - the gauntlet picks the winner across
        // sweeps, so these defaults only need to "not diverge."
        Self { learning_rate: 1e-3, batch_size: 64, epochs: 5 }
    }
}

/// Encode a batch of labelled positions into the input/label tensors the
/// model consumes. `inputs` is `(N, INPUT_DIM)`; `labels` is `(N,)`.
pub fn batch_to_tensors<B: AutodiffBackend>(
    examples: &[LabelledPosition],
    device: &B::Device,
) -> (Tensor<B, 2>, Tensor<B, 1>) {
    let n = examples.len();
    let mut features = Vec::with_capacity(n * INPUT_DIM);
    let mut labels = Vec::with_capacity(n);
    for ex in examples {
        features.extend_from_slice(&encode_position(&ex.position));
        labels.push(ex.label);
    }
    let inputs = Tensor::from_data(TensorData::new(features, [n, INPUT_DIM]), device);
    let labels = Tensor::from_data(TensorData::new(labels, [n]), device);
    (inputs, labels)
}

/// A position + a **scalar centipawn** label (the hand-crafted eval), for the
/// Phase-0 supervised bootstrap. Distinct from `LabelledPosition` (outcome
/// labels in {-1,+1}); the bootstrap regresses to `evaluate_scalar` centipawns.
#[derive(Clone, Debug)]
pub struct ScalarLabelled {
    pub position: core_engine::state::Position,
    /// Target centipawns (P1-POV), pre-normalization.
    pub label_cp: f32,
}

/// Scatter a set of active sparse feature indices into a dense f32 row of width
/// `NUM_FEATURES` (zeroing first). Training-only: burn's `Linear` needs a dense
/// tensor. Inference never scatters - it uses the accumulator.
pub fn scatter_dense(active: &[u32], row: &mut [f32]) {
    debug_assert_eq!(row.len(), NUM_FEATURES);
    for v in row.iter_mut() {
        *v = 0.0;
    }
    for &f in active {
        row[f as usize] = 1.0;
    }
}

/// Encode a batch of scalar-labelled positions into the sparse-input tensors:
/// `inputs` is `(N, NUM_FEATURES)` (dense-scattered sparse features), `labels`
/// is `(N,)` in **normalized** units (`label_cp / label_divisor`).
pub fn sparse_batch_to_tensors<B: AutodiffBackend>(
    examples: &[ScalarLabelled],
    label_divisor: f32,
    device: &B::Device,
) -> (Tensor<B, 2>, Tensor<B, 1>) {
    let n = examples.len();
    let mut features = vec![0.0f32; n * NUM_FEATURES];
    let mut labels = Vec::with_capacity(n);
    let mut active = Vec::with_capacity(64 * 6 + 6);
    for (i, ex) in examples.iter().enumerate() {
        encode_sparse(&ex.position, &mut active);
        let row = &mut features[i * NUM_FEATURES..(i + 1) * NUM_FEATURES];
        scatter_dense(&active, row);
        labels.push(ex.label_cp / label_divisor);
    }
    let inputs = Tensor::from_data(TensorData::new(features, [n, NUM_FEATURES]), device);
    let labels = Tensor::from_data(TensorData::new(labels, [n]), device);
    (inputs, labels)
}

/// Run one forward + MSE + backward + optimiser step. Returns the updated
/// model and the scalar MSE for telemetry.
pub fn train_step<B: AutodiffBackend, O: Optimizer<Mlp<B>, B>>(
    model: Mlp<B>,
    optimizer: &mut O,
    inputs: Tensor<B, 2>,
    labels: Tensor<B, 1>,
    learning_rate: f64,
) -> (Mlp<B>, f32) {
    // Forward: (batch, 1) → squeeze to (batch,) so it lines up with labels.
    let preds = model.forward(inputs).squeeze::<1>();
    let diff = preds - labels;
    let loss = diff.clone().powi_scalar(2).mean();

    let loss_scalar = loss.clone().into_scalar().elem::<f32>();

    let grads = loss.backward();
    let grads_params = GradientsParams::from_grads(grads, &model);
    let model = optimizer.step(learning_rate, model, grads_params);
    (model, loss_scalar)
}

/// Train `model` on `corpus` per `config`. Returns the trained model and the
/// per-epoch mean loss (one entry per epoch).
///
/// The corpus is consumed in fixed `batch_size` chunks; the trailing partial
/// batch is dropped (one short batch per epoch isn't worth the bookkeeping).
/// Batches are taken in corpus order - there is no shuffling. Self-play
/// already injects variance via random loadouts; in-epoch shuffle is a future
/// refinement once we see a need for it.
pub fn train<B: AutodiffBackend>(
    mut model: Mlp<B>,
    corpus: &[LabelledPosition],
    config: &TrainingConfig,
    device: &B::Device,
) -> (Mlp<B>, Vec<f32>) {
    let mut optimizer = AdamConfig::new().init();
    let mut epoch_losses = Vec::with_capacity(config.epochs);

    for _epoch in 0..config.epochs {
        let mut sum = 0.0_f32;
        let mut n_batches = 0usize;
        for chunk in corpus.chunks_exact(config.batch_size) {
            let (inputs, labels) = batch_to_tensors::<B>(chunk, device);
            let (next_model, loss) =
                train_step(model, &mut optimizer, inputs, labels, config.learning_rate);
            model = next_model;
            sum += loss;
            n_batches += 1;
        }
        let mean = if n_batches > 0 { sum / n_batches as f32 } else { f32::NAN };
        epoch_losses.push(mean);
    }

    (model, epoch_losses)
}

/// Strip the autograd graph from a trained model so it can be used at
/// inference time with the bare (non-Autodiff) backend.
pub fn into_inference<B: AutodiffBackend>(model: Mlp<B>) -> Mlp<B::InnerBackend> {
    model.valid()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{InferenceBackend as Inf, TrainingBackend as B};
    use crate::batch::generate_corpus;
    use crate::model::MlpConfig;
    use core_engine::search::evaluator::HeuristicEvaluator;

    #[test]
    fn batch_tensor_shapes() {
        let corpus = generate_corpus(2, 7, &HeuristicEvaluator, &HeuristicEvaluator, 2);
        assert!(!corpus.is_empty());
        let device = Default::default();
        let (inputs, labels) = batch_to_tensors::<B>(&corpus, &device);
        assert_eq!(inputs.dims(), [corpus.len(), INPUT_DIM]);
        assert_eq!(labels.dims(), [corpus.len()]);
    }

    #[test]
    fn training_reduces_loss_on_small_corpus() {
        // End-to-end gradient-flow check: train for a few epochs and assert
        // the final epoch loss is below the first. The corpus is tiny so the
        // model just memorises it - that's the point of a smoke test, the
        // real generalisation check is the gauntlet later.
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);

        let corpus = generate_corpus(4, 11, &HeuristicEvaluator, &HeuristicEvaluator, 2);
        assert!(corpus.len() >= 8,
            "need enough examples to form at least one batch; got {}", corpus.len());

        let config = TrainingConfig {
            learning_rate: 1e-3,
            batch_size: 8,
            epochs: 20,
        };

        let (_trained, losses) = train(model, &corpus, &config, &device);
        assert_eq!(losses.len(), config.epochs);
        for &l in &losses {
            assert!(l.is_finite(), "epoch loss must be finite, got {l}");
        }
        let first = losses[0];
        let last = *losses.last().unwrap();
        assert!(last < first,
            "training did not reduce loss: first={first} last={last}");
    }

    #[test]
    fn into_inference_strips_autograd() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);
        let inference_model: Mlp<Inf> = into_inference(model);

        // Forward pass on the inference model must produce a finite scalar.
        let zeros = vec![0.0_f32; INPUT_DIM];
        let data = TensorData::new(zeros, [1, INPUT_DIM]);
        let input: Tensor<Inf, 2> = Tensor::from_data(data, &device);
        let out = inference_model.forward(input);
        let v: Vec<f32> = out.into_data().to_vec().unwrap();
        assert_eq!(v.len(), 1);
        assert!(v[0].is_finite());
    }
}
