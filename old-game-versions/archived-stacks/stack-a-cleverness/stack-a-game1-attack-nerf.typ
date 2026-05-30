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

#section-quick-reference(overrides: (
  "Standard Attack": [Move onto enemy tile (1 Move Slot). *1 DMG* _(baseline: 2)_. Attacker stops before target if target survives.],
))
