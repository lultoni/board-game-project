// Static Bodyguard-protection geometry for the always-on board indicator
// (DB next_step-40). Mirrors the engine's `bodyguard_guards_for`
// (game/crates/core_engine/src/game_logic/generator.rs): a Champion/King C is
// protected from an adjacent approach square A iff a friendly Guard G is
// Chebyshev-adjacent to BOTH C and A. This is a pure, side-effect-free read of
// a PositionView so it can be unit-tested and recomputed reactively.
//
// The indicator is anchored to the CHAMPION's own tile edge (the edge between C
// and A), NOT to the shared Guard or the approach square. That is what keeps
// two Champions guarded by the same Guard unambiguous: each Champion's marks
// live on its own tile boundary, so the owner of a mark is always the piece it
// hugs - even when their protected approach squares coincide.

import { bitboardHas, readPieces, type Owner } from "$lib/engine/mailbox";
import type { PositionView } from "$lib/engine/types";

/** Chebyshev-1 neighbours of `sq` (2..8 of them near edges/corners). */
export function neighbours(sq: number): number[] {
  const f = sq & 7;
  const r = (sq >> 3) & 7;
  const out: number[] = [];
  for (let df = -1; df <= 1; df++) {
    for (let dr = -1; dr <= 1; dr++) {
      if (df === 0 && dr === 0) continue;
      const nf = f + df, nr = r + dr;
      if (nf < 0 || nf > 7 || nr < 0 || nr > 7) continue;
      out.push((nr << 3) | nf);
    }
  }
  return out;
}

function chebyshev(a: number, b: number): number {
  const dx = Math.abs((a & 7) - (b & 7));
  const dy = Math.abs(((a >> 3) & 7) - ((b >> 3) & 7));
  return Math.max(dx, dy);
}

/** One protected Champion/King and the approach squares it is shielded from.
 *  `championSq` is the defender; each entry of `protectedApproaches` is an
 *  adjacent square an attacker could stop on that a friendly Guard covers. The
 *  Board renders an edge mark between `championSq` and each approach square. */
export interface BodyguardCover {
  championSq: number;
  owner: Owner;
  protectedApproaches: number[];
}

/** Compute Bodyguard cover for every protected Champion/King on the board.
 *  Only Champions and Kings are eligible defenders; only friendly (same-owner)
 *  Guards protect them. An approach square must be empty (an attacker has to be
 *  able to stand there) and satisfy dual-adjacency with some friendly Guard. */
export function bodyguardCover(position: PositionView | null): BodyguardCover[] {
  if (!position) return [];
  const pieces = readPieces(position.bitboards, position.mailbox);
  const guards = position.bitboards[4];
  const p1 = position.bitboards[0];

  // Owner of the piece on each occupied square (for the approach-square test).
  const ownerAt = new Map<number, Owner>();
  for (const p of pieces) ownerAt.set(p.square, p.owner);

  const out: BodyguardCover[] = [];
  for (const c of pieces) {
    if (c.kind === "guard") continue; // only Champions/Kings are defended
    // Friendly Guards adjacent to the defender are the candidate interceptors.
    const cNeighbours = neighbours(c.square);
    const friendlyGuardsNearC = cNeighbours.filter(
      (n) =>
        bitboardHas(guards, n) &&
        (bitboardHas(p1, n) ? "p1" : "p2") === c.owner,
    );
    if (friendlyGuardsNearC.length === 0) continue;

    // An adjacent approach square A is protected iff some friendly Guard near C
    // is also adjacent to A. The square may be empty OR hold an enemy piece (a
    // real threat standing on a covered square still reads as "protected from
    // here"). Only a FRIENDLY piece on A suppresses the mark - an ally there is
    // not an attack vector.
    const protectedApproaches: number[] = [];
    for (const a of cNeighbours) {
      const occupant = ownerAt.get(a);
      if (occupant === c.owner) continue; // friendly piece blocks the vector
      const covered = friendlyGuardsNearC.some((g) => chebyshev(g, a) === 1);
      if (covered) protectedApproaches.push(a);
    }
    if (protectedApproaches.length > 0) {
      out.push({ championSq: c.square, owner: c.owner, protectedApproaches });
    }
  }
  return out;
}
