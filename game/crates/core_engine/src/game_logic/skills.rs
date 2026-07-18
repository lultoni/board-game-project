//! Skill enum + table-driven property lookups.
//!
//! Source of truth for cost/range/category/owner contracts is the Stack-M body
//! (`SELECT body FROM stacks WHERE id='stack-m';`) Skill Reference table.
//!
//! IDs are stable: the mailbox stores skill IDs as 4-bit fields, with 0 as the
//! "unequipped" sentinel. The Action's `skill_id` bits also use this encoding.
//!
//! # Slice 3 scope
//!
//! Slice 3 provides the *enumeration + lookup* surface. The resolver bodies in
//! `make_unmake::apply_skill` are stubs that panic; Slice 4+ replace them per
//! skill.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Skill {
    Lance   = 1,
    Hook    = 2,
    Break   = 3,
    Steal   = 4,
    Tempest = 5,
    Shield  = 6,
    Heal    = 7,
    Plate   = 8,
    Dash    = 9,
    Blast   = 10,
    Shove   = 11,
    Swap    = 12,
    Retreat = 13,
    Focus   = 14,
    Charge  = 15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillCategory { Strike, Shield, Move, Mystic }

/// Skill target-owner contract - applied at generator time so emitted Skill
/// actions are already semantically well-typed by the time `apply_skill`
/// sees them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetOwner {
    /// Target square must be enemy-occupied (Lance/Hook/Break/Steal/Tempest/Blast).
    Enemy,
    /// Target square must be ally-occupied (Heal/Plate/Swap).
    Ally,
    /// Target may be either side (Shove - push ally OR enemy).
    Either,
    /// Target must be an empty square along the Path (Dash/Retreat - Slice 5).
    Empty,
    /// Caster targets itself (Shield/Focus/Charge). `src == tgt`.
    SelfOnly,
}

/// Convert a 4-bit skill id (0..=15) to a `Skill`. id=0 is the "unequipped"
/// sentinel and returns `None`.
#[inline]
pub fn skill_from_id(id: u8) -> Option<Skill> {
    Some(match id {
        1  => Skill::Lance,
        2  => Skill::Hook,
        3  => Skill::Break,
        4  => Skill::Steal,
        5  => Skill::Tempest,
        6  => Skill::Shield,
        7  => Skill::Heal,
        8  => Skill::Plate,
        9  => Skill::Dash,
        10 => Skill::Blast,
        11 => Skill::Shove,
        12 => Skill::Swap,
        13 => Skill::Retreat,
        14 => Skill::Focus,
        15 => Skill::Charge,
        _  => return None,
    })
}

/// Money cost to activate the skill. Stack-M Skill Reference column "Cost".
#[inline]
pub fn skill_cost(s: Skill) -> u8 {
    match s {
        Skill::Lance   => 2,
        Skill::Hook    => 3,
        Skill::Break   => 2,
        Skill::Steal   => 4,
        Skill::Tempest => 4,
        Skill::Shield  => 2,
        Skill::Heal    => 3,
        Skill::Plate   => 3,
        Skill::Dash    => 3,
        Skill::Blast   => 2,
        Skill::Shove   => 3,
        Skill::Swap    => 4,
        Skill::Retreat => 4,
        Skill::Focus   => 2, // Stack N (staged S45): 1→2. See stacks.body id='stack-n'.
        Skill::Charge  => 3,
    }
}

/// Default Range in tiles. Stack-M default Range = 2. Skills override:
///   - Range 0: Self-targeting (Shield, Focus, Charge).
///   - Range 1: Adjacent (Lance - "Range-1", Heal, Plate).
///   - Range 2: Default (Hook, Break, Steal, Tempest, Blast, Swap, Dash).
///   - Range 3: Default + 1 baked in (Shove, Retreat).
///
/// Note: this is the *unbuffed* range. Focus (+1) is applied in the generator
/// when it lands (Slice 6 wires that).
#[inline]
pub fn skill_default_range(s: Skill) -> u8 {
    match s {
        Skill::Lance   => 1,
        Skill::Hook    => 2,
        Skill::Break   => 2,
        Skill::Steal   => 2,
        Skill::Tempest => 2,
        Skill::Shield  => 0,
        Skill::Heal    => 1,
        Skill::Plate   => 1,
        Skill::Dash    => 2,
        Skill::Blast   => 2,
        Skill::Shove   => 3, // default + 1
        Skill::Swap    => 2,
        Skill::Retreat => 3, // default + 1
        Skill::Focus   => 0,
        Skill::Charge  => 0,
    }
}

#[inline]
pub fn skill_category(s: Skill) -> SkillCategory {
    match s {
        Skill::Lance | Skill::Hook | Skill::Break | Skill::Steal | Skill::Tempest
            => SkillCategory::Strike,
        Skill::Shield | Skill::Heal | Skill::Plate
            => SkillCategory::Shield,
        Skill::Dash | Skill::Blast | Skill::Shove | Skill::Swap | Skill::Retreat
            => SkillCategory::Move,
        Skill::Focus | Skill::Charge
            => SkillCategory::Mystic,
    }
}

/// Target-owner contract - see `TargetOwner`. Used by the generator to filter
/// emitted skill-target squares to those the skill could actually accept.
#[inline]
pub fn skill_target_owner(s: Skill) -> TargetOwner {
    match s {
        Skill::Lance   => TargetOwner::Enemy,
        Skill::Hook    => TargetOwner::Enemy,
        Skill::Break   => TargetOwner::Enemy,
        Skill::Steal   => TargetOwner::Enemy,
        Skill::Tempest => TargetOwner::Enemy,
        Skill::Blast   => TargetOwner::Enemy,
        Skill::Heal    => TargetOwner::Ally,
        Skill::Plate   => TargetOwner::Ally,
        Skill::Swap    => TargetOwner::Ally,
        Skill::Shove   => TargetOwner::Enemy,
        Skill::Dash    => TargetOwner::Empty,
        Skill::Retreat => TargetOwner::Empty,
        Skill::Shield  => TargetOwner::SelfOnly,
        Skill::Focus   => TargetOwner::SelfOnly,
        Skill::Charge  => TargetOwner::SelfOnly,
    }
}

// === Draft loadouts (L8) ====================================================
//
// A `SideLoadout` is one side's skill assignment: 6 entries of (skill1, skill2)
// covering the side's 6 skill-bearing pieces (1 King + 5 Champions). 0 is the
// "empty slot" sentinel (matches the mailbox encoding); a fully-equipped side
// has no zeros. Index 0 is the King; indices 1..6 are Champions ordered by
// starting square ascending - matches the iteration order of `setup_stack_m`'s
// `place_back_row` (files b→g, with the King's file replaced by the King). The
// `setup_stack_m_with_loadouts` constructor walks the same iteration so callers
// don't need to materialise square indices themselves.

pub type SideLoadout = [(u8, u8); 6];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftError {
    /// Both skill slots on the same piece carry the same non-zero skill id.
    DuplicateOnPiece { piece_index: u8, skill_id: u8 },
    /// A skill id is out of range (must be 0..=15).
    BadSkillId { piece_index: u8, slot: u8, skill_id: u8 },
}

/// Validate a single side's loadout. Allows 0 ("empty") in either slot - useful
/// for partial states during `Phase::Draft`. Rejects same-skill-on-same-piece
/// duplicates (e.g. one Champion with Lance in both slots) and out-of-range ids.
pub fn validate_loadout(l: &SideLoadout) -> Result<(), DraftError> {
    for (i, &(s1, s2)) in l.iter().enumerate() {
        if s1 > 15 { return Err(DraftError::BadSkillId { piece_index: i as u8, slot: 1, skill_id: s1 }); }
        if s2 > 15 { return Err(DraftError::BadSkillId { piece_index: i as u8, slot: 2, skill_id: s2 }); }
        if s1 != 0 && s1 == s2 {
            return Err(DraftError::DuplicateOnPiece { piece_index: i as u8, skill_id: s1 });
        }
    }
    Ok(())
}

/// Convert a `SideLoadout` authored in **P1's** frame into the equivalent
/// **P2** loadout that produces a 180°-rotated board.
///
/// A `SideLoadout` is expressed in its own side's ascending-square frame
/// (index 0 = King, 1..6 = Champions b→g with the King's file skipped). The
/// two back rows are NOT file-aligned: P1's King sits on d1, P2's on e8, so a
/// point-symmetric board (`sq' = 63 - sq`) maps P1's Champion squares
/// [1,2,4,5,6] onto P2's squares [62,61,59,58,57] - which, read back in P2's
/// own ascending frame [57,58,59,61,62], is the Champion order REVERSED.
/// The King (index 0) maps d1→e8 and stays at index 0.
///
/// Use this at the preset boundary: when the same loadout is chosen for both
/// sides, pass `mirror_loadout(&preset)` as the P2 argument to
/// `setup_stack_m_with_loadouts` so P1's file-b Champion and P2's file-g
/// Champion share skills (matching the visually rotated board).
pub fn mirror_loadout(l: &SideLoadout) -> SideLoadout {
    // King unchanged; Champion indices 1..=5 reverse (1↔5, 2↔4, 3 fixed).
    [l[0], l[5], l[4], l[3], l[2], l[1]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_loadout_reverses_champions_keeps_king() {
        // King (index 0) fixed; Champions 1..5 reversed (1↔5, 2↔4, 3 fixed).
        let l: SideLoadout = [(1,1), (2,2), (3,3), (4,4), (5,5), (6,6)];
        let m = mirror_loadout(&l);
        assert_eq!(m, [(1,1), (6,6), (5,5), (4,4), (3,3), (2,2)]);
        // Mirror is an involution: applying it twice returns the original.
        assert_eq!(mirror_loadout(&m), l);
    }

    #[test]
    fn skill_from_id_roundtrip() {
        assert_eq!(skill_from_id(0), None, "0 is the unequipped sentinel");
        let expected = [
            Skill::Lance, Skill::Hook, Skill::Break, Skill::Steal, Skill::Tempest,
            Skill::Shield, Skill::Heal, Skill::Plate,
            Skill::Dash, Skill::Blast, Skill::Shove, Skill::Swap, Skill::Retreat,
            Skill::Focus, Skill::Charge,
        ];
        for (i, &s) in expected.iter().enumerate() {
            let id = (i as u8) + 1;
            assert_eq!(skill_from_id(id), Some(s), "id {} → {:?}", id, s);
            assert_eq!(s as u8, id, "Skill {:?} should encode to id {}", s, id);
        }
        // Out-of-range ids → None.
        assert_eq!(skill_from_id(16), None);
    }

    #[test]
    fn skill_cost_matches_stack_m() {
        // Source: Stack-M Skill Reference table, "Cost" column.
        assert_eq!(skill_cost(Skill::Lance),   2);
        assert_eq!(skill_cost(Skill::Hook),    3);
        assert_eq!(skill_cost(Skill::Break),   2);
        assert_eq!(skill_cost(Skill::Steal),   4);
        assert_eq!(skill_cost(Skill::Tempest), 4);
        assert_eq!(skill_cost(Skill::Shield),  2);
        assert_eq!(skill_cost(Skill::Heal),    3);
        assert_eq!(skill_cost(Skill::Plate),   3);
        assert_eq!(skill_cost(Skill::Dash),    3);
        assert_eq!(skill_cost(Skill::Blast),   2);
        assert_eq!(skill_cost(Skill::Shove),   3);
        assert_eq!(skill_cost(Skill::Swap),    4);
        assert_eq!(skill_cost(Skill::Retreat), 4);
        assert_eq!(skill_cost(Skill::Focus),   2); // Stack N (staged S45): 1→2.
        assert_eq!(skill_cost(Skill::Charge),  3);
    }

    #[test]
    fn skill_default_range_matches_stack_m() {
        // Self (range 0).
        for s in [Skill::Shield, Skill::Focus, Skill::Charge] {
            assert_eq!(skill_default_range(s), 0, "{:?} is self-targeting", s);
        }
        // Adjacent (range 1).
        for s in [Skill::Lance, Skill::Heal, Skill::Plate] {
            assert_eq!(skill_default_range(s), 1, "{:?} is adjacent", s);
        }
        // Default range 2.
        for s in [Skill::Hook, Skill::Break, Skill::Steal, Skill::Tempest,
                  Skill::Blast, Skill::Swap, Skill::Dash] {
            assert_eq!(skill_default_range(s), 2, "{:?} is default range 2", s);
        }
        // Default + 1 baked in.
        for s in [Skill::Shove, Skill::Retreat] {
            assert_eq!(skill_default_range(s), 3, "{:?} is range 3", s);
        }
    }

    #[test]
    fn skill_target_owner_matches_stack_m() {
        // Strike + Blast + Shove → Enemy.
        for s in [Skill::Lance, Skill::Hook, Skill::Break, Skill::Steal,
                  Skill::Tempest, Skill::Blast, Skill::Shove] {
            assert_eq!(skill_target_owner(s), TargetOwner::Enemy);
        }
        // Heal/Plate/Swap → Ally.
        for s in [Skill::Heal, Skill::Plate, Skill::Swap] {
            assert_eq!(skill_target_owner(s), TargetOwner::Ally);
        }
        // Dash/Retreat → Empty (Move-skills).
        assert_eq!(skill_target_owner(Skill::Dash),    TargetOwner::Empty);
        assert_eq!(skill_target_owner(Skill::Retreat), TargetOwner::Empty);
        // Shield/Focus/Charge → SelfOnly.
        for s in [Skill::Shield, Skill::Focus, Skill::Charge] {
            assert_eq!(skill_target_owner(s), TargetOwner::SelfOnly);
        }
    }

    #[test]
    fn skill_category_partitions_all_15() {
        for id in 1u8..=15 {
            let s = skill_from_id(id).unwrap();
            // Just exercise the match; we mostly care it doesn't panic.
            let _ = skill_category(s);
        }
    }
}
