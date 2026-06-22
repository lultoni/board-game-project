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
use crate::state::position::{modifier_bits, GameResult, Phase, Player};
use crate::state::{magic, Bitboard, MailboxEntry, Position, EMPTY_MAILBOX_ENTRY};

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
        prev_tracked_casters: pos.tracked_casters,
        prev_tracked_casters_len: pos.tracked_casters_len,
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
        ActionKind::Skill     => apply_skill(pos, action, &mut undo),
        ActionKind::EndPhase  => apply_end_phase(pos, &mut undo),
        ActionKind::EndTurn   => super::turn_manager::end_turn(pos),
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
    pos.tracked_casters    = undo.prev_tracked_casters;
    pos.tracked_casters_len = undo.prev_tracked_casters_len;
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

// === Skill-kind dispatch ===================================================

/// Apply a Skill-kind action. Slice 4 wires the five Strike-skill resolvers
/// (Lance, Break, Steal, Hook, Tempest). The other 10 still panic until
/// their slices land.
fn apply_skill(pos: &mut Position, action: Action, undo: &mut Undo) {
    debug_assert!(pos.actions_remaining > 0, "make() invoked with zero actions");
    let skill = super::skills::skill_from_id(action.skill_id())
        .expect("generator emitted unknown skill id");
    use super::skills::Skill;
    match skill {
        Skill::Lance   => apply_lance(pos, action, undo),
        Skill::Break   => apply_break(pos, action, undo),
        Skill::Steal   => apply_steal(pos, action, undo),
        Skill::Hook    => apply_hook(pos, action, undo),
        Skill::Tempest => apply_tempest(pos, action, undo),
        Skill::Shield
        | Skill::Heal
        | Skill::Plate
        | Skill::Dash
        | Skill::Blast
        | Skill::Shove
        | Skill::Swap
        | Skill::Retreat
        | Skill::Focus
        | Skill::Charge => {
            unimplemented!("Skill::{:?} resolver lands in Slice 5+", skill);
        }
    }
}

// === Strike-skill resolvers (Slice 4) ======================================

fn apply_lance(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    apply_strike_damage(pos, src, tgt, /*base=*/ 1, undo);
    debit_money(pos, src, /*cost=*/ 2, undo);
    pos.actions_remaining -= 1;
}

fn apply_break(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let prev = pos.mailbox[tgt as usize];
    record_affected(undo, tgt, prev);

    let charge_active = pos.pending_modifiers & modifier_bits::CHARGE != 0;
    if charge_active { pos.pending_modifiers &= !modifier_bits::CHARGE; }
    let existing_combo = prev.combo();

    // Tick first (it modifies the mailbox entry's combo field).
    let _ = combo_tick(pos, src, tgt, undo);

    // Armor reduction — applies regardless of HP-damage gating. Read the
    // mailbox AGAIN because combo_tick may have written a new entry.
    let post_tick = pos.mailbox[tgt as usize];
    let new_armor = post_tick.armor().saturating_sub(1);
    pos.mailbox[tgt as usize] = post_tick.with_armor(new_armor);

    // HP-damage gate: Stack-M says Break "does not deal HP-Damage unless
    // boosted by Charge." But the universal combo bonus ("any skill that
    // affects a target with counter > 0 deals +counter damage") still
    // applies on top.
    let dmg = (if charge_active { 1u8 } else { 0 }) + existing_combo;
    if dmg > 0 { deal_damage(pos, tgt, dmg, undo); }

    debit_money(pos, src, /*cost=*/ 2, undo);
    pos.actions_remaining -= 1;
}

fn apply_steal(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    apply_strike_damage(pos, src, tgt, /*base=*/ 1, undo);
    let (from_p, to_p) = if pos.p1_pieces.contains(src) {
        (Player::P2, Player::P1)
    } else {
        (Player::P1, Player::P2)
    };
    transfer_money(pos, from_p, to_p, /*amount=*/ 1, undo);
    debit_money(pos, src, /*cost=*/ 4, undo);
    pos.actions_remaining -= 1;
}

fn apply_hook(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    apply_strike_damage(pos, src, tgt, /*base=*/ 1, undo);
    // Pull only if target survived (still occupies the square).
    if pos.is_occupied(tgt) {
        if let Some(pull_dest) = magic::step_toward(tgt, src) {
            if pull_dest != src && !pos.is_occupied(pull_dest) {
                relocate_piece(pos, tgt, pull_dest, undo);
            }
        }
    }
    debit_money(pos, src, /*cost=*/ 3, undo);
    pos.actions_remaining -= 1;
}

fn apply_tempest(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    apply_strike_damage(pos, src, tgt, /*base=*/ 1, undo);

    // AOE push: target square is the pivot. Iterate neighbours in ascending
    // square index for deterministic resolution.
    let mut neighbours: [u8; 8] = [0; 8];
    let mut n_count = 0usize;
    for n in super::generator::eight_neighbours(tgt) {
        neighbours[n_count] = n;
        n_count += 1;
    }
    let slice = &mut neighbours[..n_count];
    slice.sort_unstable();

    for &n in slice.iter() {
        if n == src { continue; }                       // Caster not affected.
        if !pos.is_occupied(n) { continue; }
        let Some(push_dest) = magic::step_away(tgt, n) else { continue }; // off-board
        if pos.is_occupied(push_dest) { continue; }
        // Strict reading: each pushed piece is "affected" by a movement-causing
        // skill from a new Champion → tick combo.
        let _ = combo_tick(pos, src, n, undo);
        relocate_piece(pos, n, push_dest, undo);
    }

    debit_money(pos, src, /*cost=*/ 4, undo);
    pos.actions_remaining -= 1;
}

// === Strike-skill helpers ==================================================

/// Resolve a Strike effect on `tgt_sq`: consume Charge if pending, read
/// existing combo counter, tick combo (gated by caster identity), deal
/// `base + existing_combo + charge_bonus` damage through the standard
/// Armor → HP → removal pipeline. Returns the total damage scheduled.
fn apply_strike_damage(pos: &mut Position, src_sq: u8, tgt_sq: u8,
                       base: u8, undo: &mut Undo) -> u8 {
    let prev = pos.mailbox[tgt_sq as usize];
    record_affected(undo, tgt_sq, prev);

    let charge_bonus = if pos.pending_modifiers & modifier_bits::CHARGE != 0 {
        pos.pending_modifiers &= !modifier_bits::CHARGE;
        1u8
    } else {
        0
    };
    let existing_combo = prev.combo();

    // Tick BEFORE damage so the post-state reflects the combo bump even if
    // the piece is removed (the mailbox slot is overwritten on removal, but
    // the `record_affected` snapshot we just took is what restores on
    // unmake). The bonus damage uses the *pre-tick* counter — Stack-M:
    // "+counter damage to that target" using the counter the target had
    // when the skill landed.
    let _ = combo_tick(pos, src_sq, tgt_sq, undo);

    let total = base + existing_combo + charge_bonus;
    deal_damage(pos, tgt_sq, total, undo);
    total
}

/// Deal `dmg` points of damage to the piece on `hit_sq`. Loops the existing
/// 1-damage pipeline; stops early if the piece is removed. `record_affected`
/// dedups the mailbox snapshot, so the Undo only carries one entry per square.
fn deal_damage(pos: &mut Position, hit_sq: u8, dmg: u8, undo: &mut Undo) {
    let mut remaining = dmg;
    while remaining > 0 && pos.is_occupied(hit_sq) {
        deal_one_damage(pos, hit_sq, undo);
        remaining -= 1;
    }
}

/// Combo-tick `tgt_sq` IF `src_sq` (caster) hasn't already ticked this
/// target this turn. Allocates new slots in `tracked_casters` /
/// `tracked_enemies` as needed. Increments target's combo counter (clamped
/// to 7) and writes a new mailbox entry. Records the pre-tick mailbox in
/// `undo` (dedup-safe — no-op if the caller already snapshot'd this square).
/// Returns true iff a tick happened.
fn combo_tick(pos: &mut Position, src_sq: u8, tgt_sq: u8, undo: &mut Undo) -> bool {
    use crate::state::position::MAX_TRACKED_ENEMIES;
    let caster_slot = ensure_tracked_caster(pos, src_sq) as u64;
    let target_slot = ensure_tracked_enemy(pos, tgt_sq) as u64;
    let bit = 1u64 << (caster_slot * MAX_TRACKED_ENEMIES as u64 + target_slot);
    if pos.champion_credit & bit != 0 { return false; }
    pos.champion_credit |= bit;

    let prev = pos.mailbox[tgt_sq as usize];
    record_affected(undo, tgt_sq, prev);
    let new_combo = (prev.combo() + 1).min(7);
    pos.mailbox[tgt_sq as usize] = prev.with_combo(new_combo);
    true
}

fn ensure_tracked_enemy(pos: &mut Position, sq: u8) -> u8 {
    use crate::state::position::MAX_TRACKED_ENEMIES;
    for i in 0..pos.tracked_enemies_len as usize {
        if pos.tracked_enemies[i] == sq { return i as u8; }
    }
    let i = pos.tracked_enemies_len;
    debug_assert!((i as usize) < MAX_TRACKED_ENEMIES,
                  "tracked_enemies capacity exhausted in single turn");
    pos.tracked_enemies[i as usize] = sq;
    pos.tracked_enemies_len += 1;
    i
}

fn ensure_tracked_caster(pos: &mut Position, sq: u8) -> u8 {
    use crate::state::position::MAX_TRACKED_CASTERS;
    for i in 0..pos.tracked_casters_len as usize {
        if pos.tracked_casters[i] == sq { return i as u8; }
    }
    let i = pos.tracked_casters_len;
    debug_assert!((i as usize) < MAX_TRACKED_CASTERS,
                  "tracked_casters capacity exhausted in single turn");
    pos.tracked_casters[i as usize] = sq;
    pos.tracked_casters_len += 1;
    i
}

/// Move a piece from `from` to `to`. Mailbox copy, bitboard XOR across every
/// layer the piece sits on. Caller guarantees `from` is occupied and `to`
/// is empty.
fn relocate_piece(pos: &mut Position, from: u8, to: u8, undo: &mut Undo) {
    debug_assert!(pos.is_occupied(from));
    debug_assert!(!pos.is_occupied(to));

    let prev_from = pos.mailbox[from as usize];
    let prev_to   = pos.mailbox[to as usize];
    record_affected(undo, from, prev_from);
    record_affected(undo, to,   prev_to);

    pos.mailbox[from as usize] = EMPTY_MAILBOX_ENTRY;
    pos.mailbox[to as usize]   = prev_from;

    let xor = Bitboard::from_square(from).0 | Bitboard::from_square(to).0;
    if pos.p1_pieces.contains(from) {
        pos.p1_pieces = Bitboard(pos.p1_pieces.0 ^ xor);
        undo.p1_pieces_xor ^= xor;
    } else {
        pos.p2_pieces = Bitboard(pos.p2_pieces.0 ^ xor);
        undo.p2_pieces_xor ^= xor;
    }
    if pos.kings.contains(from) {
        pos.kings = Bitboard(pos.kings.0 ^ xor);
        undo.kings_xor ^= xor;
    } else if pos.champions.contains(from) {
        pos.champions = Bitboard(pos.champions.0 ^ xor);
        undo.champions_xor ^= xor;
    } else if pos.guards.contains(from) {
        pos.guards = Bitboard(pos.guards.0 ^ xor);
        undo.guards_xor ^= xor;
    }
}

/// Debit `cost` Money from the side that owns `caster_sq`.
fn debit_money(pos: &mut Position, caster_sq: u8, cost: u8, undo: &mut Undo) {
    let cost_u16 = cost as u16;
    let cost_i16 = cost as i16;
    if pos.p1_pieces.contains(caster_sq) {
        pos.p1_money = pos.p1_money.saturating_sub(cost_u16);
        undo.p1_money_delta -= cost_i16;
    } else {
        pos.p2_money = pos.p2_money.saturating_sub(cost_u16);
        undo.p2_money_delta -= cost_i16;
    }
}

/// Transfer up to `amount` Money from `from` to `to`, clamped to `from`'s
/// actual money pool. Used by Steal.
fn transfer_money(pos: &mut Position, from: Player, to: Player,
                  amount: u16, undo: &mut Undo) {
    let actual = amount.min(match from {
        Player::P1 => pos.p1_money,
        Player::P2 => pos.p2_money,
    });
    if actual == 0 { return; }
    let actual_i16 = actual as i16;
    match from {
        Player::P1 => { pos.p1_money -= actual; undo.p1_money_delta -= actual_i16; }
        Player::P2 => { pos.p2_money -= actual; undo.p2_money_delta -= actual_i16; }
    }
    match to {
        Player::P1 => { pos.p1_money += actual; undo.p1_money_delta += actual_i16; }
        Player::P2 => { pos.p2_money += actual; undo.p2_money_delta += actual_i16; }
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

    // === Slice 4 — Strike-skill resolvers ===================================

    use super::super::skills::Skill;

    fn skill_phase_pos(actions: u8) -> Position {
        let mut p = Position::empty();
        p.current_phase = Phase::Skill;
        p.actions_remaining = actions;
        p.to_move = Player::P1;
        p.p1_money = 10;
        p.p2_money = 10;
        p
    }

    fn equip(pos: &mut Position, sq: u8, skill_id: u8) {
        pos.mailbox[sq as usize] = pos.mailbox[sq as usize].with_skill1(skill_id);
    }

    fn skill_action(src: u8, tgt: u8, skill: Skill) -> Action {
        Action::encode(src, tgt, ActionKind::Skill, skill as u8, 0)
    }

    fn fen_snapshot(pos: &Position) -> String {
        // Skill-phase positions with mid-cast tracking state aren't valid FEN
        // (FEN is between-turn). For roundtrip-equality, compare structural
        // state via the `position_eq_for_fen` helper instead. Provided here
        // as a no-op placeholder if a test wants `format!` debug output.
        format!("{:?}", (pos.p1_pieces.0, pos.p2_pieces.0, pos.kings.0,
                        pos.champions.0, pos.guards.0,
                        pos.p1_money, pos.p2_money))
    }

    /// Deep-equal positions ignoring zobrist (still 0 in Slice 4).
    fn pos_eq(a: &Position, b: &Position) -> bool {
        pos_diff(a, b).is_none()
    }

    /// Returns a human-readable diff string if the two positions differ, else None.
    fn pos_diff(a: &Position, b: &Position) -> Option<String> {
        if a.p1_pieces.0 != b.p1_pieces.0 { return Some(format!("p1_pieces: {:#x} vs {:#x}", a.p1_pieces.0, b.p1_pieces.0)); }
        if a.p2_pieces.0 != b.p2_pieces.0 { return Some(format!("p2_pieces: {:#x} vs {:#x}", a.p2_pieces.0, b.p2_pieces.0)); }
        if a.kings.0     != b.kings.0     { return Some("kings".into()); }
        if a.champions.0 != b.champions.0 { return Some("champions".into()); }
        if a.guards.0    != b.guards.0    { return Some("guards".into()); }
        if a.p1_money != b.p1_money { return Some(format!("p1_money: {} vs {}", a.p1_money, b.p1_money)); }
        if a.p2_money != b.p2_money { return Some(format!("p2_money: {} vs {}", a.p2_money, b.p2_money)); }
        if a.actions_remaining != b.actions_remaining { return Some(format!("actions_remaining: {} vs {}", a.actions_remaining, b.actions_remaining)); }
        if a.pending_modifiers != b.pending_modifiers { return Some("pending_modifiers".into()); }
        if a.to_move != b.to_move { return Some("to_move".into()); }
        if a.current_phase != b.current_phase { return Some("current_phase".into()); }
        if a.round_number != b.round_number { return Some("round_number".into()); }
        if a.moved_this_phase.0 != b.moved_this_phase.0 { return Some("moved_this_phase".into()); }
        if a.tracked_enemies_len != b.tracked_enemies_len { return Some(format!("tracked_enemies_len: {} vs {}", a.tracked_enemies_len, b.tracked_enemies_len)); }
        if a.tracked_casters_len != b.tracked_casters_len { return Some(format!("tracked_casters_len: {} vs {}", a.tracked_casters_len, b.tracked_casters_len)); }
        if a.champion_credit != b.champion_credit { return Some(format!("champion_credit: {:#x} vs {:#x}", a.champion_credit, b.champion_credit)); }
        if a.game_result != b.game_result { return Some("game_result".into()); }
        let occ = a.p1_pieces.0 | a.p2_pieces.0;
        for sq in 0u8..64 {
            if occ & (1u64 << sq) != 0 && a.mailbox[sq as usize].0 != b.mailbox[sq as usize].0 {
                return Some(format!("mailbox[{}]: {:#x} vs {:#x}", sq, a.mailbox[sq as usize].0, b.mailbox[sq as usize].0));
            }
        }
        None
    }

    // --- Lance (5) ---------------------------------------------------------

    #[test]
    fn lance_deals_1_damage_then_unmake_restores() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.mailbox[36].hp(), 1);
        assert!(pos.is_occupied(36));

        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos), "snap before vs after roundtrip");
    }

    #[test]
    fn lance_kills_injured_target() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 1, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert!(!pos.is_occupied(36), "target removed at HP 0");
        assert!(!pos.p2_pieces.contains(36));
        assert!(!pos.champions.contains(36));

        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos));
    }

    #[test]
    fn lance_armor_absorbs() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 1);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.mailbox[36].armor(), 0);
        assert_eq!(pos.mailbox[36].hp(), 2);
    }

    #[test]
    fn lance_costs_2_money() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 10;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.p1_money, 8);
        unmake(&mut pos, &undo);
        assert_eq!(pos.p1_money, 10);
    }

    #[test]
    fn lance_decrements_actions_remaining() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.actions_remaining, 1);
        unmake(&mut pos, &undo);
        assert_eq!(pos.actions_remaining, 2);
    }

    // --- Break (4) ---------------------------------------------------------

    #[test]
    fn break_removes_one_armor_no_hp_damage() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Break as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Break));
        assert_eq!(pos.mailbox[36].armor(), 1);
        assert_eq!(pos.mailbox[36].hp(), 2);
    }

    #[test]
    fn break_at_zero_armor_no_hp_change_no_charge() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Break as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Break));
        assert_eq!(pos.mailbox[36].armor(), 0);
        assert_eq!(pos.mailbox[36].hp(), 2, "no Charge, no combo → no HP damage");
        assert!(pos.is_occupied(36));
    }

    #[test]
    fn break_with_charge_deals_1_hp_damage() {
        let mut pos = skill_phase_pos(2);
        pos.pending_modifiers |= modifier_bits::CHARGE;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Break as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Break));
        assert_eq!(pos.mailbox[36].hp(), 1);
        assert_eq!(pos.pending_modifiers & modifier_bits::CHARGE, 0,
                   "Charge consumed");
    }

    #[test]
    fn break_costs_2_money_and_consumes_action() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 10;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Break as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 1);

        let undo = make(&mut pos, skill_action(28, 36, Skill::Break));
        assert_eq!(pos.p1_money, 8);
        assert_eq!(pos.actions_remaining, 1);
        unmake(&mut pos, &undo);
        assert_eq!(pos.p1_money, 10);
        assert_eq!(pos.actions_remaining, 2);
    }

    // --- Steal (3) --------------------------------------------------------

    #[test]
    fn steal_deals_1_damage_and_transfers_money() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 10;
        pos.p2_money = 6;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Steal as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Steal));
        assert_eq!(pos.p1_money, 10 - 4 + 1, "cost 4, +1 stolen");
        assert_eq!(pos.p2_money, 6 - 1);
        assert_eq!(pos.mailbox[36].hp(), 1);
    }

    #[test]
    fn steal_when_opponent_has_zero_money_only_deals_damage() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 10;
        pos.p2_money = 0;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Steal as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Steal));
        assert_eq!(pos.p1_money, 6, "cost 4, nothing stolen");
        assert_eq!(pos.p2_money, 0);
        assert_eq!(pos.mailbox[36].hp(), 1);
    }

    #[test]
    fn steal_unmake_restores_both_money_pools() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 10;
        pos.p2_money = 6;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Steal as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Steal));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos));
    }

    // --- Hook (5) ---------------------------------------------------------

    #[test]
    fn hook_pulls_target_one_tile_toward_caster() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0); // e4
        equip(&mut pos, 28, Skill::Hook as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0); // e6
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 44, Skill::Hook));
        assert!(!pos.is_occupied(44), "target moved out of e6");
        assert!(pos.is_occupied(36), "target now at e5");
        assert_eq!(pos.mailbox[36].hp(), 1);

        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos), "hook unmake roundtrip");
    }

    #[test]
    fn hook_kills_target_no_pull() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Hook as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 1, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 44, Skill::Hook));
        assert!(!pos.is_occupied(44));
        assert!(!pos.is_occupied(36), "no pull because target removed");

        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos));
    }

    #[test]
    fn hook_pull_blocked_caster_no_relocation() {
        // Adjacent: caster at e4 (28), target at e5 (36). step_toward(36, 28) = 28
        // (the caster itself). Pull skipped.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Hook as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Hook));
        assert!(pos.is_occupied(36), "target stays put");
        assert_eq!(pos.mailbox[36].hp(), 1, "damage still applied");
    }

    #[test]
    fn hook_diagonal_pull() {
        // P1 at a1 (sq 0), P2 at c3 (sq 18). Diagonal NE.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 0, Skill::Hook as u8);
        place(&mut pos, 18, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(0, 18, Skill::Hook));
        assert!(!pos.is_occupied(18));
        assert!(pos.is_occupied(9), "pulled to b2");
    }

    #[test]
    fn hook_combo_tick_two_casters() {
        // Two casters on rays to a tough target. P1 Champion A at d3 (sq 19),
        // P1 Champion B at f5 (sq 37), P2 target Champion at e4 (sq 28) —
        // both casters within range 2 (Chebyshev dist 1, diagonal).
        // Target Armor=2 to absorb damage so it survives two casts.
        let mut pos = skill_phase_pos(4);
        pos.p1_money = 20;
        place(&mut pos, 19, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 19, Skill::Hook as u8);
        place(&mut pos, 37, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 37, Skill::Hook as u8);
        place(&mut pos, 28, Player::P2, PieceKind::Champion, 2, 2);

        // First Hook from sq 19 → tgt 28. step_toward(28,19)=sq 19 (caster),
        // pull skipped. 1 dmg absorbed by Armor (2→1). Combo tick → 1.
        let _ = make(&mut pos, skill_action(19, 28, Skill::Hook));
        assert!(pos.is_occupied(28));
        assert_eq!(pos.mailbox[28].armor(), 1);
        assert_eq!(pos.mailbox[28].combo(), 1);

        // Second Hook from sq 37 → tgt 28: existing_combo=1, total dmg = 1+1=2.
        // Armor absorbs first point (1→0), HP absorbs second (2→1).
        let _ = make(&mut pos, skill_action(37, 28, Skill::Hook));
        assert!(pos.is_occupied(28));
        assert_eq!(pos.mailbox[28].armor(), 0);
        assert_eq!(pos.mailbox[28].hp(), 1);
        assert_eq!(pos.mailbox[28].combo(), 2);
    }

    // --- Tempest (6) ------------------------------------------------------

    #[test]
    fn tempest_damages_target_and_pushes_neighbours() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0); // e4
        equip(&mut pos, 28, Skill::Tempest as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0); // e6 target
        place(&mut pos, 43, Player::P2, PieceKind::Guard,    2, 0); // d6
        place(&mut pos, 52, Player::P2, PieceKind::Guard,    2, 0); // e7
        place(&mut pos, 45, Player::P2, PieceKind::Guard,    2, 0); // f6

        let _ = make(&mut pos, skill_action(28, 44, Skill::Tempest));
        // Target damage
        assert_eq!(pos.mailbox[44].hp(), 1, "target takes 1");
        // d6 → c6 (sq 42)
        assert!(!pos.is_occupied(43));
        assert!(pos.is_occupied(42));
        // e7 → e8 (sq 60)
        assert!(!pos.is_occupied(52));
        assert!(pos.is_occupied(60));
        // f6 → g6 (sq 46)
        assert!(!pos.is_occupied(45));
        assert!(pos.is_occupied(46));
    }

    #[test]
    fn tempest_does_not_push_caster() {
        // Caster adjacent to target: e5 (sq 36) → target e6 (sq 44).
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 36, Skill::Tempest as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(36, 44, Skill::Tempest));
        assert!(pos.is_occupied(36), "caster not pushed");
        assert_eq!(pos.mailbox[44].hp(), 1);
    }

    #[test]
    fn tempest_push_blocked_by_piece_no_effect() {
        // Caster e4 (28), target e6 (44), neighbour d6 (43), c6 (42) blocker.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Tempest as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 43, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 42, Player::P2, PieceKind::Guard,    2, 0); // blocks d6→c6

        let _ = make(&mut pos, skill_action(28, 44, Skill::Tempest));
        assert!(pos.is_occupied(43), "d6 stays — push blocked by c6");
        assert!(pos.is_occupied(42), "c6 unchanged");
    }

    #[test]
    fn tempest_push_off_board_no_effect() {
        // Caster c4 (sq 26), target a4 (sq 24), neighbour a3 (sq 16). Push
        // direction tgt(a4)→a3: dr = -1, df = 0 → a2 (sq 8). On-board. To
        // force off-board, use a4 target and a5 neighbour: push dir tgt→a5
        // is dr=+1, df=0 → a6 on-board too. The truly off-board case for
        // Tempest is on the rank/file edge: target a4, neighbour b4 (sq 25)
        // — push direction tgt(a4)→b4 is dr=0 df=+1 → c4 (caster!). Use
        // target a1 (sq 0), neighbour a2 (sq 8) — pivot a1, push_away → a3 (16).
        // Truly off-board: target a1, neighbour-that-would-push-off would
        // sit at sq -1 etc. So use target h8 (sq 63), neighbour h7 (sq 55)
        // — push direction (63,55) is dr=-1,df=0 → h6 on-board too.
        //
        // The off-board case: target square at edge AND neighbour sits in
        // a corner-direction. Target a1 (sq 0), neighbour b1 (sq 1): push
        // dir (0,1) is dr=0,df=+1 → c1 (sq 2), on-board. There is NO
        // off-board push from a centre-piece target. Off-board only when
        // target on edge, neighbour ALSO on edge, pushed in the off-board
        // direction.
        //
        // Concrete off-board case: target a2 (sq 8), neighbour a1 (sq 0).
        // step_away(8, 0): dr=signum(-1)=-1, df=0, new rank = -1 → off-board.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 17, Player::P1, PieceKind::Champion, 2, 0); // b3, caster
        equip(&mut pos, 17, Skill::Tempest as u8);
        place(&mut pos, 8,  Player::P2, PieceKind::Champion, 2, 0); // a2 target
        place(&mut pos, 0,  Player::P2, PieceKind::Guard,    2, 0); // a1 neighbour

        let _ = make(&mut pos, skill_action(17, 8, Skill::Tempest));
        assert!(pos.is_occupied(0), "a1 stays — push would go off-board");
    }

    #[test]
    fn tempest_combo_tick_on_all_pushed_and_target() {
        // Caster e4 (28), target e6 (44), pushed: d6 (43), e7 (52), f6 (45).
        // Target has Armor=2 to survive the damage so we can read its combo.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Tempest as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 2); // survives via armor
        place(&mut pos, 43, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 52, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 45, Player::P2, PieceKind::Guard,    2, 0);

        let _ = make(&mut pos, skill_action(28, 44, Skill::Tempest));
        assert_eq!(pos.mailbox[44].combo(), 1, "target ticked");
        // Pushed pieces now sit at new squares.
        assert_eq!(pos.mailbox[42].combo(), 1, "c6 (formerly d6) ticked");
        assert_eq!(pos.mailbox[60].combo(), 1, "e8 (formerly e7) ticked");
        assert_eq!(pos.mailbox[46].combo(), 1, "g6 (formerly f6) ticked");
    }

    #[test]
    fn tempest_unmake_full_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Tempest as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 43, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 52, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 45, Player::P2, PieceKind::Guard,    2, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 44, Skill::Tempest));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos),
                "tempest+unmake roundtrip diff: {:?}", pos_diff(&snapshot, &pos));
    }

    // --- Combo + Charge integration (4) -----------------------------------

    #[test]
    fn combo_tick_increments_target_counter_on_first_strike() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.mailbox[36].combo(), 1);
    }

    #[test]
    fn combo_no_double_tick_same_caster() {
        let mut pos = skill_phase_pos(4);
        pos.p1_money = 20;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        // Armor=2 absorbs first cast; second cast (with combo bonus) deals 2,
        // armor 1→0 then HP 2→1. Target alive both times.
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.mailbox[36].combo(), 1);
        assert_eq!(pos.mailbox[36].armor(), 1, "1 dmg, no combo bonus first cast");
        assert_eq!(pos.mailbox[36].hp(), 2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert_eq!(pos.mailbox[36].combo(), 1, "same caster does not re-tick");
        // Second cast: 1 base + 1 combo bonus = 2 dmg. Armor 1→0, HP 2→1.
        assert_eq!(pos.mailbox[36].armor(), 0);
        assert_eq!(pos.mailbox[36].hp(), 1);
    }

    #[test]
    fn combo_two_casters_increment_independently() {
        let mut pos = skill_phase_pos(4);
        pos.p1_money = 20;
        place(&mut pos, 19, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 19, Skill::Lance as u8);
        place(&mut pos, 37, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 37, Skill::Lance as u8);
        // Lance is range 1 — both casters are diagonal-adjacent to sq 28.
        place(&mut pos, 28, Player::P2, PieceKind::Champion, 2, 2);

        let _ = make(&mut pos, skill_action(19, 28, Skill::Lance));
        assert_eq!(pos.mailbox[28].combo(), 1);
        let _ = make(&mut pos, skill_action(37, 28, Skill::Lance));
        assert_eq!(pos.mailbox[28].combo(), 2, "second caster ticks again");
    }

    #[test]
    fn charge_consumed_first_strike_not_second() {
        let mut pos = skill_phase_pos(4);
        pos.p1_money = 20;
        pos.pending_modifiers |= modifier_bits::CHARGE;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        // 1 base + 1 charge = 2 dmg. Armor 2→0.
        assert_eq!(pos.mailbox[36].armor(), 0, "2 dmg absorbed by 2 Armor");
        assert_eq!(pos.mailbox[36].hp(), 2);
        assert_eq!(pos.pending_modifiers & modifier_bits::CHARGE, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        // Second cast: existing_combo=1, same caster won't re-tick → dmg = 2.
        // Armor 0, so HP takes 2 → HP 0 → target removed.
        assert!(!pos.is_occupied(36), "1 base + 1 combo bonus = 2 dmg kills");
    }

    // --- Cross-skill roundtrip (2) ----------------------------------------

    #[test]
    fn make_unmake_roundtrip_lance_break_steal() {
        let mut pos = skill_phase_pos(6);
        pos.p1_money = 20;
        pos.p2_money = 8;
        // Three P1 Champions adjacent to a tough target — all within range 1.
        place(&mut pos, 19, Player::P1, PieceKind::Champion, 2, 0); // d3
        equip(&mut pos, 19, Skill::Lance as u8);
        place(&mut pos, 37, Player::P1, PieceKind::Champion, 2, 0); // f5
        equip(&mut pos, 37, Skill::Break as u8);
        place(&mut pos, 27, Player::P1, PieceKind::Champion, 2, 0); // d4
        equip(&mut pos, 27, Skill::Steal as u8);
        // Target at e4 (sq 28), Armor=2 HP=2 to soak the strikes.
        place(&mut pos, 28, Player::P2, PieceKind::Champion, 2, 2);
        let snapshot = pos.clone();

        let u1 = make(&mut pos, skill_action(19, 28, Skill::Lance));
        let u2 = make(&mut pos, skill_action(37, 28, Skill::Break));
        let u3 = make(&mut pos, skill_action(27, 28, Skill::Steal));

        unmake(&mut pos, &u3);
        unmake(&mut pos, &u2);
        unmake(&mut pos, &u1);
        assert!(pos_eq(&snapshot, &pos), "three-skill roundtrip identity");
    }

    #[test]
    fn make_unmake_roundtrip_hook_then_tempest() {
        let mut pos = skill_phase_pos(2);
        pos.p1_money = 20;
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Hook as u8);
        place(&mut pos, 44, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 43, Player::P2, PieceKind::Guard,    2, 0);
        place(&mut pos, 45, Player::P2, PieceKind::Guard,    2, 0);
        let snapshot = pos.clone();

        let u = make(&mut pos, skill_action(28, 44, Skill::Hook));
        unmake(&mut pos, &u);
        assert!(pos_eq(&snapshot, &pos), "hook-only roundtrip in a tempest-shaped setup");
    }

    // --- King-capture-via-skill (1) ---------------------------------------

    #[test]
    fn lance_kills_king_ends_game() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Lance as u8);
        place(&mut pos, 36, Player::P2, PieceKind::King, 1, 0);
        let snapshot = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert!(!pos.is_occupied(36));
        assert_eq!(pos.game_result, Some(GameResult::P1Wins));

        unmake(&mut pos, &undo);
        assert!(pos_eq(&snapshot, &pos));
    }
}
