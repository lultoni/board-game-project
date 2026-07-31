// Integration test for the sandbox ↔ multiplayer-wrapper contract (ns-37).
//
// The pure wrapper unit tests (`multiplayer-engine.test.ts`) stub the
// `ensureLiveEngine` hook. This file wires the wrapper to a route-shaped
// carrier + fake engine that mirrors what `routes/match/+page.svelte` does:
//   - `enterSandbox` captures a true-line snapshot, flips `match.mode`
//     to "sandbox", and forks the shared engine locally.
//   - `ensureLiveEngineOnTrueLine` (the wire-path hook) restores the true
//     line and flips `match.mode` back BEFORE the wrapper validates an
//     incoming opponent action.
//
// Regression target: an opponent move arriving DURING local sandbox must NOT
// trip the anti-cheat audit against the sandbox-forked state. It must
// auto-exit sandbox and apply cleanly on the true line, with seq advanced
// (never rolled back to a stale pre-move snapshot).

import { describe, it, expect } from "vitest";
import { createMpEngine } from "./multiplayer-engine";
import type { WireMessageV2, WirePhase } from "./multiplayer-protocol-v2";
import type { EngineClient, EvalBreakdown, GameConstantsWire, PositionView, SkillMetadataWire, StepResult } from "./engine";

// A fake engine that models the ONE shared handle. `applied` is the move
// history; snapshot/restore round-trip it as JSON so a sandbox fork can be
// rolled back exactly the way the real engine's snapshotJson/restoreFromSnapshot
// does. Zobrist is deterministic from `applied` (same formula as the unit-test
// fake) so the host's claimed postZobrist can be reproduced.
class SharedFakeEngine implements EngineClient {
  applied: number[] = [];
  currentPhase = 0; // play

  async version(): Promise<string> { return "fake"; }
  async createEngine(): Promise<void> {}
  async createEngineWithDraft(): Promise<void> {}
  async createEngineWithLoadouts(): Promise<void> {}
  async draftState() { return { turnNo: 0, sideToMove: 0, usedSlots: [] }; }
  async positionView(): Promise<PositionView> {
    return {
      bitboards: new BigUint64Array(5),
      mailbox: new Uint16Array(64),
      toMove: 0,
      currentPhase: this.currentPhase,
      actionsRemaining: 0,
      roundNumber: 0,
      p1Money: 0,
      p2Money: 0,
      pendingModifiers: 0,
      gameResult: 0,
      zobrist: this.computeZobrist(),
      pendingBodyguard: null,
      movedThisPhase: 0n,
    };
  }
  async legalActions(): Promise<Uint32Array> { return new Uint32Array(); }
  async actionToNotation(_raw: number): Promise<string> { return ""; }
  async skillMetadata(): Promise<SkillMetadataWire[]> { return []; }
  async gameConstants(): Promise<GameConstantsWire> {
    return {
      phaseMove: 0, phaseSkill: 1, phaseDraft: 2,
      modifierFocus: 1, modifierCharge: 2, modifierMoveAttackUsed: 4,
      playerP1: 0, playerP2: 1,
      gameOngoing: 0, gameP1Wins: 1, gameP2Wins: 2, skillCount: 15,
    };
  }
  async tryApply(raw: number): Promise<StepResult> {
    this.applied.push(raw);
    return { appliedAction: raw, score: 0, depth: 0, nodes: 0n, thoughtMs: 0, gameResult: 0 };
  }
  async stepAi(): Promise<StepResult> { throw new Error("not used"); }
  async requestAiMoveForced(): Promise<StepResult> { throw new Error("not used"); }
  async requestAiMoveAtDepth(): Promise<StepResult> { throw new Error("not used"); }
  async heuristicEval(): Promise<EvalBreakdown> {
    return {
      material_p1: 0, material_p2: 0, hp_p1: 0, hp_p2: 0, armor_p1: 0, armor_p2: 0,
      skills_p1: 0, skills_p2: 0, money_p1: 0, money_p2: 0, mobility_p1: 0, mobility_p2: 0,
      threat_p1: 0, threat_p2: 0, skill_act_p1: 0, skill_act_p2: 0,
      offensive_range_p1: 0, offensive_range_p2: 0, total: 0,
    };
  }
  async heuristicEvalBySquare(): Promise<never> { throw new Error("not used"); }
  async positionFen(): Promise<string> { return ""; }
  // Snapshot carries a valid envelope AND the applied history so restore is exact.
  async snapshotJson(): Promise<string> {
    return JSON.stringify({ start_fen: "start", config: {}, actions: this.applied });
  }
  async restoreFromSnapshot(json: string): Promise<void> {
    const obj = JSON.parse(json) as { actions?: number[] };
    this.applied = Array.isArray(obj.actions) ? [...obj.actions] : [];
  }
  async matchLogJson(): Promise<string | null> { return null; }
  async latestPlyJson(): Promise<string | null> { return null; }
  async onBackgroundEvalReady(): Promise<() => void> { return () => { /* no-op */ }; }
  async startAivaiProducer(): Promise<void> { /* no-op */ }
  async aivaiProducerLog(): Promise<string | null> { return null; }
  async stopAivaiProducer(): Promise<string | null> { return null; }
  async onAivaiProgress(): Promise<() => void> { return () => { /* no-op */ }; }
  async finaliseLog(): Promise<void> {}
  async evaluateDrawOffer(): Promise<boolean> { return true; }
  async dispose(): Promise<void> {}
  async setAiEvaluator(): Promise<void> {}

  private computeZobrist(): bigint {
    let h = 1n;
    for (const r of this.applied) h = h * 31n + BigInt(r);
    return h;
  }
}

// Minimal route-shaped carrier - the fields the sandbox flow touches.
interface RouteCarrier {
  mode: "multiplayer" | "sandbox";
  trueSnapshotJson: string | null;
  preSandboxMode: "multiplayer" | "sandbox" | null;
  sandboxMovesApplied: number;
}

describe("sandbox ↔ MP integration (ns-37)", () => {
  // Reconstruct the route's wiring around the REAL wrapper.
  function setup(role: "host" | "joiner") {
    const eng = new SharedFakeEngine();
    const sent: WireMessageV2[] = [];
    const handlers = new Set<(m: WireMessageV2) => void>();
    const cheats: Array<{ side: string }> = [];
    const applied: number[] = [];

    const match: RouteCarrier = {
      mode: "multiplayer",
      trueSnapshotJson: null,
      preSandboxMode: null,
      sandboxMovesApplied: 0,
    };

    // Mirror of routes/match/+page.svelte enterSandbox().
    async function enterSandbox() {
      const snap = await eng.snapshotJson();
      match.trueSnapshotJson = snap;
      match.sandboxMovesApplied = 0;
      match.preSandboxMode = match.mode;
      match.mode = "sandbox";
    }
    // Mirror of applyRaw()'s sandbox branch: fork the shared engine locally.
    async function sandboxApply(raw: number) {
      await eng.tryApply(raw);
      match.sandboxMovesApplied += 1;
    }
    // Mirror of restoreTrueLineFromSandbox() + ensureLiveEngineOnTrueLine().
    async function ensureLiveEngineOnTrueLine() {
      if (match.mode !== "sandbox" || !match.trueSnapshotJson) return;
      await eng.restoreFromSnapshot(match.trueSnapshotJson);
      match.trueSnapshotJson = null;
      match.sandboxMovesApplied = 0;
      match.mode = match.preSandboxMode ?? "multiplayer";
      match.preSandboxMode = null;
    }

    const handle = createMpEngine(
      { phase: "play", matchId: role === "host" ? "m1" : null, nonceFactory: () => "n1", warn: () => {} },
      {
        eng,
        getRole: () => role,
        getCode: () => "123456",
        ensureLiveEngine: ensureLiveEngineOnTrueLine,
        send: (m) => sent.push(m),
        subscribe: (cb) => { handlers.add(cb); return () => handlers.delete(cb); },
        onApplied: (raw) => { applied.push(raw); },
        onSnapshotApplied: () => {},
        onPhaseChange: () => {},
        onCheatDetected: (info) => { cheats.push({ side: info.side }); },
        onPausedChange: () => {},
        onHostCommitted: () => {},
        onResyncFailed: () => {},
      },
    );
    const push = (m: WireMessageV2, phase: WirePhase = "play") => { void phase; for (const h of handlers) h(m); };
    return { eng, match, handle, sent, cheats, applied, enterSandbox, sandboxApply, push };
  }

  it("joiner in sandbox: opponent committed mid-sandbox auto-exits, applies on true line, no cheat", async () => {
    const t = setup("joiner");
    // True line advances by one real move first (seq 1) with nobody in sandbox.
    t.push({ kind: "committed", seq: 1, phase: "play", raw: 10, postZobrist: (1n * 31n + 10n).toString(), originNonce: null });
    for (let i = 0; i < 6; i++) await Promise.resolve();
    expect(t.eng.applied).toEqual([10]);
    expect(t.handle.getSeq()).toBe(1);

    // Now the joiner enters sandbox and explores (forks the shared engine).
    await t.enterSandbox();
    await t.sandboxApply(999); // exploratory move - engine is now [10, 999]
    expect(t.match.mode).toBe("sandbox");
    expect(t.eng.applied).toEqual([10, 999]);

    // Opponent's REAL move arrives DURING sandbox. Host computed its
    // postZobrist on the true line [10, 20] = (1*31+10)*31 + 20 = 1291.
    const trueZ = ((1n * 31n + 10n) * 31n + 20n).toString();
    t.push({ kind: "committed", seq: 2, phase: "play", raw: 20, postZobrist: trueZ, originNonce: null });
    for (let i = 0; i < 6; i++) await Promise.resolve();

    // Auto-exited sandbox, applied on the TRUE line - not rolled back, not cheat.
    expect(t.cheats).toEqual([]);
    expect(t.sent.filter((m) => m.kind === "cheat-detected")).toEqual([]);
    expect(t.sent.filter((m) => m.kind === "request-snapshot")).toEqual([]);
    expect(t.match.mode).toBe("multiplayer"); // back on the live line
    expect(t.eng.applied).toEqual([10, 20]); // exploration discarded, real move applied
    expect(t.handle.getSeq()).toBe(2); // seq advanced, NOT rolled back
  });

  it("host in sandbox: opponent intent mid-sandbox auto-exits, accepted + committed, no reject", async () => {
    const t = setup("host");
    // Host makes a real move first (seq 1).
    await t.handle.submitAction(10);
    expect(t.eng.applied).toEqual([10]);
    expect(t.handle.getSeq()).toBe(1);

    // Host enters sandbox and explores.
    await t.enterSandbox();
    await t.sandboxApply(999);
    expect(t.match.mode).toBe("sandbox");

    // Joiner's REAL intent arrives during host sandbox.
    t.push({ kind: "intent", phase: "play", nonce: "j-9", raw: 20 });
    for (let i = 0; i < 6; i++) await Promise.resolve();

    // Host auto-exited, validated on the true line, broadcast committed.
    // (Two committed total: seq 1 from the host's own opening move, seq 2 from
    // the joiner's intent.) The intent's commit is the one under test.
    const committed = t.sent.filter((m) => m.kind === "committed");
    expect(committed).toHaveLength(2);
    expect(committed[committed.length - 1]).toMatchObject({ seq: 2, raw: 20, originNonce: "j-9" });
    expect(t.sent.filter((m) => m.kind === "intent-rejected")).toEqual([]);
    expect(t.match.mode).toBe("multiplayer");
    expect(t.eng.applied).toEqual([10, 20]); // exploration gone, intent applied
    expect(t.handle.getSeq()).toBe(2);
  });
});
