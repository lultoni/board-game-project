// WasmClient — runs the engine in a dedicated Web Worker, exposes a
// promise-based proxy. See engine/worker.ts for the worker side.

import type {
  DraftStateView,
  EngineClient,
  FinalResultByte,
  PositionView,
  SideLoadout,
  StepResult,
} from "./types";

type Resolver = {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
};

export class WasmClient implements EngineClient {
  #worker: Worker;
  #pending = new Map<number, Resolver>();
  #seq = 0;
  /** Set to true when the worker fires `onerror` or `dispose()` is called.
   *  Once dead, every subsequent `#call` rejects synchronously rather than
   *  posting a message that will never be answered — callers can re-create
   *  the engine via `resetEngine()` + `getEngine()` instead of hanging on a
   *  pending promise. */
  #dead = false;
  /** Optional listener invoked once the worker is observed dead. Used by
   *  `engine/index.ts` to invalidate its cached client so the next
   *  `getEngine()` re-spawns a fresh worker. Set via `onDead()`. */
  #onDead: (() => void) | null = null;

  constructor() {
    this.#worker = new Worker(new URL("./worker.ts", import.meta.url), {
      type: "module",
      name: "engine-wasm",
    });
    this.#worker.onmessage = (ev: MessageEvent) => {
      const { id, ok, value, error } = ev.data;
      const r = this.#pending.get(id);
      if (!r) return;
      this.#pending.delete(id);
      if (ok) r.resolve(value);
      else r.reject(new Error(error ?? "worker error"));
    };
    this.#worker.onerror = (ev) => {
      this.#markDead(ev.message ?? "worker error");
    };
  }

  /** Trip the dead flag, reject every outstanding request, and notify the
   *  module-level cache so the next `getEngine()` re-spawns. Idempotent. */
  #markDead(message: string): void {
    if (this.#dead) return;
    this.#dead = true;
    for (const r of this.#pending.values()) {
      r.reject(new Error(message));
    }
    this.#pending.clear();
    this.#onDead?.();
  }

  /** Subscribe to the one-shot dead notification. Replaces any prior
   *  listener — only the engine cache needs this hook. */
  onDead(cb: () => void): void {
    this.#onDead = cb;
    if (this.#dead) cb();
  }

  #call<T>(req: object): Promise<T> {
    if (this.#dead) {
      return Promise.reject(new Error("engine worker is dead"));
    }
    const id = ++this.#seq;
    return new Promise<T>((resolve, reject) => {
      this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.#worker.postMessage({ id, ...req });
    });
  }

  version(): Promise<string> {
    return this.#call<string>({ kind: "version" });
  }
  createEngine(configJson?: string): Promise<void> {
    return this.#call<void>({ kind: "create", configJson });
  }
  createEngineWithDraft(configJson?: string): Promise<void> {
    return this.#call<void>({ kind: "createWithDraft", configJson });
  }
  createEngineWithLoadouts(
    configJson: string | undefined,
    p1Loadout: SideLoadout,
    p2Loadout: SideLoadout,
  ): Promise<void> {
    return this.#call<void>({
      kind: "createWithLoadouts",
      configJson,
      p1Loadout: JSON.stringify(p1Loadout),
      p2Loadout: JSON.stringify(p2Loadout),
    });
  }
  draftState(): Promise<DraftStateView> {
    return this.#call<DraftStateView>({ kind: "draftState" });
  }
  positionView(): Promise<PositionView> {
    return this.#call<PositionView>({ kind: "positionView" });
  }
  legalActions(): Promise<Uint32Array> {
    return this.#call<Uint32Array>({ kind: "legalActions" });
  }
  tryApply(action: number): Promise<StepResult> {
    return this.#call<StepResult>({ kind: "tryApply", action });
  }
  stepAi(): Promise<StepResult> {
    return this.#call<StepResult>({ kind: "stepAi" });
  }
  requestAiMoveForced(): Promise<StepResult> {
    return this.#call<StepResult>({ kind: "requestAiMoveForced" });
  }
  requestAiMoveAtDepth(maxDepth: number): Promise<StepResult> {
    return this.#call<StepResult>({ kind: "requestAiMoveAtDepth", maxDepth });
  }
  positionFen(): Promise<string> {
    return this.#call<string>({ kind: "positionFen" });
  }
  snapshotJson(): Promise<string> {
    return this.#call<string>({ kind: "snapshotJson" });
  }
  restoreFromSnapshot(json: string): Promise<void> {
    return this.#call<void>({ kind: "restore", json });
  }
  matchLogJson(): Promise<string | null> {
    return this.#call<string | null>({ kind: "matchLogJson" });
  }
  latestPlyJson(): Promise<string | null> {
    return this.#call<string | null>({ kind: "latestPlyJson" });
  }
  finaliseLog(result: FinalResultByte): Promise<void> {
    return this.#call<void>({ kind: "finaliseLog", result });
  }
  async dispose(): Promise<void> {
    this.#markDead("engine worker disposed");
    this.#worker.terminate();
  }
}
