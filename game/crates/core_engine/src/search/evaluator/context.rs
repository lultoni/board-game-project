//! Shared, precomputed evaluation state (ns-43). Built once per `evaluate()`
//! call and borrowed by every [`EvalTerm`](super::term::EvalTerm), so trait
//! dispatch never recomputes the board scan, attacker table, or availability
//! tables per term.
//!
//! All helper functions here are lifted verbatim from the pre-ns-43 flat
//! evaluator; their math is unchanged (golden-equality invariant).

use crate::state::Position;
use crate::state::position::Phase;
use crate::search::see::{build_attackers_table, AttackersTable};
use crate::game_logic::skills::{Skill, SkillCategory, skill_cost, skill_default_range, skill_category};
use crate::game_logic::make_unmake::skill_phase_budget;
use super::params::EvalParams;

/// Coarse game stage (ns-43 stage infra). Derived from total material on the
/// board with a round-number bias (rounds drive income + skill budget in this
/// game, so a long game is "later" even at high material). Consumed by phase-
/// gated terms via [`EvalTerm::is_active`](super::term::EvalTerm::is_active) —
/// notably the asymmetric `endgame_closing` term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStage {
    Opening,
    Mid,
    End,
}

/// Per-`evaluate()` shared state. Holds a borrow of the params so terms read
/// weights without cloning.
pub struct EvalContext<'a> {
    pub pos:    &'a Position,
    pub params: &'a EvalParams,

    pub all_occ:   u64,
    pub p1_bb:     u64,
    pub p2_bb:     u64,
    pub p1_guards: u64,
    pub p2_guards: u64,

    /// Per-side skill-availability lookup (16 entries), fixed-point 0..=skill_avail_max.
    pub p1_avail: [i32; 16],
    pub p2_avail: [i32; 16],

    /// Attacker table shared by exposure.
    pub atk: AttackersTable,

    pub phase:            Phase,
    /// Skill actions per round for the current round (0 during Draft).
    pub actions_per_round: u8,
    /// E3 money caps: `max_owned_skill_cost × actions_per_round`.
    pub p1_money_cap: u16,
    pub p2_money_cap: u16,

    /// Unified P1-POV lead score (ns-43 stage infra): summed differential of
    /// material + hp + armor + equipped-skill value. Positive → P1 ahead. The
    /// "who is ahead" signal the asymmetric closing term reads.
    pub advantage: i32,
    /// Coarse game stage from total on-board material + round bias.
    pub stage: GameStage,
}

impl<'a> EvalContext<'a> {
    /// Build the shared state. Mirrors the setup block of the old
    /// `evaluate_breakdown` exactly.
    pub fn new(pos: &'a Position, params: &'a EvalParams) -> Self {
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let p1_bb = pos.p1_pieces.0;
        let p2_bb = pos.p2_pieces.0;
        let p1_guards = p1_bb & pos.guards.0;
        let p2_guards = p2_bb & pos.guards.0;

        let p1_avail = side_availability_table(pos.p1_money, params);
        let p2_avail = side_availability_table(pos.p2_money, params);

        let atk = build_attackers_table(pos, all_occ);

        let phase = pos.current_phase;
        let actions = actions_per_round(phase, pos.round_number);
        let p1_max_cost = max_owned_skill_cost(pos, p1_bb);
        let p2_max_cost = max_owned_skill_cost(pos, p2_bb);

        // Stage infra: per-side summed value (material + hp + armor + skills)
        // and total on-board material. Single scan; integer-only.
        let (p1_val, p1_mat) = side_value_and_material(pos, p1_bb, params);
        let (p2_val, p2_mat) = side_value_and_material(pos, p2_bb, params);
        let advantage = p1_val - p2_val;
        let stage = classify_stage(p1_mat + p2_mat, pos.round_number, params);

        EvalContext {
            pos, params, all_occ, p1_bb, p2_bb, p1_guards, p2_guards,
            p1_avail, p2_avail, atk, phase,
            actions_per_round: actions,
            p1_money_cap: p1_max_cost as u16 * actions as u16,
            p2_money_cap: p2_max_cost as u16 * actions as u16,
            advantage,
            stage,
        }
    }
}

/// Per-side summed value `(total_value, material_value)` for the stage infra.
/// `total_value` = material + hp + armor + equipped-skill base value (the
/// `advantage` differential uses this); `material_value` = piece material only
/// (the stage classifier uses the board total). Reuses the same weights the
/// material/hp/armor/skills terms use — single source of truth via `params`.
#[inline]
pub fn side_value_and_material(pos: &Position, side_bb: u64, params: &EvalParams) -> (i32, i32) {
    let mut total = 0i32;
    let mut material = 0i32;
    let mut bits = side_bb;
    while bits != 0 {
        let sq = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let m = pos.mailbox[sq];
        let mat = if pos.kings.0 & mask != 0 {
            params.king_material
        } else if pos.champions.0 & mask != 0 {
            params.champion_value
        } else {
            params.guard_value
        };
        material += mat;
        total += mat
            + params.hp_per_point * m.hp() as i32
            + params.armor_per_point * m.armor() as i32;
        for id in [m.skill1(), m.skill2()] {
            let idx = id as usize;
            if idx > 0 && idx < params.skill_value.len() {
                total += params.skill_value[idx];
            }
        }
    }
    (total, material)
}

/// Classify the coarse game stage from total on-board material with a round-
/// number bias (each round elapsed credits `stage_round_bias` toward "later").
#[inline]
pub fn classify_stage(total_material: i32, round_number: u16, params: &EvalParams) -> GameStage {
    let effective = total_material - round_number as i32 * params.stage_round_bias;
    if effective >= params.stage_mid_threshold {
        GameStage::Opening
    } else if effective >= params.stage_end_threshold {
        GameStage::Mid
    } else {
        GameStage::End
    }
}

/// King-expand: bitboard OR of all 8-directional 1-step neighbours.
#[inline]
pub fn king_expand(x: u64) -> u64 {
    const NOT_A: u64 = 0xfefefefefefefefe;
    const NOT_H: u64 = 0x7f7f7f7f7f7f7f7f;
    let l = (x & NOT_A) >> 1;
    let r = (x & NOT_H) << 1;
    let h = x | l | r;
    h | (h << 8) | (h >> 8)
}

/// Chebyshev dilation by `radius`: the set of all squares within Chebyshev
/// distance `radius` of any set bit in `x` (includes `x` itself). `radius`
/// applications of [`king_expand`]. `radius == 0` returns `x` unchanged.
#[inline]
pub fn expand_n(x: u64, radius: u8) -> u64 {
    let mut out = x;
    for _ in 0..radius { out = king_expand(out); }
    out
}

/// Per-skill availability given a side's money snapshot. Piecewise-linear
/// sigmoid centred at `money - cost`; fixed-point 0..=`skill_avail_max`.
#[inline]
pub fn skill_availability_fp(money: i32, cost: i32, params: &EvalParams) -> i32 {
    let x = money - cost + params.skill_avail_k;
    let denom = 2 * params.skill_avail_k;
    if x <= 0 { 0 }
    else if x >= denom { params.skill_avail_max }
    else { (x * params.skill_avail_max) / denom }
}

/// Per-side [16]-entry availability lookup.
#[inline]
pub fn side_availability_table(money: u16, params: &EvalParams) -> [i32; 16] {
    let mut t = [0i32; 16];
    for id in 1u8..=15 {
        if let Some(s) = crate::game_logic::skills::skill_from_id(id) {
            t[id as usize] = skill_availability_fp(money as i32, skill_cost(s) as i32, params);
        }
    }
    t
}

/// Max cost across a side's equipped skills. 0 during Draft or if no skills.
#[inline]
pub fn max_owned_skill_cost(pos: &Position, side_bb: u64) -> u8 {
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

/// Skill actions per round. Move actions cost no money → excluded. Draft → 0.
#[inline]
pub fn actions_per_round(phase: Phase, round_number: u16) -> u8 {
    match phase {
        Phase::Draft => 0,
        _            => skill_phase_budget(round_number),
    }
}

/// E3 — money value with diminishing returns capped at `cap`. Cap 0 → 0.
#[inline]
pub fn useful_money(money: u16, cap: u16, params: &EvalParams) -> i32 {
    if cap == 0 { return 0; }
    let m = money as i64;
    let c = cap as i64;
    let mpu = params.money_per_unit as i64;
    let value = if m <= c {
        (mpu * m * (2 * c - m)) / (2 * c)
    } else {
        (mpu * c) / 2
    };
    value as i32
}

/// E9 — side's max offensive range across castable strike + Shove skills.
/// Verbatim from the pre-ns-43 evaluator (only the const `Phase::Draft` guard
/// and `skill_*` lookups are used; no params needed — this is pure reach).
pub fn max_offensive_range(pos: &Position, side_bb: u64, money: u16) -> u8 {
    if pos.current_phase == Phase::Draft { return 0; }
    if money < 2 { return 0; }

    let actions = actions_per_round(pos.current_phase, pos.round_number);
    let focus_bonus_possible = actions >= 2;

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
        if raw_best >= 3 && boosted_best >= 3 && owns_focus { break; }
    }

    if focus_bonus_possible && owns_focus && boosted_best > 0 {
        (boosted_best + 1).max(raw_best)
    } else {
        raw_best
    }
}
