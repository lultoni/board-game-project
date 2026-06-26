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
  | (ReqBase & { kind: "createWithDraft"; configJson?: string })
  | (ReqBase & {
      kind: "createWithLoadouts";
      configJson?: string;
      p1Loadout: string;
      p2Loadout: string;
    })
  | (ReqBase & { kind: "draftState" })
  | (ReqBase & { kind: "positionView" })
  | (ReqBase & { kind: "legalActions" })
  | (ReqBase & { kind: "tryApply"; action: number })
  | (ReqBase & { kind: "stepAi" })
  | (ReqBase & { kind: "requestAiMoveForced" })
  | (ReqBase & { kind: "requestAiMoveAtDepth"; maxDepth: number })
  | (ReqBase & { kind: "positionFen" })
  | (ReqBase & { kind: "snapshotJson" })
  | (ReqBase & { kind: "restore"; json: string })
  | (ReqBase & { kind: "matchLogJson" })
  | (ReqBase & { kind: "latestPlyJson" })
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
  const pbg = e.pendingBodyguard();
  let pendingBodyguard: {
    attackerSrc: number; attackerNow: number; targetSq: number; eligible: number[];
  } | null = null;
  if (pbg) {
    // Copy the eligible view immediately — its underlying wasm buffer can be
    // invalidated by the next Engine call.
    const eligible = Array.from(new Uint8Array(pbg.eligible));
    pendingBodyguard = {
      attackerSrc: pbg.attackerSrc,
      attackerNow: pbg.attackerNow,
      targetSq: pbg.targetSq,
      eligible,
    };
  }
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
    pendingBodyguard,
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
      case "createWithDraft": {
        if (!msg.configJson) throw new Error("createWithDraft requires a configJson");
        engine = Engine.newWithDraft(msg.configJson, nowMs());
        self.postMessage({ id: msg.id, ok: true, value: null });
        break;
      }
      case "createWithLoadouts": {
        if (!msg.configJson) throw new Error("createWithLoadouts requires a configJson");
        engine = Engine.newWithLoadouts(msg.configJson, msg.p1Loadout, msg.p2Loadout, nowMs());
        self.postMessage({ id: msg.id, ok: true, value: null });
        break;
      }
      case "draftState": {
        const e = requireEngine();
        const s = e.draftState();
        // `usedSlotsFlat` aliases wasm memory — copy into JS-owned arrays
        // before posting back (postMessage structured-clones anyway, but we
        // also reshape into [12][2] booleans for the consumer).
        const flat = new Uint8Array(s.usedSlotsFlat);
        const usedSlots: boolean[][] = [];
        for (let p = 0; p < 12; p++) {
          usedSlots.push([flat[p * 2] !== 0, flat[p * 2 + 1] !== 0]);
        }
        self.postMessage({
          id: msg.id,
          ok: true,
          value: { turnNo: s.turnNo, sideToMove: s.sideToMove, usedSlots },
        });
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
      case "requestAiMoveForced": {
        const r = requireEngine().requestAiMoveForced();
        self.postMessage({ id: msg.id, ok: true, value: snapshotStepResult(r) });
        break;
      }
      case "requestAiMoveAtDepth": {
        const r = requireEngine().requestAiMoveAtDepth(msg.maxDepth);
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
      case "latestPlyJson": {
        const v = requireEngine().latestPlyJson();
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
