//! Gauntlet match-matrix persistence (plan §9, §10 panel 5).
//!
//! `<run_dir>/matrix.json` holds the NxN grid of every-version-vs-every-
//! version match results, keyed by `(challenger, defender, bracket)`. The
//! orchestrator updates an entry every time a series finishes; the UI
//! polls the file at low cadence to render the gauntlet-matrix panel.
//!
//! ## Why a flat `Vec` and not a `BTreeMap`
//!
//! Serde / JSON can't round-trip a `BTreeMap` whose key is a tuple - JSON
//! objects only accept string keys. We could flatten the key into a
//! `"challenger|defender|bracket"` string, but that wedges a delimiter
//! into rater IDs forever. A flat `Vec<MatrixEntry>` is simpler, diff-
//! readable, and the `O(N²)` linear scan is fine at the scale we expect
//! (panels render after hundreds of accepted raters, not millions).
//!
//! ## Atomic write
//!
//! `.tmp` + rename, same trick as `snapshot.rs` and `live.rs`. A polling
//! UI never observes a half-written file.

use crate::gauntlet::SeriesTally;
use crate::persistence::BracketWinRate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Schema version. Bump on incompatible changes.
pub const MATRIX_FORMAT_VERSION: u32 = 1;

/// Default filename inside the run directory.
pub const MATRIX_FILENAME: &str = "matrix.json";

/// One cell in the gauntlet matrix - outcomes of (challenger vs defender)
/// at one bracket. Bracket is a free-form string (`"fast" | "medium" |
/// "slow"`) so the schema can absorb new brackets without a code change.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatrixEntry {
    pub challenger: String,
    pub defender: String,
    pub bracket: String,
    pub result: BracketWinRate,
}

/// The whole matrix.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GauntletMatrix {
    pub format_version: u32,
    #[serde(default)]
    pub entries: Vec<MatrixEntry>,
}

impl Default for GauntletMatrix {
    fn default() -> Self {
        Self {
            format_version: MATRIX_FORMAT_VERSION,
            entries: Vec::new(),
        }
    }
}

/// Errors from matrix save / load.
#[derive(Debug)]
pub enum MatrixError {
    Io(std::io::Error),
    Json(serde_json::Error),
    FormatVersionMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for MatrixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "matrix format version {} not supported (expected {})",
                found, expected,
            ),
        }
    }
}

impl std::error::Error for MatrixError {}

impl From<std::io::Error> for MatrixError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for MatrixError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

impl GauntletMatrix {
    /// Find a cell. `None` if the orchestrator hasn't recorded it yet.
    pub fn get(&self, challenger: &str, defender: &str, bracket: &str)
        -> Option<&MatrixEntry>
    {
        self.entries.iter().find(|e| {
            e.challenger == challenger
                && e.defender == defender
                && e.bracket == bracket
        })
    }

    /// Append a series tally into the cell `(challenger, defender, bracket)`.
    /// Idempotent in the sense that re-recording the same series adds to the
    /// existing counters - callers that want overwrite semantics should
    /// remove the entry first or build a fresh matrix.
    ///
    /// The plan calls for an NxN grid where each cell is a single series,
    /// but the orchestrator may run a candidate against a baseline multiple
    /// times across generations (different mirror seeds, different perturbs).
    /// Accumulating is the right default: the win-rate stabilises as more
    /// data lands.
    pub fn record_series(
        &mut self,
        challenger: &str,
        defender: &str,
        bracket: &str,
        tally: SeriesTally,
    ) {
        if let Some(existing) = self.entries.iter_mut().find(|e| {
            e.challenger == challenger
                && e.defender == defender
                && e.bracket == bracket
        }) {
            existing.result.games_played += tally.games_played();
            existing.result.candidate_wins += tally.candidate_wins;
            existing.result.baseline_wins += tally.baseline_wins;
            existing.result.indecisive += tally.indecisive;
        } else {
            self.entries.push(MatrixEntry {
                challenger: challenger.to_string(),
                defender: defender.to_string(),
                bracket: bracket.to_string(),
                result: BracketWinRate {
                    games_played: tally.games_played(),
                    candidate_wins: tally.candidate_wins,
                    baseline_wins: tally.baseline_wins,
                    indecisive: tally.indecisive,
                },
            });
        }
    }
}

/// Load `<dir>/matrix.json`. Returns an empty matrix if the file doesn't
/// exist (bootstrap-friendly), errors on schema-version mismatch.
pub fn load_matrix(dir: &Path) -> Result<GauntletMatrix, MatrixError> {
    let path = dir.join(MATRIX_FILENAME);
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            let m: GauntletMatrix = serde_json::from_str(&json)?;
            if m.format_version != MATRIX_FORMAT_VERSION {
                return Err(MatrixError::FormatVersionMismatch {
                    found: m.format_version,
                    expected: MATRIX_FORMAT_VERSION,
                });
            }
            Ok(m)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GauntletMatrix::default()),
        Err(e) => Err(MatrixError::Io(e)),
    }
}

/// Write `<dir>/matrix.json` atomically.
pub fn save_matrix(dir: &Path, m: &GauntletMatrix) -> Result<(), MatrixError> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(m)?;

    let final_path = dir.join(MATRIX_FILENAME);
    let tmp_path: PathBuf = dir.join(format!("{}.tmp", MATRIX_FILENAME));
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tempdir() -> PathBuf {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let n = NONCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir()
            .join(format!("nn_trainer_matrix_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn series(c: u32, b: u32, i: u32) -> SeriesTally {
        SeriesTally {
            candidate_wins: c,
            baseline_wins: b,
            indecisive: i,
        }
    }

    #[test]
    fn load_missing_returns_empty() {
        let dir = tempdir();
        let m = load_matrix(&dir).expect("load");
        assert!(m.entries.is_empty());
        assert_eq!(m.format_version, MATRIX_FORMAT_VERSION);
    }

    #[test]
    fn record_then_get_roundtrip() {
        let mut m = GauntletMatrix::default();
        m.record_series("v0002", "v0001", "fast", series(2, 1, 0));
        m.record_series("v0002", "v0001", "medium", series(3, 0, 0));
        m.record_series("v0003", "v0001", "fast", series(2, 1, 0));

        let cell = m.get("v0002", "v0001", "fast").expect("cell");
        assert_eq!(cell.result.candidate_wins, 2);
        assert_eq!(cell.result.baseline_wins, 1);
        assert_eq!(cell.result.games_played, 3);

        let medium = m.get("v0002", "v0001", "medium").expect("cell");
        assert_eq!(medium.result.candidate_wins, 3);
        assert_eq!(medium.result.games_played, 3);

        assert!(m.get("v0042", "v0001", "fast").is_none());
    }

    #[test]
    fn record_accumulates_across_runs() {
        let mut m = GauntletMatrix::default();
        m.record_series("v0002", "v0001", "fast", series(2, 1, 0));
        m.record_series("v0002", "v0001", "fast", series(1, 0, 1));

        let cell = m.get("v0002", "v0001", "fast").expect("cell");
        assert_eq!(cell.result.candidate_wins, 3);
        assert_eq!(cell.result.baseline_wins, 1);
        assert_eq!(cell.result.indecisive, 1);
        assert_eq!(cell.result.games_played, 5);
    }

    #[test]
    fn save_load_roundtrips() {
        let dir = tempdir();
        let mut m = GauntletMatrix::default();
        m.record_series("v0002", "v0001", "fast", series(2, 1, 0));
        m.record_series("v0003", "v0002", "slow", series(3, 0, 0));
        save_matrix(&dir, &m).expect("save");

        let reloaded = load_matrix(&dir).expect("load");
        assert_eq!(reloaded.entries.len(), 2);
        assert_eq!(
            reloaded.get("v0003", "v0002", "slow").unwrap().result.candidate_wins,
            3,
        );
    }

    #[test]
    fn load_rejects_wrong_version() {
        let dir = tempdir();
        let raw = r#"{ "format_version": 999, "entries": [] }"#;
        std::fs::write(dir.join(MATRIX_FILENAME), raw).unwrap();
        let err = load_matrix(&dir).expect_err("must reject");
        assert!(matches!(err, MatrixError::FormatVersionMismatch { found: 999, expected: 1 }));
    }

    #[test]
    fn rapid_writes_leave_no_tmp_file() {
        let dir = tempdir();
        for i in 0..5 {
            let mut m = GauntletMatrix::default();
            m.record_series("v0001", "v0000", "fast", series(i, 0, 0));
            save_matrix(&dir, &m).expect("save");
            let r = load_matrix(&dir).expect("load");
            assert_eq!(r.get("v0001", "v0000", "fast").unwrap().result.candidate_wins, i);
        }
        let tmp = dir.join(format!("{}.tmp", MATRIX_FILENAME));
        assert!(!tmp.exists(), "stale .tmp left behind");
    }
}
