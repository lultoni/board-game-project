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

*How a school holiday idea survived three digital restarts, a summer without WiFi, and an AI co-designer — and became something worth finishing.*

---

### June–July 2023 — The First Spark

End of 11th grade, two years left until Abitur. It was the kind of summer holiday where your brain has nothing to do and starts doing things on its own. The thought appeared: *what if chess had magic?*

Not a chess clone. Not just "chess with cooler pieces." Something with a real spell system, where the interesting decisions lived in how you combined abilities — not just where you moved a piece. The grid stays, because the grid is what makes spells interesting. Remove the board, and magic becomes math. Keep it, and suddenly positioning is everything.

The first version had five **elemental mages** — Fire, Water, Earth, Air, Spirit — replacing the chess back rank. Guards replaced pawns. The Spirit Mage was the king: lose it and you lose the game. Four terrain types (Forest, Mountain, Lake, Plains) gave each mage type a home advantage and a weakness. Rules were written, art was drawn, a German version was produced, and a pitch script for an explainer video was drafted. The video was never recorded. The rules were never quite finished. But the idea had legs.

---

### October 2023 — First Digital Version: `realm-of-elements`

11th grade was now 12th grade. Abitur loomed. Somehow, between studying, the game got built.

A full Java/Swing desktop application ([`lultoni/realm-of-elements`](https://github.com/lultoni/realm-of-elements)) with all 15 element spells implemented in code, a turn state machine, guard prioritisation logic, an Elo rating system with SQLite persistence, and — because why not — a 65-track medieval music library scraped from Age of Empires and Crusader Kings soundtracks. The game was playable as local multiplayer. It had piece sprites, board rendering, move validation. It had *vibes*.

It also got played. A lot, actually — the SQLite database survived in the repo and still holds the records: **Elias Glauert, 20 games played, Elo 1054**. Jonathan Vierus: 6 games, Elo 1003. Jonathan Payk: 10 games, Elo 943. Mario Glauert even showed up for 4 games. The Test1/Test2 entries (22 games each, Elo ~1000) suggest someone was running balance experiments. In total, at least **62 games** of this version were played and tracked — probably more that were played without logging. That's not a prototype, that's a game people were genuinely playing.

---

### June–July 2024 — Project ROE: Flying Too Close to the Sun

Abitur done. Spring 2024. The weight of two years of exams lifted and the summer stretched out open and full of possibility. There was exactly one obvious thing to do: rebuild the whole game, better, bigger, smarter.

This was **Project ROE**. The ambition was real: 12×12 board, 18 Champions across 6 specialised classes (Offense, Defense, Mobility, Boost, Tile Control), a pick-and-ban draft, 50+ skills in both German and English, a Shield system with three distinct types, and a Tile Control class that could modify the board itself. Also: a full AI opponent using minimax with alpha-beta pruning and transposition tables.

The Java reimplementation ([`lultoni/project-roe`](https://github.com/lultoni/project-roe)) was built in 16 days of genuinely obsessive development — June 30 to July 16, 2024. The evaluation function had 12 weighted factors. MCTS was experimented with and abandoned ("not worth it"). Every spell worked. The game loop ran.

And then it hit a wall. The minimax AI could only search to depth 1 — the branching factor of a 12×12 board with 18 pieces and 50+ skills made anything deeper computationally brutal. The game engine was ~85% complete and solo engine development at this scale proved unsustainable. On July 16, the last commit was pushed. The project went quiet.

A Unity prototype followed — a lighter version of the same 18-Champion ruleset, basically a drag-and-drop board with no rule enforcement, used for a handful of remote games with Jonathan (and once his cousin). But it was always a workaround, not a solution. The game deserved more than a digital clipboard.

---

### September 2024 — University, and a Quiet Period

Bachelor of Business Computer Science. New city, new rhythm, new priorities. The game didn't disappear — it never quite does — but it moved to the back of the mind. The digital implementation path felt exhausted. Building a game engine solo, to a quality worth playing, was a different project entirely from designing the game itself.

Somewhere in this period, a quiet reframing happened: *what if the game didn't need to be digital?*

---

### August 2025 — The Netherlands, No WiFi, and a Decision

Summer between second and third semester. A holiday in the Netherlands, staying in a Ferienhaus — the kind of place with no WiFi, just the sound of the wind and whatever you brought with you on your laptop.

Going through old files. Finding the old rules. The sprites. The Java code. The pitch script that was never recorded. Two years of the same idea, started and stopped, started and stopped.

And this time: *you know what? I'm going to actually finish this.*

Not as a digital game — that path had already been tried twice. As a **board game**. A real one. Something you could print, cut out, put on a table, and play with another person in the same room. The design could finally be about the design, not about rendering engines and network sync and AI search depth. The rules could be iterated in an afternoon instead of a sprint.

The first thing built when the WiFi was back: a small web turn-tracker ([`lultoni/prototype-turn-tracker`](https://github.com/lultoni/prototype-turn-tracker)) to manage Runes and piece counts during physical playtests. Nothing fancy — an HTML page with some JavaScript. But it was the first tool built specifically for *playing the game at a table*, and that felt different.

---

### October 2025 — Playtest 1: Elias vs Pasco

The first real physical playtest. Pieces on a board. Two people across a table. Handwritten feedback forms.

The results were honest: game too long (~30 rounds), Rune economy too slow (first 6 rounds were skill-less — you just moved pieces and waited), Bodyguard never triggered, Injured state almost never reached because a standard 2 DMG attack killed a piece outright and skipped Injured entirely. The feedback was clear and specific and useful in the way that only someone actually playing the game can be.

These findings became the entire testing programme. Every stack, every layer, every open question in this repo traces its origin to what Elias and Pasco found that evening.

---

### April 2026 — The Claude Code Partnership

Semester break. A decision to approach the design more systematically — not just "try a change and see what happens," but a real methodology: incremental layers, isolated variables, documented decisions, living documents that survive between sessions.

Enter Claude Code as AI co-creator. Not a chatbot to ask questions — a design partner to think with. The first session established the project infrastructure that now underlies everything in this repo: `game-state/`, `docs/systems/`, `docs/decisions/`, `docs/test-scenarios/`. Architecture Decision Records were written. The terrain system was formally removed (overhead complexity confirmed). The grid was formally confirmed as non-negotiable (it's what makes skills interesting — queens move, blocked by pieces, geometry matters). Perfect information, no luck, no dice: the design principles crystallised into writing.

Over nine sessions through April 2026, the design process became genuinely collaborative. Claude would read all the living documents at the start of each session, run research threads on comparable games (Onitama, Hive, Twilight Struggle, Go), extract ideas from the old version archives, generate Architecture Decision Records when multiple paths diverged, and write test scenario rule sheets in Typst ready to compile to PDF and print. The methodology evolved: linear test layers became a dynamic stack system, where evidence from each playtest determined which stack to run next rather than following a fixed sequence. Twelve PDFs now build from a composable Typst template system — change the baseline section, every stack updates automatically.

The partnership has a specific character. Claude brings breadth (research, comparable games, MDA analysis, design frameworks) and structure (documenting decisions, maintaining living docs, generating rule sheets). The human brings taste, judgment, and the thing that can only come from actually sitting across a table from another person and watching them play.

---

### April 24, 2026 — Playtest 2: Elias vs Jonathan

Economy fix confirmed: skills active from Round 1, Skill Slots now the real limiter, Injured state relevant ("Often" — both players). Defensive skills used meaningfully for the first time. Bodyguard triggered twice. Overall enjoyment: 4–5 out of 5. Jonathan: "Mid to late game Bombe — 6 out of 5."

But the game still ran four hours, ending as a draw at Round 26. Only one Champion kill in 26 rounds. The underlying problem became clear: standard attacks deal 2 DMG for free, outperforming every skill combo in the game. Skills are supposed to be the core fantasy — they're structurally the supporting act. That's what Stack A tests.

---

### April 28, 2026 — Sessions 8–9: Infrastructure Complete

Dynamic stack testing system built. Eight stacks defined (A through G, plus Accepted). Composable Typst section functions. `TESTING_PLAN.pdf` with a decision tree. All twelve PDFs building cleanly from source. Stack A — standard attack nerf + multi-Champion combo bonus — printed and ready to play.

The game is no longer being designed. It's being refined.

---

### Now — Session 10 onwards

The core systems are stable. The rules are coherent. Two playtests have generated real data. The question is no longer "will this work?" — it's "how do we make it genuinely great?"

Stack A tests whether nerfing the standard attack and rewarding coordinated Champions makes skill combos the dominant strategy. Stack B tests the Bodyguard fix. After those, the decision tree branches based on what the data shows.

---

## The Road Ahead

*A prediction, not a plan. Based on the pace of this project so far.*

---

### Phase A — Design Complete (late 2026, optimistically)

The testing programme currently has eight stacks. Stack A and B are ready to play. Stacks C through G will be written as their entry conditions are triggered by playtest data. Based on the cadence so far — roughly one major playtest every few months, with design sessions between — the core stack programme will likely take until **late 2026** to complete.

"Design complete" means: all critical stacks accepted, kill timing at R10–15, combo ceiling high enough that clever multi-Champion plays beat grinding, game length under 90 minutes for experienced players. A named, stable ruleset. A baseline PDF that doesn't change anymore.

What this phase probably looks like in practice: a handful of playtests with Elias, Jonathan, possibly new players who haven't seen the game before (blind playtests are the real test — they reveal what the rules *actually* communicate, not what you think they communicate). Design sessions with Claude to process results, adjust stacks, and write new scenario sheets. The `TESTING_PLAN.pdf` gets updated after each playtest. At some point, the remaining open questions stop being critical and start being polish.

---

### Phase B — Art & Visual Design (2027)

The game currently runs on black-and-white printed Typst PDFs and whatever tokens you have lying around. That's fine for playtesting and exactly wrong for everything else.

The existing sprite assets from v1 (elemental mage characters, terrain tiles) and the `images/` folder (15 skill card illustrations) are a starting point — they show what the visual language *could* be. But a manufacturable board game needs a coherent visual identity: piece iconography, card layout, board design, colour scheme, iconographic skill symbols, box art.

This phase probably involves commissioning an illustrator — likely through something like ArtStation or a local design school contact. The style direction is already implicit in the design: clean, readable, chess-adjacent but warmer. Not pixel art. Not overly fantasy-grimdark. Something that looks good at table distance and survives being printed at low resolution on cardboard.

The physical components list at this point will be roughly: 1 game board (foldable, ~A2 printed), 26 pieces per player in two colours (resin or wooden pawns + sticker labels, or custom meeples), skill cards (~20 cards per player), Rune tokens, Armor tokens, a rule booklet, and a quick-reference card. The Typst system already generates most of the rule text — adapting it for print layout is a real task, not a huge one.

Timeline estimate: **2027**, contingent on having a stable design to hand to an artist.

---

### Phase C — Manufacturing (late 2027 / early 2028)

The modern small-run board game manufacturing path is well-established: **The Game Crafter** (US, ~$30–50 per unit at small run) or **Ludo Fact / Panda** (Europe, minimum ~500 units, ~€8–15 per unit). For a first run, The Game Crafter or a European equivalent like **PrintPlayGames** or **PrintNinja** is the realistic starting point — no minimum order, upload your files, order a copy.

A "manufacturing complete" milestone at this stage means: one physical copy exists that you'd be comfortable handing to a stranger. Components feel right. Cards shuffle properly. The board lies flat. The pieces are distinguishable by touch in low light. This is a prototype run of maybe 5–10 copies — for gifting to friends, for leaving at a game café, for photographing.

The unit cost at this scale will be high (~€25–40 per copy) and that's fine. This isn't the commercial run. It's proof that the game is real.

Timeline estimate: **late 2027 to early 2028**, depending on art completion and how many revision cycles the physical prototype needs.

---

### Phase D — Marketing (2028)

Board game marketing in 2028 for an independent designer without a publisher will almost certainly run through a few channels simultaneously:

**Kickstarter / Gamefound** — the canonical path for self-published hobby games. A campaign for a 2-player abstract-tactical game with a clean visual identity and two years of documented design history is a realistic proposition. The stretch goal structure writes itself: base game, deluxe pieces (wooden meeples vs cardboard tokens), an expansion with new skill cards, a carrying case. The documentation in this repo — ADRs, playtest analyses, design session logs — is unusually thorough by indie board game standards and makes for compelling campaign content. People back games they believe in; showing the work is a real differentiator.

**BoardGameGeek** — a BGG listing with playtest photos, the design diary, and a rulebook PDF download. The BGG community is small enough that a genuinely novel 2-player abstract with documented design rigour gets noticed if you engage with it.

**Social media / content** — the YouTube script that was written in 2023 and never recorded finally gets recorded. The "how I designed a board game over 5 years" story is genuinely interesting and the documentation exists to tell it well. Short-form content (skill combo highlights, design decision breakdowns) maps well to existing board game content creator formats.

Target Kickstarter goal: €3,000–8,000 for a run of 100–300 copies. Not retire-on-it money — proof of market, recovery of costs, and a reason to keep going.

Timeline estimate: **2028**, assuming manufacturing prototype is done in early 2028 and the Kickstarter takes 3–6 months to prepare properly.

---

### Phase E — Sell & Ship (2028 onwards)

A successful Kickstarter campaign for a 2-player abstract game in the €25–35 price range would likely fund a print run of **200–500 copies** through a European manufacturer. Fulfillment for a small run at this scale is manageable solo or with a single fulfillment partner — ship backer copies first, then move remaining inventory to a webshop (Shopify or direct BGG marketplace listing) and potentially one or two friendly local game stores.

The realistic ceiling for a self-published, solo-designed abstract strategy game without a major publisher is modest — comparable games (Onitama, for reference, started as a small indie project before Pandasaurus picked it up) sell in the low thousands of copies in year one. That's not failure; that's the market for the category. The upside is that abstract strategy games have unusually long tails — they don't go out of fashion the way thematic games do, they generate word-of-mouth from the kind of player who takes the game seriously, and a good 2-player game is exactly what game cafés and couples buy.

The more interesting long-term path: if the game finds its audience through BGG and the content, a **publisher approach** becomes viable. Pandasaurus, Osprey Games, and Kosmos all have catalogues with 2-player abstracts and approachable tactical games. A proven, playtested design with documented iteration history and an existing community is a much stronger pitch than a prototype and a dream.

What shipping day probably looks like: a small apartment with boxes stacked to the ceiling, a label printer, 300 padded envelopes, and a very understanding flatmate. Exactly as unglamorous and exactly as satisfying as it sounds.

Timeline estimate: **2028–2029** for first commercial copies shipped. Beyond that: wherever the game takes it.

---

*The idea started in a school holiday in summer 2023. It's now April 2026. In three years it went from a thought experiment about chess and magic to a documented, tested, systematically iterated design with a version history, a composable rule sheet system, and a testing methodology rigorous enough to isolate individual mechanics. That's not nothing. That's most of the hard work.*

*The rest is just time.*
