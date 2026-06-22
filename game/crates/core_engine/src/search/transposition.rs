//! Transposition table — preallocated `Vec<Entry>` keyed by Zobrist hash
//! modulo table size. No dynamic allocation in the hot path.

use crate::game_logic::action::Action;

#[derive(Clone, Copy, Debug, Default)]
pub struct Entry {
    pub key:       u64,
    pub depth:     u8,
    pub score:     i32,
    /// 0 = exact, 1 = lower bound, 2 = upper bound.
    pub flag:      u8,
    /// Best-known move from this position. Drives move ordering across
    /// iterative-deepening passes — the single biggest pruning improvement
    /// after alpha-beta itself. `Action(0)` is the "no entry" sentinel.
    pub best_move: Action,
}

pub struct TranspositionTable {
    entries: Vec<Entry>,
    mask:    u64,
}

impl TranspositionTable {
    pub fn with_capacity_pow2(log2_entries: u32) -> Self {
        let n = 1usize << log2_entries;
        TranspositionTable {
            entries: vec![Entry::default(); n],
            mask:    (n as u64) - 1,
        }
    }

    #[inline]
    pub fn probe(&self, key: u64) -> Option<&Entry> {
        let e = &self.entries[(key & self.mask) as usize];
        if e.key == key { Some(e) } else { None }
    }

    #[inline]
    pub fn store(&mut self, entry: Entry) {
        let idx = (entry.key & self.mask) as usize;
        self.entries[idx] = entry;
    }
}
