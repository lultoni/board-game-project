# ADR-001: Game Architecture Direction

*Date: 2026-04-17*
*Status: PROPOSED — awaiting decision*
*Context: Post-Playtest 1, informed by genre research*

---

## Decision Context

After Playtest 1 and identifying the core fantasy ("discovering and executing clever spell/skill combos"), we need to decide the fundamental architecture of the game. The current design is a grid-based tactical game where spells are layered on top of chess-like movement — but the playtest and designer reflection suggest the grid may be generating overhead complexity that competes with the spell-combo experience rather than enhancing it.

### The Core Tension

The research on cognitive load (Sweller, Koster) establishes that players can hold **3-5 simultaneous variables** before decision paralysis. In the current design, a player's decision on any given turn involves:

1. **Where to move** piece A (spatial)
2. **Where to move** piece B (spatial)
3. **Which skill(s) to use** (spell selection)
4. **On which target** (targeting / range / line-of-sight)
5. **Rune budget** (resource management)
6. ~~**Terrain effects**~~ (removed — confirmed overhead)
7. **Bodyguard positioning** (Guard adjacency — never triggered)
8. **Opponent's likely response** (prediction)

That's 6-8 axes. The research says 3-5 is the sweet spot. The spells (#3-4) are the core fantasy — everything else should either *serve* the spells or be cut.

---

## Options

### Direction A: Streamlined Tactical Grid ("Onitama / War Chest model")

**Keep the board. Strip it to the bone. Make spells the star.**

**Reference games**: Onitama (5x5, rotating movement cards), War Chest (hex grid, dual-use coins), Neuroshima Hex (hex, tile placement + simultaneous battle)

**What changes from current design**:
- **Smaller board**: 6x6 or 8x8. Pieces start in contact range within 2-3 rounds.
- **No terrain**: Board is uniform. Spatial decisions = pure positioning.
- **Fewer pieces**: Maybe 1 King + 3 Champions + 3 Guards = 7 per side (14 total on a 36-64 tile board).
- **Spells define movement**: Instead of base movement + separate spells, movement IS a spell action. Like Onitama — your "move cards" rotate.
- **Kill the dead opening**: Pieces start close enough that spells are relevant from Turn 1.

**Cognitive axes**: Position (1), Spell choice (2), Target (3), Resources (4), Prediction (5). That's 5 — right at the ceiling.

**Serves the core fantasy because**: The board becomes a *targeting constraint* for spells, not a game unto itself. "Can I reach them with this spell?" replaces "where do I move for 6 rounds before anything happens?"

**Risks**:
- May still inherit "positioning overhead" if the board is too big
- War Chest and Onitama succeed partly because they have very FEW abilities — your skill catalogue is much larger
- The "gearing up" arc is hard to create on a grid (pre-game draft exists but no in-game build progression)

**Best feature from research**: War Chest's **dual-use coins** (every piece can be spent as a move OR deployed as a unit). This mirrors your Rune economy elegantly — what if skills could be "spent" in multiple ways?

---

### Direction B: Card Fighter ("BattleCon / Flesh and Blood model")

**Drop the board entirely. The game IS the spells.**

**Reference games**: BattleCon (style+base card combos, 1D range track), Flesh and Blood (combat chains, cards as attacks/blocks/currency), Yomi (simultaneous reveal, reads)

**What changes from current design**:
- **No board, no grid, no movement phase**. Spatial positioning replaced by timing, sequencing, and resource management.
- **Champions become "characters" or "loadouts"**: Each has a unique set of skills. You build your team pre-game, then play their skill cards.
- **Combat is card-driven**: Attacker plays skills, defender responds with blocks/counters. BattleCon's simultaneous reveal or FaB's combat chain.
- **Guards become defensive resources**: Instead of pieces on a board, they're cards you can play to absorb damage (like FaB's block-from-hand).
- **"Range" becomes abstract**: Maybe a simple front-line / back-line system, or no range at all.

**Cognitive axes**: Skill combo selection (1), Timing/sequencing (2), Resource budget (3), Opponent read (4). That's 4 — very clean.

**Serves the core fantasy because**: 100% of decisions are about spells. No spatial overhead at all. The entire game is "which combo do I play, and when?"

**Risks**:
- **Loses the Air Blast moment** — pushing a piece into a bad position was Elias's favourite moment. Spatial disruption spells have no home.
- **Loses visual/tactile appeal** of moving pieces on a board. Physical presence matters for a board game.
- **BattleCon's lesson**: Pure card fighters can become extremely math-heavy ("my speed is 5, range is 2-4, power is 3, you have armor 1..."). Could trade spatial overhead for *calculation* overhead.
- The "gearing up" experience is excellent here (FaB, Ashes Reborn nail it), but may feel like "just another card game."

**Best feature from research**: Flesh and Blood's **multi-use cards** (every card is an attack, a block, and a resource). This creates impossible decisions every hand — exactly the "agonising choices" you want.

---

### Direction C: Minimal Spatial Hybrid ("Summoner Wars / Ashes Reborn model")

**Keep a small spatial element. Spells are primary, board is secondary.**

**Reference games**: Summoner Wars (small grid, cards are units AND currency), Ashes Reborn (unit placement without a grid, dice as resources), Unmatched (point-to-point zones, character decks)

**What changes from current design**:
- **Minimal spatial system**: Not a full grid. Options:
  - **Lane system** (3 lanes, pieces in front/back of each) — like a simplified MOBA
  - **Zone system** (3-5 connected zones, Unmatched-style) — position matters but isn't a puzzle
  - **Formation system** (front row / back row per player) — Keyforge-style battleline
- **Spells are the primary action**: You play skill cards, which may move units between zones/lanes or deal damage.
- **Board exists to give spatial spells a home**: Push, pull, swap — these all work in a lane/zone system but don't require grid-level precision.
- **Progressive disclosure**: Start the game with basic units in lanes. Skills/spells introduce complexity gradually as you draft or draw them.

**Cognitive axes**: Spell combo (1), Zone/lane positioning (2), Resource budget (3), Opponent read (4). That's 4, with #2 being much lighter than a full grid.

**Serves the core fantasy because**: Spells dominate decisions (70-80%) while the spatial component keeps push/pull/swap spells meaningful. The board is a *stage for spell combos*, not a separate system competing for attention.

**Risks**:
- "Just enough board" is a narrow design target. Too much → back to current problems. Too little → might as well drop it (Direction B).
- Summoner Wars works because cards are ALSO the economy (you burn cards to summon). Your Rune system doesn't have this elegance yet.
- Lane/zone systems can feel abstract and unsatisfying if not given a strong thematic wrapper.

**Best feature from research**: Summoner Wars' **cards-as-currency** (discard a card to gain magic, or play it as a unit). Ashes Reborn's **dice pool as resource** (roll dice, each face = a different type of magic you can spend). Both give the resource system physicality and decision weight.

---

## Comparison Matrix

| Criterion | A: Streamlined Grid | B: Card Fighter | C: Spatial Hybrid |
|-----------|:---:|:---:|:---:|
| **Spell combos as primary decision** | Medium | Very High | High |
| **Cognitive load** | Medium (5 axes) | Low (4 axes) | Low-Medium (4 axes) |
| **"Gearing up" arc** | Low (draft only) | High (in-game build) | Medium-High |
| **Push/pull/swap spells viable** | Yes | No | Yes (simplified) |
| **Physical table presence** | High | Low-Medium | Medium |
| **Eliminates "dead opening"** | If board is small enough | Yes, by design | Yes, if starting engaged |
| **Closest to current design** | Most similar | Most different | Middle ground |
| **Design risk** | Medium | Medium-High | Highest (narrow target) |
| **Depth-to-complexity ratio** | Good | Excellent | Excellent if executed well |

---

## My Assessment

**Direction C (Spatial Hybrid)** is the most promising for your specific game, for these reasons:

1. **Preserves what worked**: The Air Blast moment, Shadow Shift's positional trickery, the *feeling* of commanding an army on a field. These are spatial experiences that your playtest showed players enjoyed.

2. **Centres what was best**: Spell combos become 70-80% of the decision space. The zone/lane system provides enough spatial context for push/pull/swap without the cognitive overhead of a 10x10 grid.

3. **Enables the "gearing up" arc**: A hybrid system can incorporate in-game progression (drawing new skills, unlocking power) that a pure grid game struggles with.

4. **Addresses every Playtest 1 problem**:
   - Game too long → fewer spatial decisions = faster turns
   - Rune economy too slow → can redesign economy simultaneously
   - Bodyguard never triggered → Guards become a simpler concept (front row absorbs damage)
   - Injured irrelevant → can redesign health in the new framework
   - Dead opening → no opening if pieces start engaged

5. **The cognitive load math works**: Spell combo (1) + simple positioning (2) + resources (3) + opponent read (4) = 4 axes. That's the sweet spot.

However, **Direction B** is worth considering seriously if you find during prototyping that even a minimal spatial system feels like overhead. The BattleCon "style + base" combo system (where you combine two half-cards to create a unique attack each turn) is *extremely* close to your core fantasy of "discovering cool combos."

**Direction A** is the safest — smallest change from current design — but I think it doesn't solve the fundamental problem: the board is competing with the spells for cognitive bandwidth.

---

## Recommended Next Step

Before committing: **prototype Direction C on paper in 30 minutes.** 

Take 6 index cards per player (3 Champions, 3 Guards-as-front-row). Lay them out in a 3-lane formation (left/center/right). Write 4-5 spells on scraps of paper. Play 5 turns against yourself. See if the spells feel like the star or if the lanes feel like overhead.

If the lanes feel like noise → pivot to Direction B.
If the lanes make push/pull/swap spells shine → Direction C is the one.
