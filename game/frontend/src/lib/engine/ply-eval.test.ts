import { describe, it, expect } from "vitest";
import { plyEvalOf, formatPlyEval } from "./ply-eval";

describe("plyEvalOf", () => {
  it("returns null for a ply with no assessment", () => {
    expect(plyEvalOf(null)).toBeNull();
    expect(plyEvalOf(undefined)).toBeNull();
    expect(plyEvalOf({})).toBeNull();
    expect(plyEvalOf({ ai: null, background_eval: null })).toBeNull();
  });

  it("prefers `ai` (AI ply) over background_eval", () => {
    const e = plyEvalOf({
      ai: { depth: 8, score_cp: 120, was_mate: false },
      background_eval: { depth: 4, score_cp: 50, was_mate: false },
    });
    expect(e).not.toBeNull();
    expect(e!.source).toBe("ai");
    expect(e!.depth).toBe(8);
    expect(e!.scoreCp).toBe(120);
  });

  it("falls back to background_eval for a human ply", () => {
    const e = plyEvalOf({
      background_eval: { depth: 6, score_cp: -30, was_mate: false },
    });
    expect(e!.source).toBe("background");
    expect(e!.depth).toBe(6);
    expect(e!.scoreCp).toBe(-30);
  });

  it("carries mate info", () => {
    const e = plyEvalOf({ ai: { depth: 10, was_mate: true, mate_in: 3, score_cp: null } });
    expect(e!.wasMate).toBe(true);
    expect(e!.mateIn).toBe(3);
    expect(e!.scoreCp).toBeNull();
  });
});

describe("formatPlyEval", () => {
  it("formats a positive score with depth", () => {
    expect(formatPlyEval(plyEvalOf({ ai: { depth: 8, score_cp: 120, was_mate: false } })))
      .toBe("AI +120 (d8)");
  });
  it("formats a negative background score", () => {
    expect(formatPlyEval(plyEvalOf({ background_eval: { depth: 6, score_cp: -30, was_mate: false } })))
      .toBe("engine -30 (d6)");
  });
  it("formats a mate", () => {
    expect(formatPlyEval(plyEvalOf({ ai: { depth: 10, was_mate: true, mate_in: 3, score_cp: null } })))
      .toBe("AI mate in 3 (d10)");
  });
  it("returns null for no eval", () => {
    expect(formatPlyEval(null)).toBeNull();
  });
});
