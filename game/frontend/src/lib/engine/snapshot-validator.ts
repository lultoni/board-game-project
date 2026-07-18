// Shared trust gate for every place that ingests external snapshot or
// MatchLog JSON before handing it to `eng.restoreFromSnapshot`. Centralises
// the size/shape/range checks that were previously absent (joiner accepted
// host JSON with only a `typeof start_fen === "string"` check; the inspector
// iterated unbounded `plies[]` from clipboard paste; etc.).
//
// The validator is intentionally permissive on `config` shape and `start_fen`
// content - those are engine-arbitrated. It is strict on:
//   - the JSON envelope size,
//   - the actions array length,
//   - per-action u32 range,
//   - top-level fields being the right primitive kinds.
//
// Callers throw / catch by source so the UI banner can map a typed reason
// onto a localised string.

export type SnapshotSource =
  | "host-snapshot"      // joiner inbound `snapshot` frame
  | "phase-change"       // joiner inbound `phase-change` frame
  | "joiner-paste"       // inspector paste box
  | "library-handoff"    // sessionStorage cross-route handoff
  | "idb-resume"         // host rejoin from persisted MatchLog
  | "sandbox-restore";   // sandbox exit (trusted, but cheap to validate)

export type SnapshotValidationReason =
  | "not-a-string"
  | "too-large"
  | "malformed-json"
  | "missing-start-fen"
  | "missing-config"
  | "actions-not-array"
  | "actions-too-many"
  | "action-malformed"
  | "plies-not-array"
  | "plies-too-many"
  | "ply-malformed";

export class SnapshotValidationError extends Error {
  readonly source: SnapshotSource;
  readonly reason: SnapshotValidationReason;
  constructor(source: SnapshotSource, reason: SnapshotValidationReason, detail?: string) {
    super(detail ? `${reason}: ${detail}` : reason);
    this.name = "SnapshotValidationError";
    this.source = source;
    this.reason = reason;
  }
}

export interface SnapshotValidationOpts {
  /** Hard cap on `actions[].length` after parsing. Suggested: 4096 for resume,
   *  1024 for paste. */
  maxActions: number;
  /** Hard cap on the input string length (bytes ~= chars for JSON). Suggested
   *  4 MiB. The engine `Snapshot` JSON for a 200-ply Stack-M match is well
   *  under 128 KiB; the cap is to refuse pathological inputs, not to police
   *  legitimate ones. */
  maxJsonBytes: number;
  /** Require `config` to be present (object). Joiner snapshots and library
   *  handoffs MUST carry it. Fresh paste can fall back to defaults. */
  requireConfig: boolean;
  source: SnapshotSource;
}

/** Validated engine `Snapshot` JSON. The `json` field is the ORIGINAL input
 *  string, not a re-serialised copy - `restoreFromSnapshot` consumes the same
 *  bytes the host or peer produced, so the engine doesn't see a different
 *  shape than what we audited. */
export interface ValidatedSnapshot {
  json: string;
  actionCount: number;
}

/** Validate an engine `Snapshot` JSON envelope: `{ start_fen, actions, config }`.
 *  Used by every site that calls `eng.restoreFromSnapshot` with bytes that
 *  could have crossed a trust boundary. */
export function validateSnapshot(raw: unknown, opts: SnapshotValidationOpts): ValidatedSnapshot {
  if (typeof raw !== "string") {
    throw new SnapshotValidationError(opts.source, "not-a-string");
  }
  if (raw.length > opts.maxJsonBytes) {
    throw new SnapshotValidationError(opts.source, "too-large", `${raw.length} > ${opts.maxJsonBytes}`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (e) {
    throw new SnapshotValidationError(opts.source, "malformed-json", (e as Error).message);
  }
  if (!parsed || typeof parsed !== "object") {
    throw new SnapshotValidationError(opts.source, "missing-start-fen");
  }
  const obj = parsed as { start_fen?: unknown; actions?: unknown; config?: unknown };
  if (typeof obj.start_fen !== "string" || obj.start_fen.length === 0) {
    throw new SnapshotValidationError(opts.source, "missing-start-fen");
  }
  if (opts.requireConfig && (obj.config === undefined || obj.config === null || typeof obj.config !== "object")) {
    throw new SnapshotValidationError(opts.source, "missing-config");
  }
  let actionCount = 0;
  if (obj.actions !== undefined && obj.actions !== null) {
    if (!Array.isArray(obj.actions)) {
      throw new SnapshotValidationError(opts.source, "actions-not-array");
    }
    if (obj.actions.length > opts.maxActions) {
      throw new SnapshotValidationError(opts.source, "actions-too-many", `${obj.actions.length} > ${opts.maxActions}`);
    }
    for (let i = 0; i < obj.actions.length; i++) {
      const a = obj.actions[i];
      if (!Number.isInteger(a) || (a as number) < 0 || (a as number) > 0xffffffff) {
        throw new SnapshotValidationError(opts.source, "action-malformed", `actions[${i}]=${String(a)}`);
      }
    }
    actionCount = obj.actions.length;
  }
  return { json: raw, actionCount };
}

/** Validate a `MatchLog` JSON envelope: `{ start_fen, config, plies: [{action:{raw}}] }`.
 *  Used by the inspector / replay paths that consume persisted match logs
 *  rather than engine snapshots. Output is shape-checked but NOT converted -
 *  callers still pass the original string into `snapshotJsonFromMatchLog` or
 *  similar to derive the engine-shaped Snapshot. */
export function validateMatchLog(raw: unknown, opts: SnapshotValidationOpts): ValidatedSnapshot {
  if (typeof raw !== "string") {
    throw new SnapshotValidationError(opts.source, "not-a-string");
  }
  if (raw.length > opts.maxJsonBytes) {
    throw new SnapshotValidationError(opts.source, "too-large", `${raw.length} > ${opts.maxJsonBytes}`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (e) {
    throw new SnapshotValidationError(opts.source, "malformed-json", (e as Error).message);
  }
  if (!parsed || typeof parsed !== "object") {
    throw new SnapshotValidationError(opts.source, "missing-start-fen");
  }
  const obj = parsed as { start_fen?: unknown; config?: unknown; plies?: unknown };
  if (typeof obj.start_fen !== "string" || obj.start_fen.length === 0) {
    throw new SnapshotValidationError(opts.source, "missing-start-fen");
  }
  if (opts.requireConfig && (obj.config === undefined || obj.config === null || typeof obj.config !== "object")) {
    throw new SnapshotValidationError(opts.source, "missing-config");
  }
  let plyCount = 0;
  if (obj.plies !== undefined && obj.plies !== null) {
    if (!Array.isArray(obj.plies)) {
      throw new SnapshotValidationError(opts.source, "plies-not-array");
    }
    if (obj.plies.length > opts.maxActions) {
      throw new SnapshotValidationError(opts.source, "plies-too-many", `${obj.plies.length} > ${opts.maxActions}`);
    }
    for (let i = 0; i < obj.plies.length; i++) {
      const ply = obj.plies[i] as { action?: { raw?: unknown } } | null | undefined;
      const raw = ply?.action?.raw;
      if (!Number.isInteger(raw) || (raw as number) < 0 || (raw as number) > 0xffffffff) {
        throw new SnapshotValidationError(opts.source, "ply-malformed", `plies[${i}]`);
      }
    }
    plyCount = obj.plies.length;
  }
  return { json: raw, actionCount: plyCount };
}

/** Default budgets, exported so callers don't sprinkle magic numbers. */
export const SNAPSHOT_BUDGETS = {
  /** Resume from IDB or host snapshot - a real match can be hundreds of plies. */
  RESUME_MAX_ACTIONS: 4096,
  /** Paste / library handoff - same upper bound, room to grow. */
  PASTE_MAX_ACTIONS: 4096,
  /** 4 MiB. Engine snapshots are tiny; this catches pathological clipboard
   *  payloads without limiting legitimate ones. */
  MAX_JSON_BYTES: 4 * 1024 * 1024,
} as const;
