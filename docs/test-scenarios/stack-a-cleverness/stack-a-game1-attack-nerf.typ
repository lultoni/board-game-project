#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack A — Game 1: Standard Attack Nerf")

= Stack A — Game 1: Standard Attack Nerf

_Version: 2026-05-19. Based on baseline rules v2026-05-19 + Layer 1 economy (accepted)._\
_Feedback form (fill out after Game 2): stack-a-feedback.pdf_

#note-box[
  *One change from baseline + Layer 1.* Standard Attacks deal *1 DMG* instead of 2. Economy uses Layer 1 values (6 starting Runes, +2/turn scaling). All other rules unchanged.

  *Play this game FIRST*, then play Game 2 (adds combo bonus).
]

#designer-box[
*Designer notes — read before facilitating, players can skip.*

*What we're testing:* Does reducing Standard Attack damage from 2 to 1 make skills the primary damage source, create meaningful Injured states, and reduce the "wait and pounce" dynamic?

*Hypothesis:* Standard attacks at 2 DMG dominate the damage economy (free instant kills). At 1 DMG, a single attack only Injures — finishing a piece requires a second attack or a skill, making skills worth their Rune cost.

*Watch for:*
- Does the Injured state come up more often? Do Injured pieces survive multiple rounds?
- Is Guard clearing painfully slow? Track rounds to first Guard kill.
- Does the no-man's-land / standoff dissolve (lower risk to move forward)?
- Do players spend more Runes on Strike skills to finish pieces off?
- Does game length increase, decrease, or stay similar?
]

#hr

*All rules below carry Layer 1 economy (accepted). The only new change is marked ⚡.*

#section-goal()
#section-components()
#section-skill-drafting()
#section-setup()
#section-round-structure()
#section-turn-structure()
#section-movement-phase()

== ⚡ CHANGED: Standard Attack

#changed-box[
  #table(
    columns: (1fr, 1fr, 1fr),
    table.header([], [Baseline], [This Layer]),
    [Standard Attack damage], [*2 DMG*], [*1 DMG*],
    [Everything else], [—], [Unchanged],
  )
]

To attack, spend a Move Slot to move your piece *onto a tile occupied by an enemy piece.*

- Deal *1 DMG* to the enemy _(baseline: 2 DMG)_.
- *If the enemy is removed:* your piece occupies the tile.
- *If the enemy survives:* your piece stops on the tile immediately before the target. _(If there are multiple paths toward the target you may choose which one to take — relevant for Bodyguard.)_

You may attack with one Move Slot and move a different piece with the other Move Slot in the same turn.

#section-action-phase()
#section-skill-system()
#section-resource-economy()
#section-progression()
#section-health-armor()
#section-bodyguard()

#pagebreak()

#section-skill-reference()

== Quick Reference

#block(breakable: false)[
#table(
  columns: (1fr, 1.5fr),
  table.header([Concept], [Rule]),
  [Movement], [Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece once per Movement Phase.],
  [Guard speed], [Normal: 2 tiles. Injured: 1 tile.],
  [Champion / King speed], [1 tile (Normal or Injured).],
  [Standard Attack ⚡], [Move onto enemy tile (1 Move Slot). *1 DMG* _(baseline: 2)_. Attacker stops before target if target survives.],
  [Skill Path], [Straight line (Queen-style). Blocked by *all* pieces — ally and enemy.],
  [Skill Range], [Default Range 2. Skills with "self" = Range 0. Skills with "adjacent" = Range 1. Range modifiers apply from default.],
  [Injured Range penalty], [Injured pieces: Skill Range −1. Does not affect "self" or "adjacent" skills.],
  [Bodyguard], [Standard Attacks on Champion/King only. Guard adjacent to both tile-before-target AND defender. Guard takes the hit. Defender chooses which eligible Guard intercepts.],
  [Armor], [Max 3 per piece. Each point absorbs 1 DMG before HP, then destroyed. Does not prevent Injured status.],
  [Health], [2 HP: Normal → Injured → Removed. 1 DMG = one step. 2 DMG = skip Injured, Removed instantly.],
  [Rune income], [Collected at start of YOUR turn (not Round 1). Starts 6, then +2/+3/+4/+5 scaling.],
  [Skill Slots], [Start at 2/turn. Grow with Progression (Rounds 1–10: 2, 11–20: 3, 21–30: 4, 31+: 5).],
  [Focus Strike], [+1 Range to next skill this turn. Can boost self (→ adjacent) and adjacent (→ Range 2) skills.],
  [Blade Call], [+1 DMG to one Strike skill this turn.],
)
]
