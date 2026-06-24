// Unified TS types for the engine-bridge boundary. Both WasmClient and
// TauriClient normalise to these shapes so components never branch on backend.

export interface PositionView {
  /** [p1, p2, kings, champions, guards] as 5 × u64. */
  bitboards: BigUint64Array;
  /** 64 entries, u16 packed (see decodeMailbox). */
  mailbox: Uint16Array;
  toMove: number;
  currentPhase: number;
  actionsRemaining: number;
  roundNumber: number;
  p1Money: number;
  p2Money: number;
  pendingModifiers: number;
  gameResult: number;
  zobrist: bigint;
}

export interface StepResult {
  appliedAction: number;
  score: number;
  depth: number;
  nodes: bigint;
  thoughtMs: number;
  gameResult: number;
}

export type FinalResultByte = 0 | 1 | 2 | 3; // P1Win | P2Win | Draw | Aborted

export interface EngineClient {
  version(): Promise<string>;
  createEngine(configJson?: string): Promise<void>;
  positionView(): Promise<PositionView>;
  legalActions(): Promise<Uint32Array>;
  tryApply(action: number): Promise<StepResult>;
  stepAi(): Promise<StepResult>;
  /** Run the AI search without applying. `appliedAction` carries the best
   *  candidate move (0 if none was found). For inspector / hint UIs. */
  requestAiMove(): Promise<StepResult>;
  /** Inspector variant: runs the search regardless of seat kind so HvH
   *  positions can also ask "what would the AI play here?". */
  requestAiMoveForced(): Promise<StepResult>;
  /** Inspector iterative-deepening: runs ID up to `maxDepth` with no time
   *  bound. Caller drives the deepening loop by stepping `maxDepth` up
   *  by 1 each call and polling cancellation between calls. */
  requestAiMoveAtDepth(maxDepth: number): Promise<StepResult>;
  positionFen(): Promise<string>;
  snapshotJson(): Promise<string>;
  restoreFromSnapshot(json: string): Promise<void>;
  matchLogJson(): Promise<string | null>;
  /** Latest `PlyRecord` JSON (the newest entry in the match log). `null` when
   *  `auto_log` is off or no plies recorded yet. Used by the telemetry
   *  persistence layer to write per-ply incrementally without re-serialising
   *  the entire log. */
  latestPlyJson(): Promise<string | null>;
  finaliseLog(result: FinalResultByte): Promise<void>;
  /** Free engine resources (Tauri only; no-op on WASM). */
  dispose(): Promise<void>;
}
