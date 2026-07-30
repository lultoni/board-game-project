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

/// Recover a poisoned mutex rather than propagating the panic. A panic inside
/// a `with` closure (e.g. a search bug in core_engine) poisons the Mutex;
/// without recovery every subsequent IPC call crashes on `.unwrap()`. The
/// data inside is still valid — only the poison flag is cleared.
fn lock_unpoisoned<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl EngineRegistry {
    fn fresh_handle(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn insert(&self, m: Match) -> u64 {
        let h = self.fresh_handle();
        lock_unpoisoned(&self.engines).insert(h, EngineEntry {
            m,
            legal_buf: Vec::with_capacity(256),
        });
        h
    }

    fn with<R>(&self, handle: u64, f: impl FnOnce(&mut EngineEntry) -> R) -> Result<R, String> {
        let mut guard = lock_unpoisoned(&self.engines);
        let entry = guard.get_mut(&handle)
            .ok_or_else(|| format!("unknown engine handle {handle}"))?;
        Ok(f(entry))
    }

    fn drop_handle(&self, handle: u64) -> bool {
        lock_unpoisoned(&self.engines).remove(&handle).is_some()
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

// ---------------------------------------------------------------------------
// Static metadata - the engine is the single source of truth for the skill
// table and game constants the frontend mirrors. `skill_metadata` /
// `game_constants_cmd` are pure reads (no engine handle); the frontend asserts
// its synchronous mirror against them in a contract test so the two can't drift.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkillMetadataDto {
    pub id:            u8,
    pub key:           String,
    pub category:      String,
    pub cost:          u8,
    pub default_range: u8,
    pub target_owner:  String,
    pub has_focus_mode_choice: bool,
    pub needs_direction_pick:  bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameConstantsDto {
    pub phase_move:                u8,
    pub phase_skill:               u8,
    pub phase_draft:               u8,
    pub modifier_focus:            u8,
    pub modifier_charge:           u8,
    pub modifier_move_attack_used: u8,
    pub player_p1:                 u8,
    pub player_p2:                 u8,
    pub game_ongoing:              u8,
    pub game_p1_wins:              u8,
    pub game_p2_wins:              u8,
    pub skill_count:               u8,
}

#[tauri::command]
fn skill_metadata() -> Vec<SkillMetadataDto> {
    core_engine::all_skill_metadata()
        .into_iter()
        .map(|m| SkillMetadataDto {
            id:            m.id,
            key:           m.key.to_string(),
            category:      m.category.to_string(),
            cost:          m.cost,
            default_range: m.default_range,
            target_owner:  m.target_owner.to_string(),
            has_focus_mode_choice: m.has_focus_mode_choice,
            needs_direction_pick:  m.needs_direction_pick,
        })
        .collect()
}

#[tauri::command]
fn game_constants_cmd() -> GameConstantsDto {
    let c = core_engine::game_constants();
    GameConstantsDto {
        phase_move:                c.phase_move,
        phase_skill:               c.phase_skill,
        phase_draft:               c.phase_draft,
        modifier_focus:            c.modifier_focus,
        modifier_charge:           c.modifier_charge,
        modifier_move_attack_used: c.modifier_move_attack_used,
        player_p1:                 c.player_p1,
        player_p2:                 c.player_p2,
        game_ongoing:              c.game_ongoing,
        game_p1_wins:              c.game_p1_wins,
        game_p2_wins:              c.game_p2_wins,
        skill_count:               c.skill_count,
    }
}

#[tauri::command]
async fn try_apply(
    handle:           u64,
    raw_action:       u32,
    turn_started_ms:  Option<u64>,
    background_eval:  Option<bool>,
    app:              tauri::AppHandle,
    registry:         State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let now = unix_ms_now();
    // Human decision time = now − start-of-turn. A missing / zero start means
    // "no timing available" (replay, inspector, snapshot rebuild) → record 0.
    let turn_started_ms = turn_started_ms.unwrap_or(0);
    let thought_ms = if turn_started_ms == 0 {
        0
    } else {
        now.saturating_sub(turn_started_ms).min(u32::MAX as u64) as u32
    };
    let dto = registry.with(handle, |e| {
        api::try_apply_with_thought(&mut e.m, raw_action, thought_ms, now)
            .map(StepResultDto::from)
            .map_err(|err| format!("{err:?}"))
    })??;

    // Background eval: annotate the last ply with a shallow search result.
    // The search must NOT hold the registry mutex — it takes ~1 s and would
    // stall every subsequent IPC call on this handle for that duration.
    // Instead: clone the position under a brief lock, do the search lock-free,
    // then re-acquire the lock just to write the result back.
    if background_eval.unwrap_or(false) {
        // Step 1: clone the position while holding the lock (microseconds).
        let pos_snapshot = registry.with(handle, |e| e.m.prepare_background_eval());
        if let Ok(Some(mut pos)) = pos_snapshot {
            let app_for_task = app.clone();
            tokio::task::spawn_blocking(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    // Step 2: run the search with NO registry lock held.
                    use core_engine::search::evaluator::HeuristicEvaluator;
                    use core_engine::search::alpha_beta::find_best_with_evaluator;
                    use core_engine::search::transposition::TranspositionTable;
                    use core_engine::telemetry::SearchMeta;
                    use core_engine::search::evaluator::evaluate_breakdown;
                    const BUDGET_MS: u64 = 1000;
                    const MAX_DEPTH: u8 = 20;
                    let mut tt = TranspositionTable::with_capacity_mb(4);
                    let r = find_best_with_evaluator(
                        &mut pos, &mut tt, BUDGET_MS, MAX_DEPTH,
                        &HeuristicEvaluator, None,
                    );
                    let breakdown = evaluate_breakdown(&pos);
                    let meta = SearchMeta::from_search_with_breakdown(
                        r.depth, r.nodes, r.score, Some(breakdown),
                    );
                    // Step 3: re-acquire lock only to write the result (microseconds).
                    let registry = app_for_task.state::<EngineRegistry>();
                    let _ = registry.with(handle, |e| e.m.write_background_eval(meta));
                }));
                let _ = app_for_task.emit(
                    "background-eval-ready",
                    serde_json::json!({ "handle": handle }),
                );
            });
        }
    }

    Ok(dto)
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
use std::sync::atomic::{AtomicBool, AtomicUsize};

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
/// Build a position evaluator from a `(source, id, run_dir)` triple. `source`
/// is `"heuristic"`, `"run"`, or `"blessed"`; for the NN sources `id` names a
/// rater under the appropriate index. Shared by the `set_ai_evaluator` command
/// and the background AIvAI producer (which must re-install evaluators after
/// building its Match from a snapshot, since `from_snapshot` resets the
/// evaluator to `HeuristicEvaluator`).
fn build_evaluator(
    source: &str,
    id: Option<String>,
    run_dir: Option<String>,
) -> Result<Box<dyn core_engine::search::evaluator::Evaluator + Send>, String> {
    match source {
        "heuristic" => Ok(Box::new(core_engine::search::evaluator::HeuristicEvaluator)),
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
                Ok(Box::new(nnue))
            } else {
                let nn = nn_trainer::NnEvaluator::load_from_stem(&stem)
                    .map_err(|e| format!("load rater {id}: {e}"))?;
                Ok(Box::new(nn))
            }
        }
        other => Err(format!("unknown evaluator source: {other}")),
    }
}

#[tauri::command]
fn set_ai_evaluator(
    handle: u64,
    source: String,
    id: Option<String>,
    run_dir: Option<String>,
    registry: State<'_, EngineRegistry>,
) -> Result<(), String> {
    let evaluator = build_evaluator(&source, id, run_dir)?;
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
// AIvAI background producer (plan §6, Change 6).
//
// For AI-vs-AI matches the engine plays the whole game to completion on its
// own background thread as fast as it can, appending each ply to ITS match
// log. The frontend is a "log player" that replays the producer's log at the
// display cadence through a SEPARATE view engine. This decouples engine speed
// from display rate: the producer races ahead; the view renders when it wants.
//
// Key design points (see the plan doc):
//   * The producer owns its OWN `Match` MOVED INTO THE THREAD - it is NOT in
//     the shared `EngineRegistry`, so its ~1s-per-ply search never holds the
//     registry mutex and never starves the view engine's sub-ms reads.
//   * It publishes its latest log JSON + ply count into shared state each ply
//     so the frontend can pull raw actions and raise its "known ply" ceiling.
//   * Abort is loop-level only (no mid-search abort): the loop checks the flag
//     between plies. On leave the frontend awaits `stop_aivai_producer`, which
//     sets the flag and JOINS the thread so the in-flight ply is appended
//     before the authoritative log is read - guaranteeing the saved log length
//     equals what the producer actually computed.
// ---------------------------------------------------------------------------

/// Shared state written by the producer thread and read by the frontend via
/// `aivai_producer_log`. `log` holds the latest serialized `MatchLog`; `plies`
/// is the ply-count ceiling the frontend log-player advances toward.
#[derive(Default)]
struct AivaiProducerShared {
    log:   Mutex<Option<String>>,
    plies: AtomicUsize,
}

#[derive(Default)]
struct AivaiProducerInner {
    abort:  Option<Arc<AtomicBool>>,
    handle: Option<std::thread::JoinHandle<()>>,
    shared: Option<Arc<AivaiProducerShared>>,
}

/// Process-global AIvAI producer state. At most one producer runs at a time
/// (a new match aborts any prior producer first). Registered with `.manage`.
#[derive(Default)]
pub struct AivaiProducerState {
    inner: Mutex<AivaiProducerInner>,
}

/// Set the abort flag and JOIN the producer thread, returning its final
/// (authoritative) log JSON. Shared between the `stop_aivai_producer` command
/// and the app exit hook. Bounded by one ply's think-time: the loop only
/// checks the flag between plies, so a join waits at most for the in-flight
/// search to finish (then the ply is appended + the log is finalised).
fn join_aivai_producer(state: &AivaiProducerState) -> Option<String> {
    let (abort, handle, shared) = {
        let mut inner = state.inner.lock().unwrap();
        (inner.abort.take(), inner.handle.take(), inner.shared.take())
    };
    if let Some(flag) = abort.as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    if let Some(h) = handle {
        // The thread writes the finalised log into `shared` before exiting, so
        // joining guarantees we read the complete log below.
        let _ = h.join();
    }
    shared.and_then(|s| s.log.lock().unwrap().clone())
}

/// Start the background AIvAI producer from the view engine's snapshot so the
/// producer and view share an identical `start_fen` + config. Aborts any prior
/// producer first. `p{1,2}_source`/`id` re-install the seat evaluators, because
/// `from_snapshot` resets the evaluator to `HeuristicEvaluator` - without this
/// the producer would silently play the heuristic instead of the picked rater.
#[tauri::command]
fn start_aivai_producer(
    view_snapshot_json: String,
    p1_source: String,
    p1_id: Option<String>,
    p2_source: String,
    p2_id: Option<String>,
    app: tauri::AppHandle,
    state: State<'_, AivaiProducerState>,
) -> Result<(), String> {
    // Abort + join any prior producer (e.g. a rapid re-entry into /match/).
    let _ = join_aivai_producer(&state);

    let mut m = api::from_snapshot_json(&view_snapshot_json, unix_ms_now())
        .map_err(|e| format!("producer snapshot parse: {e:?}"))?;

    // Re-install both seats' evaluators. Match holds a single evaluator, so the
    // second install wins - mirroring the frontend's `applyEvaluatorSettings`
    // (which calls setAiEvaluator twice, p1 then p2). We keep that behaviour
    // identical rather than introducing per-seat evaluators here.
    if let Ok(e1) = build_evaluator(&p1_source, p1_id, None) {
        m.set_evaluator(e1);
    }
    if let Ok(e2) = build_evaluator(&p2_source, p2_id, None) {
        m.set_evaluator(e2);
    }

    let shared = Arc::new(AivaiProducerShared::default());
    // Seed the published log with the pre-play state so a very-early leave
    // (before the first ply completes) still reads a valid (possibly 0-ply)
    // log rather than None.
    *shared.log.lock().unwrap() = api::match_log_json(&m);
    shared.plies.store(api::log_ply_count(&m), Ordering::Relaxed);

    let abort = Arc::new(AtomicBool::new(false));
    let abort_thread = Arc::clone(&abort);
    let shared_thread = Arc::clone(&shared);
    let app_thread = app.clone();

    let handle = std::thread::spawn(move || {
        // Catch panics so a search bug tears down the producer cleanly instead
        // of poisoning the process (mirrors the training thread).
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_aivai_producer_loop(&mut m, &abort_thread, &shared_thread, &app_thread);
        }));
    });

    let mut inner = state.inner.lock().unwrap();
    inner.abort = Some(abort);
    inner.handle = Some(handle);
    inner.shared = Some(shared);
    Ok(())
}

/// The producer's play-to-completion loop. Steps the AI ply-by-ply until the
/// game ends, a no-move wedge is hit, or the abort flag is set. Publishes the
/// log + ply count and emits a throttled `aivai-progress` event after each ply.
fn run_aivai_producer_loop(
    m: &mut Match,
    abort: &AtomicBool,
    shared: &AivaiProducerShared,
    app: &tauri::AppHandle,
) {
    loop {
        if abort.load(Ordering::Relaxed) {
            break;
        }
        let now = unix_ms_now();
        match api::step_ai(m, now) {
            // Natural end (mate) or a misconfigured non-AI seat: stop the loop.
            Err(_) => break,
            // No-move on a live position (engine wedge): STOP rather than spin
            // at 100% CPU re-searching the same position forever. The frontend
            // has an `aiAutoPlay=false` guard for this; the headless loop must
            // break explicitly.
            Ok(r) if r.applied_action == 0 => break,
            Ok(_) => {}
        }
        // Publish the freshly-extended log + ceiling for the frontend to pull.
        *shared.log.lock().unwrap() = api::match_log_json(m);
        let n = api::log_ply_count(m);
        shared.plies.store(n, Ordering::Relaxed);
        let _ = app.emit("aivai-progress", serde_json::json!({ "plies": n }));

        if m.game_result().is_some() {
            break;
        }
    }
    // Finalise: pick the terminal result (or Aborted for a leave / wedge) and
    // stamp the log so the persisted library entry is complete.
    let now = unix_ms_now();
    let result_byte = api::finalise_result_byte(m);
    api::finalise_log(m, now, result_byte);
    *shared.log.lock().unwrap() = api::match_log_json(m);
    let n = api::log_ply_count(m);
    shared.plies.store(n, Ordering::Relaxed);
    let _ = app.emit("aivai-progress", serde_json::json!({ "plies": n, "done": true }));
}

/// Non-joining read of the producer's currently-published log. Used by the
/// frontend log-player to pull raw actions and by the natural-end finalise
/// path (the producer has already finished by the time the view plays through
/// to the game-over ply, so this holds the complete log).
#[tauri::command]
fn aivai_producer_log(state: State<'_, AivaiProducerState>) -> Option<String> {
    let inner = state.inner.lock().unwrap();
    inner.shared.as_ref().and_then(|s| s.log.lock().unwrap().clone())
}

/// Abort + join the producer and return its final authoritative log. The
/// frontend awaits this on leaving an AIvAI match: joining guarantees the
/// in-flight ply is appended and the log finalised before we persist, so the
/// saved log length equals exactly what the producer computed.
#[tauri::command]
async fn stop_aivai_producer(state: State<'_, AivaiProducerState>) -> Result<Option<String>, String> {
    // The join can block up to one ply's think-time; run it off the async
    // executor so we don't pin a runtime worker.
    Ok(tokio::task::block_in_place(|| join_aivai_producer(&state)))
}

// ---------------------------------------------------------------------------
// Entry point.
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EngineRegistry::default())
        .manage(TrainingState::default())
        .manage(AivaiProducerState::default())
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
            skill_metadata,
            game_constants_cmd,
            try_apply,
            step_ai,
            start_aivai_producer,
            aivai_producer_log,
            stop_aivai_producer,
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
                // Abort + join any running AIvAI producer so Cmd+Q doesn't
                // leave a detached search thread racing the process teardown.
                let producer = app_handle.state::<AivaiProducerState>();
                let _ = join_aivai_producer(&producer);
            }
        });
}

// ---------------------------------------------------------------------------
// Smoke tests - exercise the registry layer without spinning up Tauri.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Regenerate the frontend contract fixture from the engine's canonical
    /// metadata. The frontend's `skills.contract.test.ts` asserts its
    /// synchronous `SKILLS` mirror against this file, so the two can only drift
    /// if this fixture is stale. Run `cargo test -p tauri_wrapper
    /// emit_skill_metadata_fixture` after any skill-table change to refresh it;
    /// the Rust `skill_metadata` / `game_constants_cmd` DTOs are the source.
    #[test]
    fn emit_skill_metadata_fixture() {
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let fixture = std::path::Path::new(crate_dir)
            .parent()  // crates/
            .and_then(|p| p.parent()) // game/
            .expect("crate dir has game/ ancestor")
            .join("frontend/src/lib/engine/__fixtures__/skill-metadata.json");
        let payload = serde_json::json!({
            "skills": skill_metadata(),
            "constants": game_constants_cmd(),
        });
        let json = serde_json::to_string_pretty(&payload).unwrap() + "\n";
        std::fs::create_dir_all(fixture.parent().unwrap()).unwrap();
        // Only rewrite when the content changes so the test is idempotent and
        // doesn't dirty the tree on every run.
        let stale = std::fs::read_to_string(&fixture).map_or(true, |existing| existing != json);
        if stale {
            std::fs::write(&fixture, &json).unwrap();
        }
        // Sanity: the fixture round-trips and covers all 15 skills.
        assert_eq!(skill_metadata().len(), 15);
    }

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

    #[test]
    fn join_aivai_producer_on_empty_state_is_noop() {
        // No producer running: joining must not panic and returns None. Hit on
        // the exit hook when no AIvAI match was ever started.
        let state = AivaiProducerState::default();
        assert!(join_aivai_producer(&state).is_none());
        // Idempotent - a second join is still a clean no-op.
        assert!(join_aivai_producer(&state).is_none());
        let inner = state.inner.lock().unwrap();
        assert!(inner.abort.is_none());
        assert!(inner.handle.is_none());
        assert!(inner.shared.is_none());
    }

    #[test]
    fn join_aivai_producer_returns_published_log_and_clears() {
        // A producer that has finished leaves its abort flag set, its thread
        // joinable, and its final log published in `shared`. join_ must return
        // that log and clear all three slots so the next start sees a fresh
        // state. We simulate this without a real AppHandle by populating the
        // state the way `start_aivai_producer` would and letting the thread
        // exit immediately.
        let state = AivaiProducerState::default();
        let shared = Arc::new(AivaiProducerShared::default());
        *shared.log.lock().unwrap() = Some("{\"plies\":[]}".to_string());
        shared.plies.store(0, Ordering::Relaxed);
        let abort = Arc::new(AtomicBool::new(false));
        let handle = std::thread::spawn(|| { /* exits immediately */ });
        {
            let mut inner = state.inner.lock().unwrap();
            inner.abort = Some(abort);
            inner.handle = Some(handle);
            inner.shared = Some(Arc::clone(&shared));
        }

        let log = join_aivai_producer(&state);
        assert_eq!(log.as_deref(), Some("{\"plies\":[]}"));

        let inner = state.inner.lock().unwrap();
        assert!(inner.abort.is_none(), "abort slot cleared after join");
        assert!(inner.handle.is_none(), "handle slot cleared after join");
        assert!(inner.shared.is_none(), "shared slot cleared after join");
    }
}
