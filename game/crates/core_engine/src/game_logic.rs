//! Layer 2 — Game Logic & Action Pipeline.
//!
//! - `action`        — packed Primitive Action representation
//! - `magic`         — precomputed straight-line move tables
//! - `generator`     — enumerate legal primitive actions for a Position
//! - `make_unmake`   — apply / reverse an action against a Position
//! - `turn_manager`  — phase transitions and end-of-turn bookkeeping

pub mod action;
pub mod magic;
pub mod generator;
pub mod make_unmake;
pub mod turn_manager;
