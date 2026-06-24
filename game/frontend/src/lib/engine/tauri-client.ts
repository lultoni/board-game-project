// TauriClient — invokes commands on the native engine via Tauri v2 IPC.
// Holds a u64 handle returned by `create_engine`.

import { invoke } from "@tauri-apps/api/core";
import type {
  DraftStateView,
  EngineClient,
  FinalResultByte,
  PositionView,
  SideLoadout,
  StepResult,
} from "./types";

interface PositionViewDto {
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

function normalisePositionView(dto: PositionViewDto): PositionView {
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

  async version(): Promise<string> {
    return await invoke<string>("engine_version");
  }

  async createEngine(configJson?: string): Promise<void> {
    this.#handle = await invoke<number>("create_engine", { configJson: configJson ?? null });
  }

  async createEngineWithDraft(configJson?: string): Promise<void> {
    this.#handle = await invoke<number>("create_engine_with_draft", {
      configJson: configJson ?? null,
    });
  }

  async createEngineWithLoadouts(
    configJson: string | undefined,
    p1Loadout: SideLoadout,
    p2Loadout: SideLoadout,
  ): Promise<void> {
    this.#handle = await invoke<number>("create_engine_with_loadouts", {
      configJson:     configJson ?? null,
      p1LoadoutJson:  JSON.stringify(p1Loadout),
      p2LoadoutJson:  JSON.stringify(p2Loadout),
    });
  }

  async draftState(): Promise<DraftStateView> {
    const dto = await invoke<DraftStateDto>("draft_state", { handle: this.#handle });
    return {
      turnNo:     dto.turnNo,
      sideToMove: dto.sideToMove,
      usedSlots:  dto.usedSlots,
    };
  }

  async positionView(): Promise<PositionView> {
    const dto = await invoke<PositionViewDto>("position_view", { handle: this.#handle });
    return normalisePositionView(dto);
  }

  async legalActions(): Promise<Uint32Array> {
    const arr = await invoke<number[]>("legal_actions", { handle: this.#handle });
    return Uint32Array.from(arr);
  }

  async tryApply(action: number): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("try_apply", {
      handle: this.#handle,
      rawAction: action >>> 0,
    });
    return normaliseStepResult(dto);
  }

  async stepAi(): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("step_ai", { handle: this.#handle });
    return normaliseStepResult(dto);
  }

  async requestAiMove(): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("request_ai_move", { handle: this.#handle });
    return normaliseStepResult(dto);
  }

  async requestAiMoveForced(): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("request_ai_move_forced", { handle: this.#handle });
    return normaliseStepResult(dto);
  }

  async requestAiMoveAtDepth(maxDepth: number): Promise<StepResult> {
    const dto = await invoke<StepResultDto>("request_ai_move_at_depth", {
      handle: this.#handle,
      maxDepth,
    });
    return normaliseStepResult(dto);
  }

  async positionFen(): Promise<string> {
    return await invoke<string>("position_fen", { handle: this.#handle });
  }

  async snapshotJson(): Promise<string> {
    return await invoke<string>("snapshot_json", { handle: this.#handle });
  }

  async restoreFromSnapshot(json: string): Promise<void> {
    this.#handle = await invoke<number>("create_engine_from_snapshot", { snapshotJson: json });
  }

  async matchLogJson(): Promise<string | null> {
    return await invoke<string | null>("match_log_json", { handle: this.#handle });
  }

  async latestPlyJson(): Promise<string | null> {
    return await invoke<string | null>("latest_ply_json", { handle: this.#handle });
  }

  async finaliseLog(result: FinalResultByte): Promise<void> {
    await invoke<void>("finalise_log", { handle: this.#handle, resultByte: result });
  }

  async dispose(): Promise<void> {
    if (this.#handle === 0) return;
    await invoke<boolean>("drop_engine", { handle: this.#handle });
    this.#handle = 0;
  }
}
