---
name: wrapup
description: "Session end — updates all living documents, writes session log entry, and updates handover prompt for next session."
disable-model-invocation: true
---

# Session Wrap-Up

Perform all of the following updates to close out the current session. Ask the user for a brief summary of what was accomplished if it's not clear from the conversation.

## Step 1: Determine Session Info

- Read `docs/brainstorm/session-log.md` to find the current session number.
- New session number = last session number + 1.
- Session date = today's date.

## Step 2: Update Living Documents

Update each of these files. Only modify sections that changed this session.

### `game-state/CURRENT_DESIGN.md`
- Update the "Last updated" timestamp at the top.
- Add/modify any systems, rules, or playtest evidence that changed this session.
- Update the Design Health Check if any scores shifted.
- Update the Incremental Test Plan table if layer statuses changed.

### `game-state/OPEN_QUESTIONS.md`
- Mark any resolved questions with resolution and date.
- Add any new questions raised this session (assign next OQ number).
- Re-prioritise if needed.

### `game-state/NEXT_STEPS.md`
- Check off completed items.
- Add new action items from this session.
- Re-prioritise based on current state.
- Update the "Last updated" timestamp.

### `docs/test-scenarios/TESTING_PLAN.typ`
- Update the decision tree tables if any stack results came in or entry conditions changed.
- Update stack statuses in the Testing Stacks table (e.g., mark a layer as "Accepted").
- Update the Accepted Layers table if a new layer was accepted.
- Update the "Current Priority Sequence" table if priorities shifted.
- After editing, run `zsh docs/test-scenarios/build-pdfs.sh` to rebuild TESTING_PLAN.pdf.

### `README.md` and `old-game-versions/README.md`
- Update "Current Status" table in `README.md` if any stack statuses or descriptions changed.
- Update the session number in the "Current Status" heading.
- Update the "Now — Session N onwards" section in `old-game-versions/README.md` if the project narrative changed meaningfully (new playtests, major milestones).
- Keep changes minimal — only update what actually shifted this session.

### `docs/brainstorm/session-log.md`
- Append a new session entry at the top (below the heading) using this format:

```markdown
## Session N — YYYY-MM-DD — [Short Title]

**Goal**: [What we set out to do]

**What was done**:
- [Bullet points of accomplishments]

**Key findings**:
- [Any discoveries, playtest results, or design insights]

**Decisions made**:
- [Any design decisions with ADR references if applicable]

**Open items for next session**:
- [What needs to happen next]

---
```

## Step 3: Update Handover Prompt

Update `HANDOVER.md`:
- Update "Last updated" line with new session number and date.
- **Overwrite** the "Where We Are" section with current state (don't append history).
- **Overwrite** the "Immediate Next Action" with the single most important next step.
- Keep everything else (instructions, key files table, design principles) unchanged.
- Verify the file stays under 80 lines.

## Step 4: Commit and Push

1. Run `git status --short` to see all changed files.
2. Stage all changed files: `git add -A`
3. Commit with a message in this format:
   ```
   Session N — <short title>
   
   <2-3 sentence summary of what changed>
   ```
4. Push to origin: `git push`
5. Output a summary of what was updated (each file and what changed), and confirm the push succeeded with the commit hash.
