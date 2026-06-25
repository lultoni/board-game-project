// Tests for the telemetry session glue.
//
// The helpers live in `telemetry-session.ts` as vanilla TS so vitest can load
// them without compiling Svelte runes. The match-store wires them to the
// `$state` carrier; here we pass a plain object that satisfies the
// `TelemetryCarrier` contract.

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  startTelemetrySession,
  recordPly,
  finalizeTelemetrySession,
  abandonTelemetrySession,
  networkLostTelemetrySession,
  extractPlyNo,
  type TelemetryCarrier,
} from "./telemetry-session";
import type { EngineClient } from "../engine/types";
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

  beforeEach(async () => {
    await resetIdb();
    store = new IdbTelemetryStore();
    _setTelemetryStoreForTest(store);
  });

  afterEach(async () => {
    await store.close();
    _setTelemetryStoreForTest(null);
    await resetIdb();
  });

  it("startTelemetrySession assigns a matchId for logged modes", async () => {
    const carrier = newCarrier();
    const id = await startTelemetrySession(carrier, "hvai");
    expect(id).not.toBeNull();
    expect(carrier.telemetryMatchId).toBe(id);
  });

  it("startTelemetrySession is a no-op for sandbox/replay/idle", async () => {
    const carrier = newCarrier();
    expect(await startTelemetrySession(carrier, "sandbox")).toBeNull();
    expect(await startTelemetrySession(carrier, "replay")).toBeNull();
    expect(await startTelemetrySession(carrier, "idle")).toBeNull();
    expect(carrier.telemetryMatchId).toBeNull();
  });

  it("startTelemetrySession skips multiplayer joiner role (authoritative-host model)", async () => {
    const carrier = newCarrier();
    const id = await startTelemetrySession(carrier, "multiplayer", {
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
    const id = (await startTelemetrySession(carrier, "hvh"))!;
    state.plies.push(JSON.stringify({ ply_no: 1, foo: "a" }));
    await recordPly(carrier, eng);
    state.plies.push(JSON.stringify({ ply_no: 2, foo: "b" }));
    await recordPly(carrier, eng);
    const plies = await getTelemetryStore().getPlies(id);
    expect(plies.map((p) => p.plyNo)).toEqual([1, 2]);
    expect(JSON.parse(plies[0].plyJson).foo).toBe("a");
  });

  it("recordPly is a no-op when no session is active", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [JSON.stringify({ ply_no: 1 })], finaliseCalled: 0 };
    const eng = stubEngine(state);
    await recordPly(carrier, eng);
    expect(carrier.telemetryMatchId).toBeNull();
  });

  it("finalizeTelemetrySession writes a consolidated finalised match", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await startTelemetrySession(carrier, "hvai"))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    state.plies.push(JSON.stringify({ ply_no: 2 }));
    await recordPly(carrier, eng);
    await recordPly(carrier, eng);
    await finalizeTelemetrySession(carrier, eng, "checkmate", 0);
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
    const id = (await startTelemetrySession(carrier, "hvh"))!;
    await abandonTelemetrySession(carrier);
    expect(carrier.telemetryMatchId).toBeNull();
    const meta = await getTelemetryStore().getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
  });

  it("abandonTelemetrySession with an engine stashes the partial MatchLog", async () => {
    const carrier = newCarrier();
    const state: StubState = { plies: [], finaliseCalled: 0 };
    const eng = stubEngine(state);
    const id = (await startTelemetrySession(carrier, "aivai"))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    state.plies.push(JSON.stringify({ ply_no: 2 }));
    await recordPly(carrier, eng);
    await recordPly(carrier, eng);
    await abandonTelemetrySession(carrier, eng);

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
    const id = (await startTelemetrySession(carrier, "multiplayer", {
      multiplayerCode: "281947",
      multiplayerRole: "host",
    }))!;
    state.plies.push(JSON.stringify({ ply_no: 1 }));
    await recordPly(carrier, eng);
    await networkLostTelemetrySession(carrier, eng);
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
});
