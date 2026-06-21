---
name: wrapup
description: "Session end — writes session row to DB, updates open_questions/next_steps/mechanics/stacks/principles as needed, regenerates STATUS.md + HANDOVER.md, commits and pushes."
disable-model-invocation: true
---

# Session Wrap-Up

Close out the current session by persisting changes to the DB and regenerating the two on-disk re-entry docs. Ask the user for a brief summary of what was accomplished if it's not clear from the conversation.

## Step 1: Determine Session Info

```bash
sqlite3 design/design.db "SELECT MAX(n) FROM sessions;"
```

New session number = last + 1. Session date = today.

## Step 2: Persist Session Changes to the DB

All design knowledge lives in `design/design.db`. Update rows for whatever changed this session. The general pattern:

```bash
sqlite3 design/design.db <<'SQL'
-- Always wrap a session's writes in a transaction so partial failure rolls back.
BEGIN;

-- Resolve OQs that were answered this session
UPDATE open_questions SET status='resolved' WHERE id IN ('oq-N','oq-M');

-- Add new OQs raised this session
INSERT INTO open_questions (id, title, status, priority, body) VALUES
  ('oq-<next>', '<title>', 'high', 'high', '<full markdown body>');

-- Update next_steps
UPDATE next_steps SET status='done' WHERE id='ns-<id>';
INSERT INTO next_steps (id, priority, title, status, body) VALUES (...);

-- New / changed mechanic verdicts
INSERT INTO mechanics (id, name, verdict, source_oq, body) VALUES (...);
UPDATE mechanics SET verdict='accepted', body='<new body>' WHERE id='mech-<slug>';

-- Stack lifecycle
UPDATE stacks SET status='resolved' WHERE id='stack-M';
INSERT INTO stacks (id, letter, name, status, body) VALUES (...);

-- New principle or hard constraint (rare)
INSERT INTO principles (id, kind, n, name, body) VALUES (...);

-- Cross-references added this session
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('stack-M', 'oq-N', 'addresses', NULL),
  ('essay-<slug>', 'mech-<slug>', 'evidence-for', NULL);

COMMIT;
SQL
```

Only write rows that actually changed. Bodies are full markdown — they're the canonical text, not pointers to MDs.

## Step 3: Insert the Session Row

Compose a session narrative (markdown body, 2-4 short sections — what was decided, what changed in the DB, what's deferred) and insert:

```bash
sqlite3 design/design.db <<SQL
INSERT INTO sessions (id, n, date, title, body) VALUES (
  'session-<N>',
  <N>,
  date('now'),
  '<Short Title>',
  '<Full markdown body>'
);
SQL
```

## Step 4: Verify DB Integrity

```bash
sqlite3 design/design.db "PRAGMA foreign_key_check; PRAGMA integrity_check;"
```

Both must return no errors / "ok". If either fails, stop and surface the error — do NOT commit a broken DB.

## Step 5: Regenerate STATUS.md and HANDOVER.md

`game-state/STATUS.md` and `.claude/HANDOVER.md` are the two on-disk re-entry docs. Rewrite both from current DB state:

- **`game-state/STATUS.md`** — one-screen re-entry doc. Keep ≤45 lines. Sections: Current focus · Active stack · What changed this session · Immediate next action · Live critical/high OQs (as a query, not enumerated) · Open methodological loose ends · DB sanity.

- **`.claude/HANDOVER.md`** — full handover prompt. Keep ≤80 lines. Overwrite "Last updated", "Where We Are", and "Immediate Next Action" sections. The "Key DB Queries" table should reflect what's relevant for the next session's anchor task.

Both files should read as "facts derived from the DB at session end", not as living documents.

## Step 6: Commit and Push

1. `git status --short` to see all changed files.
2. Stage explicitly by name — never `git add -A` (sweeps `.DS_Store`, stray binaries). The committed binary `design/design.db` IS expected to change; stage it.
3. Commit message format:
   ```
   Session N — <short title>

   <2-3 sentence summary of what changed in the DB + on disk>
   ```
4. `git push`
5. Confirm push succeeded.

## Notes

- Migrators in `.claude/migrate_*.py` are one-shot artefacts from Session 27. Do NOT use them as a model for ongoing writes — those should be inline `sqlite3` invocations.
- If you discover the DB diverged from STATUS/HANDOVER mid-session (e.g. someone edited the file by hand), trust the DB and regenerate. Don't merge by hand.
