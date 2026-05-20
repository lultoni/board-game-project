#!/usr/bin/env zsh
# build-pdfs.sh
# Discovers and compiles every .typ file under docs/test-scenarios/ to its
# sibling PDF, except library files in shared/ that are imported by others.
# Usage: zsh docs/test-scenarios/build-pdfs.sh

SCRIPT_DIR="${0:A:h}"
PROJECT_ROOT="${SCRIPT_DIR:h:h}"

# Library files in shared/ that are imported but not standalone documents.
# game-tracking.typ and feedback-baseline.typ remain compilable (they produce
# PDFs intentionally), so they are NOT in this exclusion list — only the
# pure-import libraries are.
typeset -a SKIP=(
  "$SCRIPT_DIR/shared/template.typ"
  "$SCRIPT_DIR/shared/baseline-sections.typ"
)

typeset -a SOURCES
SOURCES=("${(@f)$(find "$SCRIPT_DIR" -name '*.typ' -type f | sort)}")

typeset -a FAILURES
FAILURES=()

for src in "${SOURCES[@]}"; do
  # Skip library files
  if (( ${SKIP[(I)$src]} )); then
    continue
  fi

  dir="${src:h}"
  base="${src:t:r}"
  out="$dir/${base}.pdf"

  if typst compile --root "$PROJECT_ROOT" "$src" "$out"; then
    echo "Built: $out"
  else
    echo "FAILED: $src" >&2
    FAILURES+=("$src")
  fi
done

if (( ${#FAILURES[@]} > 0 )); then
  echo
  echo "=== ${#FAILURES[@]} build failure(s): ==="
  for f in "${FAILURES[@]}"; do echo "  - $f"; done
  exit 1
fi

echo "Done — all PDFs built."
