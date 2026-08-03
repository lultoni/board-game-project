//! core_engine - pure Rust game engine for (GAME NAME).
//!
//! Layered per ADR-005:
//!   Layer 1 - state        (bitboards + mailbox + zobrist)
//!   Layer 2 - game_logic   (action generator + make/unmake + magic bitboards)
//!   Layer 3 - search       (alpha-beta + iterative deepening + transposition table)
//!   Layer 4 - session      (match state + action history + serialisation)
//!   Layer 5 - telemetry    (auto-log + export)
//!
//! No I/O, no platform dependencies. Stateless math. The same crate is
//! consumed by `wasm_wrapper` (web) and `tauri_wrapper` (desktop native).

pub mod state;
pub mod game_logic;
pub mod search;
pub mod session;
pub mod telemetry;
pub mod time;
pub mod wrapper_api;

pub use session::{
    Match, Config, SeatKind, AiBudget,
    ApplyError, UndoError, AiError,
    Snapshot, SnapshotError,
    ApplyEvent, NetworkTransport, LocalTransport,
};

pub use telemetry::{
    MatchLog, PlyRecord, ActionDecoded, SearchMeta,
    MatchResult, Bundle,
    to_json, to_json_pretty, from_json, config_hash, notation,
};

pub use search::evaluator::{evaluate, evaluate_report, EvalReport, BreakdownDetail, MATE_SCORE};

pub use game_logic::skills::{
    all_skill_metadata, game_constants, SkillMetadata, GameConstants,
};

pub use state::action_notation::{action_to_notation, notation_to_action, NotationError};
