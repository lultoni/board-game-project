// Match-level reactive state. Populated by routes/match/+page.svelte and the
// engine bridge. This slice creates the skeleton; later slices fill in
// selection/legal/effects.

import type { PositionView } from "../engine";
import type { EngineClient } from "../engine";
import type { EndReason, MatchMode } from "../storage";
import { buildEngineConfigJson as buildEngineConfigJsonPure, type SeatTag } from "../engine/config";
import { settings } from "./settings.svelte";
import { createTelemetrySession } from "./telemetry-session";

const telemetry = createTelemetrySession();

export type SeatKind = "human" | "ai";
/** Re-exported from `storage/types` so route code can keep importing it from
 *  this module. The canonical home is `storage/types.ts` — the engine and
 *  storage layers must not import from state. */
export type { MatchMode } from "../storage";
/** L8 — which draft flow the user picked at /setup/. `custom` runs the
 *  full /draft/ route (12 alternating picks). `preMade` skips the draft and
 *  /match/ opens with both sides preloaded from a curated `SideLoadout`. */
export type DraftMode = "custom" | "preMade";
/** L8 — identifier for the chosen pre-made loadout. The catalogue lives in
 *  `state/draft.ts` (`PRE_MADE_LOADOUTS`). Designer picks one of these on
 *  the setup screen; both sides play the same loadout (mirror match). */
export type PreMadeLoadoutId = "firstGame" | "secondGame" | "thirdGame";

export interface MatchState {
  mode: MatchMode;
  side: { p1: SeatKind; p2: SeatKind };
  /** L8 — which draft flow to enter on `/draft/`. Set by `/setup/`. */
  draftMode: DraftMode;
  /** L8 — only consulted when `draftMode === "preMade"`. `/match/` reads
   *  this to call `createEngineWithLoadouts`, then clears it. */
  preMadeLoadoutId: PreMadeLoadoutId | null;
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
  /** Which board seat (P1=0, P2=1) this peer occupies for the LIFETIME of the
   *  current multiplayer match. Set once on session origin (host start → 0,
   *  joiner connect → 1) and **never changes on takeover** — the new-host who
   *  was originally joiner stays at seat 1; the displaced-host who rejoins
   *  as joiner stays at seat 0. Drives the "am I P1?" UI mapping so identity
   *  survives leader handoff. Null outside multiplayer. */
  localSeat: 0 | 1 | null;
  /** The 6-digit session code for the active multiplayer session, kept on
   *  the carrier so /match/ can pass it to telemetry's startMatch opts. */
  multiplayerCode: string | null;
  /** Idempotency flag for telemetry finalisation. Set once a natural game-end
   *  or a claim-win has persisted the telemetry row; consulted to avoid
   *  double-finalise across reactive re-runs, claim-win double-clicks, and
   *  the beforeunload guard. Reset whenever a new match begins. */
  telemetryFinalised: boolean;
}

export const match = $state<MatchState>({
  mode: "idle",
  side: { p1: "human", p2: "human" },
  draftMode: "custom",
  preMadeLoadoutId: null,
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
  localSeat: null,
  telemetryFinalised: false,
});

export function resetMatchState(): void {
  match.mode = "idle";
  // Preserve `side` across resets — set by setup, consumed by draft & match.
  // `draftMode` and `preMadeLoadoutId` reset to their defaults so that direct
  // navigation to /match/ without going through /setup/ doesn't inherit stale
  // mode picks from a previous match. `/setup/` re-writes both on commit.
  match.draftMode = "custom";
  match.preMadeLoadoutId = null;
  match.position = null;
  match.legal = new Uint32Array();
  match.selection = null;
  match.lastApplied = null;
  match.pendingSnapshotJson = null;
  match.trueSnapshotJson = null;
  match.sandboxMovesApplied = 0;
  match.telemetryMatchId = null;
  match.telemetryFinalised = false;
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

function seatTag(s: SeatKind): SeatTag {
  return s === "ai" ? "Ai" : "Human";
}

/** State-aware wrapper around the pure `engine/config.ts` builder. Pulls AI
 *  budgets and AIvAI delay from `settings` so callers don't have to thread
 *  six fields through every site. The pure builder is the one that goes to
 *  the engine — this is just the adapter that reads runes. */
export function buildEngineConfigJson(side: { p1: SeatKind; p2: SeatKind }): string {
  return buildEngineConfigJsonPure({
    p1: seatTag(side.p1),
    p2: seatTag(side.p2),
    p1Ai: { timeLimitMs: settings.p1ThinkTimeMs, maxDepth: settings.p1MaxDepth },
    p2Ai: { timeLimitMs: settings.p2ThinkTimeMs, maxDepth: settings.p2MaxDepth },
    aivaiStepDelayMs: settings.aivaiStepDelayMs,
  });
}

// === Telemetry session lifecycle (bound to `match`) ========================
// Routes call these without passing the carrier. The pure helpers live in
// telemetry-session.ts and are tested there.

/** Internal handle on the app-wide telemetry session instance. Exposed for
 *  cross-module wiring that needs the carrier-ful API (e.g. the multiplayer
 *  handoff orchestrator, which passes a stub carrier in tests). Routes should
 *  use the carrier-less wrappers below instead. */
export const _telemetrySession = telemetry;

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

export function networkLostTelemetrySession(eng?: EngineClient): Promise<void> {
  return telemetry.networkLostTelemetrySession(match, eng);
}

/** Sync-entry wrappers for the `pagehide` event path. The browser may discard
 *  the IDB write on tab-close; we accept the loss. Callers must NOT await. */
export function abandonTelemetrySessionSync(): void {
  telemetry.abandonTelemetrySessionSync(match);
}

export function networkLostTelemetrySessionSync(): void {
  telemetry.networkLostTelemetrySessionSync(match);
}

/** Multiplayer claim-win: the present player declares victory after the grace
 *  window expires (peer never came back). Finalises the engine log and the
 *  telemetry row with endReason "opponent_forfeit" and the result favouring
 *  the present player.
 *
 *  Result is keyed off `localSeat` (which is stable across leader handoff),
 *  not `multiplayerRole` — a joiner-promoted new-host still occupies seat 1,
 *  so their claim still means "P2 wins" (resultByte 1). No-op outside
 *  multiplayer.
 */
export async function claimWinByOpponentForfeit(eng: EngineClient): Promise<void> {
  if (match.mode !== "multiplayer" || !match.multiplayerRole) return;
  if (match.telemetryFinalised) return;
  const seat = match.localSeat ?? (match.multiplayerRole === "host" ? 0 : 1);
  const resultByte: 0 | 1 = seat === 0 ? 0 : 1;
  try {
    await eng.finaliseLog(resultByte);
  } catch {
    // If the engine refuses (already finalised) we still want to persist the
    // telemetry verdict — fall through.
  }
  await telemetry.finalizeTelemetrySession(match, eng, "opponent_forfeit", resultByte);
  match.telemetryFinalised = true;
  // Refresh the live position so the game-end UI fires.
  try {
    match.position = await eng.positionView();
  } catch { /* noop */ }
}
