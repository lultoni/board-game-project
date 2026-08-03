//! Ported eval terms (ns-43). Each is a zero-size struct implementing
//! [`EvalTerm`](super::term::EvalTerm). Math is verbatim from the pre-ns-43
//! flat evaluator; the `golden_eval_unchanged` test enforces this.
//!
//! Per-piece terms: material, hp, armor, skills, mobility, exposure, coverage.
//! Side-level terms: money, tempo, offensive_range, wasted_modifier.

use crate::state::position::Player;
use crate::state::magic;
use crate::game_logic::skills::{skill_from_id, skill_category, skill_target_owner, skill_default_range, SkillCategory, TargetOwner};
use super::context::{EvalContext, king_expand, expand_n, useful_money, max_offensive_range};
use super::term::{EvalTerm, PieceContext};

/// Integer soft cap: saturating hyperbola `x·k/(x+k)` for `x, k ≥ 0`. Rises
/// ~linearly for small `x`, asymptotes to `k`. Order-independent → deterministic.
#[inline]
fn softcap(x: i32, k: i32) -> i32 {
    if x <= 0 || k <= 0 { return 0; }
    ((x as i64 * k as i64) / (x as i64 + k as i64)) as i32
}

/// Champion-threat score for one champion. Shared by [`ChampionThreat`] and the
/// diagnostic `evaluate_by_square` so both stay byte-identical. `is_p1` selects
/// the champion's side; `atk` is the shared attackers table.
///
/// Returns the champion's total threat contribution (offensive + defensive,
/// each soft-capped and weighted). All integer / bitboard math → deterministic.
#[allow(clippy::too_many_arguments)]
pub(crate) fn champion_threat_score(
    pos: &crate::state::Position,
    params: &super::params::EvalParams,
    atk: &crate::search::see::AttackersTable,
    sq: u8,
    mailbox: crate::state::MailboxEntry,
    is_p1: bool,
    all_occ: u64,
    p1_bb: u64,
    p2_bb: u64,
) -> i32 {
    let p = params;
    let (own_bb, opp_bb) = if is_p1 { (p1_bb, p2_bb) } else { (p2_bb, p1_bb) };
    let enemy_player  = if is_p1 { Player::P2 } else { Player::P1 };
    let friend_player = if is_p1 { Player::P1 } else { Player::P2 };
    let self_mask = 1u64 << sq;

    let target_value = |t_mask: u64| -> i32 {
        let base = if pos.kings.0 & t_mask != 0 {
            p.champion_value + (p.threat_king_bonus << p.threat_value_shift)
        } else if pos.champions.0 & t_mask != 0 {
            p.champion_value
        } else {
            p.guard_value
        };
        base >> p.threat_value_shift
    };

    let mut offensive_raw = 0i32;
    let mut defensive_raw = 0i32;

    for sid in [mailbox.skill1(), mailbox.skill2()] {
        let Some(sk) = skill_from_id(sid) else { continue };
        let owner = skill_target_owner(sk);
        let range = skill_default_range(sk);
        if range == 0 { continue; }

        match owner {
            TargetOwner::Enemy | TargetOwner::Either => {
                // Only trace the ray for owners whose branch reads it. Empty /
                // SelfOnly skills (Dash, Retreat, Shield, Focus, Charge) score
                // nothing here, so tracing their ray was pure waste - the common
                // case for many equipped champions. Byte-identical: the ray was
                // unused in those branches.
                let ray = magic::skill_attacks(sq, all_occ, range).0;
                let is_strike = matches!(skill_category(sk), SkillCategory::Strike);
                let mut hits = ray & opp_bb;
                while hits != 0 {
                    let t = hits.trailing_zeros() as u8;
                    hits &= hits - 1;
                    let tmask = 1u64 << t;
                    let mut v = target_value(tmask);
                    if is_strike {
                        if let Some(land) = magic::step_toward(sq, t) {
                            let land_mask = 1u64 << land;
                            if land_mask != tmask {
                                let enemy_atk  = atk.any_attackers_of(enemy_player, land).count_ones() as i32;
                                let friend_atk = atk.any_attackers_of(friend_player, land).count_ones() as i32;
                                if enemy_atk > friend_atk {
                                    v = (v * p.threat_safety_penalty_pct) / 100;
                                }
                            }
                        }
                    }
                    offensive_raw += v;
                }
            }
            TargetOwner::Ally => {
                let ray = magic::skill_attacks(sq, all_occ, range).0;
                let mut hits = ray & own_bb & !self_mask;
                while hits != 0 {
                    let t = hits.trailing_zeros() as u8;
                    hits &= hits - 1;
                    let tmask = 1u64 << t;
                    let base = target_value(tmask);
                    let m = pos.mailbox[t as usize];
                    let low_hp = if m.hp() <= 1 { 1 } else { 0 };
                    let attacked = if atk.any_attackers_of(enemy_player, t) != 0 { 1 } else { 0 };
                    defensive_raw += base + (base * (low_hp + attacked)) / 2;
                }
            }
            TargetOwner::Empty | TargetOwner::SelfOnly => {}
        }
    }

    let offensive = (softcap(offensive_raw, p.threat_softcap) * p.threat_offensive_weight) / 100;
    let defensive = (softcap(defensive_raw, p.threat_softcap) * p.threat_defensive_weight) / 100;
    offensive + defensive
}

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
        if pc.is_guard {
            magic::movement_targets_speed2(pc.sq, ctx.all_occ).0.count_ones() as i32
                * p.guard_mob_per_sq
        } else if pc.is_king {
            (magic::movement_targets_speed1(pc.sq).0 & !own_bb).count_ones() as i32
                * p.king_mob_per_sq
        } else {
            // Champion (ns-43 Term 3a): REAL movement-space. Champions are
            // speed-1; reward reachable empty squares (skill-cast path
            // flexibility + endgame maneuvering). The former enemy-in-range
            // "mobility" moved to the `champion_threat` term.
            (magic::movement_targets_speed1(pc.sq).0 & !ctx.all_occ).count_ones() as i32
                * p.champ_mob_per_sq
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
        let enemy_bb = if pc.is_p1 { ctx.p2_bb } else { ctx.p1_bb };
        let defender_neighbours = king_expand(pc.mask) & !pc.mask;
        let empty_ring = defender_neighbours & !ctx.all_occ;

        // Threat gate (ns-43+): an empty ring square is only worth shielding if
        // an enemy could actually attack *through* it - otherwise "coverage" is
        // free reward for a wall of guards with nothing to guard against (the
        // starting-rank line-placement exploit). Mark the enemies within r3 of
        // the defender, then count a ring square only when an r3-enemy lies in
        // its direction (its outward r2-dilation reaches one). No threat in a
        // direction → that square does not count toward denom or shielded.
        // Approximates the in-game Bodyguard rule (screens an attack path).
        let enemies_r3 = expand_n(pc.mask, 3) & enemy_bb;
        if enemies_r3 == 0 { return 0; }

        let mut denom = 0i32;
        let mut shielded = 0i32;
        let mut ring_bits = empty_ring;
        while ring_bits != 0 {
            let s = ring_bits.trailing_zeros();
            ring_bits &= ring_bits - 1;
            let s_bit = 1u64 << s;
            if expand_n(s_bit, 2) & enemies_r3 == 0 { continue; } // no threat in this direction
            denom += 1;
            let dual_neigh = king_expand(s_bit) & defender_neighbours & own_guards;
            if dual_neigh != 0 { shielded += 1; }
        }
        // No *threatened* ring square (enemies near but not lined up with any
        // open ring square) → nothing to cover → neutral, not full credit.
        if denom == 0 { return 0; }
        let coverage_fp = (shielded * p.skill_avail_max) / denom;
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

/// E10 (ns-43) - wasted Focus/Charge penalty.
///
/// Symptom it fixes: the AI casts Focus/Charge and then ends its Skill phase
/// without ever consuming the buff, burning 2-5 money/round. The `skills` term
/// rewards *owning* Focus/Charge but nothing reads `pos.pending_modifiers`, so a
/// live buff bit with no skill left to spend it is invisible to eval.
///
/// The bits are turn-scoped and belong to `pos.to_move`, so this only fires for
/// the side to move, and only in `Phase::Skill` (buffs are cast + consumed in
/// the Skill phase). A buff is "wasted" iff there is no *castable consumer* left
/// this phase:
///   - Focus (+1 range) is consumed by any castable offensive skill - a Strike
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
        // castable - every live buff is wasted this phase.
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

/// E11 (ns-43) - guard isolation penalty.
///
/// Symptom it fixes: the AI shoves a guard forward and strands it - not in open
/// space, but **crammed among enemy pieces, unsupported**. `exposure` only fires
/// on *direct attackers* of the guard's square, so a guard sitting one tile from
/// a cluster of enemies (about to be attacked, or blocking nothing) is invisible
/// until the trade is already on. This term reads the local balance of force:
/// a guard with more enemies than friendlies within `guard_iso_radius` tiles is
/// a hanging piece. Location-agnostic - a guard alone among enemies is bad
/// anywhere (per designer: the bad guard was deep + surrounded, not in space).
///
/// Penalty (positive magnitude; `signed_total` negates it, like `exposure`):
///   `guard_iso_per_step x outnumber`, where
///   `outnumber = max(0, enemies_near - friendlies_near)`,
/// optionally amplified by `guard_iso_depth_pct` when the guard stands on the
/// enemy's half (P1 guard on rank ≥ 4, P2 guard on rank ≤ 3).
pub struct GuardIsolation;
impl EvalTerm for GuardIsolation {
    fn name(&self) -> &'static str { "guard_isolation" }
    fn is_per_piece(&self) -> bool { true }
    /// Behavior-preserving skip: scores only guards, so with no guards on the
    /// board it is uniformly 0. Golden byte-identical.
    fn is_active(&self, ctx: &EvalContext) -> bool { ctx.pos.guards.0 != 0 }
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

/// E12 (ns-43 Term 3b) - champion threat.
///
/// Replaces the crude enemy-in-range coverage the `mobility` term used to carry
/// (capped flat 60, offensive-only). Two symmetric sub-scores per champion,
/// each soft-capped so neither category is inherently rated above the other
/// (designer requirement):
///   - OFFENSIVE: enemy pieces the champion's Strike/Shove/Blast skills can hit,
///     weighted by target value (king ≫ champion > guard). Post-Stack-N a Strike
///     moves the caster 1 tile toward the target - so a strike whose landing
///     square is unsafe (more enemy than friendly attackers on it) keeps only
///     `threat_safety_penalty_pct` of its value. "Right targets you can execute
///     safely", per designer.
///   - DEFENSIVE: ally pieces the champion's Heal/Plate/Swap skills can reach,
///     weighted by the ally's value AND vulnerability (a wounded/exposed ally in
///     reach is worth more to be able to cover).
/// Folds into `total` only (no legacy field). All integer / bitboard math →
/// deterministic.
pub struct ChampionThreat;
impl EvalTerm for ChampionThreat {
    fn name(&self) -> &'static str { "champion_threat" }
    fn is_per_piece(&self) -> bool { true }
    /// Behavior-preserving skip: the term scores only champions, so if there are
    /// no champions on the board it is uniformly 0. Skipping avoids the per-
    /// champion skill-ray tracing entirely in stripped positions (the term's
    /// dominant cost). Golden byte-identical - skipped ⇔ would-be-0.
    fn is_active(&self, ctx: &EvalContext) -> bool { ctx.pos.champions.0 != 0 }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        if !pc.is_champion { return 0; }
        champion_threat_score(
            ctx.pos, ctx.params, &ctx.atk, pc.sq, pc.mailbox, pc.is_p1,
            ctx.all_occ, ctx.p1_bb, ctx.p2_bb,
        )
    }
}

/// Asymmetric endgame-closing score `(p1, p2)`. Shared by [`EndgameClosing`]
/// and the diagnostic `evaluate_by_square`. Returns `(0, 0)` unless
/// `stage == End`, the lead exceeds `close_lead_min`, and both kings exist.
/// The leader's side gets a closing score, the trailer's a stalling score.
pub(crate) fn endgame_closing_score(
    pos: &crate::state::Position,
    params: &super::params::EvalParams,
    stage: super::context::GameStage,
    advantage: i32,
    p1_bb: u64,
    p2_bb: u64,
) -> (i32, i32) {
    use super::context::GameStage;
    if !matches!(stage, GameStage::End) { return (0, 0); }
    let p = params;
    if advantage.abs() < p.close_lead_min { return (0, 0); }

    let king_sq = |side_bb: u64| -> Option<u8> {
        let k = pos.kings.0 & side_bb;
        if k == 0 { None } else { Some(k.trailing_zeros() as u8) }
    };
    let (Some(p1_king), Some(p2_king)) = (king_sq(p1_bb), king_sq(p2_bb)) else { return (0, 0); };

    let nearest = |attacker_bb: u64, target: u8| -> u8 {
        let non_king = attacker_bb & !pos.kings.0;
        let mut best = 7u8;
        let mut bits = non_king;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let d = magic::cheby_dist(sq, target);
            if d < best { best = d; }
        }
        best
    };
    let leader_score = |leader_bb: u64, enemy_bb: u64, enemy_king: u8| -> i32 {
        let dist = nearest(leader_bb, enemy_king) as i32;
        let pressure = p.close_king_pressure * (7 - dist);
        let escapes = (magic::movement_targets_speed1(enemy_king).0 & !(leader_bb | enemy_bb)).count_ones() as i32;
        let denial = p.close_escape_denial * (8 - escapes);
        pressure + denial
    };
    let trailer_score = |own_bb: u64, enemy_bb: u64, own_king: u8| -> i32 {
        let dist = nearest(enemy_bb, own_king) as i32;
        let safety = p.defend_king_safety * dist;
        let hugging = (king_expand(1u64 << own_king) & own_bb & !(1u64 << own_king)).count_ones() as i32;
        safety + p.defend_compactness * hugging
    };

    if advantage > 0 {
        (leader_score(p1_bb, p2_bb, p2_king), trailer_score(p2_bb, p1_bb, p2_king))
    } else {
        (trailer_score(p1_bb, p2_bb, p1_king), leader_score(p2_bb, p1_bb, p1_king))
    }
}

/// E13 (ns-43 Term 4) - asymmetric endgame closing.
///
/// Active only when `ctx.stage == End` (via `is_active`). The side that is
/// AHEAD (`advantage` beyond `close_lead_min`) is rewarded for *closing the
/// game out*; the side BEHIND is rewarded for *stretching it out* - the
/// asymmetry the designer asked for. Side-level; folds into `total` via the
/// default `p1 - p2`, so a leader closing well pushes toward the leader and a
/// trailer defending well pushes back toward the trailer. Delegates to the
/// shared [`endgame_closing_score`] so `evaluate_by_square` stays consistent.
pub struct EndgameClosing;
impl EvalTerm for EndgameClosing {
    fn name(&self) -> &'static str { "endgame_closing" }
    fn is_active(&self, ctx: &EvalContext) -> bool {
        matches!(ctx.stage, super::context::GameStage::End)
    }
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        endgame_closing_score(ctx.pos, ctx.params, ctx.stage, ctx.advantage, ctx.p1_bb, ctx.p2_bb)
    }
}

/// Hanging-piece SEE term (ns-53, per-piece penalty). The evaluator's other
/// terms read the current moment statically; this one lets a genuinely bounded
/// exchange calculation (SEE — alternating captures on ONE square, not a game
/// tree) inform the static score, so a piece that is SEE-losing where it stands
/// is recognised as effectively hanging.
///
/// GATING (the load-bearing part): SEE is invoked ONLY for a non-king piece
/// that an enemy actually attacks *right now*, detected via the cheap physical
/// attacker table (`ctx.atk`, already built). Quiet/unattacked pieces cost
/// nothing here, and the full (skill-scatter) attacker table is built lazily
/// only when the first attacked piece is found — so a position with no live
/// threats never pays the SEE cost, matching where the search's own QS/SEE
/// would engage.
pub struct HangingPiece;
impl EvalTerm for HangingPiece {
    fn name(&self) -> &'static str { "hanging_piece" }
    fn is_per_piece(&self) -> bool { true }
    /// Penalty: positive magnitude, subtracts in the total.
    fn signed_total(&self, p1: i32, p2: i32, _params: &super::params::EvalParams) -> i32 { -(p1 - p2) }
    fn score_piece(&self, ctx: &EvalContext, pc: &PieceContext) -> i32 {
        // King captures are the MATE branch / king_exposure / king_tempo, not this.
        if pc.is_king { return 0; }
        // Cheap gate: is this piece attacked by an enemy at all? Physical table
        // is already built; skip the SEE rollout entirely if not.
        let enemy = if pc.is_p1 { Player::P2 } else { Player::P1 };
        if ctx.atk.any_attackers_of(enemy, pc.sq) == 0 { return 0; }

        // Attacked → run the exchange from the enemy initiator's POV using the
        // full attacker table (built lazily on first need). The enemy picks its
        // least-valuable attacker; `see_capture` returns net material for the
        // initiator (the enemy). A positive result means the enemy wins material
        // by capturing here → our piece is (partly) hanging.
        let table = ctx.atk_full();
        let enemy_attackers = table.any_attackers_of(enemy, pc.sq);
        if enemy_attackers == 0 { return 0; }
        // Cheapest enemy attacker as the initiator (lowest square is a stable
        // deterministic pick; see_capture internally orders by LVA anyway).
        let src = enemy_attackers.trailing_zeros() as u8;
        let gain = crate::search::see::see_capture(ctx.pos, table, src, pc.sq);
        if gain <= 0 { return 0; }
        (gain * ctx.params.hanging_penalty_pct) / 100
    }
}

/// King-tempo SEE term (ns-53, side-level penalty). Flags the specific "king one
/// loud action from capture" state that the attacker-count `king_exposure` curve
/// under-weights. Reuses `is_king_threatened` — a cheap bitboard reachability
/// scan, NOT an exchange rollout — so it is always affordable.
pub struct KingTempo;
impl EvalTerm for KingTempo {
    fn name(&self) -> &'static str { "king_tempo" }
    /// Penalty: positive magnitude, subtracts in the total.
    fn signed_total(&self, p1: i32, p2: i32, _params: &super::params::EvalParams) -> i32 { -(p1 - p2) }
    fn score_side(&self, ctx: &EvalContext) -> (i32, i32) {
        let pen = ctx.params.king_tempo_penalty;
        let p1 = if crate::search::quiescence::is_king_threatened(ctx.pos, Player::P1) { pen } else { 0 };
        let p2 = if crate::search::quiescence::is_king_threatened(ctx.pos, Player::P2) { pen } else { 0 };
        (p1, p2)
    }
}
