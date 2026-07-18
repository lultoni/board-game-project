//! Static Exchange Evaluation (SEE) for move ordering.
//!
//! Called from quiescence to score capture moves before descent. The exchange
//! math is the same rollout the evaluator's now-removed MAEE term performed -
//! per-target, LVA-ordered, HP/armor multi-hit, kill-follow-through - but the
//! caller now asks a different question: *this specific capture (src, target),
//! is it a winning exchange?* MAEE asked *for each hanging enemy, what's the
//! best outcome?* Same rollout, different framing.
//!
//! ## Why not evaluator-side any more
//!
//! MAEE-in-eval violated the eval-is-a-pure-position-rater discipline
//! (`.claude/eval-correctness-passes.md`). Pricing the exchange at every leaf
//! double-counted with quiescence's own capture rollout and perturbed move
//! ordering. Moving the same math to move-ordering-time (as SEE) fixes both:
//! search decides ordering, eval decides value.
//!
//! ## Public surface
//!
//! - [`AttackersTable`]: per-square attacker bitboards, both sides. Built once
//!   per QS node against the current occupancy.
//! - [`build_attackers_table`]: constructor.
//! - [`see_capture`]: score one candidate capture (src → target). Returns net
//!   material from the *initiator's* POV; positive = winning exchange.
//!
//! Kings are excluded as both attackers and targets - king captures are
//! terminal (±MATE_SCORE) and handled upstream by [`crate::search::evaluator`].
//! A capture whose target is a king is scored `+∞` (MATE_SCORE) at the call
//! site - callers must check for that before invoking `see_capture`.

use crate::state::Position;
use crate::state::position::Player;
use crate::state::magic;

// ─── Skill-attacker LUT ──────────────────────────────────────────────────
//
// SEE participates skill damage in exchange rollouts. Only "strike" skills that
// deal direct HP/armor damage on the target square are modeled - Blast/Shove
// are excluded (0 direct damage, combo-stateful); movement + shield skills are
// excluded (don't hit the target square). Money is gated at table build time
// only (the caster's side has ≥ cost); not decremented per ply.
//
// See `.claude/eval-perf-passes.md` for the SEE integration plan.

const KIND_PHYS:         u8 = 0;
const KIND_SKILL_STRIKE: u8 = 1; // Lance, Hook, Steal (1 dmg)
const KIND_SKILL_BREAK:  u8 = 2; // Break (armor strip only, filtered when armor=0)
const KIND_SKILL_ENDER:  u8 = 3; // Tempest (1 dmg then exchange terminates)

/// Per-skill LUT keyed by 4-bit skill id (0..=15). Fields:
///   0: kind (0 = not-a-participating-strike, else KIND_SKILL_*)
///   1: cost
///   2: range
///
/// The `kind = 0` slot means "excluded from SEE". Skills we exclude:
///   0 = no skill, 6..=9 = Shield/Heal/Plate/Focus, 10 = Blast, 11 = Shove,
///   12..=15 = Dash/Swap/Retreat/Charge (movement-class).
const SKILL_LUT: [(u8, u8, u8); 16] = [
    (0,                  0, 0),  // 0  = none
    (KIND_SKILL_STRIKE,  2, 1),  // 1  = Lance
    (KIND_SKILL_STRIKE,  3, 2),  // 2  = Hook
    (KIND_SKILL_BREAK,   2, 2),  // 3  = Break
    (KIND_SKILL_STRIKE,  4, 2),  // 4  = Steal
    (KIND_SKILL_ENDER,   4, 2),  // 5  = Tempest
    (0,                  0, 0),  // 6  = Shield
    (0,                  0, 0),  // 7  = Heal
    (0,                  0, 0),  // 8  = Plate
    (0,                  0, 0),  // 9  = Focus
    (0,                  0, 0),  // 10 = Blast (excluded - combo-stateful)
    (0,                  0, 0),  // 11 = Shove (excluded - combo-stateful)
    (0,                  0, 0),  // 12 = Dash
    (0,                  0, 0),  // 13 = Swap
    (0,                  0, 0),  // 14 = Retreat
    (0,                  0, 0),  // 15 = Charge
];

/// Kind priority for tie-break when a piece equips two participating strikes:
/// STRIKE > ENDER > BREAK. Higher = preferred. Used so a Champion with (Lance,
/// Tempest) is scored as a strike attacker, not a terminating one - strikes
/// are unconditionally more useful in exchanges.
#[inline]
fn kind_priority(k: u8) -> u8 {
    match k {
        KIND_SKILL_STRIKE => 3,
        KIND_SKILL_ENDER  => 2,
        KIND_SKILL_BREAK  => 1,
        _                 => 0,
    }
}

// ─── Piece / damage weights ──────────────────────────────────────────────
//
// Kept in sync with evaluator.rs's material weights. Duplicated here rather
// than imported because SEE is a self-contained module - the eval could
// eventually diverge (different piece weights for tactical ordering vs
// positional scoring) without breaking SEE.

const CHAMPION_VALUE:  i32 = 1000;
const GUARD_VALUE:     i32 = 600;
const HP_PER_POINT:    i32 = 150;
const ARMOR_PER_POINT: i32 = 120;

/// Geometric ceiling: 8 cheby-1 slots per side (see MAEE-era analysis in
/// `.claude/eval-perf-passes.md`). Realistic in-game is 3-5.
const SEE_MAX_ATTACKERS: usize = 8;

/// Two sides × 8 slots = 16 total plies in a single-target exchange rollout.
const SEE_MAX_PLIES: usize = 16;

/// Precomputed 1 << sq lookup.
const SQ_BIT: [u64; 64] = {
    let mut t = [0u64; 64];
    let mut i = 0usize;
    while i < 64 { t[i] = 1u64 << i; i += 1; }
    t
};

// ─── Attacker bookkeeping ────────────────────────────────────────────────

/// One attacker candidate: (cost, source-square, kind). Cost packs into i16
/// (max is CHAMPION_VALUE(1000) + 2·HP(300) + 2·ARMOR(240) = 1540).
///
/// `kind` selects the exchange semantics: KIND_PHYS is a Move-Attack (full
/// swap-off on kill), KIND_SKILL_STRIKE deals 1 damage without relocating,
/// KIND_SKILL_BREAK strips 1 armor and is filtered out when victim has none,
/// KIND_SKILL_ENDER deals 1 damage and terminates the exchange (Tempest).
#[derive(Copy, Clone)]
struct Attacker {
    cost: i16,
    sq:   u8,
    kind: u8,
}

/// Fixed-size sorted-cheapest-first list - heap-free.
struct AttackerList {
    items: [Attacker; SEE_MAX_ATTACKERS],
    len:   u8,
}

impl AttackerList {
    #[inline]
    fn new() -> Self {
        Self { items: [Attacker { cost: 0, sq: 0, kind: 0 }; SEE_MAX_ATTACKERS], len: 0 }
    }

    /// Insertion-sort push. Drops the most expensive on overflow.
    #[inline]
    fn push(&mut self, a: Attacker) {
        let len = self.len as usize;
        let mut i = 0;
        while i < len && self.items[i].cost <= a.cost { i += 1; }
        if len < SEE_MAX_ATTACKERS {
            let mut j = len;
            while j > i {
                self.items[j] = self.items[j - 1];
                j -= 1;
            }
            self.items[i] = a;
            self.len += 1;
        } else if i < SEE_MAX_ATTACKERS {
            let mut j = SEE_MAX_ATTACKERS - 1;
            while j > i {
                self.items[j] = self.items[j - 1];
                j -= 1;
            }
            self.items[i] = a;
        }
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

    /// Remove a specific square if present. Used to lift a fixed initiator out
    /// of the LVA list before the exchange rollout begins.
    #[inline]
    fn remove_sq(&mut self, sq: u8) -> Option<Attacker> {
        let len = self.len as usize;
        let mut i = 0;
        while i < len {
            if self.items[i].sq == sq {
                let out = self.items[i];
                for j in (i + 1)..len { self.items[j - 1] = self.items[j]; }
                self.len -= 1;
                return Some(out);
            }
            i += 1;
        }
        None
    }
}

// ─── Attackers table ─────────────────────────────────────────────────────

/// Per-square attacker bitboards, both sides. `p1_of[t]` is the bitmask of P1
/// non-king pieces that can move-attack square `t`; symmetric for `p2_of`.
/// `p1_skill_of[t]` / `p2_skill_of[t]` add skill-strike attackers (Lance,
/// Hook, Steal, Break, Tempest) money-gated at build time. `p1_skill_kind[sq]`
/// records which skill kind each skill-attacker uses; unused when the sq bit
/// is not set. Built once per QS node against the current occupancy.
pub struct AttackersTable {
    p1_of:         [u64; 64],
    p2_of:         [u64; 64],
    p1_skill_of:   [u64; 64],
    p2_skill_of:   [u64; 64],
    p1_skill_kind: [u8;  64],
    p2_skill_kind: [u8;  64],
}

impl AttackersTable {
    /// Bitmask of `side`'s physical attackers on `target_sq`, minus the target.
    #[inline]
    fn attackers_of(&self, side: Player, target_sq: u8) -> u64 {
        let bits = match side {
            Player::P1 => self.p1_of[target_sq as usize],
            Player::P2 => self.p2_of[target_sq as usize],
        };
        bits & !SQ_BIT[target_sq as usize]
    }

    /// Bitmask of `side`'s skill attackers on `target_sq`, minus the target,
    /// and minus any square that's also a physical attacker (physical wins
    /// classification when a piece has both reach modes on the same target).
    #[inline]
    fn skill_attackers_of(&self, side: Player, target_sq: u8) -> u64 {
        let (skill_bits, phys_bits) = match side {
            Player::P1 => (self.p1_skill_of[target_sq as usize], self.p1_of[target_sq as usize]),
            Player::P2 => (self.p2_skill_of[target_sq as usize], self.p2_of[target_sq as usize]),
        };
        skill_bits & !phys_bits & !SQ_BIT[target_sq as usize]
    }

    /// Any attackers of `side` on this square (physical only - kept for
    /// external callers that don't care about skill reach).
    #[inline]
    pub fn any_attackers_of(&self, side: Player, target_sq: u8) -> u64 {
        self.attackers_of(side, target_sq)
    }
}

/// Build the attackers table for the current position.
///
/// For each non-king piece at `sq`, computes the bitmask of squares it can
/// move-attack given `all_occ`, then transposes: for each attackable target
/// `t`, sets bit `sq` in `of[t]`. Skill-attackers (Lance / Hook / Break /
/// Steal / Tempest) are additionally scattered into `skill_of` for
/// participating in exchange rollouts. Money is gated at build time: an
/// equipped skill is only projected if the owner's current money ≥ cost.
///
/// Guard move-attack semantics per generator.rs: approach ≤ speed-1 through
/// empties, then attack cheby-1. Landing set = {src ∪ empty cheby-1 of src};
/// fanout = magic::king_expand(landing) minus src. Guards do not participate in
/// skill attacks (they cannot equip skills).
pub fn build_attackers_table(pos: &Position, all_occ: u64) -> AttackersTable {
    let mut table = build_attackers_table_phys(pos, all_occ);

    // Skill scatter. Champions AND kings can equip skills; guards cannot.
    // Money-gate at build time using the owner's current money.
    let p1_skill_srcs = (pos.p1_pieces.0 & !pos.guards.0) & !0u64; // champs + kings
    let p2_skill_srcs =  pos.p2_pieces.0 & !pos.guards.0;
    scatter_skills_side(pos, all_occ, p1_skill_srcs, pos.p1_money,
                        &mut table.p1_skill_of, &mut table.p1_skill_kind);
    scatter_skills_side(pos, all_occ, p2_skill_srcs, pos.p2_money,
                        &mut table.p2_skill_of, &mut table.p2_skill_kind);

    table
}

/// Physical-only attackers table: fills `p1_of` / `p2_of` (Move-Attack scatter)
/// and leaves the skill-scatter fields (`*_skill_of`, `*_skill_kind`) zeroed.
///
/// The **evaluator** (`EvalContext::new` → exposure / champion_threat) reads only
/// the physical scatter via [`AttackersTable::any_attackers_of`] - it never
/// touches the skill fields, which exist solely for the `see_capture` exchange
/// rollout. Building the skill scatter traces a queen-ray per champion/king
/// (`scatter_skills_side` → `magic::skill_attacks`), the dominant per-call cost;
/// skipping it on the ~3.37M-calls/sweep eval path is byte-identical to the full
/// build for every value the evaluator can observe. SEE keeps using
/// [`build_attackers_table`] (both scatters).
pub fn build_attackers_table_phys(pos: &Position, all_occ: u64) -> AttackersTable {
    let mut table = AttackersTable {
        p1_of:         [0u64; 64],
        p2_of:         [0u64; 64],
        p1_skill_of:   [0u64; 64],
        p2_skill_of:   [0u64; 64],
        p1_skill_kind: [0u8;  64],
        p2_skill_kind: [0u8;  64],
    };

    // Physical scatter (Move-Attack). Non-king pieces only. Kings are excluded
    // as physical attackers per the module invariant (king exchanges are
    // terminal). The full-table build re-includes kings via skill-scatter so
    // they can cast into an exchange without relocating.
    scatter_side(pos, all_occ, pos.p1_pieces.0 & !pos.kings.0, &mut table.p1_of);
    scatter_side(pos, all_occ, pos.p2_pieces.0 & !pos.kings.0, &mut table.p2_of);

    table
}

#[inline]
fn scatter_side(pos: &Position, all_occ: u64, mut side_bits: u64, of: &mut [u64; 64]) {
    while side_bits != 0 {
        let sq = side_bits.trailing_zeros() as u8;
        side_bits &= side_bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let is_guard = pos.guards.0 & sq_bit != 0;
        let attacks = if is_guard {
            let approach = (magic::movement_targets_speed1(sq).0 & !all_occ) | sq_bit;
            magic::king_expand(approach) & !sq_bit
        } else {
            // Champion: 8 immediate neighbours.
            magic::movement_targets_speed1(sq).0
        };
        let mut a = attacks;
        while a != 0 {
            let t = a.trailing_zeros() as usize;
            a &= a - 1;
            of[t] |= sq_bit;
        }
    }
}

/// Scatter skill-attacker bitmasks for one side. For each Champion/King on
/// `side_bits`, look up the two equipped skills; keep the highest-priority
/// participating skill (STRIKE > ENDER > BREAK) whose cost the owner can
/// currently afford. Project its fanout via `magic::skill_attacks`, mask to
/// enemy-occupied squares (participating strike skills all target enemies),
/// and scatter the source bit into each hit target. Record the winning kind
/// in `kind_of[sq]`.
///
/// Money-gate uses the current `money` budget only - we do not decrement it
/// per attacker. Overestimates when a side would need multiple casts.
#[inline]
fn scatter_skills_side(
    pos: &Position,
    all_occ: u64,
    mut side_bits: u64,
    money: u16,
    of:       &mut [u64; 64],
    kind_of:  &mut [u8;  64],
) {
    // Which side are we scattering? Determined by whether side_bits is a
    // subset of p1_pieces (only both sides are disjoint, so a single sample
    // bit suffices). Currently the side identity isn't needed downstream - we
    // just walk the pieces and project their skill reach - but the sample
    // gives an early-out for empty side_bits and lets us assert consistency.
    if side_bits == 0 { return; }
    let _side_is_p1 = (pos.p1_pieces.0 & side_bits) != 0;

    while side_bits != 0 {
        let sq = side_bits.trailing_zeros() as u8;
        side_bits &= side_bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let m = pos.mailbox[sq as usize];
        let s1 = m.skill1() as usize;
        let s2 = m.skill2() as usize;
        let (k1, c1, r1) = SKILL_LUT[s1 & 0xF];
        let (k2, c2, r2) = SKILL_LUT[s2 & 0xF];

        // Pick the best affordable participating skill.
        let mut best_kind:  u8 = 0;
        let mut best_range: u8 = 0;
        let mut best_prio:  u8 = 0;

        if k1 != 0 && (c1 as u16) <= money {
            let p = kind_priority(k1);
            if p > best_prio { best_prio = p; best_kind = k1; best_range = r1; }
        }
        if k2 != 0 && (c2 as u16) <= money {
            let p = kind_priority(k2);
            if p > best_prio { best_kind = k2; best_range = r2; }
        }
        if best_kind == 0 { continue; }

        // Fanout: queen-ray up to range, blocker-inclusive. We intentionally
        // do NOT mask to enemy-occupied squares - during an exchange rollout
        // the target square's occupant flips (swap-in), and the caller only
        // invokes see_capture on a legal enemy target so we don't need to
        // filter own-side squares (they're never asked).
        let fan = magic::skill_attacks(sq, all_occ, best_range).0 & !sq_bit;
        let mut a = fan;
        while a != 0 {
            let t = a.trailing_zeros() as usize;
            a &= a - 1;
            of[t] |= sq_bit;
        }
        kind_of[sq as usize] = best_kind;
    }
}

// ─── Attacker enumeration ────────────────────────────────────────────────

/// Build an attacker list combining physical + skill sources on the same
/// target. Physical wins classification when a piece appears in both. Skill
/// attackers use their caster's material cost for LVA ordering - this over-
/// prioritises cheap-material Champions over expensive-material Champions
/// even though the skill-caster doesn't lose material on hit, but a
/// consistent LVA order for tie-break is what matters for reproducibility.
///
/// Kings can be skill attackers but not physical; their material is treated
/// as CHAMPION_VALUE for LVA purposes (kings are never captured mid-
/// exchange so their absolute cost doesn't matter - only their relative
/// ordering).
#[inline]
fn build_attacker_list_mixed(
    pos: &Position,
    phys_bits: u64,
    skill_bits: u64,
    kind_of: &[u8; 64],
) -> AttackerList {
    let mut out = AttackerList::new();

    let mut bits = phys_bits;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let m = pos.mailbox[sq as usize];
        let mat = if pos.champions.0 & sq_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
        let cost = attacker_cost(mat, m.hp(), m.armor());
        out.push(Attacker { cost, sq, kind: KIND_PHYS });
    }

    // Skill-only attackers (physical bits already consumed above).
    let mut bits = skill_bits & !phys_bits;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let sq_bit = SQ_BIT[sq as usize];
        let m = pos.mailbox[sq as usize];
        // Kings are treated as CHAMPION_VALUE for LVA purposes.
        let mat = if pos.guards.0 & sq_bit != 0 { GUARD_VALUE } else { CHAMPION_VALUE };
        let cost = attacker_cost(mat, m.hp(), m.armor());
        let kind = kind_of[sq as usize];
        out.push(Attacker { cost, sq, kind });
    }

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
    else { 0 } // kings excluded - see_capture rejects king targets upstream
}

#[inline]
fn other(p: Player) -> Player {
    match p { Player::P1 => Player::P2, Player::P2 => Player::P1 }
}

// ─── SEE per-capture rollout ─────────────────────────────────────────────

/// Score a single-hit attack on `target` with no exchange follow-up.
///
/// Used for move-ordering of Strike/Blast skill actions in quiescence: the
/// caster deals 1 damage to `target` but doesn't move onto its square, so
/// no swap-off is possible. Returns the material value from stm's POV of
/// the damage this hit deals:
///
/// - `armor > 0` → `ARMOR_PER_POINT` (small)
/// - `hp > 1`    → `HP_PER_POINT`    (medium)
/// - killing     → `piece_value + HP_PER_POINT + armor*ARMOR_PER_POINT`
///
/// Returns 0 for empty squares or king targets (callers handle king specially).
/// Cheap - no AttackersTable needed.
pub fn see_single_hit(pos: &Position, target: u8) -> i32 {
    let target_bit = SQ_BIT[target as usize];
    let victim_val = piece_material_of(pos, target);
    if victim_val == 0 { return 0; }
    let entry = pos.mailbox[target as usize];
    let armor = entry.armor();
    let hp    = entry.hp();
    if armor > 0 {
        return ARMOR_PER_POINT;
    }
    if hp > 1 {
        return HP_PER_POINT;
    }
    // hp == 1, armor == 0 → killing blow.
    // (armor is 0 here, so the armor*ARMOR_PER_POINT term is 0 too.)
    let _ = target_bit;
    victim_val + HP_PER_POINT
}

/// Score the exchange initiated by `src` capturing `target`.
///
/// Returns net material change from the *initiator's* (side owning `src`)
/// POV: positive = winning capture, zero = neutral, negative = losing.
///
/// Semantics (mirror MAEE, differences from classical SEE):
/// - Multi-hit kills: 1 damage per attacker; armor absorbs first (down to 0),
///   then HP. Only the final *physical* attacker (that lands the killing blow)
///   takes the target square. Skill-strike killing blows do NOT swap the
///   caster in (Lance/Hook/Steal/Tempest are ranged, no relocate).
/// - Kill-follow-through: after a physical kill, subsequent attackers hit the
///   *new* occupant (initial attacker's material + full HP + full armor).
/// - Guard reach gain: when a blocker vacates a cheby-1 square of the target,
///   Guards adjacent to that vacated square can newly reach the target as an
///   approach square. Maintained incrementally.
/// - Skill attackers: Lance/Hook/Steal (STRIKE) and Tempest (ENDER) can
///   contribute damage. Break contributes only when victim has armor. Skill
///   attackers are money-gated at table build. Tempest terminates the
///   exchange after firing. Skill bitmasks are snapshot-frozen at build time
///   (we do not re-project skill reach as pieces vacate mid-exchange).
/// - Stand-pat fold-back: each side may refuse their last exchange step if
///   it's bad for them.
///
/// Preconditions:
/// - `target` is not a king square (caller must score king captures as +∞).
/// - `src` is a legal attacker of `target` for its side (this is guaranteed
///   by the move generator having emitted the capture).
pub fn see_capture(pos: &Position, table: &AttackersTable, src: u8, target: u8) -> i32 {
    let src_bit = SQ_BIT[src as usize];
    let target_bit = SQ_BIT[target as usize];

    // Determine initiator side from src ownership.
    let stm = if pos.p1_pieces.0 & src_bit != 0 { Player::P1 } else { Player::P2 };
    let dfd = other(stm);

    // Victim state.
    let mut victim_val = piece_material_of(pos, target);
    if victim_val == 0 { return 0; } // king or empty - caller shouldn't invoke us
    let entry = pos.mailbox[target as usize];
    let mut victim_hp = entry.hp();
    let mut victim_armor = entry.armor();

    // Attacker sets from the table. Both physical and skill bitmasks tracked
    // separately (they have different rebuild semantics).
    let mut attackers_stm_phys_bb  = table.attackers_of(stm, target);
    let mut attackers_dfd_phys_bb  = table.attackers_of(dfd, target);
    let mut attackers_stm_skill_bb = table.skill_attackers_of(stm, target);
    let mut attackers_dfd_skill_bb = table.skill_attackers_of(dfd, target);

    let (stm_kind_of, dfd_kind_of) = match stm {
        Player::P1 => (&table.p1_skill_kind, &table.p2_skill_kind),
        Player::P2 => (&table.p2_skill_kind, &table.p1_skill_kind),
    };

    let mut attackers_stm = build_attacker_list_mixed(
        pos, attackers_stm_phys_bb, attackers_stm_skill_bb, stm_kind_of);
    let mut attackers_dfd = build_attacker_list_mixed(
        pos, attackers_dfd_phys_bb, attackers_dfd_skill_bb, dfd_kind_of);

    // Ply 0: the fixed initiator (always physical - the caller invoked us
    // because a Move-Attack landed on `target`).
    let initiator = match attackers_stm.remove_sq(src) {
        Some(a) => a,
        None => {
            let m = pos.mailbox[src as usize];
            let mat = if pos.champions.0 & src_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
            Attacker { cost: attacker_cost(mat, m.hp(), m.armor()), sq: src, kind: KIND_PHYS }
        }
    };
    attackers_stm_phys_bb &= !src_bit;

    // Ply-by-ply signed gains from stm's POV.
    let mut gains = [0i32; SEE_MAX_PLIES];
    let mut n_gains = 0usize;

    let target_cheby1 = magic::king_expand(target_bit);

    // Attack ply 0: initiator hits the victim.
    let mut vacated = 0u64;
    let mut terminate = apply_hit(
        &initiator, 1, stm, target_bit, target_cheby1,
        pos,
        &mut victim_val, &mut victim_hp, &mut victim_armor,
        &mut vacated,
        &mut attackers_stm, &mut attackers_stm_phys_bb, &mut attackers_stm_skill_bb, stm_kind_of,
        &mut attackers_dfd, &mut attackers_dfd_phys_bb, &mut attackers_dfd_skill_bb, dfd_kind_of,
        gains.as_mut_slice(), &mut n_gains,
    );

    // Alternating LVA plies until one side runs out or a Tempest terminates.
    let mut side = dfd;
    while !terminate {
        // Pop attackers, skipping Break entries when victim has no armor.
        let att_opt = loop {
            let a = if side == stm {
                attackers_stm.pop_front()
            } else {
                attackers_dfd.pop_front()
            };
            match a {
                None => break None,
                Some(x) => {
                    if x.kind == KIND_SKILL_BREAK && victim_armor == 0 {
                        // Break can't contribute - armor is already stripped.
                        // Also clear its skill bit so it doesn't come back on
                        // a rebuild.
                        let bit = SQ_BIT[x.sq as usize];
                        if side == stm { attackers_stm_skill_bb &= !bit; }
                        else           { attackers_dfd_skill_bb &= !bit; }
                        continue;
                    }
                    break Some(x);
                }
            }
        };
        let Some(att) = att_opt else { break };
        if n_gains >= SEE_MAX_PLIES { break; }

        let sign = if side == stm { 1 } else { -1 };
        terminate = apply_hit(
            &att, sign, stm, target_bit, target_cheby1,
            pos,
            &mut victim_val, &mut victim_hp, &mut victim_armor,
            &mut vacated,
            &mut attackers_stm, &mut attackers_stm_phys_bb, &mut attackers_stm_skill_bb, stm_kind_of,
            &mut attackers_dfd, &mut attackers_dfd_phys_bb, &mut attackers_dfd_skill_bb, dfd_kind_of,
            gains.as_mut_slice(), &mut n_gains,
        );

        side = other(side);
    }

    if n_gains == 0 { return 0; }

    // Stand-pat fold-back. Ply indices from stm's POV:
    //   even i → stm ply → clamp low at 0 (stm won't take a losing step)
    //   odd  i → dfd ply → clamp high at 0 (dfd won't take a losing step)
    // Ply 0 is the initiator's forced attack - we do NOT clamp it.
    let mut val: i32 = gains[n_gains - 1];
    let mut i = n_gains as i32 - 2;
    while i >= 0 {
        let ply_after = (i + 1) as usize;
        val = if (ply_after & 1) == 0 { val.max(0) } else { val.min(0) };
        val += gains[i as usize];
        i -= 1;
    }
    val
}

/// Apply one attack ply. Updates victim state, appends to `gains`. Returns
/// `true` if the exchange must terminate after this ply (Tempest ender).
///
/// On a *physical* killing blow the killer moves onto the target square; the
/// new occupant's stats become the victim state and previously-blocked Guards
/// unlock via the vacated-origin trick. On a *skill-strike* killing blow the
/// caster does NOT swap in - the target square goes empty. Subsequent
/// attackers have nothing left to hit, so the outer loop naturally ends via
/// `victim_val = 0` at the next apply_hit call. (For safety we also mark
/// terminate=true on skill-kills.)
#[inline]
#[allow(clippy::too_many_arguments)]
fn apply_hit(
    att: &Attacker,
    sign: i32,
    stm: Player,
    target_bit: u64,
    target_cheby1: u64,
    pos: &Position,
    victim_val: &mut i32,
    victim_hp: &mut u8,
    victim_armor: &mut u8,
    vacated: &mut u64,
    attackers_stm:          &mut AttackerList,
    attackers_stm_phys_bb:  &mut u64,
    attackers_stm_skill_bb: &mut u64,
    stm_kind_of:            &[u8; 64],
    attackers_dfd:          &mut AttackerList,
    attackers_dfd_phys_bb:  &mut u64,
    attackers_dfd_skill_bb: &mut u64,
    dfd_kind_of:            &[u8; 64],
    gains: &mut [i32],
    n_gains: &mut usize,
) -> bool {
    // Non-killing damage: armor absorbs first, then HP. Same for all kinds.
    if *victim_armor > 0 {
        *victim_armor -= 1;
        gains[*n_gains] = sign * ARMOR_PER_POINT;
        *n_gains += 1;
        // Skill attackers spend themselves - clear their bit even on non-kill.
        if att.kind != KIND_PHYS {
            let bit = SQ_BIT[att.sq as usize];
            let killer_is_stm = sign > 0;
            if killer_is_stm { *attackers_stm_skill_bb &= !bit; }
            else             { *attackers_dfd_skill_bb &= !bit; }
        }
        return att.kind == KIND_SKILL_ENDER;
    }
    if *victim_hp > 1 {
        *victim_hp -= 1;
        gains[*n_gains] = sign * HP_PER_POINT;
        *n_gains += 1;
        if att.kind != KIND_PHYS {
            let bit = SQ_BIT[att.sq as usize];
            let killer_is_stm = sign > 0;
            if killer_is_stm { *attackers_stm_skill_bb &= !bit; }
            else             { *attackers_dfd_skill_bb &= !bit; }
        }
        return att.kind == KIND_SKILL_ENDER;
    }
    // Killing blow.
    let kill_gain = *victim_val + HP_PER_POINT + ARMOR_PER_POINT * (*victim_armor) as i32;
    gains[*n_gains] = sign * kill_gain;
    *n_gains += 1;

    let att_bit = SQ_BIT[att.sq as usize];
    let killer_is_stm = sign > 0;

    if att.kind != KIND_PHYS {
        // Skill kill: caster does NOT relocate. Target square is now empty →
        // exchange terminates (no more meaningful attackers to model).
        if killer_is_stm { *attackers_stm_skill_bb &= !att_bit; }
        else             { *attackers_dfd_skill_bb &= !att_bit; }
        return true;
    }

    // Physical kill: attacker takes target_sq; its origin vacates.
    *vacated |= att_bit;
    let att_entry = pos.mailbox[att.sq as usize];
    let att_mat = if pos.champions.0 & att_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
    *victim_val = att_mat;
    *victim_hp = att_entry.hp();
    *victim_armor = att_entry.armor();

    // Clear killed attacker's physical bit.
    if killer_is_stm {
        *attackers_stm_phys_bb &= !att_bit;
    } else {
        *attackers_dfd_phys_bb &= !att_bit;
    }

    // Newly-reachable Guards from the vacated square (only if vacated sits
    // cheby-1 of the target). Skill bitmasks stay frozen - see doc comment.
    if target_cheby1 & att_bit != 0 {
        let neigh = magic::king_expand(att_bit) & pos.guards.0 & !*vacated & !target_bit;
        let (stm_own, dfd_own) = match stm {
            Player::P1 => (pos.p1_pieces.0, pos.p2_pieces.0),
            Player::P2 => (pos.p2_pieces.0, pos.p1_pieces.0),
        };
        *attackers_stm_phys_bb |= neigh & stm_own & !*attackers_stm_phys_bb;
        *attackers_dfd_phys_bb |= neigh & dfd_own & !*attackers_dfd_phys_bb;
    }

    // Rebuild sorted lists from the maintained bitmasks (physical + skill).
    *attackers_stm = build_attacker_list_mixed(
        pos, *attackers_stm_phys_bb, *attackers_stm_skill_bb, stm_kind_of);
    *attackers_dfd = build_attacker_list_mixed(
        pos, *attackers_dfd_phys_bb, *attackers_dfd_skill_bb, dfd_kind_of);

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Bitboard, MailboxEntry, Position};

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

    /// P1 Champion adjacent to a P2 Champion with HP=1, no armor. Single hit
    /// kills - SEE should return CHAMPION_VALUE + HP_PER_POINT (the killing
    /// blow rolls in the final HP).
    #[test]
    fn single_hit_kill_of_bare_champion() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 1, MailboxEntry::default().with_hp(1));
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        // Kill gain = victim_material + HP_PER_POINT + ARMOR*0 = 1000 + 150.
        assert_eq!(s, CHAMPION_VALUE + HP_PER_POINT);
    }

    /// P1 Champion attacks a P2 Champion with HP=2, no armor, and no defender
    /// nearby. Attacker only strips 1 HP → SEE ply-0 is HP_PER_POINT, no
    /// killing blow lands, no follow-up plies. Returns HP_PER_POINT.
    #[test]
    fn hp_only_hit_no_defenders() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, HP_PER_POINT);
    }

    /// P1 Champion at 27 attacks P2 Champion at 28 (HP=1). P2 has a defender
    /// Champion at 29 (HP=1) that can hit the P1 Champion once it lands on
    /// 28. Ply 0: P1 kills P2C@28 for +1150. Ply 1: P2C@29 strips 1 HP from
    /// P1's Champion (HP was 2 → 1), gain -150. Net = +1000.
    #[test]
    fn recapture_zero() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 1, MailboxEntry::default().with_hp(1));
        place(&mut pos, 29, Player::P2, 1, MailboxEntry::default().with_hp(1));
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, CHAMPION_VALUE + HP_PER_POINT - HP_PER_POINT);
    }

    /// Losing capture: two P2 Champions defend a bare P2 Guard, so P1
    /// initiating a capture on the Guard sees the exchange run through
    /// multiple recaptures.
    ///
    /// P1 Champion HP=1 at 27 (dies in one hit). P2 Guard HP=1 at 28. P2
    /// Champion HP=1 at 29. P2 Champion HP=1 at 19 (also cheby-1 of 28).
    ///
    /// Ply 0 (stm=P1): kill Guard on 28 → +750 (600+150). P1's Champion
    /// occupies 28 with HP=1. Ply 1 (dfd=P2, cheapest attacker of 28): P2
    /// Champion kills P1 Champion on 28 → -1150. Now a P2 Champion HP=1
    /// occupies 28. Ply 2 (stm=P1): no more P1 attackers → exchange ends.
    ///
    /// Fold-back: gains = [+750, -1150]. From index 0, ply_after=1 (dfd),
    /// clamp high 0 → -1150 stays (dfd's ply is bad for stm, so P2 will
    /// play it). Net = 750 + (-1150) = -400. Negative → losing capture.
    #[test]
    fn losing_capture_negative() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(1));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        place(&mut pos, 29, Player::P2, 1, MailboxEntry::default().with_hp(1));
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert!(s < 0, "expected losing exchange, got {}", s);
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - (CHAMPION_VALUE + HP_PER_POINT));
    }

    // ─── Skill-attacker tests ────────────────────────────────────────────

    const LANCE:   u8 = 1;
    const HOOK:    u8 = 2;
    const BREAK:   u8 = 3;
    const TEMPEST: u8 = 5;

    /// P1 Champion (HP=2) captures a P2 Guard (HP=1) at sq 28. The Guard has
    /// no physical defenders on cheby-1 of 28, but P2 has a Champion at sq 20
    /// (cheby-1 of 28) equipped with Lance and 2 gold - it counter-strikes
    /// after P1 lands. Ply 0 kills the Guard (+GUARD+HP). Ply 1 is a skill-
    /// strike that shaves P1's Champion (which now occupies 28, HP=2 → 1)
    /// for -HP_PER_POINT. Net = GUARD_VALUE + HP_PER_POINT - HP_PER_POINT.
    #[test]
    fn skill_defender_lance_counter_strike() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        // P2 Champion at sq 20 with Lance equipped, has 2g to cast.
        place(&mut pos, 20, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(LANCE));
        pos.p2_money = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        // Killed a Guard, took 1 HP damage on the swap-in from Lance counter.
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - HP_PER_POINT);
    }

    /// Same layout as `skill_defender_lance_counter_strike` but P2 has only 2
    /// gold and their skill is Hook (cost 3 → excluded), placed at a range-2
    /// square that's NOT cheby-1 of target (so no physical attack path).
    /// No counter, so net = full Guard-kill.
    #[test]
    fn skill_money_gate_excludes_broke_attacker() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        // Sq 12 is cheby-2 of 28 (N ray, same file, row diff 2). Not cheby-1.
        place(&mut pos, 12, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(HOOK));
        pos.p2_money = 2; // insufficient for Hook (cost 3)
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT);
    }

    /// P1 Champion (HP=1) attempts to Move-Attack sq 28 (P2 Guard, HP=1). P2
    /// has a Champion at sq 20 with Hook (cost 3, range 2) that could counter-
    /// strike from 2 squares away. Ply 0 kills the guard. Ply 1 = Hook killing
    /// blow on P1's Champion (HP=1) - this must NOT swap the P2 caster in;
    /// exchange terminates. Net = GUARD_KILL - CHAMPION_KILL.
    #[test]
    fn skill_hook_range2_kills_no_swap_in() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(1));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        // Hook at range 2 → sq 20 to sq 28 is a queen-ray at chebyshev 2 (both
        // on the same column at offset 8+8=... actually 28-20=8, one file
        // down. That's a knight offset - not on a queen ray. Use sq 12 instead
        // (28-12=16, two rows above target, same file → N ray).
        place(&mut pos, 12, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(HOOK));
        pos.p2_money = 3;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, (GUARD_VALUE + HP_PER_POINT) - (CHAMPION_VALUE + HP_PER_POINT));
    }

    /// Tempest terminates the exchange. Setup: P1 Champion (HP=3) - wait, HP
    /// max is 2. Use HP=2 with armor=1. Ply 0: P1 attacks P2 Guard (HP=1) at
    /// 28, kills → +GUARD_KILL. Now P1's Champion (HP=2, AR=1) occupies 28.
    /// Ply 1: P2 defender at sq 12 with Tempest → strips 1 armor from the
    /// new occupant (-ARMOR_PER_POINT), then terminates. Net = GUARD_KILL -
    /// ARMOR_PER_POINT.
    #[test]
    fn skill_tempest_terminates_exchange() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1,
              MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        // P2 Tempest caster at sq 12 (N ray, range 2 from 28). Also give P2 a
        // second physical defender at sq 29 (adj cheby-1 of 28) that WOULD
        // recapture if Tempest didn't terminate - this proves termination.
        place(&mut pos, 12, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(TEMPEST));
        place(&mut pos, 29, Player::P2, 1, MailboxEntry::default().with_hp(1));
        pos.p2_money = 4;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        // Ply 0: kill Guard (+750). Ply 1: Tempest (LVA-cheapest skill, HP=2
        // Champion cost < HP=1 Champion cost so Tempest at sq 12 pops before
        // the physical HP=1 defender at 29? Depends on LVA ordering).
        //
        // Attacker costs: sq 29 (HP=1) = 1000+150 = 1150. sq 12 (HP=2,
        // Tempest) = 1000+300 = 1300. Cheaper attacker (sq 29) pops first -
        // it kills the P1 Champion (which now occupies 28 with HP=2/AR=1).
        // Ply 1 pops phys@29: armor-strip (-120). Ply 2 pops Tempest@12:
        // armor was 0 now, so HP-strip (-150), then terminate. Ply 3 wouldn't
        // execute even if it could - Tempest ended things.
        //
        // Gains stm-POV: [+750, -120, -150]. Fold-back from right:
        //   val = -150
        //   i=1: ply_after=2 (stm), val = max(-150, 0) = 0, val += -120 = -120
        //   i=0: ply_after=1 (dfd), val = min(-120, 0) = -120, val += 750 = 630
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - ARMOR_PER_POINT);
    }

    /// Break is a skill attacker only when victim currently has armor. In this
    /// test P2 has both a physical HP=1 Champion at sq 29 (cheby-1 of target)
    /// that will kill P1's Champion (HP=1) after the swap-in, AND a Break-only
    /// caster at sq 12 with 2 gold. The physical defender fires first (LVA);
    /// Break is filtered because after Ply 0 the victim (P1's Champion post-
    /// swap-in) has armor=0, so Break contributes nothing extra.
    ///
    /// Expected: same as `losing_capture_negative` (Break invisible).
    #[test]
    fn skill_break_filtered_when_no_armor() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(1));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        place(&mut pos, 29, Player::P2, 1, MailboxEntry::default().with_hp(1));
        place(&mut pos, 12, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(BREAK));
        pos.p2_money = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - (CHAMPION_VALUE + HP_PER_POINT));
    }

    /// Break contributes when the victim (post-swap-in occupant) has armor.
    /// Setup: P1 Champion (HP=2, AR=1) captures P2 Guard (HP=1). P2 Break
    /// caster at sq 12 can strip 1 armor from P1's Champion after the swap-in.
    /// No other P2 attackers. Net = GUARD_KILL - ARMOR_PER_POINT.
    #[test]
    fn skill_break_contributes_when_armor_present() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1,
              MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        place(&mut pos, 12, Player::P2, 1,
              MailboxEntry::default().with_hp(2).with_skill1(BREAK));
        pos.p2_money = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - ARMOR_PER_POINT);
    }

    /// King can participate as a skill attacker. P1 Champion (HP=2) captures
    /// P2 Guard (HP=1). P2's King at sq 12 has Lance equipped and 2 gold →
    /// counter-strikes for 1 HP off P1's Champion. Net = GUARD_KILL -
    /// HP_PER_POINT.
    #[test]
    fn skill_king_as_lance_attacker() {
        let mut pos = Position::empty();
        place(&mut pos, 27, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P2, 2, MailboxEntry::default().with_hp(1));
        // King is kind=0 in `place`. Lance range=1 → King must be adjacent
        // to the target square (28). Place at sq 20 (cheby-1 of 28).
        place(&mut pos, 20, Player::P2, 0,
              MailboxEntry::default().with_hp(2).with_skill1(LANCE));
        pos.p2_money = 2;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let all_occ = (pos.p1_pieces | pos.p2_pieces).0;
        let table = build_attackers_table(&pos, all_occ);
        let s = see_capture(&pos, &table, 27, 28);
        assert_eq!(s, GUARD_VALUE + HP_PER_POINT - HP_PER_POINT);
    }
}
