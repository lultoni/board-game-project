// Pure-function coverage for the claim-win flow split in Phase 3e (M1).
// `computeClaimResultByte` is pure and side-effect-free - testing it here
// pins the seat→resultByte mapping that the orchestrator relies on.
import { describe, it, expect } from "vitest";
import { computeClaimResultByte } from "./match-store.svelte";

describe("computeClaimResultByte", () => {
  it("localSeat 0 → resultByte 0 regardless of role", () => {
    expect(computeClaimResultByte(0, "host")).toBe(0);
    expect(computeClaimResultByte(0, "joiner")).toBe(0);
  });

  it("localSeat 1 → resultByte 1 regardless of role", () => {
    expect(computeClaimResultByte(1, "host")).toBe(1);
    expect(computeClaimResultByte(1, "joiner")).toBe(1);
  });

  it("null localSeat falls back to role→seat (host=0, joiner=1)", () => {
    expect(computeClaimResultByte(null, "host")).toBe(0);
    expect(computeClaimResultByte(null, "joiner")).toBe(1);
  });

  it("null localSeat AND null role → resultByte 1 (joiner-by-default)", () => {
    // Defensive: the orchestrator gates on role !== null before calling, so
    // this branch is unreachable in prod. Verified here for completeness.
    expect(computeClaimResultByte(null, null)).toBe(1);
  });
});
