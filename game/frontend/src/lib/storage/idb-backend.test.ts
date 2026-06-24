// Tests for the IDB telemetry backend. Run with `npm test`.
//
// fake-indexeddb is loaded globally via vitest.setup.ts.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { IdbTelemetryStore } from "./idb-backend";
import { newMatchId } from "./types";

// fake-indexeddb resets when we delete the database. Each test gets a
// fresh store via beforeEach so state doesn't leak between cases.
async function resetDb(): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const req = indexedDB.deleteDatabase("boardgame-matches");
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    req.onblocked = () => resolve();
  });
}

describe("newMatchId", () => {
  it("produces 26-char Crockford-base32 IDs", () => {
    const id = newMatchId();
    expect(id).toHaveLength(26);
    expect(id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  });

  it("sorts lexicographically by time", () => {
    const a = newMatchId(1_000_000);
    const b = newMatchId(2_000_000);
    expect(a < b).toBe(true);
  });

  it("is unique across rapid calls", () => {
    const ids = new Set(Array.from({ length: 500 }, () => newMatchId()));
    expect(ids.size).toBe(500);
  });
});

describe("IdbTelemetryStore", () => {
  let store: IdbTelemetryStore;

  beforeEach(async () => {
    await resetDb();
    store = new IdbTelemetryStore();
  });

  afterEach(async () => {
    await store.close();
    await resetDb();
  });

  it("startMatch returns a ULID and persists an in-progress meta row", async () => {
    const id = await store.startMatch({ mode: "hvai" });
    expect(id).toHaveLength(26);
    const meta = await store.getMatchMeta(id);
    expect(meta).not.toBeNull();
    expect(meta!.matchId).toBe(id);
    expect(meta!.mode).toBe("hvai");
    expect(meta!.status).toBe("in-progress");
  });

  it("appendPly stores per-ply entries and getPlies returns them in order", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    await store.appendPly(id, '{"ply":1}', 1);
    await store.appendPly(id, '{"ply":3}', 3); // out-of-order on purpose
    await store.appendPly(id, '{"ply":2}', 2);
    const plies = await store.getPlies(id);
    expect(plies.map((p) => p.plyNo)).toEqual([1, 2, 3]);
    expect(plies[0].plyJson).toBe('{"ply":1}');
  });

  it("finalizeMatch transitions status to ended and stores the consolidated log", async () => {
    const id = await store.startMatch({ mode: "hvai" });
    await store.appendPly(id, '{"ply":1}', 1);
    await store.finalizeMatch(id, '{"plies":[{"ply":1}]}', "checkmate", 0, 1, 1234);
    const m = await store.getMatch(id);
    expect(m).not.toBeNull();
    expect(m!.status).toBe("ended");
    expect(m!.endReason).toBe("checkmate");
    expect(m!.resultByte).toBe(0);
    expect(m!.totalPlies).toBe(1);
    expect(m!.totalWallMs).toBe(1234);
    expect(JSON.parse(m!.matchLogJson)).toEqual({ plies: [{ ply: 1 }] });
  });

  it("finalizeMatch throws if no match exists", async () => {
    await expect(
      store.finalizeMatch("does-not-exist", "{}", "checkmate", 0, 0, 0),
    ).rejects.toThrow(/no match with id/);
  });

  it("getMatch returns null for in-progress matches", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    expect(await store.getMatch(id)).toBeNull();
  });

  it("markAbandoned sets status when in-progress", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    await store.markAbandoned(id);
    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
  });

  it("markAbandoned does not overwrite a finalised match", async () => {
    const id = await store.startMatch({ mode: "hvai" });
    await store.finalizeMatch(id, "{}", "checkmate", 1, 1, 100);
    await store.markAbandoned(id);
    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("ended");
  });

  it("markNetworkLost transitions in-progress → mid-match-network-lost", async () => {
    const id = await store.startMatch({
      mode: "hvh",
      multiplayerCode: "482917",
      multiplayerRole: "host",
    });
    await store.markNetworkLost(id);
    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("mid-match-network-lost");
    expect(meta!.multiplayerCode).toBe("482917");
  });

  it("dismissNetworkLost transitions mid-match-network-lost → abandoned and preserves the partial log", async () => {
    const id = await store.startMatch({
      mode: "hvh",
      multiplayerCode: "194723",
      multiplayerRole: "joiner",
    });
    const partial = JSON.stringify({ plies: [{ action: { raw: 7 } }], total_plies: 1, total_wall_ms: 50 });
    await store.markNetworkLost(id, partial);

    await store.dismissNetworkLost(id);
    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
    expect(meta!.multiplayerCode).toBe("194723");
    expect(meta!.totalPlies).toBe(1);

    // Partial log still recoverable via getMatch.
    const full = await store.getMatch(id);
    expect(full).not.toBeNull();
    expect(JSON.parse(full!.matchLogJson).total_plies).toBe(1);
  });

  it("dismissNetworkLost is a no-op on non-network-lost rows", async () => {
    const inProg = await store.startMatch({ mode: "hvh" });
    await store.dismissNetworkLost(inProg);
    expect((await store.getMatchMeta(inProg))!.status).toBe("in-progress");

    const finished = await store.startMatch({ mode: "hvai" });
    await store.finalizeMatch(finished, "{}", "checkmate", 0, 1, 100);
    await store.dismissNetworkLost(finished);
    expect((await store.getMatchMeta(finished))!.status).toBe("ended");
  });

  it("listMatches sorts most-recent-first and filters by mode + status", async () => {
    const a = await store.startMatch({ mode: "hvh" });
    await new Promise((r) => setTimeout(r, 5));
    const b = await store.startMatch({ mode: "hvai" });
    await new Promise((r) => setTimeout(r, 5));
    const c = await store.startMatch({ mode: "hvai" });
    await store.finalizeMatch(b, "{}", "checkmate", 0, 1, 0);

    const all = await store.listMatches();
    expect(all.map((m) => m.matchId)).toEqual([c, b, a]);

    const onlyHvAi = await store.listMatches({ mode: "hvai" });
    expect(onlyHvAi.map((m) => m.matchId).sort()).toEqual([b, c].sort());

    const onlyEnded = await store.listMatches({ status: "ended" });
    expect(onlyEnded.map((m) => m.matchId)).toEqual([b]);
  });

  it("deleteMatch removes both the meta and per-ply rows", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    await store.appendPly(id, "{}", 1);
    await store.appendPly(id, "{}", 2);
    await store.deleteMatch(id);
    expect(await store.getMatchMeta(id)).toBeNull();
    expect(await store.getPlies(id)).toEqual([]);
  });

  it("bundleMatches wraps finalised match logs in an envelope and skips corrupt ones", async () => {
    const a = await store.startMatch({ mode: "hvh" });
    const b = await store.startMatch({ mode: "hvai" });
    await store.finalizeMatch(a, '{"plies":[],"a":1}', "checkmate", 0, 0, 0);
    await store.finalizeMatch(b, "not-json-at-all", "checkmate", 1, 0, 0);
    const bundle = JSON.parse(await store.bundleMatches([a, b]));
    expect(bundle.schema).toBe("boardgame-bundle-v1");
    expect(bundle.logs).toHaveLength(1);
    expect(bundle.logs[0]).toEqual({ plies: [], a: 1 });
  });

  it("bundleMatches with no finalised matches still returns a valid envelope", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    const bundle = JSON.parse(await store.bundleMatches([id]));
    expect(bundle.logs).toEqual([]);
  });

  it("listMatches projects end-of-match fields on ended rows only", async () => {
    const pending = await store.startMatch({ mode: "hvh" });
    const finished = await store.startMatch({ mode: "hvai" });
    await store.finalizeMatch(finished, '{"plies":[]}', "checkmate", 1, 7, 4200);

    const all = await store.listMatches();
    const byId = new Map(all.map((m) => [m.matchId, m]));

    const pm = byId.get(pending)!;
    expect(pm.status).toBe("in-progress");
    expect(pm.endReason).toBeUndefined();
    expect(pm.resultByte).toBeUndefined();
    expect(pm.totalPlies).toBeUndefined();
    expect(pm.endedAtUnixMs).toBeUndefined();

    const fm = byId.get(finished)!;
    expect(fm.status).toBe("ended");
    expect(fm.endReason).toBe("checkmate");
    expect(fm.resultByte).toBe(1);
    expect(fm.totalPlies).toBe(7);
    expect(typeof fm.endedAtUnixMs).toBe("number");
  });

  it("markAbandoned with a partial log lets getMatch return the abandoned row", async () => {
    const id = await store.startMatch({ mode: "aivai" });
    const partial = JSON.stringify({
      start_fen: "8/8/…",
      plies: [{ action: { raw: 1 } }, { action: { raw: 2 } }],
      total_plies: 2,
      total_wall_ms: 250,
    });
    await store.markAbandoned(id, partial);

    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
    expect(meta!.totalPlies).toBe(2);
    expect(typeof meta!.endedAtUnixMs).toBe("number");

    const full = await store.getMatch(id);
    expect(full).not.toBeNull();
    expect(full!.status).toBe("abandoned");
    expect(full!.endReason).toBe("abandoned");
    expect(full!.resultByte).toBe(3);
    expect(JSON.parse(full!.matchLogJson).total_plies).toBe(2);
  });

  it("markAbandoned without a partial log keeps getMatch null", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    await store.markAbandoned(id);
    expect(await store.getMatch(id)).toBeNull();
    const meta = await store.getMatchMeta(id);
    expect(meta!.status).toBe("abandoned");
    expect(meta!.totalPlies).toBeUndefined();
  });

  it("bundleMatches includes abandoned matches that carry a partial log", async () => {
    const a = await store.startMatch({ mode: "aivai" });
    const b = await store.startMatch({ mode: "hvh" });
    await store.markAbandoned(a, JSON.stringify({ plies: [], a: 1 }));
    await store.markAbandoned(b); // no log
    const bundle = JSON.parse(await store.bundleMatches([a, b]));
    expect(bundle.logs).toHaveLength(1);
    expect(bundle.logs[0]).toEqual({ plies: [], a: 1 });
  });
});
