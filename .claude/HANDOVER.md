# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-17 — end of Session 14.*

---

## Instructions for Claude: How to Maintain This Handover Prompt

**When to update**: At the end of every session (or when the user says "wrap up"), update this file with:
1. The current session number and date.
2. A 2-3 sentence summary of what was accomplished this session.
3. The current "Where We Are" section (overwrite, don't append history — the timeline in `old-game-versions/README.md` has history).
4. The current "Immediate Next Action" (the very first thing to do next session).

**What NOT to put here**: Full design details, rule text, or long explanations. This is a pointer document — it tells you WHERE to look, not WHAT the answers are. Keep it under 80 lines.

---

## The Prompt

You are my board game design co-creator and systems architect. We are working on a 2-player tactical board game (working title: "(GAME NAME)") inside this repository.

### How to start this session

1. Read `CLAUDE.md` (project conventions, mandatory methodology rules).
2. Read `docs/design-principles.md` (rules to design by).
3. Read `game-state/NEXT_STEPS.md` (prioritised action items).
4. Read `game-state/OPEN_QUESTIONS.md` (unresolved design questions).
5. Check if the user has added any new files in `playtest-results/` or `docs/research/` since last session.

### Where We Are (Session 14 complete, 2026-05-17)

- **Fresh playtest pending transcription**: `playtest-results/elias-vs-mario-17_05_26/` (11 photos + side-notes) — needs `/playtest 3` next session. Confirm variant (Stack A vs baseline) before transcribing.
- **3 new ideas captured** in `backpocket.md` (Guard buffs, mid-game events, private draft + trade) — all `[TO DISCUSS]`, none active.
- **New mandatory rule** in `CLAUDE.md`: Justification Rule — every new idea must answer "what problem does this fix / what game-feel does this improve?" Variety alone is not justification.
- **`design-principles.md` amended**: strategy-specific economy is OK if balanced across multiple paths; cognitive-load 4-axis model marked as aspirational (experienced players), not descriptive of new players.
- **Test-scenario UX issues logged** in NEXT_STEPS for next template rebuild (separate src/PDFs, facilitator-page pattern, independent feedback forms, more writing space).

### Immediate Next Action

**Run `/playtest 3`** on `playtest-results/elias-vs-mario-17_05_26/`. Confirm variant first (check `side-notes.md`). After transcription, extract metrics + feedback themes, then consult `docs/backpocket.md` for pre-staged responses.

### Key Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Project conventions + mandatory rules |
| `docs/design-principles.md` | 5 principles, hard constraints, methodology |
| `docs/systems-and-mechanics.md` | All 7 systems: MDA, health scores, open questions |
| `docs/test-scenarios/baseline/ruleset-baseline.typ` | Canonical player-facing rules |
| `docs/test-scenarios/TESTING_PLAN.pdf` | Dynamic stack decision tree |
| `docs/test-scenarios/shared/baseline-sections.typ` | Parameterless baseline section functions |
| `docs/test-scenarios/stack-a-cleverness/` | Stack A (ready to print) |
| `docs/test-scenarios/stack-b-guards/` | Stack B (ready to print) |
| `docs/backpocket.md` | Guardrails (G1-G8) + staged fixes + skill candidates |
| `docs/game-identity-visual-naming.md` | Future visual/identity direction (Phase B) |
| `docs/mechanics-log/mechanics-evaluated.md` | Decision registry: accepted, deferred, withdrawn |
| `game-state/OPEN_QUESTIONS.md` | All unresolved questions |
| `game-state/NEXT_STEPS.md` | Prioritised action items |
| `old-game-versions/README.md` | Full game timeline + session history |
