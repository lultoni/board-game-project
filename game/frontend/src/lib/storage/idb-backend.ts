// IndexedDB backend for the telemetry store.
//
// Two object stores:
//   - "matches": keyed by matchId. Holds MatchMeta + (on finalise) the full
//     consolidated log. Indexes on mode, startedAtUnixMs, status.
//   - "plies": keyed by [matchId, plyNo]. Append-only, kept separate so
//     per-ply writes don't churn the consolidated doc.
//
// All operations transact independently. The per-ply append is the hot
// path; everything else is cold-path (open/finalize/list/bundle).

import type {
  EndReason,
  FinalisedMatch,
  MatchFilter,
  MatchMeta,
  MatchStatus,
  PlyEntry,
  TelemetryStore,
} from "./types";
import { newMatchId } from "./types";
import type { MatchMode } from "../state/match-store.svelte";

const DB_NAME = "boardgame-matches";
const DB_VERSION = 1;
const STORE_MATCHES = "matches";
const STORE_PLIES = "plies";

interface MatchRow extends MatchMeta {
  // finalise fields, present only once status==="ended"
  endedAtUnixMs?: number;
  endReason?: EndReason;
  resultByte?: 0 | 1 | 2 | 3;
  matchLogJson?: string;
  totalPlies?: number;
  totalWallMs?: number;
}

function openDb(factory: IDBFactory): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = factory.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_MATCHES)) {
        const s = db.createObjectStore(STORE_MATCHES, { keyPath: "matchId" });
        s.createIndex("mode", "mode", { unique: false });
        s.createIndex("status", "status", { unique: false });
        s.createIndex("startedAt", "startedAtUnixMs", { unique: false });
      }
      if (!db.objectStoreNames.contains(STORE_PLIES)) {
        db.createObjectStore(STORE_PLIES, { keyPath: ["matchId", "plyNo"] });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function awaitTx(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
    tx.onabort = () => reject(tx.error);
  });
}

function awaitReq<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

/** Project a stored row into the `MatchMeta` projection used by listMatches
 *  and getMatchMeta. End-of-match fields are populated when present on the
 *  row — `finalizeMatch` always sets them; `markAbandoned`/`markNetworkLost`
 *  set them too when a partial MatchLog is supplied. */
function rowToMeta(r: MatchRow): MatchMeta {
  const m: MatchMeta = {
    matchId: r.matchId,
    mode: r.mode,
    startedAtUnixMs: r.startedAtUnixMs,
    status: r.status,
    multiplayerCode: r.multiplayerCode ?? null,
    multiplayerRole: r.multiplayerRole ?? null,
  };
  if (r.endedAtUnixMs !== undefined) m.endedAtUnixMs = r.endedAtUnixMs;
  if (r.endReason !== undefined)     m.endReason = r.endReason;
  if (r.resultByte !== undefined)    m.resultByte = r.resultByte;
  if (r.totalPlies !== undefined)    m.totalPlies = r.totalPlies;
  return m;
}

function matchesFilter(row: MatchMeta, f: MatchFilter): boolean {
  if (f.mode) {
    const allowed = Array.isArray(f.mode) ? f.mode : [f.mode];
    if (!allowed.includes(row.mode)) return false;
  }
  if (f.status) {
    const allowed: MatchStatus[] = Array.isArray(f.status) ? f.status : [f.status];
    if (!allowed.includes(row.status)) return false;
  }
  if (f.startedAfterUnixMs !== undefined && row.startedAtUnixMs < f.startedAfterUnixMs) return false;
  if (f.startedBeforeUnixMs !== undefined && row.startedAtUnixMs > f.startedBeforeUnixMs) return false;
  return true;
}

export class IdbTelemetryStore implements TelemetryStore {
  #dbPromise: Promise<IDBDatabase>;

  constructor(factory: IDBFactory = (globalThis as { indexedDB: IDBFactory }).indexedDB) {
    if (!factory) {
      throw new Error("IndexedDB not available in this environment");
    }
    this.#dbPromise = openDb(factory);
  }

  /** Closes the underlying IDB connection. Tests call this between cases
   *  so `deleteDatabase` can complete. Production code rarely needs it —
   *  the connection lives for the app's lifetime. */
  async close(): Promise<void> {
    const db = await this.#dbPromise;
    db.close();
  }

  async startMatch(
    meta: { mode: MatchMode; multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null },
  ): Promise<string> {
    const db = await this.#dbPromise;
    const matchId = newMatchId();
    const row: MatchRow = {
      matchId,
      mode: meta.mode,
      startedAtUnixMs: Date.now(),
      status: "in-progress",
      multiplayerCode: meta.multiplayerCode ?? null,
      multiplayerRole: meta.multiplayerRole ?? null,
    };
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    tx.objectStore(STORE_MATCHES).put(row);
    await awaitTx(tx);
    return matchId;
  }

  async appendPly(matchId: string, plyJson: string, plyNo: number): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_PLIES, "readwrite");
    const entry: PlyEntry = {
      matchId,
      plyNo,
      recordedAtUnixMs: Date.now(),
      plyJson,
    };
    tx.objectStore(STORE_PLIES).put(entry);
    await awaitTx(tx);
  }

  async finalizeMatch(
    matchId: string,
    matchLogJson: string,
    endReason: EndReason,
    resultByte: 0 | 1 | 2 | 3,
    totalPlies: number,
    totalWallMs: number,
  ): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing) {
      // Caller bug — finalising a match we never started. Refuse rather
      // than silently inventing metadata.
      throw new Error(`finalizeMatch: no match with id ${matchId}`);
    }
    const updated: MatchRow = {
      ...existing,
      status: "ended",
      endedAtUnixMs: Date.now(),
      endReason,
      resultByte,
      matchLogJson,
      totalPlies,
      totalWallMs,
    };
    store.put(updated);
    await awaitTx(tx);
  }

  async markAbandoned(matchId: string, partialLogJson?: string): Promise<void> {
    await this.#setStatusIfInProgress(matchId, "abandoned", partialLogJson);
  }

  async markNetworkLost(matchId: string, partialLogJson?: string): Promise<void> {
    await this.#setStatusIfInProgress(matchId, "mid-match-network-lost", partialLogJson);
  }

  async dismissNetworkLost(matchId: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing) return;
    if (existing.status !== "mid-match-network-lost") return;
    const updated: MatchRow = { ...existing, status: "abandoned" };
    store.put(updated);
    await awaitTx(tx);
  }

  async #setStatusIfInProgress(
    matchId: string,
    status: MatchStatus,
    partialLogJson?: string,
  ): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing) return;
    // Don't overwrite a terminal status — finalize wins over abandonment.
    if (existing.status !== "in-progress") return;
    const updated: MatchRow = { ...existing, status };
    // If the caller hands us a partial MatchLog (engine's current view), keep
    // it so the library can still open this match in the inspector / export
    // it. Parse defensively to count plies for the meta projection.
    if (partialLogJson) {
      updated.matchLogJson = partialLogJson;
      updated.endedAtUnixMs = Date.now();
      try {
        const parsed = JSON.parse(partialLogJson) as {
          total_plies?: number;
          total_wall_ms?: number;
        };
        if (typeof parsed.total_plies === "number") updated.totalPlies = parsed.total_plies;
        if (typeof parsed.total_wall_ms === "number") updated.totalWallMs = parsed.total_wall_ms;
      } catch {
        // Corrupt JSON shouldn't block marking the status.
      }
    }
    store.put(updated);
    await awaitTx(tx);
  }

  async getMatchMeta(matchId: string): Promise<MatchMeta | null> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readonly");
    const row = await awaitReq<MatchRow | undefined>(tx.objectStore(STORE_MATCHES).get(matchId));
    await awaitTx(tx);
    if (!row) return null;
    return rowToMeta(row);
  }

  async getMatch(matchId: string): Promise<FinalisedMatch | null> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readonly");
    const row = await awaitReq<MatchRow | undefined>(tx.objectStore(STORE_MATCHES).get(matchId));
    await awaitTx(tx);
    if (!row) return null;
    // Return the consolidated record for ended matches AND for abandoned /
    // network-lost matches that carry a partial MatchLog (captured by
    // markAbandoned when the engine view was available). Without a log
    // there's nothing to hand off to inspector / export.
    if (!row.matchLogJson) return null;
    return {
      matchId: row.matchId,
      mode: row.mode,
      startedAtUnixMs: row.startedAtUnixMs,
      status: row.status,
      multiplayerCode: row.multiplayerCode ?? null,
      multiplayerRole: row.multiplayerRole ?? null,
      endedAtUnixMs: row.endedAtUnixMs ?? row.startedAtUnixMs,
      endReason: row.endReason ?? "abandoned",
      resultByte: row.resultByte ?? 3,
      matchLogJson: row.matchLogJson,
      totalPlies: row.totalPlies ?? 0,
      totalWallMs: row.totalWallMs ?? 0,
    };
  }

  async getPlies(matchId: string): Promise<PlyEntry[]> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_PLIES, "readonly");
    const range = IDBKeyRange.bound([matchId, -Infinity], [matchId, Infinity]);
    const all = await awaitReq<PlyEntry[]>(tx.objectStore(STORE_PLIES).getAll(range));
    await awaitTx(tx);
    return all.sort((a, b) => a.plyNo - b.plyNo);
  }

  async listMatches(filter: MatchFilter = {}): Promise<MatchMeta[]> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readonly");
    const all = await awaitReq<MatchRow[]>(tx.objectStore(STORE_MATCHES).getAll());
    await awaitTx(tx);
    return all
      .map(rowToMeta)
      .filter((m) => matchesFilter(m, filter))
      .sort((a, b) => b.startedAtUnixMs - a.startedAtUnixMs);
  }

  async deleteMatch(matchId: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction([STORE_MATCHES, STORE_PLIES], "readwrite");
    tx.objectStore(STORE_MATCHES).delete(matchId);
    const range = IDBKeyRange.bound([matchId, -Infinity], [matchId, Infinity]);
    const plies = tx.objectStore(STORE_PLIES);
    const cursor = plies.openCursor(range);
    cursor.onsuccess = () => {
      const c = cursor.result;
      if (c) {
        c.delete();
        c.continue();
      }
    };
    await awaitTx(tx);
  }

  async bundleMatches(matchIds: string[]): Promise<string> {
    // L5b will surface this as the "Send to Designer" download. Bundle
    // format matches ADR-005 L5: a JSON object containing an array of
    // engine MatchLogs plus a thin envelope.
    const logs: unknown[] = [];
    for (const id of matchIds) {
      const m = await this.getMatch(id);
      if (m) {
        try {
          logs.push(JSON.parse(m.matchLogJson));
        } catch {
          // Skip corrupted logs rather than aborting the whole bundle.
        }
      }
    }
    const envelope = {
      exported_at_unix_ms: Date.now(),
      schema: "boardgame-bundle-v1",
      logs,
    };
    return JSON.stringify(envelope);
  }
}
