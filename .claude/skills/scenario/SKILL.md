---
name: scenario
description: "Stage a candidate rule bundle as a row in the `stacks` table. The bundle becomes a digital variant (toggleable in `game/`) or a paper rule sheet (only via archived paper-pipeline)."
argument-hint: "<stack-X> <short description>"
---

# Test Scenario: $ARGUMENTS

A "scenario" (a.k.a. test stack) is now a row in the `stacks` table whose `body` markdown captures everything a future Rust prototype or paper playtest needs: what changes, why, hypothesis, watch list, routing. It is NOT a Typst file — the paper pipeline is archived.

## Step 1: Validate Methodology

Pull the canonical methodology from the DB before designing the stack:

```bash
sqlite3 design/design.db <<'SQL'
SELECT body FROM principles WHERE kind='methodology';
SELECT id, letter, name, status FROM stacks ORDER BY letter;
SQL
```

Check each:

1. **Independence**: Is this change independent of other untested changes? If not, identify the coupling — either bundle (document why) or defer.
2. **Stack assignment**: Is this a new stack or an extension of an existing one? Existing letters: query above.
3. **Ordering**: Does it depend on results of a prior untested stack? Note the dependency.
4. **Isolation**: Can we attribute observed effects to THIS change alone? If not, decompose.

Pull baseline + active stack bodies for context:

```bash
sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"
sqlite3 design/design.db "SELECT id, name, body FROM mechanics WHERE verdict='baseline';"
```

## Step 2: Pick the Next Stack Letter

```bash
sqlite3 design/design.db "SELECT letter FROM stacks ORDER BY letter;"
```

Conventional next-letter logic (alphabetic). If the new stack is a sibling variant of an existing stack (e.g. M.1 dose), use a sub-id.

## Step 3: Write the Stack Body

The `body` is full markdown. Required sections:

```markdown
# Stack X — <Name>

**Status**: queued | active
**Targets**: oq-N, oq-M
**Baseline reference**: <date or "Stack <previous letter>">

## What changes vs baseline / previous

| Concept | Before | After | Why |
|---|---|---|---|
| ... | ... | ... | ... |

## Hypothesis

[1-2 paragraphs: the specific effect this change is predicted to produce, framed against the core fantasy.]

## What "good" looks like

- [Bullet 1 — observable outcome]
- [Bullet 2]

## Watch list

- [Risk 1 — what could go wrong, and how we'd notice]
- [Risk 2]

## Routing on result

- **If hypothesis confirmed**: [next stack to queue]
- **If partial**: [adjustment / dose change]
- **If rejected**: [rollback path]

## Digital toggle

[Once `game/` exists: how this stack maps to a feature flag / config switch in the Rust core. For paper-only stacks: "paper-only, see archive/paper-pipeline/test-scenarios/ for the rule sheet."]
```

## Step 4: Insert into DB

```bash
sqlite3 design/design.db <<SQL
BEGIN;

INSERT INTO stacks (id, letter, name, status, body) VALUES (
  'stack-<X>',
  '<X>',
  '<Name>',
  'queued',
  '<full markdown body from Step 3>'
);

-- Link to OQs this stack addresses
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('stack-<X>', 'oq-N', 'addresses', NULL),
  ('stack-<X>', 'oq-M', 'addresses', NULL);

-- If extending or superseding a previous stack
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('stack-<X>', 'stack-<prev>', 'derived-from', NULL);

COMMIT;
SQL
```

If activating this stack (replacing the current Active), mark the predecessor first:

```bash
sqlite3 design/design.db <<'SQL'
UPDATE stacks SET status='resolved' WHERE id='stack-m';
UPDATE stacks SET status='active' WHERE id='stack-<X>';
SQL
```

Exactly one stack should be `active` at a time.

## Step 5: Update Affected Rows

- `next_steps`: insert a new row for "playtest stack-<X>" or "implement stack-<X> toggle in game/" depending on whether it's paper or digital.
- `mechanics`: if the stack stages a new mechanic candidate, insert into `mechanics` with `verdict='staged'` and a `link` from the stack to the mechanic (`evidence-for`).

## Step 6: Regenerate STATUS.md if Activated

If the new stack is now Active, regenerate `.claude/STATUS.md` so "Active stack" reflects the change. (The `/wrapup` skill normally does this at session end — only do it inline if the user wants the file updated now.)

## Step 7: Confirm

Output a short summary:

1. Stack ID, letter, name, status.
2. The hypothesis (one sentence).
3. OQs addressed.
4. Whether this maps to a digital toggle in `game/` or a paper run (and if paper, note that the rule-sheet generator is no longer maintained — the body in the DB is the spec).
5. Any methodology concerns (coupling, dependencies).
