# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-24 — Session 32 end (match HUD export + in-game Sandbox mode shipped).*

---

## Instructions for Claude: How to Maintain This Handover Prompt

**When to update**: At the end of every session (or when the user says "wrap up"), update this file with:
1. Current session number and date.
2. 2-3 sentence summary of what was accomplished.
3. Current "Where We Are" section (overwrite, don't append).
4. Current "Immediate Next Action".

**What NOT to put here**: Full design details, rule text, or long explanations. This is a pointer document — it tells you WHERE to look, not WHAT the answers are. Keep it under 80 lines.

---

## The Prompt

You are my board game design co-creator and systems architect. We are working on a 2-player tactical board game (working title: "(GAME NAME)") inside this repository.

### How to start this session

1. Read `CLAUDE.md` (orientation; tells you the DB owns the facts).
2. Read `.claude/STATUS.md` (one-screen re-entry doc).
3. Query the DB for current focus — example one-liners in CLAUDE.md "Working with the DB" section.
4. Check `design/inbox/brainstorm/`, `design/inbox/ai-chats/`, and `design/inbox/digital/` for new dumps from the designer. Mine load-bearing content into the DB.
5. Check `design/raw/playtest-photos/` for any new playtest folders since last session.

### Where We Are (Session 32 end, 2026-06-24)

- **Engine complete for Stack M** (locked in at S32 start; 297 lib tests green).
- **Inspector core shipped (S31)** with four entry points (paste MatchLog, paste FEN, restore tree JSON, fresh-draft via `/setup/`), tree-of-positions, POI bookmarks, "Play this position" handoff, AI hint with iterative deepening.
- **S32 shipped match-HUD surface improvements**: Copy FEN / Copy MatchLog / Download MatchLog buttons, plus in-game **Sandbox mode** (toggle on `/match/`; pulsing blue inset border; capture-then-restore via engine snapshot; confirm-before-discard on exit). The old `/match/` → `/inspector/` "Open in Inspector" button is deleted; `/inspector/` remains a standalone analysis tool reachable via paste/restore entry points only.
- **Centralised action labelling** — new `lib/engine/action-label.ts` is the single source of truth across inspector picker, `MoveListItem`, and `AiHintBanner`.
- **Remaining digital work is frontend-only.** Eight follow-up `next_steps` rows (priorities 20–28) cover inspector slices, frontend polish, undo/redo, i18n, a11y, plus one engine optimisation (exact-depth AI search).

### Immediate Next Action

**Frontend follow-up — highest leverage:** `next_steps` id=12 (Inspector L6.7d — preview window primitive). Unblocks both L6.7b (top-K AI candidates) and L6.8 (skill tooltips). Full body: `sqlite3 design/design.db "SELECT body FROM next_steps WHERE id=12;"`.

Other queued frontend rows: 11, 13–16, 27, 28. Plus row 26 (engine exact-depth AI optimisation) for when frontend has a lull.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be marked open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding lives in engine Focus + Move-skill resolvers; verify OQ status against current code.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id=12;` | Inspector L6.7d preview window primitive (highest-leverage next item) |
| `SELECT body FROM next_steps WHERE id=15;` | Inspector polish (draft handoff + radial wheel; "Open in Inspector" sub-item now obsolete) |
| `SELECT id, priority, title FROM next_steps WHERE priority >= 20 ORDER BY priority;` | All inspector/frontend follow-ups |
| `SELECT body FROM sessions WHERE id='session-32';` | This session's narrative |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/session.rs` | Match API (incl. `request_ai_move_forced` / `_at_depth`) |
| `game/crates/core_engine/src/game_logic/make_unmake.rs` | Skill resolvers + Move-kind apply/unmake |
| `game/crates/core_engine/src/game_logic/generator.rs` | Legal-action enumeration |
| `game/frontend/src/routes/match/+page.svelte` | Match route — now hosts export + Sandbox mode |
| `game/frontend/src/routes/inspector/+page.svelte` | Inspector route (paste/restore entry points + tree + board + AI hint) |
| `game/frontend/src/lib/engine/action-label.ts` | Single source of truth for action labels |
| `game/frontend/src/lib/state/match-store.svelte.ts` | Match-level reactive state (incl. sandbox fields) |
| `game/frontend/src/lib/state/inspector-store.svelte.ts` | Tree-of-positions data model |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
