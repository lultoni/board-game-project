#import "/docs/test-scenarios/shared/template.typ": *
#import "/docs/test-scenarios/shared/baseline-sections.typ": *
#show: template.with(title: "Stack M — Game Length Cut")

= Stack M — Game Length Cut

_Bundled change set. Session 25 (2026-06-21). Six simultaneous changes — intentional methodology deviation per Principle 7 ("while core identity is unsettled, prefer fundamental shifts over variable tweaking")._

== Introduction

*(GAME NAME)* is a 2-player abstract-tactical board game. There is no luck — no dice, no hidden information. Every decision is made with full knowledge of the board.

*What you command:* Each player leads an army of Guards and Champions, all serving a King. Guards are fast, Champions carry powerful skills. Your King is the most important piece on the board — and the target.

*How you win:* Capture the opponent's King. The game ends immediately when a King is removed.

*How a game flows:*
Players alternate turns. On your turn you first move your pieces (and attack), then activate skills. Piece capture, skill combos, and board control build toward the moment you threaten — and take — the enemy King.

== Simple Overview

_A surface-level map of every system. No edge cases — just what each thing does._

*Rounds and turns:* The game is played in Rounds. Each Round, P1 takes a full turn, then P2. A turn has two phases: Move Phase, then Skill Phase.

*Move Phase:* You have 2 actions. Spend one to move a piece up to its speed. Each piece can only be moved once per phase.

*Move-Attack:* Instead of moving into empty space, spend an action to move your piece onto an enemy tile. The enemy takes 1 damage.

*Health:* Every piece has 2 HP: Normal → Injured → Removed. Taking 1 damage moves a piece one step along that track. Injured is purely an HP-tracker — pieces operate at full speed and Range.

*Armor:* A piece can hold up to 2 Armor. Each point absorbs 1 incoming damage *before* HP is affected, then is destroyed.

*Skill Phase:* You have actions (starting at 2 per turn). Spend one action + Money to activate an equipped skill on one of your Champions or King.

*Money:* Your currency. You earn it automatically each turn (starting at +2/turn, scaling up over time). Skills cost Money.

*Skills and Path:* Skills travel in a straight line from the caster (like a chess Queen). That path is blocked by any piece in the way. Most skills have a default range of 2 tiles.

*Bodyguard:* When a Champion or King is hit by a Move-Attack, you can choose to have an adjacent Guard take the hit instead.

*Skill Drafting:* Before the game, players alternate picking skills from a shared pool and assigning them to their Champions and King. Each piece gets 2 skills.

*Progression:* As the game goes on, your Money income and the number of actions in your Skill Phase both grow — late-game turns are more powerful because you can cast more skills.

_For full rules and edge cases, read the sections below._

#pagebreak()

== Goal

Capture the opponent's *King*. The game ends immediately when a King is removed from the board.

#section-components()

#section-skill-drafting()

== Setup

- *Board:* Use the *8×8 grid*.
- *First player:* Flip a coin (or decide verbally).
- *Piece placement:* Fixed layout — players do not choose tiles.
  - Back row (row 1/8): The King stands in the middle of the row, *offset so the two Kings are not directly opposite each other.* On one side of the King stand *2 Champions*; on the other side stand *3 Champions*. (Both players use the same layout, but each chooses independently which side gets 2 and which gets 3 — mirror or stagger as needed so the Kings are not on the same file.)
  - Second row (row 2/7): One Guard directly in front of each Champion *and* one Guard directly in front of the King — *6 Guards total.*
- *Skill Draft:* Alternating picks. P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King, then P2. Repeat until both Equip Slots on every Champion and King are filled (12 skills per player). Duplicates allowed.
- *Starting Money:* Each player starts with *6 Money*.
- P1 begins Round 1.

#section-round-structure()

#section-turn-structure()

#section-move-phase()

#section-move-attack()

#section-bodyguard()

== ⚡ CHANGED: Multi-Champion Combo Bonus

Each enemy piece has a *combo counter* (starts at 0, resets at the end of your turn).

*Triggers that tick the counter* (when a *new Champion* — one that didn't already increment this counter this turn — performs one of these on the target):
- A *Strike skill* that hits the target.
- A *Movement-causing skill* that moves the target (e.g. #sk("Tempest")'s push, #sk("Blast"), #sk("Shove"), #sk("Hook")'s pull, #sk("Swap") when it relocates an enemy).

*Bonus damage:* Any skill — Strike *or* movement-causing — that affects a target with a combo counter > 0 deals *+counter damage* to that target. This means even pure movement skills become a damage vector once the counter is built up.

*What doesn't count:* Move-Attacks. Pure buffs (#sk("Charge"), #sk("Focus"), #sk("Plate")). Pure heals. Self-movement (#sk("Dash"), #sk("Retreat")). Pushing a friendly piece.

Stacks with #sk("Charge") as before. *A single skill that both hits and moves the target ticks the counter only once* (both effects from the same skill collapse into one increment, and bonus damage is applied once).

#section-skill-phase()

== Skill System

*Path:* Skills travel in a *straight line* (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

*Blocking:* The Path is blocked by *all pieces* — ally and opponent alike. The skill cannot reach past the first piece in its path.

*Range:* The distance in tiles along the Path from caster to target. Range is measured as:
- Range 0 = self (caster's own tile)
- Range 1 = adjacent tile along the Path
- Range 2 = 2 tiles away along the Path (etc.)

*Default Range = 2.* Unless a skill explicitly names "self" or "adjacent" in its effect text. A skill with a Range modifier (e.g. "Range -1") is still a Range 2 skill with a modifier applied — it is not treated as adjacent or self.

*Self vs. adjacent:* "Self" skills (Range 0) target only the caster — they cannot target adjacent pieces by default. "Adjacent" skills (Range 1) target only neighbouring pieces — they cannot target the caster, even with a Range buff. Range buffs (e.g. #sk("Focus")) shift the targeting window outward. A Range buff on a Self skill *extends* its reach: Self + #sk("Focus") → Range 1. Range buffs do not collapse Adjacent skills inward toward Self.

*Skills that move pieces do not deal damage.* Movement-via-skill (#sk("Dash"), #sk("Swap"), #sk("Retreat")) does not count as a Move-Attack and deals no damage on arrival — you are instead stopped from moving the piece further in that direction. _(Exception: bonus damage from the Combo Bonus still applies.)_

*A Champion may use its skills multiple times in the same turn* if actions are available — including the same skill twice.

All skills cost 1 action unless noted otherwise.

#section-resource-economy()

#section-progression()

== ⚡ CHANGED: Health & Armor

Every piece has *2 HP*: Normal → Injured → Removed. There are no debuffs at any HP state — Injured is just a marker showing the piece is one hit from death.

*Armor:* *Max 2 per piece.* Each point absorbs 1 damage *before* HP is affected, then is destroyed.

#pagebreak()

== Skill Reference

#skill-table(
  columns: (auto, auto, auto, auto, 1fr),
  table.header([], [Cat.], [Name], [Cost], [Effect]),
  skill-icon("lance_thrust"), [Strike], [Lance], [2], [Target within Range-1 takes 1 damage],
  skill-icon("hook_pull"), [Strike], [Hook], [3], [Target takes 1 damage, pulled 1 tile toward caster along the Path],
  skill-icon("armor_breaker"), [Strike], [Break], [2], [Remove 1 Armor from target. _(Note: does not deal "HP-Damage" unless boosted by Charge)_],
  skill-icon("rune_theft"), [Strike], [Steal], [4], [Target takes 1 damage. Steal 1 Money from opponent.],
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

#section-quick-reference(overrides: (
  "Armor": [*Max 2* per piece. Each point absorbs 1 damage before HP, then is destroyed.],
  "Injured Range penalty": [None — Injured pieces operate at full speed and full Range.],
  "Guard speed": [Normal: 2 tiles. Injured: 2 tiles (no penalty).],
  "Champion / King speed": [1 tile (Normal or Injured).],
  "Combo Bonus": [Each enemy has a combo counter (resets end of your turn). A new Champion that *hits with a Strike* OR *moves the target with a movement-causing skill* ticks the counter +1. *Any skill* (Strike or movement-causing) that affects a target with counter > 0 deals *+counter bonus damage*. Move-Attacks excluded.],
), extra-rows: (
  ("Board", [*8×8 grid.* No terrain.]),
  ("Draw conditions", [None. Game ends only when a King is captured.]),
  ("Steal cost", [4 Money (both Modes).]),
))

/*
================================================================================
FACILITATOR NOTES — NOT FOR PLAYERS
================================================================================
Hidden from PDF by Typst block comment. Kept in source for the facilitator
running the playtest. If you need to print these, move them out of the comment
block before building.

== Facilitator Notes

_This page is for the playtest facilitator — not for the players. The rules above are the rules of the game; the notes below explain what makes Stack M a bundled experiment and what to watch._

=== What is bundled

Stack M intentionally bundles six simultaneous changes against the baseline:

| Change | Targets |
| Board: 10×10 → 8×8 | Tile economy (occupation up); per-piece move-option count down; pieces always close to each other. |
| Armor cap: 3 → 2 | Chassis volume (OQ-11 cross-pole confirmed); "easy progressive escape" stalling pattern. |
| Injured State: penalties removed | Injured remains as HP-tracker (2hp → 1hp → removed). Speed-cap, Range-1, and self/adjacent carve-out are gone. Targets damage-vs-heal asymmetry. |
| Draw conditions: removed | Both baseline draw conditions removed entirely. Not replaced. Targets Principle 8 (single climax → natural end). |
| Steal cost: 3 → 4 (both Modes) | Money economy faucet rebalance; OQ-34 (Mode B dominant) addressed. |
| Combo Bonus: also triggered by movement-causing skills + bonus damage applies to any skill | Engagement becomes more rewarding vs stalling. Damage strategies unlocked without Strike skills. Move-Attacks still excluded from the trigger. |

=== What we're watching

- Game length: total rounds + wall-clock minutes. Target: 30-60 min.
- Game shape (Principle 8): did the game peak once and end naturally, or did it peak and limp?
- Stalling pattern: did the mid-game Armor-stack cluster (P4 R15-R21 pattern) appear?
- Combo widening reception: was the new movement-counter trigger intuitive? Did dual-effect skills (Tempest, Hook) feel overpowered?
- Movement-as-damage: did players exploit movement skills to deal bonus damage on counter-loaded targets? Did it feel cool or confusing?
- Injured-as-tracker: did Injured pieces feel structureless without the penalties, or did the cleaner state-machine feel better? (OQ-57 carry-over.)
- No-draws environment: did games actually end, or did some go very long?
- Steal at cost 4: still must-pick? Or finally tunable?
- 8×8 cramping: did the smaller board feel right, or claustrophobic?
- Felt-PI (OQ-64) revisit: per-piece move-option count goes down on 8×8; did the game feel more graspable mid-turn?
- Cognitive load (OQ-60): with Injured penalties gone and the Armor ceiling lower, does the per-turn computation feel lighter?

=== Decision routing on result

1. Game length 30-60 min, single-climax shape, no stalling cluster → Stack M accepted into baseline. Follow-up isolation stacks (piece count, unified actions, 6×6 board) become candidates.
2. Game length right, but combo widening dominates / Tempest broken → roll back movement-counter trigger only. Keep the other five changes.
3. Game length still too long, no stalling → next stack is piece-count cut (Stack K) or 6×6 board.
4. Game length right, but cleverness feels gone (pure aggression dominates) → roll back combo widening OR Steal cost increase, depending on which axis feels off.
5. Game length right, but Injured-no-penalty makes pieces feel disposable → roll back Injured change only (re-enable speed cap + Range-1).
6. No-draws causes infinite games in some positions → restore one draw condition (only-Kings-remain is the cheaper one to keep).
7. Bundle is uninterpretable (too many things changed feel) → individual rollback per axis, sequentially, in next stacks.

=== Why bundled (methodology deviation, documented)

The Incremental Testing Methodology in CLAUDE.md says: "Never propose changing multiple interacting systems at once." Stack M deliberately violates this, and that deviation is documented rather than hidden, per the methodology's "Document which stack produced which result" rule.

Justification:
- Principle 7 (Session 23) explicitly says: "While the core identity is unsettled, prefer fundamental shifts over variable tweaking." Stack M is a fundamental shift; the unsettled-core condition holds.
- Six sequential stacks would take 6+ playtest sessions to clear what Stack M clears in one.
- The bundle's components have been independently validated as candidate fixes across P3-P5. Bundle is a coordinated deployment, not exploratory speculation.
- If the bundle works, follow-up isolation stacks (piece count, unified actions, 6×6 board) will narrow further. We do not lose the ability to attribute — we defer it.
- If the bundle fails, we know the bundle as a whole missed, and rollback proceeds one axis at a time per the routing above.

The methodology recovers on the next stack, not on this one.

Stack M is Active as of Session 25 (2026-06-21). When this stack lands or rolls back, update TESTING_PLAN.typ Active section, STATUS.md current focus, and mechanics-evaluated.md with the Methodology row.

================================================================================
*/
