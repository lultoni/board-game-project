//! Transposition table - preallocated `Vec<Entry>` keyed by Zobrist hash
//! modulo table size. No dynamic allocation in the hot path.
//!
//! # Replacement policy
//!
//! Single slot per index, depth-preferred with a generation counter.
//! `store(entry)` overwrites iff any of:
//!
//! 1. The slot is empty (`existing.key == 0`).
//! 2. The slot's entry is from an older generation
//!    (`existing.generation != tt.generation`).
//! 3. The new entry's depth ≥ the existing entry's depth.
//! 4. The new entry has the same key as the existing one (a re-search of
//!    the same position always wins, even at shallower depth - the new
//!    score reflects more recent information).
//!
//! Otherwise the store is rejected. This is the canonical scheme: deeper,
//! more authoritative entries displace shallow ones; old entries from
//! previous searches are reclaimable. Not Stockfish-grade (no two-tier
//! depth+always slot pair) but a robust baseline. Revisit after the
//! alpha-beta integration slice can measure hit rate empirically.
//!
//! # Key=0 sentinel
//!
//! We treat `key == 0` as "slot empty". Any real position whose Zobrist
//! hash happens to be exactly 0 will be indistinguishable from an empty
//! slot - a missed TT hit, never a correctness bug. The probability over
//! a 10⁹-node search horizon is ~5×10⁻¹¹; accepted.
//!
//! # Generation counter
//!
//! `new_search()` increments the table's generation byte and resets stats.
//! Entries stored after that carry the new generation; old entries become
//! eligible for replacement regardless of depth. This is an O(1) "soft
//! clear" - far cheaper than zeroing the table. Wraparound at 256 is fine:
//! by the time we cycle through 256 generations, every slot has been
//! overwritten many times.
//!
//! # Mate-score handling
//!
//! Mate-in-N scores must be adjusted by ply distance when stored and
//! probed (the same mate looks different from different search depths).
//! This is a *search-side* concern - the TT doesn't know which ply it's
//! at. Alpha-beta will wrap probe/store with the standard
//! `score ± ply` adjustment when that slice lands. The TT itself stores
//! raw scores.
//!
//! # Threading
//!
//! Single-threaded for now (per ADR-005: web target runs in a Web Worker,
//! desktop Lazy-SMP arrives later). Stats are plain `u64`, no atomics.

use crate::game_logic::action::Action;

/// Score bound for a stored entry - alpha-beta interpretation of the score.
///
/// - `Exact` - score is the true minimax value (fully searched window).
/// - `Lower` - score is a lower bound (fail-high; opponent had a better move).
/// - `Upper` - score is an upper bound (fail-low; we couldn't reach alpha).
///
/// `#[repr(u8)]` keeps the field one byte; the variant order matches the
/// pre-Slice-8 raw-u8 encoding (0/1/2) so any future serialised TT dumps
/// stay readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BoundFlag {
    Exact = 0,
    Lower = 1,
    Upper = 2,
}

impl Default for BoundFlag {
    fn default() -> Self { BoundFlag::Exact }
}

/// One transposition-table entry. Exactly 24 bytes; field order is tuned
/// so the compiler inserts no hidden padding (verified by
/// `entry_size_is_24_bytes`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Entry {
    pub key:        u64,        // 8
    pub score:      i32,        // 4
    /// Best-known move from this position. Drives move ordering across
    /// iterative-deepening passes - the single biggest pruning improvement
    /// after alpha-beta itself. `Action::default()` (= `Action(0)`) is the
    /// "no entry" sentinel, never a valid encoded action.
    pub best_move:  Action,     // 4 (u32)
    pub depth:      u8,         // 1
    pub flag:       BoundFlag,  // 1
    pub generation: u8,         // 1
    _pad:           u8,         // 1 - alignment to 8
}

/// Snapshot of stats counters - returned by `TranspositionTable::stats()`.
/// Holds the values at the moment of the call; subsequent operations on
/// the TT don't mutate the snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub probes:       u64,
    pub hits:         u64,
    pub collisions:   u64,
    pub stores:       u64,
    pub replacements: u64,
    pub rejections:   u64,
}

pub struct TranspositionTable {
    entries:    Vec<Entry>,
    mask:       u64,
    generation: u8,
    // Stats - single-threaded, no atomics needed.
    probes:       u64,
    hits:         u64,
    collisions:   u64,
    stores:       u64,
    replacements: u64,
    rejections:   u64,
}

impl TranspositionTable {
    /// Construct a TT with `2^log2_entries` slots.
    pub fn with_capacity_pow2(log2_entries: u32) -> Self {
        let n = 1usize << log2_entries;
        TranspositionTable {
            entries:      vec![Entry::default(); n],
            mask:         (n as u64) - 1,
            generation:   0,
            probes:       0,
            hits:         0,
            collisions:   0,
            stores:       0,
            replacements: 0,
            rejections:   0,
        }
    }

    /// Construct a TT sized to fit within `mb` MiB. Picks the largest
    /// power-of-two slot count whose total byte size is ≤ `mb * 1 MiB`.
    /// `mb == 0` still yields a 1-slot table - the TT is never empty.
    pub fn with_capacity_mb(mb: usize) -> Self {
        let entry_size = core::mem::size_of::<Entry>();
        let target_bytes = mb.saturating_mul(1024 * 1024);
        let max_entries = (target_bytes / entry_size).max(1);
        // Largest power of two ≤ max_entries.
        let log2 = if max_entries == 0 { 0 } else {
            (usize::BITS - 1 - max_entries.leading_zeros()) as u32
        };
        Self::with_capacity_pow2(log2)
    }

    /// Number of slots in the table.
    #[inline]
    pub fn len(&self) -> usize { self.entries.len() }

    /// Proportion of slots whose `key != 0` (i.e. have ever been stored
    /// into). O(n); intended for diagnostics, not the hot path.
    pub fn fill_rate(&self) -> f64 {
        let n = self.entries.len();
        if n == 0 { return 0.0; }
        let occupied = self.entries.iter().filter(|e| e.key != 0).count();
        (occupied as f64) / (n as f64)
    }

    /// Current generation counter.
    #[inline]
    pub fn generation(&self) -> u8 { self.generation }

    /// Snapshot the stats counters.
    #[inline]
    pub fn stats(&self) -> Stats {
        Stats {
            probes:       self.probes,
            hits:         self.hits,
            collisions:   self.collisions,
            stores:       self.stores,
            replacements: self.replacements,
            rejections:   self.rejections,
        }
    }

    /// Reset stats only (preserve entries and generation). Used by tests.
    fn reset_stats(&mut self) {
        self.probes       = 0;
        self.hits         = 0;
        self.collisions   = 0;
        self.stores       = 0;
        self.replacements = 0;
        self.rejections   = 0;
    }

    /// Bump the generation counter and reset stats. Old entries stay in
    /// place but become eligible for unconditional replacement on the next
    /// store collision. O(1).
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.reset_stats();
    }

    /// Hard clear: zero every slot, reset generation, reset stats. O(n).
    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = Entry::default();
        }
        self.generation = 0;
        self.reset_stats();
    }

    /// Probe for `key`. Returns `Some(&Entry)` iff a stored entry's key
    /// matches exactly. Increments `probes`; on hit also increments `hits`;
    /// on a non-empty mismatching slot also increments `collisions`.
    #[inline]
    pub fn probe(&mut self, key: u64) -> Option<&Entry> {
        self.probes += 1;
        let idx = (key & self.mask) as usize;
        let e = &self.entries[idx];
        if e.key == 0 {
            // Empty slot.
            None
        } else if e.key == key {
            self.hits += 1;
            Some(e)
        } else {
            self.collisions += 1;
            None
        }
    }

    /// Store `entry` according to the depth-preferred + generation policy
    /// (see module docs). The current `tt.generation` is copied into the
    /// stored entry, overriding whatever the caller passed in.
    #[inline]
    pub fn store(&mut self, mut entry: Entry) {
        self.stores += 1;
        entry.generation = self.generation;
        let idx = (entry.key & self.mask) as usize;
        let existing = &self.entries[idx];

        let accept =
            existing.key == 0
            || existing.generation != self.generation
            || entry.depth >= existing.depth
            || existing.key == entry.key;

        if accept {
            if existing.key != 0 && existing.key != entry.key {
                self.replacements += 1;
            }
            self.entries[idx] = entry;
        } else {
            self.rejections += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    /// Build a key K2 that hashes to the same index as K1 under `mask`,
    /// but differs in some bit above the mask. For a power-of-two table
    /// of size N, the mask covers the low log2(N) bits - flipping a bit
    /// above that leaves `key & mask` unchanged.
    fn collide_with(k1: u64, mask: u64) -> u64 {
        // Find the lowest bit position strictly above the mask.
        let bit_above = (mask + 1).max(1); // mask+1 is exactly the bit above mask
        k1 ^ bit_above
    }

    #[test]
    fn entry_default_is_empty() {
        let e = Entry::default();
        assert_eq!(e.key, 0);
        assert_eq!(e.score, 0);
        assert_eq!(e.best_move, Action::default());
        assert_eq!(e.depth, 0);
        assert_eq!(e.flag, BoundFlag::Exact);
        assert_eq!(e.generation, 0);
    }

    #[test]
    fn entry_size_is_24_bytes() {
        // Pinned at 24 bytes by deliberate field ordering. If this fires
        // after adding a field, either reorder to fit in the existing
        // pad slot or accept the size bump consciously.
        assert_eq!(size_of::<Entry>(), 24);
    }

    #[test]
    fn probe_empty_returns_none() {
        let mut tt = TranspositionTable::with_capacity_pow2(4); // 16 slots
        for k in 1u64..=10 {
            assert!(tt.probe(k).is_none());
        }
        let s = tt.stats();
        assert_eq!(s.probes, 10);
        assert_eq!(s.hits, 0);
        assert_eq!(s.collisions, 0);
    }

    #[test]
    fn store_then_probe_hits() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        let key = 0xDEAD_BEEF_CAFE_BABE;
        tt.store(Entry { key, score: 42, depth: 5, flag: BoundFlag::Exact, ..Default::default() });
        let e = tt.probe(key).expect("hit expected");
        assert_eq!(e.score, 42);
        assert_eq!(e.depth, 5);
        let s = tt.stats();
        assert_eq!(s.hits, 1);
        assert_eq!(s.collisions, 0);
        assert_eq!(s.stores, 1);
    }

    #[test]
    fn probe_wrong_key_is_collision() {
        let mut tt = TranspositionTable::with_capacity_pow2(4); // mask = 0xF
        let k1 = 0xAAAA_AAAA_AAAA_AAA1;
        let k2 = collide_with(k1, 0xF);
        // Sanity: same index, different key.
        assert_eq!(k1 & 0xF, k2 & 0xF);
        assert_ne!(k1, k2);
        tt.store(Entry { key: k1, depth: 3, ..Default::default() });
        assert!(tt.probe(k2).is_none());
        let s = tt.stats();
        assert_eq!(s.hits, 0);
        assert_eq!(s.collisions, 1);
    }

    #[test]
    fn depth_preferred_rejects_shallower() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        let k1 = 0x1111_1111_1111_1111;
        let k2 = collide_with(k1, 0xF);
        tt.store(Entry { key: k1, depth: 10, score: 100, ..Default::default() });
        tt.store(Entry { key: k2, depth: 5,  score: 200, ..Default::default() });
        // k1's slot should be untouched.
        let e = tt.probe(k1).expect("k1 still present");
        assert_eq!(e.depth, 10);
        assert_eq!(e.score, 100);
        let s = tt.stats();
        assert_eq!(s.rejections, 1);
        assert_eq!(s.replacements, 0);
    }

    #[test]
    fn depth_preferred_accepts_deeper() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        let k1 = 0x2222_2222_2222_2222;
        let k2 = collide_with(k1, 0xF);
        tt.store(Entry { key: k1, depth: 3, score: 100, ..Default::default() });
        tt.store(Entry { key: k2, depth: 8, score: 200, ..Default::default() });
        // k2 wins; k1 is now invisible.
        assert!(tt.probe(k1).is_none());
        let e = tt.probe(k2).expect("k2 present");
        assert_eq!(e.depth, 8);
        assert_eq!(e.score, 200);
        let s = tt.stats();
        assert_eq!(s.replacements, 1);
        assert_eq!(s.rejections, 0);
    }

    #[test]
    fn same_key_always_replaces() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        let key = 0x3333_3333_3333_3333;
        tt.store(Entry { key, depth: 10, score: 100, ..Default::default() });
        // Re-search of same position at shallower depth - should still overwrite.
        tt.store(Entry { key, depth: 2, score: 999, ..Default::default() });
        let e = tt.probe(key).expect("hit");
        assert_eq!(e.depth, 2);
        assert_eq!(e.score, 999);
        let s = tt.stats();
        // Same-key overwrite isn't counted as a replacement (existing.key == entry.key).
        assert_eq!(s.replacements, 0);
        assert_eq!(s.rejections, 0);
    }

    #[test]
    fn new_search_bumps_generation_and_resets_stats() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        tt.store(Entry { key: 1, depth: 1, ..Default::default() });
        let _ = tt.probe(1);
        let _ = tt.probe(2); // miss
        assert!(tt.stats().probes > 0);
        let g0 = tt.generation();
        tt.new_search();
        assert_eq!(tt.generation(), g0.wrapping_add(1));
        let s = tt.stats();
        assert_eq!(s.probes, 0);
        assert_eq!(s.hits, 0);
        assert_eq!(s.collisions, 0);
        assert_eq!(s.stores, 0);
        assert_eq!(s.replacements, 0);
        assert_eq!(s.rejections, 0);
    }

    #[test]
    fn cross_generation_replacement() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        let k1 = 0x4444_4444_4444_4444;
        let k2 = collide_with(k1, 0xF);
        // Store deep entry in generation 0.
        tt.store(Entry { key: k1, depth: 20, score: 50, ..Default::default() });
        tt.new_search(); // → generation 1
        // Shallower different-key entry would normally be rejected, but the
        // existing entry is from generation 0; it gets replaced regardless.
        tt.store(Entry { key: k2, depth: 1, score: 7, ..Default::default() });
        assert!(tt.probe(k1).is_none());
        let e = tt.probe(k2).expect("k2 won via generation");
        assert_eq!(e.depth, 1);
        assert_eq!(e.generation, 1);
    }

    #[test]
    fn with_capacity_mb_sizing() {
        let tt = TranspositionTable::with_capacity_mb(1);
        // Each entry is 24 bytes. 1 MiB = 1_048_576 bytes / 24 = 43690 entries
        // max; largest power of two ≤ 43690 is 32768 = 2^15.
        assert_eq!(tt.len(), 32_768);
        assert!(tt.len() * size_of::<Entry>() <= 1024 * 1024);
        assert!(tt.len().is_power_of_two());

        // mb=0 still yields a non-empty table.
        let tiny = TranspositionTable::with_capacity_mb(0);
        assert_eq!(tiny.len(), 1);
        assert!(tiny.len().is_power_of_two());
    }

    #[test]
    fn clear_zeroes_everything() {
        let mut tt = TranspositionTable::with_capacity_pow2(4);
        for k in 1u64..=5 { tt.store(Entry { key: k, depth: 1, ..Default::default() }); }
        tt.new_search();
        tt.new_search();
        assert!(tt.fill_rate() > 0.0);
        assert_ne!(tt.generation(), 0);

        tt.clear();
        assert_eq!(tt.fill_rate(), 0.0);
        assert_eq!(tt.generation(), 0);
        let s = tt.stats();
        assert_eq!(s.probes, 0);
        assert_eq!(s.hits, 0);
        assert_eq!(s.collisions, 0);
        assert_eq!(s.stores, 0);
        assert_eq!(s.replacements, 0);
        assert_eq!(s.rejections, 0);
    }

    #[test]
    fn fill_rate_tracks_occupied_slots() {
        // Use distinct keys whose low bits cover distinct slot indices,
        // so no collisions and fill_rate == K/N exactly.
        let mut tt = TranspositionTable::with_capacity_pow2(4); // N = 16
        let n = tt.len();
        // Store 7 entries with distinct indices.
        for i in 1u64..=7 {
            tt.store(Entry { key: i, depth: 1, ..Default::default() });
        }
        let expected = 7.0 / (n as f64);
        let actual = tt.fill_rate();
        assert!((actual - expected).abs() < 1e-9,
            "fill_rate {} vs expected {}", actual, expected);
    }
}
