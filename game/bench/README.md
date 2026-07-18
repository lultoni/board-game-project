# search_bench - Search-Speed Benchmark

Manual-run-only tool that grades alpha-beta optimisations on a fixed FEN
corpus. See `game/plans/search-speed-benchmark-plan.md` for the design and
`game/plans/alpha-beta-optimisation-catalogue.md` for the optimisation queue.

## How to run

The canonical sweep runner is `game/bench/run_sweep.sh` - one command runs
the full 5-budget grid (`depth6` + `time100/500/1000/3000ms`) plus the
determinism smoke check:

```
game/bench/run_sweep.sh baseline    # → bench/baseline-<budget>.json (canonical)
game/bench/run_sweep.sh nmp         # → bench/results/nmp-<budget>.json (A/B)
```

The `baseline` prefix is special-cased to write to the top-level `bench/`
directory so accepted baselines are hard to overwrite by accident;
every other prefix writes to `bench/results/` which is gitignored scratch.

For ad-hoc single runs, invoke the binary directly:

```
cargo run --release -p search_bench --manifest-path game/Cargo.toml -- \
    --corpus game/bench/corpus/corpus.txt \
    --depth 6 \
    --out game/bench/results/run-depth6.json
```

Or for the standalone determinism check:
```
cargo run --release -p search_bench --manifest-path game/Cargo.toml -- \
    --determinism --corpus game/bench/corpus/corpus.txt
```

The binary exits with code 0 on success, 4 if any tactical position's
correctness assertion fails (a "REGRESSION: <id>" line precedes the exit),
3 if the determinism check fails, 2 on argument / IO errors.

## How to interpret

`baseline-depth6.json` is the canonical accepted baseline at fixed depth 6
(median of 5 runs). The matching multi-budget baselines are:
- `baseline-time100ms.json`
- `baseline-time500ms.json`
- `baseline-time1000ms.json`
- `baseline-time3000ms.json`

Each entry reports nodes, depth-reached, time, nodes-per-sec, TT hit rate,
EBF (effective branching factor = `nodes^(1/depth)`), best move, and score.

The aggregate block reports the geometric mean of nodes-per-sec across the
corpus - that single number is the headline "search throughput" metric, but
**at fixed-time budgets, depth-reached is the more meaningful AI-strength
signal**. NPS can fall as the search does more bookkeeping per node, yet
total time-to-depth still improve if the bookkeeping prunes enough nodes.

## How to update the baseline

When an optimisation lands and we accept it:

1. Run the full sweep with a candidate prefix: `game/bench/run_sweep.sh <name>`.
2. Diff the per-position results vs `baseline-<budget>.json`. Reject if any
   time budget shows positions reaching lower depth than baseline
   (regression in player-facing strength) or a score drift at fixed depth 6.
3. If accepted, re-run as `game/bench/run_sweep.sh baseline` to overwrite
   all five committed baselines in one shot.
4. The new baselines become the bar for the next optimisation.

## Test protocol - multi-budget sweep

Each candidate optimisation is graded against **every** budget, not just
one. Killers/history (Session 36) revealed that an optimisation can be
neutral or hurt at 1000ms while winning at 3000ms - testing one budget in
isolation would have rejected a real win. Session 41 (delta pruning) added
the further lesson that fixed-depth-6 alone hides score drifts that only
surface when you compare per-position and per-category, not aggregates.

`run_sweep.sh` runs the canonical order (depth6 → 100ms → 500ms → 1000ms
→ 3000ms → determinism). Accept iff:
- Zero positions reach lower depth at any budget,
- Total depth-6 wall-clock is non-positive,
- No score drifts or best-move disagreements at fixed depth 6,
- Determinism check passes.

## Corpus

`corpus/corpus.txt` is corpus v2 (Session 41, 2026-07-05): 30 positions
sampled from search-driven self-play at depths 2/3/4 (cycled), with
`MAX_PER_STM_PER_GAME=2` and view-key dedup. Categories:
opening-with-skills, midgame-move, skill-phase-full, combo-loaded,
endgame-with-skills, king-in-danger.

Corpus v1 (Session 37 and earlier) was random-play-generated and produced
eval-neutral positions; see `alpha-beta-optimisation-catalogue.md` Session
37 retrospective. Corpus v2 was regenerated once positional eval landed.

To regenerate raw samples:
```
cargo run --release --manifest-path game/Cargo.toml \
    -p core_engine --example build_corpus -- \
    --games 500 --seed 0xC0FFEE
```
…then hand-curate the output into `corpus.txt`.

## Storage layout

- `corpus/` - committed corpus and raw samples.
- `baseline-<budget>.json` - committed baselines (5 files).
- `results/` - gitignored scratch directory for A/B runs.
- `run_sweep.sh` - canonical sweep runner.
