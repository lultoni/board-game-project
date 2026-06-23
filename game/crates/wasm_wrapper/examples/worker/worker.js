// Web Worker harness for wasm_wrapper.
//
// Contract:
//   main → worker:
//     { type: 'init',  configJson?: string }
//     { type: 'apply', action: number }       // raw u32 Action bits
//     { type: 'stepAi' }
//     { type: 'legal' }
//     { type: 'snapshot' }
//     { type: 'restore', snapshotJson: string }
//
//   worker → main:
//     { type: 'ready', version: string }
//     { type: 'state', bitboards: BigUint64Array, mailbox: Uint16Array, phase: {...} }
//     { type: 'legal', actions: Uint32Array }
//     { type: 'step',  result: {...} }
//     { type: 'snapshot', json: string }
//     { type: 'error', message: string }
//
// IMPORTANT: zero-copy typed arrays from Engine alias wasm linear memory and
// are invalidated by the next Engine call. We COPY before posting back, since
// postMessage on a non-transferred TypedArray serialises (i.e. copies) anyway.

import init, { Engine, engineVersion } from '../../pkg/wasm_wrapper.js';

let engine = null;

function postState() {
  // Copy each view before the next Engine call invalidates it.
  const bb       = new BigUint64Array(engine.positionBitboards());
  const mailbox  = new Uint16Array(engine.positionMailbox());
  const p        = engine.phaseState();
  const phase = {
    toMove:           p.toMove,
    currentPhase:     p.currentPhase,
    actionsRemaining: p.actionsRemaining,
    roundNumber:      p.roundNumber,
    p1Money:          p.p1Money,
    p2Money:          p.p2Money,
    pendingModifiers: p.pendingModifiers,
    gameResult:       p.gameResult,
    zobrist:          p.zobrist,
  };
  self.postMessage({ type: 'state', bitboards: bb, mailbox, phase });
}

self.onmessage = async (ev) => {
  const msg = ev.data;
  try {
    switch (msg.type) {
      case 'init': {
        await init();
        const now = Date.now();
        engine = msg.configJson
          ? Engine.newWithConfigJson(msg.configJson, now)
          : new Engine(now);
        self.postMessage({ type: 'ready', version: engineVersion() });
        postState();
        break;
      }

      case 'apply': {
        const r = engine.tryApply(msg.action >>> 0, Date.now());
        self.postMessage({
          type: 'step',
          result: {
            appliedAction: r.appliedAction,
            score:         r.score,
            depth:         r.depth,
            nodes:         Number(r.nodes),
            thoughtMs:     r.thoughtMs,
            gameResult:    r.gameResult,
          },
        });
        postState();
        break;
      }

      case 'stepAi': {
        const r = engine.stepAi(Date.now());
        self.postMessage({
          type: 'step',
          result: {
            appliedAction: r.appliedAction,
            score:         r.score,
            depth:         r.depth,
            nodes:         Number(r.nodes),
            thoughtMs:     r.thoughtMs,
            gameResult:    r.gameResult,
          },
        });
        postState();
        break;
      }

      case 'legal': {
        const view = engine.legalActions();
        const actions = new Uint32Array(view); // copy out
        self.postMessage({ type: 'legal', actions });
        break;
      }

      case 'snapshot': {
        const json = engine.snapshotJson();
        self.postMessage({ type: 'snapshot', json });
        break;
      }

      case 'restore': {
        engine = Engine.fromSnapshotJson(msg.snapshotJson, Date.now());
        postState();
        break;
      }

      default:
        throw new Error(`unknown message type: ${msg.type}`);
    }
  } catch (e) {
    self.postMessage({ type: 'error', message: String(e?.message ?? e) });
  }
};
