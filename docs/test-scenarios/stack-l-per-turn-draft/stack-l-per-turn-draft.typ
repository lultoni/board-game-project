#import "/docs/test-scenarios/shared/template.typ": *
#show: template.with(title: "Per-Turn-Draft Prototype")

/*
  Internal labels (not for player use): this is Stack L — Pole B per-turn-draft prototype.
  Players don't need either name. Read this sheet as the standalone ruleset for the game we're playing.

  Rationale (do not surface to players):
  - Skills are added to pieces during play, not all at once at the start.
  - Skills are consumed on activation: when a skill is activated in the Skill Phase, it is removed from the piece and returned to the shared pool.
  - Equipped count cap = 12 skills per player (6 Champions × 2 slots; King doesn't change the count).
  - Shared actions: Move Phase and Draft Phase use the same 4-action pool. Drafting a skill is a tempo cost vs. moving.
  - No Money-economy activation gate. Activate as many equipped skills as you want per turn; no per-turn cap.
  - Effectively infinite skill pool for drafting. The 12-equipped cap is the constraint, not the pool size.

  Full design discussion: docs/research/path-y-defense-redesign.md.
*/

= Per-Turn-Draft Prototype — Rules

_Version: 2026-05-31. Standalone — does not reference any other rule document._

#hr

== Introduction

You and your opponent each command an army of *Guards*, *Champions*, and a *King* on a 10×10 grid board. There is no luck — no dice, no hidden information.

In this version of the game, *you do not pick your skills before the game starts.* Instead, on each of your turns you can spend an action to draft a new skill onto one of your Champions or your King. Skills are drawn from a shared pool. You build your army's identity *while* you play it.

*How you win:* Capture the opponent's *King*. The game ends immediately when a King is removed.

*Turn flow:*
+ *Move Phase + Draft Phase* — share a pool of *4 actions*. Spend each action to either move a piece or draft a skill onto one of your Champions or King.
+ *Skill Phase* — activate any equipped skills you want. Each activation removes the skill from the piece and returns it to the pool.

#hr

== Goal

Capture the opponent's *King*. The game ends immediately when a King is removed from the board.

*Draw conditions:*
- No piece has been captured for 10 consecutive full rounds.
- Only the two Kings remain on the board.

== Components

*Per player:* 1 King · 5 Champions · 6 Guards

*Champions and the King have 2 Equip Slots each.* Skills are drafted into these slots during play. Guards have no Equip Slots and carry no skills.

*Shared (prototype):* 10×10 grid board · Skill cards (effectively infinite shared pool) · Injured markers · Armor pieces.

== Setup

- *Board:* 10×10 grid.
- *First player:* Flip a coin (or decide verbally).
- *Piece placement:* Each player places pieces on their own back two rows:
  - Back row (row 1/10): King in the centre, Champions fill the remaining 5 spaces.
  - Second row (row 2/9): All 6 Guards.
- *No pre-game draft.* All Champions and the King begin with *empty Equip Slots*.
- *Skill pool:* The full skill catalogue is laid out face-up between players, available to both throughout the game. Duplicates allowed.
- P1 begins Round 1.

== Round Structure

A *Round* = P1's Turn + P2's Turn.

== Turn Structure

#block(breakable: false)[
Each turn has three phases, in this order:

+ *Move Phase* — spend actions from your shared pool to move pieces.
+ *Draft Phase* — spend remaining actions from the same pool to draft skills onto your Champions or King.
+ *Skill Phase* — activate any of your equipped skills. No action cost, no per-turn cap, no resource cost.

The *Move Phase and Draft Phase share a pool of 4 actions.* You decide how many to spend on movement before advancing to the Draft Phase; any actions left over are available to draft. Once you advance from one phase to the next, you cannot return.

You may use 0 actions in any phase. Unused actions are *lost* — they do not carry into the next turn.
]

== Move Phase

#block(breakable: false)[
Spend actions from your shared 4-action pool to move pieces. *Each piece may only be moved once per turn.* Each action moves *one* piece.

A move can either go *into empty space* (normal movement) or *into an enemy tile* (a Move-Attack — see below).

#table(
  columns: (auto, auto, auto),
  table.header([Piece], [Normal speed], [Injured speed]),
  [Guard], [2 tiles], [1 tile],
  [Champion / King], [1 tile], [1 tile],
)

*Free pathing:* A piece may move in any direction — horizontal, vertical, diagonal — taking any route up to its speed in tiles. The route does not have to be a straight line.

*Pieces block movement:* A moving piece cannot pass through any other piece, ally or opponent.
- _Exception:_ A piece may go around a blocking piece if total tiles moved stays within speed (e.g. a Guard using 2 diagonal moves to go around a piece directly in the way).

*Move-Attack:* a Move that ends on an enemy tile. Spend 1 action to move your piece *onto a tile occupied by an enemy piece.*

- Deal *1 damage* to the enemy.
- *If the enemy is removed:* your piece occupies the tile (the attack consumed the move; remaining unused movement tiles do not carry over).
- *If the enemy survives* (Armor absorbed the damage): your piece *stops on the tile immediately before the target.* This applies to *every* attacker — a Guard with speed 2 ends up having moved only 1 tile; a Champion or King with speed 1 does not move at all. Either way the damage is dealt. (_If multiple paths reach the target, you choose which one — important for the Bodyguard Rule._)

You may attack with one action and move a different piece with another action in the same phase.

When you advance to the Draft Phase, any unused actions from your pool carry over and can be spent on drafting.
]

== Bodyguard Rule

#block(breakable: false)[
When you make a *Move-Attack* against an opponent's Champion or King, the defender may choose to have a Guard intercept — *if* a friendly Guard is on a tile adjacent to *both the tile immediately before the target (along the attack path) and the defending piece.*

*Interception:*
- Defender announces a Guard to intercept.
- The Guard takes the damage instead of the original target.
- The attacker moves *1 tile* toward the target (stops on the tile immediately before the target).

*Interception is optional.* The defender may decline even if a Guard is eligible, and may choose which Guard intercepts if multiple are eligible.

Only Move-Attacks can be intercepted. Skills always hit directly.
]

== Draft Phase

#block(breakable: false)[
Spend any remaining actions from your shared 4-action pool to *draft skills from the shared pool onto one of your Champions or King*. Each action drafts *one* skill.

To draft:
+ Choose any skill from the shared pool.
+ Choose one of your Champions or your King to receive it.
+ The chosen piece must have a *free Equip Slot* (each piece has 2).
+ Place the skill into that Equip Slot.

The skill is now *equipped* on that piece. Equipped skills are *not* used immediately — they sit on the piece, waiting for the Skill Phase, until either you activate them or the piece is removed.

*Drafting is public.* Your opponent sees which skill went onto which piece.

*The skill pool is effectively infinite.* You cannot run out of skills to draft. Duplicates of the same skill are allowed (on the same piece or on different pieces).

When you advance to the Skill Phase, any unused actions are *lost*.
]

== Skill Phase

#block(breakable: false)[
After the Draft Phase ends, you enter the Skill Phase.

*You may activate any of your equipped skills, in any order, with no per-turn cap and no resource cost.* You can activate one, several, or all of them. You can activate zero — the Skill Phase is optional.

To activate one equipped skill:
+ Announce the skill, the caster, and the target.
+ Apply the skill's effect.
+ *Remove the skill* from the piece — it is exhausted. Return it to the shared pool.

The same Champion can activate multiple equipped skills in one Skill Phase. Each activation exhausts and removes that one skill instance.

A skill that is removed this way is back in the pool — it can be drafted again on a future turn (by you or by your opponent).
]

== Skill System

#block(breakable: false)[
*Path:* Skills travel in a *straight line* (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

*Blocking:* The Path is blocked by *all pieces* — ally and opponent alike. The skill cannot reach past the first piece in its path.

*Range:* The distance in tiles along the Path from caster to target. Range is measured as:
- Range 0 = self (caster's own tile).
- Range 1 = adjacent tile along the Path.
- Range 2 = 2 tiles away along the Path (etc.).

*Default Range = 2.* Unless a skill's effect text explicitly says "self" or "adjacent". A skill with a Range modifier (e.g. "Range −1") is still a Range 2 skill with a modifier applied — it is not treated as adjacent or self.

*Self vs. adjacent:* "Self" skills (Range 0) target only the caster by default. "Adjacent" skills (Range 1) target only neighbouring pieces — they cannot target the caster, even with a Range buff. Range buffs (e.g. Focus) shift the targeting window outward. *A Range buff on a Self skill extends its reach: Self + Focus → Range 1.* Range buffs do not collapse Adjacent skills inward toward Self.

*Injured Range penalty:* Injured pieces have Range −1. Skills that explicitly name "self" or "adjacent" in their text are unaffected — they always work regardless of Injured status.

*Skills that move pieces do not deal damage.* Movement-via-skill (Dash, Swap, Retreat) does not count as a Move-Attack and deals no damage on arrival — you are instead stopped from moving the piece further in that direction.
]

== Combo Bonus

#block(breakable: false)[
Each enemy piece carries a *combo counter* — starts at 0 at the beginning of your turn and resets to 0 at the end of your turn.

When one of your *Strike* skills hits an enemy piece:
+ *Deal extra damage* equal to that enemy's current combo counter.
+ If the hitter is a *Champion that has not yet incremented this enemy's counter this turn*, then *increase the counter by 1* after the hit.

*What counts:* Strike skills only.\
*What does not count:* Move-Attacks, Move skills, Shield skills, Mystic skills.

The Charge skill (+1 damage to one Strike this turn) stacks on top of the combo bonus.
]

== Health & Armor

#block(breakable: false)[
*All pieces have 2 HP:* Normal → Injured → Removed.

#table(
  columns: (auto, auto, auto),
  table.header([State], [HP], [Effect]),
  [Normal], [2], [No penalty.],
  [Injured], [1], [Speed capped at 1. Range −1 (affects Range 2+ only).],
  [Removed], [0], [Piece leaves the board permanently. *Any skills equipped on the piece are removed with it and return to the shared pool.*],
)

*Damage:*
- 1 damage to Normal → Injured.
- 1 damage to Injured → Removed.
- 2 damage to Normal → Removed instantly (Injured state is skipped).

*Armor:* Max 3 points per piece. Each absorbs 1 damage, then is destroyed. Resolves before HP damage. *Does not prevent Injured status.*
]

== Skill Reference

#block(breakable: false)[
#skill-table(
  columns: (auto, auto, auto, auto),
  table.header([], [Cat.], [Name], [Effect]),
  skill-icon("lance_thrust"), [Strike], [Lance], [Target within Range −1 takes 1 damage.],
  skill-icon("hook_pull"), [Strike], [Hook], [Target takes 1 damage, pulled 1 tile toward caster along the Path.],
  skill-icon("armor_breaker"), [Strike], [Break], [Remove 1 Armor from target. _(No HP damage unless boosted by Charge.)_],
  skill-icon("rune_theft"), [Strike], [Steal], [Target takes 1 damage. _(Note: in this prototype there is no Money to steal — Steal acts as a 1-damage Strike skill.)_],
  skill-icon("blade_tempest"), [Strike], [Tempest], [Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. Caster unaffected.],
  skill-icon("rust_shield"), [Shield], [Shield], [Self: gain +1 Armor.],
  skill-icon("field_medic"), [Shield], [Heal], [Remove Injured from one adjacent ally.],
  skill-icon("armor_smith"), [Shield], [Plate], [Adjacent ally gains +1 Armor.],
  skill-icon("quick_dash"), [Move], [Dash], [Self: move up to 2 tiles along the Path.],
  skill-icon("air_blast"), [Move], [Blast], [Push target enemy 1 tile directly away from caster.],
  skill-icon("precision_thrust"), [Move], [Shove], [Push target enemy 1 tile in any direction (caster chooses). *Range +1.*],
  skill-icon("shadow_shift"), [Move], [Swap], [Swap position with an allied piece. Requires unobstructed Path.],
  skill-icon("retreat_plan"), [Move], [Retreat], [Self: move along the Path to land adjacent to one of your Guards. *Range +1.*],
  skill-icon("focus_strike"), [Mystic], [Focus], [The next skill used by *any of your pieces* this turn gains +1 Range. _(On Move skills: caster picks at activation whether the +1 applies to the activation range or the effect range. Not both.)_],
  skill-icon("blade_call"), [Mystic], [Charge], [One Strike skill used by *any of your pieces* this turn deals +1 damage.],
)
]

#pagebreak()

== Quick Reference

#block(breakable: false)[
#table(
  columns: (auto, auto),
  table.header([Concept], [Rule]),

  [Turn structure],
  [*Move Phase* → *Draft Phase* (share 4 actions) → *Skill Phase* (activate any equipped skills, no cap).],

  [Movement],
  [Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece moves once per turn.],

  [Guard speed],
  [Normal: 2 tiles. Injured: 1 tile.],

  [Champion / King speed],
  [1 tile (Normal or Injured).],

  [Move-Attack],
  [Move onto enemy tile (1 action). Deals *1 damage*. Attacker stops before target if target survives.],

  [Drafting],
  [1 action: take a skill from the shared pool and place it on a Champion or King with a free Equip Slot. Public.],

  [Equip Slots],
  [Each Champion and King has 2. Permanent until the slot is used (skill activated → slot freed) or the piece is removed.],

  [Skill activation],
  [In Skill Phase only. No action cost, no resource cost, no per-turn cap. Activating exhausts the skill — it leaves the piece and returns to the pool.],

  [Skill pool],
  [Shared, effectively infinite, public. Duplicates allowed.],

  [Path],
  [Straight line (Queen-style). Blocked by *all* pieces — ally and enemy.],

  [Range],
  [Default Range 2. Skills with "self" = Range 0. Skills with "adjacent" = Range 1. Range modifiers (e.g. Range −1) apply from default.],

  [Self + Focus],
  [Self skills (Range 0) extend to Range 1 with Focus. Adjacent skills do not collapse to Self.],

  [Injured Range penalty],
  [Injured pieces: Range −1. Does not affect "self" or "adjacent" skills.],

  [Bodyguard],
  [Move-Attacks on Champion/King only. Guard must be adjacent to both tile-before-target AND defender. Guard takes the hit.],

  [Armor],
  [Max 3 per piece. Each point absorbs 1 damage before HP, then is destroyed. Does not prevent Injured status.],

  [Health],
  [2 HP: Normal → Injured → Removed. 1 damage = one step. 2 damage = skip Injured, Removed instantly. Removed piece's equipped skills return to pool.],

  [Combo Bonus],
  [Each enemy has a combo counter (resets end of your turn). Strike skill hits enemy → deal *+counter damage*; if hitter is a *new* Champion, counter +1. Move-Attacks excluded.],

  [Charge],
  [+1 damage to one Strike skill this turn. Stacks with Combo Bonus.],

  [Focus],
  [+1 Range to next skill this turn. On Move skills: choose at activation whether +1 applies to the activation range OR the effect range.],
)
]

/*
  Internal — for the playtest pair only. Not for player consumption.

  Skip this for any actual rules question; the rules end at the Quick Reference.

  What we're testing: Whether moving the draft into the game flow (a) produces a different game-feel,
  (b) shortens games, and (c) dissolves the "Armor as late-game tax" pattern at the structural level.
  Full rationale in `docs/research/path-y-defense-redesign.md`.

  Watch flags during play:
  - One-turn killer. Did either player hold equipped skills across multiple turns and dump 4–5 activations
    in a single Skill Phase that ended the game? Log the round and the activation count.
  - Cognitive load. Drafting + playing + reading the opponent's likely future picks. Does the Move /
    Draft decision feel rich, or paralysing?
  - Game length. Compared to P4 baseline (28–29 rounds, ~2h30). Did games come in shorter, longer,
    or the same?
  - Defense identity. Did either player draft a defensive loadout (Plate, Shield, Heal) early or did they
    default to offensive picks and only build defense reactively?
  - Empty-piece exposure. Did pieces sit on the board with 0 skills equipped for long stretches? How did
    that change tactics?
  - 4 actions feels. Too few? Too many? Right? Note any turn where you wanted a 5th action or sat with
    leftover actions you couldn't use.

  Backpocket items for this stack (do not apply during play):
  - Skills cost a resource to activate.
  - Per-Skill-Phase activation cap.
  - Permanently equipped (non-consumable) drafted skills.
*/
