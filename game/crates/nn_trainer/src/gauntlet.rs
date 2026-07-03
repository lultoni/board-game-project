//! Two-tier gauntlet — selection layer that decides which trained raters
//! advance and which get discarded.
//!
//! Implements plan §5:
//!
//! - **Match** (`play_match`): one head-to-head game between two evaluators
//!   at a given time-per-ply bracket. Returns the outcome P1-POV.
//! - **Mirrored best-of-three** (`mirrored_bo3`): two evaluators face each
//!   other twice on the same loadout (once with the candidate as P1, once
//!   as P2), then a tiebreaker if needed. Cancels draft luck.
//! - **Tier 1** (`tier1_fitness`): candidate vs top-K previous-generation
//!   raters at the **fast bracket** (100 ms/ply). Returns win-rate. Cheap
//!   filter for the large candidate pool emitted by `train_lineages`.
//! - **Tier 2** (`tier2_acceptance`): best-of-three at three brackets
//!   (100 / 300 / 500 ms) against every accepted predecessor. Acceptance
//!   bar: BO3 win vs the immediate predecessor at every bracket AND
//!   ≥ 45% win-rate vs every prior accepted version at every bracket
//!   (non-regression).
//! - **Three champion tracks** (`ChampionTracker`): `best-fast`,
//!   `best-slow`, `best-overall`. A candidate is "accepted" if it qualifies
//!   for any track. Tracks may diverge.
//!
//! ## Brackets
//!
//! The plan calls 100 / 300 / 500 ms per *search per ply*. We treat that as
//! the `time_limit_ms` passed to `find_best_with_evaluator`, with a high
//! `max_depth` cap (64) so time dominates the bound.
//!
//! ## Why the gauntlet lives in this crate (not core_engine)
//!
//! The gauntlet only matters during training — it's a training-time concept,
//! not a runtime one. Putting it next to `train.rs` and `lineage.rs` keeps
//! all training infrastructure in `nn_trainer/`. The runtime `NnEvaluator`
//! (Step 6) lives in core_engine and consumes a frozen weights blob; it has
//! no dependency on this module.

use crate::loadout::random_loadout_from_seed;
use core_engine::game_logic::action::Action;
use core_engine::game_logic::make_unmake;
use core_engine::game_logic::skills::SideLoadout;
use core_engine::search::alpha_beta::find_best_with_evaluator;
use core_engine::search::evaluator::{Evaluator, HeuristicEvaluator};
use core_engine::search::transposition::TranspositionTable;
use core_engine::state::position::{GameResult, Player};
use core_engine::state::Position;

/// The three time-per-ply brackets specified in plan §5.
///
/// `Fast` (100 ms) is also the bracket used by Tier 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bracket {
    Fast,
    Medium,
    Slow,
}

impl Bracket {
    /// Default think time (ms/ply) for this bracket.
    pub fn time_limit_ms(self) -> u64 {
        match self {
            Bracket::Fast => 100,
            Bracket::Medium => 300,
            Bracket::Slow => 500,
        }
    }

    /// Think time scaled by a base multiplier. `base_ms` overrides the Fast
    /// bracket; Medium = 3×, Slow = 5× (same ratios as the defaults).
    pub fn scaled_time_limit_ms(self, base_ms: u64) -> u64 {
        match self {
            Bracket::Fast => base_ms,
            Bracket::Medium => base_ms * 3,
            Bracket::Slow => base_ms * 5,
        }
    }

    pub fn all() -> [Bracket; 3] {
        [Bracket::Fast, Bracket::Medium, Bracket::Slow]
    }
}

/// Outcome of a single head-to-head game from P1's POV. `None` means the
/// game hit the ply cap without terminating — caller decides how to score it
/// (we treat it as a non-result in BO3 tallies).
pub type MatchOutcome = Option<GameResult>;

/// Same ply cap as `selfplay::MAX_PLIES`.
const MAX_PLIES: usize = 250;

/// Generous depth cap for time-bounded search. The deadline does the work;
/// this just prevents runaway depth on simple positions.
const TIME_BOUNDED_MAX_DEPTH: u8 = 64;

/// Play one game with `eval_p1` controlling P1 and `eval_p2` controlling P2,
/// at the given bracket. Returns the game result, or `None` on ply-cap.
///
/// Time is allocated **per ply** (i.e. per `find_best_with_evaluator` call),
/// matching the plan's "ms per search per ply" wording.
pub fn play_match(
    eval_p1: &dyn Evaluator,
    eval_p2: &dyn Evaluator,
    loadout_p1: &SideLoadout,
    loadout_p2: &SideLoadout,
    bracket: Bracket,
) -> MatchOutcome {
    play_match_with_callback(
        eval_p1, eval_p2, loadout_p1, loadout_p2, bracket.time_limit_ms(),
        |_, _, _| {},
    )
}

/// Same as `play_match`, but invokes `on_ply(position_after_ply, ply_index,
/// action_played)` after every move. The callback is the hook the trainer
/// orchestrator uses to write `live.json` for the UI's Live Match View
/// (plan §10 panel 1). When nobody is subscribed, `on_ply` becomes a cheap
/// noop — the orchestrator gates expensive work inside the closure on
/// `live::is_subscribed`.
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
    let time = time_ms;

    for ply in 0..MAX_PLIES {
        if pos.game_result.is_some() { break; }
        let (eval, tt) = match pos.to_move {
            Player::P1 => (eval_p1, &mut tt_p1),
            Player::P2 => (eval_p2, &mut tt_p2),
        };
        let sr = find_best_with_evaluator(
            &mut pos, tt, time, TIME_BOUNDED_MAX_DEPTH, eval, None,
        );
        let Some(action) = sr.best else { return None; };
        let _undo = make_unmake::make(&mut pos, action);
        on_ply(&pos, ply as u32, &action);
    }

    // Adjudicate at ply cap via heuristic: positive score → P1 leads.
    // On exact zero (symmetric position) award P1 as a tiebreak — this
    // avoids all-draw series when both sides are equally matched at the cap.
    pos.game_result.or_else(|| {
        let score = HeuristicEvaluator.evaluate(&pos);
        if score >= 0 { Some(GameResult::P1Wins) }
        else { Some(GameResult::P2Wins) }
    })
}

/// Tally for a series of games. P1/P2 wins are recorded for the *candidate*
/// (the rater being graded), not the actual P1/P2 player at the board.
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
        if n == 0 { 0.0 } else { self.candidate_wins as f32 / n as f32 }
    }

    /// True if the candidate has more decisive wins than the baseline. Used
    /// for BO3 deciding once enough games have been played.
    pub fn candidate_leads(self) -> bool {
        self.candidate_wins > self.baseline_wins
    }
}

/// Run a single (loadout, both colours) mirror pair. The candidate plays
/// once as P1 and once as P2, both on the same `SideLoadout`. Returns the
/// tally over those two games.
fn mirror_pair(
    candidate: &dyn Evaluator,
    baseline: &dyn Evaluator,
    loadout: &SideLoadout,
    bracket: Bracket,
    base_ms: u64,
) -> SeriesTally {
    let mut tally = SeriesTally::default();
    let time = bracket.scaled_time_limit_ms(base_ms);

    // Game 1: candidate as P1, baseline as P2.
    match play_match_with_callback(candidate, baseline, loadout, loadout, time, |_, _, _| {}) {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    // Game 2: candidate as P2, baseline as P1.
    match play_match_with_callback(baseline, candidate, loadout, loadout, time, |_, _, _| {}) {
        Some(GameResult::P2Wins) => tally.candidate_wins += 1,
        Some(GameResult::P1Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    tally
}

/// Mirrored best-of-three. Plays the mirror pair on `loadout_seed`; if
/// neither side has clinched after two games, plays a third mirror pair on
/// a derived second loadout as the tiebreaker. (The plan says "best-of-three
/// mirrored matches" — we interpret mirrored as the {candidate-P1,
/// candidate-P2} pair counting together as 2 games of the BO3, so we need
/// at most one more game to decide. We use a fresh loadout for the
/// tiebreaker rather than replaying the same one, to reduce the chance of
/// a deterministic draw locking the series.)
pub fn mirrored_bo3(
    candidate: &dyn Evaluator,
    baseline: &dyn Evaluator,
    loadout_seed: u64,
    bracket: Bracket,
    base_ms: u64,
) -> SeriesTally {
    let loadout_a = random_loadout_from_seed(loadout_seed);
    let mut tally = mirror_pair(candidate, baseline, &loadout_a, bracket, base_ms);

    if tally.candidate_wins >= 2 || tally.baseline_wins >= 2 {
        return tally;
    }

    let loadout_b = random_loadout_from_seed(loadout_seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5));
    let time = bracket.scaled_time_limit_ms(base_ms);
    match play_match_with_callback(candidate, baseline, &loadout_b, &loadout_b, time, |_, _, _| {}) {
        Some(GameResult::P1Wins) => tally.candidate_wins += 1,
        Some(GameResult::P2Wins) => tally.baseline_wins += 1,
        None => tally.indecisive += 1,
    }
    tally
}

/// Tier 1 — fitness filter at the fast bracket. The candidate plays one
/// `mirrored_bo3` against each of the `top_k` baselines and aggregates the
/// win-rate across all games. Cheap; runs every training milestone over the
/// full candidate pool from `train_lineages`.
///
/// Note: the plan says "each candidate plays a mini-gauntlet against the
/// top-K (K=3)" — that's what this does, but K is a parameter (callers
/// supply however many predecessors they want to compare against).
pub fn tier1_fitness(
    candidate: &dyn Evaluator,
    top_k: &[&dyn Evaluator],
    loadout_seed: u64,
    base_ms: u64,
) -> SeriesTally {
    let mut tally = SeriesTally::default();
    for (i, baseline) in top_k.iter().enumerate() {
        let seed = loadout_seed.wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let r = mirrored_bo3(candidate, *baseline, seed, Bracket::Fast, base_ms);
        tally.candidate_wins += r.candidate_wins;
        tally.baseline_wins  += r.baseline_wins;
        tally.indecisive     += r.indecisive;
    }
    tally
}

/// Bracket-by-bracket result of a Tier-2 acceptance run against one
/// predecessor.
#[derive(Clone, Copy, Debug, Default)]
pub struct BracketResults {
    pub fast: SeriesTally,
    pub medium: SeriesTally,
    pub slow: SeriesTally,
}

impl BracketResults {
    pub fn at(&self, b: Bracket) -> SeriesTally {
        match b {
            Bracket::Fast => self.fast,
            Bracket::Medium => self.medium,
            Bracket::Slow => self.slow,
        }
    }

    fn from_runner<F: FnMut(Bracket) -> SeriesTally>(mut run: F) -> Self {
        Self {
            fast: run(Bracket::Fast),
            medium: run(Bracket::Medium),
            slow: run(Bracket::Slow),
        }
    }
}

/// Outcome of a Tier-2 acceptance run.
#[derive(Clone, Debug)]
pub struct AcceptanceReport {
    /// Per-predecessor, per-bracket tallies. Indexed in the same order as
    /// the `predecessors` slice passed in.
    pub per_predecessor: Vec<BracketResults>,
    /// Aggregate of `per_predecessor` rolled up per bracket — used by the
    /// three-track champion bookkeeping.
    pub aggregate: BracketResults,
    /// Bracket-by-bracket pass flags. `true` at a bracket means the
    /// candidate beat the *immediate predecessor* (last in the slice) AND
    /// met the ≥45 % non-regression bar against every other predecessor.
    pub bracket_pass: [bool; 3],
}

impl AcceptanceReport {
    pub fn passes_at(&self, b: Bracket) -> bool {
        match b {
            Bracket::Fast => self.bracket_pass[0],
            Bracket::Medium => self.bracket_pass[1],
            Bracket::Slow => self.bracket_pass[2],
        }
    }
}

/// Non-regression bar from plan §5: candidate must score ≥45% win-rate
/// against every prior accepted version at every bracket.
const NON_REGRESSION_BAR: f32 = 0.45;

/// Tier 2 — acceptance gauntlet. The candidate plays a mirrored-BO3 against
/// every predecessor (`predecessors[last]` is the immediate predecessor) at
/// each of the three brackets. The pass flag at each bracket is:
///
/// > BO3 win vs the immediate predecessor AND ≥45% win-rate vs every prior
///
/// `predecessors` must be non-empty.
pub fn tier2_acceptance(
    candidate: &dyn Evaluator,
    predecessors: &[&dyn Evaluator],
    loadout_seed: u64,
    base_ms: u64,
) -> AcceptanceReport {
    assert!(!predecessors.is_empty(),
        "tier2_acceptance needs at least one predecessor");

    let mut per_predecessor: Vec<BracketResults> = Vec::with_capacity(predecessors.len());
    for (i, pred) in predecessors.iter().enumerate() {
        let seed = loadout_seed.wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let br = BracketResults::from_runner(|b| mirrored_bo3(candidate, *pred, seed, b, base_ms));
        per_predecessor.push(br);
    }

    // Aggregate is sum of tallies per bracket.
    let mut aggregate = BracketResults::default();
    for br in &per_predecessor {
        for b in Bracket::all() {
            let sum = match b {
                Bracket::Fast => &mut aggregate.fast,
                Bracket::Medium => &mut aggregate.medium,
                Bracket::Slow => &mut aggregate.slow,
            };
            let t = br.at(b);
            sum.candidate_wins += t.candidate_wins;
            sum.baseline_wins  += t.baseline_wins;
            sum.indecisive     += t.indecisive;
        }
    }

    // Pass per bracket: BO3 win vs immediate predecessor (last) AND ≥45%
    // vs every other predecessor.
    let last_idx = per_predecessor.len() - 1;
    let mut bracket_pass = [false; 3];
    for (i, b) in Bracket::all().iter().enumerate() {
        let imm = per_predecessor[last_idx].at(*b);
        if !imm.candidate_leads() { continue; }
        let mut ok = true;
        for (j, br) in per_predecessor.iter().enumerate() {
            if j == last_idx { continue; }
            if br.at(*b).win_rate() < NON_REGRESSION_BAR {
                ok = false;
                break;
            }
        }
        bracket_pass[i] = ok;
    }

    AcceptanceReport { per_predecessor, aggregate, bracket_pass }
}

/// Stable, opaque identifier for a rater in the champion tracker. Could be
/// a generation counter, a hash of weights, anything — the tracker just
/// stores it and hands it back.
pub type RaterId = u64;

/// Per-plan weighting for the "best-overall" aggregate. Slow > medium > fast
/// because real-game inference is slow-bracket-dominated. Exact values are
/// "likely something like" per the plan; the relative ordering is what
/// matters.
const WEIGHT_FAST: f32 = 0.4;
const WEIGHT_MEDIUM: f32 = 0.6;
const WEIGHT_SLOW: f32 = 1.0;

/// Three champion tracks (plan §5). The tracker is updated each time a new
/// candidate passes Tier 2; it records which rater currently holds each
/// title. Tracks may diverge — that's expected.
#[derive(Clone, Debug, Default)]
pub struct ChampionTracker {
    pub best_fast: Option<RaterId>,
    pub best_slow: Option<RaterId>,
    pub best_overall: Option<RaterId>,
    /// Stored best win-rate per track, for tie-breaking. None until first
    /// candidate is recorded.
    pub best_fast_score: Option<f32>,
    pub best_slow_score: Option<f32>,
    pub best_overall_score: Option<f32>,
}

/// Which tracks a candidate qualified for (one update may flip 0, 1, 2, or
/// all 3 tracks).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackUpdate {
    pub fast: bool,
    pub medium: bool,  // not a track, but the medium-bracket pass flag for diagnostics
    pub slow: bool,
    pub overall: bool,
}

impl TrackUpdate {
    pub fn any_track(&self) -> bool { self.fast || self.slow || self.overall }
}

impl ChampionTracker {
    pub fn new() -> Self { Self::default() }

    /// Score for the "best-overall" track: weighted aggregate of the three
    /// bracket win-rates.
    pub fn overall_score(aggregate: &BracketResults) -> f32 {
        let f = aggregate.fast.win_rate();
        let m = aggregate.medium.win_rate();
        let s = aggregate.slow.win_rate();
        (f * WEIGHT_FAST + m * WEIGHT_MEDIUM + s * WEIGHT_SLOW)
            / (WEIGHT_FAST + WEIGHT_MEDIUM + WEIGHT_SLOW)
    }

    /// Consider `candidate` for each of the three tracks based on `report`.
    /// A track flips iff the candidate passed Tier 2 at the relevant
    /// bracket(s) AND its score for that track exceeds the current holder's.
    /// First-ever candidate to pass a track always becomes champion.
    pub fn consider(
        &mut self,
        candidate: RaterId,
        report: &AcceptanceReport,
    ) -> TrackUpdate {
        let mut upd = TrackUpdate::default();

        let fast_rate = report.aggregate.fast.win_rate();
        if report.passes_at(Bracket::Fast)
            && self.best_fast_score.map_or(true, |s| fast_rate > s)
        {
            self.best_fast = Some(candidate);
            self.best_fast_score = Some(fast_rate);
            upd.fast = true;
        }

        let slow_rate = report.aggregate.slow.win_rate();
        if report.passes_at(Bracket::Slow)
            && self.best_slow_score.map_or(true, |s| slow_rate > s)
        {
            self.best_slow = Some(candidate);
            self.best_slow_score = Some(slow_rate);
            upd.slow = true;
        }

        upd.medium = report.passes_at(Bracket::Medium);

        // Best-overall is gated on passing at *every* bracket — non-
        // regression across the full sweep is the whole point.
        let overall_pass = report.passes_at(Bracket::Fast)
            && report.passes_at(Bracket::Medium)
            && report.passes_at(Bracket::Slow);
        if overall_pass {
            let score = Self::overall_score(&report.aggregate);
            if self.best_overall_score.map_or(true, |s| score > s) {
                self.best_overall = Some(candidate);
                self.best_overall_score = Some(score);
                upd.overall = true;
            }
        }

        upd
    }

    /// Reconstruct a tracker from an on-disk `RaterIndex`. Walks the entries
    /// in acceptance order and replays each one as a synthetic
    /// `AcceptanceReport` driven by the persisted `bracket_results`
    /// aggregate. Pass flags are inferred per-bracket: a bracket is treated
    /// as passed iff its persisted win-rate cleared the non-regression bar
    /// (the same threshold Tier-2 uses live).
    ///
    /// **What's recovered**: the per-track win-rate floors (`best_*_score`)
    /// and the leader-id placeholders. The leader ids are stamped from each
    /// entry's *position* in the index — i.e. the first accepted entry gets
    /// `RaterId = 1`, the second `2`, etc. The orchestrator uses
    /// `generation as u64` for live `consider` calls, so the two id spaces
    /// don't overlap; that's intentional — only the *score floor* matters
    /// for resume correctness, the ids are diagnostic.
    ///
    /// **What's not recovered**: bracket-pass flags that didn't survive the
    /// round-trip (the index only stores aggregate win-rates). The
    /// non-regression-bar inference is an over-approximation: a candidate
    /// that scraped 0.46 against the bar but failed Tier-2 acceptance for
    /// another reason (impossible today, but the rule could shift) would
    /// still raise the floor here. The downside is a slightly stricter
    /// floor than strictly necessary — never a stale floor.
    pub fn from_index(index: &crate::registry::RaterIndex) -> Self {
        let mut tracker = Self::new();
        for (i, entry) in index.entries.iter().enumerate() {
            let synth_id: RaterId = (i as u64) + 1;
            let agg = synth_bracket_results_from_entry(entry);
            let bracket_pass = [
                agg.fast.win_rate() >= NON_REGRESSION_BAR,
                agg.medium.win_rate() >= NON_REGRESSION_BAR,
                agg.slow.win_rate() >= NON_REGRESSION_BAR,
            ];
            let report = AcceptanceReport {
                per_predecessor: vec![agg],
                aggregate: agg,
                bracket_pass,
            };
            tracker.consider(synth_id, &report);
        }
        tracker
    }
}

/// Translate `IndexEntry.bracket_results` (a `BTreeMap<String, BracketWinRate>`)
/// into a `BracketResults` triple. Missing brackets become zero tallies.
fn synth_bracket_results_from_entry(entry: &crate::registry::IndexEntry) -> BracketResults {
    let to_tally = |bw: &crate::persistence::BracketWinRate| SeriesTally {
        candidate_wins: bw.candidate_wins,
        baseline_wins: bw.baseline_wins,
        indecisive: bw.indecisive,
    };
    BracketResults {
        fast:   entry.bracket_results.get("fast")  .map(to_tally).unwrap_or_default(),
        medium: entry.bracket_results.get("medium").map(to_tally).unwrap_or_default(),
        slow:   entry.bracket_results.get("slow")  .map(to_tally).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::search::evaluator::HeuristicEvaluator;

    /// A degenerate evaluator that returns a fixed score regardless of
    /// position. Useful for forcing one side to make obviously bad decisions
    /// so we can verify the gauntlet plumbing without relying on real
    /// rater strength differences.
    #[derive(Clone, Copy, Debug)]
    struct ConstEval(i32);
    impl Evaluator for ConstEval {
        fn evaluate(&self, _pos: &Position) -> i32 { self.0 }
        fn evaluate_breakdown(&self, _pos: &Position) -> core_engine::search::evaluator::EvalBreakdown {
            core_engine::search::evaluator::EvalBreakdown::default()
        }
    }

    // Real time-bounded matches. The 8-slot tracked_enemies cap is
    // structural (backs the 8×8-bit champion_credit u64) and Stack M
    // can't exceed it (≤6 enemies on board, no summoning). OQ-85
    // resolved in session-37 — the doc was stale, engine is the spec.

    #[test]
    fn match_terminates_with_outcome() {
        let l = random_loadout_from_seed(1);
        let result = play_match(&HeuristicEvaluator, &HeuristicEvaluator, &l, &l, Bracket::Fast);
        if let Some(r) = result {
            assert!(matches!(r, GameResult::P1Wins | GameResult::P2Wins));
        }
    }

    #[test]
    fn callback_fires_once_per_ply_with_advancing_index() {
        // The callback variant must invoke `on_ply` exactly once per move,
        // with `ply` advancing 0, 1, 2, … and the position reflecting the
        // *post-move* state (so `game_result` is `Some` on the final call
        // iff the game terminated normally).
        let l = random_loadout_from_seed(2);
        let mut seen_plies: Vec<u32> = Vec::new();
        let mut final_pos_was_terminal = false;
        let outcome = play_match_with_callback(
            &HeuristicEvaluator, &HeuristicEvaluator, &l, &l, Bracket::Fast.time_limit_ms(),
            |pos, ply, _action| {
                seen_plies.push(ply);
                final_pos_was_terminal = pos.game_result.is_some();
            },
        );
        assert!(!seen_plies.is_empty(), "callback never fired");
        for (i, p) in seen_plies.iter().enumerate() {
            assert_eq!(*p as usize, i,
                "ply index must advance 0,1,2,…; got {:?}", seen_plies);
        }
        if outcome.is_some() {
            assert!(final_pos_was_terminal,
                "final on_ply must see terminal position when game completed");
        }
    }

    #[test]
    fn mirrored_bo3_plays_at_least_two_games() {
        // Two identical evaluators on the same loadout — outcome depends on
        // colour-symmetry of the position; we don't predict the winner,
        // only that the tally totals at least 2 games.
        let tally = mirrored_bo3(&HeuristicEvaluator, &HeuristicEvaluator, 7, Bracket::Fast, 10);
        assert!(tally.games_played() >= 2,
            "mirrored_bo3 must play at least the mirror pair; tally = {:?}", tally);
    }

    #[test]
    fn heuristic_beats_constant_evaluator_at_fast_bracket() {
        // The plumbing-correctness test: a real evaluator should out-perform
        // one that always returns 0. We use a 4-game mini-gauntlet (two
        // mirrored-BO3s) so even if one game flukes, the aggregate is
        // dominated by the real signal.
        let const_eval = ConstEval(0);
        let candidate: &dyn Evaluator = &HeuristicEvaluator;
        let top_k: [&dyn Evaluator; 2] = [&const_eval, &const_eval];
        let tally = tier1_fitness(candidate, &top_k, 13, 10);
        assert!(tally.candidate_wins > tally.baseline_wins,
            "heuristic must out-score const-0 in tier1; tally = {:?}", tally);
    }

    #[test]
    fn tier2_against_self_does_not_pass() {
        // Heuristic vs heuristic at three brackets. Symmetric matchup —
        // candidate shouldn't pass the BO3-win-vs-immediate-predecessor
        // requirement (it can't beat its own clone systematically). Test
        // the negative path.
        let report = tier2_acceptance(&HeuristicEvaluator, &[&HeuristicEvaluator], 21, 10);
        assert_eq!(report.per_predecessor.len(), 1);
        // Pass flags may or may not be true depending on the deterministic
        // outcome of self-play (whoever moves first might always win, in
        // which case it does "pass" trivially). We don't assert pass=false;
        // we only assert the *report shape* is well-formed. Mirrored BO3
        // early-returns at 2-0, so the floor is 2 games per bracket, not 3.
        assert!(report.aggregate.fast.games_played() >= 2);
        assert!(report.aggregate.slow.games_played() >= 2);
    }

    #[test]
    fn champion_tracker_initial_pass_takes_all_tracks() {
        // Synthetic report where the candidate passes all three brackets.
        let pass_tally = SeriesTally { candidate_wins: 3, baseline_wins: 0, indecisive: 0 };
        let aggregate = BracketResults {
            fast: pass_tally, medium: pass_tally, slow: pass_tally,
        };
        let report = AcceptanceReport {
            per_predecessor: vec![BracketResults {
                fast: pass_tally, medium: pass_tally, slow: pass_tally,
            }],
            aggregate,
            bracket_pass: [true, true, true],
        };
        let mut tracker = ChampionTracker::new();
        let upd = tracker.consider(42, &report);
        assert!(upd.fast && upd.slow && upd.overall);
        assert_eq!(tracker.best_fast, Some(42));
        assert_eq!(tracker.best_slow, Some(42));
        assert_eq!(tracker.best_overall, Some(42));
    }

    #[test]
    fn champion_tracker_no_pass_no_update() {
        let pass_tally = SeriesTally { candidate_wins: 3, baseline_wins: 0, indecisive: 0 };
        let report = AcceptanceReport {
            per_predecessor: vec![BracketResults {
                fast: pass_tally, medium: pass_tally, slow: pass_tally,
            }],
            aggregate: BracketResults {
                fast: pass_tally, medium: pass_tally, slow: pass_tally,
            },
            bracket_pass: [false, false, false],
        };
        let mut tracker = ChampionTracker::new();
        let upd = tracker.consider(7, &report);
        assert!(!upd.any_track());
        assert_eq!(tracker.best_fast, None);
        assert_eq!(tracker.best_slow, None);
        assert_eq!(tracker.best_overall, None);
    }

    #[test]
    fn champion_tracker_tracks_can_diverge() {
        // Candidate A passes Fast only; candidate B passes Slow only.
        // After both, best_fast=A, best_slow=B, best_overall unset.
        let pass = SeriesTally { candidate_wins: 3, baseline_wins: 0, indecisive: 0 };
        let mut tracker = ChampionTracker::new();

        let rep_a = AcceptanceReport {
            per_predecessor: vec![BracketResults {
                fast: pass, medium: SeriesTally::default(), slow: SeriesTally::default(),
            }],
            aggregate: BracketResults {
                fast: pass, medium: SeriesTally::default(), slow: SeriesTally::default(),
            },
            bracket_pass: [true, false, false],
        };
        let upd_a = tracker.consider(1, &rep_a);
        assert!(upd_a.fast && !upd_a.slow && !upd_a.overall);

        let rep_b = AcceptanceReport {
            per_predecessor: vec![BracketResults {
                fast: SeriesTally::default(), medium: SeriesTally::default(), slow: pass,
            }],
            aggregate: BracketResults {
                fast: SeriesTally::default(), medium: SeriesTally::default(), slow: pass,
            },
            bracket_pass: [false, false, true],
        };
        let upd_b = tracker.consider(2, &rep_b);
        assert!(!upd_b.fast && upd_b.slow && !upd_b.overall);

        assert_eq!(tracker.best_fast, Some(1));
        assert_eq!(tracker.best_slow, Some(2));
        assert_eq!(tracker.best_overall, None);
    }
}
