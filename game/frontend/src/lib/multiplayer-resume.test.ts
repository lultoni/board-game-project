import { describe, expect, it } from "vitest";
import {
  extractPostZobristForPly,
  extractStartZobrist,
  logIsMidDraftCheap,
  snapshotJsonFromMatchLog,
} from "./multiplayer-resume";

// These tests pin the engine's MatchLog v1 JSON shape to the resume regexes.
// If any test starts failing because the engine renamed/restructured a field,
// update the regexes in multiplayer-resume.ts AND the fixtures here in lockstep.

/** Construct a minimal MatchLog JSON shape that mirrors what
 *  core_engine emits via serde. `ply_no` is 1-indexed (see session.rs:360). */
function makeMatchLog(opts: {
  startZobrist: string;
  plies: { plyNo: number; postZobrist: string }[];
  totalPlies?: number;
}): string {
  const totalPlies = opts.totalPlies ?? opts.plies.length;
  const plies = opts.plies.map((p) =>
    `{"ply_no":${p.plyNo},"side":0,"action_raw":1234,"post_zobrist":${p.postZobrist}}`,
  );
  return `{"start_zobrist":${opts.startZobrist},"total_plies":${totalPlies},"plies":[${plies.join(",")}]}`;
}

describe("extractStartZobrist", () => {
  it("returns the digit string", () => {
    const log = makeMatchLog({ startZobrist: "9876543210123456789", plies: [] });
    expect(extractStartZobrist(log)).toBe("9876543210123456789");
  });

  it("returns null for missing field", () => {
    expect(extractStartZobrist("{}")).toBeNull();
  });

  it("tolerates whitespace and reordered keys", () => {
    const log = `{ "total_plies"  :  0 ,  "start_zobrist" : 42 }`;
    expect(extractStartZobrist(log)).toBe("42");
  });
});

describe("extractPostZobristForPly (1-indexed ply_no)", () => {
  const log = makeMatchLog({
    startZobrist: "100",
    plies: [
      { plyNo: 1, postZobrist: "111" },
      { plyNo: 2, postZobrist: "222" },
      { plyNo: 3, postZobrist: "333" },
    ],
  });

  it("returns post_zobrist for ply 1", () => {
    expect(extractPostZobristForPly(log, 1)).toBe("111");
  });

  it("returns post_zobrist for ply 2", () => {
    expect(extractPostZobristForPly(log, 2)).toBe("222");
  });

  it("returns post_zobrist for ply 3", () => {
    expect(extractPostZobristForPly(log, 3)).toBe("333");
  });

  it("returns null for a ply that doesn't exist", () => {
    expect(extractPostZobristForPly(log, 4)).toBeNull();
  });

  it("does not confuse ply_no with other numeric fields", () => {
    // action_raw and post_zobrist also carry the digit `2`; the anchor regex
    // must still pick the right ply via `ply_no`.
    expect(extractPostZobristForPly(log, 2)).toBe("222");
  });

  it("survives whitespace around the colon and value", () => {
    const padded = log.replace(/"ply_no":\s*2/g, '"ply_no"  :  2 ');
    expect(extractPostZobristForPly(padded, 2)).toBe("222");
  });
});

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
