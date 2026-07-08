//! Static Exchange Evaluation (SEE) for move ordering.
//!
//! Called from quiescence to score capture moves before descent. The exchange
//! math is the same rollout the evaluator's now-removed MAEE term performed —
//! per-target, LVA-ordered, HP/armor multi-hit, kill-follow-through — but the
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
//! Kings are excluded as both attackers and targets — king captures are
//! terminal (±MATE_SCORE) and handled upstream by [`crate::search::evaluator`].
//! A capture whose target is a king is scored `+∞` (MATE_SCORE) at the call
//! site — callers must check for that before invoking `see_capture`.

use crate::state::Position;
use crate::state::position::Player;
use crate::state::magic;

// ─── Piece / damage weights ──────────────────────────────────────────────
//
// Kept in sync with evaluator.rs's material weights. Duplicated here rather
// than imported because SEE is a self-contained module — the eval could
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

/// One attacker candidate: (cost, source-square). Cost packs into i16
/// (max is CHAMPION_VALUE(1000) + 2·HP(300) + 2·ARMOR(240) = 1540).
#[derive(Copy, Clone)]
struct Attacker {
    cost: i16,
    sq:   u8,
}

/// Fixed-size sorted-cheapest-first list — heap-free.
struct AttackerList {
    items: [Attacker; SEE_MAX_ATTACKERS],
    len:   u8,
}

impl AttackerList {
    #[inline]
    fn new() -> Self {
        Self { items: [Attacker { cost: 0, sq: 0 }; SEE_MAX_ATTACKERS], len: 0 }
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
/// Built once per QS node against the current occupancy.
pub struct AttackersTable {
    p1_of: [u64; 64],
    p2_of: [u64; 64],
}

impl AttackersTable {
    /// Bitmask of `side`'s attackers on `target_sq`, minus the target itself.
    #[inline]
    fn attackers_of(&self, side: Player, target_sq: u8) -> u64 {
        let bits = match side {
            Player::P1 => self.p1_of[target_sq as usize],
            Player::P2 => self.p2_of[target_sq as usize],
        };
        bits & !SQ_BIT[target_sq as usize]
    }

    /// Any attackers of either side on this square (for callers that only
    /// care about the initiator's side; `attackers_of` gives the per-side
    /// view).
    #[inline]
    pub fn any_attackers_of(&self, side: Player, target_sq: u8) -> u64 {
        self.attackers_of(side, target_sq)
    }
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

/// Build the attackers table for the current position.
///
/// For each non-king piece at `sq`, computes the bitmask of squares it can
/// move-attack given `all_occ`, then transposes: for each attackable target
/// `t`, sets bit `sq` in `of[t]`.
///
/// Guard move-attack semantics per generator.rs: approach ≤ speed-1 through
/// empties, then attack cheby-1. Landing set = {src ∪ empty cheby-1 of src};
/// fanout = king_expand(landing) minus src.
pub fn build_attackers_table(pos: &Position, all_occ: u64) -> AttackersTable {
    let mut table = AttackersTable { p1_of: [0u64; 64], p2_of: [0u64; 64] };

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
            king_expand(approach) & !sq_bit
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

// ─── Attacker enumeration ────────────────────────────────────────────────

#[inline]
fn build_attacker_list(pos: &Position, mut bits: u64) -> AttackerList {
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
    else { 0 } // kings excluded — see_capture rejects king targets upstream
}

#[inline]
fn other(p: Player) -> Player {
    match p { Player::P1 => Player::P2, Player::P2 => Player::P1 }
}

// ─── SEE per-capture rollout ─────────────────────────────────────────────

/// Score the exchange initiated by `src` capturing `target`.
///
/// Returns net material change from the *initiator's* (side owning `src`)
/// POV: positive = winning capture, zero = neutral, negative = losing.
///
/// Semantics (mirror MAEE, differences from classical SEE):
/// - Multi-hit kills: 1 damage per attacker; armor absorbs first (down to 0),
///   then HP. Only the final attacker (that lands the killing blow) takes
///   the target square.
/// - Kill-follow-through: after a kill, subsequent attackers are attacking
///   the *new* occupant (initial attacker's material + full HP + full armor).
/// - Guard reach gain: when a blocker vacates a cheby-1 square of the target,
///   Guards adjacent to that vacated square can newly reach the target as an
///   approach square. Maintained incrementally.
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

    // Victim state.
    let mut victim_val = piece_material_of(pos, target);
    if victim_val == 0 { return 0; } // king or empty — caller shouldn't invoke us
    let entry = pos.mailbox[target as usize];
    let mut victim_hp = entry.hp();
    let mut victim_armor = entry.armor();

    // Attacker sets from the table. Lift the specific initiator (`src`) out
    // of the stm's list so it plays first regardless of LVA cost — the caller
    // is asking "what if this specific piece initiates?"
    let mut attackers_stm_bb = table.attackers_of(stm, target);
    let mut attackers_dfd_bb = table.attackers_of(other(stm), target);

    let mut attackers_stm = build_attacker_list(pos, attackers_stm_bb);
    let mut attackers_dfd = build_attacker_list(pos, attackers_dfd_bb);

    // Ply 0: the fixed initiator.
    let initiator = match attackers_stm.remove_sq(src) {
        Some(a) => a,
        // src wasn't in the table — shouldn't happen for a legal move-attack,
        // but if it does (e.g. approach-square asymmetry), synthesize the
        // attacker from the piece at src.
        None => {
            let m = pos.mailbox[src as usize];
            let mat = if pos.champions.0 & src_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
            Attacker { cost: attacker_cost(mat, m.hp(), m.armor()), sq: src }
        }
    };
    attackers_stm_bb &= !src_bit;

    // Ply-by-ply signed gains from stm's POV.
    let mut gains = [0i32; SEE_MAX_PLIES];
    let mut n_gains = 0usize;

    let target_cheby1 = king_expand(target_bit);

    // Attack ply 0: initiator hits the victim.
    let mut vacated = 0u64;
    apply_hit(
        &initiator, 1, stm, target, target_bit, target_cheby1,
        pos,
        &mut victim_val, &mut victim_hp, &mut victim_armor,
        &mut vacated,
        &mut attackers_stm, &mut attackers_stm_bb,
        &mut attackers_dfd, &mut attackers_dfd_bb,
        gains.as_mut_slice(), &mut n_gains,
    );

    // Alternating LVA plies until one side runs out.
    let mut side = other(stm);
    loop {
        let att_opt = if side == stm {
            attackers_stm.pop_front()
        } else {
            attackers_dfd.pop_front()
        };
        let Some(att) = att_opt else { break };
        if n_gains >= SEE_MAX_PLIES { break; }

        let sign = if side == stm { 1 } else { -1 };
        apply_hit(
            &att, sign, stm, target, target_bit, target_cheby1,
            pos,
            &mut victim_val, &mut victim_hp, &mut victim_armor,
            &mut vacated,
            &mut attackers_stm, &mut attackers_stm_bb,
            &mut attackers_dfd, &mut attackers_dfd_bb,
            gains.as_mut_slice(), &mut n_gains,
        );

        side = other(side);
    }

    if n_gains == 0 { return 0; }

    // Stand-pat fold-back. Ply indices from stm's POV:
    //   even i → stm ply → clamp low at 0 (stm won't take a losing step)
    //   odd  i → dfd ply → clamp high at 0 (dfd won't take a losing step)
    // Ply 0 is the initiator's forced attack — we do NOT clamp it (the
    // question is "given this initiation, what's the value?"). If the whole
    // sequence is bad for stm, the caller sees a negative return and can
    // deprioritise ordering-wise; but they can't refuse the ply-0 hit.
    let mut val: i32 = gains[n_gains - 1];
    let mut i = n_gains as i32 - 2;
    while i >= 0 {
        let ply_after = (i + 1) as usize;
        // The clamp is done from the perspective of whoever plays at ply
        // `ply_after`. That player is stm iff `ply_after` is even.
        val = if (ply_after & 1) == 0 { val.max(0) } else { val.min(0) };
        val += gains[i as usize];
        i -= 1;
    }
    val
}

/// Apply one attack ply. Updates victim state, appends to `gains`, and — if
/// this ply was a killing blow — updates the incremental attacker bitmasks
/// (drops the killed attacker's bit, adds newly-reachable Guards adjacent to
/// the vacated origin).
#[inline]
#[allow(clippy::too_many_arguments)]
fn apply_hit(
    att: &Attacker,
    sign: i32,
    stm: Player,
    _target_sq: u8,
    target_bit: u64,
    target_cheby1: u64,
    pos: &Position,
    victim_val: &mut i32,
    victim_hp: &mut u8,
    victim_armor: &mut u8,
    vacated: &mut u64,
    attackers_stm: &mut AttackerList,
    attackers_stm_bb: &mut u64,
    attackers_dfd: &mut AttackerList,
    attackers_dfd_bb: &mut u64,
    gains: &mut [i32],
    n_gains: &mut usize,
) {
    if *victim_armor > 0 {
        *victim_armor -= 1;
        gains[*n_gains] = sign * ARMOR_PER_POINT;
        *n_gains += 1;
        return;
    }
    if *victim_hp > 1 {
        *victim_hp -= 1;
        gains[*n_gains] = sign * HP_PER_POINT;
        *n_gains += 1;
        return;
    }
    // Killing blow.
    let kill_gain = *victim_val + HP_PER_POINT + ARMOR_PER_POINT * (*victim_armor) as i32;
    gains[*n_gains] = sign * kill_gain;
    *n_gains += 1;

    // Attacker now occupies target_sq; its origin is vacated.
    let att_bit = SQ_BIT[att.sq as usize];
    *vacated |= att_bit;
    let att_entry = pos.mailbox[att.sq as usize];
    let att_mat = if pos.champions.0 & att_bit != 0 { CHAMPION_VALUE } else { GUARD_VALUE };
    *victim_val = att_mat;
    *victim_hp = att_entry.hp();
    *victim_armor = att_entry.armor();

    // Determine which side the killer belonged to by parity of `sign`.
    let killer_is_stm = sign > 0;

    // Clear killed attacker's bit.
    if killer_is_stm {
        *attackers_stm_bb &= !att_bit;
    } else {
        *attackers_dfd_bb &= !att_bit;
    }

    // Newly-reachable Guards from the vacated square (only if vacated sits
    // cheby-1 of the target).
    if target_cheby1 & att_bit != 0 {
        let neigh = king_expand(att_bit) & pos.guards.0 & !*vacated & !target_bit;
        let (stm_own, dfd_own) = match stm {
            Player::P1 => (pos.p1_pieces.0, pos.p2_pieces.0),
            Player::P2 => (pos.p2_pieces.0, pos.p1_pieces.0),
        };
        *attackers_stm_bb |= neigh & stm_own & !*attackers_stm_bb;
        *attackers_dfd_bb |= neigh & dfd_own & !*attackers_dfd_bb;
    }

    // Rebuild sorted lists from the maintained bitmasks.
    *attackers_stm = build_attacker_list(pos, *attackers_stm_bb);
    *attackers_dfd = build_attacker_list(pos, *attackers_dfd_bb);
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
    /// kills — SEE should return CHAMPION_VALUE + HP_PER_POINT (the killing
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
}
