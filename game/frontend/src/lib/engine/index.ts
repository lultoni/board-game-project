// Runtime-detected engine client. Components import `engine` from here and
// never branch on backend.

import type { EngineClient } from "./types";

export type { EngineClient, PositionView, StepResult, FinalResultByte } from "./types";
export { ActionKind, decodeAction, actionKindName, encodeDraftTurn, decodeDraftTurn, isDraftTurn, DRAFT_TURN_TAG, ACTION_BG_CHOICE_TAG, MAX_BODYGUARD_ELIGIBLE, isBodyguardChoice, bgGuardIdx, encodeBodyguardChoice } from "./action";
export type { ActionDecoded, ActionKindValue, DraftTurnDecoded } from "./action";
export { decodeMailbox, readPieces, squareToFileRank, bitsOf, bitboardHas } from "./mailbox";
export type { MailboxEntry, BoardPiece, PieceKind, Owner } from "./mailbox";
export * from "./skills";

let cached: EngineClient | null = null;

function detectTauri(): boolean {
  if (typeof window === "undefined") return false;
  const w = window as unknown as {
    __TAURI__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return typeof w.__TAURI__ !== "undefined" || typeof w.__TAURI_INTERNALS__ !== "undefined";
}

/**
 * Returns the singleton EngineClient for this session. Lazily constructed on
 * first call so SSR-safe imports don't fire the Worker/IPC during build.
 */
export async function getEngine(): Promise<EngineClient> {
  if (cached) return cached;
  if (detectTauri()) {
    const { TauriClient } = await import("./tauri-client");
    cached = new TauriClient();
  } else {
    const { WasmClient } = await import("./wasm-client");
    cached = new WasmClient();
  }
  return cached;
}

/** For tests / hot-reload scenarios — drop the cached client. */
export function resetEngine(): void {
  cached?.dispose();
  cached = null;
}
