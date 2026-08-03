//! Term registry + the driving evaluation loop (ns-43).
//!
//! `default_terms` returns the ported term set in a stable order. `evaluate_dyn`
//! runs the terminal short-circuit, builds one [`EvalContext`], drives a single
//! shared board pass for per-piece terms (fanning each occupied square out to
//! every active per-piece term), runs side-level terms once, and returns a
//! [`DynBreakdown`] - the source of truth from which the legacy fixed-field
//! `EvalBreakdown` is projected.

use crate::state::Position;
use crate::state::position::GameResult;
use crate::search::counters;
use super::MATE_SCORE;
use super::params::EvalParams;
use super::context::EvalContext;
use super::term::{EvalTerm, PieceContext};
use super::terms;
use super::breakdown::{BreakdownDetail, EvalReport, PieceTermBreakdown, TermEntry};

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
        Box::new(terms::HangingPiece),
        // Side-level.
        Box::new(terms::Money),
        Box::new(terms::Tempo),
        Box::new(terms::OffensiveRange),
        Box::new(terms::WastedModifier),
        Box::new(terms::EndgameClosing),
        Box::new(terms::KingTempo),
    ]
}

/// Phase 1a (Session 48) - the term set is static: all terms are zero-size
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
        for i in 0..N_PIECE_TERMS {
            if is_p1 { sums.piece[i].0 += mags[i]; } else { sums.piece[i].1 += mags[i]; }
        }

        if let Some(rows) = pieces.as_mut() {
            // Per-piece term rows. Owner-signed via the term's own signing so a
            // penalty (exposure / guard_iso) reads negative for the owner.
            let mut terms_here = Vec::with_capacity(N_PIECE_TERMS);
            let mut piece_total = 0i32;
            for i in 0..N_PIECE_TERMS {
                let mag = mags[i];
                if mag == 0 { continue; }
                // owner-signed: signed_total(mag, 0) gives the P1-POV contribution
                // of a P1 magnitude; for a P2 piece we negate to owner-relative.
                let p1_pov = PIECE_TERM_SIGN[i] * mag;
                let owner_signed = if is_p1 { p1_pov } else { -p1_pov };
                piece_total += owner_signed;
                terms_here.push(TermEntry {
                    name: PIECE_TERM_NAMES[i].to_string(),
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

    let (money, tempo, off, wasted, close, king_tempo) = score_side_all(&ctx);
    sums.money = money; sums.tempo = tempo; sums.off = off;
    sums.wasted = wasted; sums.close = close; sums.king_tempo = king_tempo;

    let total = sums.fold_total(params);

    // Assemble the aggregate + side-level term lists (active terms only — a term
    // whose both-side magnitudes are zero is omitted, matching the old behaviour
    // of only emitting active terms).
    let mut agg = Vec::with_capacity(N_PIECE_TERMS);
    for i in 0..N_PIECE_TERMS {
        let (p1, p2) = sums.piece[i];
        if p1 == 0 && p2 == 0 { continue; }
        agg.push(TermEntry {
            name: PIECE_TERM_NAMES[i].to_string(),
            p1, p2,
            signed: PIECE_TERM_SIGN[i] * (p1 - p2),
        });
    }

    let mut side = Vec::with_capacity(6);
    push_side(&mut side, "money", sums.money, sums.money.0 - sums.money.1);
    push_side(&mut side, "tempo", sums.tempo, sums.tempo.0 - sums.tempo.1);
    push_side(&mut side, "offensive_range", sums.off,
              (sums.off.0 - sums.off.1) * params.offensive_range_weight);
    push_side(&mut side, "wasted_modifier", sums.wasted, -(sums.wasted.0 - sums.wasted.1));
    push_side(&mut side, "endgame_closing", sums.close, sums.close.0 - sums.close.1);
    push_side(&mut side, "king_tempo", sums.king_tempo, -(sums.king_tempo.0 - sums.king_tempo.1));

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    EvalReport { terms: agg, side_terms: side, pieces, total, terminal: false }
}

/// Push a side-level term entry, skipping it when both magnitudes are zero.
#[inline]
fn push_side(out: &mut Vec<TermEntry>, name: &'static str, mags: (i32, i32), signed: i32) {
    if mags.0 == 0 && mags.1 == 0 { return; }
    out.push(TermEntry { name: name.to_string(), p1: mags.0, p2: mags.1, signed });
}

/// Phase 1b (Session 48) - scalar search-leaf eval. Returns just the `i32`
/// total, with **no `DynBreakdown`, no `Vec` of active-term borrows, no
/// per-term `acc`/`entries` allocation, and no `to_legacy` projection.** All
/// of that machinery exists to feed the frontend eval panel / telemetry /
/// nn_trainer - none of which run in the inner search loop, which wants a
/// single scalar.
///
/// This is a **monomorphic** rewrite of `evaluate_dyn(...).total`: it calls
/// each ported term's concrete `score_piece` / `score_side` / `signed_total`
/// directly (static dispatch, no `dyn` vtable) and folds into one running
/// `i32`. The term set, order, `is_active` gates, and per-term signs are the
/// SAME as `default_terms` + each term's `signed_total`, so the result is
/// byte-identical to `evaluate_breakdown(pos).total` - pinned by
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

/// Number of per-piece terms, in the fixed order the cache / scalar path use.
pub(crate) const N_PIECE_TERMS: usize = 10;
/// Per-piece term indices (order = `default_terms` per-piece order).
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

/// Stable machine names for the per-piece terms, in `pt::*` order. The breakdown
/// wire format keys on these; the frontend maps them to display labels.
pub(crate) const PIECE_TERM_NAMES: [&str; N_PIECE_TERMS] = [
    "material", "hp", "armor", "skills", "mobility",
    "exposure", "coverage", "guard_isolation", "champion_threat", "hanging_piece",
];

/// Per-piece term signs, in `pt::*` order — the sign applied to `(p1 - p2)` when
/// folding into the P1-POV total. MUST stay in lockstep with
/// [`TermSums::fold_total`] (exposure, guard_isolation, and hanging are penalties).
pub(crate) const PIECE_TERM_SIGN: [i32; N_PIECE_TERMS] = [
    1,  // material
    1,  // hp
    1,  // armor
    1,  // skills
    1,  // mobility
    -1, // exposure (penalty)
    1,  // coverage
    -1, // guard_isolation (penalty)
    1,  // champion_threat
    -1, // hanging_piece (penalty)
];

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
    /// Fold with each term's own sign / weight. Byte-identical to the inline
    /// fold `evaluate_scalar` used before extraction (golden-gated).
    #[inline]
    pub(crate) fn fold_total(&self, params: &EvalParams) -> i32 {
        let p = &self.piece;
        let mut total = 0i32;
        total += p[pt::MATERIAL].0 - p[pt::MATERIAL].1;
        total += p[pt::HP].0       - p[pt::HP].1;
        total += p[pt::ARMOR].0    - p[pt::ARMOR].1;
        total += p[pt::SKILLS].0   - p[pt::SKILLS].1;
        total += p[pt::MOBILITY].0 - p[pt::MOBILITY].1;
        total -= p[pt::EXPOSURE].0 - p[pt::EXPOSURE].1;   // penalty
        total += p[pt::COVERAGE].0 - p[pt::COVERAGE].1;
        total -= p[pt::GUARD_ISO].0 - p[pt::GUARD_ISO].1; // penalty
        total += p[pt::CHAMPION_THREAT].0 - p[pt::CHAMPION_THREAT].1;
        total -= p[pt::HANGING].0 - p[pt::HANGING].1;      // penalty
        total += self.money.0 - self.money.1;
        total += self.tempo.0 - self.tempo.1;
        total += (self.off.0 - self.off.1) * params.offensive_range_weight;
        total -= self.wasted.0 - self.wasted.1;           // penalty
        total += self.close.0 - self.close.1;
        total -= self.king_tempo.0 - self.king_tempo.1;   // penalty
        total
    }
}

/// Score one occupied square across all per-piece terms, returning each term's
/// owner-relative magnitude in `pt::*` order. `guards_present` / `champions_present`
/// reproduce the `is_active` gates for GuardIsolation / ChampionThreat. Shared by
/// the full accumulation pass and the incremental per-piece update - one source
/// of truth for per-square term math.
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

/// Compute the side-level term values `(p1,p2)` with their `is_active` gates.
/// Order: money, tempo, off, wasted, close, king_tempo.
#[inline]
pub(crate) fn score_side_all(ctx: &EvalContext) -> (
    (i32, i32), (i32, i32), (i32, i32), (i32, i32), (i32, i32), (i32, i32),
) {
    use crate::state::position::Phase;
    let skill_phase = ctx.phase == Phase::Skill;
    let end_stage = matches!(ctx.stage, super::context::GameStage::End);
    let money  = terms::Money.score_side(ctx);
    let tempo  = terms::Tempo.score_side(ctx);
    let off    = terms::OffensiveRange.score_side(ctx);
    let wasted = if skill_phase { terms::WastedModifier.score_side(ctx) } else { (0, 0) };
    let close  = if end_stage   { terms::EndgameClosing.score_side(ctx) } else { (0, 0) };
    let king_tempo = terms::KingTempo.score_side(ctx);
    (money, tempo, off, wasted, close, king_tempo)
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
        for i in 0..N_PIECE_TERMS {
            if is_p1 { sums.piece[i].0 += mags[i]; } else { sums.piece[i].1 += mags[i]; }
        }
    }

    let (money, tempo, off, wasted, close, king_tempo) = score_side_all(ctx);
    sums.money = money; sums.tempo = tempo; sums.off = off;
    sums.wasted = wasted; sums.close = close; sums.king_tempo = king_tempo;
    sums
}
