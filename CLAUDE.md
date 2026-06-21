# (GAME NAME) — Board Game Design Project

## Project Overview

A 2-player abstract-tactical board game. Players command armies of Guards and Champions led by a King on a grid (Stack M default: 8×8). Victory: capture the enemy King. Design philosophy: depth through interlocking systems, no luck (perfect information, no dice, no hidden cards).

As of Session 27 (2026-06-22), the project is pivoting **digital-first**: a complete digital implementation with Stack M rules as default, replacing the paper-prototype pipeline as the primary iteration channel.

## Architecture

```
design/
  design.db                 ← SQLite source of truth (all design knowledge)
  schema.sql                ← 12-table schema, CHECK constraints, FKs, triggers
  README.md                 ← DB usage notes
  raw/                      ← Binary artefacts (photos, scans, card images)
    playtest-photos/
    brainstorm-scans/
    skill-card-images/
  inbox/                    ← Fast-write staging — promoted to DB by Claude
    brainstorm/             ← Raw idea dumps
    ai-chats/               ← Pasted chat transcripts (ChatGPT/Perplexity/etc)
    digital/                ← Architecture / UI / AI-opponent notes for game/

game/                       ← Digital implementation (architecture TBD; empty as of S27)
  README.md                 ← Status + open architecture questions

archive/
  old-game-versions/        ← v1/v2/v3 + archived stacks
  paper-pipeline/
    test-scenarios/         ← Typst rule sheets + PDFs (paper-prototype era)

.claude/
  STATUS.md                 ← One-screen re-entry doc (regenerated each session)
  HANDOVER.md               ← Session-to-session continuity notes
  migrate_*.py              ← One-shot migrators that built the DB (kept for audit)
  skills/                   ← Slash-command definitions
```

## The DB is the source of truth

Everything design-related — sessions, principles, open questions, ADRs, mechanics, stacks, playtests, the backpocket, next steps, essays, design docs, cross-references — lives in `design/design.db` as **markdown bodies in TEXT columns**. The migrator scripts deleted the source `.md` files in Session 27; the DB now owns those facts.

**12 tables:** `sessions`, `principles`, `open_questions`, `adrs`, `mechanics`, `stacks`, `playtests`, `backpocket`, `next_steps`, `essays`, `design_docs`, `links`.

### Working with the DB

Query patterns (use `Bash` tool with `sqlite3 design/design.db "..."`):

```sql
-- Status briefing (one-screen re-entry)
SELECT id, n, date, title FROM sessions ORDER BY n DESC LIMIT 5;
SELECT id, letter, name, status FROM stacks WHERE status IN ('active','queued') ORDER BY status, letter;
SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority;
SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority LIMIT 5;

-- Full body of a thing
SELECT body FROM stacks WHERE id='stack-m';
SELECT body FROM open_questions WHERE id='oq-66';
SELECT body FROM principles WHERE id='principle-8';
SELECT body FROM adrs WHERE id='adr-004';
SELECT body FROM essays WHERE id='essay-game-economy-map';

-- Cross-references (graph)
SELECT to_id, relation, note FROM links WHERE from_id='stack-m';
SELECT from_id, relation, note FROM links WHERE to_id='oq-11';

-- Recent narrative (reproduces SESSION_LOG.md)
SELECT body FROM sessions ORDER BY n DESC;
```

### Writing to the DB

Use `Bash` + `sqlite3` with `INSERT` / `UPDATE`. Examples:

```sql
-- New OQ
INSERT INTO open_questions (id, title, status, priority, body, created_in)
VALUES ('oq-72', 'Title', 'high', 2, 'Body…', 'session-27');

-- New session entry (at session end / /wrapup)
INSERT INTO sessions (id, n, date, title, body)
VALUES ('session-27', 27, '2026-06-22', 'Title', 'Body…');

-- Mark an OQ resolved
UPDATE open_questions SET status='resolved', resolved_in='session-27' WHERE id='oq-66';
```

The `updated_at` trigger fires on every UPDATE. CHECK constraints enforce valid enum values for `status`, `verdict`, `kind`, `category`, `relation`. `PRAGMA foreign_keys = ON` then `PRAGMA foreign_key_check` to validate.

The DB is committed as a binary blob (single workstation, no merge conflicts expected).

## Living docs (kept outside the DB)

- **`.claude/STATUS.md`** — one-screen re-entry doc. Rebuilt each session from DB summaries; not a source of truth itself.
- **`.claude/HANDOVER.md`** — session-to-session notes; what's loaded in working memory, what's next.

Both are *summaries pointing at the DB*. Never restate facts here — link by ID.

## Hard constraints (canonical IDs in DB `principles` table, kind='hard-constraint')

`constraint-perfect-info`, `constraint-grid-based`, `constraint-no-terrain`, `constraint-two-skill-slots`, `constraint-blank-champions`, `constraint-guards-late-game`. Read bodies for full text.

## Design lens

- **North star** (`principle-north-star`): "A small number of interlocking systems that generate surprising, meaningful decisions."
- **Core fantasy** (`principle-core-fantasy`): Discovering and executing clever spell/skill combos.
- **High-concept framing** (`principle-high-concept-framing`, ADR-004): "Two minds, one puzzle."
- **Chassis vs. Engine** (`principle-chassis-and-engine`) — diagnostic lens. Engine = skills/draft/actions; chassis = board/HP/economy. Cut chassis bloat to make the engine louder.
- **MDA** — apply to new mechanics.

## Justification Rule (MANDATORY)

Every new mechanic / rule / system change must justify its purpose: **what problem does this fix, or what specific game-feel improvement does this deliver?** "It sounds cool" is not enough. Apply when staging a `backpocket` entry, drafting a stack, or proposing a rule change.

## Incremental Testing Methodology (MANDATORY when core is settled)

One layer per playtest. Decompose, identify coupling, order from independent to dependent, document which stack produced which result. **Conditional override**: Principle 7 (Session 23) suspends this while core identity is unsettled — bundled stacks are allowed when justified (Stack M is the canonical example).

## Hygiene principles (Session 17 lessons — load-bearing)

1. **One source of truth per fact, with pointers — never restatements.** Now enforced by the DB: facts have IDs; other rows link by ID. Summaries must label themselves as such and point at the canonical row.
2. **State docs need lifecycle, not just append.** Resolved OQs carry `status='resolved'` and `resolved_in='session-N'`. Closed mechanics carry `verdict='rejected'/'withdrawn'/'superseded'`. Lifecycle lives in the DB enum columns, not in folder names.
3. **Cross-link by ID, don't restate verdicts.** Use the `links` table for any cross-reference. Never copy a verdict into a second row's body — link to it.
4. **Vocabulary renames must be project-wide and atomic.** `UPDATE` queries that touch every affected body, in one transaction. Then commit.
5. **Skills must reference real, current paths.** Slash-command skills point at DB queries now, not file paths. Verify before adding.
6. **Memory is for immutable facts, not current state.** Auto-memories should read as historical claims; current state lives in the DB.
7. **Templates that get copy-pasted should be functions instead.** (Carry-over from paper-pipeline; less load-bearing now that rule rendering is downstream.)
8. **Build scripts should discover, not enumerate.** Migrators in `.claude/migrate_*.py` are one-shot — discovery happens via SQL queries now.
9. **Justify before staging, archive before piling.** Backpocket entries carry `status` (active/parked/promoted/withdrawn). No graveyards.
10. **CLAUDE.md is for orientation, not facts.** This file. It tells you *where to look* (the DB). It does not restate game numbers, principles, or stack rules.

## Skills (Slash Commands)

| Command | Trigger | Description |
|---------|---------|-------------|
| `/start` | User only | Session start — reads STATUS.md + queries DB for current focus, presents briefing |
| `/wrapup` | User only | Session end — appends a `sessions` row, updates STATUS.md, HANDOVER.md, commits |
| `/research <topic>` | Auto or user | Perplexity research prompt with project context. Auto-triggers on knowledge gaps. |
| `/playtest <N>` | Auto or user | Transcribes playtest photos/scans into a `playtests` row + cross-links |
| `/scenario <stack-X> <desc>` | Auto or user | Stages a new `stacks` row + hypothesis. Auto-triggers when a discussion yields a testable change. |
| `/adr <topic>` | Auto or user | Stages a new `adrs` row. Auto-triggers when multiple valid approaches emerge. |

**Note**: Skill definitions in `.claude/skills/` were written for the paper-pipeline era and still reference deleted MD paths. They need a rewrite to query the DB instead. Until then, expect skills to fail on the old paths — fall back to direct DB queries.

## Stack M (current Active stack)

Stack M is the active design (Session 25-26). Six bundled simultaneous changes from the paper baseline:
1. Board 10×10 → 8×8.
2. Armor cap 3 → 2.
3. Injured penalties removed (HP-tracker only).
4. Draw conditions removed.
5. Steal cost 3 → 4.
6. Multi-Champion combo bonus widened to movement-causing skills (both trigger and bonus).

Full rule substance: `SELECT body FROM stacks WHERE id='stack-m';`. **Stack M's body is the foundation for the digital implementation in `game/`.**

P6 has not yet run as of Session 27. The digital-first pivot may absorb P6 into a digital playtest rather than a paper one.
