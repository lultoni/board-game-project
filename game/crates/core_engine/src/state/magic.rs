//! Layer 1 — Path/Range/Block primitives.
//!
//! Provides the queen-style 8-direction "first piece on each ray" lookup that
//! every Skill targeting rule in Stack M sits on top of. Output is a single
//! bitboard of squares that are either empty AND on a ray AND within range,
//! OR the first occupied square on each ray within range (the blocker).
//!
//! # Why this lives in `state/`
//!
//! The lookup is pure: it depends only on `(src, occupancy, range)` and not on
//! turn order, ownership, or any mutable game state. It is foundational
//! geometry — same role as `mailbox` / `bitboard` / `zobrist`.
//!
//! # API
//!
//! ```rust,ignore
//! use core_engine::state::magic;
//!
//! // 8-direction attacks from `sq`, bounded by Chebyshev distance `range`.
//! let bb = magic::skill_attacks(sq, occupancy, range);
//!
//! // BETWEEN[a][b] = squares strictly between a and b on the queen-ray
//! // (or 0 if not on a ray / same square). Useful for "is the path clear?"
//! let between = magic::between(a, b);
//! ```
//!
//! # Implementation note
//!
//! The plan called for full chess-style Magic Bitboards with magic-multiply-
//! shift perfect hashing. We implement the same API with classical ray
//! scanning over per-direction ray-masks (8 rays × 64 squares = 512 u64s,
//! plus a 64×64 BETWEEN table = 32 KiB). For Stack M's branching factor
//! this is more than fast enough; the swap-in to true magic is local to
//! `skill_attacks_along` if profiling later demands it.

use super::bitboard::Bitboard;
use std::sync::OnceLock;

/// (dr, df) deltas in canonical direction order: N, NE, E, SE, S, SW, W, NW.
const DELTAS: [(i8, i8); 8] = [
    ( 1,  0), // N
    ( 1,  1), // NE
    ( 0,  1), // E
    (-1,  1), // SE
    (-1,  0), // S
    (-1, -1), // SW
    ( 0, -1), // W
    ( 1, -1), // NW
];

/// `RAYS[dir][sq]` = bitboard of all squares on the ray FROM `sq` in `dir`,
/// EXCLUDING `sq` itself, including the edge of the board.
static RAYS: OnceLock<[[u64; 64]; 8]> = OnceLock::new();

/// `BETWEEN[a][b]` = bitboard of squares strictly between `a` and `b` on the
/// queen-ray that connects them (empty if they don't share a ray or `a == b`).
static BETWEEN: OnceLock<[[u64; 64]; 64]> = OnceLock::new();

/// `WITHIN_RANGE[sq][r]` = bitboard of all squares with Chebyshev distance
/// `1..=r` from `sq`. Index 0 is the empty bitboard.
static WITHIN_RANGE: OnceLock<[[u64; 5]; 64]> = OnceLock::new();

fn rays() -> &'static [[u64; 64]; 8] {
    RAYS.get_or_init(|| {
        let mut out = [[0u64; 64]; 8];
        for sq in 0u8..64 {
            let rank = (sq / 8) as i8;
            let file = (sq % 8) as i8;
            for (i, &(dr, df)) in DELTAS.iter().enumerate() {
                let mut r = rank + dr;
                let mut f = file + df;
                let mut bb = 0u64;
                while (0..8).contains(&r) && (0..8).contains(&f) {
                    bb |= 1u64 << ((r * 8 + f) as u8);
                    r += dr;
                    f += df;
                }
                out[i][sq as usize] = bb;
            }
        }
        out
    })
}

fn between_table() -> &'static [[u64; 64]; 64] {
    BETWEEN.get_or_init(|| {
        let mut out = [[0u64; 64]; 64];
        for a in 0u8..64 {
            let ar = (a / 8) as i8;
            let af = (a % 8) as i8;
            for b in 0u8..64 {
                if a == b { continue; }
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;
                let dr = br - ar;
                let df = bf - af;
                // Same square already handled. Require same rank, same file,
                // or |dr| == |df| (diagonal).
                let on_ray = dr == 0 || df == 0 || dr.abs() == df.abs();
                if !on_ray { continue; }
                let step_r = dr.signum();
                let step_f = df.signum();
                let mut r = ar + step_r;
                let mut f = af + step_f;
                let mut bb = 0u64;
                while (r, f) != (br, bf) {
                    bb |= 1u64 << ((r * 8 + f) as u8);
                    r += step_r;
                    f += step_f;
                }
                out[a as usize][b as usize] = bb;
            }
        }
        out
    })
}

fn within_range_table() -> &'static [[u64; 5]; 64] {
    WITHIN_RANGE.get_or_init(|| {
        let mut out = [[0u64; 5]; 64];
        for sq in 0u8..64 {
            let sr = (sq / 8) as i8;
            let sf = (sq % 8) as i8;
            for r_max in 1usize..=4 {
                let mut bb = 0u64;
                for t in 0u8..64 {
                    if t == sq { continue; }
                    let tr = (t / 8) as i8;
                    let tf = (t % 8) as i8;
                    let cheby = (tr - sr).abs().max((tf - sf).abs()) as usize;
                    if cheby >= 1 && cheby <= r_max {
                        bb |= 1u64 << t;
                    }
                }
                out[sq as usize][r_max] = bb;
            }
            // index 0 left as 0 (empty)
        }
        out
    })
}

/// 8-direction queen-style "skill attack" bitboard from `sq` against the given
/// occupancy, bounded by Chebyshev distance `range`.
///
/// The returned bitboard includes:
/// - every empty square on a ray from `sq` within `range`, AND
/// - the first occupied square ("blocker") on each ray within `range`, if any.
///
/// `range = 0` → empty bitboard.
/// `range ≥ 4` → the full board reach (Stack M's effective maximum range is
/// Retreat at +1 = 4; the lookup table covers 0..=4).
pub fn skill_attacks(sq: u8, occ: u64, range: u8) -> Bitboard {
    debug_assert!(sq < 64);
    if range == 0 {
        return Bitboard::EMPTY;
    }
    let rays_t = rays();
    let mut out: u64 = 0;
    for d in 0..8 {
        let ray = rays_t[d][sq as usize];
        let blockers = ray & occ;
        let reach_full = if blockers == 0 {
            // Entire ray is empty.
            ray
        } else {
            // Find the closest blocker on this ray. Direction determines whether
            // "closest" means lowest or highest bit-index.
            //   N, NE, E, NW → increasing bit index (away from sq goes up)
            //   S, SE, W, SW → decreasing bit index (away from sq goes down)
            // We picked the deltas s.t. (dr > 0) or (dr == 0 && df > 0) → up,
            // anything else → down. Mirror that here.
            let (dr, df) = DELTAS[d];
            let upward = dr > 0 || (dr == 0 && df > 0);
            let first_blocker = if upward {
                blockers.trailing_zeros() as u8
            } else {
                63 - (blockers.leading_zeros() as u8)
            };
            // Squares on the ray from sq up to AND INCLUDING the blocker.
            let mask_to_blocker = ray ^ rays_t[d][first_blocker as usize];
            // ^ flips off all squares strictly past the blocker. Result still
            // includes the blocker itself (since it's on `ray` but not on
            // the blocker's own outgoing ray).
            mask_to_blocker
        };
        out |= reach_full;
    }
    // Apply the range cap.
    let capped = out & within_range_table()[sq as usize][range.min(4) as usize];
    Bitboard(capped)
}

/// Squares strictly between `a` and `b` on the queen-ray connecting them.
/// Returns the empty bitboard if `a == b` or they are not on a ray.
pub fn between(a: u8, b: u8) -> Bitboard {
    debug_assert!(a < 64 && b < 64);
    Bitboard(between_table()[a as usize][b as usize])
}

/// True iff `a` and `b` lie on the same 8-direction ray and are distinct.
pub fn on_ray(a: u8, b: u8) -> bool {
    if a == b { return false; }
    let ar = (a / 8) as i8; let af = (a % 8) as i8;
    let br = (b / 8) as i8; let bf = (b % 8) as i8;
    let dr = br - ar;
    let df = bf - af;
    dr == 0 || df == 0 || dr.abs() == df.abs()
}

/// One square step from `from` toward `to` along the queen-ray they share.
/// Returns `None` if `from == to` or they aren't on a ray. The result is
/// always on-board when `Some` (any ray between two on-board squares has
/// at least one intermediate or one-step-from-`from` square also on-board).
pub fn step_toward(from: u8, to: u8) -> Option<u8> {
    if !on_ray(from, to) { return None; }
    let dr = ((to / 8) as i8 - (from / 8) as i8).signum();
    let df = ((to % 8) as i8 - (from % 8) as i8).signum();
    let r = (from / 8) as i8 + dr;
    let f = (from % 8) as i8 + df;
    Some((r * 8 + f) as u8)
}

/// One square step from `at` in the direction *away* from `pivot`, i.e. the
/// next square along the queen-ray going `pivot → at → ?`. Returns `None`
/// if not on a ray, the same-square case, or the step would leave the board.
pub fn step_away(pivot: u8, at: u8) -> Option<u8> {
    if !on_ray(pivot, at) { return None; }
    let dr = ((at / 8) as i8 - (pivot / 8) as i8).signum();
    let df = ((at % 8) as i8 - (pivot % 8) as i8).signum();
    let r = (at / 8) as i8 + dr;
    let f = (at % 8) as i8 + df;
    if !(0..8).contains(&r) || !(0..8).contains(&f) { return None; }
    Some((r * 8 + f) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bb_of(squares: &[u8]) -> u64 {
        squares.iter().fold(0u64, |acc, &s| acc | (1u64 << s))
    }

    #[test]
    fn empty_board_full_8_directions() {
        // Centre square e4 = file 4 rank 3 → sq 28. Range 4 covers the whole
        // 8-direction reach (which on an 8×8 board with src in the middle
        // is the Chebyshev-≤4 ring minus squares off the rays).
        let attacks = skill_attacks(28, 0, 4);
        // Expected: every square on a rank/file/diagonal of sq 28, exclusive of sq 28.
        let mut expected = 0u64;
        for t in 0u8..64 {
            if on_ray(28, t) {
                expected |= 1u64 << t;
            }
        }
        assert_eq!(attacks.0, expected);
    }

    #[test]
    fn blocked_by_first_piece() {
        // Caster at e4 (sq 28). Blocker at e6 (sq 44, two squares N).
        // skill_attacks should include e5 (sq 36), e6 (sq 44), but NOT e7 (52)
        // or e8 (60).
        let occ = 1u64 << 44;
        let attacks = skill_attacks(28, occ, 4);
        assert!(attacks.contains(36), "e5 reachable");
        assert!(attacks.contains(44), "e6 reachable (blocker square is included)");
        assert!(!attacks.contains(52), "e7 past blocker");
        assert!(!attacks.contains(60), "e8 past blocker");
    }

    #[test]
    fn range_0_returns_empty() {
        let attacks = skill_attacks(28, 0, 0);
        assert_eq!(attacks.0, 0);
    }

    #[test]
    fn range_2_caps_default() {
        // From d4 (sq 27, rank 3 file 3), range 2 should reach exactly the
        // queen-ray squares with Chebyshev distance 1 or 2.
        let attacks = skill_attacks(27, 0, 2);
        let mut expected = 0u64;
        for t in 0u8..64 {
            if !on_ray(27, t) { continue; }
            let tr = (t / 8) as i8;
            let tf = (t % 8) as i8;
            let dr = (tr - 3).abs();
            let df = (tf - 3).abs();
            let cheby = dr.max(df);
            if cheby == 1 || cheby == 2 {
                expected |= 1u64 << t;
            }
        }
        assert_eq!(attacks.0, expected);
    }

    #[test]
    fn between_ray_squares_correct() {
        // a1 = sq 0, h8 = sq 63. Diagonal — between should be {b2, c3, d4, e5, f6, g7}.
        let b = between(0, 63);
        assert_eq!(b.0, bb_of(&[9, 18, 27, 36, 45, 54]));
    }

    #[test]
    fn between_off_ray_is_zero() {
        // a1 → b4 is not a queen-ray (dr=3, df=1).
        let b = between(0, 25);
        assert_eq!(b.0, 0);
    }

    #[test]
    fn between_same_square_is_zero() {
        assert_eq!(between(0, 0).0, 0);
    }

    #[test]
    fn on_ray_matches_geometry() {
        // a1 ↔ h8 diagonal.
        assert!(on_ray(0, 63));
        // a1 ↔ a8 same file.
        assert!(on_ray(0, 56));
        // a1 ↔ h1 same rank.
        assert!(on_ray(0, 7));
        // a1 ↔ b4 off-ray.
        assert!(!on_ray(0, 25));
        // Same square is not on a ray.
        assert!(!on_ray(0, 0));
    }

    #[test]
    fn step_toward_diagonal_and_orthogonal() {
        // a1 (sq 0) → h8 (sq 63) is the main diagonal. Step → b2 (sq 9).
        assert_eq!(step_toward(0, 63), Some(9));
        // e4 (sq 28) → e8 (sq 60), orthogonal N. Step → e5 (sq 36).
        assert_eq!(step_toward(28, 60), Some(36));
        // Same-square is None.
        assert_eq!(step_toward(28, 28), None);
        // Off-ray: a1 → b4 (sq 25). dr=3, df=1 — not a queen-ray.
        assert_eq!(step_toward(0, 25), None);
    }

    #[test]
    fn step_away_diagonal_and_orthogonal() {
        // pivot a1 (0), at b2 (9) — step away → c3 (sq 18).
        assert_eq!(step_away(0, 9), Some(18));
        // pivot e8 (60), at e7 (52) — step away → e6 (sq 44).
        assert_eq!(step_away(60, 52), Some(44));
        // pivot == at → None.
        assert_eq!(step_away(0, 0), None);
        // Off-ray → None.
        assert_eq!(step_away(0, 25), None);
    }

    #[test]
    fn step_away_off_board_returns_none() {
        // pivot b1 (sq 1), at a1 (sq 0). step_away → file -1 → off-board.
        assert_eq!(step_away(1, 0), None);
        // pivot b8 (sq 57), at a8 (sq 56). file -1 → None.
        assert_eq!(step_away(57, 56), None);
        // pivot b7 (sq 49), at a8 (sq 56). diagonal NW off-board.
        assert_eq!(step_away(49, 56), None);
    }
}
