// Engine bridge — abstracts the difference between the WASM (web) and Tauri
// (desktop) backends. Components only ever talk to this module.
//
// Per ADR-005: same Svelte code ships to both targets. This file is the only
// place that knows which runtime it's in.

const isTauri = typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__ !== "undefined";

export interface EngineApi {
  version(): Promise<string>;
  // TODO: legalActions, applyAction, bestMove, serialise, ...
}

class WebEngine implements EngineApi {
  async version(): Promise<string> {
    // TODO: import the WASM module built from `wasm_wrapper`.
    return "wasm: not wired yet";
  }
}

class DesktopEngine implements EngineApi {
  async version(): Promise<string> {
    // TODO: import @tauri-apps/api/core and invoke("engine_version").
    return "tauri: not wired yet";
  }
}

export const engine: EngineApi = isTauri ? new DesktopEngine() : new WebEngine();
