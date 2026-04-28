#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack B: Bodyguard Rule Fix")

= Stack B: Bodyguard Rule Fix

_Version: 2026-04-25. Based on baseline rules v2026-04-25._\
_Feedback form (fill out after the game): stack-b-feedback.pdf_

#note-box[
  *One change from baseline.* Only the Bodyguard Rule adjacency requirement is different. Independent of Stack A — run any time. Carry all accepted changes (Layer 1 economy) forward.
]

*What we're testing:* Does loosening the Bodyguard adjacency requirement to "adjacent to defender only" make the rule actually trigger and Guards genuinely useful?

*Hypothesis:* "Adjacent to both attacker AND defender" was too restrictive. "Adjacent to defender only" makes Guard positioning much simpler and the rule much more likely to fire.

*Watch for:*
- Does the Bodyguard rule trigger? How many times per game?
- Do players actively position Guards to protect Champions/King?
- Does it feel too easy to bodyguard? Is a 1-Guard wall impenetrable?
- Do Guards survive longer or die faster?

#hr

*All rules below are current baseline. The only changed section is marked ⚡.*

#section-goal()
#section-components()
#section-setup(start-runes: 6, layer1-accepted: true)
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack()
#section-action-phase()
#section-skill-system()
#section-resource-economy(start-runes: 6, layer1-accepted: true)
#section-health-armor()
#section-bodyguard(adjacency: "defender", changed: true)
#section-skill-drafting()
#section-progression()

#pagebreak()

#section-skill-reference()
#section-quick-reference(
  bodyguard-adjacency: "defender",
  layer1-accepted: true,
)
