#import "../shared/template.typ": *
#show: template.with(title: "Test Layer 1: Economy Fix")

= Test Layer 1: Economy Fix

_Feedback form (fill out after the game): layer-1-feedback.pdf_

#note-box[
  *Based on the baseline rules.* Only the Resource Economy section is changed. Everything else is identical.
]

*What we're testing:* Does a faster Rune economy fix the dead opening and shorten the game?

*Hypothesis:* With more Runes earlier, skills come online from Turn 1. The first rounds of pure positioning are eliminated. Game should be shorter and more action-dense.

*Watch for:*
- Do players use skills in Rounds 1--3?
- Is the game shorter? (Target: under 25 rounds)
- Do cheap skills (2-cost) feel spammable?
- Is hoarding a problem?
- Do expensive skills (4-cost) feel reachable at a reasonable time?

#hr

== Goal

Capture the opponent's *King*. The game ends immediately when a King is removed from the board.

*Draw conditions:*
- No piece has been captured for 10 consecutive full rounds.
- Only the two Kings remain on the board.

== Components

*Per player:* 1 King · 5 Champions · 6 Guards

*Shared:* 10×10 grid board (no terrain effects) · Skill tokens or cards · Rune tokens · Round tracker

== Setup

+ *Board:* Use the 10×10 grid. Ignore any terrain markings.
+ *First player:* Flip a coin.
+ *Piece placement:* Each player places pieces on their own back two rows:
  - Back row (row 1/10): King in the centre, Champions fill the remaining 5 spaces.
  - Second row (row 2/9): All 6 Guards.
+ *Skill Draft:* Alternating picks. P1 picks 2 skills for one Champion, then P2. Repeat until all Champions and the King have 2 skills each (12 per player). Duplicates allowed.
+ *Starting Runes:* Each player starts with *6 Runes.* _(Changed from baseline 4.)_
+ P1 begins Round 1.

== Round Structure

A *Round* = P1's Turn + P2's Turn.

At the *start of each player's turn* (before they do anything), that player collects their Rune income. _(Exception: Round 1 — players begin with their starting Runes and collect nothing before their first turn.)_

== Turn Structure

Each *Turn* has two phases, in order:

+ *Movement Phase* — spend Move Slots to move pieces (and attack).
+ *Action Phase* — spend Skill Slots to activate skills.

You may use 0 slots in either phase.

== Movement Phase

#block(breakable: false)[
You have *2 Move Slots* per turn. Spend 1 Move Slot to move one piece.

#table(
  columns: (1fr, auto, auto),
  table.header([Piece], [Normal speed], [Injured speed]),
  [Guard], [2 tiles], [1 tile],
  [Champion / King], [1 tile], [1 tile],
)

*Free pathing:* A piece may move in any direction — horizontal, vertical, diagonal — taking any route up to its speed in tiles. The route does not have to be a straight line.

*Pieces block movement:* A moving piece cannot pass through any other piece, ally or opponent. It must stop when its path is blocked.
- _Exception:_ With free pathing, a piece may go around a blocking piece if total tiles moved stays within speed (e.g. a Guard using 2 diagonal moves to go around a piece directly in the way).

You may move the same piece with both Move Slots, or move two different pieces.
]

== Standard Attack

To attack, spend a Move Slot to move your piece *onto a tile occupied by an enemy piece.*

- Deal *2 DMG* to the enemy.
- *If the enemy is removed:* your piece occupies the tile.
- *If the enemy survives* (Armor absorbed the damage): your piece stops on the tile immediately before the target.

You may attack with one Move Slot and move a different piece with the other Move Slot in the same turn.

== Action Phase

You have *2 Skill Slots* per turn (to start — see Progression).

Spend 1 Skill Slot to activate one equipped skill on one of your Champions or King:
+ Announce the skill and the target.
+ Pay the skill's Rune cost.
+ Apply the skill's effect.

The same Champion can activate multiple skills in one turn if you have Skill Slots remaining.

== Skill System

*Skill Path:* Skills travel in a *straight line* (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

*Blocking:* The Skill Path is blocked by *all pieces* — ally and opponent alike. The skill cannot reach past the first piece in its path.

*Range:* The distance in tiles along the Skill Path from caster to target.

All skills cost 1 Skill Slot unless noted otherwise.

== ⚡ CHANGED: Resource Economy

#changed-box[
  #table(
    columns: (1fr, auto, auto),
    table.header([], [Baseline], [Layer 1]),
    [Starting Runes], [4], [*6*],
    [Rune income (from Round 2)], [+1 / turn], [*+2 / turn*],
    [Income scaling], [+1 every 7 rounds], [*+1 every 5 rounds*],
  )
]

*Rune income* is collected at the *start of each player's own turn* (not Round 1).

#table(
  columns: (auto, 1fr),
  table.header([Round], [Income per player turn]),
  [1], [0 (start with 6)],
  [2–5], [+2],
  [6–10], [+3],
  [11+], [+4],
)

*No Rune cap.*

== Health & Armor

*All pieces have 2 HP:* Normal → Injured → Removed.

#table(
  columns: (auto, auto, 1fr),
  table.header([State], [HP], [Effect]),
  [Normal], [2], [No penalty],
  [Injured], [1], [Speed capped at 1. Skill Range −1.],
  [Removed], [0], [Piece leaves the board permanently.],
)

*Damage:*
- 1 DMG to Normal → Injured.
- 1 DMG to Injured → Removed.
- 2 DMG to Normal → Removed instantly (Injured state is skipped).

*Armor:*
- Pieces can hold up to 3 Armor points (granted by skills).
- Each Armor point absorbs *1 DMG instance*, then is destroyed.
- Armor is resolved *before* HP damage.
- Armor does not prevent Injured status.

== Bodyguard Rule

When you make a *Standard Attack* against an opponent's Champion or King, the defender may intercept — *if* a friendly Guard is adjacent to *both the tile immediately before the target (along the attack path) and the defending piece.*

*Interception:*
+ Defender announces a Guard to intercept.
+ The Guard is removed instead of the original target.
+ The attacker *does not move.*

Only Standard Attacks can be intercepted. Skills always hit directly. Interception is optional.

== Skill Drafting

+ Lay out all available skills face-up as a shared pool.
+ Alternating draft: P1 assigns 2 skills to one Champion → P2 assigns 2 skills to one Champion → repeat.
+ Continue until all 5 Champions and the King on each side have 2 skills.
+ Both players draft from the same pool. Duplicates are allowed.

== Skill Reference

#skill-table(
  columns: (auto, 1fr, auto, 2fr),
  table.header([Cat.], [Name], [Cost], [Effect]),
  [Strike], [Lance Thrust], [2], [Target within range takes 1 DMG],
  [Strike], [Hook Pull], [3], [Target takes 1 DMG, pulled 1 tile toward caster along Skill Path],
  [Strike], [Armor Breaker], [2], [Remove 1 Armor from target],
  [Strike], [Rune Theft], [3], [Target takes 1 DMG. Steal 1 Rune from opponent.],
  [Strike], [Blade Tempest], [4], [Target takes 1 DMG. Adjacent pieces pushed 1 tile away from target.],
  [Shield], [Rust Shield], [2], [Self: gain +1 Armor],
  [Shield], [Field Medic], [3], [Heal 1 HP on an adjacent ally (cannot exceed starting HP)],
  [Shield], [Armorsmith], [3], [Adjacent ally gains +1 Armor],
  [Move], [Quick Dash], [3], [Self: move up to 2 tiles along Skill Path],
  [Move], [Air Blast], [2], [Push target enemy 1 tile directly away from caster],
  [Move], [Precision Thrust], [3], [Push target enemy 1 tile in any direction (caster chooses)],
  [Move], [Shadow Shift], [4], [Swap position with an allied piece. *Range 3. Requires unobstructed Skill Path.*],
  [Move], [Retreat Plan], [4], [Self: move along Skill Path to land adjacent to one of your Guards (range = Skill Range + 1)],
  [Mystic], [Focus Strike], [1], [Your next skill this turn gains +1 Range],
  [Mystic], [Blade Call], [2], [Your next skill this turn: pay +1 Rune to deal +1 DMG],
)

== Progression

#block(breakable: false)[
#table(
  columns: (auto, auto),
  table.header([Round], [Skill Slots per turn]),
  [1–9], [2],
  [10–19], [3],
  [20+], [4],
)
]

== Quick Reference

#block(breakable: false)[
#table(
  columns: (1fr, 1.5fr),
  table.header([Concept], [Rule]),
  [Movement], [Free pathing, ≤ speed in tiles, cannot pass through pieces],
  [Attack], [Move onto enemy tile (1 Move Slot). 2 DMG.],
  [Attack — target survives], [Attacker stops on tile before target],
  [Skill Path], [Straight line (Queen). Blocked by all pieces.],
  [Bodyguard], [Guard adjacent to both attacker and defender. Standard Attacks only.],
  [Rune income], [Start of YOUR turn (not Round 1)],
  [Healing], [No cap. Same piece can be healed multiple times per turn.],
  [Armor], [Armor absorbs damage first, then HP],
)
]
