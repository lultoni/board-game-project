// Unit coverage for the static Bodyguard-protection geometry (ns-40).
// Mirrors the engine's dual-adjacency rule: a Champion/King is protected from
// an adjacent approach square iff a friendly Guard is Chebyshev-adjacent to
// BOTH the Champion and that approach square.

import { describe, it, expect } from "vitest";
import { bodyguardCover, neighbours } from "./bodyguard-cover";
import type { PositionView } from "$lib/engine/types";

type Kind = "king" | "champion" | "guard";
type Owner = "p1" | "p2";

/** Build a minimal PositionView from a square→piece map. Mailbox cells are set
 *  to a non-zero value (hp 2) so occupancy reads as non-empty; bitboards drive
 *  owner/kind. */
function makeView(
  pieces: Record<number, { owner: Owner; kind: Kind }>,
): PositionView {
  const mailbox = new Uint16Array(64);
  let p1 = 0n, p2 = 0n, kings = 0n, champions = 0n, guards = 0n;
  for (const [sqStr, info] of Object.entries(pieces)) {
    const sq = Number(sqStr);
    mailbox[sq] = 2; // hp 2 → non-empty
    const bit = 1n << BigInt(sq);
    if (info.owner === "p1") p1 |= bit; else p2 |= bit;
    if (info.kind === "king") kings |= bit;
    else if (info.kind === "champion") champions |= bit;
    else guards |= bit;
  }
  return {
    bitboards: BigUint64Array.from([p1, p2, kings, champions, guards]),
    mailbox,
    toMove: 0,
    currentPhase: 0,
    actionsRemaining: 2,
    roundNumber: 1,
    p1Money: 0,
    p2Money: 0,
    pendingModifiers: 0,
    gameResult: 0,
    zobrist: 0n,
    pendingBodyguard: null,
    movedThisPhase: 0n,
  };
}

// Square helpers (file + rank*8). a1 = 0.
const sq = (file: number, rank: number) => (rank << 3) | file;

describe("neighbours", () => {
  it("returns 8 for a central square, 3 for a corner", () => {
    expect(neighbours(sq(3, 3)).length).toBe(8);
    expect(neighbours(sq(0, 0)).length).toBe(3);
  });
});

describe("bodyguardCover", () => {
  it("null position → empty", () => {
    expect(bodyguardCover(null)).toEqual([]);
  });

  it("a Champion with an adjacent friendly Guard is protected from covered approach squares", () => {
    // Champion at d4, friendly Guard at d5 (directly above).
    const champion = sq(3, 3);
    const guard = sq(3, 4);
    const cover = bodyguardCover(makeView({
      [champion]: { owner: "p1", kind: "champion" },
      [guard]: { owner: "p1", kind: "guard" },
    }));
    expect(cover.length).toBe(1);
    expect(cover[0].championSq).toBe(champion);
    // Covered approaches = empty neighbours of the Champion also adjacent to
    // the Guard: the three squares around d5's lower band - c4, e4, and the
    // guard's own-row diagonals c5/e5 are adjacent to guard too. Assert the
    // guard's tile is NOT an approach (occupied) and that all reported
    // approaches are adjacent to BOTH champion and guard.
    for (const a of cover[0].protectedApproaches) {
      const chebToChamp = Math.max(Math.abs((a & 7) - (champion & 7)), Math.abs(((a >> 3) & 7) - ((champion >> 3) & 7)));
      const chebToGuard = Math.max(Math.abs((a & 7) - (guard & 7)), Math.abs(((a >> 3) & 7) - ((guard >> 3) & 7)));
      expect(chebToChamp).toBe(1);
      expect(chebToGuard).toBe(1);
    }
    expect(cover[0].protectedApproaches).not.toContain(guard);
  });

  it("a Guard adjacent to the Champion but not to an approach yields no edge for far approaches (zig-zag bypass)", () => {
    // Champion at d4, Guard at d5. The square d3 (directly below the champion)
    // is NOT adjacent to the guard at d5 (cheb distance 2) → not protected.
    const champion = sq(3, 3);
    const guard = sq(3, 4);
    const below = sq(3, 2); // d3
    const cover = bodyguardCover(makeView({
      [champion]: { owner: "p1", kind: "champion" },
      [guard]: { owner: "p1", kind: "guard" },
    }));
    expect(cover[0].protectedApproaches).not.toContain(below);
  });

  it("an enemy Guard does not protect the Champion", () => {
    const champion = sq(3, 3);
    const enemyGuard = sq(3, 4);
    const cover = bodyguardCover(makeView({
      [champion]: { owner: "p1", kind: "champion" },
      [enemyGuard]: { owner: "p2", kind: "guard" },
    }));
    expect(cover).toEqual([]);
  });

  it("a Guard-typed defender is never protected (Bodyguard only shields Champions/Kings)", () => {
    const guardDefender = sq(3, 3);
    const friendlyGuard = sq(3, 4);
    const cover = bodyguardCover(makeView({
      [guardDefender]: { owner: "p1", kind: "guard" },
      [friendlyGuard]: { owner: "p1", kind: "guard" },
    }));
    expect(cover.find((c) => c.championSq === guardDefender)).toBeUndefined();
  });

  it("two Champions sharing one Guard each get their own distinct edge set", () => {    // Guard at d4 (center). Champions at d5 (above) and d3 (below) - both
    // adjacent to the guard. Each Champion is a separate cover entry keyed by
    // its own square, so the marks are unambiguous.
    const guard = sq(3, 3);
    const champA = sq(3, 4); // d5
    const champB = sq(3, 2); // d3
    const cover = bodyguardCover(makeView({
      [guard]: { owner: "p1", kind: "guard" },
      [champA]: { owner: "p1", kind: "champion" },
      [champB]: { owner: "p1", kind: "champion" },
    }));
    const entryA = cover.find((c) => c.championSq === champA);
    const entryB = cover.find((c) => c.championSq === champB);
    expect(entryA).toBeDefined();
    expect(entryB).toBeDefined();
    // Distinct entries; each champion's approaches are adjacent to itself.
    for (const a of entryA!.protectedApproaches) {
      const cheb = Math.max(Math.abs((a & 7) - (champA & 7)), Math.abs(((a >> 3) & 7) - ((champA >> 3) & 7)));
      expect(cheb).toBe(1);
    }
    for (const a of entryB!.protectedApproaches) {
      const cheb = Math.max(Math.abs((a & 7) - (champB & 7)), Math.abs(((a >> 3) & 7) - ((champB >> 3) & 7)));
      expect(cheb).toBe(1);
    }
  });

  it("an ENEMY piece standing on a covered approach square still shows (real threat)", () => {
    // Champion d4, friendly Guard d5, enemy champion on c5 (a covered approach:
    // adjacent to both d4 and d5). The mark should include c5.
    const champion = sq(3, 3);
    const guard = sq(3, 4);
    const enemyOnApproach = sq(2, 4); // c5
    const cover = bodyguardCover(makeView({
      [champion]: { owner: "p1", kind: "champion" },
      [guard]: { owner: "p1", kind: "guard" },
      [enemyOnApproach]: { owner: "p2", kind: "champion" },
    }));
    expect(cover[0].protectedApproaches).toContain(enemyOnApproach);
  });

  it("a FRIENDLY piece on a covered approach square suppresses the mark (not a vector)", () => {
    // Same as above but the piece on c5 is friendly - no attacker comes from an
    // ally's tile, so the mark is suppressed there.
    const champion = sq(3, 3);
    const guard = sq(3, 4);
    const allyOnApproach = sq(2, 4); // c5
    const cover = bodyguardCover(makeView({
      [champion]: { owner: "p1", kind: "champion" },
      [guard]: { owner: "p1", kind: "guard" },
      [allyOnApproach]: { owner: "p1", kind: "champion" },
    }));
    const entry = cover.find((c) => c.championSq === champion);
    expect(entry?.protectedApproaches ?? []).not.toContain(allyOnApproach);
  });
});
