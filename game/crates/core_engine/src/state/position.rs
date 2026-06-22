//! Full game position: bitboards + mailbox + global resources + Zobrist hash.
//!
//! # Invariants
//!
//! **Bitboards are authoritative for occupancy.** A square is occupied iff
//! `(p1_pieces | p2_pieces).contains(sq)`. The `mailbox` entry for an
//! unoccupied square is *undefined* — never read it without first checking
//! the bitboards. As a discipline, `make/unmake` clears the mailbox slot to
//! `EMPTY_MAILBOX_ENTRY` on piece removal, but correctness must not depend
//! on that. This matches Stockfish's bitboard+mailbox convention.

use super::{Bitboard, MailboxEntry, EMPTY_MAILBOX_ENTRY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player { P1, P2 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Move, Skill }

/// Stack M has 5 Champions per player. Index 0..5 identifies a specific
/// Champion within a player's army; mapping from index to current square is
/// maintained by the session/match layer (Champions are stable identities
/// across moves; squares change).
pub const CHAMPIONS_PER_PLAYER: usize = 5;

/// Maximum number of distinct enemy targets a player's Champions can have
/// struck for combo purposes within a single turn. 8 is loose upper bound:
/// 2 actions × Skill Phase × multi-strike skills can realistically hit at
/// most a handful of distinct enemies. Sized for u64 indexing convenience
/// (`champion_idx * 8 + tracked_enemy_idx` fits in u64).
pub const MAX_TRACKED_ENEMIES: usize = 8;

/// Bitfield positions in `Position::pending_modifiers`.
pub mod modifier_bits {
    pub const FOCUS:  u8 = 1 << 0;  // next skill: +1 Range
    pub const CHARGE: u8 = 1 << 1;  // next Strike skill: +1 damage
    // bits 2..8 reserved for future turn-scoped modifiers.
}

#[derive(Clone, Debug)]
pub struct Position {
    // === Layer 1: Spatial state (bitboards) — 5× u64. ===
    pub p1_pieces: Bitboard,
    pub p2_pieces: Bitboard,
    pub kings:     Bitboard,
    pub champions: Bitboard,
    pub guards:    Bitboard,

    // === Layer 1: Entity state — packed per-square piece data. ===
    // Reads valid only for squares where (p1_pieces | p2_pieces) contains the bit.
    pub mailbox: [MailboxEntry; 64],

    // === Layer 1: Global resources. ===
    pub p1_money: u16,
    pub p2_money: u16,
    pub current_phase: Phase,
    pub actions_remaining: u8,
    pub to_move: Player,

    /// Turn-scoped modifier bits. See `modifier_bits` module.
    /// Cleared at end of every turn. Each bit has a dedicated Zobrist key.
    pub pending_modifiers: u8,

    // === Layer 1: Per-turn combo-credit tracking. ===
    /// Squares of enemies struck this turn that contributed to a combo counter.
    /// Index into this list is `tracked_enemy_idx` (0..MAX_TRACKED_ENEMIES).
    /// `tracked_enemies_len` is the active count; positions beyond are stale.
    /// Cleared at end of turn.
    pub tracked_enemies: [u8; MAX_TRACKED_ENEMIES],
    pub tracked_enemies_len: u8,

    /// Bitmap: bit `champion_idx * MAX_TRACKED_ENEMIES + tracked_enemy_idx` is
    /// set iff that Champion has already ticked that enemy's combo counter
    /// this turn. Cleared at end of turn.
    pub champion_credit: u64,

    // === Layer 1: Incrementally maintained Zobrist hash. ===
    pub zobrist: u64,
}

impl Position {
    /// Empty board with no pieces. Use `setup_stack_m()` for the canonical start.
    pub fn empty() -> Self {
        Position {
            p1_pieces: Bitboard::EMPTY,
            p2_pieces: Bitboard::EMPTY,
            kings:     Bitboard::EMPTY,
            champions: Bitboard::EMPTY,
            guards:    Bitboard::EMPTY,
            mailbox:   [EMPTY_MAILBOX_ENTRY; 64],
            p1_money: 0,
            p2_money: 0,
            current_phase: Phase::Move,
            actions_remaining: 0,
            to_move: Player::P1,
            pending_modifiers: 0,
            tracked_enemies: [0; MAX_TRACKED_ENEMIES],
            tracked_enemies_len: 0,
            champion_credit: 0,
            zobrist: 0,
        }
    }

    /// True iff the square is currently occupied (per the authoritative bitboards).
    #[inline]
    pub fn is_occupied(&self, sq: u8) -> bool {
        (self.p1_pieces | self.p2_pieces).contains(sq)
    }

    // TODO: setup_stack_m() — Stack M canonical start (8×8 board, piece placement,
    // starting money = 6, fixed back-row + front-row layout per Stack M setup).
    // Source of truth: SELECT body FROM stacks WHERE id='stack-m';
}
