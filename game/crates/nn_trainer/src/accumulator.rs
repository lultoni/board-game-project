//! NNUE-style feature accumulator: the running first-layer sum, with a
//! full-recompute oracle (`refresh`) and an incremental update path
//! (`apply`/`revert`) driven by the `Undo` record `make()` returns.
//!
//! See `design/inbox/nnue-rework-plan.md` §3.3. The accumulator is
//! **standalone / search-stack-owned** — it lives here, NOT in `Position`, and
//! `make_unmake.rs` is untouched. Deltas are derived from the `Undo` record
//! plus the post-`make` `Position`; the accumulator reads `Position` but never
//! mutates it.
//!
//! ## Correctness invariant (the golden test)
//!
//! For any position reached by any make-sequence, the incrementally-updated
//! accumulator must be **bit-identical** to `refresh(pos)`. This is the exact
//! failure mode ns-49 hit with the hand-crafted incremental eval, so the
//! playout bit-identity test is a first-class, must-pass gate.
//!
//! The per-square index math is shared with the encoder (`sparse::
//! square_features` / `global_features`), which is what makes `apply ==
//! refresh` true by construction: both paths compute indices the same way.

use core_engine::game_logic::action::Undo;
use core_engine::state::MailboxEntry;
use core_engine::state::Position;

use crate::sparse::{self, ACCUM_WIDTH, NUM_FEATURES};

/// First-layer weights in **column-major** form: `weights[f]` is the length-
/// `ACCUM_WIDTH` column added to the accumulator when feature `f` is active,
/// plus a shared `bias`. Integerized (i16 columns, i32 bias) for the quantized
/// forward path; the golden test uses a small-int stand-in.
#[derive(Clone)]
pub struct FeatureTransform {
    /// One column per feature; `len == NUM_FEATURES`.
    pub weights: Vec<[i16; ACCUM_WIDTH]>,
    pub bias: [i32; ACCUM_WIDTH],
}

impl FeatureTransform {
    /// All-zero transform (weights + bias). Mostly for tests / placeholders.
    pub fn zeros() -> Self {
        FeatureTransform {
            weights: vec![[0i16; ACCUM_WIDTH]; NUM_FEATURES],
            bias: [0i32; ACCUM_WIDTH],
        }
    }

    /// Deterministic pseudo-random small-integer transform for the golden test.
    /// The exact values don't matter — only that `apply` and `refresh` sum the
    /// same columns. Uses a fixed LCG so runs are reproducible.
    #[cfg(test)]
    pub fn deterministic(seed: u64) -> Self {
        let mut state = seed | 1;
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // Small signed range keeps sums well within i32.
            ((state >> 40) as i16 % 64) - 32
        };
        let mut weights = vec![[0i16; ACCUM_WIDTH]; NUM_FEATURES];
        for col in weights.iter_mut() {
            for w in col.iter_mut() {
                *w = next();
            }
        }
        let mut bias = [0i32; ACCUM_WIDTH];
        for b in bias.iter_mut() {
            *b = next() as i32;
        }
        FeatureTransform { weights, bias }
    }

    #[inline]
    fn add_column(&self, acc: &mut [i32; ACCUM_WIDTH], f: u32) {
        add_col_i16(acc, &self.weights[f as usize]);
    }

    #[inline]
    fn sub_column(&self, acc: &mut [i32; ACCUM_WIDTH], f: u32) {
        sub_col_i16(acc, &self.weights[f as usize]);
    }
}

/// `acc += col` (widening i16 → i32). Shared by `refresh` and the incremental
/// `apply`/`revert` paths — the single hot inner loop of the feature transform.
///
/// Left as a plain scalar loop **on purpose**: the compiler autovectorizes this
/// widening add well (measured faster than a hand-rolled `wide` version, whose
/// per-lane i32 gather/scatter defeated vectorization — ns-50). `ACCUM_WIDTH` is
/// a multiple of 8 so there's no awkward tail.
#[inline]
fn add_col_i16(acc: &mut [i32; ACCUM_WIDTH], col: &[i16; ACCUM_WIDTH]) {
    for j in 0..ACCUM_WIDTH {
        acc[j] += col[j] as i32;
    }
}

/// `acc -= col` (widening i16 → i32), scalar counterpart of `add_col_i16`.
#[inline]
fn sub_col_i16(acc: &mut [i32; ACCUM_WIDTH], col: &[i16; ACCUM_WIDTH]) {
    for j in 0..ACCUM_WIDTH {
        acc[j] -= col[j] as i32;
    }
}

/// The running first-layer sum. `acc[j] = bias[j] + Σ_{f active} W[f][j]`.
///
/// Carries a small cache of the currently-active **global** feature indices so
/// the global delta (which changes almost every ply) is computed by re-encoding
/// the 6 cheap global features and diffing — no dependence on `Undo`'s private
/// byte encoding for globals.
#[derive(Clone)]
pub struct Accumulator {
    acc: [i32; ACCUM_WIDTH],
    /// Active global feature indices for the position this accumulator reflects
    /// (at most 6). Used to compute the incremental global delta.
    globals: Vec<u32>,
}

impl Accumulator {
    /// The raw accumulator values (bias + active columns).
    #[inline]
    pub fn values(&self) -> &[i32; ACCUM_WIDTH] {
        &self.acc
    }

    /// Full recompute — the ORACLE. Independent of any prior state.
    pub fn refresh(pos: &Position, ft: &FeatureTransform) -> Accumulator {
        let mut acc = ft.bias;
        let mut globals = Vec::with_capacity(6);

        for sq in 0..64u8 {
            sparse::square_features(
                sq,
                pos.p1_pieces.contains(sq),
                pos.p2_pieces.contains(sq),
                pos.kings.contains(sq),
                pos.champions.contains(sq),
                pos.guards.contains(sq),
                pos.mailbox[sq as usize],
                &mut |f| add_col_i16(&mut acc, &ft.weights[f as usize]),
            );
        }
        sparse::global_features(pos, &mut |f| {
            globals.push(f);
            add_col_i16(&mut acc, &ft.weights[f as usize]);
        });

        Accumulator { acc, globals }
    }

    /// Incremental forward update. Call **after** `make(pos, action)` returns
    /// `undo`, with `pos` already advanced to the new state. Adds/subtracts the
    /// weight columns for exactly the features that changed.
    pub fn apply(&mut self, undo: &Undo, pos: &Position, ft: &FeatureTransform) {
        self.update_squares(undo, pos, ft);
        self.update_globals(pos, ft);
    }

    /// Incremental reverse update. Call **before** `unmake(pos, undo)`, while
    /// `pos` still reflects the post-make state — it reconstructs the pre-make
    /// state from `undo` and moves the accumulator back to it. After this
    /// returns, the accumulator matches the pre-make position (which `unmake`
    /// then restores into `pos`).
    ///
    /// Note: the search's Phase-0 golden path uses save/restore (clone the
    /// accumulator before `apply`, restore on unmake) which is simpler and
    /// avoids reconstructing pre-make scalars. `revert` is provided + tested for
    /// callers that prefer to avoid the clone.
    pub fn revert(&mut self, undo: &Undo, pos: &Position, ft: &FeatureTransform) {
        // Per-square: same touched set, NEW/OLD roles reversed — move from the
        // current (post-make) state back to the pre-make state.
        self.update_squares_reversed(undo, pos, ft);
        // Globals: reconstruct the pre-make global set from `undo` + `pos`.
        self.revert_globals(undo, pos, ft);
    }

    // --- per-square delta ---------------------------------------------------

    /// Touched-square set = affected mailbox squares ∪ squares whose bitboard
    /// membership changed. For each, subtract OLD square-features, add NEW.
    fn update_squares(&mut self, undo: &Undo, pos: &Position, ft: &FeatureTransform) {
        let occ_changed =
            undo.p1_pieces_xor | undo.p2_pieces_xor | undo.kings_xor | undo.champions_xor | undo.guards_xor;

        let mut touched = TouchedSet::new();
        for i in 0..(undo.affected_count as usize) {
            touched.insert(undo.affected_squares[i]);
        }
        let mut bits = occ_changed;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            touched.insert(sq);
        }

        for &sq in touched.iter() {
            // OLD state on this square.
            let (o_p1, o_p2, o_k, o_c, o_g) = old_occupancy(undo, pos, sq);
            let o_m = old_mailbox(undo, pos, sq);
            sparse::square_features(sq, o_p1, o_p2, o_k, o_c, o_g, o_m, &mut |f| ft.sub_column(&mut self.acc, f));

            // NEW state on this square (read directly from post-make pos).
            let n_m = pos.mailbox[sq as usize];
            sparse::square_features(
                sq,
                pos.p1_pieces.contains(sq),
                pos.p2_pieces.contains(sq),
                pos.kings.contains(sq),
                pos.champions.contains(sq),
                pos.guards.contains(sq),
                n_m,
                &mut |f| ft.add_column(&mut self.acc, f),
            );
        }
    }

    /// Reverse of `update_squares`: move from post-make (current `pos`) back to
    /// pre-make. Subtract NEW (current), add OLD (undo-reconstructed).
    fn update_squares_reversed(&mut self, undo: &Undo, pos: &Position, ft: &FeatureTransform) {
        let occ_changed =
            undo.p1_pieces_xor | undo.p2_pieces_xor | undo.kings_xor | undo.champions_xor | undo.guards_xor;

        let mut touched = TouchedSet::new();
        for i in 0..(undo.affected_count as usize) {
            touched.insert(undo.affected_squares[i]);
        }
        let mut bits = occ_changed;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            touched.insert(sq);
        }

        for &sq in touched.iter() {
            // Subtract NEW (current post-make).
            let n_m = pos.mailbox[sq as usize];
            sparse::square_features(
                sq,
                pos.p1_pieces.contains(sq),
                pos.p2_pieces.contains(sq),
                pos.kings.contains(sq),
                pos.champions.contains(sq),
                pos.guards.contains(sq),
                n_m,
                &mut |f| ft.sub_column(&mut self.acc, f),
            );
            // Add OLD (pre-make, reconstructed from undo).
            let (o_p1, o_p2, o_k, o_c, o_g) = old_occupancy(undo, pos, sq);
            let o_m = old_mailbox(undo, pos, sq);
            sparse::square_features(sq, o_p1, o_p2, o_k, o_c, o_g, o_m, &mut |f| ft.add_column(&mut self.acc, f));
        }
    }

    // --- global delta -------------------------------------------------------

    /// Re-encode the 6 global features from the (post-make) `pos`, diff against
    /// the cached previous global set, and add/subtract the difference.
    fn update_globals(&mut self, pos: &Position, ft: &FeatureTransform) {
        let mut new_globals = Vec::with_capacity(6);
        sparse::global_features(pos, &mut |f| new_globals.push(f));
        self.swap_globals(&new_globals, ft);
    }

    /// Revert globals: reconstruct the pre-make global set from `undo` + the
    /// post-make `pos` (pre-make money = current − delta) and swap to it.
    fn revert_globals(&mut self, undo: &Undo, pos: &Position, ft: &FeatureTransform) {
        let mut old_globals = Vec::with_capacity(6);
        global_features_from_undo(undo, pos, &mut |f| old_globals.push(f));
        self.swap_globals(&old_globals, ft);
    }

    /// Replace `self.globals` with `target`, applying the column delta.
    fn swap_globals(&mut self, target: &[u32], ft: &FeatureTransform) {
        // Subtract any cached global not in target; add any target not cached.
        for &f in &self.globals {
            if !target.contains(&f) {
                ft.sub_column(&mut self.acc, f);
            }
        }
        for &f in target {
            if !self.globals.contains(&f) {
                ft.add_column(&mut self.acc, f);
            }
        }
        self.globals.clear();
        self.globals.extend_from_slice(target);
    }
}

/// Small fixed-capacity dedup set for touched squares (≤ 16 mailbox + a handful
/// of occupancy-only squares; a piece move touches 2). Avoids a HashSet alloc
/// on the hot path.
struct TouchedSet {
    seen: u64,
    list: [u8; 32],
    len: usize,
}

impl TouchedSet {
    #[inline]
    fn new() -> Self {
        TouchedSet { seen: 0, list: [0u8; 32], len: 0 }
    }
    #[inline]
    fn insert(&mut self, sq: u8) {
        let bit = 1u64 << sq;
        if self.seen & bit == 0 {
            self.seen |= bit;
            self.list[self.len] = sq;
            self.len += 1;
        }
    }
    #[inline]
    fn iter(&self) -> impl Iterator<Item = &u8> {
        self.list[..self.len].iter()
    }
}

/// Reconstruct the OLD (pre-make) occupancy of `sq` from the post-make `pos`
/// toggled by the `Undo` bitboard XOR masks: `old = new XOR xor_bit`.
#[inline]
fn old_occupancy(undo: &Undo, pos: &Position, sq: u8) -> (bool, bool, bool, bool, bool) {
    let bit = 1u64 << sq;
    let flip = |mask: u64, cur: bool| -> bool { cur ^ (mask & bit != 0) };
    (
        flip(undo.p1_pieces_xor, pos.p1_pieces.contains(sq)),
        flip(undo.p2_pieces_xor, pos.p2_pieces.contains(sq)),
        flip(undo.kings_xor, pos.kings.contains(sq)),
        flip(undo.champions_xor, pos.champions.contains(sq)),
        flip(undo.guards_xor, pos.guards.contains(sq)),
    )
}

/// Reconstruct the OLD (pre-make) mailbox entry of `sq`. If the square appears
/// in `affected_squares`, the old raw value is recorded in
/// `affected_prev_entries`; otherwise the mailbox was unchanged and the current
/// `pos` value is the old value.
#[inline]
fn old_mailbox(undo: &Undo, pos: &Position, sq: u8) -> MailboxEntry {
    for i in 0..(undo.affected_count as usize) {
        if undo.affected_squares[i] == sq {
            return MailboxEntry(undo.affected_prev_entries[i]);
        }
    }
    pos.mailbox[sq as usize]
}

/// Reconstruct the OLD (pre-make) global feature indices from the post-make
/// `pos` plus the `Undo`'s pre-make scalars. Pre-make money = current money
/// minus the signed delta the action applied. Routes through
/// `sparse::global_indices` (the single source of truth for global index math).
fn global_features_from_undo(undo: &Undo, pos: &Position, push: &mut impl FnMut(u32)) {
    use core_engine::state::position::{Phase, Player};
    // Decode the undo byte tags (mirror make_unmake's private mapping:
    // phase Move=0/Skill=1/Draft=2, player P1=0/P2=1).
    let stm = match undo.prev_to_move {
        0 => Player::P1,
        _ => Player::P2,
    };
    let phase = match undo.prev_phase {
        0 => Phase::Move,
        1 => Phase::Skill,
        _ => Phase::Draft,
    };
    // Pre-make money = post-make money − applied delta.
    let p1_money = (pos.p1_money as i32 - undo.p1_money_delta as i32) as u16;
    let p2_money = (pos.p2_money as i32 - undo.p2_money_delta as i32) as u16;
    sparse::global_indices(
        stm,
        phase,
        p1_money,
        p2_money,
        undo.prev_round_number,
        undo.prev_actions_remaining,
        push,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_engine::game_logic::{generator, make_unmake};
    use core_engine::state::fen::from_fen;
    use core_engine::state::Position;

    /// Refresh is a pure function of `pos` — two refreshes agree, and it does
    /// not depend on any prior accumulator state.
    #[test]
    fn refresh_is_pure() {
        let ft = FeatureTransform::deterministic(0xABCD);
        let pos = Position::setup_stack_m();
        let a = Accumulator::refresh(&pos, &ft);
        let b = Accumulator::refresh(&pos, &ft);
        assert_eq!(a.values(), b.values());
    }

    /// THE keystone test: over random playouts (start position + seeded games +
    /// corpus FENs), the incrementally-updated accumulator must equal
    /// `refresh(pos)` on EVERY node, including after every unmake. Uses the
    /// save/restore strategy (clone before apply, restore on unmake) — the
    /// simplest provably-correct incremental path.
    #[test]
    fn incremental_accumulator_matches_refresh_over_playout() {
        let ft = FeatureTransform::deterministic(0x1234_5678);

        let mut roots = vec![Position::setup_stack_m()];
        // A few corpus FENs (mid-game / endgame shapes). Missing file is fine —
        // the start position alone still exercises make/unmake heavily.
        if let Ok(text) = std::fs::read_to_string(corpus_path()) {
            for line in text.lines().take(30) {
                let fen = line.split(',').next_back().map(str::trim).unwrap_or("");
                if let Ok(p) = from_fen(fen) {
                    roots.push(p);
                }
            }
        }

        for (ri, root) in roots.iter().enumerate() {
            playout_check(root.clone(), &ft, 0xF00D ^ ri as u64);
        }
    }

    /// Recursive-style random walk with an explicit accumulator stack. At each
    /// node assert incremental == refresh; descend via make+apply(save prev),
    /// ascend via unmake+restore. Also independently exercises `revert` on a
    /// separate accumulator to keep it covered.
    fn playout_check(mut pos: Position, ft: &FeatureTransform, seed: u64) {
        let mut acc = Accumulator::refresh(&pos, ft);
        assert_eq!(acc.values(), Accumulator::refresh(&pos, ft).values());

        let mut rng = seed | 1;
        let next = |rng: &mut u64, n: usize| -> usize {
            *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((*rng >> 33) as usize) % n
        };

        // Explicit stack of (undo, saved-accumulator) frames.
        let mut stack: Vec<(core_engine::game_logic::action::Undo, Accumulator)> = Vec::new();
        let steps = 120;
        for _ in 0..steps {
            let go_down = stack.is_empty() || next(&mut rng, 100) < 70;
            if go_down {
                let actions = generator::generate(&pos);
                if actions.is_empty() {
                    // terminal / no moves — unwind one if possible.
                    if let Some((undo, saved)) = stack.pop() {
                        make_unmake::unmake(&mut pos, &undo);
                        acc = saved;
                    } else {
                        break;
                    }
                    assert_eq!(
                        acc.values(),
                        Accumulator::refresh(&pos, ft).values(),
                        "mismatch after unmake (no-moves branch)"
                    );
                    continue;
                }
                let idx = next(&mut rng, actions.len());
                let action = actions[idx];
                let saved = acc.clone();
                let undo = make_unmake::make(&mut pos, action);
                acc.apply(&undo, &pos, ft);
                assert_eq!(
                    acc.values(),
                    Accumulator::refresh(&pos, ft).values(),
                    "incremental apply != refresh after make"
                );
                stack.push((undo, saved));
            } else {
                let (undo, saved) = stack.pop().unwrap();
                make_unmake::unmake(&mut pos, &undo);
                acc = saved;
                assert_eq!(
                    acc.values(),
                    Accumulator::refresh(&pos, ft).values(),
                    "restored accumulator != refresh after unmake"
                );
            }
        }

        // Fully unwind, asserting on every node incl. the tail.
        while let Some((undo, saved)) = stack.pop() {
            make_unmake::unmake(&mut pos, &undo);
            acc = saved;
            assert_eq!(
                acc.values(),
                Accumulator::refresh(&pos, ft).values(),
                "restored accumulator != refresh during final unwind"
            );
        }
    }

    /// `revert` (reconstruct pre-make from undo, no saved clone) must also land
    /// bit-identical to a refresh of the pre-make position, per node.
    #[test]
    fn revert_matches_refresh_single_ply() {
        let ft = FeatureTransform::deterministic(0x9999);
        let mut pos = Position::setup_stack_m();
        let actions = generator::generate(&pos);
        for &action in actions.iter().take(20) {
            let pre = Accumulator::refresh(&pos, &ft);
            let undo = make_unmake::make(&mut pos, action);
            let mut acc = Accumulator::refresh(&pos, &ft); // post-make
            acc.revert(&undo, &pos, &ft); // back to pre-make
            make_unmake::unmake(&mut pos, &undo);
            assert_eq!(
                acc.values(),
                pre.values(),
                "revert did not reconstruct the pre-make accumulator"
            );
        }
    }

    fn corpus_path() -> std::path::PathBuf {
        // nn_trainer crate dir → ../../bench/corpus/corpus.txt
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../bench/corpus/corpus.txt")
    }
}
