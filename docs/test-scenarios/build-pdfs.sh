#!/usr/bin/env zsh
# build-pdfs.sh
# Compiles all .typ files in the per-stack subfolders to their sibling PDFs.
# Usage: zsh docs/test-scenarios/build-pdfs.sh

set -e
SCRIPT_DIR="${0:A:h}"
PROJECT_ROOT="${SCRIPT_DIR:h:h}"

compile() {
  local src="$1"
  local dir="${src:h}"
  local base="${src:t:r}"
  local out="$dir/${base}.pdf"
  typst compile --root "$PROJECT_ROOT" "$src" "$out"
  echo "Built: $out"
}

# Shared components
compile "$SCRIPT_DIR/shared/game-tracking.typ"
compile "$SCRIPT_DIR/shared/feedback-baseline.typ"

# Testing plan + decision tree
compile "$SCRIPT_DIR/TESTING_PLAN.typ"

# Canonical baseline
compile "$SCRIPT_DIR/baseline/ruleset-baseline.typ"

# Accepted Layer 1 — economy fix
compile "$SCRIPT_DIR/accepted-layer-1-economy/layer-1-economy-fix.typ"
compile "$SCRIPT_DIR/accepted-layer-1-economy/layer-1-feedback.typ"

# Stack A — Cleverness (attack nerf + combo bonus, two-game format)
compile "$SCRIPT_DIR/stack-a-cleverness/stack-a-game1-attack-nerf.typ"
compile "$SCRIPT_DIR/stack-a-cleverness/stack-a-game2-attack-nerf-combo.typ"
compile "$SCRIPT_DIR/stack-a-cleverness/stack-a-feedback.typ"

# Stack B — Guards (bodyguard fix)
compile "$SCRIPT_DIR/stack-b-guards/stack-b-bodyguard-fix.typ"
compile "$SCRIPT_DIR/stack-b-guards/stack-b-feedback.typ"

# Stack G — Structure (unified AP framework)
compile "$SCRIPT_DIR/stack-g-structure/stack-g-unified-ap.typ"

echo "Done — all PDFs built."
