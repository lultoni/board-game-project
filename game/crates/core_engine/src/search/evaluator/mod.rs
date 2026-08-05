//! Evaluation seam: the [`Evaluator`] trait, the [`builtin`] registry, and the
//! score conventions every evaluator shares.
//!
//! Score convention: positive = P1 advantage, negative = P2 advantage.
//! Win/loss are represented as ±(MATE_SCORE - depth_to_mate) so shorter wins
//! score higher and the search prefers fast mates.
//!
//! ## Entry point — read this first
//!
//! A caller (the search, the UI eval bar) hands a `Position` to an `Evaluator`
//! and gets back a P1-POV `i32`. That is the whole contract. To write a new
//! evaluator you implement **one trait with two required methods**:
//!
//! - [`Evaluator::evaluate`] — the scalar score (this is what the search calls).
//! - [`Evaluator::evaluate_report`] — a breakdown for the UI panel; return
//!   [`EvalReport::single`] if you have no term structure.
//!
//! The other five `*_acc` methods are OPTIONAL (default no-ops) — override them
//! only for an NNUE-style incremental accumulator. A hand-rolled evaluator
//! ignores them entirely (see [`custom::CustomEvaluator`] for the minimal shape).
//!
//! ## The registry — how an evaluator becomes selectable
//!
//! [`builtin::BUILTINS`] is a flat list mapping a stable `id` + display `label`
//! to a constructor. Registering a new evaluator is TWO edits, both here at the
//! top level: write your struct (a file next to `custom.rs`) and add ONE
//! `BUILTINS` line. The struct does not self-register; the registry names it.
//!
//! ## Where the code lives
//!
//! - `mod.rs` (this file) — the seam: the trait, `MATE_SCORE`, the free
//!   `evaluate`/`evaluate_report` shortcuts to the default heuristic, and the
//!   `builtin` registry. The `HeuristicEvaluator`/`ParamHeuristicEvaluator`
//!   shells live here too (they delegate into `heuristic`).
//! - [`report`] — `EvalReport`/`TermEntry`/`BreakdownDetail`: the shared wire
//!   type sent to the frontend. Any evaluator may build one.
//! - [`heuristic`] — the DEFAULT evaluator, fully self-contained. Its
//!   term-registry machinery (`params`, `context`, `term`, `terms`) is its
//!   private business; nothing else routes through it.
//! - [`custom`] — the designer's hand-rolled evaluator, one file, implements the
//!   trait directly.
//!
//! ============================================================
//! Design philosophy (load-bearing - read before changing eval)
//! ============================================================
//!
//! Source: designer's eval-function notes (Session 28 inbox, Perplexity
//! transcript). These principles outlive any particular term set.
//!
//! 1. WIN/LOSS OVERRULES EVERYTHING.
//!    Captured-King = ±MATE_SCORE. Checked before any term. Encoded as
//!    ±(MATE_SCORE - depth) so a mate-in-2 scores higher than a mate-in-5.
//! 2. "FASTEST PATH" LIVES IN THE SEARCH, NOT IN EVAL.
//!    Eval scores the position as-is; tempo-to-resolution is search's job.
//! 3. AFTER WIN/LOSS: COUNT REAL THINGS. Material first (pieces, HP, armor,
//!    money, equipped skills). Must beat random play before anything fancier.
//! 4. TWO ANGLES ON EVERY ADVANTAGE - TEMPO AND MONEY.
//! 5. EVAL COST IS A FIRST-CLASS BUDGET. A cheap eval at depth 6 beats an
//!    expensive one at depth 1. Phase/stage gating (in the heuristic's
//!    `score_*_all`) skips terms that can't matter for the current position.
//! 6. START STUPID. Material-only first; every later term proves itself against
//!    that baseline.

pub mod report;
pub mod heuristic;
pub mod custom;

use crate::game_logic::action::Undo;
use crate::state::Position;

pub use heuristic::params::EvalParams;
pub use heuristic::term::{EvalTerm, PieceContext};
pub use heuristic::context::EvalContext;
pub use heuristic::context::GameStage;
pub use report::{BreakdownDetail, EvalReport, PieceTermBreakdown, TermEntry};

pub const MATE_SCORE: i32 = 1_000_000;

// Const aliases to the default params, kept so the existing test module (and
// any external callers) that referenced the old top-level consts compile
// unchanged - and to make the faithful port self-evident. Only referenced from
// the test module, so allow dead_code in non-test builds.
#[cfg(test)] pub(crate) const CHAMPION_VALUE:  i32 = EvalParams::DEFAULT.champion_value;
#[cfg(test)] pub(crate) const HP_PER_POINT:    i32 = EvalParams::DEFAULT.hp_per_point;
#[cfg(test)] pub(crate) const ARMOR_PER_POINT: i32 = EvalParams::DEFAULT.armor_per_point;
#[cfg(test)] pub(crate) const TEMPO_PER_ACTION: i32 = EvalParams::DEFAULT.tempo_per_action;

/// Scalar P1-POV static eval with the default weights. `+`=P1, `±MATE_SCORE`
/// for terminals. Convenience shortcut for callers that don't hold an
/// `Evaluator`; delegates to the heuristic's monomorphic search-leaf path.
pub fn evaluate(pos: &Position) -> i32 {
    heuristic::evaluate_scalar(pos, &EvalParams::DEFAULT)
}

/// Dynamic breakdown for the default heuristic. `detail` selects aggregate-only
/// or full per-piece decomposition. Used by the UI eval panel + telemetry.
pub fn evaluate_report(pos: &Position, detail: report::BreakdownDetail) -> EvalReport {
    heuristic::evaluate_report(pos, &EvalParams::DEFAULT, detail)
}

/// Opaque, evaluator-owned incremental-eval state for one search node.
///
/// `core_engine` never inspects the inner value - only the `Evaluator` that
/// produced it downcasts back to its concrete type. This is the layering seam:
/// the incremental NNUE `Accumulator` lives in `nn_trainer` (which depends on
/// `core_engine`), so the search can't name it; it threads an `AccHandle`
/// through make/unmake generically instead. `None` = the evaluator keeps no
/// incremental state (the default; `HeuristicEvaluator`).
///
/// `Send` matches the `Evaluator: Send` bound; the handle lives in `SearchCtx`
/// on the single search thread and never crosses threads. Deliberately NOT
/// `Clone` - only the producing evaluator knows the concrete type, so cloning
/// goes through [`Evaluator::clone_acc`].
pub struct AccHandle(Option<Box<dyn core::any::Any + Send>>);

impl AccHandle {
    /// The empty handle - an evaluator with no incremental state.
    #[inline]
    pub fn none() -> Self { AccHandle(None) }

    /// Wrap concrete evaluator-owned state.
    #[inline]
    pub fn new<T: core::any::Any + Send>(state: T) -> Self {
        AccHandle(Some(Box::new(state)))
    }

    #[inline]
    pub fn is_none(&self) -> bool { self.0.is_none() }

    /// Downcast the inner state to `&T` (None if empty or the type mismatches).
    #[inline]
    pub fn downcast_ref<T: core::any::Any>(&self) -> Option<&T> {
        self.0.as_ref().and_then(|b| b.downcast_ref::<T>())
    }

    /// Downcast the inner state to `&mut T` (None if empty or the type mismatches).
    #[inline]
    pub fn downcast_mut<T: core::any::Any>(&mut self) -> Option<&mut T> {
        self.0.as_mut().and_then(|b| b.downcast_mut::<T>())
    }
}

/// Position-rater interface. The search calls `evaluate` once per leaf; an
/// `Evaluator` impl returns a P1-POV score (positive = P1, ±MATE_SCORE for
/// terminals).
///
/// **Send-only** bound: the search is single-threaded but evaluators are owned
/// by `Match` (one per AI seat) and moved between thread-pool tasks. Code that
/// shares an evaluator across threads re-asserts `+ Sync` locally.
///
/// ## Incremental-accumulator seam (default no-op)
///
/// An evaluator MAY maintain an incrementally-updated accumulator that the
/// search threads through make/unmake (NNUE). The five `*_acc` methods below
/// default to a no-op / scratch-path fallback, so evaluators that don't use one
/// (the default `HeuristicEvaluator`) pay nothing and stay object-safe (no
/// `Self`-typed params, only `core_engine` types + the opaque [`AccHandle`]).
/// Save/restore lifecycle: the search clones the current handle before a
/// `make`, `push_acc`s it forward, reads it at leaves via `eval_acc`, and
/// restores the saved clone on `unmake`.
pub trait Evaluator: Send {
    fn evaluate(&self, pos: &Position) -> i32;

    /// Dynamic breakdown for the UI / diagnostics. `detail` selects aggregate vs
    /// per-piece. The default heuristic decomposes into its terms; an NN
    /// evaluator returns a single synthetic term (it has no term structure).
    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport;

    /// True iff this evaluator maintains an incremental accumulator. When
    /// false, the search skips ALL handle machinery (empty stack, no clones).
    #[inline]
    fn uses_accumulator(&self) -> bool { false }

    /// Build the root accumulator for `pos` (full recompute).
    #[inline]
    fn fresh_acc(&self, _pos: &Position) -> AccHandle { AccHandle::none() }

    /// Independent copy of `h` to stash before a `make` (save/restore).
    #[inline]
    fn clone_acc(&self, _h: &AccHandle) -> AccHandle { AccHandle::none() }

    /// Advance `h` in place AFTER `make(pos, action)` returned `undo`, with
    /// `pos` already at the post-make state.
    #[inline]
    fn push_acc(&self, _h: &mut AccHandle, _undo: &Undo, _pos: &Position) {}

    /// Evaluate the CURRENT `pos` via the incremental handle (leaf read). MUST
    /// return bit-identically to `self.evaluate(pos)`. The default falls back to
    /// the scratch path, so a `None`/mis-wired handle can never diverge.
    #[inline]
    fn eval_acc(&self, _h: &AccHandle, pos: &Position) -> i32 { self.evaluate(pos) }
}

/// The concrete heuristic evaluators live with their implementation, in
/// [`heuristic`] — re-exported here so callers name them at the seam
/// (`evaluator::HeuristicEvaluator`) without reaching into the submodule.
/// `HeuristicEvaluator` is the zero-config default; `ParamHeuristicEvaluator`
/// is the same term math with a custom [`EvalParams`] weight set. For a
/// genuinely different evaluator, add a sibling module to [`heuristic`] /
/// [`custom`] and register it in [`builtin::BUILTINS`].
pub use heuristic::{HeuristicEvaluator, ParamHeuristicEvaluator};

/// Builtin evaluator registry (ns-54). The single place where `core_engine`'s
/// own concrete evaluators are enumerated for UI selection. Each entry maps a
/// stable `id` (persisted in settings, sent over IPC) and a human `label` to a
/// constructor. To add a new evaluator: write your struct implementing
/// [`Evaluator`] (a fresh file with its own term registry, a tuned
/// [`ParamHeuristicEvaluator`], whatever), then add ONE entry here.
///
/// NN raters are NOT here — they live in `nn_trainer` (which depends on this
/// crate) and are unioned in at the wrapper layer. This registry is only the
/// pure, in-crate evaluators.
pub mod builtin {
    use super::{Evaluator, HeuristicEvaluator};

    /// A selectable builtin evaluator: stable id, display label, constructor.
    pub struct BuiltinEvaluator {
        pub id:    &'static str,
        pub label: &'static str,
        pub make:  fn() -> Box<dyn Evaluator + Send + Sync>,
    }

    /// All builtin evaluators, in display order. The first entry is the default.
    /// ADD NEW EVALUATORS HERE (one line each).
    pub const BUILTINS: &[BuiltinEvaluator] = &[
        BuiltinEvaluator {
            id: "heuristic",
            label: "Heuristic (default)",
            make: || Box::new(HeuristicEvaluator),
        },
        // Editable scaffold for a hand-rolled evaluator — see `custom.rs`.
        BuiltinEvaluator {
            id: "custom-stub",
            label: "Custom (stub)",
            make: || Box::new(super::custom::CustomEvaluator),
        },
        // Example weight-variant / experimental slot — add real entries here:
        // BuiltinEvaluator {
        //     id: "experimental-v2",
        //     label: "Experimental v2",
        //     make: || Box::new(crate::search::evaluator::experimental_v2::Eval),
        // },
    ];

    /// Look up a builtin by id and construct it. `None` if the id is unknown.
    pub fn make(id: &str) -> Option<Box<dyn Evaluator + Send + Sync>> {
        BUILTINS.iter().find(|b| b.id == id).map(|b| (b.make)())
    }
}


#[cfg(test)]
mod builtin_tests {
    use super::*;

    #[test]
    fn builtin_registry_has_heuristic_default_first() {
        // The first entry is the default and must be the heuristic.
        assert_eq!(builtin::BUILTINS[0].id, "heuristic");
        // Every entry has a non-empty id + label and constructs a working evaluator.
        for b in builtin::BUILTINS {
            assert!(!b.id.is_empty() && !b.label.is_empty());
            let ev = (b.make)();
            // Constructs and scores the start position without panicking.
            let _ = ev.evaluate(&Position::setup_stack_m());
        }
    }

    #[test]
    fn builtin_make_resolves_and_rejects() {
        assert!(builtin::make("heuristic").is_some());
        assert!(builtin::make("does-not-exist").is_none());
    }

    #[test]
    fn param_heuristic_matches_default_heuristic_with_default_params() {
        // A ParamHeuristicEvaluator built with DEFAULT params must score
        // identically to the plain HeuristicEvaluator.
        let pos = Position::setup_stack_m();
        let a = HeuristicEvaluator.evaluate(&pos);
        let b = ParamHeuristicEvaluator::new(EvalParams::DEFAULT).evaluate(&pos);
        assert_eq!(a, b);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{GameResult, Player};
    use crate::state::position::Phase;
    use crate::game_logic::skills::Skill;

    /// Place a piece on `sq` for `player` of `kind` (0=King, 1=Champion, 2=Guard)
    /// with mailbox `entry`. Mirrors the structure of `make_unmake::tests::place`
    /// (which is pub(super)-scoped and not reachable from here).
    fn place(p: &mut Position, sq: u8, player: Player, kind: u8, entry: MailboxEntry) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        match kind {
            0 => p.kings     = p.kings     | bit,
            1 => p.champions = p.champions | bit,
            _ => p.guards    = p.guards    | bit,
        }
        p.mailbox[sq as usize] = entry;
    }

    #[test]
    fn empty_board_is_zero() {
        let pos = Position::empty();
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn terminal_p1_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn terminal_overrules_material() {
        // Place a P2 Champion (which would give P1 a negative material score)
        // but set game_result = P1Wins. Terminal must short-circuit the loop
        // and return exactly +MATE_SCORE.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn mirrored_single_champion_is_zero() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn hp_differential() {
        // P1 Champion HP=2 vs P2 Champion HP=1, no armor, no skills.
        // Differential is exactly HP_PER_POINT.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));
        assert_eq!(evaluate(&pos), HP_PER_POINT);
    }

    #[test]
    fn armor_differential() {
        // P1 Champion armor=1 vs P2 Champion armor=0, identical otherwise.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2).with_armor(0));
        assert_eq!(evaluate(&pos), ARMOR_PER_POINT);
    }

    #[test]
    fn money_differential() {
        // E3: money value requires owned skills (cap = max_skill_cost x actions).
        // Give both sides an identical piece so both have a skill_cost baseline,
        // but different money. Differential must be positive when P1 has more.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        pos.p1_money = 10;
        pos.p2_money = 4;
        pos.actions_remaining = 2;
        pos.current_phase = Phase::Move;
        // With cap = 2 (Lance cost) x 2 (actions) = 4, both sides plateau at
        // MONEY_PER_UNIT x cap / 2 = 50 each - but P2 is under cap so gets less
        // than the plateau. P1 differential should be non-negative and small.
        // The load-bearing invariant: score should be non-negative and reflect
        // P1's money advantage.
        assert!(evaluate(&pos) >= 0, "P1 money advantage should not be negative");
    }

    #[test]
    fn money_symmetric_when_equal() {
        // Two symmetric pieces + equal money → material terms cancel.
        // With actions_remaining=0 the tempo term (E8) is also 0, so total=0.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        pos.p1_money = 5;
        pos.p2_money = 5;
        pos.actions_remaining = 0;
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn skill_equipped_beats_unequipped() {
        // P1 Champion with Lance equipped vs P2 Champion bare. Assert P1 > P2
        // (skill contributes positively) rather than the raw skill_value
        // (E4 gates by availability, so exact value depends on money).
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2));
        // Money high enough that Lance availability saturates.
        pos.p1_money = 20;
        pos.p2_money = 20;
        pos.actions_remaining = 2;
        assert!(evaluate(&pos) > 0);
    }

    #[test]
    fn stack_m_setup_is_zero() {
        // Canonical start: identical material on both sides, 6 money each.
        // Under E8 (tempo), P1-to-move contributes +TEMPO_PER_ACTION * 2 = +30;
        // material/skills/money/mobility/exposure/coverage are perfectly mirrored
        // and cancel. So the invariant is: score == tempo bonus for the moving side.
        let pos = Position::setup_stack_m();
        assert_eq!(evaluate(&pos), TEMPO_PER_ACTION * pos.actions_remaining as i32);
    }

    #[test]
    fn sign_convention_p1_positive_p2_negative() {
        // A lone P1 Champion → positive score.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) > 0);

        // Symmetric: a lone P2 Champion → negative.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) < 0);
    }

    #[test]
    fn additivity() {
        // Build three positions:
        //   A: P1 +1 HP advantage (P1 HP=2, P2 HP=1, no armor)
        //   B: P1 +1 armor advantage (HP=2 both, P1 armor=1, P2 armor=0)
        //   AB: both effects combined
        // Assert evaluate(AB) == evaluate(A) + evaluate(B).
        let mut a = Position::empty();
        place(&mut a, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut a, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        let mut b = Position::empty();
        place(&mut b, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut b, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));

        let mut ab = Position::empty();
        place(&mut ab, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut ab, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        assert_eq!(evaluate(&ab), evaluate(&a) + evaluate(&b));
    }

    #[test]
    fn maxed_piece_formula() {
        // Pin the math for a lone P1 Champion HP=2 armor=2 skills=Tempest+Charge,
        // no enemies, no money. Under the current terms:
        //   - mobility (ns-43 Term 3a) = champion real movement-space: 8 empty
        //     neighbours at d4 x champ_mob_per_sq.
        //   - skill_term (E4) gated by money=0 → availability=0 → 0.
        //   - champion_threat (Term 3b) - no enemies/allies in range → 0.
        //   - exposure (E2) = 0 (no attackers); coverage (E6) = 0 (no guards).
        //   - tempo (E8) skipped in Draft; offensive_range needs money → 0.
        // Result: material + hp + armor + champion movement-space.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default()
                .with_hp(2)
                .with_armor(2)
                .with_skill1(Skill::Tempest as u8)
                .with_skill2(Skill::Charge as u8));
        let champ_mob = 8 * EvalParams::DEFAULT.champ_mob_per_sq; // d4: 8 empty neighbours
        let expected = CHAMPION_VALUE
            + 2 * HP_PER_POINT
            + 2 * ARMOR_PER_POINT
            + champ_mob;
        assert_eq!(evaluate(&pos), expected);
    }

    #[test]
    fn asymmetric_kings_no_panic() {
        // Malformed: P2 has a king, P1 doesn't, but game_result is None.
        // Eval must return a finite i32 without panicking.
        let mut pos = Position::empty();
        place(&mut pos, 4, Player::P2, 0, MailboxEntry::default().with_hp(2));
        // game_result stays None.
        let s = evaluate(&pos);
        // We don't assert a specific value - just that it computed.
        // KING_MATERIAL is 0, so the king contributes only its HP. P1 has nothing.
        // The point is: no panic, no overflow.
        assert!(s > i32::MIN && s < i32::MAX);
    }

    #[test]
    fn per_piece_report_sums_to_total_stack_m() {
        // Invariant: the per-piece decomposition must reconstruct the scalar
        // total, for the canonical Stack M opening position.
        let pos = Position::setup_stack_m();
        assert_report_reconstructs_total(&pos);
    }

    #[test]
    fn per_piece_report_sums_to_total_asymmetric() {
        // A P1 Champion adjacent to a P2 Guard - exercises exposure, coverage,
        // mobility on both sides.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_armor(1)
                .with_skill1(crate::game_logic::skills::Skill::Lance as u8));
        place(&mut pos, 29, Player::P2, 2, MailboxEntry::default().with_hp(1));
        pos.p1_money = 4;
        pos.p2_money = 2;
        pos.actions_remaining = 2;
        pos.current_phase = Phase::Move;
        pos.to_move = Player::P1;
        assert_report_reconstructs_total(&pos);
    }

    /// The core consistency invariant of the per-piece report: summing every
    /// piece's owner-signed `piece_total` and every side term's `signed` yields
    /// exactly `evaluate(pos)`. Because the rows come from the SAME term pass as
    /// the scalar path, this is definitional - but we pin it against drift.
    fn assert_report_reconstructs_total(pos: &Position) {
        let r = evaluate_report(pos, BreakdownDetail::PerPiece);
        assert_eq!(r.total, evaluate(pos), "report.total must equal evaluate()");
        let pieces = r.pieces.as_ref().expect("PerPiece requested → Some");
        let piece_sum: i32 = pieces.iter().map(|p| p.piece_total).sum();
        let side_sum: i32 = r.side_terms.iter().map(|t| t.signed).sum();
        assert_eq!(piece_sum + side_sum, r.total,
            "Σ owner-signed piece_total + Σ side_terms.signed must equal total");
    }

    #[test]
    fn report_terminal_p1_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        let r = evaluate_report(&pos, BreakdownDetail::PerPiece);
        assert_eq!(r.total, MATE_SCORE);
        assert!(r.terminal);
        assert!(r.terms.is_empty() && r.side_terms.is_empty());
        assert!(r.pieces.is_none(), "terminal report carries no per-piece rows");
    }

    #[test]
    fn coverage_requires_dual_adjacency_to_defender_and_ring_square() {
        // Regression: E6 coverage only counts an empty ring square as shielded
        // when a friendly Guard is adjacent to BOTH the defender and the ring
        // square, AND an enemy is within r3 in that direction (threat-gated).
        // A distant Guard cannot contribute.

        // Case A: Guard at c3 (sq 18) is NOT adjacent to defender at e4 (sq 28)
        // - chebyshev distance 2. Coverage MUST be 0 even with an enemy at e6.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 18, Player::P1, 2, MailboxEntry::default().with_hp(2));
        place(&mut pos, 44, Player::P2, 2, MailboxEntry::default().with_hp(2));
        assert_eq!(term_pair(&pos, "coverage").0, 0,
            "guard at sq 18 is not adjacent to defender at sq 28; coverage must be 0");

        // Case B: Guard at f4 (sq 29) IS adjacent to defender at e4 (sq 28), and
        // enemy champion at e6 (sq 44) gates in the north-facing ring → coverage > 0.
        let mut pos2 = Position::empty();
        place(&mut pos2, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos2, 29, Player::P1, 2, MailboxEntry::default().with_hp(2));
        place(&mut pos2, 44, Player::P2, 2, MailboxEntry::default().with_hp(2));
        assert!(term_pair(&pos2, "coverage").0 > 0,
            "dual-adjacent guard on a threatened ring produces positive coverage");
    }

    // ============================================================
    // SEE terms (ns-53): hanging_piece (per-piece, SEE-gated) and
    // king_tempo (side-level, reuses is_king_threatened).
    // ============================================================

    #[test]
    fn hanging_piece_zero_when_unattacked() {
        // A lone P1 champion with no enemy in reach → not attacked → term absent.
        let mut p = Position::empty();
        place(&mut p, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(term_pair(&p, "hanging_piece"), (0, 0),
            "an unattacked piece is not hanging (and pays no SEE cost)");
    }

    #[test]
    fn hanging_piece_penalises_a_losing_exchange() {
        // P1 guard at d4 (27) attacked by an ADJACENT P2 champion at e4 (28).
        // The guard is undefended, so an enemy capture nets material → the guard
        // registers as hanging on P1's side, a P1-POV penalty. (Both pieces are
        // mutually adjacent here, so the term may also see the champion as
        // capturable; the assertion below only pins the guard-is-hanging signal.)
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 2, MailboxEntry::default().with_hp(1));
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;
        let (p1, _p2) = term_pair(&p, "hanging_piece");
        assert!(p1 > 0, "P1's undefended attacked guard should register as hanging");
        assert!(term_signed(&p, "hanging_piece") < 0,
            "with only P1's guard truly losing the exchange, hanging is a net P1-POV penalty");
    }

    #[test]
    fn king_tempo_penalises_threatened_king() {
        // P1 king at e1 (4) with a P2 champion adjacent at d2 (11): in the Move
        // phase the champion is one Move-Attack from capturing the king.
        let mut p = Position::empty();
        place(&mut p, 4, Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut p, 11, Player::P2, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2)); // well-formed P2 king
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;
        let (p1, p2) = term_pair(&p, "king_tempo");
        assert_eq!(p1, EvalParams::DEFAULT.king_tempo_penalty, "P1 king is one tempo from capture");
        assert_eq!(p2, 0, "P2 king is safe");
        assert!(term_signed(&p, "king_tempo") < 0, "king_tempo is a P1-POV penalty here");
    }

    #[test]
    fn king_tempo_zero_when_safe() {
        // Kings far apart, no threat → term absent.
        let mut p = Position::empty();
        place(&mut p, 4,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2));
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;
        assert_eq!(term_pair(&p, "king_tempo"), (0, 0), "no king is threatened");
    }

    // ============================================================
    // WastedModifier term (E10, ns-43) - a live Focus/Charge bit with no
    // castable consumer this Skill phase is a penalty on the holding side.
    // ============================================================

    use crate::state::position::modifier_bits;

    /// Build a lone-champion position in the Skill phase for the given player,
    /// equipping `skill_id` on that champion, with the given money + buff bits.
    fn skill_phase_champ(to_move: Player, skill_id: u8, money: u16, mods: u8, actions: u8) -> Position {
        let mut p = Position::empty();
        let (sq, kind) = (28u8, 1u8);
        place(&mut p, sq, to_move, kind,
            MailboxEntry::default().with_hp(2).with_skill1(skill_id));
        match to_move { Player::P1 => p.p1_money = money, Player::P2 => p.p2_money = money }
        p.current_phase = Phase::Skill;
        p.to_move = to_move;
        p.actions_remaining = actions;
        p.round_number = 6;
        p.pending_modifiers = mods;
        p
    }

    #[test]
    fn wasted_focus_penalised_when_no_offensive_skill() {
        // Champion equips only Shield (defensive) → Focus has no consumer.
        let p = skill_phase_champ(Player::P1, Skill::Shield as u8, 10, modifier_bits::FOCUS, 2);
        let with_focus = evaluate(&p);
        // Same position with the buff bit cleared: no penalty.
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        let baseline = evaluate(&clean);
        // P1 holds the wasted buff → its eval is LOWER than baseline by exactly
        // wasted_modifier_per_cost x Focus cost.
        let expected_pen = EvalParams::DEFAULT.wasted_modifier_per_cost
            * crate::game_logic::skills::skill_cost(Skill::Focus) as i32;
        assert_eq!(baseline - with_focus, expected_pen);
    }

    #[test]
    fn wasted_focus_not_penalised_when_offensive_skill_castable() {
        // Champion equips Lance (Strike) and can afford it → Focus is consumable.
        let p = skill_phase_champ(Player::P1, Skill::Lance as u8, 10, modifier_bits::FOCUS, 2);
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        assert_eq!(evaluate(&p), evaluate(&clean), "Focus has a consumer → no penalty");
    }

    #[test]
    fn wasted_charge_penalised_when_only_shove_castable() {
        // Shove is offensive (consumes Focus) but NOT a Strike (does not consume
        // Charge). So Charge is wasted even though Shove is castable.
        let p = skill_phase_champ(Player::P1, Skill::Shove as u8, 10, modifier_bits::CHARGE, 2);
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        let expected_pen = EvalParams::DEFAULT.wasted_modifier_per_cost
            * crate::game_logic::skills::skill_cost(Skill::Charge) as i32;
        assert_eq!(evaluate(&clean) - evaluate(&p), expected_pen);
    }

    #[test]
    fn wasted_modifier_dormant_outside_skill_phase() {
        // Same live Focus bit but in the Move phase → term is inactive.
        let mut p = skill_phase_champ(Player::P1, Skill::Shield as u8, 10, modifier_bits::FOCUS, 2);
        p.current_phase = Phase::Move;
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        assert_eq!(evaluate(&p), evaluate(&clean), "not Skill phase → no penalty");
    }

    #[test]
    fn wasted_modifier_penalised_when_no_actions_left() {
        // Lance is equipped+affordable, but actions_remaining=0 → nothing is
        // castable, so the Focus bit is wasted.
        let p = skill_phase_champ(Player::P1, Skill::Lance as u8, 10, modifier_bits::FOCUS, 0);
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        let expected_pen = EvalParams::DEFAULT.wasted_modifier_per_cost
            * crate::game_logic::skills::skill_cost(Skill::Focus) as i32;
        assert_eq!(evaluate(&clean) - evaluate(&p), expected_pen);
    }

    #[test]
    fn wasted_modifier_p2_sign() {
        // P2 holds the wasted buff → P1-POV eval goes UP (penalty on P2).
        let p = skill_phase_champ(Player::P2, Skill::Shield as u8, 10, modifier_bits::FOCUS, 2);
        let mut clean = p.clone();
        clean.pending_modifiers = 0;
        let expected_pen = EvalParams::DEFAULT.wasted_modifier_per_cost
            * crate::game_logic::skills::skill_cost(Skill::Focus) as i32;
        assert_eq!(evaluate(&p) - evaluate(&clean), expected_pen);
    }

    // ============================================================
    // Stage infra (ns-43) - advantage score + game-stage classifier.
    // Infra-only: no term consumes these yet, so goldens are unchanged.
    // ============================================================

    #[test]
    fn advantage_sign_tracks_who_is_ahead() {
        let params = EvalParams::DEFAULT;
        // P1 has an extra champion → P1 ahead → advantage > 0.
        let mut p = Position::empty();
        place(&mut p, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 1,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        let ctx = EvalContext::new(&p, &params);
        assert!(ctx.advantage > 0, "P1 up a champion → advantage > 0");

        // Mirror: P2 up a champion → advantage < 0.
        let mut q = Position::empty();
        place(&mut q, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut q, 62, Player::P2, 1, MailboxEntry::default().with_hp(2));
        place(&mut q, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        let ctx2 = EvalContext::new(&q, &params);
        assert!(ctx2.advantage < 0, "P2 up a champion → advantage < 0");
    }

    #[test]
    fn stage_opening_at_full_material() {
        let params = EvalParams::DEFAULT;
        let pos = Position::setup_stack_m(); // full material, round 0
        let ctx = EvalContext::new(&pos, &params);
        assert_eq!(ctx.stage, GameStage::Opening,
            "full-material Stack-M start classifies as Opening");
    }

    #[test]
    fn stage_progresses_to_end_as_material_drops() {
        let params = EvalParams::DEFAULT;
        // Only two lone kings left, early round → below end threshold → End.
        let mut p = Position::empty();
        place(&mut p, 4,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2));
        let ctx = EvalContext::new(&p, &params);
        assert_eq!(ctx.stage, GameStage::End, "two lone kings → End");
    }

    #[test]
    fn stage_round_bias_pushes_later() {
        let params = EvalParams::DEFAULT;
        // classify_stage directly: same material, higher round → not-earlier.
        let mat = params.stage_mid_threshold + 100; // just into Opening at round 0
        assert_eq!(heuristic::context::classify_stage(mat, 0, &params), GameStage::Opening);
        // A high round biases the same material toward Mid/End.
        let late = heuristic::context::classify_stage(mat, 30, &params);
        assert_ne!(late, GameStage::Opening, "30 rounds elapsed pushes past Opening");
    }

    // ============================================================
    // GuardIsolation term (E11, ns-43) - a guard locally outnumbered
    // (more enemies than friendlies within radius) is penalised.
    // ============================================================

    // ============================================================
    // GuardIsolation term (E11, ns-43) - a guard locally outnumbered
    // (more enemies than friendlies within radius) is penalised.
    // ============================================================

    /// Test helper: find an aggregate term's `(p1, p2)` in a report (searches
    /// both per-piece-aggregate `terms` and `side_terms`); `(0,0)` if absent.
    fn term_pair(pos: &Position, name: &str) -> (i32, i32) {
        let r = evaluate_report(pos, BreakdownDetail::Aggregate);
        r.terms.iter().chain(r.side_terms.iter())
            .find(|t| t.name == name)
            .map(|t| (t.p1, t.p2)).unwrap_or((0, 0))
    }

    /// Test helper: an aggregate term's signed contribution; `0` if absent.
    fn term_signed(pos: &Position, name: &str) -> i32 {
        let r = evaluate_report(pos, BreakdownDetail::Aggregate);
        r.terms.iter().chain(r.side_terms.iter())
            .find(|t| t.name == name)
            .map(|t| t.signed).unwrap_or(0)
    }

    #[test]
    fn guard_isolation_penalises_outnumbered_guard() {
        // P1 guard at d4 (sq 27) with TWO P2 champions within radius 2 and no
        // friendly support → outnumber = 2. Penalty = per_step x 2 on P1.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 2, MailboxEntry::default().with_hp(2));
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2)); // adjacent enemy
        place(&mut p, 29, Player::P2, 1, MailboxEntry::default().with_hp(2)); // radius-2 enemy
        let (p1, p2) = term_pair(&p, "guard_isolation");
        assert_eq!(p1, EvalParams::DEFAULT.guard_iso_per_step * 2,
            "lone P1 guard outnumbered 2-0 → penalty magnitude 2xper_step");
        assert_eq!(p2, 0, "P2 champions are not outnumbered here");
        // signed negates the penalty into the P1-POV total.
        assert_eq!(term_signed(&p, "guard_isolation"), -(EvalParams::DEFAULT.guard_iso_per_step * 2));
    }

    #[test]
    fn guard_isolation_zero_when_supported() {
        // P1 guard at d4 (sq 27), one P2 champion adjacent (28), but TWO P1
        // friendlies also within radius 2 → enemies(1) - friendlies(2) < 0 →
        // outnumber clamps to 0 → no penalty on that guard.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 2, MailboxEntry::default().with_hp(2));
        place(&mut p, 26, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 25, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        // The guard at 27 sees enemies_near=1, friendlies_near=2 → 0. Term absent
        // (both-side magnitude zero) ⇔ zero contribution.
        assert_eq!(term_pair(&p, "guard_isolation").0, 0, "supported guard not penalised");
    }

    #[test]
    fn guard_isolation_ignores_champions_and_kings() {
        // A lone P1 CHAMPION surrounded by enemies must NOT be penalised by this
        // guard-only term (champion_threat/exposure handle champions). With no
        // guards on the board the term is skipped entirely - absent ⇔ zero.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        place(&mut p, 29, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(term_pair(&p, "guard_isolation").0, 0, "champion is not a guard → no isolation penalty");
    }

    // ============================================================
    // ChampionThreat term (E12, ns-43 Term 3b) - offensive + defensive
    // targeting, value-weighted, strike-safety-aware.
    // ============================================================

    fn champ_threat_of(pos: &Position, is_p1: bool) -> i32 {
        let (p1, p2) = term_pair(pos, "champion_threat");
        if is_p1 { p1 } else { p2 }
    }

    #[test]
    fn champion_threat_rewards_offensive_target() {
        // P1 champion at d4 (27) with Lance (Strike, range 1) adjacent to a P2
        // champion at e4 (28). Offensive threat should be > 0.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 1, MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert!(champ_threat_of(&p, true) > 0, "champion threatening an enemy scores > 0");
    }

    #[test]
    fn champion_threat_rewards_defensive_target() {
        // P1 champion at d4 (27) with Heal (Ally, range 1) adjacent to a wounded
        // P1 guard at e4 (28). Defensive threat should be > 0.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 1, MailboxEntry::default().with_hp(2).with_skill1(Skill::Heal as u8));
        place(&mut p, 28, Player::P1, 2, MailboxEntry::default().with_hp(1)); // wounded ally
        assert!(champ_threat_of(&p, true) > 0, "champion able to heal a wounded ally scores > 0");
    }

    #[test]
    fn champion_threat_zero_with_no_targets() {
        // Lone P1 champion with Lance, no enemies/allies in range → 0.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 1, MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        assert_eq!(champ_threat_of(&p, true), 0);
    }

    #[test]
    fn champion_threat_king_worth_more_than_guard() {
        // Same champion+Hook (range 2), targeting a lone enemy KING vs a lone
        // enemy GUARD at the same square → king case scores strictly higher.
        let mut king_pos = Position::empty();
        place(&mut king_pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2).with_skill1(Skill::Hook as u8));
        place(&mut king_pos, 29, Player::P2, 0, MailboxEntry::default().with_hp(2)); // enemy king in range 2

        let mut guard_pos = Position::empty();
        place(&mut guard_pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2).with_skill1(Skill::Hook as u8));
        place(&mut guard_pos, 29, Player::P2, 2, MailboxEntry::default().with_hp(2)); // enemy guard in range 2

        assert!(champ_threat_of(&king_pos, true) > champ_threat_of(&guard_pos, true),
            "threatening the enemy king is worth more than a guard");
    }

    #[test]
    fn champion_threat_ignores_non_champions() {
        // A GUARD is not a champion → champion_threat contributes nothing for it.
        let mut p = Position::empty();
        place(&mut p, 27, Player::P1, 2, MailboxEntry::default().with_hp(2)); // guard, no skills anyway
        place(&mut p, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(champ_threat_of(&p, true), 0);
    }

    // ============================================================
    // EndgameClosing term (E13, ns-43 Term 4) - asymmetric, stage-gated.
    // ============================================================

    fn closing_of(pos: &Position) -> (i32, i32) {
        term_pair(pos, "endgame_closing")
    }

    #[test]
    fn endgame_closing_dormant_in_opening() {
        // Full-material opening → stage Opening → term inactive → absent.
        let pos = Position::setup_stack_m();
        assert_eq!(closing_of(&pos), (0, 0), "closing term must not fire in the opening");
    }

    #[test]
    fn endgame_closing_asymmetric_leader_and_trailer() {
        // End stage: P1 leads (king + champion) vs P2 (lone king). P1 champion
        // near the P2 king. Leader (P1) should get a closing score; trailer (P2)
        // a stalling score. Both positive magnitudes on their own side.
        let mut p = Position::empty();
        place(&mut p, 4,  Player::P1, 0, MailboxEntry::default().with_hp(2)); // P1 king
        place(&mut p, 45, Player::P1, 1, MailboxEntry::default().with_hp(2)   // P1 champion near P2 king
            .with_skill1(Skill::Lance as u8));
        place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2)); // P2 lone king
        p.round_number = 20;
        // Confirm we are in the End stage and P1 leads.
        let ctx = EvalContext::new(&p, &EvalParams::DEFAULT);
        assert_eq!(ctx.stage, GameStage::End);
        assert!(ctx.advantage > 0, "P1 is up a champion");
        let (p1, p2) = closing_of(&p);
        assert!(p1 > 0, "leader P1 gets a closing score");
        assert!(p2 > 0, "trailer P2 gets a stalling score");
    }

    #[test]
    fn endgame_closing_neutral_when_even() {
        // End stage but dead even (mirrored lone kings) → advantage 0 < lead_min
        // → neutral.
        let mut p = Position::empty();
        place(&mut p, 4,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2));
        p.round_number = 20;
        assert_eq!(closing_of(&p), (0, 0), "even endgame → no forced aggression");
    }

    // ============================================================
    // Golden-equality suite (ns-43 refactor safety net).    //
    // A fixed set of labelled positions exercising every eval term. The
    // `golden_eval_unchanged` test asserts `evaluate()` and the full
    // per-field `EvalBreakdown` match hand-captured expected values. The
    // ns-43 term-registry refactor is behaviour-preserving iff this test
    // still passes byte-for-byte afterwards. NO Date/rand - fixed positions
    // only, so the goldens are deterministic and reproducible.
    // ============================================================

    /// Build the labelled golden suite. Each entry: (label, position).
    /// Chosen to cover: terminal, material/hp/armor, skills+money (E4/E3),
    /// mobility (E7: guard/king/champion variants), exposure (E2 + king),
    /// coverage (E6), tempo (E8), offensive-range (E9), and the canonical
    /// Stack M opening (all terms mirrored).
    fn golden_suite() -> Vec<(&'static str, Position)> {
        let mut suite: Vec<(&'static str, Position)> = Vec::new();

        // 1. Empty board.
        suite.push(("empty", Position::empty()));

        // 2. Terminal P1 wins (short-circuit).
        {
            let mut p = Position::empty();
            place(&mut p, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
            p.game_result = Some(GameResult::P1Wins);
            suite.push(("terminal_p1", p));
        }

        // 3. Canonical opening layout (the `setup_stack_m` constructor is the
        //    standard start position; its name is a historical artefact - the
        //    layout is unchanged under Stack N).
        suite.push(("opening", Position::setup_stack_m()));

        // 4. HP + armor + skill differential with money (E3/E4).
        {
            let mut p = Position::empty();
            place(&mut p, 28, Player::P1, 1,
                MailboxEntry::default().with_hp(2).with_armor(2)
                    .with_skill1(Skill::Lance as u8).with_skill2(Skill::Shove as u8));
            place(&mut p, 35, Player::P2, 1,
                MailboxEntry::default().with_hp(1).with_armor(0)
                    .with_skill1(Skill::Focus as u8));
            p.p1_money = 12;
            p.p2_money = 3;
            p.actions_remaining = 2;
            p.current_phase = Phase::Move;
            p.to_move = Player::P1;
            p.round_number = 6;
            suite.push(("champ_diff_skills_money", p));
        }

        // 5. Exposure + coverage: P1 champ flanked by a P2 champ, with a P1
        //    guard shielding it. Exercises E2 (exposure) and E6 (coverage).
        {
            let mut p = Position::empty();
            place(&mut p, 28, Player::P1, 1, MailboxEntry::default().with_hp(2)
                .with_skill1(Skill::Lance as u8));
            place(&mut p, 29, Player::P1, 2, MailboxEntry::default().with_hp(2)); // shielding guard
            place(&mut p, 27, Player::P2, 1, MailboxEntry::default().with_hp(2)
                .with_skill1(Skill::Lance as u8)); // attacker
            p.p1_money = 6;
            p.p2_money = 6;
            p.actions_remaining = 1;
            p.current_phase = Phase::Move;
            p.to_move = Player::P2;
            p.round_number = 8;
            suite.push(("exposure_coverage", p));
        }

        // 6. King exposure + king mobility (E2 king curve, E7 king escape).
        {
            let mut p = Position::empty();
            place(&mut p, 4, Player::P1, 0, MailboxEntry::default().with_hp(2)); // P1 king
            place(&mut p, 12, Player::P2, 1, MailboxEntry::default().with_hp(2)
                .with_skill1(Skill::Hook as u8)); // threatens king
            place(&mut p, 60, Player::P2, 0, MailboxEntry::default().with_hp(2)); // P2 king (well-formed)
            p.p1_money = 4;
            p.p2_money = 8;
            p.actions_remaining = 2;
            p.current_phase = Phase::Move;
            p.to_move = Player::P1;
            p.round_number = 10;
            suite.push(("king_exposure_mobility", p));
        }

        // 7. Guard mobility (E7 guard BFS-2) + offensive-range (E9).
        {
            let mut p = Position::empty();
            place(&mut p, 27, Player::P1, 2, MailboxEntry::default().with_hp(2)); // free guard, high BFS-2
            place(&mut p, 28, Player::P1, 1, MailboxEntry::default().with_hp(2)
                .with_skill1(Skill::Shove as u8).with_skill2(Skill::Focus as u8)); // reach 3 + focus
            place(&mut p, 36, Player::P2, 1, MailboxEntry::default().with_hp(2)
                .with_skill1(Skill::Lance as u8)); // reach 2
            p.p1_money = 10;
            p.p2_money = 10;
            p.actions_remaining = 2;
            p.current_phase = Phase::Move;
            p.to_move = Player::P1;
            p.round_number = 12;
            suite.push(("guard_mob_offensive_range", p));
        }

        suite
    }

    /// Golden totals: `evaluate()` must return exactly these P1-POV scalars on
    /// the fixed suite. The per-term breakdown is no longer pinned field-by-field
    /// (the fixed struct is gone); we pin the scalar total and the per-piece
    /// report's reconstruction of it. If a deliberate eval change lands, re-capture
    /// these totals in the SAME commit.
    #[test]
    fn golden_eval_unchanged() {
        // Scalar P1-POV totals for the labelled suite. The per-term breakdown is
        // no longer pinned field-by-field (the fixed EvalBreakdown struct is
        // gone); instead we pin the scalar total AND assert the per-piece report
        // reconstructs it. If a deliberate eval change lands, re-capture these
        // totals in the SAME commit.
        // Scalar P1-POV totals (recaptured ns-53 when the SEE terms — hanging_piece
        // + king_tempo — went live in the default set; the earlier values were the
        // pre-SEE baseline).
        let expected: &[(&str, i32)] = &[
            ("empty", 0),
            ("terminal_p1", 1_000_000),
            ("opening", 30),
            ("champ_diff_skills_money", 2900),
            ("exposure_coverage", 1558),
            ("king_exposure_mobility", -4356),
            ("guard_mob_offensive_range", 3168),
        ];

        let suite = golden_suite();
        assert_eq!(suite.len(), expected.len(), "suite/expected length mismatch");
        for ((label, pos), (elabel, etotal)) in suite.iter().zip(expected.iter()) {
            assert_eq!(label, elabel, "suite ordering mismatch");
            assert_eq!(evaluate(pos), *etotal, "evaluate() total mismatch for '{label}'");
            let r = evaluate_report(pos, BreakdownDetail::Aggregate);
            assert_eq!(r.total, *etotal, "report total mismatch for '{label}'");
            // Per-piece report must reconstruct the total (skip terminals - no pieces).
            if !r.terminal {
                assert_report_reconstructs_total(pos);
            }
        }
    }

    /// Determinism guard (HARD requirement): the same position must produce the
    /// exact same score + report on every call - no rand/Date, no order-dependent
    /// float accumulation. Cheap insurance against a future term introducing
    /// nondeterminism.
    #[test]
    fn eval_is_deterministic() {
        let suite = golden_suite();
        for (label, pos) in suite.iter() {
            let first_total = evaluate(pos);
            let first_report = evaluate_report(pos, BreakdownDetail::PerPiece);
            for _ in 0..64 {
                assert_eq!(evaluate(pos), first_total,
                    "evaluate() nondeterministic for '{label}'");
                assert_eq!(evaluate_report(pos, BreakdownDetail::PerPiece), first_report,
                    "evaluate_report() nondeterministic for '{label}'");
            }
        }
    }
}
