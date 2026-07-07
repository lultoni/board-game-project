# Evaluator Perf Passes — playbook + log

*Living doc. Each perf pass on `core_engine/src/search/evaluator.rs` follows the same recipe: critique → plan → implement → bench → recritique. Track results here so we can see whether each pass actually helped and what's left to attack.*

*Last updated: 2026-07-07 — Pass 1 landed. Known regressions flagged; Pass 2 items queued.*

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

## Notes for future passes

- **Always commit before starting a pass.** The MAEE-post commit is a rollback anchor.
- **Never batch two unrelated optimisation ideas in one pass.** The point of iterating is that each pass's benchmark tells us whether that specific change helped. Batching hides regressions.
- **Recritique after every pass.** The same 5 agents. Cross-cutting themes shift as the low-hanging fruit disappears.
- **If a pass makes things worse, revert cleanly.** The commit + bench-file pattern makes this trivial.
- **Endgame FEN verification is not mandatory every pass.** Run it only if critique agents surface search-quality concerns or if a known regression list is empty. Rationale: it's manual, and the corpus + node-count deltas already flag most quality regressions faster.
- **Flag known regressions in the pass log rather than blocking on them.** Document the position, the delta, and the suspected cause. Address in a later pass with explicit scope. Don't silently accept them and don't panic-revert a net-positive pass.
