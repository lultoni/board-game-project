//! Layer 5 — Telemetry & Analytics.
//!
//! Per ADR-005: every match is auto-logged with action history + per-move
//! timing. The "send to designer" upload bundles N recent logs into a JSON
//! blob the user transmits out-of-band. No server endpoint required.
//!
//! This layer is pure data shaping. Persistence is the frontend's job
//! (browser localStorage / Tauri filesystem).

use crate::game_logic::action::Action;

#[derive(Clone, Debug)]
pub struct MoveTiming {
    pub action:    Action,
    pub thought_ms: u32,
}

#[derive(Clone, Debug, Default)]
pub struct MatchLog {
    pub started_at_unix: u64,
    pub moves:           Vec<MoveTiming>,
    pub final_result:    Option<MatchResult>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult { P1Win, P2Win, Aborted }

// TODO: serialise MatchLog as PGN-like notation + JSON.
// TODO: bundle N recent MatchLogs for export.
