// Pure helpers over the engine's legal-actions buffer. Kept out of components
// so derivations stay testable and component code is layout-only.

import { ActionKind, decodeAction, type ActionKindValue } from "$lib/engine";

/** Bodyguard redirect variants for a single (src, target, approach) triple.
 *
 * The engine emits the no-redirect variant (`choice_idx = 0`) plus one
 * variant per eligible adjacent Guard (`choice_idx = 1..N`, k-th in the
 * canonical ascending-square-index order of those Guards). We split them
 * into `defenderRaw` (no redirect) + `redirects` (k-th → raw).
 */
export interface BodyguardVariants {
  /** Raw action for choice_idx = 0 (defender takes the hit). Always present
   *  for Move-Attacks; for plain moves we store the move action here and
   *  leave `redirects` empty. */
  defenderRaw: number;
  /** Variants with choice_idx >= 1, indexed by k (1-based). The k-th entry
   *  corresponds to the k-th eligible Bodyguard Guard in ascending square
   *  index order (see `bodyguard_guards_for` in the Rust generator). */
  redirects: { choiceIdx: number; raw: number }[];
}

export interface MoveTargetSet {
  /** Squares reachable from `src` by a Move action. */
  squares: Set<number>;
  /**
   * Per target square: a map of approach_sq → variants. For non-attack
   * moves (or speed-1 attacks), there's exactly one entry whose approach key
   * equals the target itself. For speed-2 Move-Attacks with multiple paths,
   * there's one entry per distinct approach.
   */
  byTarget: Map<number, Map<number, BodyguardVariants>>;
  /** True iff the target has more than one distinct approach. */
  hasPathChoice(target: number): boolean;
}

function emptyTargets(): MoveTargetSet {
  const t: MoveTargetSet = {
    squares: new Set(),
    byTarget: new Map(),
    hasPathChoice(_target: number) {
      return false;
    },
  };
  return t;
}

export const EMPTY_MOVE_TARGETS: MoveTargetSet = emptyTargets();

/** Walk `legal` and pick the Move actions whose `src` matches the selection. */
export function moveTargetsFor(
  legal: Uint32Array,
  src: number | null,
): MoveTargetSet {
  if (src === null) return EMPTY_MOVE_TARGETS;
  const squares = new Set<number>();
  const byTarget = new Map<number, Map<number, BodyguardVariants>>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Move) continue;
    if (a.src !== src) continue;
    squares.add(a.target);
    const approach = a.hasAux ? a.auxSq : a.target;
    let perTarget = byTarget.get(a.target);
    if (!perTarget) {
      perTarget = new Map();
      byTarget.set(a.target, perTarget);
    }
    let variants = perTarget.get(approach);
    if (!variants) {
      variants = { defenderRaw: raw, redirects: [] };
      perTarget.set(approach, variants);
    }
    if (a.choiceIdx === 0) {
      variants.defenderRaw = raw;
    } else {
      variants.redirects.push({ choiceIdx: a.choiceIdx, raw });
    }
  }
  // Sort each redirects list by choiceIdx so the k-th entry maps to the
  // k-th eligible Guard. (The engine emits them in order, but be defensive
  // - Move actions for a given (src, target, approach) may arrive
  // interleaved with other variants.)
  for (const perTarget of byTarget.values()) {
    for (const variants of perTarget.values()) {
      variants.redirects.sort((x, y) => x.choiceIdx - y.choiceIdx);
    }
  }
  return {
    squares,
    byTarget,
    hasPathChoice(target: number) {
      const m = byTarget.get(target);
      return m !== undefined && m.size > 1;
    },
  };
}

/** Look up the raw u32 for (target, approach) with no Bodyguard redirect.
 * Returns null if no such variant exists. */
export function rawForTargetApproach(
  targets: MoveTargetSet,
  target: number,
  approach: number,
): number | null {
  const m = targets.byTarget.get(target);
  if (!m) return null;
  const v = m.get(approach);
  return v ? v.defenderRaw : null;
}

/**
 * When the player has clicked/dropped on `target` but multiple paths exist,
 * return the candidate approach squares (sorted ascending for stability).
 */
export function approachChoicesFor(
  targets: MoveTargetSet,
  target: number,
): number[] {
  const m = targets.byTarget.get(target);
  if (!m) return [];
  return [...m.keys()].sort((a, b) => a - b);
}

/** The first action in `legal` with the given kind, or null. */
export function findActionByKind(
  legal: Uint32Array,
  kind: ActionKindValue,
): number | null {
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    if (((raw >>> 12) & 0x3) === kind) return raw;
  }
  return null;
}

/** Squares that own a Move action in this `legal` set (i.e. movable pieces). */
export function movableSources(legal: Uint32Array): Set<number> {
  const out = new Set<number>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    if (((raw >>> 12) & 0x3) !== ActionKind.Move) continue;
    out.add(raw & 0x3f);
  }
  return out;
}

/** Squares that own either a Move or a Skill action - i.e. pieces the
 *  player can pick up / select to act with. Used to drive piece-selection
 *  interactivity; the resulting selection opens the radial wheel, whose
 *  internal slice legality decides which actions are actually castable. */
export function actableSources(legal: Uint32Array): Set<number> {
  const out = new Set<number>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const kind = (raw >>> 12) & 0x3;
    if (kind !== ActionKind.Move && kind !== ActionKind.Skill) continue;
    out.add(raw & 0x3f);
  }
  return out;
}

const SQUARE_SIZE = 100; // board viewBox is 800, 8 tiles

/** Given a target square and cursor position in SVG coords, pick the
 *  approach square from `candidates` whose direction from the target center
 *  best aligns with the cursor offset. Returns null when candidates is empty.
 *
 *  Used for both drag-and-drop and click-mode multi-approach Move-Attacks:
 *  the player "aims where they're coming from" to resolve ambiguous paths
 *  without a chooser dialog. */
export function pickApproachByCursor<T>(
  target: number,
  cx: number,
  cy: number,
  candidates: Map<number, T>,
): number | null {
  if (candidates.size === 0) return null;
  if (candidates.size === 1) {
    return candidates.keys().next().value as number;
  }
  const tgtFile = target & 7;
  const tgtRank = (target >> 3) & 7;
  const tgtCX = tgtFile * SQUARE_SIZE + SQUARE_SIZE / 2;
  const tgtCY = (7 - tgtRank) * SQUARE_SIZE + SQUARE_SIZE / 2;
  const offX = cx - tgtCX;
  const offY = cy - tgtCY;
  const offLen2 = offX * offX + offY * offY;
  if (offLen2 < 4) {
    if (candidates.has(target)) return target;
    return candidates.keys().next().value as number;
  }
  let best: number | null = null;
  let bestScore = -Infinity;
  for (const ap of candidates.keys()) {
    const apFile = ap & 7;
    const apRank = (ap >> 3) & 7;
    const apCX = apFile * SQUARE_SIZE + SQUARE_SIZE / 2;
    const apCY = (7 - apRank) * SQUARE_SIZE + SQUARE_SIZE / 2;
    const dirX = apCX - tgtCX;
    const dirY = apCY - tgtCY;
    const dirLen2 = dirX * dirX + dirY * dirY;
    if (dirLen2 === 0) continue;
    const score = (offX * dirX + offY * dirY) / Math.sqrt(dirLen2);
    if (score > bestScore) {
      bestScore = score;
      best = ap;
    }
  }
  return best;
}
