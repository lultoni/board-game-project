# Search-Deepening Plan - hit d6 < 1s everywhere

*Session 48 (2026-07-12). Companion to `search-speed-benchmark-plan.md` (infra) and
`alpha-beta-optimisation-catalogue.md` (technique reference).*
*Status: scoped, ready to implement. No ADR needed - this is optimisation, not a design decision.*

---

## Goal (the termination condition)

**Every corpus position completes depth-6 search in under 1 second**, measured by
`search_bench --depth 6` on this workstation. Secondary win: the fixed-time sweep
should show the whole corpus reaching depth ≥ 6 within a 1000 ms budget.

We are done when that holds. If after the high-leverage work below some position
still can't hit d6 < 1s, we fall back to trimming eval cost further (§3) and/or
accept a documented exception with a reason.

---

## Current reality (fresh baseline, 2026-07-12)

The committed baseline files were regenerated this session. **The previous
baseline (commit `b70c005`, 2026-07-05) predated the entire ns-43 term-registry
eval rewrite** (14 commits `f14f2ba`…`f6cf584`) and was worthless as a comparison
point - it reported 2.5M geo-NPS against an evaluator that no longer exists.

Fresh `--depth 6` numbers (`--runs 5`, median-by-nodes, this box under normal load):

| Metric | Value |
|---|---|
| Geometric-mean NPS | **~587 K** (was 2.5M pre-ns-43 - eval rewrite cut throughput ~4×) |
| Positions over 1000 ms at d6 | **5 / 30** |
| Worst position | `opening-with-skills-03` - **5504 ms**, 2.52M nodes, EBF 11.67 |
| eval calls / node | **0.58** (more than half of all nodes pay a full static eval) |
| QS-node fraction | 0.58 |

The five d6-over-1s offenders (all high-branching midgame / skill-phase / opening):

```
opening-with-skills-03   5504 ms   2 522 954 nodes   ebf 11.67
midgame-move-03          4880 ms   3 057 444 nodes   ebf 12.05
skill-phase-full-03      3481 ms   1 176 732 nodes   ebf 10.28
midgame-move-05          1519 ms     992 398 nodes   ebf  9.99
skill-phase-full-04      1008 ms     399 931 nodes   ebf  8.58
```

Two independent problems fall straight out of these numbers, and both are large:

### Problem 1 - eval is 10–40× too slow (throughput axis)

`search_bench --eval-only` measures the static eval in isolation:

```
ns/eval: min 1242   mean 2304   geo 2224   max 3129
```

A hand-written static eval of this complexity should be **50–200 ns**. We are at
**~2300 ns**. At eval/node = 0.58, that's ~1300 ns of eval per node - plausibly
**>50 % of all node time is the evaluator**, which explains the 4× NPS collapse
since the pre-ns-43 baseline.

**Root cause - the ns-43 registry rebuilds itself on every call.** The search
leaf path is `HeuristicEvaluator::evaluate → evaluate() → evaluate_breakdown() →
registry::default_terms() + evaluate_dyn() → DynBreakdown::to_legacy()`
(`search/evaluator/mod.rs:70-79`, `registry.rs:23-117`). Per single leaf eval it:

1. Allocates a `Vec<Box<dyn EvalTerm>>` of **14 boxed trait objects**
   (`default_terms`) - 14 heap allocations + vtable setup, every call.
2. Allocates 3 more `Vec`s (`active`, `per_piece`, `side_level`).
3. Allocates `acc: Vec<(i32,i32)>` and `entries: Vec<TermEntry>`.
4. Dispatches through `dyn EvalTerm` for every term × every occupied square.
5. Projects the whole `DynBreakdown` to the legacy fixed-field struct
   (`to_legacy()`) - building the frontend-facing breakdown at every leaf.

The terms themselves are stateless zero-size structs (`terms::Material` etc.), so
**all of that allocation + dispatch + projection is pure overhead in the search
path.** The registry/breakdown machinery exists to serve the frontend eval panel,
telemetry, and nn_trainer - none of which run in the inner loop. The search wants
a scalar `i32` and pays for a diagnostic data structure to get it.

This is by far the highest-leverage single fix available.

### Problem 2 - move ordering doesn't contain branching (tree-shape axis)

EBF 8–12 on the slow positions with TT-hit rates as low as 33–36 %. For reference
a well-ordered alpha-beta gets EBF toward `sqrt(branching)`. The catalogue's
**#1 highest-Elo remaining item - LMR + PVS as a pair - was never implemented**
(grep confirms no LMR/PVS/aspiration in the tree). NMP is the only pruning we have
past TT + killers + history. LMR directly attacks the Skill-phase / midgame fan-out
that produces these EBFs, and PVS is its required re-search partner (catalogue §5).

Attacking throughput (Problem 1) multiplies every position's speed by a constant;
attacking tree shape (Problem 2) shrinks the exponent on exactly the positions that
blow the 1s budget. **We need both.** A 3× eval speedup alone would pull
`opening-with-skills-03` from 5.5s to ~1.8s - still over budget - so ordering work
is not optional.

---

## Methodology (unchanged, load-bearing)

Everything in `search-speed-benchmark-plan.md` and the catalogue's methodological
notes still applies, **with one deliberate relaxation from the old Session-36/41
strict-no-regression protocol** (designer decision, 2026-07-12):

- **Accept on net win, not zero regression.** Grade every change on the full sweep
  (`run_sweep.sh <prefix>`): `depth6` + `time{100,500,1000,3000}ms` + determinism.
  A change is **accepted if it is a net improvement toward the goal** - a solution
  that cuts overall d6 time ~20 % while making a few positions ~50 % slower is
  totally acceptable. We are optimising the *aggregate* (geo-mean NPS / total d6
  wall-clock across the corpus, and the count of positions over 1s), not defending
  every individual cell. **Rule of thumb:** accept if geo-mean d6 time drops AND
  the count of positions-over-1s does not increase AND no *previously-under-1s
  target position* is pushed decisively over 1s by the change alone. Judgement
  call at the margins - record the trade in the commit message so it's auditable.
- **Determinism stays a hard gate.** `--determinism` must stay 30/30 after every
  landing. (Confirmed 30/30 on the fresh baseline.) A non-deterministic search is
  never acceptable, regardless of speed.
- **Category-by-category node breakdown** stays useful - not as a veto, but so an
  accepted trade is understood (e.g. "this speeds midgame at the cost of
  skill-phase; net win because midgame dominates the over-1s set").
- **One technique per commit**, benchmark before/after, baseline moved when
  accepted, rationale + the trade-off in the commit message and a retro entry
  appended to the catalogue.

**Important accept-gate nuance for eval-throughput work (Phase 1).** There the
correctness bar is *stronger* than the speed relaxation above: a pure-speed eval
refactor must be **byte-identical** to today's output. The `golden_eval_unchanged`
and `eval_is_deterministic` tests (`evaluator/mod.rs`) pin that. Changing an eval
*score* to go faster is a different kind of change (it alters play strength) and
is out of scope for the throughput phase - keep those separate.

---

## Work, in priority order (Elo-per-hour × fits-our-goal)

### Phase 1 - Eval throughput (Problem 1). Do this first; it's the biggest, safest win.

The wins here are behaviour-preserving, so they carry **no play-strength risk** and
are gated by the golden test rather than the fuzzy sweep. Land them in this order,
`--eval-only` before/after each:

**1a. Hoist the term set out of the per-call path.**
`default_terms()` builds 14 boxes every call. The term set is static. Options,
cheapest first:
   - Make the per-piece and side-level term application a **fixed straight-line
     function** - call each term's logic directly, no `Vec<Box<dyn>>`, no dispatch.
     The terms are already free functions in spirit (zero-size structs); inline
     their `score_piece` / `score_side` bodies into one monomorphic pass. This
     also lets LLVM see through the whole thing.
   - If keeping the registry shape is desired for tuning, build the term list
     **once** (`OnceLock`/`lazy_static` of `&'static [&'static dyn EvalTerm]`) and
     borrow it - kills the 14 allocations but keeps dynamic dispatch. Weaker than
     full inlining but a smaller diff. Prefer the inline path if the golden test
     stays green.

**1b. Give the search a scalar eval path that never builds a breakdown.**
The search only needs `i32`. Add `evaluate_scalar(pos) -> i32` that runs the shared
board pass accumulating a single total, with **no `DynBreakdown`, no `entries`
Vec, no `to_legacy()` projection.** `HeuristicEvaluator::evaluate` calls that;
`evaluate_breakdown` / `evaluate_dyn` stay for the frontend/telemetry/trainer and
keep their golden coverage. Expected: removes allocations 3–5 above from every leaf.

**1c. Kill remaining per-call allocations.** After 1a/1b, profile once more
(`--eval-only`). Any residual `Vec::with_capacity` in the hot path becomes a
stack array (term count is a small compile-time constant) or is removed. Target:
**≤ 300 ns/eval**, ideally ≤ 150. That alone is a 7–15× eval speedup and roughly a
2–3× search NPS improvement given eval is >50 % of node time.

**1d. (If needed) incremental eval.** Only if 1a–1c don't get the corpus under
budget when combined with Phase 2. Make/unmake already touch the affected squares;
a delta-updated running eval on `make`/`unmake` turns per-node eval into O(changed
squares) instead of O(board). High complexity, high payoff, but **deferred until
measured necessary** - it also unblocks the shelved RFP (catalogue revisit
condition explicitly names "after incremental eval lands"). Don't start here.

*Rationale for ordering:* 1a–1c are a few hours, behaviour-preserving, golden-gated,
and hit the single dominant cost. Doing eval first also makes Phase 2's LMR
re-searches cheaper (every re-searched node pays the eval), so the two compound.

### Phase 2 - Move ordering / tree shape (Problem 2). LMR + PVS as a pair.

Per catalogue §5, these must be implemented and graded **together** - PVS's
null-window re-search is the structure LMR's reduced-depth re-search rides on, and
each was rejected standalone in Session 36 (provisionally, pre-positional-eval).

**2a. PVS conversion.** First move full window `[α,β]`; siblings null-window; on a
null-window raise, re-search full. Absolute-POV framing already scoped in the
catalogue's Session-36 PVS entry (P1 null-window `[β-1,β]`, P2 `[α,α+1]`).

**2b. LMR on top.** Reduce depth for late, quiet, non-PV moves:
`R = base + ln(depth)·ln(move_idx)/divisor`, re-search at full depth on fail-high.
**Do not reduce:** PV node, in-check (`is_king_threatened`), TT-move, killers,
loud/King-threatening actions, `move_idx` below a small threshold. This is aimed
squarely at the EBF-10-12 Skill-phase tails. Tune `base`/`divisor` on the sweep.

Grade the pair on the full multi-budget sweep. Expected: the biggest single EBF
reduction of anything remaining; this is what pulls the 3–5s positions under 1s
once eval is fast.

### Phase 3 - Cheap follow-ons, re-graded atop fast-eval + LMR/PVS.

These were rejected or shelved earlier **against the slow eval and/or non-playing
engine** - their revisit conditions ("after LMR/PVS", "after incremental eval",
"after QS") are now in play. Grade each alone, in this order, keep only clean wins:

1. **Aspiration windows** (catalogue §5) - re-grade; LMR/PVS make the fail re-search
   cheaper, which is exactly why Session 36 rejected it standalone.
2. **Check extension** (King-threat +1 ply) - simple, ~20–40 Elo, test alone.
3. **RFP** (catalogue §2, Session 36 reject) - its stated revisit condition is
   "after QS lands / after incremental eval"; QS is in, and if 1d lands the +6 %
   per-node eval cost that killed it drops. Re-grade only then.
4. **Countermove + 1-ply continuation history** - diminishing but cumulative.

### Phase 4 - Only if still short: parallelism.

**Lazy SMP** (catalogue §8). Native-only, straightforward given the TT. This is a
throughput multiplier of last resort - a clean single-threaded d6 < 1s is the goal,
and the trainer + in-browser WASM path have their own threading constraints. Don't
reach for this until Phases 1–3 are exhausted and measured insufficient.

---

## Execution checklist

1. ~~Regenerate the stale baseline.~~ **Done this session** - all five
   `baseline-*.json` regenerated, determinism 30/30. (Uncommitted; commit alongside
   the first optimisation or as a standalone "refresh baseline post-ns-43" commit.)
2. **Phase 1a–1c** (eval throughput). `--eval-only` before/after; `golden_eval_unchanged`
   must stay green. Full sweep to confirm NPS win + net improvement. Land, move baseline.
3. **Phase 2** (LMR + PVS paired). Full sweep, category breakdown, tune formula. Land.
4. **Re-run `--depth 6`.** If all 30 < 1s → **done**, pivot back to eval-quality /
   NN-rater work. If not, Phase 3, then re-check.
5. **Phase 1d / Phase 4** only if the target still isn't met.

## Cross-references

- `search-speed-benchmark-plan.md` - bench infra, corpus, sweep protocol.
- `alpha-beta-optimisation-catalogue.md` - technique reference; LMR §2, PVS/aspiration §5,
  RFP §2, Lazy SMP §8, and the Session 36/37/41 retros (all rejections provisional).
- `core_engine/src/search/evaluator/{mod,registry,term,terms}.rs` - the eval hot path (Problem 1).
- `core_engine/src/search/alpha_beta.rs` - search loop; LMR/PVS land here (Problem 2).
- `core_engine/src/search/quiescence.rs` - QS (unchanged; second eval call site to keep fast).
- `game/bench/` - fresh baselines + `run_sweep.sh`.
