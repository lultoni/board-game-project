//! tauri_wrapper — Tauri 2 desktop wrapper.
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
//! Handles are u64 monotonic counters — never reused, never zero (zero is
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
//!   the IPC bridge — no zero-copy across the boundary, but native build is
//!   fast enough that the copy is invisible.
//! - **Cold path** (snapshot, log export): owned JSON strings.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use core_engine::wrapper_api as api;
use core_engine::Match;

// ---------------------------------------------------------------------------
// Handle table — process-global, one per Match.
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
// Wire types — mirror the wasm_wrapper JS shape exactly. camelCase via serde
// so the frontend reads `appliedAction`, not `applied_action`.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionViewDto {
    pub bitboards:         [u64; 5],
    pub mailbox:           Vec<u16>,
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
// Commands — names mirror wasm_wrapper.Engine methods. Tauri exposes them on
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

#[tauri::command]
fn drop_engine(handle: u64, registry: State<'_, EngineRegistry>) -> bool {
    registry.drop_handle(handle)
}

#[tauri::command]
fn position_view(handle: u64, registry: State<'_, EngineRegistry>) -> Result<PositionViewDto, String> {
    registry.with(handle, |e| {
        let v  = api::position_view(&e.m);
        let mb = api::position_mailbox(&e.m);
        PositionViewDto {
            bitboards:         v.bitboards,
            mailbox:           mb.to_vec(),
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
fn legal_actions(handle: u64, registry: State<'_, EngineRegistry>) -> Result<Vec<u32>, String> {
    registry.with(handle, |e| {
        api::legal_actions_into(&e.m, &mut e.legal_buf);
        e.legal_buf.clone()
    })
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

/// AI step. CPU-bound — runs inside `block_in_place` so the Tauri async
/// runtime keeps responding to other IPC traffic.
#[tauri::command]
async fn step_ai(
    handle:   u64,
    registry: State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let now = unix_ms_now();
    let res: Result<Result<StepResultDto, String>, String> =
        tokio::task::block_in_place(|| {
            registry.with(handle, |e| {
                api::step_ai(&mut e.m, now)
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
// Entry point.
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EngineRegistry::default())
        .invoke_handler(tauri::generate_handler![
            engine_version,
            create_engine,
            create_engine_from_snapshot,
            drop_engine,
            position_view,
            phase_state,
            position_fen,
            legal_actions,
            try_apply,
            step_ai,
            match_log_json,
            finalise_log,
            snapshot_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---------------------------------------------------------------------------
// Smoke tests — exercise the registry layer without spinning up Tauri.
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
}
