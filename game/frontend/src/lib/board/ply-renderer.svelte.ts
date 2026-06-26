// Shared ply renderer driver.
//
// Owns the producer side of the visual-effects pipeline that used to live
// inside `match/+page.svelte`. Both `/match/` and `/replay/` (and eventually
// `/inspector/`) create one of these and feed actions through `applyAndRender`
// (or `renderApplied` for the MP non-acting-peer path). The driver:
//
//   * holds the current rendered `position` + `legal` (or writes through to a
//     supplied carrier — see `positionSink`),
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
  type EngineClient,
  type PositionView,
} from "$lib/engine";
import type { Effect } from "$lib/viz/effects";
import { sfx } from "$lib/audio/sfx";

// === Public types ==========================================================

export type BodyguardSnapshot = {
  sq: number;
  entry: ReturnType<typeof decodeMailbox>;
}[];

export interface PreStateSnapshot {
  preFull: Uint16Array | null;
  preTarget: ReturnType<typeof decodeMailbox> | null;
  preBodyguard: BodyguardSnapshot;
}

/** A `$state`-backed carrier the renderer writes through to. Match passes
 *  the global `match` store here; replay/inspector leave it unset and read
 *  the renderer's own `position`/`legal` instead. */
export interface PositionSink {
  position: PositionView | null;
  legal: Uint32Array;
}

/** Opaque timer handle. Browser's `setTimeout` returns `number`; Node's
 *  returns an object — we don't care which, only that callers thread it
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

export interface PlyRenderer {
  // Reactive state — bind into Board / EffectsLayer.
  readonly position: PositionView | null;
  readonly legal: Uint32Array;
  readonly pieceIds: Map<number, number>;
  readonly shakingSquares: Set<number>;
  readonly effectQueue: Effect[];
  readonly lastApplied: { src: number; target: number } | null;

  // Lifecycle.
  /** Capture pre-state from the currently-rendered position, run `applyFn`
   *  (which is expected to advance the engine — caller's choice whether
   *  that's `eng.tryApply` directly or via an MP wrapper), then emit effects
   *  + SFX and flip rendered state. Any pending deferred refresh is drained
   *  synchronously at entry. */
  applyAndRender(raw: number, applyFn: () => Promise<void>): Promise<void>;

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
   *  refresh — call this when crossing match boundaries. */
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

// === Internal helpers (module-private) =====================================

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

  // Driver-local position/legal — used only when no positionSink is supplied.
  let localPosition = $state<PositionView | null>(null);
  let localLegal = $state<Uint32Array>(new Uint32Array());

  let pieceIds = $state<Map<number, number>>(new Map());
  let nextPieceId = 1;

  let shakingSquares = $state<Set<number>>(new Set());

  const effectQueue: Effect[] = $state([]);

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

  function pushDamageEffect(targetSq: number, before: number, after: number): void {
    const dmg = before - after;
    if (dmg <= 0) return;
    const now = clock.now();
    effectQueue.push({ kind: "impact", at: targetSq, startedAt: now });
    effectQueue.push({ kind: "damageNumber", at: targetSq, amount: dmg, startedAt: now + 80 });
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
        effectQueue.push({ kind: "heal", at: renderSq, amount: hpDelta, startedAt: now });
        playSfx("heal");
        fired = true;
      }
      if (arDelta > 0) {
        effectQueue.push({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now + 40 });
        playSfx("armor");
        fired = true;
      } else if (arDelta < 0 && hpDelta === 0) {
        effectQueue.push({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now });
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
        effectQueue.push({ kind: "dust", path, startedAt: now });
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
      effectQueue.push({ kind: "impact", at: v, startedAt: now });
      if (dmg > 0) {
        effectQueue.push({ kind: "damageNumber", at: v, amount: dmg, startedAt: now + 80 });
      }
      triggerShake(v);
      pieceIds.delete(v);
      playSfx("death");
    }
  }

  // === Pre-state snapshot ==================================================

  function snapshotPreState(raw: number, prePosition?: PositionView): PreStateSnapshot {
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
    return { preFull, preTarget, preBodyguard };
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
    const { preFull, preTarget, preBodyguard } = pre;
    const decoded = decodeAction(raw);

    // Caller may or may not have drained; drain defensively.
    drainPendingSkillRefresh();

    if (decoded.kind === ActionKind.Skill) {
      playSfx("skillFire");
    } else if (decoded.kind === ActionKind.EndPhase) {
      playSfx("phaseEnd");
    }

    // Transfer piece ids along the move BEFORE refresh, so the new bitboards
    // see a piece with stable identity at the destination.
    if (decoded.kind === ActionKind.Move) {
      const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
      const srcId = pieceIds.get(decoded.src);
      if (srcId !== undefined) {
        pieceIds.delete(decoded.src);
        pieceIds.set(approach, srcId);
      }
    }

    if (decoded.kind !== ActionKind.Skill) {
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
        effectQueue.push({ kind: "dust", path, startedAt: clock.now() });
      }
      const finalAttackerSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      const tiles = chebyshev(decoded.src, finalAttackerSq);
      playSfx(decoded.hasAux ? "attack" : "move", { tiles });
      if (killed) playSfx("death");
      const postPos = getPosition();
      if (decoded.hasAux && preTarget && postPos) {
        const postTarget = decodeMailbox(postPos.mailbox[decoded.target]);
        const before = preTarget.hp + preTarget.armor;
        const after = killed ? 0 : postTarget.hp + postTarget.armor;
        if (after < before) {
          pushDamageEffect(decoded.target, before, after);
        } else {
          for (const bg of preBodyguard) {
            const post = decodeMailbox(postPos.mailbox[bg.sq]);
            const bgBefore = bg.entry.hp + bg.entry.armor;
            const bgAfter = post.hp + post.armor;
            if (bgAfter < bgBefore) {
              pushDamageEffect(bg.sq, bgBefore, bgAfter);
              break;
            }
          }
        }
      }
      const finalSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      opts.onMoveLanding?.(finalSq);
    }

    if (decoded.kind === ActionKind.Skill && preFull) {
      const fresh = await fetchFreshState();
      if (!fresh) return;
      const newMailbox = fresh.pos.mailbox;
      const diff = diffSkillMailbox(preFull, newMailbox);
      const hasReloc = hasRelocationOrDeath(diff);
      const impactFired = emitImpactEvents(preFull, newMailbox, diff);
      const applyFresh = () => {
        setPosition(fresh.pos);
        setLegal(fresh.legal);
        if (hasReloc) emitRelocationAndDeathEvents(preFull, diff);
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

  async function applyAndRender(raw: number, applyFn: () => Promise<void>): Promise<void> {
    drainPendingSkillRefresh();
    const pre = snapshotPreState(raw);
    await applyFn();
    await renderApplied(raw, pre);
  }

  // === Bulk operations =====================================================

  async function resyncFromEngine(): Promise<void> {
    drainPendingSkillRefresh();
    pieceIds = new Map();
    nextPieceId = 1;
    shakingSquares = new Set();
    effectQueue.length = 0;
    lastApplied = null;
    const fresh = await fetchFreshState();
    if (!fresh) return;
    setPosition(fresh.pos);
    setLegal(fresh.legal);
    reconcilePieceIds();
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
    effectQueue.length = 0;
    lastApplied = null;
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
  }

  async function fastForwardTo(
    baseSnapshotJson: string,
    plies: number[],
    target: number,
  ): Promise<void> {
    drainPendingSkillRefresh();
    const clamped = Math.max(0, Math.min(plies.length, target | 0));

    // Restore to ply 0.
    await eng.restoreFromSnapshot(baseSnapshotJson);

    if (clamped === 0) {
      await resyncFromEngine();
      return;
    }

    // Silent fast-forward through plies 0..clamped-2. No effects, no SFX,
    // no position writes — we only care about engine state at the end.
    for (let i = 0; i < clamped - 1; i++) {
      await eng.tryApply(plies[i]);
    }

    // Reconcile pieceIds from the now-restored intermediate state. We need
    // the renderer's position to reflect "post ply clamped-2" so renderApplied
    // can diff against it.
    effectQueue.length = 0;
    shakingSquares = new Set();
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
    get effectQueue() { return effectQueue; },
    get lastApplied() { return lastApplied; },
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
