// Engine client - always TauriClient (WASM path removed; the game ships as a
// Tauri desktop app only; WASM cannot run the CUDA training backend).

import type { EngineClient } from "./types";

export type {
  EngineClient,
  PositionView,
  PendingBodyguardView,
  StepResult,
  FinalResultByte,
  DraftStateView,
  SideLoadout,
  EvalBreakdown,
  SquareBreakdown,
  EvalBreakdownBySquare,
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

export async function getEngine(): Promise<EngineClient> {
  if (cached) return cached;
  const { TauriClient } = await import("./tauri-client");
  cached = new TauriClient();
  return cached;
}

/** For tests / hot-reload scenarios - drop the cached client. */
export function resetEngine(): void {
  cached?.dispose();
  cached = null;
}
