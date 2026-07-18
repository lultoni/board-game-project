//! Single-track gauntlet - the selection layer that decides which mutated
//! candidates advance and which get discarded.
//!
//! Implements plan §5 (ns-50 rework: **single 100 ms/ply track**, superseding
//! the retired three-bracket / three-champion-track design):
//!
//! - **Match** (`play_match`): one head-to-head game between two evaluators at
//!   a given time-per-ply budget. Returns the outcome P1-POV.
//! - **Mirrored best-of-three** (`mirrored_bo3`): the candidate plays the
//!   baseline twice on the same loadout (once as P1, once as P2), then a
//!   tiebreaker on a fresh loadout if neither has clinched. Cancels draft luck.
//! - **Acceptance** (`accept_vs`): a candidate is accepted iff it wins the
//!   mirrored BO3 against the current champion. One opponent, one think time -
//!   the mutation loop makes one candidate at a time, so there is no candidate
//!   pool to pre-filter (the retired Tier-1/Tier-2 split is gone).
//! - **Champion tracker** (`ChampionTracker`): a single champion pointer,
//!   updated when a candidate is accepted.
//!
//! ## Think time
//!
//! Plan §5.1 selects at the **100 ms/ply** bracket only. We treat that as the
//! `time_limit_ms` passed to `find_best_with_evaluator`, with a high
//! `max_depth` cap (64) so time dominates the bound.
//!
//! ## Why the gauntlet lives in this crate (not core_engine)
//!
//! The gauntlet only matters during training - a training-time concept, not a
//! runtime one. Putting it next to the orchestrator keeps all training
//! infrastructure in `nn_trainer/`.

use crate::loadout::random_loadout_from_seed;
use core_engine::game_logic::action::Action;
use core_engine::game_logic::make_unmake;
use core_engine::game_logic::skills::SideLoadout;
use core_engine::search::alpha_beta::find_best_with_evaluator;
use core_engine::search::evaluator::{Evaluator, HeuristicEvaluator};
use core_engine::search::transposition::TranspositionTable;
use core_engine::state::position::{GameResult, Player};
use core_engine::state::Position;

/// Outcome of a single head-to-head game from P1's POV. `None` means the game
/// hit the ply cap without terminating - caller decides how to score it (we
/// adjudicate via the heuristic in `play_match_with_callback`).
pub type MatchOutcome = Option<GameResult>;

/// Same ply cap as `selfplay::MAX_PLIES`.
const MAX_PLIES: usize = 250;

/// Generous depth cap for time-bounded search. The deadline does the work;
/// this just prevents runaway depth on simple positions.
const TIME_BOUNDED_MAX_DEPTH: u8 = 64;

/// Play one game with `eval_p1` controlling P1 and `eval_p2` controlling P2,
/// at `time_ms` per ply. Returns the game result, or adjudicates at the ply
/// cap via the heuristic.
pub fn play_match(
    eval_p1: &dyn Evaluator,
    eval_p2: &dyn Evaluator,
    loadout_p1: &SideLoadout,
    loadout_p2: &SideLoadout,
    time_ms: u64,
) -> MatchOutcome {
    play_match_with_callback(eval_p1, eval_p2, loadout_p1, loadout_p2, time_ms, |_, _, _| {})
}

/// Same as `play_match`, but invokes `on_ply(position_after_ply, ply_index,
/// action_played)` after every move. The callback is the hook the orchestrator
/// uses to write `live.json` for the UI's Live Match View. When nobody is
/// subscribed, `on_ply` becomes a cheap noop - the orchestrator gates
/// expensive work inside the closure on `live::is_subscribed`.
pub fn play_match_with_callback<F>(
    eval_p1: &dyn Evaluator,
    eval_p2: &dyn Evaluator,
    loadout_p1: &SideLoadout,
    loadout_p2: &SideLoadout,
    time_ms: u64,
    mut on_ply: F,
) -> MatchOutcome
where
    F: FnMut(&Position, u32, &Action),
{
    let mut pos = Position::setup_stack_m_with_loadouts(loadout_p1, loadout_p2);
    let mut tt_p1 = TranspositionTable::with_capacity_pow2(16);
    let mut tt_p2 = TranspositionTable::with_capacity_pow2(16);

    for ply in 0..MAX_PLIES {
        if pos.game_result.is_some() {
            break;
        }
        let (eval, tt) = match pos.to_move {
            Player::P1 => (eval_p1, &mut tt_p1),
            Player::P2 => (eval_p2, &mut tt_p2),
        };
        let sr = find_best_with_evaluator(&mut pos, tt, time_ms, TIME_BOUNDED_MAX_DEPTH, eval, None);
        let Some(action) = sr.best else { return None; };
        let _undo = make_unmake::make(&mut pos, action);
        on_ply(&pos, ply as u32, &action);
    }

    // Adjudicate at ply cap via heuristic: positive score → P1 leads. On exact
    // zero (symmetric position) award P1 as a tiebreak - this avoids all-draw
    // series when both sides are equally matched at the cap.
    pos.game_result.or_else(|| {
        let score = HeuristicEvaluator.evaluate(&pos);
        if score >= 0 {
            Some(GameResult::P1Wins)
        } else {
            Some(GameResult::P2Wins)
        }
    })
}

/// Tally for a series of games. Wins are recorded for the *candidate* (the
/// rater being graded), not the actual P1/P2 player at the board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeriesTally {
    pub candidate_wins: u32,
    pub baseline_wins: u32,
    pub indecisive: u32,
}

impl SeriesTally {
    pub fn games_played(self) -> u32 {
        self.candidate_wins + self.baseline_wins + self.indecisive
    }

    pub fn win_rate(self) -> f32 {
        let n = self.candidate_wins + self.baseline_wins;
        if n == 0 {
            0.0
        } else {
            self.candidate_wins as f32 / n as f32
        }
    }

    /// True if the candidate has more decisive wins than the baseline. The BO3
    /// acceptance rule: the candidate must out-win the champion.
    pub fn candidate_leads(self) -> bool {
        self.candidate_wins > self.baseline_wins
    }
}

/// Run a single (loadout, both colours) mirror pair. The candidate plays once
/// as P1 and once as P2, both on the same `SideLoadout`. Returns the tally
/// over those two games.
fn mirror_pair(
    candidate: &dyn Evaluator,
    baseline: &dyn Evaluator,
    loadout: &SideLoadout,
    time_ms: u64,
) -> SeriesTally {
    let mut tally = SeriesTally::default();

    // Game 1: candidate as P1, baseline as P2.
    match play_match(candidate, baseline, loadout, loadout, time_ms) {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    // Game 2: candidate as P2, baseline as P1.
    match play_match(baseline, candidate, loadout, loadout, time_ms) {
        Some(GameResult::P2Wins) => tally.candidate_wins += 1,
        Some(GameResult::P1Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    tally
}

/// Mirrored best-of-three at a single think time. Plays the mirror pair on
/// `loadout_seed`; if neither side has clinched after two games, plays a third
/// game on a derived second loadout as the tiebreaker (the {candidate-P1,
/// candidate-P2} pair counts as 2 games of the BO3, so at most one more game
/// decides it; a fresh loadout for the tiebreaker reduces the chance of a
/// deterministic draw locking the series).
pub fn mirrored_bo3(
    candidate: &dyn Evaluator,
    baseline: &dyn Evaluator,
    loadout_seed: u64,
    time_ms: u64,
) -> SeriesTally {
    let loadout_a = random_loadout_from_seed(loadout_seed);
    let mut tally = mirror_pair(candidate, baseline, &loadout_a, time_ms);

    if tally.candidate_wins >= 2 || tally.baseline_wins >= 2 {
        return tally;
    }

    let loadout_b = random_loadout_from_seed(loadout_seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5));
    match play_match(candidate, baseline, &loadout_b, &loadout_b, time_ms) {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    tally
}

/// Outcome of an acceptance run: the series tally plus whether the candidate
/// cleared the bar (won the mirrored BO3 against the champion).
#[derive(Clone, Copy, Debug, Default)]
pub struct Acceptance {
    pub tally: SeriesTally,
    pub pass: bool,
}

/// Single-track acceptance: the candidate plays one mirrored BO3 against the
/// current champion at `time_ms`. Accepted iff the candidate out-wins the
/// champion. This is the whole gate - the mutation loop makes one candidate at
/// a time, so there is no pool to rank and no non-regression sweep to run.
/// (A hall-of-fame non-regression check can be added later if mutation causes
/// cycling - plan §5.2.)
pub fn accept_vs(
    candidate: &dyn Evaluator,
    champion: &dyn Evaluator,
    loadout_seed: u64,
    time_ms: u64,
) -> Acceptance {
    let tally = mirrored_bo3(candidate, champion, loadout_seed, time_ms);
    Acceptance {
        tally,
        pass: tally.candidate_leads(),
    }
}

/// Stable, opaque identifier for a rater in the champion tracker. The tracker
/// just stores it and hands it back (a generation counter, a version index,
/// anything).
pub type RaterId = u64;

/// Single champion tracker (ns-50: one track, replacing the retired
/// best-fast / best-slow / best-overall triple). Records which rater currently
/// holds the title and its win-rate floor, so a fresh process can resume the
/// floor from the on-disk index.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChampionTracker {
    pub best: Option<RaterId>,
    /// Win-rate of the current champion at the point it was crowned. `None`
    /// until the first champion is recorded.
    pub best_score: Option<f32>,
}

impl ChampionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the tracker with a known champion + score floor (e.g. recovered
    /// from the on-disk index on process restart).
    pub fn seed(&mut self, id: RaterId, score: f32) {
        self.best = Some(id);
        self.best_score = Some(score);
    }

    /// Consider `candidate` (which already passed `accept_vs`) for the title.
    /// It takes the title (returns `true`) iff it beats the current champion's
    /// win-rate floor - the first accepted candidate always wins. Callers only
    /// call this when acceptance passed, so this is the tie-break between
    /// multiple passers over a run.
    pub fn consider(&mut self, candidate: RaterId, win_rate: f32) -> bool {
        if self.best_score.map_or(true, |s| win_rate > s) {
            self.best = Some(candidate);
            self.best_score = Some(win_rate);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::search::evaluator::HeuristicEvaluator;

    /// A degenerate evaluator that returns a fixed score regardless of
    /// position - forces obviously bad decisions so we can verify plumbing
    /// without relying on real rater strength differences.
    #[derive(Clone, Copy, Debug)]
    struct ConstEval(i32);
    impl Evaluator for ConstEval {
        fn evaluate(&self, _pos: &Position) -> i32 {
            self.0
        }
        fn evaluate_breakdown(&self, _pos: &Position) -> core_engine::search::evaluator::EvalBreakdown {
            core_engine::search::evaluator::EvalBreakdown::default()
        }
    }

    #[test]
    fn match_terminates_with_outcome() {
        let l = random_loadout_from_seed(1);
        let result = play_match(&HeuristicEvaluator, &HeuristicEvaluator, &l, &l, 100);
        if let Some(r) = result {
            assert!(matches!(r, GameResult::P1Wins | GameResult::P2Wins));
        }
    }

    #[test]
    fn callback_fires_once_per_ply_with_advancing_index() {
        let l = random_loadout_from_seed(2);
        let mut seen_plies: Vec<u32> = Vec::new();
        let mut final_pos_was_terminal = false;
        let outcome = play_match_with_callback(
            &HeuristicEvaluator, &HeuristicEvaluator, &l, &l, 100,
            |pos, ply, _action| {
                seen_plies.push(ply);
                final_pos_was_terminal = pos.game_result.is_some();
            },
        );
        assert!(!seen_plies.is_empty(), "callback never fired");
        for (i, p) in seen_plies.iter().enumerate() {
            assert_eq!(*p as usize, i, "ply index must advance 0,1,2,…; got {:?}", seen_plies);
        }
        // A natural terminal → outcome is Some AND the final on_ply saw it.
        // But the ply cap also yields Some (heuristic-adjudicated) with a
        // NON-terminal final position, so we can't assert the converse.
        if final_pos_was_terminal {
            assert!(outcome.is_some(), "terminal final position must yield an outcome");
        }
    }

    #[test]
    fn mirrored_bo3_plays_at_least_two_games() {
        // Two identical evaluators on the same loadout - outcome depends on
        // colour-symmetry; we don't predict the winner, only that the tally
        // totals at least the mirror pair (2 games).
        let tally = mirrored_bo3(&HeuristicEvaluator, &HeuristicEvaluator, 7, 10);
        assert!(
            tally.games_played() >= 2,
            "mirrored_bo3 must play at least the mirror pair; tally = {:?}",
            tally
        );
    }

    #[test]
    fn real_evaluator_beats_constant() {
        // Plumbing-correctness: a real evaluator should out-perform one that
        // always returns 0, and `accept_vs` should report `pass`.
        let const_eval = ConstEval(0);
        let acc = accept_vs(&HeuristicEvaluator, &const_eval, 13, 10);
        assert!(
            acc.tally.candidate_wins > acc.tally.baseline_wins,
            "heuristic must out-score const-0; tally = {:?}",
            acc.tally
        );
        assert!(acc.pass, "candidate that out-wins the champion must pass");
    }

    #[test]
    fn champion_tracker_crowns_first_then_stronger() {
        let mut tracker = ChampionTracker::new();
        // First passer always takes the title.
        assert!(tracker.consider(1, 0.6));
        assert_eq!(tracker.best, Some(1));
        assert_eq!(tracker.best_score, Some(0.6));
        // A weaker passer does not displace it.
        assert!(!tracker.consider(2, 0.55));
        assert_eq!(tracker.best, Some(1));
        // A stronger passer does.
        assert!(tracker.consider(3, 0.7));
        assert_eq!(tracker.best, Some(3));
        assert_eq!(tracker.best_score, Some(0.7));
    }

    #[test]
    fn champion_tracker_seed_sets_floor() {
        let mut tracker = ChampionTracker::new();
        tracker.seed(42, 0.65);
        assert_eq!(tracker.best, Some(42));
        // A candidate below the seeded floor is rejected.
        assert!(!tracker.consider(43, 0.6));
        assert_eq!(tracker.best, Some(42));
    }
}
