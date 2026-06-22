//! End-of-phase and end-of-turn bookkeeping. Resets combo counters and the
//! turn-scoped state added in the audit pass: pending_modifiers, combo-
//! credit tracking, tracked-enemies list, moved_this_phase.

use crate::state::Position;

pub fn end_phase(_pos: &mut Position) {
    // TODO:
    // - Move → Skill transition:
    //     * clear `moved_this_phase` (Move-Phase-only bitmap).
    //     * set actions_remaining to the Skill-Phase budget for this round.
    //       OQ-pending: exact Skill-Phase action progression curve. Stack M
    //       body only says "starts at 2 per turn, scaling up over the game".
    //       Until the curve is locked, treat it as 2 throughout — slice 5
    //       wires the real curve.
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
    //         income = 2 + (round_number / 5)
    //      (income value is round-based, but paid every turn; both players
    //      receive it on their own turn-start. The +1 step kicks in at
    //      rounds 5, 10, 15, …)
    //   6. Reset current_phase = Move; reset actions_remaining = 2.
    //   7. Clear moved_this_phase (defence-in-depth — Move→Skill already did).
    //   8. XOR Zobrist for all of the above changes.
    let _ = pos; // silence unused-var until implementation lands
}
