// Pure helpers for filtering legal skill actions by caster + skill. The
// engine emits one Skill action per (src, skill_id, target, choice_idx,
// modifier_bits) combo; here we surface the *target tiles* a player would
// see when hovering or arming a skill.

import { ActionKind, decodeAction } from "$lib/engine/action";

export interface SkillVariant {
  raw: number;
  target: number;
  /** `choice_idx` field. For Shove this is the push direction (0..=7). */
  choiceIdx: number;
  /** `focus_mode` bit. 0 = activation-range buff (default), 1 = effect-range
   *  buff. Only meaningful for Move-skills (Blast/Shove) when Focus is staged. */
  focusMode: boolean;
  /** `aux_sq` for retargeted skills (Focus → Shield/Dash/Retreat). 0 when unused. */
  auxSq: number;
  /** Whether `aux_sq` is meaningful for this variant. */
  hasAux: boolean;
}

export interface SkillTargetSet {
  /** Target squares legal for this caster + skill (any variant). */
  squares: Set<number>;
  /** Per target square, the raw u32 actions. May contain multiple variants
   *  (different choice_idx for Shove direction, different focus_mode bits). */
  byTarget: Map<number, number[]>;
  /** Per target square, fully decoded variant info. Same ordering as `byTarget`. */
  variantsByTarget: Map<number, SkillVariant[]>;
}

export function skillTargetsFor(
  legal: Uint32Array,
  src: number,
  skillId: number,
): SkillTargetSet {
  const squares = new Set<number>();
  const byTarget = new Map<number, number[]>();
  const variantsByTarget = new Map<number, SkillVariant[]>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    squares.add(a.target);
    const v: SkillVariant = {
      raw,
      target: a.target,
      choiceIdx: a.choiceIdx,
      focusMode: a.focusMode,
      auxSq: a.auxSq,
      hasAux: a.hasAux,
    };
    const list = byTarget.get(a.target);
    if (list) list.push(raw);
    else byTarget.set(a.target, [raw]);
    const vlist = variantsByTarget.get(a.target);
    if (vlist) vlist.push(v);
    else variantsByTarget.set(a.target, [v]);
  }
  return { squares, byTarget, variantsByTarget };
}

/** Whether *any* legal skill action exists for (src, skillId). Used to
 *  decide if a slice on the wheel is enabled. */
export function skillIsCastable(
  legal: Uint32Array,
  src: number,
  skillId: number,
): boolean {
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    return true;
  }
  return false;
}

/** Whether the (src, skillId) pair has variants distinguished by `focus_mode`.
 *  Only Blast (skill 10) and Shove (skill 11) under Focus have two distinct
 *  interpretations the player must choose between. */
export function hasFocusModeChoice(
  legal: Uint32Array,
  src: number,
  skillId: number,
): boolean {
  if (skillId !== 10 && skillId !== 11) return false;
  let sawActivation = false;
  let sawEffect = false;
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    if (a.focusMode) sawEffect = true;
    else sawActivation = true;
    if (sawActivation && sawEffect) return true;
  }
  return false;
}

/** Whether the (src, skillId) pair has retarget variants — i.e. Focus is
 *  staged and the engine emitted variants where `aux_sq` (or for non-aux
 *  skills, `target`) names a different recipient than the caster. Used to
 *  detect when a normally self-cast skill (Shield) needs to ARM and let the
 *  player pick a recipient instead of auto-firing on self. */
export function hasRetargetVariants(
  legal: Uint32Array,
  src: number,
  skillId: number,
): boolean {
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    // Focus-retargeted Shield/Dash/Retreat carry hasAux=true; aux_sq is the
    // recipient. (Non-Focus self-casts have target == src and hasAux=false.)
    if (a.hasAux && a.auxSq !== src) return true;
  }
  return false;
}

/** Whether (src, skillId) has BOTH a self-cast branch (no aux) and at least
 *  one retarget branch (aux to a different square). Distinct from
 *  `hasRetargetVariants`, which is "any retarget exists". Used to decide
 *  whether to show the Self/Ally picker for Focus-staged Shield/Dash/Retreat. */
export function hasSelfAndRetargetChoice(
  legal: Uint32Array,
  src: number,
  skillId: number,
): boolean {
  let sawSelf = false;
  let sawRetarget = false;
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    if (a.hasAux && a.auxSq !== src) sawRetarget = true;
    else sawSelf = true;
    if (sawSelf && sawRetarget) return true;
  }
  return false;
}

/** True iff this variant is a self-cast branch (no retarget aux). */
export function variantIsSelfCast(v: SkillVariant, src: number): boolean {
  return !(v.hasAux && v.auxSq !== src);
}

/** True iff this variant is a retarget branch where the recipient ally
 *  differs from the destination — i.e. Dash/Retreat retarget. For Shield
 *  retarget the recipient IS the target square, so `auxSq == target`. */
export function variantIsAllyMover(v: SkillVariant, src: number): boolean {
  if (!v.hasAux || v.auxSq === src) return false;
  return v.auxSq !== v.target;
}

/** For a retarget skill in "ally" mode, list the adjacent allies that have
 *  at least one legal destination. Returns ally squares in canonical
 *  ascending order. */
export function allyMoverCandidates(
  legal: Uint32Array,
  src: number,
  skillId: number,
): number[] {
  const set = new Set<number>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    if (!a.hasAux || a.auxSq === src) continue;
    if (a.auxSq === a.target) continue; // Shield: ally IS the target, not a mover.
    set.add(a.auxSq);
  }
  return [...set].sort((x, y) => x - y);
}

/** For a retarget skill in "ally" mode after the player has chosen an ally
 *  mover, list the destination squares that ally can reach. */
export function allyMoverDestinations(
  legal: Uint32Array,
  src: number,
  skillId: number,
  allySq: number,
  focusMode: boolean | null,
): Set<number> {
  const out = new Set<number>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    if (!a.hasAux || a.auxSq !== allySq) continue;
    if (focusMode !== null && a.focusMode !== focusMode) continue;
    out.add(a.target);
  }
  return out;
}

/** Look up the raw u32 for a Dash/Retreat retarget action with the given
 *  ally + destination + optional focus mode. */
export function rawForAllyMove(
  legal: Uint32Array,
  src: number,
  skillId: number,
  allySq: number,
  destSq: number,
  focusMode: boolean | null,
): number | null {
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    if (!a.hasAux || a.auxSq !== allySq) continue;
    if (a.target !== destSq) continue;
    if (focusMode !== null && a.focusMode !== focusMode) continue;
    return raw;
  }
  return null;
}
