//! Per-section counters for evaluator/search diagnostic instrumentation.
//!
//! Compiles to no-ops unless `--features bench_counters` is set (the
//! `search_bench` crate enables this transitively). Ships as no-ops in
//! Tauri/nn_trainer builds — zero cost on the shipped hot path.
//!
//! Thread-local storage: each thread accumulates its own counts. The bench
//! is single-threaded so `snapshot()` returns the current thread's totals.
//! Multi-threaded consumers would need to aggregate across threads.
//!
//! Convention: `bump_*` for scalar increments, `record_attacker_list_len`
//! for the histogram. Call sites are `#[inline]` so with the feature off
//! they collapse to nothing.

/// AttackerList capacity is 8 (see evaluator.rs). Histogram covers 0..=8
/// so we can see when the list is empty (all 0-cost enemies filtered upstream)
/// versus saturated (rare — the geometric ceiling).
pub const ATTACKER_LIST_HIST_BUCKETS: usize = 9;

#[derive(Clone, Copy, Default, Debug)]
pub struct Snapshot {
    // Evaluator entry counts.
    pub eval_calls: u64,

    // Phase gates — how often did each gate fire?
    pub maee_gate_pass: u64,       // Phase::Move at eval entry
    pub maee_gate_skip: u64,       // Phase::Skill or Phase::Draft at eval entry
    pub skill_gate_pass: u64,      // Phase::Skill at eval entry
    pub skill_gate_skip: u64,      // Phase::Move or Phase::Draft at eval entry
    pub actions_zero_hit: u64,     // actions_remaining == 0 short-circuit fired

    // MAEE internals.
    pub maee_side_calls: u64,      // maee_side() invocations (2 per eval when gate passes)
    pub maee_target_calls: u64,    // maee() invocations (one per candidate target square)
    pub enumerate_attackers_calls: u64,

    // AttackerList size histogram — bucket i is #enumerations that produced i attackers.
    pub attacker_list_hist: [u64; ATTACKER_LIST_HIST_BUCKETS],

    // Skill-activity call count (whole function; per-piece iteration is not counted).
    pub skill_activity_calls: u64,

    // Search-side counters.
    pub ab_nodes: u64,     // alpha_beta node visits
    pub qs_nodes: u64,     // quiescence node visits
}

impl Snapshot {
    /// Total attackers observed (weighted by bucket index) — useful for a mean.
    pub fn attackers_total(&self) -> u64 {
        self.attacker_list_hist
            .iter()
            .enumerate()
            .map(|(i, c)| i as u64 * *c)
            .sum()
    }

    /// Total enumerate_attackers observations (should equal enumerate_attackers_calls).
    pub fn attackers_observations(&self) -> u64 {
        self.attacker_list_hist.iter().sum()
    }
}

// ── Feature-gated impl ───────────────────────────────────────────────────

#[cfg(feature = "bench_counters")]
mod imp {
    use super::{Snapshot, ATTACKER_LIST_HIST_BUCKETS};
    use std::cell::Cell;

    thread_local! {
        static SNAP: Cell<Snapshot> = const { Cell::new(Snapshot {
            eval_calls: 0,
            maee_gate_pass: 0,
            maee_gate_skip: 0,
            skill_gate_pass: 0,
            skill_gate_skip: 0,
            actions_zero_hit: 0,
            maee_side_calls: 0,
            maee_target_calls: 0,
            enumerate_attackers_calls: 0,
            attacker_list_hist: [0; ATTACKER_LIST_HIST_BUCKETS],
            skill_activity_calls: 0,
            ab_nodes: 0,
            qs_nodes: 0,
        }) };
    }

    #[inline]
    fn with_mut<F: FnOnce(&mut Snapshot)>(f: F) {
        SNAP.with(|c| {
            let mut s = c.get();
            f(&mut s);
            c.set(s);
        });
    }

    #[inline] pub fn bump_eval_calls()                { with_mut(|s| s.eval_calls += 1); }
    #[inline] pub fn bump_maee_gate_pass()            { with_mut(|s| s.maee_gate_pass += 1); }
    #[inline] pub fn bump_maee_gate_skip()            { with_mut(|s| s.maee_gate_skip += 1); }
    #[inline] pub fn bump_skill_gate_pass()           { with_mut(|s| s.skill_gate_pass += 1); }
    #[inline] pub fn bump_skill_gate_skip()           { with_mut(|s| s.skill_gate_skip += 1); }
    #[inline] pub fn bump_actions_zero_hit()          { with_mut(|s| s.actions_zero_hit += 1); }
    #[inline] pub fn bump_maee_side_calls()           { with_mut(|s| s.maee_side_calls += 1); }
    #[inline] pub fn bump_maee_target_calls()         { with_mut(|s| s.maee_target_calls += 1); }
    #[inline] pub fn bump_enumerate_attackers_calls() { with_mut(|s| s.enumerate_attackers_calls += 1); }
    #[inline] pub fn bump_skill_activity_calls()      { with_mut(|s| s.skill_activity_calls += 1); }
    #[inline] pub fn bump_ab_nodes()                  { with_mut(|s| s.ab_nodes += 1); }
    #[inline] pub fn bump_qs_nodes()                  { with_mut(|s| s.qs_nodes += 1); }

    #[inline]
    pub fn record_attacker_list_len(len: usize) {
        let idx = if len >= ATTACKER_LIST_HIST_BUCKETS { ATTACKER_LIST_HIST_BUCKETS - 1 } else { len };
        with_mut(|s| s.attacker_list_hist[idx] += 1);
    }

    pub fn snapshot() -> Snapshot { SNAP.with(|c| c.get()) }

    pub fn reset() {
        SNAP.with(|c| c.set(Snapshot::default()));
    }
}

#[cfg(not(feature = "bench_counters"))]
mod imp {
    use super::Snapshot;

    #[inline] pub fn bump_eval_calls() {}
    #[inline] pub fn bump_maee_gate_pass() {}
    #[inline] pub fn bump_maee_gate_skip() {}
    #[inline] pub fn bump_skill_gate_pass() {}
    #[inline] pub fn bump_skill_gate_skip() {}
    #[inline] pub fn bump_actions_zero_hit() {}
    #[inline] pub fn bump_maee_side_calls() {}
    #[inline] pub fn bump_maee_target_calls() {}
    #[inline] pub fn bump_enumerate_attackers_calls() {}
    #[inline] pub fn bump_skill_activity_calls() {}
    #[inline] pub fn bump_ab_nodes() {}
    #[inline] pub fn bump_qs_nodes() {}
    #[inline] pub fn record_attacker_list_len(_len: usize) {}
    #[inline] pub fn snapshot() -> Snapshot { Snapshot::default() }
    #[inline] pub fn reset() {}
}

pub use imp::*;
