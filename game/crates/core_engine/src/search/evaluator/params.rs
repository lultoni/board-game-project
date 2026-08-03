//! Data-driven eval weights (ns-43). Every tunable constant the evaluator
//! uses lives here as a field of [`EvalParams`], so an offline tuner can
//! perturb them and the term registry can read them by borrow.
//!
//! **Faithful-port invariant:** `EvalParams::DEFAULT` reproduces the exact
//! constant values the pre-ns-43 flat evaluator used. The `golden_eval_unchanged`
//! test in the parent module enforces byte-identical output. When a deliberate
//! balance change lands later, update these defaults AND re-capture the goldens
//! in the same commit.
//!
//! Params holds only *eval weights* - game rules (skill costs, ranges, phase
//! budgets) stay pulled live from `game_logic::skills` / `make_unmake`, never
//! restated here.

/// All tunable eval weights. `Copy` so terms can hold it cheaply; `serde` so
/// the future tuner can round-trip candidate params to disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvalParams {
    // Material (E-material).
    /// King material weight = 0: presence/absence is already the MATE branch.
    pub king_material:  i32,
    pub champion_value: i32,
    pub guard_value:    i32,
    pub hp_per_point:    i32,
    pub armor_per_point: i32,
    pub money_per_unit:  i32,

    // Mobility (E7).
    pub guard_mob_per_sq:         i32,
    pub champ_skill_cov_per_enemy: i32,
    pub champ_skill_cov_cap:       i32,
    pub king_mob_per_sq:          i32,
    /// Champion real movement-space (ns-43 Term 3a): reachable empty squares x
    /// this. Champions are speed-1. Mild positional value (skill-cast paths,
    /// endgame maneuvering) - not the main event. The former champion
    /// "mobility" (enemies-in-range) moved to the `champion_threat` term.
    pub champ_mob_per_sq:         i32,

    // Exposure (E2).
    /// % of piece_val (÷100), indexed by min(unshielded_attackers, 3).
    pub exposure_mult:  [i32; 4],
    /// King exposure curve, indexed directly by min(n_attackers, 3).
    pub king_exposure:  [i32; 4],

    // Coverage (E6).
    /// % of piece_val at full bodyguard coverage.
    pub coverage_per_piece: i32,

    // Tempo (E8).
    pub tempo_per_action: i32,

    // Offensive-range (E9).
    pub offensive_range_weight: i32,

    // Wasted-modifier penalty (E10 - ns-43). A live Focus/Charge bit with no
    // castable consumer this Skill phase is money already spent for nothing.
    // Penalty = this x the buff's own skill_cost (Focus 2 / Charge 3), so a
    // wasted Charge (3 money) stings more than a wasted Focus (2 money).
    pub wasted_modifier_per_cost: i32,

    // Guard isolation (E11 - ns-43). A guard that is locally OUTNUMBERED (more
    // enemy pieces than friendly pieces within `guard_iso_radius` Chebyshev
    // tiles) is a hanging piece - the "crammed alone deep among enemies" case
    // that `exposure` (direct-attacker only) misses. Penalty scales by the
    // outnumber count. `guard_iso_depth_pct` optionally amplifies when the
    // guard is past the midline on the enemy's half (100 = neutral/off).
    pub guard_iso_radius:    u8,
    pub guard_iso_per_step:  i32,
    pub guard_iso_depth_pct: i32,

    // Champion threat (E12 - ns-43 Term 3b). Two symmetric sub-scores per
    // champion: OFFENSIVE (enemy pieces its Strike/Shove/Blast skills can hit,
    // value-weighted, safety-scaled) + DEFENSIVE (ally pieces its Heal/Plate/
    // Swap skills can reach, value+vulnerability-weighted). Each soft-capped so
    // neither category dominates. Target values reuse material weights but are
    // scaled down by `threat_value_shift` (a right-shift) so a champion in
    // range of an enemy champion isn't worth another whole champion.
    pub threat_offensive_weight: i32, // x/100 applied to the offensive sub-score
    pub threat_defensive_weight: i32, // x/100 applied to the defensive sub-score
    pub threat_value_shift:      u32, // right-shift on target material value
    pub threat_safety_penalty_pct: i32, // % kept when a strike's landing sq is unsafe
    pub threat_softcap:          i32, // saturation ceiling per sub-score (hyperbola k)
    pub threat_king_bonus:       i32, // flat extra for threatening the enemy king

    // Game-stage thresholds (ns-43 stage infra). Total on-board material (both
    // sides, using material weights only) is the primary stage signal; a high
    // round_number biases the stage later via `stage_round_bias` (material
    // "credited" per round elapsed). `Opening` above `stage_mid_threshold`,
    // `End` below `stage_end_threshold`, `Mid` between. Anchored so the
    // full-material Stack-M opening classifies as Opening.
    pub stage_mid_threshold: i32,
    pub stage_end_threshold: i32,
    pub stage_round_bias:    i32,

    // Endgame closing (E13 - ns-43 Term 4). Active only when stage == End.
    // Asymmetric by `advantage`: the LEADER is rewarded for closing (king
    // pressure + denying the enemy king escape squares), the TRAILER for
    // stalling (own-king safety + compactness). `close_lead_min` is the
    // advantage magnitude below which the position is "even" and the term stays
    // neutral (no forced aggression in a dead-even endgame).
    pub close_lead_min:         i32,
    pub close_king_pressure:    i32, // leader: x (7 - dist(nearest threatener → enemy king))
    pub close_escape_denial:    i32, // leader: x (8 - enemy-king escape squares)
    pub defend_king_safety:     i32, // trailer: x (dist(nearest enemy threatener → own king))
    pub defend_compactness:     i32, // trailer: x (own pieces adjacent to own king)

    // Skill availability sigmoid (E4).
    pub skill_avail_k:   i32,
    pub skill_avail_max: i32,

    // Hanging-piece SEE term (ns-53). For a piece that is CURRENTLY attacked by
    // an enemy (cheap physical-attacker gate), run the SEE exchange on its
    // square; when the exchange is losing for the owner, penalise by
    // `hanging_penalty_pct` % of the SEE-loss magnitude. Only attacked pieces
    // pay the SEE cost — quiet pieces are skipped, so the hot path is unchanged
    // for positions with no live threats (the same positions the search's own
    // QS/SEE would skip).
    pub hanging_penalty_pct: i32,

    // King-tempo SEE term (ns-53). A flat penalty on a side whose king is one
    // loud enemy action from being captured (reuses `is_king_threatened`, a
    // cheap bitboard scan — no exchange rollout). Distinct from `king_exposure`
    // (attacker-count curve): this is the specific "one tempo from death" flag.
    pub king_tempo_penalty: i32,

    // Per-skill base value (E4), indexed by `Skill as u8` (0 = unequipped).
    pub skill_value: [i32; 16],
}

impl EvalParams {
    /// The pre-ns-43 constant values. `Default` returns this.
    pub const DEFAULT: EvalParams = EvalParams {
        king_material:  0,
        champion_value: 1000,
        guard_value:    600,
        hp_per_point:    150,
        armor_per_point: 120,
        money_per_unit:  25,

        guard_mob_per_sq:          4,
        champ_skill_cov_per_enemy: 10,
        champ_skill_cov_cap:       60,
        king_mob_per_sq:           6,
        champ_mob_per_sq:          4,  // ns-43 Term 3a: champion real movement-space.

        exposure_mult: [0, 10, 30, 55],
        king_exposure: [0, 800, 2400, 4000],

        coverage_per_piece: 30,

        tempo_per_action: 15,

        offensive_range_weight: 500,

        wasted_modifier_per_cost: 25, // ≈ money_per_unit: a wasted buff ≈ its money burned.

        guard_iso_radius:    2,
        guard_iso_per_step:  120,  // ≈ 0.2 x guard_value per net enemy in the neighbourhood.
        guard_iso_depth_pct: 100,  // neutral: depth amplification off until measured.

        // Champion threat (Term 3b). Target values are material >> shift (4):
        // champion 1000>>4 ≈ 62, guard 600>>4 ≈ 37 per target in range. Weights
        // x/100. Soft cap ~200 per sub-score keeps a champ's threat contribution
        // in the low-hundreds (comparable to coverage/mobility, below material).
        threat_offensive_weight:   100,
        threat_defensive_weight:   100,
        threat_value_shift:        4,
        threat_safety_penalty_pct: 40,  // unsafe strike keeps 40% of its offensive value.
        threat_softcap:            200,
        threat_king_bonus:         80,

        // Stage thresholds. Full Stack-M opening = 17200 total material (2 x
        // (5x1000 + 6x600)) → Opening. Mid once ~40% of material is gone; End
        // once ~65% is gone. Each round elapsed credits stage_round_bias toward
        // "later" so a long game trends to End even at higher material.
        stage_mid_threshold: 10000,
        stage_end_threshold: 6000,
        stage_round_bias:    150,

        // Endgame closing. Only fires in the End stage; magnitudes kept modest
        // (low-hundreds) so material still dominates - this shapes HOW a won/
        // lost endgame is played, it doesn't invent advantages.
        close_lead_min:      400,  // < ~0.7 guard of lead → treat as even, stay neutral.
        close_king_pressure: 40,   // per tile closer the nearest threatener is to enemy king.
        close_escape_denial: 30,   // per denied enemy-king escape square.
        defend_king_safety:  30,   // per tile the nearest enemy threatener is from own king.
        defend_compactness:  25,   // per own piece hugging own king.

        skill_avail_k:   3,
        skill_avail_max: 256,

        // SEE terms (ns-53). Hanging: penalise 60% of the exchange loss on an
        // attacked, SEE-losing piece — enough to make the AI defend/retreat it
        // without treating a soft-loss as certain material down. King-tempo: a
        // firm flat penalty (a king one action from capture is a near-loss).
        hanging_penalty_pct: 60,
        king_tempo_penalty:  600,

        // costx40 + range_bonus{0→0,1→10,2→20,≥3→30} + cat_bonus{Strike→30,Move→20,Shield→15,Mystic→10}.
        skill_value: [
              0, // 0  unequipped
            120, // 1  Lance
            170, // 2  Hook
            130, // 3  Break
            210, // 4  Steal
            210, // 5  Tempest
             95, // 6  Shield
            145, // 7  Heal
            145, // 8  Plate
            160, // 9  Dash
            120, // 10 Blast
            170, // 11 Shove
            200, // 12 Swap
            210, // 13 Retreat
             50, // 14 Focus
            130, // 15 Charge
        ],
    };
}

impl Default for EvalParams {
    fn default() -> Self { EvalParams::DEFAULT }
}
