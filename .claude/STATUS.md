# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-11 — Session 44 end (ns-37 sandbox/MP fix + settings pass + ns-35 help modal).*

---

## Current focus

**UI cleanup / UX gaps first**, then decide the next game change for the next playtest. Designer's sequencing: fix broken stuff and close UX gaps → *then* think about design changes. Not release-testing this cycle.

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped. P6 result: substantially met, no rollback. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session (44)

- **ns-37 DONE** (commit `0f12282`): sandbox-in-MP false anti-cheat "engine disagreed" fixed. Whenever the mp wrapper touches the shared engine with incoming real traffic it auto-exits sandbox first (reuses restore-to-true-line). Symmetric host/joiner; new `ensureLiveEngine` dep.
- **Settings pass** (commit `e2b2043`): `showThinkProgressBar` default OFF, `replayStepDelayMs` default 300, working language selector (reactive `i18n-locale.svelte.ts` rune), Run/Blessed evaluator picker removed from setup. "Adjust AI think-time" left unresolved (no value named) → ns-39.
- **ns-35 part A DONE** (commit `f276410`): in-game help modal — Help button next to Settings gear (global), Skills/Rules/Controls tabs, new `help.*` i18n in en+de. Part B (UI unification) **deferred by designer** → split to ns-38.

## Immediate next action

Designer to run `cargo tauri dev` (from `game/crates/tauri_wrapper`) and visually verify the help modal (placement, opens-over-board, live DE switch). Then, in priority order:
1. **ns-38** — unify duplicated UI components (skill-card primitive; panel/modal chrome). Deferred half of ns-35.
2. Then (separate design mode): pick the next game change for the next playtest — candidate levers: ns-32 (Focus 1→2), ns-34 / oq-58 first-mover, oq-86 (loser rebate).
3. **6 commits on `main` are unpushed** — decide whether to push (needs explicit designer OK).

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (design OQs, not touched during the UI/UX work).

## Open loose ends

- **`main` is 6 commits ahead of `origin/main`** — unpushed (pushing needs designer approval)
- ns-35 part A manual visual verification pending in the running app
- v0.1.0 cross-platform release smoke test — still outstanding from Session 40/42 (ns-28, ns-29)
- ns-39 (AI think-time default value — awaiting designer), ns-36 (QoL grab-bag), MP loadout fairness — deferred

## DB sanity

Session 44 row inserted. ns-37 → done; ns-35 body updated (part A done / B deferred); ns-38 + ns-39 created. `PRAGMA integrity_check` → ok, `foreign_key_check` clean.
