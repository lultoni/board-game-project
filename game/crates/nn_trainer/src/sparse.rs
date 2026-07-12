//! Sparse binary feature encoding for the NNUE-style position evaluator.
//!
//! Replaces the dense `encoding::encode_position` (2825 f32) with a **sparse
//! set of active binary feature indices** — the input shape an incrementally
//! updatable accumulator (`accumulator.rs`) needs. See
//! `design/inbox/nnue-rework-plan.md` §3.1.
//!
//! The encoder is **P1-POV** by convention, matching
//! `core_engine::search::evaluator::evaluate_scalar` (the Phase-0 regression
//! target). We never mirror the board; the side-to-move feature tells the net
//! whose turn it is.
//!
//! ## Feature space (NUM_FEATURES = 3352)
//!
//! Every feature is binary. Within each logical group at most one feature is
//! active (one-hot), so changing a single attribute flips **exactly two**
//! features (old off, new on) — the property that keeps the accumulator delta
//! small and makes `apply == refresh` cheap.
//!
//! Per-square block (64 squares × PER_SQUARE = 3328), for square `sq` at base
//! `sq * PER_SQUARE`:
//!   - owner+kind (6) : one-hot over {P1,P2}×{King,Champion,Guard}
//!                      (empty square → none active)
//!   - hp        (3) : one-hot 0..=2                (Stack M cap)
//!   - armor     (3) : one-hot 0..=2
//!   - skill1   (16) : one-hot id 0..=15            (0 = unequipped)
//!   - skill2   (16) : one-hot id 0..=15
//!   - combo     (8) : one-hot counter 0..=7
//!
//! Global block (24), appended at offset BOARD_BLOCK. Bucketed features rather
//! than a dense side-input: a dense side path would break the accumulator's
//! single "add/subtract weight columns" model and complicate the golden
//! bit-identity test. Coarse money/round granularity is irrelevant to Phase 0
//! (the first layer absorbs bucket scale).
//!   - side_to_move (2) : one-hot [P1, P2]
//!   - phase        (3) : one-hot [Draft, Move, Skill]
//!   - p1_money     (5) : bucket (see `money_bucket`)
//!   - p2_money     (5) : bucket
//!   - round        (5) : bucket (see `round_bucket`)
//!   - actions      (4) : one-hot 0..=3
//!
//! ## NOT encoded (mirrors the dense encoder's v1 omissions)
//! pending_modifiers, tracked_enemies/casters, pending_bodyguard,
//! moved_this_phase, champion_credit, game_result (terminals bypass the NN).

use core_engine::state::MailboxEntry;
use core_engine::state::Position;
use core_engine::state::position::{Phase, Player};

// --- Per-square group widths + intra-square offsets ------------------------

const OWNER_KIND: usize = 6; // {P1,P2} × {K,C,G}
const HP: usize = 3;
const ARMOR: usize = 3;
const SKILL1: usize = 16;
const SKILL2: usize = 16;
const COMBO: usize = 8;

/// Per-square feature width. Keep in sync with `square_features`.
pub const PER_SQUARE: usize = OWNER_KIND + HP + ARMOR + SKILL1 + SKILL2 + COMBO; // 52

// Intra-square group offsets (relative to the square's base index).
const OFF_OWNER_KIND: usize = 0;
const OFF_HP: usize = OFF_OWNER_KIND + OWNER_KIND; // 6
const OFF_ARMOR: usize = OFF_HP + HP; // 9
const OFF_SKILL1: usize = OFF_ARMOR + ARMOR; // 12
const OFF_SKILL2: usize = OFF_SKILL1 + SKILL1; // 28
const OFF_COMBO: usize = OFF_SKILL2 + SKILL2; // 44

/// 64 squares × PER_SQUARE.
pub const BOARD_BLOCK: usize = 64 * PER_SQUARE; // 3328

// --- Global group widths + offsets (relative to BOARD_BLOCK) ---------------

const G_STM: usize = 2;
const G_PHASE: usize = 3;
const G_MONEY: usize = 5; // per side
const G_ROUND: usize = 5;
const G_ACTIONS: usize = 4;

/// Global feature width.
pub const GLOBAL_BLOCK: usize = G_STM + G_PHASE + G_MONEY + G_MONEY + G_ROUND + G_ACTIONS; // 24

const OFF_G_STM: usize = 0;
const OFF_G_PHASE: usize = OFF_G_STM + G_STM; // 2
const OFF_G_P1_MONEY: usize = OFF_G_PHASE + G_PHASE; // 5
const OFF_G_P2_MONEY: usize = OFF_G_P1_MONEY + G_MONEY; // 10
const OFF_G_ROUND: usize = OFF_G_P2_MONEY + G_MONEY; // 15
const OFF_G_ACTIONS: usize = OFF_G_ROUND + G_ROUND; // 20

/// Full sparse feature space width. The trained f32 net's `input_dim` equals
/// this, and the accumulator's `FeatureTransform` has this many weight columns.
pub const NUM_FEATURES: usize = BOARD_BLOCK + GLOBAL_BLOCK; // 3352

/// First-layer / accumulator width. Matches the trained net's `hidden_sizes[0]`.
///
/// ns-50 tail-cost rework: dropped 256→128. The tail forward runs fully every
/// node and L1 (ACCUM_WIDTH → hidden_sizes[1]) dominated the NNUE eval cost;
/// halving the width quarters L1 alongside the paired `hidden_sizes[1]` 64→32
/// cut (see `model.rs`). A standard small-NNUE size; kept a config knob so
/// Phase-1 can grow it back if strength stalls (the plan's "start small, grow
/// if strength stalls").
pub const ACCUM_WIDTH: usize = 128;

// --- Bucket helpers (single source of truth, shared with the accumulator) --

/// Map a money value to one of 5 buckets: {0, 1–2, 3–5, 6–10, 11+}.
#[inline]
pub fn money_bucket(m: u16) -> usize {
    match m {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        6..=10 => 3,
        _ => 4,
    }
}

/// Map a round number to one of 5 buckets: {1–3, 4–8, 9–12, 13–20, 21+}.
/// (round_number is 1-based; 0 is treated as bucket 0 defensively.)
#[inline]
pub fn round_bucket(r: u16) -> usize {
    match r {
        0..=3 => 0,
        4..=8 => 1,
        9..=12 => 2,
        13..=20 => 3,
        _ => 4,
    }
}

// --- Shared per-square / global index math ---------------------------------
//
// These two helpers are THE single source of truth for the feature-index
// space. `encode_sparse` and the accumulator's `refresh`/`apply` both route
// through them, which is what guarantees the incremental accumulator stays
// bit-identical to a full refresh (both compute the same indices).

/// Push the active feature indices contributed by ONE square, given its
/// occupancy (bitboard membership) + mailbox entry. Empty squares still emit
/// the hp=0 / armor=0 / skill=0 / combo=0 one-hots (matching the dense
/// encoder), but no owner/kind feature.
#[inline]
pub(crate) fn square_features(
    sq: u8,
    is_p1: bool,
    is_p2: bool,
    is_king: bool,
    is_champion: bool,
    is_guard: bool,
    m: MailboxEntry,
    push: &mut impl FnMut(u32),
) {
    let base = (sq as usize) * PER_SQUARE;

    // owner+kind one-hot: index = owner_offset(0=P1,3=P2) + kind(0=K,1=C,2=G).
    // Only fires when the square is occupied by exactly one owner + kind.
    if is_p1 || is_p2 {
        let owner_off = if is_p1 { 0 } else { 3 };
        let kind = if is_king {
            0
        } else if is_champion {
            1
        } else {
            debug_assert!(is_guard, "occupied square must be K/C/G");
            2
        };
        push((base + OFF_OWNER_KIND + owner_off + kind) as u32);
    }

    // hp one-hot 0..=2 (clamp defensively; Stack M caps at 2).
    let hp = (m.hp() as usize).min(HP - 1);
    push((base + OFF_HP + hp) as u32);

    // armor one-hot 0..=2.
    let armor = (m.armor() as usize).min(ARMOR - 1);
    push((base + OFF_ARMOR + armor) as u32);

    // skill1 one-hot 0..=15.
    let s1 = (m.skill1() as usize).min(SKILL1 - 1);
    push((base + OFF_SKILL1 + s1) as u32);

    // skill2 one-hot 0..=15.
    let s2 = (m.skill2() as usize).min(SKILL2 - 1);
    push((base + OFF_SKILL2 + s2) as u32);

    // combo one-hot 0..=7.
    let combo = (m.combo() as usize).min(COMBO - 1);
    push((base + OFF_COMBO + combo) as u32);
}

/// Push the active global feature indices for `pos`.
#[inline]
pub(crate) fn global_features(pos: &Position, push: &mut impl FnMut(u32)) {
    global_indices(
        pos.to_move,
        pos.current_phase,
        pos.p1_money,
        pos.p2_money,
        pos.round_number,
        pos.actions_remaining,
        push,
    );
}

/// Push the active global feature indices from raw scalar values. Split out so
/// the accumulator's `revert` path can reconstruct pre-make globals from `Undo`
/// scalars without materialising a `Position`. Single source of truth for the
/// global index math.
#[inline]
pub(crate) fn global_indices(
    to_move: Player,
    phase: Phase,
    p1_money: u16,
    p2_money: u16,
    round_number: u16,
    actions_remaining: u8,
    push: &mut impl FnMut(u32),
) {
    let g = BOARD_BLOCK;

    // side to move one-hot.
    let stm = match to_move {
        Player::P1 => 0,
        Player::P2 => 1,
    };
    push((g + OFF_G_STM + stm) as u32);

    // phase one-hot.
    let phase = match phase {
        Phase::Draft => 0,
        Phase::Move => 1,
        Phase::Skill => 2,
    };
    push((g + OFF_G_PHASE + phase) as u32);

    // money buckets (per side).
    push((g + OFF_G_P1_MONEY + money_bucket(p1_money)) as u32);
    push((g + OFF_G_P2_MONEY + money_bucket(p2_money)) as u32);

    // round bucket.
    push((g + OFF_G_ROUND + round_bucket(round_number)) as u32);

    // actions_remaining one-hot 0..=3.
    let actions = (actions_remaining as usize).min(G_ACTIONS - 1);
    push((g + OFF_G_ACTIONS + actions) as u32);
}

// --- Public encoder --------------------------------------------------------

/// Encode `pos` as its set of active sparse feature indices (P1-POV), filling
/// `out`. `out` is cleared first, so a single buffer can be reused across nodes
/// without reallocating. Indices are order-independent and deduped by
/// construction (each one-hot group contributes at most one index).
pub fn encode_sparse(pos: &Position, out: &mut Vec<u32>) {
    out.clear();
    for sq in 0..64u8 {
        square_features(
            sq,
            pos.p1_pieces.contains(sq),
            pos.p2_pieces.contains(sq),
            pos.kings.contains(sq),
            pos.champions.contains(sq),
            pos.guards.contains(sq),
            pos.mailbox[sq as usize],
            &mut |f| out.push(f),
        );
    }
    global_features(pos, &mut |f| out.push(f));
}

/// Convenience wrapper allocating a fresh Vec. Prefer `encode_sparse` with a
/// reused buffer in hot loops.
pub fn encode_sparse_vec(pos: &Position) -> Vec<u32> {
    let mut out = Vec::with_capacity(64 * 6 + 6);
    encode_sparse(pos, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::game_logic::generator;
    use core_engine::game_logic::make_unmake;
    use core_engine::state::Position;
    use std::collections::HashSet;

    #[test]
    fn feature_space_dims() {
        assert_eq!(PER_SQUARE, 52);
        assert_eq!(BOARD_BLOCK, 3328);
        assert_eq!(GLOBAL_BLOCK, 24);
        assert_eq!(NUM_FEATURES, 3352);
    }

    #[test]
    fn empty_position_active_count() {
        // Empty board: every square emits hp=0, armor=0, skill1=0, skill2=0,
        // combo=0 one-hots (5 per square, no owner/kind) = 320. Globals emit
        // exactly one feature per group = 6. Total 326.
        let pos = Position::empty();
        let v = encode_sparse_vec(&pos);
        assert_eq!(v.len(), 320 + 6, "empty board active-feature count");
        assert!(v.iter().all(|&f| (f as usize) < NUM_FEATURES));
    }

    #[test]
    fn all_indices_in_range_and_unique() {
        let mut positions = vec![Position::setup_stack_m(), Position::empty()];
        for seed in 0..5u64 {
            positions.push(seeded_position(seed));
        }
        for pos in &positions {
            let v = encode_sparse_vec(pos);
            let set: HashSet<u32> = v.iter().copied().collect();
            assert_eq!(set.len(), v.len(), "duplicate feature index in a single encode");
            assert!(v.iter().all(|&f| (f as usize) < NUM_FEATURES), "index out of range");
        }
    }

    #[test]
    fn occupied_square_emits_six_features() {
        // A start position has 24 pieces; each occupied square emits 6 features
        // (owner/kind + hp + armor + s1 + s2 + combo), empty squares emit 5.
        let pos = Position::setup_stack_m();
        let v = encode_sparse_vec(&pos);
        let occupied = 24usize;
        let expected = occupied * 6 + (64 - occupied) * 5 + 6; // +6 globals
        assert_eq!(v.len(), expected);
    }

    #[test]
    fn make_flips_small_set() {
        // The load-bearing invariant: a single legal action flips only a small,
        // bounded set of features vs a full refresh. Enumerate the first several
        // legal actions from the start position and assert the symmetric
        // difference stays well under a full refresh.
        let mut pos = Position::setup_stack_m();
        let before: HashSet<u32> = encode_sparse_vec(&pos).into_iter().collect();

        let actions = generator::generate(&pos);
        assert!(!actions.is_empty());

        let mut checked = 0;
        for &action in actions.iter().take(12) {
            let undo = make_unmake::make(&mut pos, action);
            let after: HashSet<u32> = encode_sparse_vec(&pos).into_iter().collect();
            let diff = before.symmetric_difference(&after).count();
            // A full refresh touches ~326 features; any single action must flip
            // dramatically fewer. AOE (Tempest, ≤9 squares) is the worst case:
            // ≤ 9 squares × 6 feature-flips + a few globals — comfortably < 80.
            assert!(
                diff <= 80,
                "action flipped {diff} features (expected a small bounded set)"
            );
            make_unmake::unmake(&mut pos, &undo);
            checked += 1;
        }
        assert!(checked > 0, "no actions were checked");

        // Round-trip: unmake restored the original feature set exactly.
        let restored: HashSet<u32> = encode_sparse_vec(&pos).into_iter().collect();
        assert_eq!(before, restored, "unmake did not restore the feature set");
    }

    /// Deterministic seeded position for range/uniqueness fuzzing: apply a few
    /// legal actions from the start position via a simple LCG index picker.
    fn seeded_position(seed: u64) -> Position {
        let mut pos = Position::setup_stack_m();
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
        for _ in 0..(3 + (seed % 4)) {
            let actions = generator::generate(&pos);
            if actions.is_empty() {
                break;
            }
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let idx = (state >> 33) as usize % actions.len();
            make_unmake::make(&mut pos, actions[idx]);
        }
        pos
    }
}
