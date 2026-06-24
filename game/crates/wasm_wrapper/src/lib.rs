//! wasm_wrapper — wasm-bindgen surface for the browser build.
//!
//! Designed for use inside a Web Worker. The worker owns an `Engine` (which
//! owns a `core_engine::Match`); the main thread drives it via `postMessage`.
//!
//! # Performance contract
//!
//! - **Hot path** (board reads, legal actions, step): zero-copy typed-array
//!   views over wasm linear memory. The returned `Uint32Array` / `Uint16Array`
//!   / `BigUint64Array` aliases the engine's internal buffers. JS callers
//!   MUST copy if they hold the value past the next call into `Engine` —
//!   subsequent mutations can invalidate the view.
//! - **Cold path** (telemetry, snapshot): owned JSON strings.
//!
//! # Clock binding
//!
//! `core_engine::time::now_ms()` on `wasm32` imports `engine_now_ms` from the
//! host. We export it here as `#[no_mangle]` so the linker resolves the
//! reference; the implementation calls `performance.now()` (which works in
//! both `Window` and `DedicatedWorkerGlobalScope`).

use wasm_bindgen::prelude::*;
use core_engine::wrapper_api as api;

// ---------------------------------------------------------------------------
// Clock binding — satisfies `extern "C" { fn engine_now_ms() -> u64; }` in
// `core_engine::time` (wasm32 branch).
// ---------------------------------------------------------------------------

#[wasm_bindgen]
extern "C" {
    /// `globalThis.performance.now()` — high-resolution monotonic ms.
    /// Works in both Window and DedicatedWorkerGlobalScope contexts.
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn perf_now() -> f64;
}

#[no_mangle]
pub extern "C" fn engine_now_ms() -> u64 {
    perf_now() as u64
}

// ---------------------------------------------------------------------------
// Engine — one per Match. The worker constructs a single Engine; multiple
// matches = multiple workers (or multiple Engine instances if the worker
// wants to multiplex).
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct Engine {
    m: core_engine::Match,
    /// Reusable buffer for legal-action enumeration. Sized once, reused per
    /// call. JS gets a `Uint32Array::view()` over this — valid until the
    /// next mutating call on Engine.
    legal_buf: Vec<u32>,
}

#[wasm_bindgen]
impl Engine {
    /// Construct an Engine with default Stack-M setup using `Config::local_aivai()`.
    /// Most callers will follow up with `new_with_config_json` to pick a mode.
    #[wasm_bindgen(constructor)]
    pub fn new(now_unix_ms: f64) -> Engine {
        console_error_panic_hook::set_once();
        let cfg = core_engine::Config::local_aivai();
        let m = core_engine::Match::new_with_clock(cfg, now_unix_ms as u64);
        Engine { m, legal_buf: Vec::with_capacity(256) }
    }

    /// Construct an Engine from a JSON-encoded `Config`. Use for HvH/HvAI
    /// modes. Returns an Error if the config doesn't parse.
    #[wasm_bindgen(js_name = newWithConfigJson)]
    pub fn new_with_config_json(config_json: &str, now_unix_ms: f64) -> Result<Engine, JsValue> {
        console_error_panic_hook::set_once();
        let cfg: core_engine::Config = core_engine::from_json(config_json)
            .map_err(|e| JsValue::from_str(&format!("config parse error: {e}")))?;
        let m = core_engine::Match::new_with_clock(cfg, now_unix_ms as u64);
        Ok(Engine { m, legal_buf: Vec::with_capacity(256) })
    }

    /// Restore an Engine from a snapshot JSON (`Engine::snapshotJson`).
    #[wasm_bindgen(js_name = fromSnapshotJson)]
    pub fn from_snapshot_json(snapshot_json: &str, now_unix_ms: f64) -> Result<Engine, JsValue> {
        console_error_panic_hook::set_once();
        let m = api::from_snapshot_json(snapshot_json, now_unix_ms as u64)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Engine { m, legal_buf: Vec::with_capacity(256) })
    }

    // --- Position reads (hot path, zero-copy views) ------------------------

    /// `[p1, p2, kings, champions, guards]` as a `BigUint64Array` view.
    /// View aliases wasm memory; copy if held past next call.
    #[wasm_bindgen(js_name = positionBitboards)]
    pub fn position_bitboards(&self) -> js_sys::BigUint64Array {
        let v = api::position_view(&self.m);
        // SAFETY: caller is contracted to copy before re-entering Engine.
        unsafe { js_sys::BigUint64Array::view(&v.bitboards) }
    }

    /// 64-square mailbox as a `Uint16Array` view aliasing wasm memory.
    #[wasm_bindgen(js_name = positionMailbox)]
    pub fn position_mailbox(&self) -> js_sys::Uint16Array {
        let mb = api::position_mailbox(&self.m);
        // SAFETY: caller is contracted to copy before re-entering Engine.
        unsafe { js_sys::Uint16Array::view(mb.as_slice()) }
    }

    /// Tiny owned `PhaseStateJs` — phase/round/money/modifiers/result.
    #[wasm_bindgen(js_name = phaseState)]
    pub fn phase_state(&self) -> PhaseStateJs {
        let v = api::position_view(&self.m);
        PhaseStateJs {
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
    }

    /// Debug-only FEN.
    #[wasm_bindgen(js_name = positionFen)]
    pub fn position_fen(&self) -> String {
        api::position_fen(&self.m)
    }

    // --- Play (hot path) ---------------------------------------------------

    /// Returns a `Uint32Array` view over the engine's legal-actions buffer.
    /// Re-populated on every call; view valid until next Engine call.
    #[wasm_bindgen(js_name = legalActions)]
    pub fn legal_actions(&mut self) -> js_sys::Uint32Array {
        api::legal_actions_into(&self.m, &mut self.legal_buf);
        // SAFETY: caller is contracted to copy before re-entering Engine.
        unsafe { js_sys::Uint32Array::view(&self.legal_buf) }
    }

    /// Apply a human action. Returns a flat `StepResultJs`.
    #[wasm_bindgen(js_name = tryApply)]
    pub fn try_apply(
        &mut self,
        raw_action: u32,
        applied_at_unix_ms: f64,
    ) -> Result<StepResultJs, JsValue> {
        api::try_apply(&mut self.m, raw_action, applied_at_unix_ms as u64)
            .map(StepResultJs::from)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Run AI search under the side-to-move's configured budget and apply
    /// the chosen action. Returns a flat `StepResultJs`.
    #[wasm_bindgen(js_name = stepAi)]
    pub fn step_ai(&mut self, applied_at_unix_ms: f64) -> Result<StepResultJs, JsValue> {
        api::step_ai(&mut self.m, applied_at_unix_ms as u64)
            .map(StepResultJs::from)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Like `stepAi` but **does not apply** — returns the best move the AI
    /// found without mutating state. Used by the inspector to surface
    /// suggested moves the user can choose to apply (or ignore). The
    /// returned `applied_action` is the candidate; the caller decides what
    /// to do with it.
    #[wasm_bindgen(js_name = requestAiMove)]
    pub fn request_ai_move(&mut self) -> Result<StepResultJs, JsValue> {
        api::request_ai_move(&mut self.m)
            .map(StepResultJs::from)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Like `requestAiMove`, but runs the search regardless of seat kind
    /// (HvH matches included). The inspector uses this so the user can ask
    /// "what would a strong player do here?" at any position.
    #[wasm_bindgen(js_name = requestAiMoveForced)]
    pub fn request_ai_move_forced(&mut self) -> Result<StepResultJs, JsValue> {
        api::request_ai_move_forced(&mut self.m)
            .map(StepResultJs::from)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Iterative-deepening helper for the inspector. Runs ID up to
    /// `maxDepth` with no time bound; caller drives the loop and polls
    /// cancellation between calls.
    #[wasm_bindgen(js_name = requestAiMoveAtDepth)]
    pub fn request_ai_move_at_depth(&mut self, max_depth: u8) -> Result<StepResultJs, JsValue> {
        api::request_ai_move_at_depth(&mut self.m, max_depth)
            .map(StepResultJs::from)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    // --- Telemetry / persistence (cold path) -------------------------------

    /// `MatchLog` JSON if `auto_log` is on, else `null` (mapped from Option).
    #[wasm_bindgen(js_name = matchLogJson)]
    pub fn match_log_json(&self) -> Option<String> {
        api::match_log_json(&self.m)
    }

    /// `result_byte`: 0=P1Win, 1=P2Win, 2=Draw, 3=Aborted.
    #[wasm_bindgen(js_name = finaliseLog)]
    pub fn finalise_log(&mut self, now_unix_ms: f64, result_byte: u8) {
        api::finalise_log(&mut self.m, now_unix_ms as u64, result_byte);
    }

    /// Save-game snapshot JSON.
    #[wasm_bindgen(js_name = snapshotJson)]
    pub fn snapshot_json(&self) -> String {
        api::snapshot_json(&self.m)
    }
}

// ---------------------------------------------------------------------------
// Flat data structures crossing the boundary. Each is a #[wasm_bindgen]
// struct with getters; JS reads via `result.appliedAction` etc.
// ---------------------------------------------------------------------------

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct StepResultJs {
    applied_action: u32,
    score:          i32,
    depth:          u8,
    nodes:          u64,
    thought_ms:     u32,
    game_result:    u8,
}

#[wasm_bindgen]
impl StepResultJs {
    #[wasm_bindgen(getter, js_name = appliedAction)]
    pub fn applied_action(&self) -> u32 { self.applied_action }
    #[wasm_bindgen(getter)]
    pub fn score(&self) -> i32 { self.score }
    #[wasm_bindgen(getter)]
    pub fn depth(&self) -> u8 { self.depth }
    #[wasm_bindgen(getter)]
    pub fn nodes(&self) -> u64 { self.nodes }
    #[wasm_bindgen(getter, js_name = thoughtMs)]
    pub fn thought_ms(&self) -> u32 { self.thought_ms }
    #[wasm_bindgen(getter, js_name = gameResult)]
    pub fn game_result(&self) -> u8 { self.game_result }
}

impl From<api::StepResult> for StepResultJs {
    fn from(r: api::StepResult) -> Self {
        StepResultJs {
            applied_action: r.applied_action,
            score:          r.score,
            depth:          r.depth,
            nodes:          r.nodes,
            thought_ms:     r.thought_ms,
            game_result:    r.game_result,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct PhaseStateJs {
    to_move:           u8,
    current_phase:     u8,
    actions_remaining: u8,
    round_number:      u16,
    p1_money:          u16,
    p2_money:          u16,
    pending_modifiers: u8,
    game_result:       u8,
    zobrist:           u64,
}

#[wasm_bindgen]
impl PhaseStateJs {
    #[wasm_bindgen(getter, js_name = toMove)]
    pub fn to_move(&self) -> u8 { self.to_move }
    #[wasm_bindgen(getter, js_name = currentPhase)]
    pub fn current_phase(&self) -> u8 { self.current_phase }
    #[wasm_bindgen(getter, js_name = actionsRemaining)]
    pub fn actions_remaining(&self) -> u8 { self.actions_remaining }
    #[wasm_bindgen(getter, js_name = roundNumber)]
    pub fn round_number(&self) -> u16 { self.round_number }
    #[wasm_bindgen(getter, js_name = p1Money)]
    pub fn p1_money(&self) -> u16 { self.p1_money }
    #[wasm_bindgen(getter, js_name = p2Money)]
    pub fn p2_money(&self) -> u16 { self.p2_money }
    #[wasm_bindgen(getter, js_name = pendingModifiers)]
    pub fn pending_modifiers(&self) -> u8 { self.pending_modifiers }
    #[wasm_bindgen(getter, js_name = gameResult)]
    pub fn game_result(&self) -> u8 { self.game_result }
    #[wasm_bindgen(getter)]
    pub fn zobrist(&self) -> u64 { self.zobrist }
}

// ---------------------------------------------------------------------------
// Free-function helpers — not strictly needed but nice for diagnostics.
// ---------------------------------------------------------------------------

#[wasm_bindgen(js_name = engineVersion)]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
