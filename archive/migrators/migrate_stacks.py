#!/usr/bin/env python3
"""Migrate stacks from TESTING_PLAN.typ + per-stack .typ files into stacks table.

Stack M gets its full rule substance in the body (it becomes the Rust prototype's
foundation). Other stacks get a structural summary + cross-refs.
"""

import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

# Stack M full rule substance (verbatim from stack-m-game-length-cut.typ — markdown form).
# This is the load-bearing payload — Rust prototype reads stacks.body for Stack M.
STACK_M_BODY = """*Bundled change set. Session 25 (2026-06-21). Six simultaneous changes — intentional methodology deviation per Principle 7 ("while core identity is unsettled, prefer fundamental shifts over variable tweaking").*

## Goal

Capture the opponent's **King**. The game ends immediately when a King is removed from the board.

## Setup

- **Board:** 8×8 grid. No terrain.
- **First player:** Flip a coin (or decide verbally).
- **Piece placement:** Fixed layout — players do not choose tiles.
  - Back row (row 1/8): The King stands in the middle of the row, *offset so the two Kings are not directly opposite each other.* On one side of the King stand 2 Champions; on the other side stand 3 Champions. (Both players use the same layout, but each chooses independently which side gets 2 and which gets 3 — mirror or stagger as needed so the Kings are not on the same file.)
  - Second row (row 2/7): One Guard directly in front of each Champion *and* one Guard directly in front of the King — 6 Guards total.
- **Skill Draft:** Alternating picks. P1 picks 2 skills from the pool and assigns them freely to any of their Champions or King, then P2. Repeat until both Equip Slots on every Champion and King are filled (12 skills per player). Duplicates allowed.
- **Starting Money:** Each player starts with 6 Money.
- P1 begins Round 1.

## Components (per player)

- 1 King (carries 2 skills)
- 5 Champions (carry 2 skills each)
- 6 Guards (no skills)

## Round and Turn Structure

The game is played in Rounds. Each Round: P1 takes a full turn, then P2. A turn has two phases:
1. **Move Phase** — 2 actions. Spend one to move a piece up to its speed. Each piece can only be moved once per phase.
2. **Skill Phase** — actions scaling over time (see Progression). Spend one action + Money to activate an equipped skill.

## Move Phase

You have 2 actions. Each action moves one piece up to its speed.
- **Guard speed:** 2 tiles per move.
- **Champion / King speed:** 1 tile per move.
- Each piece can only be moved once per Move Phase.

## Move-Attack

Instead of moving into an empty tile, spend a move action to move your piece onto an enemy tile. The enemy takes 1 damage. Your piece *does not* enter the enemy's tile — the enemy is on the tile until Armor/HP resolves the hit. Movement skills do not Move-Attack.

## Bodyguard

When a Champion or King is hit by a Move-Attack, you may have an adjacent friendly Guard take the hit instead. The Guard takes the damage; the original target is unhurt. Only triggers on Move-Attacks, not skill damage.

## Multi-Champion Combo Bonus (CHANGED in Stack M)

Each enemy piece has a **combo counter** (starts at 0, resets at the end of your turn).

**Triggers that tick the counter** (when a new Champion — one that didn't already increment this counter this turn — performs one of these on the target):
- A **Strike skill** that hits the target.
- A **Movement-causing skill** that moves the target (e.g. Tempest's push, Blast, Shove, Hook's pull, Swap when it relocates an enemy).

**Bonus damage:** Any skill — Strike or movement-causing — that affects a target with a combo counter > 0 deals **+counter damage** to that target. This means even pure movement skills become a damage vector once the counter is built up.

**What doesn't count:** Move-Attacks. Pure buffs (Charge, Focus, Plate). Pure heals. Self-movement (Dash, Retreat). Pushing a friendly piece.

Stacks with Charge as before. A single skill that both hits and moves the target ticks the counter only once.

## Skill Phase

Spend actions (starting at 2 per turn, see Progression) to activate equipped skills. Each skill costs 1 action + its Money cost. A Champion may use its skills multiple times in the same turn if actions are available — including the same skill twice.

## Skill System

- **Path:** Skills travel in a straight line (horizontal, vertical, or diagonal) from the caster, like a chess Queen.
- **Blocking:** The Path is blocked by all pieces — ally and opponent alike. The skill cannot reach past the first piece in its path.
- **Range:** Distance in tiles along the Path. Range 0 = self; Range 1 = adjacent; Range 2 = 2 tiles away (default).
- **Default Range = 2** unless a skill explicitly names "self" or "adjacent" in its effect text.
- **Self vs. adjacent:** "Self" skills (Range 0) target only the caster. "Adjacent" skills (Range 1) target only neighbouring pieces. Range buffs on Self skills extend reach (Self + Focus → Range 1). Range buffs do not collapse Adjacent skills inward.
- **Movement skills deal no damage** by themselves. (Exception: Combo Bonus damage still applies.)

## Resource Economy

- **Starting Money:** 6.
- **Per-turn income:** +2/turn baseline.
- **Scaling:** +1 to per-turn income every 5 rounds.

## Progression

- **Skill Phase actions** start at 2 per turn, scaling up over the game (matching the existing progression curve).

## Health & Armor (CHANGED in Stack M)

Every piece has 2 HP: Normal → Injured → Removed. **There are no debuffs at any HP state — Injured is just a marker showing the piece is one hit from death.**

**Armor: Max 2 per piece.** Each point absorbs 1 damage *before* HP is affected, then is destroyed.

## Skill Reference (15 skills)

| Cat. | Name | Cost | Effect |
|---|---|---|---|
| Strike | Lance | 2 | Target within Range-1 takes 1 damage |
| Strike | Hook | 3 | Target takes 1 damage, pulled 1 tile toward caster along the Path |
| Strike | Break | 2 | Remove 1 Armor from target. Does not deal HP-Damage unless boosted by Charge |
| Strike | Steal | 4 | Target takes 1 damage. Steal 1 Money from opponent. |
| Strike | Tempest | 4 | Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. Caster not affected. |
| Shield | Shield | 2 | Self: gain +1 Armor |
| Shield | Heal | 3 | Remove Injured from one adjacent ally |
| Shield | Plate | 3 | Adjacent ally gains +1 Armor |
| Move | Dash | 3 | Self: move up to 2 tiles along the Path |
| Move | Blast | 2 | Push target enemy 1 tile directly away from caster |
| Move | Shove | 3 | Push target enemy 1 tile in any direction. Range+1. |
| Move | Swap | 4 | Swap position with an allied piece. Requires unobstructed Path. |
| Move | Retreat | 4 | Self: move along the Path to land adjacent to one of your Guards. Range+1. |
| Mystic | Focus | 1 | The next skill used by any of your pieces this turn gains +1 Range. Move skills: caster chooses activation-range or effect-range, not both. |
| Mystic | Charge | 3 | One Strike skill used by any of your pieces this turn deals +1 damage. |

## Quick Reference (overrides vs. baseline)

- **Armor:** Max 2 per piece.
- **Injured Range penalty:** None — Injured pieces operate at full speed and Range.
- **Guard speed:** Normal 2, Injured 2 (no penalty).
- **Champion / King speed:** 1 (Normal or Injured).
- **Combo Bonus:** Counter ticks on Strike OR movement-causing skill from new Champion. Bonus damage applies to any qualifying skill.
- **Board:** 8×8 grid. No terrain.
- **Draw conditions:** None. Game ends only when a King is captured.
- **Steal cost:** 4 Money (both Modes).

## What is bundled (six simultaneous changes)

| Change | Targets |
|---|---|
| Board 10×10 → 8×8 | Tile economy; per-piece move-option count down; pieces always close. |
| Armor cap 3 → 2 | Chassis volume (OQ-11 cross-pole confirmed); stalling pattern. |
| Injured penalties removed | Damage-vs-heal asymmetry; Injured remains HP-tracker only. |
| Draw conditions removed | Principle 8 (single climax → natural end). |
| Steal cost 3 → 4 (both Modes) | Money economy rebalance; OQ-34 addressed. |
| Combo Bonus widened | Movement-causing skills tick counter + bonus damage on any qualifying skill. |

## Why bundled (methodology deviation)

The Incremental Testing Methodology says: "Never propose changing multiple interacting systems at once." Stack M deliberately violates this, documented per Principle 7 ("while core identity is unsettled, prefer fundamental shifts over variable tweaking"). Six sequential stacks would take 6+ playtest sessions to clear what Stack M clears in one. Components are independently validated candidate fixes from P3-P5. If the bundle works, follow-up isolation stacks (piece count, unified actions, 6×6 board) narrow further. If it fails, rollback proceeds one axis at a time per the routing rules.

## Hypothesis

A single coordinated cut to chassis-volume, end conditions, and engagement-geometry produces a 30-60 minute game with a single-climax shape (Principle 8) without breaking the combo fantasy.

## Watching

Game length (rounds + wall-clock vs 30-60 min target); single-climax shape (Principle 8 KPI); mid-game stalling pattern (P4 R15-R21 cluster — gone?); combo widening reception; Injured-as-tracker feel; no-draws environment (any infinite games?); Steal at cost 4 still must-pick?; 8×8 cramping; felt-PI (OQ-64); cognitive load (OQ-60).

## Routing on result

- **Length + shape + no-stalling all land** → Stack M accepted into baseline. Next: piece count (Stack K) or 6×6 board (Stack D variant) as isolation stacks.
- **Combo widening dominates / Tempest broken** → roll back movement-counter trigger only. Keep other five.
- **Length still too long, no stalling** → next is piece-count cut.
- **Cleverness gone (pure aggression)** → roll back combo widening OR Steal cost increase.
- **Injured-no-penalty makes pieces disposable** → roll back Injured change only.
- **No-draws causes infinite games** → restore only-Kings-remain draw condition.
- **Bundle uninterpretable** → individual rollback per axis in sequenced next stacks.

## Cross-refs

OQ-11 (Armor — addressed); OQ-34 (Steal — addressed); OQ-38 (combo widening — addressed); OQ-57 (Injured penalties — addressed via removal); OQ-66 (game length target — primary axis); OQ-68 (draw conditions — resolved by removal); Principle 6 + Principle 7 + Principle 8; `essay-game-economy-map`.
"""

stacks = [
    # (id, letter, name, status, hypothesis, body, playtested_in, created_in)
    ("stack-m", "M", "Game Length Cut", "active",
     "A single coordinated cut to chassis-volume, end conditions, and engagement-geometry produces a 30-60 minute game with a single-climax shape (Principle 8) without breaking the combo fantasy.",
     STACK_M_BODY, None, "session-25"),

    ("stack-a-g1", "A-G1", "Attack Nerf", "resolved",
     "Reducing Move-Attack damage from 2 to 1 forces engagement through skills rather than free attacks, ending the standoff zone.",
     """*Accepted into baseline (P3, 2026-05-17). First Champion kill moved R26 → R11. Standoff dissolved.*

**Change:** Move-Attack deals 1 damage (was 2).

**Result:** Skills become primary damage source. Standoff pattern fully dissolved across P3. Folded into baseline.

**Cross-refs:** OQ-23, `essay-playtest-3-analysis`.""",
     "playtest-3", "session-7"),

    ("stack-a-g2", "A-G2", "Multi-Champion Combo Bonus", "resolved",
     "Rewarding multi-Champion target focus with bonus damage incentivises coordinated setups over solo grinding.",
     """*Confirmed in mechanics, design-aligned in feel (P4, 2026-05-28). Migrated into baseline in Session 23.*

**Change:** Multi-Champion Strike-only counter scales +0/+1/+2. When a new Champion hits the same target with a Strike, the counter ticks up; next hit deals +counter damage.

**Result:** Bonus mechanic worked as designed. Players coordinated multi-Champion sequences. Scope-widening discussion produced Stack A G3 (Dual-Counter Combo) — see queued. Now lives in `baseline-sections.typ` as `section-multi-champion-combo()`.

**Cross-refs:** OQ-38, `essay-playtest-4-analysis`.""",
     "playtest-4", "session-19"),

    ("stack-a-g3", "A-G3", "Dual-Counter Combo", "queued",
     "Adding an attacker-counter alongside the target-counter rewards distributing pressure across multiple fronts, dissolving the exchange-pit pattern.",
     """*Gated on Stack H. Mechanic in `bp-dual-counter-combo`.*

**Mechanic:**
- Target counter (kept from G2): different friendly Champions hit same enemy target → bonus on 2nd+ hit.
- Attacker counter (new): same friendly Champion hits different enemy targets → bonus on 2nd+ hit. *Session 25 narrowing note: attacker counter felt too generous on reflection — narrow before shipping.*
- Both counters live in parallel; if a hit qualifies for both, both fire.
- Scope widened (target counter): any skill that hits an enemy piece counts. Move-Attacks excluded.

**Targets:** OQ-38 scope-not-strength reframe + OQ-58 exchange-pit + OQ-59 (esp. 59b endgame conversion gap).

**Justifications:** (a) Cross-category crowd-out (P4 #3, Q-D3-risk); (b) Late-game offensive lockout (P4 #6); (c) Mid-game exchange-pit pattern (P4 OQ-58).

**Teaching-cost flag (G4 / OQ-60):** two parallel counters is strictly more complex than current G2 — likely needs physical tokens. Watched for cognitive-load violation.

**Routing:**
- Exchange-pit dissolves → resolve OQ-58 + OQ-38 dual-counter accepted.
- Exchange-pit persists, dual-counter scope OK → advance to Stack F (Sente Skills).
- Cognitive load too high (G4 violation) → roll back to single-counter widened scope.""",
     None, "session-22"),

    ("stack-b", "B", "Bodyguard Fix", "withdrawn",
     "Defender-only adjacency would force Guards to physically intercept rather than passively absorb damage.",
     """*Withdrawn (Session 22, 2026-05-29).*

**Reasoning:** P4 confirmed Bodyguard tracks standoff state, not the rule (0 triggers when Armor stalling returned). Different solutions would be on the table even if Bodyguard remains broken post-Stack-H. The stack as drafted is not the right fix.

**Cross-refs:** P4 evidence; Session 22 designer call.""",
     None, "session-7"),

    ("stack-c", "C", "Pacing", "dormant",
     "Hard caps on game length (King HP, Armor decay) prevent indefinite play.",
     """*Trigger to revive:* First Champion kill past Round 20 in any future stack. P4 first-kill = R13 → not yet triggered.

**Variants:**
- King Lifetime HP (unkillable-King → fixed length).
- Armor Decay (Armor breaks down each round).

**Status:** Rule sheet not yet written. OQ-19, OQ-41.""",
     None, "session-11"),

    ("stack-d", "D", "Board Geometry", "dormant",
     "Alternative board sizes/shapes change tile economy and movement patterns.",
     """*Trigger to revive:* Board size or geometry surfaces as a bottleneck in any future stack. Also a candidate game-length lever (OQ-66). *Note: Stack M includes 8×8 as one of its six bundled changes — if Stack M's routing isolates the board lever, Stack D steps in.*

**Variants:** 8×10 (OQ-52 narrower board), 8×8, hex grid (gated on `/research hex vs square grid in tactical games` per OQ-42).

**Status:** Rule sheet not yet written.""",
     None, "session-22"),

    ("stack-e", "E", "Draft Flow", "dormant",
     "Alternative draft mechanics change setup choice space.",
     """*Trigger to revive:* Draft feels stale or under-explored after Pole A revival stacks land. May be partially subsumed if pre-made loadouts (OQ-65) ship with simultaneous-reveal selection (OQ-62).

**Variants:**
- Pool draft (OQ-35 — draft pool first, assign after).
- Placement order (OQ-36 + OQ-48 — equip skills first, then place on board).

**Status:** Rule sheet not yet written.""",
     None, "session-22"),

    ("stack-f", "F", "Sente Skills", "dormant",
     "Sente threats force forward commitment as a different mechanism for dissolving exchange-pit patterns.",
     """*Trigger to revive:* Stack A G3 ran but exchange-pit pattern persists. Sente threats are a different mechanism for the same problem — not a duplicate.

**Variants:** Cascade trigger (+1 action on kill, OQ-51), Pin / Threatened restriction, midline pressure skills (10 candidates staged in backpocket).

**Status:** Rule sheet not yet written. Sequenced after Stack A G3 per Session 22 decision.""",
     None, "session-11"),

    ("stack-g", "G", "Unified AP", "dormant",
     "A 3-action unified action-point model replacing separate Move/Skill phases simplifies turn structure and opens new tactical patterns.",
     """*Trigger to revive:* Core systems stable across A G3, H, J, K. Radical structural change — do not test alongside other active experiments.

**Variant:** 3 actions/turn unified action-point model, replacing separate Move and Skill phases.

**Status:** Draft written (`docs/test-scenarios/stack-g-structure/`), not yet run. OQ-26.""",
     None, "session-22"),

    ("stack-h", "H", "Armor Trim", "queued",
     "Reducing Armor cap (and buffing Plate to one-shot fortify) compresses the chassis-volume Armor↔Armor-Breaker loop and makes the engine more audible.",
     """*Absorbed into Stack M; remains as isolation-fallback.*

**Status:** Armor cap 3→2 is one of Stack M's six bundled changes. Stack H as a standalone is *not* the next stack — but if Stack M's routing produces "rollback Armor only" or "rollback everything except Armor", Stack H steps in as the isolation stack for the Armor lever.

**Targets:** OQ-11 chassis-volume hypothesis (Armor↔Armor-Breaker loop crowds out combo loop, P4-confirmed; P5 cross-pole confirmed). Now also a game-length lever (OQ-66).

**Bundled (lead) dose (if revived):** Armor cap 3→2 AND Plate +1→+2 (one-shot fortify, not stack-grind). *Note: Stack M only takes the cap change; the Plate buff would re-enter via Stack H if needed.*

**Smaller dose (rollback within Stack H):** Armor cap 3→2 only, Plate unchanged. Run if the bundled dose stalls.

**"Build cheaper than break" risk** — bigger than originally framed (Session 23): *"if it is way easier to stack armor then it is to get rid of it... the change can exponetiallise this even more."*""",
     None, "session-19"),

    ("stack-i", "I", "Armor Rollback", "absorbed",
     "A smaller-dose Armor cap reduction as contingency.",
     """*Folded into Stack H (Session 22, 2026-05-29).* Was a contingency dose, not a distinct stack. Now lives as the smaller dose within Stack H.""",
     None, "session-19"),

    ("stack-j", "J", "Injured Trim", "queued",
     "Removing Injured's mechanical downsides (speed cap, Range −1, self/adjacent carve-out) makes Injured an HP-tracker only, cutting chassis volume.",
     """*Gated on Stack H. Also absorbed into Stack M (Stack M removes Injured penalties entirely).*

**Targets:** OQ-57 — does Injured's mechanical chassis (speed cap, Range −1, self/adjacent carve-out) pay for itself in game-feel?

**Variant:** Remove Injured's mechanical downsides. State persists as HP-tracker only.

**Why gated on H:** Armor chassis-volume reduction must land first so Injured-volume signal reads cleaner. *Note: Stack M includes this — Stack J as standalone only if Stack M rolls back and Injured remains the primary lever.*

**Routing:**
- Game still reads well, no downside felt → Injured-as-HP-tracker baseline candidate.
- Injured pieces feel structureless (no threat/penalty) → keep current rules. OQ-57 closes negative.""",
     None, "session-22"),

    ("stack-k", "K", "Piece Count Reduction", "queued",
     "Reducing total piece count from 5+6+1 to 3+4+1 per side increases per-piece importance and decision density.",
     """*Gated on Stack H; also game-length lever OQ-66.*

**Targets:** OQ-27 piece density. *Decoupled from board geometry as of Session 22* — Stack D owns board size. Now also a game-length lever.

**Variant:** Current board with 3 Champions + 4 Guards + 1 King per side (vs current 5+6+1).

**Entry conditions:** Two experienced players, full session.

**Routing:**
- Density feels right, decisions sharper → OQ-27 leans toward fewer pieces.
- Game gets too thin / too short → density was load-bearing. Revisit only with smaller board (Stack D 8×8) bundled.""",
     None, "session-22"),

    ("stack-l", "L", "Pole B Per-Turn-Draft Prototype", "dormant",
     "A radical alternative where players draft skills mid-game from a shared pool every turn, rather than pre-game.",
     """*PAUSED Session 25 after P5. Three structural problems surfaced.*

**Variant:** Three-phase turn (Move → Draft → Skill). Move and Draft share a 4-action pool. Skill Phase is free with consumable activations. Bodyguard sits between Move and Draft. Players draft skills mid-game from a shared pool.

**P5 result (Elias vs Jonathan, digital, 15 rounds):**
1. Armor 3 still felt mandatory (cross-pole confirmation of OQ-11).
2. Play collapsed to pure reaction (no multi-turn planning).
3. Felt-PI broke under combinatorial breadth (OQ-64).

**Trigger to revive:** Pole A revival track stalls AND a clear Pole B variant addresses P5's surfaced problems. Candidates in backpocket: permanently-equipped (non-consumable) drafted skills; per-Skill-Phase activation cap; skills cost a resource to activate.

**Status:** Rule sheet at `docs/test-scenarios/stack-l-per-turn-draft/`. See OQ-61 partial resolution.""",
     "playtest-5", "session-23"),
]

conn = sqlite3.connect(DB)
cur = conn.cursor()
ok = 0
fail = []
for r in stacks:
    try:
        cur.execute(
            """INSERT INTO stacks (id, letter, name, status, hypothesis, body, playtested_in, created_in)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            r,
        )
        ok += 1
    except Exception as e:
        fail.append((r[0], str(e)))
conn.commit()
print(f"Inserted {ok} stack rows.")
if fail:
    print("Failures:")
    for f in fail:
        print(f"  {f[0]}: {f[1]}")

cur.execute("SELECT status, COUNT(*) FROM stacks GROUP BY status ORDER BY status;")
print("\nBy status:")
for s, c in cur.fetchall():
    print(f"  {s:<12} {c}")
conn.close()
