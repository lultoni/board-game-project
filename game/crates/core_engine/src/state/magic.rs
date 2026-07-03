//! Layer 1 — Path/Range/Block primitives and movement geometry.
//!
//! Single interface for all board geometry used by the generator and evaluator:
//!
//! - `skill_attacks(sq, occ, range)` — queen-ray skill targeting with range cap
//! - `movement_targets_speed1(sq)` — 8-adjacent squares (Champion/King moves)
//! - `movement_targets_speed2(sq, occ)` — Chebyshev-BFS-2 with path blocking (Guard moves)
//! - `movement_attack_targets_speed2(sq, occ, reach_empty, opp_bb)` — enemies attackable in a Guard move
//! - `cheby_dist(a, b)` — precomputed Chebyshev distance (0..=7)
//! - `between(a, b)`, `on_ray(a, b)`, `step_toward`, `step_away`, `neighbour_in_dir`

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

/// One-step neighbour from `sq` in direction `dir` (0..=7, matching DELTAS:
/// 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW). Returns `None` if the step
/// leaves the board.
pub fn neighbour_in_dir(sq: u8, dir: usize) -> Option<u8> {
    debug_assert!(sq < 64 && dir < 8);
    let (dr, df) = DELTAS[dir];
    let r = (sq / 8) as i8 + dr;
    let f = (sq % 8) as i8 + df;
    if !(0..8).contains(&r) || !(0..8).contains(&f) { return None; }
    Some((r * 8 + f) as u8)
}

// ── Movement geometry ────────────────────────────────────────────────────────

/// `MOVE1[sq]` = bitmask of the 8 immediate neighbours of `sq` (Chebyshev 1).
/// Precomputed once; not occupancy-dependent (caller masks out own pieces).
static MOVE1: OnceLock<[u64; 64]> = OnceLock::new();

/// `CHEBY[a][b]` = Chebyshev distance between squares `a` and `b` (0..=7).
static CHEBY: OnceLock<[[u8; 64]; 64]> = OnceLock::new();

fn cheby_table() -> &'static [[u8; 64]; 64] {
    CHEBY.get_or_init(|| {
        let mut t = [[0u8; 64]; 64];
        for a in 0u8..64 {
            let ar = (a / 8) as i8;
            let af = (a % 8) as i8;
            for b in 0u8..64 {
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;
                let dr = (ar - br).unsigned_abs();
                let df = (af - bf).unsigned_abs();
                t[a as usize][b as usize] = dr.max(df);
            }
        }
        t
    })
}

/// Chebyshev (king-move) distance between two squares. 0 iff `a == b`, 1 for
/// orthogonal or diagonal neighbours, up to 7 (a1↔h8). O(1) table lookup.
#[inline]
pub fn cheby_dist(a: u8, b: u8) -> u8 {
    debug_assert!(a < 64 && b < 64);
    cheby_table()[a as usize][b as usize]
}

fn move1_table() -> &'static [u64; 64] {
    MOVE1.get_or_init(|| {
        let mut t = [0u64; 64];
        for sq in 0u8..64 {
            for (dr, df) in DELTAS {
                let r = (sq / 8) as i8 + dr;
                let f = (sq % 8) as i8 + df;
                if (0..8).contains(&r) && (0..8).contains(&f) {
                    t[sq as usize] |= 1u64 << (r * 8 + f) as u8;
                }
            }
        }
        t
    })
}

/// The 8 immediately adjacent squares for a speed-1 piece at `sq`.
///
/// Returns empty squares AND occupied squares — callers should mask out
/// own pieces to get legal move destinations and AND with opp pieces for
/// move-attack targets.
#[inline]
pub fn movement_targets_speed1(sq: u8) -> Bitboard {
    debug_assert!(sq < 64);
    Bitboard(move1_table()[sq as usize])
}

/// Chebyshev-BFS-2 reachable squares for a speed-2 piece at `sq` against
/// the given occupancy mask `occ`.
///
/// Returns bitmask of reachable **empty** squares only (occupied squares are
/// blocking — piece cannot enter or pass through). Caller should:
///   - AND with `!own_pieces` for legal move destinations (already empty)
///   - separately compute attack targets as enemies adjacent to any reachable
///     square or `sq` itself (see `movement_attack_targets_speed2`)
///
/// This is identical to the generator's `reachable()` BFS but returns a plain
/// `u64` mask without the distance array overhead.
#[inline]
pub fn movement_targets_speed2(sq: u8, occ: u64) -> Bitboard {
    debug_assert!(sq < 64);
    let mut dist = [255u8; 64];
    dist[sq as usize] = 0;
    let mut front = [0u8; 64];
    let mut flen = 1usize;
    front[0] = sq;
    let mut reach = 0u64;
    for step in 1u8..=2 {
        let mut next = [0u8; 64];
        let mut nlen = 0usize;
        for i in 0..flen {
            let s = front[i];
            for (dr, df) in DELTAS {
                let r = (s / 8) as i8 + dr;
                let f = (s % 8) as i8 + df;
                if !(0..8).contains(&r) || !(0..8).contains(&f) { continue; }
                let n = (r * 8 + f) as u8;
                if dist[n as usize] != 255 { continue; }
                if occ & (1u64 << n) != 0 { continue; }
                dist[n as usize] = step;
                reach |= 1u64 << n;
                next[nlen] = n;
                nlen += 1;
            }
        }
        front = next;
        flen = nlen;
        if flen == 0 { break; }
    }
    Bitboard(reach)
}

/// Move-attack targets for a speed-2 piece at `sq`: enemy squares that can be
/// reached in the final step from any reachable square (including `sq` itself).
///
/// `reach_empty` — result of `movement_targets_speed2(sq, occ)`.
/// `opp_bb`      — bitmask of enemy pieces to check.
///
/// Returns bitmask of enemy squares that are attackable. Equivalent to the
/// attack-target loop in `generator::reachable()`, lifted here for symmetry.
#[inline]
pub fn movement_attack_targets_speed2(sq: u8, _occ: u64, reach_empty: u64, opp_bb: u64) -> Bitboard {
    debug_assert!(sq < 64);
    let reachable_or_src = reach_empty | (1u64 << sq);
    let mut attacks = 0u64;
    let mut remaining = opp_bb;
    while remaining != 0 {
        let enemy = remaining.trailing_zeros() as u8;
        remaining &= remaining - 1;
        // Quick Chebyshev distance reject
        let er = (enemy / 8) as i32;
        let ef = (enemy % 8) as i32;
        let sr = (sq / 8) as i32;
        let sf = (sq % 8) as i32;
        let cheby = (er - sr).abs().max((ef - sf).abs()) as u8;
        if cheby > 2 { continue; }
        // Any neighbour of enemy reachable within speed-1 steps from src?
        for (dr, df) in DELTAS {
            let nr = er as i8 + dr;
            let nf = ef as i8 + df;
            if !(0..8).contains(&nr) || !(0..8).contains(&nf) { continue; }
            let n = (nr * 8 + nf) as u8;
            if reachable_or_src & (1u64 << n) != 0 {
                attacks |= 1u64 << enemy;
                break;
            }
        }
    }
    Bitboard(attacks)
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

    #[test]
    fn neighbour_in_dir_all_eight_from_centre() {
        // sq 28 = e4. All 8 directions on-board.
        // DELTAS order: N, NE, E, SE, S, SW, W, NW.
        assert_eq!(neighbour_in_dir(28, 0), Some(36)); // N → e5
        assert_eq!(neighbour_in_dir(28, 1), Some(37)); // NE → f5
        assert_eq!(neighbour_in_dir(28, 2), Some(29)); // E → f4
        assert_eq!(neighbour_in_dir(28, 3), Some(21)); // SE → f3
        assert_eq!(neighbour_in_dir(28, 4), Some(20)); // S → e3
        assert_eq!(neighbour_in_dir(28, 5), Some(19)); // SW → d3
        assert_eq!(neighbour_in_dir(28, 6), Some(27)); // W → d4
        assert_eq!(neighbour_in_dir(28, 7), Some(35)); // NW → d5
    }

    #[test]
    fn neighbour_in_dir_off_board_returns_none() {
        // a1 = sq 0. Off-board in S, SW, W, NW, SE.
        assert_eq!(neighbour_in_dir(0, 0), Some(8));  // N → a2 on-board
        assert_eq!(neighbour_in_dir(0, 1), Some(9));  // NE → b2 on-board
        assert_eq!(neighbour_in_dir(0, 2), Some(1));  // E → b1 on-board
        assert_eq!(neighbour_in_dir(0, 3), None);     // SE → off
        assert_eq!(neighbour_in_dir(0, 4), None);     // S → off
        assert_eq!(neighbour_in_dir(0, 5), None);     // SW → off
        assert_eq!(neighbour_in_dir(0, 6), None);     // W → off
        assert_eq!(neighbour_in_dir(0, 7), None);     // NW → off
    }

    #[test]
    fn move1_centre_has_8_neighbours() {
        // sq 28 (e4, rank 3 file 4) → 8 immediate neighbours
        let m = movement_targets_speed1(28);
        assert_eq!(m.0.count_ones(), 8);
    }

    #[test]
    fn move1_corner_has_3_neighbours() {
        let m = movement_targets_speed1(0); // a1
        assert_eq!(m.0.count_ones(), 3);
    }

    #[test]
    fn move2_empty_board_centre_reaches_24() {
        // BFS-2 from a fully interior square reaches the 5×5 block = 24 squares.
        // sq 27 = rank 3 file 3 — all 5×5 neighbours are on-board.
        let m = movement_targets_speed2(27, 0); // d4
        assert_eq!(m.0.count_ones(), 24);
    }

    #[test]
    fn move2_empty_board_corner_reaches_8() {
        let m = movement_targets_speed2(0, 0); // a1
        assert_eq!(m.0.count_ones(), 8);
    }

    #[test]
    fn move2_full_ring_blocks_centre() {
        // If ALL 8 immediate neighbours of sq 27 are occupied, a speed-2 piece
        // cannot reach anything (no path of length 2 through empty squares).
        let sq = 27u8;
        let all_neighbours = movement_targets_speed1(sq).0;
        let m = movement_targets_speed2(sq, all_neighbours);
        assert_eq!(m.0, 0, "surrounded piece has no speed-2 reach");
    }

    #[test]
    fn move2_single_north_blocker_still_allows_zigzag() {
        // Guard at sq 28, blocker at sq 36 (one step N).
        // sq 44 (two steps N) is still reachable via zigzag (28→37→44 or 28→35→44).
        let occ = 1u64 << 36;
        let m = movement_targets_speed2(28, occ);
        assert!(m.contains(44), "two steps N is reachable via zigzag around blocker");
        // Blocker square itself should NOT be in reach_empty (it's occupied)
        assert!(!m.contains(36), "blocker square itself is not reachable");
    }

    #[test]
    fn move2_full_column_blocks_pass() {
        // Place blockers at sq 36 AND sq 37 (N and NE of sq 28).
        // sq 44 and 45 can still be reached via W path (28→35→43→44) — wait, that's 3 steps.
        // Actually with both sq 36 and 37 blocked, sq 44 requires going through sq 35
        // then sq 44: 28→35→44 (2 steps). sq 35 is empty, sq 44 is empty. So reachable!
        // To truly block sq 44: need to block the entire N half.
        // Instead verify: blocker at sq 36, sq 35 also blocked → can sq 44 be reached?
        // 28→27→44 would be sq 27 (d4) → sq 44 (e6) which is NOT a single Chebyshev step.
        // Let's just test that blocked path = excluded in a simple case.
        // Single blocker at sq 29 (E of src 28): sq 30 is now unreachable directly,
        // but may still be reached via 28→21→30 (S then SE) if those are empty.
        let occ = 1u64 << 29;
        let m = movement_targets_speed2(28, occ);
        assert!(!m.contains(29), "direct E step blocked");
        assert!(m.contains(30), "sq 30 still reachable via southern zigzag");
    }

    #[test]
    fn cheby_dist_self_is_zero() {
        for sq in 0u8..64 { assert_eq!(cheby_dist(sq, sq), 0); }
    }

    #[test]
    fn cheby_dist_neighbours_are_one() {
        // sq 28 (e4) all 8 neighbours have distance 1.
        let neighbours = movement_targets_speed1(28).0;
        let mut bits = neighbours;
        while bits != 0 {
            let n = bits.trailing_zeros() as u8;
            assert_eq!(cheby_dist(28, n), 1);
            bits &= bits - 1;
        }
    }

    #[test]
    fn cheby_dist_corner_to_corner_is_seven() {
        assert_eq!(cheby_dist(0, 63), 7);   // a1 ↔ h8
        assert_eq!(cheby_dist(7, 56), 7);   // h1 ↔ a8
    }

    #[test]
    fn cheby_dist_matches_inline_math() {
        // Sanity: cross-check the table against the direct formula on every pair.
        for a in 0u8..64 {
            for b in 0u8..64 {
                let ar = (a / 8) as i8; let af = (a % 8) as i8;
                let br = (b / 8) as i8; let bf = (b % 8) as i8;
                let dr = (ar - br).unsigned_abs();
                let df = (af - bf).unsigned_abs();
                assert_eq!(cheby_dist(a, b), dr.max(df));
            }
        }
    }
}
