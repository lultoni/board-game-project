// baseline-sections.typ
// Parameterized section functions for all (GAME NAME) rule sheets.
// Import this file alongside template.typ in any rule sheet or layer file.
//
// Usage:
//   #import "../shared/template.typ": *
//   #import "../shared/baseline-sections.typ": *
//
// Each function renders one document section with baseline defaults.
// Pass named arguments only for what changes in your layer.

#import "./template.typ": *

// ── GOAL ──────────────────────────────────────────────────────────────────────

#let section-goal() = [
== Goal

Capture the opponent's *King*. The game ends immediately when a King is removed from the board.

*Draw conditions:*
- No piece has been captured for 10 consecutive full rounds.
- Only the two Kings remain on the board.
]

// ── COMPONENTS ────────────────────────────────────────────────────────────────

#let section-components() = [
== Components

*Per player:* 1 King · 5 Champions · 6 Guards

*Shared:* 10×10 grid board (no terrain effects) · Skill tokens or cards · Rune tokens · Round tracker
]

// ── SETUP ─────────────────────────────────────────────────────────────────────
// start-runes: starting Rune count (default 4, Layer 1 accepted = 6)
// layer1-accepted: if true, adds "(Layer 1 accepted)" label next to Rune count

#let section-setup(start-runes: 4, layer1-accepted: false) = [
== Setup

+ *Board:* Use the 10×10 grid. Ignore any terrain markings.
+ *First player:* Flip a coin.
+ *Piece placement:* Each player places pieces on their own back two rows:
  - Back row (row 1/10): King in the centre, Champions fill the remaining 5 spaces.
  - Second row (row 2/9): All 6 Guards.
+ *Skill Draft:* Alternating picks. P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King, then P2. Repeat until all Champions and the King on each side have 2 skills each (12 per player). Duplicates allowed.
+ *Starting Runes:* Each player starts with *#start-runes Runes*#if layer1-accepted [ (Layer 1 accepted)].
+ P1 begins Round 1.
]

// ── ROUND STRUCTURE ───────────────────────────────────────────────────────────

#let section-round-structure() = [
== Round Structure

A *Round* = P1's Turn + P2's Turn.

At the *start of each player's turn* (before they do anything), that player collects their Rune income. _(Exception: Round 1 — players begin with their starting Runes and collect nothing before their first turn.)_
]

// ── TURN STRUCTURE ────────────────────────────────────────────────────────────

#let section-turn-structure() = [
== Turn Structure

Each *Turn* has two phases, in order:

+ *Movement Phase* — spend Move Slots to move pieces (and attack).
+ *Action Phase* — spend Skill Slots to activate skills.

You may use 0 slots in either phase.
]

// ── MOVEMENT PHASE ────────────────────────────────────────────────────────────

#let section-movement-phase() = [
== Movement Phase

#block(breakable: false)[
You have *2 Move Slots* per turn. Spend 1 Move Slot to move one piece. *Each piece may only be moved once per Movement Phase.*

#table(
  columns: (1fr, auto, auto),
  table.header([Piece], [Normal speed], [Injured speed]),
  [Guard], [2 tiles], [1 tile],
  [Champion / King], [1 tile], [1 tile],
)

*Free pathing:* A piece may move in any direction — horizontal, vertical, diagonal — taking any route up to its speed in tiles. The route does not have to be a straight line.

*Pieces block movement:* A moving piece cannot pass through any other piece, ally or opponent.
- _Exception:_ A piece may go around a blocking piece if total tiles moved stays within speed (e.g. a Guard using 2 diagonal moves to go around a piece directly in the way).
]
]

// ── STANDARD ATTACK ───────────────────────────────────────────────────────────
// damage: DMG dealt (default 2; Layer 2+ uses 1)
// changed: if true, adds ⚡ heading and changed-box comparison table

#let section-standard-attack(damage: 2, changed: false) = {
  let heading = if changed [== ⚡ CHANGED: Standard Attack] else [== Standard Attack]
  let survives-text = if damage == 2 [
    - *If the enemy survives* (Armor absorbed the damage): your piece stops on the tile immediately before the target.
  ] else [
    - *If the enemy survives:* your piece stops on the tile immediately before the target.
  ]
  let damage-note = if changed [ _(baseline: 2 DMG)_] else []
  [
    #heading
    #if changed [
      #changed-box[
        #table(
          columns: (1fr, 1fr, 1fr),
          table.header([], [Baseline], [This Layer]),
          [Standard Attack damage], [*2 DMG*], [*#damage DMG*],
          [Everything else], [—], [Unchanged],
        )
      ]
    ]
    To attack, spend a Move Slot to move your piece *onto a tile occupied by an enemy piece.*

    - Deal *#damage DMG* to the enemy#damage-note.
    - *If the enemy is removed:* your piece occupies the tile.
    #survives-text
    You may attack with one Move Slot and move a different piece with the other Move Slot in the same turn.
  ]
}

// ── MULTI-CHAMPION COMBO BONUS ────────────────────────────────────────────────
// show: if true, renders this section (only appears in L2-G2+)

#let section-combo-bonus(enabled: false) = {
  if enabled [
    == ⚡ NEW: Multi-Champion Combo Bonus

    #changed-box[
      *New rule — not in baseline.* When a second (or third, etc.) Champion's Strike skill hits the *same enemy target* in the *same turn*, each subsequent Strike deals *+1 DMG*.

      The first Strike skill against a target deals normal damage. The second Strike from a *different Champion* deals +1 DMG. A third from yet another Champion would deal +2 DMG (etc.).
    ]

    *Rules:*
    - Only *Strike-category skills* qualify (Lance Thrust, Hook Pull, Armor Breaker, Rune Theft, Blade Tempest).
    - The bonus applies when *different Champions* (or King) hit the *same target* in the same turn.
    - One Champion hitting the same target twice with two Strike skills does *not* trigger the bonus — it must be a different attacking piece.
    - You may use other skills (buffs, heals, movement) in between the Strikes that contribute to the combo. It does not have to be a continuous streak — it just has to be in the same turn.
    - *Buff skills targeting your own pieces* (Focus Strike, Blade Call) do not count as "hitting a target." They enhance damage but don't themselves qualify for the combo bonus.
    - The combo bonus stacks with Blade Call. Each Blade Call activation boosts exactly *one* Strike skill by +1 DMG, then is spent. Multiple Blade Call activations can boost the same or different Strikes.
    - Standard Attacks (Movement Phase) do *not* count toward the combo chain — only Action Phase Strike skills.

    *Example:*
    - Champion A casts Hook Pull on enemy Guard → 1 DMG (normal), pulls Guard 1 tile closer.
    - Champion B casts Lance Thrust on the *same* Guard → 1 DMG + 1 combo bonus = *2 DMG*.
    - That Guard took 3 DMG total this turn (1 + 2). Even with 1 Armor it's Removed (Armor absorbs 1 → 2 DMG to HP → Removed).

    *Example with Blade Call:*
    - Champion A casts Blade Call (boosts the next Strike by +1 DMG — one use only).
    - Champion A casts Lance Thrust on target → 1 + 1 (Blade Call) = 2 DMG. This is the *first* Strike on that target. The Blade Call is now spent.
    - Champion B casts Hook Pull on the *same* target → 1 + 1 (combo bonus) = 2 DMG.
    - Total: 4 DMG this turn (kills a Normal piece with up to 2 Armor).
  ]
}

// ── ACTION PHASE ──────────────────────────────────────────────────────────────

#let section-action-phase() = [
== Action Phase

You have *2 Skill Slots* per turn (to start — see Progression).

Spend 1 Skill Slot to activate one equipped skill on one of your Champions or King:
+ Announce the skill and the target.
+ Pay the skill's Rune cost.
+ Apply the skill's effect.

The same Champion can activate multiple skills in one turn if you have Skill Slots remaining.
]

// ── SKILL SYSTEM ──────────────────────────────────────────────────────────────

#let section-skill-system() = [
== Skill System

*Skill Path:* Skills travel in a *straight line* (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

*Blocking:* The Skill Path is blocked by *all pieces* — ally and opponent alike. The skill cannot reach past the first piece in its path.

*Range:* The distance in tiles along the Skill Path from caster to target. Range is measured as:
- Range 0 = self (caster's own tile)
- Range 1 = adjacent tile along Skill Path
- Range 2 = 2 tiles away along Skill Path (etc.)

*Default Range = 2.* Unless a skill specifies otherwise.

*Injured Range penalty:* Injured pieces have Skill Range −1. This only affects skills with Range 2 or higher — adjacent (Range 1) and self (Range 0) always work regardless of Injured status.

*Skills that move pieces do not deal damage.* Movement-via-skill (Quick Dash, Shadow Shift, Retreat Plan) does not count as a Standard Attack and deals no damage on arrival.

*A Champion may use its skills multiple times in the same turn* if Skill Slots are available — including the same skill twice.

All skills cost 1 Skill Slot unless noted otherwise.
]

// ── RESOURCE ECONOMY ──────────────────────────────────────────────────────────
// start-runes: starting Rune count (default 4)
// layer1-accepted: if true, uses Layer 1 income table (+2/+3/+4/+5) and "(Layer 1 accepted)" label
// changed: if true, adds ⚡ heading and changed-box

#let section-resource-economy(start-runes: 4, layer1-accepted: false, changed: false) = {
  let heading = if changed [== ⚡ CHANGED: Resource Economy (Runes)] else [== Resource Economy (Runes)]
  let label = if layer1-accepted [ (Layer 1 accepted)] else []
  let income-table = if layer1-accepted [
    #table(
      columns: (auto, 1fr),
      table.header([Round], [Income per player turn]),
      [1], [0 (starting Runes only)],
      [2–4], [+2],
      [5–9], [+3],
      [10–14], [+4],
      [15+], [+5 (+1 every 5 rounds)],
    )
  ] else [
    #table(
      columns: (auto, 1fr),
      table.header([Round], [Income per player turn]),
      [1], [0 (starting Runes only)],
      [2–4], [+1],
      [5–9], [+2],
      [10–14], [+3],
      [15+], [+4 (+1 every 5 rounds)],
    )
  ]
  [
    #heading
    #if changed [
      #changed-box[
        #table(
          columns: (1fr, 1fr, 1fr),
          table.header([], [Baseline], [This Layer]),
          [Starting Runes], [*4*], [*#start-runes*],
          [Rune income], [+1/+2/+3/+4], [+2/+3/+4/+5],
        )
      ]
    ]
    *Starting Runes:* #start-runes per player#label.

    Rune income is collected at the *start of each player's own turn:*

    #income-table

    *No Rune cap.*
  ]
}

// ── HEALTH & ARMOR ────────────────────────────────────────────────────────────

#let section-health-armor() = [
== Health & Armor

*All pieces have 2 HP:* Normal → Injured → Removed.

#table(
  columns: (auto, auto, 1fr),
  table.header([State], [HP], [Effect]),
  [Normal], [2], [No penalty],
  [Injured], [1], [Speed capped at 1. Skill Range −1 (affects Range 2+ only).],
  [Removed], [0], [Piece leaves the board permanently.],
)

*Damage:*
- 1 DMG to Normal → Injured.
- 1 DMG to Injured → Removed.
- 2 DMG to Normal → Removed instantly (Injured state is skipped).

*Armor:* Max 3 points per piece. Each absorbs 1 DMG, then destroyed. Resolves before HP damage. Does not prevent Injured status.
]

// ── BODYGUARD RULE ────────────────────────────────────────────────────────────
// adjacency: "both" (default, baseline) or "defender" (Layer 3)
// changed: if true, adds ⚡ heading and changed-box

#let section-bodyguard(adjacency: "both", changed: false) = {
  let heading = if changed [== ⚡ CHANGED: Bodyguard Rule] else [== Bodyguard Rule]
  let condition-text = if adjacency == "defender" [
    *if* a friendly Guard is on a tile *adjacent to the defending piece* (baseline requires adjacent to both attacker AND defender).
  ] else [
    *if* a friendly Guard is on a tile adjacent to *both the tile immediately before the target (along the attack path) and the defending piece.*
  ]
  [
    #heading
    #if changed [
      #changed-box[
        #table(
          columns: (1fr, 1fr, 1fr),
          table.header([], [Baseline], [This Layer]),
          [Guard must be adjacent to], [*Both attacker AND defender*], [*Defender only*],
          [Everything else], [—], [Unchanged],
        )
      ]
    ]
    When you make a *Standard Attack* against an opponent's Champion or King, the defender may choose to have a Guard intercept — #condition-text

    *Interception:*
    + Defender announces a Guard to intercept.
    + The Guard takes the damage instead of the original target.
    + The attacker moves *1 tile* toward the target (stops on the tile immediately before the Guard, not before the original target).

    *Interception is optional.* The defender may decline even if a Guard is eligible.

    Only Standard Attacks can be intercepted. Skills always hit directly.

    #if adjacency == "defender" [
      *Example:*

      ```
        . . . . .
        . G . . .
        . . C . .     C = your Champion, G = your Guard, A = enemy piece
        . . . A .
        . . . . .
      ```

      Enemy A attacks Champion C. Your Guard G is adjacent to C (but not to A). Under baseline this would not qualify — under Layer 3 it does. You intercept: A moves 1 tile, G takes 2 DMG, C is safe.
    ]
  ]
}

// ── SKILL DRAFTING ────────────────────────────────────────────────────────────

#let section-skill-drafting() = [
== Skill Drafting

+ Lay out all available skills face-up as a shared pool.
+ Alternating draft: P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King → P2 picks 2 skills and assigns freely → repeat.
+ Continue until all 5 Champions and the King on each side have 2 skills.
+ Both players draft from the same pool. Duplicates are allowed.
]

// ── PROGRESSION ───────────────────────────────────────────────────────────────

#let section-progression() = [
== Progression

#block(breakable: false)[
#table(
  columns: (auto, auto),
  table.header([Round], [Skill Slots per turn]),
  [1–10], [2],
  [11–20], [3],
  [21–30], [4],
  [31+], [5 (+1 every 10 rounds)],
)
]
]

// ── SKILL REFERENCE ───────────────────────────────────────────────────────────

#let section-skill-reference() = [
== Skill Reference

#skill-table(
  columns: (auto, auto, 1fr, auto, 2fr),
  table.header([], [Cat.], [Name], [Cost], [Effect]),
  skill-icon("lance_thrust"), [Strike], [Lance Thrust], [2], [Target within Range−1 takes 1 DMG],
  skill-icon("hook_pull"), [Strike], [Hook Pull], [3], [Target takes 1 DMG, pulled 1 tile toward caster along Skill Path],
  skill-icon("armor_breaker"), [Strike], [Armor Breaker], [2], [Remove 1 Armor from target],
  skill-icon("rune_theft"), [Strike], [Rune Theft], [3], [Target takes 1 DMG. Steal 1 Rune from opponent.],
  skill-icon("blade_tempest"), [Strike], [Blade Tempest], [4], [Target takes 1 DMG. All pieces adjacent to the target are pushed 1 tile away from the target. The attacker/caster is not affected.],
  skill-icon("rust_shield"), [Shield], [Rust Shield], [2], [Self: gain +1 Armor],
  skill-icon("field_medic"), [Shield], [Field Medic], [3], [Remove Injured from one adjacent ally],
  skill-icon("armor_smith"), [Shield], [Armorsmith], [3], [Adjacent ally gains +1 Armor],
  skill-icon("quick_dash"), [Move], [Quick Dash], [3], [Self: move up to 2 tiles along Skill Path],
  skill-icon("air_blast"), [Move], [Air Blast], [2], [Push target enemy 1 tile directly away from caster],
  skill-icon("precision_thrust"), [Move], [Precision Thrust], [3], [Push target enemy 1 tile in any direction (caster chooses). *Range+1 (Range 3 at default).*],
  skill-icon("shadow_shift"), [Move], [Shadow Shift], [4], [Swap position with an allied piece. Requires unobstructed Skill Path.],
  skill-icon("retreat_plan"), [Move], [Retreat Plan], [4], [Self: move along Skill Path to land adjacent to one of your Guards. *Range+1.*],
  skill-icon("focus_strike"), [Mystic], [Focus Strike], [1], [The next skill used by *any of your pieces* this turn gains +1 Range.],
  skill-icon("balde_call"), [Mystic], [Blade Call], [3], [One Strike skill used by *any of your pieces* this turn deals +1 DMG. Fixed cost: 3 Runes.],
)
]

// ── QUICK REFERENCE ───────────────────────────────────────────────────────────
// attack-damage: int (default 2)
// bodyguard-adjacency: "both" or "defender" (default "both")
// layer1-accepted: bool (default false) — adds Layer 1 note to Rune income row
// show-combo-bonus: bool (default false) — adds combo bonus row

#let section-quick-reference(
  attack-damage: 2,
  bodyguard-adjacency: "both",
  layer1-accepted: false,
  show-combo-bonus: false,
) = {
  let attack-label = if attack-damage != 2 [Attack ⚡] else [Attack]
  let attack-rule = if attack-damage != 2 [
    Move onto enemy tile (1 Move Slot). *#attack-damage DMG* _(baseline: 2)_.
  ] else [
    Move onto enemy tile (1 Move Slot). 2 DMG.
  ]
  let bodyguard-label = if bodyguard-adjacency == "defender" [Bodyguard ⚡] else [Bodyguard]
  let bodyguard-rule = if bodyguard-adjacency == "defender" [
    *Adjacent to defender only.* Guard takes the damage. Attacker moves 1 tile. Standard Attacks only. Optional.
  ] else [
    Guard adjacent to both tile-before-target and defender. Guard takes the damage. Attacker moves 1 tile. Standard Attacks only. Optional.
  ]
  let rune-note = if layer1-accepted [ Layer 1: 6 start, +2/turn scaling.] else []
  [
    == Quick Reference

    #block(breakable: false)[
    #table(
      columns: (1fr, 1.5fr),
      table.header([Concept], [Rule]),
      [Movement], [Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece once per phase.],
      [#attack-label], [#attack-rule],
      [Attack — target survives], [Attacker stops on tile before target],
      ..if show-combo-bonus {(
        [Combo Bonus ⚡],
        [2nd+ Champion Strike on same target in same turn: *+1 DMG*. Standard Attacks don't count.],
      )},
      [Skill Path], [Straight line (Queen). Blocked by all pieces.],
      [Default Skill Range], [Range 2 (unless skill specifies)],
      [#bodyguard-label], [#bodyguard-rule],
      [Rune income], [Start of YOUR turn (not Round 1).#rune-note],
      [Healing], [No cap. Same piece can be healed multiple times per turn.],
      [Armor], [Armor absorbs damage first, then HP],
    )
    ]
  ]
}
