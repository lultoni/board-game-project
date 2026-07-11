//! Term registry + the driving evaluation loop (ns-43).
//!
//! `default_terms` returns the ported term set in a stable order. `evaluate_dyn`
//! runs the terminal short-circuit, builds one [`EvalContext`], drives a single
//! shared board pass for per-piece terms (fanning each occupied square out to
//! every active per-piece term), runs side-level terms once, and returns a
//! [`DynBreakdown`] — the source of truth from which the legacy fixed-field
//! `EvalBreakdown` is projected.

use crate::state::Position;
use crate::state::position::GameResult;
use crate::search::counters;
use super::MATE_SCORE;
use super::params::EvalParams;
use super::context::EvalContext;
use super::term::{EvalTerm, PieceContext};
use super::terms;
use super::breakdown::{DynBreakdown, TermEntry};

/// The ported term set, in a stable order. Per-piece terms first (they share
/// the board pass), then side-level terms. Order fixes the `DynBreakdown`
/// sequence but not the total (addition commutes).
pub fn default_terms(_params: &EvalParams) -> Vec<Box<dyn EvalTerm>> {
    vec![
        // Per-piece.
        Box::new(terms::Material),
        Box::new(terms::Hp),
        Box::new(terms::Armor),
        Box::new(terms::Skills),
        Box::new(terms::Mobility),
        Box::new(terms::Exposure),
        Box::new(terms::Coverage),
        Box::new(terms::GuardIsolation),
        // Side-level.
        Box::new(terms::Money),
        Box::new(terms::Tempo),
        Box::new(terms::OffensiveRange),
        Box::new(terms::WastedModifier),
    ]
}

/// Evaluate `pos` with the given `terms` and `params`, producing the dynamic
/// breakdown (only active terms, each with its signed contribution).
pub fn evaluate_dyn(pos: &Position, terms: &[Box<dyn EvalTerm>], params: &EvalParams) -> DynBreakdown {
    counters::bump_eval_calls();

    // Terminal — overrules everything. No terms run.
    match pos.game_result {
        Some(GameResult::P1Wins) => return DynBreakdown::terminal(MATE_SCORE),
        Some(GameResult::P2Wins) => return DynBreakdown::terminal(-MATE_SCORE),
        None => {}
    }

    let ctx = EvalContext::new(pos, params);

    // Partition active terms into per-piece vs side-level, preserving order.
    let mut active: Vec<&dyn EvalTerm> = Vec::with_capacity(terms.len());
    for t in terms {
        if t.is_active(&ctx) { active.push(t.as_ref()); }
    }
    let per_piece: Vec<&dyn EvalTerm> = active.iter().copied().filter(|t| t.is_per_piece()).collect();
    let side_level: Vec<&dyn EvalTerm> = active.iter().copied().filter(|t| !t.is_per_piece()).collect();

    // Accumulators keyed by position in `active`. We store (p1, p2) per active
    // term to preserve emission order in the DynBreakdown.
    let mut acc: Vec<(i32, i32)> = vec![(0, 0); active.len()];

    // Single shared board pass — fan each occupied square to every per-piece term.
    let mut bits = ctx.all_occ;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let is_p1 = ctx.p1_bb & mask != 0;
        let pc = PieceContext {
            sq,
            mask,
            is_p1,
            is_guard:    pos.guards.0    & mask != 0,
            is_king:     pos.kings.0     & mask != 0,
            is_champion: pos.champions.0 & mask != 0,
            mailbox:     pos.mailbox[sq as usize],
        };
        for (i, t) in active.iter().enumerate() {
            if !t.is_per_piece() { continue; }
            let mag = t.score_piece(&ctx, &pc);
            if is_p1 { acc[i].0 += mag; } else { acc[i].1 += mag; }
        }
    }
    let _ = &per_piece; // documents intent; the loop above already gates on is_per_piece.

    // Side-level terms — once each.
    for (i, t) in active.iter().enumerate() {
        if t.is_per_piece() { continue; }
        let (p1, p2) = t.score_side(&ctx);
        acc[i] = (p1, p2);
    }
    let _ = &side_level;

    // Assemble the dynamic breakdown + total.
    let mut entries = Vec::with_capacity(active.len());
    let mut total = 0i32;
    for (i, t) in active.iter().enumerate() {
        let (p1, p2) = acc[i];
        let signed = t.signed_total(p1, p2, params);
        total += signed;
        entries.push(TermEntry { name: t.name(), p1, p2, signed });
    }

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    DynBreakdown { terms: entries, total, terminal: false }
}
