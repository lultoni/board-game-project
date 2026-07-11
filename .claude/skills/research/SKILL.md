---
name: research
description: "Generate a Perplexity research request with project context when external game design knowledge is needed. Persists results into the `essays` table."
argument-hint: "<topic>"
---

# Research Request: $ARGUMENTS

## Step 1: Check for Existing Research

Query the `essays` table for existing coverage:

```bash
sqlite3 design/design.db "SELECT id, title, description FROM essays WHERE description LIKE '%<keyword>%' OR title LIKE '%<keyword>%';"
```

Also check `design/inbox/` for any unmined `chat-*.md` transcripts on this topic.

If existing material answers the question, summarise it instead of generating a fresh request. Pull the body:

```bash
sqlite3 design/design.db "SELECT body FROM essays WHERE id='essay-<slug>';"
```

## Step 2: Build Context Block

Pull the design context from the DB:

```bash
sqlite3 design/design.db <<'SQL'
SELECT body FROM principles WHERE kind IN ('north-star','lens') ORDER BY n;
SELECT id, title FROM open_questions WHERE status IN ('critical','high');
SELECT body FROM design_docs WHERE id='design-doc-systems-and-mechanics';
SQL
```

Identify which specific systems or OQs relate to this research topic.

## Step 3: Generate Perplexity Prompt

Output a formatted research request the user can paste into Perplexity:

```
RESEARCH REQUEST: [Topic]

Context: We are designing a 2-player perfect-information tactical board game where players command armies on a grid, equipping Champions with skills/spells and spending Money to activate them. The core fantasy is discovering and executing clever spell/skill combos.

[2-3 sentences of specific context about which system or OQ this relates to.]

Questions:
1. [Specific question 1]
2. [Specific question 2]
3. [Specific question 3 — if needed]

Please include:
- Specific examples from published games
- Designer commentary or postmortems where available
- Any relevant academic/GDC research
- Concrete mechanics, not just philosophy
```

## Step 4: Process Results

Tell the user to paste the Perplexity output into a file in `design/inbox/` (e.g. `chat-perplexity-2026-06-22-<topic>.md`) and tell you when ready.

When the user signals ready:

1. Read the inbox file.
2. Summarise key findings and propose design implications (don't just paraphrase — connect to our game).
3. INSERT the substantive research into the `essays` table:

   ```bash
   sqlite3 design/design.db <<SQL
   INSERT INTO essays (id, date, title, description, body) VALUES (
     'essay-<slug>',
     date('now'),
     '<Title>',
     '<One-line description for future SELECT-by-keyword>',
     '<Full markdown body of the research findings + analysis>'
   );
   SQL
   ```

4. Add `links` rows connecting the essay to the OQs/mechanics/stacks it informs:

   ```sql
   INSERT INTO links (from_id, to_id, relation, note) VALUES
     ('essay-<slug>', 'oq-N', 'evidence-for', NULL),
     ('essay-<slug>', 'stack-X', 'evidence-for', NULL);
   ```

5. Delete the inbox file (fully absorbed) or annotate it with what was promoted where.

The `essays` table is the canonical home for research; `design/inbox/` is only staging.
