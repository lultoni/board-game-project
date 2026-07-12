//! Breakdown types + projection (ns-43).
//!
//! [`DynBreakdown`] is the source of truth produced by the registry: a dynamic
//! list of only the *active* terms, each with its per-side magnitudes and
//! signed contribution. [`EvalBreakdown`] is the legacy fixed-field struct that
//! the frontend / telemetry / nn_trainer / search_bench consume by name;
//! [`DynBreakdown::to_legacy`] projects the known ported terms onto it
//! byte-identically. New terms (added in later passes) fold into `total` only
//! until the frontend learns to read the dynamic list.
//!
//! [`evaluate_by_square`] is the diagnostic per-square view; its math is moved
//! verbatim from the pre-ns-43 evaluator and is guarded by the
//! `evaluate_by_square_matches_breakdown_*` tests.

use crate::state::Position;
use crate::state::position::{GameResult, Player, Phase};
use crate::state::magic;
use crate::search::see::build_attackers_table;
use super::MATE_SCORE;
use super::params::EvalParams;
use super::context::{
    king_expand, expand_n, side_availability_table, useful_money, max_offensive_range,
    max_owned_skill_cost, actions_per_round, side_value_and_material, classify_stage,
};

/// One active term's contribution in the dynamic breakdown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermEntry {
    pub name:   &'static str,
    pub p1:     i32,
    pub p2:     i32,
    /// The term's signed contribution to `total` (sign/weight already applied).
    pub signed: i32,
}

/// Dynamic breakdown — the registry's native output. Only active terms appear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynBreakdown {
    pub terms:    Vec<TermEntry>,
    pub total:    i32,
    pub terminal: bool,
}

impl DynBreakdown {
    pub fn terminal(total: i32) -> Self {
        DynBreakdown { terms: Vec::new(), total, terminal: true }
    }

    /// Look up an active term's `(p1, p2)` magnitudes by name; `(0, 0)` if the
    /// term is inactive/absent.
    fn pair(&self, name: &str) -> (i32, i32) {
        self.terms.iter().find(|t| t.name == name).map(|t| (t.p1, t.p2)).unwrap_or((0, 0))
    }

    /// Project onto the legacy fixed-field `EvalBreakdown`. Byte-identical to
    /// the pre-ns-43 output for the ported terms. `threat_*`/`skill_act_*`
    /// remain 0 (removed in earlier passes; kept for schema compat).
    pub fn to_legacy(&self) -> EvalBreakdown {
        if self.terminal {
            return EvalBreakdown { total: self.total, ..Default::default() };
        }
        let (material_p1, material_p2) = self.pair("material");
        let (hp_p1, hp_p2)             = self.pair("hp");
        let (armor_p1, armor_p2)       = self.pair("armor");
        let (skills_p1, skills_p2)     = self.pair("skills");
        let (money_p1, money_p2)       = self.pair("money");
        let (mobility_p1, mobility_p2) = self.pair("mobility");
        let (exposure_p1, exposure_p2) = self.pair("exposure");
        let (coverage_p1, coverage_p2) = self.pair("coverage");
        let (tempo_p1, tempo_p2)       = self.pair("tempo");
        let (offensive_range_p1, offensive_range_p2) = self.pair("offensive_range");
        EvalBreakdown {
            material_p1, material_p2, hp_p1, hp_p2, armor_p1, armor_p2,
            skills_p1, skills_p2, money_p1, money_p2, mobility_p1, mobility_p2,
            threat_p1: 0, threat_p2: 0, skill_act_p1: 0, skill_act_p2: 0,
            exposure_p1, exposure_p2, coverage_p1, coverage_p2,
            tempo_p1, tempo_p2, offensive_range_p1, offensive_range_p2,
            total: self.total,
        }
    }
}

/// Per-component decomposition of the static eval (legacy fixed-field view).
/// `total` is exactly what `evaluate()` returns. Per-bucket fields are
/// sign-corrected: P1 contributions to `*_p1`, P2 to `*_p2`, both positive
/// magnitudes. Serialized to the frontend / telemetry — shape is a stable API.
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
    /// Threat term — REMOVED (Pass 4). Always 0. Kept for schema compat.
    pub threat_p1:    i32,
    pub threat_p2:    i32,
    /// Skill-activity — REMOVED (Pass 4++). Always 0. Kept for schema compat.
    pub skill_act_p1: i32,
    pub skill_act_p2: i32,
    /// E2 — exposure penalty (positive magnitude here; subtracts in total).
    pub exposure_p1:  i32,
    pub exposure_p2:  i32,
    /// E6 — bodyguard coverage bonus.
    pub coverage_p1:  i32,
    pub coverage_p2:  i32,
    /// E8 — tempo bonus (side-to-move).
    pub tempo_p1:     i32,
    pub tempo_p2:     i32,
    /// E9 — offensive-range flag (raw max range). Signed with weight in total.
    pub offensive_range_p1: i32,
    pub offensive_range_p2: i32,
    pub total:        i32,
}

// ============================================================
// Per-square diagnostic view — moved verbatim from the pre-ns-43 evaluator.
// Not called from search; drives the frontend hover popup. Its math is
// intentionally independent of the term registry (a self-contained second
// implementation) and is cross-checked against the registry total by the
// `evaluate_by_square_matches_breakdown_*` tests.
// ============================================================

/// Per-square view of the eval, for diagnostic UI. One record per board square.
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

    pub material:      i32,
    pub hp_term:       i32,
    pub armor_term:    i32,
    pub skills_term:   i32,
    pub mobility_term: i32,
    pub exposure_term: i32,
    pub coverage_term: i32,
    pub piece_total:   i32,

    pub skill1_avail_fp: i32,
    pub skill2_avail_fp: i32,
    pub n_attackers:   u8,
    pub n_adj_guards:  u8,
    pub mobility_raw:  u16,
    pub empty_ring_total:    u8,
    pub empty_ring_shielded: u8,
}

impl SquareBreakdown {
    #[inline]
    pub fn owner_signed_total(&self) -> i32 {
        if !self.occupied { 0 }
        else if self.is_p1 { self.piece_total }
        else { -self.piece_total }
    }
}

/// Full per-square eval decomposition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvalBreakdownBySquare {
    pub squares:         Vec<SquareBreakdown>,
    pub p1_money:        u16,
    pub p2_money:        u16,
    pub p1_money_cap:    u16,
    pub p2_money_cap:    u16,
    pub p1_money_term:   i32,
    pub p2_money_term:   i32,
    pub p1_tempo_term:   i32,
    pub p2_tempo_term:   i32,
    pub total:           i32,
    pub terminal:        bool,
}

impl Default for EvalBreakdownBySquare {
    fn default() -> Self {
        Self {
            squares: vec![SquareBreakdown::default(); 64],
            p1_money: 0, p2_money: 0, p1_money_cap: 0, p2_money_cap: 0,
            p1_money_term: 0, p2_money_term: 0, p1_tempo_term: 0, p2_tempo_term: 0,
            total: 0, terminal: false,
        }
    }
}

/// Diagnostic entry point — same math as the aggregate eval, but one record per
/// square. Uses `EvalParams::DEFAULT` (the diagnostic UI always reflects the
/// shipped weights). Invariant: `.total == evaluate_breakdown(pos).total`.
pub fn evaluate_by_square(pos: &Position) -> EvalBreakdownBySquare {
    let params = &EvalParams::DEFAULT;
    let mut out = EvalBreakdownBySquare::default();
    for sq in 0..64u8 {
        out.squares[sq as usize].sq = sq;
    }

    match pos.game_result {
        Some(GameResult::P1Wins) => { out.total = MATE_SCORE;  out.terminal = true; return out; }
        Some(GameResult::P2Wins) => { out.total = -MATE_SCORE; out.terminal = true; return out; }
        None => {}
    }

    let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
    let p1_bb = pos.p1_pieces.0;
    let p2_bb = pos.p2_pieces.0;
    let p1_guards = p1_bb & pos.guards.0;
    let p2_guards = p2_bb & pos.guards.0;

    let p1_avail = side_availability_table(pos.p1_money, params);
    let p2_avail = side_availability_table(pos.p2_money, params);

    let atk = build_attackers_table(pos, all_occ);

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

        let piece_kind: u8 = if is_king { 3 } else if is_champion { 2 } else { 1 };

        let material =
            if      is_king     { params.king_material }
            else if is_champion { params.champion_value }
            else                { params.guard_value };
        let hp_term    = params.hp_per_point    * m.hp()    as i32;
        let armor_term = params.armor_per_point * m.armor() as i32;

        let avail = if is_p1 { &p1_avail } else { &p2_avail };
        let sid1 = m.skill1() as usize;
        let sid2 = m.skill2() as usize;
        let sk1_base = if sid1 < params.skill_value.len() { params.skill_value[sid1] } else { 0 };
        let sk2_base = if sid2 < params.skill_value.len() { params.skill_value[sid2] } else { 0 };
        let sk1_a = if sid1 < avail.len() { avail[sid1] } else { 0 };
        let sk2_a = if sid2 < avail.len() { avail[sid2] } else { 0 };
        let skills_term = (sk1_base * sk1_a + sk2_base * sk2_a) / params.skill_avail_max;

        let own_bb = if is_p1 { p1_bb } else { p2_bb };

        let (mob_score, mob_raw): (i32, u16) = if is_guard {
            let raw = magic::movement_targets_speed2(sq, all_occ).0.count_ones();
            (raw as i32 * params.guard_mob_per_sq, raw as u16)
        } else if is_king {
            let raw = (magic::movement_targets_speed1(sq).0 & !own_bb).count_ones();
            (raw as i32 * params.king_mob_per_sq, raw as u16)
        } else {
            // Champion (ns-43 Term 3a): real movement-space (reachable empty
            // squares), matching terms::Mobility. Enemy-coverage moved to the
            // `champion_threat` term (not itemised in the per-square view yet).
            let raw = (magic::movement_targets_speed1(sq).0 & !all_occ).count_ones();
            (raw as i32 * params.champ_mob_per_sq, raw as u16)
        };

        let opp_attackers_bb = if is_p1 { atk.any_attackers_of(Player::P2, sq) }
                               else     { atk.any_attackers_of(Player::P1, sq) };
        let n_attackers = opp_attackers_bb.count_ones();
        let own_guards = if is_p1 { p1_guards } else { p2_guards };
        let n_adj_guards = (king_expand(mask) & own_guards).count_ones();

        let exposure_term = if is_king {
            params.king_exposure[(n_attackers as usize).min(3)]
        } else {
            let unshielded = (n_attackers.saturating_sub(n_adj_guards) as usize).min(3);
            let mult_pct = params.exposure_mult[unshielded];
            let piece_val = if is_champion { params.champion_value } else { params.guard_value };
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
            let coverage_fp = if denom == 0 { params.skill_avail_max } else { (shielded as i32 * params.skill_avail_max) / denom as i32 };
            let piece_val = params.champion_value;
            (
                (params.coverage_per_piece * piece_val * coverage_fp) / (100 * params.skill_avail_max),
                denom as u8,
                shielded as u8,
            )
        };

        // E11 — guard isolation penalty (folds into total; not itemised in the
        // per-square struct yet). Mirrors terms::GuardIsolation exactly so the
        // `evaluate_by_square.total == evaluate_breakdown.total` invariant holds.
        let guard_iso_pen: i32 = if is_guard {
            let hood = expand_n(mask, params.guard_iso_radius);
            let (own_bb2, opp_bb2) = if is_p1 { (p1_bb, p2_bb) } else { (p2_bb, p1_bb) };
            let enemies_near = (hood & opp_bb2).count_ones() as i32;
            let friendlies_near = (hood & own_bb2 & !mask).count_ones() as i32;
            let outnumber = (enemies_near - friendlies_near).max(0);
            if outnumber == 0 { 0 } else {
                let mut pen = params.guard_iso_per_step * outnumber;
                if params.guard_iso_depth_pct != 100 {
                    let rank = sq / 8;
                    let on_enemy_half = if is_p1 { rank >= 4 } else { rank <= 3 };
                    if on_enemy_half { pen = (pen * params.guard_iso_depth_pct) / 100; }
                }
                pen
            }
        } else { 0 };

        let piece_total = material + hp_term + armor_term + skills_term
            + mob_score + coverage_term - exposure_term - guard_iso_pen;

        // E12 — champion threat (folds into total; not itemised in the
        // per-square struct yet). Mirrors terms::champion_threat_score exactly.
        let champ_threat: i32 = if is_champion {
            super::terms::champion_threat_score(
                pos, params, &atk, sq, m, is_p1, all_occ, p1_bb, p2_bb,
            )
        } else { 0 };
        let piece_total = piece_total + champ_threat;

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

    let p1_max_cost = max_owned_skill_cost(pos, p1_bb);
    let p2_max_cost = max_owned_skill_cost(pos, p2_bb);
    let actions = actions_per_round(pos.current_phase, pos.round_number);
    let p1_cap = p1_max_cost as u16 * actions as u16;
    let p2_cap = p2_max_cost as u16 * actions as u16;
    out.p1_money = pos.p1_money;
    out.p2_money = pos.p2_money;
    out.p1_money_cap = p1_cap;
    out.p2_money_cap = p2_cap;
    out.p1_money_term = useful_money(pos.p1_money, p1_cap, params);
    out.p2_money_term = useful_money(pos.p2_money, p2_cap, params);

    if pos.current_phase != Phase::Draft {
        let tempo = params.tempo_per_action * pos.actions_remaining as i32;
        match pos.to_move {
            Player::P1 => out.p1_tempo_term = tempo,
            Player::P2 => out.p2_tempo_term = tempo,
        }
    }

    let off_p1 = max_offensive_range(pos, p1_bb, pos.p1_money) as i32;
    let off_p2 = max_offensive_range(pos, p2_bb, pos.p2_money) as i32;

    // E13 — endgame closing (folds into total; mirrors terms::endgame_closing_score).
    let (p1_val, p1_mat) = side_value_and_material(pos, p1_bb, params);
    let (p2_val, p2_mat) = side_value_and_material(pos, p2_bb, params);
    let stage = classify_stage(p1_mat + p2_mat, pos.round_number, params);
    let (close_p1, close_p2) =
        super::terms::endgame_closing_score(pos, params, stage, p1_val - p2_val, p1_bb, p2_bb);

    out.total = sum_p1 - sum_p2
        + (out.p1_money_term - out.p2_money_term)
        + (out.p1_tempo_term - out.p2_tempo_term)
        + (off_p1 - off_p2) * params.offensive_range_weight
        + (close_p1 - close_p2);
    out
}
