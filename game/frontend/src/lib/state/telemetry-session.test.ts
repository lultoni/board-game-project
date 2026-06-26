// Tests for the telemetry session glue.
//
// The helpers live in `telemetry-session.ts` as vanilla TS so vitest can load
// them without compiling Svelte runes. The match-store wires them to the
// `$state` carrier; here we pass a plain object that satisfies the
// `TelemetryCarrier` contract.

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  createTelemetrySession,
  extractPlyNo,
  type TelemetryCarrier,
  type TelemetrySession,
} from "./telemetry-session";
import type { EngineClient } from "../engine";
import { _setTelemetryStoreForTest, getTelemetryStore } from "../storage";
import { IdbTelemetryStore } from "../storage/idb-backend";

function resetIdb(): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const req = indexedDB.deleteDatabase("boardgame-matches-v2");
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    req.onblocked = () => resolve();
  });
}

interface StubState {
  plies: string[];
  finaliseCalled: number;
}

function stubEngine(state: StubState): EngineClient {
  const e: Partial<EngineClient> = {
    async latestPlyJson() {
      return state.plies.length > 0 ? state.plies[state.plies.length - 1] : null;
    },
    async matchLogJson() {
      return JSON.stringify({
        plies: state.plies.map((p) => JSON.parse(p)),
        total_plies: state.plies.length,
        total_wall_ms: state.plies.length * 100,
      });
    },
    async finaliseLog() {
      state.finaliseCalled += 1;
    },
  };
  return e as EngineClient;
}

function newCarrier(): TelemetryCarrier {
  return { telemetryMatchId: null };
}

describe("telemetry-session helpers", () => {
  let store: IdbTelemetryStore;
  let session: TelemetrySession;

  beforeEach(async () => {
    await resetIdb();
    store = new IdbTelemetryStore();
    _setTelemetryStoreForTest(store);
    session = createTelemetrySession();
  });

  afterEach(async () => {
    await store.close();
    _setTelemetryStoreForTest(null);
    await resetIdb();
  });

  it("startTelemetrySession assigns a matchId for logged modes", async () => {
    const carrier = newCarrier();
    const id = await session.startTelemetrySession(carrier, "hvai");
    expect(id).not.toBeNull();
    expect(carrier.telemetryMatchId).toBe(id);
  });

  it("startTelemetrySession is a no-op for sandbox/replay/idle", async () => {
    const carrier = newCarrier();
    expect(await session.startTelemetrySession(carrier, "sandbox")).toBeNull();
    expect(await session.startTelemetrySession(carrier, "replay")).toBeNull();
    expect(await session.startTelemetrySession(carrier, "idle")).toBeNull();
    expect(carrier.telemetryMatchId).toBeNull();
  });

  it("startTelemetrySession skips multiplayer joiner role (authoritative-host model)", async () => {
    const carrier = newCarrier();
    const id = await session.startTelemetrySession(carrier, "multiplayer", {
      multiplayerCode: "281947",
      multiplayerRole: "joiner",
    });
    expect(id).toBeNull();
    expect(carrier.telemetryMatchId).toBeNull();
  });

  it("recordPly appends each ply from latestPlyJson into IDB", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await session.startTelemetrySession(carrier, "hvh"))!;
    state.plies.push(JSON.stringify({ ply_no: 1, foo: "a" }));
    await session.recordPly(carrier, eng);
    state.plies.push(JSON.stringify({ ply_no: 2, foo: "b" }));
    await session.recordPly(carrier, eng);
    const plies = await getTelemetryStore().getPlies(id);
    expect(plies.map((p) => p.plyNo)).toEqual([1, 2]);
    expect(JSON.parse(plies[0].plyJson).foo).toBe("a");
  });

  it("recordPly is a no-op when no session is active", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [JSON.stringify({ ply_no: 1 })], finaliseCalled: 0 };
    const eng = stubEngine(state);
    await session.recordPly(carrier, eng);
    expect(carrier.telemetryMatchId).toBeNull();
  });

  it("finalizeTelemetrySession writes a consolidated finalised match", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await session.startTelemetrySession(carrier, "hvai"))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    state.plies.push(JSON.stringify({ ply_no: 2 }));
    await session.recordPly(carrier, eng);
    await session.recordPly(carrier, eng);
    await session.finalizeTelemetrySession(carrier, eng, "checkmate", 0);
    expect(carrier.telemetryMatchId).toBeNull();
    const finalised = await getTelemetryStore().getMatch(id);
    expect(finalised).not.toBeNull();
    expect(finalised!.endReason).toBe("checkmate");
    expect(finalised!.resultByte).toBe(0);
    expect(finalised!.totalPlies).toBe(2);
    expect(finalised!.totalWallMs).toBe(200);
  });

  it("abandonTelemetrySession marks the session abandoned and clears the id", async () => {
    const carrier = newCarrier();
    const id = (await session.startTelemetrySession(carrier, "hvh"))!;
    await session.abandonTelemetrySession(carrier);
    expect(carrier.telemetryMatchId).toBeNull();
    const meta = await getTelemetryStore().getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
  });

  it("abandonTelemetrySession with an engine stashes the partial MatchLog", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await session.startTelemetrySession(carrier, "aivai"))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    state.plies.push(JSON.stringify({ ply_no: 2 }));
    await session.recordPly(carrier, eng);
    await session.recordPly(carrier, eng);
    await session.abandonTelemetrySession(carrier, eng);

    // Status is abandoned but the partial log is recoverable so the
    // library can hand it to the inspector / export.
    const meta = await getTelemetryStore().getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
    expect(meta!.totalPlies).toBe(2);
    const full = await getTelemetryStore().getMatch(id);
    expect(full).not.toBeNull();
    expect(full!.status).toBe("abandoned");
    expect(JSON.parse(full!.matchLogJson).total_plies).toBe(2);
  });

  it("networkLostTelemetrySession marks status network-lost and stashes the partial MatchLog", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await session.startTelemetrySession(carrier, "multiplayer", {
      multiplayerCode: "281947",
      multiplayerRole: "host",
    }))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    await session.recordPly(carrier, eng);
    await session.networkLostTelemetrySession(carrier, eng);
    expect(carrier.telemetryMatchId).toBeNull();

    const meta = await getTelemetryStore().getMatchMeta(id);
    expect(meta!.status).toBe("mid-match-network-lost");
    expect(meta!.multiplayerCode).toBe("281947");
    expect(meta!.multiplayerRole).toBe("host");
    expect(meta!.totalPlies).toBe(1);
  });

  it("extractPlyNo handles the engine's field-order convention", () => {
    // Engine emits ply_no as the first field; the regex pulls it from any
    // position via JSON.parse fallback.
    expect(extractPlyNo(JSON.stringify({ ply_no: 1, foo: "a" }))).toBe(1);
    expect(extractPlyNo(JSON.stringify({ foo: "x", ply_no: 7, bar: "y" }))).toBe(7);
    expect(extractPlyNo("not json")).toBeNull();
  });

  it("disabled-for-session latch is per-instance — second factory call ignores the first's failures", async () => {
    // Simulate startMatch failing on instance A by swapping the store with
    // one that rejects. The disabled latch on A flips. Instance B, created
    // afterwards, must NOT see the latch — it should still attempt the write.
    const failingStore: Partial<IdbTelemetryStore> = {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      startMatch: () => Promise.reject(new Error("boom")) as any,
    };
    _setTelemetryStoreForTest(failingStore as IdbTelemetryStore);

    const a = createTelemetrySession();
    const carrierA = newCarrier();
    expect(await a.startTelemetrySession(carrierA, "hvai")).toBeNull();
    // Latch on A is now hot — any subsequent recordPly/etc. would be no-op
    // even if we restored a working store. Confirm by restoring the working
    // store and verifying B (new instance) succeeds while A stays muted.
    _setTelemetryStoreForTest(store);

    const b = createTelemetrySession();
    const carrierB = newCarrier();
    const idB = await b.startTelemetrySession(carrierB, "hvai");
    expect(idB).not.toBeNull();
    expect(carrierB.telemetryMatchId).toBe(idB);

    // A is still disabled — startTelemetrySession would clear the latch, but
    // we only call recordPly here to prove the latch was set, not that it
    // can never be cleared.
    const eng = stubEngine({ plies: [JSON.stringify({ ply_no: 1 })], finaliseCalled: 0 });
    carrierA.telemetryMatchId = "fake-id-that-would-trigger-write-if-not-disabled";
    await a.recordPly(carrierA, eng);
    // No plies written under that fake id (the latch short-circuited).
    const pliesA = await getTelemetryStore().getPlies("fake-id-that-would-trigger-write-if-not-disabled");
    expect(pliesA).toEqual([]);
  });
});
