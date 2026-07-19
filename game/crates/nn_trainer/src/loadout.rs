//! Realistic Stack M loadout generation (shared shape with the corpus builder).
//!
//! Stack M draft constraints (`game_logic::skills::validate_loadout`):
//!   - 15 skill ids (1..=15). 0 is the "unequipped" sentinel - never emitted
//!     here; every slot gets a real skill at game start.
//!   - No same-skill-on-same-piece duplicates.
//!
//! ns-50 Phase 1 (§5.3): this generator was previously *uniform* (each slot an
//! independent shuffle of the 15 ids), which produces unrealistic games - a net
//! tuned on them mis-learns. It now uses the **weighted-incremental** algorithm
//! ported from `core_engine/examples/build_corpus.rs::random_loadout`: fill the
//! 12 slots (6 pieces x 2) in shuffled order under per-side skill caps, then
//! require ≥3 of the 4 categories present AND at least one Strike (retry from
//! scratch otherwise - retries are cheap). This is the corpus builder's
//! realistic distribution, so self-play loadouts match the positions the net
//! is graded on.
//!
//! The corpus builder keeps its own private copy for now; unifying the two on a
//! single shared home is a deferred follow-up (would require promoting `rand`
//! to a core_engine runtime dep). The *algorithm* is the same here.
//!
//! Plan §5 also calls for *mirrored* loadouts (same loadout, both sides play
//! both colours, draft luck cancels). This generates per-side loadouts; the
//! mirroring wraps this generator at the gauntlet layer.
//!
//! All RNG flows from a `ChaCha8Rng` seed for reproducibility - a corpus built
//! from seed N can always be regenerated bit-exact.

use core_engine::game_logic::skills::{
    skill_category, skill_from_id, validate_loadout, Skill, SkillCategory, SideLoadout,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

/// Per-side caps: max occurrences of any single skill across all 12 slots.
/// Mystics (Focus/Charge) are global so extras are wasted; a few skills are
/// capped lower to keep loadouts plausible. Everything else ≤ 4.
fn skill_cap(s: Skill) -> u8 {
    match s {
        Skill::Focus => 2,
        Skill::Charge => 2,
        Skill::Heal => 3,
        Skill::Steal => 3,
        Skill::Swap => 2,
        Skill::Retreat => 2,
        _ => 4,
    }
}

/// All 15 Stack M skills, in id order (id = `skill as u8`).
const ALL_SKILLS: [Skill; 15] = [
    Skill::Lance, Skill::Hook, Skill::Break, Skill::Steal, Skill::Tempest,
    Skill::Shield, Skill::Heal, Skill::Plate,
    Skill::Dash, Skill::Blast, Skill::Shove, Skill::Swap, Skill::Retreat,
    Skill::Focus, Skill::Charge,
];

fn cat_idx(c: SkillCategory) -> usize {
    match c {
        SkillCategory::Strike => 0,
        SkillCategory::Shield => 1,
        SkillCategory::Move => 2,
        SkillCategory::Mystic => 3,
    }
}

/// Produce a `validate_loadout`-clean, *realistic* `SideLoadout` from `rng`.
///
/// The 12 slots (6 pieces x 2 slots) are filled in a shuffled order. For each
/// slot we pick uniformly from the skills that (a) haven't hit their per-side
/// cap yet, and (b) don't already occupy the OTHER slot on the same piece
/// (the per-piece no-duplicate rule). After all slots are filled we require
/// ≥3 of the 4 categories present AND at least one Strike; if either check
/// fails we retry from scratch (cheaper and simpler than force-swapping). A
/// final `validate_loadout` guards against a bug in this function.
pub fn random_loadout(rng: &mut impl Rng) -> SideLoadout {
    loop {
        let mut lo: SideLoadout = [(0u8, 0u8); 6];
        let mut counts: HashMap<u8, u8> = HashMap::new();

        // Slot fill order: (piece_idx, slot_idx), shuffled, to avoid biasing
        // slot-1 vs slot-2 fills.
        let mut order: Vec<(usize, usize)> =
            (0..6).flat_map(|p| [(p, 0), (p, 1)]).collect();
        order.shuffle(rng);

        let mut broke = false;
        for (piece, slot) in order {
            // Candidates: below cap AND not equal to the other slot on this piece.
            let other = if slot == 0 { lo[piece].1 } else { lo[piece].0 };
            let candidates: Vec<Skill> = ALL_SKILLS
                .iter()
                .copied()
                .filter(|s| {
                    let id = *s as u8;
                    counts.get(&id).copied().unwrap_or(0) < skill_cap(*s) && id != other
                })
                .collect();
            if candidates.is_empty() {
                // Extremely unlikely given the caps - restart from scratch.
                broke = true;
                break;
            }
            let pick = *candidates.choose(rng).unwrap();
            let id = pick as u8;
            *counts.entry(id).or_insert(0) += 1;
            if slot == 0 {
                lo[piece].0 = id;
            } else {
                lo[piece].1 = id;
            }
        }
        if broke {
            continue;
        }

        // Any zero slot means the inner loop broke early - retry.
        if lo.iter().any(|(a, b)| *a == 0 || *b == 0) {
            continue;
        }

        // Category diversity (≥3 of 4) + Strike presence.
        let mut cat_present = [false; 4];
        let mut strike_present = false;
        for (a, b) in lo.iter() {
            for &id in &[*a, *b] {
                if let Some(sk) = skill_from_id(id) {
                    let c = skill_category(sk);
                    cat_present[cat_idx(c)] = true;
                    if c == SkillCategory::Strike {
                        strike_present = true;
                    }
                }
            }
        }
        let n_cats = cat_present.iter().filter(|&&x| x).count();
        if n_cats < 3 || !strike_present {
            // Retries are cheap - simpler than force-swapping a slot.
            continue;
        }

        // Final validity check (per-piece dup, id range).
        if validate_loadout(&lo).is_ok() {
            return lo;
        }
    }
}

/// Convenience wrapper: seed a `ChaCha8Rng` from `seed` and produce one
/// loadout. Deterministic - same seed always yields the same loadout.
pub fn random_loadout_from_seed(seed: u64) -> SideLoadout {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    random_loadout(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_loadout_passes_validation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..1000 {
            let l = random_loadout(&mut rng);
            validate_loadout(&l).expect("random loadout must validate");
        }
    }

    #[test]
    fn random_loadout_is_deterministic_from_seed() {
        let a = random_loadout_from_seed(123);
        let b = random_loadout_from_seed(123);
        assert_eq!(a, b);
    }

    #[test]
    fn random_loadout_emits_no_zeros() {
        let mut rng = ChaCha8Rng::seed_from_u64(99);
        for _ in 0..100 {
            let l = random_loadout(&mut rng);
            for &(s1, s2) in &l {
                assert_ne!(s1, 0, "slot 1 must be a real skill");
                assert_ne!(s2, 0, "slot 2 must be a real skill");
            }
        }
    }

    #[test]
    fn random_loadout_guarantees_categories_and_strike() {
        // The realistic generator's contract (mirrors the corpus builder):
        // every emitted loadout has ≥3 of the 4 skill categories AND at least
        // one Strike skill. Catches a regression that drops the retry guard.
        let mut rng = ChaCha8Rng::seed_from_u64(2024);
        for _ in 0..500 {
            let l = random_loadout(&mut rng);
            let mut cats = [false; 4];
            let mut strike = false;
            for &(a, b) in &l {
                for &id in &[a, b] {
                    if let Some(sk) = skill_from_id(id) {
                        let c = skill_category(sk);
                        cats[cat_idx(c)] = true;
                        if c == SkillCategory::Strike {
                            strike = true;
                        }
                    }
                }
            }
            let n = cats.iter().filter(|&&x| x).count();
            assert!(n >= 3, "loadout must have >=3 categories, got {n}: {l:?}");
            assert!(strike, "loadout must contain a Strike skill: {l:?}");
        }
    }

    #[test]
    fn random_loadout_respects_per_side_caps() {
        // Capped skills (Focus/Charge/Swap/Retreat ≤2, Heal/Steal ≤3) must
        // never exceed their cap across the 12 slots.
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        for _ in 0..500 {
            let l = random_loadout(&mut rng);
            let mut counts: HashMap<u8, u8> = HashMap::new();
            for &(a, b) in &l {
                *counts.entry(a).or_insert(0) += 1;
                *counts.entry(b).or_insert(0) += 1;
            }
            for (&id, &n) in &counts {
                if let Some(sk) = skill_from_id(id) {
                    assert!(
                        n <= skill_cap(sk),
                        "skill id {id} appears {n}x > cap {}: {l:?}",
                        skill_cap(sk)
                    );
                }
            }
        }
    }
}
