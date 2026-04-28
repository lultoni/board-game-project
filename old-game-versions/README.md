# Old Game Versions — History & Archive

This folder contains every previous iteration of the game, going back to its first spark in summer 2023. The material here is **read-only reference** — do not treat any of it as current rules. For current design, see [`game-state/CURRENT_DESIGN.md`](../game-state/CURRENT_DESIGN.md).

---

## Folder Overview

### `UPLOAD - Realm of Elements/`

The **original game concept**. Two players each control 5 elemental mages (Fire, Water, Earth, Air, Spirit) and 5 guards on an 8×8 terrain board. Spells are element-specific (15 total: attack/defense/utility per element). The Spirit Mage is the "king" — lose it and you lose the game. Economy: 5 spell tokens to start, +1 every turn scaling up every 5 rounds.

This version has full art assets: 12 character sprites (Red/Blue factions), 4 terrain tiles, board layouts. A German version (`DeutscheFassung.docx`) was also written. A YouTube-style **script** (`Realm Of Elements Script.docx`) was written to explain the game concept for a video — never recorded.

**Status at this stage**: Complete paper prototype with rules, art, and a pitch script. First uploaded to GitHub Oct 2023.

**Key GitHub repo**: [`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements) — Java/Swing desktop app with SQLite player rankings and Elo ratings. Included local multiplayer, all 15 spells implemented, full turn state machine, medieval music library (65+ tracks), guard prioritisation logic, and FEN-like board notation. Last pushed Jan 2024.

---

### `Outdated ROE/`

**Project ROE** — a major redesign and the attempt to build a full digital implementation in Java. The elemental-mage theme was kept but the mechanics were overhauled:
- Board expanded to 12×12 with terrain
- Pieces redesigned into 6 Champion **classes** (Offense, Defense, Mobility, Boost, Tile Control, a sixth TBD) — 3 Champions per class, 18 total to draft from
- Pick-and-ban draft phase added
- Champion class system: each class has unique rules (Defense = shield types; Tile Control = terrain effects; Offense = skill path rules)
- Skill system expanded (50+ skills including German and English catalogues)
- Digital AI using minimax with alpha-beta pruning and transposition tables

This folder has no md-converted counterpart (now fixed — all `.docx` files converted). Also contains: `Project-ROE Notizen.docx` (4 playtested games with notes), `Project-ROE Lore.docx` (full narrative + gods lore), `General changes.docx` (digital implementation design notes + AI architecture), `Gameplay loop changes.docx` (design thinking methodology).

**Status at this stage**: Rules complete, digital implementation ~85% done (full game loop, AI, all spells). Abandoned July 2024 due to development scope — building a polished game engine solo was too much work.

**Key GitHub repo**: [`lultoni/project-roe`](https://github.com/lultoni/project-roe) — Java implementation, minimax AI depth 1 (MCTS experimented with and abandoned), evaluation function with 12 weighted factors, 16 days of intensive development June–July 2024.

---

### `first-boardgame-oriented-concept/`

The **pivot to a pure board game** — stripping the digital ambitions and focusing entirely on the tabletop experience. This is the direct ancestor of the current game. Key changes from Outdated ROE:
- Dropped the 18-Champion class system → simpler Guards/Champions/King
- Kept terrain but simplified it
- Runes and Skill Slots formalised as the core economy
- Designed explicitly for physical play with print-ready feedback forms

This is the version that went to **Playtest 1** (Elias vs Pasco, Oct 2025). All files in `md-converted/` were already converted before this project migrated to the current design system.

**Also relevant**: [`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker) — a tiny web tool (HTML/CSS/JS) built Aug–Sep 2025 to track turns, Runes, and piece counts during playtests of this version. Shows the rules at that point: 5 starting Runes, +1/round scaling every 7 rounds, 2 skill slots scaling every 10 rounds. Last pushed Sep 2025.

---

## The Ultimate Game Timeline

*How this game grew from a school holiday idea into a serious design project.*

---

### June–July 2023 — The First Spark

End of 11th grade (Abitur year 1). The idea first appeared: *what if chess had magic?* The core concept was already clear — two players, a grid board, pieces that could cast spells, the "king" as the win condition. The spiritual ancestor was chess but the feel was meant to be something entirely new.

The first version had **elemental mages** (Fire, Water, Earth, Air, Spirit) replacing chess pieces. Guards replaced pawns. The Spirit Mage took the role of the King. Terrain (Forest, Mountain, Lake, Plains) gave each mage type advantages and disadvantages.

---

### Late 2023 — Unity Multiplayer Prototype

Before committing to a proper implementation, a quick Unity prototype was built — essentially a digital board where you could drag and drop pieces. No rule enforcement. You had to track everything yourself. Used for a handful of remote games with Jonathan (and once his cousin). Not preserved in this repo. This proved the concept was fun enough to keep working on.

---

### October 2023 — First GitHub Commit: `realm-of-elements`

The concept was formalised into a full Java/Swing desktop application ([`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements)). Features at this stage:
- 8×8 terrain board with FEN-like notation
- All 15 element spells implemented in code
- Elo rating system with SQLite persistence
- Medieval music library (65+ Age of Empires / Crusader Kings tracks)
- Turn state machine: Movement → Attack → repeat
- Guard prioritisation (intelligent defender selection)
- Move validation (no knight-like diagonal leaps)

This was a complete, playable local multiplayer game. Last pushed January 2024.

---

### June–July 2024 — Project ROE: Digital Ambitions Peak

With Abitur done (spring 2024) and before starting university, a full redesign began. The game became **Project ROE** — ambitious, complex, and ultimately too large for a solo project.

Major design changes:
- Board expanded to 12×12
- 18 Champions in 6 classes to draft from (pick-and-ban phase)
- Tile Control class: abilities that affect the board itself
- 50+ skills in an extended catalogue (German + English)
- Shield system (Weak/Strong/Mirror shields as distinct Defence mechanics)

A Java reimplementation from scratch ([`lultoni/project-roe`](https://github.com/lultoni/project-roe)) was built in 16 intensive days (June 30 – July 16, 2024):
- Full minimax AI with alpha-beta pruning and transposition tables
- 12-factor evaluation function
- MCTS experimented with ("not worth it") and abandoned
- All spells implemented with correct interactions
- ASCII board output + GUI rendering

Abandoned July 16, 2024 — the AI was only depth 1 (exponential branching factor made deeper search too slow), and the overall scope of building a polished engine solo proved unsustainable.

---

### August–September 2025 — Pivot: Pure Board Game

Starting university (September 2024, Bachelor of Business Computer Science) shifted priorities. After a pause, the game was reimagined as a **pure tabletop experience** with no digital ambitions. This reframing unlocked fast iteration.

A small web tool was built ([`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker), Aug–Sep 2025) to track turns, Runes, and piece counts during physical playtests. The rules at this point: 5 starting Runes, +1/round scaling every 7 rounds, 2 skill slots.

The terrain system was kept, and the 18-Champion class draft was simplified back to Guards/Champions/King.

---

### October 2025 — Playtest 1: Elias vs Pasco

The first real physical playtest (`playtest-results/elias-vs-pasco-31_10_25/`). Major findings:
- Game too long (~30 rounds)
- Rune economy too slow (first 6 rounds were skill-less)
- Bodyguard rule never triggered
- Injured state almost never reached (2 DMG attacks skipped it)

This playtest defined the critical problems and began the incremental testing methodology.

---

### April 2025 — Claude Code Partnership Begins (Session 1)

The design process was formalised with Claude Code as AI co-creator. Architecture ADRs written, living documents created, custom skills built. The project got its current structure: `game-state/`, `docs/systems/`, `docs/decisions/`, `docs/test-scenarios/`.

Key early decisions: terrain removed (overhead complexity), grid confirmed (makes spells interesting), perfect information / no luck.

---

### April 24, 2026 — Playtest 2: Elias vs Jonathan (Layer 1)

The economy fix (6 start Runes, +2/turn) was confirmed as working. Skills active from Round 1. Defensive skills used. Injured state relevant. But game still ran 4+ hours (ended as draw at Round 26). Standard attack dominance identified as the critical remaining problem.

---

### April 28, 2026 — Sessions 8–9: Infrastructure Complete

Dynamic stack testing system built. Composable Typst rule sheets. 12 PDFs. `TESTING_PLAN.pdf`. Stack A (cleverness: attack nerf + combo bonus) ready to print and play.

---

### Now — Session 10 onwards

The game is in active iterative playtesting. The core systems are stable. The focus is making skill combos the dominant and most rewarding strategy. The design question is no longer "will this work?" but "how do we make it great?"

---

## Reading the Files

| File | What it tells you |
|---|---|
| `UPLOAD - Realm of Elements/md-converted/Realm of Elements.md` | Original rulebook — first complete rules doc |
| `UPLOAD - Realm of Elements/md-converted/RevisedRules.md` | Cleaned-up second version of original rules |
| `UPLOAD - Realm of Elements/md-converted/DeutscheFassung.md` | German version of the original rules |
| `UPLOAD - Realm of Elements/md-converted/Realm Of Elements Script.md` | YouTube pitch script — captures the original game pitch voice |
| `Outdated ROE/md-converted/Project-ROE Rules.docx.md` → `Project-ROE Rules.md` | Full ROE redesign rulebook: 12×12, 6 classes, pick-and-ban |
| `Outdated ROE/md-converted/General changes.md` | Digital implementation design notes + AI architecture thinking |
| `Outdated ROE/md-converted/Gameplay loop changes.md` | Design methodology + mechanic iteration notes |
| `Outdated ROE/md-converted/Project-ROE Notizen.md` | 4 playtested games (V1.5.1–V1.5.2) with brief notes |
| `Outdated ROE/md-converted/Project-ROE Lore.md` | Full world lore: gods, the Rabbit, the tournament origin story |
| `Outdated ROE/md-converted/project roe _things to worry about_.md` | Implementation spec: full game logic, damage layer ordering |
| `first-boardgame-oriented-concept/md-converted/` | Pre-migration board game rulebook, skills catalogue, feedback form |
