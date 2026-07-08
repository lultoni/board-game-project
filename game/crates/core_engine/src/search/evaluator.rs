//! Heuristic evaluation for terminal / time-out search nodes.
//!
//! Score convention: positive = P1 advantage, negative = P2 advantage.
//! Win/loss are represented as ±(MATE_SCORE - depth_to_mate) so shorter wins
//! score higher and the search prefers fast mates.
//!
//! ============================================================
//! Design philosophy (load-bearing — read before changing eval)
//! ============================================================
//!
//! Source: designer's eval-function notes (Session 28 inbox, Perplexity
//! transcript). Distilled here because the original file was deleted once
//! its content landed in code. These principles outlive the stub.
//!
//! 1. WIN/LOSS OVERRULES EVERYTHING.
//!    Captured-King = ±MATE_SCORE. Checked before any other term.
//!    Encoded as ±(MATE_SCORE - depth) so a mate-in-2 scores higher than
//!    a mate-in-5. Standard chess-engine convention — get this wrong and
//!    the engine ignores forced mates in favour of positional fluff.
//!
//! 2. "FASTEST PATH" LIVES IN THE SEARCH, NOT IN EVAL.
//!    Do NOT bake depth/tempo-to-resolution into the static evaluation.
//!    Tiebreaks between equal-eval positions are search's job (via the
//!    MATE_SCORE-depth encoding above and via move ordering). Keep eval
//!    pure: it scores the position as-is, ignoring how we got here.
//!
//! 3. AFTER WIN/LOSS: COUNT REAL THINGS.
//!    Material first (pieces, HP, armor, money, equipped skills + their
//!    follow-on possibilities). This is the baseline and MUST beat random
//!    play before anything fancier is added.
//!
//! 4. TWO ANGLES ON EVERY ADVANTAGE — TEMPO AND MONEY.
//!    For each material/positional gain, measure it both ways:
//!      - TEMPO  = how many opponent actions are required to reverse it,
//!                 assuming their best counter-line.
//!      - MONEY  = how much it costs the affected player (given their
//!                 skill flags) to undo it or to compensate for it.
//!    These two angles disagree usefully. A cheap-to-undo gain is worth
//!    less than an expensive-to-undo gain of the same material weight.
//!    Project both forward to an assumed game-end horizon — the longer
//!    the effect persists, the bigger the term.
//!
//! 5. EVAL COST IS A FIRST-CLASS BUDGET.
//!    A 10 ms eval at depth 1 loses to a 0.01 ms eval at depth 6. If the
//!    full tempo+money projection turns out too expensive, fall back to
//!    a simpler eval AND keep the complex one around; diff them on a
//!    suite of random positions to see where they disagree. That diff
//!    is what tells you which terms actually matter.
//!
//! 6. START STUPID.
//!    Material-only first. It will trounce random play and gives every
//!    later term a baseline to prove itself against. Resist the urge to
//!    ship the full tempo/money model on day one — Stockfish's eval grew
//!    over 15+ years, not in one design pass.
//!
//! Implementation order (matches slice plan, slices 7–8 and beyond):
//!   a) terminal: ±MATE_SCORE for captured King.
//!   b) material: pieces + HP + armor + money, weighted.
//!   c) skill-loadout value: equipped skills × follow-on action space.
//!   d) tempo term: opponent-actions-to-revert recent gains.
//!   e) money term: cost-to-undo recent gains.
//!   f) positional hooks (central squares, Champion–Guard adjacency for
//!      Bodyguard) — small bonuses, added last.

use crate::state::Position;
use crate::state::position::{GameResult, Player, Phase};
use crate::state::magic;
use crate::search::counters;
use crate::search::see::build_attackers_table;
use crate::game_logic::skills::{Skill, SkillCategory, skill_cost, skill_default_range, skill_category};
use crate::game_logic::make_unmake::skill_phase_budget;

pub const MATE_SCORE: i32 = 1_000_000;


// === Slice 9: material weights ===========================================
//
// Order of magnitude: one Champion >> one HP swing >> one armor swing >>
// one coin. MATE_SCORE (1_000_000) is three orders above any plausible
// material sum (~24 pieces × ~1500 each ≈ 36k), so terminals never compete.

/// King material weight = 0. The king's presence/absence is *already*
/// encoded by the MATE_SCORE branch above; counting it again would only
/// "reward" malformed positions (king missing, `game_result == None`).
const KING_MATERIAL:   i32 = 0;
const CHAMPION_VALUE:  i32 = 1000;
const GUARD_VALUE:     i32 = 600;
const HP_PER_POINT:    i32 = 150;
const ARMOR_PER_POINT: i32 = 120;
const MONEY_PER_UNIT:  i32 = 25;

// Mobility scoring (E7 rework).
// Guards: BFS-2 reachable-squares count × weight. Weight halved from Pass-4
// era (was 8) because E2/E6 now reward proper Guard placement structurally.
// Champions/Kings: replaced with skill-range coverage (count of enemies in
// range of each equipped Strike/Move-attack skill), capped per piece.
const GUARD_MOB_PER_SQ:   i32 = 4;
const CHAMP_SKILL_COV_PER_ENEMY: i32 = 10; // per enemy in range of any equipped active skill
const CHAMP_SKILL_COV_CAP:       i32 = 60; // per Champion, prevents runaway on cluttered boards
const KING_MOB_PER_SQ:    i32 = 6;         // 8-adjacent empty squares × weight, prefers not-stuck king

// E2 — exposure. Piecewise multiplier by (unshielded_attackers, piece kind).
// `unshielded = max(0, popcount(attackers_bb) - popcount(adjacent_own_guards))`.
// Applied as a percentage of the piece's own material value; scales with the
// piece we're threatening.
const EXPOSURE_MULT: [i32; 4] = [0, 10, 30, 55]; // % of piece_val (÷ 100)
// King exposure — much sharper. n_attackers indexed directly (no guard subtraction —
// King loss is game-over, so bodyguard credit is folded into E6 coverage, not here).
const KING_EXPOSURE:   [i32; 4] = [0, 800, 2400, 4000];

// E6 — bodyguard coverage. Rewards structural adjacency to own Guards.
// `coverage = shielded_empty_ring / empty_ring` (0..=256 fixed-point).
// Bonus = COVERAGE_PER_PIECE × piece_val × coverage / (100 × 256).
const COVERAGE_PER_PIECE: i32 = 30; // percent of piece_val at full coverage

// E8 — tempo. Small nudge for actions_remaining on side-to-move.
const TEMPO_PER_ACTION: i32 = 15;

// E9 — offensive-range differential. Whichever side has the higher usable
// max offensive range (across castable strikes + Shove, +1 if Focus is also
// castable) gets OFFENSIVE_RANGE_WEIGHT per point of differential. Only counted
// as a flag per side ("does this side have reach X"), not per-piece.
const OFFENSIVE_RANGE_WEIGHT: i32 = 500;

// E4 — skill availability sigmoid smoothing constant (money units).
const SKILL_AVAIL_K: i32 = 3;
const SKILL_AVAIL_MAX: i32 = 256; // fixed-point scale

// Stack M caps.
const ARMOR_CAP:          u8 = 2;
const HP_CAP:             u8 = 2;
#[allow(dead_code)]
const _CAP_NOTE: (u8, u8) = (ARMOR_CAP, HP_CAP); // kept for future callers; caps still enforced elsewhere

// PLACEHOLDER. A balance-slice will replace this table with tuned values once
// we have playtest data. The scheme — cost × 40 + range bonus + category
// bonus — keeps each skill in a sensible 50..=220 range (well under
// CHAMPION_VALUE) and orders skills roughly by their resource cost. It is
// *consistent* (deterministic), so alpha-beta will still prefer the
// objectively better of two material-equivalent positions; it is just not
// strictly correct in absolute terms.
//
// Indexed by `Skill as u8` (id 1..=15); slot 0 is the "unequipped" sentinel.
// Values precomputed from `skill_cost(s)*40 + range_bonus(s) + cat_bonus(s)`
// where range_bonus is {0→0, 1→10, 2→20, ≥3→30} and cat_bonus is
// {Strike→30, Move→20, Shield→15, Mystic→10}.
const SKILL_VALUE: [i32; 16] = [
      0, // 0  unequipped
    120, // 1  Lance   (2·40 + 10 + 30)
    170, // 2  Hook    (3·40 + 20 + 30)
    130, // 3  Break   (2·40 + 20 + 30)
    210, // 4  Steal   (4·40 + 20 + 30)
    210, // 5  Tempest (4·40 + 20 + 30)
     95, // 6  Shield  (2·40 +  0 + 15)
    145, // 7  Heal    (3·40 + 10 + 15)
    145, // 8  Plate   (3·40 + 10 + 15)
    160, // 9  Dash    (3·40 + 20 + 20)
    120, // 10 Blast   (2·40 + 20 + 20)
    170, // 11 Shove   (3·40 + 30 + 20)
    200, // 12 Swap    (4·40 + 20 + 20)
    210, // 13 Retreat (4·40 + 30 + 20)
     50, // 14 Focus   (1·40 +  0 + 10)
    130, // 15 Charge  (3·40 +  0 + 10)
];

#[inline]
#[allow(dead_code)] // kept for tests + eval_breakdown_diff callers
fn skill_value(s: Skill) -> i32 {
    SKILL_VALUE[s as usize]
}

/// King-expand: bitboard OR of all 8-directional 1-step neighbours. Used
/// for coverage / exposure computations. Duplicated from `search::see::king_expand`
/// (private there) to avoid growing that module's public surface.
#[inline]
fn king_expand(x: u64) -> u64 {
    const NOT_A: u64 = 0xfefefefefefefefe;
    const NOT_H: u64 = 0x7f7f7f7f7f7f7f7f;
    let l = (x & NOT_A) >> 1;
    let r = (x & NOT_H) << 1;
    let h = x | l | r;
    h | (h << 8) | (h >> 8)
}

/// Per-skill availability given a side's money snapshot.
/// Piecewise-linear sigmoid centred at `money - cost`: 0 when money ≪ cost,
/// 1 when money ≫ cost, ramping through `2·K` units. Output is fixed-point
/// (0..=SKILL_AVAIL_MAX = 256).
#[inline]
fn skill_availability_fp(money: i32, cost: i32) -> i32 {
    // Range [-K, +K] linearly maps to [0, SKILL_AVAIL_MAX]; clamp outside.
    let x = money - cost + SKILL_AVAIL_K;
    let denom = 2 * SKILL_AVAIL_K;
    if x <= 0 { 0 }
    else if x >= denom { SKILL_AVAIL_MAX }
    else { (x * SKILL_AVAIL_MAX) / denom }
}

/// Build a per-side [16]-entry availability lookup so the main loop pays
/// exactly one table load per skill slot instead of a sigmoid each.
#[inline]
fn side_availability_table(money: u16) -> [i32; 16] {
    let mut t = [0i32; 16];
    for id in 1u8..=15 {
        if let Some(s) = crate::game_logic::skills::skill_from_id(id) {
            t[id as usize] = skill_availability_fp(money as i32, skill_cost(s) as i32);
        }
    }
    t
}

/// Max cost across a side's equipped skills. 0 during Draft (no skills owned)
/// or if the side has no non-zero skill slots yet. Used for the E3 money cap.
#[inline]
fn max_owned_skill_cost(pos: &Position, side_bb: u64) -> u8 {
    let mut bits = side_bb;
    let mut best = 0u8;
    while bits != 0 {
        let sq = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        let m = pos.mailbox[sq];
        for id in [m.skill1(), m.skill2()] {
            if let Some(s) = crate::game_logic::skills::skill_from_id(id) {
                let c = skill_cost(s);
                if c > best { best = c; }
            }
        }
    }
    best
}

/// Skill actions per round for the current round. Move actions cost no money
/// so are excluded from the useful-money cap. Draft returns 0 (no combat
/// action budget yet, and skills aren't owned).
#[inline]
fn actions_per_round(phase: Phase, round_number: u16) -> u8 {
    match phase {
        Phase::Draft => 0,
        _            => skill_phase_budget(round_number),
    }
}

/// E3 — money value with diminishing returns capped at
/// `max_owned_skill_cost × actions_per_round`. Cap 0 → 0 (correct pre-draft).
#[inline]
fn useful_money(money: u16, cap: u16) -> i32 {
    if cap == 0 { return 0; }
    let m = money as i64;
    let c = cap as i64;
    let mpu = MONEY_PER_UNIT as i64;
    let value = if m <= c {
        // MONEY_PER_UNIT × m × (1 − m/(2c)) = MONEY_PER_UNIT × m × (2c − m) / (2c)
        (mpu * m * (2 * c - m)) / (2 * c)
    } else {
        (mpu * c) / 2
    };
    value as i32
}

/// E9 — side's max offensive range across castable strike + Shove skills.
/// "Castable" here = piece alive + skill equipped + side has enough money for
/// the skill's cost. Focus availability grants +1 to non-Mystic offensives when
/// the side can also afford Focus (Focus costs 1) — because Focus buffs range
/// for the next skill action, the +1 only matters when the side has enough
/// money to also cast the buffed skill AND the actions-per-round supports two
/// skill actions this round.
///
/// Only counted once per side ("does this side have reach X"), not per-piece.
/// Returns 0 during Draft (no skills owned) or when money can't afford the
/// cheapest offensive skill (Lance @ 2).
fn max_offensive_range(pos: &Position, side_bb: u64, money: u16) -> u8 {
    if pos.current_phase == Phase::Draft { return 0; }
    if money < 2 { return 0; } // cheapest offensive is Lance@2

    let actions = actions_per_round(pos.current_phase, pos.round_number);
    let focus_bonus_possible = actions >= 2; // needs Focus + a follow-up cast

    // Single pass: track raw best (affordable at current money) and boosted best
    // (affordable at money − 1, i.e. after paying Focus). Also track whether the
    // side owns a Focus anywhere. The final +1 is applied only if Focus is
    // castable AND owned AND the +1 actually beats the raw best.
    //
    // Upper bound: highest offensive range in Stack M is Shove=3. With Focus +1,
    // best possible boosted result = 4. Short-circuit when we hit that.
    let mut raw_best: u8 = 0;
    let mut boosted_best: u8 = 0;
    let mut owns_focus = false;
    let focus_reserve = if focus_bonus_possible { 1u16 } else { u16::MAX };

    let mut bits = side_bb;
    while bits != 0 {
        let sq = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        let m = pos.mailbox[sq];
        let ids = [m.skill1(), m.skill2()];
        for id in ids {
            let Some(s) = crate::game_logic::skills::skill_from_id(id) else { continue };
            if matches!(s, Skill::Focus) {
                owns_focus = true;
                continue;
            }
            let is_offensive = matches!(skill_category(s), SkillCategory::Strike)
                || matches!(s, Skill::Shove);
            if !is_offensive { continue; }
            let cost = skill_cost(s) as u16;
            let range = skill_default_range(s);
            if money >= cost && range > raw_best { raw_best = range; }
            if focus_bonus_possible
                && money >= cost.saturating_add(focus_reserve)
                && range > boosted_best
            {
                boosted_best = range;
            }
        }
        // Early exit: once raw hits 3 (Shove) and boosted hits 3 (which becomes 4
        // after +1), we cannot improve further.
        if raw_best >= 3 && boosted_best >= 3 && owns_focus { break; }
    }

    if focus_bonus_possible && owns_focus && boosted_best > 0 {
        (boosted_best + 1).max(raw_best)
    } else {
        raw_best
    }
}

/// Per-component decomposition of the static eval. `total` is exactly what
/// `evaluate()` returns (so L3 sees zero behaviour change). The per-bucket
/// fields are sign-corrected: P1 contributions go to `*_p1`, P2 to `*_p2`,
/// both as positive magnitudes. `total = sum(*_p1) - sum(*_p2)` (terminal
/// short-circuit aside).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvalBreakdown {
    pub material_p1:  i32,
    pub material_p2:  i32,
    pub hp_p1:        i32,
    pub hp_p2:        i32,
    pub armor_p1:     i32,
    pub armor_p2:     i32,
    pub skills_p1:    i32,
    pub skills_p2:    i32,
    pub money_p1:     i32,
    pub money_p2:     i32,
    pub mobility_p1:  i32,
    pub mobility_p2:  i32,
    /// Threat term — REMOVED in Pass 4 (MAEE deleted from eval). Always 0.
    /// Kept for frontend/bench schema compat.
    pub threat_p1:    i32,
    pub threat_p2:    i32,
    /// Skill-activity — REMOVED in Pass 4++ (E5). Always 0.
    /// Kept for frontend/bench schema compat.
    pub skill_act_p1: i32,
    pub skill_act_p2: i32,
    /// E2 — exposure penalty (positive magnitude in the *_p1/*_p2 fields;
    /// subtracts in the total). Reflects own pieces attackable by opponent.
    pub exposure_p1:  i32,
    pub exposure_p2:  i32,
    /// E6 — bodyguard coverage bonus. Structural adjacency to own Guards.
    pub coverage_p1:  i32,
    pub coverage_p2:  i32,
    /// E8 — tempo bonus. Small nudge for actions_remaining on side-to-move.
    pub tempo_p1:     i32,
    pub tempo_p2:     i32,
    /// E9 — offensive-range flag (raw max range, e.g. 2..=4). Signed with
    /// OFFENSIVE_RANGE_WEIGHT in `total`. See const doc.
    pub offensive_range_p1: i32,
    pub offensive_range_p2: i32,
    pub total:        i32,
}

/// Per-square view of the eval, for diagnostic UI. One record per board
/// square (0..=63). Empty squares carry `occupied=false` and zeros elsewhere;
/// occupied squares carry the full per-piece contribution to `evaluate()` plus
/// intermediate values (attacker counts, skill availabilities, mobility raw)
/// so the hover popup can explain *why* a piece scores the way it does.
///
/// Sign convention: all `*_term` fields are the piece's own contribution as
/// a positive magnitude (owner-relative). To reconstruct `EvalBreakdown.total`:
/// P1 pieces contribute `+piece_total`, P2 pieces `-piece_total`, then add
/// side-level money/tempo differences. `SquareBreakdown::owner_signed_total()`
/// applies the sign.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SquareBreakdown {
    pub sq:            u8,
    pub occupied:      bool,
    pub is_p1:         bool,
    /// 0 = empty, 1 = guard, 2 = champion, 3 = king.
    pub piece_kind:    u8,
    pub hp:            u8,
    pub armor:         u8,
    pub skill1_id:     u8,
    pub skill2_id:     u8,

    // Eval components — magnitudes (owner-relative).
    pub material:      i32,
    pub hp_term:       i32,
    pub armor_term:    i32,
    pub skills_term:   i32,
    pub mobility_term: i32,
    /// Positive magnitude; subtracted from owner side in the game total.
    pub exposure_term: i32,
    pub coverage_term: i32,
    /// material + hp + armor + skills + mobility + coverage - exposure.
    /// The signed sum for the owning side.
    pub piece_total:   i32,

    // Intermediate raw values for the popup.
    /// 0..=SKILL_AVAIL_MAX (256). Fixed-point availability at owner's money.
    pub skill1_avail_fp: i32,
    pub skill2_avail_fp: i32,
    /// Enemy attackers threatening this square.
    pub n_attackers:   u8,
    /// Own guards on the 8-ring around this square.
    pub n_adj_guards:  u8,
    /// Guard: BFS-2 reachable target count. King: adjacent free tiles.
    /// Champion: enemies-in-skill-range count (summed over equipped Strike-ish skills).
    pub mobility_raw:  u16,
    /// 8-ring squares that are empty (denominator for coverage).
    pub empty_ring_total:    u8,
    /// 8-ring empty squares also within king_expand(own_guards).
    pub empty_ring_shielded: u8,
}

impl SquareBreakdown {
    /// This piece's signed contribution to the game total.
    /// P1 contributes `+piece_total`, P2 contributes `-piece_total`.
    #[inline]
    pub fn owner_signed_total(&self) -> i32 {
        if !self.occupied { 0 }
        else if self.is_p1 { self.piece_total }
        else { -self.piece_total }
    }
}

/// Full per-square eval decomposition. Sum of per-square owner-signed totals
/// plus side-level money/tempo differences equals `EvalBreakdown.total`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvalBreakdownBySquare {
    /// One entry per board square, indexed by square number.
    pub squares:         Vec<SquareBreakdown>,

    pub p1_money:        u16,
    pub p2_money:        u16,
    /// `max_owned_skill_cost × actions_per_round`; 0 during Draft or if the
    /// side owns no skills.
    pub p1_money_cap:    u16,
    pub p2_money_cap:    u16,
    /// `useful_money(pos.p1_money, p1_money_cap)`.
    pub p1_money_term:   i32,
    pub p2_money_term:   i32,
    /// `TEMPO_PER_ACTION × actions_remaining` for side-to-move (outside Draft).
    pub p1_tempo_term:   i32,
    pub p2_tempo_term:   i32,

    /// Same as `EvalBreakdown.total` for the same `Position`.
    pub total:           i32,
    /// True when the position was terminal — total is ±MATE_SCORE and per-piece
    /// values are all zero (short-circuited).
    pub terminal:        bool,
}

impl Default for EvalBreakdownBySquare {
    fn default() -> Self {
        Self {
            squares: vec![SquareBreakdown::default(); 64],
            p1_money: 0,
            p2_money: 0,
            p1_money_cap: 0,
            p2_money_cap: 0,
            p1_money_term: 0,
            p2_money_term: 0,
            p1_tempo_term: 0,
            p2_tempo_term: 0,
            total: 0,
            terminal: false,
        }
    }
}

pub fn evaluate(pos: &Position) -> i32 {
    evaluate_breakdown(pos).total
}

/// Position-rater interface. The search calls `evaluate` once per leaf; an
/// `Evaluator` impl returns a P1-POV score in the same units as the free
/// `evaluate()` function above (positive = P1, ±MATE_SCORE for terminals).
///
/// Two impls are planned: `HeuristicEvaluator` wraps today's hand-coded eval
/// (zero-behaviour-change default); a future `NnEvaluator` will host the
/// trained position rater (`design/inbox/digital/nn-rater-plan.md`).
///
/// **Send-only** bound: the search itself is single-threaded but evaluators
/// are owned by `Match` (one per AI seat), which lives on a worker thread
/// and gets moved between thread-pool tasks via `tauri::async_runtime`.
/// Code that needs to share an evaluator across threads (e.g. the tier-2
/// gauntlet's predecessor list) re-asserts `+ Sync` locally.
pub trait Evaluator: Send {
    fn evaluate(&self, pos: &Position) -> i32;
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown;
}

/// Zero-size wrapper around the free `evaluate()` / `evaluate_breakdown()`
/// functions. The default evaluator everywhere — preserves S36 behaviour.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicEvaluator;

impl Evaluator for HeuristicEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 { evaluate(pos) }
    #[inline]
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown { evaluate_breakdown(pos) }
}

pub fn evaluate_breakdown(pos: &Position) -> EvalBreakdown {
    counters::bump_eval_calls();
    // (a) Terminal — overrules everything. Per-bucket fields stay zero;
    //     only `total` carries the ±MATE_SCORE.
    match pos.game_result {
        Some(GameResult::P1Wins) => return EvalBreakdown { total:  MATE_SCORE, ..Default::default() },
        Some(GameResult::P2Wins) => return EvalBreakdown { total: -MATE_SCORE, ..Default::default() },
        None => {}
    }

    let mut b = EvalBreakdown::default();

    let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
    let p1_bb = pos.p1_pieces.0;
    let p2_bb = pos.p2_pieces.0;
    let p1_guards = p1_bb & pos.guards.0;
    let p2_guards = p2_bb & pos.guards.0;

    // E4 — per-side skill availability lookup (16 entries × 2 sides).
    let p1_avail = side_availability_table(pos.p1_money);
    let p2_avail = side_availability_table(pos.p2_money);

    // Attackers table (shared by E2). Built once per eval.
    let atk = build_attackers_table(pos, all_occ);

    // (b) Single pass over occupied bits.
    let mut bits = all_occ;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let m = pos.mailbox[sq as usize];
        let is_p1 = p1_bb & mask != 0;

        let is_guard    = pos.guards.0    & mask != 0;
        let is_king     = pos.kings.0     & mask != 0;

        let material =
            if      is_king                     { KING_MATERIAL }
            else if pos.champions.0 & mask != 0 { CHAMPION_VALUE }
            else                                { GUARD_VALUE };
        let hp_term    = HP_PER_POINT    * m.hp()    as i32;
        let armor_term = ARMOR_PER_POINT * m.armor() as i32;

        // E4 — skill term gated by availability.
        let avail = if is_p1 { &p1_avail } else { &p2_avail };
        let sid1 = m.skill1() as usize;
        let sid2 = m.skill2() as usize;
        let sk1_base = if sid1 < SKILL_VALUE.len() { SKILL_VALUE[sid1] } else { 0 };
        let sk2_base = if sid2 < SKILL_VALUE.len() { SKILL_VALUE[sid2] } else { 0 };
        let sk1_a = if sid1 < avail.len() { avail[sid1] } else { 0 };
        let sk2_a = if sid2 < avail.len() { avail[sid2] } else { 0 };
        let skill_term = (sk1_base * sk1_a + sk2_base * sk2_a) / SKILL_AVAIL_MAX;

        // Mobility (E7 rework).
        let own_bb = if is_p1 { p1_bb } else { p2_bb };
        let opp_bb = if is_p1 { p2_bb } else { p1_bb };
        let mob_score = if is_guard {
            magic::movement_targets_speed2(sq, all_occ).0.count_ones() as i32
                * GUARD_MOB_PER_SQ
        } else if is_king {
            // King prefers open escape squares.
            (magic::movement_targets_speed1(sq).0 & !own_bb).count_ones() as i32
                * KING_MOB_PER_SQ
        } else {
            // Champion: skill-range coverage — count enemies reachable by any
            // equipped Strike-ish active skill (Ally-target skills contribute 0
            // by design; Empty-target Dash/Retreat also 0). Cap per piece.
            let mut cov = 0i32;
            for sid in [m.skill1(), m.skill2()] {
                let Some(sk) = crate::game_logic::skills::skill_from_id(sid) else { continue };
                let owner = crate::game_logic::skills::skill_target_owner(sk);
                use crate::game_logic::skills::TargetOwner;
                if !matches!(owner, TargetOwner::Enemy | TargetOwner::Either) { continue; }
                let range = skill_default_range(sk);
                let ray = magic::skill_attacks(sq, all_occ, range).0;
                cov += (ray & opp_bb).count_ones() as i32 * CHAMP_SKILL_COV_PER_ENEMY;
            }
            cov.min(CHAMP_SKILL_COV_CAP)
        };

        // E2 — exposure. Attacker count from `atk`, minus adjacent own guards.
        // Kings use their own escalation curve; the enemy king can't attack, so
        // atk table already reflects only Champions/Guards.
        let opp_attackers_bb = if is_p1 { atk.any_attackers_of(Player::P2, sq) }
                               else     { atk.any_attackers_of(Player::P1, sq) };
        let n_attackers = opp_attackers_bb.count_ones() as usize;
        let exposure_term = if is_king {
            let idx = n_attackers.min(3);
            KING_EXPOSURE[idx]
        } else {
            let own_guards = if is_p1 { p1_guards } else { p2_guards };
            let n_adj_guards = (king_expand(mask) & own_guards).count_ones() as usize;
            let unshielded = n_attackers.saturating_sub(n_adj_guards).min(3);
            let mult_pct = EXPOSURE_MULT[unshielded];
            let piece_val = if pos.champions.0 & mask != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
            (piece_val * mult_pct) / 100
        };

        // E6 — bodyguard coverage (Champions + Kings only; Guards are the shield).
        // An empty ring-square `s` around defender `d` is "shielded" iff a
        // friendly Guard sits adjacent to BOTH `s` and `d` — that is exactly
        // the Bodyguard trigger rule (see generator::bodyguard_guards_for).
        // Just being adjacent to *some* guard nearby is not enough.
        let coverage_term = if is_guard {
            0
        } else {
            let own_guards = if is_p1 { p1_guards } else { p2_guards };
            let defender_neighbours = king_expand(mask) & !mask;
            let empty_ring = defender_neighbours & !all_occ;
            let denom = empty_ring.count_ones() as i32;
            let mut shielded = 0i32;
            let mut ring_bits = empty_ring;
            while ring_bits != 0 {
                let s = ring_bits.trailing_zeros();
                ring_bits &= ring_bits - 1;
                let s_bit = 1u64 << s;
                // Dual-adjacency: Guard neighbouring both s and d.
                let dual_neigh = king_expand(s_bit) & defender_neighbours & own_guards;
                if dual_neigh != 0 { shielded += 1; }
            }
            let coverage_fp = if denom == 0 { SKILL_AVAIL_MAX } else { (shielded * SKILL_AVAIL_MAX) / denom };
            let piece_val = CHAMPION_VALUE; // king shielded ≈ champion-scale
            (COVERAGE_PER_PIECE * piece_val * coverage_fp) / (100 * SKILL_AVAIL_MAX)
        };

        if is_p1 {
            b.material_p1  += material;
            b.hp_p1        += hp_term;
            b.armor_p1     += armor_term;
            b.skills_p1    += skill_term;
            b.mobility_p1  += mob_score;
            b.exposure_p1  += exposure_term;
            b.coverage_p1  += coverage_term;
        } else {
            b.material_p2  += material;
            b.hp_p2        += hp_term;
            b.armor_p2     += armor_term;
            b.skills_p2    += skill_term;
            b.mobility_p2  += mob_score;
            b.exposure_p2  += exposure_term;
            b.coverage_p2  += coverage_term;
        }
    }

    // (c) E3 — money with diminishing-return cap.
    let p1_max_cost = max_owned_skill_cost(pos, p1_bb);
    let p2_max_cost = max_owned_skill_cost(pos, p2_bb);
    let actions = actions_per_round(pos.current_phase, pos.round_number);
    let p1_cap = p1_max_cost as u16 * actions as u16;
    let p2_cap = p2_max_cost as u16 * actions as u16;
    b.money_p1 = useful_money(pos.p1_money, p1_cap);
    b.money_p2 = useful_money(pos.p2_money, p2_cap);

    // (d) E8 — tempo. Only side-to-move (outside Draft).
    if pos.current_phase != Phase::Draft {
        let tempo = TEMPO_PER_ACTION * pos.actions_remaining as i32;
        match pos.to_move {
            Player::P1 => b.tempo_p1 = tempo,
            Player::P2 => b.tempo_p2 = tempo,
        }
    }

    // (d2) E9 — offensive-range flag.
    b.offensive_range_p1 = max_offensive_range(pos, p1_bb, pos.p1_money) as i32;
    b.offensive_range_p2 = max_offensive_range(pos, p2_bb, pos.p2_money) as i32;

    // (e) Legacy fields — zeroed for schema compat.
    b.threat_p1 = 0;
    b.threat_p2 = 0;
    b.skill_act_p1 = 0;
    b.skill_act_p2 = 0;

    if pos.actions_remaining == 0 {
        counters::bump_actions_zero_hit();
    }

    b.total =
        (b.material_p1 - b.material_p2) +
        (b.hp_p1       - b.hp_p2)       +
        (b.armor_p1    - b.armor_p2)    +
        (b.skills_p1   - b.skills_p2)   +
        (b.money_p1    - b.money_p2)    +
        (b.mobility_p1 - b.mobility_p2) +
        (b.coverage_p1 - b.coverage_p2) -
        (b.exposure_p1 - b.exposure_p2) +
        (b.tempo_p1    - b.tempo_p2)    +
        (b.offensive_range_p1 - b.offensive_range_p2) * OFFENSIVE_RANGE_WEIGHT;
    // NOTE: threat_* and skill_act_* are always 0; omitted from total.
    // Suppress unused-variable warnings while we still reference `atk` above.
    let _ = &atk;
    b
}

/// Diagnostic entry point — same math as `evaluate_breakdown` but emits one
/// record per board square instead of aggregating by term. Used by the
/// frontend's per-square hover popup. Not called from search.
///
/// Invariant: for any position, `evaluate_by_square(pos).total == evaluate_breakdown(pos).total`.
/// The `evaluate_by_square_matches_breakdown` unit test enforces this.
pub fn evaluate_by_square(pos: &Position) -> EvalBreakdownBySquare {
    let mut out = EvalBreakdownBySquare::default();
    for sq in 0..64u8 {
        out.squares[sq as usize].sq = sq;
    }

    // Terminal short-circuit — mirror evaluate_breakdown's behaviour.
    match pos.game_result {
        Some(GameResult::P1Wins) => {
            out.total = MATE_SCORE;
            out.terminal = true;
            return out;
        }
        Some(GameResult::P2Wins) => {
            out.total = -MATE_SCORE;
            out.terminal = true;
            return out;
        }
        None => {}
    }

    let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
    let p1_bb = pos.p1_pieces.0;
    let p2_bb = pos.p2_pieces.0;
    let p1_guards = p1_bb & pos.guards.0;
    let p2_guards = p2_bb & pos.guards.0;

    let p1_avail = side_availability_table(pos.p1_money);
    let p2_avail = side_availability_table(pos.p2_money);

    let atk = build_attackers_table(pos, all_occ);

    // Per-square loop — same term math as evaluate_breakdown, but records per square.
    let mut sum_p1 = 0i32;
    let mut sum_p2 = 0i32;
    let mut bits = all_occ;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let m = pos.mailbox[sq as usize];
        let is_p1 = p1_bb & mask != 0;

        let is_guard    = pos.guards.0    & mask != 0;
        let is_king     = pos.kings.0     & mask != 0;
        let is_champion = pos.champions.0 & mask != 0;

        let piece_kind: u8 =
            if is_king      { 3 }
            else if is_champion { 2 }
            else            { 1 };

        let material =
            if      is_king     { KING_MATERIAL }
            else if is_champion { CHAMPION_VALUE }
            else                { GUARD_VALUE };
        let hp_term    = HP_PER_POINT    * m.hp()    as i32;
        let armor_term = ARMOR_PER_POINT * m.armor() as i32;

        let avail = if is_p1 { &p1_avail } else { &p2_avail };
        let sid1 = m.skill1() as usize;
        let sid2 = m.skill2() as usize;
        let sk1_base = if sid1 < SKILL_VALUE.len() { SKILL_VALUE[sid1] } else { 0 };
        let sk2_base = if sid2 < SKILL_VALUE.len() { SKILL_VALUE[sid2] } else { 0 };
        let sk1_a = if sid1 < avail.len() { avail[sid1] } else { 0 };
        let sk2_a = if sid2 < avail.len() { avail[sid2] } else { 0 };
        let skills_term = (sk1_base * sk1_a + sk2_base * sk2_a) / SKILL_AVAIL_MAX;

        let own_bb = if is_p1 { p1_bb } else { p2_bb };
        let opp_bb = if is_p1 { p2_bb } else { p1_bb };

        let (mob_score, mob_raw): (i32, u16) = if is_guard {
            let raw = magic::movement_targets_speed2(sq, all_occ).0.count_ones();
            (raw as i32 * GUARD_MOB_PER_SQ, raw as u16)
        } else if is_king {
            let raw = (magic::movement_targets_speed1(sq).0 & !own_bb).count_ones();
            (raw as i32 * KING_MOB_PER_SQ, raw as u16)
        } else {
            let mut cov = 0i32;
            let mut raw_enemies = 0u32;
            for sid in [m.skill1(), m.skill2()] {
                let Some(sk) = crate::game_logic::skills::skill_from_id(sid) else { continue };
                let owner = crate::game_logic::skills::skill_target_owner(sk);
                use crate::game_logic::skills::TargetOwner;
                if !matches!(owner, TargetOwner::Enemy | TargetOwner::Either) { continue; }
                let range = skill_default_range(sk);
                let ray = magic::skill_attacks(sq, all_occ, range).0;
                let hits = (ray & opp_bb).count_ones();
                raw_enemies += hits;
                cov += hits as i32 * CHAMP_SKILL_COV_PER_ENEMY;
            }
            (cov.min(CHAMP_SKILL_COV_CAP), raw_enemies as u16)
        };

        let opp_attackers_bb = if is_p1 { atk.any_attackers_of(Player::P2, sq) }
                               else     { atk.any_attackers_of(Player::P1, sq) };
        let n_attackers = opp_attackers_bb.count_ones();
        let own_guards = if is_p1 { p1_guards } else { p2_guards };
        let n_adj_guards = (king_expand(mask) & own_guards).count_ones();

        let exposure_term = if is_king {
            let idx = (n_attackers as usize).min(3);
            KING_EXPOSURE[idx]
        } else {
            let unshielded = (n_attackers.saturating_sub(n_adj_guards) as usize).min(3);
            let mult_pct = EXPOSURE_MULT[unshielded];
            let piece_val = if is_champion { CHAMPION_VALUE } else { GUARD_VALUE };
            (piece_val * mult_pct) / 100
        };

        let (coverage_term, empty_ring_total, empty_ring_shielded): (i32, u8, u8) = if is_guard {
            (0, 0, 0)
        } else {
            let defender_neighbours = king_expand(mask) & !mask;
            let empty_ring = defender_neighbours & !all_occ;
            let denom = empty_ring.count_ones();
            let mut shielded: u32 = 0;
            let mut ring_bits = empty_ring;
            while ring_bits != 0 {
                let s = ring_bits.trailing_zeros();
                ring_bits &= ring_bits - 1;
                let s_bit = 1u64 << s;
                if king_expand(s_bit) & defender_neighbours & own_guards != 0 {
                    shielded += 1;
                }
            }
            let coverage_fp = if denom == 0 { SKILL_AVAIL_MAX } else { (shielded as i32 * SKILL_AVAIL_MAX) / denom as i32 };
            let piece_val = CHAMPION_VALUE; // king shielded ≈ champion-scale
            (
                (COVERAGE_PER_PIECE * piece_val * coverage_fp) / (100 * SKILL_AVAIL_MAX),
                denom as u8,
                shielded as u8,
            )
        };

        let piece_total = material + hp_term + armor_term + skills_term
            + mob_score + coverage_term - exposure_term;

        if is_p1 { sum_p1 += piece_total; } else { sum_p2 += piece_total; }

        let s = &mut out.squares[sq as usize];
        s.sq = sq;
        s.occupied = true;
        s.is_p1 = is_p1;
        s.piece_kind = piece_kind;
        s.hp = m.hp();
        s.armor = m.armor();
        s.skill1_id = m.skill1();
        s.skill2_id = m.skill2();
        s.material = material;
        s.hp_term = hp_term;
        s.armor_term = armor_term;
        s.skills_term = skills_term;
        s.mobility_term = mob_score;
        s.exposure_term = exposure_term;
        s.coverage_term = coverage_term;
        s.piece_total = piece_total;
        s.skill1_avail_fp = sk1_a;
        s.skill2_avail_fp = sk2_a;
        s.n_attackers = n_attackers as u8;
        s.n_adj_guards = n_adj_guards as u8;
        s.mobility_raw = mob_raw;
        s.empty_ring_total = empty_ring_total;
        s.empty_ring_shielded = empty_ring_shielded;
    }

    // Side-level: money + tempo, mirroring evaluate_breakdown's post-loop stage.
    let p1_max_cost = max_owned_skill_cost(pos, p1_bb);
    let p2_max_cost = max_owned_skill_cost(pos, p2_bb);
    let actions = actions_per_round(pos.current_phase, pos.round_number);
    let p1_cap = p1_max_cost as u16 * actions as u16;
    let p2_cap = p2_max_cost as u16 * actions as u16;
    out.p1_money = pos.p1_money;
    out.p2_money = pos.p2_money;
    out.p1_money_cap = p1_cap;
    out.p2_money_cap = p2_cap;
    out.p1_money_term = useful_money(pos.p1_money, p1_cap);
    out.p2_money_term = useful_money(pos.p2_money, p2_cap);

    if pos.current_phase != Phase::Draft {
        let tempo = TEMPO_PER_ACTION * pos.actions_remaining as i32;
        match pos.to_move {
            Player::P1 => out.p1_tempo_term = tempo,
            Player::P2 => out.p2_tempo_term = tempo,
        }
    }

    // E9 — offensive-range flag. Only enters `total`, not stored on the
    // per-square view (side-level term, no per-square attribution).
    let off_p1 = max_offensive_range(pos, p1_bb, pos.p1_money) as i32;
    let off_p2 = max_offensive_range(pos, p2_bb, pos.p2_money) as i32;

    out.total = sum_p1 - sum_p2
        + (out.p1_money_term - out.p2_money_term)
        + (out.p1_tempo_term - out.p2_tempo_term)
        + (off_p1 - off_p2) * OFFENSIVE_RANGE_WEIGHT;
    out
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{GameResult, Player};

    /// Place a piece on `sq` for `player` of `kind` (0=King, 1=Champion, 2=Guard)
    /// with mailbox `entry`. Mirrors the structure of `make_unmake::tests::place`
    /// (which is pub(super)-scoped and not reachable from here).
    fn place(p: &mut Position, sq: u8, player: Player, kind: u8, entry: MailboxEntry) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        match kind {
            0 => p.kings     = p.kings     | bit,
            1 => p.champions = p.champions | bit,
            _ => p.guards    = p.guards    | bit,
        }
        p.mailbox[sq as usize] = entry;
    }

    #[test]
    fn empty_board_is_zero() {
        let pos = Position::empty();
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn terminal_p1_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn terminal_overrules_material() {
        // Place a P2 Champion (which would give P1 a negative material score)
        // but set game_result = P1Wins. Terminal must short-circuit the loop
        // and return exactly +MATE_SCORE.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn mirrored_single_champion_is_zero() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn hp_differential() {
        // P1 Champion HP=2 vs P2 Champion HP=1, no armor, no skills.
        // Differential is exactly HP_PER_POINT.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));
        assert_eq!(evaluate(&pos), HP_PER_POINT);
    }

    #[test]
    fn armor_differential() {
        // P1 Champion armor=1 vs P2 Champion armor=0, identical otherwise.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2).with_armor(0));
        assert_eq!(evaluate(&pos), ARMOR_PER_POINT);
    }

    #[test]
    fn money_differential() {
        // E3: money value requires owned skills (cap = max_skill_cost × actions).
        // Give both sides an identical piece so both have a skill_cost baseline,
        // but different money. Differential must be positive when P1 has more.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        pos.p1_money = 10;
        pos.p2_money = 4;
        pos.actions_remaining = 2;
        pos.current_phase = Phase::Move;
        // With cap = 2 (Lance cost) × 2 (actions) = 4, both sides plateau at
        // MONEY_PER_UNIT × cap / 2 = 50 each — but P2 is under cap so gets less
        // than the plateau. P1 differential should be non-negative and small.
        // The load-bearing invariant: score should be non-negative and reflect
        // P1's money advantage.
        assert!(evaluate(&pos) >= 0, "P1 money advantage should not be negative");
    }

    #[test]
    fn money_symmetric_when_equal() {
        // Two symmetric pieces + equal money → material terms cancel.
        // With actions_remaining=0 the tempo term (E8) is also 0, so total=0.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        pos.p1_money = 5;
        pos.p2_money = 5;
        pos.actions_remaining = 0;
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn skill_equipped_beats_unequipped() {
        // P1 Champion with Lance equipped vs P2 Champion bare. Assert P1 > P2
        // (skill contributes positively) rather than the raw skill_value
        // (E4 gates by availability, so exact value depends on money).
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2));
        // Money high enough that Lance availability saturates.
        pos.p1_money = 20;
        pos.p2_money = 20;
        pos.actions_remaining = 2;
        assert!(evaluate(&pos) > 0);
    }

    #[test]
    fn stack_m_setup_is_zero() {
        // Canonical start: identical material on both sides, 6 money each.
        // Under E8 (tempo), P1-to-move contributes +TEMPO_PER_ACTION * 2 = +30;
        // material/skills/money/mobility/exposure/coverage are perfectly mirrored
        // and cancel. So the invariant is: score == tempo bonus for the moving side.
        let pos = Position::setup_stack_m();
        assert_eq!(evaluate(&pos), TEMPO_PER_ACTION * pos.actions_remaining as i32);
    }

    #[test]
    fn sign_convention_p1_positive_p2_negative() {
        // A lone P1 Champion → positive score.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) > 0);

        // Symmetric: a lone P2 Champion → negative.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) < 0);
    }

    #[test]
    fn additivity() {
        // Build three positions:
        //   A: P1 +1 HP advantage (P1 HP=2, P2 HP=1, no armor)
        //   B: P1 +1 armor advantage (HP=2 both, P1 armor=1, P2 armor=0)
        //   AB: both effects combined
        // Assert evaluate(AB) == evaluate(A) + evaluate(B).
        let mut a = Position::empty();
        place(&mut a, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut a, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        let mut b = Position::empty();
        place(&mut b, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut b, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));

        let mut ab = Position::empty();
        place(&mut ab, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut ab, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        assert_eq!(evaluate(&ab), evaluate(&a) + evaluate(&b));
    }

    #[test]
    fn maxed_piece_formula() {
        // Pin the math for a lone P1 Champion HP=2 armor=2 skills=Tempest+Charge,
        // no enemies, no money. Under E2..E8:
        //   - mobility (E7) uses skill-range coverage; 0 enemies → mob=0
        //   - skill_term (E4) gated by money=0; both Tempest (cost 4) and Charge (cost 3)
        //     have money-cost+K ≤ 0 → availability=0 → skill_term=0
        //   - exposure (E2) = 0 (no attackers)
        //   - coverage (E6) = 0 (no adjacent guards)
        //   - tempo (E8) skipped in Draft phase; empty position has no side_to_move
        //     effect on tempo either — Draft.
        // Result: pure material + hp + armor.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default()
                .with_hp(2)
                .with_armor(2)
                .with_skill1(Skill::Tempest as u8)
                .with_skill2(Skill::Charge as u8));
        let expected = CHAMPION_VALUE
            + 2 * HP_PER_POINT
            + 2 * ARMOR_PER_POINT;
        assert_eq!(evaluate(&pos), expected);
    }

    #[test]
    fn asymmetric_kings_no_panic() {
        // Malformed: P2 has a king, P1 doesn't, but game_result is None.
        // Eval must return a finite i32 without panicking.
        let mut pos = Position::empty();
        place(&mut pos, 4, Player::P2, 0, MailboxEntry::default().with_hp(2));
        // game_result stays None.
        let s = evaluate(&pos);
        // We don't assert a specific value — just that it computed.
        // KING_MATERIAL is 0, so the king contributes only its HP. P1 has nothing.
        // The point is: no panic, no overflow.
        assert!(s > i32::MIN && s < i32::MAX);
    }

    #[test]
    fn evaluate_by_square_matches_breakdown_stack_m() {
        // Invariant: the per-square view must sum to the same total as the
        // aggregate breakdown for the canonical Stack M opening position.
        let pos = Position::setup_stack_m();
        let bd = evaluate_breakdown(&pos);
        let bs = evaluate_by_square(&pos);
        assert_eq!(bd.total, bs.total, "total mismatch");
    }

    #[test]
    fn evaluate_by_square_matches_breakdown_asymmetric() {
        // A P1 Champion adjacent to a P2 Guard, no other pieces — exercises
        // exposure, coverage, mobility on both sides.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_armor(1)
                .with_skill1(crate::game_logic::skills::Skill::Lance as u8));
        place(&mut pos, 29, Player::P2, 2, MailboxEntry::default().with_hp(1));
        pos.p1_money = 4;
        pos.p2_money = 2;
        pos.actions_remaining = 2;
        pos.current_phase = Phase::Move;
        pos.to_move = Player::P1;
        let bd = evaluate_breakdown(&pos);
        let bs = evaluate_by_square(&pos);
        assert_eq!(bd.total, bs.total, "total mismatch");
    }

    #[test]
    fn evaluate_by_square_terminal_p1_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        let bs = evaluate_by_square(&pos);
        assert_eq!(bs.total, MATE_SCORE);
        assert!(bs.terminal);
        // All per-square records must be zero — mirrors evaluate_breakdown's
        // terminal short-circuit.
        for s in bs.squares.iter() {
            assert!(!s.occupied);
            assert_eq!(s.piece_total, 0);
        }
    }

    #[test]
    fn evaluate_by_square_records_intermediates() {
        // P1 Champion at e4 (sq 28) with P2 Champion at f4 (sq 29) attacking it.
        // Assert intermediate values populated: attacker count, mobility raw.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default().with_hp(2)
                .with_skill1(crate::game_logic::skills::Skill::Lance as u8));
        place(&mut pos, 29, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.p1_money = 10;
        pos.p2_money = 10;
        let bs = evaluate_by_square(&pos);
        // The P1 Champion at sq 28 sees P2 Champion at sq 29 as an attacker.
        assert!(bs.squares[28].occupied);
        assert!(bs.squares[28].is_p1);
        assert!(bs.squares[28].n_attackers >= 1, "P2 Champion should threaten P1");
        // Champion mobility_raw counts enemies in skill range; Lance range 2.
        // P2 Champion at sq 29 is 1 square away → in range.
        assert!(bs.squares[28].mobility_raw >= 1);
        // Skill availability at p1_money=10: Lance cost 2, so availability=256 (max).
        assert_eq!(bs.squares[28].skill1_avail_fp, SKILL_AVAIL_MAX);
    }

    #[test]
    fn coverage_requires_dual_adjacency_to_defender_and_ring_square() {
        // Regression: E6 coverage previously counted an empty ring square `s`
        // as shielded if *any* own Guard sat adjacent to `s`, even when that
        // Guard was NOT adjacent to the defender. Bodyguard only triggers when
        // a friendly Guard is adjacent to BOTH the defender and the attacker's
        // approach square, so a distant Guard cannot contribute to coverage.

        // Case A: Guard at c3 (sq 18) is NOT adjacent to defender at e4 (sq 28)
        // — chebyshev distance 2. Coverage MUST be 0.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 18, Player::P1, 2, MailboxEntry::default().with_hp(2));
        let bs = evaluate_by_square(&pos);
        assert_eq!(bs.squares[28].empty_ring_shielded, 0,
            "guard at sq 18 is not adjacent to defender at sq 28; coverage must be 0");

        // Case B: Guard at f4 (sq 29) IS adjacent to defender at e4 (sq 28).
        // Its ring is {sq 20, 21, 22, 28, 30, 36, 37, 38}. Intersecting with
        // defender's empty ring {19,20,21,27,35,36,37} yields {20, 21, 36, 37}
        // = 4 shielded squares out of 7 empty (sq 29 is occupied by the guard).
        let mut pos2 = Position::empty();
        place(&mut pos2, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos2, 29, Player::P1, 2, MailboxEntry::default().with_hp(2));
        let bs2 = evaluate_by_square(&pos2);
        assert_eq!(bs2.squares[28].empty_ring_total, 7, "1 of 8 ring squares occupied");
        assert_eq!(bs2.squares[28].empty_ring_shielded, 4,
            "guard at sq 29 shields the 4 empty ring squares also adjacent to it");
    }
}
