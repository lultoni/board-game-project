# (GAME NAME) - Canonical Ruleset

> **This file is the canonical, authoritative ruleset. On any conflict, this file wins.**

---

## Introduction

*(GAME NAME)* is a 2-player abstract-tactical board game. There is no luck, no dice, no hidden information. Every decision is made with full knowledge of the board.

**What you command:** Each player leads an army of Guards and Champions, all serving a King. Guards are fast; Champions carry powerful skills. Your King is the most important piece on the board - and the target.

**How you win:** Capture the opponent's King. The game ends immediately when a King is removed.

**How a game flows:** Players alternate turns. On your turn you first move your pieces (and attack), then activate skills. Piece capture, skill combos, and board control build toward the moment you threaten, and take, the enemy King.

---

## Goal

Capture the opponent's **King**. The game ends immediately when a King is removed from the board. There are no draw conditions.

---

## Components

- **Board:** 8×8 grid.
- **Per player:** 1 King · 5 Champions · 6 Guards.
- **Champions and the King have 2 Equip Slots each.** Skills are equipped into these slots. **Guards have no Equip Slots and carry no skills.**

---

## Setup

- **Board:** the **8×8 grid**.
- **First player:** flip a coin (or decide verbally).
- **Piece placement:**
  - **Back row (row 1/8):** the King stands in the middle of the row, *offset so the two Kings are not directly opposite each other.* On one side of the King stand 2 Champions; on the other side stand 3 Champions. In the end there will be 2 free squares left and right of the pieces.
  - **Second row (row 2/7):** one Guard directly in front of each Champion / the King.
- **Skill Draft:** alternating picks (see Skill Drafting).
- **Starting Money:** each player starts with **6 Money**.
- P1 begins Round 1.

---

## Round Structure

A **Round** = P1's Turn + P2's Turn.

At the **start of each player's turn** (before they do anything), that player collects their Money income. *(Exception: Round 1 - players begin with their starting Money and collect nothing before their first turn.)*

---

## Turn Structure

Each **Turn** has two phases, in order:

- **Move Phase** - spend actions to move pieces (and attack).
- **Skill Phase** - spend actions to activate skills.

You may use 0 actions in either phase.

---

## Move Phase

You have **2 actions** per turn. Spend 1 action to move one piece - either **into empty space** (normal movement) or **into an enemy tile** (a Move-Attack - see next section). **Each piece may only be moved once per Move Phase.**

| Piece | Normal speed |
|-------|-------------|
| Guard | 2 tiles |
| Champion / King | 1 tile |

**Free pathing:** a piece may move in any direction (horizontal, vertical, diagonal) taking any route up to its speed in tiles. The route does not have to be a straight line. This means you can move around pieces in a zig-zag line or change your approach for a Move-Attack (see section Move-Attack) as this might be relevant for the Bodyguard Rule (see section Bodyguard Rule). The free pathing however does not allow for jumping over pices.

---

## Health & Armor

All pieces have 2 HP and a maximum of 2 points of Armor. Each armor point absorbs 1 damage, then is destroyed. Armor resolves before HP Damage. A piece can have Armor but not be at full health.

---

## Move-Attack

A **Move-Attack** is a Move that ends on an enemy tile. Spend 1 action to move your piece **onto a tile occupied by an enemy piece.** You only have 1 Move-Attack per Turn, so after performing one, you can only move your second piece onto a free square.

- Deal **1 damage** to the enemy.
- **If the enemy is removed:** your piece occupies the tile (the attack consumed the move; remaining unused movement tiles do not carry over).
- **If the enemy survives:** your piece **stops on the tile immediately before the target.** This applies to *every* attacker - a Guard with speed 2 ends up having moved only 1 tile; a Champion or King with speed 1 does not move at all. Either way the damage is dealt. *(If multiple paths reach the target, you choose which one - important for the Bodyguard Rule.)*

You may attack with one action and move a different piece with the other action in the same turn.

*Move-Attacks are how pieces deal damage with movement alone. Skills (activated in the Skill Phase) are the other way pieces affect each other, and use different rules (Path, Range, Money).*

---

## Bodyguard Rule

When you make a **Move-Attack** against an opponent's Champion or King, the defender may choose to have a Guard intercept - **if** a friendly Guard is on a tile adjacent to **both** the tile immediately before the target (along the attack path) **and** the defending piece.

**Interception:**
- Defender announces a Guard to intercept.
- The Guard takes the damage instead of the original target.
- When the attack was made from a range 2 square the attacker moves only **1 tile** toward the target, no matter if the bodyguard dies or not.

**Interception is optional.** The defender may decline even if a Guard is eligible, and may choose which Guard intercepts if multiple are eligible.

Only Move-Attacks can be intercepted. **Skills always hit directly.**

---

## Skill Phase

You have **2 actions** per turn at the start (grows with Progression).

Spend 1 action to activate one equipped skill on one of your Champions or King:
- Announce the skill and the target.
- Pay the skill's Money cost.
- Apply the skill's effect.

The same Champion can activate multiple skills (including the same one) in one turn if you have actions remaining and the money.

---

## Skill System

**Path:** skills travel in a **straight line** (horizontal, vertical or diagonal) from the caster, like a chess Queen.

**Blocking:** the Path is blocked by **all pieces** (ally and opponent alike). The skill cannot reach past the first piece in its path.

**Range:** the distance in tiles along the Path from caster to target:
- Range 0 = self (caster's own tile)
- Range 1 = adjacent tile along the Path
- Range 2 = 2 tiles away along the Path (etc.)

**Default Range = 2**, unless a skill explicitly names "self" or "adjacent." A skill with a Range modifier (e.g. "Range −1") is still a Range 2 skill with a modifier applied. It is not treated as adjacent or self.

**Self vs. adjacent:** "Self" skills (Range 0) target only the caster. "Adjacent" skills (Range 1) target only neighbouring pieces - never the caster, even with a Range buff. Range buffs (e.g. Focus) shift the targeting window outward: Self + Focus → Range 1. Range buffs do not collapse Adjacent skills inward toward Self.

**Skills that move pieces do not deal damage.** Movement-via-skill (Dash, Swap, Retreat) does not count as a Move-Attack and deals no damage on arrival - you are instead stopped from moving the piece further in that direction. *(Exception: bonus damage from the Combo Bonus still applies.)*

When a piece casts a **Strike skill**, after the skill's normal damage + effect fully resolve, the **caster moves 1 tile toward the target along the cast direction.** Single uniform resolution:
> 1. Resolve the skill's damage + effect first.
> 2. Attempt to move the caster 1 tile along the cast direction toward the (former) target tile using the normal movement resolution.

---

## Multi-Champion Combo Bonus

Each enemy piece has a **combo counter** (starts at 0, resets at the end of your turn).

**Triggers that tick the counter** (when a **new Champion** (one that didn't already increment this counter this turn) performs one of these on the target):
- A **Strike skill** that hits the target.
- A **Movement-causing skill** that moves the target (e.g. Tempest's push, Blast, Shove, Hook's pull, Swap when it relocates an enemy).

**Bonus damage:** any skill - Strike *or* movement-causing - that affects a target with a combo counter > 0 deals damage equal to the counter-1 to that target. Even pure movement skills become a potential damage source once the counter is built up, but hitting a piece with the same champ twice (without any other champ hitting the same piece) does not grant bonus damage due to the counter being at 1 and only counter-1 damage being dealt.

**A single skill that both hits and moves the target ticks the counter only once** (both effects collapse into one increment; bonus damage applied once).

**What doesn't count:** Move-Attacks, pure buffs, pure heals, self-movement, pushing a friendly piece.

---

## Money

**Starting Money:** 6 per player.

Money income is collected at the **start of each player's own turn** and there is **no Money cap.**

| Round | Income per player turn |
|-------|------------------------|
| 1 | 0 (starting Money only) |
| 2–4 | +2 |
| 5–9 | +3 |
| 10–14 | +4 |
| 15+ | +5 (+1 every 5 rounds) |

---

## Progression

| Round | Actions per Skill Phase |
|-------|-------------------------|
| 1–10 | 2 |
| 11–20 | 3 |
| 21–30 | 4 |
| 31+ | 5 (+1 every 10 rounds) |

As the game goes on, both Money income and Skill-Phase action count grow - late-game turns are more powerful because you can cast more skills.

---

## Skill Drafting

- Lay out all available skills face-up as a shared pool.
- **Alternating draft:** P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King → P2 picks 2 and assigns freely → repeat.
- Continue until both Equip Slots on every Champion and the King are filled (**12 skills per player**).
- Both players draft from the same pool and you can draft the same skill as often as you wish **as long as you do not put the same skill on one champ/king twice**.

---

## Skill Reference

| Cat. | Name | Cost | Effect |
|------|------|------|--------|
| Strike | Lance | 2 | Target within Range−1 takes 1 damage. |
| Strike | Hook | 3 | Target takes 1 damage, pulled 1 tile toward caster along the Path. |
| Strike | Break | 2 | Remove 1 Armor from target. *(Deals no HP damage unless boosted by Charge.)* |
| Strike | Steal | 4 | Target takes 1 damage. Steal 1 Money from opponent. |
| Strike | Tempest | 4 | Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. The caster is not affected. |
| Shield | Shield | 2 | Self: gain +1 Armor. |
| Shield | Heal | 3 | Remove Injured from one adjacent ally. |
| Shield | Plate | 3 | Adjacent ally gains +1 Armor. |
| Move | Dash | 3 | Self: move up to 2 tiles along the Path. |
| Move | Blast | 2 | Push target enemy 1 tile directly away from caster. |
| Move | Shove | 3 | Push target enemy 1 tile in any direction (caster chooses). **Range+1.** |
| Move | Swap | 4 | Swap position with an allied piece. Requires unobstructed Path. |
| Move | Retreat | 4 | Self: move along the Path to land adjacent to one of your Guards. **Range+1.** |
| Mystic | Focus | 2 | The next non-mystic skill used by *any of your pieces* this turn gains +1 Range. *(Can boost self and adjacent skills - Range 0→1, Range 1→2.)* **Move skills:** the caster chooses, when activating the Move skill, whether the +1 applies to its *activation range* or its *effect range* - not both. |
| Mystic | Charge | 3 | The next Strike skill used by *any of your pieces* this turn deals +1 damage. |
