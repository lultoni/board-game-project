---
name: start
description: "Session start — reads all living documents, checks for new files, and presents a concise status briefing to resume work."
disable-model-invocation: true
---

# Session Start

Read the following files in order to build context, then present a concise status briefing.

## Step 0: Sync with Remote

Run `git pull` to fetch any changes pushed from another device. If there are conflicts, surface them to the user before proceeding.

## Step 1: Read Living Documents

Read all of these files (in parallel where possible):

1. `CLAUDE.md`
2. `game-state/STATUS.md`  ← one-screen re-entry doc; read first for orientation
3. `docs/design-principles.md`
4. `game-state/NEXT_STEPS.md`
5. `game-state/OPEN_QUESTIONS.md`
6. `.claude/HANDOVER.md`

## Step 2: Check for New Files

Check for any new or modified files in:
- `playtest-results/`
- `docs/research/`

If new files exist since the last session, read them and note their contents.

## Step 3: Present Status Briefing

Output a concise briefing with:

1. **Session number** (increment from last session in the log)
2. **Where we are** (2-3 sentences from HANDOVER.md "Where We Are")
3. **Immediate next action** (from HANDOVER.md)
4. **New files found** (if any — summarise what they contain)
5. **Open blockers** (any Priority 1 items from NEXT_STEPS.md that are waiting on user input)

Keep the briefing under 20 lines. Do not repeat full file contents — summarise.
