# Search-Speed Benchmark Plan

*Session 35 (2026-06-26). Companion to `nnue-rework-plan.md` (search-speed prerequisite; absorbed the retired `nn-rater-plan.md`).*
*Status: scoped, ready to implement. No ADR needed.*

---

## Purpose

Grade candidate optimisations to the alpha-beta search loop before NN-rater training begins. Training time is dominated by `search_nodes_per_second x games_per_generation x generations` - every speedup compounds across millions of self-play games. We optimise here once, hard, then move to evaluator work.

**Termination condition for this work:** designer is happy with raw search speed. Then we pivot to the evaluator side (NN rater).

## Metric

**Nodes (or plies) searched per second** on a fixed corpus of predefined positions. Two modes, both reported:

1. **Fixed depth.** Run search to depth D on each position; report total nodes and nodes/sec. Pure node-throughput measurement.
2. **Fixed time.** Run search with a wall-clock budget T on each position; report depth reached, total nodes, effective branching factor (EBF). Measures player-facing strength.

**Caveat - explicitly accepted by designer.** Optimisations like the transposition table will *reduce* the node count at a given fixed depth (they prune work the previous version was doing redundantly). That's a feature, not a regression. We don't cry about fewer nodes when fixed-time depth-reached goes up.

Per-position output: nodes, nodes/sec, depth, TT hit rate, alpha-beta cutoff rate, EBF, time (median of N runs to dampen noise).
Aggregate output: geometric mean nodes/sec across corpus, percentile distribution of depth-reached at fixed time.

## Benchmark binary

Lives somewhere fast and reusable - no preference on location, only on properties:
- Native target (no WASM), zero ceremony to run (`cargo run --release -p ... -- bench`).
- Loads corpus from disk at startup, runs measurements, writes results to disk.
- Output is structured (JSON or TSV) so diffing baselines is automatable.

Likely shape: a new binary target in `core_engine` (`src/bin/search_bench.rs`) or a sibling crate `game/crates/search_bench/`. Decide when implementation starts - both are fast to set up. Avoid `criterion` - heavy dependency for the simple thing we need.

## Position corpus

**Reuse the existing FEN infrastructure** - `Position::to_fen()` / `Position::from_fen()` already exist in `core_engine/src/state/fen.rs`, are roundtrip-tested in telemetry tests, and `from_fen_strict` is available for the strict variant. Corpus positions are stored as FEN strings in a text file at `game/bench/corpus/` (or wherever the bench binary lives).

### Corpus composition

20-50 positions spanning the spectrum. Categories:

- **Opening** - full board, low piece interaction, broad fan-out from Move-Phase moves. (Stack M setup + a few plies in.)
- **Midgame, low skill density** - pieces traded, simple Move-Phase decisions, modest branching.
- **Midgame, high skill density** - many equipped skills active, Skill-Phase budget unspent, Charge/Focus state loaded. Worst-case branching.
- **Endgame, sparse board** - few pieces left, mate threats visible.
- **Endgame, attrition** - armor-stacked stalemate-adjacent positions.
- **Tactical / known-result** - positions with a known best line (e.g. mate-in-N). Double duty: speed benchmark AND correctness regression test. **At least one position with a known terminal we expect to reach within the depth/time budget.** Required.
- **Combo-counter loaded** - late-game enemy state with full combo counters, exploding Strike-target counts.
- **Phase boundary** - just-entered Skill-Phase with mixed Move/Strike options.

Each FEN in the corpus is paired with metadata:

```
# corpus.txt format
position_id, category, expected_best_move_depth_N, expected_score_range, fen
opening-01, opening, -, -, "<fen string>"
mate-in-3-01, tactical, 6, "[+MATE_SCORE-6, +MATE_SCORE]", "<fen string>"
```

Hand-curated. Building the corpus is half the work - without it the benchmark is useless. Plan: drop a few positions in from real game traces (telemetry FENs), add hand-built tactical ones.

## Two-mode runs

### Mode A - fixed depth
For each position:
- Run search at depth D (parameter, default e.g. 6).
- Record: nodes searched, time elapsed, nodes/sec, TT probes, TT hits, TT hit rate, beta cutoffs / total non-leaf nodes, best move + score.
- Repeat N times (default 5), report median.

### Mode B - fixed time
For each position:
- Run search with time budget T ms (parameter, default e.g. 1000 ms).
- Record: depth reached (iterative deepening), nodes searched, EBF (nodes^(1/depth)), best move + score.
- Repeat N times, report median depth and median nodes.

### Correctness assertions
Tactical positions have `expected_best_move_depth_N` and `expected_score_range` columns. After the search runs, the benchmark asserts:
- At depth ≥ N, the best move is one of the allowed best moves.
- The score is within the expected range.

If the assertion fails, the benchmark exits non-zero with a clear "REGRESSION: <position_id>" line. **This makes the benchmark a correctness regression test in addition to a speed test.** Same corpus, two purposes.

## Baselines and diffing

- **Baseline file** committed to repo: `game/bench/baseline.json` (or similar). Contains the median-of-N stats for every position in the corpus, on the current main branch.
- **Bench run output**: `bench-results-<timestamp>.json` to a scratch directory, not committed.
- **Diff tool**: simple script (Rust or shell+jq) compares a run to the baseline, prints per-position deltas and aggregates. Highlights regressions (>X% slower) and improvements.
- **Baseline updates**: manual. When an optimisation lands and we're happy, regenerate baseline and commit it. The new baseline becomes the bar for the next optimisation.

## Determinism

Search must produce the same node count on the same position on every run. Verify before relying on the benchmark:
- Re-run the same position 10 times at fixed depth; nodes must be identical across runs.
- If not, find the non-determinism source (likely move-ordering tiebreaks once we add them, or time-check noise - `TIME_CHECK_MASK` in `alpha_beta.rs` uses node count, so it's deterministic at fixed depth; need to confirm in fixed-time mode where it actually fires).

Fixed-depth mode: must be deterministic, no exceptions.
Fixed-time mode: nodes-searched will vary with machine load; that's expected. Run on a quiet machine, take median, report variance.

## CI / automation

**Manual only.** Designer runs the benchmark when an optimisation lands. No CI integration, no auto-run on every commit. This is a research/optimisation tool, not a gate. We stop running it once we pivot to evaluator work.

## Storage

Everything under `game/bench/`:
- `game/bench/corpus/` - position FENs + metadata.
- `game/bench/baseline.json` - current accepted baseline stats. Committed.
- `game/bench/results/` - scratch results from runs. Gitignored.
- `game/bench/README.md` - how to run, how to interpret, how to update the baseline.

## Candidate optimisations to benchmark, in priority order

**See `alpha-beta-optimisation-catalogue.md` for the full annotated technique list** - each with expected Elo, complexity, correctness properties, our-game-specific adaptations, and ordering reasoning. The catalogue is the authoritative source for what to try; this section is a summary.

The catalogue's recommended implementation order (Elo-per-engineering-hour, derived from CCRL amateur-engine consensus):

1. **PVS conversion** - drop-in replacement for straight alpha-beta. Composes with LMR.
2. **TT-move ordering** - verify it's wired into move generation.
3. **Aspiration windows** with exponential widening.
4. **Killer moves + history heuristic** - foundational quiet-move ordering.
5. **LMR** - biggest single Elo jump in amateur engines. Critical for our Skill-Phase branching.
6. **Reverse futility pruning (RFP)** - cheap.
7. **Adapt quiescence search** to our loud/quiet distinction (HP-reducing skills + King-threat). Use single-step Skill-Exchange Evaluation, NOT a chess-SEE port.
8. **Null-move pruning** with zugzwang awareness - `EndPhase` is NOT the null move; implement them distinctly.
9. **Check extension** (King-threat analog).
10. **Futility pruning + LMP** at low depth.
11. **Countermove + 1-ply continuation history.**
12. **Singular extensions.**
13. **Lazy SMP** (native first; WASM threading is separate plumbing).
14. **Correction history / ProbCut / 2+ ply continuation history** - Stockfish-class polish; revisit only after engine is competitive.

Items 1-8 alone produce a strong tactical engine.

Each technique lands as a separate commit. Benchmark before and after. Speedup logged in commit message and baseline file's history. The order above is a default - the benchmark's data may suggest reordering.

## Execution order

1. **Scaffold the bench binary** - empty target that loads a corpus file, runs search, writes structured output. No optimisations yet.
2. **Build the corpus** - 20-50 hand-curated FENs spanning the categories above. At least one tactical position with a known result.
3. **Verify determinism** - same-position-same-result-N-times sanity check.
4. **Generate initial baseline** - run on current main, commit `baseline.json`.
5. **Land optimisations one at a time** - each in its own commit, benchmark before/after, update baseline when accepted. Use the priority order above as a default but skip / reorder based on what the data says.
6. **Stop when designer is happy** - pivot to NN-eval work per `nnue-rework-plan.md` (Phase 0 onward).

## Cross-references

- `nnue-rework-plan.md` - this benchmark is the search-speed prerequisite for the NN-eval work (which absorbed the retired `nn-rater-plan.md`).
- `oq-81` - search branching factor + strategy plan. Source of the candidate optimisations list. Layer 1 (measurement) of oq-81 is partly subsumed by this benchmark.
- `core_engine/src/search/alpha_beta.rs` - the search loop under measurement.
- `core_engine/src/search/transposition.rs` - TT infrastructure (already exists).
- `core_engine/src/state/fen.rs` - `to_fen` / `from_fen` for corpus loading.
- `next_steps id=9` - debug harness / gamedbg CLI / scenario test runner. Some overlap with this work; the bench binary is a sibling, not a duplicate.
