//! Layer 1 - Core State Representation.
//!
//! Spatial state as bitboards (5x u64), entity state as a `[u16; 64]` mailbox,
//! global resources as scalars, plus an incremental Zobrist hash.
//!
//! Layout per ADR-005:
//!   mailbox u16 = [hp:2][armor:2][combo:3][skill1:4][skill2:4]  (15 bits used)

pub mod bitboard;
pub mod action_notation;
pub mod fen;
pub mod magic;
pub mod mailbox;
pub mod path;
pub mod position;
pub mod zobrist;

pub use action_notation::NotationError;
pub use bitboard::Bitboard;
pub use fen::FenError;
pub use mailbox::{MailboxEntry, EMPTY_MAILBOX_ENTRY};
pub use position::{Position, Phase, Player, GameResult};
