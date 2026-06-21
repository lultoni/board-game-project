# Restructure Plan — Session 27

*Working file for Claude during the Session 27 restructure. Steps are executed top-to-bottom. Each completed step gets marked `[x]`. Roll back via git if anything goes sideways.*

*Stop condition for Claude: after Step 3, surface result to user. After Step 4 (migration), surface result to user. After full plan, surface result to user. Otherwise proceed.*

---

## Decisions locked (Session 27)

- **DB-only**: Markdown design docs get migrated into SQLite; source MDs deleted. No auto-generated MD mirrors.
- **DB lives at**: `design/design.db`, committed.
- **Schema**: `design/schema.sql`. Versioning via git history, no `-v1` suffix.
- **Tooling**: `sqlite3` CLI direct for now. Mini-CLI later if needed.
- **IDs**: lowercase, hyphen-separated. `oq-42`, `stack-m`, `adr-004`, `session-26`, `playtest-5`.
- **Cross-refs**: one generic `links` table with CHECK constraint on `relation`.
- **Stack M paper artefacts** → `archive/paper-pipeline/` (historical). Stack M *rule substance* → `stacks.body` in DB. Baseline rules stay in archive — re-extract when Rust prototype is built.
- **`design_docs` is a new table** for living design directives (game-identity-visual-naming + future siblings). Separate from `essays` (closed research) and `principles` (rules).
- **Old prototype folder** `prototype/` already deleted by user.

---

## Target repo layout

```
.claude/                            <- unchanged
.gitignore, CLAUDE.md, README.md    <- stay at root
                                       (CLAUDE.md gets rewritten in Step 9)

design/                             <- design knowledge (DB-first)
  design.db                         <- SQLite, source of truth, committed
  schema.sql                        <- schema definition
  README.md                         <- how to query, table overview, ID conventions
  raw/                              <- non-text artefacts referenced from DB
    playtest-photos/
      playtest-1/                   <- ex elias-vs-pasco-31_10_25
      playtest-2/                   <- ex elias-vs-jonathan-24_04_26
      playtest-3/                   <- ex elias-vs-mario-17_05_26
      playtest-4/                   <- ex elias-vs-niko-28_05_26
      playtest-5/                   <- ex elias-vs-jonathan-pole-b-digital-2026-06
    brainstorm-scans/
      session-26/                   <- ex docs/research/session-26-brainstorm-scans/
    skill-card-images/              <- ex images/

game/                               <- digital prototype (Rust + multi-platform frontend)
  README.md                         <- placeholder until architecture ADR session

archive/                            <- frozen historical material
  paper-pipeline/                   <- everything from docs/test-scenarios/
  old-game-versions/                <- ex old-game-versions/

digital-prototype/                  <- temporary intake folder
  INTAKE (claude yappt iwas, was aber nicht meine notizen sind).md
                                    <- user's personal dump file (untouched)
                                       (folder will be renamed or absorbed in a later session;
                                        intentionally left in place this session)
```

Notes:
- The user's INTAKE file inside `digital-prototype/` stays put. Folder rename is a future discussion.
- `schema-v1.sql` moved out of `digital-prototype/` to `design/schema.sql`.

---

## Step 1 — Schema + design/ scaffolding

- [ ] Create `design/` directory.
- [ ] Write `design/schema.sql` (extends `digital-prototype/schema-v1.sql` with `essays` and `design_docs` tables; drop `-v1` suffix).
- [ ] Write `design/README.md` — table overview, ID conventions, common queries, how to add a record.
- [ ] Delete `digital-prototype/schema-v1.sql` (now redundant).

Schema additions vs. v1:

**`essays`** — closed research / analyses
```
id           TEXT PK              "essay-cognitive-load"
title        TEXT
topic        TEXT                  short tag: "cognitive-load", "playtest-analysis"
body         TEXT                  full markdown
date         TEXT                  best-known date
source_url   TEXT                  optional (Perplexity / YouTube etc.)
created_at, updated_at
```

**`design_docs`** — living design directives
```
id              TEXT PK            "design-doc-visual-identity"
title           TEXT
domain          TEXT               "visual", "frontend", "onboarding", "naming"
status          CHECK ('active','superseded','retired')
body            TEXT
established_in  TEXT               session-id, FK
created_at, updated_at
```

Both get `updated_at` triggers analogous to v1.

---

## Step 2 — Build the DB

- [ ] Run `sqlite3 design/design.db < design/schema.sql`.
- [ ] Sanity check: `sqlite3 design/design.db ".tables"` lists all 12 tables.
- [ ] Sanity check: `sqlite3 design/design.db ".schema links"` shows CHECK constraint.

---

## Step 3 — Directory scaffolding

- [ ] Create `design/raw/{playtest-photos,brainstorm-scans,skill-card-images}/`.
- [ ] Create `game/` + `game/README.md` placeholder.
- [ ] Create `archive/{paper-pipeline,old-game-versions}/`.

**→ Surface to user. Confirm structure before proceeding to migration.**

---

## Step 4 — Migration into DB (most fragile step)

Order is critical: tables with FKs must have referenced rows already inserted.

### 4a — `sessions` (FK anchor for almost everything)
- [ ] Parse `game-state/SESSION_LOG.md`. Each session block → one row.
- [ ] For each: `id = session-N`, `n`, `date` (ISO-8601 from header), `title`, `body` (full markdown of that block).
- [ ] Verify count: every session referenced anywhere else must exist as a row.

### 4b — `principles`
- [ ] Parse `docs/design-principles.md`.
- [ ] Records: Core Fantasy, High-Concept Framing, Chassis-and-Engine lens, North Star, Principles 1–8, Hard Constraints (one row each), Spending Tension (G8), Economy Philosophy, Incremental Testing Methodology, Cognitive Load.
- [ ] Type each via `kind`.

### 4c — `open_questions` (live + archived merged)
- [ ] Parse `game-state/OPEN_QUESTIONS.md` → live OQs with their current status.
- [ ] Parse `game-state/OPEN_QUESTIONS_ARCHIVE.md` → OQs with status that already terminated (set the appropriate terminal status, not bulk `archived` — preserve resolved/closed/scrapped/parked distinctions).
- [ ] Fields per OQ: `id`, `title`, `status`, `priority`, `affected_systems` (JSON), `body`, `created_in`, `resolved_in` (best-effort from text).

### 4d — `adrs`
- [ ] Parse `docs/mechanics-log/mechanics-evaluated.md` — extract ADR-001 through ADR-004 (and any others).
- [ ] Fields: `id`, `n`, `title`, `status`, `body`, `decided_in`, `superseded_by`.

### 4e — `mechanics`
- [ ] Parse rest of `docs/mechanics-log/mechanics-evaluated.md` — decision registry rows.
- [ ] Slugify names → `id`. Set `verdict`, `source_oq`, `decided_in`, `body`.

### 4f — `stacks`
- [ ] Parse `docs/test-scenarios/TESTING_PLAN.typ` to seed status per stack.
- [ ] For each stack folder under `docs/test-scenarios/stack-*/`: read its `.typ` → `body` markdown summary (extract from comments + section headers as reasonable).
- [ ] Plus Stack A (archived in old-game-versions/archived-stacks/) — add as `archived`.
- [ ] Plus withdrawn stacks (B, I, etc.) referenced in TESTING_PLAN — add as `withdrawn`.
- [ ] Stack M body must include the Session 26 rule substance — this is the future rules-engine source.

### 4g — `playtests`
- [ ] One row per playtest 1–5.
- [ ] `body` = `docs/research/playtest-N-analysis.md` content (for P1–P4). For P5, body = `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`.
- [ ] `raw_artefacts_path` = `design/raw/playtest-photos/playtest-N/` (the target location after Step 5).
- [ ] `stack_id` linkage where applicable.

### 4h — `backpocket`
- [ ] Parse `docs/backpocket.md`. One row per entry. Slug `id`.
- [ ] Categorize each entry into `guardrail`, `staged-fix`, `skill-candidate`, `tooling`, `process`, `note`, `to-discuss`.

### 4i — `next_steps`
- [ ] Parse `game-state/NEXT_STEPS.md`. One row per action item.
- [ ] Set `owner_oq` or `owner_stack` when clearly anchored.

### 4j — `essays`
- [ ] One row per file in `docs/research/*.md` **except** `playtest-N-analysis.md` (those went to `playtests`).
- [ ] `topic` = short tag from filename.

### 4k — `design_docs`
- [ ] Row for `docs/game-identity-visual-naming.md` → `domain='visual'`, status=`active`.

### 4l — `links` (cross-references)
- [ ] After all records exist, sweep through bodies and extract cross-references.
- [ ] For each "OQ-X" mentioned in OQ-Y's body → `(oq-y, oq-x, 'related-to')` or more specific where text indicates.
- [ ] For each Stack referenced in OQ body → `(oq-x, stack-y, 'addressed-by')` or `('resolved-by')`.
- [ ] Connected-to lists in OQ bodies → explicit `connected-to` rows.
- [ ] Backpocket promoted_to → `promoted-to` rows.
- [ ] Stack absorbed-into → `absorbed-into` rows.
- [ ] This step is best-effort. Manual review in a later session can refine.

### 4m — Sanity checks
- [ ] No NULL FK violations: `PRAGMA foreign_key_check;`
- [ ] Every OQ-id mentioned in any body has a row in `open_questions`.
- [ ] Every Stack-id mentioned in any body has a row in `stacks`.
- [ ] Row counts roughly match source MDs (e.g. ~40 OQs total live+archived).

### 4n — Delete migrated MDs
- [ ] Delete: `game-state/SESSION_LOG.md`, `STATUS.md`, `NEXT_STEPS.md`, `OPEN_QUESTIONS.md`, `OPEN_QUESTIONS_ARCHIVE.md`.
- [ ] Delete: `docs/design-principles.md`, `docs/backpocket.md`, `docs/mechanics-log/mechanics-evaluated.md`, `docs/systems-and-mechanics.md`, `docs/game-identity-visual-naming.md`.
- [ ] Delete: `docs/research/*.md` (all 17 files — the 4 playtest-analyses migrated to playtests, others to essays).

**→ Surface to user with row counts per table.**

---

## Step 5 — Raw-file moves

Use `git mv` so history is preserved. Photos and scans don't change content, just location.

- [ ] `playtest-results/elias-vs-pasco-31_10_25/` → `design/raw/playtest-photos/playtest-1/`
- [ ] `playtest-results/elias-vs-jonathan-24_04_26/` → `design/raw/playtest-photos/playtest-2/`
- [ ] `playtest-results/elias-vs-mario-17_05_26/` → `design/raw/playtest-photos/playtest-3/`
- [ ] `playtest-results/elias-vs-niko-28_05_26/` → `design/raw/playtest-photos/playtest-4/`
- [ ] `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/` → `design/raw/playtest-photos/playtest-5/`
- [ ] In each migrated folder: delete the `.md` files (notes, transcripts, side-notes, feedback markdown) — content is now in `playtests.body`. Keep all images.
- [ ] Delete `playtest-results/README.md` (user said: can go).
- [ ] Remove empty `playtest-results/` directory.

- [ ] `docs/research/session-26-brainstorm-scans/` → `design/raw/brainstorm-scans/session-26/`
- [ ] `images/*.jpg` → `design/raw/skill-card-images/`
- [ ] Delete `images/README.md` (user said: can go).
- [ ] Remove empty `images/` directory.

---

## Step 6 — Archive moves

- [ ] `old-game-versions/` → `archive/old-game-versions/` (rename via `git mv`).
- [ ] `docs/test-scenarios/` → `archive/paper-pipeline/` (rename via `git mv`).
- [ ] Verify `docs/` and its subdirs (`mechanics-log/`, `research/`) are now empty.
- [ ] Remove empty `docs/` directory.

---

## Step 7 — Cleanup

- [ ] Delete `WHAT_TO_PRINT.md` from root.
- [ ] Verify these are gone from disk: `game-state/`, `playtest-results/`, `images/`, `docs/`, `WHAT_TO_PRINT.md`, `prototype/` (already gone).
- [ ] `digital-prototype/INTAKE (claude yappt...).md` stays (user's personal file).
- [ ] `digital-prototype/schema-v1.sql` already deleted in Step 1.

---

## Step 8 — `game/README.md` placeholder

Short. One paragraph. Says: digital implementation lives here, Rust core + multi-platform frontend, awaiting architecture ADR before code lands. Points to `design/design.db` for design knowledge.

---

## Step 9 — CLAUDE.md rewrite

The existing Architecture section is now wrong end-to-end. Need to:
- [ ] Rewrite Architecture section to reflect new layout.
- [ ] Replace "Source-of-truth hierarchy" — DB is now the truth for design knowledge.
- [ ] Drop references to STATUS.md, NEXT_STEPS.md, OPEN_QUESTIONS.md, SESSION_LOG.md, mechanics-evaluated.md, backpocket.md, design-principles.md, systems-and-mechanics.md, test-scenarios/, ruleset-baseline.typ.
- [ ] Add new section: "Working with the design database" — common queries, how to add a row, ID conventions.
- [ ] Keep: Justification Rule, Incremental Testing Methodology (still valid even with digital pivot), Hygiene principles (still load-bearing).
- [ ] Strip section on Skills (Slash Commands) of references to defunct files.

---

## Step 10 — Update `.claude/HANDOVER.md`

- [ ] Update Session-27 entry. New "Where We Are" reflects digital pivot + DB migration.
- [ ] Update "Key Files" table — most rows now point to `design/design.db` queries.
- [ ] Immediate Next Action: architecture ADR session (Rust core + frontend stack).

---

## Step 11 — Verify + propose commit

- [ ] `git status` — review.
- [ ] `git diff --stat` — sanity-check no surprises.
- [ ] Propose commit message (no auto-commit — user decides).

**→ Surface to user with final summary.**

---

## Rollback plan if anything breaks

- Nothing is force-pushed. `git restore` brings back any deleted MD.
- DB itself is a single file — delete + regenerate from schema if corrupted.
- Migration is one big diff; if a class of records is wrong, revert via git, fix the importer, redo.
