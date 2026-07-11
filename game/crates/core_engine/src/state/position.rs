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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Player { P1, P2 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    /// Pre-game skill assignment. Sides alternate `DraftTurn` actions (2 picks
    /// per turn, 6 turns per side, 12 total) until every skill slot is filled,
    /// at which point the engine transitions to `Phase::Move` for play.
    Draft,
    Move,
    Skill,
}

/// Terminal-state marker. `None` means the game is still in progress; a
/// concrete variant means a King has been removed and the named player has
/// won. Stack M has no draws (`"No draw conditions"`), so this enum has no
/// Draw variant by design.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GameResult { P1Wins, P2Wins }

/// Stack M has 5 Champions per player. Index 0..5 identifies a specific
/// Champion within a player's army; mapping from index to current square is
/// maintained by the session/match layer (Champions are stable identities
/// across moves; squares change).
pub const CHAMPIONS_PER_PLAYER: usize = 5;

/// Maximum number of distinct enemy targets a player's Champions can have
/// struck for combo purposes within a single turn. Bumped from 8 to 16 in
/// session-37: multi-Tempest turns can legitimately accumulate >8 distinct
/// enemy targets across the turn (each Tempest can tick its pivot plus any
/// enemy neighbours pushed). 16 is the absolute upper bound for a single
/// player's reachable enemies — opponent has at most 12 pieces (1 King + 5
/// Champions + 6 Guards), capped further by board geometry. See
/// `champion_credit` for the cross-product packing.
pub const MAX_TRACKED_ENEMIES: usize = 16;

/// Maximum number of distinct caster squares tracked for combo-tick gating
/// within a single turn. Stack-M's combo rule is identity-based ("new
/// Champion"), but a Champion cannot cast from two squares in one turn
/// (Move-Phase is over and Strike-skills don't relocate the caster), so
/// caster-identity ≡ caster-square within a turn. NOTE for Slice 5: when
/// Dash/Retreat self-relocate the caster, `tracked_casters` entries need to
/// follow the move.
pub const MAX_TRACKED_CASTERS: usize = 8;

/// Maximum number of guards eligible to intercept a single Move-Attack.
/// Eligible = `eight_neighbours(target) ∩ eight_neighbours(approach_sq) ∩
/// guards ∩ defender_pieces`. The geometric intersection of two king-move
/// neighbourhoods (which always share an edge or corner since the approach is
/// adjacent to the target) is at most 4 squares — and friendly-Guard filtering
/// can only shrink it. `[u8; 4]` is the exact tight bound.
pub const MAX_BODYGUARD_ELIGIBLE: usize = 4;

/// Pending two-ply bodyguard resolution state. `Some` between an attacker's
/// tentative Move-Attack (first hop only, no damage) and the defender's
/// `BodyguardChoice` action that resolves it. Populated in Commit 3 when the
/// Move-Attack split lands; in this commit it is always `None`. Hashed into
/// the Zobrist via dedicated keys (see `state::zobrist`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PendingBodyguard {
    /// Square the attacker started on (pre-hop). Kept for display + unmake.
    pub attacker_src: u8,
    /// Square the attacker is currently on (post first hop == approach_sq).
    pub attacker_now: u8,
    /// Move-Attack's named target.
    pub target_sq: u8,
    /// Eligible guards in ascending square order (matches the output ordering
    /// of `generator::bodyguard_guards_for`). Slots past `eligible_len` are
    /// unused; convention is to zero-fill but consumers must respect `eligible_len`.
    pub eligible: [u8; MAX_BODYGUARD_ELIGIBLE],
    /// Active count in `eligible` (0..=MAX_BODYGUARD_ELIGIBLE).
    pub eligible_len: u8,
}

/// Bitfield positions in `Position::pending_modifiers`.
pub mod modifier_bits {
    pub const FOCUS:  u8 = 1 << 0;  // next skill: +1 Range
    pub const CHARGE: u8 = 1 << 1;  // next Strike skill: +1 damage
    /// Stack N (staged S45): a Move-Attack has been used this turn. Turn-scoped
    /// (cleared by end_turn's 0xFF clear; NOT cleared at Move→Skill phase end,
    /// which is fine — move-attacks only occur in the Move Phase). The generator
    /// suppresses further Move-Attacks while this is set, capping them at 1/turn.
    pub const MOVE_ATTACK_USED: u8 = 1 << 2;
    // bits 3..8 reserved for future turn-scoped modifiers.
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
    /// for round-based progression — both curves are **unbounded** (+1 per
    /// N rounds, no cap):
    ///   * Per-turn income: `2 + round_number / 5` — +1 per 5 rounds.
    ///   * Skill-Phase action budget: `2 + (round_number - 1) / 10` — +1 per
    ///     10 rounds. The paper rule sheet's "R31+: 5" line was shorthand
    ///     for the table cut-off, NOT a cap — R41+ is 6, R51+ is 7, etc.
    /// Income is disbursed at the start of each Player turn (per Stack M).
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
    /// Index into this list is `target_slot` (0..MAX_TRACKED_ENEMIES).
    /// `tracked_enemies_len` is the active count; positions beyond are stale.
    /// Cleared at end of turn.
    pub tracked_enemies: [u8; MAX_TRACKED_ENEMIES],
    pub tracked_enemies_len: u8,

    /// Squares of casters that have ticked at least one enemy's combo counter
    /// this turn. Index is `caster_slot` (0..MAX_TRACKED_CASTERS). See
    /// `champion_credit` for the cross-product bitmap. Cleared at end of turn.
    pub tracked_casters: [u8; MAX_TRACKED_CASTERS],
    pub tracked_casters_len: u8,

    /// Bitmap: bit `caster_slot * MAX_TRACKED_ENEMIES + target_slot` is set
    /// iff the caster at `tracked_casters[caster_slot]` has already ticked
    /// the combo counter of the enemy at `tracked_enemies[target_slot]` this
    /// turn. 8 casters × 16 enemies = 128 bits → u128. Cleared at end of turn.
    pub champion_credit: u128,

    /// Pending two-ply bodyguard resolution state. `Some` between an attacker's
    /// tentative Move-Attack and the defender's `BodyguardChoice` that resolves
    /// it. Always `None` in this commit (Commit 1 of the bodyguard refactor);
    /// populated in Commit 3 when the Move-Attack split lands.
    pub pending_bodyguard: Option<PendingBodyguard>,

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
            tracked_casters: [0; MAX_TRACKED_CASTERS],
            tracked_casters_len: 0,
            champion_credit: 0,
            pending_bodyguard: None,
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
        p.zobrist = super::zobrist::full_recompute(&p);
        p
    }

    /// Stack M setup with pre-assigned skill loadouts. Builds the canonical
    /// `setup_stack_m()` layout, then writes the supplied skill IDs into the
    /// mailbox of each of the 12 skill-bearing pieces (1 King + 5 Champions
    /// per side). Phase = Move, ready for play — bypasses the draft.
    ///
    /// Piece order within a `SideLoadout`: index 0 is the King; indices 1..6
    /// are Champions ordered by starting square ascending (files b..g with
    /// the King's file skipped). This matches the iteration order of
    /// `place_back_row`.
    ///
    /// Skill IDs follow the mailbox encoding: 1..=15 are real skills, 0 is
    /// "unequipped". The caller is responsible for validating the loadout via
    /// `crate::game_logic::skills::validate_loadout`; this constructor takes
    /// the arrays at face value and trusts them.
    pub fn setup_stack_m_with_loadouts(
        p1: &crate::game_logic::skills::SideLoadout,
        p2: &crate::game_logic::skills::SideLoadout,
    ) -> Self {
        let mut p = Self::setup_stack_m();
        // P1 King is on file 3 (d1), P2 King is on file 4 (e8). Each side's
        // `SideLoadout` is expressed in that side's own ascending-square frame
        // (index 0 = King, 1..6 = Champions b→g with the King's file skipped),
        // so P1 and P2 are applied independently — no cross-side mirroring here.
        // Preset (same-array-for-both) mirroring is handled by the caller that
        // builds the P2 loadout; see `mirror_loadout`.
        apply_back_row_loadout(&mut p, /*rank=*/ 0, /*king_file=*/ 3, p1);
        apply_back_row_loadout(&mut p, /*rank=*/ 7, /*king_file=*/ 4, p2);
        // The mailbox entries changed but the bitboards didn't — recompute the
        // hash from scratch rather than trying to deduce per-square deltas.
        p.zobrist = super::zobrist::full_recompute(&p);
        p
    }

    /// Stack M canonical starting position **in Draft phase** — every
    /// skill-bearing piece has skill1=skill2=0, awaiting `DraftTurn` actions
    /// to populate the slots. Otherwise identical to `setup_stack_m()`.
    pub fn setup_stack_m_for_draft() -> Self {
        let mut p = Self::setup_stack_m();
        // Toggle the zobrist phase key off-Move and onto Draft, then flip the
        // field. We bypass `make/unmake` because this is a fresh construction,
        // not an in-progress transition.
        p.zobrist ^= super::zobrist::phase_key_for(Phase::Move)
                   ^ super::zobrist::phase_key_for(Phase::Draft);
        p.current_phase = Phase::Draft;
        // Draft has no per-turn action budget — the only legal action while in
        // draft is a DraftTurn, and that uses its own enumeration path.
        // Setting to 0 keeps the actions_key contribution to the hash stable
        // and means `actions_remaining` reads as a meaningful "no actions
        // available in this phase" outside the draft action stream.
        let old = p.actions_remaining;
        if old != 0 {
            p.zobrist ^= super::zobrist::actions_key(old)
                       ^ super::zobrist::actions_key(0);
            p.actions_remaining = 0;
        }
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

/// Apply a `SideLoadout` to a back row that's already been populated by
/// `place_back_row`. Index 0 of the loadout is the King; indices 1..6 are
/// Champions in starting-square ascending order — matching the file iteration
/// (files b..g with the King's file skipped for Champion ordering). Both sides
/// are expressed in their own ascending-square frame; see `mirror_loadout` for
/// the preset case where P2 must be a 180° rotation of P1.
fn apply_back_row_loadout(
    p: &mut Position,
    rank: u8,
    king_file: u8,
    loadout: &crate::game_logic::skills::SideLoadout,
) {
    debug_assert!(rank == 0 || rank == 7);
    // King first (index 0).
    let king_sq = rank * 8 + king_file;
    let (ks1, ks2) = loadout[0];
    p.mailbox[king_sq as usize] = p.mailbox[king_sq as usize].with_skill1(ks1).with_skill2(ks2);
    // Champions: files 1..6, skip the King's file, in ascending order.
    let mut champ_idx = 1usize;
    for file in 1u8..=6u8 {
        if file == king_file { continue; }
        let sq = rank * 8 + file;
        let (s1, s2) = loadout[champ_idx];
        p.mailbox[sq as usize] = p.mailbox[sq as usize].with_skill1(s1).with_skill2(s2);
        champ_idx += 1;
    }
    debug_assert_eq!(champ_idx, 6, "loadout consumed all 5 Champion entries");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::skills::{SideLoadout, validate_loadout, DraftError};

    fn full_p1_loadout() -> SideLoadout {
        // King: Lance/Hook. Champions: pairs that don't repeat on a single piece.
        [(1,2), (3,4), (5,6), (7,8), (9,10), (11,12)]
    }
    fn full_p2_loadout() -> SideLoadout {
        [(2,1), (4,3), (6,5), (8,7), (10,9), (12,11)]
    }

    #[test]
    fn setup_stack_m_with_loadouts_writes_skills_into_back_rank() {
        let p1 = full_p1_loadout();
        let p2 = full_p2_loadout();
        let pos = Position::setup_stack_m_with_loadouts(&p1, &p2);

        // Bitboards / phase unchanged from setup_stack_m.
        assert_eq!(pos.current_phase, Phase::Move);
        assert_eq!(pos.to_move, Player::P1);
        assert_eq!(pos.actions_remaining, 2);

        // P1 King is on d1 (sq 3). Check its skill slots.
        let king_sq = (pos.p1_pieces & pos.kings).lsb().unwrap();
        assert_eq!(king_sq, 3);
        assert_eq!(pos.mailbox[king_sq as usize].skill1(), 1);
        assert_eq!(pos.mailbox[king_sq as usize].skill2(), 2);

        // P1 Champions on files b,c,e,f,g of rank 1 (King on d1 skipped).
        let champ_files = [1u8, 2, 4, 5, 6];
        for (idx, &file) in champ_files.iter().enumerate() {
            let sq = file;
            let (s1, s2) = p1[idx + 1];
            assert_eq!(pos.mailbox[sq as usize].skill1(), s1, "P1 champ file {} slot1", file);
            assert_eq!(pos.mailbox[sq as usize].skill2(), s2, "P1 champ file {} slot2", file);
        }

        // P2 is placed in its OWN ascending-square frame (no cross-side mirror
        // in the constructor): King index 0 → e8 (sq 60), Champions index 1..5
        // → P2 files [b,c,d,f,g] of rank 7 = squares [57,58,59,61,62].
        let p2_king_sq = (pos.p2_pieces & pos.kings).lsb().unwrap();
        assert_eq!(p2_king_sq, 60, "P2 King on e8");
        assert_eq!(pos.mailbox[60].skill1(), p2[0].0, "P2 King slot1");
        assert_eq!(pos.mailbox[60].skill2(), p2[0].1, "P2 King slot2");
        let p2_champ_sqs = [57u8, 58, 59, 61, 62];
        for (idx, &sq) in p2_champ_sqs.iter().enumerate() {
            let (s1, s2) = p2[idx + 1];
            assert_eq!(pos.mailbox[sq as usize].skill1(), s1, "P2 champ sq {} slot1", sq);
            assert_eq!(pos.mailbox[sq as usize].skill2(), s2, "P2 champ sq {} slot2", sq);
        }

        // Hash is recomputed from scratch — sanity-check it matches full_recompute.
        assert_eq!(pos.zobrist, super::super::zobrist::full_recompute(&pos));
    }

    #[test]
    fn mirror_loadout_produces_point_symmetric_board() {
        // Regression for the preset-mirroring bug: a preset is authored in P1's
        // frame; feeding `mirror_loadout(&preset)` as the P2 argument must make
        // P2 a 180° rotation of P1 — a P1 file-b Champion (Lance+Shield) and a
        // P2 file-g Champion must share skills.
        use crate::game_logic::skills::mirror_loadout;
        let preset: SideLoadout = [(1,2), (3,4), (5,6), (7,8), (9,10), (11,12)];
        let pos = Position::setup_stack_m_with_loadouts(&preset, &mirror_loadout(&preset));

        // Every P1 back-rank piece at sq must share skills with the P2 piece at
        // its 180° mirror (63 - sq).
        for sq in 1u8..=6 {
            let p1e = pos.mailbox[sq as usize];
            let p2e = pos.mailbox[(63 - sq) as usize];
            assert_eq!(p1e.skill1(), p2e.skill1(),
                "P1 sq {} and P2 sq {} must share skill1", sq, 63 - sq);
            assert_eq!(p1e.skill2(), p2e.skill2(),
                "P1 sq {} and P2 sq {} must share skill2", sq, 63 - sq);
        }
        // Designer's example: P1 file b (sq 1) skills land on P2 file g (sq 62),
        // NOT on P2 file b (sq 57).
        assert_eq!(pos.mailbox[1].skill1(), pos.mailbox[62].skill1(), "b1 → g8 slot1");
        assert_eq!(pos.mailbox[1].skill2(), pos.mailbox[62].skill2(), "b1 → g8 slot2");
    }

    #[test]
    fn setup_stack_m_for_draft_has_empty_skills_and_draft_phase() {
        let pos = Position::setup_stack_m_for_draft();
        assert_eq!(pos.current_phase, Phase::Draft);
        // Skill slots empty on every back-rank piece.
        for sq in 1u8..=6 {  // P1 back rank
            let e = pos.mailbox[sq as usize];
            assert_eq!(e.skill1(), 0, "P1 sq {} should have empty skill1", sq);
            assert_eq!(e.skill2(), 0, "P1 sq {} should have empty skill2", sq);
        }
        for sq in 57u8..=62 {  // P2 back rank
            let e = pos.mailbox[sq as usize];
            assert_eq!(e.skill1(), 0, "P2 sq {} should have empty skill1", sq);
            assert_eq!(e.skill2(), 0, "P2 sq {} should have empty skill2", sq);
        }
        // Zobrist consistent with full_recompute.
        assert_eq!(pos.zobrist, super::super::zobrist::full_recompute(&pos));
    }

    #[test]
    fn validate_loadout_accepts_full_valid_loadout() {
        assert!(validate_loadout(&full_p1_loadout()).is_ok());
    }

    #[test]
    fn validate_loadout_accepts_partial_empty_slots() {
        // Mid-draft state — some pieces still 0/0, allowed.
        let l: SideLoadout = [(0,0), (1,2), (0,0), (3,4), (0,0), (0,0)];
        assert!(validate_loadout(&l).is_ok());
    }

    #[test]
    fn validate_loadout_rejects_same_skill_twice_on_one_piece() {
        let mut l = full_p1_loadout();
        l[2] = (5, 5);
        match validate_loadout(&l) {
            Err(DraftError::DuplicateOnPiece { piece_index: 2, skill_id: 5 }) => {}
            other => panic!("expected DuplicateOnPiece(2,5), got {:?}", other),
        }
    }

    #[test]
    fn validate_loadout_accepts_zero_on_both_slots() {
        // (0, 0) means an unfilled piece — not a duplicate.
        let mut l = full_p1_loadout();
        l[1] = (0, 0);
        assert!(validate_loadout(&l).is_ok());
    }

    #[test]
    fn validate_loadout_rejects_bad_skill_id() {
        let mut l = full_p1_loadout();
        l[0] = (16, 1);
        match validate_loadout(&l) {
            Err(DraftError::BadSkillId { piece_index: 0, slot: 1, skill_id: 16 }) => {}
            other => panic!("expected BadSkillId(0,1,16), got {:?}", other),
        }
    }

    // --- Commit 1: PendingBodyguard data-layer is None by default -----------

    #[test]
    fn empty_position_has_no_pending_bodyguard() {
        assert!(Position::empty().pending_bodyguard.is_none());
    }

    #[test]
    fn setup_stack_m_has_no_pending_bodyguard() {
        assert!(Position::setup_stack_m().pending_bodyguard.is_none());
    }

    #[test]
    fn setup_stack_m_for_draft_has_no_pending_bodyguard() {
        assert!(Position::setup_stack_m_for_draft().pending_bodyguard.is_none());
    }

    #[test]
    fn setup_stack_m_with_loadouts_has_no_pending_bodyguard() {
        let p = Position::setup_stack_m_with_loadouts(&full_p1_loadout(), &full_p2_loadout());
        assert!(p.pending_bodyguard.is_none());
    }
}
