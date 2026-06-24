// WasmClient — runs the engine in a dedicated Web Worker, exposes a
// promise-based proxy. See engine/worker.ts for the worker side.

import type {
  EngineClient,
  FinalResultByte,
  PositionView,
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
      // Reject every outstanding request; the worker is wedged.
      for (const r of this.#pending.values()) {
        r.reject(new Error(ev.message ?? "worker error"));
      }
      this.#pending.clear();
    };
  }

  #call<T>(req: object): Promise<T> {
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
  requestAiMove(): Promise<StepResult> {
    return this.#call<StepResult>({ kind: "requestAiMove" });
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
    this.#worker.terminate();
    this.#pending.clear();
  }
}
