// Shared ply renderer driver.
//
// Owns the producer side of the visual-effects pipeline that used to live
// inside `match/+page.svelte`. Both `/match/` and `/replay/` (and eventually
// `/inspector/`) create one of these and feed actions through `applyAndRender`
// (or `renderApplied` for the MP non-acting-peer path). The driver:
//
//   * holds the current rendered `position` + `legal` (or writes through to a
//     supplied carrier - see `positionSink`),
//   * tracks `pieceIds` so Board's `{#each}` key stays stable across moves and
//     CSS slide transitions actually run,
//   * tracks `shakingSquares` for hit-shake animation,
//   * owns the `effectQueue` consumed by EffectsLayer,
//   * defers the post-skill state flip by RELOC_DELAY_MS so impact effects
//     land on the pre-state board before pieces visually relocate,
//   * gates SFX behind a per-instance `sfxEnabled` toggle so replay/inspector
//     can render silently without forking the producer logic.
//
// Adding a new effect (sound or visual) is a one-place edit: this file.

import {
  ActionKind,
  decodeAction,
  decodeMailbox,
  isBodyguardChoice,
  bgGuardIdx,
  isDraftTurn,
  type EngineClient,
  type PositionView,
  type PendingBodyguardView,
} from "$lib/engine";
import type { Effect } from "$lib/viz/effects";
import { FX_LIFETIME_MS } from "$lib/viz/effects";
import { skillColor, SKILLS } from "$lib/engine/skills";
import { sfx } from "$lib/audio/sfx";
import { slideDurationMs, fxSpeedMultiplier } from "$lib/state/settings.svelte";

// === Public types ==========================================================

export type BodyguardSnapshot = {
  sq: number;
  entry: ReturnType<typeof decodeMailbox>;
}[];

export interface PreStateSnapshot {
  preFull: Uint16Array | null;
  preTarget: ReturnType<typeof decodeMailbox> | null;
  preBodyguard: BodyguardSnapshot;
  /** P1 money at pre-state. Sampled so the Skill emission phase can tell
   *  whether Steal actually moved cash (target had ≥1) and suppress the
   *  coin-return glyph when nothing was pilfered. */
  preP1Money: number;
  preP2Money: number;
  /** Engine `pendingBodyguard` state at the pre-state of a BodyguardChoice
   *  ply - the attacker/target/eligible squares cached between the tentative
   *  Move-Attack and the defender's choice. Non-null only for choice plies;
   *  the choice-ply renderer reads this to animate the intercept/decline. */
  preBodyguardPending: PendingBodyguardView | null;
}

/** A `$state`-backed carrier the renderer writes through to. Match passes
 *  the global `match` store here; replay/inspector leave it unset and read
 *  the renderer's own `position`/`legal` instead. */
export interface PositionSink {
  position: PositionView | null;
  legal: Uint32Array;
}

/** Opaque timer handle. Browser's `setTimeout` returns `number`; Node's
 *  returns an object - we don't care which, only that callers thread it
 *  back through `clearTimeout`. */
export type TimerHandle = ReturnType<typeof globalThis.setTimeout>;

export interface PlyRendererScheduler {
  setTimeout: (cb: () => void, ms: number) => TimerHandle;
  clearTimeout: (h: TimerHandle) => void;
}

export interface PlyRendererOpts {
  /** When set, every post-apply state flip writes here instead of the
   *  driver-local position/legal. The driver's getters still surface the
   *  same values. */
  positionSink?: PositionSink;
  /** SFX gate. `false` (or a thunk returning false) silences `sfx.play` calls
   *  for this renderer instance. Default: enabled. */
  sfxEnabled?: boolean | (() => boolean);
  /** Optional callback invoked after a Move action lands. Caller uses this
   *  to update its `usedThisPhase` Set for the greying-out UI. */
  onMoveLanding?: (finalSq: number) => void;
  /** Clock source for effect timestamps. Defaults to `performance`. Tests
   *  pass a fake to make effect ordering deterministic. */
  clock?: { now(): number };
  /** SFX dispatch surface. Defaults to the imported `sfx` singleton. Tests
   *  pass a fake that records `.play()` calls. */
  sfxImpl?: Pick<typeof sfx, "play">;
  /** Warning hook for non-fatal anomalies. Defaults to `console.warn`. */
  warn?: (stage: string, detail?: unknown) => void;
  /** Timer surface. Defaults to global `setTimeout`/`clearTimeout`. Tests
   *  inject a fake to fire timers synchronously and assert pending count
   *  after `dispose()`. */
  scheduler?: PlyRendererScheduler;
}

export interface ApplyAndRenderOpts {
  /** Ply index (0-based) of the ply being applied, relative to a stable base
   *  snapshot. When provided AND the index is a multiple of `CHECKPOINT_STRIDE`,
   *  the renderer captures `eng.snapshotJson()` so subsequent `fastForwardTo`
   *  scrubs to nearby plies can restore from this checkpoint instead of
   *  replaying from ply 0. Replay's `stepForward` passes `currentPly` here;
   *  match/draft/inspector don't scrub and omit this, which keeps the
   *  checkpoint cache empty (and free). */
  plyHint?: number;
  /** Base snapshot the `plyHint` is relative to. Required when `plyHint` is
   *  set so the captured checkpoint can be invalidated if the base changes
   *  (e.g. a different match log is loaded). */
  plyHintBase?: string;
}

/** Descriptor for a piece's multi-hop walk. Consumed by Piece.svelte via the
 *  `motion` prop and rendered with a WAAPI keyframe animation. Keyed in the
 *  renderer's `pieceMotion` map by the piece's FINAL square (where the DOM
 *  element rests after the state flip).
 *
 *  `waypoints` is the ordered list of squares the piece walks through,
 *  inclusive of endpoints: `[src, mid1, ..., finalWalkSq]`. Each hop lasts
 *  `slideDurationMs()`, with a small Y-bounce injected at each waypoint
 *  arrival so the walk reads as "stepping" rather than a glide.
 *
 *  When `killLungeTo` is set, the piece follows the walk with an extra lunge
 *  segment from the last waypoint into the given square and stays there. Used
 *  for killing move-attacks where the attacker ends up on the dead enemy's
 *  square (approach ≠ target case) or a direct kill (approach === target).
 *
 *  When `lungeReturnTo` is set (mutually exclusive with `killLungeTo`), the
 *  piece leans ~55% of the way toward the given square then RETURNS to the
 *  last waypoint (its resting square). This is the non-kill move-attack lunge
 *  and the bodyguard intercept - a jab that recoils, driven through the same
 *  WAAPI channel as the walk/kill so it plays and replays reliably. */
export interface PieceMotion {
  waypoints: number[];
  killLungeTo: number | null;
  /** Square to lunge toward and recoil from (non-kill attack / intercept).
   *  Mutually exclusive with `killLungeTo`. Final frame returns to the last
   *  waypoint. */
  lungeReturnTo: number | null;
  startedAt: number;
  /** Number of hops = waypoints.length - 1 (0 when a piece never moves). Kept
   *  as a separate field so consumers don't recompute; also drives the total
   *  animation duration (hops × slide + optional lunge segment). */
  hops: number;
}

export interface PlyRenderer {
  // Reactive state - bind into Board / EffectsLayer.
  readonly position: PositionView | null;
  readonly legal: Uint32Array;
  readonly pieceIds: Map<number, number>;
  readonly shakingSquares: Set<number>;
  /** Walk descriptors keyed by the piece's FINAL square. Board.svelte passes
   *  the entry matching each rendered piece into `<Piece motion={...}>`. Only
   *  populated for Move actions; cleared when the animation finishes. */
  readonly pieceMotion: Map<number, PieceMotion>;
  readonly effectQueue: Effect[];
  readonly lastApplied: { src: number; target: number } | null;

  /** Resolves when the currently-emitted piece motion (if any) has finished.
   *  Used by match / replay to gate the next ply behind `respectAnimation`.
   *  When no motion is active, resolves immediately. */
  animationDone(): Promise<void>;

  // Lifecycle.
  /** Capture pre-state from the currently-rendered position, run `applyFn`
   *  (which is expected to advance the engine - caller's choice whether
   *  that's `eng.tryApply` directly or via an MP wrapper), then emit effects
   *  + SFX and flip rendered state. Any pending deferred refresh is drained
   *  synchronously at entry. */
  applyAndRender(
    raw: number,
    applyFn: () => Promise<void>,
    opts?: ApplyAndRenderOpts,
  ): Promise<void>;

  /** Caller-already-applied variant: the engine has already advanced (e.g.
   *  via the MP wrapper's onApplied callback). Caller passes the pre-state
   *  it captured before the engine moved. */
  renderApplied(raw: number, pre: PreStateSnapshot): Promise<void>;

  /** Capture a pre-state snapshot. If `prePosition` is supplied (e.g. by the
   *  MP wrapper, which captures it explicitly before `tryApply`), use that
   *  view; otherwise fall back to the renderer's currently-rendered position
   *  (the legacy path used by the AI step and the renderer's internal
   *  `applyAndRender`). */
  snapshotPreState(raw: number, prePosition?: PositionView): PreStateSnapshot;

  /** Drain any deferred skill-refresh synchronously. Idempotent. Safe to
   *  call defensively before any operation that mutates engine state. */
  drainPendingSkillRefresh(): void;

  /** Cancel all outstanding timers (shake resets, deferred skill refresh)
   *  and clear visual state. Safe to call multiple times. Routes/tests call
   *  this on teardown to ensure no timer fires against a torn-down store.
   *  Does NOT touch the engine. */
  dispose(): void;

  /** Silent fast-forward used by replay scrubbing. Restores from
   *  `baseSnapshotJson`, replays plies 0..target-1 silently (no effects, no
   *  SFX), then runs the full effect pipeline for plies[target-1] so the
   *  landing ply shows the impact pulse. `target === 0` just restores. */
  fastForwardTo(
    baseSnapshotJson: string,
    plies: number[],
    target: number,
  ): Promise<void>;

  /** Pull fresh position+legal+pieceIds from the engine. Used after a
   *  snapshot restore (sandbox enter/exit, MP onSnapshotApplied) or on
   *  initial boot. Also resets effectQueue / shakingSquares / deferred
   *  refresh - call this when crossing match boundaries. */
  resyncFromEngine(): Promise<void>;

  /** Hard reset: clear pieceIds, effectQueue, shakingSquares, drain pending
   *  refresh. Does NOT touch the engine. */
  reset(): void;
}

// === Internal constants ====================================================

/** ms to wait between impact effects landing on the pre-state board and the
 *  visual relocation/death sweep. Tuned to feel like an "earthquake settles"
 *  rather than a teleport. */
const RELOC_DELAY_MS = 260;

const SHAKE_DURATION_MS = 340;

/** Ply checkpoint stride for `fastForwardTo`. Every N plies during a silent
 *  fast-forward, we capture `eng.snapshotJson()`; subsequent scrubs near the
 *  same range restore from the nearest checkpoint instead of replaying from
 *  ply 0. Replay's autoplay path also drops a checkpoint here via the
 *  optional `plyHint` on `applyAndRender`, so re-scrubbing a region already
 *  walked through forward play is cheap. */
const CHECKPOINT_STRIDE = 32;

/** Minimum number of plies a checkpoint must save to be worth the extra
 *  `restoreFromSnapshot` round-trip. Below this, we just fall through to the
 *  naive replay-from-base path. */
const CHECKPOINT_MIN_SAVING = 4;

// === Internal helpers (module-private) =====================================

/** Wait one animation frame so the browser can paint the current DOM state.
 *  Used to split a pieceIds write (which repositions the stable DOM element
 *  to its old square key) from the position write (which drives the new
 *  coordinates), giving the CSS transition a visible "before" state to animate
 *  from. Without this gap both writes land in the same Svelte flush and the
 *  transition never fires.
 *  In non-browser environments (tests) falls back to a zero-ms setTimeout so
 *  the scheduler seam can intercept it. */
function waitFrame(): Promise<void> {
  if (typeof requestAnimationFrame !== "undefined") {
    return new Promise((resolve) => requestAnimationFrame(() => resolve()));
  }
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function chebyshev(a: number, b: number): number {
  const dx = Math.abs((a & 7) - (b & 7));
  const dy = Math.abs(((a >> 3) & 7) - ((b >> 3) & 7));
  return Math.max(dx, dy);
}

/** Straight-line path of squares between `from` and `to` along a queen-ray
 *  direction (inclusive of endpoints). Returns [from, to] as a fallback when
 *  the pair isn't on a clean ray. */
function straightPath(from: number, to: number): number[] {
  const fF = from & 7, fR = (from >> 3) & 7;
  const tF = to & 7, tR = (to >> 3) & 7;
  const dF = Math.sign(tF - fF);
  const dR = Math.sign(tR - fR);
  const steps = Math.max(Math.abs(tF - fF), Math.abs(tR - fR));
  if (steps === 0) return [from];
  const okF = dF === 0 || Math.abs(tF - fF) === steps;
  const okR = dR === 0 || Math.abs(tR - fR) === steps;
  if (!okF || !okR) return [from, to];
  const out: number[] = [];
  for (let i = 0; i <= steps; i++) {
    const f = fF + dF * i;
    const r = fR + dR * i;
    out.push((r << 3) | f);
  }
  return out;
}

/** Walked path for a Move action's dust trail. */
function walkedPath(
  decoded: ReturnType<typeof decodeAction>,
  killed: boolean,
): number[] {
  if (decoded.kind !== ActionKind.Move) return [];
  const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
  if (decoded.hasAux && killed) {
    if (approach === decoded.src) return [decoded.src, decoded.target];
    return [decoded.src, approach, decoded.target];
  }
  if (approach === decoded.src) return [decoded.src];
  return [decoded.src, approach];
}

/** Pick a chebyshev-midpoint square for a speed-2 zigzag move. Engine BFS may
 *  route around a blocker; we don't have that path so we pick a visually
 *  plausible corner. Prefer a candidate that is empty in the pre-mailbox so
 *  the walking piece doesn't visibly overlap another. If both candidates are
 *  occupied (or the pair isn't 2-apart), returns null. */
function chebyshevMidpoint(
  from: number,
  to: number,
  preMailbox: Uint16Array | null,
): number | null {
  const fF = from & 7, fR = (from >> 3) & 7;
  const tF = to & 7, tR = (to >> 3) & 7;
  const dxTotal = tF - fF;
  const dyTotal = tR - fR;
  if (Math.max(Math.abs(dxTotal), Math.abs(dyTotal)) !== 2) return null;
  // Straight 2-step (0/±2 in one axis, 0 in the other): unique midpoint.
  if (dxTotal === 0 || dyTotal === 0) {
    const mf = fF + Math.sign(dxTotal);
    const mr = fR + Math.sign(dyTotal);
    return (mr << 3) | mf;
  }
  // Diagonal 2-step (±2, ±2): unique midpoint on the diagonal.
  if (Math.abs(dxTotal) === 2 && Math.abs(dyTotal) === 2) {
    const mf = fF + Math.sign(dxTotal);
    const mr = fR + Math.sign(dyTotal);
    return (mr << 3) | mf;
  }
  // L-shape (2/±1 or 1/±2): two candidates - the corner via file first, or
  // via rank first. Prefer the empty one; fall back to file-first.
  const c1F = fF + Math.sign(dxTotal) * Math.min(1, Math.abs(dxTotal));
  const c1R = fR + (Math.abs(dyTotal) === 2 ? Math.sign(dyTotal) : 0);
  const c2F = fF + (Math.abs(dxTotal) === 2 ? Math.sign(dxTotal) : 0);
  const c2R = fR + Math.sign(dyTotal) * Math.min(1, Math.abs(dyTotal));
  const cand1 = (c1R << 3) | c1F;
  const cand2 = (c2R << 3) | c2F;
  if (preMailbox) {
    const e1 = decodeMailbox(preMailbox[cand1]).empty;
    const e2 = decodeMailbox(preMailbox[cand2]).empty;
    if (e1 && !e2) return cand1;
    if (e2 && !e1) return cand2;
  }
  return cand1;
}

interface SkillDiff {
  stayed: number[];
  moves: { from: number; to: number; dist: number }[];
  deaths: number[];
}

function diffSkillMailbox(pre: Uint16Array, post: Uint16Array): SkillDiff {
  const stayed: number[] = [];
  const vacated: number[] = [];
  const arrived: number[] = [];
  for (let sq = 0; sq < 64; sq++) {
    const a = decodeMailbox(pre[sq]);
    const b = decodeMailbox(post[sq]);
    if (a.empty && b.empty) continue;
    if (!a.empty && !b.empty) { stayed.push(sq); continue; }
    if (!a.empty && b.empty) { vacated.push(sq); continue; }
    if (a.empty && !b.empty) { arrived.push(sq); continue; }
  }
  const moves: { from: number; to: number; dist: number }[] = [];
  const usedV = new Set<number>();
  for (const dst of arrived) {
    let bestV = -1;
    let bestD = Infinity;
    for (const v of vacated) {
      if (usedV.has(v)) continue;
      const d = chebyshev(v, dst);
      if (d < bestD) { bestD = d; bestV = v; }
    }
    if (bestV >= 0) {
      usedV.add(bestV);
      moves.push({ from: bestV, to: dst, dist: bestD });
    }
  }
  const deaths: number[] = vacated.filter((v) => !usedV.has(v));
  return { stayed, moves, deaths };
}

function hasRelocationOrDeath(diff: SkillDiff): boolean {
  return diff.moves.length > 0 || diff.deaths.length > 0;
}

/** One signum step from `from` toward `to` along their shared queen-ray, or
 *  null when the pair isn't on a ray (orthogonal or diagonal) or coincides.
 *  Mirrors the engine's `step_toward` (state/magic.rs) so the renderer can
 *  reconstruct the Stack N strike-moves-caster forced step. */
function stepToward(from: number, to: number): number | null {
  const fF = from & 7, fR = (from >> 3) & 7;
  const tF = to & 7, tR = (to >> 3) & 7;
  const dF = Math.sign(tF - fF);
  const dR = Math.sign(tR - fR);
  if (dF === 0 && dR === 0) return null;
  const adF = Math.abs(tF - fF), adR = Math.abs(tR - fR);
  const onRay = dF === 0 || dR === 0 || adF === adR;
  if (!onRay) return null;
  return ((fR + dR) << 3) | (fF + dF);
}

/** Stack N strike-moves-caster: after a Strike resolves, the engine steps the
 *  caster 1 tile toward the (former) target iff that tile is empty. The mailbox
 *  diff is piece-identity-blind, so it mispairs the caster's forced step with
 *  the target's death/HP change (the "heal lands on the destination, damage on
 *  the origin" glitch). Reconstruct that KNOWN step and pull it out of the
 *  anonymous diff so it renders as a pure relocation instead.
 *
 *  Caster moved iff, in the FRESH post mailbox, `src` is now empty AND the
 *  computed `dest` is occupied. Mutates `diff` in place: drops `src` from
 *  `deaths`, any `move` whose `from === src`, and `dest` from `stayed` (the
 *  point-blank-kill collision where the caster stepped onto the victim's tile).
 *  Returns the extracted caster move, or null when the caster didn't step. */
function extractCasterMove(
  decoded: ReturnType<typeof decodeAction>,
  post: Uint16Array,
  diff: SkillDiff,
): { from: number; to: number; dist: number } | null {
  if (decoded.kind !== ActionKind.Skill) return null;
  if (SKILLS[decoded.skillId]?.category !== "strike") return null;
  const src = decoded.src;
  const dest = stepToward(src, decoded.target);
  if (dest === null) return null;
  if (!decodeMailbox(post[src]).empty) return null;   // caster didn't leave src
  if (decodeMailbox(post[dest]).empty) return null;    // dest empty → no step
  diff.deaths = diff.deaths.filter((v) => v !== src);
  diff.moves = diff.moves.filter((m) => m.from !== src);
  diff.stayed = diff.stayed.filter((s) => s !== dest);
  return { from: src, to: dest, dist: chebyshev(src, dest) };
}

// === Factory ===============================================================

export function createPlyRenderer(
  eng: EngineClient,
  opts: PlyRendererOpts = {},
): PlyRenderer {
  // === Resolve injected seams (defaults preserve prod behaviour) ===========
  const clock = opts.clock ?? { now: () => performance.now() };
  const sfxImpl: Pick<typeof sfx, "play"> = opts.sfxImpl ?? sfx;
  const warn = opts.warn ?? ((stage: string, detail?: unknown) => {
    console.warn(`ply-renderer:${stage}`, detail);
  });
  const scheduler: PlyRendererScheduler = opts.scheduler ?? {
    setTimeout: (cb, ms) => globalThis.setTimeout(cb, ms),
    clearTimeout: (h) => globalThis.clearTimeout(h),
  };

  // Tracks every outstanding scheduler timer (shake resets + pending skill
  // refresh). `dispose()` clears all of these. Without this set, P2 stood:
  // a shake setTimeout fired after `reset()` would write into the cleared
  // shakingSquares Set.
  const timers = new Set<TimerHandle>();

  // === Checkpoint cache ====================================================
  // Sparse cache of `eng.snapshotJson()` strings keyed by ply index. Read by
  // `fastForwardTo` to skip the leading silent replay; written during
  // fast-forward AND during `applyAndRender` when the caller passes a
  // `plyHint`. Cleared on `reset()` and whenever the base snapshot changes
  // (different match log loaded, replay restarted at ply 0 from a new log).
  const checkpoints = new Map<number, string>();
  let checkpointsBase: string | null = null;

  function invalidateCheckpointsIfBaseChanged(nextBase: string | null): void {
    if (nextBase === checkpointsBase) return;
    checkpoints.clear();
    checkpointsBase = nextBase;
  }

  function nearestCheckpoint(target: number): { ply: number; snap: string } | null {
    if (target <= 0) return null;
    let bestPly = -1;
    for (const ply of checkpoints.keys()) {
      if (ply >= target) continue;
      if (ply > bestPly) bestPly = ply;
    }
    if (bestPly < 0) return null;
    const snap = checkpoints.get(bestPly);
    if (snap === undefined) return null;
    return { ply: bestPly, snap };
  }

  function scheduleTimer(cb: () => void, ms: number): TimerHandle {
    let handle: TimerHandle | null = null;
    handle = scheduler.setTimeout(() => {
      if (handle !== null) timers.delete(handle);
      cb();
    }, ms);
    timers.add(handle);
    return handle;
  }

  function cancelTimer(handle: TimerHandle): void {
    if (timers.delete(handle)) scheduler.clearTimeout(handle);
  }

  function cancelAllTimers(): void {
    for (const h of timers) scheduler.clearTimeout(h);
    timers.clear();
  }

  // Driver-local position/legal - used only when no positionSink is supplied.
  let localPosition = $state<PositionView | null>(null);
  let localLegal = $state<Uint32Array>(new Uint32Array());

  let pieceIds = $state<Map<number, number>>(new Map());
  let nextPieceId = 1;

  let shakingSquares = $state<Set<number>>(new Set());
  let pieceMotion = $state<Map<number, PieceMotion>>(new Map());

  // Pending animationDone gate. `animationEndsAt` is a monotonic timestamp
  // (clock.now() basis) - when the current wall clock passes it, the
  // animation is finished. Resolved by a scheduled timer. Multiple callers
  // (e.g. AI loop + replay autoplay) can await the same underlying promise.
  let animationEndsAt = 0;
  let animationPromise: Promise<void> = Promise.resolve();
  let animationResolve: (() => void) | null = null;

  const effectQueue: Effect[] = $state([]);

  /** Push an effect into the queue, stamping `ttl` from the current animation
   *  speed multiplier so cinematic viewers get a slow, deliberate flourish
   *  while fast viewers get a snappier one. Skips the push entirely when
   *  animations are off. */
  function pushFx(eff: Effect): void {
    const mult = fxSpeedMultiplier();
    if (mult === 0) return;
    const baseline = FX_LIFETIME_MS[eff.kind];
    (eff as { ttl?: number }).ttl = baseline * mult;
    effectQueue.push(eff);
  }

  let lastApplied = $state<{ src: number; target: number } | null>(null);

  type PendingSkillRefresh = {
    handle: TimerHandle;
    targetZobrist: bigint;
    apply: () => void;
  };
  let pendingSkillRefresh: PendingSkillRefresh | null = null;

  const sfxOn: () => boolean =
    typeof opts.sfxEnabled === "function"
      ? opts.sfxEnabled
      : () => opts.sfxEnabled !== false;

  function playSfx(...args: Parameters<typeof sfx.play>): void {
    if (sfxOn()) sfxImpl.play(...args);
  }

  function getPosition(): PositionView | null {
    return opts.positionSink ? opts.positionSink.position : localPosition;
  }

  function setPosition(pv: PositionView): void {
    if (opts.positionSink) opts.positionSink.position = pv;
    else localPosition = pv;
  }

  function getLegal(): Uint32Array {
    return opts.positionSink ? opts.positionSink.legal : localLegal;
  }

  function setLegal(la: Uint32Array): void {
    if (opts.positionSink) opts.positionSink.legal = la;
    else localLegal = la;
  }

  function reconcilePieceIds(): void {
    const pos = getPosition();
    if (!pos) return;
    reconcilePieceIdsAgainst(pos);
  }

  // Reconcile against an explicit position rather than the current sink value.
  // Used by resyncFromEngine so pieceIds is consistent before the position
  // write lands on the reactive sink - preventing a Svelte flush from
  // observing an empty pieceIds map between setPosition and reconcile.
  function reconcilePieceIdsAgainst(pos: PositionView): void {
    const occupied = new Set<number>();
    const p1 = pos.bitboards[0];
    const p2 = pos.bitboards[1];
    const both = p1 | p2;
    for (let sq = 0; sq < 64; sq++) {
      if (((both >> BigInt(sq)) & 1n) === 1n) occupied.add(sq);
    }
    for (const sq of pieceIds.keys()) {
      if (!occupied.has(sq)) pieceIds.delete(sq);
    }
    for (const sq of occupied) {
      if (!pieceIds.has(sq)) pieceIds.set(sq, nextPieceId++);
    }
    pieceIds = new Map(pieceIds);
  }

  function triggerShake(sq: number): void {
    shakingSquares = new Set([...shakingSquares, sq]);
    scheduleTimer(() => {
      shakingSquares = new Set([...shakingSquares].filter((s) => s !== sq));
    }, SHAKE_DURATION_MS);
  }

  /** Total ms a PieceMotion will consume: `hops × slideDur` for the walk plus
   *  one extra hop's worth for a kill-lunge or lunge-return segment when set. */
  function motionDurationMs(m: PieceMotion, dur: number): number {
    let total = m.hops * dur;
    if (m.killLungeTo !== null || m.lungeReturnTo !== null) total += dur; // one extra hop for the lunge segment
    return total;
  }

  /** Emit a walk descriptor and arm the animationDone gate. `finalSq` is the
   *  square the piece REST at after the state flip (Board keys pieceMotion by
   *  finalSq). Also schedules a timer to clear the entry so a stale motion
   *  doesn't survive into the next ply. */
  function emitPieceMotion(finalSq: number, motion: PieceMotion): void {
    const dur = slideDurationMs();
    const total = motionDurationMs(motion, dur);
    pieceMotion = new Map([...pieceMotion, [finalSq, motion]]);
    if (total <= 0) return;
    const endsAt = clock.now() + total;
    if (endsAt > animationEndsAt) animationEndsAt = endsAt;
    // Fresh promise so late awaiters don't resolve on an old cycle.
    if (!animationResolve) {
      animationPromise = new Promise<void>((res) => { animationResolve = res; });
    }
    scheduleTimer(() => {
      // Only clear if this is still the same descriptor (defensive against
      // rapid successive plies replacing it mid-flight).
      if (pieceMotion.get(finalSq) === motion) {
        const next = new Map(pieceMotion);
        next.delete(finalSq);
        pieceMotion = next;
      }
      // Resolve only when we've reached the tracked end (a later, longer
      // motion may extend the window).
      if (clock.now() >= animationEndsAt - 1 && animationResolve) {
        const r = animationResolve;
        animationResolve = null;
        r();
      }
    }, total);
  }

  function animationDone(): Promise<void> {
    if (animationResolve === null) return Promise.resolve();
    return animationPromise;
  }

  function pushDamageEffect(targetSq: number, before: number, after: number): void {
    const dmg = before - after;
    if (dmg <= 0) return;
    const now = clock.now();
    pushFx({ kind: "impact", at: targetSq, startedAt: now });
    pushFx({ kind: "damageNumber", at: targetSq, amount: dmg, startedAt: now + 80 });
    triggerShake(targetSq);
    playSfx("damage");
  }

  /** Emit damage/heal/armor on stayed pieces and on relocated pieces (paired
   *  on Chebyshev distance). Renders on the POST-MOVE square so numbers
   *  travel with the relocated piece. */
  function emitImpactEvents(
    pre: Uint16Array,
    post: Uint16Array,
    diff: SkillDiff,
  ): boolean {
    const now = clock.now();
    let fired = false;
    const visit = (preSq: number, postSq: number) => {
      const a = decodeMailbox(pre[preSq]);
      const b = decodeMailbox(post[postSq]);
      if (a.empty || b.empty) return;
      const hpDelta = b.hp - a.hp;
      const arDelta = b.armor - a.armor;
      const renderSq = postSq;
      if (hpDelta < 0) {
        pushDamageEffect(renderSq, a.hp + a.armor, b.hp + b.armor);
        fired = true;
      } else if (hpDelta > 0) {
        pushFx({ kind: "heal", at: renderSq, amount: hpDelta, startedAt: now });
        playSfx("heal");
        fired = true;
      }
      if (arDelta > 0) {
        pushFx({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now + 40 });
        playSfx("armor");
        fired = true;
      } else if (arDelta < 0 && hpDelta === 0) {
        pushFx({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now });
        playSfx("armorBreak");
        fired = true;
      }
    };
    for (const sq of diff.stayed) visit(sq, sq);
    for (const m of diff.moves) visit(m.from, m.to);
    return fired;
  }

  function emitRelocationAndDeathEvents(pre: Uint16Array, diff: SkillDiff): void {
    const now = clock.now();
    for (const m of diff.moves) {
      const path = straightPath(m.from, m.to);
      if (path.length >= 2) {
        pushFx({ kind: "dust", path, startedAt: now });
      }
      const id = pieceIds.get(m.from);
      if (id !== undefined) {
        pieceIds.delete(m.from);
        pieceIds.set(m.to, id);
      }
      playSfx("move", { tiles: m.dist });
    }
    for (const v of diff.deaths) {
      const a = decodeMailbox(pre[v]);
      const dmg = a.hp + a.armor;
      pushFx({ kind: "impact", at: v, startedAt: now });
      if (dmg > 0) {
        pushFx({ kind: "damageNumber", at: v, amount: dmg, startedAt: now + 80 });
      }
      triggerShake(v);
      pieceIds.delete(v);
      playSfx("death");
    }
  }

  /** Render the Stack N strike-moves-caster forced step: a pure relocation
   *  (dust trail + pieceId follow), emitting NO heal/damage for the caster.
   *  When `killSq !== null` the caster stepped onto a tile whose enemy just
   *  died (point-blank kill) - emit that victim's death visuals on `killSq`
   *  here, since the anonymous diff no longer carries them. `pre` is the
   *  pre-skill mailbox so the victim's pre-death HP/armor is available. */
  function emitCasterRelocation(
    pre: Uint16Array,
    move: { from: number; to: number; dist: number },
    killSq: number | null,
  ): void {
    const now = clock.now();
    if (killSq !== null) {
      const victim = decodeMailbox(pre[killSq]);
      const dmg = victim.hp + victim.armor;
      pushFx({ kind: "impact", at: killSq, startedAt: now });
      if (dmg > 0) {
        pushFx({ kind: "damageNumber", at: killSq, amount: dmg, startedAt: now + 80 });
      }
      triggerShake(killSq);
      pieceIds.delete(killSq); // victim id lived at killSq (== move.to) pre-state
      playSfx("death");
    }
    const path = straightPath(move.from, move.to);
    if (path.length >= 2) {
      pushFx({ kind: "dust", path, startedAt: now });
    }
    const id = pieceIds.get(move.from);
    if (id !== undefined) {
      pieceIds.delete(move.from);
      pieceIds.set(move.to, id);
    }
    playSfx("move", { tiles: move.dist });
  }

  /** Render a BodyguardChoice ply. The preceding Move-Attack ply was silent
   *  (tentative apply, no damage). Here the hit actually lands:
   *   - DECLINE (idx 0): the attacker lunges at the original target (lunge-and-
   *     return if it survives / attacker stays; kill-lunge onto the tile if the
   *     target dies and the attacker steps in). Damage on the target.
   *   - ACCEPT (idx k): the chosen Guard lunges toward the attacker and recoils
   *     to its own tile (it never leaves). Damage on the guard (it may die).
   *  Damage numbers + attack/death SFX fire at the WAAPI lunge peak; the board
   *  state settles after RELOC_DELAY_MS so the motion isn't wiped mid-flight. */
  async function renderBodyguardChoice(raw: number, pre: PreStateSnapshot): Promise<void> {
    drainPendingSkillRefresh();
    const pending = pre.preBodyguardPending;
    const preFull = pre.preFull;
    const fresh = await fetchFreshState();
    // If we can't reconstruct the choice geometry, fall back to a silent resync
    // (correctness over animation).
    if (!pending || !preFull || !fresh) {
      await resyncFromEngine();
      return;
    }

    const idx = bgGuardIdx(raw);
    const post = fresh.pos.mailbox;
    const dur = slideDurationMs();
    const attackerNow = pending.attackerNow;

    // Who takes the hit, where the lunging piece rests, and the motion shape.
    let hitSq: number;
    let restSq: number;          // square the lunging piece keys/returns to
    let lungeReturnTo: number | null = null;
    let killLungeTo: number | null = null;
    let attackerStepsIn = false; // decline + target died + attacker took the tile

    if (idx === 0) {
      // Decline: attacker strikes the named target.
      hitSq = pending.targetSq;
      const targetDied = decodeMailbox(post[hitSq]).empty;
      const attackerLeft = decodeMailbox(post[attackerNow]).empty;
      if (targetDied && attackerLeft) {
        // Target died AND the attacker relocated onto the (now-empty) target
        // tile → kill-lunge that lands there.
        killLungeTo = hitSq;
        restSq = hitSq;
        attackerStepsIn = true;
      } else {
        // Target survives, or dies but the attacker stays put → lunge-and-return.
        lungeReturnTo = hitSq;
        restSq = attackerNow;
      }
    } else {
      // Accept: the chosen guard intercepts, lunging toward the attacker.
      const guardSq = pending.eligible[idx - 1];
      hitSq = guardSq;
      lungeReturnTo = attackerNow;
      restSq = guardSq;
    }

    // Damage on the hit square (pre-state HP+armor → post-state, 0 if removed).
    const preHit = decodeMailbox(preFull[hitSq]);
    const before = preHit.hp + preHit.armor;
    const postHit = decodeMailbox(post[hitSq]);
    const after = postHit.empty ? 0 : postHit.hp + postHit.armor;
    const died = after <= 0;
    const tiles = chebyshev(restSq, hitSq === restSq ? attackerNow : hitSq);

    // Emit the lunge motion (keyed by the resting square) when animations are on.
    if (dur > 0) {
      emitPieceMotion(restSq, {
        waypoints: [restSq],
        killLungeTo,
        lungeReturnTo,
        startedAt: clock.now(),
        hops: 0,
      });
    }

    // Fire damage + SFX at the lunge peak (offset ~0.40 of the single lunge hop).
    const contactDelay = dur * 0.40;
    const fire = () => {
      if (after < before) pushDamageEffect(hitSq, before, after);
      playSfx("attack", { tiles });
      if (died) playSfx("death");
    };
    if (contactDelay > 0) scheduleTimer(fire, contactDelay);
    else fire();

    // Settle the board AFTER the lunge so the motion isn't wiped by an early
    // state flip and the death sweep lands after the peak. Move the relevant
    // pieceId so the DOM element keeps identity across the flip.
    const settle = () => {
      if (attackerStepsIn) {
        // Attacker relocated onto the dead target's tile.
        const aid = pieceIds.get(attackerNow);
        if (aid !== undefined) {
          pieceIds.delete(attackerNow);
          pieceIds.set(hitSq, aid);
        }
      }
      setPosition(fresh.pos);
      setLegal(fresh.legal);
      reconcilePieceIds();
    };
    if (dur > 0) scheduleTimer(settle, RELOC_DELAY_MS);
    else settle();

    lastApplied = { src: attackerNow, target: hitSq };
  }

  // === Pre-state snapshot ==================================================

  function snapshotPreState(raw: number, prePosition?: PositionView): PreStateSnapshot {
    // Draft turns (bit 30) and bodyguard choices (bit 31) use disjoint bit
    // layouts from Move/Skill/EndPhase/EndTurn. Feeding them through
    // decodeAction extracts junk src/target/kind fields. Neither kind changes
    // board occupancy, so an empty pre-state is correct.
    if (isDraftTurn(raw) || isBodyguardChoice(raw)) {
      const pos = prePosition ?? getPosition();
      const preFull = pos?.mailbox ? new Uint16Array(pos.mailbox) : null;
      return {
        preFull,
        preTarget: null,
        preBodyguard: [],
        preP1Money: pos?.p1Money ?? 0,
        preP2Money: pos?.p2Money ?? 0,
        // Capture the engine's live pending-bodyguard cache so the choice-ply
        // renderer can animate the intercept/decline. Only present for choice
        // plies (bit 31); a draft turn leaves it null.
        preBodyguardPending: isBodyguardChoice(raw) ? (pos?.pendingBodyguard ?? null) : null,
      };
    }
    const decoded = decodeAction(raw);
    const pos = prePosition ?? getPosition();
    const preMailbox = pos?.mailbox ?? null;
    const preFull: Uint16Array | null = preMailbox ? new Uint16Array(preMailbox) : null;
    const preTarget = preMailbox ? decodeMailbox(preMailbox[decoded.target]) : null;
    const preBodyguard: BodyguardSnapshot = [];
    if (preMailbox && decoded.kind === ActionKind.Move && decoded.hasAux) {
      const tFile = decoded.target & 7;
      const tRank = (decoded.target >> 3) & 7;
      for (let df = -1; df <= 1; df++) {
        for (let dr = -1; dr <= 1; dr++) {
          if (df === 0 && dr === 0) continue;
          const nf = tFile + df, nr = tRank + dr;
          if (nf < 0 || nf > 7 || nr < 0 || nr > 7) continue;
          const sq = (nr << 3) | nf;
          const ent = decodeMailbox(preMailbox[sq]);
          if (!ent.empty) preBodyguard.push({ sq, entry: ent });
        }
      }
    }
    return {
      preFull,
      preTarget,
      preBodyguard,
      preP1Money: pos?.p1Money ?? 0,
      preP2Money: pos?.p2Money ?? 0,
      preBodyguardPending: null,
    };
  }

  // === Apply + render ======================================================

  function drainPendingSkillRefresh(): void {
    if (!pendingSkillRefresh) return;
    cancelTimer(pendingSkillRefresh.handle);
    const apply = pendingSkillRefresh.apply;
    pendingSkillRefresh = null;
    apply();
  }

  async function fetchFreshState(): Promise<{ pos: PositionView; legal: Uint32Array } | null> {
    if (!eng) return null;
    const pos = await eng.positionView();
    const legal = await eng.legalActions();
    return { pos, legal };
  }

  async function refresh(): Promise<void> {
    drainPendingSkillRefresh();
    const fresh = await fetchFreshState();
    if (!fresh) return;
    setPosition(fresh.pos);
    setLegal(fresh.legal);
  }

  async function renderApplied(raw: number, pre: PreStateSnapshot): Promise<void> {
    // Draft turns and bodyguard choices don't drive board animations. Draft
    // plies update piece skill assignments (visible via position but not by
    // motion). Bodyguard choices may swap the attacked piece; either way, the
    // Move/Skill decode path would extract junk fields. Pull fresh engine
    // state and return.
    if (isDraftTurn(raw)) {
      drainPendingSkillRefresh();
      await refresh();
      return;
    }
    if (isBodyguardChoice(raw)) {
      await renderBodyguardChoice(raw, pre);
      return;
    }

    const { preFull, preTarget, preBodyguard } = pre;
    const decoded = decodeAction(raw);

    // Caller may or may not have drained; drain defensively.
    drainPendingSkillRefresh();

    if (decoded.kind === ActionKind.Skill) {
      playSfx("skillFire");
    } else if (decoded.kind === ActionKind.EndPhase) {
      playSfx("phaseEnd");
    }

    // For Move actions we need the CSS slide transition to actually fire.
    // The browser needs one painted frame with the DOM element at its OLD
    // square before we reassign its transform to the new square. Strategy:
    //   1. Wait one rAF (piece is still at old square, browser paints it).
    //   2. Fetch the new engine state (async, but DOM hasn't changed yet).
    //   3. Compute kill detection upfront from the fresh position.
    //   4. In one synchronous block: update pieceIds AND position together -
    //      both writes land in the same Svelte flush so the DOM element's
    //      transform transitions from the old painted position to the new one.
    if (decoded.kind === ActionKind.Move) {
      if (slideDurationMs() > 0) await waitFrame();
      const fresh = await fetchFreshState();
      if (fresh) {
        const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
        // Pending-bodyguard short-circuit: a Move-Attack against a Champion/King
        // with an eligible adjacent Guard is applied TENTATIVELY by the engine -
        // no damage dealt, `pendingBodyguard` set, side flipped to the defender.
        // The attack ply must stay SILENT (no lunge, no damage, no death); the
        // actual hit + intercept/decline animation happens on the following
        // BodyguardChoice ply. Just relocate the attacker to its approach square
        // and flush.
        if (fresh.pos.pendingBodyguard != null) {
          const srcId = pieceIds.get(decoded.src);
          if (srcId !== undefined && approach !== decoded.src) {
            pieceIds.delete(decoded.src);
            pieceIds.set(approach, srcId);
          }
          pieceIds = new Map(pieceIds);
          setPosition(fresh.pos);
          setLegal(fresh.legal);
          reconcilePieceIds();
          lastApplied = { src: decoded.src, target: decoded.target };
          return;
        }
        // Detect kill from fresh position before any state writes.
        let movedKilled = false;
        if (decoded.hasAux && preTarget) {
          const postTarget = decodeMailbox(fresh.pos.mailbox[decoded.target]);
          if (!postTarget.empty && approach !== decoded.target) {
            movedKilled = decodeMailbox(fresh.pos.mailbox[approach]).empty;
          } else if (!postTarget.empty && approach === decoded.target) {
            movedKilled = true;
          }
        }
        // Move attacker's stable DOM key to its final resting square.
        const finalSrc = movedKilled ? decoded.target : approach;
        const srcId = pieceIds.get(decoded.src);
        if (srcId !== undefined) {
          pieceIds.delete(decoded.src);
          pieceIds.set(finalSrc, srcId);
        }
        // Flush position + pieceIds together so CSS transition fires.
        pieceIds = new Map(pieceIds);
        setPosition(fresh.pos);
        setLegal(fresh.legal);
      }
    } else if (decoded.kind !== ActionKind.Skill) {
      await refresh();
    }

    let killed = false;
    const curPos = getPosition();
    if (decoded.kind === ActionKind.Move && decoded.hasAux && preTarget && curPos) {
      const postTarget = decodeMailbox(curPos.mailbox[decoded.target]);
      const approach = decoded.auxSq;
      if (!postTarget.empty && approach !== decoded.target) {
        const postApproach = decodeMailbox(curPos.mailbox[approach]);
        if (postApproach.empty) {
          killed = true;
          const aid = pieceIds.get(approach);
          if (aid !== undefined) {
            pieceIds.delete(approach);
            pieceIds.set(decoded.target, aid);
          }
        }
      } else if (!postTarget.empty && approach === decoded.target) {
        killed = true;
      }
    }

    if (decoded.kind !== ActionKind.Skill) {
      reconcilePieceIds();
    }

    if (decoded.kind === ActionKind.Move) {
      const path = walkedPath(decoded, killed);
      if (path.length >= 2) {
        pushFx({ kind: "dust", path, startedAt: clock.now() });
      }
      const finalAttackerSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      const tiles = chebyshev(decoded.src, finalAttackerSq);
      // Move SFX (footstep) fires immediately - that's the walk start.
      // Attack SFX (impact) and death SFX are deferred until the piece
      // actually reaches the target, so audio matches the lunge landing.
      if (!decoded.hasAux) playSfx("move", { tiles });

      // Piece-motion emission: derive the waypoint list from the decoded
      // action + kill outcome. Rules match .claude/plans/swirling-floating-alpaca.md:
      //   pure move (no aux): [src, target], or [src, mid, target] if cheb 2.
      //   move-attack, no kill: [src(, approach)] + lungeReturnTo=target - the
      //     attacker leans toward the target and recoils to its resting square.
      //   move-attack, kill (approach != target): [src, approach] + killLungeTo=target.
      //   move-attack, kill (approach == target): [src, target] with kill lunge implicit.
      const dur = slideDurationMs();
      let walkHops = 0;
      if (dur > 0) {
        let waypoints: number[];
        let killLungeTo: number | null = null;
        let lungeReturnTo: number | null = null;
        if (!decoded.hasAux) {
          const mid = chebyshevMidpoint(decoded.src, decoded.target, preFull);
          waypoints = mid !== null
            ? [decoded.src, mid, decoded.target]
            : [decoded.src, decoded.target];
        } else if (!killed) {
          waypoints = decoded.auxSq === decoded.src
            ? [decoded.src]
            : [decoded.src, decoded.auxSq];
          lungeReturnTo = decoded.target;
        } else if (decoded.auxSq !== decoded.target) {
          waypoints = decoded.auxSq === decoded.src
            ? [decoded.src]
            : [decoded.src, decoded.auxSq];
          killLungeTo = decoded.target;
        } else {
          waypoints = [decoded.src, decoded.target];
        }
        const hops = Math.max(0, waypoints.length - 1);
        walkHops = hops;
        if (hops > 0 || killLungeTo !== null || lungeReturnTo !== null) {
          emitPieceMotion(
            finalAttackerSq,
            {
              waypoints,
              killLungeTo,
              lungeReturnTo,
              startedAt: clock.now(),
              hops,
            },
          );
        }
      }

      const postPos = getPosition();
      const finalSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      opts.onMoveLanding?.(finalSq);

      // Attack contact timing: damage numbers, hit-shake, damage SFX all fire
      // when the attacker would visually reach the target - after the walk
      // finishes, plus the fraction of the lunge segment where it peaks. Both
      // kill and non-kill lunges are one WAAPI hop (`dur`) that peaks at offset
      // ~0.40 (see buildKeyframes in Piece.svelte), so the timing is uniform.
      if (decoded.hasAux && preTarget && postPos) {
        const postTarget = decodeMailbox(postPos.mailbox[decoded.target]);
        const before = preTarget.hp + preTarget.armor;
        const after = killed ? 0 : postTarget.hp + postTarget.armor;
        const contactDelay = walkHops * dur + dur * 0.40;
        const fire = () => {
          if (after < before) {
            pushDamageEffect(decoded.target, before, after);
          }
          playSfx("attack", { tiles });
          if (killed) playSfx("death");
        };
        if (contactDelay > 0) scheduleTimer(fire, contactDelay);
        else fire();
      }
    }

    if (decoded.kind === ActionKind.Skill && preFull) {
      const fresh = await fetchFreshState();
      if (!fresh) return;
      const newMailbox = fresh.pos.mailbox;
      const diff = diffSkillMailbox(preFull, newMailbox);
      // Stack N strike-moves-caster: pull the caster's known forced step out of
      // the anonymous diff so it renders as a relocation, not a bogus heal on
      // the destination + death on the origin. Derive whether the step landed
      // on a tile whose enemy just died (point-blank kill) so its death visuals
      // fire on that square.
      const casterMove = extractCasterMove(decoded, newMailbox, diff);
      let casterKillSq: number | null = null;
      if (casterMove && !decodeMailbox(preFull[casterMove.to]).empty) {
        casterKillSq = casterMove.to;
      }
      const hasReloc = hasRelocationOrDeath(diff) || casterMove !== null;
      // Emit the per-skill choreography first (drawn behind mailbox-delta
      // effects because it's pushed earlier). Uses decoded.skillId + src
      // as caster; target = decoded.target for cast-at-square skills, or
      // src for self-casts (defaultRange 0). Focus/Charge (14/15) produce
      // no mailbox delta, so this is the ONLY visual signal they get.
      const casterSq = decoded.src;
      const targetSq = decoded.target;
      const now = clock.now();
      // Outcome-aware fields the skill renderers can consult so the drawing
      // reflects what actually happened, not just the action intent:
      // - Steal: did the target actually have money to lose? Compare pre/post
      //   totals; a non-zero delta on either side means money moved.
      // - Hook: where did the pulled target end up? diff.moves whose `from`
      //   equals `targetSq` is the target's post-square (paired by nearest
      //   dst in diffSkillMailbox).
      const moneyStolen =
        fresh.pos.p1Money !== pre.preP1Money || fresh.pos.p2Money !== pre.preP2Money;
      let targetPostSq: number | undefined;
      const pulledMove = diff.moves.find((m) => m.from === targetSq);
      if (pulledMove) targetPostSq = pulledMove.to;
      // Stack N strike-moves-caster: where the caster ended up. Strike
      // renderers animate their caster-end from `from` → this square so the
      // choreography tracks the piece as it steps toward the target.
      const casterPostSq = casterMove ? casterMove.to : undefined;
      pushFx({
        kind: "skill",
        skillId: decoded.skillId,
        from: casterSq,
        to: targetSq,
        startedAt: now,
        hasAux: decoded.hasAux,
        auxSq: decoded.hasAux ? decoded.auxSq : undefined,
        outcome: { moneyStolen, targetPostSq, casterPostSq },
      });
      // Global caster spotlight - subtle attention ring on the casting piece
      // so effects like Focus/Charge/Shield are readable even if the eye is
      // elsewhere. Uses the skill's category tint. Drawn UNDER the choreography
      // because it's pushed after (canvas draws in queue order - later entries
      // paint on top; the spotlight paints on top of the per-skill draw. That
      // matches the fireworks-ring-around-caster feel: the ring surrounds the
      // caster's centre and the skill mark extends beyond it toward target).
      pushFx({
        kind: "spotlight",
        at: casterMove ? casterMove.to : casterSq,
        color: skillColor(decoded.skillId),
        startedAt: now,
      });
      const impactFired = emitImpactEvents(preFull, newMailbox, diff);
      const applyFresh = () => {
        setPosition(fresh.pos);
        setLegal(fresh.legal);
        if (hasRelocationOrDeath(diff)) emitRelocationAndDeathEvents(preFull, diff);
        if (casterMove) emitCasterRelocation(preFull, casterMove, casterKillSq);
        reconcilePieceIds();
      };
      if (hasReloc && impactFired) {
        // Drain any prior pending skill refresh so it can't fire later and
        // clobber post-end-turn state.
        drainPendingSkillRefresh();
        const targetZobrist = fresh.pos.zobrist;
        const handle = scheduleTimer(() => {
          if (pendingSkillRefresh?.targetZobrist !== targetZobrist) return;
          pendingSkillRefresh = null;
          applyFresh();
        }, RELOC_DELAY_MS);
        pendingSkillRefresh = { handle, targetZobrist, apply: applyFresh };
      } else {
        applyFresh();
      }
    }

    lastApplied =
      decoded.kind === ActionKind.Move || decoded.kind === ActionKind.Skill
        ? { src: decoded.src, target: decoded.target }
        : null;
  }

  async function applyAndRender(
    raw: number,
    applyFn: () => Promise<void>,
    opts?: ApplyAndRenderOpts,
  ): Promise<void> {
    drainPendingSkillRefresh();
    const pre = snapshotPreState(raw);
    await applyFn();
    await renderApplied(raw, pre);

    // Opportunistic checkpoint capture. We log a checkpoint at the ply
    // *after* this one - i.e., we just finished applying `plyHint` so the
    // engine is now at position-after-ply-N. Replay's `currentPly` is
    // incremented to N+1 only after applyAndRender returns, so passing the
    // pre-apply index here is correct: the checkpoint represents "state
    // after this ply has been applied". A subsequent fastForwardTo to
    // target=N+1 can restore from this snapshot and replay zero plies.
    const hint = opts?.plyHint;
    if (typeof hint === "number" && opts?.plyHintBase) {
      invalidateCheckpointsIfBaseChanged(opts.plyHintBase);
      const checkpointPly = hint + 1;
      if (checkpointPly > 0 && checkpointPly % CHECKPOINT_STRIDE === 0) {
        try {
          const snap = await eng.snapshotJson();
          checkpoints.set(checkpointPly, snap);
        } catch {
          // Snapshot capture is best-effort; a failure here just means
          // the next scrub does a cold replay. Don't surface this.
        }
      }
    }
  }

  // === Bulk operations =====================================================

  async function resyncFromEngine(): Promise<void> {
    drainPendingSkillRefresh();
    pieceIds = new Map();
    nextPieceId = 1;
    shakingSquares = new Set();
    pieceMotion = new Map();
    effectQueue.length = 0;
    lastApplied = null;
    const fresh = await fetchFreshState();
    if (!fresh) return;
    // Reconcile pieceIds against the fresh position BEFORE writing to the
    // reactive sink. This ensures no Svelte flush can observe an empty
    // pieceIds map between setPosition and reconcile (the Tauri first-render
    // Guards bug: piece DOM nodes got fallback "sq-N" keys on first paint,
    // then remounted with numeric keys when reconcile ran, resetting CSS state).
    reconcilePieceIdsAgainst(fresh.pos);
    setPosition(fresh.pos);
    setLegal(fresh.legal);
  }

  function reset(): void {
    drainPendingSkillRefresh();
    // P2 fix: also cancel outstanding shake timers. Without this, a shake
    // setTimeout scheduled by a pre-reset action would fire after reset and
    // re-introduce stale square ids into shakingSquares.
    cancelAllTimers();
    pieceIds = new Map();
    nextPieceId = 1;
    shakingSquares = new Set();
    pieceMotion = new Map();
    animationEndsAt = 0;
    if (animationResolve) { animationResolve(); animationResolve = null; }
    effectQueue.length = 0;
    lastApplied = null;
    checkpoints.clear();
    checkpointsBase = null;
  }

  function dispose(): void {
    // Drain (apply pending refresh in case caller wants final state visible),
    // then cancel any timers scheduled BY that drain (none in current impl,
    // but defensive). Finally cancel everything else.
    drainPendingSkillRefresh();
    cancelAllTimers();
    pendingSkillRefresh = null;
    effectQueue.length = 0;
    shakingSquares = new Set();
    pieceMotion = new Map();
    animationEndsAt = 0;
    if (animationResolve) { animationResolve(); animationResolve = null; }
    checkpoints.clear();
    checkpointsBase = null;
  }

  async function fastForwardTo(
    baseSnapshotJson: string,
    plies: number[],
    target: number,
  ): Promise<void> {
    drainPendingSkillRefresh();
    const clamped = Math.max(0, Math.min(plies.length, target | 0));

    // Cache invalidation: a different base snapshot means the cached
    // checkpoints reference a different play-line and must be discarded.
    invalidateCheckpointsIfBaseChanged(baseSnapshotJson);

    if (clamped === 0) {
      await eng.restoreFromSnapshot(baseSnapshotJson);
      await resyncFromEngine();
      return;
    }

    // Try to skip the leading silent replay by restoring from the nearest
    // checkpoint < target. We require at least CHECKPOINT_MIN_SAVING plies of
    // saving to justify the extra restoreFromSnapshot round-trip.
    let startIndex = 0;
    const cp = nearestCheckpoint(clamped);
    if (cp && clamped - cp.ply >= CHECKPOINT_MIN_SAVING) {
      await eng.restoreFromSnapshot(cp.snap);
      startIndex = cp.ply;
    } else {
      await eng.restoreFromSnapshot(baseSnapshotJson);
    }

    // Silent fast-forward from startIndex through plies[clamped-2]. No
    // effects, no SFX, no position writes - we only care about engine state
    // at the end. Along the way, capture a checkpoint every CHECKPOINT_STRIDE
    // plies (using "ply count from base" as the key, not array index) so a
    // future scrub through the same range gets the perf win on the first
    // cold pass.
    for (let i = startIndex; i < clamped - 1; i++) {
      await eng.tryApply(plies[i]);
      const plyCount = i + 1;
      if (plyCount % CHECKPOINT_STRIDE === 0 && !checkpoints.has(plyCount)) {
        try {
          const snap = await eng.snapshotJson();
          checkpoints.set(plyCount, snap);
        } catch {
          // Best-effort; ignore.
        }
      }
    }

    // Reconcile pieceIds from the now-restored intermediate state. We need
    // the renderer's position to reflect "post ply clamped-2" so renderApplied
    // can diff against it.
    effectQueue.length = 0;
    shakingSquares = new Set();
    pieceMotion = new Map();
    pieceIds = new Map();
    nextPieceId = 1;
    lastApplied = null;
    const intermediate = await fetchFreshState();
    if (!intermediate) return;
    setPosition(intermediate.pos);
    setLegal(intermediate.legal);
    reconcilePieceIds();

    // Now render the landing ply with the full pipeline.
    await applyAndRender(plies[clamped - 1], async () => {
      await eng.tryApply(plies[clamped - 1]);
    });
  }

  return {
    get position() { return getPosition(); },
    get legal() { return getLegal(); },
    get pieceIds() { return pieceIds; },
    get shakingSquares() { return shakingSquares; },
    get pieceMotion() { return pieceMotion; },
    get effectQueue() { return effectQueue; },
    get lastApplied() { return lastApplied; },
    animationDone,
    applyAndRender,
    renderApplied,
    snapshotPreState,
    drainPendingSkillRefresh,
    dispose,
    fastForwardTo,
    resyncFromEngine,
    reset,
  };
}
