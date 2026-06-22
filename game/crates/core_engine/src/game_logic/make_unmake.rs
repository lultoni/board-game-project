//! Make / Unmake — apply an action to a Position, or perfectly reverse it
//! using a previously-written Undo record.
//!
//! Reversibility is mandatory: tree search must not copy state. `make`
//! writes an `Undo` describing exactly what changed; `unmake` consumes it
//! to restore the prior state. The Action itself stays immutable.

use super::action::{Action, Undo};
use crate::state::Position;

pub fn make(_pos: &mut Position, _action: Action) -> Undo {
    // TODO:
    //   1. Snapshot pending_modifiers, phase, actions_remaining, money,
    //      champion_credit, tracked_enemies into the Undo.
    //   2. Decode action.kind / skill_id.
    //   3. Dispatch to the skill resolver. The resolver:
    //      - computes affected squares (incl. AOE expansion for Tempest)
    //      - records each mutated square's prior mailbox entry in the Undo
    //      - updates bitboards (XOR-ing deltas into Undo too)
    //      - updates money, combo counters, modifier bits
    //      - XOR-updates the Zobrist hash (delta into Undo)
    //   4. Return the populated Undo.
    Undo::default()
}

pub fn unmake(_pos: &mut Position, _undo: &Undo) {
    // TODO:
    //   1. Restore mailbox entries from undo.affected_squares / _prev_entries.
    //   2. XOR bitboards with undo.*_xor to revert.
    //   3. Restore pending_modifiers / phase / actions_remaining.
    //   4. Apply money deltas with sign inverted.
    //   5. Restore champion_credit + tracked_enemies snapshot.
    //   6. XOR zobrist with undo.zobrist_xor to revert.
}
