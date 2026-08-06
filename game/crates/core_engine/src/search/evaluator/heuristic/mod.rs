//! The default heuristic evaluator (ns-43) — the term loop + fold that turns a
//! `Position` into a P1-POV score.
//!
//! This module IS the `HeuristicEvaluator`'s private implementation. Its
//! children — [`params`] (weights), [`context`] (per-call shared state),
//! [`term`] (the `EvalTerm` trait), [`terms`] (the concrete terms) — are all
//! heuristic-specific: nothing outside this folder needs them. Other evaluators
//! (see `super::custom`) implement `Evaluator` directly and never route through
//! here.
//!
//! `evaluate_scalar` is the monomorphic search-leaf fast path; `evaluate_report`
//! runs the same term math but also assembles the [`EvalReport`] breakdown. The
//! per-piece term set is declared once in [`PIECE_TERMS`]; everything (the fold,
//! the report, the count) derives from it.

// Heuristic-private submodules (moved here from the evaluator top level: the
// term machinery is this evaluator's guts, not shared infrastructure).
pub mod params;
pub mod context;
pub mod term;
pub mod terms;

use crate::state::Position;
use crate::state::position::GameResult;
use crate::search::counters;
use super::{Evaluator, MATE_SCORE};
use params::EvalParams;
use context::EvalContext;
use term::{EvalTerm, PieceContext};
use super::report::{BreakdownDetail, EvalReport, PieceTermBreakdown, TermEntry};

/// The default evaluator — zero-config, `EvalParams::DEFAULT` weights. This is
/// the concrete `Evaluator` the search installs for every AI seat unless another
/// is picked. Re-exported at the seam as `evaluator::HeuristicEvaluator`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicEvaluator;

impl Evaluator for HeuristicEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 {
        evaluate_scalar(pos, &EvalParams::DEFAULT)
    }
    #[inline]
    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport {
        evaluate_report(pos, &EvalParams::DEFAULT, detail)
    }
}

/// The same term math with a custom [`EvalParams`] weight set — the "same terms,
/// different balance" shape. A `builtin` entry can hand one out with tuned
/// weights. For genuinely different *logic*, write a separate struct impl'ing
/// [`Evaluator`] (its own module under `evaluator/`) and register that instead.
#[derive(Clone, Debug)]
pub struct ParamHeuristicEvaluator {
    params: EvalParams,
}

impl ParamHeuristicEvaluator {
    pub fn new(params: EvalParams) -> Self { Self { params } }
}

impl Evaluator for ParamHeuristicEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 {
        evaluate_scalar(pos, &self.params)
    }
    #[inline]
    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport {
        evaluate_report(pos, &self.params, detail)
    }
}


/// Evaluate `pos` and produce the dynamic [`EvalReport`] — aggregate terms
/// always, per-piece decomposition when `detail == PerPiece`. Both the aggregate
/// and the per-piece rows come from the SAME term math the scalar path uses
/// (`accumulate_terms` / `score_piece_all` / `score_side_all`), so the report is
/// definitionally consistent with `evaluate_scalar(pos, params)`.
pub fn evaluate_report(pos: &Position, params: &EvalParams, detail: BreakdownDetail) -> EvalReport {
    counters::bump_eval_calls();

    // Terminal - overrules everything. No terms run.
    match pos.game_result {
        Some(GameResult::P1Wins) => return EvalReport::terminal(MATE_SCORE),
        Some(GameResult::P2Wins) => return EvalReport::terminal(-MATE_SCORE),
        None => {}
    }

    let ctx = EvalContext::new(pos, params);
    let guards_present    = pos.guards.0 != 0;
    let champions_present = pos.champions.0 != 0;

    // Aggregate per-side sums + (optionally) per-piece rows in one board pass.
    let mut sums = TermSums::default();
    let mut pieces: Option<Vec<PieceTermBreakdown>> = match detail {
        BreakdownDetail::PerPiece => Some(Vec::with_capacity(ctx.all_occ.count_ones() as usize)),
        BreakdownDetail::Aggregate => None,
    };

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
        let mags = score_piece_all(&ctx, &pc, guards_present, champions_present);
        for (i, &mag) in mags.iter().enumerate() {
            if is_p1 { sums.piece[i].0 += mag; } else { sums.piece[i].1 += mag; }
        }

        if let Some(rows) = pieces.as_mut() {
            // Per-piece term rows. Owner-signed via the term's `PIECE_TERMS` sign
            // so a penalty (exposure / guard_iso) reads negative for the owner.
            let mut terms_here = Vec::with_capacity(N_PIECE_TERMS);
            let mut piece_total = 0i32;
            for (def, &mag) in PIECE_TERMS.iter().zip(mags.iter()) {
                if mag == 0 { continue; }
                // owner-signed: the term's `PIECE_TERMS` sign gives the P1-POV
                // contribution of a P1 magnitude; negate it for a P2 piece.
                let p1_pov = def.sign * mag;
                let owner_signed = if is_p1 { p1_pov } else { -p1_pov };
                piece_total += owner_signed;
                terms_here.push(TermEntry {
                    name: def.name.to_string(),
                    label: def.label.to_string(),
                    p1: if is_p1 { mag } else { 0 },
                    p2: if is_p1 { 0 } else { mag },
                    signed: owner_signed,
                });
            }
            let piece_kind = if pc.is_king { 3 } else if pc.is_champion { 2 } else { 1 };
            rows.push(PieceTermBreakdown {
                sq, is_p1, piece_kind,
                hp: pc.mailbox.hp(), armor: pc.mailbox.armor(),
                skill1_id: pc.mailbox.skill1(), skill2_id: pc.mailbox.skill2(),
                terms: terms_here,
                piece_total,
            });
        }
    }

    sums.set_side(score_side_all(&ctx));

    let total = sums.fold_total(params);

    // Assemble the aggregate + side-level term lists (active terms only — a term
    // whose both-side magnitudes are zero is omitted, matching the old behaviour
    // of only emitting active terms).
    let mut agg = Vec::with_capacity(N_PIECE_TERMS);
    for (i, def) in PIECE_TERMS.iter().enumerate() {
        let (p1, p2) = sums.piece[i];
        if p1 == 0 && p2 == 0 { continue; }
        agg.push(TermEntry {
            name: def.name.to_string(),
            label: def.label.to_string(),
            p1, p2,
            signed: def.sign * (p1 - p2),
        });
    }

    let mut side = Vec::with_capacity(6);
    push_side(&mut side, "money",            "Money",      sums.money, sums.money.0 - sums.money.1);
    push_side(&mut side, "tempo",            "Tempo",      sums.tempo, sums.tempo.0 - sums.tempo.1);
    push_side(&mut side, "offensive_range",  "Off reach",  sums.off,
              (sums.off.0 - sums.off.1) * params.offensive_range_weight);
    push_side(&mut side, "wasted_modifier",  "Wasted",     sums.wasted, -(sums.wasted.0 - sums.wasted.1));
    push_side(&mut side, "endgame_closing",  "Endgame",    sums.close, sums.close.0 - sums.close.1);
    push_side(&mut side, "king_tempo",       "King tempo", sums.king_tempo, -(sums.king_tempo.0 - sums.king_tempo.1));

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    EvalReport { terms: agg, side_terms: side, pieces, total, terminal: false }
}

/// Push a side-level term entry, skipping it when both magnitudes are zero.
#[inline]
fn push_side(out: &mut Vec<TermEntry>, name: &'static str, label: &'static str, mags: (i32, i32), signed: i32) {
    if mags.0 == 0 && mags.1 == 0 { return; }
    out.push(TermEntry { name: name.to_string(), label: label.to_string(), p1: mags.0, p2: mags.1, signed });
}

/// Scalar search-leaf eval — the hot path. Returns just the `i32` total, with
/// no breakdown `Vec`s or per-term allocation (that machinery feeds the UI /
/// telemetry, none of which runs in the inner search loop).
///
/// Monomorphic: it calls each term's concrete `score_piece` / `score_side`
/// directly (static dispatch, no `dyn` vtable) via [`accumulate_terms`], then
/// folds with [`TermSums::fold_total`]. Term set, order, gates, and signs are
/// shared with [`evaluate_report`] through the same helpers, so the two agree by
/// construction — pinned by `golden_eval_unchanged`.
pub fn evaluate_scalar(pos: &Position, params: &EvalParams) -> i32 {
    counters::bump_eval_calls();

    // Terminal - overrules everything. No terms run.
    match pos.game_result {
        Some(GameResult::P1Wins) => return MATE_SCORE,
        Some(GameResult::P2Wins) => return -MATE_SCORE,
        None => {}
    }

    let ctx = EvalContext::new(pos, params);
    let sums = accumulate_terms(pos, &ctx);
    let total = sums.fold_total(params);

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    total
}

/// A per-piece term's metadata: its stable machine name (the breakdown wire key)
/// its human-readable display label, and the sign applied to `(p1 - p2)` when
/// folding into the P1-POV total (`+1` for a bonus, `-1` for a penalty).
///
/// This is the SINGLE source of truth for the per-piece term set: [`N_PIECE_TERMS`],
/// the `pt::*` index aliases, the fold, and the report all derive from it, so a
/// term is described in exactly one place. The scorer for each term is dispatched
/// concretely (not through this table) in [`score_piece_all`] to keep the hot path
/// monomorphic — that function's order MUST match this table's order, which the
/// `pt::*` aliases below make self-checking.
pub(crate) struct PieceTermDef {
    pub name:  &'static str,
    pub label: &'static str,
    pub sign:  i32,
}

/// The per-piece term set, in fold/report order. ADD A TERM HERE (and give it a
/// scorer line in `score_piece_all`, at the matching index). Penalties get `sign: -1`.
pub(crate) const PIECE_TERMS: &[PieceTermDef] = &[
    PieceTermDef { name: "material",        label: "Material",   sign:  1 },
    PieceTermDef { name: "hp",              label: "HP",         sign:  1 },
    PieceTermDef { name: "armor",           label: "Armor",      sign:  1 },
    PieceTermDef { name: "skills",          label: "Skills",     sign:  1 },
    PieceTermDef { name: "mobility",        label: "Reach",      sign:  1 },
    PieceTermDef { name: "exposure",        label: "Exposure",   sign: -1 }, // penalty
    PieceTermDef { name: "coverage",        label: "Coverage",   sign:  1 },
    PieceTermDef { name: "guard_isolation", label: "Guard iso",  sign: -1 }, // penalty
    PieceTermDef { name: "champion_threat", label: "Threat",     sign:  1 },
    PieceTermDef { name: "hanging_piece",   label: "Hanging",    sign: -1 }, // penalty
];

/// Number of per-piece terms — derived from [`PIECE_TERMS`], never hand-counted.
pub(crate) const N_PIECE_TERMS: usize = PIECE_TERMS.len();

/// Readable index aliases into the per-piece term arrays, in [`PIECE_TERMS`] order.
/// `score_piece_all` uses these so its dispatch order is verifiably the table order.
pub(crate) mod pt {
    pub const MATERIAL: usize = 0;
    pub const HP: usize = 1;
    pub const ARMOR: usize = 2;
    pub const SKILLS: usize = 3;
    pub const MOBILITY: usize = 4;
    pub const EXPOSURE: usize = 5;
    pub const COVERAGE: usize = 6;
    pub const GUARD_ISO: usize = 7;
    pub const CHAMPION_THREAT: usize = 8;
    pub const HANGING: usize = 9;
}

/// The per-side term sums that fold into the scalar total. Per-piece terms hold
/// `(p1_magnitude, p2_magnitude)`; side-level terms hold their `(p1, p2)`
/// directly. `fold_total` applies each term's sign/weight - the SINGLE source
/// of truth for the fold, shared by `evaluate_scalar` and the incremental path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TermSums {
    /// Per-piece term (p1, p2) magnitudes, indexed by `pt::*`.
    pub piece: [(i32, i32); N_PIECE_TERMS],
    pub money:  (i32, i32),
    pub tempo:  (i32, i32),
    pub off:    (i32, i32),
    pub wasted: (i32, i32),
    pub close:  (i32, i32),
    pub king_tempo: (i32, i32),
}

impl TermSums {
    /// Fold with each term's own sign / weight. Per-piece terms sum via their
    /// `PIECE_TERMS[i].sign`; side-level terms are folded explicitly below.
    /// Golden-gated (`golden_eval_unchanged`).
    #[inline]
    pub(crate) fn fold_total(&self, params: &EvalParams) -> i32 {
        let mut total = 0i32;
        for (i, def) in PIECE_TERMS.iter().enumerate() {
            total += def.sign * (self.piece[i].0 - self.piece[i].1);
        }
        total += self.money.0 - self.money.1;
        total += self.tempo.0 - self.tempo.1;
        total += (self.off.0 - self.off.1) * params.offensive_range_weight;
        total -= self.wasted.0 - self.wasted.1;           // penalty
        total += self.close.0 - self.close.1;
        total -= self.king_tempo.0 - self.king_tempo.1;   // penalty
        total
    }

    /// Copy the side-level magnitudes from a [`SideSums`] into this `TermSums`.
    #[inline]
    fn set_side(&mut self, s: SideSums) {
        self.money = s.money;
        self.tempo = s.tempo;
        self.off = s.off;
        self.wasted = s.wasted;
        self.close = s.close;
        self.king_tempo = s.king_tempo;
    }
}

/// Score one occupied square across all per-piece terms, returning each term's
/// owner-relative magnitude, indexed by `pt::*`. `guards_present` /
/// `champions_present` gate GuardIsolation / ChampionThreat. Shared by the full
/// accumulation pass and the per-piece report — one source of truth for
/// per-square term math.
///
/// **Order invariant:** the assignments below MUST be in [`PIECE_TERMS`] order
/// (the `pt::*` aliases make that self-evident and are checked by a unit test).
/// Adding a term = one `PIECE_TERMS` entry + one scorer line here at that index.
#[inline]
pub(crate) fn score_piece_all(
    ctx: &EvalContext, pc: &PieceContext,
    guards_present: bool, champions_present: bool,
) -> [i32; N_PIECE_TERMS] {
    let mut out = [0i32; N_PIECE_TERMS];
    out[pt::MATERIAL]        = terms::Material.score_piece(ctx, pc);
    out[pt::HP]              = terms::Hp.score_piece(ctx, pc);
    out[pt::ARMOR]           = terms::Armor.score_piece(ctx, pc);
    out[pt::SKILLS]          = terms::Skills.score_piece(ctx, pc);
    out[pt::MOBILITY]        = terms::Mobility.score_piece(ctx, pc);
    out[pt::EXPOSURE]        = terms::Exposure.score_piece(ctx, pc);
    out[pt::COVERAGE]        = terms::Coverage.score_piece(ctx, pc);
    out[pt::GUARD_ISO]       = if guards_present { terms::GuardIsolation.score_piece(ctx, pc) } else { 0 };
    out[pt::CHAMPION_THREAT] = if champions_present { terms::ChampionThreat.score_piece(ctx, pc) } else { 0 };
    // Hanging: self-gated (returns 0 for unattacked pieces via the cheap
    // physical-attacker check before any SEE rollout).
    out[pt::HANGING]         = terms::HangingPiece.score_piece(ctx, pc);
    out
}

/// The six side-level term magnitudes `(p1, p2)`, produced together by
/// [`score_side_all`]. Named (rather than a 6-tuple) so the fields are
/// self-documenting at every call site.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SideSums {
    pub money:      (i32, i32),
    pub tempo:      (i32, i32),
    pub off:        (i32, i32),
    pub wasted:     (i32, i32),
    pub close:      (i32, i32),
    pub king_tempo: (i32, i32),
}

/// Compute the side-level term values, applying the phase / stage gates
/// (wasted → Skill phase, endgame_closing → End stage).
#[inline]
pub(crate) fn score_side_all(ctx: &EvalContext) -> SideSums {
    use crate::state::position::Phase;
    let skill_phase = ctx.phase == Phase::Skill;
    let end_stage = matches!(ctx.stage, context::GameStage::End);
    SideSums {
        money:      terms::Money.score_side(ctx),
        tempo:      terms::Tempo.score_side(ctx),
        off:        terms::OffensiveRange.score_side(ctx),
        wasted:     if skill_phase { terms::WastedModifier.score_side(ctx) } else { (0, 0) },
        close:      if end_stage   { terms::EndgameClosing.score_side(ctx) } else { (0, 0) },
        king_tempo: terms::KingTempo.score_side(ctx),
    }
}

/// Full accumulation of every term for `pos` into a [`TermSums`]. This is the
/// full-eval body shared by `evaluate_scalar` and the incremental evaluator's
/// full-rebuild path. Terminal positions must be handled by the caller.
#[inline]
pub(crate) fn accumulate_terms(pos: &Position, ctx: &EvalContext) -> TermSums {
    let guards_present    = pos.guards.0 != 0;
    let champions_present = pos.champions.0 != 0;

    let mut sums = TermSums::default();

    // Single shared board pass.
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
        let mags = score_piece_all(ctx, &pc, guards_present, champions_present);
        for (i, &mag) in mags.iter().enumerate() {
            if is_p1 { sums.piece[i].0 += mag; } else { sums.piece[i].1 += mag; }
        }
    }

    sums.set_side(score_side_all(ctx));
    sums
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `pt::*` index aliases MUST line up with [`PIECE_TERMS`] by name — this
    /// is the invariant that keeps `score_piece_all`'s concrete dispatch (which
    /// indexes with `pt::*`) aligned with the fold/report (which iterate the
    /// table). Inserting a term without updating both would desync silently; this
    /// pins it.
    #[test]
    fn pt_indices_match_piece_terms_table() {
        assert_eq!(N_PIECE_TERMS, 10, "update this test when the term set changes");
        assert_eq!(PIECE_TERMS[pt::MATERIAL].name, "material");
        assert_eq!(PIECE_TERMS[pt::HP].name, "hp");
        assert_eq!(PIECE_TERMS[pt::ARMOR].name, "armor");
        assert_eq!(PIECE_TERMS[pt::SKILLS].name, "skills");
        assert_eq!(PIECE_TERMS[pt::MOBILITY].name, "mobility");
        assert_eq!(PIECE_TERMS[pt::EXPOSURE].name, "exposure");
        assert_eq!(PIECE_TERMS[pt::COVERAGE].name, "coverage");
        assert_eq!(PIECE_TERMS[pt::GUARD_ISO].name, "guard_isolation");
        assert_eq!(PIECE_TERMS[pt::CHAMPION_THREAT].name, "champion_threat");
        assert_eq!(PIECE_TERMS[pt::HANGING].name, "hanging_piece");
    }
}
