//! Heuristic evaluation for terminal / time-out search nodes.
//!
//! Score convention: positive = P1 advantage, negative = P2 advantage.
//! Win/loss are represented as ±(INFINITY - depth_to_mate) so shorter wins
//! score higher and the search prefers fast mates.

use crate::state::Position;

pub const MATE_SCORE: i32 = 1_000_000;

pub fn evaluate(_pos: &Position) -> i32 {
    // TODO: material → resources → positional hooks.
    0
}
