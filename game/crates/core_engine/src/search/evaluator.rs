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
use crate::state::position::{GameResult, Player};
use crate::state::magic;
use crate::search::counters;
use crate::game_logic::skills::{
    Skill, SkillCategory, TargetOwner, skill_from_id, skill_cost, skill_category,
    skill_default_range, skill_target_owner,
};

pub const MATE_SCORE: i32 = 1_000_000;

/// Precomputed 1 << sq lookup for the 64 board squares. Used in hot bit-set
/// operations inside the evaluator to avoid re-materialising `1u64 << sq`
/// under a runtime shift on every access.
const SQ_BIT: [u64; 64] = {
    let mut t = [0u64; 64];
    let mut i = 0usize;
    while i < 64 { t[i] = 1u64 << i; i += 1; }
    t
};

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

// Mobility scoring: reward pieces for having reachable squares.
// Guards use BFS-2 (speed=2) discounting occupied squares; Champions/Kings
// use the 8-adjacent mask discounting own pieces.  Weights are small relative
// to material so positional advantage doesn't overshadow piece count.
const GUARD_MOB_PER_SQ:   i32 = 8;   // centre Guard (20 reachable) ≈ 160 pts
const CHAMP_MOB_PER_SQ:   i32 = 12;  // centre Champ (8 free) ≈ 96 pts

// Skill-activity weights. Kept small relative to mobility (which is 8–12/sq)
// so they nudge play toward useful casts without swamping material.
const STRIKE_PER_TARGET:  i32 = 6;   // per enemy in Strike range
const MOVE_PER_DEST:      i32 = 3;   // per legal destination (Dash/Retreat) or per pushable target (Shove/Blast) or per swap partner
const SHIELD_PER_TARGET:  i32 = 5;   // per Heal/Plate ally that would actually benefit
const SHIELD_SELF:        i32 = 5;   // Shield if own armor < cap
const MYSTIC_FLAG_BONUS:  i32 = 20;  // per Focus/Charge that has a real follow-up this turn

// Stack M caps.
const ARMOR_CAP:          u8 = 2;
const HP_CAP:             u8 = 2;

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

/// Convenience for the hot loop: value from a mailbox skill id (0..=15) with
/// a single bounds-checked table load. Returns 0 for the unequipped sentinel.
#[inline]
fn skill_value_from_id(id: u8) -> i32 {
    if (id as usize) < SKILL_VALUE.len() { SKILL_VALUE[id as usize] } else { 0 }
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
    /// Pre-priced "hanging piece" credit: 0.25× the value of enemy pieces
    /// this side could move-attack this turn. Symmetric across sides so the
    /// leaf eval doesn't flip sign with the side-to-move — the attacker's
    /// pending gain is already reflected before the capture ply resolves.
    pub threat_p1:    i32,
    pub threat_p2:    i32,
    /// Active-skill activity credit: per-target for Strike/Move/Shield (money-
    /// and legality-gated), single flag for Mystic (Focus/Charge) gated on an
    /// affordable follow-on active skill actually having ≥1 legal action.
    pub skill_act_p1: i32,
    pub skill_act_p2: i32,
    pub total:        i32,
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

    // (b) Single pass over occupied bits.
    let mut bits = all_occ;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let m = pos.mailbox[sq as usize];
        let is_p1 = pos.p1_pieces.0 & mask != 0;

        let is_guard    = pos.guards.0    & mask != 0;

        let material =
            if      pos.kings.0     & mask != 0 { KING_MATERIAL }
            else if pos.champions.0 & mask != 0 { CHAMPION_VALUE }
            else                                { GUARD_VALUE };
        let hp_term    = HP_PER_POINT    * m.hp()    as i32;
        let armor_term = ARMOR_PER_POINT * m.armor() as i32;
        let skill_term = skill_value_from_id(m.skill1()) + skill_value_from_id(m.skill2());

        // Mobility: count squares the piece can actually reach given board state.
        // Guards: BFS-2 discounting all occupied squares.
        // Champions/Kings: 8-adjacent discounting own pieces.
        let own_bb = if is_p1 { pos.p1_pieces.0 } else { pos.p2_pieces.0 };
        let mob_score = if is_guard {
            magic::movement_targets_speed2(sq, all_occ).0.count_ones() as i32
                * GUARD_MOB_PER_SQ
        } else {
            // Champion and King: 8-adjacent minus own pieces
            (magic::movement_targets_speed1(sq).0 & !own_bb).count_ones() as i32
                * CHAMP_MOB_PER_SQ
        };

        if is_p1 {
            b.material_p1  += material;
            b.hp_p1        += hp_term;
            b.armor_p1     += armor_term;
            b.skills_p1    += skill_term;
            b.mobility_p1  += mob_score;
        } else {
            b.material_p2  += material;
            b.hp_p2        += hp_term;
            b.armor_p2     += armor_term;
            b.skills_p2    += skill_term;
            b.mobility_p2  += mob_score;
        }
    }

    // (c) Money is global, not per-square.
    b.money_p1 = MONEY_PER_UNIT * pos.p1_money as i32;
    b.money_p2 = MONEY_PER_UNIT * pos.p2_money as i32;

    // (d) Threat-symmetric term (MAEE): pre-priced net-of-exchange value for
    // each capturable enemy piece. Priced unconditionally — the pricing is
    // phase-invariant (it reads bitboards + mailbox HP/armor, no phase input),
    // and gating it to Phase::Move created a horizon-effect cliff at every
    // phase transition. Cost is bounded: ~12 targets × ~10 attacker enums
    // × ~700 ops ≈ ~10k ops per side per leaf, no recursion, hard 32-ply cap.
    // Audit: game/crates/core_engine/src/search/evaluator.rs MAEE section +
    // .claude/eval-perf-passes.md Pass 2 log.
    //
    // (e) Skill-activity: same treatment — skill *potential* (has money, has
    // range) is phase-invariant. Gating to Phase::Skill produced the same
    // horizon cliff.
    //
    // NOTE: a Pass 1 short-circuit that zeroed the side-to-move's threat_*
    // and skill_act_* when actions_remaining == 0 was reverted in Pass 2 —
    // it caused ~30-70% node explosions on multiple positions because it
    // created a large asymmetric leaf-eval discontinuity that perturbed
    // move ordering.
    counters::bump_maee_gate_pass();
    let attackers_table = build_attackers_table(pos, all_occ);
    b.threat_p1 = maee_side(pos, Player::P1, &attackers_table);
    b.threat_p2 = maee_side(pos, Player::P2, &attackers_table);
    counters::bump_skill_gate_pass();
    b.skill_act_p1 = skill_activity(pos, Player::P1);
    b.skill_act_p2 = skill_activity(pos, Player::P2);
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
        (b.threat_p1   - b.threat_p2)   +
        (b.skill_act_p1 - b.skill_act_p2);
    b
}

// ── MAEE: Move-Attack Exchange Evaluation ────────────────────────────────
//
// Static-exchange evaluation adapted to this game's rules. Prices a single
// enemy square by simulating the swap-off sequence of move-attackers on that
// square, LVA-first, with stand-pat fold-back.
//
// Differences from chess SEE:
//   1. HP/armor multi-hit: one move-attack deposits 1 damage; killing blow
//      only lands after armor→HP is fully drained.
//   2. Kill-follow-through: attacker enters target square on the killing
//      blow. Subsequent attackers are attacking the new occupant.
//   3. No sliders → no X-ray reveals when an attacker vacates its origin
//      (Guards' BFS-2 can gain reach — we fully re-enumerate after each kill).
//
// Kings are EXCLUDED as both attackers and targets: king captures resolve as
// MATE_SCORE terminals upstream, and a king "threatening" a piece it can't
// actually capture without dying was the root cause of the AI-king-forward
// bias this term is designed to fix.

// Geometric ceiling for attackers-of-a-square in this game: the Chebyshev-1
// ring around any square has 8 slots (the only squares from which a Champion
// can move-attack). Guards must BFS-2 through empty squares to reach an
// adjacent square of the target, and each Guard on a Chebyshev-1 square
// blocks BFS entry for outer Guards — so filling more than 8 attackers
// requires occupying more than 8 adjacent-or-nearby squares AND leaving BFS
// paths open, which are mutually exclusive. Realistic in-game max is 3-5.
const MAEE_MAX_ATTACKERS: usize = 8;
// Geometric ceiling: 2 sides × MAEE_MAX_ATTACKERS = 16 total plies possible.
const MAEE_MAX_PLIES: usize = 16;

/// One attacker candidate: (cost, source-square).
/// Cost fits in i16: max is CHAMPION_VALUE(1000) + 2·HP_PER_POINT(300) +
/// 2·ARMOR_PER_POINT(240) = 1540. Packs to 4 bytes with 1 byte of padding.
#[derive(Copy, Clone)]
struct Attacker {
    cost: i16,
    sq: u8,
}

/// Fixed-size sorted-cheapest-first list. Avoids heap allocation on the hot
/// eval path. Total struct size = 8 · 4 B + 1 B (len) + padding ≈ 36 B.
struct AttackerList {
    items: [Attacker; MAEE_MAX_ATTACKERS],
    len: u8,
}

impl AttackerList {
    #[inline]
    fn new() -> Self {
        Self { items: [Attacker { cost: 0, sq: 0 }; MAEE_MAX_ATTACKERS], len: 0 }
    }
    /// Insertion-sort push. Drops the most expensive on overflow.
    #[inline]
    fn push(&mut self, a: Attacker) {
        let len = self.len as usize;
        // Find insert position.
        let mut i = 0;
        while i < len && self.items[i].cost <= a.cost { i += 1; }
        if len < MAEE_MAX_ATTACKERS {
            // Shift right.
            let mut j = len;
            while j > i {
                self.items[j] = self.items[j - 1];
                j -= 1;
            }
            self.items[i] = a;
            self.len += 1;
        } else if i < MAEE_MAX_ATTACKERS {
            // Full — but new attacker cheaper than the most expensive slot.
            // Shift-right dropping the last.
            let mut j = MAEE_MAX_ATTACKERS - 1;
            while j > i {
                self.items[j] = self.items[j - 1];
                j -= 1;
            }
            self.items[i] = a;
        }
        // else: full AND new attacker is more expensive than everyone → drop.
    }
    /// Pop the cheapest (front).
    #[inline]
    fn pop_front(&mut self) -> Option<Attacker> {
        if self.len == 0 { return None; }
        let out = self.items[0];
        let len = self.len as usize;
        for i in 1..len { self.items[i - 1] = self.items[i]; }
        self.len -= 1;
        Some(out)
    }
}

// ── Precomputed "who attacks square S" table ──────────────────────────────
//
// Built once at eval entry against the current occupancy. `p1_of[t]` is the
// bitmask of P1 non-king pieces that can move-attack square `t`; symmetric for
// `p2_of`. Read by `maee` at the top of each per-target repricing loop via
// `attackers_bb_from_table`.
//
// Kill-triggered re-enumerations (Guard adjacency gained when a blocker
// vacates) are handled incrementally inside `maee`: the killed attacker's
// bit is cleared, and if the vacated origin sits cheby-1 of the target,
// Guards adjacent to it get OR'd in. This avoids the full 64-square scan of
// the from-scratch path.

struct AttackersTable {
    p1_of: [u64; 64],
    p2_of: [u64; 64],
}

/// King-expand: bitboard OR of all 8-directional 1-step neighbours. Used to
/// project a Guard's reach-set into its move-attack fanout.
#[inline]
fn king_expand(x: u64) -> u64 {
    const NOT_A: u64 = 0xfefefefefefefefe; // !file A
    const NOT_H: u64 = 0x7f7f7f7f7f7f7f7f; // !file H
    let l = (x & NOT_A) >> 1;
    let r = (x & NOT_H) << 1;
    let h = x | l | r;
    h | (h << 8) | (h >> 8)
}

/// Build the per-side attackers table for the current position.
///
/// For each non-king piece at `sq`, computes the bitmask of squares it can
/// move-attack given `all_occ`, then transposes: for each attackable target
/// `t`, sets bit `sq` in `of[t]`.
#[inline]
fn build_attackers_table(pos: &Position, all_occ: u64) -> AttackersTable {
    let mut table = AttackersTable { p1_of: [0u64; 64], p2_of: [0u64; 64] };

    // P1 side.
    let mut bits = pos.p1_pieces.0 & !pos.kings.0;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let is_guard = pos.guards.0 & sq_bit != 0;
        let attacks = if is_guard {
            // Guard move-attack (game rule per generator.rs:473-512): approach
            // ≤ speed-1 steps then attack cheby-1. So the landing set is
            // {src ∪ empty cheby-1 neighbours of src}, and the attack fanout is
            // king_expand(landing) minus src itself. This is naturally within
            // cheby-2 of src — no extra mask needed.
            let approach = (magic::movement_targets_speed1(sq).0 & !all_occ) | sq_bit;
            king_expand(approach) & !sq_bit
        } else {
            // Champion: 8 immediate neighbours.
            magic::movement_targets_speed1(sq).0
        };
        let mut a = attacks;
        while a != 0 {
            let t = a.trailing_zeros() as usize;
            a &= a - 1;
            table.p1_of[t] |= sq_bit;
        }
    }

    // P2 side.
    let mut bits = pos.p2_pieces.0 & !pos.kings.0;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let is_guard = pos.guards.0 & sq_bit != 0;
        let attacks = if is_guard {
            let approach = (magic::movement_targets_speed1(sq).0 & !all_occ) | sq_bit;
            king_expand(approach) & !sq_bit
        } else {
            magic::movement_targets_speed1(sq).0
        };
        let mut a = attacks;
        while a != 0 {
            let t = a.trailing_zeros() as usize;
            a &= a - 1;
            table.p2_of[t] |= sq_bit;
        }
    }

    table
}

/// Attacker-bitmask lookup: which squares of `side` currently attack `target_sq`
/// according to the (initial) table. Callers pair this with `build_attacker_list`
/// to get a sorted list, and maintain the bitmask incrementally across kills.
#[inline]
fn attackers_bb_from_table(side: Player, target_sq: u8, table: &AttackersTable) -> u64 {
    let bits = match side {
        Player::P1 => table.p1_of[target_sq as usize],
        Player::P2 => table.p2_of[target_sq as usize],
    };
    // Exclude the target square itself (a piece can't move-attack its own square).
    bits & !SQ_BIT[target_sq as usize]
}

/// Build the sorted-cheapest-first `AttackerList` from a bitmask of attacker
/// squares. Reads material/HP/armor per bit from `pos.mailbox`.
#[inline]
fn build_attacker_list(pos: &Position, mut bits: u64) -> AttackerList {
    counters::bump_enumerate_attackers_calls();
    let mut out = AttackerList::new();
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let m = pos.mailbox[sq as usize];
        let mat = if pos.champions.0 & sq_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
        let cost = attacker_cost(mat, m.hp(), m.armor());
        out.push(Attacker { cost, sq });
    }
    counters::record_attacker_list_len(out.len as usize);
    out
}

#[inline]
fn attacker_cost(mat: i32, hp: u8, armor: u8) -> i16 {
    (mat + HP_PER_POINT * hp as i32 + ARMOR_PER_POINT * armor as i32) as i16
}

#[inline]
fn piece_material_of(pos: &Position, sq: u8) -> i32 {
    let bit = SQ_BIT[sq as usize];
    if pos.champions.0 & bit != 0 { CHAMPION_VALUE }
    else if pos.guards.0 & bit != 0 { GUARD_VALUE }
    else { KING_MATERIAL } // shouldn't reach — kings excluded upstream
}

/// From-scratch attacker enumeration for a target square. Used only as the
/// reference oracle inside `#[cfg(feature = "maee_paranoid")]` blocks — the
/// hot path in `maee` maintains an attacker bitmask incrementally across
/// kills instead. Gated behind the feature so release builds don't compile
/// this at all.
#[cfg(feature = "maee_paranoid")]
#[inline]
fn enumerate_attackers(
    pos: &Position,
    side: Player,
    target_sq: u8,
    vacated: u64,
) -> AttackerList {
    counters::bump_enumerate_attackers_calls();
    let own_bb = match side {
        Player::P1 => pos.p1_pieces.0,
        Player::P2 => pos.p2_pieces.0,
    };
    let attackers_pool = own_bb & !pos.kings.0 & !vacated;
    let all_occ = (pos.p1_pieces.0 | pos.p2_pieces.0) & !vacated;
    let target_bit = SQ_BIT[target_sq as usize];

    let mut out = AttackerList::new();
    let mut bits = attackers_pool;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        if sq == target_sq { continue; }
        let sq_bit = SQ_BIT[sq as usize];

        let is_guard = pos.guards.0 & sq_bit != 0;
        let can_attack = if is_guard {
            // Game rule: approach ≤ 1 step (empty cheby-1 neighbours of src)
            // plus src itself, then attack cheby-1.
            let approach = (magic::movement_targets_speed1(sq).0 & !all_occ) | sq_bit;
            king_expand(approach) & target_bit != 0
        } else {
            // Champion (non-king): 8-adjacency.
            magic::movement_targets_speed1(sq).0 & target_bit != 0
        };
        if !can_attack { continue; }

        let m = pos.mailbox[sq as usize];
        let mat = if pos.champions.0 & sq_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
        let cost = attacker_cost(mat, m.hp(), m.armor());
        out.push(Attacker { cost, sq });
    }
    counters::record_attacker_list_len(out.len as usize);
    out
}

/// MAEE for a single target square. Returns net material delta from the
/// initiator's POV (initiator = enemy of target's owner). Positive = the
/// initiator gains, negative = losing trade.
#[inline]
fn maee(pos: &Position, target_sq: u8, table: &AttackersTable) -> i32 {
    counters::bump_maee_target_calls();
    let target_bit = SQ_BIT[target_sq as usize];
    let target_is_p1 = pos.p1_pieces.0 & target_bit != 0;
    let stm = if target_is_p1 { Player::P2 } else { Player::P1 };

    let mut victim_val = piece_material_of(pos, target_sq);
    let entry = pos.mailbox[target_sq as usize];
    let mut victim_hp = entry.hp();
    let mut victim_armor = entry.armor();

    let mut vacated = 0u64;
    // Initial enumeration reads from the precomputed table (vacated == 0).
    // Maintain the underlying attacker bitmasks alongside the sorted lists so
    // we can update incrementally on each kill instead of re-enumerating from
    // scratch. Only Guards gain new reach when a blocker vacates — Champions
    // are geometry-invariant, so all newly-reachable attackers are Guards
    // adjacent to the vacated square (which itself must be cheby-1 of the
    // target for the freshly-empty square to serve as a valid approach).
    let mut attackers_stm_bb = attackers_bb_from_table(stm, target_sq, table);
    let mut attackers_dfd_bb = attackers_bb_from_table(other(stm), target_sq, table);
    let mut attackers_stm = build_attacker_list(pos, attackers_stm_bb);
    let mut attackers_dfd = build_attacker_list(pos, attackers_dfd_bb);
    // Cheby-1 neighbourhood of the target — a vacated square must live here
    // for it to newly enable Guards to attack the target via that empty as
    // an approach step.
    let target_cheby1 = king_expand(target_bit);

    // Correctness canary: table-driven initial enumeration must match the
    // from-scratch result exactly. Gated behind a feature (not `debug_assertions`)
    // because the from-scratch comparison quadruples eval cost — impractical for
    // normal `cargo test` cycles. Enable with `--features maee_paranoid` when
    // touching table-build logic.
    #[cfg(feature = "maee_paranoid")]
    {
        let ref_stm = enumerate_attackers(pos, stm, target_sq, 0);
        let ref_dfd = enumerate_attackers(pos, other(stm), target_sq, 0);
        assert_eq!(attackers_stm.len, ref_stm.len,
            "attackers_stm len mismatch at target {}", target_sq);
        assert_eq!(attackers_dfd.len, ref_dfd.len,
            "attackers_dfd len mismatch at target {}", target_sq);
        for i in 0..attackers_stm.len as usize {
            assert_eq!(attackers_stm.items[i].sq, ref_stm.items[i].sq);
            assert_eq!(attackers_stm.items[i].cost, ref_stm.items[i].cost);
        }
        for i in 0..attackers_dfd.len as usize {
            assert_eq!(attackers_dfd.items[i].sq, ref_dfd.items[i].sq);
            assert_eq!(attackers_dfd.items[i].cost, ref_dfd.items[i].cost);
        }
    }

    // Signed per-ply gains from stm's POV.
    let mut gains = [0i32; MAEE_MAX_PLIES];
    let mut n_gains = 0usize;
    let mut side = stm;

    loop {
        let att_opt = if side == stm {
            attackers_stm.pop_front()
        } else {
            attackers_dfd.pop_front()
        };
        let Some(att) = att_opt else { break };
        if n_gains >= MAEE_MAX_PLIES { break; }

        let sign = if side == stm { 1 } else { -1 };
        if victim_armor > 0 {
            victim_armor -= 1;
            gains[n_gains] = sign * ARMOR_PER_POINT;
            n_gains += 1;
        } else if victim_hp > 1 {
            victim_hp -= 1;
            gains[n_gains] = sign * HP_PER_POINT;
            n_gains += 1;
        } else {
            // Killing blow.
            let kill_gain = victim_val + HP_PER_POINT + ARMOR_PER_POINT * victim_armor as i32;
            gains[n_gains] = sign * kill_gain;
            n_gains += 1;

            // Attacker now occupies target_sq; its origin is vacated.
            let att_bit = SQ_BIT[att.sq as usize];
            vacated |= att_bit;
            let att_entry = pos.mailbox[att.sq as usize];
            let att_mat = if pos.champions.0 & att_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
            victim_val = att_mat;
            victim_hp = att_entry.hp();
            victim_armor = att_entry.armor();

            // Incremental attacker-set maintenance.
            //
            // 1) The killed attacker moved off `att.sq` — clear its bit from
            //    whichever side's mask it belonged to. (`side` is the attacker
            //    who just moved.)
            // 2) If the vacated origin sits cheby-1 of the target, Guards
            //    adjacent to it may now use it as an approach square. Add any
            //    such Guards not already tracked.
            //
            // Champions need no addition step: their reach is 8-adjacency of
            // their own square and does not depend on occupancy.
            if side == stm {
                attackers_stm_bb &= !att_bit;
            } else {
                attackers_dfd_bb &= !att_bit;
            }

            if target_cheby1 & att_bit != 0 {
                // Guards adjacent to the vacated square that weren't already
                // in each side's attacker set. Mask by side ownership and
                // exclude anything already vacated (dead attackers sitting on
                // target don't attack, and their origins are also gone); also
                // exclude the target square itself (the current victim on it
                // cannot attack itself).
                let neigh = king_expand(att_bit) & pos.guards.0 & !vacated & !target_bit;
                let stm_own = match stm {
                    Player::P1 => pos.p1_pieces.0,
                    Player::P2 => pos.p2_pieces.0,
                };
                let dfd_own = match other(stm) {
                    Player::P1 => pos.p1_pieces.0,
                    Player::P2 => pos.p2_pieces.0,
                };
                attackers_stm_bb |= neigh & stm_own & !attackers_stm_bb;
                attackers_dfd_bb |= neigh & dfd_own & !attackers_dfd_bb;
            }

            attackers_stm = build_attacker_list(pos, attackers_stm_bb);
            attackers_dfd = build_attacker_list(pos, attackers_dfd_bb);

            // Correctness canary for the incremental update: on every kill,
            // the incrementally-maintained list must match a from-scratch
            // enumeration against the current `vacated` set.
            #[cfg(feature = "maee_paranoid")]
            {
                let ref_stm = enumerate_attackers(pos, stm, target_sq, vacated);
                let ref_dfd = enumerate_attackers(pos, other(stm), target_sq, vacated);
                assert_eq!(attackers_stm.len, ref_stm.len,
                    "post-kill attackers_stm len mismatch at target {} (vacated={:#x})",
                    target_sq, vacated);
                assert_eq!(attackers_dfd.len, ref_dfd.len,
                    "post-kill attackers_dfd len mismatch at target {} (vacated={:#x})",
                    target_sq, vacated);
                for i in 0..attackers_stm.len as usize {
                    assert_eq!(attackers_stm.items[i].sq, ref_stm.items[i].sq);
                    assert_eq!(attackers_stm.items[i].cost, ref_stm.items[i].cost);
                }
                for i in 0..attackers_dfd.len as usize {
                    assert_eq!(attackers_dfd.items[i].sq, ref_dfd.items[i].sq);
                    assert_eq!(attackers_dfd.items[i].cost, ref_dfd.items[i].cost);
                }
            }
        }

        side = other(side);
    }

    if n_gains == 0 { return 0; }

    // Stand-pat fold-back, single-pass right-to-left in a scalar accumulator.
    // Each side may refuse their last exchange step if it's bad for them:
    // stm plies (even indices) clamp low at 0, dfd plies (odd indices) clamp
    // high at 0. Ply 0 is stm's first attack; we return its raw value (caller
    // discards non-positive maee anyway).
    let mut val: i32 = gains[n_gains - 1];
    for i in (0..n_gains - 1).rev() {
        let ply_after = i + 1;
        val = if (ply_after & 1) == 0 { val.max(0) } else { val.min(0) };
        val += gains[i];
    }
    val
}

#[inline]
fn other(p: Player) -> Player {
    match p { Player::P1 => Player::P2, Player::P2 => Player::P1 }
}

/// Sum of MAEE credits for `side` over all enemy non-king targets `side`
/// could move-attack this turn. Per-square results clamped at 0 — a losing
/// exchange contributes nothing (we don't reward not-attacking).
#[inline]
fn maee_side(pos: &Position, side: Player, table: &AttackersTable) -> i32 {
    counters::bump_maee_side_calls();
    let opp_bb = match side {
        Player::P1 => pos.p2_pieces.0,
        Player::P2 => pos.p1_pieces.0,
    };
    let non_kings = (pos.p1_pieces.0 | pos.p2_pieces.0) & !pos.kings.0;
    let enemy_targets = opp_bb & non_kings;

    // Candidate targets: enemy non-kings that `side` has at least one attacker
    // for. Derived from the precomputed table — replaces the old `threat_bb`
    // call that recomputed the same information from scratch.
    let side_of = match side {
        Player::P1 => &table.p1_of,
        Player::P2 => &table.p2_of,
    };

    let mut acc = 0i32;
    let mut bits = enemy_targets;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        if side_of[sq as usize] == 0 { continue; }
        let v = maee(pos, sq, table);
        if v > 0 { acc += v; }
    }
    acc
}


/// Skill-activity term for one side. Only credits skills that could actually
/// be used this turn: caster affords the cost AND ≥1 legal target/destination
/// exists. Mystic (Focus/Charge) get a single flag bonus gated on the caster
/// having an affordable, legally-usable follow-up active skill this turn.
///
/// Cost budget: for each equipped skill on each of `side`'s pieces, we call
/// `magic::skill_attacks` once (O(1)) and count set bits in the result. That's
/// ~24 pieces × 2 slots × O(1) = ~48 lookups per leaf. Cheap.
#[inline]
fn skill_activity(pos: &Position, side: Player) -> i32 {
    counters::bump_skill_activity_calls();
    let (own_bb, opp_bb, own_money) = match side {
        Player::P1 => (pos.p1_pieces.0, pos.p2_pieces.0, pos.p1_money),
        Player::P2 => (pos.p2_pieces.0, pos.p1_pieces.0, pos.p2_money),
    };
    let all_occ = pos.p1_pieces.0 | pos.p2_pieces.0;
    let mut acc = 0i32;

    let mut bits = own_bb;
    while bits != 0 {
        let src = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let m = pos.mailbox[src as usize];

        // Detect Focus/Charge modifiers already in play for this piece to
        // avoid crediting the mystic flag AND the buffed follow-up range.
        // Cheap approximation: we use `skill_default_range` throughout and
        // don't apply the +1 for a pending Focus. Not perfectly accurate but
        // conservative — it always undercounts, never over.

        // Iterate this piece's two skill slots.
        for slot in 0u8..2 {
            let sid = if slot == 0 { m.skill1() } else { m.skill2() };
            let Some(sk) = skill_from_id(sid) else { continue };

            let cost = skill_cost(sk) as u16;
            if own_money < cost { continue; }

            acc += skill_slot_credit(pos, side, sk, src, own_bb, opp_bb, all_occ, own_money, m);
        }
    }
    acc
}

/// Per-skill target/destination counting, returning the eval credit for one
/// slot. Split out so the outer loop stays readable.
#[inline]
fn skill_slot_credit(
    pos: &Position,
    side: Player,
    sk: Skill,
    src: u8,
    own_bb: u64,
    opp_bb: u64,
    all_occ: u64,
    own_money: u16,
    m: crate::state::MailboxEntry,
) -> i32 {
    let range = skill_default_range(sk);
    let owner = skill_target_owner(sk);
    let cat = skill_category(sk);

    // Mystic (Focus/Charge): flag bonus gated on an affordable, castable
    // follow-up active skill on the SAME piece this turn.
    if matches!(cat, SkillCategory::Mystic) {
        let mystic_cost = skill_cost(sk) as u16;
        // Look at the OTHER slot on this piece for a follow-up.
        let other_sid = if m.skill1() == sk as u8 { m.skill2() } else { m.skill1() };
        let Some(follow) = skill_from_id(other_sid) else { return 0 };
        // Follow-up must be an active category (not another mystic modifier).
        if matches!(skill_category(follow), SkillCategory::Mystic) { return 0 };
        let follow_cost = skill_cost(follow) as u16;
        // Both must be affordable together.
        if own_money < mystic_cost + follow_cost { return 0 };
        // Follow-up must have ≥1 legal target from `src`.
        let follow_range = skill_default_range(follow);
        let follow_owner = skill_target_owner(follow);
        if slot_target_count(follow, follow_owner, follow_range, src, own_bb, opp_bb, all_occ, pos, side) == 0 {
            return 0;
        }
        return MYSTIC_FLAG_BONUS;
    }

    let n = slot_target_count(sk, owner, range, src, own_bb, opp_bb, all_occ, pos, side);
    if n == 0 { return 0; }

    match cat {
        SkillCategory::Strike => n as i32 * STRIKE_PER_TARGET,
        SkillCategory::Move   => n as i32 * MOVE_PER_DEST,
        SkillCategory::Shield => {
            // Shield (SelfOnly) contributes a fixed bonus if it would stick.
            if matches!(owner, TargetOwner::SelfOnly) { SHIELD_SELF }
            else { n as i32 * SHIELD_PER_TARGET }
        }
        SkillCategory::Mystic => 0, // handled above
    }
}

/// Count legal targets for one skill from square `src`. Cheap proxy —
/// approximates the generator's logic without duplicating it. Used only for
/// the eval's activity term.
#[inline]
fn slot_target_count(
    sk: Skill,
    owner: TargetOwner,
    range: u8,
    src: u8,
    own_bb: u64,
    opp_bb: u64,
    all_occ: u64,
    pos: &Position,
    _side: Player,
) -> u32 {
    match owner {
        TargetOwner::Enemy => {
            let ray = magic::skill_attacks(src, all_occ, range).0;
            (ray & opp_bb).count_ones()
        }
        TargetOwner::Ally => {
            let ray = magic::skill_attacks(src, all_occ, range).0;
            let candidates = ray & own_bb & !(1u64 << src);
            // Filter Heal/Plate: target must actually need it, else no credit.
            match sk {
                Skill::Heal => {
                    let mut n = 0u32;
                    let mut bits = candidates;
                    while bits != 0 {
                        let t = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        if pos.mailbox[t].hp() < HP_CAP { n += 1; }
                    }
                    n
                }
                Skill::Plate => {
                    let mut n = 0u32;
                    let mut bits = candidates;
                    while bits != 0 {
                        let t = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        if pos.mailbox[t].armor() < ARMOR_CAP { n += 1; }
                    }
                    n
                }
                _ => candidates.count_ones(), // Swap: any ally partner is valid
            }
        }
        TargetOwner::Either => {
            // Shove: any target square in range on a ray.
            let ray = magic::skill_attacks(src, all_occ, range).0;
            (ray & (own_bb | opp_bb) & !(1u64 << src)).count_ones()
        }
        TargetOwner::Empty => {
            // Dash/Retreat: empty squares within range on a queen-ray. Use
            // `skill_attacks(src, 0, range)` (unblocked) then subtract occupied.
            let all_ray = magic::skill_attacks(src, 0, range).0;
            (all_ray & !all_occ).count_ones()
        }
        TargetOwner::SelfOnly => {
            // Shield: 1 credit only if own armor < cap.
            let a = pos.mailbox[src as usize].armor();
            if a < ARMOR_CAP { 1 } else { 0 }
        }
    }
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
        let mut pos = Position::empty();
        pos.p1_money = 10;
        pos.p2_money = 4;
        assert_eq!(evaluate(&pos), 6 * MONEY_PER_UNIT);
    }

    #[test]
    fn skill_equipped_beats_unequipped() {
        // P1 Champion with Lance equipped vs P2 Champion bare.
        // Both HP=2, no armor → differential is exactly skill_value(Lance).
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2));
        assert_eq!(evaluate(&pos), skill_value(Skill::Lance));
    }

    #[test]
    fn stack_m_setup_is_zero() {
        // Canonical start: identical material on both sides, 6 money each.
        let pos = Position::setup_stack_m();
        assert_eq!(evaluate(&pos), 0);
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
        // Pin the math: single P1 Champion HP=2 armor=2 skill1=Tempest skill2=Charge,
        // empty money. Score must equal the explicit sum.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default()
                .with_hp(2)
                .with_armor(2)
                .with_skill1(Skill::Tempest as u8)
                .with_skill2(Skill::Charge as u8));
        // Mobility: Champion at sq 28 (rank 3), 8 neighbours all free.
        // CHAMP_MOB_PER_SQ=12, 8 squares = 96.
        let mob = 8 * CHAMP_MOB_PER_SQ;
        let expected = CHAMPION_VALUE
            + 2 * HP_PER_POINT
            + 2 * ARMOR_PER_POINT
            + skill_value(Skill::Tempest)
            + skill_value(Skill::Charge)
            + mob;
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
}
