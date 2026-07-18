//! Top-level training orchestrator - **mutation self-play** (ns-50 Phase 1).
//!
//! Supersedes the retired gradient + three-track design. The loop:
//!
//! 1. **Seed the champion.** If `raters/index.json` is empty, bootstrap the
//!    Phase-0 net (supervised regression to the hand-crafted eval), persist it
//!    as `v0001`, and use it as the initial champion parent. Otherwise load the
//!    latest accepted rater as the parent.
//! 2. **Mutate big.** Each iteration, add large randomized Gaussian noise to the
//!    champion's weights (`perturb_model`) - the "big jump" bet of plan §4.
//! 3. **Gauntlet-filter.** Quantize the candidate, wrap it as an `NnueEvaluator`
//!    (the incremental in-search path), and play a mirrored best-of-three
//!    against the current champion at 100 ms/ply. First acceptance target: beat
//!    the hand-crafted `HeuristicEvaluator` (the seed must clear it to be
//!    crowned; thereafter candidates play the reigning NN champion).
//! 4. **Accept + persist.** A candidate that out-wins the champion becomes the
//!    new champion, is persisted as the next `vNNNN`, and is appended to the
//!    index. Repeat for `n_iterations`.
//!
//! Throughout, the driver writes:
//! - `<run_dir>/status.json` - summary state (throttled).
//! - `<run_dir>/live.json` - per-ply state during matches, only when the UI
//!   sets the `live.sub` sentinel.
//! - `<run_dir>/raters/index.json` - registry of accepted raters.
//! - `<run_dir>/raters/vNNNN.{mpk,json}` - accepted rater blobs + sidecars.
//! - `<run_dir>/matrix.json` - challenger×defender match matrix.
//!
//! ## Backends
//!
//! Bootstrap + mutation run on the fixed CPU backends (`TrainingBackend` for
//! perturbation/gradient, `InferenceBackend` for quantize + gauntlet play).
//! The dense-MLP GPU dispatch of the retired design is gone; `run_training`
//! still takes a `BackendChoice` for API compatibility but only `Cpu` is
//! supported (others return `BackendUnavailable`).
//!
//! ## Cancellation
//!
//! `should_stop` is checked at every iteration boundary and every ply (via the
//! live callback). On cancel the driver writes a final `phase=Idle` snapshot
//! and returns the partial summary.

use crate::backend::{BackendChoice, InferenceBackend, TrainingBackend};
use crate::bootstrap::train_scalar;
use crate::gauntlet::{accept_vs, play_match_with_callback, ChampionTracker, SeriesTally};
use crate::lineage::perturb_model;
use crate::live::{is_subscribed, write_if_subscribed, EvalBars, LivePosition, LIVE_POSITION_VERSION};
use crate::loadout::random_loadout_from_seed;
use crate::matrix::{load_matrix, save_matrix, MatrixError};
use crate::model::{Mlp, MlpConfig};
use crate::nnue_evaluator::NnueEvaluator;
use crate::persistence::{
    load_rater, save_rater, BracketWinRate, PerturbationEvent, PersistenceError,
    RaterMetadata, TrainingConfigSnapshot, RATER_FORMAT_VERSION,
};
use crate::quantized::{QuantScales, QuantizedNet, QA, QW};
use crate::registry::{IndexEntry, IndexError, RaterIndex, Track};
use crate::sparse::NUM_FEATURES;
use crate::snapshot::{
    write_snapshot, ActiveMatch, PopulationMember, SnapshotError, StatusSnapshot,
    TrainingPhase, STATUS_SNAPSHOT_VERSION,
};
use crate::train::{into_inference, TrainingConfig};
use crate::bootstrap::LABEL_DIVISOR;

use core_engine::search::evaluator::{Evaluator, HeuristicEvaluator};
use core_engine::state::fen;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Golden-ratio odd constant for deterministic per-iteration seed mixing.
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

/// Bracket string for the single 100 ms track. Kept as a field on the snapshot
/// / matrix schema (rather than dropped) to avoid a format-version bump.
const TRACK_LABEL: &str = "fast";

/// One-time Phase-0 bootstrap hyperparameters - the supervised regression that
/// seeds the very first champion. Only used when the index is empty.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BootstrapConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub epochs: usize,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self { learning_rate: 1e-3, batch_size: 16, epochs: 60 }
    }
}

impl From<&BootstrapConfig> for TrainingConfig {
    fn from(b: &BootstrapConfig) -> Self {
        TrainingConfig {
            learning_rate: b.learning_rate,
            batch_size: b.batch_size,
            epochs: b.epochs,
        }
    }
}

/// Top-level configuration for one mutation self-play run. Conservative
/// defaults wire up a tiny run that completes in seconds; production callers
/// crank `n_iterations` and the bootstrap epochs.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunConfig {
    /// How many mutate → gauntlet → maybe-accept iterations to run.
    pub n_iterations: usize,
    /// Think time (ms/ply) for the single 100 ms gauntlet track.
    pub gauntlet_think_ms: u64,
    /// Std-dev of the Gaussian noise added to the champion's weights each
    /// iteration - the "big jump" magnitude. Larger than the retired gradient
    /// perturb (0.03–0.05): mutation self-play bets on occasional large jumps.
    pub mutation_std: f32,
    /// Model topology. Must match a persisted champion's topology on resume.
    pub model: MlpConfig,
    /// One-time Phase-0 bootstrap config (only used when the index is empty).
    pub bootstrap: BootstrapConfig,
    /// Root seed; per-iteration seeds derive deterministically.
    pub seed_root: u64,
    /// Override the training-corpus file path (None = the default gitignored
    /// `bench/corpus/nn_training_corpus.txt`). Set by tests to a temp path so
    /// they never touch the repo corpus. `#[serde(default)]` keeps the IPC
    /// JSON round-trip compatible.
    #[serde(default)]
    pub training_corpus_path: Option<std::path::PathBuf>,
    /// Override the training-corpus target size (None = `TRAINING_CORPUS_TARGET`
    /// = 100k). Tests set a tiny value to avoid minutes of generation.
    #[serde(default)]
    pub training_corpus_target: Option<usize>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self::smoke()
    }
}

impl RunConfig {
    /// The NNUE sparse model topology: `NUM_FEATURES` sparse binary inputs →
    /// `[ACCUM_WIDTH, 32, 32] → 1`. `MlpConfig::new()`'s default input_dim is
    /// the *dense* `INPUT_DIM`; the NNUE path needs the sparse input dim, and
    /// `QuantizedNet::from_mlp` asserts `hidden_sizes[0] == ACCUM_WIDTH`.
    pub fn sparse_model() -> MlpConfig {
        MlpConfig::new().with_input_dim(NUM_FEATURES)
    }

    /// Smoke-test preset: 2 iterations, tiny bootstrap, ~seconds total.
    /// NOTE: uses the full 100k training corpus (generated on first use); the
    /// bootstrap epochs are low so the seed is weak, but the corpus is real.
    pub fn smoke() -> Self {
        Self {
            n_iterations: 2,
            gauntlet_think_ms: 10,
            mutation_std: 0.2,
            model: Self::sparse_model(),
            bootstrap: BootstrapConfig { learning_rate: 1e-3, batch_size: 16, epochs: 3 },
            seed_root: 0xCAFE_F00D,
            training_corpus_path: None,
            training_corpus_target: None,
        }
    }

    /// Medium preset: laptop iteration before a long run.
    pub fn medium() -> Self {
        Self {
            n_iterations: 30,
            gauntlet_think_ms: 100,
            mutation_std: 0.2,
            model: Self::sparse_model(),
            bootstrap: BootstrapConfig { learning_rate: 1e-3, batch_size: 64, epochs: 200 },
            seed_root: 0xCAFE_F00D,
            training_corpus_path: None,
            training_corpus_target: None,
        }
    }

    /// Long-run preset: the recommended shape for the first real session.
    pub fn long_run() -> Self {
        Self {
            n_iterations: 200,
            gauntlet_think_ms: 100,
            mutation_std: 0.25,
            model: Self::sparse_model(),
            bootstrap: BootstrapConfig { learning_rate: 1e-3, batch_size: 64, epochs: 400 },
            seed_root: 0xCAFE_F00D,
            training_corpus_path: None,
            training_corpus_target: None,
        }
    }

    /// Resolve a preset name. Unknown names error so the IPC layer can surface
    /// a typo instead of silently downgrading.
    pub fn from_preset(name: &str) -> Result<Self, String> {
        match name {
            "smoke" => Ok(Self::smoke()),
            "medium" => Ok(Self::medium()),
            "long" => Ok(Self::long_run()),
            other => Err(format!("unknown preset: {}", other)),
        }
    }

    /// Validate bounds. Returns the first violation.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=100_000).contains(&self.n_iterations) {
            return Err(format!("n_iterations out of [1,100000]: {}", self.n_iterations));
        }
        if self.gauntlet_think_ms < 1 {
            return Err("gauntlet_think_ms must be >= 1".to_string());
        }
        if !self.mutation_std.is_finite() || self.mutation_std <= 0.0 {
            return Err(format!("mutation_std must be finite and > 0: {}", self.mutation_std));
        }
        if self.bootstrap.epochs < 1 {
            return Err("bootstrap.epochs must be >= 1".to_string());
        }
        if !self.bootstrap.learning_rate.is_finite() || self.bootstrap.learning_rate <= 0.0 {
            return Err(format!("bootstrap.learning_rate must be finite and > 0: {}", self.bootstrap.learning_rate));
        }
        if self.bootstrap.batch_size < 1 {
            return Err("bootstrap.batch_size must be >= 1".to_string());
        }
        Ok(())
    }
}

/// Summary returned at the end of a run.
#[derive(Clone, Debug, Default)]
pub struct RunSummary {
    /// Iterations of the mutation loop that ran to completion. (Field name kept
    /// for IPC compatibility with the retired generation-based orchestrator.)
    pub generations_completed: usize,
    pub accepted_raters: usize,
    pub stopped_early: bool,
}

/// Errors emitted by the orchestrator.
#[derive(Debug)]
pub enum RunError {
    Persistence(PersistenceError),
    Index(IndexError),
    Snapshot(SnapshotError),
    Matrix(MatrixError),
    Live(crate::live::LiveError),
    Io(std::io::Error),
    /// Bootstrap could not build a labelled corpus (no positions to regress).
    EmptyBootstrapCorpus,
    /// The caller asked for a backend that isn't supported. Mutation self-play
    /// runs on CPU only; `Wgpu`/`Cuda` return this.
    BackendUnavailable(BackendChoice),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Persistence(e) => write!(f, "persistence error: {}", e),
            Self::Index(e) => write!(f, "index error: {}", e),
            Self::Snapshot(e) => write!(f, "snapshot error: {}", e),
            Self::Matrix(e) => write!(f, "matrix error: {}", e),
            Self::Live(e) => write!(f, "live error: {}", e),
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::EmptyBootstrapCorpus => write!(f, "bootstrap corpus was empty - no positions to regress"),
            Self::BackendUnavailable(b) => {
                write!(f, "backend `{}` not supported (mutation self-play is CPU-only)", b.as_str())
            }
        }
    }
}

impl std::error::Error for RunError {}

impl From<PersistenceError> for RunError { fn from(e: PersistenceError) -> Self { Self::Persistence(e) } }
impl From<IndexError> for RunError { fn from(e: IndexError) -> Self { Self::Index(e) } }
impl From<SnapshotError> for RunError { fn from(e: SnapshotError) -> Self { Self::Snapshot(e) } }
impl From<MatrixError> for RunError { fn from(e: MatrixError) -> Self { Self::Matrix(e) } }
impl From<crate::live::LiveError> for RunError { fn from(e: crate::live::LiveError) -> Self { Self::Live(e) } }
impl From<std::io::Error> for RunError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }

/// Path to the `raters/` subdirectory inside a run directory.
fn raters_dir(run_dir: &Path) -> PathBuf {
    run_dir.join("raters")
}

/// Build an `NnueEvaluator` from a CPU-backend f32 model by quantizing it.
/// The output scale folds in `LABEL_DIVISOR` so `forward_int` yields centipawns
/// (same convention as `bootstrap::bootstrap`).
fn evaluator_from_inference_model(model: &Mlp<InferenceBackend>) -> NnueEvaluator {
    let scales = QuantScales { qa: QA, qw: QW, out: LABEL_DIVISOR };
    NnueEvaluator::new(QuantizedNet::from_mlp(model, scales))
}

/// Run one mutation self-play session into `run_dir`. Top-level entry point.
///
/// `should_stop` is checked at every iteration boundary; setting it to `true`
/// from another thread winds the run down at the next safe point.
pub fn run_training(
    config: &RunConfig,
    run_dir: &Path,
    should_stop: Arc<AtomicBool>,
    backend: BackendChoice,
) -> Result<RunSummary, RunError> {
    match backend {
        BackendChoice::Cpu => run_training_cpu(config, run_dir, should_stop),
        other => Err(RunError::BackendUnavailable(other)),
    }
}

/// The mutation self-play driver (CPU backends throughout).
fn run_training_cpu(
    config: &RunConfig,
    run_dir: &Path,
    should_stop: Arc<AtomicBool>,
) -> Result<RunSummary, RunError> {
    let device: burn::tensor::Device<TrainingBackend> = Default::default();
    std::fs::create_dir_all(run_dir)?;
    std::fs::create_dir_all(raters_dir(run_dir))?;

    // Audit trail: persist the resolved config. Best-effort.
    if let Ok(json) = serde_json::to_vec_pretty(config) {
        let _ = std::fs::write(run_dir.join("config.json"), json);
    }

    let mut index = RaterIndex::load(&raters_dir(run_dir))?;
    let mut matrix = load_matrix(run_dir)?;
    let mut tracker = ChampionTracker::new();
    let mut summary = RunSummary::default();

    write_snapshot(run_dir, &StatusSnapshot::idle())?;

    // --- Seed the champion parent (generation 0) ---------------------------
    //
    // The parent is kept on `TrainingBackend` so it can be perturbed each
    // iteration; the gauntlet evaluator is derived per-iteration by stripping
    // autograd → `InferenceBackend` → quantize.
    let (mut champion_parent, mut champion_id): (Mlp<TrainingBackend>, String) =
        if let Some(latest) = index.latest() {
            // Resume: load the most recent accepted rater as the parent.
            let stem = raters_dir(run_dir).join(&latest.stem);
            let (model, _meta) = load_rater::<TrainingBackend>(&stem, &device)?;
            tracker.seed(index.entries.len() as u64, 1.0);
            eprintln!("[training] resuming from champion {}", latest.id);
            (model, latest.id.clone())
        } else {
            // Fresh: bootstrap the Phase-0 net and persist it as v0001.
            write_snapshot(run_dir, &snapshot_for(TrainingPhase::Training, 0, 0, &[], None))?;
            let corpus_path = config
                .training_corpus_path
                .clone()
                .unwrap_or_else(crate::bootstrap::training_corpus_path);
            let corpus_target = config
                .training_corpus_target
                .unwrap_or(crate::bootstrap::TRAINING_CORPUS_TARGET);
            let corpus = crate::bootstrap::label_training_corpus_with(
                &corpus_path,
                corpus_target,
                crate::bootstrap::TRAINING_CORPUS_MAX_GAMES,
                crate::bootstrap::TRAINING_CORPUS_SEED,
            );
            if corpus.is_empty() {
                return Err(RunError::EmptyBootstrapCorpus);
            }
            eprintln!("[training] bootstrap: {} labelled positions", corpus.len());
            let boot_cfg: TrainingConfig = (&config.bootstrap).into();
            let (seed_model, losses) = train_scalar(&corpus, &boot_cfg);
            eprintln!(
                "[training] bootstrap done - final epoch loss {:?}",
                losses.last()
            );

            let rater_id = "v0001".to_string();
            let stem = raters_dir(run_dir).join(&rater_id);
            let metadata = seed_metadata(config, &rater_id);
            // Persist the inference-backend model; reload on TrainingBackend as
            // the parent (recorder is cross-backend). This also validates the
            // round-trip immediately.
            save_rater::<InferenceBackend>(&seed_model, &stem, &metadata)?;
            index.append(IndexEntry {
                id: rater_id.clone(),
                stem: PathBuf::from(&rater_id),
                accepted_at: metadata.created_at.clone(),
                parent_id: None,
                bracket_results: metadata.bracket_results.clone(),
            })?;
            index.set_track(Track::Champion, &rater_id)?;
            index.save(&raters_dir(run_dir))?;
            summary.accepted_raters += 1;

            let (parent, _meta) = load_rater::<TrainingBackend>(&stem, &device)?;
            tracker.seed(1, 0.0); // seed has no win-rate floor yet
            (parent, rater_id)
        };

    // Force lazy-init of the parent's params so per-iteration clones start from
    // identical weights (the `perturb_model` lazy-init caveat).
    warmup(&champion_parent, &device);

    // --- Mutation loop -----------------------------------------------------
    for iter in 0..config.n_iterations {
        if should_stop.load(Ordering::Relaxed) {
            eprintln!("[training] stop flag set before iter {}, winding down", iter + 1);
            summary.stopped_early = true;
            break;
        }

        let iter_seed = config.seed_root.wrapping_add((iter as u64).wrapping_mul(SEED_MIX));
        let cand_tag = format!("iter{:05}", iter + 1);
        eprintln!(
            "[training] === iteration {}/{} (champion {champion_id}) ===",
            iter + 1, config.n_iterations
        );

        // Mutate the champion on the autodiff backend, then strip to inference.
        let candidate_parent = perturb_model(
            champion_parent.clone(),
            config.mutation_std,
            iter_seed,
            &device,
        );
        let candidate_inference: Mlp<InferenceBackend> =
            into_inference::<TrainingBackend>(candidate_parent.clone());
        let candidate_eval = evaluator_from_inference_model(&candidate_inference);

        // Opponent = current champion evaluator. On the very first iterations
        // the champion is the bootstrapped seed; the plan's first acceptance
        // target (beat the heuristic) is enforced by ALSO gating on a heuristic
        // gauntlet when the champion is still the seed (v0001) - see below.
        let champion_inference: Mlp<InferenceBackend> =
            into_inference::<TrainingBackend>(champion_parent.clone());
        let champion_eval = evaluator_from_inference_model(&champion_inference);

        // Live/snapshot: announce the active match.
        let active = ActiveMatch {
            challenger: cand_tag.clone(),
            defender: champion_id.clone(),
            game_index: 0,
            games_total: 3,
            ply: 0,
            bracket: TRACK_LABEL.to_string(),
            think_ms: config.gauntlet_think_ms as u32,
        };
        write_snapshot(run_dir, &snapshot_for(TrainingPhase::Gauntlet, (iter + 1) as u32, 0, &[], Some(active)))?;

        // Play candidate vs champion with per-ply live writes.
        let acc = accept_vs_live(
            &candidate_eval,
            &champion_eval,
            iter_seed,
            config.gauntlet_think_ms,
            run_dir,
            &cand_tag,
            &champion_id,
            should_stop.clone(),
        );
        let Some(acc) = acc else {
            eprintln!("[training] iter {}: interrupted by stop flag", iter + 1);
            summary.stopped_early = true;
            break;
        };
        matrix.record_series(&cand_tag, &champion_id, TRACK_LABEL, acc.tally);
        save_matrix(run_dir, &matrix)?;

        eprintln!(
            "[training] iter {}: candidate {}-{}-{} vs champion (win_rate {:.2}) pass={}",
            iter + 1,
            acc.tally.candidate_wins, acc.tally.baseline_wins, acc.tally.indecisive,
            acc.tally.win_rate(), acc.pass,
        );

        // First-acceptance gate: while the champion is still the bootstrapped
        // seed (v0001, never beaten anyone), a candidate must ALSO beat the
        // hand-crafted heuristic to be crowned - the plan's Phase-1 target.
        let mut accepted = acc.pass;
        let mut recorded = acc.tally;
        if accepted && champion_id == "v0001" {
            let vs_heur = accept_vs(&candidate_eval, &HeuristicEvaluator, iter_seed ^ 0xF00D, config.gauntlet_think_ms);
            eprintln!(
                "[training] iter {}: first-acceptance check vs heuristic - win_rate {:.2} pass={}",
                iter + 1, vs_heur.tally.win_rate(), vs_heur.pass,
            );
            matrix.record_series(&cand_tag, "heuristic", TRACK_LABEL, vs_heur.tally);
            save_matrix(run_dir, &matrix)?;
            accepted = vs_heur.pass;
            recorded = vs_heur.tally;
        }

        write_snapshot(run_dir, &snapshot_for(TrainingPhase::Bookkeeping, (iter + 1) as u32, 0, &[], None))?;

        if accepted {
            // Tie-break against the tracker floor (guards run-long cycling).
            let win_rate = recorded.win_rate();
            let is_new_best = tracker.consider((index.entries.len() + 1) as u64, win_rate);
            if is_new_best {
                let next_n = index.entries.len() + 1;
                let rater_id = format!("v{:04}", next_n);
                let stem = raters_dir(run_dir).join(&rater_id);
                let metadata = accepted_metadata(config, &rater_id, Some(champion_id.clone()), recorded, iter_seed);
                save_rater::<InferenceBackend>(&candidate_inference, &stem, &metadata)?;
                index.append(IndexEntry {
                    id: rater_id.clone(),
                    stem: PathBuf::from(&rater_id),
                    accepted_at: metadata.created_at.clone(),
                    parent_id: Some(champion_id.clone()),
                    bracket_results: metadata.bracket_results.clone(),
                })?;
                index.set_track(Track::Champion, &rater_id)?;
                index.save(&raters_dir(run_dir))?;
                summary.accepted_raters += 1;

                // The candidate becomes the new champion parent.
                champion_parent = candidate_parent;
                warmup(&champion_parent, &device);
                champion_id = rater_id;
                eprintln!("[training] iter {}: ACCEPTED as {champion_id}", iter + 1);
            }
        }

        summary.generations_completed += 1;
    }

    write_snapshot(run_dir, &StatusSnapshot::idle())?;
    Ok(summary)
}

/// Run one forward pass to force burn's lazy param initialisation to
/// materialise, so a subsequent `.clone()` + `perturb_model` starts from
/// identical weights (mirrors `lineage.rs` tests' warmup). Uses the sparse
/// input width the NNUE net was built with (`NUM_FEATURES`), not the dense
/// `INPUT_DIM`.
fn warmup(model: &Mlp<TrainingBackend>, device: &burn::tensor::Device<TrainingBackend>) {
    use burn::tensor::{Tensor, TensorData};
    let data = TensorData::new(vec![0.0_f32; NUM_FEATURES], [1, NUM_FEATURES]);
    let input: Tensor<TrainingBackend, 2> = Tensor::from_data(data, device);
    let _ = model.forward(input);
}

/// Mirrored BO3 (candidate vs champion) with per-ply live writes + stop-flag
/// checks. Returns `None` if cancellation interrupted before the series
/// completed. Semantics otherwise identical to `gauntlet::accept_vs`.
#[allow(clippy::too_many_arguments)]
fn accept_vs_live(
    candidate: &dyn Evaluator,
    champion: &dyn Evaluator,
    loadout_seed: u64,
    time_ms: u64,
    run_dir: &Path,
    challenger: &str,
    defender: &str,
    should_stop: Arc<AtomicBool>,
) -> Option<crate::gauntlet::Acceptance> {
    use core_engine::state::position::GameResult;

    let mut tally = SeriesTally::default();
    let loadout_a = random_loadout_from_seed(loadout_seed);

    // Game 1: candidate as P1.
    if should_stop.load(Ordering::Relaxed) { return None; }
    let g1 = play_match_with_callback(candidate, champion, &loadout_a, &loadout_a, time_ms, |pos, ply, action| {
        write_live(run_dir, pos, ply, action, challenger, defender, 1, 3);
    });
    match g1 {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }

    // Game 2: candidate as P2.
    if should_stop.load(Ordering::Relaxed) { return None; }
    let g2 = play_match_with_callback(champion, candidate, &loadout_a, &loadout_a, time_ms, |pos, ply, action| {
        write_live(run_dir, pos, ply, action, challenger, defender, 2, 3);
    });
    match g2 {
        Some(GameResult::P2Wins) => tally.candidate_wins += 1,
        Some(GameResult::P1Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }

    if tally.candidate_wins >= 2 || tally.baseline_wins >= 2 {
        return Some(crate::gauntlet::Acceptance { tally, pass: tally.candidate_leads() });
    }

    // Game 3 (tiebreaker, fresh loadout, candidate as P1).
    if should_stop.load(Ordering::Relaxed) { return None; }
    let loadout_b = random_loadout_from_seed(loadout_seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5));
    let g3 = play_match_with_callback(candidate, champion, &loadout_b, &loadout_b, time_ms, |pos, ply, action| {
        write_live(run_dir, pos, ply, action, challenger, defender, 3, 3);
    });
    match g3 {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    Some(crate::gauntlet::Acceptance { tally, pass: tally.candidate_leads() })
}

/// Serialise one ply into `live.json` iff the UI is subscribed. Failures are
/// swallowed - losing a live frame is not worth aborting a run.
#[allow(clippy::too_many_arguments)]
fn write_live(
    run_dir: &Path,
    pos: &core_engine::state::Position,
    ply: u32,
    action: &core_engine::game_logic::action::Action,
    challenger: &str,
    defender: &str,
    game_index: u32,
    games_total: u32,
) {
    if !is_subscribed(run_dir) {
        return;
    }
    let live = LivePosition {
        format_version: LIVE_POSITION_VERSION,
        written_at_ms: 0,
        fen: fen::to_fen(pos),
        last_action: format!("{:?}", action),
        ply,
        challenger: challenger.to_string(),
        defender: defender.to_string(),
        game_index,
        games_total,
        evals: EvalBars::default(),
    };
    let _ = write_if_subscribed(run_dir, &live);
}

/// Build a snapshot with the given phase/state.
fn snapshot_for(
    phase: TrainingPhase,
    generation: u32,
    round: u32,
    population: &[PopulationMember],
    active: Option<ActiveMatch>,
) -> StatusSnapshot {
    StatusSnapshot {
        format_version: STATUS_SNAPSHOT_VERSION,
        written_at_ms: 0,
        phase,
        generation,
        round,
        eta_seconds: None,
        population: population.to_vec(),
        active_match: active,
    }
}

/// Metadata for the bootstrapped seed (v0001). No parent, no gauntlet result.
fn seed_metadata(config: &RunConfig, rater_id: &str) -> RaterMetadata {
    RaterMetadata {
        format_version: RATER_FORMAT_VERSION,
        model_config: config.model.clone(),
        lineage_id: rater_id.to_string(),
        parent_id: None,
        training_step_count: config.bootstrap.epochs as u64,
        perturbation_history: Vec::new(),
        bracket_results: std::collections::BTreeMap::new(),
        training_config: TrainingConfigSnapshot {
            learning_rate: config.bootstrap.learning_rate,
            batch_size: config.bootstrap.batch_size,
            epochs: config.bootstrap.epochs,
        },
        git_sha: String::new(),
        created_at: iso8601_now(),
        eval_scale: 0.0,
    }
}

/// Metadata for an accepted mutated candidate.
fn accepted_metadata(
    config: &RunConfig,
    rater_id: &str,
    parent_id: Option<String>,
    tally: SeriesTally,
    mutation_seed: u64,
) -> RaterMetadata {
    let mut bracket_results = std::collections::BTreeMap::new();
    bracket_results.insert(TRACK_LABEL.to_string(), to_win_rate(tally));
    RaterMetadata {
        format_version: RATER_FORMAT_VERSION,
        model_config: config.model.clone(),
        lineage_id: rater_id.to_string(),
        parent_id,
        training_step_count: 0,
        perturbation_history: vec![PerturbationEvent {
            round: 0,
            std_dev: config.mutation_std,
            seed: mutation_seed,
        }],
        bracket_results,
        training_config: TrainingConfigSnapshot {
            learning_rate: config.bootstrap.learning_rate,
            batch_size: config.bootstrap.batch_size,
            epochs: config.bootstrap.epochs,
        },
        git_sha: String::new(),
        created_at: iso8601_now(),
        eval_scale: 0.0,
    }
}

fn to_win_rate(s: SeriesTally) -> BracketWinRate {
    BracketWinRate {
        games_played: s.games_played(),
        candidate_wins: s.candidate_wins,
        baseline_wins: s.baseline_wins,
        indecisive: s.indecisive,
    }
}

fn iso8601_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem_s = secs % 86_400;
    let h = rem_s / 3600;
    let m = (rem_s % 3600) / 60;
    let s = rem_s % 60;
    let (y, mo, d) = approx_ymd(days as u32);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

fn approx_ymd(mut days: u32) -> (u32, u32, u32) {
    let mut y: u32 = 1970;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let yd = if leap { 366 } else { 365 };
        if days < yd { break; }
        days -= yd;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let mlens = [31u32, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1u32;
    for ml in mlens {
        if days < ml { break; }
        days -= ml;
        m += 1;
    }
    (y, m, days + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    fn tempdir() -> PathBuf {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let n = NONCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("nn_trainer_run_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Tiny config for fast tests: a temp training corpus (small target, inside
    /// the test's own dir so it never touches the repo corpus) + minimal
    /// bootstrap + 1 mutation iteration.
    fn tiny_cfg(dir: &Path) -> RunConfig {
        RunConfig {
            n_iterations: 1,
            gauntlet_think_ms: 5,
            mutation_std: 0.2,
            model: RunConfig::sparse_model(),
            bootstrap: BootstrapConfig { learning_rate: 1e-3, batch_size: 8, epochs: 1 },
            seed_root: 1,
            training_corpus_path: Some(dir.join("train_corpus.txt")),
            training_corpus_target: Some(200),
        }
    }

    #[test]
    fn orchestrator_smoke_run_writes_observability_files_and_seeds_v0001() {
        // Tiny config: bootstrap + 1 mutation iteration. We don't assert the
        // candidate is accepted (depends on the mutation), but we DO assert the
        // run seeds v0001, produces a valid run-directory layout, and ends idle.
        let dir = tempdir();
        let cfg = tiny_cfg(&dir);
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_training(&cfg, &dir, stop, BackendChoice::Cpu).expect("orchestrator runs");

        // Final snapshot is Idle.
        let status = crate::snapshot::read_snapshot(&dir).expect("status read").expect("status present");
        assert_eq!(status.phase, TrainingPhase::Idle, "final snapshot must be Idle");

        // Matrix + index parse; v0001 seed persisted.
        let _ = load_matrix(&dir).expect("matrix parses");
        let index = RaterIndex::load(&raters_dir(&dir)).expect("index parses");
        assert!(index.get("v0001").is_some(), "bootstrap seed v0001 must be persisted");
        assert!(index.track_leader(Track::Champion).is_some(), "a champion must be set");
        assert!(summary.accepted_raters >= 1, "at least the seed is accepted");
    }

    #[test]
    fn orchestrator_respects_stop_flag_before_first_iteration() {
        // Stop set true up-front: the seed still bootstraps (that's the champion
        // setup, before the loop), but zero mutation iterations run.
        let dir = tempdir();
        let cfg = tiny_cfg(&dir);
        let stop = Arc::new(AtomicBool::new(true));
        let summary = run_training(&cfg, &dir, stop, BackendChoice::Cpu).expect("orchestrator runs");
        assert!(summary.stopped_early, "should stop before the mutation loop");
        assert_eq!(summary.generations_completed, 0);
        let status = crate::snapshot::read_snapshot(&dir).expect("status read").expect("status present");
        assert_eq!(status.phase, TrainingPhase::Idle);
    }

    #[test]
    fn non_cpu_backend_is_unavailable() {
        let dir = tempdir();
        let cfg = tiny_cfg(&dir);
        let stop = Arc::new(AtomicBool::new(false));
        let err = run_training(&cfg, &dir, stop, BackendChoice::Wgpu).expect_err("wgpu unsupported");
        assert!(matches!(err, RunError::BackendUnavailable(BackendChoice::Wgpu)));
    }

    /// Manual end-to-end harness (ns-50 Phase 1): bootstrap + a real mutation
    /// loop at 100 ms/ply, printing per-iteration BO3 outcomes and the
    /// beat-the-heuristic gate. `#[ignore]` - real 100 ms games take minutes;
    /// this is a diagnostic, not a gate. Run explicitly:
    /// `cargo test -p nn_trainer --release mutation_loop_end_to_end -- --ignored --nocapture`.
    /// Strength (does a candidate actually beat the heuristic) is a training
    /// outcome - this asserts only that the loop runs, seeds v0001, and the
    /// run directory ends in a valid state.
    #[test]
    #[ignore = "slow (minutes): manual mutation-loop E2E diagnostic"]
    fn mutation_loop_end_to_end() {
        let dir = tempdir();
        let cfg = RunConfig {
            n_iterations: 15,
            gauntlet_think_ms: 100,
            mutation_std: 0.25,
            model: RunConfig::sparse_model(),
            bootstrap: BootstrapConfig { learning_rate: 1e-3, batch_size: 64, epochs: 200 },
            seed_root: 0xABCD_1234,
            // Moderate temp corpus (bounded gen time; still ~170× the old 116).
            training_corpus_path: Some(dir.join("train_corpus.txt")),
            training_corpus_target: Some(20_000),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_training(&cfg, &dir, stop, BackendChoice::Cpu).expect("run");
        eprintln!(
            "mutation E2E: iterations={} accepted_raters={} stopped_early={}",
            summary.generations_completed, summary.accepted_raters, summary.stopped_early,
        );
        let index = RaterIndex::load(&raters_dir(&dir)).expect("index");
        eprintln!("accepted lineage: {:?}", index.entries.iter().map(|e| &e.id).collect::<Vec<_>>());
        assert!(index.get("v0001").is_some(), "seed persisted");
        assert!(summary.generations_completed >= 1, "loop ran");
    }

    #[test]
    fn approx_ymd_handles_known_dates() {
        assert_eq!(approx_ymd(0), (1970, 1, 1));
        assert_eq!(approx_ymd(365), (1971, 1, 1));
        assert_eq!(approx_ymd(730), (1972, 1, 1));
        assert_eq!(approx_ymd(790), (1972, 3, 1));
    }
}
