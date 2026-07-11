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
  inbox/                    ← Fast-write staging (single folder) — promoted to DB by Claude.
                              brainstorm-* ideas · chat-* AI transcripts · digital-* arch notes · playtest-*-notes.md
  RULES.md                  ← Canonical current ruleset (authoritative on conflict)
  _archive/                 ← Frozen historical material (Typst paper-pipeline rule sheets + PDFs)

game/                       ← Digital implementation (Rust core + Tauri frontend)
  README.md                 ← Status + open architecture questions

.claude/
  STATUS.md                 ← One-screen re-entry doc (regenerated each session)
  HANDOVER.md               ← Session-to-session continuity notes
  migrate_*.py              ← One-shot migrators that built the DB (kept for audit)
  skills/                   ← Slash-command definitions
```

## The DB is the source of truth

Everything design-related — sessions, principles, open questions, ADRs, mechanics, stacks, playtests, the backpocket, next steps, essays, design docs, cross-references — lives in `design/design.db` as **markdown bodies in TEXT columns**. The migrator scripts deleted the source `.md` files in Session 27; the DB now owns those facts.

**12 tables:** `sessions`, `principles`, `open_questions`, `adrs`, `mechanics`, `stacks`, `playtests`, `backpocket`, `next_steps`, `essays`, `design_docs`, `links`.

### Read the body. Never assume from the title. (MANDATORY)

The `title` / `name` columns are *index entries*, not facts. They are abbreviations whose meaning is in the `body`. **Before saying anything about any row, querying it again to "be sure," paraphrasing it back to the user, proposing a resolution, suggesting a change, or treating it as a blocker, you MUST `SELECT body FROM <table> WHERE id='<id>';` and read the body.**

This applies — without exception — to:
- Any OQ, ADR, principle, stack, mechanic, essay, or playtest the user references by ID or title.
- Any row that surfaces in `/start`'s briefing list (critical/high OQs, active stacks, top next_steps).
- Any row you're about to update, resolve, withdraw, or cross-link.

Common failure mode to avoid: an OQ title says "X Removal" and the body actually says "test whether removing X is justified" — those mean different things. A title says "Curve TBD" and the body actually points at the paper-baseline curve — those mean different things. A title says "X Balance" and the body has already absorbed the fix into a stack — those mean different things. **The title is a label. The body is the claim. Read the claim.**

If a user message is ambiguous about which row they mean, ask — don't guess from titles.

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
| `/scenario <desc>` | Auto or user | Parks a candidate design lever in the `backpocket` table (problem→solution→when-to-deploy). Auto-triggers when a discussion yields a testable change. *(The `stacks` "one stack per test" methodology is retired — see below.)* |
| `/adr <topic>` | Auto or user | Stages a new `adrs` row. Auto-triggers when multiple valid approaches emerge. |

**Note**: The `/start`, `/playtest`, `/scenario`, and `/research` skills were rewritten in Session 45 to query the DB / `design/RULES.md` and to write levers to `backpocket` (not `stacks`). The old paper-pipeline paths are gone (archived under `design/_archive/`).

## Git branching — HARD RULE (MANDATORY, NO EXCEPTIONS)

**NEVER create git branches. NEVER switch to a non-`main` branch to do work. All work happens on `main`.**

- Do **not** run `git branch <name>`, `git checkout -b <name>`, `git switch -c <name>`, `git worktree add`, or any command that creates a new branch — under any circumstance, for any reason, without a direct explicit request from the user naming the branch.
- Do **not** use the EnterWorktree tool (it creates a branch). Do not use `isolation: "worktree"` on agents.
- If a task "would be cleaner on a branch," it is still done on `main`. The user does not want branches.
- If branches or extra branches ever exist, the correct action is to get their commits onto `main` (fast-forward or merge) and **delete the branch**, then work on `main`.
- Committing directly to `main` is the expected, approved workflow. (Pushing still requires explicit approval — see below.)

Violating this rule is a serious error. The user has stated this in the strongest possible terms.

## Release / GitHub Actions

The build workflow (`.github/workflows/release.yml`) triggers on any `v*` tag push. To cut a release:

```bash
git push origin main        # push commits first
git tag v0.x.y              # tag the current commit
git push origin v0.x.y      # push tag — triggers the workflow
```

If you need to redo a tag (e.g. tag already exists): delete it locally (`git tag -d v0.x.y`), delete it on the remote (`git push origin :refs/tags/v0.x.y`), then recreate and push as above.

**IMPORTANT**: Never push commits or tags without explicit user approval. Always show the commands and confirm before running `git push`.

## Session Start — Read Current Rules First

At the start of any fresh session (before doing any design or implementation work), read the current full ruleset:

```
design/RULES.md
```

This is the **canonical ruleset — authoritative on conflict.** Read it in full before touching any design question or game code. Rules marked **⧗ Stack N — staged, awaiting P7** are implemented (or being implemented) in the engine but not yet playtest-confirmed. The in-game Help page (`help.rules.*` i18n) is a derived player-facing summary; RULES.md wins.

## Current ruleset & the retired stacks methodology

`design/RULES.md` is the single canonical ruleset. It is Stack M (game-length-cut baseline, provisionally landed P6) plus the three staged Stack N changes (Session 45). Full change rationale: `SELECT body FROM stacks WHERE id='stack-m';` and `… id='stack-n';`.

**The `stacks` "one stack per experiment" methodology is retired (Session 45).** We no longer mint `stack-a…stack-z` rows for every test. The 16 existing stack rows are **frozen for provenance** (linked to playtests/OQs — don't add to them, don't delete them). New candidate changes are parked as **levers in `backpocket`** (category `staged-fix`, status `parked`, with `fixes` + `trigger_cond`): "if problem X occurs in a playtest, here's candidate solution Y." Query parked levers: `SELECT id, name, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';`
