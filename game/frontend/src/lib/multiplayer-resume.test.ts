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
  // Minimal FEN stubs - only the phase field (3rd token) matters here.
  const draftFen = "8/8 w D 0 0 1 0x0";
  const moveFen  = "8/8 w M 0 0 1 0x0";

  it("returns true for drafted mode with all DraftTurn plies", () => {
    const log = JSON.stringify({
      start_fen: draftFen,
      config: {},
      plies: Array.from({ length: 6 }, () => ({ action: { kind: "DraftTurn" } })),
    });
    expect(logIsMidDraftCheap(log)).toBe(true);
  });

  it("returns true for drafted mode with an empty ply list (draft not yet started)", () => {
    const log = JSON.stringify({ start_fen: draftFen, config: {}, plies: [] });
    expect(logIsMidDraftCheap(log)).toBe(true);
  });

  it("returns false for preMade with an empty ply list - start_fen phase is M, not D", () => {
    // Regression: kind-only inspection returned true for empty plies, so a
    // preMade match right after setup (no game plies yet) got routed to
    // /draft/. start_fen's phase field distinguishes: preMade opens in phase
    // M via new_with_loadouts.
    const log = JSON.stringify({ start_fen: moveFen, config: {}, plies: [] });
    expect(logIsMidDraftCheap(log)).toBe(false);
  });

  it("returns false for preMade with 2 Move plies", () => {
    // Regression for the plies.length<12 heuristic: preMade skips /draft/
    // entirely and can be in play with <12 plies. Route was `../draft/` →
    // draft's stale-entry guard bounced back to /match/, running mp teardown
    // and killing the fresh rejoin WS.
    const log = JSON.stringify({
      start_fen: moveFen,
      config: {},
      plies: [
        { action: { kind: "Move" } },
        { action: { kind: "Move" } },
      ],
    });
    expect(logIsMidDraftCheap(log)).toBe(false);
  });

  it("returns false for a full 12-ply draft followed by game plies", () => {
    const log = JSON.stringify({
      start_fen: draftFen,
      config: {},
      plies: [
        ...Array.from({ length: 12 }, () => ({ action: { kind: "DraftTurn" } })),
        { action: { kind: "Skill" } },
      ],
    });
    expect(logIsMidDraftCheap(log)).toBe(false);
  });

  it("returns false when start_fen is missing", () => {
    const log = JSON.stringify({ config: {}, plies: [] });
    expect(logIsMidDraftCheap(log)).toBe(false);
  });

  it("returns false for malformed input", () => {
    expect(logIsMidDraftCheap("not json")).toBe(false);
    expect(logIsMidDraftCheap("null")).toBe(false);
    expect(logIsMidDraftCheap("{}")).toBe(false);
  });
});
