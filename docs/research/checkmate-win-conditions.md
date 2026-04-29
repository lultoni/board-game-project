# "Position Decided" Win Conditions in Perfect-Information Tactical Games

## Executive Summary

This report is a design-focused analysis of positional win conditions — checkmate and its analogs — across six canonical abstract games and several modern tactical games, with specific attention to the five design questions raised by a game featuring Champions, ranged skills, and Rune-activated abilities on a 10×10 grid. The core finding is that every successful checkmate-style system achieves clarity through tight *scope limitation*: the number of pieces capable of delivering the decisive threat is small, the verification logic maps onto a visual pattern, and the rules aggressively prune infinite-loop escapes. When ranged and combo effects broaden that scope, the verification burden climbs sharply — and the design challenge shifts from "define the win state" to "make the win state verifiable at the table without a calculator."

***

## Part 1: Formal Definitions of "Inescapable Position"

### Chess (FIDE)

The FIDE Laws of Chess, Article 1.4, state: *"The objective of each player is to place the opponent's king 'under attack' in such a way that the opponent has no legal move. The player who achieves this goal is said to have 'checkmated' the opponent's king and to have won the game. Leaving one's own king under attack, exposing one's own king to attack and also 'capturing' the opponent's king are not allowed."* Article 5.1(a) adds: *"The game is won by the player who has checkmated his opponent's king. This immediately ends the game, provided that the move producing the checkmate position was a legal move."*[^1][^2]

The definition has two necessary and jointly sufficient conditions: (1) the king is in check — i.e., the square it occupies is controlled by an enemy piece — and (2) there is no legal move for the defender (king move to an unchecked square, interposition, or capture of the attacking piece) that removes the check. The king is never actually captured; the game ends the instant the position is reached. The crucial boundary condition is *stalemate*: if the king is not in check but has no legal moves, the game is drawn rather than won — a rule that has been deeply controversial throughout chess history and which effectively punishes overaggressive play.[^3][^4][^5][^6]

**Historical note:** In medieval Shatranj, chess's direct ancestor, stalemate was actually a *win* for the attacking player in many regional traditions, and capturing the king was the original path to victory before checkmate became the standard around 1300–1600 CE. The shift to checkmate was partly a matter of courtly etiquette — the king should be shown mercy rather than captured — and partly because requiring announcement of check made incomplete positions less frequent. Notably, the modern rules are still debated: Theodore Tylor (1940) and GM Larry Kaufman have argued that stalemate-as-draw is irrational game design that inflates the draw rate.[^7][^8][^9][^10]

### Xiangqi (Chinese Chess)

Xiangqi offers two win conditions: checkmate and stalemate, and unlike Western chess, **both are wins for the side delivering them**. The formal definitions are:[^11][^12]

- **Checkmate:** "A position in which your opponent's King is under attack (in check) and he has no legal move to prevent his King from being captured."[^11]
- **Stalemate:** "A position in which your opponent has no legal move with any of his pieces, although his King is not in check." This is explicitly a *win* for the stalemating player, not a draw.[^12][^13][^11]

This stalemate-as-win rule is significant for design: it removes the tactical escape hatch that allows a materially inferior player in Western chess to force a half-point by giving the opponent no move. In Xiangqi, running out of moves is unconditionally losing. This also means there is no category of "close but not quite" position that benefits the defender.

An additional structural constraint in Xiangqi is the "Flying General Rule" (面将 / *miàn jiāng*): the two generals cannot face each other along the same file or rank with no pieces between them. This creates an implicit long-range mutual threat and is often the mechanism of both checkmate and stalemate patterns.[^14]

### Shogi (Japanese Chess)

Shogi's formal win condition is checkmate (*tsumi*, 詰み), defined identically to Western chess: the player whose king is in check and has no legal move to remove it loses. However, Shogi's unique complexity around this definition is driven by the **drop system** — captured pieces are retained and may be reintroduced to the board anywhere. This has two major formal implications:[^15]

1. **Pawn-drop mate (*uchifuzume*) is explicitly illegal:** You cannot win by dropping a pawn that delivers immediate checkmate. Dropping that same pawn to create a *threat* of mate is legal; only the drop itself completing mate is forbidden. This rule exists because pawn drops to create a blocking wall or mating net would be too powerful without it.[^16][^17][^18]
2. **Double pawn (*nifu*) is illegal:** A player may not have two unpromoted pawns in the same column. If a drop creates this state, the dropping player immediately *loses the game* — not merely forfeits the move.[^16]

The concept of **brinkmate (*hisshi*, 必至)** is Shogi's analog to "position decided": it refers to a position where an unavoidable *tsume* sequence will be created on the next move, no matter what the opponent does. Brinkmate is distinct from *tsumero* (threatmate), which is escapable with correct defense. The distinction between hisshi (unavoidable) and tsumero (avoidable) is a practical hierarchy players use to track how "decided" a position is before the actual forced line.[^19][^20]

Shogi also has a strict rule that making an **illegal move** — whether a drop violation, moving into check, or leaving the king in check — results in **immediate forfeit of the game**. This penalty structure is harder than in Western chess (where the first illegal move merely grants the opponent extra time) and dramatically increases the cognitive burden on players in complex positions.[^21][^15]

### Xiangqi Anti-Pattern Rules

Both perpetual check and perpetual chase are explicitly **losing** for the offender in Xiangqi, not draws. The formal distinction:[^22][^23]
- **Perpetual check:** A sequence of consecutive checks that repeats the position — the checking player loses.[^23]
- **Perpetual chase:** A repeated sequence where one side continuously attacks the same unprotected opponent piece — the chasing player loses.[^24][^22]
- **Neutral repetition:** If a position repeats three times with no check or chase, the game is a draw.[^23]

The exact rules for what constitutes a "chase" vs. a "waiting move" vs. an "offer" are notoriously complex; the Asian Xiangqi Federation rulebook defines over a dozen sub-categories of move types (check, kill, chase, exchange, block, offer) and cross-references them.[^25]

### Arimaa

Arimaa has three distinct win conditions, applied in this order of checking priority after each move:[^26][^27]
1. **Goal:** A rabbit reaches the opponent's home rank (the opposite edge of the board). This is the primary intended win path.
2. **Immobilization:** If the opponent has no legal move on their turn — because all pieces are frozen (surrounded by stronger enemy pieces on all sides without a friendly piece to "unfreeze" them) or have nowhere to move — the opponent loses.[^28][^29]
3. **Elimination:** If all of the opponent's rabbits are captured (into traps), the opponent loses.[^28]

Critically, the immobilization condition is checked *before* the opponent moves — and if the *only* moves available to the opponent would constitute a third repetition of the game state, those moves are barred, also resulting in the opponent's loss. This last clause is Arimaa's anti-loop mechanism.[^27]

### Hive

Hive has one win condition: completely surround your opponent's Queen Bee. The formal rule is simple: *"The game ends in a draw if both Queen Bees are surrounded in the same turn; otherwise the player whose Queen Bee is surrounded loses the game."* Surrounding means all six adjacent hexagonal cells are occupied; pieces of either color count. A draw can also occur by agreement or by repetition.[^30][^31][^32][^33]

The rule achieves radical clarity: the only condition requiring verification is "are all six spaces adjacent to the queen occupied?" This is a pure visual check with zero ambiguity.

### Tak

Tak has two win conditions with a strict priority ordering:[^34][^35]
1. **Road win (primary):** Complete a contiguous path of your flat stones and/or capstone connecting two opposite edges of the board (not diagonals). A standing stone cannot be part of a road.[^36][^34]
2. **Flat win (secondary):** If either player places their last piece *or* the board is completely filled, count the flat stones on top of stacks; most flat stones wins.[^37][^34]
3. **Double road (tiebreaker):** If a move creates a winning road for *both* players, the active player wins.[^38][^35]

"Tak" (analogous to "check") is called as a courtesy when a player is one move from completing a road, not as a formal rule requirement.[^34]

***

## Part 2: Anti-Stalling and Anti-Repetition Rules

### The Core Problem

All positional win conditions face the same threat: a losing player who can loop a sequence indefinitely to prevent the loss. Every major game has solved this differently.

| Game | Repetition Rule | Outcome | Notes |
|------|----------------|---------|-------|
| Chess | Threefold repetition (3× same position, any order) | **Draw** (must be claimed) | Fivefold triggers automatic draw[^39][^40] |
| Chess | Fifty-move rule (50 moves without capture/pawn move) | **Draw** (must be claimed) | Anti-grinding safety valve |
| Xiangqi | Triple repetition with perpetual check | **Loss for checker** | Enforced automatically online[^23] |
| Xiangqi | Triple repetition with perpetual chase | **Loss for chaser** | Complex case definitions[^22] |
| Xiangqi | Triple repetition, neutral | **Draw** | Both sides innocent[^23] |
| Shogi | Fourfold repetition (*sennichite*) | **Draw** (game replayed in pro play) | Perpetual check = **loss**[^41][^42] |
| Shogi | Perpetual check in repetition | **Loss for checking player** | Even if part of a larger loop[^43][^44] |
| Arimaa | Available moves are all third-repetitions | **Loss for that player** | Prevents repetition-as-defense[^26][^27] |
| Go | Ko rule (no immediate repeat) | **Illegal move** | Super-ko: no repeat of any prior board state[^45] |
| Hive | Repetition by agreement | **Draw** | Informal; no formal rule count[^32] |

**Key design insight:** Games that assign *blame* for repetition (Xiangqi, Shogi) avoid the problematic outcome where stalling grants a draw. The distinction between "aggressive repetition" (perpetual check/chase) and "passive repetition" (both sides maneuvering) is conceptually clean but operationally very complex in Xiangqi — leading to the multi-category rule system. Shogi's solution is cleaner: if the *attacker* is generating checks in the loop, the attacker loses; otherwise it is a draw.

### The Chess Stalemate Problem

The Western chess stalemate rule illustrates a genuine failure mode of checkmate-style design: **it rewards the defender for achieving a position that, in any other chess-family game, would be a loss**. In the most ancient forms like Shatranj, stalemate was a win for the aggressor; French and Italian traditions moved it to a draw; FIDE codified the draw in the early 20th century. The ongoing controversy — with multiple GMs arguing stalemate should be a win — reflects a real tension: the rule exists to discourage careless overadvantage play, but it generates a significant proportion of high-level draws and is considered "bad game design" by a vocal minority.[^46][^7]

***

## Part 3: Dual Win Conditions — Positional AND Material

### Seven Wonders Duel

The clearest modern example of a game with explicit alternative win conditions is *7 Wonders Duel* (Bauza & Cathala, 2015). It has three win types:[^47]
1. **Military Supremacy (positional):** Push the conflict pawn all the way to the opponent's capital — an immediate win at any point in the game.[^48]
2. **Scientific Supremacy (positional):** Collect 6 different scientific symbols — an immediate win at any point.[^48]
3. **Civilian Victory (points-based):** Count total victory points at the end of Age III if no supremacy occurred.[^48]

In practice, Supremacy wins function primarily as **deterrents** rather than primary win paths. Community play data shows points victories dominate, with military and science wins occurring when a player makes a "pretty egregious mistake" by allowing the opponent to go unchecked. However, the looming threat of an instant win forces both players to make non-optimal resource choices and creates meaningful tension throughout the game. One framing captures the mechanic perfectly: "the primary function of science and military is to *pressure* your adversary to spend their turn blocking rather than being genuine routes to victory on their own."[^49][^50]

**Design implication:** Dual win conditions are most effective when the alternative (non-dominant) win is held in *latent threat* status rather than serving as a primary path. This matches the game-design scenario described: the "position decided" win would primarily function to end games that are *already decided*, preventing grind-out, rather than serving as the predominant competitive path.

### Advance Wars

Advance Wars has two win conditions: **HQ capture** (positional: occupy the opponent's headquarters for one full capture) and **annihilation** (material: destroy all opponent units). These interact in a rich strategic dynamic: HQ capture requires a specific infantry-type unit to reach and capture a fortified location over multiple turns, while annihilation requires overwhelming combat power. Maps are specifically designed to make one or both conditions accessible, sometimes requiring players to choose between speed (HQ capture) and security (gradual annihilation). The HQ capture win is typically faster but riskier; annihilation is slower but avoids an exposed HQ.[^51][^52][^53][^54]

### Santorini

Santorini has a **movement-based primary win** (move a worker up onto a Level 3 building space) and each God Power card adds or modifies win conditions. The base win is positional (reach a specific height) but God Powers add material and structural alternatives. The interaction dynamic here is almost purely a threat-blocking game: since capping a Level 3 building with a dome prevents the opponent from using it for a win, the strategy is almost entirely about building and blocking structures simultaneously. The game's designer effectively made the win condition itself the core tactical resource.[^55][^56][^57]

### Warhammer 40,000 (Current Edition)

Modern competitive 40K uses a **mission score primary / annihilation secondary** system. Players score VP for completing objectives each round; tabling the opponent (eliminating all units) immediately ends the game with both scoring being settled as of that moment. In tournament play, players who treat the game as purely a deathmatch typically lose because roughly half of available points come from mission primaries, not kills. This is a clean example where a positional system and a material system interact: neither dominates abstractly, but the mission system is designed to generate more points, steering rational play toward objectives while kills remain a powerful force multiplier.[^58][^59]

***

## Part 4: Verification Without Exhaustive Calculation

### The Chess Solution: Pattern Libraries

Chess checkmate verification is fast in practice because players build up a library of pattern recognition for mate-in-one and mate-in-two positions. The position is verified visually — "king has moves to A, B, C; A is covered by bishop, B by queen, C by rook, no interposition pieces, no capture of attacker possible" — without a full search tree. At amateur level (rated 1200–1800), verification of a claimed checkmate takes 10–30 seconds; at master level, mates that are not immediately obvious are typically set up through a sequence of announced forcing moves that both players track in real time.[^60]

The resignation culture in chess is the practical solution to verification burden: in serious play, checkmate almost never actually occurs on the board at professional level because the losing player resigns as soon as forced mate is recognized. The formal win condition defines the *endpoint*, but the practical game-ending mechanism is mutual recognition of inevitability.[^60]

### Shogi: Tsume Notation and the Drop Explosion

Shogi's verification problem is dramatically harder than chess because of drops. After a forced check, the defender may have 200–300 possible responses including drops of any piece in hand. *Tsume shogi* problems — formal checkmate puzzles — require the attacker to force mate with *every* move being check; any non-checking move by the attacker is illegal in the puzzle context. In real games, players use a simpler practical heuristic: establish *hisshi* (brinkmate), defined as a position where the opponent cannot prevent the creation of a forced checkmate sequence even if given a free move.[^61][^20][^62][^63][^64][^19]

The practical table experience is that in shogi, **intermediate-level players frequently make illegal moves** — the *nifu* (two-pawn-in-column) error is common even among experienced amateurs. The strict penalty (immediate loss) means these errors are treated as genuine game-ending events rather than take-backs. In digital play this is not an issue; over-the-board, it creates significant friction for newer players.[^17][^18]

### Arimaa: Goal Threats as Verification Proxies

In Arimaa, the goal threat is the primary win-condition verification tool. If a rabbit can reach the goal row on this turn (with the current step budget), the player wins immediately. Verification of whether a *future* goal is "inescapable" is heuristic: players assess the number of steps remaining, the rabbit's distance to the goal row, and whether enemy pieces can freeze or block the path within that time horizon. The formal rules help here by limiting each player to four steps per turn; this bounds the search horizon for threat verification to a manageable look-ahead.[^65][^27][^28]

### The Ranged-Attack Problem (Novel Game Context)

For a game with ranged attacks, heals, pushes, and buffs interacting across multiple tiles, the verification problem has a known analog in **brinkmate search in Shogi**, which researchers describe as "by far more expensive than mating (tsume) search to find a solution" — the defending side's response space is broader, and the attacker must demonstrate that *no* defensive response (including every combination of drops and moves) prevents the eventual mate. Computer brinkmate solvers use AND/OR tree search with proof-number methods; this is not feasible at the table without simplification rules.[^64]

**Practical paths forward for such a game:**

1. **Named threat categories (Xiangqi-style):** Define a finite vocabulary of win-threatening configurations — e.g., "King is in [state X]" — and encode whether each defensive option (heal, push, armor) can resolve it. This is the table-verification analog of Xiangqi's multi-category rule system.
2. **Explicit escape options:** Rather than asking "can the King be saved?" ask "does the King have any of these specific escape options: (a) move to unchecked square, (b) block with an adjacent ally, (c) heal above kill threshold, (d) push attacker out of range?" If none exist, the position is formally decided.
3. **Kill-stack threshold:** Define a decisive position as one where the attackers' combined damage in one activation exceeds the King's maximum survivable HP even with one heal applied — a numeric calculation bounded by known values.
4. **Hisshi-style design:** Declare that the game ends when a player creates a "forced threat" that is demonstrably inescapable *in the same manner each time*, using a defined lookup: if this exact pattern is on the board and the specific list of counters is unavailable, the position is decided.

***

## Part 5: Known Failure Modes of Checkmate-Style Win Conditions

### Failure Mode 1: Checkmate Too Easy → Shallow, Unsatisfying Games

In games where the King is highly mobile and the attacking player can force mate quickly, mid-game strategy collapses into a race to threaten the King directly rather than building a comprehensive position. This is actually a cited concern in Shogi design discussions about the pawn-drop ban: the rule banning checkmate by pawn drop (*uchifuzume*) exists because without it, dropped-pawn mates would be too frequent and would make the game's mid-game irrelevant. Similarly, in Chess variants where the queen gains excessive mobility, forced mates in the opening ("Scholar's Mate" type patterns) become a common spoiler that must be specifically patched.[^17][^16]

### Failure Mode 2: Inescapable Position Too Hard to Prove → Table Arguments

When the verification burden is high, games develop two pathologies: (a) players declare a "winning position" prematurely and then have to retract it when the opponent finds a defensive resource, eroding confidence in the win condition; (b) players continue playing a "dead" game because neither side can formally prove the mate, generating exactly the grind problem the checkmate condition was meant to solve.

Shogi's brinkmate/threatmate distinction exists precisely to handle this: the game convention establishes that *tsumero* (threatmate) is not sufficient to end the game — the attacker must verify *hisshi* (that it is unavoidable on the next move) or play out the tsume sequence. This means there is always a well-defined "lower bar" (checkmate itself) that is formally verifiable, even if the higher-bar "position decided" concept is harder to apply.[^19]

### Failure Mode 3: Stalemate Draws Too Common (Chess-Specific)

In Western chess, the draw-as-stalemate rule actively incentivizes a losing player to force a position where they have no legal move, converting what would be a loss in every other chess-family game into a draw. This increases the overall draw rate, inflates the value of defensive play, and creates situations where an overwhelming material advantage is neutralized by a technicality. The rule is often cited as the biggest single design flaw in modern FIDE chess.[^5][^7]

### Failure Mode 4: Dual Win Conditions Create an Ignored Win Path

If one win condition is significantly harder to achieve than another, the harder path becomes strategically irrelevant and is only pursued as a forcing-threat. This is the dominant-path problem. In *7 Wonders Duel*, points victories dominate in well-matched play, and military/science wins are primarily deterrents. In Advance Wars, the HQ capture is typically the faster path on most maps but is map-design-dependent, so neither path universally dominates.[^50][^54][^49]

### Failure Mode 5: Anti-Repetition Rules Create Their Own Disputes

The complexity of Xiangqi's anti-perpetual-chase rules is a recurring source of adjudication disputes. The rule system requires distinguishing "chase" from "exchange invitation," "protected piece" from "unprotected piece," and "perpetual block" from "perpetual chase." Online implementations handle this algorithmically, but over-the-board play requires judges for contested repetition sequences. Simpler systems (Go's ko rule, Shogi's four-fold repetition with perpetual-check penalty) produce fewer disputes because the boundary conditions are more clearly visible.[^22][^25][^24]

### Failure Mode 6: Illegal-Move Penalties Punish New Players Disproportionately

Shogi's rule that *any* illegal move results in immediate loss is an aggressive enforcement mechanism that creates a stark division between experienced and inexperienced players. Over-the-board, this means players who know the win condition are incentivized to watch for an opponent who inadvertently drops into an illegal state, turning verification into a trap. In digital environments this is irrelevant, but in tabletop design it raises the question of whether forfeit-on-illegal-move or restart-on-illegal-move is the better default.[^21][^15][^17]

***

## Part 6: Synthesis — Design Recommendations for the 10×10 Champion Game

### The Core Design Problem

The game described has a "King capture" win condition where outcomes are decided 5–10 rounds early, creating a grind. The proposed alternative — ending the game when a "checkmate-style" inescapable position is reached — is exactly the approach used in Chess, Shogi, and Xiangqi. The structural barriers to implementing it cleanly are:

1. **Range expands the threat space.** In Chess, only pieces adjacent to the king's possible escape squares need evaluation. With ranged attacks, healers, pushers, and buffs, the set of pieces relevant to "can the King be saved?" can be the entire board.
2. **Healing and repositioning extend the defender's option space.** Shogi drops are analogous — they expand defensive responses beyond what's visible on the board — and Shogi addresses this through strict move-type categorization, formal puzzle culture, and the hisshi/tsumero distinction.
3. **The game has ~30 rounds, meaning the "decided" state occurs at round 20–25.** This is a well-known design problem: the grind is the gap between *strategic* decision (position is decided) and *formal* decision (game ends).

### Recommended Approaches (in increasing complexity)

**Option A: Surrender Convention + Optional Formal Rule**
The simplest approach is to codify what professional chess players already do: add an explicit "the losing player may concede immediately when they recognize a forced outcome" combined with a "mutual acknowledgment" rule where both players agreeing the position is decided ends the game. This requires no new formal win-condition logic and directly solves the 5–10 round grind. The cost is that it depends on player agreement, which fails when players disagree about the assessment.

**Option B: Bounded Threat Protocol**
Define a formal "Threatened King" declaration that can be made when: (a) the King will receive at least *K* damage from already-placed pieces this turn, (b) the damage exceeds the King's remaining HP + max one-round heal value, and (c) the King has no movement path to an unchecked square within one action. If the opponent cannot prove one of these conditions false within 60 seconds, the position is formally decided. This is the brinkmate equivalent for the game's system.

**Option C: Named Decisive Patterns**
Develop a card or reference sheet listing "decisive patterns" — specific board configurations where a King at N HP with M threats within R range and no specified defensive options is formally decided. This converts open-ended verification into pattern matching, the same mechanism that makes Chess checkmates fast to verify at the table.

**Option D: Dual Win + Clock**
Keep the King-capture win condition but add a secondary "position decided" mechanism that triggers an earlier end: if the defending player cannot reduce the attacker's projected next-turn damage below the King's survivability threshold within their current turn, they lose. This is essentially the "you have one action to survive" rule, and it reduces the grind to at most one additional turn after the position is definitively established.

### On Anti-Repetition for This Game

If the game is susceptible to loops — a King being pushed back and forth while protected, preventing the decisive blow — Arimaa's mechanic of "if your only available moves are repetitions of the last three-move sequence, you lose" is directly applicable and table-verifiable. Alternatively, Shogi's principle — "aggressive repetition (perpetual check equivalent) loses; neutral repetition draws" — could be adapted as: "if the same board state recurs three times and one player is the aggressor in each repetition, the aggressor loses; otherwise the game draws."

***

## Summary Reference Table

| Game | Win Condition | Boundary (Decided vs. Possible) | Repetition Rule | Stalemate | Verification Burden |
|------|--------------|-------------------------------|----------------|-----------|---------------------|
| Chess | King in check, no legal move | Checkmate vs. not-checkmate (exact) | 3-fold = draw (claimed) | Draw | Low (visual pattern) |
| Xiangqi | General in check/no moves | Checkmate OR stalemate = win | Perpetual check/chase = loss | **Win for aggressor** | Medium (flying general complicates) |
| Shogi | King in check, no legal move | Hisshi (brinkmate) vs. tsumero | 4-fold = draw; perp. check = loss | N/A (drops prevent stalemate) | High (drop space is vast) |
| Arimaa | Rabbit reaches goal row | Goal threat clarity | 3-repeat available moves = loss | N/A | Medium (step-counting) |
| Hive | All 6 queen hex-neighbors filled | Visual completeness | Draw by agreement | N/A | Very Low (count 6 hexes) |
| Tak | Road connects two opposite edges | Path-tracing | None formal | N/A | Low (visual path) |
| 7W Duel | Military/science supremacy OR points | Threshold metrics | N/A | N/A | Very Low (track numbers) |
| Adv. Wars | HQ capture OR annihilation | Unit presence/capture calculation | N/A | N/A | Low |

---

## References

1. [Laws of Chess: For competitions starting before 1 July 2014](https://web.archive.org/web/20180627230714/http:/www.fide.com/component/handbook/?id=124&view=article) - FIDE - World Chess Federation

2. [FIDE Handbook - The Waypoint Foundation](http://waypointfoundation.org/activities/chesstournaments/FIDELawsOfChess.pdf)

3. [Checkmate - Wikipedia](https://en.wikipedia.org/wiki/Checkmate)

4. [Checkmate & Checkmate Patterns - Chess Terms](https://www.chess.com/terms/checkmate-chess) - Learn everything about the checkmate, the most important goal of chess, and 20 different checkmate p...

5. [Stalemate - Wikipedia](https://en.wikipedia.org/wiki/Stalemate)

6. [Checkmate - Wikipedia](https://en.wikipedia.org/wiki/Checkmates)

7. [Why Is Stalemate A Draw In Chess?](https://www.chess.com/blog/SamCopeland/stalemate-should-totes-be-a-win) - The real problem with stalemate being a draw is that it is bad game design. The most common complain...

8. [Rules of chess - Wikipedia](https://en.wikipedia.org/wiki/Rules_of_chess)

9. [Checkmate is not the same as a forced capture of the enemy king in ...](https://jdh.hamkins.org/checkmate-is-not-the-same-as-a-forced-capture-of-the-enemy-king-in-simplified-chess/) - Checkmate is what occurs when you have a necessary possibility of capturing the enemy king on the ne...

10. [The Evolution of Chess Rules Through the Centuries](https://www.chess.com/blog/chess_coaching70020/the-evolution-of-chess-rules-through-the-centuries) - Chess, one of the most iconic games in history, has undergone significant transformations in its rul...

11. [Checkmates – Overview](http://xiangqibowl.org/?page_id=327)

12. [Stalemate](https://xiangqihub.com/stalemate/)

13. [XiangQi - la diagonale du fou](http://abstractstrategygames.blogspot.com/2011/02/xiangqi.html) - <<< règles du jeu XiangQi , known in the west as Chinese Chess, is an extremely popular game in the ...

14. [Xiangqi (Chinese Chess) Checkmate Strategies](https://www.xiangqi.com/articles/checkmate-strategies) - Learn the top 25 must-know checkmate tactics and patterns in Xiangqi (Chinese Chess) to start winnin...

15. [Preparing the Game](https://shogi.cz/wp-content/uploads/manual_EN.pdf)

16. [Lesson Eight – illegal moves - Shogi PL](http://shogi.pl/level-1-shogi-course/lesson-eight-illegal-moves/) - Rather obvious when you think of it. You cannot drop pieces that move only forward onto last squares...

17. [What's the reason for the "five illegal moves"?](https://www.reddit.com/r/shogi/comments/1oii9r1/whats_the_reason_for_the_five_illegal_moves/) - What's the reason for the "five illegal moves"?

18. [How to play Shogi(将棋) -Lesson#9- Illegal Pawn drop](https://www.youtube.com/watch?v=xuHVMhhc4SY) - I'll be talking about illegal pawn drops, Which are "mate with a dropped pawn" and "two pawns in a f...

19. [Brinkmate - Wikipedia](https://en.wikipedia.org/wiki/Brinkmate)

20. [Brinkmate Contents Shogi Western chess See also References Bibliography External links Navigation...](https://sfdwhf.blogspot.com/2019/04/brinkmate-contents-shogi-western-chess.html) - Shogi theoryChess theory shogicheckmatewestern chess§Western chesstactic In shogi, brinkmate or hiss...

21. [Rules - FESA - Federation of European Shogi Associations](https://fesashogi.eu/rules/) - 9.4 After having made an illegal move (as variously defined in Articles 1, 3 and 9) then the opponen...

22. [Xiangqi (Chinese Chess)](http://hgm.nubati.net/rules/Xiangqi.html)

23. [Xiangqi Move Limits](https://www.xiangqi.com/help/limits) - See how xiangqi.com avoids perpetual check and chasing on our Chinese chess website.

24. [CXQ Chinese Chess Rules](http://www.clubxiangqi.com/rules/)

25. [AXF - Rules of XiangQi PDF](https://www.scribd.com/document/351139251/AXF-Rules-of-XiangQi-pdf) - Scribd is the world's largest social reading and publishing site.

26. [ARIMAA](http://arimaa.com/arimaa/rules/Zman_Arimaa_Rules.pdf)

27. [[PDF] Arimaa Game Rules -. [ Tim Wylie ] .](https://academic.timwylie.com/17CSCI4341/Arimaalab.pdf)

28. [Designing a Winning Arimaa Program](http://icosahedral.net/downloads/djwu2015arimaa_color.pdf)

29. [Arimaa Rules](http://arimaa.com/arimaa/rules/FootballTheme.pdf)

30. [[PDF] Hive Rulebook](https://cdn.1j1ju.com/medias/96/cb/a5-hive-rulebook.pdf)

31. [Gamehelphive - Board Game Arena](https://en.doc.boardgamearena.com/Gamehelphive)

32. [• Board Game Arena](https://nl.boardgamearena.com/doc/Gamehelphive)

33. [Hive](https://www.rulespal.com/hive/rulebook) - Board game rules answered by your AI pal.

34. [[PDF] Rules for TAK - Carbondale](https://carbondale.network/std/pdfs/tak_beta.pdf)

35. [[PDF] Object of the Game The Board - racingcow.io](https://www.racingcow.io/pdf/games/tak_optimized.pdf)

36. [Concise Tak Rules](https://gist.github.com/mindplay-dk/21cf8d3bd251f3034610534f8f792737) - Concise Tak Rules. GitHub Gist: instantly share code, notes, and snippets.

37. [Companion Book](https://static1.squarespace.com/static/5e1ce8815cb76d3000d347f2/t/6482447fb0ca284557e6fc04/1686258824488/TakCompanionBookPDF.pdf)

38. [How To Play The Beautiful Game of Tak - USTak.org](https://ustak.org/play-beautiful-game-tak/) - Tak is a new abstract strategy game created by James Ernest of Cheapass Games and released into open...

39. [Threefold repetition - Wikipedia](https://en.wikipedia.org/wiki/Threefold_repetition) - In chess, the threefold repetition rule states that a player may claim a draw if the same position o...

40. [Threefold Repetition - Chess Terms](https://www.chess.com/terms/threefold-repetition-chess) - The threefold repetition rule states that if a game reaches the same position three times, a draw ca...

41. [Sennichite - Wikipedia](https://en.wikipedia.org/wiki/Sennichite)

42. [Shogi - Wikipedia](https://en.wikipedia.org/wiki/Shogi)

43. [Sennichite (draw or lose?)](https://www.reddit.com/r/shogi/comments/10f64o4/sennichite_draw_or_lose/)

44. [Game rules | Shogi - japanese chess game](https://shogi.cz/en/game-rules/)

45. [Rules of Go - Wikipedia](https://en.wikipedia.org/wiki/Rules_of_Go)

46. [The Stalemate Controversy - Chess.com](https://www.chess.com/blog/ijgeoffrey/the-stalemate-controversy) - Those who argue for stalemate being a win, believe that the stalemate-draw rule is illogical. The ab...

47. [7 Wonders Duel: A Marvel of Modern Game Design](https://chickenfrydgames.wordpress.com/2017/04/25/7-wonders-duel-a-marvel-of-modern-game-design/) - Military victory= When one player marches all the way to the end of the military scoring track. Scie...

48. [7 Wonders Duel: the essential two-player game! - Repos Production](https://www.rprod.com/en/games/7-wonders-duel) - If you gather 6 different scientific symbols, you immediately win the game via scientific supremacy....

49. [7 Wonders Duel Win Conditions : r/boardgames - Reddit](https://www.reddit.com/r/boardgames/comments/1hh76ks/7_wonders_duel_win_conditions/) - You CAN win by military/science victory but it requires a lot of luck to be able to snag the end-gam...

50. [#47 – 7 Wonders: Duel – What's Eric Playing?](https://whatsericplaying.com/2016/07/24/7-wonders-duel/) - Usually, your opponent will win by a military or science victory only if you make a pretty egregious...

51. [HQ (Advance Wars)](https://warswiki.org/wiki/HQ_(Advance_Wars)) - An HQ is the most important property on the map; if the enemy's HQ is captured, they are defeated an...

52. [Better to cap HQ or defeat all enemies to win?](https://www.reddit.com/r/Advance_Wars/comments/u8cf05/better_to_cap_hq_or_defeat_all_enemies_to_win/)

53. [Advance Wars: Days of Ruin/Gameplay](https://strategywiki.org/wiki/Advance_Wars:_Days_of_Ruin/Gameplay) - Advance Wars: Days of Ruin and all other Advance Wars games are Turn-based strategy (TBS) where play...

54. [Headquarters](https://awbw.fandom.com/wiki/Headquarters) - The Headquarters is the most strategically important property on any map, as in addition to providin...

55. [Capping Buildings in Santorini (Mechanism Spotlight) - YouTube](https://www.youtube.com/watch?v=e84EvfZ4mEw) - ... santorini How does the ease of blocking in Santorini shape how players approach strategy in the ...

56. [82 – Santorini - What's Eric Playing?](https://whatsericplaying.com/2017/01/01/santorini/) - Some give you new win conditions (jump down two levels, build five complete towers, have your Worker...

57. [Santorini Game Rules – Complete How to Play, God Powers & Hero ...](https://officialgamerules.org/game-rules/santorini-rules/) - You win immediately if one of your Workers moves onto Level 3. Some God Powers add alternative win c...

58. [If you're tabled, do you automatically lose? - Forum - DakkaDakka](https://www.dakkadakka.com/dakkaforum/posts/list/439204.page) - I think if you table your enemy you should win with full points. You utterly crushed the enemy, ther...

59. [Playing the objective vs blowing your opponent up - Reddit](https://www.reddit.com/r/WarhammerCompetitive/comments/r96cg5/playing_the_objective_vs_blowing_your_opponent_up/) - I feel the game is far too killy for objectives to be meaningful over just annihilating your opponen...

60. [TIL the last time a checkmate actually occurred on the board during ...](https://www.reddit.com/r/todayilearned/comments/1rhkbc3/til_the_last_time_a_checkmate_actually_occurred/) - Yeah it's super rude to make your opponent actually checkmate you once the game is over, save both o...

61. [Introduction to Tsume Shogi (詰将棋) — Shogi Checkmate Puzzles for Beginners](https://japanesechess.org/tsume/) - Tsume shogi mating puzzles. Practice winning shogi games through tsume. Study of shogi is known to i...

62. [Tsume shogi - Wikipedia](https://en.wikipedia.org/wiki/Tsume_shogi)

63. [[PDF] JAIST Repository](https://dspace.jaist.ac.jp/dspace/bitstream/10119/7830/1/A6484.pdf)

64. [539](https://www.jstage.jst.go.jp/article/tjsai/16/6/16_6_539/_pdf)

65. [Arimaa challenge - static evaluation function](https://arimaa.com/arimaa/papers/ThomasHrebejk/Arimaa.pdf)

