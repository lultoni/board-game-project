//! Training status snapshot writer (plan §9, IPC).
//!
//! The trainer is the source of truth. The Training Observatory UI is a
//! passive observer that polls a JSON file at low cadence (~1 Hz). This
//! module owns the snapshot schema and the atomic-write plumbing — the
//! tournament driver populates a `StatusSnapshot` and calls
//! `write_snapshot` whenever its state advances.
//!
//! ## Atomic write
//!
//! Writes go through a `<path>.tmp` file + `rename` so a UI polling at
//! the same instant never reads a half-written file. POSIX `rename` is
//! atomic on the same filesystem, which is what we use across the board.
//!
//! ## Live-position state lives elsewhere
//!
//! This module only carries summary state — what the §10 panels 2/3 need
//! at 1 Hz. The per-ply live-position stream (panel 1) has different
//! cadence + sentinel-flag subscription requirements and lives in a
//! separate writer (sub-slice 6e).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Schema version for the status snapshot file. Bump on incompatible
/// changes; the UI loader checks this and refuses older/newer versions.
pub const STATUS_SNAPSHOT_VERSION: u32 = 1;

/// Overall training-driver phase. The UI uses this to pick which panels
/// to surface (e.g. "Idle" hides the live match view).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingPhase {
    /// No training run is active.
    Idle,
    /// Generating self-play corpus / running gradient steps.
    Training,
    /// A gauntlet round is in progress (candidate vs baseline matches).
    Gauntlet,
    /// Between rounds — bookkeeping, persistence, registry updates.
    Bookkeeping,
}

/// One member of the current generation as seen by the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopulationMember {
    /// Free-form rater identifier (e.g. lineage tag + round number).
    pub rater_id: String,
    /// Parent rater ID, or `None` for the root.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Lineage index this member belongs to.
    pub lineage: u32,
    /// Generation / round counter.
    pub generation: u32,
    /// Wins so far this round.
    pub wins: u32,
    /// Losses so far this round.
    pub losses: u32,
    /// Draws / indecisive games this round.
    pub draws: u32,
    /// `false` once the gauntlet eliminated this member from the round.
    pub alive: bool,
}

/// State of the match currently being played, if any. Panel 1 uses this
/// for its header line and eval bars; the actual board state streams
/// separately through the live-position writer (6e).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveMatch {
    /// Challenger rater ID.
    pub challenger: String,
    /// Defender rater ID.
    pub defender: String,
    /// 1-based game number within the current best-of-N series.
    pub game_index: u32,
    /// Total games in the series.
    pub games_total: u32,
    /// Current ply number (P1 = 0, P2 = 1, …).
    pub ply: u32,
    /// Think budget per move in milliseconds.
    pub think_ms: u32,
    /// Bracket label — `"fast" | "medium" | "slow"`.
    pub bracket: String,
}

/// One full snapshot of training state. Trainer writes this at ~1 Hz.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusSnapshot {
    /// Schema version — checked against `STATUS_SNAPSHOT_VERSION` on load.
    pub format_version: u32,
    /// UNIX epoch milliseconds when the snapshot was written. Lets the UI
    /// detect a stalled trainer ("snapshot is 30s old → training died").
    pub written_at_ms: u64,
    /// Current driver phase.
    pub phase: TrainingPhase,
    /// 1-based generation / round counter, 0 if idle.
    pub generation: u32,
    /// Round-within-generation counter, 0 if not in a multi-round phase.
    pub round: u32,
    /// Estimated seconds until the current round / generation finishes.
    /// `None` if the driver has no estimate yet.
    #[serde(default)]
    pub eta_seconds: Option<u64>,
    /// Current population. Empty when phase = Idle.
    #[serde(default)]
    pub population: Vec<PopulationMember>,
    /// The match the trainer is playing right now, if any.
    #[serde(default)]
    pub active_match: Option<ActiveMatch>,
}

impl StatusSnapshot {
    /// Idle snapshot — used at startup before the driver has anything to
    /// report, and again when a run finishes.
    pub fn idle() -> Self {
        Self {
            format_version: STATUS_SNAPSHOT_VERSION,
            written_at_ms: now_ms(),
            phase: TrainingPhase::Idle,
            generation: 0,
            round: 0,
            eta_seconds: None,
            population: Vec::new(),
            active_match: None,
        }
    }
}

/// Errors from the snapshot writer / loader.
#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
    Json(serde_json::Error),
    FormatVersionMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "snapshot format version {} not supported (expected {})",
                found, expected,
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<std::io::Error> for SnapshotError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for SnapshotError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

/// Default filename — trainer writes `<run_dir>/status.json`.
pub const STATUS_FILENAME: &str = "status.json";

/// Write `snapshot` to `<dir>/status.json` atomically (`.tmp` + rename).
/// Always stamps `written_at_ms = now_ms()` before serialising so the
/// freshness check on the UI side is honest. Creates `dir` if missing.
pub fn write_snapshot(dir: &Path, snapshot: &StatusSnapshot) -> Result<(), SnapshotError> {
    std::fs::create_dir_all(dir)?;
    let mut stamped = snapshot.clone();
    stamped.written_at_ms = now_ms();
    let json = serde_json::to_string_pretty(&stamped)?;

    let final_path = dir.join(STATUS_FILENAME);
    let tmp_path: PathBuf = dir.join(format!("{}.tmp", STATUS_FILENAME));
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

/// Read `<dir>/status.json`. Returns `None` if the file doesn't exist
/// (trainer hasn't started yet), errors on version mismatch.
pub fn read_snapshot(dir: &Path) -> Result<Option<StatusSnapshot>, SnapshotError> {
    let path = dir.join(STATUS_FILENAME);
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            let snap: StatusSnapshot = serde_json::from_str(&json)?;
            if snap.format_version != STATUS_SNAPSHOT_VERSION {
                return Err(SnapshotError::FormatVersionMismatch {
                    found: snap.format_version,
                    expected: STATUS_SNAPSHOT_VERSION,
                });
            }
            Ok(Some(snap))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(SnapshotError::Io(e)),
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
            .join(format!("nn_trainer_snapshot_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            format_version: STATUS_SNAPSHOT_VERSION,
            written_at_ms: 0, // overwritten by writer
            phase: TrainingPhase::Gauntlet,
            generation: 7,
            round: 3,
            eta_seconds: Some(252),
            population: vec![
                PopulationMember {
                    rater_id: "l0-r3".into(),
                    parent_id: Some("l0-r2".into()),
                    lineage: 0,
                    generation: 7,
                    wins: 4,
                    losses: 1,
                    draws: 0,
                    alive: true,
                },
                PopulationMember {
                    rater_id: "l1-r3".into(),
                    parent_id: Some("l1-r2".into()),
                    lineage: 1,
                    generation: 7,
                    wins: 0,
                    losses: 5,
                    draws: 0,
                    alive: false,
                },
            ],
            active_match: Some(ActiveMatch {
                challenger: "l0-r3".into(),
                defender: "l0-r2".into(),
                game_index: 3,
                games_total: 6,
                ply: 47,
                think_ms: 300,
                bracket: "medium".into(),
            }),
        }
    }

    #[test]
    fn write_and_read_roundtrip() {
        let dir = tempdir();
        let snap = sample_snapshot();
        write_snapshot(&dir, &snap).expect("write");

        let loaded = read_snapshot(&dir).expect("read").expect("present");
        assert_eq!(loaded.format_version, STATUS_SNAPSHOT_VERSION);
        assert_eq!(loaded.phase, TrainingPhase::Gauntlet);
        assert_eq!(loaded.generation, 7);
        assert_eq!(loaded.round, 3);
        assert_eq!(loaded.eta_seconds, Some(252));
        assert_eq!(loaded.population.len(), 2);
        assert!(loaded.population[0].alive);
        assert!(!loaded.population[1].alive);
        let m = loaded.active_match.expect("match");
        assert_eq!(m.challenger, "l0-r3");
        assert_eq!(m.bracket, "medium");
    }

    #[test]
    fn writer_stamps_written_at_ms() {
        let dir = tempdir();
        let mut snap = sample_snapshot();
        snap.written_at_ms = 0;
        let before = now_ms();
        write_snapshot(&dir, &snap).expect("write");
        let after = now_ms();

        let loaded = read_snapshot(&dir).expect("read").expect("present");
        assert!(
            loaded.written_at_ms >= before && loaded.written_at_ms <= after,
            "written_at_ms = {} not in [{}, {}]",
            loaded.written_at_ms, before, after,
        );
    }

    #[test]
    fn read_missing_file_returns_none() {
        let dir = tempdir();
        let got = read_snapshot(&dir).expect("read");
        assert!(got.is_none());
    }

    #[test]
    fn read_rejects_wrong_format_version() {
        let dir = tempdir();
        let raw = r#"{
            "format_version": 999,
            "written_at_ms": 0,
            "phase": "idle",
            "generation": 0,
            "round": 0
        }"#;
        std::fs::write(dir.join(STATUS_FILENAME), raw).unwrap();
        let err = read_snapshot(&dir).expect_err("must reject");
        assert!(matches!(err, SnapshotError::FormatVersionMismatch { found: 999, expected: 1 }));
    }

    #[test]
    fn idle_snapshot_has_empty_state() {
        let snap = StatusSnapshot::idle();
        assert_eq!(snap.phase, TrainingPhase::Idle);
        assert_eq!(snap.generation, 0);
        assert!(snap.population.is_empty());
        assert!(snap.active_match.is_none());
    }

    #[test]
    fn rapid_writes_dont_leave_partial_file() {
        // Atomic-rename property: between two writes, the reader should
        // always see a fully-formed snapshot — never an empty/partial one.
        // We can't easily race a reader, but we can at least confirm
        // sequential writes always reload to valid state, and the .tmp
        // file is cleaned up.
        let dir = tempdir();
        for i in 0..10 {
            let mut snap = sample_snapshot();
            snap.generation = i;
            write_snapshot(&dir, &snap).expect("write");
            let loaded = read_snapshot(&dir).expect("read").expect("present");
            assert_eq!(loaded.generation, i);
        }
        // .tmp must not linger after a successful rename.
        let tmp = dir.join(format!("{}.tmp", STATUS_FILENAME));
        assert!(!tmp.exists(), "stale .tmp file left behind");
    }
}
