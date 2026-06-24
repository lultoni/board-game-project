// Telemetry storage — types shared across backends.
//
// A "match" is the full record of a played game (HvH / HvAI / AIvAI / MP).
// Sandbox and Inspector are NOT logged here; they're analysis surfaces.
//
// Two-level layout: every applied ply is appended to the `plies` store
// immediately (resilience to tab close mid-match). On natural game end the
// consolidated MatchLog is written to the `matches` store with end-of-match
// metadata. Per-ply records are kept for replay / inspector use.

import type { MatchMode } from "../state/match-store.svelte";

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

/** Storage backend contract. Same surface for IDB (web) and Tauri FS
 *  (desktop). Picked at boot like the engine client. */
export interface TelemetryStore {
  startMatch(meta: Omit<MatchMeta, "matchId" | "startedAtUnixMs" | "status">): Promise<string>;
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
  /** Returns a JSON bundle of the given matches for "Send to Designer" export. */
  bundleMatches(matchIds: string[]): Promise<string>;
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
