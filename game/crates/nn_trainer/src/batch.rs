//! Rayon-parallel corpus generation.
//!
//! Self-play games are embarrassingly parallel: each is a pure function of
//! `(rater_a, rater_b, loadout_seed)`. A 16-core machine runs 16 games at
//! once; corpus generation scales close to linearly with core count, which
//! is precisely the property that makes training-data production feasible.
//!
//! ## Determinism
//!
//! The (positions, labels) pairs returned from a `generate_corpus` call are
//! a deterministic function of the input seed when:
//!   - both raters are deterministic given the same input (the heuristic
//!     evaluator is; the NN evaluator is once its weights are fixed);
//!   - `max_depth` is fixed (no time limit — time-limited search introduces
//!     wall-clock nondeterminism);
//!   - the rayon thread pool is otherwise unconstrained (rayon's work
//!     stealing doesn't affect which inputs map to which outputs — each
//!     game's loadouts and seed are derived deterministically from `seed_base`
//!     before any parallel work begins).
//!
//! Game *order* in the output is preserved (rayon's `into_par_iter` over a
//! Vec maintains index order on collection).

use crate::loadout::random_loadout_from_seed;
use crate::selfplay::{play_game, LabelledPosition};
use core_engine::search::evaluator::Evaluator;
use rayon::prelude::*;

/// Generate a labelled corpus by running `n_games` parallel self-play games.
///
/// Game `i` uses loadouts derived from `(seed_base, i)` so each call with
/// the same `seed_base` produces the same corpus. Games where the engine
/// fails to terminate (returns `None` from `play_game`) are silently
/// dropped — the corpus is best-effort.
pub fn generate_corpus(
    n_games: usize,
    seed_base: u64,
    rater_p1: &(dyn Evaluator + Sync),
    rater_p2: &(dyn Evaluator + Sync),
    max_depth: u8,
) -> Vec<LabelledPosition> {
    (0..n_games)
        .into_par_iter()
        .filter_map(|i| {
            let seed_p1 = seed_base.wrapping_add((i as u64).wrapping_mul(2));
            let seed_p2 = seed_base.wrapping_add((i as u64).wrapping_mul(2).wrapping_add(1));
            let l1 = random_loadout_from_seed(seed_p1);
            let l2 = random_loadout_from_seed(seed_p2);
            play_game(rater_p1, rater_p2, &l1, &l2, max_depth)
        })
        .flat_map(|rec| rec.into_labelled())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::search::evaluator::HeuristicEvaluator;

    #[test]
    fn parallel_corpus_smoke() {
        // 4 games at depth 2 — small enough to run in CI in a few seconds.
        let corpus = generate_corpus(4, /*seed_base=*/100,
            &HeuristicEvaluator, &HeuristicEvaluator, 2);
        assert!(!corpus.is_empty(),
            "expected at least one labelled example from 4 small games");
        for ex in &corpus {
            assert!(ex.label == 1.0 || ex.label == -1.0,
                "outcome label must be in {{-1, +1}}, got {}", ex.label);
            assert!(ex.position.game_result.is_none(),
                "terminal positions must not appear in the corpus");
        }
    }

    #[test]
    fn determinism_same_seed_same_corpus() {
        let a = generate_corpus(3, 42, &HeuristicEvaluator, &HeuristicEvaluator, 2);
        let b = generate_corpus(3, 42, &HeuristicEvaluator, &HeuristicEvaluator, 2);
        assert_eq!(a.len(), b.len(), "corpus length must be deterministic from seed");
        // Spot-check: first and last labels match.
        if !a.is_empty() {
            assert_eq!(a[0].label, b[0].label);
            assert_eq!(a.last().unwrap().label, b.last().unwrap().label);
        }
    }
}
