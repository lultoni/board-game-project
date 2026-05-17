---
name: wrapup
description: "Session end — updates all living documents, writes timeline entry, and updates handover prompt for next session."
disable-model-invocation: true
---

# Session Wrap-Up

Perform all of the following updates to close out the current session. Ask the user for a brief summary of what was accomplished if it's not clear from the conversation.

## Step 1: Determine Session Info

- Check the latest session entry in `old-game-versions/README.md` (in The Ultimate Game Timeline section) to find the current session number.
- New session number = last session number + 1.
- Session date = today's date.

## Step 2: Update Living Documents

Update each of these files. Only modify sections that changed this session.

### `game-state/OPEN_QUESTIONS.md`
- Mark any resolved questions with resolution and date.
- Add any new questions raised this session (assign next OQ number).
- Re-prioritise if needed.

### `game-state/NEXT_STEPS.md`
- Check off completed items.
- Add new action items from this session.
- Re-prioritise based on current state.
- Update the "Last updated" timestamp.

### `docs/systems-and-mechanics.md`
- Update any system sections where mechanics changed, new playtest evidence came in, or open questions were resolved.
- Only touch sections that actually changed this session.

### `docs/design-principles.md`
- Add any new principles or constraints that emerged this session.
- Only modify if a new principle was explicitly established.

### `docs/mechanics-log/mechanics-evaluated.md`
- Add any new mechanics that were proposed, accepted, deferred, or withdrawn this session.

### `docs/test-scenarios/TESTING_PLAN.typ`
- Update the decision tree tables if any stack results came in or entry conditions changed.
- Update stack statuses in the Testing Stacks table.
- After editing, run `zsh docs/test-scenarios/build-pdfs.sh` to rebuild TESTING_PLAN.pdf.

### `README.md`
- Update "Current Status" table if any stack statuses changed.
- Update the session number in the heading.

### `old-game-versions/README.md`
- Add a brief session entry (2-5 lines) to the timeline in the appropriate chronological position, following the existing style:

```markdown
### [Date] — Session N: [Short Title]

[2-3 sentences summarizing what was accomplished and any key decisions made.]
```

## Step 3: Update Handover Prompt

Update `.claude/HANDOVER.md`:
- Update "Last updated" line with new session number and date.
- **Overwrite** the "Where We Are" section with current state (don't append history).
- **Overwrite** the "Immediate Next Action" with the single most important next step.
- Keep everything else unchanged.
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
5. Output a summary of what was updated and confirm the push succeeded.
