// Worker entry — owns the wasm Engine. The main thread talks to it via a
// promise-keyed postMessage protocol; see wasm-client.ts.
//
// Imports the wasm-pack output directly from `crates/wasm_wrapper/pkg/`.
// Vite's `server.fs.allow: ['..']` permits cross-package resolution in dev;
// in production the bundler inlines the chunk.

import init, {
  Engine,
  engineVersion,
} from "../../../../crates/wasm_wrapper/pkg/wasm_wrapper.js";

import type { FinalResultByte } from "./types";

interface ReqBase {
  id: number;
}
type Req =
  | (ReqBase & { kind: "version" })
  | (ReqBase & { kind: "create"; configJson?: string })
  | (ReqBase & { kind: "positionView" })
  | (ReqBase & { kind: "legalActions" })
  | (ReqBase & { kind: "tryApply"; action: number })
  | (ReqBase & { kind: "stepAi" })
  | (ReqBase & { kind: "positionFen" })
  | (ReqBase & { kind: "snapshotJson" })
  | (ReqBase & { kind: "restore"; json: string })
  | (ReqBase & { kind: "matchLogJson" })
  | (ReqBase & { kind: "finaliseLog"; result: FinalResultByte });

let engine: Engine | null = null;
let initialised = false;

async function ensureInit(): Promise<void> {
  if (initialised) return;
  await init();
  initialised = true;
}

function nowMs(): number {
  return Date.now();
}

function requireEngine(): Engine {
  if (!engine) throw new Error("engine not created — call createEngine first");
  return engine;
}

function snapshotPositionView() {
  const e = requireEngine();
  // Zero-copy views alias wasm linear memory — copy each before the next call.
  const bb = new BigUint64Array(e.positionBitboards());
  const mb = new Uint16Array(e.positionMailbox());
  const p = e.phaseState();
  return {
    bitboards: bb,
    mailbox: mb,
    toMove: p.toMove,
    currentPhase: p.currentPhase,
    actionsRemaining: p.actionsRemaining,
    roundNumber: p.roundNumber,
    p1Money: p.p1Money,
    p2Money: p.p2Money,
    pendingModifiers: p.pendingModifiers,
    gameResult: p.gameResult,
    zobrist: p.zobrist,
  };
}

function snapshotStepResult(r: {
  appliedAction: number;
  score: number;
  depth: number;
  nodes: bigint;
  thoughtMs: number;
  gameResult: number;
}) {
  return {
    appliedAction: r.appliedAction,
    score: r.score,
    depth: r.depth,
    nodes: r.nodes,
    thoughtMs: r.thoughtMs,
    gameResult: r.gameResult,
  };
}

self.onmessage = async (ev: MessageEvent<Req>) => {
  const msg = ev.data;
  try {
    await ensureInit();
    switch (msg.kind) {
      case "version": {
        const out = engineVersion();
        self.postMessage({ id: msg.id, ok: true, value: out });
        break;
      }
      case "create": {
        engine = msg.configJson
          ? Engine.newWithConfigJson(msg.configJson, nowMs())
          : new Engine(nowMs());
        self.postMessage({ id: msg.id, ok: true, value: null });
        break;
      }
      case "positionView": {
        self.postMessage({ id: msg.id, ok: true, value: snapshotPositionView() });
        break;
      }
      case "legalActions": {
        const view = requireEngine().legalActions();
        const out = new Uint32Array(view); // copy
        self.postMessage({ id: msg.id, ok: true, value: out });
        break;
      }
      case "tryApply": {
        const r = requireEngine().tryApply(msg.action >>> 0, nowMs());
        self.postMessage({ id: msg.id, ok: true, value: snapshotStepResult(r) });
        break;
      }
      case "stepAi": {
        const r = requireEngine().stepAi(nowMs());
        self.postMessage({ id: msg.id, ok: true, value: snapshotStepResult(r) });
        break;
      }
      case "positionFen": {
        self.postMessage({ id: msg.id, ok: true, value: requireEngine().positionFen() });
        break;
      }
      case "snapshotJson": {
        self.postMessage({ id: msg.id, ok: true, value: requireEngine().snapshotJson() });
        break;
      }
      case "restore": {
        engine = Engine.fromSnapshotJson(msg.json, nowMs());
        self.postMessage({ id: msg.id, ok: true, value: null });
        break;
      }
      case "matchLogJson": {
        const v = requireEngine().matchLogJson();
        self.postMessage({ id: msg.id, ok: true, value: v ?? null });
        break;
      }
      case "finaliseLog": {
        requireEngine().finaliseLog(nowMs(), msg.result);
        self.postMessage({ id: msg.id, ok: true, value: null });
        break;
      }
    }
  } catch (e) {
    self.postMessage({
      id: msg.id,
      ok: false,
      error: String((e as Error)?.message ?? e),
    });
  }
};
