//! Custom-evaluator stub (ns-55) — a scaffold to build your own evaluator on.
//!
//! This is a fully working, registered [`Evaluator`] you can select from the
//! setup / settings dropdowns (id `"custom-stub"`) and immediately see in
//! action. Out of the box it just delegates to the default heuristic, so the
//! app behaves normally until you start editing — then every change you make
//! here is live the moment you rebuild, with no other file to touch.
//!
//! ## How to make it your own
//!
//! 1. Replace the body of [`CustomEvaluator::score`] with your own math. It
//!    returns a single P1-POV `i32` (positive = P1 ahead), the same convention
//!    the whole engine uses. `±MATE_SCORE` are the terminal sentinels; the
//!    search treats anything past `MATE_SCORE - MAX_PLY` as a mate.
//! 2. If you want the eval panel / square hover card to show YOUR terms, build
//!    the [`EvalReport`] in [`CustomEvaluator::evaluate_report`] yourself: push
//!    a [`TermEntry`] per component (aggregate) and, when `detail == PerPiece`,
//!    a [`PieceTermBreakdown`] per occupied square. The panel is fully dynamic —
//!    whatever names you emit render automatically. Until you do, the report is
//!    a single synthetic `"custom"` term carrying the total.
//! 3. You are NOT limited to the shared term registry. This is your own struct;
//!    put whatever logic you like here (call `see`, walk bitboards, hold tuned
//!    weights, load a table — anything). To ship several variants, add more
//!    structs + more `builtin::BUILTINS` entries.
//!
//! The evaluator is registered in [`super::builtin::BUILTINS`]; that's the only
//! wiring — the dropdowns, the AI seats, and the UI-eval pick all resolve
//! through it.

use crate::state::Position;
use super::{
    BreakdownDetail, EvalReport, Evaluator, HeuristicEvaluator, MATE_SCORE, TermEntry,
    PieceTermBreakdown,
};

/// Your evaluator. Zero-size for now; add fields (tuned weights, cached tables,
/// a loaded model handle, …) as you flesh it out. Keep it `Send` — the search
/// owns it and moves it between thread-pool tasks.
#[derive(Clone, Debug, Default)]
pub struct CustomEvaluator;

impl CustomEvaluator {
    /// The one function to edit. Return a P1-POV score for `pos`.
    ///
    /// STARTER IMPLEMENTATION: delegate to the shipped heuristic so the app is
    /// playable while you experiment. Replace this with your own scoring — even
    /// something as crude as "material only" is a valid starting point (the
    /// designer's own eval notes recommend starting stupid and proving each term
    /// against a material baseline).
    #[inline]
    fn score(&self, pos: &Position) -> i32 {
        // <<< REPLACE ME >>>
        HeuristicEvaluator.evaluate(pos)
    }
}

impl Evaluator for CustomEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 {
        self.score(pos)
    }

    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport {
        // Terminal short-circuit (mirror the registry): no terms on a decided
        // position, just the mate total.
        match pos.game_result {
            Some(crate::state::position::GameResult::P1Wins) => return EvalReport::terminal(MATE_SCORE),
            Some(crate::state::position::GameResult::P2Wins) => return EvalReport::terminal(-MATE_SCORE),
            None => {}
        }

        let total = self.score(pos);

        // STARTER BREAKDOWN: one synthetic aggregate term carrying the whole
        // score, no per-piece rows. This is what an evaluator with no term
        // structure (e.g. an NN) reports — see `EvalReport::single`. When you
        // add real components, build `terms` / `side_terms` / `pieces` here
        // instead so the eval panel shows your decomposition.
        //
        // Example of a richer report (delete the `single` line and uncomment):
        //
        //   let material = /* your material term, P1-POV */;
        //   let mobility = /* your mobility term, P1-POV */;
        //   let mut pieces = None;
        //   if detail == BreakdownDetail::PerPiece {
        //       let mut rows = Vec::new();
        //       // for each occupied square, push a PieceTermBreakdown { .. }
        //       pieces = Some(rows);
        //   }
        //   return EvalReport {
        //       terms: vec![
        //           TermEntry { name: "material".into(), p1: 0, p2: 0, signed: material },
        //           TermEntry { name: "mobility".into(), p1: 0, p2: 0, signed: mobility },
        //       ],
        //       side_terms: Vec::new(),
        //       pieces,
        //       total,
        //       terminal: false,
        //   };
        let _ = detail; // starter path ignores detail (no per-piece rows yet)
        let _unused_types: Option<(TermEntry, PieceTermBreakdown)> = None; // keep imports live for editing
        EvalReport::single("custom", total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_stub_scores_and_reports_consistently() {
        let pos = Position::setup_stack_m();
        let ev = CustomEvaluator;
        // Report total agrees with the scalar score.
        let r = ev.evaluate_report(&pos, BreakdownDetail::Aggregate);
        assert_eq!(r.total, ev.evaluate(&pos));
        assert!(!r.terminal);
    }

    #[test]
    fn custom_stub_terminal() {
        let mut pos = Position::empty();
        pos.game_result = Some(crate::state::position::GameResult::P1Wins);
        let r = CustomEvaluator.evaluate_report(&pos, BreakdownDetail::PerPiece);
        assert!(r.terminal);
        assert_eq!(r.total, MATE_SCORE);
    }
}
