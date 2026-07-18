// TauriClient - invokes commands on the native engine via Tauri v2 IPC.
// Holds a u64 handle returned by `create_engine`.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DraftStateView,
  EngineClient,
  EvalBreakdown,
  EvalBreakdownBySquare,
  FinalResultByte,
  PositionView,
  SideLoadout,
  StepResult,
} from "./types";

export interface PositionViewDto {
  bitboards: string[] | number[]; // serde encodes u64 as JS number/string depending on size
  mailbox: number[];
  toMove: number;
  currentPhase: number;
  actionsRemaining: number;
  roundNumber: number;
  p1Money: number;
  p2Money: number;
  pendingModifiers: number;
  gameResult: number;
  zobrist: number | string;
  /** Optional: the Tauri backend may not yet project pending_bodyguard. When
   *  absent we surface `null`, matching the engine's common case. */
  pendingBodyguard?: {
    attackerSrc: number;
    attackerNow: number;
    targetSq: number;
    eligible: number[];
  } | null;
}

interface StepResultDto {
  appliedAction: number;
  score: number;
  depth: number;
  nodes: number | string;
  thoughtMs: number;
  gameResult: number;
}

interface DraftStateDto {
  turnNo: number;
  sideToMove: number;
  usedSlots: boolean[][];
}

function toBigInt(v: number | string | bigint): bigint {
  if (typeof v === "bigint") return v;
  if (typeof v === "number") return BigInt(v);
  return BigInt(v);
}

export function normalisePositionView(dto: PositionViewDto): PositionView {
  const bb = new BigUint64Array(5);
  for (let i = 0; i < 5; i++) {
    bb[i] = toBigInt(dto.bitboards[i] as number | string);
  }
  const mb = Uint16Array.from(dto.mailbox);
  return {
    bitboards: bb,
    mailbox: mb,
    toMove: dto.toMove,
    currentPhase: dto.currentPhase,
    actionsRemaining: dto.actionsRemaining,
    roundNumber: dto.roundNumber,
    p1Money: dto.p1Money,
    p2Money: dto.p2Money,
    pendingModifiers: dto.pendingModifiers,
    gameResult: dto.gameResult,
    zobrist: toBigInt(dto.zobrist),
    pendingBodyguard: dto.pendingBodyguard ?? null,
  };
}

function normaliseStepResult(dto: StepResultDto): StepResult {
  return {
    appliedAction: dto.appliedAction,
    score: dto.score,
    depth: dto.depth,
    nodes: toBigInt(dto.nodes),
    thoughtMs: dto.thoughtMs,
    gameResult: dto.gameResult,
  };
}

export class TauriClient implements EngineClient {
  #handle = 0;

  /** Guard for every method that operates on an existing engine. A zero
   *  handle means `dispose()` ran or the engine was never created - calling
   *  into the Rust `EngineRegistry` with `0` panics in debug builds and
   *  returns the wrong engine in release builds (registry lookup keyed on
   *  u64). Throwing synchronously here gives callers a recoverable error
   *  rather than IPC nonsense. */
  #requireHandle(): number {
    if (this.#handle === 0) {
      throw new Error("engine not initialized");
    }
    return this.#handle;
  }

  // Replace the active engine handle, dropping any prior one. Every
  // `createEngine*` / `restoreFromSnapshot` call routes through this so
  // re-entering a route (e.g. /draft/ → /match/ → back to /setup/ → /draft/)
  // doesn't leak Rust-side `Match` records into the `EngineRegistry`. The
  // WASM client doesn't need this - its worker holds a single engine that
  // `createEngine*` overwrites in place.
  async #replaceHandle(newHandle: number): Promise<void> {
    const prev = this.#handle;
    this.#handle = newHandle;
    if (prev !== 0 && prev !== newHandle) {
      await invoke<boolean>("drop_engine", { handle: prev });
    }
  }

  async version(): Promise<string> {
    return await invoke<string>("engine_version");
  }

  async createEngine(configJson?: string): Promise<void> {
    const h = await invoke<number>("create_engine", { configJson: configJson ?? null });
    await this.#replaceHandle(h);
  }

  async createEngineWithDraft(configJson?: string): Promise<void> {
    const h = await invoke<number>("create_engine_with_draft", {
      configJson: configJson ?? null,
    });
    await this.#replaceHandle(h);
  }

  async createEngineWithLoadouts(
    configJson: string | undefined,
    p1Loadout: SideLoadout,
    p2Loadout: SideLoadout,
  ): Promise<void> {
    const h = await invoke<number>("create_engine_with_loadouts", {
      configJson:     configJson ?? null,
      p1LoadoutJson:  JSON.stringify(p1Loadout),
      p2LoadoutJson:  JSON.stringify(p2Loadout),
    });
    await this.#replaceHandle(h);
  }

  async draftState(): Promise<DraftStateView> {
    const dto = await invoke<DraftStateDto>("draft_state", { handle: this.#requireHandle() });
    return {
      turnNo:     dto.turnNo,
      sideToMove: dto.sideToMove,
      usedSlots:  dto.usedSlots,
    };
  }

  async positionView(): Promise<PositionView> {
    const dto = await invoke<PositionViewDto>("position_view", { handle: this.#requireHandle() });
    return normalisePositionView(dto);
  }

  async legalActions(): Promise<Uint32Array> {
    const arr = await invoke<number[]>("legal_actions", { handle: this.#requireHandle() });
    return Uint32Array.from(arr);
  }

  async tryApply(action: number): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("try_apply", {
      handle: this.#requireHandle(),
      rawAction: action >>> 0,
    });
    return normaliseStepResult(dto);
  }

  async stepAi(onDepth?: (depth: number, score: number) => void): Promise<StepResult> {
    let unlisten: (() => void) | null = null;
    if (onDepth) {
      try {
        unlisten = await listen<{ depth: number; score: number }>("ai-depth-update", (ev) => {
          onDepth(ev.payload.depth, ev.payload.score);
        });
      } catch (err) {
        console.warn("ai-depth-update listen failed; continuing without depth streaming:", err);
      }
    }
    try {
      const dto = await invoke<StepResultDto>("step_ai", { handle: this.#requireHandle() });
      return normaliseStepResult(dto);
    } finally {
      unlisten?.();
    }
  }

  async requestAiMoveForced(): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("request_ai_move_forced", { handle: this.#requireHandle() });
    return normaliseStepResult(dto);
  }

  async heuristicEval(): Promise<EvalBreakdown> {
    return await invoke<EvalBreakdown>("heuristic_eval", { handle: this.#requireHandle() });
  }

  async heuristicEvalBySquare(): Promise<EvalBreakdownBySquare> {
    return await invoke<EvalBreakdownBySquare>("heuristic_eval_by_square", { handle: this.#requireHandle() });
  }

  async requestAiMoveAtDepth(maxDepth: number): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("request_ai_move_at_depth", {
      handle: this.#requireHandle(),
      maxDepth,
    });
    return normaliseStepResult(dto);
  }

  async positionFen(): Promise<string> {
    return await invoke<string>("position_fen", { handle: this.#requireHandle() });
  }

  async snapshotJson(): Promise<string> {
    return await invoke<string>("snapshot_json", { handle: this.#requireHandle() });
  }

  async restoreFromSnapshot(json: string): Promise<void> {
    const h = await invoke<number>("create_engine_from_snapshot", { snapshotJson: json });
    await this.#replaceHandle(h);
  }

  async matchLogJson(): Promise<string | null> {
    return await invoke<string | null>("match_log_json", { handle: this.#requireHandle() });
  }

  async latestPlyJson(): Promise<string | null> {
    return await invoke<string | null>("latest_ply_json", { handle: this.#requireHandle() });
  }

  async finaliseLog(result: FinalResultByte): Promise<void> {
    await invoke<void>("finalise_log", { handle: this.#requireHandle(), resultByte: result });
  }

  async dispose(): Promise<void> {
    if (this.#handle === 0) return;
    await invoke<boolean>("drop_engine", { handle: this.#handle });
    this.#handle = 0;
  }

  async setAiEvaluator(
    source: "heuristic" | "run" | "blessed",
    id?: string | null,
    runDir?: string | null,
  ): Promise<void> {
    await invoke<void>("set_ai_evaluator", {
      handle: this.#requireHandle(),
      source,
      id: id ?? null,
      runDir: runDir ?? null,
    });
  }
}
