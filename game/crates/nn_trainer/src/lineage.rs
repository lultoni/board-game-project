//! Perturbation injection + parallel training lineages.
//!
//! Plan §4 (perturbation injection) + §6 (population-based selection):
//!
//! - `perturb_model` — add seeded Gaussian noise to every weight tensor.
//!   Same seed + same std-dev + same source model ⇒ same perturbed model.
//! - `Lineage` — one (model, training-config, rng-seed) bundle. Knows how to
//!   take a *training burst* (N gradient steps over a corpus) and how to
//!   *fork* into a perturbed sibling.
//! - `train_lineages` — top-level driver: K lineages, R rounds. Each round
//!   every lineage does a training burst in parallel. After the burst, each
//!   lineage forks a perturbed copy, trains *that* for a shorter burst, and
//!   keeps whichever has lower validation loss (the "perturb and keep best"
//!   loop of §4). The set of lineages is returned for downstream gauntlet
//!   selection.
//!
//! The gauntlet itself (head-to-head play between lineage champions) is
//! plan §5 and lives in a future module. This module just builds the
//! candidate pool.
//!
//! ## Determinism
//!
//! Each lineage carries its own `ChaCha8Rng` seed. Within one process, a
//! `train_lineages` call with the same `(corpus, base_seed, config)` is
//! deterministic up to rayon's parallel order — but the *outputs* are
//! collected in lineage order, so the returned `Vec<Lineage>` is stable.
//!
//! Noise generation is external (`rand_chacha`) rather than via burn's
//! global-RNG `Tensor::random`, so independent lineages don't fight over a
//! shared backend RNG.

use crate::encoding::INPUT_DIM;
use crate::model::{Mlp, MlpConfig};
use crate::selfplay::LabelledPosition;
use crate::train::{batch_to_tensors, TrainingConfig};

use burn::module::{Module, ModuleMapper, Param};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{ElementConversion, Tensor, TensorData};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

/// ModuleMapper that adds pre-sampled Gaussian noise to every float param.
///
/// We sample noise into a CPU `Vec<f32>` from a `ChaCha8Rng` (so lineages
/// don't share an RNG) and build a fresh tensor per param. The mapper holds
/// the RNG and the std-dev; burn drives it through `model.map(&mut mapper)`.
struct GaussianNoiseMapper<B: AutodiffBackend> {
    rng: ChaCha8Rng,
    std_dev: f32,
    device: B::Device,
}

impl<B: AutodiffBackend> ModuleMapper<B> for GaussianNoiseMapper<B> {
    fn map_float<const D: usize>(
        &mut self,
        param: Param<Tensor<B, D>>,
    ) -> Param<Tensor<B, D>> {
        let dims = param.val().dims();
        let n: usize = dims.iter().product();
        let normal = Normal::new(0.0_f32, self.std_dev).expect("std_dev must be finite");
        let noise: Vec<f32> = (0..n).map(|_| normal.sample(&mut self.rng)).collect();
        let noise_tensor: Tensor<B, D> = Tensor::from_data(
            TensorData::new(noise, dims),
            &self.device,
        );
        param.map(|t| t + noise_tensor)
    }
}

/// Add seeded Gaussian noise (mean 0, std = `std_dev`) to every weight and
/// bias tensor in the model. The returned model has the same shape and
/// (param-id-wise) the same structure as the input.
///
/// Determinism: identical `(model, std_dev, seed)` produces an identical
/// perturbed model. Different seeds with the same model produce independent
/// perturbations — that's the parallel-lineage primitive.
///
/// **Lazy-init caveat:** burn `Param`s are lazily initialised on first
/// `consume`/`val`. If you clone a never-touched module and perturb both
/// copies, each clone will independently sample its initialisation from the
/// backend RNG and the *base* models will already differ before any noise
/// is added. Run a forward pass on the source model before cloning if you
/// need two perturbation runs to start from identical weights.
pub fn perturb_model<B: AutodiffBackend>(
    model: Mlp<B>,
    std_dev: f32,
    seed: u64,
    device: &B::Device,
) -> Mlp<B> {
    let mut mapper = GaussianNoiseMapper::<B> {
        rng: ChaCha8Rng::seed_from_u64(seed),
        std_dev,
        device: device.clone(),
    };
    model.map(&mut mapper)
}

/// One training lineage: a model + the seed that owns its randomness.
///
/// `id` is stable across rounds (so the caller can correlate logs); `seed`
/// advances every time we draw from it (perturbation, future shuffling).
pub struct Lineage<B: AutodiffBackend> {
    pub id: usize,
    pub model: Mlp<B>,
    pub seed: u64,
    /// Per-lineage loss history (mean loss per training round; one entry
    /// pushed per `train_burst` call).
    pub loss_history: Vec<f32>,
}

impl<B: AutodiffBackend> Lineage<B> {
    /// Fresh lineage from `MlpConfig` defaults. `id` and `seed` distinguish
    /// it from its siblings.
    pub fn new(id: usize, seed: u64, cfg: &MlpConfig, device: &B::Device) -> Self {
        let model: Mlp<B> = cfg.init(device);
        Self { id, model, seed, loss_history: Vec::new() }
    }

    /// Run one training burst: `steps` minibatch gradient steps over
    /// `corpus`, wrapping around if `corpus` is smaller than
    /// `steps * batch_size`. Returns mean loss across this burst and pushes
    /// it onto `loss_history`.
    pub fn train_burst(
        &mut self,
        corpus: &[LabelledPosition],
        steps: usize,
        config: &TrainingConfig,
        device: &B::Device,
    ) -> f32 {
        if corpus.is_empty() || steps == 0 {
            self.loss_history.push(f32::NAN);
            return f32::NAN;
        }

        let mut optimizer = AdamConfig::new().init();
        let bsz = config.batch_size.min(corpus.len());
        let mut sum = 0.0_f32;
        let mut n = 0usize;

        let mut model = std::mem::replace(&mut self.model, MlpConfig::new()
            .with_input_dim(INPUT_DIM)
            .init(device));

        for step in 0..steps {
            let start = (step * bsz) % corpus.len();
            let end = (start + bsz).min(corpus.len());
            let chunk = &corpus[start..end];
            if chunk.len() < bsz { continue; }  // skip partial wraparound

            let (inputs, labels) = batch_to_tensors::<B>(chunk, device);
            let preds = model.forward(inputs).squeeze::<1>();
            let diff = preds - labels;
            let loss = diff.clone().powi_scalar(2).mean();
            let loss_scalar = loss.clone().into_scalar().elem::<f32>();

            let grads = loss.backward();
            let grads_params = GradientsParams::from_grads(grads, &model);
            model = optimizer.step(config.learning_rate, model, grads_params);

            sum += loss_scalar;
            n += 1;
        }

        self.model = model;
        let mean = if n > 0 { sum / n as f32 } else { f32::NAN };
        self.loss_history.push(mean);
        mean
    }

    /// Produce a perturbed sibling — same id (still the same lineage), but
    /// the model has noise added and the seed is advanced.
    pub fn perturbed_clone(&mut self, std_dev: f32, device: &B::Device) -> Mlp<B>
    where
        Mlp<B>: Clone,
    {
        let perturb_seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15); // golden-ratio mix
        self.seed = self.seed.wrapping_add(1);
        perturb_model(self.model.clone(), std_dev, perturb_seed, device)
    }
}

/// Configuration for `train_lineages`. Conservative defaults; tuning happens
/// once we see real loss curves.
#[derive(Clone, Debug)]
pub struct LineageConfig {
    /// How many parallel lineages.
    pub n_lineages: usize,
    /// How many train-then-perturb rounds.
    pub n_rounds: usize,
    /// Gradient steps per training burst.
    pub steps_per_burst: usize,
    /// Gradient steps the perturbed candidate gets before we compare.
    pub steps_per_candidate: usize,
    /// Std-dev of the Gaussian noise added at each perturbation.
    pub perturb_std: f32,
    /// Underlying training hyperparameters.
    pub training: TrainingConfig,
}

impl Default for LineageConfig {
    fn default() -> Self {
        Self {
            n_lineages: 4,
            n_rounds: 5,
            steps_per_burst: 20,
            steps_per_candidate: 10,
            perturb_std: 0.05,
            training: TrainingConfig::default(),
        }
    }
}

/// Compute mean MSE of `model` on `corpus` (no gradient step, no parameter
/// update — just forward + loss). Used to decide whether a perturbed
/// candidate is better than the unperturbed lineage.
fn validation_loss<B: AutodiffBackend>(
    model: &Mlp<B>,
    corpus: &[LabelledPosition],
    batch_size: usize,
    device: &B::Device,
) -> f32 {
    if corpus.is_empty() { return f32::NAN; }
    let bsz = batch_size.min(corpus.len());
    let mut sum = 0.0_f32;
    let mut n = 0usize;
    for chunk in corpus.chunks(bsz) {
        if chunk.len() < bsz { continue; }
        let (inputs, labels) = batch_to_tensors::<B>(chunk, device);
        let preds = model.forward(inputs).squeeze::<1>();
        let diff = preds - labels;
        let loss = diff.powi_scalar(2).mean();
        sum += loss.into_scalar().elem::<f32>();
        n += 1;
    }
    if n > 0 { sum / n as f32 } else { f32::NAN }
}

/// Top-level driver: build `n_lineages`, run `n_rounds` of
/// (train-burst → perturb-and-keep-best), return the lineages.
///
/// Note on parallelism: burn's `Autodiff<NdArray<f32>>` backend isn't `Send`-
/// friendly in all configurations, so this driver is sequential across
/// lineages by default. Rayon-parallel lineages can be added once we've
/// confirmed the autodiff backend is thread-safe — the time-dominant work
/// is corpus generation anyway, which already parallelises elsewhere.
pub fn train_lineages<B: AutodiffBackend>(
    corpus: &[LabelledPosition],
    base_seed: u64,
    config: &LineageConfig,
    model_cfg: &MlpConfig,
    device: &B::Device,
) -> Vec<Lineage<B>>
where
    Mlp<B>: Clone,
{
    train_lineages_with_progress::<B, _>(
        corpus, base_seed, config, model_cfg, device, |_, _, _| {},
    )
}

/// Same as `train_lineages` but invokes `on_progress(lineage_idx, round_idx,
/// n_rounds)` after each (lineage, round) pair. Used by the orchestrator to
/// emit status heartbeats during the otherwise-silent training phase.
pub fn train_lineages_with_progress<B: AutodiffBackend, F>(
    corpus: &[LabelledPosition],
    base_seed: u64,
    config: &LineageConfig,
    model_cfg: &MlpConfig,
    device: &B::Device,
    mut on_progress: F,
) -> Vec<Lineage<B>>
where
    Mlp<B>: Clone,
    F: FnMut(usize /* lineage */, usize /* round */, usize /* n_rounds */),
{
    let mut lineages: Vec<Lineage<B>> = (0..config.n_lineages)
        .map(|i| {
            let seed = base_seed
                .wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            Lineage::new(i, seed, model_cfg, device)
        })
        .collect();

    for round in 0..config.n_rounds {
        for (idx, lin) in lineages.iter_mut().enumerate() {
            lin.train_burst(corpus, config.steps_per_burst, &config.training, device);

            let base_loss = validation_loss(
                &lin.model, corpus, config.training.batch_size, device,
            );

            let candidate = lin.perturbed_clone(config.perturb_std, device);
            // Wrap candidate in a temporary lineage to reuse train_burst.
            let mut cand_lineage = Lineage::<B> {
                id: lin.id,
                model: candidate,
                seed: lin.seed,
                loss_history: Vec::new(),
            };
            cand_lineage.train_burst(
                corpus, config.steps_per_candidate, &config.training, device,
            );
            let cand_loss = validation_loss(
                &cand_lineage.model, corpus, config.training.batch_size, device,
            );

            if cand_loss.is_finite() && cand_loss < base_loss {
                lin.model = cand_lineage.model;
                lin.seed = cand_lineage.seed;
            }

            on_progress(idx, round, config.n_rounds);
        }
    }

    lineages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::generate_corpus;
    use burn::backend::{Autodiff, NdArray};
    use core_engine::search::evaluator::HeuristicEvaluator;

    type B = Autodiff<NdArray<f32>>;

    #[test]
    fn perturbation_changes_outputs_but_preserves_shape() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);

        // Snapshot output on a fixed input.
        let probe_data = TensorData::new(vec![0.5_f32; INPUT_DIM], [1, INPUT_DIM]);
        let probe: Tensor<B, 2> = Tensor::from_data(probe_data.clone(), &device);
        let before: Vec<f32> = model.forward(probe.clone()).into_data().to_vec().unwrap();

        let perturbed = perturb_model(model, 0.1, 42, &device);

        let probe2: Tensor<B, 2> = Tensor::from_data(probe_data, &device);
        let after: Vec<f32> = perturbed.forward(probe2).into_data().to_vec().unwrap();

        assert_eq!(before.len(), after.len(), "output shape must survive perturbation");
        assert!(before[0].is_finite() && after[0].is_finite());
        assert!((before[0] - after[0]).abs() > 1e-6,
            "perturbation should change the output (before={}, after={})",
            before[0], after[0]);
    }

    #[test]
    fn perturbation_is_deterministic_from_seed() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let m1: Mlp<B> = cfg.init(&device);
        // Force lazy parameter initialization to materialise before the
        // clone — otherwise each clone independently lazily-initializes
        // from the backend RNG and the two source models diverge.
        let warmup = TensorData::new(vec![0.0_f32; INPUT_DIM], [1, INPUT_DIM]);
        let _ = m1.forward(Tensor::from_data(warmup, &device));
        let m2: Mlp<B> = m1.clone();

        let p1 = perturb_model(m1, 0.1, 7, &device);
        let p2 = perturb_model(m2, 0.1, 7, &device);

        let probe = TensorData::new(vec![0.3_f32; INPUT_DIM], [1, INPUT_DIM]);
        let t1: Tensor<B, 2> = Tensor::from_data(probe.clone(), &device);
        let t2: Tensor<B, 2> = Tensor::from_data(probe, &device);
        let o1: Vec<f32> = p1.forward(t1).into_data().to_vec().unwrap();
        let o2: Vec<f32> = p2.forward(t2).into_data().to_vec().unwrap();
        assert!((o1[0] - o2[0]).abs() < 1e-6,
            "same seed must produce same perturbed output (got {} vs {})",
            o1[0], o2[0]);
    }

    #[test]
    fn perturbation_different_seeds_diverge() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let m1: Mlp<B> = cfg.init(&device);
        let m2: Mlp<B> = m1.clone();

        let p1 = perturb_model(m1, 0.1, 1, &device);
        let p2 = perturb_model(m2, 0.1, 2, &device);

        let probe = TensorData::new(vec![0.7_f32; INPUT_DIM], [1, INPUT_DIM]);
        let t1: Tensor<B, 2> = Tensor::from_data(probe.clone(), &device);
        let t2: Tensor<B, 2> = Tensor::from_data(probe, &device);
        let o1: Vec<f32> = p1.forward(t1).into_data().to_vec().unwrap();
        let o2: Vec<f32> = p2.forward(t2).into_data().to_vec().unwrap();
        assert!((o1[0] - o2[0]).abs() > 1e-6,
            "different seeds should produce different perturbations");
    }

    #[test]
    fn train_lineages_produces_population() {
        let device = Default::default();
        let corpus = generate_corpus(2, 17, &HeuristicEvaluator, &HeuristicEvaluator, 2);
        // Need enough examples for at least one minibatch.
        assert!(corpus.len() >= 8);

        let cfg = LineageConfig {
            n_lineages: 2,
            n_rounds: 2,
            steps_per_burst: 3,
            steps_per_candidate: 2,
            perturb_std: 0.05,
            training: TrainingConfig {
                learning_rate: 1e-3,
                batch_size: 4,
                epochs: 1,
            },
        };
        let model_cfg = MlpConfig::new();

        let lineages = train_lineages::<B>(&corpus, 123, &cfg, &model_cfg, &device);
        assert_eq!(lineages.len(), cfg.n_lineages);
        for lin in &lineages {
            // Each round records one loss entry. Each perturbation also
            // calls train_burst on the candidate, but that's on a
            // disposable lineage — only the kept lineage's history matters.
            assert_eq!(lin.loss_history.len(), cfg.n_rounds);
            for &l in &lin.loss_history {
                assert!(l.is_finite(), "lineage {} produced non-finite loss", lin.id);
            }
        }
    }
}
