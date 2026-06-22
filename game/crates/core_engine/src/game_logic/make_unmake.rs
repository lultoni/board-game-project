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
use crate::state::position::{Phase, Player};
use crate::state::{Bitboard, MailboxEntry, Position, EMPTY_MAILBOX_ENTRY};

pub fn make(pos: &mut Position, action: Action) -> Undo {
    let mut undo = Undo {
        action: action.0,
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
        // Piece removed. Clear mailbox + bitboards.
        pos.mailbox[hit_sq as usize] = EMPTY_MAILBOX_ENTRY;
        let bit = Bitboard::from_square(hit_sq).0;
        if pos.p1_pieces.contains(hit_sq) {
            pos.p1_pieces = Bitboard(pos.p1_pieces.0 ^ bit);
            undo.p1_pieces_xor ^= bit;
        } else {
            pos.p2_pieces = Bitboard(pos.p2_pieces.0 ^ bit);
            undo.p2_pieces_xor ^= bit;
        }
        if pos.kings.contains(hit_sq) {
            pos.kings = Bitboard(pos.kings.0 ^ bit);
            undo.kings_xor ^= bit;
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
            PieceKind::Champion => pos.champions = pos.champions | bit,
            PieceKind::Guard    => pos.guards    = pos.guards    | bit,
        }
        pos.mailbox[sq as usize] = EMPTY_MAILBOX_ENTRY
            .with_hp(hp)
            .with_armor(armor);
    }

    enum PieceKind { Champion, Guard }

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
}
