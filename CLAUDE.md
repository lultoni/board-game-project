# (GAME NAME) - Board Game Design Project

## Project Overview

A 2-player abstract-tactical board game where players command armies of Guards and Champions led by a King across a 10x10 grid. Victory comes through tactical superiority and capturing the enemy King. The design philosophy prioritises depth through interlocking systems over breadth of features.

## Architecture

```
baseline-rules/md-converted/   <- Original rule documents (read-only reference, pre-migration)
docs/
  research/                    <- Exported Perplexity research threads
  systems/                     <- One .md file per game system (populated Session 8)
    turn-structure.md          <- Turn Structure system
    resource-economy.md        <- Resource Economy (Runes) system
    progression.md             <- Progression (Skill Slots) system
    skill-system.md            <- Skill System (catalogue, paths, combos)
    combat-attack.md           <- Combat / Attack system (standard attacks, Bodyguard)
    health-armor.md            <- Health & Armor system
    skill-drafting.md          <- Skill Drafting system
  mechanics-log/
    mechanics-evaluated.md     <- Running log of mechanics considered, accepted, deferred, or discarded
  decisions/                   <- ADR-style design decision records
  brainstorm/
    session-log.md             <- Running session history
  backpocket.md                <- Pre-thought fixes for anticipated problems (staged, not active)
  core-loops/                  <- (reserved) Core loop diagrams
  test-scenarios/
    build-pdfs.sh                <- Run this to rebuild all PDFs
    shared/
      template.typ               <- Shared Typst styling (import in all .typ files as ../shared/template.typ)
      feedback-baseline.typ      <- Feedback form template (copy for each new layer)
      game-tracking.typ          <- Per-player in-game tracking sheet (print 1 per player per game)
    baseline/
      ruleset-baseline.typ       <- CANONICAL PLAYER-FACING RULES (source of truth for all rule text)
    layer-N-<desc>/              <- One subfolder per test layer
      layer-N-<desc>.typ         <- Rule sheet (Typst source)
      layer-N-feedback.typ       <- Feedback form (Typst source)
      layer-N-<desc>.pdf         <- Compiled PDF
      layer-N-feedback.pdf       <- Compiled PDF
game-state/
  CURRENT_DESIGN.md            <- Living master design document — summary index pointing to docs/systems/
  OPEN_QUESTIONS.md            <- Unresolved design questions
  NEXT_STEPS.md                <- Prioritised action items
```

## Key Game Systems

Full system documentation lives in `docs/systems/` (one file per system). Summary:

1. **Turn Structure**: Round-based (P1 turn → P2 turn). Each turn: Movement Phase (2 Move Slots) → Action Phase (starting 2 Skill Slots). Details: [`docs/systems/turn-structure.md`](docs/systems/turn-structure.md)
2. **Piece Types**: King (1), Champions (5), Guards (6) per player. Guards are faster (speed 2) but simpler; Champions/King are slower (speed 1) but carry skills.
3. **Skill System**: Champions and King equip 2 skills each during drafting. Skills cost Runes. Categories: Strike, Shield, Move, Mystic. Skills use queen-like line-of-sight paths, blocked by all pieces. Details: [`docs/systems/skill-system.md`](docs/systems/skill-system.md)
4. **Resource Economy**: Runes (currency for skills). Start with 6 (Layer 1 accepted). Gain scales over time (+2/turn from Round 2, +1 every 5 rounds). Details: [`docs/systems/resource-economy.md`](docs/systems/resource-economy.md)
5. **Health System**: 2 HP per piece (Normal → Injured → Removed). Injured reduces Guard speed and skill range. Armor absorbs damage (max 3). Details: [`docs/systems/health-armor.md`](docs/systems/health-armor.md)
6. **Board**: 10x10 grid. No terrain effects (removed — confirmed overhead complexity).
7. **Bodyguard Rule**: Guard adjacent to both tile-before-target (along attack path) AND defender can intercept a Standard Attack on a Champion/King. Guard takes the damage. Details: [`docs/systems/combat-attack.md`](docs/systems/combat-attack.md)

## Conventions

- **Source of truth for design decisions**: `game-state/CURRENT_DESIGN.md` (summary index) + `docs/systems/` (per-system detail) — always update after design decisions
- **Source of truth for rule text**: `docs/test-scenarios/baseline/ruleset-baseline.typ` (Typst source) / `docs/test-scenarios/baseline/ruleset-baseline.pdf` (printable)
- **Research protocol**: Use `/research <topic>` skill for external knowledge — generates formatted Perplexity prompts with project context
- **Design lens**: Always apply MDA (Mechanics-Dynamics-Aesthetics) framework
- **North star**: "A small number of interlocking systems that generate surprising, meaningful decisions"
- **Core fantasy**: Discovering and executing clever spell/skill combos. Every system must serve this.
- **When in doubt**: Cut features, deepen systems
- **No luck**: Perfect information / pure strategy. No dice, no hidden cards, no randomness.
- **Spending tension (G8)**: Players must always want to do more than they can execute. Early: Runes limit. Mid/late: Skill Slots limit while Rune costs force choosing WHICH skills. If either resource becomes so abundant that spending requires no tradeoff, something is broken.

## Incremental Testing Methodology (MANDATORY)

**Never propose changing multiple interacting systems at once.**

When designing changes, follow this process:

1. **Decompose** proposed changes into independent layers. Two changes are independent if changing one does not affect how you evaluate the other.
2. **Identify coupling**: If change A affects how change B plays out, they are coupled. Coupled changes can be bundled into one test layer, but document WHY they're coupled.
3. **Order layers** from most independent / highest impact to most dependent. Test the foundation first.
4. **Write a rule sheet** for each test layer in `docs/test-scenarios/`. **Use the copy-baseline workflow**:
   - Start from `docs/test-scenarios/typ-files/ruleset-baseline.typ` (or from the most recent accepted layer's rule sheet if building on it).
   - Copy to `layer-N-<desc>.typ`.
   - Replace ONLY the changed section. Mark it clearly with a `⚡ CHANGED:` callout and a before/after table.
   - All unchanged sections: reference baseline (`*All other rules: see docs/test-scenarios/pdf-files/ruleset-baseline.pdf*`) rather than repeating them.
   - Copy `feedback-baseline.typ` to `layer-N-feedback.typ` and fill in the `[LAYER: ...]` placeholders.
5. **One layer per playtest.** After each test, evaluate results before proceeding to the next layer.
6. **Document which layer produced which result.** Never attribute an effect to a change that was bundled with other changes unless you can isolate the cause.

**Why this matters**: In Session 1, ADR-002 proposed changing board size, HP, turn structure, economy, piece count, and bodyguard simultaneously. This makes it impossible to know which change caused which effect. Elias caught this. Don't repeat it.

## Skills (Slash Commands)

| Command | Trigger | Description |
|---------|---------|-------------|
| `/start` | User only | Session start — reads all living docs, checks for new files, presents status briefing |
| `/wrapup` | User only | Session end — updates all living docs, session log, and handover prompt |
| `/research <topic>` | Auto or user | Generates Perplexity research prompt with project context. Auto-triggers on knowledge gaps about game design, comparable games, or player psychology. |
| `/playtest <N>` | Auto or user | Transcribes and analyses playtest photos/scans. Auto-triggers when user mentions playtest results. |
| `/scenario <layer-N> <desc>` | Auto or user | Creates full standalone test scenario rule sheet. Auto-triggers when a design discussion yields a testable change. |
| `/adr <topic>` | Auto or user | Creates Architecture Decision Record. Auto-triggers when multiple valid design approaches emerge. |

## Baseline Rule Files (read-only reference)

These are the original pre-migration rule documents in `baseline-rules/md-converted/`. They are outdated (12x12 board, old Rune timing, terrain effects) and exist only for historical reference. Do not treat them as current rules.

- `(GAME NAME) - Revised Rulebook.md` - Original main rulebook
- `DRAFT Player Feedback Sheet.md` - Original feedback form (superseded by `feedback-baseline.typ`)
- `Rules Cheat Sheet_ (GAME NAME).md` - Condensed rules reference
- `Systems to Test.md` - Original design variants list
- `Project ROE Skills.md` - Extended skill catalogue with German translations and variants
- `quick list skills export gemini.md` - Quick reference skill table
