# (GAME NAME) — Board Game Design Project

A 2-player abstract-tactical board game in active development. Two players command armies of Guards and Champions led by a King across a 10×10 grid, equipping Champions with skills and spending Runes to activate them. Victory by capturing the enemy King.

**Design identity**: The intersection of chess-like spatial tactics and CCG-style build customisation. The core fantasy is discovering and executing clever skill combos. Everything else is chassis.

---

## Quick Navigation

| I want to... | Go here |
|---|---|
| Understand the current design | [`game-state/CURRENT_DESIGN.md`](game-state/CURRENT_DESIGN.md) |
| See what needs doing next | [`game-state/NEXT_STEPS.md`](game-state/NEXT_STEPS.md) |
| See unresolved design questions | [`game-state/OPEN_QUESTIONS.md`](game-state/OPEN_QUESTIONS.md) |
| Read the canonical rules (printable) | [`docs/test-scenarios/baseline/ruleset-baseline.pdf`](docs/test-scenarios/baseline/ruleset-baseline.pdf) |
| See the full testing plan | [`docs/test-scenarios/TESTING_PLAN.pdf`](docs/test-scenarios/TESTING_PLAN.pdf) |
| Print materials for the next playtest | [`docs/test-scenarios/stack-a-cleverness/`](docs/test-scenarios/stack-a-cleverness/) |
| Read about a specific game system | [`docs/systems/`](docs/systems/) |
| See design decisions with reasoning | [`docs/decisions/`](docs/decisions/) |
| See what has been tried and why | [`docs/mechanics-log/mechanics-evaluated.md`](docs/mechanics-log/mechanics-evaluated.md) |
| Read session history | [`docs/brainstorm/session-log.md`](docs/brainstorm/session-log.md) |
| Resume a Claude Code session | [`HANDOVER.md`](HANDOVER.md) — paste into new session |

---

## Project Structure

```
board-game-project/
│
├── README.md                        ← you are here
├── CLAUDE.md                        ← project conventions for Claude Code (read this)
├── HANDOVER.md                      ← copy-paste into Claude to resume where you left off
│
├── game-state/                      ← living documents — always up to date
│   ├── CURRENT_DESIGN.md            ← master design summary (index into docs/systems/)
│   ├── NEXT_STEPS.md                ← prioritised action items
│   └── OPEN_QUESTIONS.md            ← every unresolved question with status + triggers
│
├── docs/
│   ├── systems/                     ← one file per game system; full rules + MDA + health
│   │   ├── turn-structure.md
│   │   ├── resource-economy.md
│   │   ├── progression.md
│   │   ├── skill-system.md
│   │   ├── combat-attack.md
│   │   ├── health-armor.md
│   │   └── skill-drafting.md
│   │
│   ├── decisions/                   ← ADR-style design decision records
│   │   ├── ADR-001-game-architecture-direction.md
│   │   ├── ADR-002-direction-a-plus.md
│   │   └── ADR-003-rewarding-cleverness.md
│   │
│   ├── test-scenarios/              ← printable rule sheets + feedback forms (Typst → PDF)
│   │   ├── build-pdfs.sh            ← run this to rebuild all PDFs
│   │   ├── TESTING_PLAN.pdf         ← decision tree: which stack to run next
│   │   ├── shared/                  ← reusable Typst components
│   │   │   ├── template.typ         ← shared styling (imported by all .typ files)
│   │   │   ├── baseline-sections.typ ← 16 parameterised section functions
│   │   │   ├── feedback-baseline.typ ← feedback form template
│   │   │   └── game-tracking.typ    ← per-player in-game tracking sheet
│   │   ├── baseline/                ← canonical ruleset (source of truth for rule text)
│   │   ├── accepted-layer-1-economy/ ← accepted: economy fix (now baked into baseline)
│   │   ├── stack-a-cleverness/      ← READY TO PLAY: attack nerf + combo bonus
│   │   ├── stack-b-guards/          ← READY TO PLAY: bodyguard fix
│   │   └── stack-g-structure/       ← DRAFT: unified AP framework
│   │
│   ├── research/                    ← Perplexity research exports + playtest analyses
│   ├── mechanics-log/
│   │   └── mechanics-evaluated.md   ← every mechanic ever considered: status + reasoning
│   ├── brainstorm/
│   │   └── session-log.md           ← full history of every design session
│   └── backpocket.md                ← pre-thought fixes for anticipated problems (not active)
│
├── playtest-results/                ← photos of handwritten feedback + game logs
│   ├── elias-vs-pasco-31_10_25/
│   └── elias-vs-jonathan-24_04_26/
│
├── images/                          ← skill card images + other assets
├── old-game-versions/               ← archived earlier game iterations (read-only reference)
│
└── .claude/                         ← Claude Code project config
    ├── settings.local.json          ← permissions + hooks
    ├── hooks/session-start.sh       ← injects skill-trigger reminders each session
    └── skills/                      ← custom slash commands (see "Working with Claude" below)
        ├── start/SKILL.md
        ├── wrapup/SKILL.md
        ├── research/SKILL.md
        ├── playtest/SKILL.md
        ├── scenario/SKILL.md
        ├── adr/SKILL.md
        └── build-pdfs/SKILL.md
```

---

## The Game in 2 Minutes

**Players**: 2  
**Board**: 10×10 grid, no terrain  
**Win condition**: Capture the enemy King  

### Pieces (per player)
| Piece | Count | Speed | Skills | HP |
|---|---|---|---|---|
| King | 1 | 1 | 2 slots | 2 |
| Champion | 5 | 1 | 2 slots | 2 |
| Guard | 6 | 2 | — | 2 |

### How a Turn Works
1. **Movement Phase** — move up to 2 pieces (each piece once)
2. **Action Phase** — activate up to N skills (limited by Skill Slots, paid in Runes)

### Key Systems
- **Runes** — the currency. Gain +2/turn from Round 2 (starts at 6). Spend to activate skills.
- **Skills** — equipped during pre-game draft. Champions/King get 2 skill slots each. Skills use line-of-sight paths blocked by all pieces.
- **2 HP** — Normal → Injured → Removed. Injured reduces Guard speed and skill range.
- **Bodyguard** — a Guard adjacent to an attacked Champion/King can intercept standard attacks.
- **Standard Attack** — move onto an enemy tile to deal 1 DMG (Stack A change, being tested).

Full rules: [`docs/test-scenarios/baseline/ruleset-baseline.pdf`](docs/test-scenarios/baseline/ruleset-baseline.pdf)

---

## Design Principles

1. **Core fantasy first** — "Does this make skill combos more interesting?" is the test for every system.
2. **Perfect information** — no dice, no hidden cards, no randomness.
3. **Depth over breadth** — cut features, deepen systems.
4. **Incremental testing** — never change multiple interacting systems at once. One stack = one experience outcome.
5. **MDA lens** — always evaluate mechanics through Mechanics → Dynamics → Aesthetics.
6. **Shared-puzzle feel** — winning should feel like "I found the better solution," not "I crushed you."

---

## Current Status (Session 10)

| Layer / Stack | Topic | Status |
|---|---|---|
| Layer 1 — Economy Fix | 6 start Runes, +2/turn | **ACCEPTED** (Playtest 2) |
| **Stack A — Cleverness** | Attack nerf + combo bonus | **Ready to print and play** |
| Stack B — Guards | Bodyguard adjacency fix | Ready to print and play |
| Stack C — Pacing | Checkmate win condition | Not yet written |
| Stack D — Board | 8×8 / hex | Not yet written |
| Stack E — Draft | Pool draft, placement order | Not yet written |
| Stack F — Cleverness II | More combo levers | Not yet written |
| Stack G — Structure | Unified AP framework | Draft written |

**Immediate next action**: Print and play Stack A (files in `docs/test-scenarios/stack-a-cleverness/`).

---

## Working with Claude Code

This project uses Claude Code as an AI design collaborator. A session works like this:

### Starting a Session
Run `/start` in Claude Code. It reads all living documents and presents a status briefing. Alternatively, paste the contents of `HANDOVER.md` into a new session.

### Ending a Session
Run `/wrapup`. It updates all living documents, writes the session log entry, updates `HANDOVER.md`, and commits + pushes changes.

### Custom Skills (Slash Commands)

| Command | When to use |
|---|---|
| `/start` | Begin a design session — reads living docs, presents briefing |
| `/wrapup` | End a session — updates all docs, commits, pushes |
| `/research <topic>` | Need external knowledge about game design, comparable games, player psychology |
| `/adr <topic>` | Multiple valid design approaches exist and need formal comparison |
| `/scenario <stack-X> <desc>` | Design discussion ends with a testable, isolated change |
| `/playtest <N>` | Analyse playtest results from photos in `playtest-results/` |
| `/build-pdfs` | Rebuild all PDFs from Typst source |

Skills auto-trigger when context matches (e.g. `/research` fires when a knowledge gap is identified). You can also invoke them manually.

### Source of Truth Hierarchy
1. **Rule text**: `docs/test-scenarios/baseline/ruleset-baseline.typ` (Typst source) / `.pdf` (printable)
2. **Design decisions**: `game-state/CURRENT_DESIGN.md` (index) + `docs/systems/` (per-system detail)
3. **What to do next**: `game-state/NEXT_STEPS.md`
4. **Historical reasoning**: `docs/decisions/ADR-*.md` and `docs/brainstorm/session-log.md`

---

## How Rule Sheets Are Made

Rule sheets use **Typst** (a modern typesetting system) and a composable section system:

- `docs/test-scenarios/shared/baseline-sections.typ` — 16 parameterised `#let` functions, one per ruleset section.
- Each stack file is ~50 lines: it imports `baseline-sections.typ`, calls each section function, and overrides only what changes.
- Changing the baseline propagates automatically to all stack files — no manual copy-paste.
- Changes are marked with a `⚡ CHANGED:` callout and a before/after table in the relevant section.

**To rebuild all PDFs**: run `docs/test-scenarios/build-pdfs.sh` (requires [Typst](https://typst.app) installed).

---

## Playtest Methodology

Testing follows an incremental, evidence-driven stack system:

1. **One stack = one experience outcome** (e.g. "make skill combos dominant strategy").
2. **Each stack contains 1–2 game variants** — the minimum change needed to test the hypothesis.
3. **After each playtest**, consult `TESTING_PLAN.pdf` to pick the highest-value next stack based on results.
4. **Never change multiple interacting systems at once.** If you can't isolate the cause of an effect, the test is invalid.

Playtest materials per session: rule sheet(s) + feedback form + game tracking sheet (1 per player).

Results go in `playtest-results/<players>-<date>/` as photo scans. Run `/playtest <N>` to transcribe and analyse.

---

## Tooling Requirements

| Tool | Purpose | Install |
|---|---|---|
| [Claude Code](https://claude.ai/code) | AI design collaborator + session workflow | `npm install -g @anthropic-ai/claude-code` |
| [Typst](https://typst.app) | Compiles `.typ` rule sheets to PDF | `brew install typst` |
| [Pandoc](https://pandoc.org) | Markdown → PDF conversion (secondary) | `brew install pandoc` |

---

## Old Game Versions (Historical Reference Only)

`old-game-versions/` contains archived material from earlier iterations of the game going back to 2023. Three subfolders correspond to distinct eras:

- `UPLOAD - Realm of Elements/` — the original concept with elemental mages on an 8×8 board (2023)
- `Outdated ROE/` — Project ROE: a redesign with Champion classes, tile control, and a digital Java implementation (2023–2024)
- `first-boardgame-oriented-concept/` — the first pure board game ruleset: 12×12 terrain board, Guards, Champions, King, Runes, Skills (2024–2025, pre-migration)

See [`old-game-versions/README.md`](old-game-versions/README.md) for the full game history and a timeline of how the design evolved.
