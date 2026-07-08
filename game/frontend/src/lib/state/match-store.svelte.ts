// Match-level reactive state. Populated by routes/match/+page.svelte and the
// engine bridge. This slice creates the skeleton; later slices fill in
// selection/legal/effects.

import type { PositionView } from "../engine";
import type { EngineClient } from "../engine";
import type { EndReason, MatchMode } from "../storage";
import { buildEngineConfigJson as buildEngineConfigJsonPure, type SeatTag } from "../engine/config";
import { mpState } from "../multiplayer.svelte";
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

/** L8/Task 8 — a resolved reference to whichever loadout a side is playing
 *  in a `preMade`-flow local match. Either a hard-coded pre-made id, or a
 *  ULID pointing at a saved row in IDB's `loadouts` store. `/match/`
 *  resolves this to a concrete `SideLoadout` at engine-boot time via
 *  `resolveLoadout()`. Multiplayer still uses only `{ kind: "preMade" }` on
 *  both sides (shared picker; fairness constraint). */
export type LoadoutRef =
  | { kind: "preMade"; id: PreMadeLoadoutId }
  | { kind: "custom";  id: string };

export interface MatchState {
  mode: MatchMode;
  side: { p1: SeatKind; p2: SeatKind };
  /** L8 — which draft flow to enter on `/draft/`. Set by `/setup/`. */
  draftMode: DraftMode;
  /** Task 8 — per-side loadout picks for `draftMode === "preMade"` local
   *  matches. `/match/` reads this to call `createEngineWithLoadouts`
   *  after resolving each side (pre-made table lookup or IDB row fetch),
   *  then clears it. `null` means no pre-made flow was selected.
   *
   *  Multiplayer invariant: when set in MP mode, both sides MUST reference
   *  the same pre-made id. The wire protocol only ships a single
   *  `preMadeId` and the joiner mirrors it onto both slots.
   *
   *  This replaces the singular `preMadeLoadoutId` that existed before
   *  Task 8. Left un-versioned because match state is session-scoped —
   *  nothing persists it across restarts. */
  sideLoadouts: { p1: LoadoutRef; p2: LoadoutRef } | null;
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
  /** Mode the match was in immediately before sandbox was entered. Restored
   *  by `exitSandbox()`. Necessary because `modeFromSeats()` cannot recover
   *  `"multiplayer"` from the seat pair (both seats are `"human"` in MP too).
   *  Null when not in / not returning from sandbox. */
  preSandboxMode: MatchMode | null;
  /** Active telemetry session ID (ULID) — null when not logging (sandbox,
   *  inspector, or before startTelemetrySession is called). */
  telemetryMatchId: string | null;
  /** Which board seat (P1=0, P2=1) this peer occupies for the LIFETIME of the
   *  current multiplayer match. Set once on session origin (host start → 0,
   *  joiner connect → 1) and **never changes on takeover** — the new-host who
   *  was originally joiner stays at seat 1; the displaced-host who rejoins
   *  as joiner stays at seat 0. Drives the "am I P1?" UI mapping so identity
   *  survives leader handoff. Null outside multiplayer. */
  localSeat: 0 | 1 | null;
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
  sideLoadouts: null,
  position: null,
  legal: new Uint32Array(),
  selection: null,
  lastApplied: null,
  pendingSnapshotJson: null,
  trueSnapshotJson: null,
  sandboxMovesApplied: 0,
  preSandboxMode: null,
  telemetryMatchId: null,
  localSeat: null,
  telemetryFinalised: false,
});

/** Single-source-of-truth reactive accessors for multiplayer role + code.
 *  Both originate in `mpState` (the transport's view) and are read elsewhere
 *  through these functions. Svelte 5 forbids exporting `$derived` from
 *  modules, so these are getter functions — call them at the read site
 *  (`multiplayerRole()`). The read still tracks `mpState` reactively because
 *  the access happens inside the caller's reactive scope. Assignment is
 *  impossible (no setter exported), which is the structural guarantee
 *  against re-introducing parallel state. */
export function multiplayerRole(): "host" | "joiner" | null {
  return mpState.role;
}
export function multiplayerCode(): string | null {
  return mpState.code;
}

export function resetMatchState(): void {
  match.mode = "idle";
  // Preserve `side` across resets — set by setup, consumed by draft & match.
  // `draftMode` and `sideLoadouts` reset to their defaults so that direct
  // navigation to /match/ without going through /setup/ doesn't inherit stale
  // mode picks from a previous match. `/setup/` re-writes both on commit.
  match.draftMode = "custom";
  match.sideLoadouts = null;
  match.position = null;
  match.legal = new Uint32Array();
  match.selection = null;
  match.lastApplied = null;
  match.pendingSnapshotJson = null;
  match.trueSnapshotJson = null;
  match.sandboxMovesApplied = 0;
  match.preSandboxMode = null;
  match.telemetryMatchId = null;
  match.telemetryFinalised = false;
  // MP role/code now live in `mpState` (single source) and are exposed here
  // as the module-level `$derived` constants `multiplayerRole` /
  // `multiplayerCode`. We don't touch them here — the lobby owns MP
  // teardown via `mpDisconnect()`.
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

/** Apply the per-seat evaluator choice from `settings` to the engine. Routes
 *  call this once after every `createEngine*` / `restoreFromSnapshot` so the
 *  AI seats use the picked rater (heuristic / run / blessed). No-op on WASM
 *  (the client stubs `setAiEvaluator`). Tauri-side, errors are swallowed
 *  here: if a rater id has gone stale, falling back to heuristic is far
 *  better than failing match boot. */
export async function applyEvaluatorSettings(eng: EngineClient): Promise<void> {
  const p1 = settings.p1Evaluator;
  const p2 = settings.p2Evaluator;
  try {
    await eng.setAiEvaluator(p1.source, p1.id ?? null, null);
  } catch { /* fall back to heuristic */ }
  try {
    await eng.setAiEvaluator(p2.source, p2.id ?? null, null);
  } catch { /* fall back to heuristic */ }
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

/** Pure decision: which side wins on an opponent forfeit. Keyed off
 *  `localSeat` (stable across leader handoff) — a joiner-promoted new-host
 *  still occupies seat 1, so their claim still means "P2 wins" (resultByte 1).
 *  Falls back to mapping role→seat only when localSeat is null (pre-MP-boot). */
export function computeClaimResultByte(
  localSeat: 0 | 1 | null,
  role: "host" | "joiner" | null,
): 0 | 1 {
  const seat = localSeat ?? (role === "host" ? 0 : 1);
  if (localSeat === null) {
    console.warn(`[mp] seat fallback used at match-store:246 (localSeat=null, role=${role}) → seat=${seat}`);
  }
  return seat === 0 ? 0 : 1;
}

/** Persistence-only half of the opponent-forfeit flow: write the telemetry
 *  row and latch the idempotency flag. Does NOT call `eng.finaliseLog` or
 *  refresh `match.position` — the caller (route or thin orchestrator) owns
 *  those two side effects. Re-entry safe via the `telemetryFinalised` early
 *  return. */
export async function finaliseOpponentForfeit(
  eng: EngineClient,
  resultByte: 0 | 1,
): Promise<void> {
  if (match.telemetryFinalised) return;
  await telemetry.finalizeTelemetrySession(match, eng, "opponent_forfeit", resultByte);
  match.telemetryFinalised = true;
}

/** Multiplayer claim-win: the present player declares victory after the grace
 *  window expires (peer never came back). Thin orchestrator that composes
 *  the pure resultByte decision, the engine log finalisation, the telemetry
 *  persistence, and the position refresh needed to fire the game-end UI.
 *  No-op outside multiplayer.
 */
export async function claimWinByOpponentForfeit(eng: EngineClient): Promise<void> {
  const role = multiplayerRole();
  if (match.mode !== "multiplayer" || !role) return;
  if (match.telemetryFinalised) return;
  const resultByte = computeClaimResultByte(match.localSeat, role);
  try {
    await eng.finaliseLog(resultByte);
  } catch {
    // If the engine refuses (already finalised) we still want to persist the
    // telemetry verdict — fall through.
  }
  await finaliseOpponentForfeit(eng, resultByte);
  // Refresh the live position so the game-end UI fires.
  try {
    match.position = await eng.positionView();
  } catch { /* noop */ }
}
