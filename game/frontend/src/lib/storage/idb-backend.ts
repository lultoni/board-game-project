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
  JoinedCodeEntry,
  MatchFilter,
  MatchMeta,
  MatchStatus,
  PlyEntry,
  SavedLoadout,
  TelemetryStore,
} from "./types";
import { newMatchId } from "./types";
import type { MatchMode } from "../state/match-store.svelte";

// Database name bumped from `boardgame-matches` (v1) to `boardgame-matches-v2`
// for the L7c authoritative-host redesign. Old data is orphaned - pre-release
// project, no migration needed. See `.claude/plans/twinkling-questing-quiche.md`.
//
// DB_VERSION bumped 1 → 2 to re-run `onupgradeneeded` on browsers that opened
// `boardgame-matches-v2` from a build that predated the joined_codes store.
// The handler's `if (!objectStoreNames.contains(...))` guards make the upgrade
// idempotent - existing matches/plies are untouched; the missing joined_codes
// store gets created. Fresh DBs at v2 get all three stores in one pass.
//
// DB_VERSION bumped 2 → 3 for Task 8 (custom loadouts). Adds the `loadouts`
// store; existing three stores are untouched by the same guarded upgrade.
const DB_NAME = "boardgame-matches-v2";
const DB_VERSION = 3;
const STORE_MATCHES = "matches";
const STORE_PLIES = "plies";
// Joiner-side record of multiplayer codes the user has connected to. Joiners
// do NOT write `matches` rows in the v2 model (host owns the single row), so
// they need a separate place to remember "I joined code 281947 yesterday;
// show me a Rejoin card for it". Keyed by `code`.
const STORE_JOINED_CODES = "joined_codes";
// User-authored custom loadouts. Keyed by ULID. Separate store so listing
// loadouts (fast, small) doesn't touch matches/plies.
const STORE_LOADOUTS = "loadouts";

interface MatchRow extends MatchMeta {
  // finalise fields, present only once status==="ended"
  endedAtUnixMs?: number;
  endReason?: EndReason;
  resultByte?: 0 | 1 | 2 | 3;
  matchLogJson?: string;
  totalPlies?: number;
  totalWallMs?: number;
  // Resume snapshot — engine state JSON written after every ply for local
  // in-progress games. Cleared (set to undefined) on finalise.
  resumeSnapshotJson?: string;
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
      if (!db.objectStoreNames.contains(STORE_JOINED_CODES)) {
        const s = db.createObjectStore(STORE_JOINED_CODES, { keyPath: "code" });
        s.createIndex("lastJoinedAt", "lastJoinedAtUnixMs", { unique: false });
      }
      if (!db.objectStoreNames.contains(STORE_LOADOUTS)) {
        const s = db.createObjectStore(STORE_LOADOUTS, { keyPath: "id" });
        s.createIndex("createdAt", "createdAt", { unique: false });
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
 *  row - `finalizeMatch` always sets them; `markAbandoned`/`markNetworkLost`
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
   *  so `deleteDatabase` can complete. Production code rarely needs it -
   *  the connection lives for the app's lifetime. */
  async close(): Promise<void> {
    const db = await this.#dbPromise;
    db.close();
  }

  async startMatch(
    meta: { mode: MatchMode; multiplayerCode?: string | null; multiplayerRole?: "host" | "joiner" | null },
    nowMs?: number,
  ): Promise<string> {
    const db = await this.#dbPromise;
    const now = nowMs ?? Date.now();
    const matchId = newMatchId(now);
    const row: MatchRow = {
      matchId,
      mode: meta.mode,
      startedAtUnixMs: now,
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
      // Caller bug - finalising a match we never started. Refuse rather
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

  async checkpointMatchLog(matchId: string, matchLogJson: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing) return;
    // `ended` is terminal - `finalizeMatch` owns that row from then on. Refuse
    // writes; we should never be called post-finalise but defend against it.
    if (existing.status === "ended") return;
    const updated: MatchRow = { ...existing, matchLogJson };
    // Parse defensively to keep the indexed totals fresh - the library view
    // reads these without loading the full log.
    try {
      const parsed = JSON.parse(matchLogJson) as {
        total_plies?: number;
        total_wall_ms?: number;
      };
      if (typeof parsed.total_plies === "number") updated.totalPlies = parsed.total_plies;
      if (typeof parsed.total_wall_ms === "number") updated.totalWallMs = parsed.total_wall_ms;
    } catch {
      // Corrupt JSON shouldn't block the checkpoint; the log itself was
      // produced by the engine so this should never happen in practice.
    }
    store.put(updated);
    await awaitTx(tx);
  }

  async updateMultiplayerRole(matchId: string, role: "host" | "joiner"): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing) return;
    if (existing.multiplayerRole === role) return;
    store.put({ ...existing, multiplayerRole: role });
    await awaitTx(tx);
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
    // Don't overwrite a terminal status - finalize wins over abandonment.
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

  async saveResumeSnapshot(matchId: string, snapshotJson: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readwrite");
    const store = tx.objectStore(STORE_MATCHES);
    const existing = await awaitReq<MatchRow | undefined>(store.get(matchId));
    if (!existing || existing.status === "ended") return;
    store.put({ ...existing, resumeSnapshotJson: snapshotJson });
    await awaitTx(tx);
  }

  async getResumeSnapshot(matchId: string): Promise<string | null> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_MATCHES, "readonly");
    const row = await awaitReq<MatchRow | undefined>(
      tx.objectStore(STORE_MATCHES).get(matchId),
    );
    if (!row || row.status === "ended") return null;
    return row.resumeSnapshotJson ?? null;
  }

  async bundleMatches(matchIds: string[]): Promise<{ bundle: string; skipped: string[] }> {
    // L5b will surface this as the "Send to Designer" download. Bundle
    // format matches ADR-005 L5: a JSON object containing an array of
    // engine MatchLogs plus a thin envelope. Logs that are missing or
    // unparseable are reported back to the caller via `skipped` so the
    // user sees a clear "N of M exported" message instead of a silent loss.
    const logs: unknown[] = [];
    const skipped: string[] = [];
    for (const id of matchIds) {
      const m = await this.getMatch(id);
      if (!m || !m.matchLogJson) {
        skipped.push(id);
        continue;
      }
      try {
        logs.push(JSON.parse(m.matchLogJson));
      } catch {
        skipped.push(id);
      }
    }
    const envelope = {
      exported_at_unix_ms: Date.now(),
      schema: "boardgame-bundle-v1",
      logs,
    };
    return { bundle: JSON.stringify(envelope), skipped };
  }

  async recordJoinedCode(entry: { code: string; hostPeerId?: string | null; lastSeenSeq?: number }): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_JOINED_CODES, "readwrite");
    const store = tx.objectStore(STORE_JOINED_CODES);
    const existing = await awaitReq<JoinedCodeEntry | undefined>(store.get(entry.code));
    const row: JoinedCodeEntry = {
      code: entry.code,
      lastJoinedAtUnixMs: Date.now(),
      hostPeerId: entry.hostPeerId ?? existing?.hostPeerId ?? null,
      lastSeenSeq: entry.lastSeenSeq ?? existing?.lastSeenSeq ?? 0,
    };
    store.put(row);
    await awaitTx(tx);
  }

  async listJoinedCodes(): Promise<JoinedCodeEntry[]> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_JOINED_CODES, "readonly");
    const all = await awaitReq<JoinedCodeEntry[]>(tx.objectStore(STORE_JOINED_CODES).getAll());
    await awaitTx(tx);
    return all.sort((a, b) => b.lastJoinedAtUnixMs - a.lastJoinedAtUnixMs);
  }

  async forgetJoinedCode(code: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_JOINED_CODES, "readwrite");
    tx.objectStore(STORE_JOINED_CODES).delete(code);
    await awaitTx(tx);
  }

  async saveLoadout(row: SavedLoadout): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_LOADOUTS, "readwrite");
    tx.objectStore(STORE_LOADOUTS).put(row);
    await awaitTx(tx);
  }

  async listLoadouts(): Promise<SavedLoadout[]> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_LOADOUTS, "readonly");
    const all = await awaitReq<SavedLoadout[]>(tx.objectStore(STORE_LOADOUTS).getAll());
    await awaitTx(tx);
    return all.sort((a, b) => b.createdAt - a.createdAt);
  }

  async getLoadout(id: string): Promise<SavedLoadout | null> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_LOADOUTS, "readonly");
    const row = await awaitReq<SavedLoadout | undefined>(tx.objectStore(STORE_LOADOUTS).get(id));
    await awaitTx(tx);
    return row ?? null;
  }

  async deleteLoadout(id: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_LOADOUTS, "readwrite");
    tx.objectStore(STORE_LOADOUTS).delete(id);
    await awaitTx(tx);
  }

  async updateLoadoutName(id: string, name: string): Promise<void> {
    const db = await this.#dbPromise;
    const tx = db.transaction(STORE_LOADOUTS, "readwrite");
    const store = tx.objectStore(STORE_LOADOUTS);
    const existing = await awaitReq<SavedLoadout | undefined>(store.get(id));
    if (!existing) return;
    if (existing.name === name) return;
    store.put({ ...existing, name });
    await awaitTx(tx);
  }
}
