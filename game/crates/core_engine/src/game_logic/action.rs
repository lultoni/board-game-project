//! Primitive Action and matching Undo Record.
//!
//! # Design (post-audit, see ADR-005 + session-28 audit + session-30 fixup)
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
//!   bits  0..6   src           (6 bits, 0..=63)   caster / mover square
//!   bits  6..12  target        (6 bits, 0..=63)   primary target square
//!   bits 12..14  kind          (2 bits)           ActionKind variant
//!   bits 14..18  skill_id      (4 bits, 0..=15)   skill index, 0 = none/sentinel
//!   bits 18..22  choice_idx    (4 bits, 0..=15)   player disambiguation
//!                                                  - Shove direction: 0..=7 (N..NW)
//!                                                  - Retreat Guard-pick: 0..=N
//!                                                  - Move-Attack bodyguard pick:
//!                                                      0 = no redirect (attacker
//!                                                      hits the named target),
//!                                                      1..=k = redirect to k-th
//!                                                      eligible adjacent Guard
//!                                                      (canonical ordering by
//!                                                      square index, ascending)
//!   bit  22      focus_mode    (1 bit)            Focus interpretation tag
//!                                                  (Slice 6 / oq-70 resolution).
//!                                                  Only meaningful when the
//!                                                  caster has Focus pending AND
//!                                                  the skill is a Move-skill
//!                                                  whose activation-range and
//!                                                  effect-range are both real.
//!                                                  0 = Focus buffs activation-range,
//!                                                  1 = Focus buffs effect-range.
//!                                                  Generator emits this bit
//!                                                  per legal interpretation;
//!                                                  resolver reads it.
//!   bits 23..29  aux_sq        (6 bits, 0..=63)   Auxiliary square. Used by
//!                                                  Focus-retargeted Self-only
//!                                                  skills (Shield/Dash/Retreat):
//!                                                  the caster (src) channels
//!                                                  the skill onto an *adjacent
//!                                                  ally* (aux_sq), which is the
//!                                                  actual recipient/mover.
//!                                                  Meaningless unless bit 29
//!                                                  (has_aux) is set.
//!   bit  29      has_aux       (1 bit)            1 iff aux_sq carries a real
//!                                                  square. Cheaper than
//!                                                  reserving a sentinel value.
//!   bits 30..32  reserved
//! ```
//!
//! ## Move-phase actions (kind=Move)
//!
//! - **Plain move:** `target` is an empty square. `choice_idx` = 0 (unused).
//! - **Move-Attack:** `target` is an *enemy*-occupied square. The mover does
//!   NOT enter the target tile (Stack M); the enemy takes 1 damage. Bodyguard
//!   redirect is encoded via `choice_idx` — see above.
//! - Bodyguard enumeration in the generator: for every move-attack target
//!   that has ≥1 eligible adjacent friendly Guard (i.e. the *defender's*
//!   adjacent Guard), the generator emits one action per `choice_idx` value
//!   (0 = no redirect, 1..=k = each Guard). The UI/Session layer mirrors this
//!   by deferring on the defender's choice during HvH play before forwarding
//!   the chosen action to `make()`.
//!
//! ## Tempest AOE (Skill kind)
//!
//! Stack M Tempest: "Target takes 1 damage. All pieces *adjacent to the
//! target* are pushed 1 tile away from the target. Caster not affected."
//! The target itself is NOT pushed — only its (up to 8) neighbours, minus
//! the caster if the caster sits on a neighbour square. The resolver
//! computes the push set inside `make()`; the Undo stores prior mailbox
//! entries for any neighbour that ended up displaced or pushed off the
//! board (which removes it, per the no-falling-off rule once defined —
//! TODO file as OQ if Stack M is silent on push-off-board).
//!
//! Direction-only skills (Shove) use `choice_idx` for the 8 cardinal/
//! diagonal directions. Path-implicit skills (Retreat) pre-resolve their
//! destination in the generator and write it into `target`.
//!
//! ## Focus retargeting Self-only skills (Slice 6 / oq-70 final)
//!
//! Focus on Shield/Dash/Retreat lets the caster channel the skill onto an
//! adjacent ally instead of themselves. Encoding:
//! - `src` = caster (the one paying the cost and consuming Focus).
//! - `target` = primary target (matches the un-retargeted convention: for
//!   Shield this is the recipient; for Dash/Retreat this is the destination
//!   the *recipient* moves to).
//! - `aux_sq` = the adjacent-ally recipient.
//! - `has_aux` = 1.
//! The resolver applies the Shield-armor / Dash-move / Retreat-move effect
//! to `aux_sq`, not `src`.

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

    /// Same as `encode` but sets bit 22 (focus_mode) to mark this action as
    /// "Focus buffs effect-range, not activation-range." Only used for
    /// Move-skills (Dash/Blast/Shove) when Focus is pending and both
    /// activation/effect interpretations are meaningful (per oq-70 resolution).
    #[inline]
    pub fn encode_focus_effect(src: u8, target: u8, kind: ActionKind,
                               skill_id: u8, choice_idx: u8) -> Self {
        let mut a = Self::encode(src, target, kind, skill_id, choice_idx);
        a.0 |= 1 << 22;
        a
    }

    /// Encode an action with an auxiliary square. Used for Focus-retargeted
    /// Self-only skills (Shield/Dash/Retreat) where the caster channels the
    /// effect onto an adjacent ally. Sets bit 29 (has_aux) and stores the
    /// ally's square in bits 23..29.
    #[inline]
    pub fn encode_with_aux(src: u8, target: u8, kind: ActionKind,
                           skill_id: u8, choice_idx: u8, aux_sq: u8) -> Self {
        debug_assert!(aux_sq < 64);
        let mut a = Self::encode(src, target, kind, skill_id, choice_idx);
        a.0 |= (aux_sq as u32) << 23;
        a.0 |= 1 << 29;
        a
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
    /// True iff bit 22 is set — Focus is buffing this skill's effect-range
    /// rather than its activation-range. See `encode_focus_effect`.
    #[inline] pub fn focus_effect_mode(self) -> bool { (self.0 >> 22) & 1 != 0 }
    /// True iff bit 29 is set — `aux_sq()` carries a real square.
    #[inline] pub fn has_aux(self)    -> bool { (self.0 >> 29) & 1 != 0 }
    /// Auxiliary square (Focus-retargeted recipient). Only meaningful when
    /// `has_aux()` is true; otherwise reads back the reserved zero bits.
    #[inline] pub fn aux_sq(self)     -> u8 { ((self.0 >> 23) & 0b111111) as u8 }
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

    /// Snapshot of `game_result` before this action. `unmake` restores it
    /// so that a King-capturing Move-Attack is perfectly reversible. Stored
    /// as a `u8` tag: 0 = None, 1 = P1Wins, 2 = P2Wins. Keeping the Undo
    /// `Default` derivable trumps the type-safety win of `Option<GameResult>`
    /// here — the conversion lives in two helpers on `make()` / `unmake()`.
    pub prev_game_result: u8,

    /// Snapshot of `pending_modifiers` before this action consumed any.
    pub prev_pending_modifiers: u8,

    /// Snapshot of phase + actions_remaining before this action.
    pub prev_phase: u8,
    pub prev_actions_remaining: u8,

    /// Snapshot of `to_move` before this action. Only end-of-turn flips it,
    /// but unmake must restore it deterministically. 0 = P1, 1 = P2.
    pub prev_to_move: u8,

    /// Snapshot of `moved_this_phase` (Move-Phase only) and `round_number`.
    /// Both must round-trip exactly under unmake.
    pub prev_moved_this_phase: u64,
    pub prev_round_number: u16,

    /// Money deltas (signed-on-paper, stored as signed i16 to capture
    /// Steal moving money between players).
    pub p1_money_delta: i16,
    pub p2_money_delta: i16,

    /// Combo-credit + tracked-enemies + tracked-casters snapshot.
    pub prev_champion_credit: u64,
    pub prev_tracked_enemies: [u8; crate::state::position::MAX_TRACKED_ENEMIES],
    pub prev_tracked_enemies_len: u8,
    pub prev_tracked_casters: [u8; crate::state::position::MAX_TRACKED_CASTERS],
    pub prev_tracked_casters_len: u8,

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

    #[test]
    fn action_encode_with_aux_roundtrip() {
        let a = Action::encode_with_aux(
            /*src*/ 12, /*target*/ 47, ActionKind::Skill,
            /*skill*/ 9, /*choice*/ 5, /*aux*/ 33,
        );
        assert_eq!(a.src(),        12);
        assert_eq!(a.target(),     47);
        assert_eq!(a.kind(),       ActionKind::Skill);
        assert_eq!(a.skill_id(),   9);
        assert_eq!(a.choice_idx(), 5);
        assert_eq!(a.aux_sq(),     33);
        assert!(a.has_aux());
        // focus_effect_mode unchanged (bit 22 stays 0).
        assert!(!a.focus_effect_mode());
    }

    #[test]
    fn action_plain_encode_has_no_aux() {
        let a = Action::encode(0, 0, ActionKind::Skill, 1, 0);
        assert!(!a.has_aux());
        assert_eq!(a.aux_sq(), 0);
    }
}
