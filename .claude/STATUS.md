# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-11 — Session 46 end (ns-43 evaluator: 4 new terms + stage infra + per-position decider).*

---

## Current focus

**AI evaluator overhaul (ns-43) landed; Stack N engine rules (ns-42) landed.** The eval is now a registry of parameterised, per-position-gated terms. Two clear wins (guard_isolation, champion movement-space); two correct-but-untuned terms (champion_threat, endgame_closing). Next real levers are the offline tuner and the search-depth investigation. **P7 (playtest Stack N) is still the outstanding design gate** — engine is ready for it.

## Current ruleset

**`design/RULES.md`** is canonical (authoritative on conflict; Help page is the derived player summary). Stack M + three **⧗ Stack N — staged, awaiting P7** rules (Focus cost 2, max 1 move-attack/turn, strike-moves-caster) — these are now **implemented in the engine** (ns-42, `e3d4d8c`) but still playtest-unconfirmed.

## Parked levers under watch

`sqlite3 design/design.db "SELECT id, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';"` — forward-Guard skill-immunity (Stack N reserve), dual-counter combo (oq-58), piece-count reduction (oq-27/66). *(Stack methodology retired; new candidates park in backpocket via `/scenario`.)*

## What changed this session (46)

- **ns-42 done** — Stack N's 3 rules implemented in `core_engine` (prior-context commit `e3d4d8c`).
- **ns-43 done (core)** — evaluator term-registry (`game/crates/core_engine/src/search/evaluator/`) gained 4 terms + stage infra + a per-position activation decider, one commit each, golden+determinism-guarded:
  - `guard_isolation` (**56.2%** self-play) · champion `mobility`→movement-space (**56.2%**) · `champion_threat` (52.5%, tuning candidate) · `endgame_closing` (asymmetric, End-stage-gated; A/B uninformative — never reached End in 60ms self-play) · `wasted_modifier` (blind-spot fix, self-play noise).
  - Stage infra: `EvalContext.advantage` + `GameStage` (Opening/Mid/End). Per-position decider = the pre-existing `is_active` hook + behavior-preserving skips on the costly terms.
- **New todos:** ns-44 (offline eval-param tuner), ns-45 (magic-bitboard `skill_attacks`), ns-46 (search-depth futility investigation).

## Immediate next action

1. **`main` is 21 commits ahead of `origin/main`, unpushed** — pushing needs explicit designer OK. (Big batch: Stack N engine + entire ns-43 eval overhaul.)
2. **P7** — playtest Stack N (does the standoff dissolve without re-inflating game length?). Engine now runs the rules.
3. **ns-46** (search-depth futility) — flagged as possibly the highest-leverage AI fix; reproduce + profile before touching eval further.
4. **ns-44** (eval tuner) — champion_threat weights + endgame_closing stage threshold are the measured under-tuned params.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (eval work touched no OQs).

## Open loose ends

- **`main` ahead of `origin/main`** — 21 commits unpushed (needs designer approval).
- v0.1.0 cross-platform release smoke test still outstanding (ns-28/ns-29).
- `champion_threat` + `endgame_closing` are correct but under-tuned (ns-44); `endgame_closing` unproven in self-play (never reached End stage).
- ns-35 (help/tutorial + UI unification), ns-39/40/41 deferred.

## DB sanity

Session 46 row inserted; ns-42/43 marked done; ns-44/45/46 added + cross-linked to ns-43. `integrity_check` → ok, `foreign_key_check` → clean.
