import { describe, it, expect } from "vitest";
import { producerRawsFromLog, producerMetaFromLog, snapshotActionCount } from "./ai-service";

describe("producerRawsFromLog", () => {
  it("returns [] for null / empty", () => {
    expect(producerRawsFromLog(null)).toEqual([]);
    expect(producerRawsFromLog("")).toEqual([]);
  });

  it("returns [] for malformed JSON", () => {
    expect(producerRawsFromLog("{not json")).toEqual([]);
  });

  it("extracts raw actions in order", () => {
    const log = JSON.stringify({
      start_fen: "start",
      plies: [
        { action: { raw: 10 } },
        { action: { raw: 20 } },
        { action: { raw: 30 } },
      ],
    });
    expect(producerRawsFromLog(log)).toEqual([10, 20, 30]);
  });

  it("stops at the first malformed ply (partial log tail)", () => {
    // A partially-written log snapshot: a trailing ply missing action.raw
    // should truncate the returned list rather than inject a bogus 0.
    const log = JSON.stringify({
      plies: [
        { action: { raw: 5 } },
        { action: {} },
        { action: { raw: 99 } },
      ],
    });
    expect(producerRawsFromLog(log)).toEqual([5]);
  });

  it("normalises raws via >>> 0", () => {
    const log = JSON.stringify({ plies: [{ action: { raw: 0x80000000 } }] });
    expect(producerRawsFromLog(log)).toEqual([0x80000000]);
  });

  it("handles an empty plies array (producer just started)", () => {
    expect(producerRawsFromLog(JSON.stringify({ plies: [] }))).toEqual([]);
    expect(producerRawsFromLog(JSON.stringify({}))).toEqual([]);
  });
});

describe("producerMetaFromLog (BUG-2 AIvAI pills)", () => {
  it("returns [] for null / empty / malformed", () => {
    expect(producerMetaFromLog(null)).toEqual([]);
    expect(producerMetaFromLog("")).toEqual([]);
    expect(producerMetaFromLog("{not json")).toEqual([]);
  });

  it("extracts depth + P1-POV score aligned with the raw list", () => {
    const log = JSON.stringify({
      plies: [
        { action: { raw: 10 }, ai: { depth: 6, score_cp: 120 } },
        { action: { raw: 20 }, ai: { depth: 8, score_cp: -45 } },
      ],
    });
    expect(producerMetaFromLog(log)).toEqual([
      { depth: 6, scoreCp: 120 },
      { depth: 8, scoreCp: -45 },
    ]);
  });

  it("stays index-aligned with producerRawsFromLog by truncating at the same ply", () => {
    // A partial-tail log: last ply has no raw yet. Both extractors must stop
    // at the same index so producerMetas[i] describes producerRaws[i].
    const log = JSON.stringify({
      plies: [
        { action: { raw: 5 }, ai: { depth: 4, score_cp: 0 } },
        { action: {}, ai: { depth: 9, score_cp: 99 } },
      ],
    });
    expect(producerRawsFromLog(log)).toEqual([5]);
    expect(producerMetaFromLog(log)).toEqual([{ depth: 4, scoreCp: 0 }]);
  });

  it("emits null for a ply that carries no ai meta (legacy / not-yet-flushed)", () => {
    const log = JSON.stringify({
      plies: [
        { action: { raw: 1 } },                              // no ai
        { action: { raw: 2 }, ai: null },                    // explicit null
        { action: { raw: 3 }, ai: { depth: 7, score_cp: 10 } },
      ],
    });
    expect(producerMetaFromLog(log)).toEqual([
      null,
      null,
      { depth: 7, scoreCp: 10 },
    ]);
  });

  it("maps a missing score_cp (mate) to null score, keeping depth", () => {
    const log = JSON.stringify({
      plies: [{ action: { raw: 1 }, ai: { depth: 12 } }],
    });
    expect(producerMetaFromLog(log)).toEqual([{ depth: 12, scoreCp: null }]);
  });
});

describe("snapshotActionCount", () => {
  it("returns 0 for null / malformed / missing actions", () => {
    expect(snapshotActionCount(null)).toBe(0);
    expect(snapshotActionCount("{bad")).toBe(0);
    expect(snapshotActionCount(JSON.stringify({ start_fen: "x" }))).toBe(0);
    expect(snapshotActionCount(JSON.stringify({ actions: "nope" }))).toBe(0);
  });

  it("counts the baked-in actions (the log-player's starting offset)", () => {
    // A post-draft snapshot carries the 12 draft raws in `actions`; the view
    // engine restored from it is already past the draft, so the log-player must
    // start at ply 12 and not re-apply a draft raw onto a Move-phase engine.
    const snap = JSON.stringify({
      start_fen: "start",
      config: {},
      actions: Array.from({ length: 12 }, (_, i) => 0x40000000 | i),
    });
    expect(snapshotActionCount(snap)).toBe(12);
    expect(snapshotActionCount(JSON.stringify({ actions: [] }))).toBe(0);
  });
});
