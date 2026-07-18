// Skill registry - mirrors `core_engine/src/game_logic/skills.rs`.
// Static metadata only; live legality comes from the engine's legal_actions.

export type SkillCategory = "strike" | "shield" | "move" | "mystic";
export type SkillTargetOwner = "enemy" | "ally" | "either" | "empty" | "self";

export interface SkillInfo {
  id: number;
  key: string; // i18n key root: skills.<key>.{name,desc}
  category: SkillCategory;
  cost: number;
  defaultRange: number;
  targetOwner: SkillTargetOwner;
}

export const SKILLS: Record<number, SkillInfo> = {
  1:  { id: 1,  key: "lance",   category: "strike", cost: 2, defaultRange: 1, targetOwner: "enemy" },
  2:  { id: 2,  key: "hook",    category: "strike", cost: 3, defaultRange: 2, targetOwner: "enemy" },
  3:  { id: 3,  key: "break",   category: "strike", cost: 2, defaultRange: 2, targetOwner: "enemy" },
  4:  { id: 4,  key: "steal",   category: "strike", cost: 4, defaultRange: 2, targetOwner: "enemy" },
  5:  { id: 5,  key: "tempest", category: "strike", cost: 4, defaultRange: 2, targetOwner: "enemy" },
  6:  { id: 6,  key: "shield",  category: "shield", cost: 2, defaultRange: 0, targetOwner: "self" },
  7:  { id: 7,  key: "heal",    category: "shield", cost: 3, defaultRange: 1, targetOwner: "ally" },
  8:  { id: 8,  key: "plate",   category: "shield", cost: 3, defaultRange: 1, targetOwner: "ally" },
  9:  { id: 9,  key: "dash",    category: "move",   cost: 3, defaultRange: 2, targetOwner: "empty" },
  10: { id: 10, key: "blast",   category: "move",   cost: 2, defaultRange: 2, targetOwner: "enemy" },
  11: { id: 11, key: "shove",   category: "move",   cost: 3, defaultRange: 3, targetOwner: "either" },
  12: { id: 12, key: "swap",    category: "move",   cost: 4, defaultRange: 2, targetOwner: "ally" },
  13: { id: 13, key: "retreat", category: "move",   cost: 4, defaultRange: 3, targetOwner: "empty" },
  14: { id: 14, key: "focus",   category: "mystic", cost: 2, defaultRange: 0, targetOwner: "self" },
  15: { id: 15, key: "charge",  category: "mystic", cost: 3, defaultRange: 0, targetOwner: "self" },
};

export function skillById(id: number): SkillInfo | undefined {
  return SKILLS[id];
}

/** Total skill IDs (1..SKILL_COUNT). 0 = no skill in that slot. */
export const SKILL_COUNT = 15;

// modifier_bits from position.rs
export const MODIFIER_FOCUS = 0x01;
export const MODIFIER_CHARGE = 0x02;

// Phase
export const PHASE_MOVE = 0;
export const PHASE_SKILL = 1;

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
