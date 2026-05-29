# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-29 — end of Session 22.*

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

### Where We Are (Session 22 complete, 2026-05-29)

- **Playtest 4 analysed end-to-end.** Niko (P1, first-time) beat Elias (P2) on 2026-05-28 in a 28-29 round Stack A G2 game. Synthesis at `docs/research/playtest-4-analysis.md`. OQ-11 / Q-C1 chassis-volume hypothesis received its strongest evidence yet; Stack H now Priority 1.
- **Post-analysis design discussion** produced: OQ-38 reframe (combo softness is design-aligned; lever is scope, not strength), dual-counter combo design (Stack A G3, queued behind Stack H), Path A methodology decision, OQ-58 / OQ-59 / OQ-60 opened, six new backpocket entries.
- **TESTING_PLAN.typ rewritten.** State lifecycle introduced (Active / Queued / Dormant / Resolved — exactly one Active at a time); decision tree replaced with per-stack *Routing on result* blocks; stacks renamed for legibility (Stack H = Armor Trim, A G3 = Dual-Counter Combo, K = Piece Count Reduction); Stack I folded into H; Stack B withdrawn; Stack K decoupled from Stack D; Stack F sequenced after A G3.
- **Skill sweep.** `/playtest`, `/scenario`, `/wrapup` updated to match the new state lifecycle; `/research` and `/adr` verified clean.
- **"Nico" → "Niko" rename** completed project-wide (44 occurrences).
- **Stack H re-discussion gate**: bundled-dose framing flagged for re-discussion before any rule-sheet work.

### Immediate Next Action

**Re-discuss Stack H — Armor Trim before drafting.** Revisit the bundled dose (Armor cap 3→2 + Armorsmith +1→+2), scope, and entry conditions in the next session. Only after that conversation: write the rule sheet at `docs/test-scenarios/stack-h-armor-trim/`, build the print packet (rule sheet + skill-cards + feedback + game-tracking ×2), and schedule two experienced players. Routing rules and within-stack rollback dose live in `TESTING_PLAN.typ`.

### Key Files

| File | Purpose |
|------|---------|
| `game-state/STATUS.md` | One-screen re-entry doc (read first after a gap) |
| `CLAUDE.md` | Project conventions + mandatory rules + hygiene principles |
| `docs/design-principles.md` | 5 principles, hard constraints, methodology |
| `docs/systems-and-mechanics.md` | All 7 systems: MDA, health scores, open questions |
| `docs/test-scenarios/baseline/ruleset-baseline.typ` | Canonical player-facing rules |
| `docs/test-scenarios/TESTING_PLAN.pdf` | Active / Queued / Dormant / Resolved stack catalogue + per-stack routing |
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
