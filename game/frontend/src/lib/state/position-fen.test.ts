import { describe, it, expect } from "vitest";
import {
  parseBoardSection,
  encodeBoardSection,
  parseToken,
  buildToken,
  mutateBoardToStaticFen,
} from "./position-fen";

// FEN with moved_this_phase = 0x1 (the P1 king at sq0 is flagged as moved),
// side-to-move P1, Move phase. Board: k at h8 (sq63), K at a1 (sq0).
const MOVED_BIT_FEN = "7k/8/8/8/8/8/8/K7 P1 M 2 6 6 0 1 0x1";

describe("position-fen board parse/encode", () => {
  it("round-trips an empty-ish board", () => {
    const sq = parseBoardSection("7k/8/8/8/8/8/8/K7");
    expect(sq[0]).toBe("K"); // a1
    expect(sq[63]).toBe("k"); // h8
    expect(encodeBoardSection(sq)).toBe("7k/8/8/8/8/8/8/K7");
  });

  it("parses and rebuilds bracketed mailbox tokens", () => {
    const info = parseToken("C[1/2/0/7/15]");
    expect(info).toEqual({ char: "C", hp: 1, armor: 2, s1: 7, s2: 15 });
    expect(buildToken("C", 1, 2, 7, 15)).toBe("C[1/2/0/7/15]");
    // defaults collapse to a bare char
    expect(buildToken("G", 2, 0, 0, 0)).toBe("G");
  });
});

describe("mutateBoardToStaticFen — the P9-A fix", () => {
  it("zeroes moved_this_phase when moving the flagged piece", () => {
    // Move the P1 king a1 -> b1 (sq0 -> sq1). The moved bit (sq0) would be
    // stranded; the fix must force field 8 to 0x0.
    const out = mutateBoardToStaticFen(MOVED_BIT_FEN, (sq) => {
      sq[1] = sq[0];
      sq[0] = "";
    });
    const fields = out.split(" ");
    expect(fields[8]).toBe("0x0");            // moved_this_phase zeroed
    expect(fields.length).toBe(9);            // trailers dropped, canonical 9
    expect(fields[0]).toBe("7k/8/8/8/8/8/8/1K6"); // king now on b1
  });

  it("zeroes moved_this_phase when removing the flagged piece", () => {
    const out = mutateBoardToStaticFen(MOVED_BIT_FEN, (sq) => { sq[0] = ""; });
    expect(out.split(" ")[8]).toBe("0x0");
  });

  it("preserves money/round/modifier fields (4-7) unchanged", () => {
    const out = mutateBoardToStaticFen(MOVED_BIT_FEN, () => {});
    const f = out.split(" ");
    expect(f.slice(1, 8)).toEqual(["P1", "M", "2", "6", "6", "0", "1"]);
  });

  it("drops turn-scoped trailer fields (9+)", () => {
    // Append fake trailers (tracked_enemies etc.) and confirm they're stripped.
    const withTrailers = MOVED_BIT_FEN + " 3,4 5 2 12";
    const out = mutateBoardToStaticFen(withTrailers, () => {});
    expect(out.split(" ").length).toBe(9);
  });

  it("edits a mailbox in place and stays a valid 9-field static FEN", () => {
    // Set HP of the king at sq0 to 1 via a token rewrite.
    const out = mutateBoardToStaticFen(MOVED_BIT_FEN, (sq) => {
      sq[0] = buildToken("K", 1, 0, 0, 0);
    });
    const f = out.split(" ");
    expect(f[8]).toBe("0x0");
    expect(f[0]).toContain("K[1/0/0/0/0]");
  });
});

describe("full-board round-trip preserves kings (regression #31)", () => {
  // Canonical stack-M start position (bare tokens).
  const START = "1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCC1 P1 M 2 6 6 0 1 0x0";

  it("parse→encode preserves both kings unchanged", () => {
    const sq = parseBoardSection(START.split(" ")[0]);
    // find the two kings
    const kings = sq.filter((t) => t === "K" || t === "k").length;
    expect(kings).toBe(2);
    expect(encodeBoardSection(sq)).toBe(START.split(" ")[0]);
  });

  it("editing a guard's HP does not drop a king", () => {
    const sq = parseBoardSection(START.split(" ")[0]);
    // find a P1 guard (G) and set HP=1 via a bracket token
    const gIdx = sq.findIndex((t) => t === "G");
    expect(gIdx).toBeGreaterThanOrEqual(0);
    sq[gIdx] = buildToken("G", 1, 0, 0, 0);
    const board = encodeBoardSection(sq);
    const reparsed = parseBoardSection(board);
    const kings = reparsed.filter((t) => t[0] === "K" || t[0] === "k").length;
    expect(kings).toBe(2);
  });

  it("mutateBoardToStaticFen editing a guard keeps kings + valid board", () => {
    const out = mutateBoardToStaticFen(START, (sq) => {
      const gIdx = sq.findIndex((t) => t === "G");
      sq[gIdx] = buildToken("G", 1, 0, 0, 0);
    });
    const reparsed = parseBoardSection(out.split(" ")[0]);
    const kings = reparsed.filter((t) => t[0] === "K" || t[0] === "k").length;
    expect(kings).toBe(2);
  });
});
