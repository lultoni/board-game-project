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
        Box::new(terms::ChampionThreat),
        // Side-level.
        Box::new(terms::Money),
        Box::new(terms::Tempo),
        Box::new(terms::OffensiveRange),
        Box::new(terms::WastedModifier),
        Box::new(terms::EndgameClosing),
    ]
}

/// Phase 1a (Session 48) — the term set is static: all terms are zero-size
/// stateless structs and read their weights from `ctx.params`, never from
/// stored fields. `default_terms` used to box all 14 on **every** leaf eval
/// (14 heap allocations + vtable setup per call, pure search-path overhead).
///
/// Build the boxed set exactly once and borrow it forever. The search-path
/// entry points (`evaluate`, `evaluate_breakdown`, `evaluate_dyn` in the parent
/// module) route through here; `evaluate_dyn(pos, terms, params)` still accepts
/// an arbitrary term slice for the tuner / tests.
pub fn default_terms_static() -> &'static [Box<dyn EvalTerm>] {
    use std::sync::OnceLock;
    static TERMS: OnceLock<Vec<Box<dyn EvalTerm>>> = OnceLock::new();
    TERMS.get_or_init(|| default_terms(&EvalParams::DEFAULT))
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

/// Phase 1b (Session 48) — scalar search-leaf eval. Returns just the `i32`
/// total, with **no `DynBreakdown`, no `Vec` of active-term borrows, no
/// per-term `acc`/`entries` allocation, and no `to_legacy` projection.** All
/// of that machinery exists to feed the frontend eval panel / telemetry /
/// nn_trainer — none of which run in the inner search loop, which wants a
/// single scalar.
///
/// This is a **monomorphic** rewrite of `evaluate_dyn(...).total`: it calls
/// each ported term's concrete `score_piece` / `score_side` / `signed_total`
/// directly (static dispatch, no `dyn` vtable) and folds into one running
/// `i32`. The term set, order, `is_active` gates, and per-term signs are the
/// SAME as `default_terms` + each term's `signed_total`, so the result is
/// byte-identical to `evaluate_breakdown(pos).total` — pinned by
/// `golden_eval_unchanged` (which routes `evaluate()` through here).
///
/// Terms marked `is_per_piece` (material, hp, armor, skills, mobility,
/// exposure, coverage, guard_isolation, champion_threat) accumulate a per-side
/// magnitude in the shared board pass; side-level terms (money, tempo,
/// offensive_range, wasted_modifier, endgame_closing) run once. The three
/// gated per-piece / side terms (guard_isolation → guards present,
/// champion_threat → champions present, wasted_modifier → Skill phase,
/// endgame_closing → End stage) reproduce `is_active` exactly.
pub fn evaluate_scalar(pos: &Position, params: &EvalParams) -> i32 {
    counters::bump_eval_calls();

    // Terminal — overrules everything. No terms run.
    match pos.game_result {
        Some(GameResult::P1Wins) => return MATE_SCORE,
        Some(GameResult::P2Wins) => return -MATE_SCORE,
        None => {}
    }

    let ctx = EvalContext::new(pos, params);

    // is_active gates (mirror the term impls).
    let guards_present    = pos.guards.0 != 0;               // GuardIsolation
    let champions_present  = pos.champions.0 != 0;           // ChampionThreat
    use crate::state::position::Phase;
    let skill_phase = ctx.phase == Phase::Skill;             // WastedModifier
    let end_stage   = matches!(ctx.stage, super::context::GameStage::End); // EndgameClosing

    // Per-piece accumulators: positive magnitudes, per side.
    let (mut mat_p1, mut mat_p2) = (0i32, 0i32);
    let (mut hp_p1,  mut hp_p2)  = (0i32, 0i32);
    let (mut arm_p1, mut arm_p2) = (0i32, 0i32);
    let (mut skl_p1, mut skl_p2) = (0i32, 0i32);
    let (mut mob_p1, mut mob_p2) = (0i32, 0i32);
    let (mut exp_p1, mut exp_p2) = (0i32, 0i32);
    let (mut cov_p1, mut cov_p2) = (0i32, 0i32);
    let (mut iso_p1, mut iso_p2) = (0i32, 0i32);
    let (mut cht_p1, mut cht_p2) = (0i32, 0i32);

    let (t_mat, t_hp, t_arm, t_skl, t_mob, t_exp, t_cov, t_iso, t_cht) = (
        terms::Material, terms::Hp, terms::Armor, terms::Skills, terms::Mobility,
        terms::Exposure, terms::Coverage, terms::GuardIsolation, terms::ChampionThreat,
    );

    // Single shared board pass — static dispatch, no vtable.
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
        let m   = t_mat.score_piece(&ctx, &pc);
        let h   = t_hp.score_piece(&ctx, &pc);
        let a   = t_arm.score_piece(&ctx, &pc);
        let s   = t_skl.score_piece(&ctx, &pc);
        let mo  = t_mob.score_piece(&ctx, &pc);
        let e   = t_exp.score_piece(&ctx, &pc);
        let c   = t_cov.score_piece(&ctx, &pc);
        let iso = if guards_present    { t_iso.score_piece(&ctx, &pc) } else { 0 };
        let cht = if champions_present { t_cht.score_piece(&ctx, &pc) } else { 0 };
        if is_p1 {
            mat_p1 += m; hp_p1 += h; arm_p1 += a; skl_p1 += s; mob_p1 += mo;
            exp_p1 += e; cov_p1 += c; iso_p1 += iso; cht_p1 += cht;
        } else {
            mat_p2 += m; hp_p2 += h; arm_p2 += a; skl_p2 += s; mob_p2 += mo;
            exp_p2 += e; cov_p2 += c; iso_p2 += iso; cht_p2 += cht;
        }
    }

    // Side-level terms — once each.
    let (money_p1, money_p2)   = terms::Money.score_side(&ctx);
    let (tempo_p1, tempo_p2)   = terms::Tempo.score_side(&ctx);
    let (off_p1, off_p2)       = terms::OffensiveRange.score_side(&ctx);
    let (wasted_p1, wasted_p2) = if skill_phase { terms::WastedModifier.score_side(&ctx) } else { (0, 0) };
    let (close_p1, close_p2)   = if end_stage   { terms::EndgameClosing.score_side(&ctx) } else { (0, 0) };

    // Fold with each term's own sign / weight (mirrors `signed_total`).
    let mut total = 0i32;
    total += mat_p1 - mat_p2;
    total += hp_p1  - hp_p2;
    total += arm_p1 - arm_p2;
    total += skl_p1 - skl_p2;
    total += mob_p1 - mob_p2;
    total -= exp_p1 - exp_p2;              // exposure: penalty
    total += cov_p1 - cov_p2;
    total -= iso_p1 - iso_p2;              // guard_isolation: penalty
    total += cht_p1 - cht_p2;
    total += money_p1 - money_p2;
    total += tempo_p1 - tempo_p2;
    total += (off_p1 - off_p2) * params.offensive_range_weight;
    total -= wasted_p1 - wasted_p2;        // wasted_modifier: penalty
    total += close_p1 - close_p2;

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    total
}
