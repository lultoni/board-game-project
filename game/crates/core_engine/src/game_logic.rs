//! Layer 2 - Game Logic & Action Pipeline.
//!
//! - `action`        - packed Primitive Action representation
//! - `skills`        - skill enum + cost/range/category/owner lookups
//! - `generator`     - enumerate legal primitive actions for a Position
//! - `make_unmake`   - apply / reverse an action against a Position
//! - `turn_manager`  - phase transitions and end-of-turn bookkeeping
//! - `draft`         - L8 draft-phase helpers (preset AI, UI state)
//!
//! Path/Range/Block primitives live in `state::magic` and `state::path` -
//! they are pure geometry, not game-logic.

pub mod action;
pub mod skills;
pub mod generator;
pub mod make_unmake;
pub mod turn_manager;
pub mod draft;
