# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-20 — end of Session 17.*

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

1. Read `game-state/STATUS.md` (one-screen re-entry doc — read this first).
2. Read `CLAUDE.md` (project conventions, mandatory methodology rules, hygiene principles).
3. Read `docs/design-principles.md` (rules to design by).
4. Read `game-state/NEXT_STEPS.md` (prioritised action items).
5. Read `game-state/OPEN_QUESTIONS.md` (live design questions only — archive lives in `OPEN_QUESTIONS_ARCHIVE.md`).
6. Check if the user has added any new files in `playtest-results/` or `docs/research/` since last session.

### Where We Are (Session 17 complete, 2026-05-20)

- **Repo rework session.** Architecture audit ran across documentation, test-scenario pipeline, skill workflow, state-doc lifecycle, and repo hygiene. Findings consolidated into a single rework plan and executed in one pass.
- **Skills repaired**: ghost references to `CURRENT_DESIGN.md`, `docs/decisions/`, and `docs/brainstorm/session-log.md` repointed across `/research`, `/scenario`, `/adr`, `/playtest`, `/build-pdfs`. Layer→Stack vocabulary rename completed.
- **State docs split**: `OPEN_QUESTIONS.md` is now live-only; `OPEN_QUESTIONS_ARCHIVE.md` holds resolved/closed/scrapped/parked. `STATUS.md` added as one-screen re-entry doc. `mechanics-evaluated.md` extended with Source OQ + Evidence columns. Session log moved to `game-state/SESSION_LOG.md`.
- **Single source of truth** for rule numbers: `BASELINE_VERSION` constant added to `baseline-sections.typ`; CLAUDE.md "Key Game Systems" + `docs/systems-and-mechanics.md` numeric tables trimmed to pointers.
- **Pipeline parameterization**: `section-quick-reference()` now accepts `overrides:` and `extra-rows:`; Stack A files migrated. `build-pdfs.sh` rewritten to discover `.typ` files via `find`.
- **Hygiene**: `.DS_Store` swept; PDF commit policy documented; `playtest-results/README.md` and `images/README.md` added; `balde_call.jpg` typo fixed.
- **Hygiene principles** captured in `CLAUDE.md` so future sessions avoid the drift patterns this rework addressed.

### Immediate Next Action

**Run Stack A Game 2.** Print `stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf` + `stack-a-cleverness/stack-a-feedback.pdf` + `shared/game-tracking.pdf` (2 copies). Use `/playtest 4` next session to analyse results.

### Key Files

| File | Purpose |
|------|---------|
| `game-state/STATUS.md` | One-screen re-entry doc (read first after a gap) |
| `CLAUDE.md` | Project conventions + mandatory rules + hygiene principles |
| `docs/design-principles.md` | 5 principles, hard constraints, methodology |
| `docs/systems-and-mechanics.md` | All 7 systems: MDA, health scores, open questions |
| `docs/test-scenarios/baseline/ruleset-baseline.typ` | Canonical player-facing rules |
| `docs/test-scenarios/TESTING_PLAN.pdf` | Dynamic stack decision tree |
| `docs/test-scenarios/shared/baseline-sections.typ` | Baseline section functions + BASELINE_VERSION |
| `docs/test-scenarios/stack-a-cleverness/` | Stack A (G2 ready to print) |
| `docs/test-scenarios/stack-b-guards/` | Stack B (de-prioritised — may be obsolete) |
| `docs/backpocket.md` | Guardrails (G1-G8) + staged fixes + skill candidates |
| `docs/mechanics-log/mechanics-evaluated.md` | Decision registry (cross-linked to OQs + evidence) |
| `game-state/OPEN_QUESTIONS.md` | Live design questions only |
| `game-state/OPEN_QUESTIONS_ARCHIVE.md` | Resolved / closed / scrapped / parked OQs |
| `game-state/NEXT_STEPS.md` | Prioritised action items |
| `game-state/SESSION_LOG.md` | Per-session narrative log (newest first) |
| `old-game-versions/README.md` | v1/v2/v3 archive + pre-Session-1 history |
