# (GAME NAME) — Canonical Ruleset

> **This file is the canonical, authoritative ruleset. On any conflict, this file wins.**
>
> - The in-game **Help page** (`game/frontend/src/lib/i18n/*.json`, keys `help.rules.*`) is a *derived, player-facing summary* — keep it in sync with this file; this file is authoritative.
> - **Detailed engine behaviour** lives in `game/crates/core_engine` — if the engine and this file disagree, that is a bug to reconcile (usually the engine is right about mechanics, this file is right about intent).
> - The DB (`design/design.db`) owns *design knowledge* (why rules are the way they are: OQs, ADRs, stacks, playtests). It points here for the rules themselves and does not restate them.
>
> **Provenance:** This is Stack M (the game-length-cut baseline, provisionally landed at P6 / Session 43) **plus** the three staged Stack N changes (Session 45, awaiting P7). Rules that are staged-but-not-yet-playtest-confirmed are marked **⧗ Stack N — staged S45, awaiting P7**. Everything else is settled baseline.
>
> *Last synced from source: `design/_archive/test-scenarios/shared/baseline-sections.typ` + `.../stack-m-game-length-cut/stack-m-game-length-cut.typ` + `stacks.body` for `stack-m` / `stack-n`. Session 45 (2026-07-11).*

---

## Introduction

*(GAME NAME)* is a 2-player abstract-tactical board game. There is no luck — no dice, no hidden information. Every decision is made with full knowledge of the board.

**What you command:** Each player leads an army of Guards and Champions, all serving a King. Guards are fast; Champions carry powerful skills. Your King is the most important piece on the board — and the target.

**How you win:** Capture the opponent's King. The game ends immediately when a King is removed.

**How a game flows:** Players alternate turns. On your turn you first move your pieces (and attack), then activate skills. Piece capture, skill combos, and board control build toward the moment you threaten — and take — the enemy King.

**What makes it deep:** Skills cost Money. Money is scarce. You will always want to do more than you can. That tension — between movement, attacks, and skill combos — is the game.

---

## Goal

Capture the opponent's **King**. The game ends immediately when a King is removed from the board.

**Draw conditions: none.** (Removed in Stack M.) The game ends only when a King is captured.

---

## Components

- **Per player:** 1 King · 5 Champions · 6 Guards.
- **Champions and the King have 2 Equip Slots each.** Skills are equipped into these slots. **Guards have no Equip Slots and carry no skills.**
- **Board:** 8×8 grid. No terrain.

---

## Setup

- **Board:** the **8×8 grid**.
- **First player:** flip a coin (or decide verbally).
- **Piece placement (fixed layout — players do not choose tiles):**
  - **Back row (row 1/8):** the King stands in the middle of the row, *offset so the two Kings are not directly opposite each other.* On one side of the King stand **2 Champions**; on the other side stand **3 Champions**. Both players use the same layout, but each independently chooses which side gets 2 and which gets 3 — mirror or stagger so the Kings are not on the same file.
  - **Second row (row 2/7):** one Guard directly in front of each Champion **and** one Guard directly in front of the King — **6 Guards total.**
- **Skill Draft:** alternating picks (see Skill Drafting).
- **Starting Money:** each player starts with **6 Money**.
- P1 begins Round 1.

---

## Round Structure

A **Round** = P1's Turn + P2's Turn.

At the **start of each player's turn** (before they do anything), that player collects their Money income. *(Exception: Round 1 — players begin with their starting Money and collect nothing before their first turn.)*

---

## Turn Structure

Each **Turn** has two phases, in order:

- **Move Phase** — spend actions to move pieces (and attack).
- **Skill Phase** — spend actions to activate skills.

You may use 0 actions in either phase.

---

## Move Phase

You have **2 actions** per turn. Spend 1 action to move one piece — either **into empty space** (normal movement) or **into an enemy tile** (a Move-Attack — see next section). **Each piece may only be moved once per Move Phase.**

| Piece | Normal speed | Injured speed |
|-------|-------------|---------------|
| Guard | 2 tiles | 2 tiles (no penalty) |
| Champion / King | 1 tile | 1 tile |

**Free pathing:** a piece may move in any direction — horizontal, vertical, diagonal — taking any route up to its speed in tiles. The route does not have to be a straight line.

**Pieces block movement:** a moving piece cannot pass through any other piece, ally or opponent.
- *Exception:* a piece may go around a blocking piece if total tiles moved stays within speed (e.g. a Guard using 2 diagonal moves to go around a piece directly in the way).

> **⧗ Stack N — staged S45, awaiting P7 — Max 1 move-attack per turn.** You may still move both pieces in the Move Phase, but **at most one of your two actions may be a Move-Attack.** (Normal, non-attacking moves are unrestricted.) *Rationale: removes the "2 move-attacks delete a 2-HP piece from Round 1" vector that made Guards evaporate early. See `stacks.body` for `stack-n`.*

---

## Move-Attack

A **Move-Attack** is a Move that ends on an enemy tile. Spend 1 action to move your piece **onto a tile occupied by an enemy piece.**

- Deal **1 damage** to the enemy.
- **If the enemy is removed:** your piece occupies the tile (the attack consumed the move; remaining unused movement tiles do not carry over).
- **If the enemy survives** (Armor absorbed the damage): your piece **stops on the tile immediately before the target.** This applies to *every* attacker — a Guard with speed 2 ends up having moved only 1 tile; a Champion or King with speed 1 does not move at all. Either way the damage is dealt. *(If multiple paths reach the target, you choose which one — important for the Bodyguard Rule.)*

You may attack with one action and move a different piece with the other action in the same turn. *(But see the Stack N one-move-attack-per-turn limit above.)*

*Move-Attacks are how pieces deal damage with movement alone. Skills — activated in the Skill Phase — are the other way pieces affect each other, and use different rules (Path, Range, Money).*

---

## Multi-Champion Combo Bonus

Each enemy piece has a **combo counter** (starts at 0, resets at the end of your turn).

**Triggers that tick the counter** (when a **new Champion** — one that didn't already increment this counter this turn — performs one of these on the target):
- A **Strike skill** that hits the target.
- A **Movement-causing skill** that moves the target (e.g. Tempest's push, Blast, Shove, Hook's pull, Swap when it relocates an enemy).

**Bonus damage:** any skill — Strike *or* movement-causing — that affects a target with a combo counter > 0 deals **+counter damage** to that target. Even pure movement skills become a damage vector once the counter is built up.

**What doesn't count:** Move-Attacks. Pure buffs (Charge, Focus, Plate). Pure heals. Self-movement (Dash, Retreat). Pushing a friendly piece.

Stacks with Charge. **A single skill that both hits and moves the target ticks the counter only once** (both effects collapse into one increment; bonus damage applied once).

---

## Skill Phase

You have **2 actions** per turn at the start (grows with Progression).

Spend 1 action to activate one equipped skill on one of your Champions or King:
- Announce the skill and the target.
- Pay the skill's Money cost.
- Apply the skill's effect.

The same Champion can activate multiple skills (including the same one) in one turn if you have actions remaining.

---

## Skill System

**Path:** skills travel in a **straight line** (horizontal, vertical, or diagonal) from the caster, like a chess Queen.

**Blocking:** the Path is blocked by **all pieces** — ally and opponent alike. The skill cannot reach past the first piece in its path.

**Range:** the distance in tiles along the Path from caster to target:
- Range 0 = self (caster's own tile)
- Range 1 = adjacent tile along the Path
- Range 2 = 2 tiles away along the Path (etc.)

**Default Range = 2**, unless a skill explicitly names "self" or "adjacent." A skill with a Range modifier (e.g. "Range −1") is still a Range 2 skill with a modifier applied — it is not treated as adjacent or self.

**Self vs. adjacent:** "Self" skills (Range 0) target only the caster. "Adjacent" skills (Range 1) target only neighbouring pieces — never the caster, even with a Range buff. Range buffs (e.g. Focus) shift the targeting window outward: Self + Focus → Range 1. Range buffs do not collapse Adjacent skills inward toward Self.

**No Injured Range penalty.** (Removed in Stack M — Injured is a pure HP tracker; pieces operate at full Range regardless.)

**Skills that move pieces do not deal damage.** Movement-via-skill (Dash, Swap, Retreat) does not count as a Move-Attack and deals no damage on arrival — you are instead stopped from moving the piece further in that direction. *(Exception: bonus damage from the Combo Bonus still applies.)*

**A Champion may use its skills multiple times in the same turn** if actions are available — including the same skill twice.

All skills cost 1 action unless noted otherwise.

> **⧗ Stack N — staged S45, awaiting P7 — Strike-moves-caster.** When a piece casts a **Strike skill**, after the skill's normal damage + effect fully resolve, the **caster moves 1 tile toward the target along the cast direction.** Single uniform resolution:
> 1. Resolve the skill's damage + effect first (Tempest pushes, Hook pulls, Steal takes money, etc. — all unchanged).
> 2. Then attempt to move the caster 1 tile along the cast direction toward the (former) target tile, using the normal movement resolution.
> 3. The move happens **only if the destination tile is empty.** If occupied → no move. (No walls exist; the only other no-move case is a board edge.)
>
> Consequences: a point-blank (adjacent) strike that does **not** kill → destination still occupied → **no move**. A point-blank strike that **kills** → the tile is now vacated → caster **steps onto it** (chess-like: you take the square). A Hook that would pull the target onto the caster's own tile → target doesn't move (occupied) and caster doesn't move — unless the target died, freeing the tile.
>
> **Scope: Strike skills only** this cycle (Move/Shield/Mystic skills do not move the caster). *Rationale: restores chess-like reciprocity — ranged aggression now costs position. See `stacks.body` for `stack-n`.*

---

## Money

**Starting Money:** 6 per player.

Money income is collected at the **start of each player's own turn:**

| Round | Income per player turn |
|-------|------------------------|
| 1 | 0 (starting Money only) |
| 2–4 | +2 |
| 5–9 | +3 |
| 10–14 | +4 |
| 15+ | +5 (+1 every 5 rounds) |

**No Money cap.**

---

## Health & Armor

**All pieces have 2 HP:** Normal → Injured → Removed.

| State | HP | Effect |
|-------|-----|--------|
| Normal | 2 | No penalty |
| Injured | 1 | **No penalty** — pure HP tracker (one hit from death). Full speed, full Range. |
| Removed | 0 | Piece leaves the board permanently. |

**Damage:**
- 1 damage to Normal → Injured.
- 1 damage to Injured → Removed.
- 2 damage to Normal → Removed instantly (Injured state is skipped).

**Armor:** **Max 2 points per piece.** Each absorbs 1 damage, then is destroyed. Resolves before HP damage. *(Injured no longer carries a penalty, so Armor's only job is absorbing damage.)*

---

## Bodyguard Rule

When you make a **Move-Attack** against an opponent's Champion or King, the defender may choose to have a Guard intercept — **if** a friendly Guard is on a tile adjacent to **both** the tile immediately before the target (along the attack path) **and** the defending piece.

**Interception:**
- Defender announces a Guard to intercept.
- The Guard takes the damage instead of the original target.
- The attacker moves **1 tile** toward the target (stops on the tile immediately before the target).

**Interception is optional.** The defender may decline even if a Guard is eligible, and may choose which Guard intercepts if multiple are eligible.

Only Move-Attacks can be intercepted. **Skills always hit directly.**

---

## Skill Drafting

- Lay out all available skills face-up as a shared pool.
- **Alternating draft:** P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King → P2 picks 2 and assigns freely → repeat.
- Continue until both Equip Slots on every Champion and the King are filled (**12 skills per player**).
- Both players draft from the same pool. **Duplicates allowed.**

---

## Progression

| Round | Actions per Skill Phase |
|-------|-------------------------|
| 1–10 | 2 |
| 11–20 | 3 |
| 21–30 | 4 |
| 31+ | 5 (+1 every 10 rounds) |

As the game goes on, both Money income and Skill-Phase action count grow — late-game turns are more powerful because you can cast more skills.

---

## Skill Reference

| Cat. | Name | Cost | Effect |
|------|------|------|--------|
| Strike | Lance | 2 | Target within Range−1 takes 1 damage. |
| Strike | Hook | 3 | Target takes 1 damage, pulled 1 tile toward caster along the Path. |
| Strike | Break | 2 | Remove 1 Armor from target. *(Deals no HP damage unless boosted by Charge.)* |
| Strike | Steal | **4** | Target takes 1 damage. Steal 1 Money from opponent. *(Stack M: cost 3→4.)* |
| Strike | Tempest | 4 | Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. The caster is not affected. |
| Shield | Shield | 2 | Self: gain +1 Armor. |
| Shield | Heal | 3 | Remove Injured from one adjacent ally. |
| Shield | Plate | 3 | Adjacent ally gains +1 Armor. |
| Move | Dash | 3 | Self: move up to 2 tiles along the Path. |
| Move | Blast | 2 | Push target enemy 1 tile directly away from caster. |
| Move | Shove | 3 | Push target enemy 1 tile in any direction (caster chooses). **Range+1.** |
| Move | Swap | 4 | Swap position with an allied piece. Requires unobstructed Path. |
| Move | Retreat | 4 | Self: move along the Path to land adjacent to one of your Guards. **Range+1.** |
| Mystic | Focus | **2** | The next skill used by *any of your pieces* this turn gains +1 Range. *(Can boost self and adjacent skills — Range 0→1, Range 1→2.)* **Move skills:** the caster chooses, when activating the Move skill, whether the +1 applies to its *activation range* or its *effect range* — not both. **⧗ Stack N — staged S45, awaiting P7: cost 1→2.** |
| Mystic | Charge | 3 | One Strike skill used by *any of your pieces* this turn deals +1 damage. |

---

## Quick Reference

| Concept | Rule |
|---------|------|
| Board | **8×8 grid.** No terrain. |
| Movement | Free pathing, ≤ speed in tiles, cannot pass through pieces. Each piece once per Move Phase. |
| Guard speed | Normal: 2 tiles. Injured: 2 tiles (no penalty). |
| Champion / King speed | 1 tile (Normal or Injured). |
| Move-Attack | Move onto enemy tile (1 action). Deals 1 damage. Attacker stops before target if target survives. **⧗ Stack N: max 1 per turn.** |
| Path | Straight line (Queen-style). Blocked by *all* pieces — ally and enemy. |
| Range | Default 2. "self" = Range 0. "adjacent" = Range 1. Range modifiers apply from default. |
| Injured Range penalty | **None** — Injured pieces operate at full Range. |
| Bodyguard | Move-Attacks on Champion/King only. Guard must be adjacent to both tile-before-target AND defender. Guard takes the hit. Defender chooses which eligible Guard intercepts. |
| Armor | **Max 2** per piece. Each absorbs 1 damage before HP, then is destroyed. |
| Health | 2 HP: Normal → Injured → Removed. Injured = pure HP tracker (no penalty). 2 damage skips Injured. |
| Money income | Collected at start of YOUR turn (not Round 1). Starts 6, then +2/+3/+4/+5 scaling. No cap. |
| Skill Phase actions | Start 2/turn. Grow: R1–10: 2, 11–20: 3, 21–30: 4, 31+: 5. |
| Steal cost | **4 Money.** *(Stack M: 3→4.)* |
| Focus | +1 Range to next skill this turn. Boosts self (→adjacent) and adjacent (→Range 2). On Move skills: activation OR effect range. **⧗ Stack N: cost 2.** |
| Charge | +1 damage to one Strike skill this turn. Stacks with Combo Bonus. |
| Combo Bonus | Each enemy has a combo counter (resets end of your turn). A *new Champion* that hits with a Strike OR moves the target with a movement-causing skill ticks +1. *Any* skill affecting a target with counter > 0 deals +counter bonus damage. Move-Attacks excluded. |
| Strike-moves-caster | **⧗ Stack N — staged:** after a Strike resolves, caster steps 1 tile toward target if that tile is now empty. |
| Draw conditions | **None.** Game ends only when a King is captured. |

---

## Change provenance (what came from where)

- **Stack M — Game Length Cut** (Session 25, provisionally landed P6/S43): board 10×10→8×8; Armor cap 3→2; Injured penalties removed (pure HP tracker); draw conditions removed; Steal cost 3→4; Combo Bonus widened to movement-causing skills (trigger + bonus). Full rationale: `SELECT body FROM stacks WHERE id='stack-m';`.
- **Stack N — Lethality & Standoff Pass** (Session 45, **staged, awaiting P7**): Focus cost 1→2; max 1 move-attack/turn; strike-moves-caster. Reserve lever (not in rules): forward-Guard partial skill-immunity (`bp-forward-guard-partial-skill-immunity`). Full rationale: `SELECT body FROM stacks WHERE id='stack-n';`.
