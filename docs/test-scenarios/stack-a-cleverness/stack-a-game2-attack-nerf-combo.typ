#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack A — Game 2: Attack Nerf + Combo Bonus")

= Stack A — Game 2: Attack Nerf + Combo Bonus

_Version: 2026-05-19. Based on baseline rules v2026-05-19 + Layer 1 economy (accepted)._\
_Feedback form (fill out after this game): stack-a-feedback.pdf_

#note-box[
  *Two changes from baseline + Layer 1.* Standard Attacks deal *1 DMG* (same as Game 1). *NEW:* Multi-Champion Combo Bonus — when a second Champion's Strike skill hits the same target in one turn, it deals +1 DMG.

  *Play this game SECOND* (after Game 1, nerf only). Compare your experience.
]

#designer-box[
*Designer notes — read before facilitating, players can skip.*

*What we're testing:* Does the combo bonus create a meaningful incentive for multi-Champion coordination, raise the skill combo ceiling, and help resolve the standoff problem?

*Hypothesis:* The standard attack nerf (Game 1) makes skills the primary damage source. The combo bonus rewards positioning two Champions with line-of-sight to the same target — a spatial puzzle that rewards cleverness over grinding.

*Watch for:*
- Do players attempt multi-Champion combos? How often do they succeed?
- Does the combo bonus feel like a meaningful reward for coordination?
- Does it break anything? (Too easy to set up? Too powerful with Blade Call?)
- Does it change how players position Champions relative to Game 1?
- Does the combo bonus speed up the game compared to Game 1?
]

#hr

*All rules below carry Layer 1 economy (accepted) + Game 1 attack nerf. The additional combo bonus change is marked ⚡.*

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

== ⚡ NEW: Multi-Champion Combo Bonus

#changed-box[
  *New rule — not in baseline.* Each enemy piece has a *combo counter* (starts at 0, resets at the end of your turn). When a Strike skill successfully hits that piece, it deals bonus DMG equal to the current counter value — then the counter increments by 1, but only if a *different* Champion landed the hit.

  #table(
    columns: (auto, auto, auto),
    table.header([Counter at time of hit], [Bonus DMG], [Counter after hit]),
    [0], [+0], [1 (if new Champion)],
    [1], [+1], [2 (if new Champion)],
    [2], [+2], [3 (if new Champion)],
  )
]

*Rules:*
- Only *Strike-category skills* qualify (Lance Thrust, Hook Pull, Armor Breaker, Rune Theft, Blade Tempest).
- A Strike skill *successfully hits* a piece when it is cast and resolves on that target — regardless of whether it deals HP or armor damage. Armor Breaker successfully hits even on a piece with no Armor (removes 0 Armor, counter still increments).
- A Champion that already incremented the counter *can* hit the same target again and benefits from the current counter value — but does *not* increment the counter a second time. Only a new (previously uninvolved) Champion increments it.
- You may use other skills (buffs, heals, movement) between qualifying Strikes — it does not have to be a continuous streak, just within the same turn.
- *Buff skills targeting your own pieces* (Focus Strike, Blade Call) do not count as hitting a target and do not interact with the counter.
- The combo bonus stacks with Blade Call. Each Blade Call activation boosts exactly *one* Strike skill by +1 DMG, then is spent.
- Standard Attacks (Movement Phase) do *not* count toward the combo chain — only Action Phase Strike skills.

*Example (2 Champions, target has 1 Armor):*
- Champion A casts Hook Pull on enemy Guard (counter=0 → *+0 bonus*) → 1 DMG. Armor absorbs 1 → no HP damage. Counter → 1.
- Champion B casts Lance Thrust on the *same* Guard (counter=1 → *+1 bonus*) → 2 DMG. Last Armor absorbs 1, 1 goes to HP → Guard is Injured. Counter → 2.

*Example (Armor Breaker into kill, target has 1 Armor):*
- Champion A casts Armor Breaker on enemy Champion (counter=0 → *+0 bonus*) → removes 1 Armor. Counter → 1.
- Champion B casts Lance Thrust on the *same* Champion (counter=1 → *+1 bonus*) → 2 DMG direct to HP (Armor already gone) → Champion Removed.

*Example with Blade Call:*
- Champion A casts Blade Call (boosts next Strike by +1 DMG).
- Champion A casts Lance Thrust on target (counter=0 → *+0 bonus*, +1 Blade Call) → 2 DMG. Counter → 1. Blade Call spent.
- Champion B casts Hook Pull on the *same* target (counter=1 → *+1 bonus*) → 2 DMG.
- Total: 4 DMG this turn.

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
  [Combo Bonus ⚡], [Each enemy has a hit counter (resets end of your turn). Strike skills deal +counter DMG, then counter increments if a *different* Champion hit. Standard Attacks don't count.],
  [Skill Path], [Straight line (Queen-style). Blocked by *all* pieces — ally and enemy.],
  [Skill Range], [Default Range 2. Skills with "self" = Range 0. Skills with "adjacent" = Range 1. Range modifiers apply from default.],
  [Injured Range penalty], [Injured pieces: Skill Range −1. Does not affect "self" or "adjacent" skills.],
  [Bodyguard], [Standard Attacks on Champion/King only. Guard adjacent to both tile-before-target AND defender. Guard takes the hit. Defender chooses which eligible Guard intercepts.],
  [Armor], [Max 3 per piece. Each point absorbs 1 DMG before HP, then destroyed. Does not prevent Injured status.],
  [Health], [2 HP: Normal → Injured → Removed. 1 DMG = one step. 2 DMG = skip Injured, Removed instantly.],
  [Rune income], [Collected at start of YOUR turn (not Round 1). Starts 6, then +2/+3/+4/+5 scaling.],
  [Skill Slots], [Start at 2/turn. Grow with Progression (Rounds 1–10: 2, 11–20: 3, 21–30: 4, 31+: 5).],
  [Focus Strike], [+1 Range to next skill this turn. Can boost self (→ adjacent) and adjacent (→ Range 2) skills.],
  [Blade Call], [+1 DMG to one Strike skill this turn. Stacks with Combo Bonus.],
)
]
