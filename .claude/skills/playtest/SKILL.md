---
name: playtest
description: "Analyse playtest results — paper photos in design/raw/playtest-photos/ or a digital game log from game/. Inserts a `playtests` row + an `essays` analysis row. Auto-triggers when the user mentions playtest results."
argument-hint: "<playtest-number>"
---

# Playtest Analysis: Playtest $ARGUMENTS

This skill handles **two input modes**:

- **Paper mode**: photos/scans in `design/raw/playtest-photos/playtest-N/` (handwritten log + feedback forms + optional side-notes `.md`).
- **Digital mode**: a structured game log exported by the Rust binary in `game/` (JSON / structured markdown). Once `game/` produces logs, this is the default mode.

Detect mode automatically: if `design/raw/playtest-photos/playtest-N/` has image files → paper mode. If a digital log file is provided (or the user names one) → digital mode.

## Step 1: Pull Context from the DB

Before reading any playtest material:

```bash
sqlite3 design/design.db <<'SQL'
SELECT id, body FROM stacks WHERE status='active';
SELECT id, title, body FROM open_questions WHERE status IN ('critical','high','tracking');
SELECT id, body FROM playtests ORDER BY n DESC LIMIT 3;
SQL
```

Identify which stack this playtest was testing, and which OQs are TRACKING for that stack.

## Step 2: Transcribe / Parse

### Paper mode

Read EVERY file in `design/raw/playtest-photos/playtest-N/` — images and any `.md` side-notes. Transcribe with maximum fidelity before interpreting. Errors in transcription cascade into wrong design conclusions.

Transcription rules:

- **Game logs**: reconstruct every round row (Money, skills used, events). Preserve exact wording of margin notes — they are named moments. Note final round explicitly. Flag unclear entries with `[unclear]`.
- **Feedback forms**: transcribe exact circled position, exact wording. Distinguish "gut so" (design affirmation) from neutral, soft flags from hard flags, "?" (uncertainty — don't convert to yes/no). German handwriting: read before translating; common confusions: Schild≠Skill, limitiert≠Bremse.
- **Side-notes**: every bullet is a distinct design signal — rule clarifications, skill observations, new ideas, layout notes.

### Digital mode

The digital export is a `boardgame-bundle-v1` JSON blob (the "send to designer" button in the library UI bundles N recent matches). **It is one giant single line — do NOT `Read` or `cat` it; it will blow your context.** Instead run the analyzer:

```bash
python3 game/tools/analyze_playtest.py <bundle.json>                 # all games, full report
python3 game/tools/analyze_playtest.py <bundle.json> --game N        # one game
python3 game/tools/analyze_playtest.py <bundle.json> --combo-trace   # audit combo-bonus application
python3 game/tools/analyze_playtest.py <bundle.json> --json          # machine-readable
```

The analyzer decodes the FEN grammar (`game/crates/core_engine/src/state/fen.rs`) and per-ply schema (`game/crates/core_engine/src/telemetry.rs`) and emits, per game and per side: result + round count, material arc (piece census start→final via FEN diffs), capture timeline, first-Guard/first-Champion death rounds, move-attack vs skill-activation balance, per-skill usage counts + round ranges + drafted-but-never-used, draft loadouts, money-on-skills estimate, branching factor (legal_count) by phase, and a combo-bonus audit (tracks `tracked_enemies`/`tracked_casters` + per-square combo counter to flag where the +counter bonus should apply).

**Known telemetry gaps (state them in the analysis, don't infer around them):**
- **Wall-clock is NOT in the log** — human plies record `thought_ms=0` (only AI plies are timed). Get game duration from the designer's `notes.md`, never from telemetry.
- **Preset (first-game) loadouts have no `DraftTurn` plies** — the draft-loadout section is empty for preset games; the skills live in the `start_fen` instead (parse that if you need them).

If the analyzer is missing a metric a specific playtest needs, extend `game/tools/analyze_playtest.py` (it imports cleanly: `from analyze_playtest import parse_fen_board, combo_trace, ...`) rather than hand-parsing the blob. Feed its output into the same Block A/B templates as paper mode. **The multi-agent parallel extraction below is paper-mode only — digital mode is already structured, so skip it.**

## Step 3: Structured Extraction (Per Player)

For each player, produce two blocks:

### Block A — Tracking Data

```markdown
### Tracking Data — [Player]

**Money economy**
- Starting Money: [N]
- First round a skill was used: R[N]
- Rounds with unspent Money > 6: [list or "none"]
- Rounds with full spend: [list]
- Largest single-turn spend: [N Money on R[N]]

**Captures & key events**
| Round | Event |
|-------|-------|
| ... | ... |

- Final round: R[N]
- Captures made / suffered: [N] / [N]
- Post-game annotations: [...]
```

### Block B — Behavioral Patterns

```markdown
### Behavioral Patterns — [Player]

**Skill usage**
| Skill | Uses | Rounds | Typical context |
|-------|------|--------|-----------------|
| ... | ... | ... | ... |

- Most-used: [skill] ([N])
- Unused: [list]
- Skills used 3+ times — over-relied?: [list]

**Attack vs skill balance**
- Move-attacks made / skill activations / ratio: [N] / [N] / [N per skill]

**Combos (Stack-relevant)**
- Multi-Champion combos attempted / succeeded / pairs / blockers: ...

**Positioning**
- First forward move round / first contact / standoff evidence / Guard role: ...

**Armor**
- Granted (you / opp) / rounds active / damage absorbed ratio: ...
```

After both players, write 3–5 cross-player observations.

### Multi-agent parallel extraction (paper mode only)

To prevent bias, spawn TWO `Agent` calls in parallel (single message, two tool calls):

- Agent 1: reads Player A's files only, produces A-only Block A + B.
- Agent 2: reads Player B's files only, produces B-only Block A + B.

Each agent prompt must include exact file paths, the Block A/B templates, and the instruction "Do not read the other player's files."

In digital mode this is unnecessary — the log is structured.

## Step 4: Synthesise

Compose a full analysis markdown body. Required sections:

```markdown
# Playtest N Analysis

**Date** · **Players** · **Stack tested** · **Mode** (paper/digital) · **Game length** (final round / session time)

## Tracking Data
[Block A per player]

## Behavioral Patterns
[Block B per player + cross-player synthesis]

## Raw Transcriptions / Log Excerpts
[Paper: exact quotes preserved. Digital: representative log slices.]

## Key Findings
[Numbered, ordered by impact. Each cites source — quote, round, or log entry.
Split into:
  - Self-reported (player said it)
  - Behaviorally observed (the log shows it; player may not have named it)]

## Answers to Stack Hypothesis Questions
[For each question in the active stack's body, answer from evidence.]

## OQ Verdicts
[For each TRACKING OQ: Hypothesis / Evidence / Verdict / Recommended action.]

## Implications for Design
[Urgent / deferred / ideas — explicit.]

## Routing — Which Stack Next?
[Map metrics to the active stack's Routing on result block.]
```

## Step 5: Persist to DB

Wrap in a transaction:

```bash
sqlite3 design/design.db <<SQL
BEGIN;

-- 1. Insert the playtest row (summary metadata + short body)
INSERT INTO playtests (id, n, date, stack_id, body) VALUES (
  'playtest-<N>',
  <N>,
  '<date>',
  'stack-<X>',
  '<concise markdown summary — top findings, final round, verdict-level only>'
);

-- 2. Insert the full analysis as an essay (long-form artefact)
INSERT INTO essays (id, date, title, description, body) VALUES (
  'essay-playtest-<N>-analysis',
  '<date>',
  'Playtest <N> Analysis',
  'Full analysis with Block A/B data, OQ verdicts, routing',
  '<full markdown body from Step 4>'
);

-- 3. Link
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('playtest-<N>', 'stack-<X>', 'evidence-for', NULL),
  ('essay-playtest-<N>-analysis', 'playtest-<N>', 'related-to', NULL);

-- 4. OQ updates from Step 4 verdicts
UPDATE open_questions SET status='resolved' WHERE id='oq-N';
UPDATE open_questions SET body='<updated body with new evidence>' WHERE id='oq-M';

-- 5. New OQs raised
INSERT INTO open_questions (id, title, status, priority, body) VALUES (...);

-- 6. Mechanics impacted
UPDATE mechanics SET verdict='accepted', body='<updated>' WHERE id='mech-<slug>';

-- 7. New backpocket entries from side-notes
INSERT INTO backpocket (id, title, status, body) VALUES (...);

-- 8. next_steps
UPDATE next_steps SET status='done' WHERE id='ns-playtest-<N>';
INSERT INTO next_steps (id, priority, title, status, body) VALUES (...);

COMMIT;
SQL
```

Verify integrity:

```bash
sqlite3 design/design.db "PRAGMA foreign_key_check; PRAGMA integrity_check;"
```

## Step 6: Backpocket Trigger Check

```bash
sqlite3 design/design.db "SELECT id, title, body FROM backpocket WHERE status='staged';"
```

For each staged entry, check whether this playtest's findings triggered its activation condition. If yes, surface to the user.

## Step 7: Present Summary

Output a concise summary:
1. Top 3–5 findings (one sentence each, with source — self-reported or behavioral).
2. OQ verdicts that changed.
3. Behavioral patterns not captured in any feedback question.
4. **Routing recommendation**: next stack to activate, citing the specific metric + routing rule.
5. Any backpocket entries triggered.

The full analysis lives in `essays` (queryable via `SELECT body FROM essays WHERE id='essay-playtest-<N>-analysis';`); the summary playtest row in `playtests`. No `.md` files are written.
