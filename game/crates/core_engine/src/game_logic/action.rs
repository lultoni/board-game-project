//! Primitive Action and matching Undo Record.
//!
//! # Design (post-audit, see ADR-005 + session-28 audit)
//!
//! Action is **thin and uniform-width** — a single u32 encoding the player's
//! *choice*. Effects (AOE expansion, path-implicit destinations, captured
//! pieces, cleared combo bits) are computed by the resolver in `make()` and
//! recorded in a separate `Undo` record for reversibility. This matches the
//! Stockfish / Sabberstone / Forge convention: the action is what the
//! player picked; the consequence is reconstructed at apply-time.
//!
//! # Action bit layout (u32)
//!
//! ```text
//!   bits  0..6   src           (6 bits, 0..=63)   caster square
//!   bits  6..12  target        (6 bits, 0..=63)   primary target square
//!   bits 12..14  kind          (2 bits)           ActionKind variant
//!   bits 14..18  skill_id      (4 bits, 0..=15)   skill index, 0 = none/sentinel
//!   bits 18..22  choice_idx    (4 bits, 0..=15)   player disambiguation
//!                                                  (e.g. Shove direction 0..=7,
//!                                                   Retreat Guard-pick 0..=N)
//!   bits 22..32  reserved
//! ```
//!
//! AOE skills (Tempest) carry `target` = primary; the resolver computes the
//! 8 neighbour pushes and records them in the Undo. Path-implicit skills
//! (Retreat) pre-resolve their destination in the generator and write it
//! into `target`. Direction-only skills (Shove) use `choice_idx` for the
//! 8 cardinal/diagonal directions.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Action(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionKind {
    Move     = 0,
    Skill    = 1,
    EndPhase = 2,
    EndTurn  = 3,
}

impl Action {
    #[inline]
    pub fn encode(src: u8, target: u8, kind: ActionKind, skill_id: u8, choice_idx: u8) -> Self {
        debug_assert!(src        < 64);
        debug_assert!(target     < 64);
        debug_assert!(skill_id   < 16);
        debug_assert!(choice_idx < 16);
        let bits =  (src        as u32)
                 | ((target     as u32) << 6)
                 | ((kind as u8 as u32) << 12)
                 | ((skill_id   as u32) << 14)
                 | ((choice_idx as u32) << 18);
        Action(bits)
    }

    #[inline] pub fn src(self)        -> u8 { (self.0        & 0b111111) as u8 }
    #[inline] pub fn target(self)     -> u8 { ((self.0 >>  6) & 0b111111) as u8 }
    #[inline] pub fn kind(self)       -> ActionKind {
        match (self.0 >> 12) & 0b11 {
            0 => ActionKind::Move,
            1 => ActionKind::Skill,
            2 => ActionKind::EndPhase,
            _ => ActionKind::EndTurn,
        }
    }
    #[inline] pub fn skill_id(self)   -> u8 { ((self.0 >> 14) & 0b1111)   as u8 }
    #[inline] pub fn choice_idx(self) -> u8 { ((self.0 >> 18) & 0b1111)   as u8 }
}

/// Undo Record — written by `make()`, consumed by `unmake()` to perfectly
/// reverse an Action. Reversibility cannot live inside the Action itself
/// because effects are state-dependent (AOE membership, captured HP/armor,
/// turn-scoped modifier bits that were consumed, combo-credit bits set).
///
/// Width is permitted to be much larger than Action — there is one Undo per
/// search-stack frame, not one per Position.
#[derive(Clone, Debug, Default)]
pub struct Undo {
    /// The action this Undo reverses, for sanity checks.
    pub action: u32,

    /// Snapshot of `pending_modifiers` before this action consumed any.
    pub prev_pending_modifiers: u8,

    /// Snapshot of phase + actions_remaining before this action.
    pub prev_phase: u8,
    pub prev_actions_remaining: u8,

    /// Money deltas (signed-on-paper, stored as signed i16 to capture
    /// Steal moving money between players).
    pub p1_money_delta: i16,
    pub p2_money_delta: i16,

    /// Combo-credit + tracked-enemies snapshot.
    pub prev_champion_credit: u64,
    pub prev_tracked_enemies: [u8; crate::state::position::MAX_TRACKED_ENEMIES],
    pub prev_tracked_enemies_len: u8,

    /// Per-square mailbox snapshots — entries that this action mutated.
    /// `affected_count` is the active length of `affected_squares` /
    /// `affected_prev_entries`. Capacity 16 is sized for the worst-case AOE
    /// (Tempest: target + 8 neighbours = 9; Swap = 2; most skills ≤ 2).
    pub affected_count: u8,
    pub affected_squares: [u8; 16],
    pub affected_prev_entries: [u16; 16],

    /// Bitboard deltas — XOR these to revert.
    pub p1_pieces_xor: u64,
    pub p2_pieces_xor: u64,
    pub kings_xor:     u64,
    pub champions_xor: u64,
    pub guards_xor:    u64,

    /// Zobrist delta — XOR to revert.
    pub zobrist_xor: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_encode_decode_roundtrip() {
        let a = Action::encode(/*src*/ 12, /*target*/ 47, ActionKind::Skill, /*skill*/ 9, /*choice*/ 5);
        assert_eq!(a.src(),        12);
        assert_eq!(a.target(),     47);
        assert_eq!(a.kind(),       ActionKind::Skill);
        assert_eq!(a.skill_id(),   9);
        assert_eq!(a.choice_idx(), 5);
    }

    #[test]
    fn action_kind_variants_roundtrip() {
        for k in [ActionKind::Move, ActionKind::Skill, ActionKind::EndPhase, ActionKind::EndTurn] {
            let a = Action::encode(0, 0, k, 0, 0);
            assert_eq!(a.kind(), k);
        }
    }

    #[test]
    fn action_default_is_zero() {
        // The TT uses Action::default() as the "no entry" sentinel.
        // It must serialise to 0 so a freshly-allocated TT has no false hits.
        assert_eq!(Action::default().0, 0);
    }
}
