# (GAME NAME) - Board Game Design Project

## Project Overview

A 2-player abstract-tactical board game where players command armies of Guards and Champions led by a King across a 10x10 grid. Victory comes through tactical superiority and capturing the enemy King. The design philosophy prioritises depth through interlocking systems over breadth of features.

## Architecture

```
docs/
  design-principles.md           <- Rules to design by (5 principles, constraints, methodology)
  systems-and-mechanics.md       <- All 7 systems: how they work, MDA, health, open questions
  game-identity-visual-naming.md <- Future Phase B: art, naming, physical design direction
  backpocket.md                  <- Pre-thought fixes for anticipated problems (staged, not active)
  research/                      <- Exported Perplexity research threads + playtest analyses
  mechanics-log/
    mechanics-evaluated.md       <- Decision registry: every mechanic + Source OQ + Evidence
  test-scenarios/
    build-pdfs.sh                <- Discovery-based: rebuilds every .typ → .pdf (use zsh)
    TESTING_PLAN.typ/.pdf        <- Decision tree: which stack to run next
    shared/
      template.typ               <- Shared Typst styling (library — not compiled standalone)
      baseline-sections.typ      <- Baseline section functions + BASELINE_VERSION constant
      feedback-baseline.typ      <- Feedback form template (copy for each new stack)
      game-tracking.typ          <- Per-player in-game tracking sheet
    baseline/
      ruleset-baseline.typ       <- CANONICAL PLAYER-FACING RULES (source of truth for rule text)
    stack-X-<slug>/              <- One subfolder per test stack
      stack-X-<slug>.typ/.pdf    <- Rule sheet (Typst source + compiled PDF)
      stack-X-feedback.typ/.pdf  <- Feedback form
game-state/
  STATUS.md                      <- One-screen re-entry doc — read first after a gap
  NEXT_STEPS.md                  <- Prioritised action items
  OPEN_QUESTIONS.md              <- LIVE design questions only (sorted by status)
  OPEN_QUESTIONS_ARCHIVE.md      <- Resolved / closed / scrapped / parked OQs
  SESSION_LOG.md                 <- Per-session narrative log (newest first)
playtest-results/                <- Raw photos / scans, one folder per playtest
images/                          <- Skill card images (1 JPG per skill)
old-game-versions/
  README.md                      <- v1/v2/v3 archive + pre-Session-1 history + Road Ahead
```

## Key Game Systems

For canonical numbers (HP, Rune scaling, Skill Slot scaling, skill catalogue): `docs/test-scenarios/shared/baseline-sections.typ`.
For design rationale, MDA notes, open questions per system: `docs/systems-and-mechanics.md`.

Quick orientation only — never restate numbers here:

1. **Turn Structure** — Round-based (P1 turn → P2 turn). Each turn: Movement Phase → Action Phase. See `section-turn-structure()`.
2. **Piece Types** — King (1), Champions (5), Guards (6) per player. Champions/King carry skills; Guards do not. See `section-components()`.
3. **Skill System** — Equipped during pre-game draft. Categories: Strike, Shield, Move, Mystic. Queen-style line-of-sight, blocked by all pieces. See `section-skill-system()` + `section-skill-reference()`.
4. **Resource Economy** — Runes scale over rounds. See `section-resource-economy()`.
5. **Health & Armor** — Normal → Injured → Removed. Armor absorbs first. See `section-health-armor()`.
6. **Board** — 10x10 grid, no terrain. See `section-setup()`.
7. **Bodyguard** — Adjacent Guard intercepts Standard Attacks on Champion/King. See `section-bodyguard()`.

## Conventions

### Source-of-truth hierarchy

- **Rule text (numbers, mechanics)**: `docs/test-scenarios/shared/baseline-sections.typ` — printable form: `docs/test-scenarios/baseline/ruleset-baseline.pdf`.
- **Design principles**: `docs/design-principles.md`.
- **Systems rationale**: `docs/systems-and-mechanics.md`.
- **Decision history**: `docs/mechanics-log/mechanics-evaluated.md` (cross-linked to OQs and evidence).
- **Live state**: `game-state/STATUS.md` → `game-state/NEXT_STEPS.md` → `game-state/OPEN_QUESTIONS.md`.
- **Per-session narrative**: `game-state/SESSION_LOG.md` (newest first).

### Design lens

- **Apply MDA** (Mechanics-Dynamics-Aesthetics) when reasoning about new mechanics.
- **North star**: "A small number of interlocking systems that generate surprising, meaningful decisions."
- **Core fantasy**: Discovering and executing clever spell/skill combos. Every system must serve this.
- **When in doubt**: Cut features, deepen systems.
- **No luck**: Perfect information / pure strategy. No dice, no hidden cards, no randomness.
- **Spending tension (G8)**: Players must always want to do more than they can execute.

### File naming

- **SCREAMING_SNAKE** for top-level living docs that act as named sections of the project: `README.md`, `CLAUDE.md`, `STATUS.md`, `NEXT_STEPS.md`, `OPEN_QUESTIONS.md`, `OPEN_QUESTIONS_ARCHIVE.md`, `SESSION_LOG.md`, `MEMORY.md`, `TESTING_PLAN.typ`.
- **kebab-case** for everything else: rule files, stack folders, design docs, research files, image assets.
- **snake_case** for Typst function names (`section-quick-reference` is the existing exception; new functions follow snake_case).

## Justification Rule (MANDATORY)

**Every new idea, mechanic, rule, or system change must explicitly justify its purpose.** Before staging an idea in `backpocket.md`, before proposing a layer, before drafting a stack, the entry must answer:

> **What current problem / "uncoolheit" does this fix, OR what specific aspect of game feel does this improve?**

If the answer is "it sounds cool" or "it would add more options" — that is NOT enough. Variety is not a justification on its own. The idea must point to:
- A concrete observed issue (e.g., "standoff zone", "Bodyguard never triggered", "draft converges to one meta"), OR
- A specific game-feel improvement tied to the core fantasy (e.g., "creates a sente threat that dissolves passive play", "rewards multi-turn setup more than brute force").

Format every new entry in `backpocket.md` with a **"What it fixes / improves"** field at the top, alongside the existing trigger condition. Ideas that cannot pass this test should be discarded, not parked. The backpocket is for *anticipated solutions*, not a graveyard of "could be cool."

This rule applies retroactively to any new idea I propose unprompted, and to any user idea I capture into the doc — if the user's pitch doesn't include the justification, I must ask before writing it down.

---

## Incremental Testing Methodology (MANDATORY)

**Never propose changing multiple interacting systems at once.**

When designing changes, follow this process:

1. **Decompose** proposed changes into independent layers. Two changes are independent if changing one does not affect how you evaluate the other.
2. **Identify coupling**: If change A affects how change B plays out, they are coupled. Coupled changes can be bundled into one test stack, but document WHY they're coupled.
3. **Order stacks** from most independent / highest impact to most dependent. Test the foundation first.
4. **Write a rule sheet** for each test stack in `docs/test-scenarios/`:
   - Import `baseline-sections.typ` and call section functions for all unchanged rules.
   - For Quick Reference, prefer `section-quick-reference(overrides: (...), extra-rows: (...))` over inlining the table.
   - Inline only the genuinely changed section in the test file with a `⚡ CHANGED:` callout and before/after table.
   - Copy `feedback-baseline.typ` to `stack-X-feedback.typ` and fill in `[STACK: ...]` placeholders.
5. **One stack per playtest.** After each test, evaluate results before proceeding to the next stack.
6. **Document which stack produced which result.** Never attribute an effect to a change that was bundled with other changes unless you can isolate the cause.

**Why this matters**: In Session 1, a monolithic change (board size, HP, turn structure, economy, piece count, bodyguard) was proposed. That makes it impossible to know which change caused which effect. Don't repeat it.

---

## Hygiene principles (lessons from Session 17 rework)

These are the patterns that produced the drift addressed in the Session 17 rework. Each one is a rule plus the underlying *why*. Treat them as load-bearing — the rework should not be needed twice.

1. **One source of truth per fact, with pointers — never restatements.** If a rule number, mechanic, or schema lives in `baseline-sections.typ` (or any other canonical file), other docs must point at it, not re-state it. Restated facts always drift. When summarising, label the summary as such and link to the canonical source on the next line.

2. **State docs need lifecycle, not just append.** `OPEN_QUESTIONS.md`, `NEXT_STEPS.md`, `mechanics-evaluated.md` are working docs, not journals. Resolved items must be archived (separate file) or transformed (linked from accepted decisions), not left in-place with status tags. Append-only lists become unreadable.

3. **Cross-link by ID, don't restate verdicts.** When two docs reference the same decision (an OQ, a mechanic, a playtest), one is canonical and the other links by ID. Never copy verdicts between docs — they will diverge.

4. **Vocabulary renames must be project-wide and atomic.** When renaming a concept (e.g. "layer" → "stack"), do a full grep + rename pass in the same commit. Half-renames stay broken for months because the old name still works for the original author.

5. **Skills must reference real, current paths.** Before adding or editing a skill, verify every file path it reads or writes exists. Skills with ghost references silently fail or create files in the wrong place. Treat skill paths as load-bearing.

6. **Memory is for immutable facts, not current state.** Memories should read as historical claims ("Session 3 ruling: starting Runes was 4 at that time"), not as live state ("starting Runes is 4"). Anything pointing at "current" rots — let project docs own current state.

7. **Templates that get copy-pasted should be functions instead.** If three stack files copy a 14-row table to edit one row, the template has failed. Parameterize. Apply to: rule sections, feedback forms, anything else with hand-copied boilerplate.

8. **Build scripts should discover, not enumerate.** Hardcoded file lists go stale every time a new file is added. Use `find` / glob patterns where the convention is stable enough.

9. **Justify before staging, archive before piling.** The Justification Rule prevents idea accumulation; an analogous archive rule prevents resolution accumulation. Both are forms of the same discipline: don't let working docs become graveyards.

10. **CLAUDE.md is for orientation, not facts.** It should tell a future session *where to find* the canonical answer, not *what* the canonical answer is. Anything quantitative in CLAUDE.md is a future drift point.

---

## Skills (Slash Commands)

| Command | Trigger | Description |
|---------|---------|-------------|
| `/start` | User only | Session start — reads STATUS.md + living docs, presents status briefing |
| `/wrapup` | User only | Session end — updates STATUS.md / NEXT_STEPS.md / SESSION_LOG.md, commits |
| `/research <topic>` | Auto or user | Generates Perplexity research prompt with project context. Auto-triggers on knowledge gaps about game design, comparable games, or player psychology. |
| `/playtest <N>` | Auto or user | Transcribes and analyses playtest photos/scans. Auto-triggers when user mentions playtest results. |
| `/scenario <stack-X> <desc>` | Auto or user | Creates full standalone test scenario rule sheet + feedback form. Auto-triggers when a design discussion yields a testable change. |
| `/adr <topic>` | Auto or user | Creates Architecture Decision Record. Auto-triggers when multiple valid design approaches emerge. |
| `/build-pdfs` | User only | Runs `docs/test-scenarios/build-pdfs.sh` (discovery-based — picks up new `.typ` files automatically). |

## Baseline Rule Files (read-only reference)

These are the original pre-migration rule documents in `old-game-versions/v3-first-board-game/md-converted/`. They are outdated (12x12 board, old Rune timing, terrain effects) and exist only for historical reference. Do not treat them as current rules.
