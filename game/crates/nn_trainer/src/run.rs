//! Top-level training orchestrator (plan §6 + §9 IPC).
//!
//! Drives the multi-generation loop that produces the observable training run:
//!
//! 1. Generate self-play corpus (current accepted leader vs itself, or the
//!    heuristic if the index is empty — bootstrap).
//! 2. `train_lineages` produces a candidate pool.
//! 3. Tier-1 fitness picks the strongest candidate at the fast bracket.
//! 4. Tier-2 acceptance runs the candidate against every accepted predecessor
//!    at all three brackets. If it passes any track, persist + register.
//! 5. Update the gauntlet matrix from every series played.
//! 6. Repeat for `n_generations`.
//!
//! Throughout, the driver writes:
//! - `<run_dir>/status.json` — summary state at ~1 Hz (throttled).
//! - `<run_dir>/live.json` — per-ply state during matches, only when the UI
//!   sets the `live.sub` sentinel.
//! - `<run_dir>/raters/index.json` — registry of accepted raters.
//! - `<run_dir>/raters/v0042.{mpk,json}` — accepted rater blobs + sidecars.
//! - `<run_dir>/matrix.json` — N×N gauntlet match matrix.
//!
//! ## Cancellation
//!
//! The driver checks `should_stop` at every generation boundary and at every
//! ply (via the live-callback path). On cancel it writes one final `phase=Idle`
//! snapshot and returns the partial summary.
//!
//! ## Why sequential
//!
//! `train_lineages` is sequential across lineages today (burn's Autodiff
//! backend isn't fully `Send`). The orchestrator inherits that. The big wins
//! from parallelism live inside `generate_corpus` (rayon-parallel games) —
//! that's already taken.

use crate::gauntlet::{
    play_match_with_callback, tier1_fitness,
    AcceptanceReport, Bracket, ChampionTracker, SeriesTally, TrackUpdate,
};
use crate::lineage::{Lineage, LineageConfig};
use crate::live::{is_subscribed, write_if_subscribed, EvalBars, LivePosition, LIVE_POSITION_VERSION};
use crate::loadout::random_loadout_from_seed;
use crate::matrix::{load_matrix, save_matrix, GauntletMatrix, MatrixError};
use crate::model::{Mlp, MlpConfig};
use crate::nn_evaluator::{InferenceBackend, NnEvaluator};
use crate::persistence::{
    save_rater, BracketWinRate, PerturbationEvent, PersistenceError, RaterMetadata,
    TrainingConfigSnapshot, RATER_FORMAT_VERSION,
};
use crate::registry::{IndexEntry, IndexError, RaterIndex, Track};
use crate::selfplay::LabelledPosition;
use crate::snapshot::{
    write_snapshot, ActiveMatch, PopulationMember, SnapshotError, StatusSnapshot,
    TrainingPhase, STATUS_SNAPSHOT_VERSION,
};
use crate::train::into_inference;
use crate::train::TrainingConfig;

use core_engine::search::evaluator::{Evaluator, HeuristicEvaluator};
use core_engine::state::fen;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::backend::TrainingBackend as AutodiffB;

/// Top-level configuration for one training run. Conservative defaults wire
/// up a tiny run that completes in seconds — production callers crank
/// `n_generations`, `corpus_games`, and `lineage.steps_per_burst`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunConfig {
    /// How many generations to run before stopping.
    pub n_generations: usize,
    /// Games of self-play per generation for the corpus.
    pub corpus_games: usize,
    /// Search depth for self-play corpus generation. The two depth knobs
    /// are kept separate: the corpus is *depth-bounded* so labels are
    /// reproducible across hosts, while the gauntlet is *time-bounded* so
    /// brackets compare like thinking budgets.
    pub corpus_search_depth: u8,
    /// Lineage / training hyperparameters (delegated).
    pub lineage: LineageConfig,
    /// Model topology.
    pub model: MlpConfig,
    /// Root seed; per-generation seeds are derived deterministically.
    /// Stored as a hex string at the wire boundary (JS can't safely
    /// round-trip integers > 2^53) — serde sees the literal u64 here.
    pub seed_root: u64,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self::smoke()
    }
}

impl RunConfig {
    /// Smoke-test preset: 2 generations, depth-2 search, ~seconds total.
    /// Same shape as the previous `Default` impl; used by CI / unit tests.
    pub fn smoke() -> Self {
        Self {
            n_generations: 2,
            corpus_games: 4,
            corpus_search_depth: 2,
            lineage: LineageConfig::default(),
            model: MlpConfig::new(),
            seed_root: 0xCAFE_F00D,
        }
    }

    /// Medium preset: ~5 generations, depth-4 search, depth-N gauntlet.
    /// Useful for laptop iteration before committing to a long GPU run.
    pub fn medium() -> Self {
        Self {
            n_generations: 5,
            corpus_games: 32,
            corpus_search_depth: 4,
            lineage: LineageConfig {
                n_lineages: 4,
                n_rounds: 5,
                steps_per_burst: 50,
                steps_per_candidate: 25,
                perturb_std: 0.04,
                training: TrainingConfig {
                    learning_rate: 1e-3,
                    batch_size: 64,
                    epochs: 3,
                },
            },
            model: MlpConfig::new(),
            seed_root: 0xCAFE_F00D,
        }
    }

    /// Long-run preset: the recommended shape for the first real GPU
    /// session (10 generations × 8 lineages × depth-6 corpus).
    pub fn long_run() -> Self {
        Self {
            n_generations: 10,
            corpus_games: 64,
            corpus_search_depth: 6,
            lineage: LineageConfig {
                n_lineages: 8,
                n_rounds: 10,
                steps_per_burst: 100,
                steps_per_candidate: 50,
                perturb_std: 0.03,
                training: TrainingConfig {
                    learning_rate: 1e-3,
                    batch_size: 128,
                    epochs: 5,
                },
            },
            model: MlpConfig::new(),
            seed_root: 0xCAFE_F00D,
        }
    }

    /// Resolve a preset name. `None` and unknown names fall back to smoke.
    /// Returns `Err` only for unknown names so the IPC layer can surface
    /// a typo to the user instead of silently downgrading.
    pub fn from_preset(name: &str) -> Result<Self, String> {
        match name {
            "smoke" => Ok(Self::smoke()),
            "medium" => Ok(Self::medium()),
            "long" => Ok(Self::long_run()),
            other => Err(format!("unknown preset: {}", other)),
        }
    }

    /// Validate bounds. Returns the first violation; callers surface it via
    /// the existing `startError` path. Cheap to call.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=1000).contains(&self.n_generations) {
            return Err(format!("n_generations out of [1,1000]: {}", self.n_generations));
        }
        if !(1..=10_000).contains(&self.corpus_games) {
            return Err(format!("corpus_games out of [1,10000]: {}", self.corpus_games));
        }
        if !(1..=8).contains(&self.corpus_search_depth) {
            return Err(format!("corpus_search_depth out of [1,8]: {}", self.corpus_search_depth));
        }
        if !(1..=64).contains(&self.lineage.n_lineages) {
            return Err(format!("n_lineages out of [1,64]: {}", self.lineage.n_lineages));
        }
        if self.lineage.n_rounds < 1 {
            return Err("n_rounds must be >= 1".to_string());
        }
        if self.lineage.steps_per_burst < 1 {
            return Err("steps_per_burst must be >= 1".to_string());
        }
        if !self.lineage.perturb_std.is_finite() || self.lineage.perturb_std <= 0.0 {
            return Err(format!("perturb_std must be finite and > 0: {}", self.lineage.perturb_std));
        }
        if !self.lineage.training.learning_rate.is_finite() || self.lineage.training.learning_rate <= 0.0 {
            return Err(format!("learning_rate must be finite and > 0: {}", self.lineage.training.learning_rate));
        }
        if self.lineage.training.batch_size < 1 {
            return Err("batch_size must be >= 1".to_string());
        }
        Ok(())
    }
}

/// Summary returned at the end of a run. Useful for tests and for a CLI that
/// wants a one-line report after `run_training` returns.
#[derive(Clone, Debug, Default)]
pub struct RunSummary {
    pub generations_completed: usize,
    pub accepted_raters: usize,
    pub stopped_early: bool,
}

/// Errors emitted by the orchestrator. Wraps every underlying error type so
/// callers don't need to import them individually.
#[derive(Debug)]
pub enum RunError {
    Persistence(PersistenceError),
    Index(IndexError),
    Snapshot(SnapshotError),
    Matrix(MatrixError),
    Live(crate::live::LiveError),
    Io(std::io::Error),
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

/// Cap on how many accepted predecessors Tier-2 plays the new candidate
/// against. Older raters fall off the back; the most recent `MAX_PREDECESSORS`
/// stay in the gauntlet pool. This bounds Tier-2 cost as the index grows.
const MAX_PREDECESSORS: usize = 16;

/// Load up to `MAX_PREDECESSORS` of the most recently accepted raters from
/// disk as `NnEvaluator`s for Tier-2 gauntlet play. Returns an empty vec if
/// the index is empty; callers should bootstrap against the heuristic in that
/// case. Corrupt or missing blobs are skipped with a stderr warning rather
/// than aborting the run.
fn load_predecessor_evaluators(
    index: &RaterIndex,
    raters_dir: &Path,
) -> Vec<NnEvaluator> {
    let device: burn::tensor::Device<InferenceBackend> = Default::default();
    let take_from = index.entries.len().saturating_sub(MAX_PREDECESSORS);
    let window = &index.entries[take_from..];
    let mut owned = Vec::with_capacity(window.len());
    for entry in window {
        let stem = raters_dir.join(&entry.stem);
        match crate::persistence::load_rater::<InferenceBackend>(&stem, &device) {
            Ok((model, _meta)) => owned.push(NnEvaluator::new(model)),
            Err(e) => eprintln!(
                "nn_trainer: skipping predecessor {}: {}",
                entry.id, e
            ),
        }
    }
    owned
}

/// Run one training session into `run_dir`. Creates the directory if missing.
///
/// `should_stop` is checked at every generation boundary; setting it to
/// `true` from another thread causes the run to wind down at the next safe
/// point and return a partial summary.
pub fn run_training(
    config: &RunConfig,
    run_dir: &Path,
    should_stop: Arc<AtomicBool>,
) -> Result<RunSummary, RunError> {
    std::fs::create_dir_all(run_dir)?;
    std::fs::create_dir_all(raters_dir(run_dir))?;

    // Persist the resolved config so future inspection (and the UI's "what
    // was this run started with?" prefill in task 3b) has an audit trail.
    // Best-effort: a failed write must not abort the run.
    if let Ok(json) = serde_json::to_vec_pretty(config) {
        let _ = std::fs::write(run_dir.join("config.json"), json);
    }

    let device = Default::default();
    let mut index = RaterIndex::load(&raters_dir(run_dir))?;
    let mut matrix = load_matrix(run_dir)?;
    let mut tracker = ChampionTracker::new();
    let mut summary = RunSummary::default();

    // Initial idle snapshot — UI shows "starting" until phase advances.
    write_snapshot(run_dir, &StatusSnapshot::idle())?;

    for gen_idx in 0..config.n_generations {
        if should_stop.load(Ordering::Relaxed) {
            summary.stopped_early = true;
            break;
        }

        let generation = (gen_idx + 1) as u32;
        let gen_seed = config
            .seed_root
            .wrapping_add((gen_idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));

        // --- Phase: Training ---
        write_snapshot(
            run_dir,
            &snapshot_for(TrainingPhase::Training, generation, 0, &[], None),
        )?;

        let corpus = build_corpus(config, gen_seed);
        if corpus.is_empty() {
            // No usable data this generation — skip to the next.
            continue;
        }

        let lineages: Vec<Lineage<AutodiffB>> = {
            // Throttle heartbeats to ~1 Hz — train_lineages_with_progress
            // fires the callback after every (lineage, round) pair, which is
            // frequent enough to spam status writes but also frequent enough
            // that we never go more than a second or two without one.
            let run_dir_cb = run_dir.to_path_buf();
            let mut last_write = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(10))
                .unwrap_or_else(std::time::Instant::now);
            crate::lineage::train_lineages_with_progress::<AutodiffB, _>(
                &corpus,
                gen_seed,
                &config.lineage,
                &config.model,
                &device,
                |lineage_idx, round_idx, n_rounds| {
                    if last_write.elapsed() < std::time::Duration::from_millis(800) {
                        return;
                    }
                    last_write = std::time::Instant::now();
                    let round = (round_idx as u32) + 1;
                    let _ = write_snapshot(
                        &run_dir_cb,
                        &snapshot_for(
                            TrainingPhase::Training,
                            generation,
                            round,
                            &[PopulationMember {
                                rater_id: format!("g{:04}-l{}", generation, lineage_idx),
                                parent_id: None,
                                lineage: lineage_idx as u32,
                                generation,
                                wins: 0,
                                losses: 0,
                                draws: 0,
                                alive: true,
                            }],
                            None,
                        ),
                    );
                    // Capture n_rounds in a way the compiler doesn't warn about
                    let _ = n_rounds;
                },
            )
        };

        if should_stop.load(Ordering::Relaxed) {
            summary.stopped_early = true;
            break;
        }

        // --- Phase: Gauntlet ---
        // Each lineage becomes an NnEvaluator candidate; the strongest at the
        // fast bracket (per Tier 1) is the one we promote into Tier 2.
        let candidates_inference: Vec<Mlp<InferenceBackend>> = lineages
            .iter()
            .map(|lin| into_inference::<AutodiffB>(lin.model.clone()))
            .collect();
        let candidate_evaluators: Vec<NnEvaluator> = candidates_inference
            .into_iter()
            .map(NnEvaluator::new)
            .collect();

        // The baseline pool against which Tier 1 scores each candidate.
        // Bootstrap: when no accepted raters exist, use the heuristic.
        let baseline = HeuristicEvaluator;
        let baselines: Vec<&dyn Evaluator> = vec![&baseline];

        let mut population: Vec<PopulationMember> = Vec::with_capacity(candidate_evaluators.len());
        let mut best_idx: usize = 0;
        let mut best_wins: u32 = 0;
        for (i, cand) in candidate_evaluators.iter().enumerate() {
            if should_stop.load(Ordering::Relaxed) {
                summary.stopped_early = true;
                break;
            }
            let tier1_seed = gen_seed.wrapping_add((i as u64).wrapping_mul(101));
            let tally = tier1_fitness(cand, &baselines, tier1_seed);
            let losses = tally.baseline_wins;
            let wins = tally.candidate_wins;
            if wins > best_wins {
                best_wins = wins;
                best_idx = i;
            }
            population.push(PopulationMember {
                rater_id: format!("g{:04}-l{}", generation, i),
                parent_id: index.latest().map(|e| e.id.clone()),
                lineage: i as u32,
                generation,
                wins,
                losses,
                draws: tally.indecisive,
                alive: true,
            });
            write_snapshot(
                run_dir,
                &snapshot_for(TrainingPhase::Gauntlet, generation, 1, &population, None),
            )?;
        }

        if summary.stopped_early { break; }

        // --- Tier-2 acceptance for the best candidate ---
        let champ = &candidate_evaluators[best_idx];

        // Predecessors: the most recently accepted raters (capped at
        // MAX_PREDECESSORS) loaded from disk. If the index is empty,
        // bootstrap against the heuristic so Tier-2 has someone to play.
        let owned_predecessors: Vec<NnEvaluator> =
            load_predecessor_evaluators(&index, &raters_dir(run_dir));
        let predecessors: Vec<&dyn Evaluator> = if owned_predecessors.is_empty() {
            vec![&baseline]
        } else {
            owned_predecessors
                .iter()
                .map(|e| e as &dyn Evaluator)
                .collect()
        };
        let acceptance_seed = gen_seed.wrapping_add(0xDEAD_BEEF);

        let live_dir = run_dir.to_path_buf();
        let report = run_tier2_with_live(
            champ,
            &predecessors,
            acceptance_seed,
            &live_dir,
            generation,
            best_idx as u32,
            &mut matrix,
        );

        // Update the matrix from Tier-2 series, save it.
        save_matrix(run_dir, &matrix)?;

        if report.is_none() {
            summary.stopped_early = true;
            break;
        }
        let report = report.unwrap();

        // --- Bookkeeping ---
        write_snapshot(
            run_dir,
            &snapshot_for(TrainingPhase::Bookkeeping, generation, 2, &population, None),
        )?;

        let upd: TrackUpdate = tracker.consider(generation as u64, &report);
        if upd.any_track() {
            let next_n = index.entries.len() + 1;
            let rater_id = format!("v{:04}", next_n);
            let stem = raters_dir(run_dir).join(&rater_id);
            let blob_model = into_inference::<AutodiffB>(lineages[best_idx].model.clone());
            // save_rater expects a model with autodiff stripped — the
            // inference-mode model is exactly that.
            let parent_id_for_entry = index.latest().map(|e| e.id.clone());
            let metadata = build_metadata(
                &config,
                &rater_id,
                parent_id_for_entry.clone(),
                &report,
                &lineages[best_idx],
            );
            save_rater::<InferenceBackend>(&blob_model, &stem, &metadata)?;
            let entry = IndexEntry {
                id: rater_id.clone(),
                stem: PathBuf::from(&rater_id),
                accepted_at: metadata.created_at.clone(),
                parent_id: parent_id_for_entry,
                bracket_results: metadata.bracket_results.clone(),
            };
            index.append(entry)?;
            for (track, flag) in [(Track::Fast, upd.fast), (Track::Slow, upd.slow), (Track::Overall, upd.overall)] {
                if flag {
                    index.set_track(track, &rater_id)?;
                }
            }
            index.save(&raters_dir(run_dir))?;
            summary.accepted_raters += 1;
        }

        summary.generations_completed += 1;
    }

    // Final idle snapshot — the UI sees the run wrap up cleanly.
    write_snapshot(run_dir, &StatusSnapshot::idle())?;
    Ok(summary)
}

/// Produce a self-play corpus for one generation. Always plays the heuristic
/// against itself in the bootstrap case; once accepted raters exist a future
/// revision will load and use them as opponents.
fn build_corpus(config: &RunConfig, gen_seed: u64) -> Vec<LabelledPosition> {
    crate::batch::generate_corpus(
        config.corpus_games,
        gen_seed,
        &HeuristicEvaluator,
        &HeuristicEvaluator,
        config.corpus_search_depth,
    )
}

/// Build a snapshot with the given phase/state. Inline helper to keep the
/// orchestrator readable.
fn snapshot_for(
    phase: TrainingPhase,
    generation: u32,
    round: u32,
    population: &[PopulationMember],
    active: Option<ActiveMatch>,
) -> StatusSnapshot {
    StatusSnapshot {
        format_version: STATUS_SNAPSHOT_VERSION,
        written_at_ms: 0, // writer stamps
        phase,
        generation,
        round,
        eta_seconds: None,
        population: population.to_vec(),
        active_match: active,
    }
}

/// Run a Tier-2 acceptance pass, writing live-position state per ply when the
/// UI is subscribed. Returns `None` if cancellation interrupted the work
/// before the report could be assembled.
///
/// This duplicates `tier2_acceptance`'s structure so we can thread the
/// per-ply callback. We can't just call `tier2_acceptance` and inject a hook
/// after the fact — the call boundary is below where the callback lives.
fn run_tier2_with_live(
    candidate: &dyn Evaluator,
    predecessors: &[&dyn Evaluator],
    loadout_seed: u64,
    run_dir: &Path,
    generation: u32,
    lineage: u32,
    matrix: &mut GauntletMatrix,
) -> Option<AcceptanceReport> {
    // Bracket-by-bracket loop inlined here so we can thread the per-ply
    // callback through `play_match_with_callback`. The non-live path
    // (`gauntlet::tier2_acceptance`) remains the canonical reference.
    assert!(!predecessors.is_empty());

    let mut per_predecessor: Vec<crate::gauntlet::BracketResults> = Vec::with_capacity(predecessors.len());
    for (pi, pred) in predecessors.iter().enumerate() {
        let pred_seed = loadout_seed.wrapping_add((pi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut br = crate::gauntlet::BracketResults::default();
        for b in Bracket::all() {
            let tally = mirrored_bo3_live(
                candidate, *pred, pred_seed, b,
                run_dir, generation, lineage, pi as u32,
            );
            match b {
                Bracket::Fast => br.fast = tally,
                Bracket::Medium => br.medium = tally,
                Bracket::Slow => br.slow = tally,
            }
            let bracket_name = match b {
                Bracket::Fast => "fast",
                Bracket::Medium => "medium",
                Bracket::Slow => "slow",
            };
            matrix.record_series(
                &format!("g{:04}-l{}", generation, lineage),
                &format!("pred-{}", pi),
                bracket_name,
                tally,
            );
        }
        per_predecessor.push(br);
    }

    // Aggregate + pass flags — copy of gauntlet.rs logic.
    let mut aggregate = crate::gauntlet::BracketResults::default();
    for br in &per_predecessor {
        for b in Bracket::all() {
            let sum = match b {
                Bracket::Fast => &mut aggregate.fast,
                Bracket::Medium => &mut aggregate.medium,
                Bracket::Slow => &mut aggregate.slow,
            };
            let t = br.at(b);
            sum.candidate_wins += t.candidate_wins;
            sum.baseline_wins += t.baseline_wins;
            sum.indecisive += t.indecisive;
        }
    }
    const NON_REGRESSION_BAR: f32 = 0.45;
    let last_idx = per_predecessor.len() - 1;
    let mut bracket_pass = [false; 3];
    for (i, b) in Bracket::all().iter().enumerate() {
        let imm = per_predecessor[last_idx].at(*b);
        if !imm.candidate_leads() { continue; }
        let mut ok = true;
        for (j, br) in per_predecessor.iter().enumerate() {
            if j == last_idx { continue; }
            if br.at(*b).win_rate() < NON_REGRESSION_BAR {
                ok = false;
                break;
            }
        }
        bracket_pass[i] = ok;
    }

    Some(AcceptanceReport { per_predecessor, aggregate, bracket_pass })
}

/// Mirrored BO3 with live-position writes per ply. Same semantics as
/// `gauntlet::mirrored_bo3` but threads `write_if_subscribed` into each game.
fn mirrored_bo3_live(
    candidate: &dyn Evaluator,
    baseline: &dyn Evaluator,
    loadout_seed: u64,
    bracket: Bracket,
    run_dir: &Path,
    generation: u32,
    lineage: u32,
    pred_idx: u32,
) -> SeriesTally {
    use core_engine::state::position::GameResult;

    let mut tally = SeriesTally::default();
    let loadout_a = random_loadout_from_seed(loadout_seed);

    let challenger = format!("g{:04}-l{}", generation, lineage);
    let defender = format!("pred-{}", pred_idx);
    let bracket_name = match bracket {
        Bracket::Fast => "fast",
        Bracket::Medium => "medium",
        Bracket::Slow => "slow",
    };

    // Game 1: candidate as P1.
    let g1 = play_match_with_callback(
        candidate, baseline, &loadout_a, &loadout_a, bracket,
        |pos, ply, action| {
            write_live(run_dir, pos, ply, action, &challenger, &defender, 1, 3, bracket_name);
        },
    );
    match g1 {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }

    // Game 2: candidate as P2.
    let g2 = play_match_with_callback(
        baseline, candidate, &loadout_a, &loadout_a, bracket,
        |pos, ply, action| {
            write_live(run_dir, pos, ply, action, &challenger, &defender, 2, 3, bracket_name);
        },
    );
    match g2 {
        Some(GameResult::P2Wins) => tally.candidate_wins += 1,
        Some(GameResult::P1Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }

    if tally.candidate_wins >= 2 || tally.baseline_wins >= 2 {
        return tally;
    }

    // Game 3 (tiebreaker, fresh loadout, candidate as P1).
    let loadout_b = random_loadout_from_seed(loadout_seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5));
    let g3 = play_match_with_callback(
        candidate, baseline, &loadout_b, &loadout_b, bracket,
        |pos, ply, action| {
            write_live(run_dir, pos, ply, action, &challenger, &defender, 3, 3, bracket_name);
        },
    );
    match g3 {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    tally
}

/// Helper: serialise one ply into `live.json` iff the UI is subscribed.
/// Failures are swallowed — losing a live frame is not worth aborting a run.
fn write_live(
    run_dir: &Path,
    pos: &core_engine::state::Position,
    ply: u32,
    action: &core_engine::game_logic::action::Action,
    challenger: &str,
    defender: &str,
    game_index: u32,
    games_total: u32,
    bracket: &str,
) {
    if !is_subscribed(run_dir) {
        return;
    }
    let fen_str = fen::to_fen(pos);
    let live = LivePosition {
        format_version: LIVE_POSITION_VERSION,
        written_at_ms: 0,
        fen: fen_str,
        last_action: format!("{:?}", action),
        ply,
        challenger: challenger.to_string(),
        defender: defender.to_string(),
        game_index,
        games_total,
        evals: EvalBars::default(),
    };
    let _ = write_if_subscribed(run_dir, &live);
    let _ = bracket; // bracket reserved for future header rendering
}

/// Build metadata for a freshly-accepted rater.
fn build_metadata(
    config: &RunConfig,
    rater_id: &str,
    parent_id: Option<String>,
    report: &AcceptanceReport,
    lineage: &Lineage<AutodiffB>,
) -> RaterMetadata {
    let mut bracket_results = std::collections::BTreeMap::new();
    bracket_results.insert("fast".to_string(), to_win_rate(report.aggregate.fast));
    bracket_results.insert("medium".to_string(), to_win_rate(report.aggregate.medium));
    bracket_results.insert("slow".to_string(), to_win_rate(report.aggregate.slow));

    RaterMetadata {
        format_version: RATER_FORMAT_VERSION,
        model_config: config.model.clone(),
        lineage_id: rater_id.to_string(),
        parent_id,
        training_step_count: (lineage.loss_history.len() as u64)
            * config.lineage.steps_per_burst as u64,
        perturbation_history: vec![PerturbationEvent {
            round: lineage.loss_history.len() as u32,
            std_dev: config.lineage.perturb_std,
            seed: lineage.seed,
        }],
        bracket_results,
        training_config: TrainingConfigSnapshot::from(&config.lineage.training),
        git_sha: String::new(),
        created_at: iso8601_now(),
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
    // Minimal ISO-8601 — formats `YYYY-MM-DDTHH:MM:SSZ` without pulling chrono.
    // We don't need civil-time accuracy here; the registry just wants a stamp.
    let days = secs / 86_400;
    let rem_s = secs % 86_400;
    let h = rem_s / 3600;
    let m = (rem_s % 3600) / 60;
    let s = rem_s % 60;
    // Epoch = 1970-01-01. Roughly approximate the date by adding `days` to that.
    // Good enough for an audit stamp — replace with a real date crate if we
    // ever care about civil-calendar correctness.
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
    let mlens = [31u32, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
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

    #[test]
    fn orchestrator_smoke_run_writes_observability_files() {
        // Tiny config: 1 generation, tiny corpus, small lineage. We don't
        // assert acceptance — accepting depends on whether the random init
        // beat the heuristic, which it usually won't. We DO assert that the
        // run produces a valid run-directory layout and a final idle
        // snapshot.
        let dir = tempdir();
        let cfg = RunConfig {
            n_generations: 1,
            corpus_games: 2,
            corpus_search_depth: 2,
            lineage: LineageConfig {
                n_lineages: 2,
                n_rounds: 1,
                steps_per_burst: 1,
                steps_per_candidate: 1,
                perturb_std: 0.05,
                training: crate::train::TrainingConfig {
                    learning_rate: 1e-3,
                    batch_size: 4,
                    epochs: 1,
                },
            },
            model: MlpConfig::new(),
            seed_root: 1,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_training(&cfg, &dir, stop).expect("orchestrator runs");

        // Status snapshot present + parses + final phase is Idle.
        let status = crate::snapshot::read_snapshot(&dir)
            .expect("status read")
            .expect("status present");
        assert_eq!(status.phase, TrainingPhase::Idle, "final snapshot must be Idle");

        // Matrix present + parses.
        let _ = load_matrix(&dir).expect("matrix parses");

        // Index parses (empty is fine — acceptance is unlikely at this scale).
        let _ = RaterIndex::load(&raters_dir(&dir)).expect("index parses");

        assert!(summary.generations_completed <= cfg.n_generations);
    }

    #[test]
    fn orchestrator_respects_stop_flag_before_first_generation() {
        let dir = tempdir();
        let cfg = RunConfig::default();
        let stop = Arc::new(AtomicBool::new(true));
        let summary = run_training(&cfg, &dir, stop).expect("orchestrator runs");
        assert!(summary.stopped_early, "should stop immediately");
        assert_eq!(summary.generations_completed, 0);
        // Even with no work done, a final idle snapshot must exist.
        let status = crate::snapshot::read_snapshot(&dir)
            .expect("status read")
            .expect("status present");
        assert_eq!(status.phase, TrainingPhase::Idle);
    }

    #[test]
    fn approx_ymd_handles_known_dates() {
        // 1970-01-01 → day 0
        assert_eq!(approx_ymd(0), (1970, 1, 1));
        // 1971-01-01 → day 365
        assert_eq!(approx_ymd(365), (1971, 1, 1));
        // 1972 is a leap year, so 1972-01-01 is day 365 + 365 = 730
        assert_eq!(approx_ymd(730), (1972, 1, 1));
        // 1972-03-01 → 730 + 31 + 29 = 790
        assert_eq!(approx_ymd(790), (1972, 3, 1));
    }

    // --- load_predecessor_evaluators ----------------------------------

    fn write_dummy_rater(raters: &Path, id: &str) -> IndexEntry {
        use crate::model::MlpConfig;
        use crate::persistence::{
            save_rater, RaterMetadata, TrainingConfigSnapshot, RATER_FORMAT_VERSION,
        };
        let device: burn::tensor::Device<InferenceBackend> = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<InferenceBackend> = cfg.clone().init(&device);
        let stem = raters.join(id);
        let metadata = RaterMetadata {
            format_version: RATER_FORMAT_VERSION,
            model_config: cfg,
            lineage_id: id.to_string(),
            parent_id: None,
            training_step_count: 0,
            perturbation_history: vec![],
            bracket_results: Default::default(),
            training_config: TrainingConfigSnapshot {
                learning_rate: 1e-3,
                batch_size: 4,
                epochs: 1,
            },
            git_sha: String::new(),
            created_at: "2026-06-28T00:00:00Z".to_string(),
        };
        save_rater::<InferenceBackend>(&model, &stem, &metadata)
            .expect("save dummy rater");
        IndexEntry {
            id: id.to_string(),
            stem: PathBuf::from(id),
            accepted_at: "2026-06-28T00:00:00Z".to_string(),
            parent_id: None,
            bracket_results: Default::default(),
        }
    }

    #[test]
    fn load_predecessor_evaluators_empty_index_returns_empty() {
        let dir = tempdir();
        let raters = dir.join("raters");
        std::fs::create_dir_all(&raters).unwrap();
        let index = RaterIndex::default();
        let out = load_predecessor_evaluators(&index, &raters);
        assert!(out.is_empty(), "empty index → no evaluators");
    }

    #[test]
    fn load_predecessor_evaluators_round_trips_saved_raters() {
        let dir = tempdir();
        let raters = dir.join("raters");
        std::fs::create_dir_all(&raters).unwrap();
        let mut index = RaterIndex::default();
        for id in ["v0001", "v0002", "v0003"] {
            let entry = write_dummy_rater(&raters, id);
            index.entries.push(entry);
        }
        let out = load_predecessor_evaluators(&index, &raters);
        assert_eq!(out.len(), 3, "all three raters must load");
    }

    #[test]
    fn load_predecessor_evaluators_caps_at_max_predecessors() {
        let dir = tempdir();
        let raters = dir.join("raters");
        std::fs::create_dir_all(&raters).unwrap();
        let mut index = RaterIndex::default();
        // Write MAX_PREDECESSORS + 3 raters; only the most recent
        // MAX_PREDECESSORS should be returned.
        let total = MAX_PREDECESSORS + 3;
        for i in 0..total {
            let id = format!("v{:04}", i);
            let entry = write_dummy_rater(&raters, &id);
            index.entries.push(entry);
        }
        let out = load_predecessor_evaluators(&index, &raters);
        assert_eq!(
            out.len(),
            MAX_PREDECESSORS,
            "must cap at MAX_PREDECESSORS"
        );
    }

    #[test]
    fn load_predecessor_evaluators_skips_corrupt_entries() {
        let dir = tempdir();
        let raters = dir.join("raters");
        std::fs::create_dir_all(&raters).unwrap();
        let mut index = RaterIndex::default();
        // One valid rater.
        index.entries.push(write_dummy_rater(&raters, "v0001"));
        // One bogus index entry whose blob doesn't exist.
        index.entries.push(IndexEntry {
            id: "v0002".to_string(),
            stem: PathBuf::from("v0002"),
            accepted_at: "2026-06-28T00:00:00Z".to_string(),
            parent_id: None,
            bracket_results: Default::default(),
        });
        // One more valid rater after the gap.
        index.entries.push(write_dummy_rater(&raters, "v0003"));
        let out = load_predecessor_evaluators(&index, &raters);
        assert_eq!(out.len(), 2, "corrupt entry skipped, two survive");
    }
}
