//! FEN-like single-line serialisation of a `Position`.
//!
//! # Grammar
//!
//! ```text
//! <fen>           ::= <board> ' ' <to_move> ' ' <phase> ' ' <actions_remaining>
//!                     ' ' <p1_money> ' ' <p2_money> ' ' <pending_modifiers>
//! <board>         ::= <rank> ('/' <rank>){7}        ; rank 8 first, rank 1 last
//! <rank>          ::= ( <piece-token> | <digit> ){1..}    ; squares per rank sum to 8
//! <piece-token>   ::= <piece-char> [ '[' <hp> '/' <armor> '/' <combo>
//!                                    '/' <s1> '/' <s2> ']' ]
//! <piece-char>    ::= 'K' | 'C' | 'G'                ; uppercase = P1
//!                   | 'k' | 'c' | 'g'                ; lowercase = P2
//! <digit>         ::= '1'..='8'                      ; run-length empties
//! <to_move>       ::= 'P1' | 'P2'
//! <phase>         ::= 'M' | 'S'                      ; Move | Skill
//! <actions_remaining> ::= 0..=255 decimal
//! <p1_money>, <p2_money> ::= 0..=65535 decimal
//! <pending_modifiers> ::= 0..=255 decimal (bit 0 = FOCUS, bit 1 = CHARGE)
//! ```
//!
//! Bracketed mailbox fields default to `2/0/0/0/0` (full HP, no armor, no combo,
//! no skills) when omitted. Encoder emits the bracket iff *any* field is
//! non-default.
//!
//! # Square ordering
//!
//! Within a rank, files run **a..h left-to-right**. Bitboard square index
//! `sq = rank_idx * 8 + file_idx` (rank 0 = bottom = P1 side, file 0 = a).
//! When parsing rank-8-first FEN, we walk top-to-bottom so the first rank
//! token corresponds to bitboard rank 7.
//!
//! # Fields NOT serialised
//!
//! - `tracked_enemies`, `tracked_enemies_len`, `champion_credit` — turn-scoped,
//!   cleared at end of turn. `from_fen` zeroes them; `to_fen` only round-trips
//!   a Position where they are already zero (FEN is between-turn state).
//! - `zobrist` — derived. `from_fen` sets it to 0 (Zobrist keys aren't wired
//!   yet; slice 6 revisits). Tests use `position_eq_for_fen` to skip it.
//!
//! See `crates/core_engine/SCENARIO_FORMAT.md` for the action-text and
//! scenario-file grammars built on top of FEN.

use crate::state::{
    bitboard::Bitboard,
    mailbox::EMPTY_MAILBOX_ENTRY,
    position::{Phase, Player, Position, MAX_TRACKED_ENEMIES},
};
use std::fmt::Write as _;

// --- Default mailbox values (for "no bracket" piece tokens) -----------------

const DEFAULT_HP:    u8 = 2;
const DEFAULT_ARMOR: u8 = 0;
const DEFAULT_COMBO: u8 = 0;
const DEFAULT_SKILL: u8 = 0;

// --- Errors -----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    /// FEN had the wrong number of space-separated fields. Expected 7.
    WrongFieldCount { got: usize },
    /// `<board>` did not contain exactly 8 `/`-separated ranks.
    WrongRankCount { got: usize },
    /// A single rank's squares did not sum to 8.
    RankSquareCountMismatch { rank_idx_from_top: usize, got: usize },
    /// Unknown character where a piece or run-length digit was expected.
    UnexpectedChar { rank_idx_from_top: usize, ch: char },
    /// Bracket opened but never closed before next piece / rank delimiter.
    UnterminatedBracket { rank_idx_from_top: usize },
    /// Bracket contents weren't 5 slash-separated decimals.
    MalformedBracket { rank_idx_from_top: usize },
    /// Mailbox field out of range (e.g. HP > 2, armor > 3, combo > 7, skill > 15).
    MailboxFieldOutOfRange { field: &'static str, value: u32 },
    /// `<to_move>` was neither `P1` nor `P2`.
    BadToMove,
    /// `<phase>` was neither `M` nor `S`.
    BadPhase,
    /// A decimal field failed to parse or was out of its type's range.
    BadDecimal { field: &'static str },
    /// Exactly one King per side is required.
    KingCount { p1_kings: u32, p2_kings: u32 },
    /// Strict-mode only: piece counts per side aren't 1K + 5C + 6G.
    /// `kind` ∈ `{"kings", "champions", "guards"}`.
    WrongPieceCount { player: Player, kind: &'static str, expected: u32, got: u32 },
    /// Strict-mode only: both Kings are on the same file. Stack M requires the offset.
    KingsOnSameFile { file: u8 },
    /// Internal sanity check: after parse, occupancy bitboards disagree with
    /// the per-piece bitboards. Indicates an encoder/parser bug, not bad input.
    InternalOccupancyMismatch,
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for FenError {}

// --- Encoder ----------------------------------------------------------------

pub fn to_fen(pos: &Position) -> String {
    let mut out = String::with_capacity(96);

    // Board: rank 7 (top of display) down to rank 0.
    for rank_top in 0..8u8 {
        let rank_idx = 7 - rank_top; // bitboard rank
        if rank_top > 0 {
            out.push('/');
        }
        let mut empty_run: u8 = 0;
        for file in 0..8u8 {
            let sq = rank_idx * 8 + file;
            if pos.is_occupied(sq) {
                if empty_run > 0 {
                    write!(&mut out, "{}", empty_run).unwrap();
                    empty_run = 0;
                }
                write_piece_token(&mut out, pos, sq);
            } else {
                empty_run += 1;
            }
        }
        if empty_run > 0 {
            write!(&mut out, "{}", empty_run).unwrap();
        }
    }

    let to_move = match pos.to_move { Player::P1 => "P1", Player::P2 => "P2" };
    let phase   = match pos.current_phase { Phase::Move => "M", Phase::Skill => "S" };

    write!(
        &mut out,
        " {} {} {} {} {} {}",
        to_move,
        phase,
        pos.actions_remaining,
        pos.p1_money,
        pos.p2_money,
        pos.pending_modifiers,
    ).unwrap();

    out
}

fn write_piece_token(out: &mut String, pos: &Position, sq: u8) {
    let is_p1 = pos.p1_pieces.contains(sq);
    let ch = if pos.kings.contains(sq) {
        if is_p1 { 'K' } else { 'k' }
    } else if pos.champions.contains(sq) {
        if is_p1 { 'C' } else { 'c' }
    } else {
        // Must be a Guard. If neither King/Champion/Guard bit is set on an
        // occupied square, that's an internal invariant violation; we still
        // emit *something* rather than panic during a read-only operation.
        debug_assert!(pos.guards.contains(sq), "occupied square missing piece type bit");
        if is_p1 { 'G' } else { 'g' }
    };
    out.push(ch);

    let entry = pos.mailbox[sq as usize];
    let hp = entry.hp();
    let ar = entry.armor();
    let co = entry.combo();
    let s1 = entry.skill1();
    let s2 = entry.skill2();
    let is_default = hp == DEFAULT_HP
        && ar == DEFAULT_ARMOR
        && co == DEFAULT_COMBO
        && s1 == DEFAULT_SKILL
        && s2 == DEFAULT_SKILL;
    if !is_default {
        write!(out, "[{}/{}/{}/{}/{}]", hp, ar, co, s1, s2).unwrap();
    }
}

// --- Parser -----------------------------------------------------------------

pub fn from_fen(s: &str) -> Result<Position, FenError> {
    let fields: Vec<&str> = s.split_ascii_whitespace().collect();
    if fields.len() != 7 {
        return Err(FenError::WrongFieldCount { got: fields.len() });
    }

    let board_str = fields[0];
    let to_move_s = fields[1];
    let phase_s   = fields[2];
    let actions_s = fields[3];
    let p1_money_s = fields[4];
    let p2_money_s = fields[5];
    let modifiers_s = fields[6];

    let mut pos = Position::empty();

    parse_board(board_str, &mut pos)?;

    pos.to_move = match to_move_s {
        "P1" => Player::P1,
        "P2" => Player::P2,
        _    => return Err(FenError::BadToMove),
    };
    pos.current_phase = match phase_s {
        "M" => Phase::Move,
        "S" => Phase::Skill,
        _   => return Err(FenError::BadPhase),
    };
    pos.actions_remaining = actions_s
        .parse::<u8>()
        .map_err(|_| FenError::BadDecimal { field: "actions_remaining" })?;
    pos.p1_money = p1_money_s
        .parse::<u16>()
        .map_err(|_| FenError::BadDecimal { field: "p1_money" })?;
    pos.p2_money = p2_money_s
        .parse::<u16>()
        .map_err(|_| FenError::BadDecimal { field: "p2_money" })?;
    pos.pending_modifiers = modifiers_s
        .parse::<u8>()
        .map_err(|_| FenError::BadDecimal { field: "pending_modifiers" })?;

    // Validate exactly one King per side.
    let p1_king_count = (pos.p1_pieces & pos.kings).count();
    let p2_king_count = (pos.p2_pieces & pos.kings).count();
    if p1_king_count != 1 || p2_king_count != 1 {
        return Err(FenError::KingCount {
            p1_kings: p1_king_count,
            p2_kings: p2_king_count,
        });
    }

    // Sanity: bitboard occupancy union should match per-type bitboards.
    let by_type = pos.kings | pos.champions | pos.guards;
    let by_player = pos.p1_pieces | pos.p2_pieces;
    if by_type.0 != by_player.0 {
        return Err(FenError::InternalOccupancyMismatch);
    }

    // Turn-scoped / derived fields are left at their `empty()` defaults.
    debug_assert_eq!(pos.tracked_enemies_len, 0);
    debug_assert_eq!(pos.champion_credit, 0);
    debug_assert_eq!(pos.tracked_enemies, [0u8; MAX_TRACKED_ENEMIES]);
    debug_assert_eq!(pos.zobrist, 0);

    Ok(pos)
}

fn split_ranks_respecting_brackets(board: &str) -> Result<Vec<&str>, FenError> {
    let mut ranks: Vec<&str> = Vec::with_capacity(8);
    let mut depth: i32 = 0;
    let mut start: usize = 0;
    for (i, b) in board.bytes().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth < 0 {
                    // Stray ']' — bubble up as malformed; we don't know the rank
                    // yet, so report against rank 0 with the offending char.
                    return Err(FenError::UnexpectedChar {
                        rank_idx_from_top: ranks.len(),
                        ch: ']',
                    });
                }
            }
            b'/' if depth == 0 => {
                ranks.push(&board[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(FenError::UnterminatedBracket { rank_idx_from_top: ranks.len() });
    }
    ranks.push(&board[start..]);
    Ok(ranks)
}

fn parse_board(board: &str, pos: &mut Position) -> Result<(), FenError> {
    // Split on '/' but only when NOT inside a [...] bracket — bracket contents
    // contain '/' as field separators (h/a/c/s1/s2).
    let ranks = split_ranks_respecting_brackets(board)?;
    if ranks.len() != 8 {
        return Err(FenError::WrongRankCount { got: ranks.len() });
    }

    for (top_idx, rank_str) in ranks.iter().enumerate() {
        let rank_idx = (7 - top_idx) as u8; // bitboard rank
        let mut file: u8 = 0;
        let mut chars = rank_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if let Some(n) = ch.to_digit(10) {
                if !(1..=8).contains(&n) {
                    return Err(FenError::UnexpectedChar {
                        rank_idx_from_top: top_idx,
                        ch,
                    });
                }
                file = file.saturating_add(n as u8);
                continue;
            }

            let (is_p1, piece_kind) = match ch {
                'K' => (true,  PieceKind::King),
                'k' => (false, PieceKind::King),
                'C' => (true,  PieceKind::Champion),
                'c' => (false, PieceKind::Champion),
                'G' => (true,  PieceKind::Guard),
                'g' => (false, PieceKind::Guard),
                other => return Err(FenError::UnexpectedChar {
                    rank_idx_from_top: top_idx,
                    ch: other,
                }),
            };

            // Optional `[h/a/c/s1/s2]` bracket.
            let (hp, armor, combo, s1, s2) = if matches!(chars.peek(), Some('[')) {
                chars.next(); // consume '['
                let mut buf = String::new();
                let mut closed = false;
                for inner in chars.by_ref() {
                    if inner == ']' { closed = true; break; }
                    buf.push(inner);
                }
                if !closed {
                    return Err(FenError::UnterminatedBracket { rank_idx_from_top: top_idx });
                }
                parse_bracket(&buf, top_idx)?
            } else {
                (DEFAULT_HP, DEFAULT_ARMOR, DEFAULT_COMBO, DEFAULT_SKILL, DEFAULT_SKILL)
            };

            if file >= 8 {
                return Err(FenError::RankSquareCountMismatch {
                    rank_idx_from_top: top_idx,
                    got: file as usize + 1,
                });
            }

            let sq = rank_idx * 8 + file;
            let bit = Bitboard::from_square(sq);

            // Set player bitboard.
            if is_p1 {
                pos.p1_pieces = pos.p1_pieces | bit;
            } else {
                pos.p2_pieces = pos.p2_pieces | bit;
            }
            // Set piece-type bitboard.
            match piece_kind {
                PieceKind::King     => pos.kings     = pos.kings     | bit,
                PieceKind::Champion => pos.champions = pos.champions | bit,
                PieceKind::Guard    => pos.guards    = pos.guards    | bit,
            }
            // Populate mailbox.
            let entry = EMPTY_MAILBOX_ENTRY
                .with_hp(hp)
                .with_armor(armor)
                .with_combo(combo)
                .with_skill1(s1)
                .with_skill2(s2);
            pos.mailbox[sq as usize] = entry;

            file += 1;
        }

        if file != 8 {
            return Err(FenError::RankSquareCountMismatch {
                rank_idx_from_top: top_idx,
                got: file as usize,
            });
        }
    }

    Ok(())
}

// --- Strict-mode validator --------------------------------------------------

/// Strict parse: structural validity (via `from_fen`) **plus** Stack M setup
/// invariants (1 King + 5 Champions + 6 Guards per side, Kings on different
/// files). Use this for setup-position scenarios and the roundtrip test of
/// `setup_stack_m()`. For mid-game positions (captures bring counts down),
/// stick with `from_fen`.
pub fn from_fen_strict(s: &str) -> Result<Position, FenError> {
    let pos = from_fen(s)?;
    validate_stack_m_invariants(&pos)?;
    Ok(pos)
}

fn validate_stack_m_invariants(pos: &Position) -> Result<(), FenError> {
    // Per-side per-type counts. `from_fen` already guarantees 1 King per side,
    // but we re-check here so the error message points at the right `kind`.
    for (player, player_bb) in [(Player::P1, pos.p1_pieces), (Player::P2, pos.p2_pieces)] {
        let kings     = (player_bb & pos.kings).count();
        let champions = (player_bb & pos.champions).count();
        let guards    = (player_bb & pos.guards).count();
        if kings != 1 {
            return Err(FenError::WrongPieceCount {
                player, kind: "kings", expected: 1, got: kings,
            });
        }
        if champions != 5 {
            return Err(FenError::WrongPieceCount {
                player, kind: "champions", expected: 5, got: champions,
            });
        }
        if guards != 6 {
            return Err(FenError::WrongPieceCount {
                player, kind: "guards", expected: 6, got: guards,
            });
        }
    }

    let p1_king_sq = (pos.p1_pieces & pos.kings).lsb().expect("p1 king present");
    let p2_king_sq = (pos.p2_pieces & pos.kings).lsb().expect("p2 king present");
    let p1_file = p1_king_sq % 8;
    let p2_file = p2_king_sq % 8;
    if p1_file == p2_file {
        return Err(FenError::KingsOnSameFile { file: p1_file });
    }

    Ok(())
}

fn parse_bracket(buf: &str, rank_idx_from_top: usize) -> Result<(u8, u8, u8, u8, u8), FenError> {
    let parts: Vec<&str> = buf.split('/').collect();
    if parts.len() != 5 {
        return Err(FenError::MalformedBracket { rank_idx_from_top });
    }
    let nums: Vec<u32> = parts
        .iter()
        .map(|p| p.parse::<u32>().map_err(|_| FenError::MalformedBracket { rank_idx_from_top }))
        .collect::<Result<_, _>>()?;

    if nums[0] > 2  { return Err(FenError::MailboxFieldOutOfRange { field: "hp",     value: nums[0] }); }
    if nums[1] > 3  { return Err(FenError::MailboxFieldOutOfRange { field: "armor",  value: nums[1] }); }
    if nums[2] > 7  { return Err(FenError::MailboxFieldOutOfRange { field: "combo",  value: nums[2] }); }
    if nums[3] > 15 { return Err(FenError::MailboxFieldOutOfRange { field: "skill1", value: nums[3] }); }
    if nums[4] > 15 { return Err(FenError::MailboxFieldOutOfRange { field: "skill2", value: nums[4] }); }

    Ok((nums[0] as u8, nums[1] as u8, nums[2] as u8, nums[3] as u8, nums[4] as u8))
}

#[derive(Clone, Copy)]
enum PieceKind { King, Champion, Guard }

// --- Test helper: structural equality ignoring derived/transient fields -----

/// Compare two Positions for FEN-roundtrip equivalence. Excludes the three
/// fields that FEN intentionally does not carry (`tracked_enemies*`,
/// `champion_credit`, `zobrist`) plus the mailbox slots on unoccupied squares
/// (per the bitboards-authoritative invariant, those are undefined).
#[cfg(test)]
pub(crate) fn position_eq_for_fen(a: &Position, b: &Position) -> bool {
    if a.p1_pieces.0 != b.p1_pieces.0 { return false; }
    if a.p2_pieces.0 != b.p2_pieces.0 { return false; }
    if a.kings.0     != b.kings.0     { return false; }
    if a.champions.0 != b.champions.0 { return false; }
    if a.guards.0    != b.guards.0    { return false; }
    if a.p1_money != b.p1_money { return false; }
    if a.p2_money != b.p2_money { return false; }
    if a.actions_remaining != b.actions_remaining { return false; }
    if a.pending_modifiers != b.pending_modifiers { return false; }
    if a.to_move != b.to_move { return false; }
    if a.current_phase != b.current_phase { return false; }
    // Mailbox: only the occupied squares matter.
    let occ = (a.p1_pieces | a.p2_pieces).0;
    for sq in 0..64u8 {
        if (occ >> sq) & 1 == 1 {
            if a.mailbox[sq as usize].0 != b.mailbox[sq as usize].0 {
                return false;
            }
        }
    }
    true
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn round(pos: &Position) -> Position {
        let s = to_fen(pos);
        from_fen(&s).expect("roundtrip parse")
    }

    // Helper: build a minimal-valid Position with one King per side at the
    // corners and nothing else. Used by tests that need a baseline.
    fn two_kings() -> Position {
        let mut p = Position::empty();
        let p1_king_sq = 0u8;   // a1
        let p2_king_sq = 63u8;  // h8
        p.p1_pieces = Bitboard::from_square(p1_king_sq);
        p.p2_pieces = Bitboard::from_square(p2_king_sq);
        p.kings     = Bitboard::from_square(p1_king_sq) | Bitboard::from_square(p2_king_sq);
        p.mailbox[p1_king_sq as usize] = EMPTY_MAILBOX_ENTRY.with_hp(DEFAULT_HP);
        p.mailbox[p2_king_sq as usize] = EMPTY_MAILBOX_ENTRY.with_hp(DEFAULT_HP);
        p.actions_remaining = 2;
        p.p1_money = 6;
        p.p2_money = 6;
        p
    }

    #[test]
    fn two_kings_roundtrip() {
        let p = two_kings();
        let p2 = round(&p);
        assert!(position_eq_for_fen(&p, &p2), "got: {}", to_fen(&p2));
    }

    #[test]
    fn single_piece_each_player_format() {
        let p = two_kings();
        let s = to_fen(&p);
        // h8 is rank 7, file 7 → first rank in FEN. a1 is bitboard rank 0, last in FEN.
        // Expected board: "7k/8/8/8/8/8/8/K7"
        assert_eq!(s, "7k/8/8/8/8/8/8/K7 P1 M 2 6 6 0");
    }

    #[test]
    fn full_board_no_brackets() {
        // Fill every square with a default piece. Kings at a1/h8, Guards elsewhere.
        let mut p = Position::empty();
        for sq in 0..64u8 {
            let is_p1 = sq < 32;
            let bit = Bitboard::from_square(sq);
            if is_p1 {
                p.p1_pieces = p.p1_pieces | bit;
            } else {
                p.p2_pieces = p.p2_pieces | bit;
            }
            if sq == 0 || sq == 63 {
                p.kings = p.kings | bit;
            } else {
                p.guards = p.guards | bit;
            }
            p.mailbox[sq as usize] = EMPTY_MAILBOX_ENTRY.with_hp(DEFAULT_HP);
        }
        let p2 = round(&p);
        assert!(position_eq_for_fen(&p, &p2));
    }

    #[test]
    fn non_default_mailbox_roundtrip() {
        let mut p = two_kings();
        // Place a P1 Champion at e2 (rank 1, file 4 → sq 12) with custom mailbox.
        let sq = 12u8;
        p.p1_pieces = p.p1_pieces | Bitboard::from_square(sq);
        p.champions = p.champions | Bitboard::from_square(sq);
        p.mailbox[sq as usize] = EMPTY_MAILBOX_ENTRY
            .with_hp(1)
            .with_armor(2)
            .with_combo(3)
            .with_skill1(7)
            .with_skill2(15);

        let s = to_fen(&p);
        assert!(s.contains("C[1/2/3/7/15]"), "got: {}", s);

        let p2 = from_fen(&s).expect("parse");
        assert!(position_eq_for_fen(&p, &p2));
    }

    #[test]
    fn phase_money_actions_roundtrip() {
        let mut p = two_kings();
        p.to_move = Player::P2;
        p.current_phase = Phase::Skill;
        p.actions_remaining = 17;
        p.p1_money = 42;
        p.p2_money = 1337;
        let p2 = round(&p);
        assert!(position_eq_for_fen(&p, &p2));
        let s = to_fen(&p);
        assert!(s.ends_with(" P2 S 17 42 1337 0"), "got: {}", s);
    }

    #[test]
    fn pending_modifiers_roundtrip() {
        for bits in 0u8..=3 {
            let mut p = two_kings();
            p.pending_modifiers = bits;
            let p2 = round(&p);
            assert_eq!(p2.pending_modifiers, bits);
        }
    }

    #[test]
    fn rejects_two_kings_one_side() {
        // Two P1 Kings, no P2 King.
        let bad = "K6k/8/K7/8/8/8/8/8 P1 M 2 6 6 0";
        // Wait — that has 1 P1 king and 1 P2 king. Fix: both K's on top rank.
        let _ = bad;
        let bad = "KK5k/8/8/8/8/8/8/8 P1 M 2 6 6 0";
        match from_fen(bad) {
            Err(FenError::KingCount { p1_kings: 2, p2_kings: 1 }) => {}
            other => panic!("expected KingCount(2,1), got {:?}", other),
        }
    }

    #[test]
    fn rejects_zero_kings() {
        let bad = "8/8/8/8/8/8/8/8 P1 M 2 6 6 0";
        match from_fen(bad) {
            Err(FenError::KingCount { p1_kings: 0, p2_kings: 0 }) => {}
            other => panic!("expected KingCount(0,0), got {:?}", other),
        }
    }

    #[test]
    fn rejects_rank_not_summing_to_8() {
        let bad = "7k/7/8/8/8/8/8/K7 P1 M 2 6 6 0"; // rank "7" alone = 7 squares
        match from_fen(bad) {
            Err(FenError::RankSquareCountMismatch { .. }) => {}
            other => panic!("expected RankSquareCountMismatch, got {:?}", other),
        }
    }

    #[test]
    fn rejects_bad_piece_char() {
        let bad = "7k/8/8/8/X7/8/8/K7 P1 M 2 6 6 0";
        match from_fen(bad) {
            Err(FenError::UnexpectedChar { ch: 'X', .. }) => {}
            other => panic!("expected UnexpectedChar('X'), got {:?}", other),
        }
    }

    #[test]
    fn rejects_bad_to_move() {
        let bad = "7k/8/8/8/8/8/8/K7 P3 M 2 6 6 0";
        assert!(matches!(from_fen(bad), Err(FenError::BadToMove)));
    }

    #[test]
    fn rejects_bad_phase() {
        let bad = "7k/8/8/8/8/8/8/K7 P1 X 2 6 6 0";
        assert!(matches!(from_fen(bad), Err(FenError::BadPhase)));
    }

    #[test]
    fn rejects_wrong_field_count() {
        let bad = "7k/8/8/8/8/8/8/K7 P1 M 2 6 6"; // missing pending_modifiers
        assert!(matches!(from_fen(bad), Err(FenError::WrongFieldCount { got: 6 })));
    }

    #[test]
    fn rejects_mailbox_field_out_of_range() {
        let bad = "7k/8/8/8/C[3/0/0/0/0]7/8/8/K7 P1 M 2 6 6 0"; // hp=3
        match from_fen(bad) {
            Err(FenError::MailboxFieldOutOfRange { field: "hp", value: 3 }) => {}
            other => panic!("expected MailboxFieldOutOfRange(hp,3), got {:?}", other),
        }
    }

    #[test]
    fn rejects_malformed_bracket() {
        let bad = "7k/8/8/8/C[1/2/3]7/8/8/K7 P1 M 2 6 6 0"; // 3 fields, not 5
        assert!(matches!(from_fen(bad), Err(FenError::MalformedBracket { .. })));
    }

    // --- Slice 0: setup_stack_m + strict validation -----------------------

    const CANONICAL_STACK_M_FEN: &str =
        "1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCC1 P1 M 2 6 6 0";

    #[test]
    fn setup_stack_m_matches_expected_fen() {
        assert_eq!(Position::setup_stack_m().to_fen(), CANONICAL_STACK_M_FEN);
    }

    #[test]
    fn setup_stack_m_piece_counts() {
        let p = Position::setup_stack_m();
        // P1 side
        assert_eq!((p.p1_pieces & p.kings).count(),     1);
        assert_eq!((p.p1_pieces & p.champions).count(), 5);
        assert_eq!((p.p1_pieces & p.guards).count(),    6);
        // P2 side
        assert_eq!((p.p2_pieces & p.kings).count(),     1);
        assert_eq!((p.p2_pieces & p.champions).count(), 5);
        assert_eq!((p.p2_pieces & p.guards).count(),    6);
        // Totals
        assert_eq!(p.p1_pieces.count(), 12);
        assert_eq!(p.p2_pieces.count(), 12);
    }

    #[test]
    fn setup_stack_m_kings_on_different_files() {
        let p = Position::setup_stack_m();
        let p1_king_sq = (p.p1_pieces & p.kings).lsb().unwrap();
        let p2_king_sq = (p.p2_pieces & p.kings).lsb().unwrap();
        assert_eq!(p1_king_sq % 8, 3, "P1 king on file d");
        assert_eq!(p2_king_sq % 8, 4, "P2 king on file e");
    }

    #[test]
    fn setup_stack_m_money() {
        let p = Position::setup_stack_m();
        assert_eq!(p.p1_money, 6);
        assert_eq!(p.p2_money, 6);
    }

    #[test]
    fn setup_stack_m_phase_and_actions() {
        let p = Position::setup_stack_m();
        assert_eq!(p.to_move, Player::P1);
        assert_eq!(p.current_phase, Phase::Move);
        assert_eq!(p.actions_remaining, 2);
    }

    #[test]
    fn setup_stack_m_roundtrip() {
        let p = Position::setup_stack_m();
        let s = to_fen(&p);
        let p2 = from_fen(&s).expect("parse setup_stack_m FEN");
        assert!(position_eq_for_fen(&p, &p2));
    }

    #[test]
    fn setup_stack_m_passes_strict() {
        let s = to_fen(&Position::setup_stack_m());
        from_fen_strict(&s).expect("strict accepts canonical Stack M setup");
    }

    #[test]
    fn strict_rejects_kings_same_file() {
        // Both Kings on file d (d1 and d8). Otherwise valid 1+5+6 setup.
        // Swap P2 king from e8 to d8, and one P2 champion from d8 to e8.
        let bad = "1cckccc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCC1 P1 M 2 6 6 0";
        match from_fen_strict(bad) {
            Err(FenError::KingsOnSameFile { file: 3 }) => {}
            other => panic!("expected KingsOnSameFile(3), got {:?}", other),
        }
    }

    #[test]
    fn strict_rejects_wrong_champion_count() {
        // P1 side has 4 Champions instead of 5 (one C replaced by a Guard).
        // 1CCKCCG1 → C,C,K,C,C,G = 4 champs.
        let bad = "1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCG1 P1 M 2 6 6 0";
        match from_fen_strict(bad) {
            Err(FenError::WrongPieceCount { player: Player::P1, kind: "champions", expected: 5, got: 4 }) => {}
            other => panic!("expected WrongPieceCount(P1, champions, 5, 4), got {:?}", other),
        }
    }

    #[test]
    fn strict_rejects_wrong_guard_count() {
        // P1 side has 5 Guards instead of 6.
        let bad = "1ccckcc1/1gggggg1/8/8/8/8/2GGGGG1/1CCKCCC1 P1 M 2 6 6 0";
        match from_fen_strict(bad) {
            Err(FenError::WrongPieceCount { player: Player::P1, kind: "guards", expected: 6, got: 5 }) => {}
            other => panic!("expected WrongPieceCount(P1, guards, 6, 5), got {:?}", other),
        }
    }

    #[test]
    fn lax_accepts_wrong_counts() {
        // Same FEN as strict_rejects_wrong_champion_count — plain from_fen accepts it.
        let mid_game = "1ccckcc1/1gggggg1/8/8/8/8/1GGGGGG1/1CCKCCG1 P1 M 2 6 6 0";
        from_fen(mid_game).expect("lax accepts mid-game piece counts");
    }
}
