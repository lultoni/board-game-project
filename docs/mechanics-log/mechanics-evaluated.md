# Mechanics Evaluated Log

*A running log of mechanics considered, tested, accepted, deferred, or discarded.*
*Supplements the ADR records (`docs/decisions/`) with smaller-scale evaluations.*
*Update whenever a mechanic is proposed, tested, or resolved.*

---

## Accepted — In Current Baseline

These mechanics were tested in Playtest 1 or resolved by designer ruling and are now canonical.

| Mechanic | Accepted | Reason |
|----------|----------|--------|
| 10x10 board | Session 1 / P1 | Both players confirmed size felt right |
| Unlinked Movement + Action | Session 1 / P1 | Intuitive and appreciated by both players |
| Skill Path blocked by all pieces (ally + opponent) | Session 3 (ruling) | Both players confirmed |
| Rune income at start of each player's OWN turn | Session 3 (ruling) | Resolves ambiguity — not end of round |
| Round 1: no Rune income (use starting Runes only) | Session 3 (ruling) | Clarifies first turn |
| Standard attack survival: attacker stops before target | Session 3 (ruling) | Occupies tile only if target removed |
| Bodyguard: Standard Attacks only, not skills | Session 3 (ruling) | Skills always hit directly |
| Healing: no cap | Session 3 (ruling) | Keeps it simple |
| Rune cap: none | Session 3 (ruling) | Natural spending observed in P1 |
| Free pathing for movement (any route ≤ speed) | Session 3 (ruling) | Cannot pass through any piece |
| Defender chooses which Guard intercepts | Session 1 / P1 suggestion | Accepted as ruling |
| No terrain effects | Session 1 (ADR-001/002) | Confirmed overhead complexity; removed |

---

## Accepted — Pending Test (in test layers)

These are accepted in principle but need playtest confirmation.

| Mechanic | Layer | Hypothesis |
|----------|-------|------------|
| 6 start Runes, +2/turn, scaling every 5 rounds | Layer 1 | Fixes dead opening; skills come online Turn 1 |
| 3 HP for Champions/King (Guards stay 2 HP) | Layer 2 | Combat becomes gradual; Injured state matters |
| Bodyguard: adjacent to defender only; Guard takes 2 DMG | Layer 3 | Bodyguard triggers frequently; Guards feel useful |
| Unified AP system (3 AP/turn) | Layer 4 | Cleaner decisions; merges Move + Skill phases |
| 8x8 board, fewer pieces | Layer 5 | Spells become the star; faster first contact |

---

## Deferred

Mechanics that may be revisited after core systems are stable.

| Mechanic | Deferred Since | Condition to Revisit |
|----------|---------------|----------------------|
| Economy skills as Skill Slots | Session 1 (ADR-002) | Only viable if Skill Slots increase significantly (4+ per turn). 2 slots is too few to dedicate one to economy. |
| Performance-based Rune gain (captures → Runes) | Session 1 | Revisit if Layer 1 automatic economy still feels flat. **Session 8 analysis**: Keeping closed. Performance-based income biases toward whichever strategy earns fastest, constraining creative expression and making games feel samey. Auto-economy is strategy-neutral. The combo bonus (Layer 2) is the better lever — it rewards cleverness of execution, not which action you chose. |
| Damage escalation after Round X | Session 1 (ADR-002) | Only if games are still 25+ rounds after Layers 1–3 |
| 3 Move Slots (vs current 2) | Session 1 / P1 suggestion | May be superseded by AP system (Layer 4) |
| 3rd skill slot per Champion | Session 1 (ADR-002/003) | Inherited as 2 slots from baseline. Justified by principle: 2 slots forces specialist builds → rock-paper-scissors draft dynamics → meaningful tradeoffs. 3 slots risks generalist meta where draft tension dissolves. **Session 8 analysis**: Keeping closed. The fix for "narrow build variety" is better skill design, not more slots. Revisit only if post-Layer-4 (unified AP) reveals build variety feels too narrow despite diverse skill catalogue. |
| Restricted movement (straight-line only, Skill Path model) | Session 3 (OQ-28) | Layer 6 candidate — would make Move skills stronger and diagonals exclusive to them |
| Board 8x8 with fewer pieces | Session 1 | Deferred to Layer 5, awaiting Layers 1–4 results |

---

## Withdrawn / Rejected

Mechanics that were explicitly ruled out, with reasons.

| Mechanic | Withdrawn | Reason |
|----------|-----------|--------|
| YINSH-inspired capture penalty | Session 1 (ADR-002 feedback) | Creates asymmetric cost — if one player runs out of Guards, only the other player pays the penalty. Punishes playing correctly. |
| Hex grid | Moved to Reopened — see below | See OQ-42 |
| No board (Direction B — card fighter) | Session 1 (ADR-001) | Loses spatial skills (push/pull/swap). "Just another card game." Designer rejected. |
| Zone/lane system (Direction C — spatial hybrid) | Session 1 (ADR-001) | Direction A+ chosen instead; grid preserved. |
| Terrain effects (Water/Forest/Plains/Mountains) | Session 1 (ADR-001) | Confirmed overhead complexity; removed from current design. Reversible — could return as "map variant" expansion. |
| Linked Movement-Action (move to act) | Session 1 / P1 | Unlinked preferred; likely superseded by AP system. |
| Performance-based Rune gain | Session 8 (OQ-47) | Forces single playstyle, constrains creative expression. Auto-economy is strategy-neutral; combo bonus rewards cleverness of execution. KPI problem — rewards symptoms, not systems. |
| Economy skills as Skill Slots | Session 8 (OQ-25) | 2-slot scarcity makes this unworkable. Sacrifices combat versatility entirely. Could be post-v1 expansion variant. |
| Restricted movement (straight-line only) | Session 8 (OQ-28) | Not solving a real problem. Punishes Guards (Speed 2 straight-line = terrible navigation). Meaningless for Champions (Speed 1 = 1 tile any direction). Dissolves if hex adopted. |
| CR-style draft picks (strict interleaving) | Session 8 (OQ-43) | Restricts free strategy; counter-picking with small catalogue leads to "correct" picks that reduce variety. Variant material, not core game. |
| Ban phase in draft | Session 8 (OQ-44) | From older game version with unique fixed-skill Champions. Needs 20+ skills and a different draft model. |
| Starting player bid (hidden Rune auction) | Session 8 (OQ-45) | No first-player advantage observed. If it surfaces, use Go-style komi (fewer starting Runes) instead. |
| Coordinated movement bonus (−1 Rune if pieces move to same zone) | Session 11 (OQ-51) | Too easy to trigger accidentally — rewards moving pieces in the same direction, which players do anyway. Doesn't actually require cleverness. |
| Checkmate-style win condition (formal "inescapable position" rule) | Session 11 (OQ-19) | Verification burden too high for our game — too many defensive options (heal, armor, push, LoS block) to formally prove "100% lost" at the table. Research confirmed this is analogous to Shogi brinkmate (impractical without a computer). King capture remains the only formal win condition. |

---

## Reopened / Under Review

Mechanics that were previously withdrawn but have been re-evaluated and reopened for investigation.

| Mechanic | Reopened | Reason |
|----------|----------|--------|
| Hex grid | Session 8 (2026-04-28) — OQ-42 | Original Session 1 rejection was by omission: ADR-001 confirmed "grid over card-fighter," not "square over hex." Hex IS a grid variant. The reasoning for keeping the grid (makes skills/push/pull meaningful) applies to hex equally or better. Requires proper evaluation. See OQ-42 and trigger `/research hex vs square grid in tactical games` before scheduling a test layer. |
