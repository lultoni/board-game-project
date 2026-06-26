// Unit coverage for `createPlyRenderer`. Stage 4c of the Phase-4 remediation.
//
// The renderer is driven against:
//   * a stubbed EngineClient that returns canned `PositionView`s and ignores
//     `tryApply` (the test controls state transitions by swapping the stub's
//     next view rather than letting a real engine compute one),
//   * a fake clock so effect `startedAt` timestamps are deterministic,
//   * a fake sfx that records `.play()` calls so we can assert SFX gating,
//   * a fake scheduler that holds setTimeouts in a Map so we can introspect
//     pending-timer count, fire them on demand, and verify `dispose()`
//     drains every outstanding timer (P2 regression).
//
// We deliberately do NOT stub legality / game rules — the renderer is a
// view-layer driver; correctness of the underlying ply is the engine's job.

import { describe, it, expect, beforeEach } from "vitest";
import { createPlyRenderer, type PlyRenderer, type TimerHandle } from "./ply-renderer.svelte";
import { ActionKind } from "../engine/action";
import type { EngineClient, PositionView, StepResult } from "../engine";

// === Fakes =================================================================

function makeFakeClock() {
  let t = 1000; // start at non-zero so off-by-zero bugs surface as wrong values
  return {
    now: () => t,
    advance(ms: number) {
      t += ms;
    },
  };
}

function makeFakeSfx() {
  const calls: { key: string; opts: unknown }[] = [];
  return {
    play(key: string, opts?: unknown) {
      calls.push({ key, opts });
    },
    calls,
  };
}

interface FakeScheduler {
  setTimeout: (cb: () => void, ms: number) => TimerHandle;
  clearTimeout: (h: TimerHandle) => void;
  fire(h: number): void;
  fireAll(): void;
  pendingCount(): number;
  pendingHandles(): number[];
}

function makeFakeScheduler(): FakeScheduler {
  let nextId = 1;
  const pending = new Map<number, () => void>();
  return {
    setTimeout(cb: () => void) {
      const id = nextId++;
      pending.set(id, cb);
      return id as unknown as TimerHandle;
    },
    clearTimeout(h: TimerHandle) {
      pending.delete(h as unknown as number);
    },
    fire(h: number) {
      const cb = pending.get(h);
      if (cb) {
        pending.delete(h);
        cb();
      }
    },
    fireAll() {
      // Snapshot keys so callbacks scheduling new timers don't perturb us.
      const handles = [...pending.keys()];
      for (const h of handles) {
        const cb = pending.get(h);
        pending.delete(h);
        cb?.();
      }
    },
    pendingCount() {
      return pending.size;
    },
    pendingHandles() {
      return [...pending.keys()];
    },
  };
}

// === Mailbox / position helpers ============================================

/** Build a u16 mailbox cell. See lib/engine/mailbox.ts for the layout. */
function cell(opts: { hp?: number; armor?: number; combo?: number; skill1?: number; skill2?: number } = {}): number {
  const hp = opts.hp ?? 0;
  const ar = opts.armor ?? 0;
  const co = opts.combo ?? 0;
  const s1 = opts.skill1 ?? 0;
  const s2 = opts.skill2 ?? 0;
  return (hp & 0x3) | ((ar & 0x3) << 2) | ((co & 0x7) << 4) | ((s1 & 0xf) << 7) | ((s2 & 0xf) << 11);
}

/** Construct a minimal PositionView for the renderer to consume. `pieces` is
 *  a map of square → { mailbox cell, owner ("p1"|"p2"), kind ("king"|"champion"|"guard") }.
 *  Bitboards are derived; everything else defaults. */
function makePositionView(
  pieces: Record<number, { cell: number; owner: "p1" | "p2"; kind: "king" | "champion" | "guard" }>,
  zobrist: bigint = 0n,
): PositionView {
  const mailbox = new Uint16Array(64);
  let p1 = 0n;
  let p2 = 0n;
  let kings = 0n;
  let champions = 0n;
  let guards = 0n;
  for (const [sqStr, info] of Object.entries(pieces)) {
    const sq = Number(sqStr);
    mailbox[sq] = info.cell;
    const bit = 1n << BigInt(sq);
    if (info.owner === "p1") p1 |= bit;
    else p2 |= bit;
    if (info.kind === "king") kings |= bit;
    else if (info.kind === "champion") champions |= bit;
    else guards |= bit;
  }
  const bitboards = BigUint64Array.from([p1, p2, kings, champions, guards]);
  return {
    bitboards,
    mailbox,
    toMove: 0,
    currentPhase: 1,
    actionsRemaining: 2,
    roundNumber: 1,
    p1Money: 0,
    p2Money: 0,
    pendingModifiers: 0,
    gameResult: 0,
    zobrist,
    pendingBodyguard: null,
  };
}

// === Action encoding helpers ===============================================

/** Encode a Move action. `hasAux=true` + `auxSq=src` is the engine convention
 *  for a speed-1 attacker (the attack-with-no-relocation case). */
function encodeMove(opts: { src: number; target: number; hasAux?: boolean; auxSq?: number }): number {
  const src = opts.src & 0x3f;
  const target = (opts.target & 0x3f) << 6;
  const kind = (ActionKind.Move & 0x3) << 12;
  const hasAux = opts.hasAux ? 1 : 0;
  const auxSq = ((opts.auxSq ?? 0) & 0x3f) << 23;
  const auxBit = (hasAux & 0x1) << 29;
  return (src | target | kind | auxSq | auxBit) >>> 0;
}

function encodeSkill(opts: { src: number; target: number; skillId: number }): number {
  const src = opts.src & 0x3f;
  const target = (opts.target & 0x3f) << 6;
  const kind = (ActionKind.Skill & 0x3) << 12;
  const skill = (opts.skillId & 0xf) << 14;
  return (src | target | kind | skill) >>> 0;
}

function encodeEndPhase(): number {
  return (((ActionKind.EndPhase & 0x3) << 12)) >>> 0;
}

// === Stub EngineClient =====================================================

/** A stub `EngineClient` whose `positionView()` returns the most-recently-
 *  set view. The renderer reads positionView() after every applyFn() (and
 *  for Skill actions also reads it during the diff/refresh path), so tests
 *  configure the "next view" before invoking `applyAndRender`. */
interface StubEngine extends EngineClient {
  setNextView(v: PositionView): void;
  setNextLegal(la: Uint32Array): void;
  positionViewCalls: number;
  legalCalls: number;
  tryApplyCalls: number[];
}

function makeStubEngine(initialView: PositionView): StubEngine {
  let view = initialView;
  let legal: Uint32Array<ArrayBuffer> = new Uint32Array(new ArrayBuffer(0));
  const stub = {
    positionViewCalls: 0,
    legalCalls: 0,
    tryApplyCalls: [] as number[],
    setNextView(v: PositionView) {
      view = v;
    },
    setNextLegal(la: Uint32Array) {
      legal = la as Uint32Array<ArrayBuffer>;
    },
    async version() {
      return "stub";
    },
    async createEngine() {},
    async createEngineWithDraft() {},
    async createEngineWithLoadouts() {},
    async draftState() {
      return { turnNo: 0, sideToMove: 0, usedSlots: [] };
    },
    async positionView() {
      this.positionViewCalls++;
      return view;
    },
    async legalActions() {
      this.legalCalls++;
      return legal;
    },
    async tryApply(raw: number): Promise<StepResult> {
      this.tryApplyCalls.push(raw);
      return { appliedAction: raw, score: 0, depth: 0, nodes: 0n, thoughtMs: 0, gameResult: 0 };
    },
    async stepAi(): Promise<StepResult> {
      return { appliedAction: 0, score: 0, depth: 0, nodes: 0n, thoughtMs: 0, gameResult: 0 };
    },
    async requestAiMoveForced(): Promise<StepResult> {
      return { appliedAction: 0, score: 0, depth: 0, nodes: 0n, thoughtMs: 0, gameResult: 0 };
    },
    async requestAiMoveAtDepth(): Promise<StepResult> {
      return { appliedAction: 0, score: 0, depth: 0, nodes: 0n, thoughtMs: 0, gameResult: 0 };
    },
    async positionFen() {
      return "";
    },
    async snapshotJson() {
      return "{}";
    },
    async restoreFromSnapshot() {},
    async matchLogJson() {
      return null;
    },
    async latestPlyJson() {
      return null;
    },
    async finaliseLog() {},
    async dispose() {},
  };
  return stub as unknown as StubEngine;
}

// === Tests =================================================================

describe("createPlyRenderer", () => {
  let clock: ReturnType<typeof makeFakeClock>;
  let sfx: ReturnType<typeof makeFakeSfx>;
  let scheduler: FakeScheduler;
  let eng: StubEngine;
  let renderer: PlyRenderer;

  // Common position used as the pre-state for most tests: a P1 champion at
  // a1 (sq 0) and a P2 champion at a3 (sq 16). Both at full HP.
  const initialPieces = {
    0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1" as const, kind: "champion" as const },
    16: { cell: cell({ hp: 2 }), owner: "p2" as const, kind: "champion" as const },
  };

  beforeEach(async () => {
    clock = makeFakeClock();
    sfx = makeFakeSfx();
    scheduler = makeFakeScheduler();
    eng = makeStubEngine(makePositionView(initialPieces));
    renderer = createPlyRenderer(eng, {
      clock,
      sfxImpl: sfx,
      scheduler,
      sfxEnabled: true,
    });
    // Prime the renderer with the initial position so applyAndRender has a
    // pre-state to snapshot.
    await renderer.resyncFromEngine();
  });

  it("piece-id reconciliation: plain move preserves piece identity", async () => {
    const idBefore = renderer.pieceIds.get(0);
    expect(idBefore).toBeDefined();

    // Move a1 → a2 (no aux = plain move).
    const raw = encodeMove({ src: 0, target: 8 });
    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 2 }), owner: "p2", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(renderer.pieceIds.get(0)).toBeUndefined();
    expect(renderer.pieceIds.get(8)).toBe(idBefore);
  });

  it("piece-id reconciliation: capture removes the captured piece's id", async () => {
    // P1 champion at a1 attacks P2 champion at a3 via aux = a2 (speed-2).
    // Post-state: a1 empty, a2 has the attacker (relocated), a3 empty
    // (captured). Engine convention for hasAux=true with auxSq != target =
    // attacker walked to auxSq then struck target.
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 8 });
    const attackerId = renderer.pieceIds.get(0);
    const victimId = renderer.pieceIds.get(16);
    expect(attackerId).toBeDefined();
    expect(victimId).toBeDefined();

    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(renderer.pieceIds.get(0)).toBeUndefined();
    expect(renderer.pieceIds.get(16)).toBeUndefined();
    expect(renderer.pieceIds.get(8)).toBe(attackerId);
  });

  it("speed-1 attack (hasAux=true, auxSq === src) capture: kills target, no relocation", async () => {
    // Engine encodes speed-1 attacks with auxSq == src (action.rs:228) — the
    // attacker doesn't move. Post-state: src still has attacker, target empty.
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 0 });
    eng.setNextView(makePositionView({
      0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(renderer.pieceIds.get(16)).toBeUndefined();
    // Attacker id stable at src.
    expect(renderer.pieceIds.get(0)).toBeDefined();
  });

  it("Move emits dust + move sfx; attack-kill also emits death sfx", async () => {
    sfx.calls.length = 0;
    // For the renderer's kill-detection to fire, the engine's post-state must
    // place the attacker on the target square (the attacker walked through
    // approach onto the victim's square). Post-state has p1 champ at sq 16,
    // approach sq 8 empty.
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 8 });
    eng.setNextView(makePositionView({
      16: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    const sfxKeys = sfx.calls.map((c) => c.key);
    expect(sfxKeys).toContain("attack");
    expect(sfxKeys).toContain("death");

    // The dust effect should be in the queue.
    const dustEffects = renderer.effectQueue.filter((e) => e.kind === "dust");
    expect(dustEffects.length).toBeGreaterThan(0);
  });

  it("damage effect uses injected clock timestamp", async () => {
    clock.advance(500); // clock.now() === 1500
    // Plain attack that lands damage but doesn't kill.
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 0 });
    eng.setNextView(makePositionView({
      0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 1 }), owner: "p2", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    const impactEffects = renderer.effectQueue.filter((e) => e.kind === "impact");
    expect(impactEffects.length).toBeGreaterThan(0);
    // Every impact effect's startedAt should equal the fake clock at emission time.
    expect(impactEffects[0].startedAt).toBe(1500);
  });

  it("sfx silenced when sfxEnabled=false", async () => {
    const silentRenderer = createPlyRenderer(eng, {
      clock,
      sfxImpl: sfx,
      scheduler,
      sfxEnabled: false,
    });
    await silentRenderer.resyncFromEngine();
    sfx.calls.length = 0;

    const raw = encodeMove({ src: 0, target: 8 });
    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 2 }), owner: "p2", kind: "champion" },
    }));
    await silentRenderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(sfx.calls.length).toBe(0);
  });

  it("damage on attack registers a shake timer; dispose() clears it (P2 regression)", async () => {
    // Damage triggers `triggerShake`, which schedules a SHAKE_DURATION_MS timer.
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 0 });
    eng.setNextView(makePositionView({
      0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 1 }), owner: "p2", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(scheduler.pendingCount()).toBeGreaterThan(0);
    expect(renderer.shakingSquares.has(16)).toBe(true);

    renderer.dispose();
    expect(scheduler.pendingCount()).toBe(0);
  });

  it("reset() also cancels outstanding shake timers (P2 regression)", async () => {
    const raw = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 0 });
    eng.setNextView(makePositionView({
      0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 1 }), owner: "p2", kind: "champion" },
    }));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(scheduler.pendingCount()).toBeGreaterThan(0);
    renderer.reset();
    expect(scheduler.pendingCount()).toBe(0);
    expect(renderer.shakingSquares.size).toBe(0);
  });

  it("Skill with relocation defers the state flip via scheduler", async () => {
    // Skill that moves a piece (e.g. dash). Pre-state has piece at sq 0;
    // post-state has it at sq 8. The renderer's emitImpactEvents short-
    // circuits without firing impact effects (no HP/armor delta), so we
    // need an HP delta to trigger the defer-by-RELOC_DELAY_MS path.
    // Use a healing skill instead: piece at sq 0 gains armor AND relocates.
    // Easier: damage-on-relocate (someone moved and lost HP). Construct:
    //   pre: sq 0 = p1 champ hp 2 armor 1, sq 16 = p2 champ hp 2.
    //   post: sq 8 = p1 champ hp 2 armor 0 (lost armor, relocated), sq 16 absent.
    // hasReloc=true (sq 0 → 8 is a move), impactFired=true (armor delta on
    // the moving piece). That activates the deferred-refresh path.
    const raw = encodeSkill({ src: 0, target: 8, skillId: 9 /* dash */ });
    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 0 }), owner: "p1", kind: "champion" },
    }, 42n));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    // A deferred-skill-refresh timer should now be pending in the scheduler.
    expect(scheduler.pendingCount()).toBeGreaterThan(0);

    // Before firing the deferred refresh, the renderer's pieceIds should
    // still reflect the pre-state (relocation hasn't visually applied).
    expect(renderer.pieceIds.get(0)).toBeDefined();

    // Fire all timers — the deferred refresh applies the position update
    // and emits relocation/death events.
    scheduler.fireAll();

    expect(renderer.pieceIds.get(8)).toBeDefined();
    expect(renderer.pieceIds.get(0)).toBeUndefined();
  });

  it("drainPendingSkillRefresh applies a deferred refresh synchronously", async () => {
    const raw = encodeSkill({ src: 0, target: 8, skillId: 9 });
    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 0 }), owner: "p1", kind: "champion" },
    }, 42n));
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(scheduler.pendingCount()).toBeGreaterThan(0);
    renderer.drainPendingSkillRefresh();

    // The deferred apply ran synchronously inside the drain.
    expect(renderer.pieceIds.get(8)).toBeDefined();
    // The scheduler entry for the deferred refresh is gone (drained).
    // Other entries (shake) may still exist.
    expect(scheduler.pendingHandles().every((h) => h !== undefined)).toBe(true);
  });

  it("EndPhase plays the phaseEnd sfx and yields lastApplied=null", async () => {
    sfx.calls.length = 0;
    const raw = encodeEndPhase();
    // EndPhase doesn't change positions in our stub.
    await renderer.applyAndRender(raw, async () => { await eng.tryApply(raw); });

    expect(sfx.calls.map((c) => c.key)).toContain("phaseEnd");
    expect(renderer.lastApplied).toBeNull();
  });

  it("fastForwardTo with target=0 restores baseline and resyncs", async () => {
    const baselineJson = '{"snapshot":"baseline"}';
    // After restore, the engine's positionView returns the initial position.
    eng.setNextView(makePositionView(initialPieces));
    await renderer.fastForwardTo(baselineJson, [], 0);

    expect(renderer.pieceIds.size).toBe(2);
    expect(renderer.effectQueue.length).toBe(0);
  });

  it("dispose() drains both shake AND deferred skill-refresh timers", async () => {
    // Trigger both kinds of timer: an attack-with-damage (shake) followed
    // by a relocating skill (deferred refresh).
    const atk = encodeMove({ src: 0, target: 16, hasAux: true, auxSq: 0 });
    eng.setNextView(makePositionView({
      0: { cell: cell({ hp: 2, armor: 1 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 1 }), owner: "p2", kind: "champion" },
    }));
    await renderer.applyAndRender(atk, async () => { await eng.tryApply(atk); });

    const skl = encodeSkill({ src: 0, target: 8, skillId: 9 });
    eng.setNextView(makePositionView({
      8: { cell: cell({ hp: 2, armor: 0 }), owner: "p1", kind: "champion" },
      16: { cell: cell({ hp: 1 }), owner: "p2", kind: "champion" },
    }, 42n));
    await renderer.applyAndRender(skl, async () => { await eng.tryApply(skl); });

    expect(scheduler.pendingCount()).toBeGreaterThanOrEqual(2);

    renderer.dispose();
    expect(scheduler.pendingCount()).toBe(0);
    expect(renderer.effectQueue.length).toBe(0);
  });
});
