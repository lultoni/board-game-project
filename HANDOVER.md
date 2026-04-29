# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-04-29 — end of Session 11.*

---

## Instructions for Claude: How to Maintain This Handover Prompt

**When to update**: At the end of every session (or when the user says "wrap up"), update this file with:
1. The current session number and date.
2. A 2-3 sentence summary of what was accomplished this session.
3. The current "Where We Are" section (overwrite, don't append history — the session log has history).
4. The current "Immediate Next Action" (the very first thing to do next session).

**What NOT to put here**: Full design details, rule text, or long explanations. This is a pointer document — it tells you WHERE to look, not WHAT the answers are. Keep it under 80 lines.

---

## The Prompt

You are my board game design co-creator and systems architect. We are working on a 2-player tactical board game (working title: "(GAME NAME)") inside this repository.

### How to start this session

1. Read `CLAUDE.md` (project conventions, mandatory methodology rules).
2. Read `game-state/CURRENT_DESIGN.md` (living master document — source of truth for design decisions).
3. Read `game-state/NEXT_STEPS.md` (prioritised action items).
4. Read `game-state/OPEN_QUESTIONS.md` (unresolved design questions).
5. Skim `docs/brainstorm/session-log.md` for the latest session entry to understand recent context.
6. Check if the user has added any new files in `playtest-results/` or `docs/research/` since last session.

### Where We Are (Session 11 complete, 2026-04-29)

- **Playtest-response research complete**: 4 research topics finished (clever-play levers, checkmate, forward positioning, skill catalogue balance). Results in `docs/research/`.
- **10 skill candidates staged**: Thorn Armor, Runic Ward, Bulwark, Bind, Energize, Skill Drain, Mini-Step, Swap Step, Ram, Gravity Well — all in `docs/backpocket.md` with sente analysis and conflict notes.
- **Sente as primary standoff solution**: Design skills that create must-respond threats from forward positions. Confirmed G1/G8 compatible.
- **G8 guardrail added**: "Players must always want to do more than they can execute." In CLAUDE.md + backpocket.
- **Checkmate killed**: Replaced by King Lifetime HP + Armor Decay as Stack C candidates.
- **Stack A still ready to print and play** — unchanged from Session 9. This remains the immediate priority.

### Immediate Next Action

**Print and play Stack A.** Files in `docs/test-scenarios/stack-a-cleverness/`:
- `stack-a-game1-attack-nerf.pdf` — play first
- `stack-a-game2-attack-nerf-combo.pdf` — play second
- `stack-a-feedback.pdf` — fill out after Game 2
- `shared/game-tracking.pdf` — 1 per player per game

After results: use `/playtest` skill to transcribe. Then consult `docs/backpocket.md` for pre-staged responses (sente skills, cascade trigger, skill candidates) based on what the data shows.

### Key Design Principles (always follow)

- **Core fantasy**: Spell/skill combos. "Does this make spell combos more interesting?" is the test for every system.
- **Incremental testing**: NEVER propose changing multiple interacting systems at once. See CLAUDE.md for the full methodology.
- **Perfect information**: No dice, no hidden cards, no randomness.
- **Research protocol**: Use `/research <topic>` skill for external knowledge — generates formatted Perplexity prompts with project context.
- **Living documents**: Update `CURRENT_DESIGN.md` after every significant decision. It is the source of truth for design decisions. `docs/test-scenarios/baseline/ruleset-baseline.typ` is the source of truth for rule text.

### Key Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Project conventions + mandatory rules |
| `docs/systems/` | Per-system design docs (7 files) — full rules, MDA, health scores |
| `docs/test-scenarios/baseline/ruleset-baseline.typ` | Canonical player-facing rules (source of truth for rule text) |
| `docs/test-scenarios/TESTING_PLAN.pdf` | Dynamic stack decision tree — which stack to run next |
| `docs/test-scenarios/shared/baseline-sections.typ` | 16 parameterized section functions for composable rule sheets |
| `docs/test-scenarios/stack-a-cleverness/` | Stack A rule sheets + feedback (ready to print) |
| `docs/test-scenarios/stack-b-guards/` | Stack B rule sheet + feedback (ready to print) |
| `docs/test-scenarios/build-pdfs.sh` | Rebuilds all PDFs from Typst source |
| `docs/backpocket.md` | Design Guardrails (G1-G8) + staged fixes + skill candidates |
| `docs/design-language.md` | Future visual/identity direction (Phase B) |
| `docs/mechanics-log/mechanics-evaluated.md` | Running log: accepted, deferred, withdrawn mechanics |
| `game-state/CURRENT_DESIGN.md` | Living master design doc (summary index) |
| `game-state/OPEN_QUESTIONS.md` | All unresolved questions with status + triggers |
| `game-state/NEXT_STEPS.md` | Prioritised action items |
| `docs/decisions/ADR-001/002/003` | Architecture direction decisions |
| `docs/research/` | Research threads + playtest analyses |
| `docs/brainstorm/session-log.md` | Session history |
