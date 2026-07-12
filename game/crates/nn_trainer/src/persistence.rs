//! Versioned rater persistence.
//!
//! A rater on disk = two files sharing a stem:
//!
//! - `<stem>.mpk`  — weight blob via burn's `NamedMpkFileRecorder` (msgpack).
//! - `<stem>.json` — `RaterMetadata` sidecar: topology, lineage provenance,
//!   training stats, hyperparameters, git SHA, ISO-8601 date.
//!
//! Why split? The weights blob is opaque binary; the metadata is human-
//! readable. The sidecar carries the `MlpConfig` so a loader can reconstruct
//! the right-shaped skeleton before calling `load_record` (burn needs the
//! topology in hand to deserialise).
//!
//! Plan §9 calls for `raters/v0042.bin` + JSON sidecar — same idea. The
//! registry that picks which version is "best-fast / best-slow / best-overall"
//! is sub-slice 6b; this module is only concerned with one rater's pair of
//! files.
//!
//! ## Determinism note
//!
//! `save_rater(model)` followed by `load_rater(...)` reconstitutes a model
//! whose `forward()` produces bit-identical outputs to the original for any
//! given input (modulo backend determinism — `NdArray<f32>` is deterministic
//! on a given machine). This is the property the gauntlet needs to compare a
//! reloaded champion against a fresh candidate.

use crate::model::{Mlp, MlpConfig};
use crate::train::TrainingConfig;

use burn::module::Module;
use burn::record::{FullPrecisionSettings, NamedMpkFileRecorder, Recorder};
use burn::tensor::backend::Backend;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Disk-format version. Bump when the layout of the .mpk blob or the JSON
/// sidecar changes incompatibly so older raters still error cleanly.
pub const RATER_FORMAT_VERSION: u32 = 1;

/// Win-rate against the immediate predecessor in a given think-time bracket.
/// Optional — only present once the gauntlet has produced a verdict.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BracketWinRate {
    pub games_played: u32,
    pub candidate_wins: u32,
    pub baseline_wins: u32,
    pub indecisive: u32,
}

/// One entry in a rater's perturbation history — recorded each time the
/// "perturb and keep best" loop accepted a noisy candidate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerturbationEvent {
    /// Lineage round at which this perturbation was accepted.
    pub round: u32,
    /// Gaussian std-dev used for the noise injection.
    pub std_dev: f32,
    /// Seed fed to the noise RNG. Reproducibility hook.
    pub seed: u64,
}

/// Provenance + stats for one accepted rater version.
///
/// Designed to be append-only: new fields use `#[serde(default)]` so older
/// JSON sidecars still parse cleanly after the schema grows.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaterMetadata {
    /// Sidecar schema version. Mismatch with `RATER_FORMAT_VERSION` aborts
    /// the load.
    pub format_version: u32,

    /// Topology of the saved model. Used by `load_rater` to rebuild the
    /// skeleton before `load_record` fills in the weights.
    pub model_config: MlpConfig,

    /// Lineage identifier (e.g. `"l3-r12"` for lineage 3, round 12). Free-
    /// form; the registry in 6b assigns it.
    pub lineage_id: String,

    /// Parent rater (the predecessor this one was forked or trained from).
    /// `None` for the very first rater.
    #[serde(default)]
    pub parent_id: Option<String>,

    /// Total gradient steps taken across this rater's training history.
    pub training_step_count: u64,

    /// Perturbation events in temporal order. Empty for raters trained
    /// without the perturb-and-keep-best loop.
    #[serde(default)]
    pub perturbation_history: Vec<PerturbationEvent>,

    /// Win-rates from the gauntlet that accepted this rater, keyed by
    /// bracket name (`"fast"`, `"medium"`, `"slow"`). Empty until the
    /// gauntlet has run.
    #[serde(default)]
    pub bracket_results: std::collections::BTreeMap<String, BracketWinRate>,

    /// Echo of the hyperparameters used during training. Captured for
    /// reproducibility; the trainer itself doesn't reload these.
    pub training_config: TrainingConfigSnapshot,

    /// Git SHA at the time of save. Empty string if not in a git checkout.
    #[serde(default)]
    pub git_sha: String,

    /// ISO-8601 UTC timestamp at the moment of save.
    pub created_at: String,

    /// Centipawn-scale conversion factor fitted by the calibration pass
    /// against the heuristic. `0.0` means "not yet calibrated" — callers
    /// fall back to `DEFAULT_EVAL_SCALE`. `#[serde(default)]` so older
    /// sidecars without the field deserialise cleanly.
    #[serde(default)]
    pub eval_scale: f32,
}

/// Serialisable mirror of `train::TrainingConfig`. We keep the field set
/// fixed here so adding a `TrainingConfig` field doesn't silently break old
/// sidecars — the mirror is the schema, the live struct is the runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingConfigSnapshot {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub epochs: usize,
}

impl From<&TrainingConfig> for TrainingConfigSnapshot {
    fn from(c: &TrainingConfig) -> Self {
        Self {
            learning_rate: c.learning_rate,
            batch_size: c.batch_size,
            epochs: c.epochs,
        }
    }
}

/// Errors from save/load. Wraps the underlying IO / serde / burn errors so
/// callers don't need to depend on burn types directly.
#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Record(String),
    FormatVersionMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::Record(e) => write!(f, "record error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "rater format version {} not supported (expected {})",
                found, expected,
            ),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

/// Derive the two file paths from a stem. `"raters/v0042"` →
/// `("raters/v0042.mpk", "raters/v0042.json")`.
pub fn paths_from_stem(stem: &Path) -> (PathBuf, PathBuf) {
    // burn's recorder appends ".mpk" itself when given a stem; we pass the
    // stem unmodified to keep the two sides aligned.
    let blob = stem.with_extension("mpk");
    let sidecar = stem.with_extension("json");
    (blob, sidecar)
}

/// Write a rater to `<stem>.mpk` + `<stem>.json`. Creates parent directories.
pub fn save_rater<B: Backend>(
    model: &Mlp<B>,
    stem: &Path,
    metadata: &RaterMetadata,
) -> Result<(), PersistenceError> {
    if let Some(parent) = stem.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let (_blob_path, sidecar_path) = paths_from_stem(stem);

    // burn appends the extension; we pass the stem.
    let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
    recorder
        .record(model.clone().into_record(), stem.to_path_buf())
        .map_err(|e| PersistenceError::Record(format!("{:?}", e)))?;

    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(sidecar_path, json)?;
    Ok(())
}

/// Read only the sidecar metadata (`<stem>.json`) without loading the weight
/// blob. Cheap topology/provenance peek — used by the in-game AI load path to
/// decide whether a rater is dense (`NnEvaluator`) or sparse/NNUE
/// (`NnueEvaluator`) via `model_config.input_dim`.
pub fn load_metadata(stem: &Path) -> Result<RaterMetadata, PersistenceError> {
    let (_blob_path, sidecar_path) = paths_from_stem(stem);
    let json = std::fs::read_to_string(&sidecar_path)?;
    let metadata: RaterMetadata = serde_json::from_str(&json)?;
    if metadata.format_version != RATER_FORMAT_VERSION {
        return Err(PersistenceError::FormatVersionMismatch {
            found: metadata.format_version,
            expected: RATER_FORMAT_VERSION,
        });
    }
    Ok(metadata)
}

/// Load a rater. Reads the sidecar first to recover the topology, then loads
/// the weight blob into a freshly-built skeleton.
pub fn load_rater<B: Backend>(
    stem: &Path,
    device: &B::Device,
) -> Result<(Mlp<B>, RaterMetadata), PersistenceError> {
    let (_blob_path, sidecar_path) = paths_from_stem(stem);
    let json = std::fs::read_to_string(&sidecar_path)?;
    let metadata: RaterMetadata = serde_json::from_str(&json)?;
    if metadata.format_version != RATER_FORMAT_VERSION {
        return Err(PersistenceError::FormatVersionMismatch {
            found: metadata.format_version,
            expected: RATER_FORMAT_VERSION,
        });
    }

    let skeleton: Mlp<B> = metadata.model_config.init(device);
    let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
    let record = recorder
        .load(stem.to_path_buf(), device)
        .map_err(|e| PersistenceError::Record(format!("{:?}", e)))?;
    let model = skeleton.load_record(record);
    Ok((model, metadata))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::InferenceBackend as B;
    use crate::encoding::{encode_position, INPUT_DIM};
    use burn::tensor::{Tensor, TensorData};
    use core_engine::state::Position;

    fn sample_metadata() -> RaterMetadata {
        RaterMetadata {
            format_version: RATER_FORMAT_VERSION,
            model_config: MlpConfig::new(),
            lineage_id: "l0-r0".to_string(),
            parent_id: None,
            training_step_count: 100,
            perturbation_history: vec![],
            bracket_results: Default::default(),
            training_config: TrainingConfigSnapshot {
                learning_rate: 1e-3,
                batch_size: 64,
                epochs: 5,
            },
            git_sha: String::new(),
            created_at: "2026-06-27T12:00:00Z".to_string(),
            eval_scale: 0.0,
        }
    }

    fn probe_output<Bk: Backend>(model: &Mlp<Bk>, device: &Bk::Device) -> f32 {
        let pos = Position::setup_stack_m();
        let features = encode_position(&pos);
        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<Bk, 2> = Tensor::from_data(data, device);
        let out = model.forward(input);
        out.into_data().to_vec::<f32>().unwrap()[0]
    }

    #[test]
    fn save_and_load_roundtrips_outputs() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);

        let before = probe_output(&model, &device);

        let tmp = tempdir();
        let stem = tmp.join("v0001");
        save_rater(&model, &stem, &sample_metadata()).expect("save");

        let (reloaded, meta) = load_rater::<B>(&stem, &device).expect("load");
        let after = probe_output(&reloaded, &device);

        assert_eq!(meta.format_version, RATER_FORMAT_VERSION);
        assert_eq!(meta.lineage_id, "l0-r0");
        assert!((before - after).abs() < 1e-6,
            "reloaded model output diverged: {} vs {}", before, after);
    }

    #[test]
    fn load_rejects_wrong_format_version() {
        let device = Default::default();
        let model: Mlp<B> = MlpConfig::new().init(&device);

        let tmp = tempdir();
        let stem = tmp.join("v0002");
        let mut meta = sample_metadata();
        meta.format_version = 999;
        save_rater(&model, &stem, &meta).expect("save");

        let err = load_rater::<B>(&stem, &device).expect_err("must reject");
        assert!(matches!(err, PersistenceError::FormatVersionMismatch { found: 999, expected: 1 }));
    }

    #[test]
    fn load_missing_sidecar_errors() {
        let device = Default::default();
        let tmp = tempdir();
        let stem = tmp.join("nonexistent");
        let err = load_rater::<B>(&stem, &device).expect_err("must error");
        assert!(matches!(err, PersistenceError::Io(_)));
    }

    #[test]
    fn custom_topology_roundtrips() {
        let device = Default::default();
        let cfg = MlpConfig::new().with_hidden_sizes(vec![32, 16]);
        let model: Mlp<B> = cfg.init(&device);

        let before = probe_output(&model, &device);

        let tmp = tempdir();
        let stem = tmp.join("v0003");
        let mut meta = sample_metadata();
        meta.model_config = MlpConfig::new().with_hidden_sizes(vec![32, 16]);
        save_rater(&model, &stem, &meta).expect("save");

        let (reloaded, meta2) = load_rater::<B>(&stem, &device).expect("load");
        assert_eq!(meta2.model_config.hidden_sizes, vec![32, 16]);
        let after = probe_output(&reloaded, &device);
        assert!((before - after).abs() < 1e-6);
    }

    /// Inline temp-dir helper. We don't pull in the `tempfile` crate for one
    /// test — just create a unique subdir under the OS temp root and let the
    /// test runner clean up. Returns the directory; each test gets a fresh
    /// nonce so parallel tests don't collide.
    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let n = NONCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir()
            .join(format!("nn_trainer_persistence_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
