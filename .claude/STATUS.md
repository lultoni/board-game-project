# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-10 — Session 43 end (P6 analysis + B1/B2 fixes + idea-capture sweep).*

---

## Current focus

**Clean up currently bugged / broken UI things first**, then lock in and decide which game changes to make for the next playtest. Designer's explicit sequencing: fix the broken stuff → *then* think about design changes. Not release-testing this cycle.

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped. P6 result: substantially met, no rollback. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session (43)

- **P6 analysis landed** (`essay-playtest-6-analysis`, `playtest-6`): combo widening accepted, length just-right (17-26 rounds), no exchange-pit. New pattern = mid-game **first-mover-loses** standoff (oq-58, routed to tempo levers not draws).
- **B1 + B2 engine bugs fixed** on branch `fix/combo-bonus-and-preset-mirror` (NOT merged/pushed). Combo-bonus ruling (new champ +counter / returning +counter-1) — **combo is now done/closed**. Preset P2 loadout mirror (`mirror_loadout`/`mirrorLoadout`).
- **P6 idea-capture sweep** wrote the four missing items: ns-35 (help surface + UI unification), ns-37 (sandbox anti-cheat BUG), ns-36 (UI QoL grab-bag), oq-86 (loser-gets-money design question).

## Immediate next action

Work the UI cleanup, in priority order:
1. **ns-37** — fix the sandbox-in-MP false anti-cheat "engine disagreed" bug (isolate sandbox from the authoritative MP-validation engine).
2. **ns-35** — build the in-game help/reference surface + unify duplicated UI components.
3. Then (separate mode): decide the next game change for the next playtest — candidate levers: ns-32 (Focus 1→2), ns-34 / oq-58 first-mover, oq-86 (loser rebate).
4. Decide whether to merge/push branch `fix/combo-bonus-and-preset-mirror` (awaiting designer go-ahead).

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (design OQs, not touched during the UI/bug work).

## Open loose ends

- Branch `fix/combo-bonus-and-preset-mirror` — committed, not merged, not pushed
- v0.1.0 cross-platform release smoke test — still outstanding from Session 40/42
- A5 replay parity; ETA field null; MP loadout fairness — all still deferred

## DB sanity

Session 43 row inserted. ns-35/36/37 + oq-86 written earlier this session. `PRAGMA integrity_check` → ok.
