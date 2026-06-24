// Match-level reactive state. Populated by routes/match/+page.svelte and the
// engine bridge. This slice creates the skeleton; later slices fill in
// selection/legal/effects.

import type { PositionView } from "../engine/types";
import type { EngineClient } from "../engine/types";
import type { EndReason } from "../storage";
import * as telemetry from "./telemetry-session";

export type SeatKind = "human" | "ai";
export type MatchMode = "idle" | "hvh" | "hvai" | "aivai" | "replay" | "sandbox" | "multiplayer";

export interface MatchState {
  mode: MatchMode;
  side: { p1: SeatKind; p2: SeatKind };
  position: PositionView | null;
  legal: Uint32Array;
  /** Square (0..63) currently selected by the human, if any. */
  selection: number | null;
  /** The raw action just applied, for "what just happened" effects. */
  lastApplied: number | null;
  /**
   * Pre-built engine snapshot stashed by the draft route. When set, the
   * match route restores from this snapshot instead of creating a fresh
   * engine. Cleared after consumption.
   */
  pendingSnapshotJson: string | null;
  /**
   * Snapshot of the live position taken at sandbox-mode entry. While
   * `mode === "sandbox"`, all engine state changes are tentative; on exit
   * the engine is restored from this snapshot and the field is cleared.
   */
  trueSnapshotJson: string | null;
  /** Count of player/AI applications since sandbox-mode was entered. */
  sandboxMovesApplied: number;
  /** Active telemetry session ID (ULID) — null when not logging (sandbox,
   *  inspector, or before startTelemetrySession is called). */
  telemetryMatchId: string | null;
  /** Which seat we hold in a multiplayer session. Host plays P1, joiner P2.
   *  Null outside multiplayer. Used by /match/ to decide whether input on a
   *  given seat should be accepted locally or ignored as the peer's. */
  multiplayerRole: "host" | "joiner" | null;
  /** The 6-digit session code for the active multiplayer session, kept on
   *  the carrier so /match/ can pass it to telemetry's startMatch opts. */
  multiplayerCode: string | null;
}

export const match = $state<MatchState>({
  mode: "idle",
  side: { p1: "human", p2: "human" },
  position: null,
  legal: new Uint32Array(),
  selection: null,
  lastApplied: null,
  pendingSnapshotJson: null,
  trueSnapshotJson: null,
  sandboxMovesApplied: 0,
  telemetryMatchId: null,
  multiplayerRole: null,
  multiplayerCode: null,
});

export function resetMatchState(): void {
  match.mode = "idle";
  // Preserve `side` across resets — it's set by the setup screen before
  // entering draft, and we don't want draft's reset to wipe it.
  match.position = null;
  match.legal = new Uint32Array();
  match.selection = null;
  match.lastApplied = null;
  match.pendingSnapshotJson = null;
  match.trueSnapshotJson = null;
  match.sandboxMovesApplied = 0;
  match.telemetryMatchId = null;
  // multiplayerRole and multiplayerCode are owned by the lobby; routes
  // downstream (setup/draft/match) only read them. The lobby is responsible
  // for clearing them on session teardown.
}

/** Derive the user-facing mode label from the two seat assignments. */
export function modeFromSeats(side: { p1: SeatKind; p2: SeatKind }): MatchMode {
  if (side.p1 === "human" && side.p2 === "human") return "hvh";
  if (side.p1 === "ai"    && side.p2 === "ai")    return "aivai";
  return "hvai";
}

// === Telemetry session lifecycle (bound to `match`) ========================
// Routes call these without passing the carrier. The pure helpers live in
// telemetry-session.ts and are tested there.

export function startTelemetrySession(
  mode: MatchMode,
  opts: { multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null } = {},
): Promise<string | null> {
  return telemetry.startTelemetrySession(match, mode, opts);
}

export function recordPly(eng: EngineClient): Promise<void> {
  return telemetry.recordPly(match, eng);
}

export function finalizeTelemetrySession(
  eng: EngineClient,
  endReason: EndReason,
  resultByte: 0 | 1 | 2 | 3,
): Promise<void> {
  return telemetry.finalizeTelemetrySession(match, eng, endReason, resultByte);
}

export function abandonTelemetrySession(eng?: EngineClient): Promise<void> {
  return telemetry.abandonTelemetrySession(match, eng);
}
