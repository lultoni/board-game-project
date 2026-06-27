# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-27 — Session 36 end (Quiescence Search + head-to-head match reveals evaluator bottleneck).*

---

## Current focus

**Pivot to NN position-rater** per `design/inbox/digital/nn-rater-plan.md`. The search-speed pass surfaced a deeper finding: the current hand-eval has no positional terms and the engine plays 80+ consecutive EndPhases in self-play. Every search optimisation graded in S36 was graded against a non-playing engine and is therefore provisional. Eval is the bottleneck for both play strength and the search-optimisation grading protocol.

## Active stack

**Stack M — Game Length Cut.** Engine and digital UI are Stack M-shaped. Still awaiting a real playtest. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

- **QS module shipped** (`game/crates/core_engine/src/search/quiescence.rs`). Catalogue §3 minus SEE. Stand-pat + loud-action loop. MAX_QS_PLY=8. 380/380 tests pass.
- **Bitboard `is_king_threatened`** replaces generator-based first cut. Chebyshev + skill cost/range gating.
- **`DISABLE_QS` kill-switch** + **`qs_match` example** (head-to-head match harness). Lets us A/B without recompiling.
- **RFP and aspiration retested on top of QS** — both still failed sweep. Reverted.
- **3-game head-to-head match run** at 1000 ms/move: QS 1, base 0, 2 caps. 0 skills cast across 105 rounds. EndPhase-dominated.
- **`alpha-beta-optimisation-catalogue.md`** appended with Session 37 retrospective (catalogue numbering is +1 vs DB: catalogue "Session 36" = DB S35, catalogue "Session 37" = DB S36).

## Immediate next action

Begin NN position-rater work per `design/inbox/digital/nn-rater-plan.md`:
1. Native-only training crate (rayon-parallel).
2. Path 3 (gradient descent) + perturbation injection.
3. Two-tier gauntlet (best-of-three at 100/300/500 ms, mirrored loadouts, three champion tracks).
4. Opt-in observability UI via local-file polling.

Once eval supports real play, **re-run the full S36 sweep and the `qs_match` head-to-head harness** on top of the new evaluator. All S36 rejections (RFP, aspiration, PVS, LMP) are provisional pending that re-run.

## Banked wins from S36 (kept hooked)

- QS module — correct, ready to grade once eval supports real play.
- Bitboard `is_king_threatened` — load-bearing primitive.
- `DISABLE_QS` + `qs_match` — play-strength accept/reject path independent of corpus depth-reached.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical + 10 high. Unchanged this session.

## DB sanity

`PRAGMA integrity_check` → ok. Pre-existing dangling FKs in `open_questions` rows 10, 86, 87 (created_in pointing at non-existent sessions); not introduced this session.
