#!/usr/bin/env bash
# Run the canonical benchmark sweep and write JSON to game/bench/<prefix>-<budget>.json.
#
# Usage:
#   game/bench/run_sweep.sh <prefix>
#
# Example:
#   game/bench/run_sweep.sh baseline    # writes game/bench/baseline-{depth6,time100ms,time500ms,time1000ms,time3000ms}.json
#   game/bench/run_sweep.sh nmp         # writes game/bench/results/nmp-{...}.json
#
# Special-case: prefixes other than "baseline" go into game/bench/results/.
# Baselines live at the top level so they are hard to overwrite by accident.
#
# The script also runs the determinism check at the end and fails loudly if
# any position isn't reproducible. That is the smoke gate before any A/B result
# is trusted.

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <prefix>" >&2
  echo "  writes <prefix>-{depth6,time100ms,time500ms,time1000ms,time3000ms}.json" >&2
  exit 2
fi

PREFIX="$1"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CORPUS="$REPO_ROOT/game/bench/corpus/corpus.txt"

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

run_bench() {
  local flag="$1"  # e.g. "--depth 6" or "--time-ms 500"
  local tag="$2"   # e.g. "depth6" or "time500ms"
  local out="$OUT_DIR/${PREFIX}-${tag}.json"
  echo
  echo "=== $tag → $out ==="
  # Route stdout to a summary tail; keep stderr visible.
  "$BIN" --corpus "$CORPUS" $flag --out "$out" | tail -3
}

run_bench "--depth 6"     "depth6"
run_bench "--time-ms 100" "time100ms"
run_bench "--time-ms 500" "time500ms"
run_bench "--time-ms 1000" "time1000ms"
run_bench "--time-ms 3000" "time3000ms"

echo
echo "=== Determinism check ==="
if ! "$BIN" --determinism --corpus "$CORPUS" | tail -5; then
  echo "DETERMINISM FAILED" >&2
  exit 1
fi

echo
echo "All done. Files written under $OUT_DIR:"
ls -1 "$OUT_DIR"/${PREFIX}-*.json
