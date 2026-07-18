//! Per-generation lineage checkpoint (plan task §8).
//!
//! When the orchestrator finishes `train_lineages` for a generation but
//! hasn't yet completed Tier-1/Tier-2 acceptance, a kill (SIGKILL, window
//! close, power loss) loses the gradient-descent work for that generation.
//! On a CPU smoke run that's seconds; on a long GPU run, hours.
//!
//! Granularity is per-generation, not mid-generation. Mid-`train_lineages`
//! cancellation would require plumbing a callback through every gradient
//! step and serialising Adam optimiser moments - out of scope. Checkpoint
//! location is fixed at `<raters_dir>/in_progress/` + an umbrella
//! `in_progress.json` sidecar. On normal end-of-generation (after
//! `index.save`) the directory + sidecar are deleted; presence on next
//! run signals "skip corpus + train_lineages and resume from Tier-1".
//!
//! Determinism is best-effort, not bit-identical: `generate_corpus` is
//! not bit-reproducible across hosts/threads, and we don't checkpoint the
//! corpus itself. The contract is: *if* resume succeeds, training proceeds
//! as if the kill hadn't happened *from the saved lineage pool onward*.
//! Numbers downstream may differ from an uninterrupted run; gauntlet
//! comparators are on disk so candidates that should have been accepted
//! still get a fair shake.
//!
//! Atomicity uses write-temp-then-rename on the sidecar; per-lineage
//! `.mpk` blobs go through the same `save_rater` flow the accepted-rater
//! path uses.

use crate::lineage::Lineage;
use crate::model::{Mlp, MlpConfig};
use crate::persistence::{
    load_rater, save_rater, PersistenceError, RaterMetadata,
    TrainingConfigSnapshot, RATER_FORMAT_VERSION,
};
use burn::module::AutodiffModule;
use burn::tensor::backend::AutodiffBackend;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Sidecar format version. Bumped whenever the JSON shape changes in a
/// non-backward-compatible way.
pub const CHECKPOINT_FORMAT_VERSION: u32 = 1;

/// On-disk sidecar describing the in-progress generation. Sibling files
/// `lin-{i}.mpk` + `lin-{i}.json` carry each lineage's weights and a
/// minimal metadata stub (the metadata stub is required by
/// `save_rater`/`load_rater` - we ignore most fields when round-tripping).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointSidecar {
    pub format_version: u32,
    /// Hex digest of the `RunConfig` that produced this pool. Used to
    /// refuse a resume when the caller's config differs.
    pub run_config_digest: String,
    /// Generation index (0-based, same convention as `run_training`'s
    /// `gen_idx`). On resume the orchestrator jumps straight to Tier-1
    /// for this generation.
    pub gen_idx: usize,
    /// Per-generation seed (`run_training` computes this from `seed_root`).
    /// Re-derived on resume, stored here so a check would catch
    /// drift.
    pub gen_seed: u64,
    /// `MlpConfig` snapshot - needed to build the load skeleton.
    pub model_config: MlpConfig,
    /// Per-lineage metadata. Index order = lineage index = `Lineage.id`.
    pub lineages: Vec<LineageStub>,
}

/// Per-lineage stub stored in the sidecar. The weights live in
/// `<dir>/lin-{i}.mpk`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageStub {
    pub id: usize,
    pub seed: u64,
    pub loss_history: Vec<f32>,
    /// Path stem relative to the checkpoint directory. Always
    /// `lin-{id}` - kept explicit so a future rename doesn't silently
    /// break old checkpoints.
    pub stem: String,
}

/// Errors from checkpoint I/O. Wraps the persistence + IO error types so
/// the orchestrator can propagate them via the existing `RunError` arms.
#[derive(Debug)]
pub enum CheckpointError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Persistence(PersistenceError),
    FormatVersionMismatch { found: u32, expected: u32 },
    DigestMismatch { found: String, expected: String },
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::Persistence(e) => write!(f, "persistence error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "checkpoint format version {} not supported (expected {})",
                found, expected,
            ),
            Self::DigestMismatch { found, expected } => write!(
                f, "checkpoint config digest mismatch (found {}, expected {})",
                found, expected,
            ),
        }
    }
}

impl std::error::Error for CheckpointError {}

impl From<std::io::Error> for CheckpointError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<serde_json::Error> for CheckpointError { fn from(e: serde_json::Error) -> Self { Self::Json(e) } }
impl From<PersistenceError> for CheckpointError { fn from(e: PersistenceError) -> Self { Self::Persistence(e) } }

/// Directory holding the in-progress blobs, sibling to the umbrella
/// sidecar. `<raters_dir>/in_progress/`.
pub fn checkpoint_dir(raters_dir: &Path) -> PathBuf {
    raters_dir.join("in_progress")
}

/// Sidecar path: `<raters_dir>/in_progress.json`.
pub fn checkpoint_sidecar_path(raters_dir: &Path) -> PathBuf {
    raters_dir.join("in_progress.json")
}

/// Save a per-generation lineage pool. Writes one `.mpk` + minimal
/// `.json` per lineage via the existing `save_rater` flow (with autograd
/// stripped via `valid()`), plus an umbrella sidecar describing the pool.
///
/// Atomicity: blobs are written sequentially; the sidecar is written
/// last via temp-then-rename so an interrupted save leaves no sidecar
/// (and therefore no resume).
pub fn save_lineages<B: AutodiffBackend>(
    lineages: &[Lineage<B>],
    raters_dir: &Path,
    gen_idx: usize,
    gen_seed: u64,
    model_config: &MlpConfig,
    run_config_digest: &str,
) -> Result<(), CheckpointError>
where
    Mlp<B>: Clone,
{
    let dir = checkpoint_dir(raters_dir);
    std::fs::create_dir_all(&dir)?;

    let mut stubs = Vec::with_capacity(lineages.len());
    for lin in lineages {
        let stem_str = format!("lin-{}", lin.id);
        let stem = dir.join(&stem_str);
        // Strip autograd; save_rater is generic over `B: Backend` but
        // autodiff-side serialisation isn't a supported path. The
        // round-trip is: training-backend → valid() → inner backend
        // blob → load_rater::<Autodiff> on resume builds a fresh
        // autodiff skeleton and loads the blob into it.
        let inference_model = lin.model.clone().valid();
        // Minimal metadata stub. Most fields are irrelevant for the
        // checkpoint round-trip (we keep them so `save_rater`'s sidecar
        // shape is consistent and `load_rater` can validate the
        // format-version header). The fields we actually use later live
        // in the umbrella `CheckpointSidecar`.
        let metadata = RaterMetadata {
            format_version: RATER_FORMAT_VERSION,
            model_config: model_config.clone(),
            lineage_id: format!("ckpt-g{}-l{}", gen_idx, lin.id),
            parent_id: None,
            training_step_count: lin.loss_history.len() as u64,
            perturbation_history: Vec::new(),
            bracket_results: Default::default(),
            training_config: TrainingConfigSnapshot {
                learning_rate: 0.0,
                batch_size: 0,
                epochs: 0,
            },
            git_sha: String::new(),
            created_at: String::new(),
            eval_scale: 0.0,
        };
        save_rater::<B::InnerBackend>(&inference_model, &stem, &metadata)?;
        stubs.push(LineageStub {
            id: lin.id,
            seed: lin.seed,
            loss_history: lin.loss_history.clone(),
            stem: stem_str,
        });
    }

    let sidecar = CheckpointSidecar {
        format_version: CHECKPOINT_FORMAT_VERSION,
        run_config_digest: run_config_digest.to_string(),
        gen_idx,
        gen_seed,
        model_config: model_config.clone(),
        lineages: stubs,
    };
    write_sidecar_atomic(raters_dir, &sidecar)?;
    Ok(())
}

/// Load a previously-saved lineage pool. Returns `Ok(None)` when no
/// sidecar is present (the common case - no resume needed). Returns
/// `Err(DigestMismatch)` when the sidecar exists but the caller's
/// `RunConfig` digest differs from the one on disk; the orchestrator
/// quarantines the stale checkpoint and falls through to a fresh
/// generation.
pub fn load_lineages<B: AutodiffBackend>(
    raters_dir: &Path,
    expected_digest: &str,
    device: &B::Device,
) -> Result<Option<ResumeState<B>>, CheckpointError> {
    let sidecar_path = checkpoint_sidecar_path(raters_dir);
    let json = match std::fs::read_to_string(&sidecar_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let sidecar: CheckpointSidecar = serde_json::from_str(&json)?;
    if sidecar.format_version != CHECKPOINT_FORMAT_VERSION {
        return Err(CheckpointError::FormatVersionMismatch {
            found: sidecar.format_version,
            expected: CHECKPOINT_FORMAT_VERSION,
        });
    }
    if sidecar.run_config_digest != expected_digest {
        return Err(CheckpointError::DigestMismatch {
            found: sidecar.run_config_digest,
            expected: expected_digest.to_string(),
        });
    }

    let dir = checkpoint_dir(raters_dir);
    let mut lineages = Vec::with_capacity(sidecar.lineages.len());
    for stub in &sidecar.lineages {
        let stem = dir.join(&stub.stem);
        // load_rater::<B> with the autodiff backend builds a skeleton
        // via `MlpConfig::init::<B>(device)` and loads the inference-
        // recorded record into it. burn's NamedMpkFileRecorder is
        // backend-agnostic at the wire level - the parameters
        // re-acquire autograd hooks when loaded into the autodiff
        // skeleton.
        let (model, _meta) = load_rater::<B>(&stem, device)?;
        lineages.push(Lineage::<B> {
            id: stub.id,
            model,
            seed: stub.seed,
            loss_history: stub.loss_history.clone(),
        });
    }

    Ok(Some(ResumeState { gen_idx: sidecar.gen_idx, gen_seed: sidecar.gen_seed, lineages }))
}

/// State recovered from a checkpoint. The orchestrator consults
/// `gen_idx` + `gen_seed` to skip corpus generation and `train_lineages`
/// for the saved generation, jumping straight to Tier-1.
pub struct ResumeState<B: AutodiffBackend> {
    pub gen_idx: usize,
    pub gen_seed: u64,
    pub lineages: Vec<Lineage<B>>,
}

/// Delete a checkpoint after the generation that produced it has been
/// accepted (or rejected - either way, the work is no longer in
/// progress). Idempotent: missing directory / sidecar is a no-op.
pub fn clear_lineages(raters_dir: &Path) -> Result<(), CheckpointError> {
    let dir = checkpoint_dir(raters_dir);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    let sidecar = checkpoint_sidecar_path(raters_dir);
    if sidecar.exists() {
        std::fs::remove_file(&sidecar)?;
    }
    Ok(())
}

/// Move a stale checkpoint aside (config-digest mismatch) so the
/// orchestrator can start the generation fresh without losing the
/// evidence. Renames to `in_progress.{stem,json}.stale-{epoch_ms}`.
pub fn quarantine_stale(raters_dir: &Path) -> Result<(), CheckpointError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let dir = checkpoint_dir(raters_dir);
    if dir.exists() {
        let dst = raters_dir.join(format!("in_progress.stale-{}", ms));
        std::fs::rename(&dir, &dst)?;
    }
    let sidecar = checkpoint_sidecar_path(raters_dir);
    if sidecar.exists() {
        let dst = raters_dir.join(format!("in_progress.json.stale-{}", ms));
        std::fs::rename(&sidecar, &dst)?;
    }
    Ok(())
}

/// Write the umbrella sidecar atomically (temp-then-rename). On Unix,
/// rename within the same filesystem is atomic; on Windows the
/// `std::fs::rename` semantics replace the target. Either way, an
/// interrupted write leaves no sidecar - and therefore no resume.
fn write_sidecar_atomic(
    raters_dir: &Path,
    sidecar: &CheckpointSidecar,
) -> Result<(), CheckpointError> {
    let final_path = checkpoint_sidecar_path(raters_dir);
    let tmp_path = raters_dir.join("in_progress.json.tmp");
    let json = serde_json::to_string_pretty(sidecar)?;
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::TrainingBackend as B;
    use crate::lineage::Lineage;
    use crate::model::MlpConfig;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tempdir() -> PathBuf {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let n = NONCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("nn_trainer_ckpt_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fresh_lineages(
        n: usize,
        device: &burn::backend::ndarray::NdArrayDevice,
    ) -> Vec<Lineage<B>> {
        let cfg = MlpConfig::new();
        (0..n).map(|i| Lineage::<B>::new(i, (i as u64) + 1, &cfg, device)).collect()
    }

    #[test]
    fn save_then_load_round_trips_lineage_ids_and_seeds() {
        let raters = tempdir();
        std::fs::create_dir_all(&raters).unwrap();
        let device = Default::default();
        let cfg = MlpConfig::new();
        let lineages = fresh_lineages(3, &device);

        save_lineages::<B>(&lineages, &raters, 7, 0xDEAD, &cfg, "abc123")
            .expect("save lineages");
        let loaded = load_lineages::<B>(&raters, "abc123", &device)
            .expect("load lineages")
            .expect("checkpoint present");
        assert_eq!(loaded.gen_idx, 7);
        assert_eq!(loaded.gen_seed, 0xDEAD);
        assert_eq!(loaded.lineages.len(), 3);
        for (i, l) in loaded.lineages.iter().enumerate() {
            assert_eq!(l.id, i);
            assert_eq!(l.seed, (i as u64) + 1);
        }
    }

    #[test]
    fn load_missing_checkpoint_returns_none() {
        let raters = tempdir();
        std::fs::create_dir_all(&raters).unwrap();
        let device = Default::default();
        let r = load_lineages::<B>(&raters, "xx", &device).expect("clean miss");
        assert!(r.is_none());
    }

    #[test]
    fn digest_mismatch_is_an_error() {
        let raters = tempdir();
        std::fs::create_dir_all(&raters).unwrap();
        let device = Default::default();
        let cfg = MlpConfig::new();
        let lineages = fresh_lineages(2, &device);
        save_lineages::<B>(&lineages, &raters, 0, 0, &cfg, "first").expect("save");
        let err = load_lineages::<B>(&raters, "second", &device).err().expect("expected error");
        match err {
            CheckpointError::DigestMismatch { .. } => {}
            other => panic!("expected DigestMismatch, got {:?}", other),
        }
    }

    #[test]
    fn clear_is_idempotent() {
        let raters = tempdir();
        std::fs::create_dir_all(&raters).unwrap();
        clear_lineages(&raters).expect("clear empty");
        clear_lineages(&raters).expect("clear empty again");

        let device = Default::default();
        let cfg = MlpConfig::new();
        let lineages = fresh_lineages(1, &device);
        save_lineages::<B>(&lineages, &raters, 0, 0, &cfg, "x").expect("save");
        assert!(checkpoint_sidecar_path(&raters).exists());
        clear_lineages(&raters).expect("clear after save");
        assert!(!checkpoint_sidecar_path(&raters).exists());
        assert!(!checkpoint_dir(&raters).exists());
    }

    #[test]
    fn quarantine_renames_aside() {
        let raters = tempdir();
        std::fs::create_dir_all(&raters).unwrap();
        let device = Default::default();
        let cfg = MlpConfig::new();
        let lineages = fresh_lineages(1, &device);
        save_lineages::<B>(&lineages, &raters, 0, 0, &cfg, "x").expect("save");
        quarantine_stale(&raters).expect("quarantine");
        assert!(!checkpoint_sidecar_path(&raters).exists());
        assert!(!checkpoint_dir(&raters).exists());
        // At least one stale-* entry should now sit beside the (now-absent) checkpoint.
        let stale_count = std::fs::read_dir(&raters)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("stale-"))
            .count();
        assert!(stale_count >= 1, "expected stale-* siblings after quarantine");
    }
}
