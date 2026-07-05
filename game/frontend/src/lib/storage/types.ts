// Telemetry storage — types shared across backends.
//
// A "match" is the full record of a played game (HvH / HvAI / AIvAI / MP).
// Sandbox and Inspector are NOT logged here; they're analysis surfaces.
//
// Loadouts are custom `SideLoadout` records the user assembles in the
// `/loadouts/` route. They live in their own store keyed by ULID; the
// dedupe check operates on the skill tuple, not the display name.

import type { SideLoadout } from "$lib/engine";
//
// Two-level layout: every applied ply is appended to the `plies` store
// immediately (resilience to tab close mid-match). On natural game end the
// consolidated MatchLog is written to the `matches` store with end-of-match
// metadata. Per-ply records are kept for replay / inspector use.

/** The shape of a match's run-time mode. Lives here (not in state/) so the
 *  storage layer doesn't import upward into the rune store — the engine and
 *  storage layers are leaves with respect to state. `state/match-store`
 *  re-exports this so route code can keep its existing import path. */
export type MatchMode =
  | "idle"
  | "hvh"
  | "hvai"
  | "aivai"
  | "replay"
  | "sandbox"
  | "multiplayer";

/** End-of-match reason. Authoritative on finalised matches. */
export type EndReason =
  | "checkmate"        // Engine reported gameResult != 0 from the rules.
  | "resign"           // Player surrendered via the resign UI (not yet built).
  | "opponent_forfeit" // Multiplayer: present player claimed win after grace.
  | "abandoned";       // Match was closed/navigated-away without natural end.

/** Match status while in storage. */
export type MatchStatus =
  | "in-progress"          // Started, no finalise yet. Default after startMatch.
  | "ended"                // finalizeMatch called with a natural-end reason.
  | "abandoned"            // Navigated away from /match/ without natural end.
  | "mid-match-network-lost"; // Multiplayer: opponent dropped (L7b uses this).

/** Metadata recorded at match start. Stored separately from the engine's
 *  MatchLog so the library view can filter without parsing huge logs.
 *
 *  The end-of-match fields (`endedAtUnixMs`, `endReason`, `resultByte`,
 *  `totalPlies`) are populated only after `finalizeMatch`; the library view
 *  reads them to render result chips without loading the full MatchLog blob. */
export interface MatchMeta {
  matchId: string;            // ULID, time-sortable.
  mode: MatchMode;            // hvh | hvai | aivai | (multiplayer in L7).
  startedAtUnixMs: number;
  status: MatchStatus;
  // Multiplayer fields populated by L7; null for local modes.
  multiplayerCode?: string | null;
  multiplayerRole?: "host" | "joiner" | null;
  // End-of-match summary. Present only when status === "ended".
  endedAtUnixMs?: number;
  endReason?: EndReason;
  resultByte?: 0 | 1 | 2 | 3;
  totalPlies?: number;
}

/** The consolidated record written at finaliseMatch. The engine's MatchLog
 *  JSON is the source of truth for the move list; we just keep an indexed
 *  view of the metadata in `MatchMeta` for fast library queries. */
export interface FinalisedMatch extends MatchMeta {
  endedAtUnixMs: number;
  endReason: EndReason;
  resultByte: 0 | 1 | 2 | 3; // P1Win | P2Win | Draw | Aborted
  matchLogJson: string;       // Full engine MatchLog (consolidated).
  totalPlies: number;
  totalWallMs: number;
}

/** A single per-ply entry as recorded incrementally. Body is the raw JSON
 *  string returned by engine.latestPlyJson() — kept opaque to avoid
 *  re-parsing on the hot path. The library / replay views parse on demand. */
export interface PlyEntry {
  matchId: string;
  plyNo: number;             // 1-based, matches engine's PlyRecord.ply_no.
  recordedAtUnixMs: number;
  plyJson: string;           // Opaque: serialised core_engine::telemetry::PlyRecord.
}

/** Filter for listMatches. All fields optional and AND-combined. */
export interface MatchFilter {
  mode?: MatchMode | MatchMode[];
  status?: MatchStatus | MatchStatus[];
  startedAfterUnixMs?: number;
  startedBeforeUnixMs?: number;
}

/** A multiplayer code the user has joined as the non-host peer. Kept
 *  separately from `MatchMeta` so a peer who entered a code but never
 *  started a match (setup-phase drop) still shows up in the lobby's
 *  "recently joined" list — those sessions have no `matches` row yet.
 *  Once a match row exists for the code, the lobby prefers the row (it
 *  has phase + status info); this table is the pre-match fallback. */
export interface JoinedCodeEntry {
  code: string;
  lastJoinedAtUnixMs: number;
  /** PeerJS ID the joiner used to dial in. Kept for debugging only. */
  hostPeerId?: string | null;
  /** Most recent committed seq the joiner observed for this code. Lets the
   *  lobby decide whether to show "you were mid-game" vs. a stale code. */
  lastSeenSeq?: number;
}

/** A user-authored custom loadout, saved in the loadouts store. `loadout` is
 *  a `SideLoadout` (6 pairs of skill IDs, King @ 0 + 5 Champions). The
 *  dedupe check runs on the skill tuple only — two rows with the same
 *  skills and different names are still duplicates. */
export interface SavedLoadout {
  id: string;              // ULID, time-sortable (via `newMatchId`).
  name: string;            // Display label. Not used for dedupe.
  loadout: SideLoadout;
  createdAt: number;       // Unix ms.
}

/** Storage backend contract. Same surface for IDB (web) and Tauri FS
 *  (desktop). Picked at boot like the engine client. */
export interface TelemetryStore {
  startMatch(meta: Omit<MatchMeta, "matchId" | "startedAtUnixMs" | "status">, nowMs?: number): Promise<string>;
  appendPly(matchId: string, plyJson: string, plyNo: number): Promise<void>;
  finalizeMatch(
    matchId: string,
    matchLogJson: string,
    endReason: EndReason,
    resultByte: 0 | 1 | 2 | 3,
    totalPlies: number,
    totalWallMs: number,
  ): Promise<void>;
  /** Marks an in-progress match as abandoned; called on /match/ teardown
   *  if no natural end was observed. No-op if the match is already ended.
   *  Per-ply records are preserved. If `partialLogJson` is supplied (the
   *  engine's current MatchLog view), it's stored so the library can still
   *  open the abandoned match in the inspector or export it. */
  markAbandoned(matchId: string, partialLogJson?: string): Promise<void>;
  /** Marks an in-progress multiplayer match as mid-match-network-lost.
   *  Used by L7b's reconnect flow. `partialLogJson` semantics match
   *  markAbandoned. */
  markNetworkLost(matchId: string, partialLogJson?: string): Promise<void>;
  /** Refreshes the stored MatchLog without changing status. Called from the
   *  draft + match routes after every applied ply so the resume snapshot
   *  always reflects the engine's latest state. Distinct from
   *  `markNetworkLost` (which is a one-shot status transition that ignores
   *  writes after the first flip): this is idempotent and runs on every
   *  ply, regardless of current `status` — except `ended`, which is
   *  terminal and must not be overwritten. No-op if the row is missing. */
  checkpointMatchLog(
    matchId: string,
    matchLogJson: string,
  ): Promise<void>;
  /** Transitions a `mid-match-network-lost` row to `abandoned` (carrying any
   *  partial MatchLog through). No-op if the row is in any other state. Used
   *  by the lobby's "Dismiss" button so the row stops appearing in the
   *  recent-sessions list but stays in the library. */
  dismissNetworkLost(matchId: string): Promise<void>;
  /** Returns a match's metadata (without the full log) by ID. */
  getMatchMeta(matchId: string): Promise<MatchMeta | null>;
  /** Returns a finalised match including its consolidated log. */
  getMatch(matchId: string): Promise<FinalisedMatch | null>;
  /** Returns per-ply entries for a match in order. */
  getPlies(matchId: string): Promise<PlyEntry[]>;
  /** List match metas matching the filter, sorted by startedAt descending. */
  listMatches(filter?: MatchFilter): Promise<MatchMeta[]>;
  deleteMatch(matchId: string): Promise<void>;
  /** Returns a JSON bundle of the given matches for "Send to Designer" export.
   *  Returns the JSON string and a list of match IDs that were skipped because
   *  their stored matchLogJson was missing or unparseable. Callers surface the
   *  skip list to the user (e.g. a toast) so silent data loss is visible. */
  bundleMatches(matchIds: string[]): Promise<{ bundle: string; skipped: string[] }>;

  // --- joiner-side multiplayer code memory (v2 redesign) -------------------
  /** Record (or refresh `lastJoinedAtUnixMs` on) a code the user joined as
   *  the non-host peer. Idempotent on `code` — re-recording updates the
   *  timestamp and any newer fields. */
  recordJoinedCode(entry: { code: string; hostPeerId?: string | null; lastSeenSeq?: number }): Promise<void>;
  /** All codes the user has joined, most-recent first. */
  listJoinedCodes(): Promise<JoinedCodeEntry[]>;
  /** Remove a code from the joiner's resume-card list. Used by the "Dismiss"
   *  affordance on a recent-codes card. No-op if the code isn't recorded. */
  forgetJoinedCode(code: string): Promise<void>;

  /** Update the `multiplayerRole` on an existing matches row in place. Used
   *  when a joiner is promoted to host (in-tab takeover or lobby re-entry
   *  with an empty host slot): the row's log is authoritative, only the
   *  role changes. No-op if the row is missing. */
  updateMultiplayerRole(matchId: string, role: "host" | "joiner"): Promise<void>;

  // --- custom loadouts store (Task 8) ---------------------------------------
  /** Persist a new custom loadout row. Caller supplies the full `SavedLoadout`
   *  including a freshly-minted ULID (`newMatchId()` reused). No dedupe here —
   *  the caller runs `findDuplicate` against `listLoadouts()` first and either
   *  disables the save button (manual save) or skips-and-reports (import). */
  saveLoadout(row: SavedLoadout): Promise<void>;
  /** All saved custom loadouts, sorted `createdAt` descending. */
  listLoadouts(): Promise<SavedLoadout[]>;
  /** Fetch one loadout by ID, or null if it doesn't exist. */
  getLoadout(id: string): Promise<SavedLoadout | null>;
  /** Remove a loadout by ID. No-op if the row is missing. */
  deleteLoadout(id: string): Promise<void>;
  /** Rename a loadout in place. No-op if the row is missing. Name is a
   *  display label only — dedupe is on the skill tuple, so a rename can
   *  never introduce or resolve a duplicate. */
  updateLoadoutName(id: string, name: string): Promise<void>;
}

// --- ULID generation -------------------------------------------------------
//
// ULID (https://github.com/ulid/spec): 26-char Crockford-base32, time-sortable.
// First 10 chars are 48-bit ms timestamp, next 16 chars are 80-bit randomness.
// Lex-sortable matches chronological order — exactly what the library view
// wants ("most recent first" is `.reverse()` of a sorted key list).

const CROCKFORD32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

function randomBytes(n: number): Uint8Array {
  const a = new Uint8Array(n);
  // Both browsers and Node 19+ expose crypto.getRandomValues globally.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const c: any = (globalThis as any).crypto;
  if (c?.getRandomValues) {
    c.getRandomValues(a);
  } else {
    for (let i = 0; i < n; i++) a[i] = Math.floor(Math.random() * 256);
  }
  return a;
}

export function newMatchId(nowMs: number = Date.now()): string {
  // 10-char time part: encode 48 bits MSB-first.
  let time = nowMs;
  const timeChars: string[] = new Array(10);
  for (let i = 9; i >= 0; i--) {
    timeChars[i] = CROCKFORD32[time & 31];
    time = Math.floor(time / 32);
  }
  // 16-char random part: 80 bits = 10 bytes → 16 base32 chars.
  const rnd = randomBytes(10);
  // Pack the 10 bytes (80 bits) into a 16-char base32 string. Read 5 bits
  // at a time from the most significant end.
  const rndChars: string[] = new Array(16);
  let bitBuf = 0;
  let bitCount = 0;
  let outIdx = 0;
  for (let i = 0; i < 10; i++) {
    bitBuf = (bitBuf << 8) | rnd[i];
    bitCount += 8;
    while (bitCount >= 5) {
      bitCount -= 5;
      rndChars[outIdx++] = CROCKFORD32[(bitBuf >> bitCount) & 31];
    }
  }
  return timeChars.join("") + rndChars.join("");
}
