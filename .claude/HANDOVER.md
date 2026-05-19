# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-19 — end of Session 16.*

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

### Where We Are (Session 16 complete, 2026-05-19)

- **All pre-Stack-A-G2 prep done.** Stack A Game 2 (`stack-a-game2-attack-nerf-combo.pdf`) is ready to print and play.
- **Rule clarifications accepted into baseline**: Range system (default=2, "self"/"adjacent" explicit only, modifiers from default, no inward shifting), Focus Strike note added, self/adjacent targeting constraint explicit.
- **Tracking sheet redesigned**: pre-filled R+ and SS columns (change rounds show value, others show `|`); Atk column added; cost column dropped.
- **Rule document restructured**: Introduction + Simple Overview pages added; section order is now dependency-correct; Quick Reference expanded to 14 rows; designer-box style added to stack files.
- **Combo-bonus scope decided**: Strike-only for Game 2. Cross-category to reconsider after data.
- **OQ-54 + OQ-55 logged**: Lance Thrust wording question; Blade Call broader interaction question.
- **OQ-52 + OQ-53 brainstorm (centre attractor + King as real target)** not yet done — deferred, not urgent before the playtest.

### Immediate Next Action

**Run Stack A Game 2.** Print `stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf` + `stack-a-cleverness/stack-a-feedback.pdf` + `shared/game-tracking.pdf` (2 copies). Use `/playtest 4` next session to analyse results.

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
