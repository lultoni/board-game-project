# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-25 — end of Session 19.*

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

### Where We Are (Session 19 complete, 2026-05-25)

- **Digital prototype shipped.** Single-file offline PWA at `prototype/index.html`, deployed to GitHub Pages (repo now public). Full game loop: 10×10 board, drag-and-drop, piece state (armor/injured/skill icons), rune tracking, end-turn notes, post-game feedback form, JSON export. All 15 skill icons base64-embedded — fully offline once cached.
- **iOS touch fixed.** Rewrote Touch Events → Pointer Events API. Combined 10px distance + 100ms time threshold before committing drag; `setPointerCapture` for routing; `requestAnimationFrame` for iOS 17.4 repaint compat. Tap-to-modal and drag-to-move confirmed working on iPad.
- **Prototype is for after Nico's game.** Nico's 2026-05-28 playtest is paper-only. Prototype comes into use after that data lands.
- **2026-05-28 print packet unchanged**: same Priority 1 items from Session 18.

### Immediate Next Action

**Print the 2026-05-28 packet.** Per Priority 1 in `NEXT_STEPS.md`: `stack-a-game2-attack-nerf-combo.pdf` ×2 + `shared/skill-cards.pdf` ×2 + `shared/feedback-onboarding.pdf` ×1 (Nico) + `stack-a-feedback.pdf` ×2 + `shared/game-tracking.pdf` ×2. Run `/playtest 4` next session to analyse.

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
