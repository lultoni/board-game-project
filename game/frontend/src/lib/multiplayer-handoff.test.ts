// Unit tests for the leader-handoff orchestrator. Pure: every dependency
// (PeerJS, telemetry, IDB) is injected via the hooks parameter.

import { describe, it, expect, beforeEach } from "vitest";
import { takeoverAsHost, type HandoffCarrier } from "./multiplayer-handoff";
import type { MpEngineHandle } from "./multiplayer-engine";
import type { EngineClient } from "./engine";
import { mpState } from "./multiplayer.svelte";

// --- Stubs -----------------------------------------------------------------

function makeCarrier(overrides: Partial<HandoffCarrier> = {}): HandoffCarrier {
  return {
    mode: "multiplayer",
    telemetryMatchId: null,
    ...overrides,
  };
}

function makeEng(opts: { matchLogJson?: () => Promise<string | null> } = {}): EngineClient {
  return {
    matchLogJson: opts.matchLogJson ?? (async () => '{"plies":[]}'),
  } as unknown as EngineClient;
}

function makeMpEngine(): MpEngineHandle & { promoteCalls: Array<{ matchId: string }> } {
  const promoteCalls: Array<{ matchId: string }> = [];
  return {
    submitAction: async () => ({ accepted: true }),
    notifyConnectionOpen: () => {},
    notifyConnectionLost: () => {},
    hostSendSnapshot: async () => {},
    setMatchId: () => {},
    promoteToHost: (opts) => { promoteCalls.push(opts); },
    getSeq: () => 0,
    dispose: () => {},
    promoteCalls,
  } as MpEngineHandle & { promoteCalls: Array<{ matchId: string }> };
}

interface CallLog {
  events: string[];
}

function buildHooks(log: CallLog, overrides: {
  hostWithCodeRejects?: boolean;
  startTelemetryReturns?: string | null;
  startTelemetryRejects?: boolean;
  checkpointRejects?: boolean;
} = {}): Parameters<typeof takeoverAsHost>[1] {
  return {
    destroyPeerKeepState: () => { log.events.push("destroyPeer"); },
    hostWithCode: async (code) => {
      log.events.push(`hostWithCode:${code}`);
      if (overrides.hostWithCodeRejects) throw new Error("peer-unavailable-id");
      return code;
    },
    startTelemetrySession: async (carrier, _mode, _opts) => {
      log.events.push("startTelemetry");
      if (overrides.startTelemetryRejects) throw new Error("idb-failed");
      const id = overrides.startTelemetryReturns === undefined
        ? "new-match-id-001"
        : overrides.startTelemetryReturns;
      carrier.telemetryMatchId = id;
      return id;
    },
    checkpointMatchLog: async (_id, _log) => {
      log.events.push("checkpoint");
      if (overrides.checkpointRejects) throw new Error("checkpoint-failed");
    },
  };
}

// --- Cases -----------------------------------------------------------------

describe("takeoverAsHost", () => {
  beforeEach(() => {
    // Reset mpState to a clean joiner-on-424242 baseline so each test starts
    // from the pre-handoff state the orchestrator expects.
    mpState.role = "joiner";
    mpState.code = "424242";
  });

  it("happy path: events fire in correct order, returns ok", async () => {
    const carrier = makeCarrier();
    const eng = makeEng();
    const mpEngine = makeMpEngine();
    const log: CallLog = { events: [] };
    const hooks = buildHooks(log);

    const r = await takeoverAsHost(
      { eng, mpEngine, code: "424242" },
      { ...hooks, carrier },
    );

    expect(r.ok).toBe(true);
    expect(log.events).toEqual([
      "destroyPeer",
      "startTelemetry",
      "checkpoint",
      "hostWithCode:424242",
    ]);
    expect(mpEngine.promoteCalls).toEqual([{ matchId: "new-match-id-001" }]);
    // mpState flipped to host AFTER promoteToHost; promoteToHost was called
    // BEFORE hostWithCode (verified via log ordering above).
    expect(mpState.role).toBe("host");
    expect(mpState.code).toBe("424242");
  });

  it("rejects with no-code when called with empty code", async () => {
    const log: CallLog = { events: [] };
    const r = await takeoverAsHost(
      { eng: makeEng(), mpEngine: makeMpEngine(), code: "" },
      { ...buildHooks(log), carrier: makeCarrier() },
    );
    expect(r).toEqual({ ok: false, reason: "no-code" });
    expect(log.events).toEqual([]);
  });

  it("returns telemetry-failed when startTelemetrySession returns null", async () => {
    const carrier = makeCarrier();
    const mpEngine = makeMpEngine();
    const log: CallLog = { events: [] };
    const r = await takeoverAsHost(
      { eng: makeEng(), mpEngine, code: "424242" },
      { ...buildHooks(log, { startTelemetryReturns: null }), carrier },
    );
    expect(r).toEqual({ ok: false, reason: "telemetry-failed" });
    expect(log.events).toEqual(["destroyPeer", "startTelemetry"]);
    expect(mpEngine.promoteCalls).toEqual([]);
    expect(mpState.role).toBe("joiner"); // unchanged
  });

  it("returns engine-failed when matchLogJson rejects, does NOT call hostWithCode or promoteToHost", async () => {
    const carrier = makeCarrier();
    const mpEngine = makeMpEngine();
    const log: CallLog = { events: [] };
    const eng = makeEng({ matchLogJson: async () => { throw new Error("engine-dead"); } });

    const r = await takeoverAsHost(
      { eng, mpEngine, code: "424242" },
      { ...buildHooks(log), carrier },
    );

    expect(r.ok).toBe(false);
    expect((r as { reason?: string }).reason).toBe("engine-failed");
    expect(log.events).toEqual(["destroyPeer", "startTelemetry"]);
    expect(mpEngine.promoteCalls).toEqual([]);
    expect(mpState.role).toBe("joiner");
  });

  it("returns rehost-failed when hostWithCode rejects; promotion already happened", async () => {
    const carrier = makeCarrier();
    const mpEngine = makeMpEngine();
    const log: CallLog = { events: [] };

    const r = await takeoverAsHost(
      { eng: makeEng(), mpEngine, code: "424242" },
      { ...buildHooks(log, { hostWithCodeRejects: true }), carrier },
    );

    expect(r.ok).toBe(false);
    expect((r as { reason?: string }).reason).toBe("rehost-failed");
    // Promotion DID happen — mpState is host now, wrapper was flipped.
    // The user must navigate to lobby or click again to recover.
    expect(mpEngine.promoteCalls).toEqual([{ matchId: "new-match-id-001" }]);
    expect(mpState.role).toBe("host");
    expect(log.events).toEqual([
      "destroyPeer",
      "startTelemetry",
      "checkpoint",
      "hostWithCode:424242",
    ]);
  });
});
