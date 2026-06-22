//! Legal primitive-action generator. Reads the Position, emits the set of
//! legal Actions for the current player and phase.
//!
//! Per the audit decision: path-implicit destinations (Retreat) are
//! pre-resolved here — the generator emits one Action per legal landing
//! square. Direction-only skills (Shove) emit one Action per legal
//! direction using `Action::choice_idx`. AOE skills emit a single Action;
//! the AOE expansion happens inside `make()`.

use super::action::Action;
use crate::state::Position;

pub fn generate(_pos: &Position) -> Vec<Action> {
    // TODO: emit all legal Move / Skill / EndPhase / EndTurn actions
    // for `pos.to_move` in `pos.current_phase`, respecting actions_remaining,
    // Path/Range/Block rules, Bodyguard redirect availability, and the
    // current pending_modifiers (Focus extends Range; Charge is consumed
    // by the next Strike).
    Vec::new()
}
