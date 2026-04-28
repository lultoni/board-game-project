#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack A — Game 1: Standard Attack Nerf")

= Stack A — Game 1: Standard Attack Nerf

_Version: 2026-04-27. Based on baseline rules v2026-04-25 + Layer 1 economy (accepted)._\
_Feedback form (fill out after Game 2): stack-a-feedback.pdf_

#note-box[
  *One change from baseline + Layer 1.* Standard Attacks deal *1 DMG* instead of 2. Economy uses Layer 1 values (6 starting Runes, +2/turn scaling). All other rules unchanged.

  *Play this game FIRST*, then play Game 2 (adds combo bonus).
]

*What we're testing:* Does reducing Standard Attack damage from 2 to 1 make skills the primary damage source, create meaningful Injured states, and reduce the "wait and pounce" dynamic?

*Hypothesis:* Standard attacks at 2 DMG dominate the damage economy (free instant kills). At 1 DMG, a single attack only Injures — finishing a piece requires a second attack or a skill, making skills worth their Rune cost.

*Watch for:*
- Does the Injured state come up more often? Do Injured pieces survive multiple rounds?
- Is Guard clearing painfully slow? Track rounds to first Guard kill.
- Does the no-man's-land / standoff dissolve (lower risk to move forward)?
- Do players spend more Runes on Strike skills to finish pieces off?
- Does game length increase, decrease, or stay similar?

#hr

*All rules below carry Layer 1 economy (accepted). The only new change is marked ⚡.*

#section-goal()
#section-components()
#section-setup(start-runes: 6, layer1-accepted: true)
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack(damage: 1, changed: true)
#section-action-phase()
#section-skill-system()
#section-resource-economy(start-runes: 6, layer1-accepted: true)
#section-health-armor()
#section-bodyguard()
#section-skill-drafting()
#section-progression()

#pagebreak()

#section-skill-reference()
#section-quick-reference(
  attack-damage: 1,
  layer1-accepted: true,
)
