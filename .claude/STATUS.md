# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-24 — end of Session 31 (Inspector core ships).*

---

## Current focus

**Digital implementation runs on two parallel tracks.**

- **Engine track** — paused at end of Slice 5 (last advanced in S30). Next: Slice 6 (Focus + Charge wiring, end-of-turn clearance, Skill-Phase action curve, Zobrist).
- **Frontend track** — shipped the interactive Inspector core (`/inspector/`) this session. Four entry points, tree-of-positions, POI bookmarks, AI hint with proper iterative-deepening "Search continuously."

The frontend already works against engine Slice 5 (Stack M is fully playable end-to-end in the browser). The two tracks are independently advanceable; pick the track that matches the next sitting.

## Active stack

**Stack M — Game Length Cut.** The engine implements Stack M's body as written. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

1. **`/inspector/` route** — tree-of-positions, four entry points (paste MatchLog, paste FEN, restore tree JSON, fresh draft via `/setup/`), POI bookmarks, "Play this position" handoff back to `/setup/`. Click-to-move during Move phase; legal-action list panel for Skill phase (full radial wheel not yet extracted from the match page).
2. **AI hint** — added `request_ai_move_forced` (HvH-friendly; ignores seat kind) and `request_ai_move_at_depth` (iterative deepening with no time bound, caller drives the loop). Threaded through wrapper_api → WASM + Tauri → frontend `EngineClient` → worker + both clients. Inspector "Search continuously" climbs depth until cancelled, bails on forced mate.
3. **Three bugs fixed** mid-slice: `configJson` null deref via `defaultConfigJson()` fallback, a stray line comment rendering as template text in `MoveListItem.svelte`, and `NotAiTurn` error on HvH positions (fixed by the `_forced` variant).
4. **Nine deferred items captured in `next_steps`** (priorities 20–28): L6.7b top-K, L6.7d preview window, L6.7c search-tree viewer, L6.8 skill tooltips, inspector polish (match HUD button + draft handoff + skill-phase wheel), inspector i18n, engine exact-depth AI, L6.9 (undo/redo + menu), L6.10 (polish + i18n + a11y).

## Immediate next action

**Two equally valid options — pick at session start.**

- **Engine Slice 6** — Focus + Charge resolvers, end-of-turn clearance, Skill-Phase action curve, Zobrist. Full plan: `sqlite3 design/design.db "SELECT body FROM next_steps WHERE id='8';"`.
- **Inspector follow-up** — pick the highest-leverage row from `sqlite3 design/design.db "SELECT id, priority, title FROM next_steps WHERE id IN (11,12,13,14,15,16) ORDER BY priority;"`. L6.7d (preview window primitive) unblocks both L6.7b and L6.8.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 12 critical, ~8 high (unchanged this session; nothing new opened or resolved on the design side).

## Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Stack M says "starts at 2, scaling up." Still no curve; engine treats as constant 2 until resolved.
- **oq-70 — Focus on Move-skills.** Action-encoding decision pending; will surface concretely when Slice 6 wires Focus.

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok at session-31 end.
