// Mirror of `core_engine::game_logic::generator::bodyguard_guards_for`.
//
// Stack M rule: when a Champion or King is hit by a Move-Attack, an adjacent
// friendly Guard may intercept iff it is **dual-adjacent** — i.e. Chebyshev
// distance 1 to BOTH the defender AND the attacker's approach tile (the
// penultimate tile along the attack path). The k-th Guard in canonical
// ascending-square-index order corresponds to the action variant with
// `choice_idx = k`.
//
// We recompute the list in the frontend so we can show "which physical
// square" each Bodyguard-redirect variant points at, without having to plumb
// extra data through the engine surface.

import { bitboardHas } from "$lib/engine/mailbox";
import type { PositionView } from "$lib/engine/types";

function eightNeighbours(sq: number): number[] {
  const file = sq & 7;
  const rank = (sq >> 3) & 7;
  const out: number[] = [];
  for (let df = -1; df <= 1; df++) {
    for (let dr = -1; dr <= 1; dr++) {
      if (df === 0 && dr === 0) continue;
      const f = file + df;
      const r = rank + dr;
      if (f < 0 || f > 7 || r < 0 || r > 7) continue;
      out.push((r << 3) | f);
    }
  }
  return out;
}

/** Returns the squares of eligible Bodyguard Guards in canonical
 *  ascending order. Empty if no Bodyguard variants apply (defender is a
 *  Guard, no friendly Guard satisfies dual-adjacency, zigzag bypass, etc.). */
export function bodyguardGuardsFor(
  pos: PositionView,
  targetSq: number,
  approachSq: number,
): number[] {
  const p1 = pos.bitboards[0];
  const p2 = pos.bitboards[1];
  const kings = pos.bitboards[2];
  const champions = pos.bitboards[3];
  const guards = pos.bitboards[4];

  // Guards can't be Bodyguard-protected.
  if (bitboardHas(guards, targetSq)) return [];
  if (!bitboardHas(kings, targetSq) && !bitboardHas(champions, targetSq)) {
    return [];
  }
  // Defender owner = which side's bitboard contains target.
  const defenderIsP1 = bitboardHas(p1, targetSq);
  const defenderPieces = defenderIsP1 ? p1 : p2;

  const approachNeighbours = new Set(eightNeighbours(approachSq));

  const result: number[] = [];
  for (const n of eightNeighbours(targetSq)) {
    if (!approachNeighbours.has(n)) continue; // must be dual-adjacent
    if (!bitboardHas(guards, n)) continue;
    if (!bitboardHas(defenderPieces, n)) continue;
    result.push(n);
  }
  result.sort((a, b) => a - b);
  return result;
}
