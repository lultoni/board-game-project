# Old Game Versions — History & Archive

This folder contains every previous iteration of the game, going back to its first spark in summer 2023. The material here is **read-only reference** — do not treat any of it as current rules. For current design, see [`docs/test-scenarios/baseline/ruleset-baseline.typ`](../docs/test-scenarios/baseline/ruleset-baseline.typ) (player-facing rules) and [`docs/systems-and-mechanics.md`](../docs/systems-and-mechanics.md) (design documentation).

For the per-session narrative log of the current design programme (Session 1 onwards), see [`game-state/SESSION_LOG.md`](../game-state/SESSION_LOG.md).

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

## Pre-Session-1 History

*How a school holiday idea survived three digital restarts, a summer without WiFi, and an AI co-designer — and became something worth finishing. From Session 1 onwards, see [`game-state/SESSION_LOG.md`](../game-state/SESSION_LOG.md).*

---

### June–July 2023 — The First Spark

End of 11th grade, two years left until Abitur. It was the kind of summer holiday where your brain has nothing to do and starts doing things on its own. The thought appeared: *what if chess had magic?*

Not a chess clone. Not just "chess with cooler pieces." Something with a real spell system, where the interesting decisions lived in how you combined abilities — not just where you moved a piece. The grid stays, because the grid is what makes spells interesting. Remove the board, and magic becomes math. Keep it, and suddenly positioning is everything.

The first version had five **elemental mages** — Fire, Water, Earth, Air, Spirit — replacing the chess back rank. Guards replaced pawns. The Spirit Mage was the king: lose it and you lose the game. Four terrain types (Forest, Mountain, Lake, Plains) gave each mage type a home advantage and a weakness. Rules were written, art was drawn, a German version was produced, and a pitch script for an explainer video was drafted. The video was never recorded. The rules were never quite finished. But the idea had legs.

---

### October 2023 — First Digital Version: `realm-of-elements`

11th grade was now 12th grade. Abitur loomed. Somehow, between studying, the game got built.

A full Java/Swing desktop application ([`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements)) with all 15 element spells implemented in code, a turn state machine, guard prioritisation logic, an Elo rating system with SQLite persistence, and — because why not — a 65-track medieval music library scraped from Age of Empires and Crusader Kings soundtracks. The game was playable as local multiplayer. It had piece sprites, board rendering, move validation. It had *vibes*.

It also got played. A lot, actually — the SQLite database survived in the repo and still holds the records: **Elias Glauert, 20 games played, Elo 1054**. Jonathan Vierus: 6 games, Elo 1003. Jonathan Payk: 10 games, Elo 943. Mario Glauert even showed up for 4 games. The Test1/Test2 entries (22 games each, Elo ~1000) suggest someone was running balance experiments. In total, at least **62 games** of this version were played and tracked. That's not a prototype, that's a game people were genuinely playing.

---

### June–July 2024 — Project ROE: Flying Too Close to the Sun

Abitur done. Spring 2024. The weight of two years of exams lifted and the summer stretched out open and full of possibility. There was exactly one obvious thing to do: rebuild the whole game, better, bigger, smarter.

This was **Project ROE**. The ambition was real: 12×12 board, 18 Champions across 6 specialised classes, a pick-and-ban draft, 50+ skills in both German and English, a Shield system with three distinct types, and a Tile Control class that could modify the board itself. Also: a full AI opponent using minimax with alpha-beta pruning and transposition tables.

The Java reimplementation ([`lultoni/project-roe`](https://github.com/lultoni/project-roe)) was built in 16 days of genuinely obsessive development — June 30 to July 16, 2024. The evaluation function had 12 weighted factors. MCTS was experimented with and abandoned ("not worth it"). Every spell worked. The game loop ran.

And then it hit a wall. The minimax AI could only search to depth 1 — the branching factor of a 12×12 board with 18 pieces and 50+ skills made anything deeper computationally brutal. The game engine was ~85% complete and solo engine development at this scale proved unsustainable. On July 16, the last commit was pushed. The project went quiet.

A Unity prototype followed — a lighter version of the same 18-Champion ruleset, basically a drag-and-drop board with no rule enforcement, used for a handful of remote games with Jonathan (and once his cousin). But it was always a workaround, not a solution.

---

### September 2024 — University, and a Quiet Period

Bachelor of Business Computer Science. New city, new rhythm, new priorities. The game didn't disappear — it never quite does — but it moved to the back of the mind. The digital implementation path felt exhausted. Building a game engine solo, to a quality worth playing, was a different project entirely from designing the game itself.

Somewhere in this period, a quiet reframing happened: *what if the game didn't need to be digital?*

---

### August 2025 — The Netherlands, No WiFi, and a Decision

Summer between second and third semester. A holiday in the Netherlands, staying in a Ferienhaus — the kind of place with no WiFi, just the sound of the wind and whatever you brought with you on your laptop.

Going through old files. Finding the old rules. The sprites. The Java code. The pitch script that was never recorded. Two years of the same idea, started and stopped, started and stopped.

And this time: *you know what? I'm going to actually finish this.*

Not as a digital game — that path had already been tried twice. As a **board game**. A real one. Something you could print, cut out, put on a table, and play with another person in the same room.

The first thing built when the WiFi was back: a small web turn-tracker ([`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker)) to manage Runes and piece counts during physical playtests.

---

### October 2025 — Playtest 1: Elias vs Pasco

The first real physical playtest. Pieces on a board. Two people across a table. Handwritten feedback forms.

The results were honest: game too long (~30 rounds), Rune economy too slow (first 6 rounds were skill-less — you just moved pieces and waited), Bodyguard never triggered, Injured state almost never reached because a standard 2 DMG attack killed a piece outright and skipped Injured entirely.

These findings became the entire testing programme. From here, see [`game-state/SESSION_LOG.md`](../game-state/SESSION_LOG.md) for Session 1 (April 2026) onwards.

---

## The Road Ahead

*A prediction, not a plan. Based on the pace of this project so far.*

### Phase A — Design Complete (late 2026, optimistically)

Stack programme accepted, kill timing at R10–15, combo ceiling high, game length under 90 minutes for experienced players. A named, stable ruleset.

### Phase B — Art & Visual Design (2027)

Coherent visual identity (piece iconography, card layout, board design, colour scheme, iconographic skill symbols, box art). Likely commissioned through ArtStation or a local design school contact.

### Phase C — Manufacturing (late 2027 / early 2028)

The Game Crafter or PrintPlayGames / PrintNinja for a first run of 5–10 prototype copies (~€25–40 per copy).

### Phase D — Marketing (2028)

Kickstarter / Gamefound campaign, BoardGameGeek listing, social media / "how I designed a board game over 5 years" content. Target Kickstarter goal: €3,000–8,000 for a run of 100–300 copies.

### Phase E — Sell & Ship (2028 onwards)

200–500 copies through a European manufacturer. Fulfillment manageable solo. Long-term: publisher approach (Pandasaurus, Osprey Games, Kosmos) becomes viable if the game finds its audience.

---

*The idea started in a school holiday in summer 2023. It's now May 2026. In three years it went from a thought experiment about chess and magic to a documented, tested, systematically iterated design with a version history, a composable rule sheet system, and a testing methodology rigorous enough to isolate individual mechanics. That's most of the hard work. The rest is just time.*
