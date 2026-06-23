// Pure helpers for filtering legal skill actions by caster + skill. The
// engine emits one Skill action per (src, skill_id, target, choice_idx,
// modifier_bits) combo; here we surface the *target tiles* a player would
// see when hovering or arming a skill.

import { ActionKind, decodeAction } from "$lib/engine/action";

export interface SkillTargetSet {
  /** Target squares legal for this caster + skill (any modifier). */
  squares: Set<number>;
  /** Per target square, the raw u32 actions. May contain multiple variants
   *  (different choice_idx for Bodyguard / different modifier_bits). */
  byTarget: Map<number, number[]>;
}

export function skillTargetsFor(
  legal: Uint32Array,
  src: number,
  skillId: number,
): SkillTargetSet {
  const squares = new Set<number>();
  const byTarget = new Map<number, number[]>();
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    squares.add(a.target);
    const list = byTarget.get(a.target);
    if (list) {
      list.push(raw);
    } else {
      byTarget.set(a.target, [raw]);
    }
  }
  return { squares, byTarget };
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
