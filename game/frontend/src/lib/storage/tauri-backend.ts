// Tauri filesystem backend for telemetry — desktop build only.
//
// Status (S33): placeholder. The Tauri FS plugin isn't yet wired in the
// tauri_wrapper crate, and L5a's scope is the web path. When the desktop
// build is exercised, this is the right place to add Tauri FS reads/writes
// under $APPDATA/board-game/matches/{matchId}/{plies.jsonl,match.json}.
//
// The IDB backend works fine inside Tauri's webview (it uses the underlying
// browser engine), so until this is implemented, desktop users will also
// see telemetry via IDB. The dedicated FS backend is for when we want
// matches to survive a `webview-data-cleared` event or live in a
// human-inspectable location.

import type {
  EndReason,
  FinalisedMatch,
  JoinedCodeEntry,
  MatchFilter,
  MatchMeta,
  PlyEntry,
  TelemetryStore,
} from "./types";

export class TauriTelemetryStore implements TelemetryStore {
  async startMatch(): Promise<string> {
    throw new Error("TauriTelemetryStore not implemented — use IdbTelemetryStore");
  }
  async appendPly(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async finalizeMatch(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async markAbandoned(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async markNetworkLost(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async checkpointMatchLog(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async dismissNetworkLost(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async getMatchMeta(): Promise<MatchMeta | null> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async getMatch(): Promise<FinalisedMatch | null> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async getPlies(): Promise<PlyEntry[]> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async listMatches(_filter?: MatchFilter): Promise<MatchMeta[]> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async deleteMatch(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async bundleMatches(): Promise<{ bundle: string; skipped: string[] }> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async recordJoinedCode(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async listJoinedCodes(): Promise<JoinedCodeEntry[]> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async forgetJoinedCode(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  async updateMultiplayerRole(): Promise<void> {
    throw new Error("TauriTelemetryStore not implemented");
  }
  // Silence unused warnings on the placeholder fields.
  _endReason?: EndReason;
}
