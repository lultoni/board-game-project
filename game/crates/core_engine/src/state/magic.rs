//! Layer 1 - Path/Range/Block primitives and movement geometry.
//!
//! Single interface for all board geometry used by the generator and evaluator:
//!
//! - `skill_attacks(sq, occ, range)` - queen-ray skill targeting with range cap
//! - `movement_targets_speed1(sq)` - 8-adjacent squares (Champion/King moves)
//! - `movement_targets_speed2(sq, occ)` - Chebyshev-BFS-2 with path blocking (Guard moves)
//! - `movement_attack_targets_speed2(sq, occ, reach_empty, opp_bb)` - enemies attackable in a Guard move
//! - `cheby_dist(a, b)` - precomputed Chebyshev distance (0..=7)
//! - `between(a, b)`, `on_ray(a, b)`, `step_toward`, `step_away`, `neighbour_in_dir`
//!
//! ## This module actually is magic bitboards (the slider path)
//!
//! `skill_attacks` - the only occupancy-dependent *sliding* query - resolves via
//! classic **split rook + bishop plain-magic bitboards**: per-square
//! `(mask, magic, shift, attack-table)` so a query is
//! `(occ & mask).wrapping_mul(magic) >> shift → table[idx]` with NO per-call ray
//! walk. The rook + bishop tables are OR'd (queen = rook ∪ bishop) and the
//! Chebyshev range cap is applied afterward via `within_range`. Tables (~0.82 MB
//! total) and the 128 magic numbers are built once at first use in the same
//! `OnceLock` idiom as `RAYS`/`BETWEEN`. **PEXT is deliberately NOT used** - it's a
//! BMI2 (x86) instruction; this project targets aarch64, where plain multiply-
//! shift magics are the correct technique.
//!
//! `movement_targets_speed2` (Guard BFS-2) is occupancy-dependent but a magic
//! table for it is infeasible (24 relevant blocker bits ⇒ ~2 GB). Instead it uses
//! an allocation-free **bitboard flood-fill**: two king-dilation steps masked by
//! empties, reusing `MOVE1`. Proven equivalent to the old BFS.
//!
//! Everything else here is a pure precomputed table lookup (`between`, `cheby`,
//! `MOVE1`, `within_range`) or cheaper-than-a-lookup integer math (`on_ray`,
//! `step_*`, `neighbour_in_dir`) - none benefit from magic, so they are left as-is.

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

/// `BETWEEN[a][b]` = bitboard of squares strictly between `a` and `b` on the
/// queen-ray that connects them (empty if they don't share a ray or `a == b`).
static BETWEEN: OnceLock<[[u64; 64]; 64]> = OnceLock::new();

/// `WITHIN_RANGE[sq][r]` = bitboard of all squares with Chebyshev distance
/// `1..=r` from `sq`. Index 0 is the empty bitboard.
static WITHIN_RANGE: OnceLock<[[u64; 5]; 64]> = OnceLock::new();

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

// ── Magic bitboards for sliding (skill) attacks ──────────────────────────────
//
// Split rook + bishop plain magics. A query masks occupancy to the relevant
// blocker squares, multiplies by the per-square magic, shifts down to an index,
// and reads a precomputed blocker-inclusive attack set. Rook ∪ bishop = queen.

/// Split rook+bishop plain-magic slider tables, built once. `*_attacks` are flat
/// Vecs indexed by `*_offset[sq] + ((occ & mask) * magic >> shift)`.
struct SliderTables {
    rook_mask:     [u64; 64],
    rook_magic:    [u64; 64],
    rook_shift:    [u32; 64],
    rook_offset:   [usize; 64],
    rook_attacks:  Vec<u64>,
    bishop_mask:   [u64; 64],
    bishop_magic:  [u64; 64],
    bishop_shift:  [u32; 64],
    bishop_offset: [usize; 64],
    bishop_attacks: Vec<u64>,
}

static SLIDER: OnceLock<SliderTables> = OnceLock::new();

/// Deterministic xorshift64 RNG for magic search. `Math.random`/`Date::now` are
/// unavailable/banned; a fixed seed keeps `find_magics` reproducible so tables
/// are identical across runs (important for the determinism gate).
struct MagicRng(u64);
impl MagicRng {
    #[inline]
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// Sparse candidate - AND of three draws biases toward few set bits, which
    /// is where good magics cluster.
    #[inline]
    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

/// Is direction `d` orthogonal (rook) vs diagonal (bishop)?
#[inline]
fn is_ortho(d: usize) -> bool {
    let (dr, df) = DELTAS[d];
    dr == 0 || df == 0
}

/// Relevant-occupancy mask for a rook (`ortho=true`) or bishop (`ortho=false`)
/// at `sq`: every ray square whose *next* step is still on-board (the edge
/// square can't block anything beyond it, so it's excluded - the standard
/// magic relevant mask).
fn slider_relevant_mask(sq: u8, ortho: bool) -> u64 {
    let mut m = 0u64;
    for d in 0..8 {
        if is_ortho(d) != ortho { continue; }
        let (dr, df) = DELTAS[d];
        let mut r = (sq / 8) as i8 + dr;
        let mut f = (sq % 8) as i8 + df;
        loop {
            let on = (0..8).contains(&r) && (0..8).contains(&f);
            if !on { break; }
            let nr = r + dr;
            let nf = f + df;
            let next_on = (0..8).contains(&nr) && (0..8).contains(&nf);
            if next_on {
                m |= 1u64 << (r * 8 + f) as u8;
            }
            r = nr;
            f = nf;
        }
    }
    m
}

/// Reference blocker-inclusive ray attacks for a rook/bishop at `sq` against
/// `occ`: walk each relevant ray, include every empty square and the first
/// blocker, stop at the blocker. Used only at table-build time.
fn slider_reference_attacks(sq: u8, occ: u64, ortho: bool) -> u64 {
    let mut a = 0u64;
    for d in 0..8 {
        if is_ortho(d) != ortho { continue; }
        let (dr, df) = DELTAS[d];
        let mut r = (sq / 8) as i8 + dr;
        let mut f = (sq % 8) as i8 + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let s = (r * 8 + f) as u8;
            a |= 1u64 << s;
            if occ & (1u64 << s) != 0 { break; }
            r += dr;
            f += df;
        }
    }
    a
}

/// Enumerate every occupancy subset of `mask` (Carry-Rippler), in order.
fn occupancy_subsets(mask: u64) -> Vec<u64> {
    let mut out = Vec::with_capacity(1usize << mask.count_ones());
    let mut sub: u64 = 0;
    loop {
        out.push(sub);
        sub = sub.wrapping_sub(mask) & mask;
        if sub == 0 { break; }
    }
    out
}

/// Find a collision-free magic for one square/piece and return
/// `(magic, shift, attack_table)`. `attack_table` has `1 << bits` entries.
fn find_one_magic(sq: u8, ortho: bool, rng: &mut MagicRng) -> (u64, u32, Vec<u64>) {
    let mask = slider_relevant_mask(sq, ortho);
    let bits = mask.count_ones();
    let shift = 64 - bits;
    let size = 1usize << bits;
    let subsets = occupancy_subsets(mask);
    let refs: Vec<u64> = subsets
        .iter()
        .map(|&o| slider_reference_attacks(sq, o, ortho))
        .collect();

    for _ in 0..100_000_000u64 {
        let magic = rng.sparse();
        // Cheap reject: a good magic spreads the mask's high bits.
        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 { continue; }
        let mut table = vec![u64::MAX; size];
        let mut ok = true;
        for (i, &o) in subsets.iter().enumerate() {
            let idx = ((o.wrapping_mul(magic)) >> shift) as usize;
            if table[idx] == u64::MAX {
                table[idx] = refs[i];
            } else if table[idx] != refs[i] {
                ok = false;
                break;
            }
        }
        if ok {
            return (magic, shift, table);
        }
    }
    // 8×8 magics are dense; failure would indicate a logic bug, not bad luck.
    panic!("no magic found for sq {sq} ortho {ortho}");
}

fn slider() -> &'static SliderTables {
    SLIDER.get_or_init(|| {
        let mut rng = MagicRng(0x1234_5678_9abc_def1);
        let mut t = SliderTables {
            rook_mask: [0; 64],
            rook_magic: [0; 64],
            rook_shift: [0; 64],
            rook_offset: [0; 64],
            rook_attacks: Vec::new(),
            bishop_mask: [0; 64],
            bishop_magic: [0; 64],
            bishop_shift: [0; 64],
            bishop_offset: [0; 64],
            bishop_attacks: Vec::new(),
        };
        for sq in 0u8..64 {
            // Rook.
            let (rm, rs, rtab) = find_one_magic(sq, true, &mut rng);
            t.rook_mask[sq as usize] = slider_relevant_mask(sq, true);
            t.rook_magic[sq as usize] = rm;
            t.rook_shift[sq as usize] = rs;
            t.rook_offset[sq as usize] = t.rook_attacks.len();
            t.rook_attacks.extend_from_slice(&rtab);
            // Bishop.
            let (bm, bs, btab) = find_one_magic(sq, false, &mut rng);
            t.bishop_mask[sq as usize] = slider_relevant_mask(sq, false);
            t.bishop_magic[sq as usize] = bm;
            t.bishop_shift[sq as usize] = bs;
            t.bishop_offset[sq as usize] = t.bishop_attacks.len();
            t.bishop_attacks.extend_from_slice(&btab);
        }
        t
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
/// Retreat/Shove at +1 = 4; the `within_range` cap covers 0..=4).
///
/// Resolved via split rook+bishop plain-magic bitboards (see `SliderTables`):
/// two `(occ & mask) * magic >> shift` lookups OR'd (queen = rook ∪ bishop),
/// then AND'd with the Chebyshev range mask. No per-call ray walk.
pub fn skill_attacks(sq: u8, occ: u64, range: u8) -> Bitboard {
    debug_assert!(sq < 64);
    // Max reachable range in Stack M is 4 (range-3 skill + Focus +1). The magic
    // tables are range-independent (full queen attack); only this cap depends on
    // `range`, and `within_range` covers 0..=4. Clamp keeps us in-bounds if a
    // future skill ever passes a larger value.
    debug_assert!(range <= 4, "skill range {range} exceeds within_range table (0..=4)");
    if range == 0 {
        return Bitboard::EMPTY;
    }
    let t = slider();
    let s = sq as usize;
    let rook_idx = ((occ & t.rook_mask[s]).wrapping_mul(t.rook_magic[s]) >> t.rook_shift[s]) as usize;
    let bishop_idx = ((occ & t.bishop_mask[s]).wrapping_mul(t.bishop_magic[s]) >> t.bishop_shift[s]) as usize;
    let queen = t.rook_attacks[t.rook_offset[s] + rook_idx]
        | t.bishop_attacks[t.bishop_offset[s] + bishop_idx];
    // Apply the Chebyshev range cap.
    Bitboard(queen & within_range_table()[s][range.min(4) as usize])
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

/// King-dilate a whole bitboard: OR every set square with its 8 immediate
/// neighbours (the result INCLUDES the input set). Branchless shift-mask form -
/// faster than iterating set bits, and the single shared primitive for all
/// bitboard dilation in the engine (SEE fanout, evaluator neighbourhoods, the
/// speed-2 flood-fill below).
#[inline]
pub fn king_expand(x: u64) -> u64 {
    const NOT_A: u64 = 0xfefe_fefe_fefe_fefe; // !file A
    const NOT_H: u64 = 0x7f7f_7f7f_7f7f_7f7f; // !file H
    let l = (x & NOT_A) >> 1;
    let r = (x & NOT_H) << 1;
    let h = x | l | r;
    h | (h << 8) | (h >> 8)
}

/// The 8 immediately adjacent squares for a speed-1 piece at `sq`.
///
/// Returns empty squares AND occupied squares - callers should mask out
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
/// blocking - piece cannot enter or pass through). Caller should:
///   - AND with `!own_pieces` for legal move destinations (already empty)
///   - separately compute attack targets as enemies adjacent to any reachable
///     square or `sq` itself (see `movement_attack_targets_speed2`)
///
/// Allocation-free bitboard flood-fill: two king-dilation steps, each masked to
/// empty squares (and excluding the origin). `s1` = empties reachable in one
/// step; `s2` = empties reachable from `sq ∪ s1` (i.e. within two steps). Proven
/// equivalent to the previous per-call BFS (see the `speed2_flood_matches_bfs`
/// test). `king_expand` includes its input, so `& !start` drops the origin.
#[inline]
pub fn movement_targets_speed2(sq: u8, occ: u64) -> Bitboard {
    debug_assert!(sq < 64);
    let empty = !occ;
    let start = 1u64 << sq;
    let s1 = king_expand(start) & empty & !start;
    let s2 = king_expand(start | s1) & empty & !start;
    Bitboard((s1 | s2) & !start)
}

/// Move-attack targets for a speed-2 piece at `sq`: enemy squares that can be
/// reached in the final step from any reachable square (including `sq` itself).
///
/// `reach_empty` - result of `movement_targets_speed2(sq, occ)`.
/// `opp_bb`      - bitmask of enemy pieces to check.
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

    // ── Reference implementations (the pre-magic versions) ───────────────────
    // These are the exact algorithms `skill_attacks` and `movement_targets_speed2`
    // used before the magic-bitboard / flood-fill rewrite. Kept here so the
    // equivalence tests prove the fast paths return byte-identical results.

    /// Old `skill_attacks`: per-call 8-ray walk with blocker-inclusive stop and
    /// a Chebyshev range cap.
    fn ref_skill_attacks(sq: u8, occ: u64, range: u8) -> u64 {
        if range == 0 { return 0; }
        let mut out = 0u64;
        for d in 0..8 {
            let (dr, df) = DELTAS[d];
            let mut r = (sq / 8) as i8 + dr;
            let mut f = (sq % 8) as i8 + df;
            while (0..8).contains(&r) && (0..8).contains(&f) {
                let s = (r * 8 + f) as u8;
                out |= 1u64 << s;
                if occ & (1u64 << s) != 0 { break; }
                r += dr;
                f += df;
            }
        }
        out & within_range_table()[sq as usize][range.min(4) as usize]
    }

    /// Old `movement_targets_speed2`: BFS-2 over empty squares.
    fn ref_speed2(sq: u8, occ: u64) -> u64 {
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
        reach
    }

    /// Deterministic xorshift for test occupancies (no `rand`, no `Math.random`).
    struct TestRng(u64);
    impl TestRng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            self.0 = x; x
        }
    }

    #[test]
    fn skill_attacks_matches_reference_over_random_occupancies() {
        let mut rng = TestRng(0xF00D_BABE_1234_5678);
        for sq in 0u8..64 {
            for _ in 0..4000 {
                let occ = rng.next() & !(1u64 << sq); // caster square unoccupied
                for range in 1u8..=4 {
                    let got = skill_attacks(sq, occ, range).0;
                    let want = ref_skill_attacks(sq, occ, range);
                    assert_eq!(
                        got, want,
                        "skill_attacks mismatch sq={sq} range={range} occ={occ:#018x}"
                    );
                }
                // range 0 must be empty.
                assert_eq!(skill_attacks(sq, occ, 0).0, 0);
            }
        }
    }

    #[test]
    fn speed2_flood_matches_bfs() {
        let mut rng = TestRng(0x1357_9BDF_2468_ACE0);
        for sq in 0u8..64 {
            for _ in 0..4000 {
                let occ = rng.next() & !(1u64 << sq);
                let got = movement_targets_speed2(sq, occ).0;
                let want = ref_speed2(sq, occ);
                assert_eq!(got, want, "speed2 mismatch sq={sq} occ={occ:#018x}");
            }
        }
    }

    #[test]
    fn magic_tables_are_deterministic_across_builds() {
        // The magic search uses a fixed seed, so an independent rebuild of the
        // per-square rook/bishop attack tables via the reference walk must agree
        // with what the live tables return for every occupancy subset.
        let t = slider();
        for sq in 0u8..64 {
            let s = sq as usize;
            for (mask, magic, shift, off, tab, ortho) in [
                (t.rook_mask[s], t.rook_magic[s], t.rook_shift[s], t.rook_offset[s], &t.rook_attacks, true),
                (t.bishop_mask[s], t.bishop_magic[s], t.bishop_shift[s], t.bishop_offset[s], &t.bishop_attacks, false),
            ] {
                for occ in occupancy_subsets(mask) {
                    let idx = ((occ.wrapping_mul(magic)) >> shift) as usize;
                    assert_eq!(
                        tab[off + idx],
                        slider_reference_attacks(sq, occ, ortho),
                        "slider table mismatch sq={sq} ortho={ortho} occ={occ:#018x}"
                    );
                }
            }
        }
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
        // a1 = sq 0, h8 = sq 63. Diagonal - between should be {b2, c3, d4, e5, f6, g7}.
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
        // Off-ray: a1 → b4 (sq 25). dr=3, df=1 - not a queen-ray.
        assert_eq!(step_toward(0, 25), None);
    }

    #[test]
    fn step_away_diagonal_and_orthogonal() {
        // pivot a1 (0), at b2 (9) - step away → c3 (sq 18).
        assert_eq!(step_away(0, 9), Some(18));
        // pivot e8 (60), at e7 (52) - step away → e6 (sq 44).
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
        // sq 27 = rank 3 file 3 - all 5×5 neighbours are on-board.
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
        // sq 44 and 45 can still be reached via W path (28→35→43→44) - wait, that's 3 steps.
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
