# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — end of Session 27.*

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
2. Read `game-state/STATUS.md` (one-screen re-entry doc).
3. Query the DB for current focus — example one-liners in CLAUDE.md "Working with the DB" section.
4. Check `design/inbox/brainstorm/` and `design/inbox/ai-chats/` for any new dumps from the designer. Mine load-bearing content into the DB.
5. Check `design/raw/playtest-photos/` for any new playtest folders since last session.

### Where We Are (end of Session 27, 2026-06-22)

- **Digital-first pivot declared.** During holiday, designer decided: a "rumschieb simulator" is not enough. The deliverable in `game/` is a complete digital implementation of (GAME NAME) with Stack M rules as default. Multiplayer + AI opponent + multi-platform (Desktop / Web / Mobile). No code written yet — the architecture ADR is the next-session anchor.
- **Repository restructure complete.** All design knowledge migrated into `design/design.db` (12 tables, 365 rows, integrity verified). Source `.md` files deleted from `docs/`, `game-state/` (except STATUS.md), and the top level (`WHAT_TO_PRINT.md` gone). `old-game-versions/` moved to `archive/old-game-versions/`. `docs/test-scenarios/` moved to `archive/paper-pipeline/test-scenarios/`. Raw photos/scans/card images moved to `design/raw/`. `playtest-results/` no longer exists as a top-level folder.
- **Inboxes staged for designer dumps.** `design/inbox/brainstorm/` and `design/inbox/ai-chats/` with READMEs explaining the fast-write → DB-distill flow.
- **Stack M (Active) lives in the DB with its full rule substance.** `SELECT body FROM stacks WHERE id='stack-m';` — that body is the Rust prototype's rule foundation. P6 has not yet run; the pivot may absorb P6 into a digital playtest instead of paper.
- **Skills in `.claude/skills/` still point at deleted paths.** They will fail. Rewrite required (next session, or as-needed when a skill is invoked).

### Immediate Next Action

**Write the architecture ADR for `game/`.** Topic: Rust core + multi-platform frontend split. Inputs: digital-first pivot brief (Session 27 user message); Stack M rule substance from the DB; OQ-64 felt-PI considerations; P5's lost-game-log incident (digital persistence is mandatory). Insert as `adr-005` once decided. After the ADR, scaffold the Rust workspace in `game/` and write the first failing test (board representation + Stack M setup).

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M full rule substance (foundation for `game/`) |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |
| `SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;` | Active todos |
| `SELECT body FROM sessions ORDER BY n DESC LIMIT 3;` | Last three session narratives |
| `SELECT id, letter, name, status FROM stacks WHERE status IN ('active','queued');` | Stacks in flight |
| `SELECT body FROM principles WHERE kind='principle' ORDER BY n;` | The eight numbered principles |
| `SELECT body FROM essays WHERE id='essay-game-economy-map';` | Structural justification for Stack M |
| `SELECT body FROM adrs ORDER BY n;` | All architecture decisions |
| `SELECT to_id, relation, note FROM links WHERE from_id='stack-m';` | What Stack M relates to |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/brainstorm/` | Designer's fast-write idea dumps |
| `design/inbox/ai-chats/` | Pasted chat transcripts |
| `game-state/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
| `archive/paper-pipeline/test-scenarios/` | Paper-era Typst rule sheets (read-only history) |

### Open methodological loose ends (carry into Session 28)

- Slash-command skills (`.claude/skills/*`) reference deleted MD paths and need a rewrite to query the DB.
- `STATUS.md` was kept but holds Session-26-era content; needs a Session-27 rewrite from DB queries.
- One-shot migrators (`.claude/migrate_*.py`) are kept for audit; could be moved to `archive/migrators/` once we're confident the DB is correct.
