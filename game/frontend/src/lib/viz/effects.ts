// Lightweight FX queue consumed by the Canvas overlay. Components push
// effect descriptors after each `tryApply` and the overlay drains them over
// the next ~700ms. Effects own no state outside the canvas itself.
//
// All effects carry an optional `ttl` field: when set, the overlay uses it
// as the lifetime in place of the FX_LIFETIME_MS default. Producers stamp
// this at creation time (scaled by animationSpeed via `fxLifetime(kind)`)
// so a cinematic viewer's flourishes run 2.5× as long as a normal viewer's
// without needing the renderer to track speed itself.

export type Effect =
  | {
      kind: "dust";
      /** path of squares walked, src → ... → final (inclusive). */
      path: number[];
      /** ms timestamp when this effect was created. */
      startedAt: number;
      /** Optional per-instance lifetime override; defaults to FX_LIFETIME_MS.dust. */
      ttl?: number;
    }
  | {
      kind: "impact";
      at: number;
      startedAt: number;
      ttl?: number;
    }
  | {
      kind: "damageNumber";
      at: number;
      amount: number;
      startedAt: number;
      ttl?: number;
    }
  | {
      kind: "shake";
      /** Square whose piece should shake briefly. */
      at: number;
      startedAt: number;
      ttl?: number;
    }
  | {
      kind: "heal";
      /** Square of the piece that was healed. */
      at: number;
      /** HP restored (currently always 1). */
      amount: number;
      startedAt: number;
      ttl?: number;
    }
  | {
      kind: "armor";
      /** Square of the piece that gained armor. */
      at: number;
      /** Armor delta (currently always 1, or -1 when stripped). */
      amount: number;
      startedAt: number;
      ttl?: number;
    }
  | SkillEffect
  | SpotlightEffect;

/** Per-skill choreography descriptors. `skillId` picks the drawing routine in
 *  EffectsLayer; `from`/`to` locate the animation on the board. Self-cast
 *  skills set `from === to`. See .claude/plans/skill-animations.md. */
export type SkillEffect = {
  kind: "skill";
  skillId: number;
  from: number;
  to: number;
  startedAt: number;
  ttl?: number;
  /** True when the action carried an aux square (Focus-retargeted Shield,
   *  Dash, or Retreat). The renderer uses this to switch e.g. self-Shield
   *  into an ally-thread flavour. */
  hasAux?: boolean;
  /** Aux square when `hasAux` is set. For Focus-retargeted Shield this is
   *  the ally receiving the buff. */
  auxSq?: number;
  /** Outcome-aware fields sampled from the post-state. Present when the
   *  ply-renderer could compute them (skill actions only). */
  outcome?: {
    /** True iff Steal actually moved money (target had cash). Steal renderer
     *  suppresses the coin-return glyph when false. */
    moneyStolen?: boolean;
    /** Actual post-move square of the *target* piece (used by Hook so the
     *  chain end tracks the pulled target rather than sticking on the
     *  original target square). */
    targetPostSq?: number;
    /** Actual post-cast square of the *caster* (Stack N strike-moves-caster:
     *  a Strike steps the caster 1 tile toward the target). When set and
     *  different from `from`, the strike renderers animate their caster-end
     *  from `from` → `casterPostSq` so lines/coins/bursts track the piece as
     *  it slides. Undefined for skills that don't move the caster. */
    casterPostSq?: number;
  };
};

/** A brief, subtle attention ring drawn on the CASTER square each time any
 *  skill fires. Purpose: draw the eye to the caster so Focus / Charge /
 *  Shield reads even when the user is looking elsewhere on the board. Kept
 *  quiet — a thin ink ring, not a bloom. */
export type SpotlightEffect = {
  kind: "spotlight";
  at: number;
  /** Category tint — same colour system as the skill's own choreography. */
  color: string;
  startedAt: number;
  ttl?: number;
};

export const FX_LIFETIME_MS = {
  dust: 650,
  impact: 450,
  damageNumber: 800,
  shake: 320,
  heal: 720,
  armor: 720,
  skill: 900,
  spotlight: 520,
} as const;
