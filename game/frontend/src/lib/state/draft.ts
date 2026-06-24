// L8 — draft / loadout helpers used by /setup/, /draft/, and /match/.
//
// The big makeshift-draft helpers that lived here (presets, merge,
// FEN-rewrite glue) are gone — /draft/ now drives the real Phase::Draft
// flow on the engine via DraftTurn actions, and pre-made mode goes through
// `createEngineWithLoadouts`. What remains: the canonical Stack M piece-
// squares, a `squareName` helper, the pre-made loadout catalogue, and the
// designer-readiness check for the /setup/ picker.

import { SKILLS, SKILL_COUNT } from "$lib/engine/skills";
import type { Owner } from "$lib/engine/mailbox";
import type { SideLoadout } from "$lib/engine/types";
import type { PreMadeLoadoutId } from "$lib/state/match-store.svelte";

/** Two skill IDs (1..15). 0 = empty slot. */
export type Loadout = [number, number];

export function isStrike(id: number): boolean {
  return SKILLS[id]?.category === "strike";
}

/** Map an `Owner` to the squares its King + Champions occupy in the Stack M
 *  canonical starting position. */
export const STACK_M_LOADOUT_SQUARES: Record<Owner, number[]> = {
  // rank 1 (P1 back): files b..g → squares 1..6. King on d1 = sq 3,
  // Champions on b1, c1, e1, f1, g1 = sq 1, 2, 4, 5, 6.
  p1: [3 /* K */, 1, 2, 4, 5, 6],
  // rank 8 (P2 back): files b..g → squares 57..62. King on e8 = sq 60,
  // Champions on b8, c8, d8, f8, g8 = 57, 58, 59, 61, 62.
  p2: [60 /* K */, 57, 58, 59, 61, 62],
};

/** Human-readable square label (e.g. 0 → "a1", 60 → "e8"). */
export function squareName(sq: number): string {
  const file = "abcdefgh"[sq & 7];
  const rank = ((sq >> 3) & 7) + 1;
  return `${file}${rank}`;
}

// === L8 — pre-made loadouts (OQ-65) ========================================
//
// Each entry is a `SideLoadout`: 6 `[skill1, skill2]` pairs in canonical
// piece order (King at index 0, Champions 1..5 by ascending starting sq).
// Both sides play the same loadout — pre-made mode is a mirror match.
//
// **PLACEHOLDER DATA.** The values below are all-zero stand-ins that will
// be rejected by the engine's `validate_loadout` (skill 0 = empty slot is
// only legal during Phase::Draft). The designer fills these in with curated
// "First / Second / Third game" loadouts at the end of L8 (see DB OQ-65 /
// task #32). Once filled, the values must satisfy the engine validator:
//   - 1 <= skill_id <= 15
//   - skill1 !== skill2 within a single piece
//
// Until the designer fills these in, /setup/ disables the relevant radio
// + the Continue button via `isPreMadeLoadoutReady()`.

export const PRE_MADE_LOADOUTS: Record<PreMadeLoadoutId, SideLoadout> = {
  firstGame: [
    [0, 0], // TODO(OQ-65): King
    [0, 0], // TODO(OQ-65): Champion 1
    [0, 0], // TODO(OQ-65): Champion 2
    [0, 0], // TODO(OQ-65): Champion 3
    [0, 0], // TODO(OQ-65): Champion 4
    [0, 0], // TODO(OQ-65): Champion 5
  ] as const,
  secondGame: [
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
  ] as const,
  thirdGame: [
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
  ] as const,
};

/** Returns true iff the loadout entry has been filled in by the designer —
 *  i.e. every slot is a valid skill id (1..15). UI should disable the
 *  matching radio button when this is false. */
export function isPreMadeLoadoutReady(id: PreMadeLoadoutId): boolean {
  const lo = PRE_MADE_LOADOUTS[id];
  for (const [s1, s2] of lo) {
    if (s1 < 1 || s1 > SKILL_COUNT) return false;
    if (s2 < 1 || s2 > SKILL_COUNT) return false;
    if (s1 === s2) return false;
  }
  return true;
}
