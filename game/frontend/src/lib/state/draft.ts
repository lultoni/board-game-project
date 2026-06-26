// L8 — draft / loadout helpers used by /setup/, /draft/, and /match/.
//
// The big makeshift-draft helpers that lived here (presets, merge,
// FEN-rewrite glue) are gone — /draft/ now drives the real Phase::Draft
// flow on the engine via DraftTurn actions, and pre-made mode goes through
// `createEngineWithLoadouts`. What remains: the canonical Stack M piece-
// squares, a `squareName` helper, the pre-made loadout catalogue, and the
// designer-readiness check for the /setup/ picker.

import {
  SKILLS,
  SKILL_COUNT,
  type Owner,
  type SideLoadout,
} from "$lib/engine";
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
// Designed Session 33+ as **escalating teaching decks** (OQ-65). Framed as
// "First / Second / Third game" rather than playstyle archetypes, so a new
// player builds a mental model over three games:
//
//  - Game 1 introduces Strike (Lance), self-defense (Shield), and basic
//    movement (Dash + Blast). 4 distinct skills. No Heal precondition, no
//    mystic timing, no combos.
//  - Game 2 adds range-2 strikes (Hook), ally support (Plate), and the
//    Mystic concept (Focus = +1 Range buff). King gains a Strike (Hook)
//    so the player learns the King can defend itself. 6 distinct skills.
//  - Game 3 explicitly teaches the Multi-Champion Combo Bonus via Tempest
//    (Strike + AOE push — ticks the counter on the target and the whole
//    surrounding ring). Includes Heal (medic loop) and one each of Focus
//    and Charge so the player sees both mystics. 9 distinct skills.
//
// Steal is intentionally excluded from all three (money-warfare adds a
// resource axis on top of the mechanical lessons — saved for custom draft).
//
// Skill IDs (per core_engine/src/game_logic/skills.rs):
//   1 Lance · 2 Hook · 3 Break · 4 Steal · 5 Tempest · 6 Shield · 7 Heal
//   8 Plate · 9 Dash · 10 Blast · 11 Shove · 12 Swap · 13 Retreat
//   14 Focus · 15 Charge

export const PRE_MADE_LOADOUTS: Record<PreMadeLoadoutId, SideLoadout> = {
  // Game 1 — "Pieces with personalities". Lance / Shield / Dash / Blast.
  firstGame: [
    [6, 9],   // King:        Shield + Dash    (mobile defender)
    [1, 6],   // Champion 1:  Lance + Shield   (tank-striker)
    [1, 10],  // Champion 2:  Lance + Blast    (skirmisher)
    [1, 9],   // Champion 3:  Lance + Dash     (mobile striker)
    [6, 10],  // Champion 4:  Shield + Blast   (pure disruptor)
    [1, 9],   // Champion 5:  Lance + Dash     (mobile striker)
  ] as const,

  // Game 2 — "Reach, support, the King fights back". Adds Hook, Plate, Focus.
  secondGame: [
    [2, 6],   // King:        Hook + Shield    (range-2 self-defender)
    [1, 8],   // Champion 1:  Lance + Plate    (frontline support)
    [2, 9],   // Champion 2:  Hook + Dash      (reach-fighter)
    [1, 14],  // Champion 3:  Lance + Focus    (buff caster)
    [2, 6],   // Champion 4:  Hook + Shield    (reach-tank)
    [1, 8],   // Champion 5:  Lance + Plate    (frontline support)
  ] as const,

  // Game 3 — "Combos via Tempest". Adds Tempest, Blast, Heal, Charge.
  thirdGame: [
    [5, 6],   // King:        Tempest + Shield (combo-opener / self-defender)
    [2, 15],  // Champion 1:  Hook + Charge    (setup → big Strike finisher)
    [1, 10],  // Champion 2:  Lance + Blast    (cheap ticker + finisher)
    [5, 9],   // Champion 3:  Tempest + Dash   (mobile AOE)
    [2, 7],   // Champion 4:  Hook + Heal      (reach + medic)
    [1, 14],  // Champion 5:  Lance + Focus    (backline buff caster)
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
