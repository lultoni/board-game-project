//! Draft-phase helpers — L8 Phase C.
//!
//! Phase B added the `DraftTurn` action and the `legal_draft_turns` /
//! `apply_draft_turn` mechanics. This module sits one layer up:
//!
//! 1. **`DraftState`** — a compact, UI-facing snapshot of where the draft is.
//!    `which side picks next`, `how many turns have been committed`, and a
//!    `used_slots[ply][slot]` bitmap so the frontend can grey out targets
//!    that are already filled.
//!
//! 2. **`next_preset_draft_turn`** — given a target `SideLoadout` for the
//!    side-to-move, return the next legal `DraftTurn` that drives the
//!    mailbox toward that loadout. This is the AI's draft strategy in L8:
//!    no search, no heuristic — just unroll a fixed preset two picks per
//!    ply. See `oq-83` for the real-AI-draft follow-up slice.
//!
//! 3. **`DEFAULT_AI_LOADOUT`** — placeholder constant used by `Match::step_ai`
//!    when the AI side enters Phase::Draft. The designer will replace this
//!    with the curated "First/Second/Third game" presets in a later slice
//!    (see task #32 / oq-65).

use crate::game_logic::action::Action;
use crate::game_logic::skills::SideLoadout;
use crate::state::position::{Phase, Player, Position};

// === DraftState (UI-facing) =================================================

/// A snapshot of where the draft is. Generated on demand from `Position` —
/// no extra state is stored on the Position itself. Used by the Tauri /
/// wasm wrappers to feed the frontend's draft UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraftState {
    /// Number of `DraftTurn` plies committed so far (0..=12).
    pub turn_no: u8,
    /// Whose turn it is to pick. Undefined once `turn_no == 12` — the phase
    /// has already transitioned to Move and `pos.to_move` reflects the next
    /// play-phase actor (P1, per Stack M setup).
    pub side_to_move: Player,
    /// `used_slots[piece_idx][slot]` — `true` iff that mailbox slot already
    /// holds a non-zero skill. Indexed as (piece_index, slot):
    ///   - piece_index 0..6  → P1's 6 skill-bearers (King at 0, Champions
    ///     1..5 by ascending starting square — same order as `SideLoadout`).
    ///   - piece_index 6..12 → P2's 6 skill-bearers (same internal order).
    ///   - slot 0..2          → mailbox slot1, slot2.
    pub used_slots: [[bool; 2]; 12],
}

/// Build a `DraftState` snapshot for the given position. Cheap enough to
/// call from the Tauri wrapper on every UI refresh (12 mailbox reads + a
/// pair of bitboard pops). Returns a sentinel `turn_no=12` state when the
/// position is no longer in Phase::Draft, so the UI can detect "draft
/// finished" without checking the phase separately.
pub fn draft_state(pos: &Position) -> DraftState {
    // Walk skill-bearers in canonical order (king, then champions by sq asc),
    // per side. This matches `SideLoadout` indexing and `apply_back_row_loadout`.
    let mut used = [[false; 2]; 12];
    let mut filled = 0u8;

    let p1_bearers = (pos.kings | pos.champions) & pos.p1_pieces;
    let p2_bearers = (pos.kings | pos.champions) & pos.p2_pieces;

    let write = |slot_idx: usize, sqs: u64, _is_p1: bool, used: &mut [[bool; 2]; 12], filled: &mut u8| {
        // King first, then champions ascending. Bitboard iteration is
        // ascending by `trailing_zeros`, so we need to extract the King
        // separately to put it at index 0.
        let kings_bits = pos.kings.0 & sqs;
        let champs_bits = pos.champions.0 & sqs;
        let mut idx = slot_idx;
        if kings_bits != 0 {
            let sq = kings_bits.trailing_zeros() as u8;
            let e = pos.mailbox[sq as usize];
            used[idx][0] = e.skill1() != 0;
            used[idx][1] = e.skill2() != 0;
            if e.skill1() != 0 { *filled += 1; }
            if e.skill2() != 0 { *filled += 1; }
            idx += 1;
        }
        let mut bits = champs_bits;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let e = pos.mailbox[sq as usize];
            used[idx][0] = e.skill1() != 0;
            used[idx][1] = e.skill2() != 0;
            if e.skill1() != 0 { *filled += 1; }
            if e.skill2() != 0 { *filled += 1; }
            idx += 1;
        }
    };
    write(0, p1_bearers.0, true,  &mut used, &mut filled);
    write(6, p2_bearers.0, false, &mut used, &mut filled);

    // turn_no = #DraftTurns committed = filled_slots / 2 (2 picks per turn).
    // Cap at 12 because once the phase is Move we want a stable sentinel.
    let turn_no = (filled / 2).min(12);

    let side_to_move = if pos.current_phase == Phase::Draft {
        pos.to_move
    } else {
        // Phase already moved on; expose the play-phase to_move so the UI
        // can branch cleanly. `turn_no` will read 12 in this case.
        pos.to_move
    };

    DraftState { turn_no, side_to_move, used_slots: used }
}

// === Preset-driven AI draft =================================================

/// Placeholder default loadout the AI plays during Phase::Draft until a real
/// draft heuristic / search lands (oq-83). Same loadout for both sides; the
/// designer will replace this with curated "First / Second / Third game"
/// loadouts in a later slice (task #32, oq-65).
///
/// Layout (matches `SideLoadout` indexing — King first, then 5 Champions in
/// ascending starting-square order):
///
///   piece 0 (King)      : Shield (6),  Heal   (7)   — defensive King
///   piece 1 (Champion)  : Lance  (1),  Dash   (9)   — frontline striker
///   piece 2 (Champion)  : Hook   (2),  Plate  (8)   — pulling tank
///   piece 3 (Champion)  : Break  (3),  Focus  (14)  — armour-cracker w/ buff
///   piece 4 (Champion)  : Tempest(5),  Charge (15)  — AOE swing
///   piece 5 (Champion)  : Swap   (12), Shove  (11)  — repositioning
///
/// No same-skill-on-same-piece duplicates. Hits every skill category
/// (Strike, Shield, Move, Mystic) so post-draft play exercises the full
/// engine. Not balanced — just non-broken.
pub const DEFAULT_AI_LOADOUT: SideLoadout = [
    (6,  7),    // King: Shield + Heal
    (1,  9),    // Champ 1: Lance + Dash
    (2,  8),    // Champ 2: Hook + Plate
    (3,  14),   // Champ 3: Break + Focus
    (5,  15),   // Champ 4: Tempest + Charge
    (12, 11),   // Champ 5: Swap + Shove
];

/// Alternate loadout used by P2 in AIvAI matches so both AIs don't draft the
/// same army. Different picks across every piece — same coverage guarantee
/// (all four skill categories present) but a distinct playstyle: heavier on
/// direct strikes and reach, lighter on tanking.
pub const DEFAULT_AI_LOADOUT_P2: SideLoadout = [
    (7,  8),    // King: Heal + Plate
    (5,  13),   // Champ 1: Tempest + Retreat
    (1,  15),   // Champ 2: Lance + Charge
    (2,  9),    // Champ 3: Hook + Dash
    (3,  14),   // Champ 4: Break + Focus
    (4,  11),   // Champ 5: Steal + Shove
];

/// Given a target loadout for the side-to-move, return the next
/// `DraftTurn` action that fills two as-yet-unfilled slots on that side
/// with the preset's prescribed skills.
///
/// Returns `None` if:
///   - the position is not in `Phase::Draft`, or
///   - the side-to-move has fewer than 2 unfilled slots remaining (which
///     can only happen during a transitional or corrupt state — the normal
///     flow guarantees 2 unfilled slots per ply until the 6th).
///
/// Strategy: walk the side-to-move's skill-bearers in `SideLoadout` index
/// order; for each piece compare current mailbox slots to the preset; emit
/// picks for any slot whose preset entry is non-zero and whose current
/// mailbox slot is empty. Return the first two such picks bundled as one
/// `DraftTurn`. Deterministic — given the same `(pos, preset)` the same
/// action comes back.
pub fn next_preset_draft_turn(pos: &Position, preset: &SideLoadout) -> Option<Action> {
    if pos.current_phase != Phase::Draft { return None; }

    // Walk stm bearers in canonical order (King, then Champions ascending
    // by sq). Same as `apply_back_row_loadout` ordering.
    let stm_pieces = match pos.to_move {
        Player::P1 => pos.p1_pieces,
        Player::P2 => pos.p2_pieces,
    };
    let kings  = (pos.kings & stm_pieces).0;
    let champs = (pos.champions & stm_pieces).0;

    let mut needed: [(u8 /*skill*/, u8 /*sq*/, u8 /*slot*/); 2] = [(0, 0, 0); 2];
    let mut n = 0usize;

    let consider = |sq: u8, preset_idx: usize, needed: &mut [(u8, u8, u8); 2], n: &mut usize| {
        if *n >= 2 { return; }
        let e = pos.mailbox[sq as usize];
        let (ps1, ps2) = preset[preset_idx];
        // Slot 1 first, then slot 2 — order doesn't matter to the engine
        // (resolver doesn't care which slot a skill is in) but we pick a
        // stable ordering for determinism.
        if e.skill1() == 0 && ps1 != 0 {
            // Avoid duplicating the preset's other slot if it's already on
            // this piece. (Shouldn't happen with a well-formed preset.)
            if ps1 != e.skill2() {
                needed[*n] = (ps1, sq, 0);
                *n += 1;
                if *n >= 2 { return; }
            }
        }
        if e.skill2() == 0 && ps2 != 0 {
            if ps2 != e.skill1() && !(*n == 1 && needed[0].0 == ps2 && needed[0].1 == sq) {
                needed[*n] = (ps2, sq, 1);
                *n += 1;
            }
        }
    };

    // King first.
    if kings != 0 {
        let king_sq = kings.trailing_zeros() as u8;
        consider(king_sq, 0, &mut needed, &mut n);
    }
    // Then Champions ascending.
    let mut bits = champs;
    let mut champ_idx = 1usize;
    while bits != 0 && n < 2 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        consider(sq, champ_idx, &mut needed, &mut n);
        champ_idx += 1;
    }

    if n < 2 { return None; }

    let (s1, q1, l1) = needed[0];
    let (s2, q2, l2) = needed[1];
    Some(Action::encode_draft_turn(s1, q1, l1, s2, q2, l2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::make_unmake::make;
    use crate::game_logic::skills::validate_loadout;

    #[test]
    fn default_ai_loadout_validates() {
        validate_loadout(&DEFAULT_AI_LOADOUT).expect("DEFAULT_AI_LOADOUT must be a valid loadout");
    }

    #[test]
    fn draft_state_initial_is_all_empty() {
        let pos = Position::setup_stack_m_for_draft();
        let s = draft_state(&pos);
        assert_eq!(s.turn_no, 0, "fresh draft has no turns committed");
        assert_eq!(s.side_to_move, Player::P1);
        for piece in 0..12 {
            for slot in 0..2 {
                assert!(!s.used_slots[piece][slot],
                    "piece {} slot {} should be empty at draft start", piece, slot);
            }
        }
    }

    #[test]
    fn draft_state_advances_after_each_turn() {
        let mut pos = Position::setup_stack_m_for_draft();
        let pick = next_preset_draft_turn(&pos, &DEFAULT_AI_LOADOUT)
            .expect("preset must yield a turn at start of draft");
        let _ = make(&mut pos, pick);
        let s = draft_state(&pos);
        assert_eq!(s.turn_no, 1, "one turn committed");
        // Exactly 2 slots flipped to true.
        let used_count: u32 = s.used_slots.iter()
            .flat_map(|p| p.iter())
            .map(|&b| b as u32)
            .sum();
        assert_eq!(used_count, 2, "exactly one DraftTurn = 2 filled slots");
    }

    #[test]
    fn preset_drives_draft_to_completion() {
        let mut pos = Position::setup_stack_m_for_draft();
        let mut plies = 0;
        while pos.current_phase == Phase::Draft {
            let pick = next_preset_draft_turn(&pos, &DEFAULT_AI_LOADOUT)
                .expect("preset must yield a turn whenever Phase::Draft");
            let _ = make(&mut pos, pick);
            plies += 1;
            assert!(plies <= 12, "preset must drain in ≤12 plies");
        }
        assert_eq!(plies, 12, "exactly 12 DraftTurns to complete");
        assert_eq!(pos.current_phase, Phase::Move);

        // Both sides should mirror the DEFAULT_AI_LOADOUT.
        // King first (asc sq), then Champions.
        for player_pieces in [pos.p1_pieces, pos.p2_pieces] {
            let bearers = (pos.kings | pos.champions) & player_pieces;
            let king_sq = (pos.kings.0 & bearers.0).trailing_zeros() as u8;
            let king_e = pos.mailbox[king_sq as usize];
            // Order-independent comparison — engine doesn't care which slot.
            let king_skills = sorted2(king_e.skill1(), king_e.skill2());
            let expected_king = sorted2(DEFAULT_AI_LOADOUT[0].0, DEFAULT_AI_LOADOUT[0].1);
            assert_eq!(king_skills, expected_king,
                "King at sq {} got skills {:?}, expected {:?}", king_sq, king_skills, expected_king);

            let mut champ_bits = (pos.champions.0) & bearers.0;
            let mut idx = 1usize;
            while champ_bits != 0 {
                let sq = champ_bits.trailing_zeros() as u8;
                champ_bits &= champ_bits - 1;
                let e = pos.mailbox[sq as usize];
                let got = sorted2(e.skill1(), e.skill2());
                let want = sorted2(DEFAULT_AI_LOADOUT[idx].0, DEFAULT_AI_LOADOUT[idx].1);
                assert_eq!(got, want,
                    "Champion #{} at sq {} got {:?}, expected {:?}", idx, sq, got, want);
                idx += 1;
            }
        }
    }

    fn sorted2(a: u8, b: u8) -> (u8, u8) {
        if a <= b { (a, b) } else { (b, a) }
    }

    #[test]
    fn next_preset_returns_none_outside_draft_phase() {
        let pos = Position::setup_stack_m(); // Move phase
        assert!(next_preset_draft_turn(&pos, &DEFAULT_AI_LOADOUT).is_none(),
            "outside Phase::Draft the preset has nothing to do");
    }

    #[test]
    fn draft_state_after_completion_reads_turn_no_twelve() {
        let mut pos = Position::setup_stack_m_for_draft();
        while pos.current_phase == Phase::Draft {
            let pick = next_preset_draft_turn(&pos, &DEFAULT_AI_LOADOUT).unwrap();
            let _ = make(&mut pos, pick);
        }
        let s = draft_state(&pos);
        assert_eq!(s.turn_no, 12, "after completion turn_no should be 12 (sentinel)");
    }
}
