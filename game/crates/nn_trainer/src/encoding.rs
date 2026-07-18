//! Position → input-tensor encoding for the NN position rater.
//!
//! The encoder is **P1-POV** by convention - same sign as
//! `core_engine::search::evaluator::evaluate()`. The NN trains on positions
//! framed as "what's P1's expected outcome here," so we never mirror or
//! re-orient the board. P2-to-move positions get their P2/P1 features
//! emitted in their natural slots; the side-to-move flag tells the net which
//! player chooses next.
//!
//! ## Feature layout (INPUT_DIM = 2825)
//!
//! Per square (64 squares × 44 features = 2816):
//!   - owner       (2) : [is_p1, is_p2]            (both zero → empty)
//!   - kind        (3) : [is_king, is_champion, is_guard]
//!   - hp          (3) : one-hot 0..=2             (Stack M cap)
//!   - armor       (3) : one-hot 0..=2             (Stack M cap)
//!   - skill1     (16) : one-hot id 0..=15         (0 = unequipped)
//!   - skill2     (16) : one-hot id 0..=15
//!   - combo       (1) : scalar (raw counter / 4.0, clamped to [0, 2])
//!
//! Global (9 features, appended after the per-square block):
//!   - money_p1            (1) : scalar p1_money / 20.0
//!   - money_p2            (1) : scalar p2_money / 20.0
//!   - side_to_move        (2) : one-hot [is_p1, is_p2]
//!   - phase               (3) : one-hot [draft, move, skill]
//!   - round_number        (1) : scalar round / 50.0
//!   - actions_remaining   (1) : scalar / 3.0
//!
//! Scalar normalisations are chosen so typical values land in roughly
//! [0, 1.5]; they're not tight clamps (a deep mid-game can push round/50
//! above 1.0). The first hidden layer can absorb any scale via its weights;
//! the normalisation is only to keep gradients well-conditioned at init.
//!
//! ## NOT encoded (yet)
//!
//! Skipped intentionally for v1:
//! - `pending_modifiers` (Focus / Charge flags) - small, encoded later if
//!   the rater plateaus.
//! - `tracked_enemies` / `tracked_casters` - multi-Champion combo bookkeeping;
//!   the combo *counter* on the target square already carries the load.
//! - `pending_bodyguard` - extremely transient mid-stack state. The rater
//!   never sees these positions in self-play (the search resolves them
//!   before yielding to eval).
//! - `moved_this_phase` - derivable from history; for a static-eval input
//!   this is borderline cheating and biases the rater toward the current
//!   search's blind spots.
//! - `champion_credit` - endgame accounting; defer to v2.
//! - `game_result` - terminal positions bypass the NN entirely
//!   (`HeuristicEvaluator`'s ±MATE_SCORE branch fires first).
//!
//! Adding any of these is a backwards-compatible widening of `INPUT_DIM`
//! plus retraining; no engine-side changes.

use core_engine::state::Position;
use core_engine::state::position::{Phase, Player};

/// Per-square feature width - keep in sync with the writer below.
pub const PER_SQUARE_DIM: usize = 2 + 3 + 3 + 3 + 16 + 16 + 1;

/// 64 squares × PER_SQUARE_DIM = 2816.
pub const BOARD_BLOCK_DIM: usize = 64 * PER_SQUARE_DIM;

/// Global features appended after the board block.
pub const GLOBAL_DIM: usize = 1 + 1 + 2 + 3 + 1 + 1;

/// Full input width. Network constructors consume this.
pub const INPUT_DIM: usize = BOARD_BLOCK_DIM + GLOBAL_DIM;

/// Encode `pos` as a flat `f32` vector of length `INPUT_DIM`. P1-POV.
///
/// Heap-allocates a Vec<f32>; for batch encoding in the training loop we'll
/// add an in-place variant writing into a caller-provided slice. v1 keeps
/// the simple signature.
pub fn encode_position(pos: &Position) -> Vec<f32> {
    let mut out = vec![0.0_f32; INPUT_DIM];

    // --- Per-square block --------------------------------------------------
    for sq in 0..64u8 {
        let base = (sq as usize) * PER_SQUARE_DIM;
        let mut off = 0;

        let p1 = pos.p1_pieces.contains(sq);
        let p2 = pos.p2_pieces.contains(sq);
        out[base + off    ] = p1 as u8 as f32;
        out[base + off + 1] = p2 as u8 as f32;
        off += 2;

        let king = pos.kings.contains(sq);
        let champ = pos.champions.contains(sq);
        let guard = pos.guards.contains(sq);
        out[base + off    ] = king as u8 as f32;
        out[base + off + 1] = champ as u8 as f32;
        out[base + off + 2] = guard as u8 as f32;
        off += 3;

        let m = pos.mailbox[sq as usize];

        // HP one-hot 0..=2 - clamp defensively (Stack M caps at 2).
        let hp = m.hp().min(2) as usize;
        out[base + off + hp] = 1.0;
        off += 3;

        // Armor one-hot 0..=2.
        let armor = m.armor().min(2) as usize;
        out[base + off + armor] = 1.0;
        off += 3;

        // Skill1 one-hot 0..=15.
        let s1 = m.skill1().min(15) as usize;
        out[base + off + s1] = 1.0;
        off += 16;

        // Skill2 one-hot 0..=15.
        let s2 = m.skill2().min(15) as usize;
        out[base + off + s2] = 1.0;
        off += 16;

        // Combo scalar, normalised. Clamp 5+ to 5 then /4 → values in [0, 1.25].
        let combo = m.combo().min(5) as f32 / 4.0;
        out[base + off] = combo;
        // off += 1;  // (unused - last in block)
    }

    // --- Global block ------------------------------------------------------
    let g = BOARD_BLOCK_DIM;
    out[g    ] = pos.p1_money as f32 / 20.0;
    out[g + 1] = pos.p2_money as f32 / 20.0;

    // Side to move one-hot.
    match pos.to_move {
        Player::P1 => out[g + 2] = 1.0,
        Player::P2 => out[g + 3] = 1.0,
    }

    // Phase one-hot.
    match pos.current_phase {
        Phase::Draft => out[g + 4] = 1.0,
        Phase::Move  => out[g + 5] = 1.0,
        Phase::Skill => out[g + 6] = 1.0,
    }

    out[g + 7] = pos.round_number as f32 / 50.0;
    out[g + 8] = pos.actions_remaining as f32 / 3.0;

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::state::Position;

    #[test]
    fn encoding_has_expected_width() {
        let pos = Position::setup_stack_m();
        let v = encode_position(&pos);
        assert_eq!(v.len(), INPUT_DIM);
        assert_eq!(INPUT_DIM, 2825);
    }

    #[test]
    fn empty_position_encodes_to_mostly_zero() {
        let pos = Position::empty();
        let v = encode_position(&pos);
        // Per-square block, owner+kind+combo zero on every square. HP one-hot,
        // armor one-hot, skill1 one-hot (id=0=unequipped), and skill2 one-hot
        // each fire at index 0 → 4 ones per square × 64 = 256 ones.
        // Globals: side_to_move (P1, default), phase (Move, default), and
        // round_number (=1, scalar 1/50) fire = 3 more nonzero entries.
        let nonzero: usize = v.iter().filter(|x| **x != 0.0).count();
        assert_eq!(nonzero, 256 + 3,
            "expected 256 per-square zero-slot one-hots + 3 global signals");
    }

    #[test]
    fn setup_stack_m_is_mirror_symmetric_in_board_block() {
        // Stack M setup is mirror-symmetric: P1 features should equal P2
        // features when swapped across the board. We check it's not the
        // all-zero case (which would also be symmetric - and wrong).
        let pos = Position::setup_stack_m();
        let v = encode_position(&pos);
        let board: &[f32] = &v[..BOARD_BLOCK_DIM];
        let total: f32 = board.iter().sum();
        assert!(total > 100.0, "expected substantial activations in start position, got {total}");
    }

    #[test]
    fn no_nan_or_inf_in_encoding() {
        let pos = Position::setup_stack_m();
        let v = encode_position(&pos);
        assert!(v.iter().all(|x| x.is_finite()), "encoding produced non-finite values");
    }
}
