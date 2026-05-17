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
  research/                      <- Exported Perplexity research threads
  mechanics-log/
    mechanics-evaluated.md       <- Decision registry: every mechanic considered + status
  test-scenarios/
    build-pdfs.sh                <- Run this to rebuild all PDFs (use zsh)
    TESTING_PLAN.typ/.pdf        <- Decision tree: which stack to run next
    shared/
      template.typ               <- Shared Typst styling (import as ../shared/template.typ)
      baseline-sections.typ      <- Parameterless baseline section functions
      feedback-baseline.typ      <- Feedback form template (copy for each new stack)
      game-tracking.typ          <- Per-player in-game tracking sheet
    baseline/
      ruleset-baseline.typ       <- CANONICAL PLAYER-FACING RULES (source of truth for all rule text)
    stack-X-<desc>/              <- One subfolder per test stack
      stack-X-<desc>.typ/.pdf    <- Rule sheet (Typst source + compiled PDF)
      stack-X-feedback.typ/.pdf  <- Feedback form
game-state/
  OPEN_QUESTIONS.md              <- Unresolved design questions
  NEXT_STEPS.md                  <- Prioritised action items
old-game-versions/
  README.md                      <- The Ultimate Game Timeline (full history + session log)
```

## Key Game Systems

Full system documentation lives in `docs/systems-and-mechanics.md`. Summary:

1. **Turn Structure**: Round-based (P1 turn → P2 turn). Each turn: Movement Phase (2 Move Slots) → Action Phase (starting 2 Skill Slots).
2. **Piece Types**: King (1), Champions (5), Guards (6) per player. Guards are faster (speed 2) but simpler; Champions/King are slower (speed 1) but carry skills.
3. **Skill System**: Champions and King equip 2 skills each during drafting. Skills cost Runes. Categories: Strike, Shield, Move, Mystic. Skills use queen-like line-of-sight paths, blocked by all pieces.
4. **Resource Economy**: Runes (currency for skills). Start with 6 (Layer 1 accepted). Gain scales over time (+2/turn from Round 2, +1 every 5 rounds).
5. **Health System**: 2 HP per piece (Normal → Injured → Removed). Injured reduces Guard speed and skill range. Armor absorbs damage (max 3).
6. **Board**: 10x10 grid. No terrain effects (removed — confirmed overhead complexity).
7. **Bodyguard Rule**: Guard adjacent to both tile-before-target (along attack path) AND defender can intercept a Standard Attack on a Champion/King. Guard takes the damage.

## Conventions

- **Source of truth for rule text**: `docs/test-scenarios/baseline/ruleset-baseline.typ` (Typst source) / `.pdf` (printable)
- **Source of truth for design principles**: `docs/design-principles.md`
- **Source of truth for systems detail**: `docs/systems-and-mechanics.md`
- **Decision history**: `docs/mechanics-log/mechanics-evaluated.md`
- **Research protocol**: Use `/research <topic>` skill for external knowledge — generates formatted Perplexity prompts with project context
- **Design lens**: Always apply MDA (Mechanics-Dynamics-Aesthetics) framework
- **North star**: "A small number of interlocking systems that generate surprising, meaningful decisions"
- **Core fantasy**: Discovering and executing clever spell/skill combos. Every system must serve this.
- **When in doubt**: Cut features, deepen systems
- **No luck**: Perfect information / pure strategy. No dice, no hidden cards, no randomness.
- **Spending tension (G8)**: Players must always want to do more than they can execute. Early: Runes limit. Mid/late: Skill Slots limit while Rune costs force choosing WHICH skills. If either resource becomes so abundant that spending requires no tradeoff, something is broken.

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
2. **Identify coupling**: If change A affects how change B plays out, they are coupled. Coupled changes can be bundled into one test layer, but document WHY they're coupled.
3. **Order layers** from most independent / highest impact to most dependent. Test the foundation first.
4. **Write a rule sheet** for each test stack in `docs/test-scenarios/`:
   - Import `baseline-sections.typ` and call parameterless section functions for all unchanged rules.
   - Inline the changed section directly in the test file with a `⚡ CHANGED:` callout and before/after table.
   - Copy `feedback-baseline.typ` to `stack-X-feedback.typ` and fill in placeholders.
5. **One stack per playtest.** After each test, evaluate results before proceeding to the next stack.
6. **Document which stack produced which result.** Never attribute an effect to a change that was bundled with other changes unless you can isolate the cause.

**Why this matters**: In Session 1, a monolithic change (board size, HP, turn structure, economy, piece count, bodyguard) was proposed. That makes it impossible to know which change caused which effect. Don't repeat it.

## Skills (Slash Commands)

| Command | Trigger | Description |
|---------|---------|-------------|
| `/start` | User only | Session start — reads all living docs, checks for new files, presents status briefing |
| `/wrapup` | User only | Session end — updates all living docs, timeline entry, commits |
| `/research <topic>` | Auto or user | Generates Perplexity research prompt with project context. Auto-triggers on knowledge gaps about game design, comparable games, or player psychology. |
| `/playtest <N>` | Auto or user | Transcribes and analyses playtest photos/scans. Auto-triggers when user mentions playtest results. |
| `/scenario <stack-X> <desc>` | Auto or user | Creates full standalone test scenario rule sheet. Auto-triggers when a design discussion yields a testable change. |
| `/adr <topic>` | Auto or user | Creates Architecture Decision Record. Auto-triggers when multiple valid design approaches emerge. |

## Baseline Rule Files (read-only reference)

These are the original pre-migration rule documents in `old-game-versions/v3-first-board-game/md-converted/`. They are outdated (12x12 board, old Rune timing, terrain effects) and exist only for historical reference. Do not treat them as current rules.
