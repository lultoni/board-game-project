# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-18 — end of Session 15.*

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

### Where We Are (Session 15 complete, 2026-05-18)

- **Playtest 3 analysed** (`docs/research/playtest-3-analysis.md`). Stack A Game 1 (standard attack 1 DMG) **confirmed working** — standoff dissolved, first Champion kill R11. Standard Attack 1 DMG **accepted into baseline**.
- **Stack B de-prioritised**: Bodyguard activated organically under Stack A. Re-evaluate after Stack A G2.
- **Two new OQs**: OQ-52 (centre-of-board has no attractor) and OQ-53 (attrition vs regicide — King isn't a real target). Both need brainstorm sessions before Stack F.
- **Four new backpocket entries**: 8×10 narrower board, starting-formation swap to expose King, "spec for a programmer" (with `/research requirements engineering` note), digital playtest prototype (sleep-on-it, ADR required).
- **Rune Theft (OQ-34) reframed** into state-dependent Mode A / Mode B with time-dependent disable value. Inconclusive, monitor in Stack A G2.

### Immediate Next Action

**Session 16 = pre-Stack-A-G2 prep** (do NOT print Stack A G2 yet). Priority order:
1. Resolve baseline rule ambiguities: Lance Thrust + Injured Range penalty (cost Elias R22), Focus Strike + adjacent self-target. Update `ruleset-baseline.typ` so 1 DMG is canonical.
2. Fix tracking sheet: add Standard Attack count column, bake Rune-gain + Skill-Slot scaling into the sheet, drop redundant skill-cost column.
3. Brainstorm OQ-52 + OQ-53 (combined session — they share solution space). Constraint: no "queen" piece; King must *participate*, not just *survive*.
4. Decide combo-bonus scope (Strike+Strike-only vs cross-category) before Stack A G2 rule sheet is finalised.
5. Sleep-on-it: digital prototype ADR — only after physical iteration loop is unblocked.

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
