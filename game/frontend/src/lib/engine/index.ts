// Runtime-detected engine client. Components import `engine` from here and
// never branch on backend.

import type { EngineClient } from "./types";

export type {
  EngineClient,
  PositionView,
  PendingBodyguardView,
  StepResult,
  FinalResultByte,
  DraftStateView,
  SideLoadout,
} from "./types";
export { ActionKind, decodeAction, actionKindName, encodeDraftTurn, decodeDraftTurn, isDraftTurn, DRAFT_TURN_TAG, ACTION_BG_CHOICE_TAG, MAX_BODYGUARD_ELIGIBLE, isBodyguardChoice, bgGuardIdx, encodeBodyguardChoice } from "./action";
export type { ActionDecoded, ActionKindValue, DraftTurnDecoded } from "./action";
export { decodeMailbox, readPieces, squareToFileRank, bitsOf, bitboardHas } from "./mailbox";
export type { MailboxEntry, BoardPiece, PieceKind, Owner } from "./mailbox";
export * from "./skills";
export { formatAction, formatSquare } from "./action-label";
export {
  validateSnapshot,
  validateMatchLog,
  SnapshotValidationError,
  SNAPSHOT_BUDGETS,
} from "./snapshot-validator";
export type {
  SnapshotSource,
  SnapshotValidationReason,
  SnapshotValidationOpts,
  ValidatedSnapshot,
} from "./snapshot-validator";
export { runAiCall, AiCallError } from "./ai-hooks";
export type { AiCallOpts, AiCallReason } from "./ai-hooks";

let cached: EngineClient | null = null;

function detectTauri(): boolean {
  if (typeof window === "undefined") return false;
  const w = window as unknown as {
    isTauri?: boolean;
    __TAURI__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return (
    w.isTauri === true ||
    typeof w.__TAURI_INTERNALS__ !== "undefined" ||
    typeof w.__TAURI__ !== "undefined"
  );
}

/**
 * Returns the singleton EngineClient for this session. Lazily constructed on
 * first call so SSR-safe imports don't fire the Worker/IPC during build.
 *
 * For the WASM backend, subscribes to `onDead` so a wedged worker (uncaught
 * exception inside the engine, OOM, etc.) invalidates the cache automatically
 * — the next caller gets a fresh worker rather than a permanently-pending
 * promise. Callers that hold a stale reference will see their next `#call`
 * reject with "engine worker is dead"; they should fall back to `getEngine()`
 * and re-issue the request (typically through `resetEngine()` for state
 * cleanup, then restore from snapshot).
 */
export async function getEngine(): Promise<EngineClient> {
  if (cached) return cached;
  if (detectTauri()) {
    const { TauriClient } = await import("./tauri-client");
    cached = new TauriClient();
  } else {
    const { WasmClient } = await import("./wasm-client");
    const client = new WasmClient();
    client.onDead(() => {
      // Invalidate the cache so the next getEngine() re-spawns. We don't
      // call dispose() here — the worker is already dead, and the client
      // self-rejects pending calls in #markDead.
      if (cached === client) cached = null;
    });
    cached = client;
  }
  return cached;
}

/** For tests / hot-reload scenarios — drop the cached client. */
export function resetEngine(): void {
  cached?.dispose();
  cached = null;
}
