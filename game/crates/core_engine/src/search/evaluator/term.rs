//! The [`EvalTerm`] trait (ns-43): one self-describing, independently
//! parameterised unit of the evaluation.
//!
//! ## Two kinds of term
//!
//! Most eval work is a single pass over occupied squares. Running that pass
//! once per term (7+ terms) would be wasteful, so terms come in two shapes:
//!
//! - **Per-piece terms** (`material`, `hp`, `armor`, `skills`, `mobility`,
//!   `exposure`, `coverage`) contribute per occupied square. They implement
//!   [`EvalTerm::score_piece`]; the registry drives ONE shared board pass and
//!   fans each square out to every active per-piece term.
//! - **Side-level terms** (`money`, `tempo`, `offensive_range`) depend only on
//!   whole-side aggregates. They implement [`EvalTerm::score_side`] and run
//!   once.
//!
//! A term implements exactly one of the two; the default impls return `(0, 0)`.
//!
//! ## Signing
//!
//! Each term owns how its per-side magnitudes fold into the scalar total via
//! [`EvalTerm::signed_total`]. Default `p1 - p2`; `exposure` overrides to
//! `-(p1 - p2)` (it is a penalty); `offensive_range` overrides to
//! `(p1 - p2) x weight`. Keeping the sign/weight inside the term is what lets
//! the registry stay term-agnostic.

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

/// One evaluation term.
pub trait EvalTerm: Send + Sync {
    /// Stable machine name - the dynamic-breakdown key and the legacy-projection key.
    fn name(&self) -> &'static str;

    /// Cheap gating predicate. Phase gating hooks here in a later pass; today
    /// every ported term is always active (Draft guards are folded into the
    /// side-level terms' own math, matching the pre-ns-43 behaviour).
    #[allow(unused_variables)]
    fn is_active(&self, ctx: &EvalContext) -> bool { true }

    /// True if this is a per-piece term (contributes via `score_piece`).
    fn is_per_piece(&self) -> bool { false }

    /// Per-piece contribution as a positive magnitude for the piece's owner.
    /// Only called for per-piece terms. Returns the owner-relative magnitude
    /// (the registry routes it to p1 or p2 by `pc.is_p1`).
    #[allow(unused_variables)]
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 { 0 }

    /// Side-level contribution `(p1, p2)` as positive magnitudes. Only called
    /// for non-per-piece terms.
    #[allow(unused_variables)]
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) { (0, 0) }

    /// Fold this term's per-side magnitudes into the scalar total. Receives
    /// `params` so weight-scaled terms (offensive_range) stay tunable and
    /// self-describing. Default `p1 - p2`.
    #[allow(unused_variables)]
    fn signed_total(&self, p1: i32, p2: i32, params: &super::params::EvalParams) -> i32 { p1 - p2 }
}
