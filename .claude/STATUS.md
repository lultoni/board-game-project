# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-26 — Session 35 end (NN-rater scoping + search-speed benchmark + AB optimisation catalogue).*

---

## Current focus

**Search-speed pass.** S35 produced three linked plan documents in `design/inbox/digital/` scoping the next work tranche (search-speed first, NN evaluator second). No code changes this session. Next action is benchmark scaffolding.

## Active stack

**Stack M — Game Length Cut.** Engine and digital UI are Stack M-shaped. Still awaiting a real playtest. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

Three companion plans written to `design/inbox/digital/` (not yet promoted to DB rows):

1. **`nn-rater-plan.md`** — full NN-rater scope. Path 3 (gradient descent) + perturbation injection. Two-tier gauntlet (best-of-three at 100/300/500 ms, mirrored loadouts, three champion tracks). Native-only training crate, rayon-parallel. Opt-in observability UI via local-file polling. No blocking ADR.
2. **`search-speed-benchmark-plan.md`** — benchmark infrastructure. FEN-corpus driven, two modes (fixed depth + fixed time), doubles as correctness regression test, manual-run-only.
3. **`alpha-beta-optimisation-catalogue.md`** — web-search synthesised catalogue (chessprogramming wiki + Stockfish + amateur engines). 9 categories, each technique annotated with expected Elo / complexity / our-game-specific adaptations. Flags `EndPhase ≠ null-move`, no chess-SEE port, QS loud/quiet redefinition.

`next_steps id=25` body appended with a pointer to the three docs.

## Immediate next action

Begin search-speed work per `search-speed-benchmark-plan.md` §11:
1. Scaffold the bench binary (native, FEN-driven, structured output).
2. Build the 20-50-position FEN corpus including ≥1 known-result tactical position.
3. Verify determinism.
4. Generate initial baseline.
5. Land optimisations one at a time per the catalogue ordering (PVS → TT-move → aspiration → killers+history → LMR → ...).

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical + 10 high. Unchanged this session.

## Open methodological loose ends

- **oq-69** (Skill-Phase action progression curve) — resolved in code as `2 + (round_number-1)/10`. OQ row may still be marked open; verify and resolve.
- **oq-70** (Focus on Move-skills) — encoding shipped; verify OQ status against current code.

## DB sanity

`PRAGMA integrity_check` → ok. Pre-existing dangling FKs in `open_questions` rows 10, 86, 87 (created_in pointing at non-existent sessions); not introduced this session.
