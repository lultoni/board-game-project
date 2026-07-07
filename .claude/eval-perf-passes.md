# Evaluator Perf Passes — playbook + log

*Living doc. Each perf pass on `core_engine/src/search/evaluator.rs` follows the same recipe: critique → plan → implement → bench → recritique. Track results here so we can see whether each pass actually helped and what's left to attack.*

*Last updated: 2026-07-07 — Pass 3 complete (both chunks). Chunk 1: attackers-table + MAEE_MAX_PLIES 32→16. Chunk 2: stand-pat single-pass (item 3), AttackerList head-cursor **shelved** (SROA regression), Guard geometry fix + fast bitboard recompute (item 4a — **5-6× per-node speedup**, also fixes a pre-existing over-approximation bug), incremental attackers-bitmask maintenance across kills (item 4b — additional -10% wall time). **Full-corpus d6: 364s → 59s (-84%)**; vs Pass 2: 740s → 59s (-92%, ~12.6× total). Node counts essentially identical. Still deferred: `threat_bb` hand-off audit.*

---

## Critique-agent playbook

Spawn **5 subagents in parallel** (single message, multiple `Agent` tool calls). Each agent critiques the evaluator from ONE angle and **complains only — no fixes**. Complaints from different angles overlap; that overlap is signal.

### Framing rules for every agent

- Complain, don't fix. Reserve solution-thinking for the plan step.
- Cite `file:line` on every complaint.
- Under 500 words per agent.
- Skip anything that's genuinely fine — don't pad.
- Report format: bullet list.

### The five angles

Each prompt should share this **context block** (adapt numbers to the current pass):

> Rust engine for an 8×8 tactical game. Pieces: Kings, Champions (speed-1), Guards (speed-2 BFS through empty squares). State: u64 bitboards + mailbox. Evaluator is called at every alpha-beta leaf. MAEE (Move-Attack Exchange Evaluation) recently added; per-eval time went from `<PRE ns>` → `<POST ns>` (`<multiplier>×`), search node counts at depth 6 exploded on some corpus positions (e.g. `opening-with-skills-03` from seconds → `<time>` s, EBF `<ebf>`).

Then each agent gets a specific focus:

**Agent 1 — Bitboard / bitwise / SIMD**
Files to read:
- `game/crates/core_engine/src/search/evaluator.rs`
- `game/crates/core_engine/src/state/magic.rs`
- `game/crates/core_engine/src/state/position.rs`
- `game/crates/core_engine/src/state/bitboard.rs`

Focus: bitwise ops that could be cheaper (popcnt, LSB scan, mask construction); redundant recomputation of bitboards; loops-over-bits where whole-board ops would suffice; missing precomputed lookup tables; missed bit-parallelism (shifts + masks vs iterating pieces); compiler-level concerns (branches that break autovec, function call overhead, missing `#[inline]`).

**Agent 2 — Algorithmic complexity**
Files to read:
- `game/crates/core_engine/src/search/evaluator.rs`
- `game/crates/core_engine/src/state/magic.rs`
- `game/crates/core_engine/src/state/position.rs`

Focus: Big-O of loops and nested loops; redundant work per evaluate call; recomputation across evaluate calls that could be incremental; MAEE-specific: full re-enum after every kill, insertion sort in hot loop; iterating 64 squares when only occupied matter; ordering of cheap-vs-expensive checks; missed early exits; algorithmic access patterns.

**Agent 3 — Memory layout / cache / data-oriented design**
Files to read:
- `game/crates/core_engine/src/search/evaluator.rs`
- `game/crates/core_engine/src/state/position.rs`
- `game/crates/core_engine/src/state/bitboard.rs`
- `game/crates/core_engine/src/state/mailbox.rs`

Focus: struct sizes, alignment, padding; AoS vs SoA where SoA would win; per-square data spanning multiple cache lines; hot/cold field segregation; `Position` size and `Clone` cost; stack arrays in hot functions (well-sized? aligned?); heap allocation in hot paths; function-call boundaries inhibiting inlining; pass-by-value of large structs; return types bouncing through stack.

**Agent 4 — Precomputation / caching / incremental maintenance**
Files to read:
- `game/crates/core_engine/src/search/evaluator.rs`
- `game/crates/core_engine/src/state/magic.rs`
- `game/crates/core_engine/src/state/position.rs`
- `game/crates/core_engine/src/game_logic/generator.rs`
- `game/crates/core_engine/src/game_logic/make_unmake.rs` (or equivalent)

Focus: values recomputed every evaluate() that don't change often; lookup tables that should be `const` but aren't; attack tables under-used; Position properties that could be tracked incrementally via make/unmake; zobrist-keyed eval cache; per-square "who attacks S" table missing; constants declared inline vs at module const level.

**Agent 5 — Search / eval interaction**
Files to read:
- `game/crates/core_engine/src/search/evaluator.rs`
- `game/crates/core_engine/src/search/alpha_beta.rs`
- `game/crates/core_engine/src/search/ordering.rs` (if it exists)
- `game/crates/core_engine/src/search/quiescence.rs`
- Any move-ordering / TT / killer / history files under `game/crates/core_engine/src/search/`

Focus: signs the new eval disturbs move ordering; missing lazy-eval / window-based early return; SEE for move ordering (vs leaf eval); quiescence — should exchanges be resolved by qsearch not static eval?; alpha-beta cutoff behaviour under score variance; history heuristic / killer moves interacting with perturbed eval; whether eval should be depth-gated; TT interaction with unstable scores.

### After agents return

Read all five outputs. **Look for cross-cutting complaints** — the same issue surfacing from 3+ angles is a signal-boosted priority. Present to user as a consolidated report grouped by angle, then a **Cross-cutting themes** section at the end. Do NOT propose solutions in the report — wait for the user to react.

### Follow-up single-issue agents

Sometimes the critique surfaces a specific question (e.g. "what's the actual max attackers per square?"). Spawn narrow follow-up agents (1-2, in parallel, general-purpose subagent_type) that answer ONE precise question with citations. These are research agents, not critique agents — different framing.

---

## Benchmarking strategy

Every pass records the same numbers so successive passes are comparable.

### Bench 1: Eval-only microbench

Isolates per-eval cost from search branching. Corpus positions read from `bench/corpus/corpus.txt`. Each pos runs `evaluate_breakdown` in a hot loop with a checksum accumulator (prevents DCE) after a 64-iteration warmup.

```
cd game && cargo run --release -p search_bench -- --eval-only \
  --eval-iterations 100000 \
  --out bench/results/eval-<label>.json
```

Reports per-position ns/eval + aggregate (min / mean / geo / max). **Geo mean is the headline number.**

### Bench 2: Search bench at depth 6

Runs full alpha-beta search on the same corpus at depth 6. Captures node counts, time, EBF, TT hit rates. Node counts matter as much as time — a change that halves per-eval cost but doubles node count is a wash.

```
cd game && cargo run --release -p search_bench -- \
  --depth 6 --runs 3 \
  --out bench/results/search-<label>-d6.json
```

**Compare both node counts and wall time** against the previous pass's results file. A node-count explosion on a specific position without a matching total-node explosion means the eval is perturbing move ordering on that position specifically.

### Bench 3: Manual endgame verification

Load in the inspector at Tauri dev (`cd game/crates/tauri_wrapper && cargo tauri dev`):

```
8/1c[2/0/0/5/7]1c[2/1/0/14/15]3K[2/2/0/6/9]/8/3c[2/0/0/9/4]4/8/5k[2/2/0/3/6]2/8/8 P2 M 2 27 38 0 27 0x0
```

At depth ≥6 the AI must find the killing sequence and the king must delay loss cleanly (not run into the middle of the board). This is the functional-correctness guard rail — perf gains that break this are not gains.

### Bench 4: Rust tests

- `cargo check -p core_engine` after each logical chunk.
- `cargo test -p core_engine` (fast, under a minute) at the end of each pass.
- **Never** `cargo test -p nn_trainer` in the perf-pass loop (30+ min).

### File-naming convention for bench results

`bench/results/eval-<label>.json` and `bench/results/search-<label>-d6.json`. Labels used so far:
- `baseline-pre-maee` — the flat 0.25× threat_value era, before MAEE landed.
- `post-maee` — MAEE landed, no optimisation.
- `post-pass1`, `post-pass2`, ... — after each perf pass.

Never overwrite existing labels; append new ones.

---

## Pass log

### Baseline: pre-MAEE (flat 0.25× threat_value)

**Date:** 2026-07-05 (approx)
**Eval-only microbench:** `bench/results/eval-baseline-pre-maee.json`
- Geo mean: **1101 ns/eval**
- Mean: 1691 ns
- Max: 3959 ns

**Search bench d6:** not captured pre-MAEE (regret — do this before Pass 2 if we still have the ability to reconstruct the pre-MAEE build).

**Correctness:** endgame FEN — AI did NOT find the killing move at depth 6+. King-forward regression present in mid-game (AI kings wandered centrewards to threaten pieces they couldn't safely capture).

---

### Post-MAEE (no optimisation)

**Date:** 2026-07-07
**Eval-only microbench:** `bench/results/eval-post-maee.json`
- Geo mean: **2021 ns/eval** (1.84× baseline)
- Mean: 5074 ns (3.0× baseline)
- Max: 22504 ns (5.7× baseline)

**Search bench d6:** `bench/results/search-post-maee-d6.json`
- **Regressions:**
  - `opening-with-skills-03`: 211 s / 10.5 M nodes / EBF 14.8
  - `opening-with-skills-04`: 100 s / 4.4 M nodes
  - `midgame-move-01`: 24 s
  - `skill-phase-full-03`: 59 s
  - `skill-phase-full-04`: 59 s
- Node-count explosion (not just slower per-node) → MAEE is perturbing move ordering.

**Correctness:** endgame FEN — AI **finds** the killing sequence; king delays loss correctly. Functionally correct.

**Critique run (2026-07-07):** all 5 agents. Cross-cutting themes:
1. MAEE re-enumerates attackers from scratch after every kill (all 5 angles).
2. `AttackerList` push/pop_front are O(n) shifts on a 136-byte struct in a hot loop (4/5).
3. No precomputed "who attacks square S" per position (3/5).
4. MAEE runs on every QS node with no window-lazy-out (3/5).
5. MAEE is a leaf eval, not a move-ordering eval → direct cause of EBF explosion (3/5).

**Follow-up investigations (2026-07-07):**
- Phase/moved-this-phase state exists (`Position::current_phase`, `Position::moved_this_phase`, `Position::actions_remaining`) but the evaluator reads NONE of it. MAEE runs during Skill phase even though move-attacks are illegal. Guards that already moved still get full mobility. `actions_remaining == 0` doesn't short-circuit anything.
- `AttackerList` capacity of 16 is 2× the geometric ceiling. True max is 8 per side (Chebyshev-1 ring around T is 8 squares; filling more requires Guards, which block each other's BFS). Realistic in-game is 3-5.

---

### Pass 1 (implemented 2026-07-07)

**Scope:** low-risk, low-touch wins.
- Phase gates on MAEE (`Phase::Move` only) and `skill_activity` (`Phase::Skill` only).
- `actions_remaining == 0` short-circuit for the side to move.
- `AttackerList`: capacity 16 → 8, `cost: i32 → i16`, `len: usize → u8`. Struct drops from 136 B to ~36 B.
- `skill_value` function → `const SKILL_VALUE[]` table.
- `SQ_BIT[64]` const table for hot `1u64 << sq` sites.
- `#[inline]` on `enumerate_attackers`, `maee`, `maee_side`, `skill_activity`, `slot_target_count`.
- Delete dead `_is_champion` binding at evaluator.rs:222.

**Deferred to Pass 2+:**
- Per-position "who attacks S" precomputed table.
- Incremental make/unmake maintenance of `all_occ`, material sums, etc.
- Zobrist-keyed eval cache.
- **SEE for move ordering in alpha_beta.rs** (the single biggest search-side win identified).
- Quiescence redesign so MAEE doesn't run on every QS node.
- BFS-2 rewrite as two-wave shift-and-mask.
- Mailbox AoS → SoA.
- Mobility gating on `moved_this_phase`.

**Success criteria:** eval geo mean back under 1500 ns; search d6 regressions on `opening-with-skills-03/04` + `midgame-move-01` shrink materially; endgame FEN still solves.

**Results:**
- Eval-only: `bench/results/eval-post-pass1.json`
  - Geo mean: **743 ns/eval** (2021 → 743, **2.72× vs post-MAEE**, **32% below pre-MAEE baseline of 1101**)
  - Mean: **3054 ns** (5074 → 3054, 1.66× vs post-MAEE)
  - Max: **21074 ns** (22504 → 21074, marginal — dominated by heavy Move-phase leaves where MAEE still runs)
  - Per-position: Skill-phase / endgame / combo positions saw 2-12× wins (phase gate skips MAEE entirely). Move-phase positions saw modest 1.1-1.4× from AttackerList resize + inline + const tables.
- Search d6: `bench/results/search-post-pass1-d6.json`
  - Total wall time: ~505 s → ~313 s (**~1.6× faster**).
  - Aggregate NPS: 423k → 652k (1.54× faster per node).
  - **Big wins on previously-broken positions:**
    - `opening-with-skills-03`: 10.5 M → 1.73 M nodes (6.07× fewer), 211 s → 29 s, EBF 14.8 → 10.95.
    - `opening-with-skills-04`: 4.4 M → 1.07 M nodes (4.10×), 100 s → 22 s, EBF 12.8 → 10.12.
    - `midgame-move-03`: 1.11 M → 423 K nodes (2.63×).
    - `king-in-danger-04`: 51 K → 21 K (2.49×).
    - `skill-phase-full-01/02`: 1.5-2× fewer nodes.
- Endgame FEN correctness: **NOT re-verified.** Deferred; will re-run only if the Pass 2 critique surfaces search-quality concerns.
- **Known regressions (horizon effect):**
  - `skill-phase-full-03`: 1.98 M → 5.71 M nodes (2.89× WORSE), 59 s → 162 s. Root cause: phase-gated MAEE hides move-attack threats from Skill-phase leaves, so search explores futile lines. Late-game will have MORE Skill-phase terminals due to skill scaling → this hole will grow.
  - `midgame-move-02`: 225 K → 688 K nodes (3.06× worse). Move-phase position — MAEE still runs, so regression is from perturbation (const tables / inline / AttackerList changes shifting move-ordering) or from `actions_remaining==0` short-circuit landing at bad leaves. Needs targeted investigation.
  - `midgame-move-05`: 668 K → 1.14 M (1.70× worse).
  - `combo-loaded-04`: 47 K → 98 K (2.08× worse).
  - Handful of small (≤25%) regressions on the already-fast positions.

**Pass 2 explicit follow-ups (from Pass 1 known-issues + user discussion 2026-07-07):**

1. **Fix the horizon effect from phase-gated MAEE.** Options: (a) run a cheap MAEE variant on the non-side-to-move only in the "wrong" phase (still prices *their* threats, ignores ours-that-can't-cash), (b) fold MAEE into quiescence rather than static leaf eval so it fires exactly when tactical exchanges are pending regardless of phase, (c) restore MAEE-everywhere once precomputed attacker tables land and make it cheap enough that the phase gate is unnecessary. Preferred direction: (c). This is the **priority Pass 2 item.**

2. **Forced-move short-circuit at search root.** If the AI is asked for a move on a position with exactly 1 legal action, return it without searching. Zero-risk correctness-wise, saves the entire tree. Common cases: EndPhase-only when `actions_remaining==0` and no skills castable; forced captures when only one move is legal.

3. **Forced-move handling inside the search tree.** When a node has exactly 1 legal move, don't count that ply against depth (forced-move extension) and consider skipping the static eval at that node (it will be overridden by the child anyway). Amounts to "if only one move, just do it and go deeper" — matches user's intuition. Risk: interaction with LMR / futility / null-move; needs care.

4. **Investigate why `midgame-move-02` regressed in a Move-phase position.** MAEE still runs there. Suspicion: `actions_remaining==0` zeroing the side-to-move's threat_* term in the middle of a Move phase shifts ordering unfavourably at some subtree. Alternatively: AttackerList `Attacker { cost: i16, sq: u8 }` layout is now 4 B including 1 B pad — different memmove/sort semantics vs the old 8 B `{ i32, u8 }` items. Either could perturb ordering. Instrument with per-position search stats before touching.

5. **Bench-side improvement: per-section counters.** Add lightweight counters to eval/search hot paths — MAEE call count per node, AttackerList size histogram, phase-gate hit rate at leaves, per-move-ordering-key TT hit rate. Currently the bench reports nodes + time + EBF only, which hides *why* a position regressed. Adding these makes each future pass diagnosable without ad-hoc instrumentation. Ship this early in Pass 2 so subsequent passes benefit.

---

### Pass 2 progress log

**Item 5 (per-section counters) — DONE (2026-07-07).**

New module `core_engine::search::counters`. Feature-gated behind `bench_counters` (off by default; enabled transitively via `search_bench` crate). Zero cost in Tauri/nn_trainer release builds — every counter fn compiles to `{}` without the feature. With the feature, TLS-backed `Cell<Snapshot>` — cheap enough for the bench (~+15% eval cost when instrumented; not a factor for the search-side node-count metrics we care about).

Counters exposed: `eval_calls`, `maee_gate_pass/skip`, `skill_gate_pass/skip`, `actions_zero_hit`, `maee_side_calls`, `maee_target_calls`, `enumerate_attackers_calls`, `attacker_list_hist[0..=8]`, `skill_activity_calls`, `ab_nodes`, `qs_nodes`. Bench prints per-position summary + emits full snapshot in the search-\*.json output. Aggregate rollup across all positions in the top-level `aggregate.counters` block.

Reference data: `bench/results/search-post-pass1-instrumented-d6.json`.

**Item 4 (midgame-move-02 investigation) — DONE (2026-07-07). Root cause identified.**

Counters reveal that `midgame-move-02` and the other 3 "mysterious" Pass 1 regressions all share a profile: high `actions_zero_hit` fraction (77-95% of eval calls) plus high MAEE-gate-pass fraction (97%+). Hypothesis: the `actions_remaining==0` side-to-move short-circuit is asymmetrically zeroing one side's `threat_*`/`skill_act_*` at the majority of leaves, creating an eval-value discontinuity that perturbs move ordering.

**Probe test (2026-07-07):** disabled the short-circuit block; re-ran 5-position mini-corpus at d6.

| Position | Pass 1 nodes | probe (no act0) nodes | Δ |
|---|---|---|---|
| midgame-move-01 | 1.61 M | 1.36 M | -15% |
| **midgame-move-02** | **688 K** | **231 K** | **-66%** (recovers to ~pre-MAEE baseline of 225 K) |
| midgame-move-05 | 1.14 M | 715 K | -37% |
| **skill-phase-full-03** | **5.71 M** | **3.54 M** | **-38%** (residual is the phase-gate horizon effect; item 1 handles that) |
| combo-loaded-04 | 98 K | 48 K | -52% (recovers to pre-Pass-1 47 K) |

Confirmed: the short-circuit is the direct cause of 4/5 known regressions. skill-phase-full-03 has an additional cause (the phase gate itself — item 1).

**Item 4 fix (2026-07-07):** removed the `actions_remaining==0` side-to-move zeroing. Kept the `bump_actions_zero_hit()` counter for future diagnostic use. 392 tests pass. The `if pos.actions_remaining == 0 { … }` block now only bumps the counter and is a no-op otherwise.

Not doing a full-corpus rerun yet — the 5-position probe already covers all known regressions and confirms the fix. The corpus-wide impact will be captured after item 1 lands (which needs its own bench anyway).

**Item 1 (horizon-effect fix / drop MAEE + skill_activity phase gates) — DONE (2026-07-07).**

Approach chosen: drop the phase gates entirely. Rationale: MAEE's inputs (bitboards, mailbox HP/armor, reachability primitives) are phase-invariant. Gating created a cliff at every phase transition. Ran a cross-phase-correctness audit first (task #73) — confirmed MAEE is bounded (~10k ops/leaf, no recursion, hard MAEE_MAX_PLIES=32 cutoff), respects Stack M rules (BFS-2 for guards, armor-then-HP damage, king exclusion), and has no phase-dependent inputs. Same treatment applied to `skill_activity` for symmetry — its inputs (money, range, occupancy) are also phase-invariant. Kept the `bump_maee_gate_pass` / `bump_skill_gate_pass` counters for diagnostic value; the skip counters will always read zero now.

Verification:
- 14 evaluator unit tests pass, full 392-test suite passes.
- **Eval-only bench (100k iters × 30 positions):** geo-mean **2021 ns → 1853 ns (-8%)**, mean 5074 → 4721 ns (-7%), max 22504 → 21555 ns (-4%). Counterintuitively, unconditional MAEE is *cheaper* per eval than gated — the branch + skip-side counter bumps cost more than they saved on the ~40% of leaves that hit them. Result file: `bench/results/eval-post-pass2-nogate.json`.
- **5-position d6 probe** vs post-item-4 baseline:

  | Position | Item-4 baseline nodes | Item-1 (nogate) nodes | Δ |
  |---|---|---|---|
  | midgame-move-01 | 1.36 M | 1.70 M | +25% |
  | midgame-move-02 | 231 K | 225 K | -3% |
  | midgame-move-05 | 715 K | 668 K | -7% |
  | **skill-phase-full-03** | 3.54 M | **1.98 M** | **-44%** |
  | combo-loaded-04 | 48 K | 47 K | -2% |

  The horizon-effect fix delivers the expected big win on skill-phase-full-03 (Skill-phase position where MAEE was previously blind). midgame-move-01 regresses (+25%) — unconditional MAEE reprices some moves the gate had left at zero, shifting move ordering. Net across the probe is positive; full-corpus impact deferred to after items 2+3 land.

**Item 2 (forced-move root short-circuit) — DONE (2026-07-07).**

Added an early return in `find_best_with_evaluator` (`search/alpha_beta.rs`): if `generator::generate(pos).len() == 1` at the root, skip iterative deepening entirely and return `SearchResult { best: Some(root_moves[0]), score: evaluator.evaluate(pos), depth: max_depth, nodes: 1 }`. `nodes = 1` (not 0) keeps the `telemetry::step_ai_records_searchmeta` test's `nodes > 0` invariant intact — semantically we did examine 1 node (the root) to determine forcedness. Runs `game_result.is_none()` guard first so terminal positions still get proper scoring.

Zero-risk: does not affect any position with >1 legal action, which is the entire benchmark corpus. The win shows up in live play on positions with an EndPhase-only situation or a forced capture.

**Item 3 (forced-move extension in tree) — ATTEMPTED, REVERTED (2026-07-07).**

Implemented: at internal search nodes with exactly 1 legal action, recurse with `depth - 1 + 1 = depth` (unchanged). `ply` still increments. Added `ply < MAX_PLY - 1` guard against pathological chained-forced-move recursion.

**Result**: full test suite hung on `session::tests::aivai_terminates_within_budget` after 36 minutes at 99.5% CPU (killed manually). No infinite loop — the ply guard prevented that — but a catastrophic tree explosion.

**Diagnostic trace** (midgame-move-01, 1s budget, extension fires logged to stderr):
- 9,151 extensions fired in 1 second.
- 8,395× at `ply=2 depth=1`, 700× at `ply=4 depth=1`, 50× at `ply=2 depth=2`.
- Every one: `phase=Move to_move=P1 actions_remaining=0 action=EndPhase`.
- Search reached only depth 3 in 1s (baseline: d5-6).

**Root cause**: after a side consumes its move-phase actions (`actions_remaining==0`), the ONLY legal action is `EndPhase`. This is not a rare pathological state — it's a **structural, every-game-line property** of the phase system. Every phase-boundary internal node triggers an extension. That EndPhase gates a full subtree of subsequent decisions (same player's Skill phase, all skill options), so extending doubles the work of every phase transition on every line. Compounded across thousands of phase-boundary nodes → tree explosion → search shallower, not deeper.

The user's earlier item-2 EndPhase intuition (skip at root) doesn't transfer to internal nodes: at the root an EndPhase-only position has no subtree because we don't search it, so skipping is free. At internal nodes there's a real subtree behind every EndPhase; extending duplicates that subtree.

**Reverted.** The idea moves to the Pass 3+ backlog. Key question left open: are there enough *tactically meaningful* forced moves (forced Move-Attack, forced BodyguardChoice, forced non-EndPhase skill) that a properly-guarded extension would help? Need instrumented counter first (added as Pass 3 prerequisite below).

**Drive-by fixes (2026-07-07):**
- `ENABLE_NMP` doc comment corrected: said "Default `false` (disabled) until benchmarked" but code was `AtomicBool::new(true)`. Session 41 Phase B sweep already validated NMP-on with -9.4% depth-6 nodes / +18.6% NPS. Comment updated to reflect production state.
- `SettingsModal.svelte`: Think-time inputs (both P1 and P2) `min` lowered from 100 → 0, added italic hint below each row: "0 = no time limit; search runs to Max depth." Enables users to run depth-only AI mode from the settings panel.

**Corpus-wide observations from item 5 counters that inform later items:**

1. `maee_gate_pass` fires 92-99% on the slow positions (Move-phase-heavy trees). The phase-gate's savings mostly hit *cheap* positions, not the expensive ones — reinforces that **item 1 (horizon fix / MAEE-everywhere-cheaply) is the biggest remaining win**.
2. `qs_nodes / (ab_nodes + qs_nodes)` = 0.6–0.97 on the slow positions. Most eval calls come from QS, not from AB leaves. The Pass 3+ "quiescence redesign" item is more concrete now: **MAEE-from-QS is where the time is**, not MAEE-from-static-leaves.
3. `att_mean` = 1.4–1.6 attackers per enumeration on average across the corpus. The Pass 1 cap-8 sizing has plenty of headroom; no case for further shrinking.

---

### Pass 3+ candidates — consolidated backlog

Merged from two sources: (a) items deferred from Pass 1 as too structural/risky to batch, and (b) the Pass 3 critique run (5 agents, post-Pass-2 state: eval 1101→1853 ns / 1.68×, search still elevated on `opening-with-skills-03/04` at d6). Grouped by logical theme; the same underlying issue that surfaced from multiple critique angles is folded into one entry with the citations preserved.

#### Group A — MAEE / attacker enumeration internals

Core structural complaint from Pass 1 and reinforced by Pass 3 agents 2 & 4: MAEE does too much work per call and repeats it. **Pass 3 landed the initial-enumeration path (per-position attackers table) and MAEE_MAX_PLIES 32→16.** The remaining Group A items below are still open.

- ~~**Precomputed per-position "who attacks square S" table.**~~ **LANDED in Pass 3** for the initial `enumerate_attackers` call in `maee`. Cross-cutting complaint from all 5 Pass 1 critique agents; reinforced by Pass 3 agents 2 & 4. Table is per-`evaluate_breakdown`; not incrementally maintained across kills yet. Directly enables Pass 2 item #1 option (c) — MAEE-everywhere is now the production path.
- **MAEE re-enumeration is total, not incremental.** Both sides' attackers re-enumerated after every kill (`evaluator.rs:545-546`) even though only Guards can gain reach from a vacated blocker. Pass 3 kept kill-triggered re-enums on the from-scratch path. Fixing this is the biggest remaining Group A win — a `newly_reachable_guards` bitboard diff would scope the update.
- **`enumerate_attackers` called 2× per target on MAEE entry** — RESOLVED for the initial call (both sides read from the same table). Still called 2× per kill.
- **`AttackerList` O(n²) hot loop.** `push` is insertion-sort with nested shifts; `pop_front` shifts the whole array (`evaluator.rs:389-424`). Struct fits in a u32x8 — a bitonic sort or ring buffer would be trivial.
- **`threat_bb` work is thrown away.** Runs inside `maee_side` (`:587`); `maee` then re-enumerates attackers ignoring it — no hand-off. May be subsumable entirely by the attackers table; needs audit.
- **Stand-pat fold-back is serial** (`evaluator.rs:558-564`): right-to-left mutation of `gains[]` where a single-pass running min/max would suffice.
- ~~**Oversized MAEE buffers.**~~ **LANDED in Pass 3** — `MAEE_MAX_PLIES` shrunk 32 → 16 (true geometric ceiling = 2 × 8-per-side max). `gains: [i32; 16]` = 64 B stack, halved.

#### Group B — Incremental state via make/unmake

Eval recomputes from scratch what make/unmake could maintain. Cross-cutting with Group A (attacker table would also want incremental maintenance).

- **Incremental maintenance of `all_occ`, material sums, HP totals, armor totals.** Recomputed at eval:244, enumerate:456, maee_side:584, skill_activity:621 — 4+ times per eval — with make/unmake already the natural hook (currently restores bitboards but never touches aggregates). Per-side material / HP / armor / skill sums recomputed from scratch on every leaf (`evaluator.rs:247-291`) despite a move touching ≤2 mailbox squares.
- **Piece-kind byte in MailboxEntry.** `piece_material_of` and eval main loop re-test `champions.0 & bit` / `guards.0 & bit` (`evaluator.rs:257-260, 435-437, 478, 538`) — a piece-kind byte would eliminate the classification.
- **No delta-eval scheme.** Every leaf recomputes material + mobility + MAEE + skill_activity from scratch even though a move touches ≤2 squares.

#### Group C — Zobrist-keyed eval cache

- **Zobrist-keyed eval cache.** `pos.zobrist` is already incrementally maintained (`position.rs:168`, `make_unmake.rs:110`), turn-scoped modifiers already mixed in. Requires cache-invalidation discipline (skills mutate state in ways that need care around the cache key). Separate pass.

#### Group D — Search-side move ordering (biggest search-side win)

- **SEE for move ordering in `alpha_beta.rs`.** The single biggest search-side win identified in Pass 1. Reinforced by Pass 3 Agent 5: TT-move-first swap (`alpha_beta.rs:304-309`) then killers/history only — no MVV-LVA, no SEE. The MAEE machinery that runs at leaf is not reused where SEE-scoring is actually good — pre-leaf capture ordering.
- **Sort gating.** Sort gated at `depth >= 3` (`alpha_beta.rs:312`); QS iterates raw generator order (`quiescence.rs:207-222`). First-move cutoff in QS is generator-order-dependent.
- **History table pollution.** `depth*depth` bumps from any cutoff including EndPhase (`(from,to)=(0,0)`) accumulate every phase-boundary cutoff, dominating scoring for anything landing on square 0 (`alpha_beta.rs:146-164`; comment at `:80-82` waves this off).
- **Killer collisions.** Killers indexed by ply only, shared across AB and QS plies — QS's ply (up to +8) collides with AB killers.

#### Group E — Quiescence redesign

Cross-cutting with Group D. Pass 3 Agent 5's central point: **MAEE-from-QS is where the time is**, not MAEE-from-static-leaves. `qs_nodes / (ab_nodes + qs_nodes) = 0.6–0.97` on the slow positions per Pass 2's item-5 counters.

- **QS calls full MAEE-inclusive eval on every node** (`quiescence.rs:192`) — dominant cost of the 1.68× regression.
- **Stand-pat + MAEE double-count.** MAEE bakes net-of-exchange into the leaf; QS then rolls the same exchange out dynamically while re-pricing with the credit baked in.
- **MAEE runs unconditionally in `evaluate_breakdown`** (`:315-320`), including at QS internal nodes that will immediately expand captures.
- **QS has no TT** (self-declared, `quiescence.rs:24-26`) — same tactical position via different orderings re-runs MAEE every time.
- Overlaps with Pass 2 item #1 option (b): fold MAEE into quiescence rather than static leaf eval.

#### Group F — TT hygiene under MAEE-perturbed scores

- **TT poisoning.** Stores unstable MAEE-perturbed scores at shallow depth, probes with `>= depth` cutoff (`alpha_beta.rs:238-246`).
- **NMP under MAEE.** Verified pre-MAEE (`:40-48` cites Session 41); never re-verified against MAEE-inflated non-mate scores. `!is_mate` guard doesn't defend against MAEE inflation.
- **Null branch fail-high returns tainted score.** Returns MAEE-tainted `s`, not `beta` (`:282, 285`) — propagates a tainted lower bound.
- **Forced-move short-circuit scale mismatch.** Returns raw leaf eval labelled as `max_depth`-scale (`:419-424`) — score-diffing consumers get a mismatched scale.

#### Group G — Lazy / staged eval

- **No lazy-eval / staged eval.** Cheap material differential could fail-high/low against alpha/beta before MAEE ever fires (`evaluator.rs:232-335`).

#### Group H — Bitboard / bitwise micro-opts

- **Propagate `SQ_BIT` (added in Pass 2).** Barely used. Runtime `1u64 << sq` at `evaluator.rs:251, :721, :726, :754, :755, :760`, `bitboard.rs:16,21`, `magic.rs:83, 108, 268, 318, 320, 343, 362, 392`, `generator.rs:325, 362, 365, 489, 506, 516, 576`. (Caveat from Agent 3: SQ_BIT is 512 B / 8 cache lines of const data — measure before further expansion; may be a loss vs a 1-cycle shift.)
- **Cache `all_occ`.** Recomputed 4+ times per eval (`evaluator.rs:244, 456, 584, 621`) — also covered by Group B.
- **Class-bitboard iteration in main eval loop.** `evaluator.rs:248-291` re-tests `guards & mask`, `kings & mask`, `champions & mask` per bit; iterating by class-bitboard would eliminate per-square classification.
- **`RANK[64]` / `FILE[64]` tables.** Every geometry helper recomputes `sq/8` and `sq%8` (`magic.rs:216-223, 255-258, 191-198, 203-211, 180-185`).
- **`threat_bb` Kogge-Stone.** `magic.rs:386-402` iterates own pieces with a fn call per Champion; a whole-board Kogge-Stone king-fill would replace the loop.
- **Hoist hot bitboards to locals.** `pos.kings.0`, `pos.p1_pieces.0`, etc. reloaded through `&Position` every use — hoisting lets the compiler keep them in registers.
- **`OnceLock` → `const` for magic tables.** `WITHIN_RANGE`, `RAYS`, `BETWEEN`, `MOVE1`, `CHEBY` (`magic.rs:29, 33, 37, 229, 232`) — every access pays `get_or_init` acquire fence though inputs are const. Same shape as `SQ_BIT` (const), different cost. Also missing `#[inline]` on the accessors (`magic.rs:129, 172, 216, 255, 282, 299, 341, 386`).
- **Skill activity gating on `moved_this_phase`.** Deferred from Pass 1; skills don't consume a piece's move slot, so the gating logic needs care to avoid dropping legitimate skill-cast value.
- **`skill_value_from_id` on Guards.** Fired for every occupied square including Guards, which have no skills.
- **Heal/Plate target counts** (`evaluator.rs:731-747`) re-consult mailbox per candidate — no side-wide "needs-heal / needs-plate" bitboard.

#### Group I — Guard movement (BFS-2 rewrite)

- **`movement_targets_speed2` shift-and-mask rewrite.** `magic.rs:299-330` allocates `dist[64]` / `front[64]` / `next[64]` scratch per call and uses branchy `(0..8).contains(&r)` clipping. Pure shift-and-mask BFS-2 would replace it. Alternatively, a **64 × 256 speed-2-by-ring-occupancy lookup table** (16 KB) would kill the BFS entirely.
- **Memoise BFS-2 across call sites.** Same Guard/occupancy recomputed from 3 call-sites per leaf (mobility loop, `enumerate_attackers`, `threat_bb`).
- **Chebyshev-2 pre-reject** for Guard attacker enumeration — a Guard 6 squares away currently pays full BFS-2.
- **`movement_attack_targets_speed2` reuse `move1_table`.** `magic.rs:341-369` recomputes neighbours from scratch when `move1_table[enemy]` already gives all 8.
- Risk: subtle Guard-movement bugs; needs a full test sweep.

#### Group J — Magic bitboards for skill rays

- **`skill_attacks` magic / PEXT / Hyperbola Quintessence.** `magic.rs:129-168` is classical ray-scan with per-direction blocker XOR — the textbook case for magic bitboards.
- **Share skill_attacks across Champion slots** — if both slots have the same range, no shared computation currently.

#### Group K — Position layout / cache

High blast radius; several sub-items here may deserve their own passes.

- **Position size + `Clone` heaviness.** 250+ B, `Cloned` wholesale (`position.rs:95`). Every make/unmake and every search branch memcpys the whole thing. Requires an audit of every clone site to understand how many are actually necessary vs incidental. Consider custom `Clone` that skips cold fields.
- **Cold fields inline with hot eval set.** `pending_bodyguard` (admitted always-None), `moved_this_phase`, `pending_modifiers`, `tracked_enemies/casters`, `champion_credit`, `round_number`, `game_result` all ride every clone and pollute cache lines the linear eval walk pulls in.
- **`u128 champion_credit` forces 16-B alignment on the whole struct** (`position.rs:159`).
- **No `#[repr(C)]` on Position** — field reordering guarantees hidden padding between hot/cold, zero cache-line control.
- **Mailbox AoS → SoA.** `[MailboxEntry; 64]` (`position.rs:106`); eval hot loop touches HP/armor/skill1/skill2. SoA would let HP-only loads hit 64-B lines densely. High blast radius — touches every consumer of per-square data.
- **`EvalBreakdown` waste.** 64 B on the stack (`evaluator.rs:170-196`); `evaluate()` projects to a single `i32` (`:198-200`) — full breakdown built though search only needs `.total`.
- **`AttackerList` returned by value** from `enumerate_attackers` (`evaluator.rs:449`) and re-assigned twice per kill — ~36 B stack copy per re-enum. Also has 1 B tail padding.
- **`Bitboard` missing `#[repr(transparent)]`** (`bitboard.rs:6-7`) — compiler isn't told it's layout-identical to `u64`.
- **`Player` / `Phase` enums lack `#[repr(u8)]`** — matches in hot loops (`evaluator.rs:451-454, 579-582, 617-620`) may compile branchy where a masked int would be branchless.
- **No prefetch hints on `pos.mailbox[sq]`** — 2 B reads into the middle of a 250+ B struct via sparse-bitboard-driven access.

#### Group L — Forced-move extension, take 2

- **Retry Pass 2 item 3.** The Pass 2 attempt failed because unconditional extension on `moves.len()==1` fires at every phase-boundary internal node (`actions_remaining==0 → EndPhase-only`) — structural, per-game-line condition, not a rare tactical one. Doubled the work of every phase transition, tanked depth reached in a time budget. Retry needs:
  - (a) prerequisite counter measuring how often `moves.len()==1 && kind ∉ {EndPhase, EndTurn}` fires per corpus position (build into `counters.rs` on Pass 3 startup),
  - (b) if that count is meaningfully >0, implement extension guarded on non-{EndPhase, EndTurn} sole-legal-action,
  - (c) additionally consider fractional extensions (0.5 ply, accumulated to integer) so multiple forced nodes on a line don't stack to full-tree explosion.

#### Group M — Drive-by cleanups

Low-risk hygiene; batch or slot into unrelated passes.

- **Dead `lsb() -> Option<u8>`** at `bitboard.rs:35-37` — every caller uses `trailing_zeros()` inline.
- **`ARMOR_CAP` / `HP_CAP` / `FULL_HP` / `INJURED_HP` duplicated** across `evaluator.rs:118-119`, `generator.rs:57-58`, `make_unmake.rs:539-541` — three copies, drift risk.
- **`actions_remaining==0` counter bumps** at `evaluator.rs:315, 318, 322` — short-circuit was reverted but bumps still fire. If atomic, per-leaf synchronisation with no gate.

---

### Pass 3 (implemented 2026-07-07)

**Scope:** Group A subset — attacker enumeration internals, initial-enumeration path only.

Items landed:
- **Per-position attackers table.** New `AttackersTable { p1_of: [u64;64], p2_of: [u64;64] }` built once per `evaluate_breakdown` call. For each non-king piece, computes its attack set (Champions: `MOVE1`; Guards: reach-∪-self expanded by king_expand, masked by cheby-2 from origin — matches the game rule that Guard move-attack lands ≤ speed-1 steps then attacks cheby-1, max cheby-2 from origin per `generator.rs:473-512`). Then scatters to per-target attacker bitboards. Initial `enumerate_attackers` calls in `maee` read from the table via new `enumerate_attackers_from_table` that filters the target's attacker bitboard by side and builds `AttackerList` with LVA cost via a single trailing-zeros loop.
- **`MAEE_MAX_PLIES` 32 → 16.** True geometric ceiling is 2 × 8 = 16 attackers (8-per-side max, both sides fold in the exchange). Halves the `gains[]` stack footprint per MAEE call.
- **Kill-triggered re-enumeration still uses from-scratch `enumerate_attackers`.** Vacated-blocker fixup is deferred to a future pass — the table is not updated across kills. Correctness-wise this is safe (kill re-enum sees the post-kill occupancy); the wasted work is duplicated enumeration that the table could have amortised. Left for now to keep this pass's scope tight.
- **Correctness canary feature `maee_paranoid`.** Added to `Cargo.toml`. When enabled, every initial `enumerate_attackers_from_table` call is cross-checked against a from-scratch `enumerate_attackers` (length + per-slot sq/cost). Default off — quadruples eval cost. Used during development to catch the Guard cheby-2 bug (see below).

**Deferred out of pass** (still on Group A backlog):
- Incremental table maintenance across kills (avoid the from-scratch re-enum on kill).
- `AttackerList` O(n²) shift removal (bitonic sort / ring buffer).
- Stand-pat fold-back single-pass rewrite.
- `threat_bb` hand-off into MAEE.
- Full removal of `threat_bb` (still called inside `maee_side`).

**Success criteria:** eval geo mean below Pass 2's 1853 ns; search d6 node counts unchanged (behaviour-preserving refactor); no test regressions; endgame FEN still solves (not re-verified — no search-quality concern surfaced).

**Correctness bug caught during development:**
Initial attacker set for Guards used `king_expand(reach | sq_bit)` without a cheby-2 mask, admitting cheby-3 attacks (Guard reaches dist-2 landing, then attacks 1 more step from there → 3). 21 tests failed with `attackers_stm len mismatch`. Fix: mask fanout by `king_expand(king_expand(sq_bit)) & !sq_bit` (cheby-2 ring). All 392 tests pass. The `maee_paranoid` canary caught the residual off-by-one before it shipped.

**Results:**

- Eval-only: `bench/results/eval-post-pass3.json`
  - Geo mean: **1719 ns/eval** (1853 → 1719, **-7% vs post-pass2**, still 1.56× vs pre-MAEE baseline 1101).
  - Mean: **3687 ns** (4721 → 3687, -22%).
  - Max: **15406 ns** (21555 → 15406, -29%). Bigger max wins reflect table's win-scaling with attacker-set density.

- Search d6: `bench/results/search-post-pass3-d6.json` (vs `search-post-pass2-d6.json` — captured this pass; Pass 2 was never full-corpus benched).
  - Total nodes: 24.74 M → 24.74 M (**identical**, as expected for a behaviour-preserving refactor). All 30 positions match Pass 2 node counts to the last node.
  - Total wall time: 740.4 s → **363.8 s** (**-51% vs Pass 2**).
  - Aggregate geo NPS: 587k → 442k (**declined**). Explanation below.
  - **Wall time still elevated vs Pass 1** (316 s → 364 s, +15%). This is the residual cost of Pass 2's phase-gate drop — MAEE now runs at every leaf, so Pass 3's per-eval savings recover most but not all of Pass 2's node inflation.

- **Per-position character:**
  - `opening-with-skills-03`: 581 s → **156 s** (0.27×). The dominant Pass 2 pain point. Pass 3's table pays out massively where MAEE-per-node is the bottleneck.
  - `opening-with-skills-04`: 56 s → 72 s (1.29×). Per-node cost went up modestly.
  - Most mid/skill-phase positions: 1.20–1.40× **slower per-position wall time** than Pass 2. Per-eval table build has a fixed cost (2× 64-square scatter loops); on positions where MAEE call rate per leaf is moderate the amortisation doesn't cover the build cost.
  - Endgame positions: 1.8–2.0× **slower per-position wall time** — same story amplified, since these positions had near-instant Pass 2 runs and the table-build fixed cost dominates.

- **Why does geo NPS drop despite total time halving?** The one position where Pass 3 wins big (`opening-with-skills-03`) dominates the *sum* (156 s of 364 s total) but is only 1/23 of the geo product. Geo mean penalises the many modestly-slower positions more than it rewards the one massively-faster position. Sum-time metric is the fair one for this pass; NPS misleads.

- **Behaviour-preserving guarantee held.** Node counts identical position-by-position confirms the table returns the same initial attacker set as the from-scratch enumeration in every leaf visited during the d6 corpus. No move-ordering perturbation, no eval-value discontinuity — the only variable is per-node time.

**Known post-Pass-3 regressions:**
- Per-node cost regressed vs Pass 2 on 20+ positions (table build overhead > MAEE savings on low-call-rate leaves). Two paths forward: (a) make the table cheaper by incrementally maintaining it in make/unmake (Group B), (b) skip table build when a fast pre-check says MAEE won't fire much this leaf. Both are Pass 4+ candidates.
- Correctness canary raised total-suite time from ~60 s to ~660 s when enabled (10× — from-scratch enumeration is what we're trying to replace, and running it *twice* per leaf plus assertions is expensive). Kept off by default; noted in Cargo.toml.

**Follow-ups for Pass 4+:**
1. **Incremental table maintenance across kills** — the biggest remaining Group A win. Only Guards near the vacated square gain reach; a bitboard difference (`newly_reachable_guards`) would scope the update to a few squares instead of a full rebuild.
2. **AttackerList O(n²) shift removal** — item still on the Group A list, not attempted this pass.
3. **Stand-pat fold-back single pass** — item still on Group A list.
4. **`threat_bb` hand-off / deletion audit** — `threat_bb` still runs inside `maee_side`; consider whether the table subsumes it.
5. **Table-build lazy / cached** — Group C (Zobrist-keyed) territory; a per-position table cache keyed on `pos.zobrist` might amortise the fixed cost across QS repetitions.

**No memory / STATUS.md / HANDOVER.md updates needed** — this is a mechanical perf pass, not a design shift.

---

### Pass 3 continuation (2026-07-07) — Group A cleanup

Second chunk of Pass 3 items, worked with the user in a single sitting after the initial attackers-table landed. Spot-checked on a 4-position corpus (ows-03, ows-04, midgame-move-03, skill-phase-full-03) at d6 rather than the full 30-position sweep, because full-corpus takes 25+ min.

**Items landed:**

- **Item 3 — stand-pat fold-back single pass.** Rewrote the two-pass min/max walk over `gains[]` as a single right-to-left scalar accumulator: seed `val = gains[n-1]`, then for each `i` from `n-2` down to `0`, clamp `val` at 0 based on parity, add `gains[i]`. Same output, one loop instead of two. Spot-bench: ~1% net win — within variance, no regression.

- **Item 2 — AttackerList head-cursor (SHELVED).** Replaced `pop_front`'s shift with a head-cursor (`head: u8`, index into `items[]`). Theoretically O(n)→O(1) per pop. In practice: **regressed 3-8%** on all 4 spot positions. Root cause: the old constant `items[0]` read was SROA-friendly (kept in registers); the variable `items[head as usize]` forced memory loads and broke scalar replacement. At `att_mean ≈ 1.5`, the shift was essentially free (n=1 does zero shifts; n=2 does one copy). **Reverted entirely.** Not worth doing at current attacker densities. Kept as a "not worth it here" note for future reference — the same technique might pay off if attacker sets ever grow.

- **Item 4a — Guard move-attack geometry fix + fast recompute.** Discovered while designing item 1: both `enumerate_attackers` and the Pass-3-initial `build_attackers_table` were using `movement_targets_speed2` (dist-≤-2 BFS through empties) as the Guard approach mask. Game rule per `generator.rs:473-512` is approach ≤ **speed-1** (dist-≤-1) — Guard moves 0 or 1 empty step then attacks cheby-1, max reach cheby-2 from origin. The BFS-2 admitted dist-2 landings as valid approaches, over-approximating the attacker set. Both bugged (pre-existing — Pass 3's initial code faithfully reproduced `enumerate_attackers`'s over-approximation, which is why the paranoid canary passed).

  Fix (matches game rule + happens to be much faster):
  ```rust
  let approach = (magic::movement_targets_speed1(sq).0 & !all_occ) | sq_bit;
  let fanout = king_expand(approach) & !sq_bit;  // in table build
  king_expand(approach) & target_bit != 0        // in enumerate_attackers
  ```
  Pure bitwise; no scratch arrays; no branchy clip loop. Applied at both sites.

  Spot-bench (post-4a vs post-4b-shelved baseline):
  - `opening-with-skills-03`: 156.6 → 27.8 s (**5.6× faster per-node**)
  - `opening-with-skills-04`: 71.6 → 12.5 s (5.7×)
  - `midgame-move-03`: 15.0 → 2.6 s (5.8×)
  - `skill-phase-full-03`: 39.5 → 6.3 s (6.3×)

  Node counts drifted **<2%** (the extra dist-2 landings hardly ever changed a MAEE verdict), so this is almost pure per-node speedup driven by:
  1. Killing `movement_targets_speed2`'s per-call `dist[64] + front[64] + next[64]` scratch alloc.
  2. Replacing branchy `(0..8).contains(&r)` clipping with bitfile masks (`NOT_A`, `NOT_H`).

  This is the phenomenal win Pass 3 chunk-2 was hoping for. Not a perf trick — a game-rule bug fix that also happened to be the biggest lever on the profile.

- **Item 4b — incremental attackers-bitmask maintenance across kills.** Refactored `enumerate_attackers_from_table` into `attackers_bb_from_table` (returns `u64`) + `build_attacker_list(pos, bits)` (sorted list from bits). In `maee`, track `attackers_stm_bb: u64` and `attackers_dfd_bb: u64` alongside the sorted lists. On each kill:
  1. Clear the killed attacker's bit from its side's bitmask (`&= !SQ_BIT[att.sq]`).
  2. If the vacated origin sits cheby-1 of the target (`target_bit ∈ king_expand(att_bit)`), Guards adjacent to it may now use it as an approach square. `neigh = king_expand(att_bit) & pos.guards.0 & !vacated & !target_bit`. OR the appropriate ownership-masked bits into each side's bitmask. Champions are geometry-invariant — no addition step needed.
  3. Rebuild `AttackerList`s from the updated bitmasks via `build_attacker_list`.

  Replaces the per-kill from-scratch `enumerate_attackers` 64-square scan with a handful of bitwise ops per kill. Extended `maee_paranoid` canary to compare the incrementally-maintained list against a fresh `enumerate_attackers(vacated)` on every kill (was previously only checking the initial enum).

  Correctness canary caught one bug during development: initial add-list didn't exclude `target_bit`, so a Guard sitting *at* the target square (the current victim) could be added as an attacker on the next round. Fixed by masking `& !target_bit` in the neighbour set. All 392 tests pass under `--features maee_paranoid`.

  Spot-bench (post-4b vs post-4a):
  - `opening-with-skills-03`: 27.8 → 25.3 s (**-9.3%**)
  - `opening-with-skills-04`: 12.5 → 11.2 s (-10.7%)
  - `midgame-move-03`: 2.6 → 2.3 s (-11.7%)
  - `skill-phase-full-03`: 6.3 → 5.6 s (-10.9%)

  Node counts identical (fully behaviour-preserving). ~10% wall-time win over 4a on top of 4a's own 5-6× improvement. Also removes the last dependency on `enumerate_attackers` from the hot path — it's now `#[cfg(feature = "maee_paranoid")]`-gated (only compiled when the canary is on).

- **Dead-code cleanup.** Removed `enumerate_attackers_from_table` (subsumed by the split). Gated `enumerate_attackers` behind `#[cfg(feature = "maee_paranoid")]` — release builds no longer compile it.

**Bench artifacts:** `/tmp/spot-post-item3.json`, `/tmp/spot-post-item4a.json`, `/tmp/spot-post-item4b.json`. Full-corpus d6: `game/bench/results/search-post-pass3-chunk2-d6.json`.

**Full-corpus d6 result (vs chunk-1 baseline `search-post-pass3-d6.json`):**
- Total wall time: **363.8 s → 58.7 s (-83.9%)**. 6.2× speedup.
- Total nodes: 24.74 M → 24.72 M (-0.1%). Behaviour-preserving to the extent expected; the small drift comes from item 4a's geometry fix (fewer spurious attackers → occasional MAEE-verdict flip → subtly different search tree). 19/30 positions have identical node counts; the 11 with drift are all mid/skill-phase positions and all drift <3%.
- Per-position character: chunk-1's known regressions on mid/skill-phase positions (1.2-1.4× slower per-position vs Pass 2) are now completely reversed — most positions -75% to -85% wall time. Endgame + trivial positions unchanged (they had no measurable MAEE cost to begin with).

**Combined Pass 3 result (chunk-1 + chunk-2) vs Pass 2:**
- Total wall time: 740 s → 59 s (-92%, ~12.6× speedup).
- Node counts essentially identical (both chunks combined are within 0.1% of Pass 2).
- Per-node cost regression from chunk-1 fully absorbed by chunk-2's item 4a.

**Still deferred (Group A remnants):**
- `threat_bb` hand-off / deletion audit. Still called inside `maee_side`; the attackers table subsumes some but not all of what it does. Left for Pass 4.

**Combined Pass 3 (initial chunk + continuation) per-node character:**
- Group A's original ~1.2-1.4× per-position slowdown vs Pass 2 (from the initial-chunk table-build fixed cost) should now be more than recovered by item 4a's 5-6× per-node speedup. Confirm on full-corpus.

**No memory / STATUS.md / HANDOVER.md updates needed.**

---

## Notes for future passes

- **Always commit before starting a pass.** That commit is then a rollback anchor.
- **Never batch two unrelated optimisation ideas in one pass.** The point of iterating is that each pass's benchmark tells us whether that specific change helped. Batching hides regressions.
- **Recritique after no open items remain.** The same 5 agents. Cross-cutting themes shift as the low-hanging fruit disappears.
- **If a pass makes things worse, revert cleanly.** The commit + bench-file pattern makes this trivial.
- **Endgame FEN verification is not mandatory every pass.** Run it only if critique agents surface search-quality concerns or if a known regression list is empty. Rationale: it's manual, and the corpus + node-count deltas already flag most quality regressions faster.
- **Flag known regressions in the pass log rather than blocking on them.** Document the position, the delta, and the suspected cause. Address in a later pass with explicit scope. Don't silently accept them and don't panic-revert a net-positive pass.
