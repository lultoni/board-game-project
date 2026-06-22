//! Layer 3 — AI & Search.
//!
//! Alpha-beta + iterative deepening + transposition table. Heuristic eval
//! lives in `evaluator`. Search is parameterised by both time budget and
//! max depth (per ADR-005).

pub mod evaluator;
pub mod transposition;
pub mod alpha_beta;
