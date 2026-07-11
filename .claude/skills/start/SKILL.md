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
4. `design/RULES.md` — **the canonical current ruleset (authoritative on conflict).** Read this before any session involving game rules, piece behaviour, skill descriptions, or engine work. Key facts: 8×8 board; Guards speed=2 zigzag BFS (blocked, NOT ray-sliding); Champions/King speed=1; **only Champions and King have skill slots — Guards have none**; armor cap=2; no injured penalties; combo bonus triggers on Strike OR movement-causing skills by a new Champion. Rules marked **⧗ Stack N — staged, awaiting P7** are in the engine but not yet playtest-confirmed (Focus cost 2, max 1 move-attack/turn, strike-moves-caster).

## Step 2: Query the DB for Live State

Run these queries (one combined `sqlite3` call is fine):

```bash
sqlite3 design/design.db <<'SQL'
.headers on
.mode column
SELECT n, date, title FROM sessions ORDER BY n DESC LIMIT 3;
SELECT id, title, priority, status FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;
SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;
SELECT id, name, status, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked' ORDER BY id;
SQL
```

If anything in the STATUS / HANDOVER references an ID, query the body:

```bash
sqlite3 design/design.db "SELECT body FROM <table> WHERE id='<id>';"
```

## Step 3: Check Inboxes and Raw Assets

Check for new content the designer dropped between sessions:

- `design/inbox/` — fast-write staging (not the README): `brainstorm-*` idea dumps, `chat-*` AI transcripts, `digital-*` architecture notes, `playtest-*-notes.md` feedback drops
- `design/raw/playtest-photos/` — new playtest folders (binary artefacts)

If new files exist:
- Brainstorm / chat / digital dumps: read, summarise in the briefing, flag for mining into DB (`backpocket`, `essays`, `open_questions`, or `adrs` rows). Don't mine yet — that's a separate user-initiated step.
- New playtest notes or photos: surface them; the user will likely invoke `/playtest <N>` next.

## Step 4: Present Status Briefing

Output a concise briefing with:

1. **Session number** (last `n` from `sessions` + 1)
2. **Where we are** (2-3 sentences from HANDOVER.md "Where We Are")
3. **Immediate next action** (from HANDOVER.md)
4. **Parked levers** (parked `staged-fix` backpocket rows — the "if problem X, deploy Y" candidates)
5. **Live critical/high OQs** (count + topmost titles)
6. **Inbox / raw deltas** (anything new since last session — summarise, don't dump)
7. **Open blockers** (Priority 1 todos awaiting user input)

Keep the briefing under 25 lines. Do not paste full file or row bodies — summarise.
