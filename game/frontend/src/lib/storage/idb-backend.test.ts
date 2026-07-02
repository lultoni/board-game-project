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
    const req = indexedDB.deleteDatabase("boardgame-matches-v2");
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
    // Inject explicit nowMs values so ULID timestamp prefixes are strictly
    // ordered. Avoids a real-time `setTimeout(5)` whose 5ms spacing can be
    // swallowed by fast CI OR exceeded by slow CI.
    const t0 = Date.now();
    const a = await store.startMatch({ mode: "hvh" }, t0);
    const b = await store.startMatch({ mode: "hvai" }, t0 + 10);
    const c = await store.startMatch({ mode: "hvai" }, t0 + 20);
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
    const { bundle: bundleJson, skipped } = await store.bundleMatches([a, b]);
    const bundle = JSON.parse(bundleJson);
    expect(bundle.schema).toBe("boardgame-bundle-v1");
    expect(bundle.logs).toHaveLength(1);
    expect(bundle.logs[0]).toEqual({ plies: [], a: 1 });
    expect(skipped).toEqual([b]);
  });

  it("bundleMatches with no finalised matches still returns a valid envelope", async () => {
    const id = await store.startMatch({ mode: "hvh" });
    const { bundle: bundleJson, skipped } = await store.bundleMatches([id]);
    const bundle = JSON.parse(bundleJson);
    expect(bundle.logs).toEqual([]);
    expect(skipped).toEqual([id]);
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
    const { bundle: bundleJson, skipped } = await store.bundleMatches([a, b]);
    const bundle = JSON.parse(bundleJson);
    expect(bundle.logs).toHaveLength(1);
    expect(bundle.logs[0]).toEqual({ plies: [], a: 1 });
    expect(skipped).toEqual([b]);
  });

  describe("updateMultiplayerRole", () => {
    it("flips joiner → host in place", async () => {
      const id = await store.startMatch({
        mode: "multiplayer",
        multiplayerCode: "555555",
        multiplayerRole: "joiner",
      });
      await store.updateMultiplayerRole(id, "host");
      const meta = await store.getMatchMeta(id);
      expect(meta!.multiplayerRole).toBe("host");
      // Other fields untouched.
      expect(meta!.multiplayerCode).toBe("555555");
      expect(meta!.status).toBe("in-progress");
    });

    it("is a no-op when the row is missing", async () => {
      await expect(store.updateMultiplayerRole("does-not-exist", "host")).resolves.toBeUndefined();
    });

    it("is idempotent — same role is a no-op", async () => {
      const id = await store.startMatch({ mode: "multiplayer", multiplayerRole: "host" });
      await store.updateMultiplayerRole(id, "host");
      const meta = await store.getMatchMeta(id);
      expect(meta!.multiplayerRole).toBe("host");
    });
  });

  describe("joined_codes", () => {
    it("recordJoinedCode → listJoinedCodes returns the entry, most-recent first", async () => {
      await store.recordJoinedCode({ code: "281947", hostPeerId: "peer-a" });
      await new Promise((r) => setTimeout(r, 5));
      await store.recordJoinedCode({ code: "194723", hostPeerId: "peer-b", lastSeenSeq: 12 });
      const all = await store.listJoinedCodes();
      expect(all.map((e) => e.code)).toEqual(["194723", "281947"]);
      expect(all[0].lastSeenSeq).toBe(12);
      expect(all[0].hostPeerId).toBe("peer-b");
    });

    it("recordJoinedCode is idempotent and refreshes lastJoinedAt + optional fields", async () => {
      await store.recordJoinedCode({ code: "100000", hostPeerId: "old", lastSeenSeq: 5 });
      const before = (await store.listJoinedCodes())[0];
      await new Promise((r) => setTimeout(r, 5));
      await store.recordJoinedCode({ code: "100000", lastSeenSeq: 9 });
      const after = (await store.listJoinedCodes())[0];
      expect(after.code).toBe("100000");
      expect(after.lastJoinedAtUnixMs).toBeGreaterThan(before.lastJoinedAtUnixMs);
      // hostPeerId preserved when re-recording without supplying it.
      expect(after.hostPeerId).toBe("old");
      expect(after.lastSeenSeq).toBe(9);
    });

    it("forgetJoinedCode removes the entry; no-op when missing", async () => {
      await store.recordJoinedCode({ code: "111111" });
      await store.recordJoinedCode({ code: "222222" });
      await store.forgetJoinedCode("111111");
      const all = await store.listJoinedCodes();
      expect(all.map((e) => e.code)).toEqual(["222222"]);
      // No-op for absent code.
      await store.forgetJoinedCode("does-not-exist");
      expect((await store.listJoinedCodes()).map((e) => e.code)).toEqual(["222222"]);
    });
  });
});

// Regression: users who opened `boardgame-matches-v2` from a build that
// predated the joined_codes store had a v=1 DB on disk without that store.
// Calling listJoinedCodes / recordJoinedCode on that DB threw
// "One of the specified object stores was not found". Bumping DB_VERSION to 2
// re-runs onupgradeneeded; the store-creation guards in idb-backend already
// branch on objectStoreNames.contains, so the upgrade is idempotent.
describe("schema migration v1 → v2", () => {
  beforeEach(async () => {
    await new Promise<void>((resolve, reject) => {
      const req = indexedDB.deleteDatabase("boardgame-matches-v2");
      req.onsuccess = () => resolve();
      req.onerror = () => reject(req.error);
      req.onblocked = () => resolve();
    });
  });

  it("creates joined_codes on browsers that have v1 without it; preserves matches", async () => {
    // 1. Hand-create v1 of the DB containing matches+plies but NOT joined_codes.
    //    This mirrors the on-disk state of users who opened the DB between the
    //    DB_NAME bump and the joined_codes store landing.
    await new Promise<void>((resolve, reject) => {
      const req = indexedDB.open("boardgame-matches-v2", 1);
      req.onupgradeneeded = () => {
        const db = req.result;
        const m = db.createObjectStore("matches", { keyPath: "matchId" });
        m.createIndex("mode", "mode", { unique: false });
        m.createIndex("status", "status", { unique: false });
        m.createIndex("startedAt", "startedAtUnixMs", { unique: false });
        db.createObjectStore("plies", { keyPath: ["matchId", "plyNo"] });
      };
      req.onsuccess = () => {
        const db = req.result;
        // Seed a row so we can verify it survives the migration.
        const tx = db.transaction("matches", "readwrite");
        tx.objectStore("matches").put({
          matchId: "01TEST-LEGACY-ROW",
          mode: "multiplayer",
          startedAtUnixMs: 1_700_000_000_000,
          status: "in-progress",
          multiplayerCode: "123456",
          multiplayerRole: "host",
        });
        tx.oncomplete = () => {
          db.close();
          resolve();
        };
        tx.onerror = () => reject(tx.error);
      };
      req.onerror = () => reject(req.error);
    });

    // 2. Open under the production code (v2). Should run onupgradeneeded and
    //    create joined_codes without touching matches.
    const store = new IdbTelemetryStore();

    // 3. joined_codes is usable.
    await store.recordJoinedCode({ code: "999999", hostPeerId: "p" });
    const codes = await store.listJoinedCodes();
    expect(codes.map((e) => e.code)).toEqual(["999999"]);

    // 4. The pre-existing matches row is still readable.
    const meta = await store.getMatchMeta("01TEST-LEGACY-ROW");
    expect(meta).not.toBeNull();
    expect(meta?.multiplayerRole).toBe("host");
    expect(meta?.multiplayerCode).toBe("123456");

    await store.close();
  });
});
