#!/usr/bin/env bash
# Run the canonical benchmark sweep and write JSON to game/bench/<prefix>-<budget>.json.
#
# Usage:
#   game/bench/run_sweep.sh <prefix> [eval-id] [corpus-path] [--time-only]
#
# Example:
#   game/bench/run_sweep.sh baseline    # writes game/bench/baseline-{depth6,time100ms,time500ms,time1000ms,time3000ms}.json
#   game/bench/run_sweep.sh nmp         # writes game/bench/results/nmp-{...}.json
#
#   # Dual-evaluator cliff baselines for the search-cliff work (Phase 2):
#   game/bench/run_sweep.sh baseline-cliff-heur   heuristic   game/bench/corpus/cliff.txt --time-only
#   game/bench/run_sweep.sh baseline-cliff-custom custom-stub game/bench/corpus/cliff.txt --time-only
#
# Special-case: prefixes other than "baseline" go into game/bench/results/.
# Baselines live at the top level so they are hard to overwrite by accident.
#
# The optional 2nd arg is the evaluator id passed through as `--eval` (default
# "heuristic"). The search-cliff plan REQUIRES every fix to be A/B'd on BOTH
# `heuristic` and `custom-stub`; run this twice with distinct prefixes to get
# both baselines. The optional 3rd arg overrides the corpus (default: the main
# corpus.txt); pass corpus/cliff.txt to measure the depth-cliff position set.
#
# `--time-only` (any position after the prefix) skips the fixed-depth6 budget
# and runs the 4 time budgets only. Use it for the cliff corpus, where fixed
# depth6 on the ebf-25 king-danger positions blows to hundreds of millions of
# nodes (~10min/position) — the time budgets already capture the cliff via
# depth-reached, so the fixed-depth budget just makes every A/B slow.
#
# The script also runs the determinism check at the end and fails loudly if
# any position isn't reproducible. That is the smoke gate before any A/B result
# is trusted.

set -euo pipefail

# --time-only may appear anywhere; strip it out of the positional args first.
TIME_ONLY=0
POSITIONAL=()
for arg in "$@"; do
  if [[ "$arg" == "--time-only" ]]; then
    TIME_ONLY=1
  else
    POSITIONAL+=("$arg")
  fi
done
set -- "${POSITIONAL[@]}"

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 <prefix> [eval-id] [corpus-path] [--time-only]" >&2
  echo "  writes <prefix>-{depth6,time100ms,time500ms,time1000ms,time3000ms}.json" >&2
  echo "  eval-id defaults to 'heuristic'; corpus defaults to game/bench/corpus/corpus.txt" >&2
  echo "  --time-only skips the fixed-depth6 budget (use for the cliff corpus)" >&2
  exit 2
fi

PREFIX="$1"
EVAL_ID="${2:-heuristic}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CORPUS="${3:-$REPO_ROOT/game/bench/corpus/corpus.txt}"

if [[ "$PREFIX" == "baseline" ]]; then
  OUT_DIR="$REPO_ROOT/game/bench"
else
  OUT_DIR="$REPO_ROOT/game/bench/results"
fi
mkdir -p "$OUT_DIR"

cd "$REPO_ROOT"

echo "=== Building search_bench (release) ==="
cargo build --release -p search_bench --manifest-path game/Cargo.toml

BIN="$REPO_ROOT/game/target/release/search_bench"

echo "=== eval=$EVAL_ID  corpus=$CORPUS  prefix=$PREFIX  time_only=$TIME_ONLY ==="

run_bench() {
  local flag="$1"  # e.g. "--depth 6" or "--time-ms 500"
  local tag="$2"   # e.g. "depth6" or "time500ms"
  local out="$OUT_DIR/${PREFIX}-${tag}.json"
  echo
  echo "=== $tag → $out ==="
  # Route stdout to a summary tail; keep stderr visible.
  "$BIN" --corpus "$CORPUS" --eval "$EVAL_ID" $flag --out "$out" | tail -3
}

if [[ "$TIME_ONLY" -eq 0 ]]; then
  run_bench "--depth 6"     "depth6"
fi
run_bench "--time-ms 100" "time100ms"
run_bench "--time-ms 500" "time500ms"
run_bench "--time-ms 1000" "time1000ms"
run_bench "--time-ms 3000" "time3000ms"

echo
echo "=== Determinism check (eval=$EVAL_ID) ==="
if ! "$BIN" --determinism --corpus "$CORPUS" --eval "$EVAL_ID" | tail -5; then
  echo "DETERMINISM FAILED" >&2
  exit 1
fi

echo
echo "All done. Files written under $OUT_DIR:"
ls -1 "$OUT_DIR"/${PREFIX}-*.json
