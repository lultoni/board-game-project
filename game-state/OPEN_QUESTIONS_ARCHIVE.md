# OPEN QUESTIONS — ARCHIVE

*Resolved, closed, scrapped, or parked-indefinitely OQs. Do not edit historical entries — append only when an item is moved here from `OPEN_QUESTIONS.md`.*

*Live questions live in `OPEN_QUESTIONS.md`. When an OQ is closed, move it here and link from `mechanics-evaluated.md` if applicable.*

---

## Resolved

### OQ-1: Board Size — PARTIALLY RESOLVED
**10x10 confirmed as viable.** Both players said size was generally right. Early game still has too much empty space (first contact at ~Round 7). 12x12 likely worse. Hex untested.
- **Remaining question**: Would 8x8 be better? Deferred to Stack D testing.
- **Status**: 10x10 is the baseline. Board size change tracked under Stack D scope.

### OQ-3: Movement-Action Link — PARTIALLY RESOLVED
**Unlinked works and is liked.** May be superseded by Unified AP system (Stack involving AP), which merges the concept entirely.
- **Status**: Unlinked is the baseline.

### OQ-5: Skill Path Blockage — RESOLVED
**Blocked by all pieces.** Both players confirmed.

### OQ-9: Starting Piece Placement — RESOLVED
**Current placement is balanced.** Both players confirmed.
- **Note**: Becomes important again if piece count or board size changes. On 8x8 with fewer pieces, placement matters a lot more — will need re-evaluation.

### OQ-13: First Player Advantage — PARTIALLY RESOLVED
**No strong advantage observed** in Playtest 1. Low priority. Re-evaluate only if first-player win rate becomes consistent across many games.

### OQ-17: Rune Start Rate — RESOLVED (Stack A baseline, Playtest 2, 24.04.2026)
**Start with 6 Runes, +2 gain/turn, scaling +1 every 5 rounds. ACCEPTED.**
- Both players used skills from Round 1. Dead opening eliminated.
- Skill Slots became the action limiter in early game — as intended.
- Economy is fast enough without being overwhelming.
- **Status**: Accepted. Carry forward into all future stacks.

### OQ-20: Shadow Shift Balance — PARTIALLY RESOLVED
**Shadow Shift uses default Range 2** (not a special fixed cap).
- The prior "Range 3" fix was a misunderstanding — Shadow Shift uses the default Skill Range (2).
- **Status**: Resolved as Range 2 (default). Watch whether Range 2 feels limiting in practice.

### OQ-29: Move Slots per Piece — RESOLVED (applied to baseline)
**Each piece may only be moved once per Movement Phase.** Ruled mid-game Playtest 2.

### OQ-30: Default Skill Range — RESOLVED (applied to baseline)
**Default Skill Range = 2.** Range 0 = self, Range 1 = adjacent, Range 2 = 2 tiles along Skill Path.
- Adjacent-while-injured edge case: minimum Range is 0 (self). A piece at Range 0 may always target itself.

### OQ-31: Focus Strike / Blade Call Scope — RESOLVED (applied to baseline)
**Focus Strike buffs the *next* skill used by any of your pieces that turn. Blade Call buffs any *one* Strike skill used by any of your pieces that turn (before or after).** Focus Strike must come first; Blade Call can be declared retroactively. Each Blade Call activation boosts exactly one Strike, then is spent.

### OQ-32: Movement-via-Skills Damage — RESOLVED (applied to baseline)
**Skills that move pieces do NOT deal damage.**

### OQ-33: Blade Tempest Self-Affect — RESOLVED (applied to baseline)
**Blade Tempest does NOT affect the caster.** Only the target takes 1 DMG; only pieces adjacent to the *target* are pushed.

### OQ-37: Standard Attack Damage — CONFIRMED (Playtest 3, accepted into baseline)
**Standard attacks deal 1 DMG instead of 2.**
- Playtest 3 (L2G1): first Champion kill **Round 11** (vs P2's R26). Both players Q2 "Felt right." Combat feel "Better / Much Better." Standoff dissolved.
- Risk that Guards become hard to remove — **not observed**. First Guard kill R10. Guards behaved as screens rather than damage sponges.
- **Status**: Accepted into baseline. See `mechanics-evaluated.md`.

### OQ-40: Standoff / No-Man's-Land Problem — CONFIRMED RESOLVED (Playtest 3, primary lever worked)
**The standard attack nerf (OQ-37) dissolved the standoff in Playtest 3.**
- Both players: "Much less standoff" / "Not reluctant to move forward." Forward movement active from early rounds.
- **Confirms primary research finding**: lower entry risk + sente design dissolves standoff.
- **Status**: Confirmed resolved by Stack A Game 1. Sente design (Stack F) is no longer urgent. Watch for re-emergence with two experienced players.

### OQ-46: Rune Cap — CLOSED FROM MONITORING (Playtest 3)
`[System: Resource Economy] [Affects: Skill System]`
**No hoarding observed in Playtest 3** — neither player accumulated unspent Runes meaningfully. Both players said "always wanted more / balanced." Confirms G2 (encourage spending via attractiveness) is working.
- **Status**: Closed. No cap needed.

### OQ-10: Injured Penalty Severity — CLOSED (ruling confirmed in baseline)
- Speed penalty is Guard-only. The only Champion/King Injured effect is Skill Range -1 (affects Range 2+ skills only).
- **P3 blocker**: Lance Thrust + Injured ambiguity cost Elias a turn at R22. Blocked severity evaluation.
- **Resolution (Session 18)**: Baseline already contains the complete ruling. Range-1 is a modifier from default Range 2, so an Injured piece using Lance Thrust has effective Range 0 (self only) — it cannot fire. This is derivable from `baseline-sections.typ` lines covering Range modifiers and the Injured Range penalty.
- **Remaining watch**: The rule requires chaining three lines of text — not explicit at the table. OQ-54 tracks whether "Range-1" language causes confusion in practice.
- **Status**: Closed. Severity is evaluatable; watch OQ-54 for wording clarity.

### OQ-11: Armor Cap — CONFIRMED WORKING (Playtest 3, RPS loop functions)
- Keep at 3.
- **P3 update (Session 15)**: Mario stacked Armor heavily; Elias drafted Armor Breaker as the counter and used it effectively. The RPS loop **functions as designed**. Elias Q17: Armor "slightly extended / well balanced."
- **Status**: Confirmed working at cap 3. Watch for re-emergence with two experienced players.

---

## Closed (permanent — no longer revisitable as core)

### OQ-25: Economy Skills as Skill Slots — CLOSED (Session 8)
- With only 2 skill slots per Champion, equipping an economy skill makes that piece a "one-trick pony" — sacrifices combat versatility entirely.
- **Session 8 verdict**: Closed permanently. 2-slot scarcity makes this unworkable. The core idea (ultra-defense → late-game payoff strategy) is interesting as a post-v1 expansion variant, not a core system change.

### OQ-47: Performance-Based Rune Gain — CLOSED (Session 8)
`[System: Resource Economy] [Affects: Combat, Progression]`
- Performance-based income forces players toward whichever strategy earns Runes fastest, constraining creative expression. Auto-economy is strategy-neutral. The combo bonus is the correct lever: it rewards *cleverness of execution*, not *which action you performed*.
- **Status**: Closed. Not revisiting.

---

## Scrapped (proposed, then dropped without testing)

### OQ-18: Health System (3 HP) — SCRAPPED
**3 HP for Champions/King was the proposed test, but it has been scrapped before testing.**
- Reason: 3 HP would make the game even longer (first kill was already at R26 with 2 HP). Guards at 2 HP with Champions at 3 HP creates an artificial rank.
- The underlying problem (combat too coarse, Injured state bypassed by standard attacks) was solved by OQ-37 (1 DMG standard attack) instead.
- **Status**: Scrapped.

---

## Parked Indefinitely (v1 out of scope)

### OQ-15: Terrain System
**Confirmed as overhead complexity.** Removed in A+ direction. Reversible — can return as "map variant" expansion if missed.

### OQ-28: Restricted vs. Free Movement
**Free pathing is the baseline.** Restricting movement to straight-line-only would punish Guards (Speed 2 in a straight line is terrible for navigation) and is meaningless for Champions (Speed 1). Dissolves entirely if hex grid is adopted.
- **Status**: Parked. Only revisit if Move skills feel weak AND hex is not adopted.

### OQ-43: CR-Style Draft Picks
`[System: Skill Drafting]`
- Restricts free strategy picking — with a small catalogue, counter-picking leads to "correct" picks that reduce variety. Only relevant with a much larger catalogue (20+ skills).
- **Status**: Parked. Variant/expansion material.

### OQ-44: Ban Phase in Skill Draft
`[System: Skill Drafting]`
- From an older game version with unique Champions that had fixed skills. The current system (shared skills, free assignment) makes banning less meaningful. Needs 20+ skills AND a different draft model to be viable.
- **Status**: Parked.

### OQ-45: Starting Player Decision
`[System: Turn Structure]`
- No first-player advantage observed in 2 playtests. If it ever surfaces over many games, the fix is Go-style komi (P1 starts with fewer Runes), not a bidding war.
- **Status**: Parked. Only revisit if consistent first-player win rate observed across many games.
