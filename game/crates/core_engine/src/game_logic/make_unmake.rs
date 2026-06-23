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
use crate::state::zobrist::{self, PieceKind as ZKind};
use crate::state::{magic, Bitboard, MailboxEntry, Position, EMPTY_MAILBOX_ENTRY};

pub fn make(pos: &mut Position, action: Action) -> Undo {
    let mut undo = Undo {
        action: action.0,
        prev_game_result: game_result_to_tag(pos.game_result),
        prev_pending_modifiers: pos.pending_modifiers,
        prev_phase: phase_to_byte(pos.current_phase),
        prev_actions_remaining: pos.actions_remaining,
        prev_to_move: player_to_byte(pos.to_move),
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
        ActionKind::EndTurn   => super::turn_manager::end_turn(pos, &mut undo),
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
    pos.to_move            = player_from_byte(undo.prev_to_move);
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
    dec_actions(pos, undo);
}

/// Move mover from `src` to `tgt`. `moved_this_phase` is set on `tgt`.
fn apply_plain_move(pos: &mut Position, src: u8, tgt: u8, undo: &mut Undo) {
    debug_assert!(pos.is_occupied(src), "plain move from empty square");
    debug_assert!(!pos.is_occupied(tgt), "plain move into occupied square");

    let prev_entry = pos.mailbox[src as usize];
    let owner = player_at(pos, src);
    let kind  = piece_kind_at(pos, src);

    // Mailbox: clear src, copy entry to tgt (zobrist-aware).
    write_mailbox(pos, undo, src, EMPTY_MAILBOX_ENTRY);
    write_mailbox(pos, undo, tgt, prev_entry);

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

    // Zobrist: piece leaves src, appears at tgt.
    xor_piece(pos, undo, src, owner, kind);
    xor_piece(pos, undo, tgt, owner, kind);

    // Mark destination as moved-this-phase.
    moved_set(pos, undo, tgt);
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
    moved_set(pos, undo, src);
}

/// Deal 1 point of damage to the piece on `hit_sq`. Armor absorbs first;
/// otherwise HP drops by 1; piece is removed from all bitboards if HP hits 0.
fn deal_one_damage(pos: &mut Position, hit_sq: u8, undo: &mut Undo) {
    let prev_entry = pos.mailbox[hit_sq as usize];

    if prev_entry.armor() > 0 {
        // Armor absorbs the hit — HP unchanged.
        write_mailbox(pos, undo, hit_sq, prev_entry.with_armor(prev_entry.armor() - 1));
        return;
    }

    // No armor — HP drops.
    let new_hp = prev_entry.hp().saturating_sub(1);
    if new_hp == 0 {
        // Piece removed. Capture King + owner identity *before* mutating any
        // bitboards so we can set game_result correctly afterwards.
        let was_king  = pos.kings.contains(hit_sq);
        let owned_by_p1 = pos.p1_pieces.contains(hit_sq);
        let owner = if owned_by_p1 { Player::P1 } else { Player::P2 };
        let kind  = piece_kind_at(pos, hit_sq);

        write_mailbox(pos, undo, hit_sq, EMPTY_MAILBOX_ENTRY);
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
            set_game_result(pos, undo, Some(if owned_by_p1 {
                GameResult::P2Wins
            } else {
                GameResult::P1Wins
            }));
        } else if pos.champions.contains(hit_sq) {
            pos.champions = Bitboard(pos.champions.0 ^ bit);
            undo.champions_xor ^= bit;
        } else if pos.guards.contains(hit_sq) {
            pos.guards = Bitboard(pos.guards.0 ^ bit);
            undo.guards_xor ^= bit;
        }

        // Zobrist: piece disappears from hit_sq.
        xor_piece(pos, undo, hit_sq, owner, kind);
    } else {
        write_mailbox(pos, undo, hit_sq, prev_entry.with_hp(new_hp));
    }
}

// === Skill-kind dispatch ===================================================

/// Apply a Skill-kind action. Slices 4+5 wire the thirteen non-Mystic
/// resolvers. Focus and Charge (Mystic setters) land here in Slice 6.
fn apply_skill(pos: &mut Position, action: Action, undo: &mut Undo) {
    debug_assert!(pos.actions_remaining > 0, "make() invoked with zero actions");
    let skill = super::skills::skill_from_id(action.skill_id())
        .expect("generator emitted unknown skill id");
    use super::skills::{Skill, SkillCategory, skill_category};

    // Stack-M (session-31 clarification): Focus = "the next non-Mystic skill",
    // so a Mystic skill (Focus/Charge) cast does NOT consume a pending Focus.
    // The legal sequence Charge → Focus → Strike applies BOTH buffs to the
    // Strike; Focus → Charge → Strike applies only Charge (the Focus was
    // consumed by Charge if Charge were non-Mystic — but it isn't, so we
    // must NOT clear here). For non-Mystic skills, the generator already
    // enumerated them at +1 Range when Focus is pending, so no recomputation
    // is needed here — the legal-action set already reflects the buff. For
    // Move-skills where Focus chose effect-range, the resolver below reads
    // `action.focus_effect_mode()` to pick the buffed effect.
    let is_mystic = matches!(skill_category(skill), SkillCategory::Mystic);
    if !is_mystic {
        clear_pending(pos, undo, modifier_bits::FOCUS);
    }

    match skill {
        Skill::Lance   => apply_lance(pos, action, undo),
        Skill::Break   => apply_break(pos, action, undo),
        Skill::Steal   => apply_steal(pos, action, undo),
        Skill::Hook    => apply_hook(pos, action, undo),
        Skill::Tempest => apply_tempest(pos, action, undo),
        Skill::Shield  => apply_shield(pos, action, undo),
        Skill::Heal    => apply_heal(pos, action, undo),
        Skill::Plate   => apply_plate(pos, action, undo),
        Skill::Dash    => apply_dash(pos, action, undo),
        Skill::Blast   => apply_blast(pos, action, undo),
        Skill::Shove   => apply_shove(pos, action, undo),
        Skill::Swap    => apply_swap(pos, action, undo),
        Skill::Retreat => apply_retreat(pos, action, undo),
        Skill::Focus   => apply_focus(pos, action, undo),
        Skill::Charge  => apply_charge(pos, action, undo),
    }
}

// === Strike-skill resolvers (Slice 4) ======================================

const ARMOR_CAP: u8 = 2;
const FULL_HP: u8 = 2;
const INJURED_HP: u8 = 1;

fn apply_lance(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    apply_strike_damage(pos, src, tgt, /*base=*/ 1, undo);
    debit_money(pos, src, /*cost=*/ 2, undo);
    dec_actions(pos, undo);
}

fn apply_break(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let prev = pos.mailbox[tgt as usize];

    let charge_active = pos.pending_modifiers & modifier_bits::CHARGE != 0;
    if charge_active { clear_pending(pos, undo, modifier_bits::CHARGE); }
    let existing_combo = prev.combo();

    // Tick first (it modifies the mailbox entry's combo field).
    let _ = combo_tick(pos, src, tgt, undo);

    // Armor reduction — applies regardless of HP-damage gating. Read the
    // mailbox AGAIN because combo_tick may have written a new entry.
    let post_tick = pos.mailbox[tgt as usize];
    let new_armor = post_tick.armor().saturating_sub(1);
    write_mailbox(pos, undo, tgt, post_tick.with_armor(new_armor));

    // HP-damage gate: Stack-M says Break "does not deal HP-Damage unless
    // boosted by Charge." But the universal combo bonus ("any skill that
    // affects a target with counter > 0 deals +counter damage") still
    // applies on top.
    let dmg = (if charge_active { 1u8 } else { 0 }) + existing_combo;
    if dmg > 0 { deal_damage(pos, tgt, dmg, undo); }

    debit_money(pos, src, /*cost=*/ 2, undo);
    dec_actions(pos, undo);
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
    dec_actions(pos, undo);
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
    dec_actions(pos, undo);
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
    dec_actions(pos, undo);
}

// === Shield-class + Move-class resolvers (Slice 5) =========================

fn apply_shield(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    // Focus retarget: when has_aux, the recipient is the adjacent ally at
    // aux_sq, not the caster. The caster still pays the cost and consumes
    // Focus, but the +1 armor lands on the ally.
    let recipient = if action.has_aux() { action.aux_sq() } else {
        debug_assert_eq!(src, action.target(), "Shield is SelfOnly (no retarget)");
        src
    };
    let prev = pos.mailbox[recipient as usize];
    debug_assert!(prev.armor() < ARMOR_CAP, "generator must filter at-cap");
    write_mailbox(pos, undo, recipient, prev.with_armor(prev.armor() + 1));
    debit_money(pos, src, /*cost=*/ 2, undo);
    dec_actions(pos, undo);
}

fn apply_heal(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let prev = pos.mailbox[tgt as usize];
    debug_assert!(prev.hp() == INJURED_HP, "generator must filter non-Injured");
    write_mailbox(pos, undo, tgt, prev.with_hp(FULL_HP));
    debit_money(pos, src, /*cost=*/ 3, undo);
    dec_actions(pos, undo);
}

fn apply_plate(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let prev = pos.mailbox[tgt as usize];
    debug_assert!(prev.armor() < ARMOR_CAP, "generator must filter at-cap");
    write_mailbox(pos, undo, tgt, prev.with_armor(prev.armor() + 1));
    debit_money(pos, src, /*cost=*/ 3, undo);
    dec_actions(pos, undo);
}

fn apply_dash(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let dest = action.target();
    // Focus retarget: when has_aux, the ally at aux_sq moves (Range 2) and
    // the caster (src) stays put — caster only pays the cost. The caster's
    // square (used as the lookup for which side pays) is `src` if retargeted
    // (caster didn't move), `dest` otherwise (caster moved there).
    let (mover, payer_sq) = if action.has_aux() {
        (action.aux_sq(), src)
    } else {
        (src, dest)
    };
    debug_assert!(pos.is_occupied(mover) && !pos.is_occupied(dest) && mover != dest);
    relocate_piece(pos, mover, dest, undo);
    // No combo-tick (self/ally movement). No moved_this_phase write (Skill-Phase).
    debit_money(pos, payer_sq, /*cost=*/ 3, undo);
    dec_actions(pos, undo);
}

/// Blast: pure push, no damage. Movement-causing → ticks combo on enemy
/// target. Pre-tick combo counter is applied as bonus damage per Stack-M
/// ("any skill that affects a target with counter > 0 deals +counter
/// damage"). Push fizzles silently if blocked or off-board.
///
/// Focus-effect mode (`action.focus_effect_mode() == true`): push 2 tiles
/// instead of 1. Intermediate tile must be empty; if it is occupied or
/// off-board, the second hop is cancelled and the push lands at 1 tile
/// (i.e. the same result as no-Focus). If the 1-tile destination is
/// occupied/off-board, the whole push fizzles as usual.
fn apply_blast(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let pre_tick_combo = pos.mailbox[tgt as usize].combo();
    let _ = combo_tick(pos, src, tgt, undo);
    if pre_tick_combo > 0 {
        deal_damage(pos, tgt, pre_tick_combo, undo);
    }
    if pos.is_occupied(tgt) {
        if let Some(step1) = magic::step_away(src, tgt) {
            if !pos.is_occupied(step1) {
                // Focus-effect mode tries to extend to a 2-tile push.
                // Use the SAME direction as step1 (away from src), not
                // step_away(src, step1) which would re-anchor on step1.
                let final_dest = if action.focus_effect_mode() {
                    magic::step_away(src, step1)
                        .filter(|&d| !pos.is_occupied(d))
                        .unwrap_or(step1)
                } else {
                    step1
                };
                relocate_piece(pos, tgt, final_dest, undo);
            }
        }
    }
    debit_money(pos, src, /*cost=*/ 2, undo);
    dec_actions(pos, undo);
}

/// Shove: push target 1 tile in chosen direction (encoded in choice_idx).
/// Combo-tick gated by target-is-enemy (Stack-M: friendly pushes don't
/// count). Pre-tick combo bonus damage applies on enemy only.
///
/// Focus-effect mode (`action.focus_effect_mode() == true`): push 2 tiles
/// in the chosen direction. The generator only emits Focus-effect Shove
/// actions where both intermediate and final squares are empty and on-board,
/// so the resolver can trust the destination is reachable.
fn apply_shove(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    let dir = action.choice_idx() as usize;
    debug_assert!(dir < 8);

    let step1 = magic::neighbour_in_dir(tgt, dir)
        .expect("generator must filter off-board pushes");
    debug_assert!(!pos.is_occupied(step1),
                  "generator must filter into-occupied pushes");

    let push_dest = if action.focus_effect_mode() {
        let step2 = magic::neighbour_in_dir(step1, dir)
            .expect("generator must filter off-board Focus-shoves");
        debug_assert!(!pos.is_occupied(step2),
                      "generator must filter into-occupied Focus-shoves");
        step2
    } else {
        step1
    };

    let caster_is_p1 = pos.p1_pieces.contains(src);
    let target_is_enemy = if caster_is_p1 {
        pos.p2_pieces.contains(tgt)
    } else {
        pos.p1_pieces.contains(tgt)
    };

    if target_is_enemy {
        let pre_tick_combo = pos.mailbox[tgt as usize].combo();
        let _ = combo_tick(pos, src, tgt, undo);
        if pre_tick_combo > 0 {
            deal_damage(pos, tgt, pre_tick_combo, undo);
        }
    }
    if pos.is_occupied(tgt) {
        relocate_piece(pos, tgt, push_dest, undo);
    }
    debit_money(pos, src, /*cost=*/ 3, undo);
    dec_actions(pos, undo);
}

/// Swap: exchange caster + allied piece. Both squares allied → same-side
/// bitboard unchanged. Kind layers (kings/champions/guards) XOR only where
/// the two pieces differ in kind. No combo-tick (ally-only).
fn apply_swap(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let tgt = action.target();
    debug_assert!(pos.is_occupied(src) && pos.is_occupied(tgt) && src != tgt);

    let prev_src = pos.mailbox[src as usize];
    let prev_tgt = pos.mailbox[tgt as usize];
    let owner_src = player_at(pos, src);
    let owner_tgt = player_at(pos, tgt);
    let kind_src  = piece_kind_at(pos, src);
    let kind_tgt  = piece_kind_at(pos, tgt);

    // Zobrist: each piece leaves its square and reappears at the other.
    xor_piece(pos, undo, src, owner_src, kind_src);
    xor_piece(pos, undo, tgt, owner_tgt, kind_tgt);
    xor_piece(pos, undo, tgt, owner_src, kind_src);
    xor_piece(pos, undo, src, owner_tgt, kind_tgt);

    write_mailbox(pos, undo, src, prev_tgt);
    write_mailbox(pos, undo, tgt, prev_src);

    let xor = Bitboard::from_square(src).0 | Bitboard::from_square(tgt).0;
    let src_in_k = pos.kings.contains(src);     let tgt_in_k = pos.kings.contains(tgt);
    let src_in_c = pos.champions.contains(src); let tgt_in_c = pos.champions.contains(tgt);
    let src_in_g = pos.guards.contains(src);    let tgt_in_g = pos.guards.contains(tgt);
    if src_in_k != tgt_in_k {
        pos.kings = Bitboard(pos.kings.0 ^ xor);
        undo.kings_xor ^= xor;
    }
    if src_in_c != tgt_in_c {
        pos.champions = Bitboard(pos.champions.0 ^ xor);
        undo.champions_xor ^= xor;
    }
    if src_in_g != tgt_in_g {
        pos.guards = Bitboard(pos.guards.0 ^ xor);
        undo.guards_xor ^= xor;
    }

    debit_money(pos, tgt, /*cost=*/ 4, undo);
    dec_actions(pos, undo);
}

fn apply_retreat(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    let dest = action.target();
    // Focus retarget: when has_aux, the ally at aux_sq retreats (lands
    // adjacent to a friendly Guard) while the caster stays put.
    let (mover, payer_sq) = if action.has_aux() {
        (action.aux_sq(), src)
    } else {
        (src, dest)
    };
    debug_assert!(pos.is_occupied(mover) && !pos.is_occupied(dest) && mover != dest);
    relocate_piece(pos, mover, dest, undo);
    debit_money(pos, payer_sq, /*cost=*/ 4, undo);
    dec_actions(pos, undo);
}

// === Mystic-skill resolvers (Slice 6) =======================================

/// Focus (cost 1): set the FOCUS bit in pending_modifiers.
///
/// Stack-M (session-31): "The next non-Mystic skill used by any of your
/// pieces this turn gains +1 Range." A subsequent Mystic skill (Focus or
/// Charge) does NOT consume the pending Focus — only a Strike/Shield/Move
/// skill does. At most one Focus may be active at a time (generator filters
/// the duplicate; we debug-assert here as defence-in-depth). No combo-tick
/// (pure buff). Action encoding: `src == tgt == caster`.
fn apply_focus(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    debug_assert_eq!(src, action.target(), "Focus is a self-cast");
    debug_assert!(pos.is_occupied(src));
    debug_assert_eq!(
        pos.pending_modifiers & modifier_bits::FOCUS, 0,
        "generator emitted Focus while Focus already pending — illegal per Stack-M"
    );
    add_pending(pos, undo, modifier_bits::FOCUS);
    debit_money(pos, src, /*cost=*/ 1, undo);
    dec_actions(pos, undo);
}

/// Charge (cost 3): set the CHARGE bit in pending_modifiers.
///
/// Stack-M (session-31): "The next Strike skill used by any of your pieces
/// this turn deals +1 damage." Charge waits for the next *Strike* skill
/// specifically (consumption is wired inside `apply_strike_damage`); other
/// skills do not consume it. At most one Charge may be active at a time
/// (generator filters the duplicate; debug-assert here as defence-in-depth).
/// No combo-tick (pure buff).
fn apply_charge(pos: &mut Position, action: Action, undo: &mut Undo) {
    let src = action.src();
    debug_assert_eq!(src, action.target(), "Charge is a self-cast");
    debug_assert!(pos.is_occupied(src));
    debug_assert_eq!(
        pos.pending_modifiers & modifier_bits::CHARGE, 0,
        "generator emitted Charge while Charge already pending — illegal per Stack-M"
    );
    add_pending(pos, undo, modifier_bits::CHARGE);
    debit_money(pos, src, /*cost=*/ 3, undo);
    dec_actions(pos, undo);
}

// === Strike-skill helpers ==================================================

/// Resolve a Strike effect on `tgt_sq`: consume Charge if pending, read
/// existing combo counter, tick combo (gated by caster identity), deal
/// `base + existing_combo + charge_bonus` damage through the standard
/// Armor → HP → removal pipeline. Returns the total damage scheduled.
fn apply_strike_damage(pos: &mut Position, src_sq: u8, tgt_sq: u8,
                       base: u8, undo: &mut Undo) -> u8 {
    let prev = pos.mailbox[tgt_sq as usize];
    // Snapshot the prior mailbox even if no damage is dealt — combo_tick or
    // deal_damage may not actually mutate this square (if the target survives
    // with combo bumped only), and we still want the dedup'd prior recorded.
    record_affected(undo, tgt_sq, prev);

    let charge_bonus = if pos.pending_modifiers & modifier_bits::CHARGE != 0 {
        clear_pending(pos, undo, modifier_bits::CHARGE);
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
    let new_combo = (prev.combo() + 1).min(7);
    write_mailbox(pos, undo, tgt_sq, prev.with_combo(new_combo));
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
    let owner = player_at(pos, from);
    let kind  = piece_kind_at(pos, from);

    write_mailbox(pos, undo, from, EMPTY_MAILBOX_ENTRY);
    write_mailbox(pos, undo, to,   prev_from);

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

    // Zobrist: piece leaves `from`, appears at `to`.
    xor_piece(pos, undo, from, owner, kind);
    xor_piece(pos, undo, to,   owner, kind);
}

/// Debit `cost` Money from the side that owns `caster_sq`.
fn debit_money(pos: &mut Position, caster_sq: u8, cost: u8, undo: &mut Undo) {
    let cost_u16 = cost as u16;
    if pos.p1_pieces.contains(caster_sq) {
        let new = pos.p1_money.saturating_sub(cost_u16);
        set_p1_money(pos, undo, new);
    } else {
        let new = pos.p2_money.saturating_sub(cost_u16);
        set_p2_money(pos, undo, new);
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
    match from {
        Player::P1 => set_p1_money(pos, undo, pos.p1_money - actual),
        Player::P2 => set_p2_money(pos, undo, pos.p2_money - actual),
    }
    match to {
        Player::P1 => set_p1_money(pos, undo, pos.p1_money + actual),
        Player::P2 => set_p2_money(pos, undo, pos.p2_money + actual),
    }
}

// === EndPhase ==============================================================

/// Move → Skill transition (Slice 1 simplification).
///
/// Skill-Phase action budget follows the paper-baseline progression curve
/// adopted into Stack M (oq-69 resolved, session-31): +1 action per 10 rounds,
/// starting at 2. R1–10:2, R11–20:3, R21–30:4, R31–40:5, R41–50:6, …
/// End-of-turn (Skill → next turn) is delegated to `turn_manager::end_turn`.
fn apply_end_phase(pos: &mut Position, undo: &mut Undo) {
    match pos.current_phase {
        Phase::Move => {
            moved_clear_all(pos, undo);
            set_phase(pos, undo, Phase::Skill);
            set_actions(pos, undo, skill_phase_budget(pos.round_number));
        }
        Phase::Skill => {
            super::turn_manager::end_turn(pos, undo);
        }
    }
}

/// Skill-Phase action budget for the given round (Stack M).
///
/// Formula: `2 + (round_number - 1) / 10`. Unbounded — +1 per 10 rounds.
/// The paper rule sheet shows "R31+: 5" as a table cut-off, not a cap;
/// R41–50 is 6, R51–60 is 7, and so on. Saturates at u8::MAX defensively
/// (games will never reach that, but we don't want to panic on overflow).
#[inline]
pub(crate) fn skill_phase_budget(round_number: u16) -> u8 {
    let tier = round_number.saturating_sub(1) / 10;
    (2u16 + tier).min(u8::MAX as u16) as u8
}

// === Tiny helpers ===========================================================

pub(super) fn record_affected(undo: &mut Undo, sq: u8, prev: MailboxEntry) {
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

// === Zobrist-aware mutation helpers =======================================
//
// These are the single source of state mutation for everything that affects
// the hash. Callers that go around them will silently desync `pos.zobrist`
// from the position's actual state. Slice 7 routed every existing mutation
// site through this layer.

#[inline]
fn z_apply(pos: &mut Position, undo: &mut Undo, delta: u64) {
    pos.zobrist ^= delta;
    undo.zobrist_xor ^= delta;
}

/// Write `new` into `pos.mailbox[sq]`, recording the prior state in `undo`
/// (dedup'd) and XOR-ing the mailbox-key delta into the zobrist hash. No-op
/// if `new == prev`.
pub(super) fn write_mailbox(pos: &mut Position, undo: &mut Undo, sq: u8, new: MailboxEntry) {
    let prev = pos.mailbox[sq as usize];
    if prev == new { return; }
    record_affected(undo, sq, prev);
    let delta = zobrist::mailbox_xor(sq, prev, new);
    z_apply(pos, undo, delta);
    pos.mailbox[sq as usize] = new;
}

/// XOR a piece-occupancy key into the zobrist hash. Call this each time a
/// piece appears at or disappears from `sq` (the key is its own inverse).
#[inline]
pub(super) fn xor_piece(pos: &mut Position, undo: &mut Undo,
                        sq: u8, player: Player, kind: ZKind) {
    z_apply(pos, undo, zobrist::piece_key(sq, player, kind));
}

/// Determine the piece kind at `sq` from the bitboards. Panics in debug if
/// `sq` is unoccupied. Slice 7: occupancy keys need this lookup.
#[inline]
pub(super) fn piece_kind_at(pos: &Position, sq: u8) -> ZKind {
    if pos.kings.contains(sq)       { ZKind::King }
    else if pos.champions.contains(sq) { ZKind::Champion }
    else if pos.guards.contains(sq)    { ZKind::Guard }
    else { debug_assert!(false, "piece_kind_at on empty sq {}", sq); ZKind::Guard }
}

/// Determine the owning player at `sq` from the bitboards.
#[inline]
pub(super) fn player_at(pos: &Position, sq: u8) -> Player {
    if pos.p1_pieces.contains(sq) { Player::P1 } else { Player::P2 }
}

// --- Scalar setters: each XORs prev→new key delta into zobrist. ---------

#[inline]
pub(super) fn set_actions(pos: &mut Position, undo: &mut Undo, new: u8) {
    if pos.actions_remaining == new { return; }
    let delta = zobrist::actions_key(pos.actions_remaining) ^ zobrist::actions_key(new);
    z_apply(pos, undo, delta);
    pos.actions_remaining = new;
}

#[inline]
pub(super) fn dec_actions(pos: &mut Position, undo: &mut Undo) {
    debug_assert!(pos.actions_remaining > 0, "make() invoked with zero actions");
    set_actions(pos, undo, pos.actions_remaining - 1);
}

#[inline]
pub(super) fn set_phase(pos: &mut Position, undo: &mut Undo, new: Phase) {
    if matches!(pos.current_phase, _) && phase_eq(pos.current_phase, new) { return; }
    z_apply(pos, undo, zobrist::phase_key()); // Move↔Skill is a single key flip.
    pos.current_phase = new;
}

#[inline]
fn phase_eq(a: Phase, b: Phase) -> bool {
    matches!((a, b), (Phase::Move, Phase::Move) | (Phase::Skill, Phase::Skill))
}

#[inline]
pub(super) fn flip_to_move(pos: &mut Position, undo: &mut Undo) {
    z_apply(pos, undo, zobrist::side_key());
    pos.to_move = match pos.to_move { Player::P1 => Player::P2, Player::P2 => Player::P1 };
}

#[inline]
pub(super) fn set_round(pos: &mut Position, undo: &mut Undo, new: u16) {
    if pos.round_number == new { return; }
    let delta = zobrist::round_key(pos.round_number) ^ zobrist::round_key(new);
    z_apply(pos, undo, delta);
    pos.round_number = new;
}

#[inline]
pub(super) fn set_pending(pos: &mut Position, undo: &mut Undo, new: u8) {
    if pos.pending_modifiers == new { return; }
    z_apply(pos, undo, zobrist::pending_mod_xor(pos.pending_modifiers, new));
    pos.pending_modifiers = new;
}

#[inline]
pub(super) fn add_pending(pos: &mut Position, undo: &mut Undo, bits: u8) {
    set_pending(pos, undo, pos.pending_modifiers | bits);
}

#[inline]
pub(super) fn clear_pending(pos: &mut Position, undo: &mut Undo, bits: u8) {
    set_pending(pos, undo, pos.pending_modifiers & !bits);
}

#[inline]
pub(super) fn set_p1_money(pos: &mut Position, undo: &mut Undo, new: u16) {
    if pos.p1_money == new { return; }
    let delta = zobrist::money_key_p1(pos.p1_money) ^ zobrist::money_key_p1(new);
    z_apply(pos, undo, delta);
    undo.p1_money_delta = undo.p1_money_delta
        .saturating_add(new as i32 as i16 - pos.p1_money as i32 as i16);
    pos.p1_money = new;
}

#[inline]
pub(super) fn set_p2_money(pos: &mut Position, undo: &mut Undo, new: u16) {
    if pos.p2_money == new { return; }
    let delta = zobrist::money_key_p2(pos.p2_money) ^ zobrist::money_key_p2(new);
    z_apply(pos, undo, delta);
    undo.p2_money_delta = undo.p2_money_delta
        .saturating_add(new as i32 as i16 - pos.p2_money as i32 as i16);
    pos.p2_money = new;
}

#[inline]
pub(super) fn set_game_result(pos: &mut Position, undo: &mut Undo, new: Option<GameResult>) {
    if pos.game_result == new { return; }
    let delta = zobrist::game_result_key(pos.game_result) ^ zobrist::game_result_key(new);
    z_apply(pos, undo, delta);
    pos.game_result = new;
}

#[inline]
pub(super) fn moved_set(pos: &mut Position, undo: &mut Undo, sq: u8) {
    if pos.moved_this_phase.contains(sq) { return; }
    z_apply(pos, undo, zobrist::moved_key(sq));
    pos.moved_this_phase = pos.moved_this_phase | Bitboard::from_square(sq);
}

#[inline]
pub(super) fn moved_clear_all(pos: &mut Position, undo: &mut Undo) {
    let mut bits = pos.moved_this_phase.0;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        z_apply(pos, undo, zobrist::moved_key(sq));
    }
    pos.moved_this_phase = Bitboard::EMPTY;
}

fn phase_to_byte(p: Phase) -> u8 {
    match p { Phase::Move => 0, Phase::Skill => 1 }
}
fn phase_from_byte(b: u8) -> Phase {
    match b { 0 => Phase::Move, _ => Phase::Skill }
}

fn player_to_byte(p: Player) -> u8 {
    match p { Player::P1 => 0, Player::P2 => 1 }
}
fn player_from_byte(b: u8) -> Player {
    match b { 0 => Player::P1, _ => Player::P2 }
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

    /// Deep-equal positions including the incremental zobrist hash.
    fn pos_eq(a: &Position, b: &Position) -> bool {
        pos_diff(a, b).is_none()
    }

    /// Returns a human-readable diff string if the two positions differ, else None.
    fn pos_diff(a: &Position, b: &Position) -> Option<String> {
        if a.zobrist != b.zobrist { return Some(format!("zobrist: {:#x} vs {:#x}", a.zobrist, b.zobrist)); }
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

    // === Slice 5 — Shield-class + Move-class resolvers ====================

    use super::super::generator::generate;

    fn shove_action(src: u8, tgt: u8, dir: u8) -> Action {
        Action::encode(src, tgt, ActionKind::Skill, Skill::Shove as u8, dir)
    }

    // --- Shield ---------------------------------------------------------

    #[test]
    fn shield_increments_armor_by_one() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shield as u8);

        let _ = make(&mut pos, skill_action(28, 28, Skill::Shield));
        assert_eq!(pos.mailbox[28].armor(), 1);
    }

    #[test]
    fn shield_costs_2_money_decrements_action() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shield as u8);
        let money_before = pos.p1_money;

        let _ = make(&mut pos, skill_action(28, 28, Skill::Shield));
        assert_eq!(pos.p1_money, money_before - 2);
        assert_eq!(pos.actions_remaining, 1);
    }

    #[test]
    fn shield_at_cap_filtered_by_generator() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 2);
        equip(&mut pos, 28, Skill::Shield as u8);

        let shield_id = Skill::Shield as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == shield_id),
            "Shield at armor cap must not be emitted"
        );
    }

    #[test]
    fn shield_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shield as u8);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 28, Skill::Shield));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Heal -----------------------------------------------------------

    #[test]
    fn heal_restores_injured_ally_to_full_hp() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Heal as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 1, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Heal));
        assert_eq!(pos.mailbox[36].hp(), 2);
    }

    #[test]
    fn heal_on_non_injured_ally_filtered() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Heal as u8);
        // Ally at full HP — not a legal Heal target.
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);

        let heal_id = Skill::Heal as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == heal_id
                && a.target() == 36),
            "Heal on full-HP ally must not be emitted"
        );
    }

    #[test]
    fn heal_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Heal as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 1, 0);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Heal));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Plate ----------------------------------------------------------

    #[test]
    fn plate_adds_armor_to_adjacent_ally() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Plate as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Plate));
        assert_eq!(pos.mailbox[36].armor(), 1);
    }

    #[test]
    fn plate_at_cap_filtered_by_generator() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Plate as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 2);

        let plate_id = Skill::Plate as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == plate_id
                && a.target() == 36),
            "Plate on armor-capped ally must not be emitted"
        );
    }

    #[test]
    fn plate_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Plate as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Plate));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Dash -----------------------------------------------------------

    #[test]
    fn dash_relocates_caster_one_tile() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Dash as u8);

        // e4 → e5 (N).
        let _ = make(&mut pos, skill_action(28, 36, Skill::Dash));
        assert!(!pos.is_occupied(28));
        assert!(pos.is_occupied(36));
        assert!(pos.p1_pieces.contains(36));
        assert!(pos.champions.contains(36));
    }

    #[test]
    fn dash_relocates_caster_two_tiles_diagonal() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 27, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 27, Skill::Dash as u8);

        // d4 (27) → f6 (45) — diagonal NE, range 2.
        let _ = make(&mut pos, skill_action(27, 45, Skill::Dash));
        assert!(!pos.is_occupied(27));
        assert!(pos.is_occupied(45));
        assert!(pos.p1_pieces.contains(45));
    }

    #[test]
    fn dash_path_blocked_no_emission() {
        // Dash 2-tile target on a ray blocked by piece in between.
        // Caster at e4 (sq 28). Ally at e5 (36). e6 (44) is the 2-tile target
        // along the N-ray, but skill_attacks treats the ally as a blocker —
        // so e6 must not appear as a Dash destination.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Dash as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);

        let dash_id = Skill::Dash as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == dash_id
                && a.src() == 28
                && a.target() == 44),
            "Dash through ally blocker must not be emitted"
        );
    }

    #[test]
    fn dash_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Dash as u8);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Dash));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Blast ----------------------------------------------------------

    #[test]
    fn blast_pushes_enemy_one_tile_away_no_damage() {
        // Caster e4 (28), enemy e5 (36). Blast pushes N → e6 (44).
        // Stack-M: Blast deals NO base damage.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Blast as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Blast));
        assert!(!pos.is_occupied(36), "enemy left e5");
        assert!(pos.is_occupied(44), "enemy arrived at e6");
        assert_eq!(pos.mailbox[44].hp(), 2, "Blast deals NO damage");
    }

    #[test]
    fn blast_off_board_push_fizzles() {
        // Caster at e7 (sq 52), enemy at e8 (sq 60). Push N is off-board.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 52, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 52, Skill::Blast as u8);
        place(&mut pos, 60, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(52, 60, Skill::Blast));
        assert!(pos.is_occupied(60), "enemy stays at e8, push fizzles off-board");
        assert_eq!(pos.mailbox[60].hp(), 2);
    }

    #[test]
    fn blast_into_occupied_push_fizzles() {
        // Caster e4 (28), enemy e5 (36), blocker at e6 (44).
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Blast as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 44, Player::P1, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Blast));
        assert!(pos.is_occupied(36), "enemy stays — push blocked");
        assert!(pos.is_occupied(44), "blocker undisturbed");
    }

    #[test]
    fn blast_with_combo_counter_deals_bonus_damage() {
        // Enemy at e5 with combo=2 (manually set). Caster at e4 with Blast.
        // Pre-tick combo 2 → +2 bonus damage. Enemy HP=2, armor=0 → killed.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Blast as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
        pos.mailbox[36] = pos.mailbox[36].with_combo(2);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Blast));
        assert!(!pos.is_occupied(36), "combo-bonus damage 2 + HP 2 → removed");
    }

    #[test]
    fn blast_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Blast as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
        pos.mailbox[36] = pos.mailbox[36].with_combo(1);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Blast));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Shove ----------------------------------------------------------

    #[test]
    fn shove_pushes_enemy_in_each_of_eight_directions() {
        // Caster e4 (28), enemy at d4 (27). Shove can push in any direction
        // because surrounding squares are empty. Verify each direction
        // results in the expected push_dest via choice_idx.
        // DELTAS order: 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW.
        // Enemy at d4 = sq 27 (rank 3, file 3). Expected push targets:
        // 0=N→35, 1=NE→36, 2=E→28(caster!), 3=SE→20, 4=S→19, 5=SW→18,
        // 6=W→26, 7=NW→34.
        // Direction 2 (E) toward sq 28 would push onto the caster — blocked,
        // generator wouldn't emit. We test 7 of the 8 directions here.
        let expected: [(u8, u8); 7] = [
            (0, 35), (1, 36), (3, 20), (4, 19),
            (5, 18), (6, 26), (7, 34),
        ];
        for (dir, push_dest) in expected {
            let mut pos = skill_phase_pos(2);
            place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
            equip(&mut pos, 28, Skill::Shove as u8);
            place(&mut pos, 27, Player::P2, PieceKind::Champion, 2, 0);

            let _ = make(&mut pos, shove_action(28, 27, dir));
            assert!(!pos.is_occupied(27), "enemy left dir={}", dir);
            assert!(pos.is_occupied(push_dest),
                    "enemy at push_dest={} for dir={}", push_dest, dir);
            assert!(pos.p2_pieces.contains(push_dest), "ownership preserved");
        }
    }

    #[test]
    fn shove_pushes_ally_does_not_tick_combo() {
        // Caster e4 (28), ally at d4 (27, combo=0). Shove N → d5 (35).
        // Ally push — combo on ally must remain 0.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shove as u8);
        place(&mut pos, 27, Player::P1, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, shove_action(28, 27, /*dir=N*/0));
        assert!(pos.is_occupied(35), "ally pushed to d5");
        assert_eq!(pos.mailbox[35].combo(), 0, "no combo tick on ally push");
    }

    #[test]
    fn shove_off_board_no_emission() {
        // Caster b1 (1), enemy a1 (0). Pushes W or any S-ish dir are
        // off-board; pushing E lands on caster (blocked); pushing N lands
        // on a2 (sq 8) — legal. Generator should emit some dirs and skip
        // off-board ones.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 1, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 1, Skill::Shove as u8);
        place(&mut pos, 0, Player::P2, PieceKind::Champion, 2, 0);

        let shove_id = Skill::Shove as u8;
        let dirs: Vec<u8> = generate(&pos).into_iter()
            .filter(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == shove_id
                && a.target() == 0)
            .map(|a| a.choice_idx())
            .collect();
        // SW/S/SE/W all push enemy off-board → must not appear.
        for forbidden in [3u8, 4, 5, 6] {
            assert!(!dirs.contains(&forbidden),
                    "dir {} would push a1 off-board", forbidden);
        }
    }

    #[test]
    fn shove_into_occupied_no_emission() {
        // Caster e4 (28), enemy d4 (27), blocker at d5 (35). Shove N (dir=0)
        // would land on the blocker → generator must skip dir=0.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shove as u8);
        place(&mut pos, 27, Player::P2, PieceKind::Champion, 2, 0);
        place(&mut pos, 35, Player::P1, PieceKind::Champion, 2, 0);

        let shove_id = Skill::Shove as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == shove_id
                && a.target() == 27
                && a.choice_idx() == 0),
            "Shove into occupied d5 must not be emitted"
        );
    }

    #[test]
    fn shove_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shove as u8);
        place(&mut pos, 27, Player::P2, PieceKind::Champion, 2, 0);
        let snap = pos.clone();

        let undo = make(&mut pos, shove_action(28, 27, 0));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Swap -----------------------------------------------------------

    #[test]
    fn swap_exchanges_champion_and_guard() {
        // Caster Champion at e4 (28), ally Guard at e5 (36). Swap → caster
        // ends at 36, Guard at 28. Verify kind-layer XOR works.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Swap as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Guard, 2, 1);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Swap));
        assert!(pos.guards.contains(28), "Guard now at 28");
        assert!(pos.champions.contains(36), "Champion now at 36");
        assert!(!pos.champions.contains(28));
        assert!(!pos.guards.contains(36));
        assert!(pos.p1_pieces.contains(28) && pos.p1_pieces.contains(36));
    }

    #[test]
    fn swap_ignores_moved_this_phase() {
        // moved_this_phase is Move-Phase only. Swap occurring in Skill-Phase
        // must not be affected by what's set there. Set both squares as
        // "moved" before swap; confirm swap proceeds and mailbox/bitboard
        // state is correct.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Swap as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Guard, 2, 1);
        pos.moved_this_phase = Bitboard::from_square(28) | Bitboard::from_square(36);

        let _ = make(&mut pos, skill_action(28, 36, Skill::Swap));
        // Skill-Phase: moved_this_phase is irrelevant to legality and not
        // mutated by skill resolvers.
        assert!(pos.champions.contains(36));
        assert!(pos.guards.contains(28));
    }

    #[test]
    fn swap_path_blocked_no_emission() {
        // Caster e4 (28) with Swap (range 2). Ally e5 (36) is fine (range 1).
        // Far ally e6 (44) is range 2 but blocked by e5 (the first piece on
        // the ray). path::skill_targets only returns first-blocker — so e6
        // must NOT be a Swap target.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Swap as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Guard, 2, 1);
        place(&mut pos, 44, Player::P1, PieceKind::Guard, 2, 1);

        let swap_id = Skill::Swap as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == swap_id
                && a.src() == 28
                && a.target() == 44),
            "Swap to e6 past blocker at e5 must not be emitted"
        );
    }

    #[test]
    fn swap_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Swap as u8);
        place(&mut pos, 36, Player::P1, PieceKind::Guard, 2, 1);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 36, Skill::Swap));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Retreat --------------------------------------------------------

    #[test]
    fn retreat_lands_adjacent_to_ally_guard() {
        // Caster Champion at e4 (28). Ally Guard at a1 (0). Retreat range 3,
        // queen-ray from e4. The diagonal SW reaches b1 (sq 1) at range 3,
        // which IS adjacent to a1 → legal Retreat dest.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Retreat as u8);
        place(&mut pos, 0, Player::P1, PieceKind::Guard, 2, 1);

        // e4 → b1: dr=-3, df=-3 — pure SW diagonal at Chebyshev 3 ✓.
        let _ = make(&mut pos, skill_action(28, 1, Skill::Retreat));
        assert!(!pos.is_occupied(28));
        assert!(pos.is_occupied(1));
        assert!(pos.champions.contains(1));
    }

    #[test]
    fn retreat_no_ally_guards_no_emission() {
        // Caster Champion at e4 (28) with Retreat. No ally Guards anywhere
        // → no Retreat actions emitted.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Retreat as u8);

        let retreat_id = Skill::Retreat as u8;
        let acts = generate(&pos);
        assert!(
            !acts.iter().any(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == retreat_id),
            "Retreat without ally Guards must not emit"
        );
    }

    #[test]
    fn retreat_dest_not_adjacent_to_guard_no_emission() {
        // Caster at a1 (0). Ally Guard at h8 (63). Caster has range 3
        // queen-ray destinations. h8 has 3 neighbours: g7 (54), g8 (62),
        // h7 (55). From a1, Chebyshev distances are g7=6, g8=7, h7=7.
        // All > range 3 → no Retreat destination is adjacent to any ally
        // Guard, so no emission.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 0, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 0, Skill::Retreat as u8);
        place(&mut pos, 63, Player::P1, PieceKind::Guard, 2, 1);

        let retreat_id = Skill::Retreat as u8;
        let acts = generate(&pos);
        let retreat_acts: Vec<_> = acts.iter()
            .filter(|a| a.kind() == ActionKind::Skill
                && a.skill_id() == retreat_id)
            .collect();
        assert!(retreat_acts.is_empty(),
                "no Retreat dest from a1 in range 3 is adj to h8, got {:?}",
                retreat_acts.iter().map(|a| a.target()).collect::<Vec<_>>());
    }

    #[test]
    fn retreat_unmake_roundtrip() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Retreat as u8);
        place(&mut pos, 0, Player::P1, PieceKind::Guard, 2, 1);
        let snap = pos.clone();

        let undo = make(&mut pos, skill_action(28, 1, Skill::Retreat));
        unmake(&mut pos, &undo);
        assert!(pos_eq(&snap, &pos), "{:?}", pos_diff(&snap, &pos));
    }

    // --- Cross-cutting --------------------------------------------------

    #[test]
    fn slice5_all_eight_skills_unmake_identity() {
        // Apply each Slice-5 skill in sequence on a fresh position; reverse-
        // unmake each undo and assert full pos_eq against the snapshot.
        // Each subtest builds its own position because the legal context for
        // each skill is different.
        let cases: Vec<(&str, Box<dyn Fn() -> (Position, Action)>)> = vec![
            ("Shield", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Shield as u8);
                (p, skill_action(28, 28, Skill::Shield))
            })),
            ("Heal", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Heal as u8);
                place(&mut p, 36, Player::P1, PieceKind::Champion, 1, 0);
                (p, skill_action(28, 36, Skill::Heal))
            })),
            ("Plate", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Plate as u8);
                place(&mut p, 36, Player::P1, PieceKind::Champion, 2, 0);
                (p, skill_action(28, 36, Skill::Plate))
            })),
            ("Dash", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Dash as u8);
                (p, skill_action(28, 36, Skill::Dash))
            })),
            ("Blast", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Blast as u8);
                place(&mut p, 36, Player::P2, PieceKind::Champion, 2, 0);
                (p, skill_action(28, 36, Skill::Blast))
            })),
            ("Shove", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Shove as u8);
                place(&mut p, 27, Player::P2, PieceKind::Champion, 2, 0);
                (p, shove_action(28, 27, 0))
            })),
            ("Swap", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Swap as u8);
                place(&mut p, 36, Player::P1, PieceKind::Guard, 2, 1);
                (p, skill_action(28, 36, Skill::Swap))
            })),
            ("Retreat", Box::new(|| {
                let mut p = skill_phase_pos(2);
                place(&mut p, 28, Player::P1, PieceKind::Champion, 2, 0);
                equip(&mut p, 28, Skill::Retreat as u8);
                place(&mut p, 0, Player::P1, PieceKind::Guard, 2, 1);
                (p, skill_action(28, 1, Skill::Retreat))
            })),
        ];
        for (name, build) in cases {
            let (mut pos, action) = build();
            let snap = pos.clone();
            let undo = make(&mut pos, action);
            unmake(&mut pos, &undo);
            assert!(pos_eq(&snap, &pos),
                    "{}: roundtrip diff: {:?}", name, pos_diff(&snap, &pos));
        }
    }

    #[test]
    fn dash_then_lance_uses_new_caster_position() {
        // Caster at e4 (28) with Dash + Lance. Enemy at h4 (sq 31) — out of
        // Lance range (1) from e4 but in range from e.g. g4 (30). Dash e4→g4
        // first (dir E, range 2), then Lance from g4 hits h4.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        // Equip both Dash (slot 1) and Lance (slot 2).
        pos.mailbox[28] = pos.mailbox[28]
            .with_skill1(Skill::Dash as u8)
            .with_skill2(Skill::Lance as u8);
        place(&mut pos, 31, Player::P2, PieceKind::Champion, 2, 0);

        let _ = make(&mut pos, skill_action(28, 30, Skill::Dash));
        assert!(pos.is_occupied(30), "caster dashed to g4");

        // Lance from g4 (30) targets h4 (31).
        let _ = make(&mut pos, skill_action(30, 31, Skill::Lance));
        assert_eq!(pos.mailbox[31].hp(), 1, "Lance dealt 1 dmg from new position");
    }

    #[test]
    fn blast_then_shove_chained_combo_on_same_enemy() {
        // Enemy at d5 (35) starting combo 0. Caster1 Champion at e4 (28) with
        // Blast pushes enemy NW out of the way and ticks combo. Caster2 (a
        // different Champion) Shoves the now-relocated enemy, ticks again
        // and applies pre-tick bonus.
        // After Blast: enemy at d6 (43), combo = 1.
        // Caster2 at d5 (35) Shoves enemy at d6 (43) east → enemy at e6 (44).
        // Pre-tick combo was 1 → 1 bonus damage applied; enemy HP 2 → 1.
        let mut pos = skill_phase_pos(4);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Blast as u8);
        place(&mut pos, 35, Player::P2, PieceKind::Champion, 2, 0);

        // Blast e4 → enemy at d5 (35). step_away(28,35): dr=signum(4-3)=1,
        // df=signum(3-4)=-1, so push goes from d5 to c6 (sq 42).
        let _ = make(&mut pos, skill_action(28, 35, Skill::Blast));
        assert!(pos.is_occupied(42), "enemy pushed to c6");
        assert_eq!(pos.mailbox[42].combo(), 1, "combo ticked to 1");

        // Now add caster2 at b6 (sq 41) with Shove, push enemy E.
        place(&mut pos, 41, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 41, Skill::Shove as u8);
        // Shove c6 (42) east → d6 (sq 43). dir=2 (E).
        let _ = make(&mut pos, shove_action(41, 42, 2));
        assert!(pos.is_occupied(43), "enemy at d6 after Shove");
        assert_eq!(pos.mailbox[43].hp(), 1, "pre-tick combo 1 dealt 1 bonus dmg");
        assert_eq!(pos.mailbox[43].combo(), 2, "combo ticked to 2");
    }

    #[test]
    fn skill_phase_budget_paper_curve() {
        // Paper baseline: R1–10:2, R11–20:3, R21–30:4, R31+:5… and unbounded
        // beyond. The "R31+:5" line in the paper rule sheet was shorthand
        // for the table cut-off, NOT a cap.
        assert_eq!(skill_phase_budget(1), 2);
        assert_eq!(skill_phase_budget(10), 2);
        assert_eq!(skill_phase_budget(11), 3);
        assert_eq!(skill_phase_budget(20), 3);
        assert_eq!(skill_phase_budget(21), 4);
        assert_eq!(skill_phase_budget(30), 4);
        assert_eq!(skill_phase_budget(31), 5);
        assert_eq!(skill_phase_budget(40), 5);
        // Crucially: it keeps climbing past 31.
        assert_eq!(skill_phase_budget(41), 6);
        assert_eq!(skill_phase_budget(50), 6);
        assert_eq!(skill_phase_budget(51), 7);
        assert_eq!(skill_phase_budget(100), 11);
    }

    // === Slice 6 — Focus / Charge / end_turn ===============================

    #[test]
    fn focus_sets_pending_bit_and_consumes_action() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Focus as u8);

        let pre_money = pos.p1_money;
        let _ = make(&mut pos, skill_action(28, 28, Skill::Focus));
        assert_ne!(pos.pending_modifiers & modifier_bits::FOCUS, 0);
        assert_eq!(pos.actions_remaining, 1);
        assert_eq!(pos.p1_money, pre_money - 1);
    }

    #[test]
    fn charge_sets_pending_bit_and_consumes_action() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Charge as u8);

        let pre_money = pos.p1_money;
        let _ = make(&mut pos, skill_action(28, 28, Skill::Charge));
        assert_ne!(pos.pending_modifiers & modifier_bits::CHARGE, 0);
        assert_eq!(pos.actions_remaining, 1);
        assert_eq!(pos.p1_money, pre_money - 3);
    }

    #[test]
    fn focus_is_not_consumed_by_charge() {
        // Stack-M (session-31): Focus = next non-Mystic skill. Casting Charge
        // (Mystic) while Focus is pending MUST leave Focus pending — so a
        // subsequent Strike skill gets BOTH buffs.
        let mut pos = skill_phase_pos(3);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        pos.mailbox[28] = pos.mailbox[28]
            .with_skill1(Skill::Focus as u8)
            .with_skill2(Skill::Charge as u8);

        let _ = make(&mut pos, skill_action(28, 28, Skill::Focus));
        assert_ne!(pos.pending_modifiers & modifier_bits::FOCUS, 0);
        let _ = make(&mut pos, skill_action(28, 28, Skill::Charge));
        // Focus must STILL be pending; Charge added on top.
        assert_ne!(pos.pending_modifiers & modifier_bits::FOCUS, 0,
            "Charge must not consume Focus");
        assert_ne!(pos.pending_modifiers & modifier_bits::CHARGE, 0);
    }

    #[test]
    fn charge_then_strike_grants_plus_one_damage() {
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Charge as u8);
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0); // e5

        pos.pending_modifiers |= modifier_bits::CHARGE;
        // Lance hits e5 with +1 from Charge → 2 damage. Target had HP 2 / armor 0,
        // so this removes it.
        equip(&mut pos, 28, Skill::Lance as u8);
        let _ = make(&mut pos, skill_action(28, 36, Skill::Lance));
        assert!(!pos.is_occupied(36), "Charge+Lance should KO a HP2/armor0 enemy");
        // CHARGE bit cleared.
        assert_eq!(pos.pending_modifiers & modifier_bits::CHARGE, 0);
    }

    #[test]
    fn focus_dash_retarget_moves_ally_two_tiles() {
        // Caster Champ at e4 (28), ally Champ at e5 (36). Focus pending.
        // Retarget Dash: aux_sq = 36 (ally), target = 52 (e7), 2 tiles N of ally.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Dash as u8);
        pos.pending_modifiers |= modifier_bits::FOCUS;

        let pre_money = pos.p1_money;
        let action = Action::encode_with_aux(
            28, 52, ActionKind::Skill, Skill::Dash as u8, 0, 36,
        );
        let _ = make(&mut pos, action);

        assert!( pos.is_occupied(28), "caster stays put");
        assert!(!pos.is_occupied(36), "ally moved off origin");
        assert!( pos.is_occupied(52), "ally arrived at destination");
        assert!( pos.champions.contains(52));
        // Caster (still at 28) pays — that's the only P1 piece paying.
        assert_eq!(pos.p1_money, pre_money - 3);
        // FOCUS bit consumed.
        assert_eq!(pos.pending_modifiers & modifier_bits::FOCUS, 0);
    }

    #[test]
    fn focus_shield_retarget_buffs_adjacent_ally() {
        // Caster Champ at e4 (28), ally Champ at e5 (36). Focus pending.
        // Retarget Shield: ally gets +1 armor (not the caster).
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shield as u8);
        pos.pending_modifiers |= modifier_bits::FOCUS;

        let action = Action::encode_with_aux(
            28, 36, ActionKind::Skill, Skill::Shield as u8, 0, 36,
        );
        let _ = make(&mut pos, action);

        assert_eq!(pos.mailbox[36].armor(), 1, "ally got +1 armor");
        assert_eq!(pos.mailbox[28].armor(), 0, "caster's armor unchanged");
    }

    #[test]
    fn focus_unmake_roundtrip() {
        // Focus + retarget Shield unmakes cleanly.
        let mut pos = skill_phase_pos(2);
        place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
        place(&mut pos, 36, Player::P1, PieceKind::Champion, 2, 0);
        equip(&mut pos, 28, Skill::Shield as u8);
        pos.pending_modifiers |= modifier_bits::FOCUS;
        let before = pos.clone();

        let action = Action::encode_with_aux(
            28, 36, ActionKind::Skill, Skill::Shield as u8, 0, 36,
        );
        let undo = make(&mut pos, action);
        unmake(&mut pos, &undo);
        assert!(pos_eq(&pos, &before), "diff: {:?}", pos_diff(&pos, &before));
    }

    #[test]
    fn end_turn_clears_pending_and_disburses_income() {
        // Skill Phase ends → EndTurn flips to_move, disburses income.
        let mut pos = skill_phase_pos(0); // P1's Skill Phase, 0 actions left
        pos.round_number = 1;
        pos.p1_money = 5;
        pos.p2_money = 5;
        pos.pending_modifiers = modifier_bits::FOCUS | modifier_bits::CHARGE;

        // EndPhase from Skill triggers end_turn.
        let undo = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        assert_eq!(pos.to_move, Player::P2, "flipped to P2");
        assert_eq!(pos.round_number, 1, "no bump (flip to P2)");
        assert_eq!(pos.current_phase, Phase::Move);
        assert_eq!(pos.actions_remaining, 2);
        assert_eq!(pos.pending_modifiers, 0);
        // P2 got income at round 1 = 2.
        assert_eq!(pos.p2_money, 7);
        assert_eq!(pos.p1_money, 5);

        // Unmake restores everything.
        unmake(&mut pos, &undo);
        assert_eq!(pos.to_move, Player::P1);
        assert_eq!(pos.p2_money, 5);
        assert_eq!(pos.pending_modifiers,
            modifier_bits::FOCUS | modifier_bits::CHARGE);
    }

    #[test]
    fn end_turn_p2_to_p1_bumps_round() {
        let mut pos = skill_phase_pos(0);
        pos.to_move = Player::P2;
        pos.round_number = 1;
        let undo = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        assert_eq!(pos.to_move, Player::P1);
        assert_eq!(pos.round_number, 2, "bump on P2 → P1 flip");
        unmake(&mut pos, &undo);
        assert_eq!(pos.round_number, 1, "unmake restores round");
    }

    #[test]
    fn end_turn_clears_combo_on_new_stm_pieces() {
        // P1's turn ended; P2's pieces should have their combo counters reset.
        let mut pos = skill_phase_pos(0);
        pos.to_move = Player::P1;
        place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0); // e5
        pos.mailbox[36] = pos.mailbox[36].with_combo(2);
        let _ = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        assert_eq!(pos.mailbox[36].combo(), 0,
            "new STM (P2) pieces have combo counter cleared");
    }

    // --- Slice 7: Zobrist ----------------------------------------------------

    use crate::state::zobrist;

    /// Sync the zobrist field with what `full_recompute` says. Test setup
    /// helpers (`place`, `equip`, `skill_phase_pos`) mutate fields directly
    /// without going through the make_unmake choke points, so the field stays
    /// at whatever `Position::empty()` left it. Tests that care about the
    /// hash call this once after setup.
    fn sync_zobrist(pos: &mut Position) {
        pos.zobrist = zobrist::full_recompute(pos);
    }

    #[test]
    fn zobrist_setup_nonzero() {
        // setup_stack_m: full board, P1 to move, Move phase, 2 actions, round 1.
        let pos = Position::setup_stack_m();
        assert_ne!(pos.zobrist, 0, "setup_stack_m has a non-zero zobrist");
        // And it equals the from-scratch recompute (the constructor wired it).
        assert_eq!(pos.zobrist, zobrist::full_recompute(&pos),
            "constructor's zobrist matches full_recompute");
    }

    #[test]
    fn zobrist_distinct_positions_differ() {
        // A position before and after a one-square move must hash differently.
        let before = Position::setup_stack_m();
        let mut after = before.clone();
        let a = Action::encode(9, 17, ActionKind::Move, 0, 0); // b2 → b3
        let _ = make(&mut after, a);
        assert_ne!(before.zobrist, after.zobrist,
            "single-square move must change the hash");
    }

    #[test]
    fn zobrist_recompute_matches_incremental() {
        // After a battery of actions from a Stack-M setup, the incremental
        // hash must equal what `full_recompute` produces from scratch.
        let mut pos = Position::setup_stack_m();

        // Action 1: plain move b2→b3.
        let _u1 = make(&mut pos, Action::encode(9, 17, ActionKind::Move, 0, 0));
        assert_eq!(pos.zobrist, zobrist::full_recompute(&pos),
            "after plain move: incremental matches full_recompute");

        // Action 2: another plain move c2→c3.
        let _u2 = make(&mut pos, Action::encode(10, 18, ActionKind::Move, 0, 0));
        assert_eq!(pos.zobrist, zobrist::full_recompute(&pos),
            "after second plain move: incremental matches full_recompute");

        // Action 3: end Move Phase (Move→Skill transition).
        let _u3 = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        assert_eq!(pos.zobrist, zobrist::full_recompute(&pos),
            "after end-phase: incremental matches full_recompute");

        // Action 4: end Skill Phase (triggers end_turn — flip + income + reset).
        let _u4 = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        assert_eq!(pos.zobrist, zobrist::full_recompute(&pos),
            "after end-turn: incremental matches full_recompute");
    }

    #[test]
    fn zobrist_make_unmake_zero_delta() {
        // Battery of representative actions from a known-good Stack-M setup:
        // make then unmake must leave zobrist exactly where it started.
        let pos0 = Position::setup_stack_m();

        // 1. Plain move.
        {
            let mut pos = pos0.clone();
            let snap = pos.zobrist;
            let undo = make(&mut pos, Action::encode(9, 17, ActionKind::Move, 0, 0));
            unmake(&mut pos, &undo);
            assert_eq!(pos.zobrist, snap, "plain move round-trip preserves zobrist");
        }

        // 2. End Move Phase (no actions consumed).
        {
            let mut pos = pos0.clone();
            let snap = pos.zobrist;
            let undo = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
            unmake(&mut pos, &undo);
            assert_eq!(pos.zobrist, snap, "end-phase round-trip preserves zobrist");
        }

        // 3. Skill action (Lance) — uses the same harness as the strike tests.
        {
            let mut pos = skill_phase_pos(2);
            place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
            equip(&mut pos, 28, Skill::Lance as u8);
            place(&mut pos, 36, Player::P2, PieceKind::Champion, 2, 0);
            sync_zobrist(&mut pos);
            let snap = pos.zobrist;
            let undo = make(&mut pos, skill_action(28, 36, Skill::Lance));
            unmake(&mut pos, &undo);
            assert_eq!(pos.zobrist, snap, "Lance round-trip preserves zobrist");
        }

        // 4. Focus (pending_modifier mutation).
        {
            let mut pos = skill_phase_pos(2);
            place(&mut pos, 28, Player::P1, PieceKind::Champion, 2, 0);
            equip(&mut pos, 28, Skill::Focus as u8);
            sync_zobrist(&mut pos);
            let snap = pos.zobrist;
            let undo = make(&mut pos, skill_action(28, 28, Skill::Focus));
            unmake(&mut pos, &undo);
            assert_eq!(pos.zobrist, snap, "Focus round-trip preserves zobrist");
        }

        // 5. End-turn (Skill→Move + flip + income + round bump on P2→P1).
        {
            let mut pos = skill_phase_pos(0);
            pos.to_move = Player::P2;
            pos.round_number = 1;
            sync_zobrist(&mut pos);
            let snap = pos.zobrist;
            let undo = make(&mut pos, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
            unmake(&mut pos, &undo);
            assert_eq!(pos.zobrist, snap, "end-turn round-trip preserves zobrist");
        }
    }

    #[test]
    fn zobrist_transposition_same_hash() {
        // Two move-orderings that arrive at the same Move-Phase end-state must
        // produce identical hashes. P1 plays two plain moves in different
        // orders, ends the Move Phase, ends the Skill Phase: state is the
        // same end-of-turn position both ways, so zobrist must match.
        //
        // Sequence A: b2→b3, c2→c3, EndPhase (Move→Skill), EndPhase (end_turn)
        // Sequence B: c2→c3, b2→b3, EndPhase (Move→Skill), EndPhase (end_turn)

        let mut a = Position::setup_stack_m();
        let _ = make(&mut a, Action::encode(9,  17, ActionKind::Move, 0, 0));
        let _ = make(&mut a, Action::encode(10, 18, ActionKind::Move, 0, 0));
        let _ = make(&mut a, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        let _ = make(&mut a, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));

        let mut b = Position::setup_stack_m();
        let _ = make(&mut b, Action::encode(10, 18, ActionKind::Move, 0, 0));
        let _ = make(&mut b, Action::encode(9,  17, ActionKind::Move, 0, 0));
        let _ = make(&mut b, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        let _ = make(&mut b, Action::encode(0, 0, ActionKind::EndPhase, 0, 0));

        assert_eq!(a.zobrist, b.zobrist,
            "different move orderings reaching the same end-of-turn state must hash equal");
        // And the broader position state must agree too (sanity check).
        assert!(pos_eq(&a, &b), "transposition diff: {:?}", pos_diff(&a, &b));
    }
}
