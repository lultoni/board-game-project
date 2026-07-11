// Unit coverage for the caster-anchor interpolation added for Stack N
// strike-moves-caster. The per-skill canvas draw routines are visual and not
// unit-tested, but `casterAnchor` is pure geometry worth pinning: it decides
// where the caster-end of a strike choreography sits as the piece slides from
// its origin toward the target.

import { describe, it, expect } from "vitest";
import { casterAnchor, squareCenter } from "./skill-effects";
import type { SkillEffect } from "./effects";

const SIZE = 100;

function skillEff(partial: Partial<SkillEffect>): SkillEffect {
  return {
    kind: "skill",
    skillId: 1,
    from: 0,
    to: 16,
    startedAt: 0,
    ...partial,
  };
}

describe("casterAnchor", () => {
  it("returns the static from-centre when no casterPostSq is set", () => {
    const eff = skillEff({ from: 0, to: 16 });
    const c = squareCenter(0, SIZE);
    for (const t of [0, 0.5, 1]) {
      expect(casterAnchor(eff, SIZE, t)).toEqual(c);
    }
  });

  it("returns the static from-centre when casterPostSq === from (no step)", () => {
    const eff = skillEff({ from: 0, to: 16, outcome: { casterPostSq: 0 } });
    const c = squareCenter(0, SIZE);
    expect(casterAnchor(eff, SIZE, 0.5)).toEqual(c);
  });

  it("interpolates from → casterPostSq across t (0 → from, 1 → post)", () => {
    // Caster at sq 0 (a1) steps to sq 8 (a2, one rank up).
    const eff = skillEff({ from: 0, to: 16, outcome: { casterPostSq: 8 } });
    const from = squareCenter(0, SIZE);
    const post = squareCenter(8, SIZE);

    expect(casterAnchor(eff, SIZE, 0)).toEqual(from);
    expect(casterAnchor(eff, SIZE, 1)).toEqual(post);

    const mid = casterAnchor(eff, SIZE, 0.5);
    expect(mid.x).toBeCloseTo((from.x + post.x) / 2, 5);
    expect(mid.y).toBeCloseTo((from.y + post.y) / 2, 5);
  });

  it("clamps t outside [0,1] to the endpoints", () => {
    const eff = skillEff({ from: 0, to: 16, outcome: { casterPostSq: 8 } });
    const from = squareCenter(0, SIZE);
    const post = squareCenter(8, SIZE);
    expect(casterAnchor(eff, SIZE, -1)).toEqual(from);
    expect(casterAnchor(eff, SIZE, 2)).toEqual(post);
  });
});
