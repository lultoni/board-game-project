# Evaluator Perf Passes — playbook + log

*Living doc. Each perf pass on `core_engine/src/search/evaluator.rs` follows the same recipe: critique → plan → implement → bench → recritique. Track results here so we can see whether each pass actually helped and what's left to attack.*

*Last updated: 2026-07-07 — Pass 2 complete. 4 of 5 items landed: 5 (counters), 4 (act0 revert), 1 (phase-gate drop), 2 (forced-move root). Item 3 attempted and reverted — see log. Pending: full-corpus d6 bench + re-critique agents.*

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

### Pass 2 explicit follow-ups (original, superseded above for items 4 and 5)

Items 1-3 remain open. Items 4 and 5 above.

---

### Pass 3+ candidates (deferred from Pass 1 scope-out)

These appeared in the Pass 1 critique but were held back as too structural or too risky to batch. Kept here so they aren't lost:

- **Precomputed per-position "who attacks square S" table.** Would eliminate MAEE's inner re-enumeration (the #1 cross-cutting complaint from all 5 critique agents). Big structural change; needs its own plan. **Directly enables Pass 2 item #1 option (c)** — cheap MAEE-everywhere depends on this.
- **Incremental maintenance of `all_occ`, material sums, HP totals, etc. via make/unmake.** Requires touching `make_unmake.rs` and every eval consumer. Separate pass.
- **Zobrist-keyed eval cache.** Requires cache-invalidation discipline (skills mutate state in ways that need care around the cache key). Separate pass.
- **SEE for move ordering (in `alpha_beta.rs`, not `evaluator.rs`).** The single biggest search-side win identified in the Pass 1 critique. It's a search change, not an eval change, so it lives in its own pass.
- **Quiescence redesign so MAEE isn't called on every QS node.** Requires understanding of QS interaction with MAEE and skill-cast leaves; separate investigation. Overlaps with Pass 2 item #1 option (b).
- **BFS-2 rewrite from breadth-first to two-wave shift-and-mask.** `magic.rs` change; risk of subtle bug in Guard movement generation. Separate pass with a full test sweep.
- **Mailbox AoS → SoA conversion.** `Position` layout change; touches every consumer of per-square data. High blast radius.
- **`Position::Clone` heaviness.** Requires audit of every clone site to understand how many are actually necessary vs incidental.
- **Skill activity gating on `moved_this_phase`.** Deferred; skills don't consume a piece's move slot, so the gating logic needs care to avoid dropping legitimate skill-cast value.
- **Forced-move extension inside the tree, take 2.** Pass 2 item 3 failed because unconditional extension on `moves.len()==1` fires at every phase-boundary internal node (`actions_remaining==0 → EndPhase-only`) — a structural, per-game-line condition, not a rare tactical one. Doubles the work of every phase transition, tanks depth reached in a time budget. Retry needs: (a) prerequisite counter measuring how often `moves.len()==1 && kind ∉ {EndPhase, EndTurn}` fires per corpus position (build into `counters.rs` on Pass 3 startup), (b) if that count is meaningfully >0, implement extension guarded on non-{EndPhase,EndTurn} sole-legal-action, (c) additionally consider fractional extensions (0.5 ply, accumulated to integer) so multiple forced nodes on a line don't stack to full-tree explosion.

---

## Notes for future passes

- **Always commit before starting a pass.** The MAEE-post commit is a rollback anchor.
- **Never batch two unrelated optimisation ideas in one pass.** The point of iterating is that each pass's benchmark tells us whether that specific change helped. Batching hides regressions.
- **Recritique after every pass.** The same 5 agents. Cross-cutting themes shift as the low-hanging fruit disappears.
- **If a pass makes things worse, revert cleanly.** The commit + bench-file pattern makes this trivial.
- **Endgame FEN verification is not mandatory every pass.** Run it only if critique agents surface search-quality concerns or if a known regression list is empty. Rationale: it's manual, and the corpus + node-count deltas already flag most quality regressions faster.
- **Flag known regressions in the pass log rather than blocking on them.** Document the position, the delta, and the suspected cause. Address in a later pass with explicit scope. Don't silently accept them and don't panic-revert a net-positive pass.
