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

/// Terminal-state marker. `None` means the game is still in progress; a
/// concrete variant means a King has been removed and the named player has
/// won. Stack M has no draws (`"No draw conditions"`), so this enum has no
/// Draw variant by design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult { P1Wins, P2Wins }

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

    /// Current Round number. A Round = P1 turn + P2 turn. Increments when the
    /// turn flips back to P1 (i.e. at the *start* of P1's next turn). Used
    /// for round-based progression: income scaling is `2 + round_number / 5`,
    /// Skill-Phase action budget grows on the same cadence. Income itself is
    /// disbursed at the start of each Player turn (per Stack M).
    pub round_number: u16,

    /// Bitboard of squares whose piece has already been moved this Move Phase.
    /// Stack M: "Each piece can only be moved once per Move Phase." Cleared
    /// when the Move Phase ends. Skill Phase ignores this entirely.
    /// Note: this tracks *origin* squares as they were *at the moment the
    /// piece was moved*. Because each piece moves at most once per phase, and
    /// nothing else relocates pieces during the Move Phase, the post-move
    /// destination square is the relevant blocker the next time we generate.
    /// Slice 1 will store *destination* squares here for that reason; this
    /// stub leaves the exact semantics to the make/unmake implementation.
    pub moved_this_phase: Bitboard,

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

    /// Terminal-state marker. `Some(winner)` means a King has been removed
    /// and the game is over; the engine emits no further legal actions.
    /// Set by `make()` inside `deal_one_damage()` when a King's bit is about
    /// to be cleared; cleared by `unmake()` via the Undo snapshot. Also
    /// reconstructed deterministically from the bitboards by `from_fen()`
    /// (a parsed FEN with one King missing is by definition a finished game).
    pub game_result: Option<GameResult>,
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
            round_number: 1,
            moved_this_phase: Bitboard::EMPTY,
            pending_modifiers: 0,
            tracked_enemies: [0; MAX_TRACKED_ENEMIES],
            tracked_enemies_len: 0,
            champion_credit: 0,
            zobrist: 0,
            game_result: None,
        }
    }

    /// True iff the square is currently occupied (per the authoritative bitboards).
    #[inline]
    pub fn is_occupied(&self, sq: u8) -> bool {
        (self.p1_pieces | self.p2_pieces).contains(sq)
    }

    // === Layer 1: Setup ====================================================

    /// Stack M canonical starting position.
    /// Source of truth: `SELECT body FROM stacks WHERE id='stack-m';`.
    ///
    /// Stack M permits multiple offset-King layouts; this is the single
    /// canonical one the engine boots into. A parameterised
    /// `setup_stack_m_with(...)` arrives when the frontend exposes layout
    /// choice to players.
    ///
    /// Layout (a..h on files, 1..=8 on ranks):
    /// ```text
    /// rank 8: .ccckcc.    P2 back  — King on e8
    /// rank 7: .gggggg.    P2 front
    /// rank 6: ........
    /// rank 5: ........
    /// rank 4: ........
    /// rank 3: ........
    /// rank 2: .GGGGGG.    P1 front
    /// rank 1: .CCKCCC.    P1 back  — King on d1
    /// ```
    /// Counts per side: 1 King + 5 Champions + 6 Guards = 12 pieces.
    /// Starting money 6 each, P1 to move, Move Phase, 2 actions.
    pub fn setup_stack_m() -> Self {
        let mut p = Self::empty();

        // Files 1..=6 (b..g) are occupied on both back rows; files 0 and 7 empty.
        place_back_row(&mut p, /*rank=*/ 0, Player::P1, /*king_file=*/ 3);
        place_front_row(&mut p, /*rank=*/ 1, Player::P1);
        place_front_row(&mut p, /*rank=*/ 6, Player::P2);
        place_back_row(&mut p, /*rank=*/ 7, Player::P2, /*king_file=*/ 4);

        p.to_move = Player::P1;
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;
        p.p1_money = 6;
        p.p2_money = 6;
        p.round_number = 1;
        p.moved_this_phase = Bitboard::EMPTY;
        p.pending_modifiers = 0;
        p
    }

    /// Serialise to the project's FEN-like single-line format.
    /// See `state::fen` module docs for the grammar.
    pub fn to_fen(&self) -> String {
        super::fen::to_fen(self)
    }

    /// Parse a FEN-like string into a Position. Returns a tagged `FenError`
    /// describing the first parse failure encountered.
    pub fn from_fen(s: &str) -> Result<Self, super::fen::FenError> {
        super::fen::from_fen(s)
    }

    /// Derive `game_result` from the bitboards alone. A side with zero Kings
    /// has lost (Stack M: "The game ends immediately when a King is removed
    /// from the board."). Called by `from_fen()` after parsing, and by tests
    /// that hand-build positions. `make()` maintains `game_result`
    /// incrementally so this need not be called in the hot path.
    pub fn recompute_game_result(&mut self) {
        let p1_has_king = !(self.kings & self.p1_pieces).is_empty();
        let p2_has_king = !(self.kings & self.p2_pieces).is_empty();
        self.game_result = match (p1_has_king, p2_has_king) {
            (true,  true)  => None,
            (false, true)  => Some(GameResult::P2Wins),
            (true,  false) => Some(GameResult::P1Wins),
            // Both Kings removed in a single state is impossible in Stack M:
            // a Move-Phase action removes at most one piece, and FEN parsing
            // already requires ≥1 King per side via the strict validator
            // (or accepts 0 Kings under lax mode — in which case we pick
            // P2Wins as a deterministic fallback rather than introduce a
            // Draw variant).
            (false, false) => Some(GameResult::P2Wins),
        };
    }
}

// === Setup helpers (module-private) =========================================

/// Place a back row (rank 0 for P1, rank 7 for P2). Files 1..=6 hold pieces;
/// `king_file` carries the King, the other five files carry Champions.
fn place_back_row(p: &mut Position, rank: u8, player: Player, king_file: u8) {
    debug_assert!(rank == 0 || rank == 7);
    debug_assert!((1..=6).contains(&king_file), "king must sit on b..g");
    let default_entry = EMPTY_MAILBOX_ENTRY.with_hp(2);
    for file in 1u8..=6u8 {
        let sq = rank * 8 + file;
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        if file == king_file {
            p.kings = p.kings | bit;
        } else {
            p.champions = p.champions | bit;
        }
        p.mailbox[sq as usize] = default_entry;
    }
}

/// Place a front row of Guards (rank 1 for P1, rank 6 for P2). Files 1..=6.
fn place_front_row(p: &mut Position, rank: u8, player: Player) {
    debug_assert!(rank == 1 || rank == 6);
    let default_entry = EMPTY_MAILBOX_ENTRY.with_hp(2);
    for file in 1u8..=6u8 {
        let sq = rank * 8 + file;
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        p.guards = p.guards | bit;
        p.mailbox[sq as usize] = default_entry;
    }
}
