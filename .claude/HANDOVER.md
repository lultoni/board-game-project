# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — end of Session 27 (cleanup follow-on).*

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

### Where We Are (end of Session 27, 2026-06-22 — cleanup follow-on)

- **Digital-first pivot declared and structurally landed.** Deliverable in `game/` is a complete digital implementation of (GAME NAME) with Stack M rules as default. Rust core + multi-platform frontend (Desktop / Web / Mobile) + AI opponent + multiplayer. No code yet — the architecture ADR is the next anchor.
- **Repository fully restructured.** All design knowledge in `design/design.db` (12 tables, 366 rows, integrity verified). `game-state/` folded into `.claude/STATUS.md`. `digital-prototype/` deleted. `archive/` holds paper-pipeline and old game versions. `design/raw/` holds photos/scans/card images.
- **Three inboxes operational** for designer dumps: `design/inbox/brainstorm/` (game-design ideas), `design/inbox/ai-chats/` (pasted transcripts), `design/inbox/digital/` (architecture / UI / AI-opponent notes for `game/`).
- **All slash-command skills rewritten** to query/write the DB. `/build-pdfs` retired. Skills no longer reference deleted MD paths.
- **One-shot migrators archived** to `archive/migrators/`. No longer in working memory.
- **Stack M (Active) lives in the DB with full rule substance.** `SELECT body FROM stacks WHERE id='stack-m';` — that body is the Rust prototype's rule foundation. P6 has not yet run; the pivot may absorb P6 into a digital playtest instead of paper.

### Immediate Next Action

**Write the architecture ADR for `game/`.** Topic: Rust core + multi-platform frontend split + multiplayer transport + AI-opponent approach + save format. Inputs: Stack M rule substance from the DB; OQ-64 felt-PI considerations; P5's lost-game-log incident (digital persistence is mandatory); any prep notes the designer drops into `design/inbox/digital/`. Insert as `adr-005` once decided. After the ADR, scaffold the Rust workspace in `game/` and write the first failing test (board representation + Stack M setup).

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
| `design/inbox/brainstorm/` | Designer's fast-write game-design idea dumps |
| `design/inbox/ai-chats/` | Pasted chat transcripts |
| `design/inbox/digital/` | Architecture / UI / AI-opponent notes for `game/` |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
| `archive/migrators/` | One-shot Session-27 migrators (audit-only) |
| `archive/paper-pipeline/test-scenarios/` | Paper-era Typst rule sheets (read-only history) |

### Open methodological loose ends

None. Migrators archived, skills rewritten, STATUS/HANDOVER paths consistent, README current.
