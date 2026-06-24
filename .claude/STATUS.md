# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-24 — Session 32 end (match HUD export + in-game Sandbox mode shipped).*

---

## Current focus

**Frontend follow-up.** Engine is complete for Stack M (locked in at S32 start; 297 lib tests green). The remaining digital work is Inspector polish, preview-window primitive, top-K candidates, search-tree viewer, skill tooltips, undo/redo, i18n, a11y. See `next_steps` rows 11–16, 27–28 (priorities 20–28).

## Active stack

**Stack M — Game Length Cut.** The engine implements Stack M's body as written. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

1. **Centralised action labelling.** New `lib/engine/action-label.ts` is the single source of truth for `formatAction(raw)` / `formatSquare(sq)` — handles Move-Attack approaches, Bodyguard redirects, Shove direction arrows, Focus retargets onto allies, Focus-effect 2-tile mode, generic `[focus]` tag. Three call sites consolidated (inspector picker, `MoveListItem`, `AiHintBanner`).
2. **Defensive fix for "tree has no configJson" runtime error.** `buildSnapshotForNode` now accepts a fallback config and repairs the tree in-place; `loadTree` fails fast on malformed paste.
3. **Match HUD export buttons.** Copy FEN, Copy MatchLog, Download MatchLog (timestamped `.json`). Round-trip into the inspector via its existing paste entry points. `matchLogAvailable` flag gates log buttons; transient 2.2s toasts for feedback.
4. **In-game Sandbox mode.** Toggle on `/match/` HUD; pulsing blue inset border via `main.sandbox-mode` class; all moves apply normally so AI hint / legal-action / animations keep working; exit captures-then-restores via `snapshotJson` / `restoreFromSnapshot` with confirm-before-discard. AI scheduler gated (both at the effect and inside `runAiStep`). Move counter increments in the central `applyRaw` wrapper. The "Open in Inspector" button was deleted — `/inspector/` stays a standalone analysis tool, `/match/` gets its own what-if affordance.
5. **`next_steps id=15` body updated.** The original "Open in Inspector button" sub-item is OBSOLETE — superseded by Sandbox + export buttons. Two sub-items still stand: fresh-draft handoff, and porting the radial wheel into the inspector.

## Immediate next action

**Frontend follow-up.** Highest leverage remains `next_steps id=12` (Inspector L6.7d — preview window primitive); unblocks both L6.7b (top-K candidates) and L6.8 (skill tooltips). Full body: `sqlite3 design/design.db "SELECT body FROM next_steps WHERE id=12;"`. Other queued: rows 11, 13–16, 26–28 (priorities 20–28).

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (nothing opened or resolved on the design side; all S32 work was on the digital surface).

## Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding lives in the engine via Focus + Move-skill resolvers; verify OQ status against current code.

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok at session-32 end.
