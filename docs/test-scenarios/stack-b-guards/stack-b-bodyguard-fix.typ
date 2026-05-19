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
#section-skill-drafting()
#section-setup()
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack()
#section-action-phase()
#section-skill-system()
#section-resource-economy()
#section-progression()
#section-health-armor()

== ⚡ CHANGED: Bodyguard Rule

#changed-box[
  #table(
    columns: (1fr, 1fr, 1fr),
    table.header([], [Baseline], [This Layer]),
    [Guard must be adjacent to], [*Both tile-before-target AND defender*], [*Defender only*],
    [Everything else], [—], [Unchanged],
  )
]

When you make a *Standard Attack* against an opponent's Champion or King, the defender may choose to have a Guard intercept — *if* a friendly Guard is on a tile *adjacent to the defending piece* _(baseline requires adjacent to both the tile immediately before the target AND the defending piece)_.

*Interception:*
+ Defender announces a Guard to intercept.
+ The Guard takes the damage instead of the original target.
+ The attacker moves *1 tile* toward the target (stops on the tile immediately before the Guard, not before the original target).

*Interception is optional.* The defender may decline even if a Guard is eligible.

Only Standard Attacks can be intercepted. Skills always hit directly.

*Example:*

```
  . . . . .
  . G . . .
  . . C . .     C = your Champion, G = your Guard, A = enemy piece
  . . . A .
  . . . . .
```

Enemy A attacks Champion C. Your Guard G is adjacent to C (but not to A). Under baseline this would not qualify — under this layer it does. You intercept: A moves 1 tile toward C, G takes the damage, C is safe.

#section-skill-drafting()

#pagebreak()

#section-skill-reference()

== Quick Reference

#block(breakable: false)[
#table(
  columns: (1fr, 1.5fr),
  table.header([Concept], [Rule]),
  [Movement], [Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece once per phase.],
  [Attack], [Move onto enemy tile (1 Move Slot). *1 DMG*.],
  [Attack — target survives], [Attacker stops on tile before target],
  [Skill Path], [Straight line (Queen). Blocked by all pieces.],
  [Default Skill Range], [Range 2 (unless skill specifies)],
  [Bodyguard ⚡], [Guard adjacent to *defender only*. Guard takes the damage. Attacker moves 1 tile. Standard Attacks only. Optional.],
  [Rune income], [Start of YOUR turn (not Round 1). 6 start, +2/+3/+4/+5 scaling.],
  [Healing], [No cap. Same piece can be healed multiple times per turn.],
  [Armor], [Armor absorbs damage first, then HP],
)
]
