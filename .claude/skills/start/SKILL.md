---
name: start
description: "Session start — pulls remote, reads STATUS + HANDOVER, queries DB for live state, checks inboxes and raw assets, presents a concise status briefing."
disable-model-invocation: true
---

# Session Start

The DB at `design/design.db` is the source of truth. This skill orients you to current state via DB queries, not by re-reading deleted MD files.

## Step 0: Sync with Remote

Run `git pull` to fetch any changes pushed from another device. If there are conflicts, surface them to the user before proceeding.

## Step 1: Read Orientation Files

Read these files (in parallel):

1. `CLAUDE.md` — orientation map
2. `.claude/STATUS.md` — one-screen re-entry doc
3. `.claude/HANDOVER.md` — last session's wrap-up + immediate next action
4. `design/raw/paper-pipeline-archive/test-scenarios/stack-m-game-length-cut/stack-m-game-length-cut.typ` — **current active ruleset (Stack M)**. Read this before any session involving game rules, piece behaviour, skill descriptions, or engine work. Key facts: Guards speed=2 zigzag BFS (blocked, NOT ray-sliding); Champions/King speed=1; **only Champions and King have skill slots — Guards have none**; armor cap=2; no injured penalties; combo bonus triggers on Strike OR movement-causing skills by a new Champion.

## Step 2: Query the DB for Live State

Run these queries (one combined `sqlite3` call is fine):

```bash
sqlite3 design/design.db <<'SQL'
.headers on
.mode column
SELECT n, date, title FROM sessions ORDER BY n DESC LIMIT 3;
SELECT id, title, priority, status FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;
SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;
SELECT id, letter, name, status FROM stacks WHERE status IN ('active','queued') ORDER BY letter;
SQL
```

If anything in the STATUS / HANDOVER references an ID, query the body:

```bash
sqlite3 design/design.db "SELECT body FROM <table> WHERE id='<id>';"
```

## Step 3: Check Inboxes and Raw Assets

Check for new content the designer dropped between sessions:

- `design/inbox/brainstorm/` — fast-write idea dumps (not the README)
- `design/inbox/ai-chats/` — pasted chat transcripts (not the README)
- `design/inbox/digital/` — architecture / UI / AI-opponent notes for `game/` (not the README)
- `design/raw/playtest-photos/` — new playtest folders

If new files exist:
- Brainstorm/ai-chat dumps: read, summarise in the briefing, flag for mining into DB (`backpocket`, `essays`, `open_questions`, or `adrs` rows). Don't mine yet — that's a separate user-initiated step.
- New playtest photos: surface them; the user will likely invoke `/playtest <N>` next.

## Step 4: Present Status Briefing

Output a concise briefing with:

1. **Session number** (last `n` from `sessions` + 1)
2. **Where we are** (2-3 sentences from HANDOVER.md "Where We Are")
3. **Immediate next action** (from HANDOVER.md)
4. **Active/queued stacks** (from the DB query)
5. **Live critical/high OQs** (count + topmost titles)
6. **Inbox / raw deltas** (anything new since last session — summarise, don't dump)
7. **Open blockers** (Priority 1 todos awaiting user input)

Keep the briefing under 25 lines. Do not paste full file or row bodies — summarise.
