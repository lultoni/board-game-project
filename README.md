# (GAME NAME) — Board Game Design Project

A 2-player abstract-tactical board game in active development. Two players command armies of Guards and Champions led by a King across a 10×10 grid, equipping Champions with skills and spending Runes to activate them. Victory by capturing the enemy King.

**Design identity**: The intersection of chess-like spatial tactics and CCG-style build customisation. The core fantasy is discovering and executing clever skill combos. Everything else is chassis.

---

## Quick Navigation

| I want to... | Go here |
|---|---|
| Read the canonical rules (printable) | [`docs/test-scenarios/baseline/ruleset-baseline.pdf`](docs/test-scenarios/baseline/ruleset-baseline.pdf) |
| Understand design principles | [`docs/design-principles.md`](docs/design-principles.md) |
| Understand how each system works (MDA, health, open questions) | [`docs/systems-and-mechanics.md`](docs/systems-and-mechanics.md) |
| See what needs doing next | [`game-state/NEXT_STEPS.md`](game-state/NEXT_STEPS.md) |
| See unresolved design questions | [`game-state/OPEN_QUESTIONS.md`](game-state/OPEN_QUESTIONS.md) (live) / [`game-state/OPEN_QUESTIONS_ARCHIVE.md`](game-state/OPEN_QUESTIONS_ARCHIVE.md) (resolved) |
| Re-enter the project after a gap | [`game-state/STATUS.md`](game-state/STATUS.md) |
| Read the per-session narrative log | [`game-state/SESSION_LOG.md`](game-state/SESSION_LOG.md) |
| See the full testing plan | [`docs/test-scenarios/TESTING_PLAN.pdf`](docs/test-scenarios/TESTING_PLAN.pdf) |
| Print materials for the next playtest | [`docs/test-scenarios/stack-a-cleverness/`](docs/test-scenarios/stack-a-cleverness/) |
| See what has been tried and why | [`docs/mechanics-log/mechanics-evaluated.md`](docs/mechanics-log/mechanics-evaluated.md) |
| See future visual/identity direction | [`docs/game-identity-visual-naming.md`](docs/game-identity-visual-naming.md) |
| Read the full game history | [`old-game-versions/README.md`](old-game-versions/README.md) |
| Pre-thought fixes for anticipated problems | [`docs/backpocket.md`](docs/backpocket.md) |

---

## Project Structure

```
board-game-project/
│
├── README.md                        ← you are here
├── CLAUDE.md                        ← project conventions for Claude Code
│
├── game-state/                      ← living documents — always up to date
│   ├── NEXT_STEPS.md                ← prioritised action items
│   └── OPEN_QUESTIONS.md            ← every unresolved question with status + triggers
│
├── docs/
│   ├── design-principles.md         ← rules to design by (5 principles, constraints, methodology)
│   ├── systems-and-mechanics.md     ← all 7 systems: how they work, MDA, health, open questions
│   ├── game-identity-visual-naming.md ← future Phase B: art, naming, physical design direction
│   ├── backpocket.md                ← pre-thought fixes for anticipated problems (staged, not active)
│   │
│   ├── test-scenarios/              ← printable rule sheets + feedback forms (Typst → PDF)
│   │   ├── build-pdfs.sh            ← run this to rebuild all PDFs
│   │   ├── TESTING_PLAN.typ/.pdf    ← decision tree: which stack to run next
│   │   ├── shared/                  ← reusable Typst components
│   │   │   ├── template.typ         ← shared styling
│   │   │   ├── baseline-sections.typ ← parameterless baseline section functions
│   │   │   ├── feedback-baseline.typ ← feedback form template
│   │   │   └── game-tracking.typ    ← per-player in-game tracking sheet
│   │   ├── baseline/                ← canonical ruleset (source of truth for rule text)
│   │   ├── stack-a-cleverness/      ← READY TO PLAY: attack nerf + combo bonus
│   │   ├── stack-b-guards/          ← READY TO PLAY: bodyguard fix
│   │   └── stack-g-structure/       ← DRAFT: unified AP framework
│   │
│   ├── research/                    ← Perplexity research exports + playtest analyses
│   └── mechanics-log/
│       └── mechanics-evaluated.md   ← decision registry: every mechanic considered + status
│
├── playtest-results/                ← photos of handwritten feedback + game logs
│   ├── elias-vs-pasco-31_10_25/
│   └── elias-vs-jonathan-24_04_26/
│
├── images/                          ← skill card images (15 skills)
│
├── old-game-versions/               ← archived earlier iterations + full game timeline
│   ├── README.md                    ← The Ultimate Game Timeline (2023–present)
│   ├── v1-realm-of-elements/
│   ├── v2-project-roe/
│   └── v3-first-board-game/
│
└── .claude/                         ← Claude Code project config
    ├── HANDOVER.md                  ← session bookmark (paste into new session)
    ├── settings.local.json
    ├── hooks/session-start.sh
    └── skills/                      ← custom slash commands
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
- **Runes** — currency. Start 6, gain +2/turn from Round 2 (scaling). Spend to activate skills.
- **Skills** — equipped during pre-game draft. 2 slots per Champion/King. Line-of-sight paths blocked by all pieces.
- **2 HP** — Normal → Injured → Removed. Injured reduces Guard speed and skill range.
- **Bodyguard** — Guard adjacent to an attacked Champion/King can intercept standard attacks.
- **Standard Attack** — move onto enemy tile = 1 DMG (baseline, accepted Session 15).

Full rules: [`docs/test-scenarios/baseline/ruleset-baseline.pdf`](docs/test-scenarios/baseline/ruleset-baseline.pdf)

---

## Current Status (Session 18)

See [`game-state/STATUS.md`](game-state/STATUS.md) for the one-screen re-entry doc.

**Current focus**: Print packet for Nico's first game (2026-05-28, Stack A G2). Skill cards and onboarding feedback form shipped this session.

**Stack pipeline**:

| Stack | Topic | Status |
|---|---|---|
| Layer 1 — Economy Fix | 6 start Runes, +2/turn | **ACCEPTED** (Playtest 2) |
| Standard Attack 1 DMG | Attack nerf | **ACCEPTED** (Playtest 3) |
| **Stack A — Cleverness** | G1 nerf accepted; G2 combo bonus next | **G2 ready to play** |
| Stack B — Guards | Bodyguard adjacency fix | De-prioritised (may be obsolete after P3) |
| Stack C — Pacing | King Lifetime HP · Armor Decay | Trigger: first kill > R20 in G2 |
| Stack D — Board | 8×10 · 8×8 · Hex | Trigger: after Stack A |
| Stack E — Draft | Pool draft, placement order | Trigger: after Stack A |
| Stack F — Cleverness II | Cascade · Pin/Threatened · Sente | Trigger: Stack A G2 results |
| Stack G — Structure | Unified AP framework | Draft written |

---

## Working with Claude Code

### Starting a Session
Run `/start` in Claude Code. It reads all living documents and presents a status briefing.

### Ending a Session
Run `/wrapup`. It updates all living documents, writes the session entry into the timeline, and commits.

### Custom Skills

| Command | When to use |
|---|---|
| `/start` | Begin a design session |
| `/wrapup` | End a session — updates docs, commits |
| `/research <topic>` | Need external knowledge about game design |
| `/adr <topic>` | Multiple valid design approaches need comparison |
| `/scenario <stack-X> <desc>` | Design discussion yields a testable change |
| `/playtest <N>` | Analyse playtest results from photos |

### Source of Truth Hierarchy
1. **Rule text**: `docs/test-scenarios/baseline/ruleset-baseline.typ`
2. **Design principles**: `docs/design-principles.md`
3. **Systems detail**: `docs/systems-and-mechanics.md`
4. **What to do next**: `game-state/NEXT_STEPS.md`
5. **Decision history**: `docs/mechanics-log/mechanics-evaluated.md`
6. **Full game timeline**: `old-game-versions/README.md`

---

## How Rule Sheets Are Made

Rule sheets use **Typst** and a composable section system:

- `shared/baseline-sections.typ` — parameterless functions, one per baseline section.
- Each stack file calls baseline functions for unchanged sections and inlines its own changed sections with `⚡ CHANGED:` callouts.
- Changing the baseline propagates automatically to all stack files.

**To rebuild all PDFs**: run `zsh docs/test-scenarios/build-pdfs.sh` (requires [Typst](https://typst.app)).

> **PDF commit policy**: Compiled `.pdf` files for rule sheets, feedback forms, and the testing plan are committed to the repo. Collaborators without Typst installed can read them directly. The `.typ` source remains the source of truth — regenerate via the script after editing.

---

## Playtest Methodology

1. **One stack = one experience outcome** (e.g. "make skill combos dominant strategy").
2. **Each stack contains 1–2 game variants** — minimum change to test the hypothesis.
3. **After each playtest**, consult `TESTING_PLAN.pdf` to pick the highest-value next stack.
4. **Never change multiple interacting systems at once.** If you can't isolate causation, the test is invalid.

Results go in `playtest-results/<players>-<date>/`. Run `/playtest <N>` to analyse.

---

## Old Game Versions

`old-game-versions/` contains archived material from earlier iterations (2023–2025). See [`old-game-versions/README.md`](old-game-versions/README.md) for the full game history and session timeline.
