//! Layer 4 — Session & Match Manager.
//!
//! Holds match-level state: mode (HvH/HvAI/AIvAI), action history, current
//! Position, plus serialisation in and out. UI calls in here; this layer
//! talks to Layer 2 for legality + Make/Unmake.

use crate::game_logic::action::Action;
use crate::state::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { HumanVsHuman, HumanVsAi, AiVsAi }

pub struct Match {
    pub position: Position,
    pub history:  Vec<Action>,
    pub mode:     Mode,
}

impl Match {
    pub fn new(mode: Mode) -> Self {
        Match {
            position: Position::empty(),  // TODO: Position::setup_stack_m()
            history:  Vec::new(),
            mode,
        }
    }

    // TODO: try_apply(action) — validate via generator, then make_unmake::make.
    // TODO: serialise() / deserialise() — compact FEN-like string + JSON.
}
