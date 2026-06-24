import { describe, expect, it } from "vitest";
import {
  extractPostZobristForPly,
  extractStartZobrist,
  extractResumeStateFromLog,
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

describe("extractResumeStateFromLog", () => {
  it("returns plyCount + last ply post_zobrist", () => {
    const log = makeMatchLog({
      startZobrist: "100",
      plies: [
        { plyNo: 1, postZobrist: "111" },
        { plyNo: 2, postZobrist: "222" },
      ],
    });
    expect(extractResumeStateFromLog(log)).toEqual({ plyCount: 2, zobrist: "222" });
  });

  it("for plyCount=0 returns start_zobrist", () => {
    const log = makeMatchLog({ startZobrist: "777", plies: [], totalPlies: 0 });
    expect(extractResumeStateFromLog(log)).toEqual({ plyCount: 0, zobrist: "777" });
  });

  it("falls back to zobrist=\"0\" when last ply is missing", () => {
    // total_plies says 5 but only ply 1 is present (truncated log).
    const log = makeMatchLog({
      startZobrist: "100",
      plies: [{ plyNo: 1, postZobrist: "111" }],
      totalPlies: 5,
    });
    expect(extractResumeStateFromLog(log)).toEqual({ plyCount: 5, zobrist: "0" });
  });

  it("survives a round-trip through JSON.parse + JSON.stringify (whitespace + key reorder)", () => {
    const orig = makeMatchLog({
      startZobrist: "5000",
      plies: [
        { plyNo: 1, postZobrist: "5001" },
        { plyNo: 2, postZobrist: "5002" },
      ],
    });
    // parse + serialise will normalise whitespace; we expect the regex to keep working.
    // We can't preserve u64 precision through JSON.parse (that's the whole reason for
    // string-extraction), so we hand-build an "equivalent re-serialised" version.
    const reSerialised = `{"start_zobrist": 5000, "total_plies": 2, "plies": [ {"action_raw":1234,"ply_no":1,"post_zobrist":5001,"side":0}, {"action_raw":1234,"ply_no":2,"post_zobrist":5002,"side":0} ]}`;
    expect(extractResumeStateFromLog(reSerialised)).toEqual({ plyCount: 2, zobrist: "5002" });
  });
});
