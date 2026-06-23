//! Legal primitive-action generator. Reads the Position, emits the set of
//! legal Actions for the current player and phase.
//!
//! Per the audit decision: path-implicit destinations (Retreat) are
//! pre-resolved here — the generator emits one Action per legal landing
//! square. Direction-only skills (Shove) emit one Action per legal
//! direction using `Action::choice_idx`. AOE skills emit a single Action;
//! the AOE expansion happens inside `make()`.
//!
//! # Move Phase generation (Stack M, slice 1)
//!
//! For each piece of the side-to-move whose square is **not** set in
//! `pos.moved_this_phase`:
//!
//! - **Plain moves**: every empty square reachable within the piece's speed
//!   (Guard=2, Champion=1, King=1) by *any* path of single-tile steps.
//!   Movement is free in all 8 directions per Stack M — speed is Chebyshev
//!   distance (a diagonal step costs 1). Zigzag is legal — the destination
//!   need not be reached along a straight line, only via a path of empty
//!   intermediate squares.
//!
//! - **Move-Attacks**: every enemy-occupied square reachable as above. The
//!   mover does NOT enter the target tile; the enemy takes 1 damage. For
//!   each such target, enumerate Bodyguard-redirect choices via
//!   `Action::choice_idx`:
//!     - `choice_idx = 0` → no redirect (defender takes the hit).
//!     - `choice_idx = k` (1..=N) → redirect to the k-th adjacent friendly
//!       Guard of the defender (canonical: ascending square index).
//!   Search uses all variants directly; HvH UI presents the choice to the
//!   defender between generator output and `make()`.
//!
//! - `moved_this_phase` blocks the *current* square of a piece that has
//!   already moved this Move Phase. A plain move clears the src bit and sets
//!   the dest bit; a Move-Attack keeps the src bit set on the (unmoved)
//!   attacker square (the attacker spent its phase action even though it
//!   didn't relocate).
//!
//! - `EndPhase` is always legal in the Move Phase. Becomes the only legal
//!   action when no piece has a legal move/attack remaining.
//!
//! # Skill Phase generation
//!
//! Skills use Path/Range/Block rules (queen-style straight lines, blocked by
//! any piece). Range buffs from `pending_modifiers` (Focus) apply. Charge
//! affects damage but does not change legality. Implementation defers to
//! slice 4. Until then, the only legal Skill-Phase action is `EndPhase`.

use super::action::{Action, ActionKind};
use super::skills::{self, Skill, TargetOwner};
use crate::state::{magic, path};
use crate::state::position::{Phase, Player};
use crate::state::{Bitboard, Position};

// Mirrors the constants in make_unmake.rs. Kept private to the generator —
// these are stack-M tuning values; promoting them to a shared module is a
// later refactor when more sites need them.
const ARMOR_CAP: u8 = 2;
const INJURED_HP: u8 = 1;

pub fn generate(pos: &Position) -> Vec<Action> {
    // Terminal positions emit no further legal actions. The game-over signal
    // (Stack M: "The game ends immediately when a King is removed") is
    // authoritative; downstream code (search, UI) interprets the empty list
    // together with `pos.game_result` to drive end-of-game flow.
    if pos.game_result.is_some() {
        return Vec::new();
    }
    match pos.current_phase {
        Phase::Move  => generate_move_phase(pos),
        Phase::Skill => generate_skill_phase(pos),
    }
}

// === Move Phase =============================================================

fn generate_move_phase(pos: &Position) -> Vec<Action> {
    let mut out = Vec::with_capacity(64);

    if pos.actions_remaining == 0 {
        // Only legal action: end the phase.
        out.push(Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        return out;
    }

    let stm_bb = side_to_move_bb(pos);
    let opp_bb = opponent_bb(pos);
    let occ    = stm_bb | opp_bb;

    // Iterate over each piece of the side-to-move that hasn't moved this phase.
    let movable = Bitboard(stm_bb.0 & !pos.moved_this_phase.0);
    for src in iter_squares(movable) {
        let speed = piece_speed(pos, src);
        let (reach_empty, reach_attack, dist) = reachable(src, speed, occ, opp_bb);

        // Plain moves: every empty reachable square.
        for dest in iter_squares(reach_empty) {
            out.push(Action::encode(src, dest, ActionKind::Move, 0, 0));
        }

        // Move-Attacks: for every enemy reachable in <= speed steps, enumerate
        // every empty neighbour `approach_sq` reachable in <= speed-1 steps.
        // Each distinct approach_sq is a distinct legal action because:
        // - the attacker physically ends on approach_sq (different end square
        //   = different observable outcome);
        // - Bodyguard eligibility is dual-adjacency (target-AND-approach), so
        //   different approach_sqs can produce different eligible-Guard sets,
        //   including the "zig-zag bypass" case where one approach has zero
        //   eligible Guards.
        for tgt in iter_squares(reach_attack) {
            for approach in eight_neighbours(tgt) {
                // approach must be reachable via empties in <= speed-1 steps
                // (src itself counts, dist[src]=0).
                let d = dist[approach as usize];
                if d == 255 { continue; }
                if d as u32 + 1 > speed as u32 { continue; }

                // For each (src, tgt, approach), enumerate Bodyguard choices.
                let bg_guards = bodyguard_guards_for(pos, tgt, approach);
                // choice_idx = 0 → no redirect, defender takes the hit.
                out.push(Action::encode_move_attack(src, tgt, 0, approach));
                // choice_idx = k → redirect to k-th eligible Guard.
                for (k, _guard_sq) in bg_guards.into_iter().enumerate() {
                    let choice_idx = (k as u8) + 1;
                    if choice_idx > 15 { break; } // 4-bit limit (never hit in Stack M)
                    out.push(Action::encode_move_attack(src, tgt, choice_idx, approach));
                }
            }
        }
    }

    // EndPhase is always legal in Move Phase.
    out.push(Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
    out
}

// === Skill Phase ============================================================

/// Enumerate Skill-Phase actions. Slices 3–6 wire the per-skill emission;
/// all 15 resolvers ship.
///
/// For each caster on the side-to-move with money ≥ skill cost, emit one
/// action per legal target by `TargetOwner`:
///   - `SelfOnly`  → one action with `src == tgt` (Shield filtered if at cap).
///                   When Focus is pending, Shield/Dash/Retreat additionally
///                   emit retarget actions onto adjacent allies (aux_sq).
///   - `Ally`      → `path::skill_targets` ∩ allies; Heal/Plate state filters.
///   - `Enemy`     → `path::skill_targets` ∩ enemies.
///   - `Either`    → Shove only; emits one action per (target, dir). When
///                   Focus is pending, also emits focus-effect mode (2-tile push).
///   - `Empty`     → Dash / Retreat; empty squares on queen-rays within range,
///                   Retreat additionally constrained to be adjacent to an
///                   ally Guard.
///
/// Focus (`pending_modifiers & FOCUS`): +1 activation Range for ALL non-Mystic
/// skills. Per oq-70 + session-31, Focus on Self-only-or-movement skills also
/// surfaces retarget / effect-range interpretations as ADDITIONAL legal
/// branches. The caster paying the Focus tax does NOT consume Focus by
/// casting a Mystic skill (that's the new rule: Focus = "next non-Mystic").
/// Focus / Charge stacking constraints:
///   - Focus while Focus already pending: filtered out (illegal — max 1 stack).
///   - Charge while Charge already pending: filtered out (illegal — max 1 stack).
fn generate_skill_phase(pos: &Position) -> Vec<Action> {
    let mut out = Vec::with_capacity(64);

    if pos.actions_remaining == 0 {
        out.push(Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
        return out;
    }

    let stm_bb = side_to_move_bb(pos);
    let opp_bb = opponent_bb(pos);
    let money = match pos.to_move {
        Player::P1 => pos.p1_money,
        Player::P2 => pos.p2_money,
    };
    let focus_pending = pos.pending_modifiers
        & crate::state::position::modifier_bits::FOCUS != 0;
    let charge_pending = pos.pending_modifiers
        & crate::state::position::modifier_bits::CHARGE != 0;

    for src in iter_squares(stm_bb) {
        let entry = pos.mailbox[src as usize];
        for slot_id in [entry.skill1(), entry.skill2()] {
            let Some(skill) = skills::skill_from_id(slot_id) else { continue };
            if (skills::skill_cost(skill) as u16) > money { continue; }

            // Max-1-stack filter: cannot cast Focus while Focus is pending,
            // cannot cast Charge while Charge is pending.
            if skill == Skill::Focus && focus_pending { continue; }
            if skill == Skill::Charge && charge_pending { continue; }

            // Focus's +1 Range buff applies to every NON-Mystic skill. The
            // resolver in `apply_skill` is what actually clears the FOCUS bit;
            // we just use the buffed range here so legal-action enumeration
            // matches what the resolver will accept.
            let is_mystic = matches!(skills::skill_category(skill),
                skills::SkillCategory::Mystic);
            let base_range = skills::skill_default_range(skill);
            let range = if focus_pending && !is_mystic { base_range + 1 } else { base_range };

            match skills::skill_target_owner(skill) {
                TargetOwner::SelfOnly => {
                    // Default branch: cast on self.
                    let self_legal = match skill {
                        // Shield at the Armor cap is illegal cast (no effect).
                        Skill::Shield => entry.armor() < ARMOR_CAP,
                        Skill::Focus | Skill::Charge => true,
                        _ => true,
                    };
                    if self_legal {
                        out.push(Action::encode(
                            src, src, ActionKind::Skill, skill as u8, 0,
                        ));
                    }
                    // Focus retarget branches: Shield/Dash/Retreat may channel
                    // the skill onto an adjacent ally (1-tile away). When
                    // Focus is pending, generate those as separate actions
                    // carrying `aux_sq`.
                    if focus_pending {
                        match skill {
                            Skill::Shield => emit_focus_shield_retargets(
                                pos, src, skill, stm_bb, &mut out),
                            // Dash/Retreat are TargetOwner::Empty in our
                            // model, so retarget for those is emitted in the
                            // Empty arm below. (Their default-targets are
                            // EmptySq, not SelfOnly — only Shield is SelfOnly
                            // among the retargetable-by-Focus skills.)
                            _ => {}
                        }
                    }
                }
                TargetOwner::Empty => {
                    // Move-skills (Dash/Retreat): enumerate empty squares on
                    // queen-rays from `src` within `range`.
                    let occ = (pos.p1_pieces | pos.p2_pieces).0;
                    let attacks = magic::skill_attacks(src, occ, range).0;
                    let empties = attacks & !occ;
                    if empties == 0 && !focus_pending { continue; }
                    match skill {
                        Skill::Dash => {
                            for dest in iter_squares(Bitboard(empties)) {
                                out.push(Action::encode(
                                    src, dest, ActionKind::Skill, skill as u8, 0,
                                ));
                            }
                            // Focus retarget: an adjacent ally moves up to 2.
                            if focus_pending {
                                emit_focus_dash_retargets(pos, src, skill, stm_bb, &mut out);
                            }
                        }
                        Skill::Retreat => {
                            let ally_guards = stm_bb.0 & pos.guards.0;
                            if ally_guards != 0 {
                                for dest in iter_squares(Bitboard(empties)) {
                                    let adj_to_ally_guard = eight_neighbours(dest)
                                        .any(|n| ally_guards & (1u64 << n) != 0);
                                    if !adj_to_ally_guard { continue; }
                                    out.push(Action::encode(
                                        src, dest, ActionKind::Skill, skill as u8, 0,
                                    ));
                                }
                            }
                            // Focus retarget: an adjacent ally retreats.
                            if focus_pending && ally_guards != 0 {
                                emit_focus_retreat_retargets(
                                    pos, src, skill, stm_bb, ally_guards, &mut out);
                            }
                        }
                        _ => unreachable!("only Dash/Retreat use TargetOwner::Empty"),
                    }
                }
                TargetOwner::Ally => {
                    let raw = path::skill_targets(pos, src, range).0;
                    let filtered = raw & stm_bb.0;
                    for tgt in iter_squares(Bitboard(filtered)) {
                        let tgt_entry = pos.mailbox[tgt as usize];
                        match skill {
                            Skill::Heal  if tgt_entry.hp()    != INJURED_HP => continue,
                            Skill::Plate if tgt_entry.armor() >= ARMOR_CAP  => continue,
                            _ => {}
                        }
                        out.push(Action::encode(
                            src, tgt, ActionKind::Skill, skill as u8, 0,
                        ));
                    }
                }
                TargetOwner::Enemy => {
                    let raw = path::skill_targets(pos, src, range).0;
                    let filtered = raw & opp_bb.0;
                    for tgt in iter_squares(Bitboard(filtered)) {
                        out.push(Action::encode(
                            src, tgt, ActionKind::Skill, skill as u8, 0,
                        ));
                    }
                    // Focus-effect mode for Blast: push 2 instead of 1. We
                    // can only land where intermediate + final squares allow
                    // it — the resolver gracefully truncates to 1-tile, so a
                    // separate enumeration here is needed only to give the
                    // search the *option*. Emit the focus-effect variant at
                    // the *unbuffed* activation range (Focus picks one or
                    // the other interpretation, not both at once).
                    if focus_pending && skill == Skill::Blast {
                        let raw_eff = path::skill_targets(pos, src, base_range).0;
                        let filtered_eff = raw_eff & opp_bb.0;
                        for tgt in iter_squares(Bitboard(filtered_eff)) {
                            out.push(Action::encode_focus_effect(
                                src, tgt, ActionKind::Skill, skill as u8, 0,
                            ));
                        }
                    }
                }
                TargetOwner::Either => {
                    // Currently only Shove. Emit one action per (target, dir)
                    // where the push lands on-board and onto an empty square.
                    debug_assert_eq!(skill, Skill::Shove);
                    let raw = path::skill_targets(pos, src, range).0;
                    let occ = (pos.p1_pieces | pos.p2_pieces).0;
                    for tgt in iter_squares(Bitboard(raw)) {
                        for dir in 0..8u8 {
                            let Some(push_dest) = magic::neighbour_in_dir(tgt, dir as usize)
                                else { continue };
                            if occ & (1u64 << push_dest) != 0 { continue; }
                            out.push(Action::encode(
                                src, tgt, ActionKind::Skill, skill as u8, dir,
                            ));
                        }
                    }
                    // Focus-effect: 2-tile push. Both intermediate and final
                    // squares must be empty + on-board. Activation range is
                    // unbuffed (Focus picks effect-mode, not activation-mode).
                    if focus_pending {
                        let raw_eff = path::skill_targets(pos, src, base_range).0;
                        for tgt in iter_squares(Bitboard(raw_eff)) {
                            for dir in 0..8u8 {
                                let Some(step1) = magic::neighbour_in_dir(tgt, dir as usize)
                                    else { continue };
                                if occ & (1u64 << step1) != 0 { continue; }
                                let Some(step2) = magic::neighbour_in_dir(step1, dir as usize)
                                    else { continue };
                                if occ & (1u64 << step2) != 0 { continue; }
                                out.push(Action::encode_focus_effect(
                                    src, tgt, ActionKind::Skill, skill as u8, dir,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    out.push(Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
    out
}

// === Focus retarget helpers (Slice 6) ======================================

/// Emit Shield-retarget actions when Focus is pending. The caster (src) may
/// channel its Shield onto an adjacent friendly piece (not at Armor cap).
/// The `aux_sq` carries the recipient ally; `target` mirrors `aux_sq` for
/// human readability (the recipient *is* the target conceptually).
fn emit_focus_shield_retargets(
    pos: &Position, src: u8, skill: Skill, stm_bb: Bitboard, out: &mut Vec<Action>,
) {
    for n in eight_neighbours(src) {
        if !stm_bb.contains(n) { continue; }
        if pos.mailbox[n as usize].armor() >= ARMOR_CAP { continue; }
        out.push(Action::encode_with_aux(
            src, n, ActionKind::Skill, skill as u8, 0, n,
        ));
    }
}

/// Emit Dash-retarget actions when Focus is pending. The caster (src) stays
/// put; an *adjacent ally* moves up to 2 tiles (queen-ray) onto an empty
/// square. `target` = destination, `aux_sq` = the ally (mover).
fn emit_focus_dash_retargets(
    pos: &Position, src: u8, skill: Skill, stm_bb: Bitboard, out: &mut Vec<Action>,
) {
    let occ = (pos.p1_pieces | pos.p2_pieces).0;
    for n in eight_neighbours(src) {
        if !stm_bb.contains(n) { continue; }
        // Range 2 from the ally, queen-rays.
        let attacks = magic::skill_attacks(n, occ, 2).0;
        let empties = attacks & !occ;
        for dest in iter_squares(Bitboard(empties)) {
            // Caster stays at src; ally moves n -> dest. Don't let dest == src.
            if dest == src { continue; }
            out.push(Action::encode_with_aux(
                src, dest, ActionKind::Skill, skill as u8, 0, n,
            ));
        }
    }
}

/// Emit Retreat-retarget actions when Focus is pending. An *adjacent ally*
/// retreats onto an empty square adjacent to a friendly Guard. Range from the
/// ally is the unbuffed default (3) — Focus picked the retarget interpretation,
/// not the activation buff.
fn emit_focus_retreat_retargets(
    pos: &Position, src: u8, skill: Skill, stm_bb: Bitboard,
    ally_guards: u64, out: &mut Vec<Action>,
) {
    let occ = (pos.p1_pieces | pos.p2_pieces).0;
    let base_range = skills::skill_default_range(skill);
    for n in eight_neighbours(src) {
        if !stm_bb.contains(n) { continue; }
        let attacks = magic::skill_attacks(n, occ, base_range).0;
        let empties = attacks & !occ;
        for dest in iter_squares(Bitboard(empties)) {
            if dest == src { continue; }
            let adj_to_ally_guard = eight_neighbours(dest)
                .any(|g| ally_guards & (1u64 << g) != 0);
            if !adj_to_ally_guard { continue; }
            out.push(Action::encode_with_aux(
                src, dest, ActionKind::Skill, skill as u8, 0, n,
            ));
        }
    }
}

// === Reachability ===========================================================

/// Chebyshev-BFS from `src` bounded by `speed`.
///
/// Returns `(reach_empty, reach_attack)`:
///   - `reach_empty`: empty squares reachable (the destination is empty AND
///     every intermediate square on *some* path is empty).
///   - `reach_attack`: enemy squares reachable as a Move-Attack target. The
///     target itself is enemy-occupied; *intermediate* squares must be empty.
///     Specifically: enemy square `t` is in `reach_attack` iff some square
///     `s` adjacent to `t` is in `reach_empty ∪ {src}` AND `dist(src, s) +
///     1 ≤ speed` (i.e. you reach `s` in ≤ speed-1 steps, then "step" onto
///     `t` as the move-attack).
///
/// `occ` is the full occupancy bitboard (both sides), `opp_bb` is the
/// opponent's occupancy.
///
/// Returns `(reach_empty, reach_attack, dist)` where `dist[sq]` is the minimum
/// Chebyshev BFS distance from `src` through empty squares (255 if unreached;
/// 0 at `src` itself). The caller uses `dist` to enumerate every valid
/// *approach square* per Move-Attack target (those neighbours `n` of target
/// with `dist[n] ≤ speed - 1`). `src` always has `dist[src] = 0` and counts
/// as a valid approach when it is adjacent to the target (speed-1 case).
fn reachable(src: u8, speed: u8, occ: Bitboard, opp_bb: Bitboard)
    -> (Bitboard, Bitboard, [u8; 64])
{
    // dist[sq] = minimum Chebyshev steps from src, or 255 if unreached.
    let mut dist = [255u8; 64];
    dist[src as usize] = 0;

    // BFS frontier — stays small (max 25 squares within Chebyshev radius 2,
    // 9 within radius 1). A simple Vec is fine.
    let mut frontier: Vec<u8> = Vec::with_capacity(16);
    frontier.push(src);

    let mut reach_empty = Bitboard::EMPTY;

    for step in 1..=speed {
        let mut next_frontier: Vec<u8> = Vec::with_capacity(16);
        for &sq in &frontier {
            for n in eight_neighbours(sq) {
                if dist[n as usize] != 255 {
                    continue; // already seen via shorter/equal path
                }
                if occ.contains(n) {
                    continue; // blocked — cannot enter or pass through
                }
                dist[n as usize] = step;
                reach_empty = reach_empty | Bitboard::from_square(n);
                next_frontier.push(n);
            }
        }
        frontier = next_frontier;
        if frontier.is_empty() { break; }
    }

    // Move-Attack targets: enemy squares adjacent to any square `s` where
    // `dist[s] ≤ speed - 1` (so stepping from `s` onto the enemy counts as
    // the final move-step). Equivalently: any enemy at Chebyshev distance
    // ≤ speed from src whose neighbour-set intersects `reach_empty ∪ {src}`
    // for squares with `dist ≤ speed - 1`.
    let mut reach_attack = Bitboard::EMPTY;
    for enemy in iter_squares(opp_bb) {
        // Quick reject by Chebyshev distance from src.
        if chebyshev_distance(src, enemy) > speed {
            continue;
        }
        // Find any neighbour `n` of `enemy` that is reachable in <= speed-1 steps.
        // Note: src itself counts (dist=0).
        for n in eight_neighbours(enemy) {
            let d = dist[n as usize];
            if d != 255 && d as u32 + 1 <= speed as u32 {
                reach_attack = reach_attack | Bitboard::from_square(enemy);
                break;
            }
        }
    }

    (reach_empty, reach_attack, dist)
}

// === Bodyguard ==============================================================

/// Returns the squares of friendly-to-the-defender Guards that are eligible to
/// intercept a Move-Attack against `target_sq` when the attacker advances to
/// `approach_sq` (the penultimate tile along the attack path), in canonical
/// ascending-square-index order.
///
/// Stack M canonical rule: "the defender may choose to have a Guard intercept
/// — if a friendly Guard is on a tile adjacent to BOTH the tile immediately
/// before the target (along the attack path) AND the defending piece."
///
/// Both adjacencies are Chebyshev (8-neighbour). A Guard adjacent to the
/// defender but NOT to `approach_sq` is *not* eligible — this is the
/// "zig-zag bypass" mechanic: a speed-2 Guard attacker can pick an approach
/// square that sidesteps Bodyguard entirely.
///
/// Returns an empty Vec if:
/// - the target is itself a Guard (Bodyguard only protects Champions/Kings);
/// - target is not a defender's Champion or King;
/// - no friendly-to-the-defender Guard satisfies dual-adjacency. The
///   approach_sq itself is excluded by construction (it's empty along the
///   BFS path), and the target square is excluded (it's the defender).
pub(super) fn bodyguard_guards_for(pos: &Position, target_sq: u8, approach_sq: u8) -> Vec<u8> {
    // Only Champions and Kings are Bodyguard-eligible defenders.
    if pos.guards.contains(target_sq) {
        return Vec::new();
    }
    if !pos.champions.contains(target_sq) && !pos.kings.contains(target_sq) {
        return Vec::new();
    }
    let defender_bb = if pos.p1_pieces.contains(target_sq) {
        pos.p1_pieces
    } else {
        pos.p2_pieces
    };
    // Build a bitmask of approach_sq's neighbours for cheap dual-adjacency check.
    let approach_neighbours: u64 = {
        let mut m = 0u64;
        for n in eight_neighbours(approach_sq) {
            m |= 1u64 << n;
        }
        m
    };
    let mut out = Vec::with_capacity(8);
    for n in eight_neighbours(target_sq) {
        if !defender_bb.contains(n) || !pos.guards.contains(n) {
            continue;
        }
        // Dual-adjacency: Guard must also be adjacent to approach_sq.
        if approach_neighbours & (1u64 << n) == 0 {
            continue;
        }
        out.push(n);
    }
    out.sort_unstable();
    out
}

// === Helpers ================================================================

#[inline]
fn side_to_move_bb(pos: &Position) -> Bitboard {
    match pos.to_move {
        Player::P1 => pos.p1_pieces,
        Player::P2 => pos.p2_pieces,
    }
}

#[inline]
fn opponent_bb(pos: &Position) -> Bitboard {
    match pos.to_move {
        Player::P1 => pos.p2_pieces,
        Player::P2 => pos.p1_pieces,
    }
}

#[inline]
fn piece_speed(pos: &Position, sq: u8) -> u8 {
    if pos.guards.contains(sq) { 2 } else { 1 }
}

#[inline]
fn iter_squares(bb: Bitboard) -> impl Iterator<Item = u8> {
    let mut bits = bb.0;
    std::iter::from_fn(move || {
        if bits == 0 {
            None
        } else {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            Some(sq)
        }
    })
}

/// 8-neighbour squares of `sq`, edge-clipped. Returns 3..=8 squares.
pub(super) fn eight_neighbours(sq: u8) -> impl Iterator<Item = u8> {
    let rank = (sq / 8) as i8;
    let file = (sq % 8) as i8;
    const DELTAS: [(i8, i8); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        ( 0, -1),          ( 0, 1),
        ( 1, -1), ( 1, 0), ( 1, 1),
    ];
    DELTAS.iter().filter_map(move |&(dr, df)| {
        let r = rank + dr;
        let f = file + df;
        if (0..8).contains(&r) && (0..8).contains(&f) {
            Some((r * 8 + f) as u8)
        } else {
            None
        }
    })
}

#[inline]
fn chebyshev_distance(a: u8, b: u8) -> u8 {
    let ar = (a / 8) as i8; let af = (a % 8) as i8;
    let br = (b / 8) as i8; let bf = (b % 8) as i8;
    let dr = (ar - br).unsigned_abs();
    let df = (af - bf).unsigned_abs();
    dr.max(df)
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::position::Position;

    /// Helper: just the unique destination squares (ignoring choice_idx
    /// expansion) for a given src.
    fn dests_from(actions: &[Action], src: u8) -> Vec<u8> {
        let mut out: Vec<u8> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Move && a.src() == src)
            .map(|a| a.target())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    // ---- Reachability primitives --------------------------------------

    #[test]
    fn champion_speed_1_lone_centre() {
        // Place a single P1 Champion at e4 (rank 3, file 4 → sq 28) on an
        // otherwise empty board. Speed 1 → all 8 neighbours legal.
        let mut p = Position::empty();
        p.p1_pieces = Bitboard::from_square(28);
        p.champions = Bitboard::from_square(28);
        p.to_move = Player::P1;
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;

        let (empty, attack, _dist) = reachable(28, 1, p.p1_pieces, Bitboard::EMPTY);
        assert_eq!(empty.count(), 8, "8 neighbours empty");
        assert_eq!(attack.count(), 0, "no enemies");
    }

    #[test]
    fn guard_speed_2_lone_centre() {
        // Single P1 Guard at d4 (sq 27, rank 3 file 3). Speed 2.
        // Reachable empties: the 5×5 Chebyshev-r=2 region minus the centre
        // (24 squares) minus the corners that BFS still reaches (all of them
        // do, since the board is empty). So 24 squares.
        let p1 = Bitboard::from_square(27);
        let (empty, attack, _dist) = reachable(27, 2, p1, Bitboard::EMPTY);
        assert_eq!(empty.count(), 24);
        assert_eq!(attack.count(), 0);
    }

    #[test]
    fn guard_blocked_intermediate() {
        // P1 Guard at a1 (sq 0). All three Chebyshev-1 neighbours (b1, a2, b2)
        // occupied by allies — Guard cannot reach ANY Chebyshev-2 square
        // because every BFS-step-1 launch pad is blocked.
        let mut p = Position::empty();
        let g = 0u8;     // a1
        let n_b1 = 1u8;  // b1
        let n_a2 = 8u8;  // a2
        let n_b2 = 9u8;  // b2
        p.p1_pieces = Bitboard::from_square(g)
            | Bitboard::from_square(n_b1)
            | Bitboard::from_square(n_a2)
            | Bitboard::from_square(n_b2);
        p.guards    = Bitboard::from_square(g);
        p.champions = Bitboard::from_square(n_b1)
            | Bitboard::from_square(n_a2)
            | Bitboard::from_square(n_b2);

        let occ = p.p1_pieces;
        let (empty, _attack, _dist) = reachable(g, 2, occ, Bitboard::EMPTY);
        // No Chebyshev-1 neighbours reachable (all occupied by allies).
        assert!(!empty.contains(n_b1));
        assert!(!empty.contains(n_a2));
        assert!(!empty.contains(n_b2));
        // And therefore no Chebyshev-2 square reachable either — frontier dies.
        assert!(!empty.contains(16), "a3 unreachable");
        assert!(!empty.contains(17), "b3 unreachable");
        assert!(!empty.contains(18), "c3 unreachable");
        assert!(!empty.contains(2),  "c1 unreachable");
        assert!(!empty.contains(10), "c2 unreachable");
        assert_eq!(empty.count(), 0, "no reachable empties when all neighbours blocked");
    }

    #[test]
    fn guard_diagonal_bypass_around_single_blocker() {
        // P1 Guard at a1 (sq 0), single P1 ally at a2 (sq 8). a3 IS reachable
        // via the diagonal route a1→b2→a3 (Chebyshev distance 2). This
        // documents the geometry: a single blocker on the file does NOT
        // prevent reaching the square behind it when diagonal steps are legal.
        let mut p = Position::empty();
        let g = 0u8;
        let blocker = 8u8;
        p.p1_pieces = Bitboard::from_square(g) | Bitboard::from_square(blocker);
        p.guards    = Bitboard::from_square(g);
        p.champions = Bitboard::from_square(blocker);

        let occ = p.p1_pieces;
        let (empty, _attack, _dist) = reachable(g, 2, occ, Bitboard::EMPTY);
        assert!(empty.contains(16), "a3 reachable via b2 diagonal");
        assert!(!empty.contains(blocker), "a2 itself occupied");
    }

    #[test]
    fn move_attack_target_enemy_adjacent() {
        // P1 Champion at e4 (sq 28), P2 Champion at e5 (sq 36).
        // Champion speed 1. Move-Attack target: sq 36.
        let mut p = Position::empty();
        p.p1_pieces = Bitboard::from_square(28);
        p.p2_pieces = Bitboard::from_square(36);
        p.champions = Bitboard::from_square(28) | Bitboard::from_square(36);
        p.to_move = Player::P1;

        let occ = p.p1_pieces | p.p2_pieces;
        let (empty, attack, _dist) = reachable(28, 1, occ, p.p2_pieces);
        assert!(attack.contains(36), "e5 is a move-attack target");
        // Plain-move destinations exclude e5 (it's enemy-occupied).
        assert!(!empty.contains(36));
        // Other 7 neighbours are plain-move legal.
        assert_eq!(empty.count(), 7);
    }

    #[test]
    fn move_attack_blocked_by_ally_intermediate() {
        // P1 Guard at a1 (speed 2). Allies fully ring a1 at b1, a2, b2 — every
        // Chebyshev-1 launch pad is occupied by allies. P2 Champion at b2's
        // far side e.g. c3 — unreachable, so no move-attack target exists.
        // This documents: an ally on the only viable launch square prevents
        // Move-Attack against an enemy beyond it.
        let mut p = Position::empty();
        let g = 0u8;
        let a_b1 = 1u8; let a_a2 = 8u8; let a_b2 = 9u8;
        let enemy = 18u8; // c3
        p.p1_pieces = Bitboard::from_square(g)
            | Bitboard::from_square(a_b1)
            | Bitboard::from_square(a_a2)
            | Bitboard::from_square(a_b2);
        p.p2_pieces = Bitboard::from_square(enemy);
        p.guards    = Bitboard::from_square(g);
        p.champions = (p.p1_pieces & !Bitboard::from_square(g))
            | Bitboard::from_square(enemy);

        let occ = p.p1_pieces | p.p2_pieces;
        let (_empty, attack, _dist) = reachable(g, 2, occ, p.p2_pieces);
        assert!(!attack.contains(enemy), "c3 unreachable when all launch pads blocked");
    }

    #[test]
    fn move_attack_reachable_via_open_diagonal_launchpad() {
        // P1 Guard at a1 (speed 2). Ally at b1 only. P2 Champion at b3 (sq 17).
        // b3 has Chebyshev distance 2 from a1; one launch pad — b2 (sq 9) —
        // is empty and at dist 1, adjacent to b3. So b3 IS attackable.
        let mut p = Position::empty();
        let g = 0u8;
        let ally = 1u8; // b1
        let enemy = 17u8; // b3
        p.p1_pieces = Bitboard::from_square(g) | Bitboard::from_square(ally);
        p.p2_pieces = Bitboard::from_square(enemy);
        p.guards    = Bitboard::from_square(g);
        p.champions = Bitboard::from_square(ally) | Bitboard::from_square(enemy);

        let occ = p.p1_pieces | p.p2_pieces;
        let (_empty, attack, _dist) = reachable(g, 2, occ, p.p2_pieces);
        assert!(attack.contains(enemy), "b3 reachable via b2 launchpad");
    }

    #[test]
    fn move_attack_completely_blocked() {
        // P1 Guard at a1 (sq 0). Allies surround it on b1 (1), a2 (8), b2 (9).
        // P2 Champion at c3 (sq 18). c3 is at Chebyshev distance 2, but every
        // intermediate-1 square (b1, a2, b2) is occupied by ally — Guard
        // cannot reach any square neighbour to c3.
        let mut p = Position::empty();
        let g = 0u8;
        let a_b1 = 1u8; let a_a2 = 8u8; let a_b2 = 9u8;
        let enemy = 18u8;
        p.p1_pieces = Bitboard::from_square(g) | Bitboard::from_square(a_b1)
            | Bitboard::from_square(a_a2) | Bitboard::from_square(a_b2);
        p.p2_pieces = Bitboard::from_square(enemy);
        p.guards    = Bitboard::from_square(g);
        p.champions = p.p1_pieces & !Bitboard::from_square(g) | Bitboard::from_square(enemy);

        let occ = p.p1_pieces | p.p2_pieces;
        let (_empty, attack, _dist) = reachable(g, 2, occ, p.p2_pieces);
        assert!(!attack.contains(enemy), "all paths to c3 blocked by allies");
    }

    // ---- Bodyguard enumeration (dual-adjacency) ----------------------

    #[test]
    fn bodyguard_dual_adjacency_filters_to_intersection() {
        // P2 King at e8 (sq 60). P2 Guards at d8 (59), e7 (52), f7 (53). All
        // three are adjacent to the King. Approach from d7 (51, empty) →
        // d7's neighbours are c6, d6, e6, c7, e7, c8, d8, e8. So Guards
        // adjacent to BOTH defender AND approach: e7 (52) and d8 (59) only.
        // f7 (53) is adjacent to the King but NOT to d7 → excluded.
        // Wrong-side P1 Guard at c8 (58) — adjacent to both King's neighbour
        // d8 and to d7, but not to King itself; doubly excluded (wrong side
        // AND not adjacent to defender).
        let mut p = Position::empty();
        let king = 60u8;
        let g_d8 = 59u8; let g_e7 = 52u8; let g_f7 = 53u8;
        let champ_neighbour = 61u8; // f8 — non-Guard, not picked
        let approach = 51u8;        // d7 — empty
        p.p2_pieces = Bitboard::from_square(king)
            | Bitboard::from_square(g_d8) | Bitboard::from_square(g_e7) | Bitboard::from_square(g_f7)
            | Bitboard::from_square(champ_neighbour);
        p.kings = Bitboard::from_square(king);
        p.guards = Bitboard::from_square(g_d8) | Bitboard::from_square(g_e7)
            | Bitboard::from_square(g_f7);
        p.champions = Bitboard::from_square(champ_neighbour);

        let guards = bodyguard_guards_for(&p, king, approach);
        // Sorted ascending: e7 (52), d8 (59).
        assert_eq!(guards, vec![g_e7, g_d8],
            "only Guards adjacent to BOTH defender AND approach are eligible");
    }

    #[test]
    fn bodyguard_zigzag_bypass_yields_empty_set() {
        // P2 King at e5 (sq 36). Single P2 Guard at e4 (sq 28) — adjacent to
        // King. Approach from d6 (sq 43): d6's neighbours are c5,d5,e5,c6,e6,
        // c7,d7,e7 — none of which is e4. So the Guard is adjacent to the
        // defender but NOT to the approach → Bodyguard CANNOT intercept.
        // This is the "zig-zag bypass" the canonical rule explicitly enables.
        let mut p = Position::empty();
        let king = 36u8;
        let g = 28u8;       // e4 — Guard adjacent to King
        let approach = 43u8; // d6 — approach square
        p.p2_pieces = Bitboard::from_square(king) | Bitboard::from_square(g);
        p.kings = Bitboard::from_square(king);
        p.guards = Bitboard::from_square(g);

        let guards = bodyguard_guards_for(&p, king, approach);
        assert!(guards.is_empty(),
            "approach d6 sidesteps Bodyguard at e4 — zig-zag bypass");
    }

    #[test]
    fn bodyguard_returns_empty_for_guard_target() {
        // Guards do not have Bodyguard protection — they ARE the Bodyguards.
        // Approach square is irrelevant when the defender is a Guard.
        let mut p = Position::empty();
        let g = 27u8;
        let neighbour_g = 28u8;
        let approach = 26u8;
        p.p1_pieces = Bitboard::from_square(g) | Bitboard::from_square(neighbour_g);
        p.guards = p.p1_pieces;

        let guards = bodyguard_guards_for(&p, g, approach);
        assert!(guards.is_empty(), "Guard target → no Bodyguard");
    }

    #[test]
    fn bodyguard_excludes_wrong_side_guards() {
        // P2 King at e8 (sq 60). P1 Guard at d8 (59) — adjacent to King AND
        // would be adjacent to any approach from d7. Must NOT be returned:
        // Bodyguards belong to the defender's side.
        let mut p = Position::empty();
        let king = 60u8;
        let enemy_guard = 59u8;
        let approach = 51u8; // d7
        p.p2_pieces = Bitboard::from_square(king);
        p.p1_pieces = Bitboard::from_square(enemy_guard);
        p.kings = Bitboard::from_square(king);
        p.guards = Bitboard::from_square(enemy_guard);

        let guards = bodyguard_guards_for(&p, king, approach);
        assert!(guards.is_empty(), "wrong-side Guards not eligible");
    }

    // ---- generate() top-level ----------------------------------------

    #[test]
    fn stack_m_setup_p1_legal_action_count() {
        // From the canonical Stack M start, P1 to move, Move phase, 2 actions.
        // P1 has: 1 King (d1, sq 3), 5 Champions (b1, c1, e1, f1, g1 — squares
        // 1, 2, 4, 5, 6), 6 Guards (b2..g2 — squares 9..=14).
        //
        // The back row pieces (King + 5 Champions) cannot move at all: their
        // rank-2 neighbours are all friendly Guards (blocked). Their other
        // potential neighbours (rank-0 same/adjacent files) are mostly empty —
        // wait, the back row IS rank 0 here. The "in front" tiles are rank 1,
        // which is the Guard row.
        //
        // Let's enumerate. Champion at b1 (sq 1): 5 neighbours on-board:
        // a1, c1 (both empty, sq 0 and 2 — wait, c1 is another P1 Champion).
        // Let me re-check: P1 layout is .CCKCCC. on rank 0 → files b,c,d,e,f,g
        // with King on file d. So sq 1 (b1) = Champion, sq 2 (c1) = Champion,
        // sq 3 (d1) = King, sq 4 (e1) = Champion, sq 5 (f1) = Champion,
        // sq 6 (g1) = Champion. Rank 1 (sq 8..15) has empty a2 (sq 8), then
        // Guards b2..g2 (sq 9..14), then empty h2 (sq 15).
        //
        // Champion at b1 (sq 1): neighbours a1(0,empty), c1(2,ally),
        // a2(8,empty), b2(9,ally-guard), c2(10,ally-guard). Speed 1 → plain
        // moves to sq 0 and sq 8. No Move-Attack targets (no enemies adjacent).
        // 2 plain-move actions for this Champion.
        //
        // Champion at g1 (sq 6, file 6): neighbours f1(5,ally), h1(7,empty),
        // f2(13,ally-guard), g2(14,ally-guard), h2(15,empty). 2 plain moves.
        //
        // Champion at c1 (sq 2): a1, b1, d1 (ally king), b2, c2, d2 (allies).
        // Wait, all neighbours on rank 0 are ally or Lance-row-empty:
        // a1(empty), b1(ally), d1(ally). On rank 1: b2(ally guard), c2(ally
        // guard), d2(ally guard). So only a1 is a legal destination? a1 is sq 0.
        // BUT wait — c1's rank-0 neighbours are files b,c,d. b1=ally, d1=ally.
        // No "a1" — c1's neighbours on rank 0 are b1 (file 1) and d1 (file 3).
        // The interior champion has NO legal move (all rank-0 neighbours
        // blocked by allies, all rank-1 neighbours blocked by Guards).
        //
        // Interior Champions (c1, e1, f1) and the King (d1): 0 moves each.
        // Corner Champions (b1, g1): 2 moves each → 4 actions.
        //
        // Guards at b2..g2 (sq 9..14), speed 2. Each Guard can move forward.
        // Take b2 (sq 9): neighbours a1(0,empty), b1(ally), c1(ally),
        // a2(8,empty), c2(ally), a3(16,empty), b3(17,empty), c3(18,empty).
        // BFS speed 2: from b2, reach distance-1 = {a1, a2, a3, b3, c3} (since
        // b1, c1, c2 are blocked). Distance-2 from b2: from a1, reach a2 (dup),
        // b1 (blocked), b2 (origin). From a2: a1, a3, b1 (blocked), b3, b2.
        // From a3: a2, b2, b3, a4 (sq 24), b4 (sq 25). From b3: a2, a3, a4,
        // b2, b4, c2 (blocked), c3, c4 (sq 26). From c3: b2 origin, b3, b4,
        // c2 (blocked), c4, d2 (blocked), d3 (sq 19), d4 (sq 27).
        //
        // So Guard b2 reachable empties at speed 2: {a1, a2, a3, b3, c3, a4,
        // b4, c4, d3, d4}. That's 10 squares. Let me re-count via the
        // generator output.
        //
        // Rather than hand-counting all 6 Guards, just smoke-test: total
        // legal actions > 0, contains EndPhase, contains a known move
        // (b1→a1, i.e. src=1 dest=0).
        let p = Position::setup_stack_m();
        let actions = generate(&p);

        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.kind() == ActionKind::EndPhase));

        // b1 (sq 1) Champion → a1 (sq 0): plain move.
        assert!(
            actions.iter().any(|a| a.kind() == ActionKind::Move
                && a.src() == 1 && a.target() == 0 && a.choice_idx() == 0),
            "expected b1→a1 move"
        );
        // b1 (sq 1) Champion → a2 (sq 8): plain move.
        assert!(
            actions.iter().any(|a| a.kind() == ActionKind::Move
                && a.src() == 1 && a.target() == 8 && a.choice_idx() == 0),
            "expected b1→a2 move"
        );

        // Interior Champion c1 (sq 2): no legal destinations.
        let c1_moves = dests_from(&actions, 2);
        assert!(c1_moves.is_empty(), "c1 is fully blocked, got dests {:?}", c1_moves);

        // King d1 (sq 3): blocked too (b/c/e/f files on rank 1 all guards,
        // c1/e1 on rank 0 are allies).
        let king_moves = dests_from(&actions, 3);
        assert!(king_moves.is_empty(), "King d1 is fully blocked");

        // No Move-Attack targets exist at setup — armies are 4 ranks apart.
        let any_attack = actions.iter().any(|a| {
            a.kind() == ActionKind::Move && p.p2_pieces.contains(a.target())
        });
        assert!(!any_attack, "no enemy adjacent at setup, no move-attacks");
    }

    #[test]
    fn generates_move_attack_with_bodyguard_choices() {
        // Construct: P1 Champion at e4 (sq 28). P2 King at e5 (sq 36) with
        // two adjacent P2 Guards (d5=35, f5=37). P1 Champion adjacent to King
        // → one Move-Attack target (sq 36), TWO Bodyguard guards.
        // Expected actions for src=28, target=36:
        //   - choice_idx=0 (no redirect)
        //   - choice_idx=1 (redirect to d5, lower sq)
        //   - choice_idx=2 (redirect to f5)
        // = 3 actions total for this src/target pair.
        let mut p = Position::empty();
        let champ = 28u8;
        let king = 36u8;
        let g_d5 = 35u8;
        let g_f5 = 37u8;
        p.p1_pieces = Bitboard::from_square(champ);
        p.p2_pieces = Bitboard::from_square(king) | Bitboard::from_square(g_d5) | Bitboard::from_square(g_f5);
        p.champions = Bitboard::from_square(champ);
        p.kings = Bitboard::from_square(king);
        p.guards = Bitboard::from_square(g_d5) | Bitboard::from_square(g_f5);
        p.to_move = Player::P1;
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;

        let actions = generate(&p);
        let attacks: Vec<u8> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Move && a.src() == champ && a.target() == king)
            .map(|a| a.choice_idx())
            .collect();
        let mut s = attacks.clone();
        s.sort_unstable();
        assert_eq!(s, vec![0, 1, 2], "expected 3 move-attack variants (no-redirect, BG d5, BG f5)");
    }

    #[test]
    fn moved_this_phase_blocks_repeat() {
        // P1 Champion at e4 (sq 28), no enemies. Mark sq 28 as moved-this-phase.
        // Expected: no Move actions originate from sq 28.
        let mut p = Position::empty();
        p.p1_pieces = Bitboard::from_square(28);
        p.champions = Bitboard::from_square(28);
        p.to_move = Player::P1;
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;
        p.moved_this_phase = Bitboard::from_square(28);

        let actions = generate(&p);
        assert!(
            !actions.iter().any(|a| a.kind() == ActionKind::Move && a.src() == 28),
            "Champion already moved this phase, no further moves expected"
        );
        // EndPhase still legal.
        assert!(actions.iter().any(|a| a.kind() == ActionKind::EndPhase));
    }

    #[test]
    fn zero_actions_remaining_only_endphase() {
        let mut p = Position::setup_stack_m();
        p.actions_remaining = 0;
        let actions = generate(&p);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].kind(), ActionKind::EndPhase);
    }

    #[test]
    fn guard_speed_2_zigzag_diagonal_legal() {
        // Single P1 Guard at d4 (sq 27). Empty board.
        // The far diagonal corner b6 (sq 41 — rank 5 file 1) is at Chebyshev
        // distance 2 (|5-3|=2, |1-3|=2 → max=2). Should be reachable.
        let mut p = Position::empty();
        let g = 27u8;
        let target = 41u8;
        p.p1_pieces = Bitboard::from_square(g);
        p.guards = Bitboard::from_square(g);
        p.to_move = Player::P1;
        p.current_phase = Phase::Move;
        p.actions_remaining = 2;

        let actions = generate(&p);
        assert!(
            actions.iter().any(|a|
                a.kind() == ActionKind::Move && a.src() == g && a.target() == target
            ),
            "diagonal-2 destination should be reachable for Guard"
        );
    }

    // ---- Skill Phase enumeration (Slice 3) ----------------------------

    use crate::state::MailboxEntry;
    use crate::state::position::GameResult;

    fn skill_phase_pos(actions: u8) -> Position {
        let mut p = Position::empty();
        p.current_phase = Phase::Skill;
        p.actions_remaining = actions;
        p.to_move = Player::P1;
        p.p1_money = 10;
        p.p2_money = 10;
        p
    }

    fn equip(p: &mut Position, sq: u8, skill_id: u8) {
        let prev = p.mailbox[sq as usize];
        // Place into slot 1 if free, else slot 2.
        p.mailbox[sq as usize] = if prev.skill1() == 0 {
            prev.with_skill1(skill_id)
        } else {
            prev.with_skill2(skill_id)
        };
    }

    fn place_champ(p: &mut Position, sq: u8, player: Player) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        p.champions = p.champions | bit;
        p.mailbox[sq as usize] = MailboxEntry::default().with_hp(2);
    }

    #[test]
    fn generate_skill_phase_caster_with_no_money_emits_only_endphase() {
        let mut p = skill_phase_pos(2);
        p.p1_money = 0;
        place_champ(&mut p, 28, Player::P1); // e4
        equip(&mut p, 28, super::skills::Skill::Lance as u8);

        let actions = generate(&p);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].kind(), ActionKind::EndPhase);
    }

    #[test]
    fn generate_skill_phase_lance_targets_enemy_in_range() {
        // P1 Champion at e4 (sq 28) with Lance (range 1 = adjacent only).
        // P2 Champion at e5 (sq 36) — adjacent N. Lance can hit it.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Lance as u8);
        place_champ(&mut p, 36, Player::P2);

        let actions = generate(&p);
        let lance_id = super::skills::Skill::Lance as u8;
        assert!(
            actions.iter().any(|a|
                a.kind() == ActionKind::Skill
                && a.src() == 28
                && a.target() == 36
                && a.skill_id() == lance_id
            ),
            "expected Lance(e4 → e5) in actions: {:?}",
            actions.iter().filter(|a| a.kind() == ActionKind::Skill).collect::<Vec<_>>()
        );
    }

    #[test]
    fn generate_skill_phase_blocked_ray_no_action() {
        // P1 Champion at e4 (sq 28) with Hook (range 2). P1 ally at e5 (36),
        // P2 enemy at e6 (44). Ally blocks the e-ray; even with range 2 Hook
        // cannot reach e6. Lance/Hook target Enemy → ally is filtered too.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        place_champ(&mut p, 36, Player::P1); // ally
        place_champ(&mut p, 44, Player::P2); // enemy past ally
        equip(&mut p, 28, super::skills::Skill::Hook as u8);

        let actions = generate(&p);
        let hook_id = super::skills::Skill::Hook as u8;
        // No Hook hits anywhere on the N-ray: ally blocks, enemy past blocker.
        assert!(
            !actions.iter().any(|a|
                a.kind() == ActionKind::Skill
                && a.src() == 28
                && a.skill_id() == hook_id
                && (a.target() == 36 || a.target() == 44)
            ),
            "Hook should not target sq 36 (ally, wrong owner) or sq 44 (blocked)"
        );
    }

    #[test]
    fn generate_skill_phase_heal_targets_only_allies() {
        // P1 Champion at e4 (sq 28) with Heal (range 1, Ally). P1 INJURED ally
        // at e5 (36) — Slice 5 added a non-Injured filter, so the target must
        // be at HP=1 to be a legal Heal target. P2 enemy at e3 (20). Heal must
        // emit ally target, NOT enemy.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        place_champ(&mut p, 36, Player::P1);
        // Drop ally HP to Injured so Heal is legal.
        p.mailbox[36] = p.mailbox[36].with_hp(1);
        place_champ(&mut p, 20, Player::P2);
        equip(&mut p, 28, super::skills::Skill::Heal as u8);

        let actions = generate(&p);
        let heal_id = super::skills::Skill::Heal as u8;
        let heal_targets: Vec<u8> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Skill
                && a.src() == 28
                && a.skill_id() == heal_id)
            .map(|a| a.target())
            .collect();
        assert!(heal_targets.contains(&36), "ally target emitted");
        assert!(!heal_targets.contains(&20), "enemy target filtered out");
    }

    #[test]
    fn generate_skill_phase_shove_targets_either_side() {
        // P1 Champion at e4 (sq 28) with Shove (range 3, Either). P1 ally at
        // e5 (36), P2 enemy at e3 (20). Both should be valid Shove targets.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        place_champ(&mut p, 36, Player::P1);
        place_champ(&mut p, 20, Player::P2);
        equip(&mut p, 28, super::skills::Skill::Shove as u8);

        let actions = generate(&p);
        let shove_id = super::skills::Skill::Shove as u8;
        let shove_targets: Vec<u8> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Skill
                && a.src() == 28
                && a.skill_id() == shove_id)
            .map(|a| a.target())
            .collect();
        assert!(shove_targets.contains(&36), "Shove emits ally target");
        assert!(shove_targets.contains(&20), "Shove emits enemy target");
    }

    #[test]
    fn generate_skill_phase_self_targeting_skill_emits_src_eq_tgt() {
        // P1 Champion at e4 (sq 28) with Shield (SelfOnly, range 0).
        // Generator emits exactly one Shield action: src=tgt=28.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Shield as u8);

        let actions = generate(&p);
        let shield_id = super::skills::Skill::Shield as u8;
        let shield_actions: Vec<_> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Skill && a.skill_id() == shield_id)
            .collect();
        assert_eq!(shield_actions.len(), 1);
        assert_eq!(shield_actions[0].src(), 28);
        assert_eq!(shield_actions[0].target(), 28);
    }

    #[test]
    fn generate_skill_phase_dash_emits_empty_targets_in_slice_5() {
        // P1 Champion at e4 (sq 28) with Dash (range 2, Empty). Empty board
        // otherwise → every queen-ray square within Chebyshev 2 should emit
        // a Dash action with dest != src.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Dash as u8);

        let actions = generate(&p);
        let dash_id = super::skills::Skill::Dash as u8;
        let dash_dests: Vec<u8> = actions.iter()
            .filter(|a| a.kind() == ActionKind::Skill
                && a.src() == 28
                && a.skill_id() == dash_id)
            .map(|a| a.target())
            .collect();
        assert!(!dash_dests.is_empty(), "Dash now emits in Slice 5");
        assert!(dash_dests.iter().all(|&d| d != 28), "no zero-move Dash");
        // Spot-check: e5 (sq 36) is range-1 N from e4 on a clear ray.
        assert!(dash_dests.contains(&36), "Dash → e5 expected");
    }

    #[test]
    fn generate_skill_phase_after_game_over_returns_empty() {
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Lance as u8);
        p.game_result = Some(GameResult::P1Wins);

        let actions = generate(&p);
        assert!(actions.is_empty());
    }

    #[test]
    fn generate_skill_phase_zero_actions_only_endphase() {
        let mut p = skill_phase_pos(0);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Lance as u8);

        let actions = generate(&p);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].kind(), ActionKind::EndPhase);
    }

    #[test]
    fn focus_resolver_sets_pending_bit() {
        // Slice-6 contract: Focus is implemented and sets the FOCUS bit.
        let mut p = skill_phase_pos(2);
        place_champ(&mut p, 28, Player::P1);
        equip(&mut p, 28, super::skills::Skill::Focus as u8);

        let focus_id = super::skills::Skill::Focus as u8;
        let a = Action::encode(28, 28, ActionKind::Skill, focus_id, 0);
        let _undo = super::super::make_unmake::make(&mut p, a);
        assert_ne!(p.pending_modifiers & crate::state::position::modifier_bits::FOCUS, 0,
            "Focus must set the FOCUS bit");
        assert_eq!(p.actions_remaining, 1, "Focus consumes one action");
    }
}
