# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-05-31 — end of Session 24.*

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

### Where We Are (Session 24 complete, 2026-05-31)

- **Stack L — Pole B Per-Turn-Draft Prototype** is the Active stack with a standalone rule sheet now at `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.pdf`. Three-phase turn: Move → Draft → Skill. Move and Draft share a 4-action pool. Skill Phase is free; activations consume the skill back to the shared pool.
- **Pole framing remains current.** *Pole A* = pre-game-draft (current game). *Pole B* = per-turn-draft (radical alternative, now playable). Full discussion: `docs/research/path-y-defense-redesign.md`.
- **Stack H — Armor Trim** still Queued.
- **Project-wide vocabulary simplification** completed — broad rename pass across docs/skills/Typst rule sheets to align terminology and reduce jargon for new players.
- **PDF template redesigned** (Session 24). Inter typography, numbered H2 with teal numerals, new `sk("Lance")` skill-chip helper, redesigned callout boxes, pagination fixed (lists/enums no longer split mid-bullet; outer `breakable: false` wraps removed from baseline-sections + stack-l). All PDFs rebuild clean.
- **`BASELINE_VERSION` = 2026-05-30** (unchanged from Session 23).

### Immediate Next Action

**Run the first Pole B per-turn-draft prototype game digitally** with Jonathan (3-week vacation window open). Use `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.pdf` as the rule sheet. After 2–3 games, compare game-feel vs Pole A and route per Stack L's *Routing on result* in `TESTING_PLAN.pdf`.

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
