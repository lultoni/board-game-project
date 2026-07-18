//! End-of-turn bookkeeping. Resets combo counters and the turn-scoped state
//! added in the audit pass: pending_modifiers, combo-credit tracking,
//! tracked-enemies list, moved_this_phase.
//!
//! Move → Skill phase transitions are handled inline in
//! `make_unmake::apply_end_phase`, which already calls
//! `make_unmake::skill_phase_budget(round_number)` to set the new action
//! budget (oq-69 resolved, session-31). This module only handles the
//! Skill → next-turn transition.

use crate::state::position::{Phase, Player, Position};
use super::action::Undo;

/// Skill → next-turn transition. Called from `apply_end_phase` when the
/// current phase is Skill. Bookkeeping done here (Slice 6):
///
/// 1. Clear combo counters on EVERY piece on the board. Per Stack M rule:
///    "combo counter resets at the end of your turn." Originally implemented
///    as "clear only on the new STM's pieces" (the typical case - combo lives
///    on enemy pieces from the caster's POV). But self-buff skills like
///    Tempest place combo on the caster's OWN pieces, and those would survive
///    the turn flip if we only cleared the new STM side, letting the opponent
///    capitalise on the stale buildup. Clear all sides defensively.
/// 2. Clear pending_modifiers (Focus / Charge are turn-scoped).
/// 3. Clear champion_credit + tracked_enemies + tracked_casters (next turn's
///    tracking starts fresh).
/// 4. Flip to_move; if the new side-to-move is P1, increment `round_number`.
/// 5. Disburse start-of-turn income to the *new* side-to-move:
///    `income = income_per_turn(round_number)`.
/// 6. Reset current_phase = Move, actions_remaining = 2 (Move-Phase always
///    has 2 actions per Stack M).
/// 7. Clear moved_this_phase (defence-in-depth - Move→Skill already did).
///
/// All previous values are captured in `Undo` so the transition is perfectly
/// reversible. Zobrist hashing is wired in a later slice.
pub fn end_turn(pos: &mut Position, undo: &mut Undo) {
    use super::make_unmake::{
        clear_pending, flip_to_move, moved_clear_all, set_actions, set_p1_money,
        set_p2_money, set_phase, set_round, write_mailbox,
    };

    // 1. Clear combo counters on EVERY piece (both sides). Stack M says combo
    //    resets at the end of your turn - and self-buff skills (e.g. Tempest)
    //    can place combo on the caster's own pieces, so a one-sided clear
    //    leaves stale combo on the just-acting side's pieces.
    let mut bits = (pos.p1_pieces.0) | (pos.p2_pieces.0);
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let prev = pos.mailbox[sq as usize];
        if prev.combo() != 0 {
            write_mailbox(pos, undo, sq, prev.with_combo(0));
        }
    }

    // The new side-to-move (after the flip). Needed below for income disburse.
    let new_stm = match pos.to_move {
        Player::P1 => Player::P2,
        Player::P2 => Player::P1,
    };

    // 2. Turn-scoped state - pending_modifiers is hashed (clear via helper).
    //    champion_credit / tracked_*_len are NOT hashed (transient) so they
    //    can be cleared by direct write; snapshots already captured in make().
    clear_pending(pos, undo, 0xFF);
    pos.champion_credit = 0;
    pos.tracked_enemies_len = 0;
    pos.tracked_casters_len = 0;

    // 3. Flip to_move; bump round on flip-back to P1.
    flip_to_move(pos, undo);
    if matches!(new_stm, Player::P1) {
        set_round(pos, undo, pos.round_number.saturating_add(1));
    }

    // 4. Disburse start-of-turn income to the new side-to-move.
    //    Stack M rule: Round 1 has NO income for either player - both sides
    //    play the opening round on their starting money. Income begins in
    //    Round 2 (each player's turn-start), and follows `income_per_turn`
    //    thereafter. `pos.round_number` here already reflects the round the
    //    new side-to-move is entering (the bump on P2→P1 happens above).
    if pos.round_number >= 2 {
        let income = income_per_turn(pos.round_number);
        match new_stm {
            Player::P1 => set_p1_money(pos, undo, pos.p1_money.saturating_add(income)),
            Player::P2 => set_p2_money(pos, undo, pos.p2_money.saturating_add(income)),
        }
    }

    // 5. Reset phase, actions, defensive moved_this_phase clear.
    set_phase(pos, undo, Phase::Move);
    set_actions(pos, undo, 2);
    moved_clear_all(pos, undo);
}

/// Per-turn income for the given round (Stack M).
///
/// Formula: `2 + round_number / 5`. Unbounded - +1 per 5 rounds, no cap.
/// R1–4: 2, R5–9: 3, R10–14: 4, R15–19: 5, R20–24: 6, … and so on without
/// limit. Saturates at u16::MAX defensively (games will never reach that,
/// but we don't want to panic on overflow).
#[inline]
pub fn income_per_turn(round_number: u16) -> u16 {
    2u16.saturating_add(round_number / 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn income_per_turn_paper_curve() {
        // Stack M: +1 income per 5 rounds, starting at 2, unbounded.
        assert_eq!(income_per_turn(1), 2);
        assert_eq!(income_per_turn(4), 2);
        assert_eq!(income_per_turn(5), 3);
        assert_eq!(income_per_turn(9), 3);
        assert_eq!(income_per_turn(10), 4);
        assert_eq!(income_per_turn(15), 5);
        assert_eq!(income_per_turn(20), 6);
        // Crucially: keeps climbing without cap.
        assert_eq!(income_per_turn(50), 12);
        assert_eq!(income_per_turn(100), 22);
    }
}
