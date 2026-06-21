# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-21 — end of Session 26.*

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
3. Read `docs/design-principles.md` (rules to design by — note Session 23 added Principles 6 and 7).
4. Read `game-state/NEXT_STEPS.md` (prioritised action items).
5. Read `game-state/OPEN_QUESTIONS.md` (live design questions only — archive lives in `OPEN_QUESTIONS_ARCHIVE.md`).
6. Read `docs/research/path-y-defense-redesign.md` (Session 23 canonical writeup — Pole framing + Pole B mechanics live here).
7. Check if the user has added any new files in `playtest-results/` or `docs/research/` since last session.

### Where We Are (end of Session 26, 2026-06-21)

- **Stack M — Game Length Cut is the Active stack and the rule sheet is print-ready.** Six bundled simultaneous changes vs baseline (intentional methodology deviation per Principle 7): board 10×10→8×8; Armor cap 3→2; Injured penalties removed (still 2 HP); draw conditions removed; Steal cost 3→4; Multi-Champion Combo Bonus widened on two axes — (a) counter ticks on movement-causing skills, and (b) **bonus damage applies to any skill (Strike OR movement) on a counter-loaded target** (Session 26 design change).
- **Session 26 was a finalisation pass on Stack M**, not new ground. Multiple correction passes restored baseline section order, added Introduction + Simple Overview, cut over-used `changed-box` callouts and diff-style annotations, fixed the fixed-layout Setup (Kings offset, 2+3 Champion split, Guards in front of each Champion+King), collapsed Health & Armor, switched in-text skill refs to `sk()` chips, and hid Facilitator Notes via Typst block comment so the player-facing PDF reads as rules-only.
- **Substantive design change inside Stack M**: bonus damage now applies to any skill (not just Strikes) on a counter-loaded target. Movement skills become a damage vector once counter is loaded. This expands OQ-38's scope beyond the Session 25 dose; first rollback if it dominates is to revert to "movement ticks counter, Strike-only deals bonus damage" (Session 25 draft).
- **Pole B remains paused** (Session 25 P5 result). Stack H (Armor Trim) remains the isolation-fallback inside Stack M's per-axis rollback routing.
- **OQ-65 (pre-made loadouts)** is sequenced after Stack M, not before. **`BASELINE_VERSION` = 2026-05-30** (unchanged).

### Immediate Next Action

**Print Stack M and run P6.** Files: `stack-m-game-length-cut.pdf` (rule sheet) + 2× `stack-m-feedback.pdf` (one per player). Game-tracking sheet + skill cards unchanged. Track rounds + wall-clock + first-Champion-kill round vs the 30-60 min target. *Caveat:* designer may still have brainstorm batches incoming — if any arrive before scheduling P6, fold them in before locking the print version.

### Key Files

| File | Purpose |
|------|---------|
| `game-state/STATUS.md` | One-screen re-entry doc (read first after a gap) |
| `CLAUDE.md` | Project conventions + mandatory rules + hygiene principles |
| `docs/design-principles.md` | 7 principles (Session 23 added 6 and 7), hard constraints, methodology |
| `docs/research/path-y-defense-redesign.md` | Session 23 canonical writeup — Pole A/B framing + Pole B mechanics |
| `docs/systems-and-mechanics.md` | All 7 systems: MDA, health scores, open questions |
| `docs/test-scenarios/baseline/ruleset-baseline.typ` | Canonical player-facing rules |
| `docs/test-scenarios/TESTING_PLAN.pdf` | Active / Queued / Dormant / Resolved stack catalogue + per-stack routing |
| `docs/test-scenarios/shared/baseline-sections.typ` | Baseline section functions + BASELINE_VERSION (2026-05-30) |
| `docs/test-scenarios/stack-g-structure/` | Stack G (Unified AP, Dormant) |
| `old-game-versions/archived-stacks/` | Frozen test-stack archive (Stack A — accepted into baseline Session 23) |
| `docs/backpocket.md` | Guardrails (G1-G8) + staged fixes + skill candidates |
| `docs/mechanics-log/mechanics-evaluated.md` | Decision registry (cross-linked to OQs + evidence) |
| `game-state/OPEN_QUESTIONS.md` | Live design questions only |
| `game-state/OPEN_QUESTIONS_ARCHIVE.md` | Resolved / closed / scrapped / parked OQs |
| `game-state/NEXT_STEPS.md` | Prioritised action items |
| `game-state/SESSION_LOG.md` | Per-session narrative log (newest first) |
| `old-game-versions/README.md` | v1/v2/v3 archive + pre-Session-1 history |
