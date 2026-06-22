//! Legal primitive-action generator. Reads the Position, emits the set of
//! legal Actions for the current player and phase.
//!
//! Per the audit decision: path-implicit destinations (Retreat) are
//! pre-resolved here — the generator emits one Action per legal landing
//! square. Direction-only skills (Shove) emit one Action per legal
//! direction using `Action::choice_idx`. AOE skills emit a single Action;
//! the AOE expansion happens inside `make()`.
//!
//! # Move Phase generation (Stack M, slice 1)
//!
//! For each piece of the side-to-move whose square is **not** set in
//! `pos.moved_this_phase`:
//!
//! - **Plain moves**: every empty square reachable within the piece's speed
//!   (Guard=2, Champion=1, King=1) by *any* path of single-tile steps.
//!   Movement is free in all 8 directions per Stack M — speed is Chebyshev
//!   distance (a diagonal step costs 1). A Guard with speed 2 may reach any
//!   empty square in the 5×5 Chebyshev-radius-2 region whose every tile on
//!   *some* connecting path is empty (i.e. the destination need not be
//!   reached cardinally — zigzag is legal).
//!   Implementation hint: a BFS bounded by speed over the 8-neighbour graph,
//!   blocked by occupied squares. The intermediate squares of a path must be
//!   empty too — the piece does not jump.
//!
//! - **Move-Attacks**: every enemy-occupied square reachable as above. The
//!   mover does NOT enter the target tile; the enemy takes 1 damage. For
//!   each such target, enumerate Bodyguard-redirect choices via
//!   `Action::choice_idx`:
//!     - `choice_idx = 0` → no redirect.
//!     - `choice_idx = k` (1..=N) → redirect to the k-th adjacent friendly
//!       Guard of the defender (canonical: ascending square index).
//!   Search uses all variants directly; HvH UI presents the choice to the
//!   defender between generator output and `make()`.
//!
//! - `moved_this_phase` blocks the *origin* square of a piece that has
//!   already moved this Move Phase. After a move, the destination square
//!   carries the bit so a later move attempt from there is rejected.
//!
//! - `EndPhase` is always legal in the Move Phase. Becomes the only legal
//!   action when no piece has a legal move/attack remaining.
//!
//! # Skill Phase generation
//!
//! Skills use Path/Range/Block rules (queen-style straight lines, blocked by
//! any piece). Range buffs from `pending_modifiers` (Focus) apply. Charge
//! affects damage but does not change legality. Implementation defers to
//! slice 4.

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
