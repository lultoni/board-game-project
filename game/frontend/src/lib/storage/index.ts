// Telemetry store boot — resolves to IDB (both dev and Tauri desktop).
// See tauri-backend.ts header for why the Tauri FS backend is stubbed.

import type { TelemetryStore } from "./types";
import { IdbTelemetryStore } from "./idb-backend";

let store: TelemetryStore | null = null;

/** Returns the singleton telemetry store. Throws if no storage backend is
 *  available in this environment (e.g. tests running without an IDB
 *  polyfill). Caller should catch and degrade gracefully — telemetry must
 *  never block gameplay. */
export function getTelemetryStore(): TelemetryStore {
  if (store) return store;
  // Future: branch on `import.meta.env.TAURI_PLATFORM` once the Tauri FS
  // backend is implemented. For now IDB serves both web and Tauri webview.
  store = new IdbTelemetryStore();
  return store;
}

/** Test-only: inject a custom store (e.g. backed by fake-indexeddb). */
export function _setTelemetryStoreForTest(s: TelemetryStore | null): void {
  store = s;
}

export type { TelemetryStore } from "./types";
export {
  type EndReason,
  type FinalisedMatch,
  type JoinedCodeEntry,
  type MatchFilter,
  type MatchMeta,
  type MatchMode,
  type MatchStatus,
  type PlyEntry,
  type SavedLoadout,
  newMatchId,
} from "./types";
