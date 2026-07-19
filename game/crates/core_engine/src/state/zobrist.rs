//! Zobrist hashing - incremental u64 fingerprint of a Position.
//!
//! `pos.zobrist` is XOR-updated on every state change so transposition-table
//! lookups and equality short-circuits stay O(1). Slice 7 wires it in.
//!
//! # Design
//!
//! - **Decomposed mailbox keys.** Instead of one key per (square, full u16
//!   mailbox value), we keep separate per-property tables for HP, Armor,
//!   Combo, Skill1, Skill2. Total footprint ~30 KiB across all tables;
//!   `mailbox_xor` decomposes prev/new entries and XORs only the properties
//!   that actually changed (1-2 lookups in the common case, 5 worst-case).
//!
//! - **Occupancy keys separately from mailbox keys.** Mailbox keys alone
//!   can't tell apart "empty square" states from each other (they're all
//!   `MailboxEntry(0)`). `OCC_KEYS[sq][player][kind]` is XOR'd whenever a
//!   piece appears or disappears at `sq` - it pairs naturally with the
//!   bitboard flips in `make_unmake.rs`.
//!
//! - **Bucketed keys for unbounded scalars.** `round_number`, `p1_money`,
//!   `p2_money` are theoretically unbounded; we hash them modulo 256 /
//!   1024 respectively. A TT collision across the modulus is a missed
//!   transposition-table hit, never a correctness issue.
//!
//! - **Transient turn-state is NOT hashed.** `tracked_enemies`,
//!   `tracked_casters`, `champion_credit` are cleared at end-of-turn and
//!   carry no future legality. Hashing them would prevent transposition
//!   between move orderings that arrive at the same end-of-turn position.
//!
//! - **Deterministic seed.** All tables are filled by a const-fn
//!   SplitMix64 seeded at `0x426F61726447616D` ("BoardGam"). Reproducible
//!   across builds and machines; no `rand` dependency.

use super::{MailboxEntry, Position};
use super::position::{GameResult, Phase, PendingBodyguard, Player, modifier_bits};

/// SplitMix64 - a tiny, high-quality 64-bit PRNG suitable for filling
/// deterministic key tables at compile time. See Vigna 2014.
#[inline]
const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const SEED: u64 = 0x426F_6172_6447_616D; // "BoardGam"

// -----------------------------------------------------------------------
// Key tables. All filled by `make_tables()` at const-eval time, so the
// resulting arrays are static read-only data.
// -----------------------------------------------------------------------

struct Tables {
    hp:        [[u64; 3];  64],   // hp ∈ {0,1,2}
    armor:     [[u64; 3];  64],   // armor ∈ {0,1,2}
    combo:     [[u64; 8];  64],   // combo ∈ 0..=7
    skill1:    [[u64; 16]; 64],   // skill_id ∈ 0..=15
    skill2:    [[u64; 16]; 64],
    occ:       [[[u64; 3]; 2]; 64], // [sq][player][kind]; kind: 0=King, 1=Champion, 2=Guard
    side_to_move: u64,
    /// Two independent keys encode the 3-phase state:
    ///   Move  → 0
    ///   Skill → phase_skill
    ///   Draft → phase_draft
    /// Each `set_phase` XORs out the old phase's key (if any) and XORs in
    /// the new phase's key (if any). Move is the canonical "no key" baseline.
    phase_skill: u64,
    phase_draft: u64,
    actions:   [u64; 64],           // actions_remaining ∈ 0..=63 (bucketed if larger)
    pending:   [u64; 8],            // one per bit in pending_modifiers (u8)
    round:     [u64; 256],          // round_number mod 256
    moved:     [u64; 64],           // moved_this_phase, per-square
    money_p1:  [u64; 1024],         // p1_money mod 1024
    money_p2:  [u64; 1024],
    game_result: [u64; 2],          // [P1Wins, P2Wins]; None contributes 0
    // === Appended after existing tables - preserves all prior key indices. ===
    // These keys are XOR'd only when `Position::pending_bodyguard` is `Some`.
    // Index assignment from `splitmix64` is order-sensitive: any new keys
    // MUST be appended at the END of `make_tables()` so existing key values
    // (and therefore every existing zobrist hash) stay byte-identical.
    /// XOR'd once whenever `pending_bodyguard` is `Some`.
    pending_bg_active: u64,
    /// Per-(target_sq, attacker_now) payload key. Eligible-guard list is
    /// deterministic given (target, attacker_now, position bitboards), so we
    /// don't hash it separately. Roughly 32 KiB of static data.
    pending_bg_payload: [[u64; 64]; 64],
}

const fn make_tables() -> Tables {
    let mut s = SEED;
    let mut t = Tables {
        hp:           [[0; 3]; 64],
        armor:        [[0; 3]; 64],
        combo:        [[0; 8]; 64],
        skill1:       [[0; 16]; 64],
        skill2:       [[0; 16]; 64],
        occ:          [[[0; 3]; 2]; 64],
        side_to_move: 0,
        phase_skill:  0,
        phase_draft:  0,
        actions:      [0; 64],
        pending:      [0; 8],
        round:        [0; 256],
        moved:        [0; 64],
        money_p1:     [0; 1024],
        money_p2:     [0; 1024],
        game_result:  [0; 2],
        pending_bg_active:  0,
        pending_bg_payload: [[0; 64]; 64],
    };

    // hp: index 0 contributes 0 so an empty/zero-HP slot adds nothing.
    // Same convention for armor/combo/skill1/skill2 - keeps `full_recompute`
    // simple and matches the property-default-is-zero semantics.
    let mut sq = 0usize;
    while sq < 64 {
        let mut v = 1; while v < 3  { t.hp[sq][v]     = splitmix64(&mut s); v += 1; }
        let mut v = 1; while v < 3  { t.armor[sq][v]  = splitmix64(&mut s); v += 1; }
        let mut v = 1; while v < 8  { t.combo[sq][v]  = splitmix64(&mut s); v += 1; }
        let mut v = 1; while v < 16 { t.skill1[sq][v] = splitmix64(&mut s); v += 1; }
        let mut v = 1; while v < 16 { t.skill2[sq][v] = splitmix64(&mut s); v += 1; }
        sq += 1;
    }

    let mut sq = 0usize;
    while sq < 64 {
        let mut p = 0usize; while p < 2 {
            let mut k = 0usize; while k < 3 {
                t.occ[sq][p][k] = splitmix64(&mut s);
                k += 1;
            }
            p += 1;
        }
        sq += 1;
    }

    t.side_to_move = splitmix64(&mut s);
    t.phase_skill  = splitmix64(&mut s);
    t.phase_draft  = splitmix64(&mut s);

    // actions[0] = 0 keeps full_recompute simple (a position with 0 actions
    // contributes nothing from this axis); other slots are random.
    let mut i = 1; while i < 64 { t.actions[i] = splitmix64(&mut s); i += 1; }

    let mut i = 0; while i <  8   { t.pending[i]  = splitmix64(&mut s); i += 1; }
    // round[0] = 0 so the canonical "round 1" position differs from a hash
    // built by skipping the round axis (avoids accidental zero hashes).
    let mut i = 1; while i <  256 { t.round[i]    = splitmix64(&mut s); i += 1; }
    let mut i = 0; while i <  64  { t.moved[i]    = splitmix64(&mut s); i += 1; }
    // money[0] = 0 (zero money contributes nothing).
    let mut i = 1; while i <  1024 { t.money_p1[i] = splitmix64(&mut s); i += 1; }
    let mut i = 1; while i <  1024 { t.money_p2[i] = splitmix64(&mut s); i += 1; }
    let mut i = 0; while i <  2   { t.game_result[i] = splitmix64(&mut s); i += 1; }

    // === Appended pending-bodyguard keys - preserves prior key indices. ===
    t.pending_bg_active = splitmix64(&mut s);
    let mut tgt = 0usize;
    while tgt < 64 {
        let mut atk = 0usize;
        while atk < 64 {
            t.pending_bg_payload[tgt][atk] = splitmix64(&mut s);
            atk += 1;
        }
        tgt += 1;
    }

    t
}

static T: Tables = make_tables();

// -----------------------------------------------------------------------
// Piece-kind index used by occupancy keys. Local to this module - the
// production code uses bitboard layers directly; only the zobrist layer
// needs to enumerate kinds.
// -----------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum PieceKind { King = 0, Champion = 1, Guard = 2 }

#[inline]
pub fn piece_key(sq: u8, player: Player, kind: PieceKind) -> u64 {
    debug_assert!(sq < 64);
    let p = match player { Player::P1 => 0, Player::P2 => 1 };
    T.occ[sq as usize][p][kind as usize]
}

#[inline]
pub fn mailbox_xor(sq: u8, prev: MailboxEntry, new: MailboxEntry) -> u64 {
    debug_assert!(sq < 64);
    if prev == new { return 0; }
    let s = sq as usize;
    let mut x = 0u64;
    if prev.hp()     != new.hp()     { x ^= T.hp[s][prev.hp() as usize]     ^ T.hp[s][new.hp() as usize]; }
    if prev.armor()  != new.armor()  { x ^= T.armor[s][prev.armor() as usize]  ^ T.armor[s][new.armor() as usize]; }
    if prev.combo()  != new.combo()  { x ^= T.combo[s][prev.combo() as usize]  ^ T.combo[s][new.combo() as usize]; }
    if prev.skill1() != new.skill1() { x ^= T.skill1[s][prev.skill1() as usize] ^ T.skill1[s][new.skill1() as usize]; }
    if prev.skill2() != new.skill2() { x ^= T.skill2[s][prev.skill2() as usize] ^ T.skill2[s][new.skill2() as usize]; }
    x
}

#[inline] pub fn side_key()  -> u64 { T.side_to_move }
/// Per-phase key contribution. Move contributes 0 (canonical baseline);
/// Skill and Draft each carry their own independent random key. `set_phase`
/// XORs out the prev key and in the new key - the helper handles that.
#[inline]
pub fn phase_key_for(phase: Phase) -> u64 {
    match phase {
        Phase::Move  => 0,
        Phase::Skill => T.phase_skill,
        Phase::Draft => T.phase_draft,
    }
}

#[inline]
pub fn actions_key(n: u8) -> u64 {
    // Bucket modulo 64; actions_remaining tops out at 21 even at R200 of
    // Stack M, so this never wraps in practice. The modulo keeps the table
    // small and the lookup branch-free.
    T.actions[(n as usize) & 0x3F]
}

/// XOR the delta needed to move pending_modifiers from `prev` to `new`.
/// Per-bit; only the differing bits contribute.
#[inline]
pub fn pending_mod_xor(prev: u8, new: u8) -> u64 {
    let diff = prev ^ new;
    if diff == 0 { return 0; }
    let mut x = 0u64;
    let mut bits = diff;
    while bits != 0 {
        let b = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        x ^= T.pending[b];
    }
    x
}

/// Single key for `pending_modifiers as u8`'s state. Used by `full_recompute`
/// (XOR every set bit's key). For incremental updates use `pending_mod_xor`.
#[inline]
pub fn pending_mod_state(bits: u8) -> u64 {
    let mut x = 0u64;
    let mut b = bits;
    while b != 0 {
        let i = b.trailing_zeros() as usize;
        b &= b - 1;
        x ^= T.pending[i];
    }
    let _ = modifier_bits::FOCUS; // doc-link sanity; compiler dead-codes this.
    x
}

#[inline]
pub fn round_key(r: u16) -> u64 {
    T.round[(r as usize) & 0xFF]
}

#[inline]
pub fn moved_key(sq: u8) -> u64 {
    debug_assert!(sq < 64);
    T.moved[sq as usize]
}

#[inline] pub fn money_key_p1(m: u16) -> u64 { T.money_p1[(m as usize) & 0x3FF] }
#[inline] pub fn money_key_p2(m: u16) -> u64 { T.money_p2[(m as usize) & 0x3FF] }

#[inline]
pub fn game_result_key(r: Option<GameResult>) -> u64 {
    match r {
        None                   => 0,
        Some(GameResult::P1Wins) => T.game_result[0],
        Some(GameResult::P2Wins) => T.game_result[1],
    }
}

/// XOR contribution of `Position::pending_bodyguard`. `None` is the canonical
/// baseline (contributes 0) - so the bulk of positions, where no Move-Attack
/// is mid-resolution, hash exactly as they did before this key was added.
/// `Some` XORs the active key together with the per-(target, attacker_now)
/// payload key. The eligible-guard list is deterministic given those two
/// squares plus the board bitboards, so it doesn't need separate hashing.
#[inline]
pub fn pending_bg_key(pbg: Option<PendingBodyguard>) -> u64 {
    match pbg {
        None => 0,
        Some(p) => {
            debug_assert!(p.target_sq < 64);
            debug_assert!(p.attacker_now < 64);
            T.pending_bg_active
                ^ T.pending_bg_payload[p.target_sq as usize][p.attacker_now as usize]
        }
    }
}

// -----------------------------------------------------------------------
// Whole-position recompute. Used by setup constructors and by tests that
// verify the incremental hash hasn't drifted from the from-scratch sum.
// -----------------------------------------------------------------------

pub fn full_recompute(pos: &Position) -> u64 {
    let mut h = 0u64;

    // Per-square contributions.
    let occ = pos.p1_pieces | pos.p2_pieces;
    let mut bits = occ.0;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let s = sq as usize;

        // Mailbox properties.
        let e = pos.mailbox[s];
        h ^= mailbox_xor(sq, MailboxEntry(0), e);

        // Occupancy: which player + which kind?
        let player = if pos.p1_pieces.contains(sq) { Player::P1 } else { Player::P2 };
        let kind = if pos.kings.contains(sq)        { PieceKind::King }
                   else if pos.champions.contains(sq) { PieceKind::Champion }
                   else                                { PieceKind::Guard };
        h ^= piece_key(sq, player, kind);
    }

    // moved_this_phase - only meaningful in Move Phase, but hash always.
    let mut bits = pos.moved_this_phase.0;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        h ^= moved_key(sq);
    }

    // Side to move: XOR the side key iff to_move == P2 (canonical: P1 = 0).
    if matches!(pos.to_move, Player::P2) {
        h ^= side_key();
    }

    // Phase: Move contributes 0 (canonical baseline); other phases XOR their key.
    h ^= phase_key_for(pos.current_phase);

    h ^= actions_key(pos.actions_remaining);
    h ^= pending_mod_state(pos.pending_modifiers);
    h ^= round_key(pos.round_number);
    h ^= money_key_p1(pos.p1_money);
    h ^= money_key_p2(pos.p2_money);
    h ^= game_result_key(pos.game_result);
    h ^= pending_bg_key(pos.pending_bodyguard);

    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_nonzero_and_distinct() {
        // Sanity: the seeded PRNG didn't produce a wall of zeros or duplicates.
        assert_ne!(T.side_to_move, 0);
        assert_ne!(T.phase_skill,  0);
        assert_ne!(T.phase_draft,  0);
        assert_ne!(T.phase_skill,  T.phase_draft);
        assert_ne!(T.side_to_move, T.phase_skill);
        assert_ne!(T.hp[0][1], T.hp[0][2]);
        assert_ne!(T.hp[0][1], T.hp[1][1]); // different squares ⇒ different keys
    }

    #[test]
    fn mailbox_xor_zero_when_unchanged() {
        let e = MailboxEntry::default().with_hp(2).with_armor(1).with_skill1(3);
        assert_eq!(mailbox_xor(5, e, e), 0);
    }

    #[test]
    fn mailbox_xor_reversible() {
        let a = MailboxEntry::default().with_hp(2).with_armor(1).with_skill1(3);
        let b = a.with_hp(1).with_combo(2);
        let fwd = mailbox_xor(5, a, b);
        let rev = mailbox_xor(5, b, a);
        assert_eq!(fwd, rev, "mailbox_xor is symmetric - XOR'ing the delta twice cancels");
    }

    #[test]
    fn pending_mod_xor_reversible() {
        let fwd = pending_mod_xor(0b001, 0b011);
        let rev = pending_mod_xor(0b011, 0b001);
        assert_eq!(fwd, rev);
        assert_ne!(fwd, 0);
    }

    #[test]
    fn full_recompute_empty_position() {
        let pos = Position::empty();
        let h = full_recompute(&pos);
        // Empty Position::empty(): no pieces, P1 to move, Move phase,
        // actions_remaining = 0, round = 1, money = 0. Only the round-1 key
        // contributes (actions[0] is 0 by construction).
        assert_eq!(h, round_key(1));
    }

    // --- Commit 1: pending-bodyguard keys append cleanly --------------------

    #[test]
    fn pending_bg_key_none_is_zero() {
        assert_eq!(pending_bg_key(None), 0);
    }

    #[test]
    fn pending_bg_key_some_is_nonzero() {
        let pbg = PendingBodyguard {
            attacker_src: 1,
            attacker_now: 18,
            target_sq: 26,
            eligible: [17, 19, 25, 0],
            eligible_len: 3,
        };
        assert_ne!(pending_bg_key(Some(pbg)), 0);
    }

    #[test]
    fn pending_bg_key_distinct_for_distinct_squares() {
        let a = PendingBodyguard {
            attacker_src: 0, attacker_now: 1, target_sq: 2,
            eligible: [0; 4], eligible_len: 0,
        };
        let b = PendingBodyguard {
            attacker_src: 0, attacker_now: 3, target_sq: 2,
            eligible: [0; 4], eligible_len: 0,
        };
        let c = PendingBodyguard {
            attacker_src: 0, attacker_now: 1, target_sq: 4,
            eligible: [0; 4], eligible_len: 0,
        };
        let ka = pending_bg_key(Some(a));
        let kb = pending_bg_key(Some(b));
        let kc = pending_bg_key(Some(c));
        assert_ne!(ka, kb);
        assert_ne!(ka, kc);
        assert_ne!(kb, kc);
    }

    #[test]
    fn pending_bg_key_xor_reversible() {
        let pbg = PendingBodyguard {
            attacker_src: 5, attacker_now: 13, target_sq: 21,
            eligible: [12, 14, 20, 22], eligible_len: 4,
        };
        let k = pending_bg_key(Some(pbg));
        // Applying twice cancels (defining property of XOR contribution).
        assert_eq!(k ^ k, 0);
    }

    #[test]
    fn pending_bg_active_distinct_from_existing_keys() {
        // Sanity check that the appended splitmix draws don't accidentally
        // collide with any prior key - probability ~2⁻⁶⁴, but cheap insurance
        // against an off-by-one in the splitmix call sequence.
        assert_ne!(T.pending_bg_active, 0);
        assert_ne!(T.pending_bg_active, T.side_to_move);
        assert_ne!(T.pending_bg_active, T.phase_skill);
        assert_ne!(T.pending_bg_active, T.phase_draft);
        assert_ne!(T.pending_bg_active, T.game_result[0]);
        assert_ne!(T.pending_bg_active, T.game_result[1]);
    }

    #[test]
    fn setup_stack_m_zobrist_matches_full_recompute() {
        // With pending_bodyguard == None, the appended keys contribute zero,
        // so the constructor's stored hash should equal full_recompute's
        // output (which now also includes the pending_bg_key call).
        let p = Position::setup_stack_m();
        assert_eq!(p.zobrist, full_recompute(&p));
    }
}
