//! Ported eval terms (ns-43). Each is a zero-size struct implementing
//! [`EvalTerm`](super::term::EvalTerm). Math is verbatim from the pre-ns-43
//! flat evaluator; the `golden_eval_unchanged` test enforces this.
//!
//! Per-piece terms: material, hp, armor, skills, mobility, exposure, coverage.
//! Side-level terms: money, tempo, offensive_range, wasted_modifier.

use crate::state::position::Player;
use crate::state::magic;
use crate::game_logic::skills::{skill_default_range, TargetOwner, skill_target_owner, skill_from_id};
use super::context::{EvalContext, king_expand, expand_n, useful_money, max_offensive_range};
use super::term::{EvalTerm, PieceContext};

// === Per-piece terms =====================================================

pub struct Material;
impl EvalTerm for Material {
    fn name(&self) -> &'static str { "material" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        let p = ctx.params;
        if pc.is_king { p.king_material }
        else if pc.is_champion { p.champion_value }
        else { p.guard_value }
    }
}

pub struct Hp;
impl EvalTerm for Hp {
    fn name(&self) -> &'static str { "hp" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        ctx.params.hp_per_point * pc.mailbox.hp() as i32
    }
}

pub struct Armor;
impl EvalTerm for Armor {
    fn name(&self) -> &'static str { "armor" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        ctx.params.armor_per_point * pc.mailbox.armor() as i32
    }
}

pub struct Skills;
impl EvalTerm for Skills {
    fn name(&self) -> &'static str { "skills" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        let p = ctx.params;
        let avail = if pc.is_p1 { &ctx.p1_avail } else { &ctx.p2_avail };
        let sid1 = pc.mailbox.skill1() as usize;
        let sid2 = pc.mailbox.skill2() as usize;
        let sk1_base = if sid1 < p.skill_value.len() { p.skill_value[sid1] } else { 0 };
        let sk2_base = if sid2 < p.skill_value.len() { p.skill_value[sid2] } else { 0 };
        let sk1_a = if sid1 < avail.len() { avail[sid1] } else { 0 };
        let sk2_a = if sid2 < avail.len() { avail[sid2] } else { 0 };
        (sk1_base * sk1_a + sk2_base * sk2_a) / p.skill_avail_max
    }
}

pub struct Mobility;
impl EvalTerm for Mobility {
    fn name(&self) -> &'static str { "mobility" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        let p = ctx.params;
        let own_bb = if pc.is_p1 { ctx.p1_bb } else { ctx.p2_bb };
        let opp_bb = if pc.is_p1 { ctx.p2_bb } else { ctx.p1_bb };
        if pc.is_guard {
            magic::movement_targets_speed2(pc.sq, ctx.all_occ).0.count_ones() as i32
                * p.guard_mob_per_sq
        } else if pc.is_king {
            (magic::movement_targets_speed1(pc.sq).0 & !own_bb).count_ones() as i32
                * p.king_mob_per_sq
        } else {
            // Champion: skill-range coverage over enemies.
            let mut cov = 0i32;
            for sid in [pc.mailbox.skill1(), pc.mailbox.skill2()] {
                let Some(sk) = skill_from_id(sid) else { continue };
                let owner = skill_target_owner(sk);
                if !matches!(owner, TargetOwner::Enemy | TargetOwner::Either) { continue; }
                let range = skill_default_range(sk);
                let ray = magic::skill_attacks(pc.sq, ctx.all_occ, range).0;
                cov += (ray & opp_bb).count_ones() as i32 * p.champ_skill_cov_per_enemy;
            }
            cov.min(p.champ_skill_cov_cap)
        }
    }
}

pub struct Exposure;
impl EvalTerm for Exposure {
    fn name(&self) -> &'static str { "exposure" }
    fn is_per_piece(&self) -> bool { true }
    /// Penalty: stored as positive magnitudes, subtracts in the total.
    fn signed_total(&self, p1: i32, p2: i32, _params: &super::params::EvalParams) -> i32 { -(p1 - p2) }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        let p = ctx.params;
        let opp_attackers_bb = if pc.is_p1 { ctx.atk.any_attackers_of(Player::P2, pc.sq) }
                               else         { ctx.atk.any_attackers_of(Player::P1, pc.sq) };
        let n_attackers = opp_attackers_bb.count_ones() as usize;
        if pc.is_king {
            p.king_exposure[n_attackers.min(3)]
        } else {
            let own_guards = if pc.is_p1 { ctx.p1_guards } else { ctx.p2_guards };
            let n_adj_guards = (king_expand(pc.mask) & own_guards).count_ones() as usize;
            let unshielded = n_attackers.saturating_sub(n_adj_guards).min(3);
            let mult_pct = p.exposure_mult[unshielded];
            let piece_val = if pc.is_champion { p.champion_value } else { p.guard_value };
            (piece_val * mult_pct) / 100
        }
    }
}

pub struct Coverage;
impl EvalTerm for Coverage {
    fn name(&self) -> &'static str { "coverage" }
    fn is_per_piece(&self) -> bool { true }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        if pc.is_guard { return 0; }
        let p = ctx.params;
        let own_guards = if pc.is_p1 { ctx.p1_guards } else { ctx.p2_guards };
        let defender_neighbours = king_expand(pc.mask) & !pc.mask;
        let empty_ring = defender_neighbours & !ctx.all_occ;
        let denom = empty_ring.count_ones() as i32;
        let mut shielded = 0i32;
        let mut ring_bits = empty_ring;
        while ring_bits != 0 {
            let s = ring_bits.trailing_zeros();
            ring_bits &= ring_bits - 1;
            let s_bit = 1u64 << s;
            let dual_neigh = king_expand(s_bit) & defender_neighbours & own_guards;
            if dual_neigh != 0 { shielded += 1; }
        }
        let coverage_fp = if denom == 0 { p.skill_avail_max } else { (shielded * p.skill_avail_max) / denom };
        let piece_val = p.champion_value; // king shielded ≈ champion-scale
        (p.coverage_per_piece * piece_val * coverage_fp) / (100 * p.skill_avail_max)
    }
}

// === Side-level terms ====================================================

pub struct Money;
impl EvalTerm for Money {
    fn name(&self) -> &'static str { "money" }
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        (
            useful_money(ctx.pos.p1_money, ctx.p1_money_cap, ctx.params),
            useful_money(ctx.pos.p2_money, ctx.p2_money_cap, ctx.params),
        )
    }
}

pub struct Tempo;
impl EvalTerm for Tempo {
    fn name(&self) -> &'static str { "tempo" }
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        use crate::state::position::Phase;
        if ctx.phase == Phase::Draft { return (0, 0); }
        let tempo = ctx.params.tempo_per_action * ctx.pos.actions_remaining as i32;
        match ctx.pos.to_move {
            Player::P1 => (tempo, 0),
            Player::P2 => (0, tempo),
        }
    }
}

pub struct OffensiveRange;
impl EvalTerm for OffensiveRange {
    fn name(&self) -> &'static str { "offensive_range" }
    /// Raw range differential scaled by the tunable weight.
    fn signed_total(&self, p1: i32, p2: i32, params: &super::params::EvalParams) -> i32 {
        (p1 - p2) * params.offensive_range_weight
    }
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        (
            max_offensive_range(ctx.pos, ctx.p1_bb, ctx.pos.p1_money) as i32,
            max_offensive_range(ctx.pos, ctx.p2_bb, ctx.pos.p2_money) as i32,
        )
    }
}

/// E10 (ns-43) — wasted Focus/Charge penalty.
///
/// Symptom it fixes: the AI casts Focus/Charge and then ends its Skill phase
/// without ever consuming the buff, burning 2–5 money/round. The `skills` term
/// rewards *owning* Focus/Charge but nothing reads `pos.pending_modifiers`, so a
/// live buff bit with no skill left to spend it is invisible to eval.
///
/// The bits are turn-scoped and belong to `pos.to_move`, so this only fires for
/// the side to move, and only in `Phase::Skill` (buffs are cast + consumed in
/// the Skill phase). A buff is "wasted" iff there is no *castable consumer* left
/// this phase:
///   - Focus (+1 range) is consumed by any castable offensive skill — a Strike
///     or a Shove.
///   - Charge (+1 dmg) is consumed by a castable Strike only.
/// "Castable" = `actions_remaining >= 1` AND the side owns a piece equipping such
/// a skill it can afford right now.
///
/// Stored as a positive magnitude on the holding side; `signed_total` negates it
/// (it is a penalty, like `exposure`).
pub struct WastedModifier;
impl EvalTerm for WastedModifier {
    fn name(&self) -> &'static str { "wasted_modifier" }

    fn is_active(&self, ctx: &EvalContext) -> bool {
        use crate::state::position::Phase;
        ctx.phase == Phase::Skill
    }

    /// Penalty: stored as positive magnitude, subtracts in the total.
    fn signed_total(&self, p1: i32, p2: i32, _params: &super::params::EvalParams) -> i32 { -(p1 - p2) }

    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        use crate::state::position::modifier_bits;
        use crate::game_logic::skills::{Skill, SkillCategory, skill_category, skill_cost};

        let bits = ctx.pos.pending_modifiers;
        let focus_live  = bits & modifier_bits::FOCUS  != 0;
        let charge_live = bits & modifier_bits::CHARGE != 0;
        if !focus_live && !charge_live { return (0, 0); }

        // The buffs belong to the side to move. If it can't act, nothing is
        // castable — every live buff is wasted this phase.
        let can_act = ctx.pos.actions_remaining >= 1;
        let (side_bb, money) = match ctx.pos.to_move {
            Player::P1 => (ctx.p1_bb, ctx.pos.p1_money),
            Player::P2 => (ctx.p2_bb, ctx.pos.p2_money),
        };

        // Scan the holding side once: does it own an affordable Strike / Shove?
        let mut has_castable_strike = false;
        let mut has_castable_offensive = false; // Strike OR Shove
        if can_act {
            let mut b = side_bb;
            while b != 0 {
                let sq = b.trailing_zeros() as usize;
                b &= b - 1;
                let m = ctx.pos.mailbox[sq];
                for id in [m.skill1(), m.skill2()] {
                    let Some(s) = skill_from_id(id) else { continue };
                    if money < skill_cost(s) as u16 { continue; }
                    let is_strike = matches!(skill_category(s), SkillCategory::Strike);
                    let is_offensive = is_strike || matches!(s, Skill::Shove);
                    if is_strike    { has_castable_strike = true; }
                    if is_offensive { has_castable_offensive = true; }
                }
                if has_castable_strike { break; } // strongest consumer found; stop early
            }
        }

        // A buff is wasted iff its consumer is absent. Penalty scales by the
        // buff's own money cost, so the "cost" of the mistake tracks the money
        // it burned.
        let mut penalty = 0i32;
        if focus_live && !has_castable_offensive {
            penalty += ctx.params.wasted_modifier_per_cost * skill_cost(Skill::Focus) as i32;
        }
        if charge_live && !has_castable_strike {
            penalty += ctx.params.wasted_modifier_per_cost * skill_cost(Skill::Charge) as i32;
        }
        if penalty == 0 { return (0, 0); }

        match ctx.pos.to_move {
            Player::P1 => (penalty, 0),
            Player::P2 => (0, penalty),
        }
    }
}

/// E11 (ns-43) — guard isolation penalty.
///
/// Symptom it fixes: the AI shoves a guard forward and strands it — not in open
/// space, but **crammed among enemy pieces, unsupported**. `exposure` only fires
/// on *direct attackers* of the guard's square, so a guard sitting one tile from
/// a cluster of enemies (about to be attacked, or blocking nothing) is invisible
/// until the trade is already on. This term reads the local balance of force:
/// a guard with more enemies than friendlies within `guard_iso_radius` tiles is
/// a hanging piece. Location-agnostic — a guard alone among enemies is bad
/// anywhere (per designer: the bad guard was deep + surrounded, not in space).
///
/// Penalty (positive magnitude; `signed_total` negates it, like `exposure`):
///   `guard_iso_per_step × outnumber`, where
///   `outnumber = max(0, enemies_near − friendlies_near)`,
/// optionally amplified by `guard_iso_depth_pct` when the guard stands on the
/// enemy's half (P1 guard on rank ≥ 4, P2 guard on rank ≤ 3).
pub struct GuardIsolation;
impl EvalTerm for GuardIsolation {
    fn name(&self) -> &'static str { "guard_isolation" }
    fn is_per_piece(&self) -> bool { true }
    /// Penalty: stored as positive magnitude, subtracts in the total.
    fn signed_total(&self, p1: i32, p2: i32, _params: &super::params::EvalParams) -> i32 { -(p1 - p2) }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        if !pc.is_guard { return 0; }
        let p = ctx.params;

        // Chebyshev-radius neighbourhood around the guard (includes the guard's
        // own square; excluded from friendly count below via `& !pc.mask`).
        let hood = expand_n(pc.mask, p.guard_iso_radius);
        let (own_bb, opp_bb) = if pc.is_p1 { (ctx.p1_bb, ctx.p2_bb) } else { (ctx.p2_bb, ctx.p1_bb) };
        let enemies_near   = (hood & opp_bb).count_ones() as i32;
        let friendlies_near = (hood & own_bb & !pc.mask).count_ones() as i32;

        let outnumber = (enemies_near - friendlies_near).max(0);
        if outnumber == 0 { return 0; }

        let mut penalty = p.guard_iso_per_step * outnumber;

        // Depth amplification: a guard stranded on the ENEMY half is worse than
        // one stranded at home. P1 advances toward high ranks, P2 toward low.
        if p.guard_iso_depth_pct != 100 {
            let rank = pc.sq / 8;
            let on_enemy_half = if pc.is_p1 { rank >= 4 } else { rank <= 3 };
            if on_enemy_half {
                penalty = (penalty * p.guard_iso_depth_pct) / 100;
            }
        }
        penalty
    }
}
