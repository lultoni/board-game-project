//! End-of-phase and end-of-turn bookkeeping. Resets combo counters and the
//! turn-scoped state added in the audit pass: pending_modifiers, combo-
//! credit tracking, tracked-enemies list.

use crate::state::Position;

pub fn end_phase(_pos: &mut Position) {
    // TODO: Move → Skill transition (reset actions_remaining to skill-phase
    // budget per Progression curve), or Skill → end of turn (delegate to
    // end_turn).
}

pub fn end_turn(_pos: &mut Position) {
    // TODO:
    //   1. Clear combo counters on every square owned by the side that's
    //      about to move (mailbox.combo = 0 for all squares in their
    //      occupancy bitboard).
    //   2. Clear pending_modifiers (Focus / Charge are turn-scoped).
    //   3. Clear champion_credit + tracked_enemies (next turn's tracking
    //      starts fresh).
    //   4. Distribute income (+2/turn baseline, +1 every 5 rounds).
    //   5. Flip to_move; reset actions_remaining for the new turn's Move
    //      Phase (2 actions per Stack M).
    //   6. XOR Zobrist for all of the above changes.
}
