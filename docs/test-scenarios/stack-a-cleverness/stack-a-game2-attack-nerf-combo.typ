#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack A — Game 2: Attack Nerf + Combo Bonus")

= Stack A — Game 2: Attack Nerf + Combo Bonus

_Version: 2026-04-27. Based on baseline rules v2026-04-25 + Layer 1 economy (accepted)._\
_Feedback form (fill out after this game): stack-a-feedback.pdf_

#note-box[
  *Two changes from baseline + Layer 1.* Standard Attacks deal *1 DMG* (same as Game 1). *NEW:* Multi-Champion Combo Bonus — when a second Champion's Strike skill hits the same target in one turn, it deals +1 DMG.

  *Play this game SECOND* (after Game 1, nerf only). Compare your experience.
]

*What we're testing:* Does the combo bonus create a meaningful incentive for multi-Champion coordination, raise the skill combo ceiling, and help resolve the standoff problem?

*Hypothesis:* The standard attack nerf (Game 1) makes skills the primary damage source. The combo bonus rewards positioning two Champions with line-of-sight to the same target — a spatial puzzle that rewards cleverness over grinding.

*Watch for:*
- Do players attempt multi-Champion combos? How often do they succeed?
- Does the combo bonus feel like a meaningful reward for coordination?
- Does it break anything? (Too easy to set up? Too powerful with Blade Call?)
- Does it change how players position Champions relative to Game 1?
- Does the combo bonus speed up the game compared to Game 1?

#hr

*All rules below carry Layer 1 economy (accepted) + Game 1 attack nerf. The additional combo bonus change is marked ⚡.*

#section-goal()
#section-components()
#section-setup(start-runes: 6, layer1-accepted: true)
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack(damage: 1, changed: true)
#section-combo-bonus(enabled: true)
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
  show-combo-bonus: true,
)
