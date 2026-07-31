import { describe, it, expect } from "vitest";
import { normalisePositionView, type PositionViewDto } from "./tauri-client";

/** Minimal well-formed DTO; individual tests override the fields they exercise. */
function baseDto(overrides: Partial<PositionViewDto> = {}): PositionViewDto {
  return {
    bitboards: ["0", "0", "0", "0", "0"],
    mailbox: new Array(64).fill(0),
    toMove: 0,
    currentPhase: 0,
    actionsRemaining: 2,
    roundNumber: 1,
    p1Money: 0,
    p2Money: 0,
    pendingModifiers: 0,
    gameResult: 0,
    zobrist: "0",
    ...overrides,
  };
}

describe("normalisePositionView movedThisPhase (P2-E)", () => {
  it("round-trips a non-zero bitboard string to bigint", () => {
    // bit 10 and bit 42 set → pieces on those squares already moved this phase.
    const bb = (1n << 10n) | (1n << 42n);
    const view = normalisePositionView(baseDto({ movedThisPhase: bb.toString() }));
    expect(view.movedThisPhase).toBe(bb);
  });

  it("handles the full 64-bit range without precision loss", () => {
    const bb = (1n << 63n) | 1n;
    const view = normalisePositionView(baseDto({ movedThisPhase: bb.toString() }));
    expect(view.movedThisPhase).toBe(bb);
  });

  it("defaults to 0n when the field is absent (older backend)", () => {
    const dto = baseDto();
    delete (dto as { movedThisPhase?: unknown }).movedThisPhase;
    const view = normalisePositionView(dto);
    expect(view.movedThisPhase).toBe(0n);
  });
});
