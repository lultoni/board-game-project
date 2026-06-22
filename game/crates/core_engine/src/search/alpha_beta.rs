//! Alpha-beta + iterative deepening driver.
//!
//! Public entry point: `find_best(pos, tt, time_limit_ms, max_depth)`.
//! The transposition table is passed in so callers can reuse it across
//! moves (warm cache).

use super::transposition::TranspositionTable;
use crate::game_logic::action::Action;
use crate::state::Position;

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchResult {
    pub best:  Option<Action>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
}

pub fn find_best(
    _pos: &mut Position,
    _tt: &mut TranspositionTable,
    _time_limit_ms: u64,
    _max_depth: u8,
) -> SearchResult {
    // TODO: iterative deepening with move ordering from previous depth.
    // At each depth, probe TT for the best_move and try it first.
    SearchResult::default()
}
