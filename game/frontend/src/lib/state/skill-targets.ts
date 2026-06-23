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
