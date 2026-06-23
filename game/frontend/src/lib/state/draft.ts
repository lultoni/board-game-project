// Makeshift draft state — UI-only, snapshot-rewritten before the match starts.
//
// This is NOT the real Stack M draft. The proper draft (alternating picks +
// Phase::Draft on the engine side + simultaneous-reveal flow per OQ-62 +
// pre-made loadouts per OQ-65) lives in `next_steps #10`. The shape here is a
// simultaneous full-loadout picker so combos can be tested in the digital
// build before that work lands.

import { SKILLS, SKILL_COUNT } from "$lib/engine/skills";
import type { Owner } from "$lib/engine/mailbox";

/** Two skill IDs (1..15). 0 = empty slot. */
export type Loadout = [number, number];

/**
 * Square → [skill1, skill2]. Only Kings and Champions appear here; Guards
 * carry no skills (FEN rejects non-zero skill fields on a Guard).
 */
export type LoadoutMap = Map<number, Loadout>;

/** Numeric skill IDs by category, for preset construction. */
const STRIKE = [1, 2, 3, 4, 5];        // Lance, Hook, Break, Steal, Tempest
const SHIELD = [6, 7, 8];              // Shield, Heal, Plate
const MOVE   = [9, 10, 11, 12, 13];    // Dash, Blast, Shove, Swap, Retreat
const MYSTIC = [14, 15];               // Focus, Charge

function pick<T>(xs: T[], rng: () => number): T {
  return xs[Math.floor(rng() * xs.length)];
}

function rngFrom(seed: number): () => number {
  // mulberry32 — deterministic per seed; enough for makeshift randomisation.
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Sanity: skill id in [1, 15]. */
function safeId(id: number): number {
  if (!Number.isInteger(id)) return 0;
  if (id < 1 || id > SKILL_COUNT) return 0;
  return id;
}

export function isStrike(id: number): boolean {
  return SKILLS[id]?.category === "strike";
}

/** Empty loadout for every King/Champion square of an owner. */
export function emptyLoadoutMap(squares: number[]): LoadoutMap {
  const m: LoadoutMap = new Map();
  for (const sq of squares) m.set(sq, [0, 0]);
  return m;
}

export type PresetName = "aggro" | "defense" | "combo" | "random";

function presetSlots(name: PresetName, idx: number, rng: () => number): Loadout {
  switch (name) {
    case "aggro": {
      // Every piece carries a Strike; second slot is Charge for some, Strike
      // for others. Heavy damage, light utility.
      const s1 = STRIKE[idx % STRIKE.length];
      const s2 = idx % 2 === 0 ? 15 /* Charge */ : pick(STRIKE, rng);
      return [s1, s2];
    }
    case "defense": {
      // Shields + Plates + a token Strike for self-defence.
      const s1 = SHIELD[idx % SHIELD.length];
      const s2 = idx % 2 === 0 ? pick(SHIELD, rng) : pick(STRIKE, rng);
      return [s1, s2];
    }
    case "combo": {
      // Movement-causing skill + Strike → ticks the widened combo counter
      // (Stack M rule). Plus Focus on a couple of pieces for extra range.
      const s1 = pick(MOVE, rng);
      const s2 = idx === 0 ? 14 /* Focus */
               : idx === 1 ? 15 /* Charge */
               : pick(STRIKE, rng);
      return [s1, s2];
    }
    case "random": {
      let s1 = 1 + Math.floor(rng() * SKILL_COUNT);
      let s2 = 1 + Math.floor(rng() * SKILL_COUNT);
      // 0 means no skill — keep both filled.
      if (s1 === 0) s1 = 1;
      if (s2 === 0) s2 = 1;
      return [s1, s2];
    }
  }
}

/**
 * Build a loadout map for one side using a named preset. `squares` is the
 * list of King/Champion squares for that owner, in any stable order
 * (deterministic preset output depends on this order).
 */
export function presetLoadout(
  squares: number[],
  preset: PresetName,
  seed = 1,
): LoadoutMap {
  const rng = rngFrom(seed);
  const m: LoadoutMap = new Map();
  squares.forEach((sq, i) => {
    m.set(sq, presetSlots(preset, i, rng));
  });
  return m;
}

/** Combine two single-side LoadoutMaps into a unified map for FEN rewriting. */
export function mergeLoadouts(...maps: LoadoutMap[]): LoadoutMap {
  const out: LoadoutMap = new Map();
  for (const m of maps) {
    for (const [sq, lo] of m) out.set(sq, [safeId(lo[0]), safeId(lo[1])]);
  }
  return out;
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
