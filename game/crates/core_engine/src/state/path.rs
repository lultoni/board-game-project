//! Layer 1 — Position-aware path helpers built on top of `state::magic`.
//!
//! These functions take a `Position` and return owner-agnostic geometry:
//! - `skill_targets(pos, src, range)` → bitboard of "first piece per ray"
//!   targets reachable from `src` within `range`. Pieces of EITHER side
//!   count as a target. Skill-specific ally/enemy filtering happens at the
//!   generator (`game_logic::generator`) using `skill_target_owner`.
//! - `path_clear(pos, src, tgt)` → true iff every square strictly between
//!   `src` and `tgt` on the queen-ray is empty. `tgt` itself is NOT
//!   required to be empty — skill resolvers decide that.
//!
//! Both are thin wrappers over `state::magic`; the only reason they live in
//! a separate module is to keep `state::magic` free of `Position`.

use super::bitboard::Bitboard;
use super::magic;
use super::position::Position;

/// Squares on which a skill cast from `src` with `range` could land:
/// the first occupied square on each of the 8 queen-rays from `src`,
/// limited to Chebyshev distance ≤ `range`. Ownership is not filtered here.
pub fn skill_targets(pos: &Position, src: u8, range: u8) -> Bitboard {
    let occ = (pos.p1_pieces | pos.p2_pieces).0;
    let attacks = magic::skill_attacks(src, occ, range);
    // `skill_attacks` returns reachable empties + first blocker per ray. The
    // intersection with `occ` drops the empty squares, leaving just the
    // blockers — i.e. the legal targets.
    Bitboard(attacks.0 & occ)
}

/// True iff every square strictly between `src` and `tgt` on the queen-ray
/// connecting them is empty. Returns false if `src` and `tgt` are not on a
/// ray or are the same square. The occupancy of `tgt` itself is NOT
/// considered — empty-target skills (Move-skills) and piece-target skills
/// (Strike/Heal/etc.) both rely on this primitive and decide on their own
/// what `tgt` must look like.
pub fn path_clear(pos: &Position, src: u8, tgt: u8) -> bool {
    if !magic::on_ray(src, tgt) { return false; }
    let between = magic::between(src, tgt).0;
    let occ = (pos.p1_pieces | pos.p2_pieces).0;
    (between & occ) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::position::Player;

    fn p1_piece(pos: &mut Position, sq: u8) {
        pos.p1_pieces = pos.p1_pieces | Bitboard::from_square(sq);
        pos.champions = pos.champions | Bitboard::from_square(sq);
    }
    fn p2_piece(pos: &mut Position, sq: u8) {
        pos.p2_pieces = pos.p2_pieces | Bitboard::from_square(sq);
        pos.champions = pos.champions | Bitboard::from_square(sq);
    }

    #[test]
    fn skill_targets_only_returns_first_piece_per_ray() {
        // Caster at a1 (sq 0). Two pieces on diagonal: c3 (sq 18) and e5 (sq 36).
        // skill_targets should return c3 only (first blocker per ray; e5 is past it).
        let mut pos = Position::empty();
        pos.to_move = Player::P1;
        p1_piece(&mut pos, 0);
        p1_piece(&mut pos, 18);
        p2_piece(&mut pos, 36);
        let tgts = skill_targets(&pos, 0, 4);
        assert!(tgts.contains(18));
        assert!(!tgts.contains(36));
    }

    #[test]
    fn skill_targets_includes_both_allies_and_enemies() {
        // Caster at e4 (sq 28), ally at e5 (sq 36), enemy at e6 (sq 44).
        // Ally blocks before enemy → only sq 36 returned.
        let mut pos = Position::empty();
        pos.to_move = Player::P1;
        p1_piece(&mut pos, 28);
        p1_piece(&mut pos, 36);
        p2_piece(&mut pos, 44);
        let tgts = skill_targets(&pos, 28, 4);
        assert!(tgts.contains(36), "ally on e5 is a valid target geometrically");
        assert!(!tgts.contains(44), "enemy on e6 is blocked by the ally");
    }

    #[test]
    fn path_clear_true_for_empty_ray() {
        let mut pos = Position::empty();
        p1_piece(&mut pos, 0);
        p2_piece(&mut pos, 63);
        // Diagonal a1↔h8, intermediate squares all empty.
        assert!(path_clear(&pos, 0, 63));
    }

    #[test]
    fn path_clear_false_for_blocked_ray() {
        let mut pos = Position::empty();
        p1_piece(&mut pos, 0);
        p2_piece(&mut pos, 63);
        p1_piece(&mut pos, 27); // d4 mid-diagonal
        assert!(!path_clear(&pos, 0, 63));
    }

    #[test]
    fn path_clear_false_off_ray() {
        let pos = Position::empty();
        // a1 (0) → b4 (25) is not a queen-ray.
        assert!(!path_clear(&pos, 0, 25));
    }

    #[test]
    fn path_clear_doesnt_require_tgt_empty() {
        let mut pos = Position::empty();
        // a1 → c3, c3 occupied, b2 empty. path_clear should still be true.
        p1_piece(&mut pos, 0);
        p2_piece(&mut pos, 18);
        assert!(path_clear(&pos, 0, 18));
    }
}
