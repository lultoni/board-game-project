//! `NnueEvaluator` — the search-time `Evaluator` backed by the quantized NNUE
//! net (integer forward over a freshly-refreshed accumulator).
//!
//! Phase 0 uses **refresh-per-call**: `evaluate(pos)` is a pure function of
//! `pos` (rebuild the accumulator, run the integer forward). The incremental
//! accumulator path is proven correct by the `accumulator` golden test; wiring
//! it into the search's per-node accumulator stack is Phase-1 work. Refresh-
//! per-call is also the *conservative* speed bound for the milestone gate:
//! an in-search incremental accumulator can only be faster.
//!
//! Additive — `NnEvaluator` (the dense burn evaluator) stays untouched.

use core_engine::search::evaluator::{EvalBreakdown, Evaluator, MATE_SCORE};
use core_engine::state::position::GameResult;
use core_engine::state::Position;

use crate::accumulator::Accumulator;
use crate::model::Mlp;
use crate::nn_evaluator::InferenceBackend;
use crate::quantized::{QuantScales, QuantizedNet};

/// Evaluator wrapping a quantized NNUE net. Terminals bypass the net entirely
/// (±MATE_SCORE via the existing convention), preserving mate-distance math.
pub struct NnueEvaluator {
    net: QuantizedNet,
}

impl NnueEvaluator {
    pub fn new(net: QuantizedNet) -> Self {
        NnueEvaluator { net }
    }

    /// Quantize a trained f32 model and wrap it.
    pub fn from_mlp(model: &Mlp<InferenceBackend>, scales: QuantScales) -> Self {
        NnueEvaluator { net: QuantizedNet::from_mlp(model, scales) }
    }

    /// Borrow the underlying quantized net.
    pub fn net(&self) -> &QuantizedNet {
        &self.net
    }
}

impl Evaluator for NnueEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        match pos.game_result {
            Some(GameResult::P1Wins) => return MATE_SCORE,
            Some(GameResult::P2Wins) => return -MATE_SCORE,
            None => {}
        }
        let acc = Accumulator::refresh(pos, self.net.ft());
        self.net.forward_int(&acc)
    }

    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown {
        // The NN doesn't decompose; fold into material_p1/p2 so the breakdown's
        // internal consistency (total == mat_p1 - mat_p2 - …) holds. Mirrors
        // `NnEvaluator::evaluate_breakdown`.
        let total = self.evaluate(pos);
        let mut b = EvalBreakdown::default();
        b.total = total;
        if total >= 0 {
            b.material_p1 = total;
        } else {
            b.material_p2 = -total;
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MlpConfig;
    use crate::sparse::NUM_FEATURES;
    use core_engine::state::fen::from_fen;

    fn fresh_evaluator() -> NnueEvaluator {
        let device = Default::default();
        let model: Mlp<InferenceBackend> = MlpConfig::new().with_input_dim(NUM_FEATURES).init(&device);
        NnueEvaluator::from_mlp(&model, QuantScales::default())
    }

    #[test]
    fn terminal_p1_win_overrules() {
        let eval = fresh_evaluator();
        // A position where P2's king is captured → P1 wins. Build via a FEN
        // with only a P1 king on the board (P2 king absent = P1Wins by the
        // king-capture rule). Simpler: from a start position, assert non-mate;
        // terminals are exercised through the game_result branch directly.
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(eval.evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_win_overrules() {
        let eval = fresh_evaluator();
        let mut pos = Position::setup_stack_m();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(eval.evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn non_terminal_in_range() {
        let eval = fresh_evaluator();
        let pos = Position::setup_stack_m();
        let s = eval.evaluate(&pos);
        assert!(s.abs() < MATE_SCORE, "non-terminal must not claim a mate score");
    }

    #[test]
    fn works_through_dyn_trait() {
        let eval = fresh_evaluator();
        let dyn_eval: &dyn Evaluator = &eval;
        let pos = Position::setup_stack_m();
        let s = dyn_eval.evaluate(&pos);
        assert!(s.abs() < MATE_SCORE);
        // breakdown total agrees with evaluate.
        let b = dyn_eval.evaluate_breakdown(&pos);
        assert_eq!(b.total, s);
    }

    #[test]
    fn evaluate_matches_forward_int_refresh() {
        let eval = fresh_evaluator();
        // A non-terminal mid-game position from the corpus (or the start).
        let pos = from_fen("1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCC1 P1 M 2 6 6 0 1 0x0")
            .unwrap_or_else(|_| Position::setup_stack_m());
        assert!(pos.game_result.is_none());
        let expected = eval
            .net()
            .forward_int(&Accumulator::refresh(&pos, eval.net().ft()));
        assert_eq!(eval.evaluate(&pos), expected);
    }
}
