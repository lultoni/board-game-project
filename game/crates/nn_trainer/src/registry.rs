//! Accepted-rater registry.
//!
//! `<raters_dir>/index.json` is the authoritative list of accepted rater
//! versions in acceptance order. It also names the leader of each of the
//! three champion tracks (best-fast / best-slow / best-overall) so the
//! gauntlet in §5 can pick which version to use as a baseline at each
//! think-time bracket.
//!
//! ## Invariants
//!
//! - **Append-only.** Once a rater enters the index it stays there. Rejected
//!   raters never enter; the index never shrinks. This makes the index
//!   diff-stable and lets the gauntlet replay historical match-ups.
//! - **Track pointers reference existing entries.** `set_track` rejects an
//!   ID that isn't in `entries`.
//! - **Schema versioned.** `INDEX_FORMAT_VERSION` bumps if the layout
//!   changes incompatibly; load aborts on mismatch.
//!
//! ## Disk layout
//!
//! ```text
//! raters/
//!   index.json
//!   v0001.mpk    v0001.json
//!   v0002.mpk    v0002.json
//!   …
//! ```
//!
//! `IndexEntry::stem` is relative to the directory holding `index.json`, so
//! the registry is portable — moving `raters/` to another machine doesn't
//! break the pointers.

use crate::persistence::BracketWinRate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Disk-format version for `index.json`. Bump on incompatible changes.
pub const INDEX_FORMAT_VERSION: u32 = 1;

/// The three champion tracks per plan §5. Each track has at most one
/// leader at a time; the gauntlet driver updates the pointer when a
/// candidate beats the current leader at that bracket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Track {
    /// Best at the fast bracket (lowest think-time).
    Fast,
    /// Best at the slow bracket (highest think-time).
    Slow,
    /// Best aggregate across all brackets.
    Overall,
}

impl Track {
    pub fn all() -> [Track; 3] { [Track::Fast, Track::Slow, Track::Overall] }
}

/// One accepted rater. `stem` is the path stem relative to the directory
/// holding `index.json` (e.g. `"v0042"` → `raters/v0042.{mpk,json}`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Stable identifier — matches the file stem on disk. Used to look up
    /// the entry and to set track pointers.
    pub id: String,

    /// Path stem relative to the registry directory. Allows
    /// `paths_from_stem(registry_dir.join(&entry.stem))` to recover the
    /// blob + sidecar paths.
    pub stem: PathBuf,

    /// ISO-8601 UTC timestamp at acceptance.
    pub accepted_at: String,

    /// Parent rater this candidate descends from. `None` for the founding
    /// entry (no prior accepted rater to clone from). Optional via
    /// `#[serde(default)]` so older index.json files without this field
    /// still parse.
    #[serde(default)]
    pub parent_id: Option<String>,

    /// Bracket results that earned this rater its place in the index.
    /// Keyed by bracket name (`"fast"`, `"medium"`, `"slow"`). Empty is
    /// allowed for legacy/seeded entries that pre-date the gauntlet.
    #[serde(default)]
    pub bracket_results: BTreeMap<String, BracketWinRate>,
}

/// The on-disk registry. Lives at `<raters_dir>/index.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaterIndex {
    pub format_version: u32,
    /// Accepted raters in acceptance order. Never reordered, never shrinks.
    pub entries: Vec<IndexEntry>,
    /// Current leader per track. `None` until the first acceptance at that
    /// bracket. Values are `IndexEntry::id` strings — must reference an
    /// entry in `entries`.
    #[serde(default)]
    pub tracks: BTreeMap<Track, String>,
}

impl Default for RaterIndex {
    fn default() -> Self {
        Self {
            format_version: INDEX_FORMAT_VERSION,
            entries: Vec::new(),
            tracks: BTreeMap::new(),
        }
    }
}

/// Errors from registry operations.
#[derive(Debug)]
pub enum IndexError {
    Io(std::io::Error),
    Json(serde_json::Error),
    FormatVersionMismatch { found: u32, expected: u32 },
    /// Tried to set a track pointer to an ID that isn't in `entries`.
    UnknownId(String),
    /// Tried to append an entry whose ID is already present.
    DuplicateId(String),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::FormatVersionMismatch { found, expected } => write!(
                f, "index format version {} not supported (expected {})",
                found, expected,
            ),
            Self::UnknownId(id) => write!(f, "unknown rater id: {}", id),
            Self::DuplicateId(id) => write!(f, "duplicate rater id: {}", id),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<std::io::Error> for IndexError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for IndexError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

impl RaterIndex {
    /// Path to the index file inside a registry directory.
    pub fn index_path(dir: &Path) -> PathBuf {
        dir.join("index.json")
    }

    /// Load `<dir>/index.json`. Returns `Default::default()` (empty) if the
    /// file doesn't exist — bootstrap-friendly.
    pub fn load(dir: &Path) -> Result<Self, IndexError> {
        let path = Self::index_path(dir);
        match std::fs::read_to_string(&path) {
            Ok(json) => {
                let idx: RaterIndex = serde_json::from_str(&json)?;
                if idx.format_version != INDEX_FORMAT_VERSION {
                    return Err(IndexError::FormatVersionMismatch {
                        found: idx.format_version,
                        expected: INDEX_FORMAT_VERSION,
                    });
                }
                Ok(idx)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(IndexError::Io(e)),
        }
    }

    /// Write `<dir>/index.json`. Creates `dir` if missing.
    pub fn save(&self, dir: &Path) -> Result<(), IndexError> {
        std::fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::index_path(dir), json)?;
        Ok(())
    }

    /// Append a new accepted rater. Rejects a duplicate ID.
    pub fn append(&mut self, entry: IndexEntry) -> Result<(), IndexError> {
        if self.entries.iter().any(|e| e.id == entry.id) {
            return Err(IndexError::DuplicateId(entry.id));
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Promote `id` to leader of `track`. The ID must already be in
    /// `entries` (an unknown ID is a logic error from the caller).
    pub fn set_track(&mut self, track: Track, id: &str) -> Result<(), IndexError> {
        if !self.entries.iter().any(|e| e.id == id) {
            return Err(IndexError::UnknownId(id.to_string()));
        }
        self.tracks.insert(track, id.to_string());
        Ok(())
    }

    /// Current leader of `track`, if any.
    pub fn track_leader(&self, track: Track) -> Option<&IndexEntry> {
        let id = self.tracks.get(&track)?;
        self.entries.iter().find(|e| e.id == *id)
    }

    /// Most recently accepted entry, regardless of track. Useful when the
    /// gauntlet wants to compare a new candidate against "the previous
    /// rater" without going through track pointers.
    pub fn latest(&self) -> Option<&IndexEntry> {
        self.entries.last()
    }

    /// Look up an entry by ID.
    pub fn get(&self, id: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
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
            .join(format!("nn_trainer_registry_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn entry(id: &str) -> IndexEntry {
        IndexEntry {
            id: id.to_string(),
            stem: PathBuf::from(id),
            accepted_at: "2026-06-27T00:00:00Z".to_string(),
            parent_id: None,
            bracket_results: BTreeMap::new(),
        }
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = tempdir();
        let idx = RaterIndex::load(&dir).expect("load");
        assert!(idx.entries.is_empty());
        assert!(idx.tracks.is_empty());
        assert_eq!(idx.format_version, INDEX_FORMAT_VERSION);
    }

    #[test]
    fn append_and_save_load_roundtrip() {
        let dir = tempdir();
        let mut idx = RaterIndex::default();
        idx.append(entry("v0001")).unwrap();
        idx.append(entry("v0002")).unwrap();
        idx.save(&dir).unwrap();

        let reloaded = RaterIndex::load(&dir).unwrap();
        assert_eq!(reloaded.entries.len(), 2);
        assert_eq!(reloaded.entries[0].id, "v0001");
        assert_eq!(reloaded.entries[1].id, "v0002");
        assert_eq!(reloaded.latest().unwrap().id, "v0002");
    }

    #[test]
    fn append_rejects_duplicate_id() {
        let mut idx = RaterIndex::default();
        idx.append(entry("v0001")).unwrap();
        let err = idx.append(entry("v0001")).expect_err("must reject");
        assert!(matches!(err, IndexError::DuplicateId(ref s) if s == "v0001"));
    }

    #[test]
    fn set_track_promotes_and_lookups_work() {
        let mut idx = RaterIndex::default();
        idx.append(entry("v0001")).unwrap();
        idx.append(entry("v0002")).unwrap();

        idx.set_track(Track::Fast, "v0001").unwrap();
        idx.set_track(Track::Overall, "v0002").unwrap();

        assert_eq!(idx.track_leader(Track::Fast).unwrap().id, "v0001");
        assert_eq!(idx.track_leader(Track::Overall).unwrap().id, "v0002");
        assert!(idx.track_leader(Track::Slow).is_none());

        // Promote: same track, different id.
        idx.set_track(Track::Fast, "v0002").unwrap();
        assert_eq!(idx.track_leader(Track::Fast).unwrap().id, "v0002");
    }

    #[test]
    fn set_track_rejects_unknown_id() {
        let mut idx = RaterIndex::default();
        idx.append(entry("v0001")).unwrap();
        let err = idx.set_track(Track::Fast, "v0099").expect_err("must reject");
        assert!(matches!(err, IndexError::UnknownId(ref s) if s == "v0099"));
    }

    #[test]
    fn tracks_survive_roundtrip() {
        let dir = tempdir();
        let mut idx = RaterIndex::default();
        idx.append(entry("v0001")).unwrap();
        idx.append(entry("v0002")).unwrap();
        idx.set_track(Track::Fast, "v0001").unwrap();
        idx.set_track(Track::Slow, "v0002").unwrap();
        idx.save(&dir).unwrap();

        let reloaded = RaterIndex::load(&dir).unwrap();
        assert_eq!(reloaded.track_leader(Track::Fast).unwrap().id, "v0001");
        assert_eq!(reloaded.track_leader(Track::Slow).unwrap().id, "v0002");
        assert!(reloaded.track_leader(Track::Overall).is_none());
    }

    #[test]
    fn load_rejects_wrong_format_version() {
        let dir = tempdir();
        let raw = format!(
            r#"{{ "format_version": 999, "entries": [], "tracks": {{}} }}"#
        );
        std::fs::write(RaterIndex::index_path(&dir), raw).unwrap();
        let err = RaterIndex::load(&dir).expect_err("must reject");
        assert!(matches!(err, IndexError::FormatVersionMismatch { found: 999, expected: 1 }));
    }
}
