//! Self-play game driver.
//!
//! One game = one call to `play_game`. Two raters play to completion from a
//! fully-equipped Stack M starting position (no draft phase - loadouts are
//! supplied directly). Each visited `Position` is logged. The terminal
//! outcome labels every logged position.
//!
//! ## Why we drive directly instead of via `core_engine::session::Match`
//!
//! `Match` carries telemetry, snapshot serialisation, draft preset
//! short-circuits, and a SeatKind-based turn dispatcher - all useful for the
//! frontend, all overhead in a self-play corpus generator that wants to run
//! tens of thousands of games. The direct driver here uses
//! `find_best_with_evaluator` + `make_unmake::make` and nothing else.
//!
//! ## Labelling (plan §4)
//!
//! v1 uses **game-outcome labels** (AlphaZero-style): every non-terminal
//! position visited during a game is labelled with the eventual game
//! result, in the same P1-POV sign convention as `evaluator::evaluate`
//! (+1 → P1 wins, −1 → P2 wins). The trainer scales these to match the
//! eval's i32 range later; here we keep the raw {−1, +1}.
//!
//! Deep-search-score labels (the alternative from §4) can be added as a
//! second pass once we have a baseline.

use core_engine::game_logic::make_unmake;
use core_engine::game_logic::skills::SideLoadout;
use core_engine::search::alpha_beta::find_best_with_evaluator;
use core_engine::search::evaluator::Evaluator;
use core_engine::search::transposition::TranspositionTable;
use core_engine::state::Position;
use core_engine::state::position::GameResult;

/// A single (position, label) training example.
///
/// `label` is in {−1.0, +1.0} for game-outcome labels. Position is captured
/// *before* the move that was played from it, so the rater learns to score
/// positions the search will actually see.
#[derive(Clone, Debug)]
pub struct LabelledPosition {
    pub position: Position,
    pub label: f32,
}

/// Outcome bundle for one self-play game.
#[derive(Debug)]
pub struct GameRecord {
    pub outcome: GameResult,
    /// Non-terminal positions visited, in ply order. Length = ply count.
    /// Terminal positions are NOT included - they bypass the NN by design
    /// (per `nn-rater-plan.md` §1).
    pub positions: Vec<Position>,
    /// Number of plies played (== positions.len()).
    pub plies: usize,
}

impl GameRecord {
    /// Expand to a flat list of (position, label) examples for the trainer.
    /// Every logged position receives the same game-outcome label.
    pub fn into_labelled(self) -> Vec<LabelledPosition> {
        let label: f32 = match self.outcome {
            GameResult::P1Wins =>  1.0,
            GameResult::P2Wins => -1.0,
        };
        self.positions.into_iter()
            .map(|p| LabelledPosition { position: p, label })
            .collect()
    }
}

/// Hard cap on plies per game. Stack M targets short games (game length cut
/// is its whole purpose); a game running past this many plies almost
/// certainly means the search has degenerated into an EndPhase-shuffle.
/// We abort and assign a default outcome based on material at the cap; in
/// practice this cap is only hit when the engine is broken (eval bug,
/// generator regression).
const MAX_PLIES: usize = 250;

/// Play one game from `setup_stack_m_with_loadouts(loadout_p1, loadout_p2)`.
/// `rater_p1` evaluates positions while P1 is to move; `rater_p2` does so
/// while P2 is to move.
///
/// `max_depth` is the per-move search budget. Time-limited search is
/// available via `find_best_with_evaluator` but reproducibility wins for
/// data generation - fixed depth means the same seed always produces the
/// same game.
///
/// Returns `None` if the game hit `MAX_PLIES` without terminating; the
/// caller drops the data and (probably) reports a bug.
pub fn play_game(
    rater_p1: &dyn Evaluator,
    rater_p2: &dyn Evaluator,
    loadout_p1: &SideLoadout,
    loadout_p2: &SideLoadout,
    max_depth: u8,
) -> Option<GameRecord> {
    let mut pos = Position::setup_stack_m_with_loadouts(loadout_p1, loadout_p2);
    let mut positions: Vec<Position> = Vec::new();

    // Each side keeps its own TT - they're not shared because the raters
    // may have wildly different scoring conventions, and a poisoned entry
    // from one side's search would mislead the other's.
    let mut tt_p1 = TranspositionTable::with_capacity_pow2(16);
    let mut tt_p2 = TranspositionTable::with_capacity_pow2(16);

    for _ply in 0..MAX_PLIES {
        if pos.game_result.is_some() { break; }

        positions.push(pos.clone());

        let (evaluator, tt) = match pos.to_move {
            core_engine::state::position::Player::P1 => (rater_p1, &mut tt_p1),
            core_engine::state::position::Player::P2 => (rater_p2, &mut tt_p2),
        };
        let sr = find_best_with_evaluator(&mut pos, tt, /*time_limit_ms=*/0, max_depth, evaluator, None);
        let Some(action) = sr.best else {
            // Search returned no move on a non-terminal position. Generator
            // always emits at least EndPhase; this means an internal bug.
            // Drop the game.
            return None;
        };
        let _undo = make_unmake::make(&mut pos, action);
    }

    let outcome = pos.game_result?;
    let plies = positions.len();
    Some(GameRecord { outcome, positions, plies })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loadout::random_loadout_from_seed;
    use core_engine::search::evaluator::HeuristicEvaluator;

    #[test]
    fn heuristic_self_play_terminates() {
        // Smoke: a heuristic-vs-heuristic game at depth 2 must terminate
        // with a valid outcome and a non-empty position log.
        let l1 = random_loadout_from_seed(1);
        let l2 = random_loadout_from_seed(2);
        let rec = play_game(&HeuristicEvaluator, &HeuristicEvaluator, &l1, &l2, 2)
            .expect("game must terminate");
        assert!(rec.plies > 0);
        assert!(matches!(rec.outcome, GameResult::P1Wins | GameResult::P2Wins));
        assert_eq!(rec.plies, rec.positions.len());
        // No logged position is terminal (we skip them by construction).
        for p in &rec.positions {
            assert!(p.game_result.is_none(),
                "terminal position should not appear in positions log");
        }
    }

    #[test]
    fn labelled_examples_have_unit_labels() {
        let l1 = random_loadout_from_seed(3);
        let l2 = random_loadout_from_seed(4);
        let rec = play_game(&HeuristicEvaluator, &HeuristicEvaluator, &l1, &l2, 2)
            .expect("game must terminate");
        let outcome = rec.outcome;
        let examples = rec.into_labelled();
        assert!(!examples.is_empty());
        let expected: f32 = match outcome {
            GameResult::P1Wins =>  1.0,
            GameResult::P2Wins => -1.0,
        };
        for ex in &examples {
            assert_eq!(ex.label, expected);
        }
    }
}
