# search_bench — Search-Speed Benchmark

Manual-run-only tool that grades alpha-beta optimisations on a fixed FEN
corpus. See `design/inbox/digital/search-speed-benchmark-plan.md` for the
design and `design/inbox/digital/alpha-beta-optimisation-catalogue.md` for
the optimisation queue.

## How to run

Build once:
```
cargo build --release -p search_bench
```

Fixed-depth mode (deterministic, ideal for compare-against-baseline):
```
cargo run --release -p search_bench -- \
    --corpus game/bench/corpus/corpus.txt \
    --mode depth --depth 6 --runs 5 \
    --out game/bench/results/run-depth6.json
```

Fixed-time mode (measures player-facing strength via depth-reached + EBF):
```
cargo run --release -p search_bench -- \
    --corpus game/bench/corpus/corpus.txt \
    --mode time --time-ms 1000 --runs 3 \
    --out game/bench/results/run-time1000ms.json
```

Determinism check (asserts identical nodes/score/best-move across 10 runs
per position at fixed depth):
```
cargo run --release -p search_bench -- \
    --corpus game/bench/corpus/corpus.txt \
    --determinism --depth 6 --determinism-runs 10
```

The binary exits with code 0 on success, 4 if any tactical position's
correctness assertion fails (a "REGRESSION: <id>" line precedes the exit),
3 if the determinism check fails, 2 on argument / IO errors.

## How to interpret

`baseline.json` is the canonical accepted baseline at fixed depth 6 (median
of 5 runs). The matching multi-budget baselines are:
- `baseline-time100ms.json`
- `baseline-time500ms.json`
- `baseline-time1000ms.json`
- `baseline-time3000ms.json`

Each entry reports nodes, depth-reached, time, nodes-per-sec, TT hit rate,
EBF (effective branching factor = `nodes^(1/depth)`), best move, and score.

The aggregate block reports the geometric mean of nodes-per-sec across the
corpus — that single number is the headline "search throughput" metric, but
**at fixed-time budgets, depth-reached is the more meaningful AI-strength
signal**. NPS can fall as the search does more bookkeeping per node, yet
total time-to-depth still improve if the bookkeeping prunes enough nodes.

## How to update the baseline

When an optimisation lands and we accept it:

1. Run the bench at fixed depth and at every committed time budget (the
   full multi-budget sweep — see "Test protocol" below).
2. Eyeball the per-position deltas vs the current baseline. Reject the
   change if any time budget shows positions reaching lower depth than
   baseline (regression in player-facing strength).
3. If accepted, overwrite all six `baseline*.json` files with the new
   run's JSON outputs.
4. The new baselines become the bar for the next optimisation.

## Test protocol — multi-budget sweep (Session 36+)

Each candidate optimisation is graded against **every** budget, not just
one. Killers/history (Session 36) revealed that an optimisation can be
neutral or hurt at 1000ms while winning at 3000ms — testing one budget in
isolation would have rejected a real win.

Per-optimisation sweep:
1. `--mode depth --depth 6 --runs 5` (deterministic node-count baseline).
2. `--mode time --time-ms 100 --runs 3`
3. `--mode time --time-ms 500 --runs 3`
4. `--mode time --time-ms 1000 --runs 3`
5. `--mode time --time-ms 3000 --runs 3`
6. `--determinism --depth 6 --determinism-runs 5` (sanity check).

Accept iff zero positions reach lower depth at any budget AND total
depth-6 wall-clock is non-positive.

## Corpus

`corpus/corpus.txt` is hand-curated from random self-play with the
`build_corpus` example. 20 entries spanning opening, midgame-low-skill,
phase-boundary, and endgame-sparse categories. The plan calls for at least
one tactical / known-result position; tracked as a TODO at the bottom of
the corpus file.

To regenerate raw samples:
```
cargo run --release -p core_engine --example build_corpus -- \
    --games 500 --seed 0xC0FFEE
```
…then hand-curate the output into `corpus.txt`.

## Storage layout

- `corpus/` — committed corpus and raw samples.
- `baseline.json`, `baseline-time1000ms.json` — committed baselines.
- `results/` — gitignored scratch directory for runs.
