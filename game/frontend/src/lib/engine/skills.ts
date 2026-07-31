// Skill registry - mirrors `core_engine/src/game_logic/skills.rs`.
// Static metadata only; live legality comes from the engine's legal_actions.
//
// This table is a *synchronous* mirror: Svelte template expressions and
// `Object.values(SKILLS)` call sites cannot await an engine round-trip. To keep
// the mirror honest, `skills.contract.test.ts` asserts it byte-for-byte against
// the engine's `skill_metadata()` / `game_constants()` output - drift is a test
// failure, never a silent render bug.

export type SkillCategory = "strike" | "shield" | "move" | "mystic";
export type SkillTargetOwner = "enemy" | "ally" | "either" | "empty" | "self";

export interface SkillInfo {
  id: number;
  key: string; // i18n key root: skills.<key>.{name,desc}
  category: SkillCategory;
  cost: number;
  defaultRange: number;
  targetOwner: SkillTargetOwner;
  /** True iff this skill has distinct focus_mode=0/1 variants under Focus.
   *  Currently only Blast + Shove. Drives the focus-mode picker without the
   *  frontend hardcoding skill ids. */
  hasFocusModeChoice: boolean;
  /** True iff this skill opens the direction picker (choice_idx = push dir).
   *  Currently only Shove. */
  needsDirectionPick: boolean;
}

export const SKILLS: Record<number, SkillInfo> = {
  1:  { id: 1,  key: "lance",   category: "strike", cost: 2, defaultRange: 1, targetOwner: "enemy",  hasFocusModeChoice: false, needsDirectionPick: false },
  2:  { id: 2,  key: "hook",    category: "strike", cost: 3, defaultRange: 2, targetOwner: "enemy",  hasFocusModeChoice: false, needsDirectionPick: false },
  3:  { id: 3,  key: "break",   category: "strike", cost: 2, defaultRange: 2, targetOwner: "enemy",  hasFocusModeChoice: false, needsDirectionPick: false },
  4:  { id: 4,  key: "steal",   category: "strike", cost: 4, defaultRange: 2, targetOwner: "enemy",  hasFocusModeChoice: false, needsDirectionPick: false },
  5:  { id: 5,  key: "tempest", category: "strike", cost: 4, defaultRange: 2, targetOwner: "enemy",  hasFocusModeChoice: false, needsDirectionPick: false },
  6:  { id: 6,  key: "shield",  category: "shield", cost: 2, defaultRange: 0, targetOwner: "self",   hasFocusModeChoice: false, needsDirectionPick: false },
  7:  { id: 7,  key: "heal",    category: "shield", cost: 3, defaultRange: 1, targetOwner: "ally",   hasFocusModeChoice: false, needsDirectionPick: false },
  8:  { id: 8,  key: "plate",   category: "shield", cost: 3, defaultRange: 1, targetOwner: "ally",   hasFocusModeChoice: false, needsDirectionPick: false },
  9:  { id: 9,  key: "dash",    category: "move",   cost: 3, defaultRange: 2, targetOwner: "empty",  hasFocusModeChoice: false, needsDirectionPick: false },
  10: { id: 10, key: "blast",   category: "move",   cost: 2, defaultRange: 2, targetOwner: "enemy",  hasFocusModeChoice: true,  needsDirectionPick: false },
  11: { id: 11, key: "shove",   category: "move",   cost: 3, defaultRange: 3, targetOwner: "enemy",  hasFocusModeChoice: true,  needsDirectionPick: true  },
  12: { id: 12, key: "swap",    category: "move",   cost: 4, defaultRange: 2, targetOwner: "ally",   hasFocusModeChoice: false, needsDirectionPick: false },
  13: { id: 13, key: "retreat", category: "move",   cost: 4, defaultRange: 3, targetOwner: "empty",  hasFocusModeChoice: false, needsDirectionPick: false },
  14: { id: 14, key: "focus",   category: "mystic", cost: 2, defaultRange: 0, targetOwner: "self",   hasFocusModeChoice: false, needsDirectionPick: false },
  15: { id: 15, key: "charge",  category: "mystic", cost: 3, defaultRange: 0, targetOwner: "self",   hasFocusModeChoice: false, needsDirectionPick: false },
};

export function skillById(id: number): SkillInfo | undefined {
  return SKILLS[id];
}

/** How a skill's wheel half splits into two quarters when Focus is staged.
 *  - "focusMode": Blast/Shove — the +1 Range applies to activation-range
 *    (quarter A) OR effect-range (quarter B). Driven by `hasFocusModeChoice`.
 *  - "retarget": Shield/Dash/Retreat — Focus's +1 Range lets the caster
 *    channel the skill onto an adjacent ALLY (quarter B) instead of SELF
 *    (quarter A). These are self/movement skills whose reach grows to an ally.
 *  - null: no focus split (single half as before).
 *
 *  Type-driven (independent of whether both quarters are currently legal) so
 *  the wheel always advertises the choice; per-quarter legality greys the
 *  unavailable side. Mirrors the engine's generator: only Shield (SelfOnly) and
 *  Dash/Retreat (Empty) emit focus-retarget branches. */
export type FocusSplitKind = "focusMode" | "retarget";
const FOCUS_RETARGET_SKILL_KEYS = new Set(["shield", "dash", "retreat"]);

export function focusSplitKind(id: number): FocusSplitKind | null {
  const s = SKILLS[id];
  if (!s) return null;
  if (s.hasFocusModeChoice) return "focusMode";
  if (FOCUS_RETARGET_SKILL_KEYS.has(s.key)) return "retarget";
  return null;
}

/** Named skill ids derived from the table by key, so no magic integers leak
 *  into behavioural checks. Kept in sync by the contract test. */
export const SKILL_BLAST = Object.values(SKILLS).find((s) => s.key === "blast")!.id;
export const SKILL_SHOVE = Object.values(SKILLS).find((s) => s.key === "shove")!.id;

/** Total skill IDs (1..SKILL_COUNT). 0 = no skill in that slot. */
export const SKILL_COUNT = 15;

// modifier_bits from position.rs
export const MODIFIER_FOCUS = 0x01;
export const MODIFIER_CHARGE = 0x02;
export const MODIFIER_MOVE_ATTACK_USED = 0x04;

// Phase
export const PHASE_MOVE = 0;
export const PHASE_SKILL = 1;
export const PHASE_DRAFT = 2;

// Player
export const PLAYER_P1 = 0;
export const PLAYER_P2 = 1;

// GameResult
export const GAME_ONGOING = 0;
export const GAME_P1_WINS = 1;
export const GAME_P2_WINS = 2;

// Skill category colors - drives glyph tint on pieces, wheel slices, range
// overlays, and info-card accents. White outline + this fill = the canonical
// paper-aesthetic render.
export const CATEGORY_COLOR: Record<SkillCategory, string> = {
  strike: "#cc3a2a", // red - kill / damage
  shield: "#3a7acc", // blue - protection
  move:   "#3aaa55", // green - repositioning
  mystic: "#8a4abd", // purple - buffs / charge
};

/** Color for a skill's glyph (by id). Returns paper-cream for id 0. */
export function skillColor(id: number): string {
  const info = SKILLS[id];
  if (!info) return "#f8f1de";
  return CATEGORY_COLOR[info.category];
}

/** Self-cast skills don't need a target tile click - clicking the slice
 *  fires immediately. */
export function isSelfCast(id: number): boolean {
  const info = SKILLS[id];
  return info?.targetOwner === "self";
}
