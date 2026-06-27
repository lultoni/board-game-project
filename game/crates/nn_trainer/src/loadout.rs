//! Random-but-legal Stack M loadout generation.
//!
//! Stack M draft constraints (`game_logic::skills::validate_loadout`):
//!   - 15 skill ids (1..=15). 0 is the "unequipped" sentinel — never emitted
//!     here; every slot gets a real skill at game start.
//!   - No same-skill-on-same-piece duplicates (e.g. one Champion with Lance
//!     in both slots).
//!
//! Plan §5 also calls for *mirrored* loadouts (same loadout, both sides
//! play both colours, draft luck cancels). v1 just generates per-side random
//! loadouts; the mirroring wraps this generator at the gauntlet layer.
//!
//! All RNG flows from a `ChaCha8Rng` seed for reproducibility — a corpus
//! built from seed N can always be regenerated bit-exact.

use core_engine::game_logic::skills::{validate_loadout, SideLoadout};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Produce a `validate_loadout`-clean random `SideLoadout` from `rng`. Each
/// piece's two slots are drawn from the 15 available skill ids without
/// replacement *on that piece* (so the no-duplicate-per-piece rule holds).
/// Different pieces can carry the same skill — Stack M permits repetition
/// across pieces.
pub fn random_loadout(rng: &mut impl Rng) -> SideLoadout {
    let mut out = [(0u8, 0u8); 6];
    let all_ids: [u8; 15] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    for slot in out.iter_mut() {
        let mut pool = all_ids;
        pool.shuffle(rng);
        *slot = (pool[0], pool[1]);
    }
    debug_assert!(validate_loadout(&out).is_ok(),
        "random_loadout produced an invalid SideLoadout: {:?}", out);
    out
}

/// Convenience wrapper: seed a `ChaCha8Rng` from `seed` and produce one
/// loadout. Deterministic — same seed always yields the same loadout.
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
    fn random_loadout_uses_all_skill_ids_across_a_corpus() {
        // Sanity check: across many samples, every skill id 1..=15 appears
        // at least once. Catches "we forgot id 15" regressions.
        let mut seen = [false; 16];
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        for _ in 0..500 {
            let l = random_loadout(&mut rng);
            for &(s1, s2) in &l {
                seen[s1 as usize] = true;
                seen[s2 as usize] = true;
            }
        }
        for id in 1..=15 {
            assert!(seen[id], "skill id {id} never appeared in 500-sample corpus");
        }
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
}
