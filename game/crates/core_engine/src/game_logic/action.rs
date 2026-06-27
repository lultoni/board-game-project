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
//!   bits 23..29  aux_sq /     (6 bits, 0..=63)   DUAL-USE — disambiguated
//!                approach_sq                       by `kind()`:
//!                                                  - kind=Skill: aux_sq, the
//!                                                    Focus-retargeted Self-only
//!                                                    recipient (Shield/Dash/
//!                                                    Retreat onto adjacent ally).
//!                                                  - kind=Move:  approach_sq,
//!                                                    the penultimate tile the
//!                                                    attacker stops on along a
//!                                                    Move-Attack path. For
//!                                                    speed-1, approach_sq=src.
//!                                                    For speed-2, one of the
//!                                                    empty tiles adjacent to
//!                                                    target that the attacker
//!                                                    reached via BFS.
//!                                                  Meaningless unless bit 29 is set.
//!   bit  29      has_aux /     (1 bit)            1 iff bits 23..29 carry a
//!                has_approach                       real square (aux for Skill,
//!                                                  approach for Move-Attack).
//!                                                  Plain moves leave this 0
//!                                                  and the mover's destination
//!                                                  is `target`.
//!   bits 30..32  reserved (bit 30 used for DRAFT_TURN_TAG — see below)
//! ```
//!
//! ## DraftTurn (L8 — pre-game skill assignment)
//!
//! When **bit 30** is set, the action is a `DraftTurn` and the rest of the
//! u32 is reinterpreted entirely. The `kind` / `src` / `target` accessors are
//! meaningless on a DraftTurn-tagged action; callers must check
//! `is_draft_turn()` first and use the `draft_pick1` / `draft_pick2`
//! accessors instead. DraftTurn actions are only emitted by the generator
//! while `pos.current_phase == Phase::Draft`; the regular Move/Skill phase
//! generator never sets bit 30.
//!
//! Layout when bit 30 = 1:
//! ```text
//!   bits  0..4    pick1.skill_id   (4 bits, 1..=15; 0 is illegal in a pick)
//!   bits  4..10   pick1.sq         (6 bits, 0..=63)
//!   bit  10       pick1.slot       (0 = slot1, 1 = slot2)
//!   bits 11..15   pick2.skill_id   (4 bits, 1..=15)
//!   bits 15..21   pick2.sq         (6 bits, 0..=63)
//!   bit  21       pick2.slot       (0 = slot1, 1 = slot2)
//!   bits 22..30   reserved (must be 0)
//!   bit  30       DRAFT_TURN_TAG = 1
//!   bit  31       reserved (must be 0)
//! ```
//!
//! ## Move-phase actions (kind=Move)
//!
//! - **Plain move:** `target` is an empty square. `choice_idx` = 0 (unused).
//!   Bit 29 (has_approach) = 0. Mover ends on `target`.
//! - **Move-Attack:** `target` is an *enemy*-occupied square. The mover advances
//!   to `approach_sq` (bits 23..29, with bit 29 set), then the defender takes
//!   1 damage. The mover does NOT enter the target tile. For speed-1 attackers
//!   `approach_sq == src` (no relocation). For speed-2 attackers reaching a
//!   distance-2 enemy, `approach_sq` is one of the empty neighbours of the
//!   target reachable from `src` in exactly one step.
//! - Bodyguard redirect is encoded via `choice_idx`. Eligibility is
//!   *dual-adjacency*: a Guard intercepts only if it sits adjacent to BOTH
//!   the defender AND `approach_sq`.
//! - Multiple zig-zag paths to the same `target` produce *different*
//!   `approach_sq` values, each a distinct legal action with potentially
//!   different Bodyguard-eligible Guards and different attacker end positions.
//! - Bodyguard enumeration in the generator: for every (src, target,
//!   approach_sq) triple, compute dual-adjacency Guard set; emit one action
//!   per `choice_idx` value (0 = no redirect, 1..=k = each eligible Guard,
//!   canonical ordering by square index ascending). The UI/Session layer
//!   mirrors this by deferring on the defender's choice during HvH play
//!   before forwarding the chosen action to `make()`.
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
//! ## BodyguardChoice (Commit 2 — defender-driven Bodyguard resolution)
//!
//! Bit **31** is `BG_CHOICE_TAG`. When set, the action is a `BodyguardChoice`
//! ply played by the *defender* in response to a tentatively-applied Move-Attack
//! that left `Position::pending_bodyguard = Some(_)`. Layout when bit 31 = 1:
//!
//! ```text
//!   bits  0..4   idx              (4 bits, 0..=N where N = eligible_len ≤ 4)
//!                                    0 = decline redirect (named target takes the hit)
//!                                    k = redirect to eligible[k-1]
//!   bit  31      BG_CHOICE_TAG    = 1
//!   all other bits                 reserved, must be 0
//! ```
//!
//! `kind()`, `src()`, `target()`, `skill_id()`, `has_aux()`, `has_approach()`
//! are MEANINGLESS on a BodyguardChoice action — callers MUST check
//! `is_bodyguard_choice()` first and use `bg_guard_idx()` instead. The
//! attacker / target / approach squares are recovered from
//! `pos.pending_bodyguard` rather than from the action bits themselves —
//! the defender is committing only to "which of the eligible squares takes
//! the hit," and the engine has the rest cached.
//!
//! Bit 31 was previously reserved (DraftTurn uses bit 30; bits 23..29 carry
//! aux_sq whose 5th bit collides with bit 28 when aux_sq ≥ 32). Bit 31 is
//! the only truly free bit in the layout, so the encoding must occupy a low
//! bit range for `idx` (0..4) rather than reusing the `choice_idx` slot.
//!
//! BodyguardChoice is distinct from DraftTurn (bit 30) and from regular
//! Move/Skill actions (bits 30, 31 both 0). The three families partition
//! the legal action space — at most one of `is_draft_turn()` /
//! `is_bodyguard_choice()` may be true on a well-formed action.
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
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

    /// Encode a Move-Attack action. `approach_sq` is the penultimate tile —
    /// the empty tile adjacent to `target` that the attacker physically moves
    /// onto along the attack path. For speed-1 attackers, `approach_sq` == `src`.
    /// For speed-2 attackers at Chebyshev distance 2, it is one of the empty
    /// neighbours of `target` reachable from `src` in exactly one BFS step.
    ///
    /// `approach_sq` is stored in bits 23..29 and bit 29 is set as a tag
    /// (sharing the layout with `aux_sq` — disambiguated by `kind() == Move`).
    /// `choice_idx`: 0 = no Bodyguard redirect; 1..=k = redirect to k-th
    /// eligible Guard (sorted ascending by square index), where eligibility is
    /// "Guard adjacent to BOTH the defender AND `approach_sq`."
    #[inline]
    pub fn encode_move_attack(src: u8, target: u8, choice_idx: u8, approach_sq: u8) -> Self {
        debug_assert!(approach_sq < 64);
        let mut a = Self::encode(src, target, ActionKind::Move, 0, choice_idx);
        a.0 |= (approach_sq as u32) << 23;
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
    /// Approach square (Move-Attack penultimate tile). Only meaningful when
    /// `kind() == Move` AND bit 29 is set. For plain moves, bit 29 is 0 and
    /// the attacker's destination is `target()`. For Move-Attacks the
    /// attacker stops on `approach_sq()`, then deals damage to `target()`
    /// (or a Bodyguard redirect target).
    #[inline] pub fn approach_sq(self) -> u8 { ((self.0 >> 23) & 0b111111) as u8 }
    /// True iff this Move-Attack action carries an approach square (bit 29
    /// set on a Move-kind action). Plain moves never set this bit.
    #[inline] pub fn has_approach(self) -> bool {
        matches!(self.kind(), ActionKind::Move) && (self.0 >> 29) & 1 != 0
    }

    // ---- DraftTurn (bit 30 = 1) ----

    /// Bit mask for the DraftTurn tag (bit 30). When set, the action's other
    /// bits are reinterpreted entirely — see the module doc-comment for layout.
    pub const DRAFT_TURN_TAG: u32 = 1 << 30;

    /// Encode a draft turn — two (skill_id, sq, slot) picks the side-to-move
    /// is committing in one DraftTurn ply. Each `skill_id` must be 1..=15
    /// (0 is illegal in a pick), each `sq` must be 0..=63, each `slot` is
    /// 0 (slot1) or 1 (slot2). Caller is responsible for cross-validation
    /// (target piece belongs to stm, slot currently empty, same-piece-same-
    /// skill check across both picks). Those checks live in
    /// `legal_draft_turns` and `apply_draft_turn`.
    #[inline]
    pub fn encode_draft_turn(
        skill1: u8, sq1: u8, slot1: u8,
        skill2: u8, sq2: u8, slot2: u8,
    ) -> Self {
        debug_assert!(skill1 >= 1 && skill1 < 16);
        debug_assert!(skill2 >= 1 && skill2 < 16);
        debug_assert!(sq1    < 64);
        debug_assert!(sq2    < 64);
        debug_assert!(slot1  < 2);
        debug_assert!(slot2  < 2);
        let bits =  (skill1 as u32)
                 | ((sq1    as u32) << 4)
                 | ((slot1  as u32) << 10)
                 | ((skill2 as u32) << 11)
                 | ((sq2    as u32) << 15)
                 | ((slot2  as u32) << 21)
                 | Self::DRAFT_TURN_TAG;
        Action(bits)
    }

    /// True iff bit 30 is set — this action is a DraftTurn and its other
    /// bits use the draft layout (see module doc-comment). When this is
    /// true, `kind()` / `src()` / `target()` / etc. are meaningless and
    /// must not be consulted.
    #[inline]
    pub fn is_draft_turn(self) -> bool {
        self.0 & Self::DRAFT_TURN_TAG != 0
    }

    /// First pick of a DraftTurn: `(skill_id, sq, slot)`. Caller must check
    /// `is_draft_turn()` first; reading this from a non-DraftTurn action
    /// yields garbage (sliced from the regular-action bit layout).
    #[inline]
    pub fn draft_pick1(self) -> (u8, u8, u8) {
        let skill = (self.0        & 0b1111)   as u8;
        let sq    = ((self.0 >>  4) & 0b111111) as u8;
        let slot  = ((self.0 >> 10) & 0b1)      as u8;
        (skill, sq, slot)
    }

    /// Second pick of a DraftTurn: `(skill_id, sq, slot)`. Same caveat as
    /// `draft_pick1`.
    #[inline]
    pub fn draft_pick2(self) -> (u8, u8, u8) {
        let skill = ((self.0 >> 11) & 0b1111)   as u8;
        let sq    = ((self.0 >> 15) & 0b111111) as u8;
        let slot  = ((self.0 >> 21) & 0b1)      as u8;
        (skill, sq, slot)
    }

    // ---- BodyguardChoice (bit 31 = 1) ----

    /// Bit mask for the BodyguardChoice tag (bit 31). When set, the action's
    /// regular fields (kind/src/target/skill/has_aux/has_approach) are
    /// meaningless — the action carries only an `idx` in bits 0..4 (0 = no
    /// redirect, k = redirect to `pending_bodyguard.eligible[k-1]`). Bit 31
    /// is the only truly free bit (bits 23..29 dual-encode aux_sq whose
    /// upper values would collide with anything mid-word; bit 30 is DraftTurn).
    /// See the module doc-comment.
    pub const BG_CHOICE_TAG: u32 = 1 << 31;

    /// Maximum legal value of the `idx` field on a BodyguardChoice action. The
    /// engine's `MAX_BODYGUARD_ELIGIBLE` is 4, so an idx of 0 (decline) plus
    /// 1..=4 (pick the k-th eligible Guard) gives a 0..=4 range. Encoded into
    /// 4 bits (bits 0..4), so the type-level max is 15; the semantic max is
    /// the smaller bound enforced here.
    pub const BG_CHOICE_MAX_IDX: u8 = crate::state::position::MAX_BODYGUARD_ELIGIBLE as u8;

    /// Encode a defender's BodyguardChoice ply. `idx == 0` declines the
    /// redirect (the named target takes the hit). `idx` in `1..=eligible_len`
    /// redirects damage to `pos.pending_bodyguard.eligible[idx-1]`. Caller
    /// must consult `pos.pending_bodyguard` for the upper bound — this
    /// encoder only enforces the type-level `BG_CHOICE_MAX_IDX` cap.
    ///
    /// The encoding deliberately omits src/target/kind: those are recovered
    /// from `pending_bodyguard` at apply-time, so the engine is the single
    /// source of truth for which attack the choice resolves.
    #[inline]
    pub fn encode_bodyguard_choice(idx: u8) -> Self {
        debug_assert!(idx <= Self::BG_CHOICE_MAX_IDX,
            "BodyguardChoice idx {} exceeds MAX_BODYGUARD_ELIGIBLE ({})",
            idx, Self::BG_CHOICE_MAX_IDX);
        Action((idx as u32 & 0b1111) | Self::BG_CHOICE_TAG)
    }

    /// True iff bit 31 is set — this action is a `BodyguardChoice` reply.
    /// When this is true, `kind()` / `src()` / `target()` etc. are
    /// meaningless and must not be consulted; use `bg_guard_idx()` instead.
    #[inline]
    pub fn is_bodyguard_choice(self) -> bool {
        self.0 & Self::BG_CHOICE_TAG != 0
    }

    /// Defender's pick index for a BodyguardChoice action. `0` = decline
    /// redirect (named target takes the hit), `k` = redirect to
    /// `pending_bodyguard.eligible[k-1]`. Reading this from a non-
    /// BodyguardChoice action yields garbage — caller must check
    /// `is_bodyguard_choice()` first.
    #[inline]
    pub fn bg_guard_idx(self) -> u8 {
        (self.0 & 0b1111) as u8
    }
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

    /// Snapshot of `pending_bodyguard` before this action. Tentative Move-
    /// Attacks write `Some(...)`; BodyguardChoice clears it. Each `make()`
    /// has its own Undo, so the two-stage transaction unwinds cleanly.
    pub prev_pending_bodyguard: Option<crate::state::position::PendingBodyguard>,

    /// Snapshot of `moved_this_phase` (Move-Phase only) and `round_number`.
    /// Both must round-trip exactly under unmake.
    pub prev_moved_this_phase: u64,
    pub prev_round_number: u16,

    /// Money deltas (signed-on-paper, stored as signed i16 to capture
    /// Steal moving money between players).
    pub p1_money_delta: i16,
    pub p2_money_delta: i16,

    /// Combo-credit + tracked-enemies + tracked-casters snapshot.
    pub prev_champion_credit: u128,
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

    #[test]
    fn draft_turn_encode_decode_roundtrip() {
        let a = Action::encode_draft_turn(
            /*skill1*/ 7, /*sq1*/ 3,  /*slot1*/ 0,
            /*skill2*/ 12,/*sq2*/ 60, /*slot2*/ 1,
        );
        assert!(a.is_draft_turn());
        assert_eq!(a.draft_pick1(), (7, 3, 0));
        assert_eq!(a.draft_pick2(), (12, 60, 1));
    }

    #[test]
    fn draft_turn_tag_distinguishes_from_regular_action() {
        let regular = Action::encode(63, 63, ActionKind::Skill, 15, 15);
        let draft   = Action::encode_draft_turn(15, 63, 1, 15, 63, 1);
        assert!(!regular.is_draft_turn());
        assert!( draft.is_draft_turn());
    }

    #[test]
    fn draft_turn_extremal_values_roundtrip() {
        let a = Action::encode_draft_turn(
            /*skill1*/ 1,  /*sq1*/ 0,  /*slot1*/ 0,
            /*skill2*/ 15, /*sq2*/ 63, /*slot2*/ 1,
        );
        assert_eq!(a.draft_pick1(), (1, 0, 0));
        assert_eq!(a.draft_pick2(), (15, 63, 1));
    }

    // ---- BodyguardChoice (Commit 2) ----

    #[test]
    fn bodyguard_choice_encode_decode_roundtrip() {
        for idx in 0..=Action::BG_CHOICE_MAX_IDX {
            let a = Action::encode_bodyguard_choice(idx);
            assert!(a.is_bodyguard_choice(),
                "encode_bodyguard_choice({idx}) must set BG_CHOICE_TAG");
            assert_eq!(a.bg_guard_idx(), idx,
                "bg_guard_idx must round-trip for idx={idx}");
            assert!(!a.is_draft_turn(),
                "BodyguardChoice and DraftTurn tags must not collide");
        }
    }

    #[test]
    fn bodyguard_choice_default_action_is_not_bg() {
        // Default action (zeroed u32) must not look like a BodyguardChoice —
        // the TT relies on Action::default() being a recognisable sentinel.
        assert!(!Action::default().is_bodyguard_choice());
    }

    #[test]
    fn bodyguard_choice_distinct_from_every_regular_kind() {
        // A regular action of any kind must never claim is_bodyguard_choice().
        for k in [ActionKind::Move, ActionKind::Skill, ActionKind::EndPhase, ActionKind::EndTurn] {
            let a = Action::encode(/*src*/ 0, /*tgt*/ 0, k, /*skill*/ 0, /*choice*/ 0);
            assert!(!a.is_bodyguard_choice(),
                "regular {k:?} action must not look like a BodyguardChoice");
        }
        // Move-Attack with non-zero approach/choice mustn't either.
        let mv = Action::encode_move_attack(/*src*/ 5, /*tgt*/ 12, /*choice*/ 3, /*approach*/ 6);
        assert!(!mv.is_bodyguard_choice());
        // Focus-effect, aux-encoded — none should collide with bit 28.
        let fx = Action::encode_focus_effect(0, 0, ActionKind::Skill, 1, 0);
        assert!(!fx.is_bodyguard_choice());
        let ax = Action::encode_with_aux(0, 0, ActionKind::Skill, 1, 0, 1);
        assert!(!ax.is_bodyguard_choice());
        // DraftTurn must not look like a BodyguardChoice and vice versa.
        let dr = Action::encode_draft_turn(1, 0, 0, 2, 1, 1);
        assert!(!dr.is_bodyguard_choice());
        let bg = Action::encode_bodyguard_choice(2);
        assert!(!bg.is_draft_turn());
    }

    #[test]
    fn bodyguard_choice_only_sets_tag_and_idx() {
        // Encoding must not bleed into other bit ranges — src/target/kind/
        // skill/aux/approach/has_aux/has_approach must read back as zero.
        // (They are meaningless on BG, but we still want the raw bits clean.)
        let a = Action::encode_bodyguard_choice(3);
        assert_eq!(a.0, 3u32 | Action::BG_CHOICE_TAG,
            "BodyguardChoice raw bits must be exactly tag | idx");
    }
}
