// Lightweight FX queue consumed by the Canvas overlay. Components push
// effect descriptors after each `tryApply` and the overlay drains them over
// the next ~700ms. Effects own no state outside the canvas itself.

export type Effect =
  | {
      kind: "dust";
      /** path of squares walked, src → ... → final (inclusive). */
      path: number[];
      /** ms timestamp when this effect was created. */
      startedAt: number;
    }
  | {
      kind: "impact";
      at: number;
      startedAt: number;
    }
  | {
      kind: "damageNumber";
      at: number;
      amount: number;
      startedAt: number;
    }
  | {
      kind: "shake";
      /** Square whose piece should shake briefly. */
      at: number;
      startedAt: number;
    }
  | {
      kind: "heal";
      /** Square of the piece that was healed. */
      at: number;
      /** HP restored (currently always 1). */
      amount: number;
      startedAt: number;
    }
  | {
      kind: "armor";
      /** Square of the piece that gained armor. */
      at: number;
      /** Armor delta (currently always 1, or -1 when stripped). */
      amount: number;
      startedAt: number;
    }
  | SkillEffect;

/** Per-skill choreography descriptors. `skillId` picks the drawing routine in
 *  EffectsLayer; `from`/`to` locate the animation on the board. Self-cast
 *  skills set `from === to`. See .claude/plans/skill-animations.md. */
export type SkillEffect = {
  kind: "skill";
  skillId: number;
  from: number;
  to: number;
  startedAt: number;
};

export const FX_LIFETIME_MS = {
  dust: 650,
  impact: 450,
  damageNumber: 800,
  shake: 320,
  heal: 720,
  armor: 720,
  skill: 900,
} as const;
