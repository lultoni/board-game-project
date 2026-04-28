# Design Ideas from Old Game Versions

*Extracted from all old-game-versions material + old GitHub repos. Quantity first — no filtering yet. Every idea recorded, however raw. Review pass happens after this document exists.*

*Sources: UPLOAD - Realm of Elements (rules, revised rules, German version), Outdated ROE (rules, gameplay changes, general changes, notizen, things to worry about), first-boardgame-oriented-concept (rulebook, skills, systems to test), realm-of-elements Java repo, project-roe Java repo, prototype-turn-tracker web app.*

---

## 1. Piece Design & Roster

### 1a. Elemental Identity for Pieces
- Each piece type has a **preferred terrain** it excels on. Movement range and skill range both increase on good terrain.
- Counterpart: bad terrain reduces range. Neutral terrain = standard values.
- Applied per-piece, not per-player — creates natural positioning incentives.

### 1b. Rock-Paper-Scissors Element Advantage
- Air beats Earth, Fire beats Air, Earth beats Water, Water beats Fire. Spirit is neutral.
- In the original version, a mage attacking their element's counter **killed instantly** even through guard protection.
- Could translate to: Champions of a matching "archetype" deal bonus damage to another archetype.

### 1c. Named Champion Classes (not generic Champions)
- 6 classes: **Offense, Defense, Mobility, Boost, Tile Control** + unnamed 6th.
- Each class has 3 distinct Champions within it — same role, different flavour/stats.
- Pick-and-ban draft: you chose which Champions to run, not just which skills.
- Opens up counter-drafting and draft identity.

### 1d. King has 3 Skill Slots (not 2)
- In the ROE rulebook, the King had 3 Joker Skills (from any class) and a standard range of 3.
- The current game gives King 2 slots. The original gave him more, making him a unique power piece.

### 1e. Spirit Mage as Win Condition Piece
- Not "capture the King" — "kill the Spirit Mage." Named piece with explicit fragility identity.
- The Spirit Mage had no terrain advantage and a specific skill set (token theft, piece swapping).
- Framing: the win condition piece has an identity beyond just "the important one."

### 1f. Champions tied to terrain types
- Each Champion class had a terrain they were *good on*. This created natural board positioning archetypes: Fire Champion wants to stand on Plains; Water Champion wants Lakes.
- Guards were neutral on all terrain.
- Could create differentiated Champion value based on board layout.

---

## 2. Board & Terrain

### 2a. Terrain bonuses to movement AND skill range
- Standing on favourable terrain: +1 move range AND +1 skill range.
- Standing on unfavourable terrain: −1 skill range (movement unchanged, minimum 1).
- Four terrain types: Plains, Forest, Mountain, Lake.

### 2b. Terrain confers Armor on entry
- **Forest**: +1 temporary Armor when a piece enters a forest tile (lost on exit).
- Simple, tactically interesting — positioning matters for defense.

### 2c. Terrain affects movement speed directly
- **Plains**: +1 Speed while on plains tile.
- Mountain: Immunity to push/pull effects.
- Water/Lake: +1 Skill Range (from revised rulebook).

### 2d. Mirror board generation via FEN
- Map built from a 4-character "seed" (e.g. "FPPM") and mirrored on both axes → symmetric 8×8.
- Guarantees fairness. Any string of 4 terrain types generates a legal, balanced board.
- Low setup cognitive load: agree on a seed or roll/pick one, then mirror.

### 2e. 8×8 board (not 10×10)
- The original game used 8×8. Smaller board = faster first contact, less empty space.
- Current game uses 10×10. Could be worth revisiting with the smaller piece count variant.

### 2f. Tile effects applied by skills
- **Death zone** (Inferno spell): pieces that move through or end turn on this tile take 1 DMG. Lasts 2 turns.
- **Blocked tile** (Rock Slide): tile becomes impassable and cannot be targeted by skills. Lasts 2 turns.
- **Stun zone** (Tremor/Overgrown): pieces on affected tiles skip their next turn.
- **Overgrown effect**: piece cannot move, attack, OR use skills (total freeze).
- All effects have a turn timer — tracked per tile, not per piece.

### 2g. Contested centre tile / zone control
- Performance-based Rune idea: occupying the centre tile for a full round = +1 Rune gain.
- Not adopted (closes off strategies), but the idea of rewarding centre control is interesting as a *skill* or *map variant*.

---

## 3. Skills / Spells

### 3a. Element-Themed Utility Skills
Full list from the original ROE spell tables:

**Fire:**
- *Fireball*: Kill target, apply 1-turn Inferno (death zone on that tile, 3×1 area variant).
- *Blazing Barrier*: Immune to attacks (not spells) for 1 turn, also protects adjacent guard.
- *Inferno*: Create 3×1 death zone for 2 turns on empty tiles.

**Water:**
- *Icy Spear*: Kill target.
- *Aqua Shield*: Immune to spells for 1 turn, protects adjacent guard.
- *Tidal Surge / Tidal Wave*: Push all pieces within range 2 tiles back. (Area push, not single-target.)

**Earth:**
- *Rock Slide*: Kill target + block that tile for 2 turns.
- *Stone Wall*: Immune to spells for 1 turn.
- *Tremor*: 2×2 area — all pieces in zone skip their next turn (Stun).

**Air:**
- *Gale Force*: Kill target + push all surrounding pieces 1 tile back.
- *Aerial Shield*: Reflect any spell cast on this piece back at the caster for 1 turn. (Mirror mechanic.)
- *Zephyr Step*: Teleport to an adjacent tile.

**Spirit:**
- *Soul Siphon*: Kill target + steal 1 Rune/token from opponent (if they have one).
- *Ethereal Shield*: Immune to spells for 1 turn.
- *Soul Swap*: Swap two of your own pieces.

### 3b. Skills from the Extended Catalogue (German + English)
Full list of skills that existed in the catalogue but aren't in the current game:

**Offensive / Strike-adjacent:**
- *Ambush*: Attack in Range 2, move 1 tile after. (Ranged attack + repositioning on one activation.)
- *Salvenstich / Salvo Thrust*: Multiple hits? (variants explored)
- *Durchbohren / Pierce*: Attack that ignores armor.
- *Schnellstich / Quick Thrust*: Cheap, short-range attack.
- *Charge*: Boost offense, reset the boost when actually using the offense. (Set-up combo mechanic.)
- *Irrlicht / Will-o'-Wisp*: Move a piece toward it (pull variant with lure mechanic).

**Defensive:**
- *Weak Shield*: 2 turns, absorb 1 DMG.
- *Strong Shield*: 2 turns, absorb 2 DMG.
- *Mirror Shield*: 2 turns, apply DMG to both attacker and target. (Punishes aggression.)
- *Standhaft / Steadfast*: Some form of stance/brace — details unclear.
- *Rüstungsschmied / Armorsmith* (already exists): Add Armor.
- *Schrottpanzer / Rust Shield* (already exists): Modify enemy armor.

**Mobility:**
- *Maelstrom*: Pull all adjacent enemy pieces 1 tile toward caster. (Mass pull.)
- *Pferdesprung / Horse Leap*: Knight-move style repositioning.
- *Wasserlauf / Water Run*: Movement skill related to water terrain.
- *Tauschschritt / Swap Step* (Shadow Shift variant): Swap with piece on path.
- *Windstoß / Air Blast* (already exists): Push target.
- *Hakenziehen / Hook Pull* (already exists): Pull target.

**Economy / Resource:**
- *Runenraub / Rune Theft* (already exists): Deal damage + steal Rune.
- *Opfergabe / Sacrifice*: Sacrifice a piece for +1 permanent Rune gain rate.
- *War Cry*: All skills next round get +1 Range. (Lasts a full round — strong.)
- *Bardic Inspiration*: Next skill used by any of your pieces gets +1 Range. (Focus Strike variant, range flavour.)
- *Raise Undead*: 10 Runes — bring a captured piece back as your own, adjacent to caster. (Resurrection.)
- *Pocket Thief*: 5 Runes — steal 1 Armor from target (if they have >0).

**Utility / Other:**
- *Heat Reduction*: Remove 1 Armor from target.
- *Dodge*: (unclear mechanic — evasion variant?)

### 3c. Shield System as Distinct Mechanic
- Shields are different from Armor:
  - Armor is permanent until destroyed (absorbs 1 DMG per point).
  - Shields have a duration (2 turns) and then expire.
  - Stacking shields extends duration, not DMG absorption.
  - Shields block skills from passing through the protected piece (Armor does not).
  - Defense skills cost only ½ a Skill Slot in the action phase. (You could use one twice in the space of one full slot.)
- The weak/strong/mirror variants create meaningful choice within the shield category.

### 3d. Terrain-Tile-Affecting Skills
- Entire skill category (Tile Control): pieces that modify board tiles, not enemy pieces.
  - Apply damage effects to tiles.
  - Block tiles entirely.
  - (Planned but incomplete): Stone Prison, Lookout Tower as structures.
- Guards are the natural "tile control" pieces if this were adapted to the current game.

### 3e. Area-of-Effect Skills
- Several skills hit a 2×2 or 3×1 area rather than a single tile.
- Current game has no AoE. Blade Tempest is adjacent-pieces-only, not a true path AoE.
- *Tremor* (2×2 stun area), *Inferno* (3×1 death zone), *Tidal Surge* (all-pieces-in-range push) are all AoE.

### 3f. Persistent Tile Effects
- Some skills create multi-turn effects on tiles, not pieces:
  - "Inferno" death zone lasts 2 turns.
  - "Rock Slide" blocked zone lasts 2 turns.
  - Pieces that enter an affected tile trigger the effect even on later turns.
- The tracker would need to record these (the digital games had tile-effect timers).

### 3g. Skill Range Modified by Terrain
- When a skill-user stands on their favourable terrain, range increases by 1.
- When on unfavourable terrain, range decreases by 1.
- Minimum range: 1 (or current range is 0 if Injured — still can self-target).

### 3h. Spell Reflection (Mirror Shield / Aerial Shield)
- Any spell cast at a piece with the Aerial Shield active is redirected at the attacker.
- Creates a read-your-opponent mechanic: if they just used Aerial Shield, using a skill against them hurts you.

### 3i. Capture-Based Combo (Soul Siphon)
- Kill target AND steal a resource from opponent.
- Creates a compounding advantage when you execute a kill: not just board presence, but economy.

---

## 4. Combat & Attack System

### 4a. Champions take self-damage on standard attacks
- In the ROE Java repo, if a Champion attacks they take 1 DMG themselves (Guards attack for free).
- Creates meaningful asymmetry: Guards are "free" attackers, Champions are "risky" attackers.
- Makes Guards more valuable as frontline fighters, Champions as precious assets.

### 4b. Mutual kill on Guard-vs-Champion melee
- If a Champion moves onto a Guard, both are removed.
- If a Guard moves onto a Champion, only the Champion is removed (Guard survives).
- Asymmetric melee outcomes based on piece type.

### 4c. Attack destroys both if equal types meet
- Mage attacks Mage → both die (in early version). Refined in later versions to element advantage determining survival.
- Conceptually: same-type pieces neutralise each other.

### 4d. Attack denial area
- After Fireball kills a target, that tile becomes a 1-turn death zone. The kill also controls space.
- Rock Slide: kill + block tile for 2 turns. Kills create positional consequences.

### 4e. Retaliation mechanic (variant from Gameplay loop changes doc)
- Idea explored: overextending a piece gives the opponent a retaliation action.
- Standoff between pieces without immediate taking — retaliation for overextension.
- "Piece exchange synergies — possible downsides to taking a piece."
- Explicitly explored as a way to make space control feel like chess positioning.

---

## 5. Economy & Progression

### 5a. Rune scaling every 7 rounds (original version)
- Start: 5 Runes. Gain: +1/turn. Increase by +1 every 7 rounds.
- So R1–6: +1/turn, R7–13: +2/turn, R14–20: +3/turn, etc.
- Current game: +2/turn from R2, +1 every 5 rounds. The 7-round cycle is an alternative cadence.

### 5b. Performance-based Rune gain (all variants)
- *Capture bonus*: +2 Runes for taking an opponent piece.
- *Centre control*: Occupy centre tile for 1 full round → +1 Rune gain rate.
- *King advancement*: Your King moves 2 rows past starting position → +1 Rune gain.
- *Piece losses*: Every 2 pieces you lose → +1 Rune gain (comeback mechanic).
- *Max cap*: 5 Runes per turn maximum gain.

### 5c. Rune cap (max 8 or other)
- Original considered a max of 8 Runes at any time.
- Prevents Rune hoarding and explosive turns.

### 5d. Skill slot scaling every 10 rounds
- Start: 2. R10+: 3. R20+: 4. R30+: 5.
- Current game uses same structure. Original had this too — validated as a good feel.

### 5e. Sacrifice skill for permanent economy gain
- *Sacrifice*: Remove one of your own pieces → gain +1 Rune permanently each turn.
- Creates a late-game resource trade mechanic: sacrifice a weak piece for economic snowball.

### 5f. Piece kill as economy gain
- *Soul Siphon*: Kill target + steal 1 Rune from opponent.
- Makes aggressive play doubly rewarding — board presence + economy swing.

### 5g. Turtle bonus (design note, not implemented)
- `Project-ROE Notizen`: "Do you want to add a turtle bonus?" — if a player chooses not to spend their skill slots, do they get a benefit?
- Never implemented, but the question is interesting: reward for restraint vs. reward for aggression.

---

## 6. Turn Structure

### 6a. Move 3 pieces per turn (not 2)
- Original ROE rules and the Java implementations gave players **3 move slots** per turn.
- Current game uses 2 Move Slots. The original used 3.
- More moves = more army coordination possible, but also more cognitive load per turn.

### 6b. Linked movement (only moved pieces act)
- Variant explored: pieces can only use skills if they moved that turn.
- Creates a "move to engage" requirement — no sniping from safety.
- Withdrawn in favour of unlinked. But the design tension is real.

### 6c. Attack phase separate from skill phase
- Original turn: Movement Phase → Attack Phase (standard attacks) → then back.
- No separate "Action Phase" for skills — spells were cast during the Attack Phase.
- Current game has Movement → Action (where both attacks and skills happen). Different split.

### 6d. Turn counter as global timer (0.5 increments)
- Game starts at turn 1. After each player's full turn, counter advances by 0.5.
- So a full round (both players) = 1 whole turn increment.
- This made "every 7 turns" mean every 7 *full rounds*, not 14 individual turns.
- Cleaner language: "turn 7" means both players have each moved 7 times.

---

## 7. Drafting & Setup

### 7a. Pick-and-ban Champion draft
- Player 1 bans 1 Champion class (or specific Champion). Player 2 does same.
- Then alternating picks: P1 picks 2, P2 picks 2, P1 picks final, P2 picks final.
- Both players CAN pick the same Champion type (mirror matchups allowed).
- Cannot pick the same Champion twice yourself.
- Adds meta-game layer: "I ban X because it counters my strategy."

### 7b. 18 Champions to draft from (6 classes × 3 each)
- Far larger pool than current 5 Champions per player.
- Each playthrough the battlefield composition is unique.
- Risk: decision fatigue in draft with large pool. Mitigated by class categories.

### 7c. Draft tied to board placement
- After picking a Champion, immediately decide where it goes on the board.
- Draft order and placement happen simultaneously.
- Creates read-your-opponent tension in setup.

### 7d. Simultaneous reveal placement (backpocket idea, already noted)
- Both players commit placements secretly, then reveal simultaneously.
- Prevents infinite adjustment loops.

### 7e. CR-style draft (one for me, one for you)
- Alternative: P1 picks 1, P2 picks 1, alternating singles rather than P1 picks 2 then P2.
- Stricter interleaving — prevents P1 taking two complementary skills in a row.

---

## 8. Winning, Losing, Draw Conditions

### 8a. Draw if no captures in 10 rounds
- Both the original and current game have this. Good anti-stall mechanic.
- Original: "No piece has been taken in 10 turns."
- Prototype turn tracker tracked this explicitly with a toggle per round.

### 8b. Draw if only Kings remain
- If every piece except the two Kings is removed → draw.
- Forces endgame resolution before losing all army.

### 8c. Checkmate-style win condition (from backpocket)
- Game ends when a lethal setup is *inescapable* — not when the King is captured, but when capture is inevitable.
- The game ends on the *setup*, not the execution.
- Reduces endgame drag: winner is declared when they've achieved inescapable threat.

### 8d. Alternative win via Spirit Mage / named victory piece
- Original: killing the Spirit Mage wins. No checkmate needed — clean.
- Current: capturing the King. Same concept.
- But with named victory pieces (each one thematic), the stakes feel more narrative.

---

## 9. AI / Digital Design Ideas

*(These are from the digital implementation thinking. Some may translate to physical game feel.)*

### 9a. Evaluation function factors (what makes a good position)
The minimax AI evaluated positions on 12 factors:
1. Spell token count (economy advantage)
2. Terrain advantage per piece (are your pieces on good terrain?)
3. Mobility bonus (how many legal moves does each piece have?)
4. Guard protection bonus (mages adjacent to guards = +bonus)
5. Enemy adjacency penalty (enemy pieces nearby = dangerous)
6. Spell path blockage (are your pieces sheltered from skill lines?)
7. Good terrain adjacency bonus (being 1–2 tiles from favourable terrain)
8. Guard positioning relative to centre
9. Piece count
10. Overgrown/disabled status penalty
11. Win/loss terminal states
12. Token value scaling down late-game (early economy matters more)

→ These factors describe what "good board states" look like. Could inform how to evaluate playtests and what metrics to track.

### 9b. Turn generation schema
The AI generated turns as combinations of: `(0–3 moves) × (attack or not) × (spell or not)`. This is the full space of legal player turns. Useful for understanding the branching factor — it's huge, which is why the AI couldn't go deeper than depth 1.

### 9c. FEN-like board notation
- Board state encoded as a compact string: piece type + status flags + position + resources.
- Useful for logging games, comparing positions, or building a game journal.
- Could enable replay notation for playtests.

---

## 10. Narrative / Lore

### 10a. The World — Primordials and Rabbit
- A god called "Rabbit" was bored with the void, convinced other primordials to create autonomous beings.
- Early space wars were destructive; a game was designed as the resolution mechanism.
- The game's in-universe name is (INSERT GAME NAME) — deliberately placeholder even in the lore.
- The lore gives the game a mythological origin story without locking it to a specific name.

### 10b. Tutorial through a character (the Advisor)
- The intro to the lore was told through an advisor NPC — "you lost your memory" framing to explain rules.
- Friendly, slightly sardonic voice. References gods (Gautaz, Teiwaz) as in-universe expletives.
- This could be adapted for a rulebook intro or flavour text.

### 10c. Named gods / swear words
- Gautaz, Teiwaz — in-universe deity names used as expletives in the lore.
- Good flavour anchors if the game ever gets a named universe.

---

## 11. Meta / Design Process Ideas

### 11a. Design methodology note (from Gameplay Loop Changes doc)
Direct quote worth keeping:
> "WHAT IS THE ROOT ISSUE WHEN I SEE SOMETHING I DON'T LIKE TO BE PART OF THE GAME EXPERIENCE. TAKE A STEP BACK AND THINK ABOUT IT. TRY A FEW IDEAS. MAKE BIG CHANGES — double it or cut it in half. No 5% changes here, 6% changes there. Be prepared to kill your darlings."

This predates the current incremental testing methodology and captures the same spirit.

### 11b. "Complexity should arise from strategy, not from confusing mechanics."
Direct quote from Gameplay Loop Changes. Already our design north star.

### 11c. Piece compatibility / synergy system
- Idea explored: pieces boost each other when adjacent based on "piece compatibility."
- "How do pieces work together to get to a goal (hearthstone card synergies)?"
- Not implemented, but a rich design space: adjacent ally bonuses, class combos.

### 11d. Game feel design questions (from ROE notes)
Questions explicitly asked during ROE design — still relevant:
- "What are the player choices per turn? Don't overwhelm them."
- "If you want to make the game about space control like in chess, how can you achieve that?"
- "How can I make the players do the things I want them to do?"
- "What should the players do?" → "Tough strategic decisions — which piece to sacrifice."
- "What is the gameplay experience you want the player to have?" → "Make it feel fair and natural."
- "Weakening pieces should feel fair and logical for the game setting, not forced from a higher entity."

### 11e. 4 playtests documented in Notizen (V1.5.1–V1.5.2)
- **Game 1 (V1.5.1)**: R7, draw. 
- **Game 2 (V1.5.1)**: R4, Air Mage trick missed (opponent oversight).
- **Game 3 (V1.5.1)**: R8, siren suck-in + vampire boost kill (opponent win).
- **Game 4 (V1.5.2)**: R10, overall overpowerment + misplay.
- Notable: these games were ending at R4–R10. Much shorter than current game. Either smaller board, fewer pieces, or faster kill mechanics made it faster.

---

## 12. Technical / Tracker Ideas

### 12a. Per-tile effect tracking
- The turn tracker (prototype-turn-tracker) tracked per-player Rune economy and piece counts.
- The digital games tracked per-tile effects with turn timers (death zones, blocked tiles).
- A physical game with tile effects would need a token or marker system on the board.

### 12b. Elo rating system
- The realm-of-elements app tracked player Elo ratings in SQLite.
- K-factor adjusted by games played and rating gap (standard chess K-factor logic).
- Could be a fun thing to run manually for serious playtest sessions.

### 12c. Sound design ideas (from General Changes doc)
- "5 stack of skill runes, like in Orlog" — physical rune tokens in a stack.
- The rune-adding animation idea: new runes "float in" and join the stack.
- Timer for effect durations displayed as a pie diagram.
- Skill range shown as a white ring on hover.
- Display last-made actions (trail behind a piece).
