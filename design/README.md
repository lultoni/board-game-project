# design/

This directory is the design knowledge base for the board game project. **SQLite is the source of truth.**

Markdown files that previously lived under `docs/`, `game-state/`, and `playtest-results/` were migrated into `design.db` in Session 27 and deleted (`game-state/` is gone; its sole remaining file, `STATUS.md`, now lives at `.claude/STATUS.md`). Do not re-create those Markdown files — write to the DB instead.

## Layout

```
design.db        SQLite database. Source of truth for all design knowledge.
schema.sql       Schema definition. Versioned via git history.
README.md        This file.
raw/             Non-text artefacts referenced from DB rows.
  playtest-photos/    Per-playtest photos + scans.
  brainstorm-scans/   Per-session brainstorm photos.
  skill-card-images/  Skill card source images.
inbox/           Fast-write staging — Claude mines into DB rows.
  brainstorm/         Raw game-design idea dumps.
  ai-chats/           Pasted chat transcripts (ChatGPT/Perplexity/etc).
  digital/            Architecture / UI / AI-opponent notes for `game/`.
```

## ID conventions

All record IDs are lowercase, hyphen-separated:

- `oq-42`, `oq-1b`
- `stack-m`, `stack-a-g3`
- `adr-004`
- `session-26`
- `playtest-5`
- `mech-<slug>`
- `bp-<slug>`
- `principle-4`, `constraint-perfect-info`
- `essay-<slug>`
- `design-doc-<slug>`

Dates are ISO-8601 text (`2026-06-21`).

## Tables

| Table | Purpose |
|-------|---------|
| `sessions` | Per-session narrative log. FK anchor for most other tables. |
| `open_questions` | Live and archived OQs in one table, separated by `status`. |
| `stacks` | Test scenarios (Active / Queued / Dormant / Resolved / Withdrawn / Absorbed / Archived). |
| `mechanics` | Decision registry — every mechanic evaluated. |
| `adrs` | Architecture Decision Records. |
| `playtests` | Playtests 1–5 (and onward). Body holds the analysis. |
| `backpocket` | Staged fixes, candidate skills, guardrails, to-discuss items. |
| `principles` | Design principles + hard constraints + lenses (Chassis/Engine, etc.). |
| `next_steps` | Prioritised action items, optionally anchored to an OQ or stack. |
| `essays` | Closed research / analyses. Pure prose, no live status. |
| `design_docs` | Living design directives (visual identity, frontend, onboarding...). |
| `links` | Generic cross-references. `(from_id, to_id, relation)`. |

## Cross-references

All cross-refs go through `links`. Constrained `relation` values:

```
addresses        stack addresses an OQ
absorbed-into    stack/OQ absorbed into another
supersedes       record supersedes another
related-to       generic see-also
evidence-for     playtest provides evidence for OQ/stack
derived-from     record derived from another
connected-to     weak association
promoted-to      backpocket promoted to stack/oq/etc.
opened-by        OQ opened by a playtest/session
resolved-by      OQ resolved by a stack/adr/mechanic
blocks           one item blocks another
parent-of        hierarchical (e.g. oq-1 → oq-1b)
```

Add a new relation type only by editing the CHECK constraint in `schema.sql`.

## Common queries

Open OQs by priority:

```sh
sqlite3 design/design.db \
  "SELECT id, title, status FROM open_questions
   WHERE status IN ('critical','high','medium','deferred','open','watch')
   ORDER BY status, priority;"
```

Active stack:

```sh
sqlite3 design/design.db \
  "SELECT id, name, hypothesis FROM stacks WHERE status='active';"
```

What does Stack M address?

```sh
sqlite3 design/design.db \
  "SELECT to_id, relation FROM links WHERE from_id='stack-m';"
```

What references OQ-66?

```sh
sqlite3 design/design.db \
  "SELECT from_id, relation FROM links WHERE to_id='oq-66';"
```

Recent sessions:

```sh
sqlite3 design/design.db \
  "SELECT n, date, title FROM sessions ORDER BY n DESC LIMIT 5;"
```

Full body of a record:

```sh
sqlite3 design/design.db \
  "SELECT body FROM open_questions WHERE id='oq-66';"
```

## Adding a record

Inserts use the standard `sqlite3` CLI. Required fields per table are enforced by `NOT NULL`. `status`/`kind`/`category`/`verdict`/`relation` columns have CHECK constraints — invalid values throw errors.

Example — open a new OQ:

```sh
sqlite3 design/design.db <<'SQL'
INSERT INTO open_questions (id, title, status, priority, affected_systems, body, created_in)
VALUES (
  'oq-69',
  'Repo restructure for digital-first',
  'open',
  1,
  '["meta","tooling"]',
  '# OQ-69\n\nFull markdown body here...',
  'session-27'
);
SQL
```

After inserting, add cross-refs:

```sh
sqlite3 design/design.db \
  "INSERT INTO links (from_id, to_id, relation) VALUES ('oq-69', 'session-27', 'opened-by');"
```

## Updating a record

Update via `UPDATE`. The `updated_at` trigger fires automatically.

```sh
sqlite3 design/design.db \
  "UPDATE open_questions SET status='resolved', resolved_in='session-28'
   WHERE id='oq-69';"
```

## Integrity checks

After any non-trivial change:

```sh
sqlite3 design/design.db "PRAGMA foreign_key_check;"
sqlite3 design/design.db "PRAGMA integrity_check;"
```

## Raw artefacts

Photos, scans, and skill-card images live under `raw/`. DB rows reference them by relative path in fields like `playtests.raw_artefacts_path`. Files are git-tracked; never delete a raw artefact without first nulling the reference.
