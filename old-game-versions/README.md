# Old Game Versions — History & Archive

This folder contains every previous iteration of the game, going back to its first spark in summer 2023. The material here is **read-only reference** — do not treat any of it as current rules. For current design, see [`game-state/CURRENT_DESIGN.md`](../game-state/CURRENT_DESIGN.md).

---

## Folder Overview

### `v1-realm-of-elements/`

The **original game concept**. Two players each control 5 elemental mages (Fire, Water, Earth, Air, Spirit) and 5 guards on an 8×8 terrain board. Spells are element-specific (15 total: attack/defense/utility per element). The Spirit Mage is the "king" — lose it and you lose the game. Economy: 5 spell tokens to start, +1 every turn scaling up every 5 rounds.

This version has full art assets: 12 character sprites (Red/Blue factions), 4 terrain tiles, board layouts. A German version was also written. A YouTube-style pitch script was written to explain the game concept — never recorded.

**Key GitHub repo**: [`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements) — Java/Swing desktop app with SQLite player rankings and Elo ratings. Local multiplayer, all 15 spells implemented, full turn state machine, medieval music library (65+ tracks), guard prioritisation logic, FEN-like board notation. Created Oct 2023, last pushed Jan 2024.

---

### `v2-project-roe/`

**Project ROE** — a major redesign of the rules and a second attempt at a full digital implementation, this time in Java with an AI opponent. The elemental-mage theme was kept but the mechanics were overhauled:
- Board expanded to 12×12 with terrain
- Pieces redesigned into 6 Champion **classes** (Offense, Defense, Mobility, Boost, Tile Control, a sixth TBD) — 3 Champions per class, 18 total to draft from
- Pick-and-ban draft phase added
- Skill system expanded (50+ skills in German and English catalogues)
- Shield system (Weak/Strong/Mirror) as distinct Defence mechanics
- Tile Control class: abilities that alter the board itself

After Project ROE was abandoned (digital scope too large), a **Unity prototype** was built as a lighter digital version of this same 18-Champion ruleset — essentially a drag-and-drop board with no rule enforcement, used for remote play with Jonathan and friends. Not preserved in this repo.

**Key GitHub repo**: [`lultoni/project-roe`](https://github.com/lultoni/project-roe) — Java implementation, minimax AI with alpha-beta pruning and transposition tables (depth 1), 12-factor evaluation function, all spells implemented. 16 days of intensive development June–July 2024. Abandoned when the AI search depth couldn't scale and solo engine development became unsustainable.

---

### `v3-first-board-game/`

The **pivot to a pure board game** — stripping all digital ambitions and focusing entirely on the tabletop experience. This is the direct ancestor of the current game. Key changes from Project ROE:
- Dropped the 18-Champion class system → simpler Guards/Champions/King
- Kept terrain initially, later removed (confirmed overhead in playtesting)
- Runes and Skill Slots formalised as the core economy
- Designed explicitly for physical play

This is the version that went to **Playtest 1** (Elias vs Pasco, Oct 2025).

**Also relevant**: [`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker) — a tiny web tool (HTML/CSS/JS) built Aug–Sep 2025 to track turns, Runes, and piece counts during physical playtests of this version. Last pushed Sep 2025.

---

## The Ultimate Game Timeline

*How this game grew from a school holiday idea into a serious design project.*

---

### June–July 2023 — The First Spark

End of 11th grade (Abitur year 1). The idea first appeared: *what if chess had magic?* The core concept was already clear — two players, a grid board, pieces that could cast spells, the "king" as the win condition.

The first version had **elemental mages** (Fire, Water, Earth, Air, Spirit) replacing chess pieces. Guards replaced pawns. The Spirit Mage took the role of the King. Terrain (Forest, Mountain, Lake, Plains) gave each mage type advantages and disadvantages. Rules were written, art was made, a German version was produced, and a pitch script for a YouTube video was drafted (never recorded).

---

### October 2023 — First Digital Implementation: `realm-of-elements`

The concept was built as a full Java/Swing desktop application ([`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements)). Features:
- 8×8 terrain board with FEN-like map notation
- All 15 element spells fully implemented
- Elo rating system with SQLite persistence
- Medieval music library (65+ Age of Empires / Crusader Kings tracks)
- Complete turn state machine: Movement → Attack → repeat
- Guard prioritisation logic, move validation

Complete and playable as local multiplayer. Last pushed January 2024.

---

### June–July 2024 — Project ROE: The Redesign

With Abitur done (spring 2024) and before starting university, a full redesign began. The game became **Project ROE** — more ambitious in both rules and technology.

Major design changes: 12×12 board, 18 Champions in 6 classes, pick-and-ban draft, Tile Control class, 50+ skills, Shield system.

A Java reimplementation from scratch ([`lultoni/project-roe`](https://github.com/lultoni/project-roe)) was built in 16 intensive days (June 30 – July 16, 2024) with full minimax AI, all spells, and a 12-factor evaluation function. Abandoned July 16 — the AI was limited to depth 1 due to the exponential branching factor, and the solo scope proved unsustainable.

---

### After July 2024 — Unity Multiplayer Prototype

After the Java implementation was abandoned, a lighter digital version was built in Unity as a way to keep playing the 18-Champion ruleset. It was essentially a drag-and-drop board — no rule enforcement, players tracked everything themselves. Used for a handful of remote games with Jonathan (and once his cousin). Not preserved in the repo.

---

### September 2024 — University Begins

Bachelor of Business Computer Science started. After a period of lower activity, the project direction shifted: instead of trying to build a digital game engine, focus entirely on the physical board game experience.

---

### August–September 2025 — Pure Board Game + Turn Tracker

The game was reimagined as a **pure tabletop game** with no digital ambitions. A small web tool was built ([`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker), Aug–Sep 2025) to track turns, Runes, and piece counts during physical playtests. Rules at this point: 5 starting Runes, +1/round scaling every 7 rounds, 2 skill slots scaling every 10 rounds.

The terrain system was kept, and the 18-Champion class draft was simplified back to Guards/Champions/King.

---

### October 2025 — Playtest 1: Elias vs Pasco

First real physical playtest. Major findings: game too long (~30 rounds), Rune economy too slow (first 6 rounds skill-less), Bodyguard never triggered, Injured state almost never reached (2 DMG attacks skipped it). These problems defined the entire incremental testing programme that followed.

---

### April 2026 — Claude Code Partnership + Session 1

The design process was formalised with Claude Code as AI co-creator. Architecture ADRs written, living documents created, custom skills built. Terrain removed. Grid confirmed as essential (makes spells interesting). Perfect information, no luck.

---

### April 24, 2026 — Playtest 2: Elias vs Jonathan

Economy fix (6 start Runes, +2/turn) confirmed working — skills active from Round 1. But game still ran 4+ hours (draw at Round 26). Standard attack dominance identified as the critical remaining problem.

---

### April 28, 2026 — Sessions 8–9: Infrastructure Complete

Dynamic stack testing system built. Composable Typst rule sheets. `TESTING_PLAN.pdf`. Stack A (attack nerf + combo bonus) ready to print and play.

---

### Now — Session 10 onwards

Active iterative playtesting. Core systems stable. The design question is no longer "will this work?" but "how do we make it great?"
