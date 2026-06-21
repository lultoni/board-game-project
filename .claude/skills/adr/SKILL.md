---
name: adr
description: "Create an Architecture Decision Record when a decision point emerges with multiple valid approaches. Persists the ADR into the `adrs` table once decided."
argument-hint: "<topic>"
---

# Architecture Decision Record: $ARGUMENTS

## Step 1: Research Context

Query the DB for relevant context:

```bash
sqlite3 design/design.db <<'SQL'
SELECT body FROM principles WHERE kind IN ('north-star','lens','hard-constraint') ORDER BY n;
SELECT id, title FROM open_questions WHERE status IN ('critical','high');
SELECT id, n, title FROM adrs ORDER BY n;
SQL
```

If a specific OQ or essay informs this decision, pull its body:

```bash
sqlite3 design/design.db "SELECT body FROM open_questions WHERE id='oq-N';"
sqlite3 design/design.db "SELECT body FROM essays WHERE id='essay-<slug>';"
```

If external game-design knowledge is needed and you're not confident, trigger `/research` first.

## Step 2: Present the ADR inline

Do NOT write a file yet. Present the ADR directly using this structure:

```
## ADR: [Decision Title]

**Date**: [today]
**Related OQs**: [oq-N, oq-M] (if any)

### Context
[2-3 paragraphs: what prompted this, what constraints exist, how it connects to the core fantasy]

### Option A: [Name]
**How it works**: [Concrete description]
**Pros**: [Bullet list]
**Cons**: [Bullet list]

### Option B: [Name]
[Same structure]

### Option C: [Name] (if applicable)
[Same structure]

### Recommendation
[Opinionated assessment. Evaluate against the core fantasy ("does this make spell combos better?") and the north stars / hard constraints.]
```

Ask the user to decide.

## Step 3: After Decision — Persist to DB

When the user decides, INSERT the ADR into `adrs` and update affected rows.

```bash
NEXT_N=$(sqlite3 design/design.db "SELECT COALESCE(MAX(n),0)+1 FROM adrs;")
sqlite3 design/design.db <<SQL
INSERT INTO adrs (id, n, date, title, body) VALUES (
  'adr-00${NEXT_N}',
  ${NEXT_N},
  date('now'),
  '[Decision Title]',
  '[Full markdown body: Context + Options + Decision + Consequences]'
);
SQL
```

Then:

1. **Resolve related OQs**: `UPDATE open_questions SET status='resolved' WHERE id IN ('oq-N','oq-M');`
2. **Link the ADR** to the OQs it resolves:
   ```sql
   INSERT INTO links (from_id, to_id, relation, note) VALUES
     ('adr-00<N>', 'oq-N', 'resolved-by', NULL);
   ```
3. If the decision requires testing, trigger `/scenario` to create the test stack.
4. If the decision changes a principle or mechanic verdict, `UPDATE` the relevant row's body and add a `supersedes` link from the new state to the old.

Keep the ADR body in the DB as full markdown — this is the canonical record. Don't write a separate `.md` file.
