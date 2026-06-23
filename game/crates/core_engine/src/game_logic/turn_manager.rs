//! End-of-phase and end-of-turn bookkeeping. Resets combo counters and the
//! turn-scoped state added in the audit pass: pending_modifiers, combo-
//! credit tracking, tracked-enemies list, moved_this_phase.

use crate::state::Position;

pub fn end_phase(_pos: &mut Position) {
    // TODO:
    // - Move → Skill transition:
    //     * clear `moved_this_phase` (Move-Phase-only bitmap).
    //     * set actions_remaining to `skill_phase_budget(round_number)` —
    //       the paper-baseline progression curve adopted into Stack M
    //       (oq-69 resolved): +1 action per 10 rounds, unbounded.
    // - Skill → end of turn: delegate to `end_turn`.
}

pub fn end_turn(pos: &mut Position) {
    // TODO:
    //   1. Clear combo counters on every square owned by the side that's
    //      about to move (mailbox.combo = 0 for all squares in their
    //      occupancy bitboard).
    //   2. Clear pending_modifiers (Focus / Charge are turn-scoped).
    //   3. Clear champion_credit + tracked_enemies + tracked_casters (next
    //      turn's tracking starts fresh).
    //   4. Flip to_move; if the new side-to-move is P1, increment
    //      `round_number` (a Round = P1 turn + P2 turn).
    //   5. Disburse start-of-turn income to the *new* side-to-move:
    //         income = income_per_turn(round_number)
    //      i.e. `2 + round_number / 5` — unbounded, +1 every 5 rounds.
    //      (income value is round-based, but paid every turn; both players
    //      receive it on their own turn-start. The +1 step kicks in at
    //      rounds 5, 10, 15, …)
    //   6. Reset current_phase = Move; reset actions_remaining = 2.
    //   7. Clear moved_this_phase (defence-in-depth — Move→Skill already did).
    //   8. XOR Zobrist for all of the above changes.
    let _ = pos; // silence unused-var until implementation lands
}

/// Per-turn income for the given round (Stack M).
///
/// Formula: `2 + round_number / 5`. Unbounded — +1 per 5 rounds, no cap.
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
