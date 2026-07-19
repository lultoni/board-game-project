//! tauri_wrapper - Tauri 2 desktop wrapper.
//!
//! Mirrors the wasm_wrapper Engine surface 1:1, but as Tauri commands rather
//! than a wasm-bindgen class. The Svelte frontend can be written against a
//! shared TS interface and dispatched to either `invoke` (Tauri) or a Web
//! Worker (wasm) at runtime.
//!
//! # Architecture
//!
//! Tauri commands are stateless; engine state lives in a process-global
//! [`EngineRegistry`] managed by `tauri::State`. Frontend gets back a numeric
//! `handle` from [`create_engine`] and passes it to every subsequent command.
//! Handles are u64 monotonic counters - never reused, never zero (zero is
//! reserved for "invalid").
//!
//! # Concurrency
//!
//! The registry is a `Mutex<HashMap<u64, EngineEntry>>`. Each entry contains
//! the `Match` itself plus a reusable `Vec<u32>` for legal-action enumeration.
//! Lock granularity is per-call: take the lock, do the work, release. AI
//! search runs inside `tokio::task::block_in_place` so we don't pin the async
//! executor on a multi-thread runtime.
//!
//! # Hot vs cold path
//!
//! - **Hot path** (board reads, legal actions, step): returns flat serde
//!   structs / `Vec<u32>` / `Vec<u16>`. Tauri serialises these to JSON for
//!   the IPC bridge - no zero-copy across the boundary, but native build is
//!   fast enough that the copy is invisible.
//! - **Cold path** (snapshot, log export): owned JSON strings.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

use core_engine::wrapper_api as api;
use core_engine::Match;

// ---------------------------------------------------------------------------
// Handle table - process-global, one per Match.
// ---------------------------------------------------------------------------

struct EngineEntry {
    m:         Match,
    legal_buf: Vec<u32>,
}

#[derive(Default)]
pub struct EngineRegistry {
    next:    AtomicU64,
    engines: Mutex<HashMap<u64, EngineEntry>>,
}

impl EngineRegistry {
    fn fresh_handle(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn insert(&self, m: Match) -> u64 {
        let h = self.fresh_handle();
        self.engines.lock().unwrap().insert(h, EngineEntry {
            m,
            legal_buf: Vec::with_capacity(256),
        });
        h
    }

    fn with<R>(&self, handle: u64, f: impl FnOnce(&mut EngineEntry) -> R) -> Result<R, String> {
        let mut guard = self.engines.lock().unwrap();
        let entry = guard.get_mut(&handle)
            .ok_or_else(|| format!("unknown engine handle {handle}"))?;
        Ok(f(entry))
    }

    fn drop_handle(&self, handle: u64) -> bool {
        self.engines.lock().unwrap().remove(&handle).is_some()
    }
}

// ---------------------------------------------------------------------------
// Wire types - mirror the wasm_wrapper JS shape exactly. camelCase via serde
// so the frontend reads `appliedAction`, not `applied_action`.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PendingBodyguardDto {
    pub attacker_src: u8,
    pub attacker_now: u8,
    pub target_sq:    u8,
    pub eligible:     Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionViewDto {
    pub bitboards:         [String; 5],
    pub mailbox:           Vec<u16>,
    pub to_move:           u8,
    pub current_phase:     u8,
    pub actions_remaining: u8,
    pub round_number:      u16,
    pub p1_money:          u16,
    pub p2_money:          u16,
    pub pending_modifiers: u8,
    pub game_result:       u8,
    pub zobrist:           String,
    pub pending_bodyguard: Option<PendingBodyguardDto>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PhaseStateDto {
    pub to_move:           u8,
    pub current_phase:     u8,
    pub actions_remaining: u8,
    pub round_number:      u16,
    pub p1_money:          u16,
    pub p2_money:          u16,
    pub pending_modifiers: u8,
    pub game_result:       u8,
    pub zobrist:           u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StepResultDto {
    pub applied_action: u32,
    pub score:          i32,
    pub depth:          u8,
    pub nodes:          u64,
    pub thought_ms:     u32,
    pub game_result:    u8,
}

impl From<api::StepResult> for StepResultDto {
    fn from(r: api::StepResult) -> Self {
        StepResultDto {
            applied_action: r.applied_action,
            score:          r.score,
            depth:          r.depth,
            nodes:          r.nodes,
            thought_ms:     r.thought_ms,
            game_result:    r.game_result,
        }
    }
}

/// L8 draft snapshot. `usedSlots[piece][slot]` mirrors
/// `core_engine::game_logic::draft::DraftState`. Indexed:
///   - piece 0..6  → P1's bearers (King at 0, Champions 1..5 by sq asc)
///   - piece 6..12 → P2's bearers (same internal order)
///   - slot 0..2   → mailbox slot1 / slot2.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DraftStateDto {
    pub turn_no:      u8,
    pub side_to_move: u8,
    pub used_slots:   [[bool; 2]; 12],
}

// ---------------------------------------------------------------------------
// Wall-clock helper. core_engine::time::now_ms() is monotonic-since-process-
// start on native, NOT wall-clock. For `applied_at_unix_ms` we use the real
// system time.
// ---------------------------------------------------------------------------

fn unix_ms_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Commands - names mirror wasm_wrapper.Engine methods. Tauri exposes them on
// the JS side as `invoke('command_name', { args })`.
// ---------------------------------------------------------------------------

#[tauri::command]
fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn create_engine(
    config_json: Option<String>,
    registry:    State<'_, EngineRegistry>,
) -> Result<u64, String> {
    let cfg = match config_json {
        Some(s) => core_engine::from_json(&s)
            .map_err(|e| format!("config parse error: {e}"))?,
        None    => core_engine::Config::local_aivai(),
    };
    let m = Match::new_with_clock(cfg, unix_ms_now());
    Ok(registry.insert(m))
}

#[tauri::command]
fn create_engine_from_snapshot(
    snapshot_json: String,
    registry:      State<'_, EngineRegistry>,
) -> Result<u64, String> {
    let m = api::from_snapshot_json(&snapshot_json, unix_ms_now())
        .map_err(|e| e.to_string())?;
    Ok(registry.insert(m))
}

/// Open a fresh match in `Phase::Draft`. Frontend then drives 12 DraftTurn
/// plies via `try_apply` / `step_ai`; engine transitions to `Phase::Move`
/// automatically after ply 12.
#[tauri::command]
fn create_engine_with_draft(
    config_json: Option<String>,
    registry:    State<'_, EngineRegistry>,
) -> Result<u64, String> {
    let cfg = match config_json {
        Some(s) => core_engine::from_json(&s)
            .map_err(|e| format!("config parse error: {e}"))?,
        None    => core_engine::Config::local_aivai(),
    };
    let m = api::new_match_with_draft(cfg, unix_ms_now());
    Ok(registry.insert(m))
}

/// Open a fresh match with both sides' loadouts already applied (no draft).
/// Loadouts arrive as JSON `[[s1,s2],[s1,s2],...]` 6-tuples; the engine
/// validates them and rejects invalid pairs (e.g. same skill in both slots).
#[tauri::command]
fn create_engine_with_loadouts(
    config_json:    Option<String>,
    p1_loadout_json: String,
    p2_loadout_json: String,
    registry:       State<'_, EngineRegistry>,
) -> Result<u64, String> {
    let cfg = match config_json {
        Some(s) => core_engine::from_json(&s)
            .map_err(|e| format!("config parse error: {e}"))?,
        None    => core_engine::Config::local_aivai(),
    };
    let p1 = api::parse_side_loadout_json(&p1_loadout_json)
        .map_err(|e| format!("p1 loadout parse error: {e}"))?;
    let p2 = api::parse_side_loadout_json(&p2_loadout_json)
        .map_err(|e| format!("p2 loadout parse error: {e}"))?;
    let m = api::new_match_with_loadouts(cfg, &p1, &p2, unix_ms_now())
        .map_err(|e| format!("loadout validation failed: {e:?}"))?;
    Ok(registry.insert(m))
}

/// Compact draft-state snapshot for UI legality hints (who picks, which
/// mailbox slots are filled). Cheap to call on every UI refresh.
#[tauri::command]
fn draft_state(handle: u64, registry: State<'_, EngineRegistry>) -> Result<DraftStateDto, String> {
    registry.with(handle, |e| {
        let s = api::current_draft_state(&e.m);
        DraftStateDto {
            turn_no:      s.turn_no,
            side_to_move: match s.side_to_move {
                core_engine::state::position::Player::P1 => 0,
                core_engine::state::position::Player::P2 => 1,
            },
            used_slots: s.used_slots,
        }
    })
}

#[tauri::command]
fn drop_engine(handle: u64, registry: State<'_, EngineRegistry>) -> bool {
    registry.drop_handle(handle)
}

#[tauri::command]
fn position_view(handle: u64, registry: State<'_, EngineRegistry>) -> Result<PositionViewDto, String> {
    registry.with(handle, |e| {
        let v  = api::position_view(&e.m);
        let mb = api::position_mailbox(&e.m);
        let pbg = api::pending_bodyguard(&e.m).map(|p| PendingBodyguardDto {
            attacker_src: p.attacker_src,
            attacker_now: p.attacker_now,
            target_sq:    p.target_sq,
            eligible:     p.eligible[..p.eligible_len as usize].to_vec(),
        });
        PositionViewDto {
            bitboards:         v.bitboards.map(|b| b.to_string()),
            mailbox:           mb.to_vec(),
            to_move:           v.to_move,
            current_phase:     v.current_phase,
            actions_remaining: v.actions_remaining,
            round_number:      v.round_number,
            p1_money:          v.p1_money,
            p2_money:          v.p2_money,
            pending_modifiers: v.pending_modifiers,
            game_result:       v.game_result,
            zobrist:           v.zobrist.to_string(),
            pending_bodyguard: pbg,
        }
    })
}

#[tauri::command]
fn phase_state(handle: u64, registry: State<'_, EngineRegistry>) -> Result<PhaseStateDto, String> {
    registry.with(handle, |e| {
        let v = api::position_view(&e.m);
        PhaseStateDto {
            to_move:           v.to_move,
            current_phase:     v.current_phase,
            actions_remaining: v.actions_remaining,
            round_number:      v.round_number,
            p1_money:          v.p1_money,
            p2_money:          v.p2_money,
            pending_modifiers: v.pending_modifiers,
            game_result:       v.game_result,
            zobrist:           v.zobrist,
        }
    })
}

#[tauri::command]
fn position_fen(handle: u64, registry: State<'_, EngineRegistry>) -> Result<String, String> {
    registry.with(handle, |e| api::position_fen(&e.m))
}

#[tauri::command]
fn heuristic_eval(
    handle:   u64,
    registry: State<'_, EngineRegistry>,
) -> Result<core_engine::search::evaluator::EvalBreakdown, String> {
    registry.with(handle, |e| api::heuristic_eval(&e.m))
}

#[tauri::command]
fn heuristic_eval_by_square(
    handle:   u64,
    registry: State<'_, EngineRegistry>,
) -> Result<core_engine::search::evaluator::EvalBreakdownBySquare, String> {
    registry.with(handle, |e| api::heuristic_eval_by_square(&e.m))
}

#[tauri::command]
fn legal_actions(handle: u64, registry: State<'_, EngineRegistry>) -> Result<Vec<u32>, String> {
    registry.with(handle, |e| {
        api::legal_actions_into(&e.m, &mut e.legal_buf);
        e.legal_buf.clone()
    })
}

#[tauri::command]
fn action_to_notation_cmd(raw: u32) -> String {
    core_engine::action_to_notation(core_engine::game_logic::action::Action(raw), None)
}

#[tauri::command]
fn try_apply(
    handle:     u64,
    raw_action: u32,
    registry:   State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let now = unix_ms_now();
    registry.with(handle, |e| {
        api::try_apply(&mut e.m, raw_action, now)
            .map(StepResultDto::from)
            .map_err(|err| format!("{err:?}"))
    })?
}

/// AI step. CPU-bound - runs inside `block_in_place` so the Tauri async
/// runtime keeps responding to other IPC traffic.
#[tauri::command]
async fn step_ai(
    handle:   u64,
    app:      tauri::AppHandle,
    registry: State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let now = unix_ms_now();
    let res: Result<Result<StepResultDto, String>, String> =
        tokio::task::block_in_place(|| {
            registry.with(handle, |e| {
                api::step_ai_with_cb(&mut e.m, now, Some(&|depth, score| {
                    let _ = app.emit("ai-depth-update", serde_json::json!({ "depth": depth, "score": score }));
                }))
                    .map(StepResultDto::from)
                    .map_err(|err| format!("{err:?}"))
            })
        });
    res?
}

/// Inspector variant: runs the AI search regardless of seat kind. Used so
/// "Ask AI" works in HvH positions too.
#[tauri::command]
async fn request_ai_move_forced(
    handle:   u64,
    registry: State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let res: Result<Result<StepResultDto, String>, String> =
        tokio::task::block_in_place(|| {
            registry.with(handle, |e| {
                api::request_ai_move_forced(&mut e.m)
                    .map(StepResultDto::from)
                    .map_err(|err| format!("{err:?}"))
            })
        });
    res?
}

/// Inspector iterative-deepening helper. Runs ID up to `max_depth` with
/// no time bound; caller drives the loop.
#[tauri::command]
async fn request_ai_move_at_depth(
    handle:    u64,
    max_depth: u8,
    registry:  State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let res: Result<Result<StepResultDto, String>, String> =
        tokio::task::block_in_place(|| {
            registry.with(handle, |e| {
                api::request_ai_move_at_depth(&mut e.m, max_depth)
                    .map(StepResultDto::from)
                    .map_err(|err| format!("{err:?}"))
            })
        });
    res?
}

#[tauri::command]
fn match_log_json(handle: u64, registry: State<'_, EngineRegistry>) -> Result<Option<String>, String> {
    registry.with(handle, |e| api::match_log_json(&e.m))
}

#[tauri::command]
fn latest_ply_json(handle: u64, registry: State<'_, EngineRegistry>) -> Result<Option<String>, String> {
    registry.with(handle, |e| api::latest_ply_json(&e.m))
}

#[tauri::command]
fn finalise_log(
    handle:      u64,
    result_byte: u8,
    registry:    State<'_, EngineRegistry>,
) -> Result<(), String> {
    let now = unix_ms_now();
    registry.with(handle, |e| api::finalise_log(&mut e.m, now, result_byte))
}

#[tauri::command]
fn snapshot_json(handle: u64, registry: State<'_, EngineRegistry>) -> Result<String, String> {
    registry.with(handle, |e| api::snapshot_json(&e.m))
}

// ---------------------------------------------------------------------------
// Training Observatory - orchestrator IPC (plan §7).
//
// The trainer writes status.json / live.json / matrix.json / raters/ into a
// run directory. These commands let the frontend:
//   - read those files (no schema duplication: serde does the round-trip),
//   - subscribe / unsubscribe to the live-position stream,
//   - parse a FEN string into the existing PositionView shape so Board.svelte
//     can render it unchanged,
//   - start / stop a training run in a background thread.
//
// State management: `TrainingState` holds the stop flag + JoinHandle for the
// running orchestrator. Registered with `.manage(...)` in `run()`.
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Process-global training-run state. At most one run can be active at a time.
#[derive(Default)]
pub struct TrainingState {
    inner: Mutex<TrainingInner>,
}

#[derive(Default)]
struct TrainingInner {
    stop: Option<Arc<AtomicBool>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

/// Repo-relative default run directory. Computed at command-invocation time
/// from `CARGO_MANIFEST_DIR` so the path follows the binary across machines.
/// Falls back to `./runs/active` if the env var isn't set (e.g. tauri bundle).
fn default_run_dir_path() -> std::path::PathBuf {
    // Walk up from the tauri_wrapper crate dir to the `game/` root, then into
    // `runs/active`. CARGO_MANIFEST_DIR points at the crate, so its parent's
    // parent is `game/`.
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let game_dir = std::path::Path::new(crate_dir)
        .parent()           // crates/
        .and_then(|p| p.parent()); // game/
    match game_dir {
        Some(g) => g.join("runs").join("active"),
        None => std::path::PathBuf::from("runs/active"),
    }
}

#[tauri::command]
fn default_run_dir() -> Result<String, String> {
    Ok(default_run_dir_path().to_string_lossy().to_string())
}

/// Parse a FEN string and produce the same `PositionViewDto` shape the rest
/// of the wrapper uses. Lets the Live Match View reuse `Board.svelte` without
/// duplicating FEN-parsing logic in the frontend.
#[tauri::command]
fn fen_to_position_view(fen: String) -> Result<PositionViewDto, String> {
    use core_engine::state::fen::from_fen;
    use core_engine::state::position::{Phase, Player};
    let pos = from_fen(&fen).map_err(|e| format!("fen parse error: {e:?}"))?;
    // Mailbox: MailboxEntry is repr(transparent) over u16 - same trick as
    // wrapper_api::position_mailbox.
    let mailbox: Vec<u16> = pos.mailbox.iter().map(|m| m.0).collect();
    let game_result = match pos.game_result {
        None => 0,
        Some(core_engine::state::position::GameResult::P1Wins) => 1,
        Some(core_engine::state::position::GameResult::P2Wins) => 2,
    };
    Ok(PositionViewDto {
        bitboards: [
            pos.p1_pieces.0.to_string(),
            pos.p2_pieces.0.to_string(),
            pos.kings.0.to_string(),
            pos.champions.0.to_string(),
            pos.guards.0.to_string(),
        ],
        mailbox,
        to_move: match pos.to_move { Player::P1 => 0, Player::P2 => 1 },
        current_phase: match pos.current_phase {
            Phase::Move => 0,
            Phase::Skill => 1,
            Phase::Draft => 2,
        },
        actions_remaining: pos.actions_remaining,
        round_number: pos.round_number,
        p1_money: pos.p1_money,
        p2_money: pos.p2_money,
        pending_modifiers: pos.pending_modifiers,
        game_result,
        zobrist: pos.zobrist.to_string(),
        pending_bodyguard: None,
    })
}

#[tauri::command]
fn read_training_status(
    run_dir: String,
) -> Result<Option<nn_trainer::StatusSnapshot>, String> {
    nn_trainer::read_snapshot(std::path::Path::new(&run_dir))
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
fn read_training_live(
    run_dir: String,
) -> Result<Option<nn_trainer::LivePosition>, String> {
    nn_trainer::read_live(std::path::Path::new(&run_dir))
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
fn subscribe_training_live(run_dir: String) -> Result<(), String> {
    nn_trainer::subscribe(std::path::Path::new(&run_dir))
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
fn unsubscribe_training_live(run_dir: String) -> Result<(), String> {
    nn_trainer::unsubscribe(std::path::Path::new(&run_dir))
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
fn read_rater_index(run_dir: String) -> Result<Option<nn_trainer::RaterIndex>, String> {
    // RaterIndex lives under <run_dir>/raters/. load() returns an empty index
    // when the file is missing; we map that to None so the UI can show
    // "no run yet" vs "empty run" without ambiguity.
    let raters = std::path::Path::new(&run_dir).join("raters");
    if !raters.join("index.json").exists() {
        return Ok(None);
    }
    nn_trainer::RaterIndex::load(&raters)
        .map(Some)
        .map_err(|e| format!("{e}"))
}

#[tauri::command]
fn read_gauntlet_matrix(run_dir: String) -> Result<Option<nn_trainer::GauntletMatrix>, String> {
    let path = std::path::Path::new(&run_dir).join(nn_trainer::MATRIX_FILENAME);
    if !path.exists() {
        return Ok(None);
    }
    nn_trainer::load_matrix(std::path::Path::new(&run_dir))
        .map(Some)
        .map_err(|e| format!("{e}"))
}

/// Network Inspector data for one (rater, position) pair. Returns weight
/// summary stats per layer plus the raw forward output. Heatmap data is
/// out-of-scope for v1 - we add it once the inspector panel is wired up.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WeightStats {
    pub layer: String,
    pub mean: f32,
    pub std: f32,
    pub min: f32,
    pub max: f32,
    pub nan_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RaterInspection {
    pub rater_id: String,
    pub forward_output: f32,
    /// Centipawn-scale conversion factor - either the calibrated value from
    /// the sidecar (`RaterMetadata::eval_scale`) or `DEFAULT_EVAL_SCALE` when
    /// the rater hasn't been calibrated yet. UI can display
    /// `forward_output * eval_scale` to show the centipawn-scale score.
    pub eval_scale: f32,
    pub weight_stats: Vec<WeightStats>,
}

#[tauri::command]
fn inspect_rater(
    run_dir: String,
    rater_id: String,
    fen: String,
) -> Result<RaterInspection, String> {
    use core_engine::state::fen::from_fen;
    let pos = from_fen(&fen).map_err(|e| format!("fen parse: {e:?}"))?;
    let stem = std::path::Path::new(&run_dir).join("raters").join(&rater_id);
    let (forward_output, eval_scale, layer_stats) =
        nn_trainer::NnEvaluator::inspect_fen_at_stem(&stem, &pos)
            .map_err(|e| format!("load rater: {e}"))?;
    let weight_stats = layer_stats
        .into_iter()
        .map(|s| WeightStats {
            layer: s.layer,
            mean: s.mean,
            std: s.std,
            min: s.min,
            max: s.max,
            nan_count: s.nan_count,
        })
        .collect();
    Ok(RaterInspection {
        rater_id,
        forward_output,
        eval_scale,
        weight_stats,
    })
}

// ---------------------------------------------------------------------------
// Rater discovery + per-seat evaluator selection.
// ---------------------------------------------------------------------------

/// `game/raters/blessed/` - the curated "good enough to play against" raters
/// promoted out of one or more `runs/active/` directories. May not exist on
/// fresh checkouts.
fn blessed_raters_dir() -> std::path::PathBuf {
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let game_dir = std::path::Path::new(crate_dir)
        .parent()
        .and_then(|p| p.parent());
    match game_dir {
        Some(g) => g.join("raters").join("blessed"),
        None => std::path::PathBuf::from("raters/blessed"),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RaterListing {
    /// `"run"` (from `<run_dir>/raters/`) or `"blessed"` (from `game/raters/
    /// blessed/`). Used in `set_ai_evaluator` to disambiguate.
    pub source: String,
    pub id: String,
    pub accepted_at: String,
    pub parent_id: Option<String>,
    /// `true` if this rater is the current champion of its index (the single
    /// `Track::Champion` pointer, ns-50). The setup-screen "Current best"
    /// quick-pick resolves to this entry.
    pub is_champion: bool,
}

/// Walks both the active run dir's `raters/` and `game/raters/blessed/`,
/// returning the union as a single list. Empty directories produce an empty
/// list rather than an error so the UI can render a friendly "no raters yet"
/// state without distinguishing "missing dir" from "empty dir".
#[tauri::command]
fn list_available_raters(run_dir: Option<String>) -> Result<Vec<RaterListing>, String> {
    let mut out: Vec<RaterListing> = Vec::new();
    let run_path = run_dir
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_run_dir_path);
    let run_raters = run_path.join("raters");
    if run_raters.is_dir() {
        match nn_trainer::RaterIndex::load(&run_raters) {
            Ok(idx) => {
                let champ = idx.track_leader(nn_trainer::Track::Champion).map(|e| e.id.clone());
                for e in &idx.entries {
                    out.push(RaterListing {
                        source: "run".to_string(),
                        id: e.id.clone(),
                        accepted_at: e.accepted_at.clone(),
                        parent_id: e.parent_id.clone(),
                        is_champion: champ.as_deref() == Some(e.id.as_str()),
                    });
                }
            }
            Err(e) => return Err(format!("run raters: {e}")),
        }
    }
    let blessed = blessed_raters_dir();
    if blessed.is_dir() {
        match nn_trainer::RaterIndex::load(&blessed) {
            Ok(idx) => {
                let champ = idx.track_leader(nn_trainer::Track::Champion).map(|e| e.id.clone());
                for e in &idx.entries {
                    out.push(RaterListing {
                        source: "blessed".to_string(),
                        id: e.id.clone(),
                        accepted_at: e.accepted_at.clone(),
                        parent_id: e.parent_id.clone(),
                        is_champion: champ.as_deref() == Some(e.id.as_str()),
                    });
                }
            }
            Err(e) => return Err(format!("blessed raters: {e}")),
        }
    }
    Ok(out)
}

/// Install a per-seat evaluator on an existing engine handle. `source` is one
/// of `"heuristic"`, `"run"`, or `"blessed"`. For `"run"` / `"blessed"`, `id`
/// names a rater under the appropriate index; the rater is loaded, wrapped in
/// an `NnEvaluator` (with the sidecar's calibrated `eval_scale`), and
/// installed via `Match::set_evaluator`. For `"heuristic"`, `id` is ignored.
///
/// Errors leave the match's existing evaluator untouched.
#[tauri::command]
fn set_ai_evaluator(
    handle: u64,
    source: String,
    id: Option<String>,
    run_dir: Option<String>,
    registry: State<'_, EngineRegistry>,
) -> Result<(), String> {
    let evaluator: Box<dyn core_engine::search::evaluator::Evaluator + Send> = match source.as_str() {
        "heuristic" => Box::new(core_engine::search::evaluator::HeuristicEvaluator),
        "run" | "blessed" => {
            let id = id.ok_or_else(|| "rater id required for non-heuristic source".to_string())?;
            let dir = if source == "run" {
                run_dir
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(default_run_dir_path)
                    .join("raters")
            } else {
                blessed_raters_dir()
            };
            let stem = dir.join(&id);
            // Discriminate dense vs NNUE raters by the sidecar's input_dim:
            // NNUE raters use the sparse feature space (NUM_FEATURES) and must
            // load as an NnueEvaluator (quantized integer forward); dense raters
            // load as NnEvaluator (f32 forward over encode_position). Feeding a
            // dense-encoded vector into a sparse-topology model would panic on a
            // matmul dimension mismatch.
            let meta = nn_trainer::load_metadata(&stem)
                .map_err(|e| format!("load rater meta {id}: {e}"))?;
            if meta.model_config.input_dim == nn_trainer::NUM_FEATURES {
                let nnue = nn_trainer::NnueEvaluator::load_from_stem(&stem)
                    .map_err(|e| format!("load NNUE rater {id}: {e}"))?;
                Box::new(nnue)
            } else {
                let nn = nn_trainer::NnEvaluator::load_from_stem(&stem)
                    .map_err(|e| format!("load rater {id}: {e}"))?;
                Box::new(nn)
            }
        }
        other => return Err(format!("unknown evaluator source: {other}")),
    };
    registry.with(handle, |entry| entry.m.set_evaluator(evaluator))?;
    Ok(())
}

/// One entry in the `list_backends` response. `id` is the lowercase tag
/// (`"cpu"`, `"wgpu"`, `"cuda"`) - same value the frontend should send back
/// to `start_training_run`'s `backend` argument. `is_default` flags the
/// recommended preselection (`BackendChoice::default_choice`).
#[derive(serde::Serialize)]
struct BackendInfo {
    id: String,
    label: String,
    is_default: bool,
}

/// Parse a lowercase tag from the frontend into `BackendChoice`. `None`
/// (omitted argument) picks the build's default. Returns an Err on an
/// unknown tag or a tag whose feature is not compiled in.
fn parse_backend_choice(raw: Option<&str>) -> Result<nn_trainer::BackendChoice, String> {
    let Some(s) = raw else {
        return Ok(nn_trainer::BackendChoice::default_choice());
    };
    let want = match s {
        "cpu"  => nn_trainer::BackendChoice::Cpu,
        "wgpu" => nn_trainer::BackendChoice::Wgpu,
        "cuda" => nn_trainer::BackendChoice::Cuda,
        other  => return Err(format!("unknown backend: {other}")),
    };
    if !nn_trainer::BackendChoice::available().contains(&want) {
        return Err(format!("backend {s} not compiled into this build"));
    }
    Ok(want)
}

#[tauri::command]
fn list_backends() -> Vec<BackendInfo> {
    let default = nn_trainer::BackendChoice::default_choice();
    nn_trainer::BackendChoice::available()
        .into_iter()
        .map(|c| BackendInfo {
            id: c.as_str().to_string(),
            label: match c {
                nn_trainer::BackendChoice::Cpu  => "CPU (ndarray)".into(),
                nn_trainer::BackendChoice::Wgpu => "GPU (Metal / Vulkan / DX12)".into(),
                nn_trainer::BackendChoice::Cuda => "GPU (CUDA)".into(),
            },
            is_default: c == default,
        })
        .collect()
}

#[tauri::command]
fn start_training_run(
    run_dir: String,
    preset: Option<String>,
    backend: Option<String>,
    state: State<'_, TrainingState>,
) -> Result<(), String> {
    let mut inner = state.inner.lock().unwrap();
    if inner.handle.as_ref().map_or(false, |h| !h.is_finished()) {
        return Err("training already running".to_string());
    }
    let preset_name = preset.as_deref().unwrap_or("smoke").to_owned();
    let cfg = nn_trainer::RunConfig::from_preset(&preset_name)?;
    cfg.validate()?;
    let backend = parse_backend_choice(backend.as_deref())?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = Arc::clone(&stop);
    let path = std::path::PathBuf::from(run_dir);
    let handle = std::thread::spawn(move || {
        eprintln!("[training] thread started - preset={preset_name} backend={backend:?} run_dir={path:?}");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            nn_trainer::run_training(&cfg, &path, stop_clone, backend)
        }));
        match result {
            Ok(Ok(summary)) => eprintln!(
                "[training] run finished - generations_completed={} accepted_raters={} stopped_early={}",
                summary.generations_completed,
                summary.accepted_raters,
                summary.stopped_early,
            ),
            Ok(Err(e)) => eprintln!("[training] run FAILED: {e}"),
            Err(panic_val) => {
                let msg = panic_val
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| panic_val.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("(non-string panic)");
                eprintln!("[training] run PANICKED: {msg}");
            }
        }
    });
    inner.stop = Some(stop);
    inner.handle = Some(handle);
    Ok(())
}

/// Set the orchestrator's stop flag and forget the handle. Shared between the
/// `stop_training_run` IPC command and the app's exit hook so Cmd+Q triggers
/// the same wind-down path as clicking Stop.
///
/// No join: the run may take up to a full Tier-1 BO3 to wind down (the
/// orchestrator only checks the flag at phase boundaries) and blocking either
/// the IPC thread or the app-exit path on that would feel like a freeze. The
/// orchestrator writes a final `phase=Idle` snapshot on the way out, and on
/// exit the OS reaps the thread when the process tears down.
fn signal_stop(state: &TrainingState) {
    let mut inner = state.inner.lock().unwrap();
    if let Some(flag) = inner.stop.as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    inner.stop = None;
    inner.handle = None;
}

#[tauri::command]
fn stop_training_run(state: State<'_, TrainingState>) -> Result<(), String> {
    signal_stop(&state);
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point.
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EngineRegistry::default())
        .manage(TrainingState::default())
        .invoke_handler(tauri::generate_handler![
            engine_version,
            create_engine,
            create_engine_from_snapshot,
            create_engine_with_draft,
            create_engine_with_loadouts,
            draft_state,
            drop_engine,
            position_view,
            phase_state,
            position_fen,
            legal_actions,
            action_to_notation_cmd,
            try_apply,
            step_ai,
            heuristic_eval,
            heuristic_eval_by_square,
            request_ai_move_forced,
            request_ai_move_at_depth,
            match_log_json,
            latest_ply_json,
            finalise_log,
            snapshot_json,
            // Training Observatory
            default_run_dir,
            fen_to_position_view,
            read_training_status,
            read_training_live,
            subscribe_training_live,
            unsubscribe_training_live,
            read_rater_index,
            read_gauntlet_matrix,
            inspect_rater,
            list_available_raters,
            set_ai_evaluator,
            list_backends,
            start_training_run,
            stop_training_run,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // When the user quits the app (Cmd+Q, menu Quit, last-window-close
            // on platforms that bind it), signal the orchestrator to stop. We
            // do NOT call api.prevent_exit() - the runtime tears down and the
            // OS reaps the worker thread. The stop flag still lets the
            // orchestrator write a clean final snapshot before the process
            // disappears, so the next launch sees `phase=Idle` instead of a
            // stale "training" snapshot. WindowEvent::CloseRequested is the
            // wrong hook on macOS - red-light only hides the window - so we
            // use the app-level RunEvent::ExitRequested instead.
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let state = app_handle.state::<TrainingState>();
                signal_stop(&state);
            }
        });
}

// ---------------------------------------------------------------------------
// Smoke tests - exercise the registry layer without spinning up Tauri.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_handles_are_distinct_and_nonzero() {
        let r = EngineRegistry::default();
        let h1 = r.insert(Match::new(core_engine::Config::local_aivai()));
        let h2 = r.insert(Match::new(core_engine::Config::local_aivai()));
        assert_ne!(h1, h2);
        assert!(h1 >= 1 && h2 >= 1);
    }

    #[test]
    fn registry_with_unknown_handle_errors() {
        let r = EngineRegistry::default();
        let res = r.with::<()>(999, |_| ());
        assert!(res.is_err());
    }

    #[test]
    fn registry_drop_removes_entry() {
        let r = EngineRegistry::default();
        let h = r.insert(Match::new(core_engine::Config::local_aivai()));
        assert!(r.drop_handle(h));
        assert!(!r.drop_handle(h), "second drop is a no-op");
        assert!(r.with::<()>(h, |_| ()).is_err());
    }

    #[test]
    fn end_to_end_apply_snapshot_roundtrip() {
        let r = EngineRegistry::default();
        let h = r.insert(Match::new_with_clock(
            core_engine::Config::local_aivai(),
            1_700_000_000_000,
        ));

        let legal = r.with(h, |e| {
            api::legal_actions_into(&e.m, &mut e.legal_buf);
            e.legal_buf.clone()
        }).unwrap();
        assert!(!legal.is_empty());

        let raw = legal[0];
        let step = r.with(h, |e| {
            api::try_apply(&mut e.m, raw, 1_700_000_000_001).unwrap()
        }).unwrap();
        assert_eq!(step.applied_action, raw);

        let snap = r.with(h, |e| api::snapshot_json(&e.m)).unwrap();
        let m2   = api::from_snapshot_json(&snap, 1_700_000_000_002).unwrap();
        let h2   = r.insert(m2);
        assert_ne!(h, h2);

        let z1 = r.with(h,  |e| e.m.position().zobrist).unwrap();
        let z2 = r.with(h2, |e| e.m.position().zobrist).unwrap();
        assert_eq!(z1, z2, "snapshot round-trips the position");
    }

    #[test]
    fn step_result_dto_round_trips_via_ai() {
        let r = EngineRegistry::default();
        let h = r.insert(Match::new_with_clock(
            core_engine::Config::local_aivai(),
            1_700_000_000_000,
        ));
        let dto = r.with(h, |e| {
            api::step_ai(&mut e.m, 1_700_000_000_001)
                .map(StepResultDto::from)
                .unwrap()
        }).unwrap();
        assert_ne!(dto.applied_action, 0, "AI must produce a move on fresh board");
    }

    #[test]
    fn draft_engine_advances_through_preset_to_move_phase() {
        // End-to-end: open a Phase::Draft engine, drive 12 AI plies (each
        // applies one preset DraftTurn), assert we land in Phase::Move with
        // the preset loadout mirrored on both sides.
        let r = EngineRegistry::default();
        let m = api::new_match_with_draft(core_engine::Config::local_aivai(), 0);
        let h = r.insert(m);

        // Draft state at start: 0 turns committed, all slots empty, P1 to pick.
        let s_start = r.with(h, |e| {
            let s = api::current_draft_state(&e.m);
            DraftStateDto {
                turn_no:      s.turn_no,
                side_to_move: match s.side_to_move {
                    core_engine::state::position::Player::P1 => 0,
                    core_engine::state::position::Player::P2 => 1,
                },
                used_slots: s.used_slots,
            }
        }).unwrap();
        assert_eq!(s_start.turn_no, 0);
        assert_eq!(s_start.side_to_move, 0);
        assert!(s_start.used_slots.iter().flatten().all(|&b| !b));

        // Drive the draft to completion via step_ai (which dispatches into
        // the preset path when Phase == Draft).
        for _ in 0..12 {
            r.with(h, |e| {
                api::step_ai(&mut e.m, 0).expect("preset draft must always produce a turn");
            }).unwrap();
        }

        // Verify post-draft state: Phase::Move, all slots filled.
        let v = r.with(h, |e| api::position_view(&e.m)).unwrap();
        assert_eq!(v.current_phase, 0, "phase must transition to Move (encoded as 0)");
        let s_end = r.with(h, |e| api::current_draft_state(&e.m)).unwrap();
        assert_eq!(s_end.turn_no, 12, "draft_state must report sentinel 12 once finished");
    }

    #[test]
    fn create_engine_with_loadouts_validates_input() {
        // Same skill in both slots of a single piece is rejected by validate_loadout.
        let r = EngineRegistry::default();
        let bad_p1 = "[[6,6],[1,9],[2,8],[3,14],[5,15],[12,11]]";
        let good   = "[[6,7],[1,9],[2,8],[3,14],[5,15],[12,11]]";
        let p1 = api::parse_side_loadout_json(bad_p1).unwrap();
        let p2 = api::parse_side_loadout_json(good).unwrap();
        let err = api::new_match_with_loadouts(core_engine::Config::local_aivai(), &p1, &p2, 0);
        assert!(err.is_err(), "duplicate skill on same piece must fail validation");

        // Good loadouts produce a Phase::Move engine.
        let p1g = api::parse_side_loadout_json(good).unwrap();
        let p2g = api::parse_side_loadout_json(good).unwrap();
        let m = api::new_match_with_loadouts(core_engine::Config::local_aivai(), &p1g, &p2g, 0)
            .expect("valid loadouts must construct");
        let h = r.insert(m);
        let v = r.with(h, |e| api::position_view(&e.m)).unwrap();
        assert_eq!(v.current_phase, 0, "loadout path must skip draft → Phase::Move");
    }

    // --- Training Observatory smoke tests ------------------------------------
    //
    // We can't spin up Tauri here, but the command functions are plain Rust -
    // we can exercise them directly and confirm the serde shapes round-trip.

    fn tempdir() -> std::path::PathBuf {
        use std::sync::atomic::AtomicU64;
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let n = NONCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir()
            .join(format!("tauri_wrapper_training_{}_{}", pid, n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_run_dir_is_under_game_runs_active() {
        let path = default_run_dir().unwrap();
        // The path must end in `runs/active`. We don't assert the absolute
        // prefix because tests run from various invocation roots.
        assert!(
            path.ends_with("runs/active") || path.ends_with("runs\\active"),
            "default run dir should end in runs/active, got {path}",
        );
    }

    #[test]
    fn fen_to_position_view_round_trips_stack_m_start() {
        // FEN of the Stack M start position. We round-trip it through the
        // command and confirm the major fields land in plausible places.
        let pos = core_engine::state::Position::setup_stack_m();
        let fen = core_engine::state::fen::to_fen(&pos);
        let dto = fen_to_position_view(fen).expect("parse");
        assert_eq!(dto.to_move, 0, "P1 starts");
        assert_eq!(dto.current_phase, 0, "Move phase");
        assert_eq!(dto.game_result, 0, "ongoing");
        assert_eq!(dto.mailbox.len(), 64);
        assert_ne!(dto.bitboards[0], "0", "P1 occupancy non-empty at start");
    }

    #[test]
    fn read_training_status_missing_returns_none() {
        let dir = tempdir();
        let got = read_training_status(dir.to_string_lossy().to_string()).expect("read");
        assert!(got.is_none(), "missing snapshot must return None");
    }

    #[test]
    fn read_training_live_missing_returns_none() {
        let dir = tempdir();
        let got = read_training_live(dir.to_string_lossy().to_string()).expect("read");
        assert!(got.is_none());
    }

    #[test]
    fn subscribe_then_unsubscribe_is_idempotent() {
        let dir = tempdir();
        let s = dir.to_string_lossy().to_string();
        subscribe_training_live(s.clone()).expect("subscribe");
        subscribe_training_live(s.clone()).expect("subscribe again");
        unsubscribe_training_live(s.clone()).expect("unsubscribe");
        unsubscribe_training_live(s).expect("unsubscribe again");
    }

    #[test]
    fn read_rater_index_missing_returns_none() {
        let dir = tempdir();
        let got = read_rater_index(dir.to_string_lossy().to_string()).expect("read");
        assert!(got.is_none());
    }

    #[test]
    fn read_gauntlet_matrix_missing_returns_none() {
        let dir = tempdir();
        let got = read_gauntlet_matrix(dir.to_string_lossy().to_string()).expect("read");
        assert!(got.is_none());
    }

    #[test]
    fn read_gauntlet_matrix_after_save_returns_entries() {
        // Save a one-cell matrix, then read it back through the command.
        let dir = tempdir();
        let mut m = nn_trainer::GauntletMatrix::default();
        m.record_series(
            "v0001", "v0000", "fast",
            nn_trainer::SeriesTally { candidate_wins: 2, baseline_wins: 1, indecisive: 0 },
        );
        nn_trainer::save_matrix(&dir, &m).expect("save");

        let got = read_gauntlet_matrix(dir.to_string_lossy().to_string())
            .expect("read")
            .expect("present");
        assert_eq!(got.entries.len(), 1);
        assert_eq!(got.entries[0].result.candidate_wins, 2);
    }

    #[test]
    fn signal_stop_sets_flag_and_clears_inner() {
        // Populated state: signal_stop must flip the underlying AtomicBool
        // (observable through a clone held outside the state) and clear both
        // `stop` and `handle` so the next start_training_run sees a fresh slot.
        let state = TrainingState::default();
        let flag = Arc::new(AtomicBool::new(false));
        let observer = Arc::clone(&flag);
        let handle = std::thread::spawn(|| {});
        {
            let mut inner = state.inner.lock().unwrap();
            inner.stop = Some(flag);
            inner.handle = Some(handle);
        }

        signal_stop(&state);

        assert!(observer.load(Ordering::Relaxed), "stop flag must be true");
        let inner = state.inner.lock().unwrap();
        assert!(inner.stop.is_none(), "stop slot must be cleared");
        assert!(inner.handle.is_none(), "handle slot must be cleared");
    }

    #[test]
    fn signal_stop_on_empty_state_is_noop() {
        // No run in progress: signal_stop must not panic. Hit on every quit,
        // including quits when nothing was training.
        let state = TrainingState::default();
        signal_stop(&state);
        let inner = state.inner.lock().unwrap();
        assert!(inner.stop.is_none());
        assert!(inner.handle.is_none());
    }
}
