#!/usr/bin/env python3
"""Insert Session 27 row capturing the digital-first pivot + DB migration."""

import sqlite3
from pathlib import Path

DB = Path("/Users/I750319/passion-projects/board-game-project/design/design.db")

BODY = """*Digital-first pivot. Repository restructured. All design knowledge migrated into `design/design.db`.*

## The pivot

During the holiday, the designer decided that a "rumschieb simulator" (push-things-around simulator) is not enough. The deliverable in `game/` is now a **complete digital implementation of (GAME NAME)** with Stack M rules as the default. Multiplayer, AI opponent, multi-platform frontend (Desktop / Web / Mobile). The paper pipeline becomes a read-only archive; iteration moves to the digital layer.

Designer's directive (verbatim, German): *"ich will mehr mich in den digitalen prototypen lehnen. ich habe mich dazu entschieden im verlauf von meinem urlaub, dass ein 'rumschieb somulator' es nicht cuttet sondern ich eine komplette digitale umsetzung von meinem spiel haben will (aber die neu gemachten regeln von stack m als default)."*

## DB migration (Step 4)

All twelve tables populated. 365 rows total:

| Table | Rows | Source |
|---|---|---|
| sessions | 26 | `SESSION_LOG.md` (joint logs 8-9 / 10-12 split per-session) |
| principles | 22 | `design-principles.md` (2 north-stars, 4 lenses, P1-P8, 6 hard constraints, 4 methodology) |
| open_questions | 71 | `OPEN_QUESTIONS.md` + `OPEN_QUESTIONS_ARCHIVE.md` (status mapped from section headers) |
| adrs | 4 | Synthesised from cross-references (originals deleted Session 17) |
| mechanics | 71 | `mechanics-evaluated.md` (24 baseline + 7 staged + 12 pending + 20 rejected + 2 withdrawn + 6 accepted methodology) |
| stacks | 15 | `TESTING_PLAN.typ` + per-stack `.typ` files (Stack M with full rule substance) |
| playtests | 5 | P1-P5 summaries; full analyses in `essays` |
| backpocket | 52 | `backpocket.md` (one row per `## ` section) |
| next_steps | 7 | `NEXT_STEPS.md` Priority sections |
| essays | 21 | `docs/research/*.md` full bodies |
| design_docs | 2 | `game-identity-visual-naming.md` + `systems-and-mechanics.md` |
| links | 69 | Hand-curated cross-references (addresses 28, evidence-for 16, related-to 10, parent-of 9, absorbed-into 3, derived-from 2, supersedes 1) |

Integrity verified: `PRAGMA foreign_key_check` returns no rows; `PRAGMA integrity_check = ok`. Migrators kept in `.claude/migrate_*.py` for audit.

## Repository restructure (Steps 5-7)

- **Raw assets → `design/raw/`**: `playtest-results/` → `design/raw/playtest-photos/`; `images/` → `design/raw/skill-card-images/`; `docs/research/session-26-brainstorm-scans/` → `design/raw/brainstorm-scans/`.
- **Archives → `archive/`**: `old-game-versions/` → `archive/old-game-versions/`; `docs/test-scenarios/` → `archive/paper-pipeline/test-scenarios/`.
- **Cleanup**: deleted all migrated MDs (`docs/backpocket.md`, `docs/design-principles.md`, `docs/game-identity-visual-naming.md`, `docs/systems-and-mechanics.md`, `docs/mechanics-log/`, all `docs/research/*.md`, `game-state/NEXT_STEPS.md`, `game-state/OPEN_QUESTIONS.md`, `game-state/OPEN_QUESTIONS_ARCHIVE.md`, `game-state/SESSION_LOG.md`, top-level `WHAT_TO_PRINT.md`). `docs/` removed entirely.
- **Inboxes added**: `design/inbox/brainstorm/` and `design/inbox/ai-chats/` with READMEs explaining the fast-write → DB-distill flow. These are the staging channels for ad-hoc thinking; mined into DB tables at session start.

## CLAUDE.md rewrite (Step 9)

`CLAUDE.md` rewritten as an orientation pointer-document. Key changes:
- Architecture map updated for new folder layout.
- "The DB is the source of truth" section with example SQL queries (read + write patterns).
- Hard constraints and design lens reference principles by ID, no longer restate them.
- Hygiene-principles section retained but updated (e.g. "one source of truth per fact" is now enforced by DB primary keys).
- Skill-table updated; flagged that `.claude/skills/*` still point at deleted MD paths and need rewrite.
- Stack M section closes the file as the rule foundation for `game/`.

## game/ placeholder (Step 8)

`game/README.md` written. Status: empty. Open architecture questions enumerated (frontend split, multiplayer transport, AI opponent approach, save format). Rule source explicitly pointed at `SELECT body FROM stacks WHERE id='stack-m';`. Explicit non-goals: not a rumschieb-simulator, rules don't live in the folder, design discussion goes to `design/inbox/`.

## What's deferred to Session 28

- **Architecture ADR for `game/`** — Rust core + frontend split. Insert as `adr-005`.
- **Rewrite `.claude/skills/*` to query the DB** instead of reading deleted MD paths. `/start`, `/wrapup`, `/research`, `/playtest`, `/scenario`, `/adr` all need updating.
- **Move migrators to `archive/migrators/`** once we're confident the DB is correct (one or two sessions of operation should be enough).
- **First Rust scaffolding** in `game/` (workspace + first failing test for Stack M board representation).

## Carry-forward state

- Stack M Active in the DB. P6 has not run; may be absorbed into the digital prototype rather than a paper playtest.
- BASELINE_VERSION (paper baseline) frozen at 2026-05-30 — the Typst files in `archive/paper-pipeline/` are now historical, not living rules.
- All principles, hard constraints, ADRs, and methodology rules unchanged — only the storage and access pattern changed."""

conn = sqlite3.connect(DB)
cur = conn.cursor()
cur.execute(
    "INSERT INTO sessions (id, n, date, title, body) VALUES (?, ?, ?, ?, ?)",
    ("session-27", 27, "2026-06-22", "Digital-first pivot + repository restructure", BODY),
)
conn.commit()
print("Inserted session-27.")
cur.execute("SELECT n, date, title FROM sessions ORDER BY n DESC LIMIT 3;")
for r in cur.fetchall():
    print(f"  n={r[0]} {r[1]} {r[2]}")
conn.close()
