# Mechanics Evaluated Log

*A running log of mechanics considered, tested, accepted, deferred, or discarded.*
*Update whenever a mechanic is proposed, tested, or resolved.*

*Schema (since Session 17): every row links the motivating OQ (or `—` if not OQ-tracked) and the evidence source (playtest analysis, research file, ruling session, or `—` if pure principle). Cross-link by ID — never restate verdicts.*

---

## Accepted — In Current Baseline

These mechanics were tested in playtests or resolved by designer ruling and are now canonical.

| Mechanic | Source OQ | Evidence | Accepted | Reason |
|----------|-----------|----------|----------|--------|
| 10x10 board | OQ-1 | P1 feedback | Session 1 / P1 | Both players confirmed size felt right |
| Unlinked Movement + Action | OQ-3 | P1 feedback | Session 1 / P1 | Intuitive and appreciated by both players |
| Skill Path blocked by all pieces (ally + opponent) | OQ-5 | P1 feedback | Session 3 ruling | Both players confirmed |
| Rune income at start of each player's OWN turn | — | Session 3 ruling | Session 3 | Resolves ambiguity — not end of round |
| Round 1: no Rune income (use starting Runes only) | — | Session 3 ruling | Session 3 | Clarifies first turn |
| Standard attack survival: attacker stops before target | — | Session 3 ruling | Session 3 | Occupies tile only if target removed |
| Bodyguard: Standard Attacks only, not skills | — | Session 3 ruling | Session 3 | Skills always hit directly |
| Healing: no cap | — | Session 3 ruling | Session 3 | Keeps it simple |
| Rune cap: none | OQ-8 / OQ-46 | P1, P2, P3 — no hoarding observed | Session 3 ruling, closed P3 | Natural spending observed across all playtests |
| Free pathing for movement (any route ≤ speed) | OQ-28 | Session 3 ruling | Session 3 | Cannot pass through any piece |
| Defender chooses which Guard intercepts | OQ-21 | P1 suggestion | Session 1 / P1 | Accepted as ruling |
| No terrain effects | OQ-15 | ADR-001/002 | Session 1 | Confirmed overhead complexity; removed |
| 6 start Runes, +2/turn, scaling every 5 rounds | OQ-17 | `playtest-2-analysis.md` | Layer 1 / P2 | Fixes dead opening; skills come online Turn 1 |
| Move Slots: each piece moved at most once / Movement Phase | OQ-29 | P2 mid-game ruling | P2 | Now explicit in all rule sheets |
| Default Skill Range = 2 (Range 0 = self, 1 = adjacent, 2 = 2 tiles) | OQ-30 | P2 mid-game ruling | P2 | Now defined in all rule sheets |
| Focus Strike / Blade Call timing & scope | OQ-31 | P2 mid-game ruling | P2 | Focus Strike must come first; Blade Call retroactive |
| Skills that move pieces deal NO damage | OQ-32 | P2 mid-game ruling | P2 | General rule in Skill System section |
| Blade Tempest does NOT affect caster | OQ-33 | P2 mid-game ruling | P2 | Only target takes 1 DMG |
| Standard attack 1 DMG (instead of 2) | OQ-37 | `playtest-3-analysis.md` | Session 15 / P3 | First Champion kill R11 vs P2 R26; standoff dissolved; combat feel "Better/Much Better" |
| Range system clarification (self/adjacent/default) | OQ-10 | Session 16 ruling | Session 16 | All skills default to Range 2 unless text names "self" or "adjacent." Range modifiers apply from default. Self/adjacent cannot be shifted inward by Range buffs. Injured penalty does not affect self/adjacent skills. |
| Focus Strike boosting self/adjacent skills | OQ-31 | Session 16 ruling | Session 16 | Can boost self→adjacent and adjacent→Range 2. Range and Injured calculated independently. |

---

## Accepted — Pending Test (in test stacks)

These are accepted in principle but need playtest confirmation.

| Mechanic | Source OQ | Stack | Hypothesis |
|----------|-----------|-------|------------|
| Bodyguard: adjacent to defender only; Guard takes damage | OQ-21 | Stack B | Bodyguard triggers frequently; Guards feel useful. **P3 update**: may be obsolete — Bodyguard organically activated under Stack A nerf without adjacency change. De-prioritised pending one more experienced-player game. |
| Multi-Champion combo bonus (+1 DMG on 2nd+ Strike same target same turn) | OQ-38 | Stack A G2 | Coordination rewarded; raises ceiling of clever play. Strike-only for G2; cross-category reconsidered after data. |
| Unified AP system (3 AP/turn) | OQ-26 | Stack G | Cleaner decisions; merges Move + Skill phases |

---

## Deferred

Mechanics that may be revisited after core systems are stable.

| Mechanic | Source OQ | Deferred Since | Condition to Revisit |
|----------|-----------|---------------|----------------------|
| Damage escalation after Round X | OQ-19 | Session 1 (ADR-002) | Only if games are still 25+ rounds after Stacks A–C |
| 3 Move Slots (vs current 2) | OQ-23 | Session 1 / P1 suggestion | May be superseded by AP system (Stack G) |
| Restricted movement (straight-line only) | OQ-28 | Session 8 | Layer 6 candidate — would make Move skills stronger. Now parked indefinitely (see archive). |
| Board 8x8 with fewer pieces | OQ-1b / OQ-27 | Session 1 | Deferred to Stack D, awaiting Stack A/B kill timing |
| Pool draft variant | OQ-35 | Session 8 | Test after Stack A/B accepted |
| Flexible piece placement | OQ-36 | Session 8 | Bundled with OQ-48; test after Stack A/B |
| Piece placement order (post-draft) | OQ-48 | Session 8 | Bundled with OQ-36; test after Stack A/B |
| Skill Path obstruction Idea 2 (only opponent Guards block) | OQ-49 | Session 8 | Conditional on Stack A/B frustration data |
| Minor/Major Skill Slot cost | OQ-50 | Session 8 | Pre-work needed: design ultimate-skill candidates first |
| Cascade trigger (+1 Skill Slot on kill) | OQ-51 | Session 11 | Backpocket; test in Stack F |
| Pin / Threatened restriction | OQ-51 | Session 11 | Backpocket; needs own stack |
| Collision damage (universal push-into-piece = 1 DMG) | OQ-51 | Session 11 | Conditional on standoff resolution |

---

## Withdrawn / Rejected

Mechanics that were explicitly ruled out, with reasons.

| Mechanic | Source OQ | Withdrawn | Reason |
|----------|-----------|-----------|--------|
| YINSH-inspired capture penalty | OQ-19 | Session 1 (ADR-002 feedback) | Creates asymmetric cost — if one player runs out of Guards, only the other player pays the penalty. Punishes playing correctly. |
| No board (Direction B — card fighter) | — | Session 1 (ADR-001) | Loses spatial skills (push/pull/swap). "Just another card game." Designer rejected. |
| Zone/lane system (Direction C — spatial hybrid) | — | Session 1 (ADR-001) | Direction A+ chosen instead; grid preserved. |
| Terrain effects (Water/Forest/Plains/Mountains) | OQ-15 | Session 1 (ADR-001) | Confirmed overhead complexity; removed. Reversible as "map variant" expansion. |
| Linked Movement-Action (move to act) | OQ-3 | Session 1 / P1 | Unlinked preferred; likely superseded by AP system. |
| 3 HP for Champions/King (Guards stay 2 HP) | OQ-18 | Session 6 | Would extend game (first Champion kill at R26 with 2 HP). Guards at 2 HP vs Champions at 3 HP creates artificial tier. |
| Performance-based Rune gain | OQ-47 | Session 8 | Forces single playstyle, constrains creative expression. Auto-economy is strategy-neutral. KPI problem — rewards symptoms, not systems. |
| Economy skills as Skill Slots | OQ-25 | Session 8 | 2-slot scarcity makes this unworkable. Could be post-v1 expansion variant. |
| 3rd skill slot per Champion | — | Session 8 | 2 slots forces specialist builds; 3 risks generalist meta. Fix for "narrow variety" is better skill design, not more slots. |
| CR-style draft picks (strict interleaving) | OQ-43 | Session 8 | Restricts free strategy with small catalogue. Variant material. |
| Ban phase in draft | OQ-44 | Session 8 | From older game version with unique fixed-skill Champions. Needs 20+ skills and a different draft model. |
| Starting player bid (hidden Rune auction) | OQ-45 | Session 8 | No first-player advantage observed. If surfaces, use Go-style komi instead. |
| Skill Path Idea 1 (only opponent pieces block) | OQ-49 | Session 7 | Creates turtle meta — cluster all pieces, use skills from safety. |
| Coordinated movement bonus (−1 Rune if pieces move to same zone) | OQ-51 | Session 11 | Too easy to trigger accidentally; doesn't reward cleverness. |
| Breakthrough bonus (+1 Slot on first Champion hit) | OQ-51 | Session 11 | Subsumed into cascade trigger; arbitrary "first hit" trigger less elegant. |
| Checkmate-style win condition | OQ-19 | Session 11 | Verification burden too high — too many defensive options (heal, armor, push, LoS block) to prove "100% lost" at the table. `docs/research/checkmate-win-conditions.md`. |
| Class-based skill pools (Champion's class determines drafting) | — | Session 10 | Restricts strategy freedom for minimal complexity reduction. Contradicts blank-slate Champion design. |
| Champion pre-naming (Blacksmith, Necromancer, etc.) | — | Session 10 | Champions are blank slates — identity emerges from equipped skills. |
| Information/scouting skill (reveal Rune count) | — | Session 10 | Irrelevant in a perfect-information game. |

---

## Reopened / Under Review

Mechanics that were previously withdrawn but have been re-evaluated.

| Mechanic | Source OQ | Reopened | Reason |
|----------|-----------|----------|--------|
| Hex grid | OQ-42 | Session 8 | Original Session 1 rejection was by omission: ADR-001 confirmed "grid over card-fighter," not "square over hex." Hex IS a grid variant. Trigger `/research hex vs square grid in tactical games` before scheduling a test stack. |
