# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-26 — end of Session 20.*

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

### Where We Are (Session 20 complete, 2026-05-26)

- **High-concept investigation closed.** All 11 questions in `docs/research/high-concept-open-questions.md` resolved. ADR-004 accepted: framing is **"Two minds, one puzzle" (Framing B)** — 2-player nature load-bearing, opponent is a fellow puzzle-solver.
- **Two new design-principles sections.** `§ High-Concept Framing` (ADR-004) and `§ Chassis and Engine` (diagnostic lens) now in `design-principles.md`. Chassis/engine becomes canonical project vocabulary.
- **Q-B4 baseline change shipped.** Standard Attack reframed as "a Move that ends on an enemy tile"; survival-stop strengthened with explicit attacker-speed cases. BASELINE_VERSION → 2026-05-26.
- **Stack pipeline expanded.** Stacks H (Armor cap+Armorsmith bundle), I (Armor rollback), J (Injured-downsides removal), K (chassis-minimisation: 8×8 then 8×8 + 3+3+1 pieces) queued in `TESTING_PLAN.typ`. All gated on Stack A G2 results.
- **OQ-11 reopened, OQ-57 added.** Both under chassis-volume framing. Live entries in `OPEN_QUESTIONS.md`.
- **2026-05-28 print packet unchanged in priority** — Nico still plays standard baseline. Add `teacher-vocab-checklist.pdf` to facilitator side.

### Immediate Next Action

**Print the 2026-05-28 packet.** Per Priority 1 in `NEXT_STEPS.md`: `stack-a-game2-attack-nerf-combo.pdf` ×2 + `shared/skill-cards.pdf` ×2 + `shared/feedback-onboarding.pdf` ×1 (Nico) + `shared/teacher-vocab-checklist.pdf` ×1 (facilitator) + `stack-a-feedback.pdf` ×2 + `shared/game-tracking.pdf` ×2. Run `/playtest 4` next session to analyse.

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
