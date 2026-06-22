//! Make / Unmake — apply an action to a Position, or perfectly reverse it
//! using a previously-written Undo record.
//!
//! Reversibility is mandatory: tree search must not copy state. `make`
//! writes an `Undo` describing exactly what changed; `unmake` consumes it
//! to restore the prior state. The Action itself stays immutable.
//!
//! # Slice 1 scope
//!
//! Implements Move-Phase mechanics only:
//! - Plain Move: clear src, set dest, mark dest in `moved_this_phase`.
//! - Move-Attack: enemy at target takes 1 damage (Armor → HP); mover stays
//!   put and is marked at its origin square in `moved_this_phase`. Bodyguard
//!   redirect is decoded via `choice_idx` and the resolver picks the same
//!   k-th adjacent friendly Guard the generator enumerated.
//! - EndPhase: Move → Skill transition (clear `moved_this_phase`, reset
//!   actions, flip current_phase). Skill→EndTurn deferred to a later slice.
//! - EndTurn: full delegation to `turn_manager::end_turn` — not exercised
//!   by Slice-1 tests but wired so the action surface is complete.
//!
//! Skill-kind actions are not implemented yet; calling `make` with one is
//! a debug-time panic (treated as an engine bug, not a user error).

use super::action::{Action, ActionKind, Undo};
use crate::state::position::{GameResult, Phase, Player};
use crate::state::{Bitboard, MailboxEntry, Position, EMPTY_MAILBOX_ENTRY};

pub fn make(pos: &mut Position, action: Action) -> Undo {
    let mut undo = Undo {
        action: action.0,
        prev_game_result: game_result_to_tag(pos.game_result),
        prev_pending_modifiers: pos.pending_modifiers,
        prev_phase: phase_to_byte(pos.current_phase),
        prev_actions_remaining: pos.actions_remaining,
        prev_moved_this_phase: pos.moved_this_phase.0,
        prev_round_number: pos.round_number,
        p1_money_delta: 0,
        p2_money_delta: 0,
        prev_champion_credit: pos.champion_credit,
        prev_tracked_enemies: pos.tracked_enemies,
        prev_tracked_enemies_len: pos.tracked_enemies_len,
        affected_count: 0,
        affected_squares: [0; 16],
        affected_prev_entries: [0; 16],
        p1_pieces_xor: 0,
        p2_pieces_xor: 0,
        kings_xor: 0,
        champions_xor: 0,
        guards_xor: 0,
        zobrist_xor: 0,
    };

    match action.kind() {
        ActionKind::Move      => apply_move(pos, action, &mut undo),
        ActionKind::EndPhase  => apply_end_phase(pos, &mut undo),
        ActionKind::EndTurn   => super::turn_manager::end_turn(pos),
        ActionKind::Skill     => panic!("Skill-kind make() not implemented until Slice 4+"),
    }

    undo
}

pub fn unmake(pos: &mut Position, undo: &Undo) {
    // Restore mailbox entries that were touched.
    for i in 0..undo.affected_count as usize {
        let sq = undo.affected_squares[i] as usize;
        pos.mailbox[sq] = MailboxEntry(undo.affected_prev_entries[i]);
    }

    // XOR bitboard deltas to revert.
    pos.p1_pieces = Bitboard(pos.p1_pieces.0 ^ undo.p1_pieces_xor);
    pos.p2_pieces = Bitboard(pos.p2_pieces.0 ^ undo.p2_pieces_xor);
    pos.kings     = Bitboard(pos.kings.0     ^ undo.kings_xor);
    pos.champions = Bitboard(pos.champions.0 ^ undo.champions_xor);
    pos.guards    = Bitboard(pos.guards.0    ^ undo.guards_xor);

    // Restore scalars and bitfields.
    pos.pending_modifiers  = undo.prev_pending_modifiers;
    pos.current_phase      = phase_from_byte(undo.prev_phase);
    pos.actions_remaining  = undo.prev_actions_remaining;
    pos.moved_this_phase   = Bitboard(undo.prev_moved_this_phase);
    pos.round_number       = undo.prev_round_number;
    pos.champion_credit    = undo.prev_champion_credit;
    pos.tracked_enemies    = undo.prev_tracked_enemies;
    pos.tracked_enemies_len = undo.prev_tracked_enemies_len;
    pos.game_result        = game_result_from_tag(undo.prev_game_result);

    // Money — invert deltas. Wrapping arithmetic is safe: any value that
    // produced a valid forward delta produces a valid reverse delta.
    if undo.p1_money_delta != 0 {
        pos.p1_money = (pos.p1_money as i32 - undo.p1_money_delta as i32) as u16;
    }
    if undo.p2_money_delta != 0 {
        pos.p2_money = (pos.p2_money as i32 - undo.p2_money_delta as i32) as u16;
    }

    // Zobrist will arrive once the keys are wired (Slice 6 / TT integration).
    pos.zobrist ^= undo.zobrist_xor;
}

// === Move-kind dispatch ====================================================

/// Apply a Move-kind action — plain move or Move-Attack — and populate Undo.
fn apply_move(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let occ_target_by_p1 = pos.p1_pieces.contains(tgt);
    let occ_target_by_p2 = pos.p2_pieces.contains(tgt);

    if occ_target_by_p1 || occ_target_by_p2 {
        apply_move_attack(pos, action, undo);
    } else {
        apply_plain_move(pos, src, tgt, undo);
    }

    debug_assert!(pos.actions_remaining > 0, "make() invoked with zero actions");
    pos.actions_remaining -= 1;
}

/// Move mover from `src` to `tgt`. `moved_this_phase` is set on `tgt`.
fn apply_plain_move(pos: &mut Position, src: u8, tgt: u8, undo: &mut Undo) {
    debug_assert!(pos.is_occupied(src), "plain move from empty square");
    debug_assert!(!pos.is_occupied(tgt), "plain move into occupied square");

    let prev_entry = pos.mailbox[src as usize];
    record_affected(undo, src, prev_entry);
    record_affected(undo, tgt, pos.mailbox[tgt as usize]);

    // Mailbox: clear src, copy entry to tgt.
    pos.mailbox[src as usize] = EMPTY_MAILBOX_ENTRY;
    pos.mailbox[tgt as usize] = prev_entry;

    // Bitboards: src and tgt flip in every layer the piece belongs to.
    let xor = Bitboard::from_square(src).0 | Bitboard::from_square(tgt).0;
    if pos.p1_pieces.contains(src) {
        pos.p1_pieces = Bitboard(pos.p1_pieces.0 ^ xor);
        undo.p1_pieces_xor ^= xor;
    } else {
        pos.p2_pieces = Bitboard(pos.p2_pieces.0 ^ xor);
        undo.p2_pieces_xor ^= xor;
    }
    if pos.kings.contains(src) {
        pos.kings = Bitboard(pos.kings.0 ^ xor);
        undo.kings_xor ^= xor;
    } else if pos.champions.contains(src) {
        pos.champions = Bitboard(pos.champions.0 ^ xor);
        undo.champions_xor ^= xor;
    } else if pos.guards.contains(src) {
        pos.guards = Bitboard(pos.guards.0 ^ xor);
        undo.guards_xor ^= xor;
    }

    // Mark destination as moved-this-phase.
    pos.moved_this_phase = pos.moved_this_phase | Bitboard::from_square(tgt);
}

/// Move-Attack: mover stays put; defender (target, or a redirected Guard via
/// Bodyguard `choice_idx`) takes 1 damage. Armor absorbs first; on Armor=0 the
/// hit removes 1 HP; HP=0 removes the piece from the board.
fn apply_move_attack(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let choice = action.choice_idx();

    debug_assert!(pos.is_occupied(src), "move-attack from empty square");
    debug_assert!(pos.is_occupied(tgt), "move-attack on empty square");

    // Decide which square actually takes the hit.
    let hit_sq = if choice == 0 {
        tgt
    } else {
        let guards = super::generator::bodyguard_guards_for(pos, tgt);
        let k = (choice as usize).checked_sub(1).expect("choice_idx>=1 here");
        debug_assert!(k < guards.len(),
            "Bodyguard choice_idx={} out of range (only {} eligible Guards for target {})",
            choice, guards.len(), tgt);
        guards[k]
    };

    deal_one_damage(pos, hit_sq, undo);

    // Mark the mover's *origin* as moved-this-phase (mover stayed put).
    pos.moved_this_phase = pos.moved_this_phase | Bitboard::from_square(src);
}

/// Deal 1 point of damage to the piece on `hit_sq`. Armor absorbs first;
/// otherwise HP drops by 1; piece is removed from all bitboards if HP hits 0.
fn deal_one_damage(pos: &mut Position, hit_sq: u8, undo: &mut Undo) {
    let prev_entry = pos.mailbox[hit_sq as usize];
    record_affected(undo, hit_sq, prev_entry);

    if prev_entry.armor() > 0 {
        // Armor absorbs the hit — HP unchanged.
        pos.mailbox[hit_sq as usize] = prev_entry.with_armor(prev_entry.armor() - 1);
        return;
    }

    // No armor — HP drops.
    let new_hp = prev_entry.hp().saturating_sub(1);
    if new_hp == 0 {
        // Piece removed. Capture King + owner identity *before* mutating any
        // bitboards so we can set game_result correctly afterwards.
        let was_king  = pos.kings.contains(hit_sq);
        let owned_by_p1 = pos.p1_pieces.contains(hit_sq);

        pos.mailbox[hit_sq as usize] = EMPTY_MAILBOX_ENTRY;
        let bit = Bitboard::from_square(hit_sq).0;
        if owned_by_p1 {
            pos.p1_pieces = Bitboard(pos.p1_pieces.0 ^ bit);
            undo.p1_pieces_xor ^= bit;
        } else {
            pos.p2_pieces = Bitboard(pos.p2_pieces.0 ^ bit);
            undo.p2_pieces_xor ^= bit;
        }
        if was_king {
            pos.kings = Bitboard(pos.kings.0 ^ bit);
            undo.kings_xor ^= bit;
            // Stack M: removing a King ends the game immediately. The other
            // player wins. `unmake` restores the prior `game_result` via
            // the Undo snapshot captured at the start of `make`.
            pos.game_result = Some(if owned_by_p1 {
                GameResult::P2Wins
            } else {
                GameResult::P1Wins
            });
        } else if pos.champions.contains(hit_sq) {
            pos.champions = Bitboard(pos.champions.0 ^ bit);
            undo.champions_xor ^= bit;
        } else if pos.guards.contains(hit_sq) {
            pos.guards = Bitboard(pos.guards.0 ^ bit);
            undo.guards_xor ^= bit;
        }
    } else {
        pos.mailbox[hit_sq as usize] = prev_entry.with_hp(new_hp);
    }
}

// === EndPhase ==============================================================

/// Move → Skill transition (Slice 1 simplification).
///
/// Stack M does not yet specify the Skill-Phase action budget curve
/// (OQ-69, critical). Until that resolves, we reset to a placeholder budget
/// of 2 actions when leaving the Move Phase. End-of-turn (Skill → next turn)
/// is delegated to `turn_manager::end_turn`.
fn apply_end_phase(pos: &mut Position, _undo: &mut Undo) {
    match pos.current_phase {
        Phase::Move => {
            pos.moved_this_phase = Bitboard::EMPTY;
            pos.current_phase = Phase::Skill;
            pos.actions_remaining = 2; // OQ-69: progression curve TBD.
        }
        Phase::Skill => {
            super::turn_manager::end_turn(pos);
        }
    }
}

// === Tiny helpers ===========================================================

fn record_affected(undo: &mut Undo, sq: u8, prev: MailboxEntry) {
    // Dedup: if this square is already recorded, leave the *original* snapshot
    // in place — that's the value we need to restore back to.
    for i in 0..undo.affected_count as usize {
        if undo.affected_squares[i] == sq { return; }
    }
    let i = undo.affected_count as usize;
    debug_assert!(i < undo.affected_squares.len(),
        "affected_squares capacity exceeded — bump size or split action");
    undo.affected_squares[i] = sq;
    undo.affected_prev_entries[i] = prev.0;
    undo.affected_count += 1;
}

fn phase_to_byte(p: Phase) -> u8 {
    match p { Phase::Move => 0, Phase::Skill => 1 }
}
fn phase_from_byte(b: u8) -> Phase {
    match b { 0 => Phase::Move, _ => Phase::Skill }
}

fn game_result_to_tag(r: Option<GameResult>) -> u8 {
    match r {
        None                       => 0,
        Some(GameResult::P1Wins)   => 1,
        Some(GameResult::P2Wins)   => 2,
    }
}
fn game_result_from_tag(t: u8) -> Option<GameResult> {
    match t {
        1 => Some(GameResult::P1Wins),
        2 => Some(GameResult::P2Wins),
        _ => None,
    }
}

#[allow(dead_code)]
fn _unused_player(_p: Player) {} // keep Player in-scope without unused-import warning

// === Tests =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::position::{Phase, Player};

    fn empty_pos_with_actions(actions: u8) -> Position {
        let mut p = Position::empty();
        p.current_phase = Phase::Move;
        p.actions_remaining = actions;
        p.to_move = Player::P1;
        p
    }

    fn place(pos: &mut Position, sq: u8, player: Player, kind: PieceKind, hp: u8, armor: u8) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => pos.p1_pieces = pos.p1_pieces | bit,
            Player::P2 => pos.p2_pieces = pos.p2_pieces | bit,
        }
        match kind {
            PieceKind::King     => pos.kings     = pos.kings     | bit,
            PieceKind::Champion => pos.champions = pos.champions | bit,
            PieceKind::Guard    => pos.guards    = pos.guards    | bit,
        }
        pos.mailbox[sq as usize] = EMPTY_MAILBOX_ENTRY
            .with_hp(hp)
            .with_armor(armor);
    }

    enum PieceKind { King, Champion, Guard }

    // --- Plain Move ------------------------------------------------------

    #[test]
    fn plain_move_relocates_piece_and_marks_dest() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);

        assert!(!pos.is_occupied(0));
        assert!(pos.is_occupied(1));
        assert!(pos.p1_pieces.contains(1));
        assert!(pos.champions.contains(1));
        assert_eq!(pos.mailbox[1].hp(), 2);
        assert!(pos.moved_this_phase.contains(1), "dest must be marked");
        assert_eq!(pos.actions_remaining, 1);

        // Reversible.
        unmake(&mut pos, &undo);
        assert!(pos.is_occupied(0));
        assert!(!pos.is_occupied(1));
        assert!(pos.p1_pieces.contains(0));
        assert!(pos.champions.contains(0));
        assert_eq!(pos.mailbox[0].hp(), 2);
        assert_eq!(pos.moved_this_phase.0, 0);
        assert_eq!(pos.actions_remaining, 2);
    }

    #[test]
    fn plain_move_preserves_armor_and_skill_loadout() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 2);
        pos.mailbox[0] = pos.mailbox[0].with_skill1(3).with_skill2(7);

        let a = Action::encode(0, 9, ActionKind::Move, 0, 0);
        let _ = make(&mut pos, a);

        let e = pos.mailbox[9];
        assert_eq!(e.hp(), 2);
        assert_eq!(e.armor(), 2);
        assert_eq!(e.skill1(), 3);
        assert_eq!(e.skill2(), 7);
    }

    // --- Move-Attack: armor / HP / removal -------------------------------

    #[test]
    fn move_attack_burns_armor_first() {
        // P1 Champion at 0 attacks P2 Guard at 1 (Armor=1).
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 1);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);

        assert!(pos.is_occupied(1), "defender still on board");
        assert_eq!(pos.mailbox[1].armor(), 0, "armor consumed");
        assert_eq!(pos.mailbox[1].hp(), 2, "hp unchanged");
        assert!(pos.is_occupied(0), "attacker stays put");
        assert!(pos.moved_this_phase.contains(0), "src marked, not dest");
        assert!(!pos.moved_this_phase.contains(1));
        assert_eq!(pos.actions_remaining, 1);

        unmake(&mut pos, &undo);
        assert_eq!(pos.mailbox[1].armor(), 1);
        assert_eq!(pos.moved_this_phase.0, 0);
        assert_eq!(pos.actions_remaining, 2);
    }

    #[test]
    fn move_attack_no_armor_drops_hp() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 0);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);

        assert!(pos.is_occupied(1));
        assert_eq!(pos.mailbox[1].hp(), 1);
        assert_eq!(pos.mailbox[1].armor(), 0);

        unmake(&mut pos, &undo);
        assert_eq!(pos.mailbox[1].hp(), 2);
    }

    #[test]
    fn move_attack_kills_injured_target() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 1, 0);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);

        assert!(!pos.is_occupied(1), "guard removed");
        assert!(!pos.p2_pieces.contains(1));
        assert!(!pos.guards.contains(1));
        assert_eq!(pos.mailbox[1].0, 0, "mailbox cleared on removal");

        unmake(&mut pos, &undo);
        assert!(pos.is_occupied(1));
        assert!(pos.p2_pieces.contains(1));
        assert!(pos.guards.contains(1));
        assert_eq!(pos.mailbox[1].hp(), 1);
    }

    // --- Bodyguard ------------------------------------------------------

    #[test]
    fn bodyguard_redirects_damage_to_chosen_guard() {
        // P1 Champion at 0. P2 Champion at 9 (b2). Adjacent P2 Guards at 1, 8, 10.
        // Generator emits choice_idx 0 (no redirect) + 1..=3 (per Guard, sorted asc).
        // Sorted Guards (ascending sq): [1, 8, 10] → choice_idx 1=sq1, 2=sq8, 3=sq10.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 9, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 0);
        place(&mut pos, 8, Player::P2, PieceKind::Guard, 2, 0);
        place(&mut pos, 10, Player::P2, PieceKind::Guard, 2, 0);

        // choice_idx = 2 → guard at sq 8 takes the hit.
        let a = Action::encode(0, 9, ActionKind::Move, 0, 2);
        let undo = make(&mut pos, a);

        // Champion at 9 untouched.
        assert_eq!(pos.mailbox[9].hp(), 2);
        // Guard at sq 8: HP dropped from 2 → 1.
        assert_eq!(pos.mailbox[8].hp(), 1);
        // Other guards untouched.
        assert_eq!(pos.mailbox[1].hp(), 2);
        assert_eq!(pos.mailbox[10].hp(), 2);
        // Attacker stays put; src marked.
        assert!(pos.is_occupied(0));
        assert!(pos.moved_this_phase.contains(0));

        unmake(&mut pos, &undo);
        assert_eq!(pos.mailbox[8].hp(), 2);
        assert_eq!(pos.moved_this_phase.0, 0);
    }

    #[test]
    fn bodyguard_no_redirect_hits_named_target() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 9, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 0);

        let a = Action::encode(0, 9, ActionKind::Move, 0, 0);
        let _undo = make(&mut pos, a);

        assert_eq!(pos.mailbox[9].hp(), 1, "named champion takes the hit");
        assert_eq!(pos.mailbox[1].hp(), 2, "guard untouched on choice=0");
    }

    // --- EndPhase + reversibility ---------------------------------------

    #[test]
    fn end_phase_transitions_move_to_skill_and_clears_moved() {
        let mut pos = empty_pos_with_actions(1);
        pos.moved_this_phase = Bitboard::from_square(12) | Bitboard::from_square(20);

        let a = Action::encode(0, 0, ActionKind::EndPhase, 0, 0);
        let undo = make(&mut pos, a);

        assert!(matches!(pos.current_phase, Phase::Skill));
        assert_eq!(pos.moved_this_phase.0, 0);
        assert_eq!(pos.actions_remaining, 2);

        unmake(&mut pos, &undo);
        assert!(matches!(pos.current_phase, Phase::Move));
        assert_eq!(pos.actions_remaining, 1);
        assert!(pos.moved_this_phase.contains(12));
        assert!(pos.moved_this_phase.contains(20));
    }

    // --- Make/Unmake round-trip: equal position --------------------------

    #[test]
    fn make_unmake_roundtrip_plain_move() {
        let mut pos = Position::setup_stack_m();
        let before = pos.to_fen();

        // P1 Guard at b2 (sq 9) → b3 (sq 17).
        let a = Action::encode(9, 17, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);
        assert_ne!(pos.to_fen(), before, "make actually changed something");
        unmake(&mut pos, &undo);
        assert_eq!(pos.to_fen(), before, "unmake restored exact FEN");
    }

    #[test]
    fn make_unmake_roundtrip_move_attack() {
        // Hand-built: a Champion next to an enemy Champion, Move-Attack.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 1);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 1);
        let before = pos.to_fen();

        let a = Action::encode(28, 36, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);
        unmake(&mut pos, &undo);
        assert_eq!(pos.to_fen(), before);
    }

    #[test]
    fn make_unmake_roundtrip_move_attack_killing_blow() {
        // Removal must round-trip exactly.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 36, Player::P2, PieceKind::Guard, 1, 0);
        let before = pos.to_fen();

        let a = Action::encode(28, 36, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);
        assert!(!pos.is_occupied(36));
        unmake(&mut pos, &undo);
        assert_eq!(pos.to_fen(), before);
    }

    // --- Slice 2: King-capture = game over ------------------------------

    #[test]
    fn move_attack_on_king_hp1_ends_game() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::King, 1, 0);
        assert_eq!(pos.game_result, None);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);

        assert!(!pos.is_occupied(1), "King removed");
        assert!(!pos.kings.contains(1));
        assert!(!pos.p2_pieces.contains(1));
        assert_eq!(pos.game_result, Some(GameResult::P1Wins));

        unmake(&mut pos, &undo);
        assert!(pos.kings.contains(1));
        assert_eq!(pos.mailbox[1].hp(), 1);
        assert_eq!(pos.game_result, None);
    }

    #[test]
    fn move_attack_on_king_with_armor_does_not_end_game() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::King, 2, 2);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let _ = make(&mut pos, a);

        assert_eq!(pos.mailbox[1].armor(), 1, "armor 2→1");
        assert_eq!(pos.mailbox[1].hp(), 2);
        assert!(pos.kings.contains(1));
        assert_eq!(pos.game_result, None);
    }

    #[test]
    fn bodyguard_can_protect_king_from_lethal_blow() {
        // P2 King at sq 1, HP=1, Armor=0. P2 Guard at sq 2 (adjacent), HP=2.
        // P1 Champion at sq 0 Move-Attacks with choice_idx=1 → Guard absorbs.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::King, 1, 0);
        place(&mut pos, 2, Player::P2, PieceKind::Guard, 2, 0);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 1);
        let undo = make(&mut pos, a);

        assert!(pos.kings.contains(1), "King survives — Bodyguard absorbed");
        assert_eq!(pos.mailbox[1].hp(), 1);
        assert_eq!(pos.mailbox[2].hp(), 1, "Guard HP 2→1");
        assert_eq!(pos.game_result, None);

        unmake(&mut pos, &undo);
        assert_eq!(pos.mailbox[2].hp(), 2);
        assert_eq!(pos.game_result, None);
    }

    #[test]
    fn make_unmake_roundtrip_king_capture() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::King, 1, 0);
        // FEN serialises position; game_result is derivable, not stored, but
        // from_fen recomputes it deterministically. So a pre-capture FEN
        // round-trips with game_result=None on both sides of unmake.
        let before = pos.to_fen();

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let undo = make(&mut pos, a);
        assert_eq!(pos.game_result, Some(GameResult::P1Wins));

        unmake(&mut pos, &undo);
        assert_eq!(pos.to_fen(), before);
        assert_eq!(pos.game_result, None);
    }

    // --- Slice 2: Bodyguard edge cases (mostly generator-side) ----------

    #[test]
    fn move_attack_on_guard_offers_no_bodyguard_choice() {
        // Generator-side: Move-Attack on a Guard must not enumerate any
        // Bodyguard redirect — Bodyguard protects only Champion/King.
        // Setup: P1 Champion at sq 0, P2 Guards at sq 1 (target) and sq 2
        // (would-be protector). Generator should emit exactly one Move-Attack
        // action against sq 1 (choice_idx = 0).
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 0);
        place(&mut pos, 2, Player::P2, PieceKind::Guard, 2, 0);
        pos.to_move = Player::P1;

        let actions = super::super::generator::generate(&pos);
        let attacks_on_1: Vec<_> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Move
                     && a.src() == 0
                     && a.target() == 1)
            .collect();
        assert_eq!(attacks_on_1.len(), 1, "exactly one Move-Attack on guard, no redirect");
        assert_eq!(attacks_on_1[0].choice_idx(), 0);
    }

    #[test]
    fn move_attack_with_no_adjacent_friendly_guards_offers_no_redirect() {
        // P1 Champion at sq 0, P2 Champion at sq 1, no adjacent P2 Guards.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Champion, 2, 0);
        pos.to_move = Player::P1;

        let actions = super::super::generator::generate(&pos);
        let attacks_on_1: Vec<_> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Move
                     && a.src() == 0
                     && a.target() == 1)
            .collect();
        assert_eq!(attacks_on_1.len(), 1);
        assert_eq!(attacks_on_1[0].choice_idx(), 0);
    }

    #[test]
    fn move_attack_with_three_adjacent_guards_emits_four_variants() {
        // P2 Champion at sq 9 (b2) with P2 Guards at sq 1, 8, 10 (b1, a2, c2).
        // Sorted ascending: 1, 8, 10. Generator emits choice 0 (no redirect)
        // plus 1..=3 mapped onto [1, 8, 10] in ascending order.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 9, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::Guard, 2, 0);
        place(&mut pos, 8, Player::P2, PieceKind::Guard, 2, 0);
        place(&mut pos, 10, Player::P2, PieceKind::Guard, 2, 0);
        pos.to_move = Player::P1;

        let actions = super::super::generator::generate(&pos);
        let mut attacks_on_9: Vec<_> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Move
                     && a.src() == 0
                     && a.target() == 9)
            .map(|a| a.choice_idx())
            .collect();
        attacks_on_9.sort_unstable();
        assert_eq!(attacks_on_9, vec![0, 1, 2, 3]);

        // Apply each redirect and confirm the right Guard takes the hit.
        for (choice, guard_sq) in [(1u8, 1u8), (2, 8), (3, 10)] {
            let mut p = pos.clone();
            let a = Action::encode(0, 9, ActionKind::Move, 0, choice);
            let _ = make(&mut p, a);
            assert_eq!(p.mailbox[guard_sq as usize].hp(), 1,
                "choice {} should hit guard at sq {}", choice, guard_sq);
            // The other two Guards untouched.
            for other in [1u8, 8, 10].iter().copied().filter(|&s| s != guard_sq) {
                assert_eq!(p.mailbox[other as usize].hp(), 2,
                    "guard at {} untouched when choice {} redirects to {}",
                    other, choice, guard_sq);
            }
        }
    }

    #[test]
    fn bodyguard_choice_zero_against_armored_king_burns_armor() {
        // Sanity check: choice 0 with an armored King keeps the standard
        // Armor→HP resolution path; King survives, game continues.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 1, Player::P2, PieceKind::King, 2, 1);
        place(&mut pos, 2, Player::P2, PieceKind::Guard, 2, 0);

        let a = Action::encode(0, 1, ActionKind::Move, 0, 0);
        let _ = make(&mut pos, a);

        assert_eq!(pos.mailbox[1].armor(), 0, "King armor consumed");
        assert_eq!(pos.mailbox[1].hp(), 2);
        assert!(pos.kings.contains(1));
        assert_eq!(pos.mailbox[2].hp(), 2, "Guard untouched on choice 0");
        assert_eq!(pos.game_result, None);
    }

    // --- Slice 2: Move-Phase integration --------------------------------

    #[test]
    fn move_phase_full_two_actions_then_endphase_roundtrips() {
        // Plain Move (Guard b2→b4) + Move-Attack (Champion d1→d2 onto P2 Guard)
        // + EndPhase. After all three: Skill phase, actions=2, moved_this_phase=0.
        // Then unmake each in reverse and assert the starting FEN.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 9, Player::P1, PieceKind::Guard, 2, 0);     // b2
        place(&mut pos, 3, Player::P1, PieceKind::Champion, 2, 0);  // d1
        place(&mut pos, 11, Player::P2, PieceKind::Guard, 2, 0);    // d2
        // Kings need to exist for game_result invariants — give each side
        // an inert King far from the action.
        place(&mut pos, 56, Player::P1, PieceKind::King, 2, 0);     // a8
        place(&mut pos, 63, Player::P2, PieceKind::King, 2, 0);     // h8
        let start = pos.to_fen();

        let move1 = Action::encode(9, 25, ActionKind::Move, 0, 0);  // Guard b2 → b4
        let move2 = Action::encode(3, 11, ActionKind::Move, 0, 0);  // Champion d1 Move-Attack onto d2
        let end   = Action::encode(0, 0, ActionKind::EndPhase, 0, 0);

        let u1 = make(&mut pos, move1);
        let u2 = make(&mut pos, move2);
        let u3 = make(&mut pos, end);

        assert!(matches!(pos.current_phase, Phase::Skill));
        assert_eq!(pos.actions_remaining, 2);
        assert_eq!(pos.moved_this_phase.0, 0);
        // Guard at sq 11 was Move-Attacked: HP 2 → 1.
        assert_eq!(pos.mailbox[11].hp(), 1);
        // Guard at b4 (sq 25) sits where b2 used to.
        assert!(pos.is_occupied(25));
        assert!(!pos.is_occupied(9));

        // Reverse in opposite order.
        unmake(&mut pos, &u3);
        unmake(&mut pos, &u2);
        unmake(&mut pos, &u1);
        assert_eq!(pos.to_fen(), start);
    }

    // --- Slice 2: Generator filter on game-over -------------------------

    #[test]
    fn no_legal_actions_after_game_over() {
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 56, Player::P1, PieceKind::King, 2, 0);
        // No P2 King → recompute_game_result sets P1Wins.
        pos.recompute_game_result();
        assert_eq!(pos.game_result, Some(GameResult::P1Wins));

        let actions = super::super::generator::generate(&pos);
        assert!(actions.is_empty(), "no legal actions after game over");
    }

    // --- Slice 2: FEN parser invariant + recompute helper ---------------

    #[test]
    fn recompute_game_result_handles_each_case() {
        // Both Kings present → None.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::King, 2, 0);
        place(&mut pos, 56, Player::P2, PieceKind::King, 2, 0);
        pos.recompute_game_result();
        assert_eq!(pos.game_result, None);

        // P2 King missing → P1Wins.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 0, Player::P1, PieceKind::King, 2, 0);
        pos.recompute_game_result();
        assert_eq!(pos.game_result, Some(GameResult::P1Wins));

        // P1 King missing → P2Wins.
        let mut pos = empty_pos_with_actions(2);
        place(&mut pos, 56, Player::P2, PieceKind::King, 2, 0);
        pos.recompute_game_result();
        assert_eq!(pos.game_result, Some(GameResult::P2Wins));
    }

    #[test]
    fn from_fen_recomputes_game_result_for_normal_position() {
        // Both Kings present in the canonical setup → game_result stays None
        // after a FEN roundtrip (the recompute helper is invoked, and it
        // returns None for a non-terminal position).
        let pos = Position::setup_stack_m();
        let parsed = Position::from_fen(&pos.to_fen()).expect("setup roundtrips");
        assert_eq!(parsed.game_result, None);
    }
}
