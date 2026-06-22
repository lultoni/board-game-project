<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# \#context / notes

## boardgame code translation

we need to find a optimal/efficient way to display/store board positions
we also need a language that allows both execution inside a web framework (so it works on all platforms!!) and that also allows us to use fast bitwise operations - this is our first priority

additionally we need to quickly be able to shift to new positions and make changes for every possible "interaction that can be made with the system/position" (aka we need to include every move, every hit, every skill and so on as method calls that both take the position (number) and the action (also formatted as a number from which we can extract the relevant information) so we can perfectly make changes to a position. this should be as minimal as possible

our implementation for finding possible actions from a specific position also has to done in bitspace (or maybe multidimensional bitspaces?) so we can optimally search them

search will take the fastest path to a score if two "final depth ratings" are the same

and of course the same goes for scoring or pruning positions for/during search so we can "basically immediately" know the rating of a position

> so what do we consider for our position rating?

first of all: ratings go from +inf to -inf (or the integer limit essentially). p1 advantage is a positive score, p2 advantage is negative score

well the simplest is winning: +/-inf for both players, that overrules anything and is checked first

our search will then prioritise the fastest winning path itself - we do not contaminate that into our evaluation function

after win/loss we just count the real existing things, like pieces, different equipped skills (and their following possibilities of play), hp, armor and at every possible point we try and measure them against money and longterm tempo/cash advantage
tempo meaning how many actions the opponent has to take to reverse something or gain something (of course taking into consideration that they might now take this route - so we assume the best path forward of both options (at least from the math side)). money meaning how much it costs for the specific player (based on certain skill flags) to undo something or do something or gain something.
so we try and approach a position from both angles and compare them to the effect on the longterm game on how they will play out until a assumed game end time point (aka how large is the actual effect of something)

our evaluation should still be clever at every turn and do things as efficiently as possible so we do not skyrocket the evaluation cost for a single position (as that would reverse our advances in the other areas)
if we realise that our complex but accurate approach costs too much resources then we shall think of a simplified but still good evaluation method and we then test the differences in random board game positions so we can understand where the differences happen to hopefully improve it further.

---

for our frontend we of course need a translator of the bitwise postition

we plan the design from the start with a focus on making it as easy as possible to adapt to new rule changes - we do not need fancy designs and a clean ux, only being able to make quick changes matters. ux is second to that

~in our search we also want a "backwards search" if that is possible
this would need for us to know each possible ending possible end position from that game state and that is basically impossible from what i can imagine right now. this typically needs a never changing same end position which our game does not have.~

# your task

find me languages or tech stacks that allow me to do what i have planned as efficiently as possible. secondly you shall give me very harsh and critical feedback so i get a new perspective and am able to improve what i have planned so far

Now I have enough information to give a comprehensive, critical response. Let me write it up.

This is a meaty system design question — here's the full breakdown: recommended stack, and then genuinely harsh critical feedback.

***

## Recommended Tech Stack

### Primary Choice: Rust + WebAssembly

This is the clear winner for your constraints. Rust compiles to WASM via `wasm-pack`/`wasm-bindgen`, runs in every modern browser without plugins, and gives you native 64-bit integer bitwise ops (`u64`, `u128`) without any overhead. The critical problem with pure JavaScript is that it has a 53-bit safe integer limit — anything beyond that requires `BigInt`, which is measurably slower and kills the primary reason for using bitboards in the first place. A real-world example: a complete Othello engine with alpha-beta search compiled to **3.7 KB of WASM** using Rust with zero heap allocations, just raw `u64` bitboards. That's the kind of footprint you want.[^1_1][^1_2][^1_3]

For the frontend/UI layer specifically, in 2026 you have two Rust-native options:[^1_4]

- **Leptos** — fine-grained reactivity, no virtual DOM, fastest DOM updates, best for pure web
- **Dioxus** — virtual DOM (React-like), larger ecosystem, runs on desktop/mobile too via Tauri

For a tool where "quick rule changes" matter more than UX polish, **Leptos** is the better pick: it's faster, the HTML-like syntax is easier to hack on, and the bundle stays small.[^1_5]

### Architecture Split

```
[Rust core crate]
  ├── position.rs     → bitboard structs, encode/decode
  ├── moves.rs        → move generation (bitwise only)
  ├── apply.rs        → all action methods (move/hit/skill)
  ├── eval.rs         → scoring function
  └── search.rs       → alpha-beta + iterative deepening

[wasm-bindgen glue]   → exposes Rust functions to JS/TS

[Leptos frontend]
  ├── board renderer  → reads position bits → visual grid
  └── interaction     → encodes user actions as numbers → calls Rust
```

The same Rust crate can be used server-side if you ever want a backend engine, which is exactly how the Onitama web app is built — shared game logic between client WASM and a server.[^1_6]

### Position Representation

Use **composite bitboards**: one `u64` (or `u128` if your board exceeds 64 squares) per *property layer*. For a game with pieces, HP, armor, and skills, you'd have:[^1_7][^1_8]

- `pieces_p1: u64` — bitmask of occupied squares, player 1
- `pieces_p2: u64` — same for player 2
- `skills_a: u64`, `skills_b: u64` — one bit per square per skill slot
- HP and armor: these **cannot be bit-packed one-bit-per-square** if they're scalar values per piece — use a compact array (`[u8; N]`) indexed by `bit_position.trailing_zeros()` alongside the bitboards

Actions as numbers: encode `(action_type, source_square, target_square, skill_id)` into a single `u32` with bitfield packing. Rust's `bitfield` or `modular-bitfield` crates make this ergonomic.

### Search: Alpha-Beta with Iterative Deepening

Alpha-beta pruning with iterative deepening is the correct choice here. MCTS is better when you lack a reliable evaluation function (e.g. Go before neural nets), but you've described a game where you *can* reason explicitly about material, tempo, and money — that's exactly the domain where a well-tuned alpha-beta with good move ordering beats MCTS. Iterative deepening means you always have a result at any time budget, and it enables move ordering from the previous depth (which dramatically improves pruning efficiency — theoretically halving the effective branching factor).[^1_9][^1_10][^1_11][^1_12]

***

## Harsh Critical Feedback

### 1. Your evaluation function is over-engineered before you have a working engine

You're describing a weighted multi-factor evaluation that accounts for tempo, money, long-term game projections, and assumed opponent best-paths — before you've written a single line of the engine. This is a classic trap. **Start with material count only. It will beat random play instantly and give you a baseline to measure every future improvement against.** You literally cannot know if your complex evaluation is better unless you have something simpler to compare it to. The chess programming community learned this the hard way: Stockfish's evaluation grew iteratively over 15+ years, not by design up front.[^1_13]

### 2. "Basically immediately know the rating" is not achievable with your described evaluation

You want to simultaneously evaluate tempo advantages, money costs, skill trees, and long-term game projection — and do it fast. These are contradictory goals. Each factor you add to your evaluation is a multiplier on search cost. A 10ms evaluation function at depth 1 is slower than a 0.01ms evaluation at depth 6 doing far better tree pruning. **Shallow search with a simple-but-correct evaluation almost always beats deep search with a slow-but-clever evaluation.** Define a hard compute budget (e.g. 100ms per move) and work backwards from there.

### 3. Your "multidimensional bitspace" for move generation is underspecified

This sounds like you're reinventing something that may or may not map cleanly to your game. Standard bitboard move generation works beautifully for games with spatial locality (chess, checkers, Go) — pieces move to adjacent squares, attacks have geometric patterns. If your game has skills with arbitrary effects, bitboards for *move generation* may not help you at all, and could actively confuse the implementation. **You need to be concrete: what does "a possible action" look like geometrically/logically for your game?** If skills target arbitrary board states rather than specific squares, you don't get the bitboard speedup on generation — you get it only on position storage and evaluation.

### 4. You crossed out "backwards search" too fast

You're right that traditional retrograde analysis requires a fixed terminal state, which your game doesn't have. But you dismissed it entirely. A more useful middle ground is **transposition tables** (hash the position, cache its score) — this gives you forward search that doesn't re-explore positions it's already evaluated. Without this, your search is burning most of its time re-evaluating the same positions from different move orderings. This is not optional at any serious search depth.[^1_14]

### 5. The "fastest path to a win" tiebreaker conflicts with your evaluation

You want search to prefer shorter wins when eval scores are equal. That's correct in principle, but it means your score must encode depth somehow (e.g. `MATE_SCORE - depth`). If you do this incorrectly, you'll get bugs where a "nearby win" scores lower than a positional advantage, causing the engine to ignore forced mates. This is a well-known pitfall — it needs to be designed from the start, not bolted on. The standard approach is to store `±(INFINITY - depth_to_mate)` so deeper mates score lower in absolute value.[^1_15]

### 6. The "adapt to new rule changes" goal is incompatible with deep bitboard coupling

You want maximum bitwise performance *and* maximum changeability. These pull in opposite directions. Bitboard code is notoriously brittle — every layout assumption is baked into every mask, shift, and AND operation. If you change the board size, piece types, or skill system, you'll need to rewrite most of the bit manipulation logic. **Pick one as the primary goal.** For a prototype or design-iteration phase, use a clean array-based mailbox representation and switch to bitboards *after* rules are stable. The performance difference only matters if your search is actually bottlenecked on position operations, which you won't know until you profile.[^1_16]
<span style="display:none">[^1_17][^1_18][^1_19][^1_20][^1_21][^1_22][^1_23][^1_24][^1_25][^1_26][^1_27][^1_28][^1_29][^1_30][^1_31][^1_32][^1_33][^1_34][^1_35][^1_36][^1_37][^1_38][^1_39][^1_40][^1_41][^1_42][^1_43][^1_44][^1_45]</span>

<div align="center">⁂</div>

[^1_1]: https://reintech.io/blog/webassembly-game-development-complete-guide-2026

[^1_2]: https://dev.to/sendotltd/a-37-kb-othello-engine-bitboards-in-rust-raw-wasm-no-wasm-bindgen-353f

[^1_3]: https://talkchess.com/viewtopic.php?t=85748

[^1_4]: https://wrenlearnsrust.com/posts/leptos-vs-dioxus-rust-web-framework-comparison.html

[^1_5]: https://leptos.dev

[^1_6]: https://www.reddit.com/r/rust/comments/olc46f/made_a_board_game_webapp_with_frontend_and/

[^1_7]: https://spin.atomicobject.com/game-playing-ai-bitboards/

[^1_8]: https://www.chessprogramming.org/Bitboards

[^1_9]: https://courses.cs.washington.edu/courses/cse473/12au/slides/lect7.pdf

[^1_10]: https://www.chessprogramming.org/Alpha-Beta

[^1_11]: https://news.ycombinator.com/item?id=19552286

[^1_12]: https://www.diva-portal.org/smash/get/diva2:1887668/FULLTEXT01.pdf

[^1_13]: https://stanford-cs221.github.io/autumn2022-extra/modules/games/evaluation-functions.pdf

[^1_14]: https://minimax.dev/docs/ultimate/efficient-representation/

[^1_15]: https://www.cs.cornell.edu/courses/cs312/2002sp/lectures/rec21.htm

[^1_16]: https://nereuxofficial.github.io/posts/bitboard-rust/

[^1_17]: https://lars.hupel.info/articles/webassembly/

[^1_18]: https://talks.ui-patterns.com/videos/building-a-javascript-based-game-engine-for-the-web

[^1_19]: https://www.reddit.com/r/gamedev/comments/dj8qr0/game_development_in_wasm_web_assembly_has_almost/

[^1_20]: https://ctjs.rocks

[^1_21]: https://ianjk.com/webassembly-vs-javascript/

[^1_22]: https://blog.bitsrc.io/improving-the-log2-algorithm-for-bigint-in-javascript-7e5692e7bf0b

[^1_23]: https://surma.dev/things/js-to-asc/

[^1_24]: https://voidst.one/blog/tic-tac-toe-with-react-redux-rust-part-3/

[^1_25]: https://github.com/collections/javascript-game-engines

[^1_26]: https://yew.rs/docs/0.20/concepts/basic-web-technologies/wasm-bindgen

[^1_27]: https://rustwasm.github.io/docs/wasm-bindgen/

[^1_28]: https://en.wikipedia.org/wiki/Bitboard

[^1_29]: https://github.com/denkspuren/BitboardC4/blob/master/BitboardDesign.md

[^1_30]: https://www.scribd.com/document/791928379/6-Chapter3-Search4

[^1_31]: https://users.rust-lang.org/t/help-us-make-wasm-bindgen-support-more-targets/101718

[^1_32]: https://www.gamedev.net/tutorials/programming/artificial-intelligence/chess-programming-part-ii-data-structures-r1046/

[^1_33]: https://cris.maastrichtuniversity.nl/en/publications/bitboard-methods-for-games/

[^1_34]: https://en.wikipedia.org/wiki/Alpha–beta_pruning

[^1_35]: https://news.ycombinator.com/item?id=30181386

[^1_36]: https://www.reddit.com/r/compsci/comments/20f0h9/can_someone_please_give_me_a_walkthrough_of_the/

[^1_37]: https://godotengine.org/article/upcoming-serious-web-performance-boost/

[^1_38]: https://github.com/flosse/rust-web-framework-comparison

[^1_39]: https://en.chessbase.com/post/monte-carlo-instead-of-alpha-beta

[^1_40]: https://www.awelm.com/posts/simd-web-assembly-experiment/

[^1_41]: https://reintech.io/blog/leptos-vs-yew-vs-dioxus-rust-frontend-framework-comparison-2026

[^1_42]: https://doc.neuro.tu-berlin.de/bachelor/2024-BA-MalteHauff.pdf

[^1_43]: https://github.com/wasmedge/docs/blob/main/docs/develop/c/simd.md

[^1_44]: https://github.com/leptos-rs/leptos

[^1_45]: https://dev.to/yangholmes/using-simd-in-webassembly-part-1-52ec


---

# these are the rules for my current game - changes soon to come to make it simpler, but in a example middle game position, a lower bound amount of possible turns for player 1 was 2.3 million, with the true number maybe around 3-4 million. that is a single move that would then entail a similar amount for the opponent. this is why i want to use bitboards anyways

Yes — with the rules you posted, bitboards still make sense, but mainly as a **state-manipulation** tool, not as the thing that magically makes search feasible. Bitboards help because they represent sets densely and support parallel bitwise operations for occupancy, attack sets, and target masks, but a huge branching factor still causes combinatorial explosion in search, with cost growing roughly like $b^d$ unless you reduce the effective branch factor aggressively.[^2_1][^2_2][^2_3][^2_4]

## What this means

Your current rules are exactly the kind of design that explodes search: 10x10 board, two move actions, two skill actions, twelve pieces per side, reusable skills, global modifiers like Focus/Charge, defender choice on Bodyguard, and free-pathing Guards all multiply combinations. The harsh truth is that if 2.3 to 4 million full turns is real, then “full-turn enumeration first, prune later” is already the wrong search model, because high branching factor kills tree search long before low-level position storage becomes the bottleneck.[^2_2][^2_5][^2_1]

## What bitboards help with

Use bitboards anyway, because they are still the right low-level representation for occupancy, line masks, adjacency, legal-target masks, and fast make/unmake of primitive state changes. But do not confuse “fast bitwise position ops” with “solved search,” because portable bitboard code still needs careful machine-aware optimization, and the bigger win comes from shrinking the action space before search even starts.[^2_5][^2_4]

## Search redesign

Do **not** search complete turns as monolithic actions; search intermediate turn states instead, with remaining move actions, remaining skill actions, per-piece moved flags, temporary buffs, combo counters, and money all included in the node. For large-branching strategy games, the literature consistently points toward pruning plus domain knowledge, action abstraction, and script-space or portfolio-style search to avoid expanding every raw action combination, especially when the number of actions per turn is flexible.[^2_6][^2_7][^2_8]

A practical version for your game is:

- Generate only primitive actions first, not full turns.
- Build turn sequences incrementally and cut them early with legality, affordability, and tactical dominance filters.
- Use iterative deepening and a transposition table keyed by board state plus intra-turn state, because iterative deepening is especially useful when branching is large and cached best moves improve cutoffs in later iterations.[^2_9][^2_10][^2_11]


## Harsh feedback

Right now, your game is not “hard for the engine” — it is structurally hostile to exact search. If millions of legal turns are normal, then every extra interaction rule is not just adding depth for players; it is adding a tax on computation that will drown alpha-beta, drown evaluation tuning, and probably drown debugging too.[^2_8][^2_1][^2_2]

The most dangerous assumption in your plan is “we need bitboards anyway, therefore the search can be made efficient enough.” No: bitboards may give you a faster disaster. If you want strong machine play, you either need to simplify the rule set so the effective branching factor collapses, or you need to accept a search model based on abstraction, selective expansion, and heavy pruning rather than exhaustive tactical enumeration.[^2_7][^2_4][^2_8]

## Changes I would make

If your priority is actually solvable AI, I would simplify these first:

- Remove “same piece can use the same skill multiple times in one turn.”
- Remove or deterministicize defender choice in Bodyguard.
- Replace free-pathing movement with fixed geometric movement templates.
- Make Focus and Charge local to one piece instead of global next-action modifiers.
- Limit skill targets so each skill has a small, precomputable target class.

Those changes do more for engine strength than another month of clever bit-packing, because they attack the true bottleneck: action-space explosion, not board storage.[^2_1][^2_2][^2_8]

My blunt recommendation is: keep Rust + WASM for the core, keep bitboards for state, but redesign the engine around **intra-turn search + action abstraction**, and treat rule simplification as a first-class AI feature rather than a compromise.
<span style="display:none">[^2_12][^2_13][^2_14][^2_15][^2_16][^2_17][^2_18][^2_19][^2_20][^2_21][^2_22][^2_23][^2_24][^2_25][^2_26][^2_27][^2_28][^2_29][^2_30][^2_31][^2_32][^2_33][^2_34][^2_35][^2_36][^2_37][^2_38][^2_39][^2_40][^2_41][^2_42][^2_43][^2_44]</span>

<div align="center">⁂</div>

[^2_1]: https://www.opentrain.ai/glossary/branching-factor/

[^2_2]: https://en.wikipedia.org/wiki/Branching_factor

[^2_3]: https://www.andrew.cmu.edu/course/15-121/lectures/Game Trees/Game Trees.html

[^2_4]: https://www.chessprogramming.org/Bitboards

[^2_5]: https://www.chessprogramming.org/Branching_Factor

[^2_6]: https://www.tnt.uni-hannover.de/papers/data/1609/2021__COG__Game_State_and_Action_Abstracting_Monte_Carlo_Tree_Search_for_General_Strategy_Game_Playing.pdf

[^2_7]: https://dl.acm.org/doi/pdf/10.1145/3297156.3297188

[^2_8]: https://gaigresearch.github.io/2020/06/15/hsu2020mcts/

[^2_9]: http://mediocrechess.blogspot.com/2007/01/guide-iterative-deepening.html

[^2_10]: https://lavalle.pl/planning/node47.html

[^2_11]: https://stackoverflow.com/questions/78250922/how-can-i-optimize-this-transposition-table-for-connect-4-ai

[^2_12]: ruleset-baseline.pdf

[^2_13]: http://togelius.blogspot.com/2016/03/a-way-to-deal-with-enormous-branching.html

[^2_14]: https://arxiv.org/pdf/1710.04805.pdf

[^2_15]: https://gki.informatik.uni-freiburg.de/teaching/ss14/gki/lectures/ai06.pdf

[^2_16]: https://hci.iwr.uni-heidelberg.de/system/files/private/downloads/1617646103/robert_klassert-monte_carlo_tree_search.pdf

[^2_17]: https://www.semanticscholar.org/paper/A.I-for-Games-with-High-Branching-Factor-Mohandas-Nizar/b957a17a39c342775be1d821e5afcf2a860b0275

[^2_18]: https://www.cs.cornell.edu/boom/2004sp/ProjectArch/Chess/chessreport.html

[^2_19]: https://ieeexplore.ieee.org/document/8531047/

[^2_20]: https://stackoverflow.com/questions/16500739/chess-high-branching-factor

[^2_21]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search

[^2_22]: https://project.dke.maastrichtuniversity.nl/games/files/bsc/Roelofs_Bsc-paper.pdf

[^2_23]: https://cris.maastrichtuniversity.nl/ws/files/95835021/Winands_2017_Monte_Carlo_Tree_Search_In_Board.pdf

[^2_24]: https://arxiv.org/pdf/2205.15126.pdf

[^2_25]: https://stackoverflow.com/questions/15489329/breadth-first-search-branching-factor

[^2_26]: https://www.ijcai.org/proceedings/2021/0470.pdf

[^2_27]: https://www.youtube.com/watch?v=ZmiHJaEAHds

[^2_28]: https://www.tnt.uni-hannover.de/papers/data/1606/2020__AIIDE_SGW__STRATEGA__A_General_Strategy_Games_Framework.pdf

[^2_29]: https://talkchess.com/viewtopic.php?t=79365

[^2_30]: https://www.reddit.com/r/chessprogramming/comments/1fhwmiq/bitboard_move_generation/

[^2_31]: https://chessenginelab.substack.com/p/magic-bitboards

[^2_32]: https://stackoverflow.com/questions/16925204/sliding-move-generation-using-magic-bitboard

[^2_33]: https://www.chessprogramming.org/Magic_Bitboards

[^2_34]: https://www.sciencedirect.com/science/article/abs/pii/S0950705112001293

[^2_35]: https://www.reddit.com/r/gamedesign/comments/1d2bvop/turn_based_combat_weighted_actions/

[^2_36]: https://www.youtube.com/watch?v=eRvCLaa-3Rk

[^2_37]: https://discussions.unity.com/t/is-it-easy-to-make-a-turn-based-game/474122

[^2_38]: http://ais.informatik.uni-freiburg.de/teaching/ss17/ki/slides/ai03_solving_problems_by_searching.pdf

[^2_39]: https://www.youtube.com/watch?v=yVyQSUYts0A

[^2_40]: https://www.chessprogramming.org/Iterative_Deepening

[^2_41]: https://www.reddit.com/r/gameai/comments/cdx9x6/why_is_monte_carlo_tree_search_being_used_in_turn/

[^2_42]: https://stackoverflow.com/questions/39197338/iterative-deepening-without-specified-depth-limit

[^2_43]: https://opus4.kobv.de/opus4-haw-landshut/files/233/bachelorarbeit_simon_huelkenberg.pdf

[^2_44]: https://www.gamedev.net/forums/topic/644765-hashing-connect-4-board-for-transposition-tableneed-some-help/?page=2


---

# okay so now please give me the whole architecture of how the system will work, which parts are inside and what each part does/handles.

i don't care about the code and implementation side, just the theoretical system design part which i can then finalise so it's easy to quickly implement.

also, i am soon making these changes here and will build the system for those:

- injury effect/debuff gone (pieces sill 2hp but now no debuff)
- old draw conditions out
- board size of 8x8 (following things to test after this lands as successful (hopefully) will be 6x8 board, then piece count lower, then or in between "unified actions" (so moves and skills can be done interchangeably from each other and you just have a set amount of actions)
- steal cost raised to 4 money

Now that you are dropping the injury debuff and tightening the board to 8x8, the engine architecture becomes significantly cleaner. Moving to an 8x8 board is the most impactful decision you could make from a technical standpoint: it maps perfectly to standard 64-bit integers (`u64`). This enables the system to utilize classic, highly optimized bitboard algorithms without the overhead of 128-bit integers or boundary clipping.

Here is the complete theoretical architecture of your board game engine, optimized for the newly reduced complexity.

## 1. State Representation Layer

This is the absolute bottom layer, responsible for holding the exact state of a game at any given moment.

### The 8x8 Bitboards

Because your board is exactly 64 squares, every spatial property is represented by a single `u64`. Each bit corresponds to a square.

- **Occupancy boards:** `P1_Pieces`, `P2_Pieces`
- **Piece Type boards:** `Guards`, `Champions`, `Kings`
- **Skill slots/flags:** If specific skills matter positionally, you can track them as bitboards (e.g., `Has_Dash_Skill_Equipped`).
- **Property boards:** To find all P1 Guards, the system simply uses a bitwise AND: `P1_Pieces & Guards`.


### Scalar State

Not everything fits into 1-bit per square. The engine maintains small, flat arrays for scalar values.

- **Armor \& HP:** Since HP is now binary (Alive/Removed, because the injury debuff is gone), HP is naturally handled by the occupancy boards. You only need an array for Armor: `[u8; 64]` where each index holds 0-3 armor points.
- **Money:** `P1_Money` (u8), `P2_Money` (u8).
- **Turn sequence state:** Current Phase (Move or Skill), Action points remaining, Combo damage counter, and "Pieces moved this phase" bitboard.


### Zobrist Hashing (Crucial for AI)

To know if a position has been seen before without comparing every array, the engine maintains a running 64-bit Zobrist Hash. Every square+piece combination, current phase, and remaining action points are XOR'd together. When a piece moves, you XOR out the old position and XOR in the new one.[^3_1][^3_2][^3_3]

## 2. Action Generation Layer (Move/Skill Generator)

This layer answers the question: "What is legally possible right now?" It translates the current State Representation into a list of mathematical instructions.

### Precomputed Magic Bitboards

Because of the 8x8 board and straight-line "Queen-style" skill paths, the engine can use **Magic Bitboards** (borrowed from chess engine design). During engine startup, it precalculates all possible movement lines for every square and every possible combination of blocking pieces.[^3_4]

- When generating a skill's range or a move path, instead of looping tile-by-tile to see if a piece is blocking the path, the engine creates a lookup index from the current occupancy bitboard. It gets the exact valid target squares instantly in `O(1)` time.[^3_5][^3_4]


### Intra-Turn Action Generation

Instead of generating an entire P1 turn (which still yields thousands of combinations), the generator creates **primitive actions**:

- "Move Guard from A to B"
- "Attack piece C with Champion D"
- "Cast Hook on target E"

These actions are represented as pure 32-bit integers containing the source, target, action type, and cost.

## 3. Forward Engine (Apply/Undo Layer)

This layer executes an action generated by Layer 2 and creates the new state. It also reverses it, which is required to explore game trees without copying memory.

- **Make_Move(Action):** Bitwise XORs the piece from the starting square to the target square, deducts money/action points, processes damage/armor, updates Zobrist Hash.
- **Unmake_Move(Action):** The exact inverse. Restores armor, refunds money, un-pushes a piece.
- **Phase Transition:** A special action that ends the Move Phase and starts the Skill Phase, or ends the turn entirely, collecting income and resetting the Combo Counter.


## 4. Evaluation Function Layer

When the search decides to stop looking ahead, this layer looks at the board and returns a single numerical score (Positive = P1 winning, Negative = P2 winning).

- **Win/Loss Override:** If P1 King is missing, return `-INFINITY`.
- **Material Advantage:** Weighs Kings > Champions > Guards.
- **Resource Advantage:** Evaluates differences in stored Money and remaining Armor.
- **Positional Advantage:** Gives small score bonuses for controlling central 8x8 squares or keeping Champions adjacent to Guards (for the Bodyguard rule).


## 5. The Search and AI Layer

This layer orchestrates the whole system. It uses Layers 1 through 4 to "think" ahead.

### The Search Tree

It uses **Alpha-Beta Pruning with Iterative Deepening**.[^3_6]

1. Searches 1 primitive action ahead.
2. If time remains, searches 2 primitive actions ahead, then 3, then 4.
3. If the user only gave the AI 2 seconds, it stops at whatever depth it was on and returns the best path found so far.

### The Transposition Table

Because skills can be used in different orders to achieve the same exact board state, the search tree will encounter the exact same board multiple times. The Transposition Table checks the Zobrist Hash. If the AI sees a board state it already calculated earlier in the search, it skips it entirely, saving massive amounts of compute time.[^3_7]

### Pruning Mechanics

Because players can skip remaining actions, the AI includes a "Null Move" or "End Phase" action in every generation step. If passing the turn completely still results in an overwhelmingly winning score for the AI (indicating total dominance), it prunes the rest of that branch and stops exploring it.

## System Interaction Flow

1. **Frontend Request:** Web browser (via JS/TS) sends a packed integer array representing the board to the WASM core.
2. **State Load:** WASM core reads the integer array into the `u64` bitboards (Layer 1).
3. **AI Trigger:** Frontend calls `calculate_best_move(time_limit_ms)`.
4. **Search Loop:** Layer 5 begins Iterative Deepening.
5. **Node Expansion:** For every step, Layer 2 generates legal moves, Layer 3 applies them, and if depth limit is reached, Layer 4 evaluates the score. Layer 3 then un-makes the move to reset the board.
6. **Output:** The search time runs out. Layer 5 translates the best integer `Action` back into an array/object and passes it back to the Javascript frontend.
<span style="display:none">[^3_10][^3_11][^3_12][^3_13][^3_8][^3_9]</span>

<div align="center">⁂</div>

[^3_1]: https://en.wikipedia.org/wiki/Zobrist_hashing

[^3_2]: https://minimax.dev/docs/ultimate/efficient-representation/hashing/

[^3_3]: https://www.cs.du.edu/~sturtevant/w13-games/Lecture7.pdf

[^3_4]: https://www.chessprogramming.org/Magic_Bitboards

[^3_5]: https://chessenginelab.substack.com/p/magic-bitboards

[^3_6]: http://ais.informatik.uni-freiburg.de/teaching/ss17/ki/slides/ai03_solving_problems_by_searching.pdf

[^3_7]: https://www.geeksforgeeks.org/dsa/minimax-algorithm-in-game-theory-set-5-zobrist-hashing/

[^3_8]: https://rustic-chess.org/board_representation/zobrist_hashing.html

[^3_9]: https://www.chessprogramming.org/Zobrist_Hashing

[^3_10]: https://github.com/Tearth/Fast-Magic-Bitboards

[^3_11]: https://github.com/vss96/Chess-Engine

[^3_12]: https://www.reddit.com/r/cbaduk/comments/t6refa/how_to_include_score_in_zobrist_hash/

[^3_13]: https://klotzandrew.com/blog/bitboards-in-chess-engines/


---

# okay you made an error, these are my corrections/open questions:

- pieces have **2 hp**. so it's alive at 2, injured at 1, dead at 0
- i am also wondering how exactly we will easily keep track of specific pieces and their skills/hp/armor. i mean is making a separate u64 for each skill, hp state and armor state the correct approach? this would result in 15 for skills, 2 for hp and 3 for armor (armor can be 0-3 but we do not need to track 0 armor as that can be inherently known by the others or do you think it's good to still track it) (hp also only has 2 as having 0 health means the piece is not there anymore)
- in the turn sequence stats: the combo counter is piece wise so we would need another u64 for each possible counter number or something
- will the zobrist hashing not easily result in overlapping hashes? like we do not want to read the same score in the end for two completely different positions
- forward engines phase transition also has the wrong combo counter idea
- evan function layer has the wrong hp idea in resource advantage and also it says "central 8x8 squares" for positional advantage - but the concrete concept for positional advantage can be thought about later, closer to the implementation
- in system interaction flow i would say that "calculate_best_score" also takes in max depth to make it dynamic

Good catches. Your corrections point to a classic crossroads in game engine design: deciding where bitboards stop being useful and where traditional arrays take over.

Here is the updated architecture incorporating your corrections and answering your open questions.

### 1. How to Track HP, Armor, Skills, and Combos

To answer your question directly: **No, making a separate `u64` for every single skill, HP state, Armor state, and Combo count is not the correct approach.**

That is a trap known as "over-bitboarding". If you use 15 bitboards for skills, 2 for HP, 3 for armor, and 5 for combo counters, you end up with 25+ `u64`s. When an attack hits square `42`, your engine would have to check dozens of bitboards just to figure out what skill the piece had and how much HP it has left. That is incredibly slow.

**The Solution: The "Bitboard + Mailbox" Hybrid**
Modern engines use bitboards for *where* things are, and a flat array (the mailbox) for *what* they are.

* **The Bitboards (for routing, masks, and move generation):**
You only keep the spatial ones: `P1_Occupancy`, `P2_Occupancy`, `Kings`, `Champions`, `Guards`.
* **The Mailbox Array (for piece states):**
You maintain an array of 64 integers: `[u16; 64]`. Each index corresponds to a square on the board. You bit-pack all of a piece's specific data into a single 16-bit integer (a `u16`):
    * `HP`: 2 bits (values 0, 1, 2)
    * `Armor`: 2 bits (values 0, 1, 2, 3)
    * `Combo Counter`: 3 bits (values 0 to 7)
    * `Skill 1 ID`: 4 bits (values 0 to 15)
    * `Skill 2 ID`: 4 bits (values 0 to 15)
*(Total: 15 bits, fits perfectly in a `u16`)*.

**How this works in practice:**
When you need to find all Champions to generate moves, you use the `Champions` bitboard. When a Champion on square 12 is attacked, you instantly look at `Mailbox[12]`. You read the Armor and HP from that single `u16`, subtract 1, and write it back. If it moves to square 20, you move the `u16` to `Mailbox[20]` and clear `Mailbox[12]`. It is incredibly fast and keeps your memory footprint tiny.

### 2. Will Zobrist Hashing result in overlapping hashes?

**No, collisions are astronomically rare.**
A `u64` Zobrist hash has $2^{64}$ possible combinations (about 18.4 quintillion). Because Zobrist uses high-quality random number generation for each feature, the hashes are perfectly distributed.

According to the Birthday Paradox, you would need to evaluate and store around $2^{32}$ (4.2 billion) unique board positions in your Transposition Table *simultaneously* before you even reach a 50% chance of a single collision. Since a 4.2-billion-entry table would require roughly 64 Gigabytes of RAM, your search will overwrite old hashes (or run out of time/memory) long before a collision becomes a mathematical probability. Chess engines have trusted 64-bit Zobrist hashes for decades without issue.

### 3. Updated Phase Transition \& Combo Counters

Because the combo counter is piece-wise, the Phase Transition action (ending a turn) handles this cleanly using the Mailbox array.

* **When P1 ends their turn:** The engine uses the `P2_Occupancy` bitboard to find all squares holding P2 pieces. It then loops through those specific indices in the `Mailbox[64]` array and bitwise-clears the 3-bit "Combo Counter" section of those integers back to 0.


### 4. Updated Evaluation Function

* **Resource Advantage:** With the debuff removed, HP is now a pure resource pool. The Evaluation function loops over the active occupancies, reading the Mailbox to sum up total HP and total Armor across the board. A side with 10 total HP is evaluated higher than a side with 8 total HP, even if piece counts are identical.
* **Positional Advantage:** Kept dynamic and abstracted for later implementation.


### 5. Updated System Interaction Flow

The frontend-to-WASM call is now fully dynamic: `calculate_best_move(time_limit_ms, max_depth)`.

* If you set `max_depth = 4`, the Iterative Deepening stops at depth 4 even if `time_limit_ms` has plenty of time left.
* If you set `max_depth = 99`, it relies entirely on the time limit to tell it when to halt and return the best action.

---

# please now write the full and complete high level system architecture of what we will build based on our current notes and pushback and such. the result should be independent of which exact tech stack will be used (of course still considering that this could be used by a independent frontend).

also keep in mind that this game should in par also be playable and not only be used for ai search, so it should be flexible as well. i also have the idea of, as this is still in development, export and analyse games and ask feedback and such which in turn also is saved/exported for me to be able to make progress on developing the game further

Here is the complete, high-level system architecture for your game. This design is tech-stack agnostic, prioritizes ultra-fast AI search, fully supports standard human play, and integrates tools for game balance and analysis.

***

# High-Level System Architecture

The architecture is divided into **six decoupled layers**. The core engine is strictly stateless and mathematical, while the higher layers handle game flow, human interaction, and data collection.

## 1. Core State Representation Layer

*The foundational data structures representing a single moment in time.*

* **Spatial State (Bitboards):** A set of 64-bit integers mapping the board (8x8 = 64 bits). Used for $O(1)$ spatial querying, masking, and collision detection.
    * *Occupancy:* `P1_Pieces`, `P2_Pieces`
    * *Piece Types:* `Kings`, `Champions`, `Guards`
* **Entity State (The Mailbox Array):** An array of 64 packed integers. Each index corresponds to a board square. Instead of searching multiple bitboards for stats, a single integer holds all volatile piece data:
    * `HP` (2 bits: 0=dead, 1=injured, 2=full) — *Note: no debuff, just a resource.*
    * `Armor` (2 bits: 0-3)
    * `Combo Counter` (3 bits: tracks incoming hits this turn)
    * `Skill 1 ID` \& `Skill 2 ID` (4 bits each)
* **Global Resources:** Flat numeric values for `P1_Money`, `P2_Money`, `Current_Phase` (Move/Skill), and `Actions_Remaining`.
* **Zobrist Hash:** A 64-bit integer that uniquely identifies the *exact* current board state, phase, and remaining actions. It is updated incrementally via XOR operations whenever the state changes.


## 2. Game Logic \& Action Pipeline

*The rulebook. This layer knows how pieces move, how skills work, and how state changes.*

* **Magic Bitboard Router:** Precalculated arrays that instantly return valid straight-line paths (Queen-style) for any square, immediately masking out blocked paths using the `Occupancy` bitboards.
* **Action Generator:** Looks at the Core State and generates a list of legal **Primitive Actions** (e.g., "Move Guard A to B", "Cast Hook on C"). It does *not* generate full turns, preventing combinatorial explosion.
* **Make/Unmake Engine:** The most performance-critical component.
    * *Make:* Applies a Primitive Action. It updates the bitboards, modifies the Mailbox array (e.g., deducts HP), spends Action Points/Money, and XORs the Zobrist hash.
    * *Unmake:* Perfectly reverses the action using the exact same logic in reverse. Essential for AI tree search without copying memory.
* **Phase \& Turn Manager:** A specialized Make/Unmake module that handles passing priority. When P1 ends their turn, this module loops through P2's Mailbox entries to reset all Combo Counters to zero, adds income to P2's money, and flips the turn indicator.


## 3. AI \& Search Subsystem

*The intelligence layer. Completely isolated; it simply asks Layer 2 for moves and tests them.*

* **Search Controller (Alpha-Beta + Iterative Deepening):** Explores the game tree primitive-action by primitive-action. It takes `time_limit` and `max_depth` parameters, ensuring it can return a solid move whether it's given 100 milliseconds or 10 seconds.
* **Transposition Table:** A fast hash map keyed by the Zobrist Hash. If the search reaches the exact same board state through a different order of skills/moves, it retrieves the previously calculated score instead of re-searching.
* **Heuristic Evaluator:** Evaluates terminal nodes (or nodes where time ran out) by calculating:
    * *Material:* King presence (Win/Loss override), Champions, Guards.
    * *Resources:* Sum of total remaining HP, Armor, and Money on the board.
    * *Positional:* Flexible hooks for future logic (e.g., proximity of Guards to Kings for the Bodyguard rule).


## 4. Session \& Match Manager (The Controller)

*This makes the engine a playable game, not just a calculator. It sits between the user interface and the core engine.*

* **Match State:** Keeps track of the current game mode (Human vs Human, Human vs AI, AI vs AI).
* **Action History Stack:** Maintains a chronologically ordered list of every Primitive Action taken since Turn 1.
* **Validation Gate:** When a human clicks a skill in the UI, this gate asks the Action Generator (Layer 2) if that move is currently legal. If yes, it passes it to the Make Engine. If no, it rejects the input.
* **State Serialization:** Can convert the entire Core State (Layer 1) into a lightweight string (similar to FEN in chess). This allows you to instantly load specific "middle-game" scenarios for testing.


## 5. Telemetry \& Analytics Subsystem

*The development and balancing toolset, tapping into the Session Manager.*

* **Game Exporter (Notation Generator):** Translates the Action History Stack into a standardized text format (like PGN). You can copy-paste a match log to save it, share it, or re-simulate it later.
* **Telemetry Tracker:** Quietly logs metrics during a match, such as:
    * Which skills were drafted most frequently.
    * Average money hoarded vs. spent per turn.
    * How often the Combo Counter exceeded 2.
    * Average turns until first piece capture.
* **Batch Analyzer:** A tool you can run offline. You feed it 1,000 AI vs AI exported matches, and it aggregates the Telemetry Data to tell you if a specific skill has a 70% win rate (indicating it needs a nerf).


## 6. Frontend / Client Layer (External)

*The visual and interactive shell.*

* **Dumb Renderer:** Reads the Core State from the Session Manager and draws sprites on an 8x8 grid. It does not know the rules of the game.
* **Input Translator:** Translates human clicks (Click Square A, Click Skill 1, Click Square B) into a Primitive Action request, which it sends to the Session Manager.
* **Visual Highlighter:** Asks the API for "Legal Actions for Piece X" to visually highlight valid movement/skill target squares for the human player.

***

### Data Flow Example (Human vs AI):

1. **Frontend** asks Layer 4 for legal moves for the selected Champion. Layer 4 queries Layer 2, returning valid targets.
2. **Frontend** sends a "Cast Skill" action to Layer 4.
3. Layer 4 validates it, tells Layer 2 to `Make_Move()`, and logs the action in Layer 5 (Telemetry).
4. Layer 4 sees the turn is over. It triggers Layer 3 (AI).
5. Layer 3 wildly Make/Unmakes millions of moves in Layer 2, using Layer 1's Zobrist hash to avoid redundancies, and scores them.
6. Layer 3 returns the best Primitive Action. Layer 4 applies it and tells the Frontend to animate the AI's move.

---

# okay now design the technical implementation stack on a theoretical side - not putting out code but just saying how it would then be done like "for bitmaps use rust's ..." and so on, also saying which parts of the system architecture are implemented where.

please research what is best for every part. i want to be able to use this application on every kind of device (phone-web, desktop-web, desktop-app) (via github pages?) and still be quick and whatnot. it should both be usable via mouse/kb and touch input from touchscreens

Here is the theoretical technical implementation stack for your game. This stack is designed to achieve maximum engine performance while keeping the frontend perfectly portable across mobile browsers, desktop browsers, and native desktop applications—all capable of being hosted for free on GitHub Pages.

***

### The Tech Stack Overview

* **Core Engine \& AI:** Rust (compiled to WebAssembly)
* **WASM Bridge:** `wasm-bindgen`
* **Frontend UI:** TypeScript + Svelte (or React) via Vite
* **Desktop App Wrapper:** Tauri
* **Hosting:** GitHub Pages (Static hosting)

***

### 1. Engine \& AI Implementation (Rust)

*Handles Layers 1, 2, 3, 4, and 5.*

**Layer 1: Core State (Data Structures)**

* **Bitboards:** Implemented using Rust's primitive `u64` type. Rust has highly optimized native hardware instructions for bit manipulation (e.g., `my_u64.trailing_zeros()` to instantly find the index of a piece, or `my_u64.count_ones()` to count pieces).
* **Mailbox Array:** A fixed-size array `[u16; 64]`. To handle the bit-packing of HP, Armor, Skills, and Combos cleanly without writing messy bit-shift code by hand, you would use a Rust crate like `modular-bitfield`. This allows you to define exactly how many bits each stat takes up inside the `u16` while keeping the code readable.
* **Zobrist Hashing:** Implemented using a 64-bit integer (`u64`). You will use the `rand` crate to generate a static array of random numbers during the engine's initialization.

**Layer 2: Game Logic**

* **Magic Bitboards:** Implemented using the `OnceLock` or `lazy_static` crate to compute all possible movement lines the very first time the engine boots up, storing them in a globally accessible array for $O(1)$ lookups.
* **Make/Unmake Engine:** Pure Rust functions that take a custom `Action` struct (a simple 32-bit integer wrapper) and apply bitwise `^` (XOR) operations to the `u64` bitboards to move pieces.

**Layer 3: AI \& Search**

* **Transposition Table:** Implemented as a flat Rust `Vec<HashEntry>`. You *do not* use a standard `HashMap` because dynamic memory allocation during search kills performance. Instead, you pre-allocate an array (e.g., 2 million entries) and use `hash_key % array_length` to find the storage index.
* **Web Workers:** AI search is computationally heavy. If you run it on the main web thread, the browser will freeze. You will use `web-sys` and Web Workers to spawn a background thread for the Rust AI. The main thread will say "start thinking," and the worker will send a message back when finished.

**Layers 4 \& 5: Session Manager \& Telemetry**

* **State Serialization:** You will use the Rust `serde` and `serde_json` crates. This allows you to easily serialize the entire game state into a JSON string or a compact FEN-like text string to save or export the game.

***

### 2. The Bridge

* **`wasm-bindgen`:** This tool automatically generates the JavaScript bindings for your Rust code. You will expose a few high-level functions from Rust to your frontend, such as `get_legal_actions(square)`, `apply_action(action_id)`, and `get_best_ai_move(time_limit)`.

***

### 3. Frontend Implementation (TypeScript + Web)

*Handles Layer 6 (The Dumb Renderer \& Input Translator)*

You want a frontend that is responsive, fluid, and handles drag-and-drop easily. While you *could* write the UI in Rust (using Leptos or Dioxus), using a modern web framework like **Svelte** (or React) with TypeScript is vastly superior for handling cross-device touch/mouse events and CSS styling.

* **Board Rendering (CSS Grid):** The 8x8 board is not drawn using an HTML Canvas. It is implemented using standard HTML `<div>` elements and **CSS Grid** (`display: grid; grid-template-columns: repeat(8, 1fr)`). This guarantees the board scales perfectly on a 4K monitor or a tiny smartphone screen without any math.
* **Input Handling (Pointer Events):** To support both a mouse and a touchscreen simultaneously, you will use the modern **Pointer Events API** (`onPointerDown`, `onPointerMove`, `onPointerUp`). Pointer events unify mouse clicks, screen taps, and stylus pens into a single event system, meaning you write the interaction code once and it works on all devices.
* **Visual Highlighter:** When the user taps a Champion, the TS frontend passes the piece index to the Rust WASM module. Rust returns an array of valid target indices. The TS frontend simply adds a CSS class (like `.highlight-red`) to those specific Grid `<div>`s.

***

### 4. Cross-Platform \& Hosting Strategy

**Web Version (GitHub Pages)**
Because your entire engine is just WebAssembly (static `.wasm` file) and your frontend is standard HTML/JS/CSS, the entire game is a **Static Site**. It requires no backend database or server runtime. You simply compile it, push it to GitHub, and GitHub Pages will serve it globally for free. It will run flawlessly on iOS Safari, Android Chrome, and desktop browsers.

**Desktop App Version (Tauri)**
If you want to package the game as a downloadable `.exe` or `.app` for PC/Mac, you will use **Tauri**. Tauri is a Rust-based framework that wraps your existing Web/WASM frontend into a native desktop window. It uses a fraction of the RAM that Electron uses, meaning your desktop app will be incredibly lightweight (often under 10MB) while reusing 100% of the exact same code you wrote for the web version.

**Telemetry \& Analytics (Serverless)**
Since you are using GitHub Pages (which has no backend server to receive your telemetry data), you will implement **Local Data Export**.

* As you or others playtest the game, Layer 5 (in Rust/WASM) builds a log of the match.
* At the end of the match, the frontend generates a downloadable `.json` or `.txt` file via the browser's native Blob API.
* You or your playtesters just click "Export Match Data" to download the file to the device. You can then write a separate small Rust script on your local machine to ingest hundreds of these exported files and run your batch balance analysis.

---

# okay but how do we make it fast? running it via web is slower than running it directly on the system, right? is it possible to have two separate "frontend versions", one for playing on the web and one for using it on desktop (plattform independent, so all OS's work)?

Yes, you are absolutely right to question the performance. WebAssembly is incredibly fast for a browser, but it still runs inside a sandboxed virtual machine. It typically suffers a 10% to 30% performance penalty compared to native machine code, and doing multi-threading (parallel search) in WASM via Web Workers is clunky and requires specific server headers that GitHub Pages doesn't support well.

If you want maximum engine performance for deep AI search, the engine must run natively on the host CPU.

**The good news: You do NOT need to build two different frontends.**

This is exactly where the **Tauri** framework shines. Tauri allows you to use the *exact same web frontend* (HTML/CSS/JS) but swaps out how the core engine is executed depending on the platform.

Here is the theoretical architecture for a dual-target, maximum-performance system.

### The "Shared Core, Dual Target" Architecture

You organize your codebase into a "Workspace" with four distinct parts. The frontend is written once. The core engine is written once. You just wrap them differently for Web vs. Desktop.

#### 1. `core_engine` (Pure Rust)

This is your Layer 1 to Layer 5. It is completely blind to the outside world. It doesn't know what a web browser or a desktop is. It just takes board states, runs Alpha-Beta search, and returns numbers.

* Because it's pure Rust, it can be compiled to literally anything.


#### 2. `frontend_ui` (TypeScript + Svelte/React)

Your single visual interface. It handles drawing the board and taking user clicks.

* Instead of calling WASM directly, it calls a generic "API interface" that you define (e.g., `api.getBestMove()`).


#### 3. Target A: The Web Build (`wasm_wrapper`)

*For quick playing, sharing links, and hosting on GitHub Pages.*

* **How it builds:** You compile `core_engine` to `wasm32-unknown-unknown`.
* **How it connects:** The `frontend_ui` loads the WASM file directly into the browser.
* **Performance:** Fast enough for human play and shallow AI. Single-threaded. Limited to browser memory.


#### 4. Target B: The Desktop Build (`tauri_wrapper`)

*For heavy AI analysis, tournaments, and serious development. Runs on Windows, Mac, and Linux.*

* **How it builds:** You compile `core_engine` to **Native Machine Code** (x86_64 for Intel/AMD, ARM64 for Apple Silicon).
* **How it connects:** Tauri creates a native OS window and loads your exact `frontend_ui` inside a lightweight webview.
* **The Magic:** When the frontend calls `api.getBestMove()`, Tauri intercepts that call and routes it via Inter-Process Communication (IPC) directly to the native Rust backend. The AI runs at 100% native CPU speed, entirely outside the browser engine.

***

### Why the Desktop (Native) Build will be massively faster

By running the engine natively inside the Tauri wrapper, you unlock hardware capabilities that WebAssembly either struggles with or outright blocks:

1. **True Multi-Threading (Lazy SMP):**
In native Rust, you can use the `rayon` crate to trivially spawn 8, 16, or 32 threads. You can have the AI search different branches of the game tree simultaneously, sharing the Transposition Table across threads. This scales beautifully on modern desktop CPUs and drastically increases search depth. Doing this in WASM on GitHub Pages is currently a nightmare of `SharedArrayBuffer` security policies.
2. **Native CPU Intrinsics (SIMD \& Bit manipulation):**
Native compilation can be flagged with `target-cpu=native`. The compiler will look at your exact processor and use advanced instructions (like AVX2 or NEON vectorization) to parallelize bitwise operations. It will also use native POPCNT (population count) for instantly counting pieces on a bitboard, whereas WASM sometimes has to emulate this depending on the browser version.
3. **No Memory Sandbox Limits:**
WASM running in a browser tab is often constrained (e.g., 2GB or 4GB limits). A native desktop app can allocate 16GB of RAM specifically for a massive Transposition Table, meaning the AI remembers vastly more calculated positions, drastically pruning the search tree.

### Summary of the Workflow

When you develop:

1. You edit the rules in `core_engine`.
2. You tweak the UI in `frontend_ui`.

When you want to share with friends:
You run `npm run build:web`. It compiles the engine to WASM, bundles the UI, and pushes a folder to GitHub Pages. People click the link and play instantly on their phones.

When you want to test the AI limits:
You run `npm run build:desktop`. It compiles the engine to native C-level speed, wraps the UI in a native window, and gives you an executable that utilizes 100% of your computer's CPU and RAM to search millions of nodes per second.

---

# in the spirit of testing and playing the game - is it possible to have online support for the app? so i mean that you can start a game with people from all over the world, but not having to rent a server and so on and basically make it in a way that both players have the core running for them (and also check if their moves are legal from the game's perspective locally) but then sync game states via the host.

is this well supported in our tech stack and can we just add in another module to the system design and make it work? also so that people can either use the web _or_ the desktop frontend? (or we make separate multiplayer support builds for both?)

please first research and explore which different approaches exists and then finetune them and select the best one based on my requirements. in the end it should be like the minecraft mod essential, where you just share your "world" and another player can join (for our purpose it's a game or session we share maybe). it should be secure so no real harm can be done if any side has ill intentions (either host or joining person)

Now I have a thorough picture. Here is the full analysis of approaches followed by the selected design and how it integrates as a new module.

***

## Approaches Considered

Before jumping to the recommendation, here are the realistic options for your case, each evaluated against your constraints:


| Approach | How it works | Cost | Cheat safety | Works in browser | Works in Tauri | Verdict |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **Dedicated Server** | Central game server holds truth | Rent a server forever | ✅ Server is authority | ✅ | ✅ | Rejected — costs money |
| **Serverless (Vercel/Cloudflare Workers)** | Pay-per-use functions + managed WebSockets | Near-zero for low traffic | ✅ | ✅ | ✅ | Viable fallback but adds build complexity |
| **P2P via WebRTC (raw)** | Direct browser-to-browser, signaling server only for handshake | Free | ❌ host can cheat trivially | ✅ | ✅ | Only works with cheat protection |
| **P2P via WebRTC + Cryptographic Commitment** | Same as above but each action is hashed before being revealed | Free | ✅ mathematical proof | ✅ | ✅ | **Selected** |


***

## Why the Selected Approach Works for Your Game Specifically

Your game has a massive hidden advantage for P2P networking: it is **fully deterministic** and **turn-based with no hidden information**. Every player already has the complete game state on screen. This means:[^8_1][^8_2]

1. You never need to sync the entire board state every frame. You only sync **actions** (tiny integers).
2. Because both players run the identical Rust engine locally, applying the same action on both sides produces the exact same board. There is no "server truth" that needs to be enforced. The players *are* the server.
3. Because turns are discrete and slow (not real-time), you tolerate latency trivially. A 300ms ping doesn't matter when turns take seconds.

***

## Layer 7: The Multiplayer Network Module

This is a self-contained new module added to your system architecture. It sits between the Session Manager (Layer 4) and the Frontend (Layer 6), and activates only when a multiplayer session is started.

### 7.1 — The Signaling Handshake (Connection Setup)

Before two players can talk directly, they need to find each other. This is the one unavoidable moment where a tiny outside server is needed. You use **PeerJS** with its free hosted `PeerServer` for this.[^8_3][^8_4]

- The host clicks "Start Online Game." PeerJS generates a short, random session ID (e.g. `"x7kp-qr8m"`).
- The host shares this ID with their opponent, exactly like an Essential world invite code.
- The opponent enters the code and PeerJS's server brokers a single handshake.
- After that, all data flows **directly** between the two clients. It never touches PeerJS's servers again.[^8_4]

For the roughly 10–20% of users behind very strict routers (symmetric NAT), the direct path fails. In that case, **STUN/TURN** relays are used as a fallback. For development and testing, Google's free STUN server (`stun.l.google.com:19302`) works. For production reliability, ExpressTURN and similar services offer a free tier. The TURN server only ever relays encrypted game action integers — extremely low bandwidth.[^8_5][^8_6]

### 7.2 — The Sync Protocol (Deterministic Lockstep)

Because your game is turn-based, you use a simplified version of **Deterministic Lockstep**:[^8_2][^8_1]

1. Player A decides on their action locally (e.g. "Cast Hook on square 34").
2. **Before revealing it**, Player A computes a hash of the action integer and sends it to Player B as a "commitment."
3. Player B sends their own commitment (if it is their phase).
4. Once both commitments are exchanged, both players reveal the actual action integers.
5. Both players verify that the revealed action matches the previously committed hash. If it matches, both clients pass the action through their local Make Engine (Layer 2), producing identical new board states.

This "commit then reveal" pattern is the standard cryptographic solution to P2P game cheating. Neither player can change their action retroactively after seeing the opponent's commitment, because the hash would no longer match.[^8_7][^8_8][^8_1]

### 7.3 — Local Validation Gate

Before accepting any action from the network, **each client passes it through their own Action Generator (Layer 2)** and checks that it is in the legal action set for that position. If the incoming action is not in the legal list, the client immediately flags it as invalid and terminates the session. This means neither player can inject impossible moves — even if they hack their own client.[^8_1]

### 7.4 — Desync Detection

Because both clients are running the same deterministic Rust engine, their Zobrist Hashes must match perfectly after every action. At every turn boundary, both clients exchange their current Zobrist Hash over the P2P channel. If the hashes ever diverge, the session knows a desync has occurred. The client can then request a full state resync by asking the host to serialize and send the current State (Layer 4's serialization tool). This is a pure safety net — with a deterministic engine it should essentially never fire.

### 7.5 — Cross-Platform Support (Web and Desktop)

Both the Web and Desktop builds use the exact same multiplayer module. The network protocol (WebRTC DataChannel) is natively available in every major browser and is accessible from Tauri's webview layer without modification. A desktop Tauri user and a mobile web browser user can play each other seamlessly, because the network layer lives entirely in the TypeScript/JS frontend layer and connects the same way regardless of how the Rust engine below it is packaged.

***

## What You Can and Cannot Protect Against

| Threat | Protected? | How |
| :-- | :-- | :-- |
| **Host revealing a fake action** | ✅ | Commit-then-reveal hash |
| **Player submitting an illegal move** | ✅ | Local legal validation on both clients |
| **Player injecting impossible game states** | ✅ | Zobrist hash desync check |
| **IP address exposure** | ✅ (with TURN) | TURN relay hides IPs from both sides [^8_5] |
| **Player lying about their Zobrist hash** | ⚠️ Detectable only | Raises a desync alert; forces resync; no silent corruption |
| **Player modifying their local AI to help them** | ❌ Not preventable | Inherent to any P2P architecture without a trusted server |

The last point is an accepted limitation of all serverless P2P games. Since your game has no hidden information, there is nothing to "see" that cheating gives access to. A player modifying their own AI just means they have a stronger AI assisting them, which is not a game-integrity issue in a casual development context.
<span style="display:none">[^8_10][^8_11][^8_12][^8_13][^8_14][^8_15][^8_16][^8_17][^8_18][^8_19][^8_20][^8_21][^8_22][^8_23][^8_24][^8_25][^8_26][^8_27][^8_28][^8_29][^8_30][^8_31][^8_32][^8_33][^8_34][^8_35][^8_36][^8_37][^8_38][^8_39][^8_40][^8_41][^8_42][^8_43][^8_9]</span>

<div align="center">⁂</div>

[^8_1]: https://en.wikipedia.org/wiki/Lockstep_protocol

[^8_2]: https://ruoyusun.com/2019/04/06/game-networking-3.html

[^8_3]: https://peerjs.com

[^8_4]: https://peerjs.com/client/faq

[^8_5]: https://www.expressturn.com

[^8_6]: https://bloggeek.me/webrtcglossary/stun/

[^8_7]: https://sol.sbc.org.br/index.php/sbseg/article/download/19544/19372/

[^8_8]: https://www.reddit.com/r/cryptography/comments/mtqyon/cryptographic_protocol_which_provides/

[^8_9]: https://dev.to/bornfightcompany/using-webtrc-for-a-browser-multiplayer-game-in-theory-59dk

[^8_10]: https://www.pushflo.dev/blog/multiplayer-games-serverless

[^8_11]: https://stackoverflow.com/questions/33551206/should-i-use-websockets-or-webrtc-for-4player-game

[^8_12]: https://medium.com/@aguiran/building-real-time-p2p-multiplayer-games-in-the-browser-why-i-eliminated-the-server-d9f4ea7d4099

[^8_13]: https://docs.aws.amazon.com/wellarchitected/latest/games-industry-lens/serverless-backend.html

[^8_14]: https://stackoverflow.com/questions/71943205/websocket-vs-webrtc-for-multiple-users-in-same-channel/71947489

[^8_15]: https://discourse.threejs.org/t/html5-multiplayer-games-over-udp-client-server-using-geckos-io/15896

[^8_16]: https://aws.amazon.com/blogs/gametech/online-multiplayer-amazon-gamelift-aws-serverless/

[^8_17]: https://www.reddit.com/r/gamedev/comments/1eijh7t/webrtc_vs_websockets_for_browserbased_coop_game/

[^8_18]: https://forum.babylonjs.com/t/theory-on-how-to-make-a-multiplayer-game/3327

[^8_19]: https://dev.to/jeremyckahn/how-i-designed-an-abuse-resistant-fault-tolerant-zero-cost-multiplayer-online-game-140g

[^8_20]: https://www.webgamedev.com/backend/webrtc

[^8_21]: https://github.com/rameshvarun/netplayjs

[^8_22]: https://aws.amazon.com/jp/blogs/compute/building-a-serverless-multiplayer-game-that-scales/

[^8_23]: https://www.hirevoipdeveloper.com/blog/webrtc-vs-websockets/

[^8_24]: https://www.innovateasterisk.com/stun-turn-webrtc-asterisk-nat-traversal/

[^8_25]: https://www.youtube.com/watch?v=_n-c1VZfksE

[^8_26]: https://edgegap.com/blog/stun-servers-for-peer-to-peer-networking-in-multiplayer-games

[^8_27]: https://gamedev.net/forums/topic/708508-lockstep-protocol-for-multiplayer-reducing-latency/

[^8_28]: https://www.reddit.com/r/javascript/comments/lpe2ai/netplayjs_make_multiplayer_p2p_browser_games_no/

[^8_29]: https://stackoverflow.com/questions/59906509/how-do-you-get-around-nats-using-webrtc-without-a-turn-server

[^8_30]: https://forum.babylonjs.com/t/how-do-i-use-deterministic-lockstep-to-help-sync-clients/17008

[^8_31]: https://getstream.io/resources/projects/webrtc/advanced/stun-turn/

[^8_32]: https://forum.babylonjs.com/t/html5-multiplayer-games-over-udp-client-server-using-geckos-io/11436

[^8_33]: https://discussions.unity.com/t/deterministic-physics-for-lockstep-networking-any-progress/585635

[^8_34]: https://www.reddit.com/r/godot/comments/kxb62c/godot_p2p_multiplayer_project_using_webrtc/

[^8_35]: https://stackoverflow.com/questions/64506949/can-i-use-peerjs-without-a-peerjs-server

[^8_36]: https://edgegap.com/blog/webrtc-relays-for-multiplayer-games

[^8_37]: https://sourceforge.net/projects/peerjs.mirror/

[^8_38]: https://digikogu.taltech.ee/en/Download/76a1ab84-2dd4-48f4-a26d-08432cea0393

[^8_39]: https://news.ycombinator.com/item?id=15893197

[^8_40]: https://peerjs.com/server/getting-started

[^8_41]: https://arxiv.org/html/2501.10881v1

[^8_42]: https://github.com/kevglass/simple-webrtc-p2p-game

[^8_43]: https://dev.to/primocys1/webrtc-peerjs-vs-sfu-choosing-the-best-for-your-real-time-chat-app-464c

