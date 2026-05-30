// baseline-sections.typ
// Baseline section functions for all (GAME NAME) rule sheets.
// Import this file alongside template.typ in any rule sheet or layer file.
//
// Usage:
//   #import "/docs/test-scenarios/shared/template.typ": *
//   #import "/docs/test-scenarios/shared/baseline-sections.typ": *
//
// Every function renders one section with the current accepted baseline rules.
// No parameters — functions are intentionally not configurable.
// Test files inline their own changed sections directly; they do not pass flags here.

#import "./template.typ": *

// ── BASELINE VERSION ──────────────────────────────────────────────────────────
// Single source of truth for the baseline ruleset version. Stack files reference
// this constant rather than restating a date. Bump when an accepted mechanic
// modifies any baseline section function below.

#let BASELINE_VERSION = "2026-05-30"

// ── INTRODUCTION ──────────────────────────────────────────────────────────────

#let section-introduction() = [
== Introduction

*(GAME NAME)* is a 2-player abstract-tactical board game. There is no luck — no dice, no hidden information. Every decision is made with full knowledge of the board.

*What you command:* Each player leads an army of Guards and Champions, all serving a King. Guards are fast, Champions carry powerful skills. Your King is the most important piece on the board — and the target.

*How you win:* Capture the opponent's King. The game ends immediately when a King is removed.

*How a game flows:*
Players alternate turns. On your turn you first move your pieces (and attack), then activate skills. Piece capture, skill combos, and board control build toward the moment you threaten — and take — the enemy King.

*What makes it deep:* Skills cost Money. Money is scarce. You will always want to do more than you can. That tension — between movement, attacks, and skill combos — is the game.
]

// ── SIMPLE OVERVIEW ───────────────────────────────────────────────────────────

#let section-simple-overview() = [
== Simple Overview

_A surface-level map of every system. No edge cases — just what each thing does._

*Rounds and turns:* The game is played in Rounds. Each Round, P1 takes a full turn, then P2. A turn has two phases: Move Phase, then Skill Phase.

*Move Phase:* You have 2 actions. Spend one to move a piece up to its speed. Each piece can only be moved once per phase.

*Move-Attack:* Instead of moving into empty space, spend an action to move your piece onto an enemy tile. The enemy takes 1 damage.

*Health:* Every piece has 2 HP: Normal → Injured → Removed. Taking 1 damage moves a piece one step along that track. Injured pieces are slower and have shorter skill range.

*Armor:* A piece can hold up to 3 Armor. Each point absorbs 1 incoming damage *before* HP is affected, then is destroyed.

*Skill Phase:* You have actions (starting at 2 per turn). Spend one action + Money to activate an equipped skill on one of your Champions or King.

*Money:* Your currency. You earn it automatically each turn (starting at +2/turn, scaling up over time). Skills cost Money.

*Skills and Path:* Skills travel in a straight line from the caster (like a chess Queen). That path is blocked by any piece in the way. Most skills have a default range of 2 tiles.

*Bodyguard:* When a Champion or King is hit by a Move-Attack, you can choose to have an adjacent Guard take the hit instead.

*Skill Drafting:* Before the game, players alternate picking skills from a shared pool and assigning them to their Champions and King. Each piece gets 2 skills.

*Progression:* As the game goes on, your Money income and the number of actions in your Skill Phase both grow — late-game turns are more powerful because you can cast more skills.

_For full rules and edge cases, read the sections below._
]

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

*Champions and the King have 2 Equip Slots each.* Skills are equipped into these slots. Guards have no Equip Slots and carry no skills.

*Shared (Prototype):* 10x10 grid board · Skill cards · Injured markers · Armor pieces
]

// ── SETUP ─────────────────────────────────────────────────────────────────────

#let section-setup() = [
== Setup

- *Board:* Use the 10x10 grid.
- *First player:* Flip a coin (or decide verbally).
- *Piece placement:* Each player places pieces on their own back two rows:
  - Back row (row 1/10): King in the centre, Champions fill the remaining 5 spaces.
  - Second row (row 2/9): All 6 Guards.
- *Skill Draft:* Alternating picks. P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King, then P2. Repeat until both Equip Slots on every Champion and King are filled (12 skills per player). Duplicates allowed.
- *Starting Money:* Each player starts with *6 Money*.
- P1 begins Round 1.
]

// ── ROUND STRUCTURE ───────────────────────────────────────────────────────────

#let section-round-structure() = [
== Round Structure

A *Round* = P1's Turn + P2's Turn.

At the *start of each player's turn* (before they do anything), that player collects their Money income. _(Exception: Round 1 — players begin with their starting Money and collect nothing before their first turn.)_
]

// ── TURN STRUCTURE ────────────────────────────────────────────────────────────

#let section-turn-structure() = [
== Turn Structure

Each *Turn* has two phases, in order:

- *Move Phase* — spend actions to move pieces (and attack).
- *Skill Phase* — spend actions to activate skills.

You may use 0 actions in either phase.
]

// ── MOVE PHASE ────────────────────────────────────────────────────────────────

#let section-move-phase() = [
== Move Phase

You have *2 actions* per turn. Spend 1 action to move one piece — either *into empty space* (normal movement) or *into an enemy tile* (a Move-Attack — see next section). *Each piece may only be moved once per Move Phase.*

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

// ── MOVE-ATTACK ───────────────────────────────────────────────────────────────

#let section-move-attack() = [
== Move-Attack

A *Move-Attack* is a Move that ends on an enemy tile. Spend 1 action to move your piece *onto a tile occupied by an enemy piece.*

- Deal *1 damage* to the enemy.
- *If the enemy is removed:* your piece occupies the tile (the attack consumed the move; remaining unused movement tiles do not carry over).
- *If the enemy survives* (Armor absorbed the damage): your piece *stops on the tile immediately before the target.* This applies to *every* attacker — a Guard with speed 2 ends up having moved only 1 tile; a Champion or King with speed 1 does not move at all. Either way the damage is dealt. (_If multiple paths reach the target, you choose which one — important for the Bodyguard Rule._)

You may attack with one action and move a different piece with the other action in the same turn.

_Move-Attacks are how pieces deal damage with movement alone. Skills — activated in the Skill Phase — are the other way pieces affect each other, and use different rules (Path, Range, Money)._
]

// ── MULTI-CHAMPION COMBO BONUS ────────────────────────────────────────────────
// Accepted into baseline Session 23 (2026-05-30) following Stack A G2 (Playtest 4)
// confirmation. Concise version — full worked examples and tracking tables live
// in the Stack A G3 design (see backpocket.md / TESTING_PLAN.typ) when needed.

#let section-multi-champion-combo() = [
== Multi-Champion Combo Bonus

Each enemy piece has a *combo counter* (starts at 0, resets at the end of your turn). When a *Strike skill* hits it, deal *+counter damage*; if the hitter is a *new Champion* (one that didn't already increment this counter this turn), the counter then *+1*. Move-Attacks don't count. Buffs, heals, and movement don't count. Stacks with Charge.
]

// ── SKILL PHASE ───────────────────────────────────────────────────────────────

#let section-skill-phase() = [
== Skill Phase

You have *2 actions* per turn (at the start (see Progression)).

Spend 1 action to activate one equipped skill on one of your Champions or King:
- Announce the skill and the target.
- Pay the skill's Money cost.
- Apply the skill's effect.

The same Champion can activate multiple skills (also the same one) in one turn if you have actions remaining.
]

// ── SKILL SYSTEM ──────────────────────────────────────────────────────────────

#let section-skill-system() = [
== Skill System

*Path:* Skills travel in a *straight line* (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

*Blocking:* The Path is blocked by *all pieces* — ally and opponent alike. The skill cannot reach past the first piece in its path.

*Range:* The distance in tiles along the Path from caster to target. Range is measured as:
- Range 0 = self (caster's own tile)
- Range 1 = adjacent tile along the Path
- Range 2 = 2 tiles away along the Path (etc.)

*Default Range = 2.* Unless a skill explicitly names "self" or "adjacent" in its effect text. A skill with a Range modifier (e.g. "Range -1") is still a Range 2 skill with a modifier applied — it is not treated as adjacent or self.

*Self vs. adjacent:* "Self" skills (Range 0) target only the caster — they cannot target adjacent pieces by default. "Adjacent" skills (Range 1) target only neighbouring pieces — they cannot target the caster, even with a Range buff. Range buffs (e.g. Focus) shift the targeting window outward. A Range buff on a Self skill *extends* its reach: Self + Focus → Range 1. Range buffs do not collapse Adjacent skills inward toward Self.

*Injured Range penalty:* Injured pieces have Range -1. Skills that explicitly name "self" or "adjacent" in their text are unaffected — they always work regardless of Injured status.

*Skills that move pieces do not deal damage.* Movement-via-skill (Dash, Swap, Retreat) does not count as a Move-Attack and deals no damage on arrival - you are instead stopped from moving the piece further in that direction.

*A Champion may use its skills multiple times in the same turn* if actions are available — including the same skill twice.

All skills cost 1 action unless noted otherwise.
]

// ── RESOURCE ECONOMY ──────────────────────────────────────────────────────────
// Layer 1 accepted — Playtest 2, 24.04.2026

#let section-resource-economy() = [
== Money

*Starting Money:* 6 per player.

Money income is collected at the *start of each player's own turn:*

#table(
  columns: (auto, 1fr),
  table.header([Round], [Income per player turn]),
  [1], [0 (starting Money only)],
  [2-4], [+2],
  [5-9], [+3],
  [10-14], [+4],
  [15+], [+5 (+1 every 5 rounds)],
)

*No Money cap.*
]

// ── HEALTH & ARMOR ────────────────────────────────────────────────────────────

#let section-health-armor() = [
== Health & Armor

*All pieces have 2 HP:* Normal → Injured → Removed.

#table(
  columns: (auto, auto, 1fr),
  table.header([State], [HP], [Effect]),
  [Normal], [2], [No penalty],
  [Injured], [1], [Speed capped at 1. Range -1 (affects Range 2+ only).],
  [Removed], [0], [Piece leaves the board permanently.],
)

*Damage:*
- 1 damage to Normal → Injured.
- 1 damage to Injured → Removed.
- 2 damage to Normal → Removed instantly (Injured state is skipped).

*Armor:* Max 3 points per piece. Each absorbs 1 damage, then destroyed. Resolves before HP damage. *Does not prevent Injured status.*
]

// ── BODYGUARD RULE ────────────────────────────────────────────────────────────

#let section-bodyguard() = [
== Bodyguard Rule

When you make a *Move-Attack* against an opponent's Champion or King, the defender may choose to have a Guard intercept — *if* a friendly Guard is on a tile adjacent to *both the tile immediately before the target (along the attack path) and the defending piece.*

*Interception:*
- Defender announces a Guard to intercept.
- The Guard takes the damage instead of the original target.
- The attacker moves *1 tile* toward the target (stops on the tile immediately before the target).

*Interception is optional.* The defender may decline even if a Guard is eligible and can select which guard should take the damage if multiple Guards are eligble.

Only Move-Attacks can be intercepted. Skills always hit directly.
]

// ── SKILL DRAFTING ────────────────────────────────────────────────────────────

#let section-skill-drafting() = [
== Skill Drafting

- Lay out all available skills face-up as a shared pool.
- Alternating draft: P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King → P2 picks 2 skills and assigns freely → repeat.
- Continue until all 5 Champions and the King on each side have 2 skills.
- Both players draft from the same pool. Duplicates are allowed.
]

// ── PROGRESSION ───────────────────────────────────────────────────────────────

#let section-progression() = [
== Progression

#table(
  columns: (auto, auto),
  table.header([Round], [Actions per Skill Phase]),
  [1-10], [2],
  [11-20], [3],
  [21-30], [4],
  [31+], [5 (+1 every 10 rounds)],
)
]

// ── SKILL REFERENCE ───────────────────────────────────────────────────────────

#let section-skill-reference() = [
== Skill Reference

#skill-table(
  columns: (auto, auto, 1fr, auto, 2fr),
  table.header([], [Cat.], [Name], [Cost], [Effect]),
  skill-icon("lance_thrust"), [Strike], [Lance], [2], [Target within Range-1 takes 1 damage],
  skill-icon("hook_pull"), [Strike], [Hook], [3], [Target takes 1 damage, pulled 1 tile toward caster along the Path],
  skill-icon("armor_breaker"), [Strike], [Break], [2], [Remove 1 Armor from target. _(Note: does not deal "HP-Damage" unless boosted by Charge)_],
  skill-icon("rune_theft"), [Strike], [Steal], [3], [Target takes 1 damage. Steal 1 Money from opponent.],
  skill-icon("blade_tempest"), [Strike], [Tempest], [4], [Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. The attacker/caster is not affected.],
  skill-icon("rust_shield"), [Shield], [Shield], [2], [Self: gain +1 Armor],
  skill-icon("field_medic"), [Shield], [Heal], [3], [Remove Injured from one adjacent ally],
  skill-icon("armor_smith"), [Shield], [Plate], [3], [Adjacent ally gains +1 Armor],
  skill-icon("quick_dash"), [Move], [Dash], [3], [Self: move up to 2 tiles along the Path],
  skill-icon("air_blast"), [Move], [Blast], [2], [Push target enemy 1 tile directly away from caster],
  skill-icon("precision_thrust"), [Move], [Shove], [3], [Push target enemy 1 tile in any direction (caster chooses). *Range+1.*],
  skill-icon("shadow_shift"), [Move], [Swap], [4], [Swap position with an allied piece. Requires unobstructed Path.],
  skill-icon("retreat_plan"), [Move], [Retreat], [4], [Self: move along the Path to land adjacent to one of your Guards. *Range+1.*],
  skill-icon("focus_strike"), [Mystic], [Focus], [1], [The next skill used by *any of your pieces* this turn gains +1 Range. _(Note: can boost self and adjacent skills — Range 0 → 1, Range 1 → 2.)_ #h(0pt) *Move skills:* the caster chooses, when activating the Move skill, whether the +1 applies to its *activation range* (how far the skill can target) or its *effect range* (how far it moves/pushes). Not both.],
  skill-icon("blade_call"), [Mystic], [Charge], [3], [One Strike skill used by *any of your pieces* this turn deals +1 damage.],
)
]

// ── QUICK REFERENCE ───────────────────────────────────────────────────────────
//
// Use `overrides` to replace specific rows for a stack — keys are the Concept
// strings shown in column 1. Pass content for the value, e.g.:
//
//   #section-quick-reference(overrides: (
//     "Move-Attack": [Move onto enemy tile (1 action). *1 damage* _(baseline: 2)_. Attacker stops before target if target survives.],
//   ))
//
// Use `extra-rows` to inject additional rows (in order). Each entry is a
// `(concept, rule)` pair, e.g.:
//
//   #section-quick-reference(extra-rows: (
//     ("Combo Bonus ⚡", [Each enemy has a hit counter ...]),
//   ))
//
// Stack files that diverge significantly from baseline (different row set
// entirely) should still inline their own table — overrides is for one-or-two
// row swaps that share the canonical structure.

#let section-quick-reference(overrides: (:), extra-rows: ()) = {
  let baseline-rows = (
    ("Movement",                 [Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece once per Move Phase.]),
    ("Guard speed",              [Normal: 2 tiles. Injured: 1 tile.]),
    ("Champion / King speed",    [1 tile (Normal or Injured).]),
    ("Move-Attack",              [Move onto enemy tile (1 action). Deals *1 damage*. Attacker stops before target if target survives.]),
    ("Path",                     [Straight line (Queen-style). Blocked by *all* pieces — ally and enemy.]),
    ("Range",                    [Default Range 2. Skills with "self" = Range 0. Skills with "adjacent" = Range 1. Range modifiers (e.g. Range−1) apply from default.]),
    ("Injured Range penalty",    [Injured pieces: Range −1. Does not affect "self" or "adjacent" skills.]),
    ("Bodyguard",                [Move-Attacks on Champion/King only. Guard must be adjacent to both tile-before-target AND defender. Guard takes the hit. Defender chooses which eligible Guard intercepts.]),
    ("Armor",                    [Max 3 per piece. Each point absorbs 1 damage before HP, then is destroyed. Does not prevent Injured status.]),
    ("Health",                   [2 HP: Normal → Injured → Removed. 1 damage = one step. 2 damage = skip Injured, Removed instantly.]),
    ("Money income",             [Collected at start of YOUR turn (not Round 1). Starts 6, then +2/+3/+4/+5 scaling.]),
    ("Skill Phase actions",      [Start at 2/turn. Grow with Progression (Rounds 1–10: 2, 11–20: 3, 21–30: 4, 31+: 5).]),
    ("Focus",                    [+1 Range to next skill this turn. Can boost self (→ adjacent) and adjacent (→ Range 2) skills. On Move skills, caster chooses: activation range OR effect range.]),
    ("Charge",                   [+1 damage to one Strike skill this turn. Stacks with Combo Bonus.]),
    ("Combo Bonus",              [Each enemy has a combo counter (resets end of your turn). Strike skill hits enemy → deal *+counter damage*; if hitter is a *new* Champion, counter +1. Move-Attacks excluded.]),
  )

  let resolved = baseline-rows.map(row => {
    let (concept, rule) = row
    if concept in overrides { (concept, overrides.at(concept)) } else { (concept, rule) }
  })
  let final-rows = resolved + extra-rows.map(r => (r.at(0), r.at(1)))

  [
== Quick Reference

#block(breakable: true)[
#table(
  columns: (1fr, 1.5fr),
  table.header([Concept], [Rule]),
  ..final-rows.map(row => ([#row.at(0)], row.at(1))).flatten()
)
]
  ]
}
