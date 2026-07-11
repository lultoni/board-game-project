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
//! Params holds only *eval weights* — game rules (skill costs, ranges, phase
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
    /// Champion real movement-space (ns-43 Term 3a): reachable empty squares ×
    /// this. Champions are speed-1. Mild positional value (skill-cast paths,
    /// endgame maneuvering) — not the main event. The former champion
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

    // Wasted-modifier penalty (E10 — ns-43). A live Focus/Charge bit with no
    // castable consumer this Skill phase is money already spent for nothing.
    // Penalty = this × the buff's own skill_cost (Focus 2 / Charge 3), so a
    // wasted Charge (3 money) stings more than a wasted Focus (2 money).
    pub wasted_modifier_per_cost: i32,

    // Guard isolation (E11 — ns-43). A guard that is locally OUTNUMBERED (more
    // enemy pieces than friendly pieces within `guard_iso_radius` Chebyshev
    // tiles) is a hanging piece — the "crammed alone deep among enemies" case
    // that `exposure` (direct-attacker only) misses. Penalty scales by the
    // outnumber count. `guard_iso_depth_pct` optionally amplifies when the
    // guard is past the midline on the enemy's half (100 = neutral/off).
    pub guard_iso_radius:    u8,
    pub guard_iso_per_step:  i32,
    pub guard_iso_depth_pct: i32,

    // Skill availability sigmoid (E4).
    pub skill_avail_k:   i32,
    pub skill_avail_max: i32,

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
        guard_iso_per_step:  120,  // ≈ 0.2 × guard_value per net enemy in the neighbourhood.
        guard_iso_depth_pct: 100,  // neutral: depth amplification off until measured.

        skill_avail_k:   3,
        skill_avail_max: 256,

        // cost×40 + range_bonus{0→0,1→10,2→20,≥3→30} + cat_bonus{Strike→30,Move→20,Shield→15,Mystic→10}.
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
