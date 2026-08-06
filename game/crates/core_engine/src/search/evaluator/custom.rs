//! Custom evaluator (ns-55) — the designer's hand-rolled per-piece eval, and the
//! shipped default. This is the file you edit to build the evaluation.
//!
//! This is a complete, registered [`Evaluator`], selected by default. It is also
//! pickable from the setup / settings dropdowns (id `"custom"`).
//!
//! ## The model: one contextual score PER PIECE, plus a few side terms
//!
//! This evaluator is **not** a sum of independent add/subtract terms. Its core is
//! [`score_piece`]: ONE function, called once per occupied piece, that returns a
//! single contextual value for that piece. Inside it you start from a base value
//! and let **factors interact** — multiply it up for activity, down for exposure,
//! bend it by conditionals — because the factors are just local variables in one
//! function and can freely read each other. That is the whole point: an exposed
//! champion that is ALSO cut off from its guards can be worth far less than either
//! penalty alone, which a sum of terms can never express.
//!
//! The position total is: **sum of every piece's score (owner-signed) + the side
//! terms**. You never write a `total()` or touch the driver — you write
//! `score_piece` and the side-term fns; the framework walks the board once, signs
//! each piece by its owner, and folds everything.
//!
//! ## Side terms (money, tempo, …) — still write-once
//!
//! Whole-side quantities that aren't about one piece live in [`SIDE_TERMS`]. Each
//! is written ONCE from one side's perspective — `fn(ctx, is_p1) -> i32` returning
//! that side's positive magnitude — and the driver runs it for P1 and P2 and diffs
//! them. Read "my" side's state via `ctx` accessors (e.g. `ctx.money(is_p1)`).
//!
//! ## The shared context — computed once, borrowed everywhere
//!
//! [`CustomCtx::new`] runs once per `evaluate()` before the board walk. Put
//! anything a factor would otherwise recompute per-square in here (occupancy is
//! seeded; add attacker tables / game stage when your activity or safety factors
//! need them). `score_piece` and every side term borrow `&CustomCtx`.
//!
//! ## Panel breakdown (for now: per-piece total only)
//!
//! The hover-card shows each piece's final score. Factor-level decomposition
//! (activity 1.3×, exposure 0.6×, …) is deliberately deferred until the scoring
//! math settles — see the note in [`score_piece`] for the one-line hook to expose
//! a factor when you want it.
//!
//! ## What you may borrow (opt-in — you are NOT forced through the heuristic)
//!
//!   - `crate::search::see::{see_capture, see_single_hit, build_attackers_table}`
//!     — static exchange eval (is this piece hanging?).
//!   - `crate::search::quiescence::is_king_threatened(pos, side)` — one-tempo-
//!     from-death check.
//!   - `super::EvalContext` / `super::EvalParams` — the heuristic's per-call state
//!     and tuned weights, if you ever want to reuse them wholesale.
//!
//! To ship a variant: copy this file, rename the struct, add another
//! `builtin::BUILTINS` line (one edit each).

use crate::state::{MailboxEntry, Position};
use crate::state::position::GameResult;
use crate::state::magic::{cheby_dist, king_expand, skill_attacks, between, on_ray, within_range};
use crate::game_logic::skills::{skill_from_id, skill_category, skill_default_range, skill_cost, skill_target_owner, Skill, SkillCategory, TargetOwner};
use super::{BreakdownDetail, EvalReport, Evaluator, MATE_SCORE, PieceTermBreakdown, TermEntry};
use crate::search::evaluator::heuristic::context::{actions_per_round, max_owned_skill_cost};

/// Your evaluator. Zero-size for now; add fields (tuned weights, a cached table,
/// a loaded model handle) as you flesh it out. Keep it `Send`.
#[derive(Clone, Debug, Default)]
pub struct CustomEvaluator;

/// What kind of piece occupies a square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Guard,
    Champion,
    King,
}

/// One piece on the board, resolved from the bitboards so `score_piece` doesn't
/// have to poke them. `sq` is the square index; `is_p1` its owner; `mb` its
/// mailbox entry (hp / armor / skill ids).
#[derive(Clone, Copy)]
struct Piece {
    sq:    u8,
    is_p1: bool,
    kind:  Kind,
    /// Mailbox entry (hp / armor / skill ids) — read by the offense/exposure factors.
    mb:    MailboxEntry,
}

// ============================================================================
// THE SCORER — this is what you edit. One function, called once per piece.
// ============================================================================

fn score_piece(ctx: &CustomCtx, p: Piece) -> i32 {
    match p.kind {
        Kind::King     => score_king(ctx, p),
        Kind::Guard    => score_guard(ctx, p),
        Kind::Champion => score_champion(ctx, p),
    }
}

fn score_king(ctx: &CustomCtx, p: Piece) -> i32 {
    // The king IS a champion under the hood (2 equip slots, skills, ticks combos,
    // controls territory), so it runs the full champion factor chain — then the
    // king-specific danger malus is subtracted on top (kept safe, still valuable).
    let mult: f32 = CHAMP_FACTORS.iter().map(|(_, _, f)| f(ctx, p)).product();
    (BASE * mult).round() as i32 - king_danger_malus(ctx, p)
}

/// Base value every non-king piece starts from before its factors bend it.
/// The whole model is `BASE × f1 × f2 × …` (principle 1).
const BASE: f32 = 100.0;

/// Lookahead window (rounds) for the survivability race: this turn + next. Shared
/// by the cached per-side budgets and `survivability_severity` so they agree.
const IMMINENT: u16 = 2;

fn score_guard(ctx: &CustomCtx, p: Piece) -> i32 {
    // Guards carry no skills, so only the over-extension factor applies.
    (BASE * factor_overextension(ctx, p)).round() as i32
}

// ----- the champion factor chain --------------------------------------------
//
// A champion's value is `BASE × Π factor`. Each factor is a small `factor_*` fn
// returning an `f32` ≥ 0; they compose by multiplication, so every factor scales
// (and is scaled by) every other — an unsupported champ gets a proportionally
// smaller combo reward. Add a factor: write one fn, add one line here; a factor
// of 1.0 is a no-op. The `&str` names feed the (deferred) per-factor panel.

const CHAMP_FACTORS: &[(&str, &str, fn(&CustomCtx, Piece) -> f32)] = &[
    ("overextension", "Overext", factor_overextension), // clustering: 1.0 (together) → 0.0 (alone)
    ("combo",         "Combo",   factor_combo),          // combo set-up: 1.0 (none) → 1.5 (maxed)
    ("offense",       "Offense", factor_offense),        // attacker worth: 1.0 (none) → 2.0 (lone)
    ("exposure",      "Exposure", factor_exposure),      // vulnerability: 1.0 (safe) → 0.1 (dead)
];

fn score_champion(ctx: &CustomCtx, p: Piece) -> i32 {
    let mult: f32 = CHAMP_FACTORS.iter().map(|(_, _, f)| f(ctx, p)).product();
    (BASE * mult).round() as i32
}

// ----- scoring helpers (the factors) ----------------------------------------

/// Over-extension factor: `1.0` when the piece has friends in every ring, falling
/// toward `0.0` as it is pushed out alone (gated on enemy pressure). This is
/// clustering ("keep pieces together"), NOT whether the piece can be hit/killed —
/// vulnerability is a separate factor (see the plan's C4).
#[inline]
fn factor_overextension(ctx: &CustomCtx, p: Piece) -> f32 {
    1.0 - overextension_rate(ctx, p)
}

/// Combo factor: `1.0` (no combo potential) up to `1.5` (maxed), from a champion
/// sharing skill targets with a *different* combo-ticking champion near a real
/// target. The raw points (see [`combo_overlap_points`]) normalise against a
/// saturation cap; because this multiplies the base, an unsupported champ's combo
/// reward shrinks with its support factor rather than being tacked on regardless.
#[inline]
fn factor_combo(ctx: &CustomCtx, p: Piece) -> f32 {
    const SATURATION: f32 = 50.0; // points at which combo is "maxed"
    const MAX_GAIN: f32 = 0.5;    // +50% at strength 1.0
    let strength = (combo_overlap_points(ctx, p) as f32 / SATURATION).min(1.0);
    1.0 + MAX_GAIN * strength
}

/// Offense factor: an attacking piece is worth more, and — the point — worth
/// MORE per piece when it is the side's LAST attacker. Driven by the side's
/// Strike-carrier count via a hand-authored curve (designer-set):
/// 1 → ×2.0, 2 → ×1.75, 3 → ×1.57, 4 → ×1.43, 5 → ×1.31, 6 → ×1.22. The curve is
/// tuned so the COST to lose one attacker (Δ total offense mass) is strictly
/// larger the fewer you have (200/150/120/100/85/75) — no bounce-back. A Strike
/// carrier gets the full curve value; a Move-only carrier (Blast/Shove, gated on
/// a Strike existing to prime the combo) gets HALF the bonus-above-1.0 and does
/// NOT change the count; a piece with neither is `1.0` (no-op).
#[inline]
fn factor_offense(ctx: &CustomCtx, p: Piece) -> f32 {
    let s = ctx.side(p.is_p1);
    // Curve keyed on the side's Strike-carrier count (King included: C1c).
    let full = match s.strike_champs {
        0 => return 1.0, // no Strike on the side → no realisable offense to prime
        1 => 2.0,
        2 => 1.75,
        3 => 1.57,
        4 => 1.43,
        5 => 1.31,
        _ => 1.22,
    };
    match piece_offense_role(p) {
        OffenseRole::Strike => full,
        // Move-only: half the bonus-above-1.0 at the side's Strike-count tier.
        OffenseRole::Move => 1.0 + 0.5 * (full - 1.0),
        OffenseRole::None => 1.0,
    }
}

/// Exposure factor: scales the WHOLE accumulated chain DOWN when a piece is
/// genuinely in danger of being lost — the "…but you're about to lose it" damp on
/// everything the other factors built up. `1.0` when safe (no-op); drops toward a
/// deep cut as the shared survivability severity rises. Amplified by low effective
/// health: a 1-hp, unsupported piece that is dead-to-rights bottoms out near ×0.1
/// (designer anchor). Consumes [`CustomCtx::survivability_severity`] — the same race
/// the king danger malus uses — so exposure and king-danger agree on "about to die".
#[inline]
fn factor_exposure(ctx: &CustomCtx, p: Piece) -> f32 {
    let sev = ctx.survivability_severity(p); // 0 (safe) .. 1 (dead-to-rights)
    if sev <= 0.0 {
        return 1.0;
    }
    // Low effective health deepens the cut: a full-health piece can't fall as far
    // as a 1-hp one. floor(sev=1) ranges from ~0.5 (2hp+2armor) down to 0.1 (1hp,
    // no armor). Interpolate the drop by how much of the multiplier severity spends.
    let mb = p.mb;
    let eff = (mb.hp() as f32 + mb.armor() as f32).max(1.0); // 1..4
    // Deepest allowed cut for this piece's health: 1hp → 0.10, 4 (2hp+2arm) → 0.50.
    let floor = 0.10 + 0.40 * ((eff - 1.0) / 3.0);
    // severity 0 → 1.0, severity 1 → floor.
    1.0 - sev * (1.0 - floor)
}

/// This piece's offensive role from its own two skill slots: Strike takes
/// precedence over an enemy-moving Move (Blast/Shove). Self-move (Dash/Retreat)
/// and ally-Swap are NOT offense — they use `skill_ticks_combo` to be excluded,
/// consistent with combo reach and the side's attacker counts.
enum OffenseRole { Strike, Move, None }

#[inline]
fn piece_offense_role(p: Piece) -> OffenseRole {
    let mut has_move = false;
    for id in [p.mb.skill1(), p.mb.skill2()] {
        if let Some(sk) = skill_from_id(id) {
            if skill_category(sk) == SkillCategory::Strike {
                return OffenseRole::Strike;
            } else if skill_ticks_combo(sk) {
                has_move = true; // enemy-moving Move only
            }
        }
    }
    if has_move { OffenseRole::Move } else { OffenseRole::None }
}

/// Can this skill tick an enemy's combo counter? Per RULES.md, only a Strike (it
/// hits an enemy) or a Move skill that moves the *enemy target* (Blast, Shove)
/// ticks. Self-movement (Dash, Retreat) and ally-relocation (Swap) do NOT — so
/// carrying Dash grants no combo potential. This is the filter for combo reach.
#[inline]
fn skill_ticks_combo(s: Skill) -> bool {
    match skill_category(s) {
        SkillCategory::Strike => true,
        // A Move skill ticks only if it can act on an enemy piece.
        SkillCategory::Move => matches!(
            skill_target_owner(s),
            TargetOwner::Enemy | TargetOwner::Either
        ),
        SkillCategory::Shield | SkillCategory::Mystic => false,
    }
}

/// Raw combo-set-up points feeding [`factor_combo`] (NOT added to the score
/// directly). Points a champion for sharing skill targets with a *different*
/// combo-ticking champion near an enemy — the precondition for a multi-champion
/// combo. Reach = `skill_attacks` at the champ's longest combo-skill range
/// (queen-rays, blocked by any piece). Overlap = shared reach clipped to R4, with
/// a clear path. Per shared square: enemy-occupied +25, enemy at cheby 1 +10,
/// cheby 2 +5, else 0.
fn combo_overlap_points(ctx: &CustomCtx, p: Piece) -> i32 {
    let my_reach = ctx.combo_reach(p.is_p1, p.sq);
    if my_reach == 0 {
        return 0; // no combo-ticking skill
    }
    let r4 = within_range(p.sq, 4).0;
    let enemy = ctx.enemy_bb(p.is_p1);

    // Squares shared with any OTHER friendly combo-champion (reaches cached in
    // SideInfo::build). OR dedupes squares shared with two partners.
    let my_clipped = my_reach & r4;
    let mut overlap = 0u64;
    let side = ctx.side(p.is_p1);
    for &(other_sq, other_reach) in &side.reach[..side.reach_len] {
        if other_sq != p.sq {
            overlap |= my_clipped & other_reach;
        }
    }

    let mut points = 0i32;
    let mut sqs = overlap;
    while sqs != 0 {
        let f = sqs.trailing_zeros() as u8;
        sqs &= sqs - 1;
        // Skip squares this champion can't actually reach (blocked ray).
        if !(on_ray(p.sq, f) && (between(p.sq, f).0 & ctx.all_occ) == 0) {
            continue;
        }
        points += if enemy & (1u64 << f) != 0 {
            25
        } else if ctx.has_piece_in_ring(enemy, f, 1) {
            10
        } else if ctx.has_piece_in_ring(enemy, f, 2) {
            5
        } else {
            0
        };
    }
    points
}

/// Over-extension (missing support): `0.0` (a friend in every ring, or no threat)
/// up to `1.0` (alone in every ring while threatened). Each empty friendly ring
/// n = 1/2/3 adds weight 0.50/0.33/0.17; gated by enemy pressure (within R3 →
/// full, only R4 → half, none within R4 → 0). This is clustering, not
/// killability — see [`factor_overextension`].
fn overextension_rate(ctx: &CustomCtx, p: Piece) -> f32 {
    const RING_WEIGHT: [f32; 3] = [0.50, 0.33, 0.17];

    let enemy = ctx.enemy_bb(p.is_p1);
    let gate = if enemy & within_range(p.sq, 3).0 != 0 {
        1.0
    } else if enemy & within_range(p.sq, 4).0 != 0 {
        0.5
    } else {
        return 0.0; // no enemy within R4 → nothing to punish being alone
    };

    let own = ctx.own_bb(p.is_p1);
    let mut rate = 0.0;
    for (i, &weight) in RING_WEIGHT.iter().enumerate() {
        if !ctx.has_piece_in_ring(own, p.sq, (i + 1) as u8) {
            rate += weight;
        }
    }
    rate * gate
}

/// King-danger penalty (positive magnitude; `score_king` negates it). Consumes the
/// shared [`CustomCtx::survivability_severity`] — the SAME race `factor_exposure`
/// uses — so danger and exposure agree on "about to die". Maps severity `[0,1]` to
/// an escalating but **CAPPED** malus: gentle for a merely-pressured king, steep as
/// it approaches dead-to-rights, but bounded well below `MATE_SCORE` so a real
/// forced mate always strictly dominates a "probably dead" read.
///
/// This replaces the old `netto × 400` over a 5-round window, which was linear and
/// UNBOUNDED — a merely-pressured king could read −1200 (`netto ≈ 3 × 400`) because
/// the 5-round `affordable_casts` over-counted one-turn lethality. Severity now
/// carries the magnitude, imminently-scoped and squashed into `[0,1]` first.
fn king_danger_malus(ctx: &CustomCtx, p: Piece) -> i32 {
    /// Max malus for a dead-to-rights king. Big enough to dominate any piece value
    /// (so the AI spends everything to avoid / force it), but far below MATE_SCORE
    /// (1_000_000) so an actual forced capture always wins.
    const CAP: f32 = 6000.0;

    let sev = ctx.survivability_severity(p);
    if sev <= 0.0 {
        return 0; // king not in real danger
    }
    // Escalating curve: sev² ramps the malus up near dead-to-rights (sev 0.5 → 25%
    // of cap, sev 0.9 → 81%, sev 1.0 → cap) so mild pressure stays modest.
    (CAP * sev * sev).round() as i32
}

// ============================================================================
// BREAKDOWN SCORING — mirrors score_piece but also emits named TermEntry rows.
// Used only on the `with_rows = true` path (hover card / breakdown panel).
// Fast evaluate() path stays untouched.
// ============================================================================

/// Compute the same value as [`score_piece`] and also return named factor rows:
/// one `"base"` row, one delta row per CHAMP_FACTORS entry, and a `"king_danger"`
/// row for kings. All entries are owner-signed (`signed > 0` means benefit for
/// this piece's owner; `p1`/`p2` carry the absolute magnitude for the owner's
/// side only).
fn score_piece_with_terms(ctx: &CustomCtx, p: Piece) -> (i32, Vec<TermEntry>) {
    match p.kind {
        Kind::Guard => score_guard_with_terms(ctx, p),
        Kind::Champion => score_champion_with_terms(ctx, p),
        Kind::King => score_king_with_terms(ctx, p),
    }
}

fn score_guard_with_terms(ctx: &CustomCtx, p: Piece) -> (i32, Vec<TermEntry>) {
    let ov = factor_overextension(ctx, p);
    let score = (BASE * ov).round() as i32;

    let mut terms = Vec::with_capacity(2);
    terms.push(make_term("base", "Base", p.is_p1, BASE as i32));
    let ov_delta = score - BASE as i32; // ≤ 0
    if ov_delta != 0 {
        terms.push(make_term("overextension", "Overext", p.is_p1, ov_delta));
    }
    (score, terms)
}

fn score_champion_with_terms(ctx: &CustomCtx, p: Piece) -> (i32, Vec<TermEntry>) {
    let mut terms = Vec::with_capacity(CHAMP_FACTORS.len() + 1);
    terms.push(make_term("base", "Base", p.is_p1, BASE as i32));

    let mut running = BASE;
    for &(name, label, f) in CHAMP_FACTORS {
        let factor_val = f(ctx, p);
        let before_cp = running.round() as i32;
        running *= factor_val;
        let after_cp = running.round() as i32;
        let delta = after_cp - before_cp;
        if delta != 0 {
            terms.push(make_term(name, label, p.is_p1, delta));
        }
    }
    (running.round() as i32, terms)
}

fn score_king_with_terms(ctx: &CustomCtx, p: Piece) -> (i32, Vec<TermEntry>) {
    let mut terms = Vec::with_capacity(CHAMP_FACTORS.len() + 2);
    terms.push(make_term("base", "Base", p.is_p1, BASE as i32));

    let mut running = BASE;
    for &(name, label, f) in CHAMP_FACTORS {
        let factor_val = f(ctx, p);
        let before_cp = running.round() as i32;
        running *= factor_val;
        let after_cp = running.round() as i32;
        let delta = after_cp - before_cp;
        if delta != 0 {
            terms.push(make_term(name, label, p.is_p1, delta));
        }
    }
    let chain_score = running.round() as i32;
    let malus = king_danger_malus(ctx, p);
    if malus != 0 {
        terms.push(make_term("king_danger", "King danger", p.is_p1, -malus));
    }
    (chain_score - malus, terms)
}

/// Build one owner-signed `TermEntry` for a per-piece factor row.
/// `delta` is signed from the owner's perspective (positive = benefit, negative = cost).
/// `p1`/`p2` carry the absolute magnitude on the owning side only.
#[inline]
fn make_term(name: &'static str, label: &'static str, is_p1: bool, delta: i32) -> TermEntry {
    let abs = delta.unsigned_abs() as i32;
    TermEntry {
        name: name.to_string(),
        label: label.to_string(),
        p1: if is_p1 { abs } else { 0 },
        p2: if is_p1 { 0 } else { abs },
        signed: if is_p1 { delta } else { -delta },
    }
}

// TODO piece activity term

// TODO piece's skill combo potential term (measuring if champs are set up well together so they can act together)

// TODO skill produktivitäts term (measuring if the skills currently hold value in the context of all other own pieces and the opponents pieces)
// (maybe a side term - but it depends on how i will design this)

// ============================================================================
// TERRITORY — Go-style board control, computed once for BOTH sides together.
// ============================================================================

/// Which side controls each square, from a simultaneous flood-fill (BFS) out of
/// both sides' pieces. The square a side reaches in fewer 8-adjacent steps is
/// theirs; pieces are walls — the flood goes around occupied squares, not through
/// them (so a square walled off behind your pieces is clearly yours, and the
/// enemy simply can't reach it).
///
/// **Contested squares** count HALF. A square is contested when either:
///   - both sides reach it in the SAME wave (equal distance — a genuine tie), or
///   - it is clearly one side's but sits ADJACENT to a square the other side
///     clearly controls (the a4/a5 front: both border squares are contested).
///
/// A tie square is handed to BOTH sides (each at half), so it nets out in the
/// diff; a border square stays its owner's but at half value.
///
/// This is symmetric in the two sides, so it is computed ONCE (in
/// [`CustomCtx::new`]) and both `score(is_p1)` values are read out of it — the
/// `territory` side-term stays write-once in shape but never double-computes the
/// expensive flood.
struct Territory {
    /// Fixed-point control value per side (scaled by [`Territory::SCALE`] so the
    /// ×0.5 contested weight stays integral). Divide by `SCALE` for "squares".
    p1: i32,
    p2: i32,
}

impl Territory {
    /// Fixed-point scale: values are held ×2 so a contested square's ½ weight is
    /// an integer. `score()` returns the scaled value; the diff is scaled too,
    /// which only changes the term's overall weight (tune it at the call site).
    const SCALE: i32 = 2;

    /// Base worth of a plain controlled square, in whole squares (2026-08: raised
    /// 1 → 2 so board control competes with material). The near-king multiplier
    /// still stacks on top of this.
    const BASE_SQUARES: i32 = 2;

    /// Run the full pipeline and return the per-side control totals.
    fn compute(pos: &Position) -> Self {
        let occ = pos.p1_pieces.0 | pos.p2_pieces.0;
        let empty = !occ;

        // ── Phase 1: simultaneous BFS flood, pieces block. ──────────────────
        // Each side's front grows one 8-adjacent ring per wave, into EMPTY
        // squares only (walls block). A square goes to whoever reaches it first;
        // a square reached by BOTH in the same wave is an equal-distance tie —
        // it goes to both (`ties`), and counts half for each.
        let mut p1_reached = pos.p1_pieces.0; // sources: own pieces (distance 0)
        let mut p2_reached = pos.p2_pieces.0;
        let mut p1_ctrl = 0u64; // empty squares controlled by P1 (incl. ties)
        let mut p2_ctrl = 0u64;
        let mut ties = 0u64;    // equal-distance squares (belong to both, halved)
        let mut claimed = occ;  // squares already decided (start: all pieces)

        // ── Phase 0: guard pre-flood (speed 2 vs 1). ────────────────────────
        // Guards move 2 tiles/turn, Champions/King 1, so a guard reaches its full
        // R2 footprint in one turn. Give guards a TWO-ring expand up front (each ring
        // blocked by pieces/walls — the second grows only from empties the first
        // reached, so no jumping), resolve P1-vs-P2 ties on that whole footprint, and
        // fold it into the claimed set. Only THEN does the champion-speed main loop
        // run — so a champion cannot steal a square a guard already reached in one
        // turn, and both guards' R2 rings tie fairly against each other.
        {
            let p1_guards = pos.guards.0 & pos.p1_pieces.0;
            let p2_guards = pos.guards.0 & pos.p2_pieces.0;
            // Ring 1 then ring 2, each into still-empty squares (blocked pathing).
            let p1_r1 = king_expand(p1_guards) & empty;
            let p2_r1 = king_expand(p2_guards) & empty;
            let p1_front = (p1_r1 | (king_expand(p1_r1) & empty)) & !claimed;
            let p2_front = (p2_r1 | (king_expand(p2_r1) & empty)) & !claimed;
            let both = p1_front & p2_front;
            p1_ctrl    |= p1_front;
            p2_ctrl    |= p2_front;
            ties       |= both;
            p1_reached |= p1_front;
            p2_reached |= p2_front;
            claimed    |= p1_front | p2_front;
        }

        // The flood saturates in ≤8 King-steps across an 8×8 grid, but loop until
        // nothing new is claimed to be safe against odd wall shapes.
        loop {
            // One wave out of each side's current front, into still-empty squares.
            let p1_front = king_expand(p1_reached) & empty & !claimed;
            let p2_front = king_expand(p2_reached) & empty & !claimed;
            if p1_front == 0 && p2_front == 0 {
                break; // nothing left reachable
            }

            // Ties (both reach this wave) go to BOTH sides; the rest split cleanly.
            let both = p1_front & p2_front;
            p1_ctrl |= p1_front; // includes ties — a tie square is P1's AND P2's
            p2_ctrl |= p2_front;
            ties    |= both;

            // Advance both fronts and mark this wave's squares decided.
            p1_reached |= p1_front;
            p2_reached |= p2_front;
            claimed    |= p1_front | p2_front;
        }

        // ── Phase 2: contested = ties ∪ border squares. ─────────────────────
        // Border: a square I clearly control that is adjacent to one the enemy
        // clearly controls (the a4/a5 front). Ties are contested by definition.
        // "Clearly" excludes ties from the border test so a tie doesn't drag its
        // neighbours in twice — ties are already contested.
        let p1_only = p1_ctrl & !ties;
        let p2_only = p2_ctrl & !ties;
        let p1_contested = ties | (p1_only & king_expand(p2_only));
        let p2_contested = ties | (p2_only & king_expand(p1_only));

        // ── Phase 3+4: enemy-king bonus on top, applied AFTER halving. ──────
        // Squares near the ENEMY king are worth more (R1 → 3, R2 → 2, else 1),
        // never around your own king (so a side isn't lured to march its king
        // forward to farm points). `>= 64` means that king is off the board.
        let p1_king_sq = (pos.kings.0 & pos.p1_pieces.0).trailing_zeros();
        let p2_king_sq = (pos.kings.0 & pos.p2_pieces.0).trailing_zeros();

        // For P1's score the relevant enemy king is P2's, and vice-versa.
        let p1 = Self::sum_side(p1_ctrl, p1_contested, p2_king_sq);
        let p2 = Self::sum_side(p2_ctrl, p2_contested, p1_king_sq);

        Territory { p1, p2 }
    }

    /// Sum one side's control value (fixed-point ×SCALE): base BASE_SQUARES per
    /// square, halved on contested squares FIRST, then raised toward the enemy king
    /// (R1 → ×3, R2 → ×2). `enemy_king_sq >= 64` skips the bonus (king gone).
    fn sum_side(ctrl: u64, contested: u64, enemy_king_sq: u32) -> i32 {
        let has_enemy_king = enemy_king_sq < 64;
        let mut total = 0i32;
        let mut bits = ctrl;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let mask = 1u64 << sq;

            // Base worth BASE_SQUARES, held in fixed point (×SCALE). Contested → half.
            let full = Self::BASE_SQUARES * Self::SCALE;
            let mut value = if contested & mask != 0 { full / 2 } else { full };

            // King bonus on top, AFTER the halving (a contested R1 square is
            // 3 × ½ = 1.5 → SCALE*3/2). Multiply the already-halved value.
            if has_enemy_king {
                let mult = match cheby_dist(sq, enemy_king_sq as u8) {
                    1 => 3,
                    2 => 2,
                    _ => 1,
                };
                value *= mult;
            }
            total += value;
        }
        total
    }

    /// This side's control total (fixed-point). The side-term reads this.
    #[inline]
    fn score(&self, is_p1: bool) -> i32 {
        if is_p1 { self.p1 } else { self.p2 }
    }
}

// ============================================================================
// SIDE TERMS — whole-side quantities, written once, driven for both players.
// ============================================================================

/// A side-level term: score ONE side as a positive magnitude. Written once from
/// one side's perspective; the driver calls it for P1 and P2 and diffs them.
struct SideTerm {
    name:  &'static str,
    label: &'static str,
    sign:  i32,
    f:     fn(ctx: &CustomCtx, is_p1: bool) -> i32,
}

/// Side-level terms, in report order. ADD A LINE to register a term.
const SIDE_TERMS: &[SideTerm] = &[
    SideTerm { name: "skill_capacity",  label: "Skill cap", sign: 1, f: term_skill_capacity },
    SideTerm { name: "offense_capable", label: "Offense",   sign: 1, f: term_offense_capable },
    SideTerm { name: "territory",       label: "Territory", sign: 1, f: term_territory },
];

/// Offensive UTILISATION (C2b): penalise a side ONLY when it is ahead on offense
/// yet not converting that advantage — and only to the extent its attackers are
/// actually safe enough to convert. Never rewards; returns a non-positive
/// magnitude (0 or a penalty) which the driver diffs by owner, so a hoarding side
/// drops relative to its opponent. The three multiplicative gates make it
/// continuous (no single-ply switch):
///
///   1. **Advantage gate.** `adv = my_potential − enemy_potential` (realisable
///      attacker counts). `adv ≤ 0` → 0: at a disadvantage you're free to do
///      anything (preserve / play cool), never penalised.
///   2. **Realisation.** How much of my offense is bearing on the enemy, weighted
///      by target quality (King 1.0 > Champion 0.7 > Guard 0.3 — pressuring what
///      matters is real conversion; harassing guards is only partial). The GAP is
///      `1 − realising_frac`.
///   3. **Takeability gate (`factor_exposure`, C4).** A takeable attacker can't
///      safely convert, so pressure fades with the mean exposure factor of my
///      attackers (safe ≈ 1.0 → full pressure; takeable → 0.1 → pressure fades).
///      Shares the exact "about to die" read the exposure factor uses.
///
///   penalty = −WEIGHT × adv × (1 − realising_frac) × mean_exposure
fn term_offense_capable(ctx: &CustomCtx, is_p1: bool) -> i32 {
    const WEIGHT: f32 = 33.0; // ~−100 for 3 fully-unrealised, safe attackers (designer)

    let s = ctx.side(is_p1);
    // Realisable attacker potential (Move carriers count only if a Strike primes).
    let potential = |si: &SideInfo| {
        si.strike_champs + if si.strike_champs > 0 { si.move_champs } else { 0 }
    };
    let adv = potential(s) - potential(ctx.side(!is_p1));
    if adv <= 0 {
        return 0; // not ahead → no pressure (designer: free when behind/even)
    }

    // My attackers are the entries in the combo-reach cache (each carries a
    // combo-ticking skill). Weight each by the best enemy target its reach covers,
    // and track how many are safe (placeholder takeability gate).
    let enemy = ctx.enemy_bb(is_p1);
    let ek = enemy & ctx.pos.kings.0;
    let ec = enemy & ctx.pos.champions.0;
    let eg = enemy & ctx.pos.guards.0;

    let mut total = 0i32;
    let mut realise_sum = 0.0f32;
    let mut safe_sum = 0.0f32;
    for &(sq, reach) in &s.reach[..s.reach_len] {
        total += 1;
        let best = if reach & ek != 0 {
            1.0
        } else if reach & ec != 0 {
            0.7
        } else if reach & eg != 0 {
            0.3
        } else {
            0.0
        };
        realise_sum += best;
        // Takeability gate = this attacker's own exposure factor (C4). A safe
        // attacker (exposure ≈ 1.0) can convert → full pressure; a takeable one
        // (exposure → 0.1) can't safely lunge → pressure fades. Continuous, and
        // shares the exact "is this piece about to die" read the exposure factor
        // uses, so the gate and the piece's own value agree.
        let attacker = Piece {
            sq,
            is_p1,
            kind: ctx.kind_at(sq),
            mb: ctx.pos.mailbox[sq as usize],
        };
        safe_sum += factor_exposure(ctx, attacker);
    }
    if total == 0 {
        return 0; // adv>0 with no reachable attackers (edge) → nothing to push
    }

    let realising_frac = realise_sum / total as f32;
    let safe_frac = safe_sum / total as f32; // mean exposure factor of my attackers
    let penalty = WEIGHT * adv as f32 * (1.0 - realising_frac) * safe_frac;
    -(penalty.round() as i32)
}


/// Skill capacity: how much max-cost skill use this side can actually PAY FOR
/// over the next few rounds — saturated at what it could ever spend.
///
/// The idea: money only matters as *potential skill throughput*. A side that can
/// fire its most expensive skill in every action, this round and the next few,
/// is ahead — but only up to the point where extra money is dead ("runaway": you
/// cannot spend faster than your actions × cost cap allows). So we compute:
///
///   - `spend_cap`  = Σ over the lookahead window of `actions(r) × max_cost` —
///     the most this side could possibly spend in the window.
///   - `available`  = current treasury + Σ income over the window.
///   - `score`      = min(available, spend_cap) — capacity, capped at the runaway
///     point. Hoarding past `spend_cap` adds nothing (flat, not falling — no
///     incentive to waste money, just none to hoard).
///
/// `max_cost` is the single most expensive owned skill: we score the POTENTIAL to
/// fire it every action, not the expected use. Productivity (does the skill do
/// anything?) is a separate term, deliberately not folded in here.
///
/// Money uses [`CustomCtx::effective_money`], which credits the not-on-move side
/// its pending start-of-turn income — so a side isn't over-rated just for having
/// banked this round's income a half-turn before its opponent does.
fn term_skill_capacity(ctx: &CustomCtx, is_p1: bool) -> i32 {
    use crate::game_logic::turn_manager::income_per_turn;

    /// How many rounds ahead to look. Beyond this, plans are too speculative.
    const LOOKAHEAD: u16 = 3;
    /// Score weight per unit of payable capacity (tune the term's magnitude here).
    /// Raised 10 → 20 (2026-08): retained spending power must out-value spraying a
    /// cheap Dash/Shove — spending 3 money now costs up to −60 here, beating the
    /// small combo/utilisation nudge a pointless cast would otherwise win on.
    const PER_UNIT: i32 = 20;

    // Effective treasury already folds in this side's current-round income (real
    // for the mover, pending-credited for the other), so the lookahead below adds
    // only genuinely FUTURE rounds' income to avoid double-counting.
    let money = ctx.effective_money(is_p1);
    if money <= 0 {
        return 0; // broke → no capacity, and avoids the degenerate case
    }

    // `max_cost` is fixed over the window (equipped skills don't change) and is
    // precomputed once per side in SideInfo::build. Zero → no capacity.
    let max_cost = ctx.side(is_p1).max_skill_cost;
    if max_cost == 0 {
        return 0;
    }

    // spend_cap spans the whole window (you can spend THIS round too); future
    // income is counted only from the next round on (this round's is in `money`).
    let mut spend_cap = 0i32;
    let mut income = 0i32;
    let start = ctx.pos.round_number;
    for r in start..start.saturating_add(LOOKAHEAD) {
        let actions = actions_per_round(ctx.pos.current_phase, r) as i32;
        spend_cap += actions * max_cost;
        if r > start {
            income += income_per_turn(r) as i32;
        }
    }

    let available = money + income;
    // Capacity, saturated at the runaway point: money you can't spend is dead.
    available.min(spend_cap) * PER_UNIT
}

/// Territory: this side's Go-style board control, read from the shared
/// [`Territory`] computed once per eval. Written once; the driver diffs P1/P2.
fn term_territory(ctx: &CustomCtx, is_p1: bool) -> i32 {
    ctx.territory.score(is_p1)
}


// ============================================================================
// THE SHARED CONTEXT — computed once per eval, borrowed by the scorer + terms.
// ============================================================================

/// Everything the scorer / terms might want precomputed, built once in
/// [`CustomCtx::new`]. Seeded with the cheap occupancy bitboards; add attacker
/// tables, game stage, availability lookups, and factor helpers here as your
/// scoring grows to need them.
struct CustomCtx<'a> {
    pos:     &'a Position,
    /// All occupied squares (both players).
    all_occ: u64,
    /// Go-style board control, computed once for both sides (see [`Territory`]).
    territory: Territory,
    /// Per-side precomputed state, indexed `[0] = P1, [1] = P2`. Built once in
    /// `new` so the per-piece walk and the side-terms never re-scan the board.
    sides: [SideInfo; 2],
    /// Per-side Move-Attack threat masks: `move_attack_threat[i]` has a bit set for
    /// every square the side `i` (0=P1, 1=P2) can Move-Attack next turn — Guard@R2
    /// ∪ Champion/King@R1. Precomputed once (pure bitwise, no per-target loop) so
    /// `enemy_can_move_attack(defender)` is a single mask test.
    move_attack_threat: [u64; 2],
    /// Per-side, per-square nearest-Strike-carrier distance: `strike_dist[i][sq]` is
    /// the Chebyshev distance of the closest side-`i` Strike-carrier that can hit
    /// `sq` (true range +1, clear path), or `u8::MAX` if none. Painted once by
    /// walking each striker's `skill_attacks` reach — so `enemy_strike_reaches` and
    /// `nearest_enemy_striker_dist` become array lookups.
    strike_dist: [[u8; 64]; 2],
}

/// Per-side state precomputed once per eval (mirrors the heuristic's `EvalContext`
/// pattern of paying board scans once). Holds the champion combo-reach cache and
/// the skill-inventory counts the side-terms need.
#[derive(Default)]
struct SideInfo {
    /// Per-champion combo-reach bitboards, one entry per champion/king that carries
    /// a combo-ticking skill. Fixed inline buffer (max 5 champions + 1 king = 6) so
    /// `CustomCtx::new` does ZERO heap allocation on the hot eval path — the old
    /// `Vec` cost two allocs per eval, called millions of times in search. Valid
    /// entries are `reach[..reach_len]`.
    reach: [(u8, u64); 6],
    reach_len: usize,
    /// Count of champions carrying a Strike skill.
    strike_champs: i32,
    /// Count of champions/king carrying an enemy-moving Move skill (Blast/Shove)
    /// but no Strike. Self-move (Dash/Retreat) and ally-Swap don't count.
    move_champs: i32,
    /// Most expensive owned skill cost on this side (0 if none).
    max_skill_cost: i32,
    /// Cheapest Strike-skill cost owned on this side (0 if the side has none) —
    /// the per-strike price used when estimating incoming king damage.
    min_strike_cost: i32,
    /// Per-side affordable-cast budgets over the IMMINENT window, cached once in
    /// `CustomCtx::new` (identical for every piece on the side, so never recomputed
    /// per piece). `0` until filled.
    ///   - `strike_budget`: casts of this side's cheapest Strike (incoming when this
    ///     side is the attacker; retaliation when it's the defender).
    ///   - `defense_budget`: casts of this side's max-cost skill (its defensive spend).
    strike_budget: i32,
    defense_budget: i32,
}

impl SideInfo {
    /// One walk over this side's champions, collecting everything the per-piece
    /// scorer and side-terms need: the combo-reach cache and the skill counts.
    /// Skill inventory (min strike cost, defensive flags) also covers the King's
    /// two slots, since the King carries skills too.
    fn build(pos: &Position, side_bb: u64, all_occ: u64) -> Self {
        // Offense counting and combo reach cover champions AND the king — the king
        // is a champion under the hood (carries Strikes, ticks combos: C1c).
        let attackers = side_bb & (pos.champions.0 | pos.kings.0);
        let mut reach = [(0u8, 0u64); 6];
        let mut reach_len = 0usize;
        let mut strike_champs = 0i32;
        let mut move_champs = 0i32;
        let mut min_strike_cost = 0i32;

        // Champions + King: reach cache + per-piece strike/move tick classification.
        let mut bits = attackers;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;

            // Combo reach (only nonzero if it carries a combo-ticking skill).
            let r = CustomCtx::combo_reach_of(pos, all_occ, sq);
            if r != 0 && reach_len < reach.len() {
                reach[reach_len] = (sq, r);
                reach_len += 1;
            }

            // Skill inventory: a piece ticks at most once, Strike over Move.
            // Strike, or an enemy-moving Move skill (Blast/Shove) — the same
            // combo-ticking test used for reach. Self-move (Dash/Retreat) and
            // ally-Swap are NOT realisable offense and don't count.
            let mb = pos.mailbox[sq as usize];
            let mut has_strike = false;
            let mut has_move = false;
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(s) = skill_from_id(id) {
                    if skill_category(s) == SkillCategory::Strike {
                        has_strike = true;
                    } else if skill_ticks_combo(s) {
                        // enemy-moving Move (Blast/Shove) — ticks a combo on an enemy
                        has_move = true;
                    }
                }
            }
            if has_strike { strike_champs += 1; }
            else if has_move { move_champs += 1; }
        }

        // Side-wide skill inventory over Champions AND the King (both carry
        // skills): cheapest strike cost + which defensive skills exist.
        let mut skilled = side_bb & (pos.champions.0 | pos.kings.0);
        while skilled != 0 {
            let sq = skilled.trailing_zeros() as usize;
            skilled &= skilled - 1;
            let mb = pos.mailbox[sq];
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(s) = skill_from_id(id) {
                    if skill_category(s) == SkillCategory::Strike {
                        let c = skill_cost(s) as i32;
                        if min_strike_cost == 0 || c < min_strike_cost {
                            min_strike_cost = c;
                        }
                    }
                }
            }
        }

        SideInfo {
            reach,
            reach_len,
            strike_champs,
            move_champs,
            max_skill_cost: max_owned_skill_cost(pos, side_bb) as i32,
            min_strike_cost,
            // Filled by CustomCtx::new once the ctx exists (affordable_casts needs it).
            strike_budget: 0,
            defense_budget: 0,
        }
    }
}

impl<'a> CustomCtx<'a> {
    fn new(pos: &'a Position) -> Self {
        let all_occ = pos.p1_pieces.0 | pos.p2_pieces.0;
        let mut ctx = CustomCtx {
            pos,
            all_occ,
            territory: Territory::compute(pos),
            sides: [
                SideInfo::build(pos, pos.p1_pieces.0, all_occ),
                SideInfo::build(pos, pos.p2_pieces.0, all_occ),
            ],
            move_attack_threat: [
                Self::build_move_attack_threat(pos, pos.p1_pieces.0),
                Self::build_move_attack_threat(pos, pos.p2_pieces.0),
            ],
            strike_dist: [
                Self::build_strike_dist(pos, all_occ, pos.p1_pieces.0),
                Self::build_strike_dist(pos, all_occ, pos.p2_pieces.0),
            ],
        };
        // Cache the per-side affordable-cast budgets (identical for every piece on
        // the side) so `survivability_severity` never recomputes them per piece.
        for is_p1 in [true, false] {
            let i = if is_p1 { 0 } else { 1 };
            let strike = ctx.affordable_casts(is_p1, ctx.sides[i].min_strike_cost, IMMINENT);
            let defense = ctx.affordable_casts(is_p1, ctx.sides[i].max_skill_cost.max(1), IMMINENT);
            ctx.sides[i].strike_budget = strike;
            ctx.sides[i].defense_budget = defense;
        }
        ctx
    }

    /// Every square `attacker_bb`'s side can Move-Attack next turn: Guard@R2 ∪
    /// Champion/King@R1. Pure bitwise expansion of each attacker's disc — no
    /// per-target loop. `king_expand` twice = the R2 disc; once = R1.
    fn build_move_attack_threat(pos: &Position, attacker_bb: u64) -> u64 {
        let guards = attacker_bb & pos.guards.0;
        let champs_kings = attacker_bb & (pos.champions.0 | pos.kings.0);
        // Guard R2 footprint (two king-expansions) ∪ Champ/King R1 (one expansion).
        let guard_r2 = king_expand(king_expand(guards));
        let ck_r1 = king_expand(champs_kings);
        guard_r2 | ck_r1
    }

    /// Per-square nearest-Strike-carrier distance for `attacker_bb`'s side. For each
    /// Strike-carrier, walk the squares it can hit (`skill_attacks` at range+1, which
    /// already models blocking) and record the min Chebyshev distance per square.
    fn build_strike_dist(pos: &Position, all_occ: u64, attacker_bb: u64) -> [u8; 64] {
        let mut dist = [u8::MAX; 64];
        let mut cand = attacker_bb & (pos.champions.0 | pos.kings.0);
        while cand != 0 {
            let s = cand.trailing_zeros() as u8;
            cand &= cand - 1;
            let mb = pos.mailbox[s as usize];
            let mut reach = 0u8;
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(sk) = skill_from_id(id) {
                    if skill_category(sk) == SkillCategory::Strike {
                        reach = reach.max(skill_default_range(sk));
                    }
                }
            }
            if reach == 0 {
                continue;
            }
            // range + 1: the Striker may step one tile in before casting. skill_attacks
            // gives the queen-ray reach at that range with blocking already applied.
            let mut hits = skill_attacks(s, all_occ, reach + 1).0;
            while hits != 0 {
                let t = hits.trailing_zeros() as usize;
                hits &= hits - 1;
                let d = cheby_dist(s, t as u8);
                if d < dist[t] {
                    dist[t] = d;
                }
            }
        }
        dist
    }

    /// Precomputed state for the side `is_p1` selects.
    #[inline]
    fn side(&self, is_p1: bool) -> &SideInfo {
        &self.sides[if is_p1 { 0 } else { 1 }]
    }

    /// Cached combo-reach for the champion at `sq` on the side `is_p1`, or `0` if
    /// there's no champion there or it carries no combo-ticking skill. Looks up
    /// the reach computed once in [`SideInfo::build`] instead of recomputing it.
    #[inline]
    fn combo_reach(&self, is_p1: bool, sq: u8) -> u64 {
        let side = self.side(is_p1);
        side.reach[..side.reach_len].iter()
            .find(|&&(s, _)| s == sq)
            .map_or(0, |&(_, r)| r)
    }

    /// Resolve the piece kind at an occupied square.
    #[inline]
    fn kind_at(&self, sq: u8) -> Kind {
        let mask = 1u64 << sq;
        if self.pos.kings.0 & mask != 0 {
            Kind::King
        } else if self.pos.champions.0 & mask != 0 {
            Kind::Champion
        } else {
            Kind::Guard
        }
    }

    // --- per-side accessors: read "my" side's state from a side term ---------
    // Each returns the value for the side `is_p1` selects, so a side term is
    // written once and the driver runs it for both players. Add one per per-side
    // field your terms need (they all follow this p1/p2 pattern).

    /// This side's treasury.
    #[inline]
    fn money(&self, is_p1: bool) -> i32 {
        if is_p1 { self.pos.p1_money as i32 } else { self.pos.p2_money as i32 }
    }

    /// This side's **effective** treasury: real money, plus the guaranteed
    /// start-of-turn income it hasn't collected yet if it's NOT on the move.
    ///
    /// Income is disbursed at each player's turn-start, so between the two turns
    /// of a round the side that already moved looks richer purely by timing. This
    /// credits the not-on-move side its pending income so a static eval doesn't
    /// over-rate whoever happens to have just banked. Which round's income is
    /// pending depends on who moves next (`round_number` bumps on P2→P1):
    ///   - P1 on move → P2 collects THIS round → credit `income(round_number)`.
    ///   - P2 on move → P1 collects NEXT round → credit `income(round_number+1)`.
    ///
    /// R1 pays no income (rule), so `income_per_turn`'s callers gate at ≥ 2; we
    /// mirror that here by only crediting when the pending round is ≥ 2.
    fn effective_money(&self, is_p1: bool) -> i32 {
        use crate::game_logic::turn_manager::income_per_turn;
        let money = self.money(is_p1);
        let on_move = (self.pos.to_move == crate::state::position::Player::P1) == is_p1;
        if on_move {
            return money; // already banked this round's income
        }
        // Not on move: which round's income is still pending for this side?
        let pending_round = if is_p1 {
            self.pos.round_number + 1 // P2 on move; P1 collects next round
        } else {
            self.pos.round_number // P1 on move; P2 collects this round
        };
        if pending_round < 2 {
            return money; // R1 pays nothing
        }
        money + income_per_turn(pending_round) as i32
    }

    /// How many skill activations of unit cost `unit_cost` this side could afford
    /// over a `lookahead`-round window, limited BOTH by skill-phase actions and by
    /// money (treasury + future income). This is the shared money/action model the
    /// king-danger term uses for incoming strikes AND our defensive casts, so the
    /// two are compared on the same footing.
    ///
    /// `unit_cost == 0` (side owns no such skill) → 0. Money uses
    /// [`effective_money`] so the income-timing artefact never tilts the estimate.
    fn affordable_casts(&self, is_p1: bool, unit_cost: i32, lookahead: u16) -> i32 {
        use crate::game_logic::turn_manager::income_per_turn;
        if unit_cost <= 0 {
            return 0;
        }
        let mut money = self.effective_money(is_p1);
        let start = self.pos.round_number;
        let mut casts = 0i32;
        for r in start..start.saturating_add(lookahead) {
            // This round's income is already in `effective_money`; future rounds add.
            if r > start {
                money += income_per_turn(r) as i32;
            }
            let actions = actions_per_round(self.pos.current_phase, r) as i32;
            let affordable_by_money = money / unit_cost;
            let this_round = actions.min(affordable_by_money);
            if this_round <= 0 {
                continue;
            }
            casts += this_round;
            money -= this_round * unit_cost; // spend it so later rounds can't reuse it
        }
        casts
    }

    /// Is there a DIFFERENT friendly piece carrying Heal or Plate on a square
    /// adjacent (R1) to `sq`? Heal/Plate are adjacent-ally casts (never self), so a
    /// piece is only defensible this way by a neighbouring teammate that actually
    /// OWNS the skill — not merely by "some teammate somewhere owns Heal/Plate" and
    /// "some teammate is adjacent" (which could be two unrelated pieces).
    fn heal_or_plate_caster_adjacent(&self, is_p1: bool, sq: u8) -> bool {
        let ring1 = within_range(sq, 1).0 & !(1u64 << sq); // exclude self
        // Champions + King carry skills; guards don't.
        let mut cand = self.own_bb(is_p1) & (self.pos.champions.0 | self.pos.kings.0) & ring1;
        while cand != 0 {
            let s = cand.trailing_zeros() as usize;
            cand &= cand - 1;
            let m = self.pos.mailbox[s];
            for id in [m.skill1(), m.skill2()] {
                if let Some(sk) = skill_from_id(id) {
                    if matches!(sk, Skill::Heal | Skill::Plate) {
                        return true;
                    }
                }
            }
        }
        false
    }

    // --- factor helpers for `score_piece` ------------------------------------

    /// Occupancy bitboard of the side `is_p1` selects (the piece's own side).
    #[inline]
    fn own_bb(&self, is_p1: bool) -> u64 {
        if is_p1 { self.pos.p1_pieces.0 } else { self.pos.p2_pieces.0 }
    }

    /// Occupancy bitboard of the side opposing `is_p1`.
    #[inline]
    fn enemy_bb(&self, is_p1: bool) -> u64 {
        if is_p1 { self.pos.p2_pieces.0 } else { self.pos.p1_pieces.0 }
    }

    /// Can the side opposing `is_p1` Move-Attack `sq` next turn? Single mask test
    /// against the precomputed per-side Move-Attack threat map (Guard@R2 ∪
    /// Champion/King@R1). See [`build_move_attack_threat`].
    #[inline]
    fn enemy_can_move_attack(&self, is_p1: bool, sq: u8) -> bool {
        let enemy_idx = if is_p1 { 1 } else { 0 };
        self.move_attack_threat[enemy_idx] & (1u64 << sq) != 0
    }

    /// Is the piece at `sq` (owner `is_p1`) fully protected from Move-Attack by the
    /// Bodyguard Rule? The Rule lets a friendly Guard intercept iff it sits adjacent
    /// to BOTH the pre-target tile (the square the attacker stops on, along its
    /// approach) AND the defended piece. With free pathing the attacker may approach
    /// from any empty square adjacent to the target, so the piece is fully covered
    /// only if EVERY such approach square has a friendly Guard adjacent to both it
    /// and the piece. Any uncovered approach → not fully protected (attacker routes
    /// through it). Guards themselves can't be bodyguarded (only Champions/King).
    /// Is the piece at `sq` (owner `is_p1`) fully protected from Move-Attack by the
    /// Bodyguard Rule? Fully covered iff a friendly Guard is adjacent to both the
    /// target and EVERY delivery tile (adjacent enemy squares = R1 attackers, plus
    /// adjacent empty tiles = R2 guard stop tiles). Conservative: a false "covered"
    /// would hide real danger, so err toward "not covered". Only Champions/King can
    /// be bodyguarded; guards cannot.
    fn bodyguard_fully_covers(&self, is_p1: bool, sq: u8) -> bool {
        // Delivery tiles that must ALL be covered: adjacent ENEMY pieces (an R1
        // attacker delivers from its own square) plus adjacent EMPTY tiles (a speed-2
        // guard can stop there coming in from R2). Friendly-occupied neighbours are
        // not delivery tiles. Including adjacent enemies is the fix for the bug that
        // let an attacker already sitting adjacent (excluded as "occupied") slip past
        // the cover check and take an "undefended" champion.
        let adj = within_range(sq, 1).0 & !(1u64 << sq);
        let empty = !self.all_occ;
        let delivery = adj & (self.enemy_bb(is_p1) | empty);
        if delivery == 0 {
            return true; // no adjacency to deliver a Move-Attack from
        }
        let guards = self.own_bb(is_p1) & self.pos.guards.0;
        if guards == 0 {
            return false;
        }
        // Every delivery tile needs a friendly Guard adjacent to both it and `sq`.
        let mut ap = delivery;
        while ap != 0 {
            let a = ap.trailing_zeros() as u8;
            ap &= ap - 1;
            let covering = guards & within_range(sq, 1).0 & within_range(a, 1).0;
            if covering == 0 {
                return false; // this delivery tile is uncovered
            }
        }
        true
    }

    /// Shared survivability severity for ANY piece: `0.0` (safe) → `1.0`
    /// (dead-to-rights). Both `king_danger_malus` (C3) and `factor_exposure` (C4)
    /// consume this so they agree on "about to die". Combines:
    ///   - **reach** via two vectors: Move-Attack (Guard@R2/Champ-King@R1) minus
    ///     bodyguard cover, and Strike (range+1, clear path, distance-scaled — a
    ///     point-blank Striker threatens more than a far one; skills ignore bodyguard);
    ///   - **incoming** damage the enemy can afford & land IMMINENTLY (this turn +
    ///     next, not a long spend);
    ///   - **defense**: the owner's Shield/Plate/Heal, but only on the OWNER's own
    ///     turn (on the enemy's turn the defender can't heal);
    ///   - **retaliation**: if the enemy can't be Struck back next turn (no Strike
    ///     skill in range / can't afford it), the piece is MORE exposed; if we can
    ///     punish the trade, less.
    /// The net `incoming − (eff_health + defense)` is squashed into `[0,1]`.
    fn survivability_severity(&self, p: Piece) -> f32 {
        let sq = p.sq;
        let enemy_idx = if p.is_p1 { 1 } else { 0 };

        // --- reach: the two attack vectors (precomputed maps) --------------
        let move_attack =
            self.enemy_can_move_attack(p.is_p1, sq) && !self.bodyguard_fully_covers(p.is_p1, sq);

        // Nearest enemy Strike-carrier distance to this square, or u8::MAX if none.
        let strike_dist = self.strike_dist[enemy_idx][sq as usize];
        let strike_reaches = strike_dist != u8::MAX;

        if !move_attack && !strike_reaches {
            return 0.0; // unreachable → safe
        }

        let atk = self.side(!p.is_p1);

        // --- defense (owner's turn only) -----------------------------------
        // Computed first so the killable ceiling below can use eff_health.
        let me = self.side(p.is_p1);
        let on_move = (self.pos.to_move == crate::state::position::Player::P1) == p.is_p1;
        let mut defense = 0.0f32;
        // Shield is SELF-cast (protects only its own caster), so it counts only if
        // THIS piece carries Shield. Heal/Plate are adjacent-ally casts, so they
        // count only if a DIFFERENT friendly piece that owns Heal/Plate sits
        // adjacent. (Fixes the bugs where a piece read safe because a teammate owned
        // Shield, or because an unrelated teammate happened to be adjacent.)
        let self_shield = {
            let m = self.pos.mailbox[sq as usize];
            m.skill1() == Skill::Shield as u8 || m.skill2() == Skill::Shield as u8
        };
        if on_move
            && (self_shield || self.heal_or_plate_caster_adjacent(p.is_p1, sq))
        {
            defense = me.defense_budget as f32; // cached per side
        }

        // --- effective health ----------------------------------------------
        let mb = self.pos.mailbox[sq as usize];
        let eff_health = mb.hp() as f32 + mb.armor() as f32 + defense;

        // --- incoming (imminent) -------------------------------------------
        let mut incoming = 0.0f32;
        if move_attack {
            incoming += 1.0; // the one Move-Attack per turn
        }
        if strike_reaches {
            // Strikes the enemy can land ON THIS PIECE next turn. Capped by ONE
            // turn's Skill-Phase actions (they get one turn before we respond — a
            // multi-round money budget does NOT all land on one piece), then
            // distance-scaled. Closest enemy Striker (from the map) sets the factor.
            let actions = actions_per_round(self.pos.current_phase, self.pos.round_number) as f32;
            let casts = (atk.strike_budget as f32).min(actions);
            // dist 1 → 1.0, 2 → 0.83, 3 → 0.67, 4 → 0.5 (linear 1 − (d−1)/6).
            let dscale = (1.0 - (strike_dist.saturating_sub(1) as f32) / 6.0).max(0.0);
            incoming += casts * dscale;
        }
        // Killable ceiling: damage past what kills the piece is wasted overkill, so
        // it must NOT keep inflating severity. Cap incoming at eff_health + 1 (enough
        // to kill, +1 so "clearly lethal" still reads above "exactly lethal"). This
        // is the key fix for over-reading armored / home pieces as near-dead when the
        // attacker merely has a big affordable-cast budget.
        incoming = incoming.min(eff_health + 1.0);

        // --- retaliation reducer -------------------------------------------
        // If WE can Strike the enemy back next turn (own a Strike we can afford),
        // the trade is punishable → dampen the severity. Uses the cached strike
        // budget (min-strike casts over the imminent window).
        let can_retaliate = me.min_strike_cost > 0 && me.strike_budget > 0;
        let retaliation_damp = if can_retaliate { 0.6 } else { 1.0 };

        // --- squash net into [0,1] -----------------------------------------
        // A piece DIES when incoming reaches eff_health (≥, not >). Frame lethality
        // as `incoming − eff_health + 1`: one short of lethal → 0 (survives), exactly
        // lethal → 1, overkill → >1. Retaliation dampens (punishable trade).
        let raw_lethal = incoming - eff_health + 1.0;
        let lethal = raw_lethal * retaliation_damp;
        if lethal <= 0.0 {
            return 0.0;
        }
        // Certain death: incoming meets-or-exceeds effective health AND the owner
        // cannot act to save it this turn (no defense was available — enemy's turn,
        // or no shield/heal). That piece is essentially gone, so floor the severity
        // high (0.85) rather than the gentle "exactly lethal = 0.5". Overkill still
        // pushes it higher toward 1.0.
        let certain_death = raw_lethal >= 1.0 && defense <= 0.0;
        // lethal 1 (exactly dead) → 0.5, 2 → 0.67, 3 → 0.75… saturating toward 1.0.
        let sev = (lethal / (lethal + 1.0)).min(1.0);
        if certain_death { sev.max(0.85) } else { sev }
    }

    /// Skill-target reach of the piece at `sq` for combo purposes: the squares it
    /// could hit with its longest-range **combo-ticking** skill (Strike or Move).
    /// Returns `0` if it carries no such skill. Queen-rays via `skill_attacks`,
    /// blocked by any piece (`all_occ`). When two combo skills are equipped we use
    /// the MAX of their true ranges (the wider reach dominates the overlap set).
    ///
    /// Free function (not a `ctx` method) because it's called during
    /// [`SideInfo::build`], before the `CustomCtx` exists.
    fn combo_reach_of(pos: &Position, all_occ: u64, sq: u8) -> u64 {
        let mb = pos.mailbox[sq as usize];
        let mut max_range = 0u8;
        for id in [mb.skill1(), mb.skill2()] {
            if let Some(s) = skill_from_id(id) {
                if skill_ticks_combo(s) {
                    max_range = max_range.max(skill_default_range(s));
                }
            }
        }
        if max_range == 0 {
            return 0;
        }
        skill_attacks(sq, all_occ, max_range).0
    }

    /// Is there at least one piece of `side_bb` at exactly Chebyshev distance
    /// `ring` from `sq`? Uses the table-backed [`within_range`] annulus (the disc
    /// at `ring` minus the disc at `ring-1`) — one mask AND, no per-piece loop.
    #[inline]
    fn has_piece_in_ring(&self, side_bb: u64, sq: u8, ring: u8) -> bool {
        let inner = if ring == 0 { 0 } else { within_range(sq, ring - 1).0 };
        let shell = within_range(sq, ring).0 & !inner;
        side_bb & shell != 0
    }
}

// ============================================================================
// THE DRIVER — you should never need to touch below this line.
// Walks the board once, scores each piece + each side term, and produces BOTH
// the scalar total and the breakdown from the same pass.
// ============================================================================

/// A side term's accumulated `(p1_magnitude, p2_magnitude)`.
#[derive(Clone, Copy, Default)]
struct Sums {
    p1: i32,
    p2: i32,
}

/// Result of one board walk: the piece-score total (owner-signed), the side-term
/// sums, the grand total, and — when requested — the per-piece breakdown rows.
struct Scored {
    piece_total: i32,
    /// Fixed-size (no heap alloc on the hot path): one `Sums` per `SIDE_TERMS`.
    side_sums:   [Sums; SIDE_TERMS.len()],
    total:       i32,
    rows:        Option<Vec<PieceTermBreakdown>>,
}

impl CustomEvaluator {
    /// The single scoring pass. `score_piece` runs on every occupied square;
    /// every side term runs once per side. `with_rows` also assembles the
    /// per-piece breakdown. Both `evaluate` and `evaluate_report` call this, so
    /// the score and the report are the same numbers by construction.
    fn score(&self, pos: &Position, with_rows: bool) -> Scored {
        let ctx = CustomCtx::new(pos);

        let mut piece_total = 0i32;
        let mut rows: Option<Vec<PieceTermBreakdown>> =
            with_rows.then(|| Vec::with_capacity(ctx.all_occ.count_ones() as usize));

        // One board pass: score each occupied piece in context, sign by owner.
        let mut bits = ctx.all_occ;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let mask = 1u64 << sq;
            let is_p1 = pos.p1_pieces.0 & mask != 0;
            let kind = ctx.kind_at(sq);
            let mb = pos.mailbox[sq as usize];
            let piece = Piece { sq, is_p1, kind, mb };

            let (mag, factor_terms) = if rows.is_some() {
                score_piece_with_terms(&ctx, piece)
            } else {
                (score_piece(&ctx, piece), Vec::new())
            };
            let owner_signed = if is_p1 { mag } else { -mag };
            piece_total += owner_signed;

            if let Some(rows) = rows.as_mut() {
                let piece_kind = match kind { Kind::King => 3, Kind::Champion => 2, Kind::Guard => 1 };
                rows.push(PieceTermBreakdown {
                    sq, is_p1, piece_kind,
                    hp: mb.hp(), armor: mb.armor(),
                    skill1_id: mb.skill1(), skill2_id: mb.skill2(),
                    terms: factor_terms,
                    piece_total: owner_signed,
                });
            }
        }

        // Side terms: run each once per side (written once, driven for both).
        // Fixed-size array — no heap allocation on the hot `evaluate` path.
        let mut side_sums = [Sums::default(); SIDE_TERMS.len()];
        let mut total = piece_total;
        for (i, t) in SIDE_TERMS.iter().enumerate() {
            let s = Sums { p1: (t.f)(&ctx, true), p2: (t.f)(&ctx, false) };
            total += t.sign * (s.p1 - s.p2);
            side_sums[i] = s;
        }

        Scored { piece_total, side_sums, total, rows }
    }
}

impl Evaluator for CustomEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 {
        match terminal_score(pos) {
            Some(s) => s,
            None => self.score(pos, false).total,
        }
    }

    /// Opt OUT of quiescence search. QS exists to cover a horizon-blind eval on
    /// mid-exchange positions; this per-piece eval weighs a piece by its exposure
    /// (hp/armor vs threat) so it reads those positions directly. On this game's
    /// King-danger endgames QS is an undisciplined full-width search that starves
    /// the main tree of depth — measured +1.5–2 plies deeper with QS off, same or
    /// better moves. See `game/plans/custom-eval-search-cliff.md`.
    #[inline]
    fn wants_qs(&self) -> bool { false }

    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport {
        if let Some(s) = terminal_score(pos) {
            return EvalReport::terminal(s);
        }

        let want_rows = matches!(detail, BreakdownDetail::PerPiece);
        let scored = self.score(pos, want_rows);

        let mut terms = Vec::new();
        if scored.piece_total != 0 {
            terms.push(TermEntry {
                name: "pieces".to_string(),
                label: "Pieces".to_string(),
                p1: scored.piece_total.max(0),
                p2: (-scored.piece_total).max(0),
                signed: scored.piece_total,
            });
        }

        let side_terms = SIDE_TERMS.iter().zip(&scored.side_sums)
            .filter(|(_, s)| s.p1 != 0 || s.p2 != 0)
            .map(|(t, s)| TermEntry {
                name: t.name.to_string(),
                label: t.label.to_string(),
                p1: s.p1, p2: s.p2,
                signed: t.sign * (s.p1 - s.p2),
            })
            .collect();

        EvalReport {
            terms,
            side_terms,
            pieces: scored.rows,
            total: scored.total, // == evaluate(): same walk, same numbers
            terminal: false,
        }
    }
}

/// Terminal shortcut: `Some(±MATE_SCORE)` if the game is decided, else `None`.
/// Shared by both trait methods so they can never disagree on terminals.
#[inline]
fn terminal_score(pos: &Position) -> Option<i32> {
    match pos.game_result {
        Some(GameResult::P1Wins) => Some(MATE_SCORE),
        Some(GameResult::P2Wins) => Some(-MATE_SCORE),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_scores_and_reports_consistently() {
        let pos = Position::setup_stack_m();
        let ev = CustomEvaluator;
        let r = ev.evaluate_report(&pos, BreakdownDetail::Aggregate);
        assert_eq!(r.total, ev.evaluate(&pos), "report total must equal evaluate()");
        assert!(!r.terminal);
    }

    #[test]
    fn custom_per_piece_rows_sum_to_pieces_term() {
        // The per-piece rows must reconstruct the aggregate "pieces" term.
        let pos = Position::setup_stack_m();
        let ev = CustomEvaluator;
        let r = ev.evaluate_report(&pos, BreakdownDetail::PerPiece);
        let rows = r.pieces.expect("PerPiece requested");
        let row_sum: i32 = rows.iter().map(|row| row.piece_total).sum();
        let pieces = r.terms.iter().find(|t| t.name == "pieces").map(|t| t.signed).unwrap_or(0);
        assert_eq!(row_sum, pieces);
    }

    #[test]
    fn factor_terms_sum_to_piece_total() {
        // Each piece's factor terms (signed) must sum to its piece_total.
        let pos = Position::setup_stack_m();
        let r = CustomEvaluator.evaluate_report(&pos, BreakdownDetail::PerPiece);
        let rows = r.pieces.expect("PerPiece requested");
        for row in &rows {
            let term_sum: i32 = row.terms.iter().map(|t| t.signed).sum();
            assert_eq!(
                term_sum, row.piece_total,
                "sq={} kind={} terms don't add up to piece_total",
                row.sq, row.piece_kind
            );
        }
    }

    #[test]
    fn custom_terminal() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        let r = CustomEvaluator.evaluate_report(&pos, BreakdownDetail::PerPiece);
        assert!(r.terminal);
        assert_eq!(r.total, MATE_SCORE);
    }

    /// A vertically-mirrored board (P1 rank r ↔ P2 rank 7-r, kings on the same
    /// file) must give both sides identical territory — the diff is exactly 0.
    /// This pins the flood, the contested split, and the enemy-king bonus all
    /// being symmetric.
    #[test]
    fn territory_is_symmetric_on_mirrored_board() {
        let mut pos = Position::empty();
        // P1: king d1 (sq 3), a champion at c1 (sq 2). P2: mirror across ranks
        // (sq ^ 56): king d8 (sq 59), champion c8 (sq 58). Same files → symmetric.
        for (sq, is_p1) in [(3u8, true), (2u8, true), (59u8, false), (58u8, false)] {
            let bit = 1u64 << sq;
            if is_p1 { pos.p1_pieces.0 |= bit; } else { pos.p2_pieces.0 |= bit; }
        }
        pos.kings.0     = (1 << 3) | (1 << 59);
        pos.champions.0 = (1 << 2) | (1 << 58);

        let t = Territory::compute(&pos);
        assert_eq!(t.p1, t.p2, "mirrored board must be territorially even");
    }

    /// Contested squares (ties + borders) must actually lower a side's raw count:
    /// a board where the two sides' territories touch should score less than the
    /// same square set with no opposing pressure. Sanity check that halving fires.
    #[test]
    fn territory_contested_halves_the_frontier() {
        // Two lone kings facing off in the centre: every empty square is a tie
        // or a border, so both sides' control is heavily halved but still equal.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = 1 << 27; // d4
        pos.p2_pieces.0 = 1 << 35; // d5
        pos.kings.0 = (1 << 27) | (1 << 35);

        let t = Territory::compute(&pos);
        assert_eq!(t.p1, t.p2, "symmetric face-off is even");
        assert!(t.p1 > 0, "each side still controls something");
    }

    /// Guard speed (2) out-races champion speed (1): a square two rings from a
    /// guard but one ring from an enemy champion is claimed by the GUARD's side
    /// (both reach it in one turn → it's the guard-owner's, not the champion's),
    /// because the guard pre-flood claims its R2 footprint before champions move.
    #[test]
    fn territory_guard_speed_beats_champion() {
        // P1 guard a1(0); P2 champion a4(24). a3(16) is Chebyshev 2 from the guard
        // (one guard turn) and 1 from the champ (one champ turn) — a real tie in
        // TURNS. Old tile-distance flood gave it to the champ (wave 1 < wave 2);
        // the speed-aware pre-flood makes the guard claim it first.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = 1 << 0;   // a1 guard
        pos.p2_pieces.0 = 1 << 24;  // a4 champ
        pos.guards.0    = 1 << 0;
        pos.champions.0 = 1 << 24;

        let t = Territory::compute(&pos);
        // With the guard reaching further, P1 should control at least as much as
        // P2 here (the guard's extra ring tips squares P1's way).
        assert!(t.p1 >= t.p2, "guard speed should not lose ground it reaches first");

        // Concretely: a3(16) must be reachable-first by the guard. Rebuild with the
        // champion removed and confirm the guard alone already owns a3 — then adding
        // the champ must not steal it (it can only tie, and a tie is shared).
        let mut solo = pos.clone();
        solo.p2_pieces.0 = 0;
        solo.champions.0 = 0;
        let guard_only = Territory::compute(&solo);
        assert!(guard_only.p1 > t.p2, "lone guard out-controls the lone champ's reach");
    }

    /// factor_offense curve + the last-attacker cliff: the per-piece multiplier is
    /// keyed on the side's Strike-carrier count (1→2.0, 2→1.75, 3→1.57, 4→1.43,
    /// 5→1.31, 6→1.22), tuned so the COST to lose an attacker is strictly larger
    /// the fewer you have. Move-only carriers get half the bonus and don't change
    /// the count; non-attackers stay 1.0.
    #[test]
    fn offense_factor_rewards_last_attacker_most() {
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Strike
        let blast = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(10); // Move-only
        let plain = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2); // no skills

        // Helper: build a P1 side with `n` Strike champs on a1.., score the first.
        let curve = |n: usize| -> f32 {
            let mut pos = Position::empty();
            let mut bb = 0u64;
            for i in 0..n { bb |= 1 << (i as u8); }
            pos.p1_pieces.0 = bb;
            pos.champions.0 = bb;
            for i in 0..n { pos.mailbox[i] = lance; }
            let ctx = CustomCtx::new(&pos);
            factor_offense(&ctx, Piece { sq: 0, is_p1: true, kind: Kind::Champion, mb: lance })
        };
        let expected = [2.00f32, 1.75, 1.57, 1.43, 1.31, 1.22];
        for (i, &want) in expected.iter().enumerate() {
            let n = i + 1;
            assert!((curve(n) - want).abs() < 1e-6, "{n} attackers → ×{want}");
        }

        // The core property: cost to lose one attacker (Δ total offense mass) is
        // STRICTLY larger the fewer you have — never bounces back up.
        let total = |n: usize| n as f32 * 100.0 * curve(n);
        let mut prev_keep = f32::INFINITY;
        for n in 1..=6 {
            let keep = total(n) - if n == 1 { 0.0 } else { total(n - 1) };
            assert!(keep < prev_keep, "keep-value must strictly fall as n grows (n={n})");
            prev_keep = keep;
        }

        // Move-only carrier alongside 1 Strike: count stays 1 (×2.0 tier), the Move
        // champ gets half the bonus → 1.0 + 0.5×(2.0−1.0) = 1.5; the plain champ 1.0.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 0) | (1 << 1) | (1 << 2);
        pos.champions.0 = (1 << 0) | (1 << 1) | (1 << 2);
        pos.mailbox[0] = lance;
        pos.mailbox[1] = blast;
        pos.mailbox[2] = plain;
        let ctx = CustomCtx::new(&pos);
        let mk = |sq: u8, mb| Piece { sq, is_p1: true, kind: Kind::Champion, mb };
        assert!((factor_offense(&ctx, mk(0, lance)) - 2.0).abs() < 1e-6, "strike ×2.0");
        assert!((factor_offense(&ctx, mk(1, blast)) - 1.5).abs() < 1e-6, "move half-bonus");
        assert!((factor_offense(&ctx, mk(2, plain)) - 1.0).abs() < 1e-6, "no offense → 1.0");

        // No Strike anywhere: even a Move carrier is 1.0 (nothing to prime).
        let mut np = Position::empty();
        np.p1_pieces.0 = 1 << 0;
        np.champions.0 = 1 << 0;
        np.mailbox[0] = blast;
        let ctx = CustomCtx::new(&np);
        assert!((factor_offense(&ctx, mk(0, blast)) - 1.0).abs() < 1e-6, "no strike → move is 1.0");
    }

    /// The income-timing fix: on a mirrored board with equal money and skills,
    /// the side NOT on the move is credited its pending income, so the
    /// `skill_capacity` term reads equal for both — no artefact from the mover
    /// having banked this round's income a half-turn early.
    #[test]
    fn skill_capacity_equalises_income_timing() {
        let mut pos = Position::empty();
        pos.round_number = 3; // ≥ 2 so income is live
        pos.to_move = crate::state::position::Player::P1;
        pos.current_phase = crate::state::position::Phase::Skill;

        // One champion each, mirrored (sq ^ 56), both carrying Steal (id 4, cost 4).
        let mb = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(4);
        pos.p1_pieces.0 = 1 << 10; // c2
        pos.p2_pieces.0 = 1 << 50; // c7 (10 ^ 56)
        pos.champions.0 = (1 << 10) | (1 << 50);
        pos.mailbox[10] = mb;
        pos.mailbox[50] = mb;

        // Realistic timing: P1 is on the move and has ALREADY banked round 3's
        // income; P2 has not yet (it collects at the start of its own turn). So a
        // genuinely even position has P2's raw money lower by exactly income(3).
        use crate::game_logic::turn_manager::income_per_turn;
        let r3 = income_per_turn(3) as u16;
        pos.p1_money = 8;
        pos.p2_money = 8 - r3;

        let ctx = CustomCtx::new(&pos);
        // effective_money credits P2 its pending income, lifting it back to P1's
        // level, so the term must read equal for both sides.
        assert_eq!(
            term_skill_capacity(&ctx, true),
            term_skill_capacity(&ctx, false),
            "income timing must not tilt an evenly-timed position"
        );

        // The credit is exactly this round's income; the mover keeps its banked total.
        assert_eq!(
            ctx.effective_money(false),
            ctx.money(false) + r3 as i32,
        );
        assert_eq!(ctx.effective_money(true), ctx.money(true), "mover already banked");
    }

    /// Offensive utilisation (C2b): penalise ONLY a side that is ahead on offense
    /// and not converting it, scaled by how safe/convertible its attackers are.
    /// Never rewards; disadvantaged/even sides are always 0.
    #[test]
    fn offense_utilisation_penalises_unconverted_advantage() {
        let strike = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(2); // Hook (range 2)

        // Case A: no offensive advantage (P2 has an equal Strike carrier) → 0.
        let mut even = Position::empty();
        even.p1_pieces.0 = 1 << 10; // c2
        even.p2_pieces.0 = 1 << 50; // c7
        even.champions.0 = (1 << 10) | (1 << 50);
        even.mailbox[10] = strike;
        even.mailbox[50] = strike;
        let ctx = CustomCtx::new(&even);
        assert_eq!(term_offense_capable(&ctx, true), 0, "no advantage → no pressure");
        assert_eq!(term_offense_capable(&ctx, false), 0, "even → both 0");

        // Case B: P1 ahead on offense (1 Strike vs 0), attacker far from any enemy
        // (no reach onto enemy pieces) and SAFE (no enemy can Move-Attack it) →
        // hoarding an unconverted, convertible advantage → penalty.
        let mut ahead = Position::empty();
        ahead.p1_pieces.0 = 1 << 0;  // a1 — tucked in the corner, far from enemy
        ahead.p2_pieces.0 = 1 << 63; // h8 lone champ, no Strike
        ahead.champions.0 = (1 << 0) | (1 << 63);
        ahead.mailbox[0] = strike;
        let ctx = CustomCtx::new(&ahead);
        let pen = term_offense_capable(&ctx, true);
        assert!(pen < 0, "ahead + not converting + safe → penalty, got {pen}");
        // The disadvantaged side is never penalised.
        assert_eq!(term_offense_capable(&ctx, false), 0, "behind side is free");

        // Case C: same advantage, but the attacker's reach now covers the enemy KING
        // (full realisation) → the gap closes → penalty shrinks toward 0. Enemy king
        // on a3(16) is 2 tiles from the a1 Hook carrier (in range-2 reach) but at R2,
        // so it can't Move-Attack the attacker — safe_frac stays 1, isolating the
        // realisation effect.
        let mut converting = ahead.clone();
        converting.p2_pieces.0 = 1 << 16;
        converting.champions.0 = 1 << 0;
        converting.kings.0 = 1 << 16;
        converting.mailbox[16] = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2);
        let ctx = CustomCtx::new(&converting);
        let pen_conv = term_offense_capable(&ctx, true);
        assert!(pen_conv > pen, "converting on the king → smaller penalty ({pen_conv} vs {pen})");
    }

    /// Combo overlap: two champions that both threaten enemy-near squares earn
    /// the proximity points, summed over every shared square; a lone champion with
    /// no partner earns nothing. This pins the RAW points fed into `factor_combo`
    /// (the multiplier itself is tested separately).
    #[test]
    fn combo_overlap_rewards_shared_target_near_enemy() {
        // P1 champs on c3(18) and e3(20), both carrying Lance (Strike, range 1).
        // Their range-1 reaches overlap on the column between them: d2(11), d3(19),
        // d4(27). With an enemy on d5(35): d4 is cheby-1 (+10), d3 cheby-2 (+5),
        // d2 cheby-3 (+0). All paths are adjacent (nothing between). Total = 15.
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1);
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 18) | (1 << 20);
        pos.champions.0 = (1 << 18) | (1 << 20) | (1 << 35);
        pos.p2_pieces.0 = 1 << 35;
        pos.mailbox[18] = lance;
        pos.mailbox[20] = lance;

        let ctx = CustomCtx::new(&pos);
        let c3 = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: lance };
        assert_eq!(combo_overlap_points(&ctx, c3), 15, "d4(+10) + d3(+5), d2 too far");

        // Remove the partner on e3 → no other champion shares any square → no points.
        let mut solo = pos.clone();
        solo.p1_pieces.0 = 1 << 18;
        solo.champions.0 = (1 << 18) | (1 << 35);
        solo.mailbox[20] = crate::state::EMPTY_MAILBOX_ENTRY;
        let ctx = CustomCtx::new(&solo);
        assert_eq!(combo_overlap_points(&ctx, c3), 0, "no partner → no combo setup");
    }

    /// The combo filter: only skills that tick an ENEMY combo counter grant reach.
    /// Self-movement (Dash/Retreat) and ally-relocation (Swap) must NOT — carrying
    /// Dash gives zero combo potential even with a partner and a real target.
    #[test]
    fn combo_reach_excludes_self_move_skills() {
        // Same geometry as the shared-target test (c3=18, e3=20, enemy d5=35), but
        // both champs carry DASH (id 9, self-move) instead of Lance. Dash can't tick
        // an enemy counter, so neither champ has combo reach → 0 points.
        let dash = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(9);
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 18) | (1 << 20);
        pos.champions.0 = (1 << 18) | (1 << 20) | (1 << 35);
        pos.p2_pieces.0 = 1 << 35;
        pos.mailbox[18] = dash;
        pos.mailbox[20] = dash;
        let ctx = CustomCtx::new(&pos);
        let c3 = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: dash };
        assert_eq!(combo_overlap_points(&ctx, c3), 0, "Dash is self-move → no combo");

        // Swap into Blast (id 10, moves an enemy → DOES tick): reach returns, so the
        // shared-target points reappear. Confirms the filter admits enemy-movers.
        let blast = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(10);
        pos.mailbox[18] = blast;
        pos.mailbox[20] = blast;
        let ctx = CustomCtx::new(&pos);
        let c3 = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: blast };
        assert!(combo_overlap_points(&ctx, c3) > 0, "Blast moves an enemy → combo");
    }

    /// The combo multiplier: raw points normalise into a factor in `[1.0, 1.5]`,
    /// and — the whole point of C1 — it COMPOSES with exposure, so a combo on an
    /// exposed champ yields a smaller absolute reward than on a safe one.
    #[test]
    fn combo_factor_composes_with_exposure() {
        // No combo potential → factor is exactly 1.0 (no-op).
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1);
        let mut solo = Position::empty();
        solo.p1_pieces.0 = 1 << 18;
        solo.champions.0 = 1 << 18;
        solo.mailbox[18] = lance;
        let ctx = CustomCtx::new(&solo);
        let c = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: lance };
        assert!((factor_combo(&ctx, c) - 1.0).abs() < 1e-6, "no combo → factor 1.0");

        // 15 points (the shared-target case above) → 1.0 + 0.5 × (15/50) = 1.15.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 18) | (1 << 20);
        pos.champions.0 = (1 << 18) | (1 << 20) | (1 << 35);
        pos.p2_pieces.0 = 1 << 35;
        pos.mailbox[18] = lance;
        pos.mailbox[20] = lance;
        let ctx = CustomCtx::new(&pos);
        let c3 = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: lance };
        assert!((factor_combo(&ctx, c3) - 1.15).abs() < 1e-6, "15 pts → ×1.15");
    }

    /// Move-Attack reachability: a Guard threatens from R2 (speed 2), but a
    /// Champion/King only from R1 (speed 1). A lone enemy Champion at R2 must NOT
    /// count as a Move-Attack threat; a Guard at the same square must.
    #[test]
    fn move_attack_reach_guard_r2_champ_r1() {
        // Target square d4(27). d6(43) is exactly Chebyshev 2 away (R2, not R1).
        let body = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2);
        let mut pos = Position::empty();
        pos.p1_pieces.0 = 1 << 27; // our piece on the target square
        pos.p2_pieces.0 = 1 << 43; // one enemy at R2

        // Enemy at R2 is a CHAMPION → cannot Move-Attack from R2 (speed 1).
        pos.champions.0 = 1 << 43;
        pos.mailbox[43] = body;
        let ctx = CustomCtx::new(&pos);
        assert!(!ctx.enemy_can_move_attack(true, 27), "champ at R2 can't reach");

        // Same square, now a GUARD → can Move-Attack from R2 (speed 2).
        pos.champions.0 = 0;
        pos.guards.0 = 1 << 43;
        let ctx = CustomCtx::new(&pos);
        assert!(ctx.enemy_can_move_attack(true, 27), "guard at R2 can reach");

        // A champion at R1 (d5 = 35) CAN Move-Attack.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = 1 << 27;
        pos.p2_pieces.0 = 1 << 35; // d5, Chebyshev 1
        pos.champions.0 = 1 << 35;
        pos.mailbox[35] = body;
        let ctx = CustomCtx::new(&pos);
        assert!(ctx.enemy_can_move_attack(true, 27), "champ at R1 can reach");
    }

    /// King danger: no malus when no enemy can reach the king; a real threat that
    /// outpaces the defense produces a positive penalty (negative king score).
    #[test]
    fn king_danger_gates_and_penalises() {
        let king = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2); // hp 2, no armor
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Strike, cost 2

        // Case A: P1 king on d1(3), lone enemy king far away on h8(63) → nobody in
        // R2 and no strike in range → no danger.
        let mut pos = Position::empty();
        pos.round_number = 6; // income live, actions available
        pos.to_move = crate::state::position::Player::P2; // enemy to move
        pos.current_phase = crate::state::position::Phase::Skill;
        pos.p1_pieces.0 = 1 << 3;
        pos.p2_pieces.0 = 1 << 63;
        pos.kings.0 = (1 << 3) | (1 << 63);
        pos.mailbox[3] = king;
        pos.mailbox[63] = king;
        let ctx = CustomCtx::new(&pos);
        let kp = Piece { sq: 3, is_p1: true, kind: Kind::King, mb: king };
        assert_eq!(king_danger_malus(&ctx, kp), 0, "no enemy in reach → no danger");

        // Case B: put P2 strike champions adjacent to the king (c2, e2) so their
        // range-1 Lance actually reaches it, with plenty of money and no P1
        // defensive skill → affordable strikes outrun the king's 2 health.
        pos.p2_money = 20;
        pos.p2_pieces.0 = (1 << 63) | (1 << 10) | (1 << 12); // c2, e2 — both king-adjacent
        pos.champions.0 = (1 << 10) | (1 << 12);
        pos.mailbox[10] = lance;
        pos.mailbox[12] = lance;
        let ctx = CustomCtx::new(&pos);
        let malus = king_danger_malus(&ctx, kp);
        assert!(malus > 0, "king outnumbered with no defense → penalty, got {malus}");

        // score_king now runs the full champion chain and subtracts the malus on
        // top (the king is a champion under the hood), so it is the chain value
        // minus the danger magnitude — not the bare negation.
        let chain: f32 = CHAMP_FACTORS.iter().map(|(_, _, f)| f(&ctx, kp)).product();
        let chain_val = (BASE * chain).round() as i32;
        assert_eq!(score_king(&ctx, kp), chain_val - malus);
    }

    /// factor_exposure: 1.0 when safe (unreachable), drops as a piece becomes
    /// reachable + fragile, bottoms out near 0.1 for a 1-hp dead-to-rights piece,
    /// and bodyguard cover removes the Move-Attack vector.
    #[test]
    fn exposure_scales_with_vulnerability() {
        // Safe: a lone P1 champion far from any enemy → exposure 1.0 (no-op).
        let champ = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Lance
        let mut safe = Position::empty();
        safe.p1_pieces.0 = 1 << 0; // a1
        safe.champions.0 = 1 << 0;
        safe.to_move = crate::state::position::Player::P2;
        let ctx = CustomCtx::new(&safe);
        let c = Piece { sq: 0, is_p1: true, kind: Kind::Champion, mb: champ };
        assert!((factor_exposure(&ctx, c) - 1.0).abs() < 1e-6, "unreachable → 1.0");

        // Dead-to-rights: a 1-hp P1 champ surrounded by enemy Strike champs, enemy
        // to move (we can't heal), no retaliation → overkill incoming → deep cut
        // toward 0.1 (designer's "1-hp, no support, surrounded → ×0.1" anchor).
        let hurt = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(1); // 1hp, no armor, no skill
        let ech  = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // enemy Lance
        let mut dead = Position::empty();
        dead.round_number = 6;
        dead.to_move = crate::state::position::Player::P2; // enemy's turn → no heal
        dead.current_phase = crate::state::position::Phase::Skill;
        dead.p2_money = 20; // affords several strikes → overkill
        // P1 champ on d4(27); enemy Lance champs on c3(18),d3(19),e3(20) — all R1.
        dead.p1_pieces.0 = 1 << 27;
        dead.p2_pieces.0 = (1 << 18) | (1 << 19) | (1 << 20);
        dead.champions.0 = (1 << 27) | (1 << 18) | (1 << 19) | (1 << 20);
        dead.mailbox[27] = hurt;
        for s in [18, 19, 20] { dead.mailbox[s] = ech; }
        let ctx = CustomCtx::new(&dead);
        let c = Piece { sq: 27, is_p1: true, kind: Kind::Champion, mb: hurt };
        let ex = factor_exposure(&ctx, c);
        assert!(ex < 0.35, "1-hp surrounded by strikers, no heal → deep cut, got {ex}");

        // Bodyguard cover removes the Move-Attack vector: same 1-hp champ, but now
        // surrounded by friendly guards that intercept, and NO enemy Strike reach →
        // exposure climbs back toward safe.
        let mut covered = dead.clone();
        // Replace the enemy champs with far-away ones (out of reach), add friendly
        // guards adjacent so any approach is bodyguarded. Simplest: no enemy in reach.
        covered.p2_pieces.0 = 1 << 63; // lone far enemy
        covered.champions.0 = (1 << 27) | (1 << 63);
        covered.mailbox[63] = ech;
        let ctx = CustomCtx::new(&covered);
        let c = Piece { sq: 27, is_p1: true, kind: Kind::Champion, mb: hurt };
        assert!(factor_exposure(&ctx, c) > ex, "removing the threat lifts exposure");
    }

    /// Defense credit: Shield counts only if THIS piece carries it (self-cast);
    /// Heal/Plate count only if a DIFFERENT adjacent ally owns the skill. A piece
    /// defended by a teammate's Shield, or by an adjacent teammate that lacks
    /// Heal/Plate, gets NO defense — so it stays exposed.
    #[test]
    fn defense_requires_correct_caster() {
        let mk = |sk1: u8, sk2: u8| crate::state::EMPTY_MAILBOX_ENTRY.with_hp(1).with_skill1(sk1).with_skill2(sk2);
        // Setup: P1 1hp champ on d4(27), threatened by an adjacent P2 guard on d3(19).
        // It's P1's turn (so defense could apply). Vary the adjacent P1 ally's skills.
        let build = |victim_sk: (u8,u8), ally_sk: (u8,u8)| {
            let mut pos = Position::empty();
            pos.round_number = 5;
            pos.to_move = crate::state::position::Player::P1;
            pos.current_phase = crate::state::position::Phase::Skill;
            pos.p1_money = 10;
            // victim d4(27), ally e4(28), P2 guard d3(19), kings tucked away
            pos.p1_pieces.0 = (1 << 27) | (1 << 28) | (1 << 0);
            pos.p2_pieces.0 = (1 << 19) | (1 << 63);
            pos.champions.0 = (1 << 27) | (1 << 28);
            pos.guards.0 = 1 << 19;
            pos.kings.0 = (1 << 0) | (1 << 63);
            pos.mailbox[27] = mk(victim_sk.0, victim_sk.1);
            pos.mailbox[28] = mk(ally_sk.0, ally_sk.1);
            pos.mailbox[0] = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2);
            pos.mailbox[63] = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2);
            let ctx = CustomCtx::new(&pos);
            let v = Piece { sq: 27, is_p1: true, kind: Kind::Champion, mb: pos.mailbox[27] };
            ctx.survivability_severity(v)
        };
        // Lance=1, Heal=7, Plate=8, Shield=6, Dash=9.
        // Victim lance-only, ally lance-only → no defense → exposed (sev > 0).
        let bare = build((1, 0), (1, 0));
        assert!(bare > 0.0, "no defensive caster → exposed, got {bare}");
        // Victim carries its OWN Shield → self-defense → less exposed than bare.
        let self_shield = build((1, 6), (1, 0));
        assert!(self_shield < bare, "own Shield lowers exposure ({self_shield} vs {bare})");
        // Ally carries Heal (adjacent) → victim defended → less exposed than bare.
        let ally_heal = build((1, 0), (1, 7));
        assert!(ally_heal < bare, "adjacent Heal ally lowers exposure ({ally_heal} vs {bare})");
        // A teammate owning Shield does NOT help the victim (Shield is self-only):
        // put Shield on the ALLY, not the victim → victim stays as exposed as bare.
        let ally_shield = build((1, 0), (1, 6));
        assert!((ally_shield - bare).abs() < 1e-6, "teammate Shield must not defend ({ally_shield} vs {bare})");
    }

    /// Exposure via the STRIKE vector alone (no Move-Attack): a piece reachable only
    /// by an enemy Strike is still exposed, and bodyguard does NOT save it (skills hit
    /// direct). Certain death (lethal incoming + no possible defense) floors severity
    /// high, so the piece reads a deep cut.
    #[test]
    fn exposure_fires_on_strike_only() {
        let victim = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2); // 2hp/0armor, no skill
        let hook = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(2); // Hook range 2
        // P2 victim d5(35); P1 Hook striker d7(51) — cheby 2, clear ray, NOT adjacent
        // (no Move-Attack). Range 2 + 1 = 3 reaches. P1 to move → victim can't heal.
        let mut pos = Position::empty();
        pos.round_number = 5;
        pos.to_move = crate::state::position::Player::P1;
        pos.current_phase = crate::state::position::Phase::Skill;
        pos.p1_money = 10;
        pos.p1_pieces.0 = 1 << 51;
        pos.p2_pieces.0 = 1 << 35;
        pos.champions.0 = (1 << 51) | (1 << 35);
        pos.mailbox[51] = hook;
        pos.mailbox[35] = victim;
        let ctx = CustomCtx::new(&pos);
        let v = Piece { sq: 35, is_p1: false, kind: Kind::Champion, mb: victim };
        // No adjacency → the ONLY vector is the Strike.
        assert!(!ctx.enemy_can_move_attack(false, 35), "not Move-Attackable (dist 2)");
        assert!(ctx.survivability_severity(v) > 0.0, "strike vector alone → exposed");
        assert!(factor_exposure(&ctx, v) < 1.0, "strike threat lowers exposure factor");
    }

    /// King exposure: a king that is exactly killable next turn with NO possible
    /// defense reads certain-death (severity ≥ 0.85) → a large but CAPPED malus,
    /// nowhere near MATE_SCORE. Fixes the old unbounded −1200.
    #[test]
    fn king_exposure_certain_death_is_capped() {
        let ech = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Lance
        let kmb = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2); // 2hp/0armor king
        // P2 king d5(35) ringed by P1 Lance champs at R1 (c4,d4,e4), P1 to move, rich.
        let mut pos = Position::empty();
        pos.round_number = 5;
        pos.to_move = crate::state::position::Player::P1;
        pos.current_phase = crate::state::position::Phase::Skill;
        pos.p1_money = 20;
        let ring = (1 << 26) | (1 << 27) | (1 << 28);
        pos.p1_pieces.0 = ring;
        pos.p2_pieces.0 = 1 << 35;
        pos.champions.0 = ring;
        pos.kings.0 = 1 << 35;
        for s in [26, 27, 28] { pos.mailbox[s] = ech; }
        pos.mailbox[35] = kmb;
        let ctx = CustomCtx::new(&pos);
        let k = Piece { sq: 35, is_p1: false, kind: Kind::King, mb: kmb };
        assert!(ctx.survivability_severity(k) >= 0.85, "certain-death king → high severity");
        let malus = king_danger_malus(&ctx, k);
        assert!(malus > 3000, "near-dead king → large malus, got {malus}");
        assert!(malus < MATE_SCORE, "but strictly below mate ({malus} vs {MATE_SCORE})");
    }
}
