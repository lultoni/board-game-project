//! Live-position sentinel + per-ply state writer (plan §9, IPC).
//!
//! Where `snapshot.rs` handles summary state at ~1 Hz, this module handles
//! the per-ply board stream that Panel 1 (Live Match View) renders. The
//! design constraints are tighter:
//!
//! - **Subscription via sentinel file.** The UI writes a `live.sub` file
//!   while Panel 1 is focused and visible; the trainer checks for that
//!   file once per ply and only writes live-position state when present.
//!   When unfocused → file gone → no live-position writes → full-speed
//!   self-play resumes. The UI cannot pause training, only stop watching.
//!
//! - **Cadence.** Per ply during a match. The writer is called from the
//!   self-play loop's `step` boundary; if the sentinel is absent, the
//!   call is a noop. No locks, no waits.
//!
//! - **Atomicity.** Same `.tmp + rename` trick as `snapshot::write_snapshot`.
//!   A UI polling at the same instant never sees a half-written file.
//!
//! ## Files
//!
//! ```text
//! <run_dir>/live.sub       sentinel — UI creates/removes
//! <run_dir>/live.json      per-ply state — trainer writes
//! ```
//!
//! ## What this is NOT
//!
//! This isn't a replay buffer — only the *current* ply is on disk. The
//! UI catches up by reading the same file again next poll cycle. If the
//! UI wants a full game log, it should subscribe with a longer-cadence
//! summary that records ply history (out of scope here).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Schema version. Bump on incompatible changes.
pub const LIVE_POSITION_VERSION: u32 = 1;

/// Sentinel filename — the UI creates this to subscribe to live-position
/// writes, removes it to unsubscribe.
pub const LIVE_SUBSCRIBE_FILENAME: &str = "live.sub";

/// State filename — the trainer writes per-ply state here.
pub const LIVE_STATE_FILENAME: &str = "live.json";

/// Eval-bar values for one ply, from three independent evaluators
/// looking at the same position. Centipawn-scale, P1-POV (matches the
/// `Evaluator` trait sign convention).
///
/// All three fields are `Option<i32>` because at run-start the trainer
/// may not have a baseline NN loaded yet, and the heuristic call may be
/// skipped if compute matters. `None` renders as a blank bar.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct EvalBars {
    /// Score from the challenger NN evaluator.
    #[serde(default)]
    pub challenger_nn: Option<i32>,
    /// Score from the defender NN evaluator.
    #[serde(default)]
    pub defender_nn: Option<i32>,
    /// Score from the hand-coded `HeuristicEvaluator`. Control signal.
    #[serde(default)]
    pub heuristic: Option<i32>,
}

/// One ply's worth of live state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivePosition {
    /// Schema version — checked against `LIVE_POSITION_VERSION` on load.
    pub format_version: u32,
    /// UNIX epoch milliseconds at write time. UI uses this to detect a
    /// stalled stream ("file is 30s old → trainer crashed mid-game").
    pub written_at_ms: u64,
    /// FEN of the position *after* the ply was played. Empty string when
    /// the trainer wants to publish "no active match" (e.g. between games).
    pub fen: String,
    /// Free-form description of the action played to reach `fen`. The UI
    /// renders this in the header ("last action: P1 e2-e4 + Tempest").
    /// Empty for the initial position of a game.
    #[serde(default)]
    pub last_action: String,
    /// 0-based ply number within the current game.
    pub ply: u32,
    /// Challenger rater ID — same as the active match in the status snap.
    pub challenger: String,
    /// Defender rater ID.
    pub defender: String,
    /// 1-based game index within the current best-of-N series.
    pub game_index: u32,
    /// Total games in the series.
    pub games_total: u32,
    /// Eval bars for the position now on the board.
    pub evals: EvalBars,
}

/// Errors from the live-position writer / loader.
#[derive(Debug)]
pub enum LiveError {
    Io(std::io::Error),
    Json(serde_json::Error),
    FormatVersionMismatch { found: u32, expected: u32 },
}

impl std::fmt::Display for LiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "live-position format version {} not supported (expected {})",
                found, expected,
            ),
        }
    }
}

impl std::error::Error for LiveError {}

impl From<std::io::Error> for LiveError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for LiveError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

/// True iff the UI has set the subscription sentinel. Trainer calls this
/// once per ply; if `false`, skip the write entirely.
pub fn is_subscribed(dir: &Path) -> bool {
    dir.join(LIVE_SUBSCRIBE_FILENAME).exists()
}

/// UI calls this to start receiving per-ply state. Creates an empty
/// sentinel file (just the existence matters). Idempotent.
pub fn subscribe(dir: &Path) -> Result<(), LiveError> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(LIVE_SUBSCRIBE_FILENAME);
    std::fs::write(&path, b"")?;
    Ok(())
}

/// UI calls this to stop receiving per-ply state. Idempotent — removing
/// an already-absent sentinel is a noop.
pub fn unsubscribe(dir: &Path) -> Result<(), LiveError> {
    let path = dir.join(LIVE_SUBSCRIBE_FILENAME);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(LiveError::Io(e)),
    }
}

/// Write `live.json` *only if* the sentinel is present. Returns `true`
/// if a write happened, `false` if the UI is unsubscribed (the caller
/// can use this to skip downstream compute — generating eval bars etc.).
///
/// Always stamps `written_at_ms` so freshness checks are honest.
pub fn write_if_subscribed(dir: &Path, live: &LivePosition) -> Result<bool, LiveError> {
    if !is_subscribed(dir) {
        return Ok(false);
    }
    std::fs::create_dir_all(dir)?;
    let mut stamped = live.clone();
    stamped.written_at_ms = now_ms();
    let json = serde_json::to_string_pretty(&stamped)?;

    let final_path = dir.join(LIVE_STATE_FILENAME);
    let tmp_path: PathBuf = dir.join(format!("{}.tmp", LIVE_STATE_FILENAME));
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(true)
}

/// Read `live.json`. Returns `None` if the trainer hasn't written one
/// yet, errors on schema-version mismatch.
pub fn read_live(dir: &Path) -> Result<Option<LivePosition>, LiveError> {
    let path = dir.join(LIVE_STATE_FILENAME);
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            let live: LivePosition = serde_json::from_str(&json)?;
            if live.format_version != LIVE_POSITION_VERSION {
                return Err(LiveError::FormatVersionMismatch {
                    found: live.format_version,
                    expected: LIVE_POSITION_VERSION,
                });
            }
            Ok(Some(live))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(LiveError::Io(e)),
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
            .join(format!("nn_trainer_live_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_live() -> LivePosition {
        LivePosition {
            format_version: LIVE_POSITION_VERSION,
            written_at_ms: 0,
            fen: "8/8/8/8/8/8/8/8 w 0 0".to_string(),
            last_action: "P1 e2-e4 + Tempest".to_string(),
            ply: 47,
            challenger: "l0-r3".to_string(),
            defender: "l0-r2".to_string(),
            game_index: 3,
            games_total: 6,
            evals: EvalBars {
                challenger_nn: Some(120),
                defender_nn: Some(-80),
                heuristic: Some(50),
            },
        }
    }

    #[test]
    fn write_is_noop_when_unsubscribed() {
        let dir = tempdir();
        let live = sample_live();
        let wrote = write_if_subscribed(&dir, &live).expect("write");
        assert!(!wrote, "should not write without sentinel");
        assert!(!dir.join(LIVE_STATE_FILENAME).exists());
    }

    #[test]
    fn subscribe_then_write_produces_file() {
        let dir = tempdir();
        subscribe(&dir).expect("subscribe");
        assert!(is_subscribed(&dir));

        let live = sample_live();
        let wrote = write_if_subscribed(&dir, &live).expect("write");
        assert!(wrote);
        assert!(dir.join(LIVE_STATE_FILENAME).exists());

        let loaded = read_live(&dir).expect("read").expect("present");
        assert_eq!(loaded.ply, 47);
        assert_eq!(loaded.challenger, "l0-r3");
        assert_eq!(loaded.evals.challenger_nn, Some(120));
    }

    #[test]
    fn unsubscribe_stops_writes() {
        let dir = tempdir();
        subscribe(&dir).expect("subscribe");
        let live = sample_live();
        assert!(write_if_subscribed(&dir, &live).expect("write"));

        unsubscribe(&dir).expect("unsubscribe");
        assert!(!is_subscribed(&dir));

        // After unsubscribe, no further writes happen.
        let dir2 = tempdir();  // fresh dir to confirm no leakage
        unsubscribe(&dir2).expect("idempotent");
        assert!(!write_if_subscribed(&dir2, &live).expect("write"));
    }

    #[test]
    fn unsubscribe_is_idempotent() {
        let dir = tempdir();
        unsubscribe(&dir).expect("idempotent");
        unsubscribe(&dir).expect("idempotent");
        assert!(!is_subscribed(&dir));
    }

    #[test]
    fn writer_stamps_written_at_ms() {
        let dir = tempdir();
        subscribe(&dir).expect("subscribe");
        let mut live = sample_live();
        live.written_at_ms = 0;

        let before = now_ms();
        write_if_subscribed(&dir, &live).expect("write");
        let after = now_ms();

        let loaded = read_live(&dir).expect("read").expect("present");
        assert!(
            loaded.written_at_ms >= before && loaded.written_at_ms <= after,
            "written_at_ms = {} not in [{}, {}]",
            loaded.written_at_ms, before, after,
        );
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = tempdir();
        assert!(read_live(&dir).expect("read").is_none());
    }

    #[test]
    fn read_rejects_wrong_format_version() {
        let dir = tempdir();
        let raw = r#"{
            "format_version": 999,
            "written_at_ms": 0,
            "fen": "",
            "ply": 0,
            "challenger": "",
            "defender": "",
            "game_index": 0,
            "games_total": 0,
            "evals": {}
        }"#;
        std::fs::write(dir.join(LIVE_STATE_FILENAME), raw).unwrap();
        let err = read_live(&dir).expect_err("must reject");
        assert!(matches!(err, LiveError::FormatVersionMismatch { found: 999, expected: 1 }));
    }

    #[test]
    fn rapid_writes_leave_no_tmp_file() {
        let dir = tempdir();
        subscribe(&dir).expect("subscribe");
        for i in 0..10 {
            let mut live = sample_live();
            live.ply = i;
            write_if_subscribed(&dir, &live).expect("write");
            let loaded = read_live(&dir).expect("read").expect("present");
            assert_eq!(loaded.ply, i);
        }
        let tmp = dir.join(format!("{}.tmp", LIVE_STATE_FILENAME));
        assert!(!tmp.exists(), "stale .tmp file left behind");
    }
}
