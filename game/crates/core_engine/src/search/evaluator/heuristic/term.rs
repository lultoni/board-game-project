//! The [`EvalTerm`] trait (ns-43): one self-describing unit of the evaluation.
//!
//! ## Two kinds of term
//!
//! Most eval work is a single pass over occupied squares. Running that pass once
//! per term would be wasteful, so terms come in two shapes:
//!
//! - **Per-piece terms** (`material`, `hp`, `exposure`, …) contribute per
//!   occupied square via [`EvalTerm::score_piece`]. The driver (`score_piece_all`
//!   in the parent module) runs ONE shared board pass and fans each square out to
//!   every per-piece term.
//! - **Side-level terms** (`money`, `tempo`, …) depend only on whole-side
//!   aggregates and run once via [`EvalTerm::score_side`].
//!
//! A term implements exactly one of the two; the other defaults to zero.
//!
//! Each term returns a **positive magnitude**; the sign that folds it into the
//! P1-POV total (`+1` bonus, `-1` penalty) lives with the term set in
//! `super::PIECE_TERMS`, not on the term itself — so the driver and the fold stay
//! in one place rather than spread across every term impl.

use super::context::EvalContext;

/// Per-square inputs handed to a per-piece term. Precomputed once per square in
/// the shared pass so terms don't re-derive occupancy/kind bits.
#[derive(Clone, Copy)]
pub struct PieceContext {
    pub sq:    u8,
    pub mask:  u64,
    pub is_p1: bool,
    pub is_guard:    bool,
    pub is_king:     bool,
    pub is_champion: bool,
    pub mailbox: crate::state::MailboxEntry,
}

/// One evaluation term. Implement exactly one of `score_piece` / `score_side`.
pub trait EvalTerm: Send + Sync {
    /// Per-piece contribution as a positive magnitude for the piece's owner.
    /// The driver routes it to p1 or p2 by `pc.is_p1`.
    #[allow(unused_variables)]
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 { 0 }

    /// Side-level contribution `(p1, p2)` as positive magnitudes.
    #[allow(unused_variables)]
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) { (0, 0) }
}
