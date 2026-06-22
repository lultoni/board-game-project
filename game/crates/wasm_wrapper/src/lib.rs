//! wasm_wrapper — wasm-bindgen surface for the web build.
//!
//! Exposes a minimal API to JS/TS:
//!   - new_match(mode)            -> opaque handle
//!   - legal_actions(handle, sq)  -> Vec<u32>
//!   - apply_action(handle, a)    -> bool
//!   - best_move(handle, ms, d)   -> Option<u32>
//!   - serialise(handle)          -> String
//!
//! Runs in a Web Worker on the frontend side to keep the main thread
//! responsive during AI search.

use wasm_bindgen::prelude::*;
// Placeholder import — silences unused-crate warnings until the API surfaces are wired.
use core_engine as _ce;

#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// TODO: expose Match, legal_actions, apply_action, best_move, serialise.
