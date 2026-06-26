import { describe, expect, it } from "vitest";
import {
  logIsMidDraftCheap,
  snapshotJsonFromMatchLog,
} from "./multiplayer-resume";

describe("snapshotJsonFromMatchLog", () => {
  it("rebuilds a Snapshot JSON from a persisted MatchLog", () => {
    const log = JSON.stringify({
      start_fen: "8/8/…",
      config: { board_w: 8, board_h: 8 },
      plies: [
        { action: { raw: 11 } },
        { action: { raw: 22 } },
      ],
    });
    const snap = snapshotJsonFromMatchLog(log);
    expect(snap).not.toBeNull();
    expect(JSON.parse(snap!)).toEqual({
      start_fen: "8/8/…",
      actions: [11, 22],
      config: { board_w: 8, board_h: 8 },
    });
  });

  it("returns null when start_fen is missing", () => {
    expect(snapshotJsonFromMatchLog('{"plies":[]}')).toBeNull();
  });

  it("returns null when a ply action.raw is malformed", () => {
    const log = JSON.stringify({
      start_fen: "8/8",
      config: {},
      plies: [{ action: { raw: -1 } }],
    });
    expect(snapshotJsonFromMatchLog(log)).toBeNull();
  });
});

describe("logIsMidDraftCheap", () => {
  it("returns true for fewer than 12 plies", () => {
    for (let n = 0; n < 12; n++) {
      const log = JSON.stringify({
        start_fen: "8/8",
        config: {},
        plies: Array.from({ length: n }, (_, i) => ({ action: { raw: i } })),
      });
      expect(logIsMidDraftCheap(log)).toBe(true);
    }
  });

  it("returns false at exactly 12 plies (draft complete)", () => {
    const log = JSON.stringify({
      start_fen: "8/8",
      config: {},
      plies: Array.from({ length: 12 }, (_, i) => ({ action: { raw: i } })),
    });
    expect(logIsMidDraftCheap(log)).toBe(false);
  });

  it("returns false for malformed input", () => {
    expect(logIsMidDraftCheap("not json")).toBe(false);
    expect(logIsMidDraftCheap("null")).toBe(false);
    expect(logIsMidDraftCheap("{}")).toBe(false);
  });
});
