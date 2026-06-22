//! tauri_wrapper — Tauri 2 desktop wrapper.
//!
//! Reuses core_engine natively (no WASM), exposing the same logical API to
//! the Svelte frontend via Tauri's `invoke` IPC. Native build can use
//! multi-threaded search via Rayon and a much larger transposition table.

use core_engine as _ce;

#[tauri::command]
fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![engine_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// TODO: matching IPC commands for new_match / legal_actions / apply_action /
// best_move / serialise — same logical API as wasm_wrapper.
