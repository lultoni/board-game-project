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
| Unlinked Move + Skill | OQ-3 | P1 feedback | Session 1 / P1 | Intuitive and appreciated by both players |
| Path blocked by all pieces (ally + opponent) | OQ-5 | P1 feedback | Session 3 ruling | Both players confirmed |
| Money income at start of each player's OWN turn | — | Session 3 ruling | Session 3 | Resolves ambiguity — not end of round |
| Round 1: no Money income (use starting Money only) | — | Session 3 ruling | Session 3 | Clarifies first turn |
| Move-attack survival: attacker stops before target | — | Session 3 ruling | Session 3 | Occupies tile only if target removed |
| Bodyguard: Move-Attacks only, not skills | — | Session 3 ruling | Session 3 | Skills always hit directly |
| Healing: no cap | — | Session 3 ruling | Session 3 | Keeps it simple |
| Money cap: none | OQ-8 / OQ-46 | P1, P2, P3 — no hoarding observed | Session 3 ruling, closed P3 | Natural spending observed across all playtests |
| Free pathing for movement (any route ≤ speed) | OQ-28 | Session 3 ruling | Session 3 | Cannot pass through any piece |
| Defender chooses which Guard intercepts | OQ-21 | P1 suggestion | Session 1 / P1 | Accepted as ruling |
| No terrain effects | OQ-15 | ADR-001/002 | Session 1 | Confirmed overhead complexity; removed |
| 6 start Money, +2/turn, scaling every 5 rounds | OQ-17 | `playtest-2-analysis.md` | Layer 1 / P2 | Fixes dead opening; skills come online Turn 1 |
| Move actions: each piece moved at most once / Move Phase | OQ-29 | P2 mid-game ruling | P2 | Now explicit in all rule sheets |
| Default Range = 2 (Range 0 = self, 1 = adjacent, 2 = 2 tiles) | OQ-30 | P2 mid-game ruling | P2 | Now defined in all rule sheets |
| Focus / Charge timing & scope | OQ-31 | P2 mid-game ruling | P2 | Focus must come first; Charge retroactive |
| Skills that move pieces deal NO damage | OQ-32 | P2 mid-game ruling | P2 | General rule in Skill System section |
| Tempest does NOT affect caster | OQ-33 | P2 mid-game ruling | P2 | Only target takes 1 damage |
| Move-attack 1 damage (instead of 2) | OQ-37 | `playtest-3-analysis.md` | Session 15 / P3 | First Champion kill R11 vs P2 R26; standoff dissolved; combat feel "Better/Much Better" |
| Range system clarification (self/adjacent/default) | OQ-10 | Session 16 ruling | Session 16 | All skills default to Range 2 unless text names "self" or "adjacent." Range modifiers apply from default. Self/adjacent cannot be shifted inward by Range buffs. Injured penalty does not affect self/adjacent skills. |
| Focus boosting self/adjacent skills | OQ-31 | Session 16 ruling | Session 16 | Can boost self→adjacent and adjacent→Range 2. Range and Injured calculated independently. |
| Focus on Move skills: caster chooses activation OR effect range | — | Session 18 ruling | Session 18 | The +1 from Focus applies to either the activation range (how far the skill can target) or the effect range (how far it moves/pushes), caster's choice at activation. Not both. Resolves ambiguity for Move-skill / Focus interaction; preserves combo variety without making any single combo dominant. |
| High-concept framing: "Two minds, one puzzle" (Framing B) | Q-A1 | ADR-004 (Session 19, 2026-05-26) | Session 19 | Locks design intent: 2-player nature is load-bearing, not delivery mechanism. Combo legibility must work in both directions; shared draft pool becomes load-bearing chassis feature; Phase B briefed under parallel-discovery imagery; asymmetry biased against. Design intent only — no immediate rule changes. Reversal criterion (updated 2026-05-26 via Q-D2): combined Q-D1 + Q-D2 must both fail across the validation window AND on-ramp interventions tested without improving either result. A weak game-1 signal followed by a strong game-2 signal counts as "lands at game 2 cadence", not failure. See `docs/design-principles.md` § High-Concept Framing. |
| Move-Attack reframed as "a Move that ends on an enemy tile" | Q-B4 | High-concept audit (Angle 2) + Session 19 decision | Session 19 (2026-05-26) | Reworded Move Phase intro and Move-Attack opening in `baseline-sections.typ` to make move-attack unity explicit and plant skill-first thinking without internal jargon. Survival-stop rule strengthened with explicit attacker-speed cases (Guard speed 2 → 1 tile moved if target survives; Champion/King speed 1 → 0 tiles moved; damage dealt either way). Closing italic line bridges Move-Attack and Skills in player-facing language. BASELINE_VERSION bumped to 2026-05-26. Pure framing change — no mechanical change. |
| Multi-Champion Combo Bonus migrated into baseline | OQ-38 | Session 23 (2026-05-30) following Stack A G2 P4 confirmation | Session 23 | Concise version of the rule promoted into `baseline-sections.typ` as `section-multi-champion-combo()` — and added as a row in the quick-reference table. Niko (P4 first-time player) skipped reading the long version and still understood the rule, proving the dense version is sufficient. Worked examples + tracking tables stay out of baseline (teaching aids only; future stacks can re-introduce). BASELINE_VERSION bumped 2026-05-26 → 2026-05-30. The Accepted-Pending-Test row below is now historical — keep for cross-reference. |

---

## Accepted — Pending Test (in test stacks)

These are accepted in principle but need playtest confirmation.

| Mechanic | Source OQ | Stack | Hypothesis |
|----------|-----------|-------|------------|
| Multi-Champion combo bonus (+1 damage on 2nd+ Strike same target same turn) | OQ-38 | Stack A G2 / `playtest-4-analysis.md` | Confirmed in mechanics, weak in feel: Elias R11 "1. ever combo!!" (first multi-Champion combo across 4 playtests); Niko R26-R28 winning loop = Strike+Strike kill loop. But neither rated "Very rewarding"; cross-category crowd-out partially confirmed (Elias Rarely AND Never; Niko Sometimes). Sample too small to revisit Strike-only scope. **Decision: keep into baseline pending one more experienced-player game.** Session 22 reframe: Q3 softness is design-aligned (few-times-a-game payoff). Lever is **scope, not strength** — see Stack A G3 dual-counter row below. |
| Combo bonus dual-counter + widened scope (target counter + attacker counter; any hit-causing skill counts; multi-target ticks all hits; Move-Attacks excluded; both counters stack if a hit qualifies for both) | OQ-38 / OQ-58 / OQ-59 | Stack A G3 (queued) / `docs/backpocket.md` / `playtest-4-analysis.md` Session 22 discussion | Replaces single-counter Stack A G2. **Justifications**: (a) cross-category crowd-out (#3 P4); (b) late-game offensive lockout (#6 P4 — Elias verbatim "I did not have any other attack champs left"); (c) "exchange pit" mid-game pattern (one cluster, pieces taken one-by-one — attacker counter rewards distributing pressure across multiple fronts). **Path A methodology**: gated behind Stack H. Designer agreed: chassis trim must land first because combo scope and Armor volume both affect mid-game pacing. **Teaching-cost flag (G4 / OQ-60)**: two parallel counters strictly more complex than current Stack A G2 — physical tokens or board-side trackers likely needed. Watch flag on multi-target ticking — first rollback if dual-counter proves OP. |
| Unified AP system (3 AP/turn) | OQ-26 | Stack G | Cleaner decisions; merges Move + Skill phases |
| Armor cap 3→2 + Plate +1→+2 (bundled) | OQ-11 / Q-C1 | Stack H / `playtest-4-analysis.md` | Reduces chassis-loop volume (Framing B alignment); Plate becomes one-shot fortify rather than stack-grind. Bundled — coupling documented. **P4 confirmed: Elias Q13 "Yes, a lot" mental focus + verbatim "armor was a part of combo calcs but it just felt like you were not able to do your combos because of it." Combo bonus did NOT dissolve Q-C1 — Niko's R26-R28 loop overran Armor only after 7-round Armor cluster. Stack H now PRIORITY 1.** Smaller dose (C1a, cap 3→2 only) runs as next iteration of Stack H if bundled stalls — folded into Stack H Session 22 (was previously tracked as Stack I). |
| Injured: remove mechanical downsides (no speed cap, no Range −1) | OQ-57 / Q-B5 | Stack J / `playtest-4-analysis.md` | Tests whether Injured's chassis volume (speed cap, Range −1, Range-modifier interactions, self/adjacent carve-out) pays for itself in game-feel terms. State still exists as HP-tracker. Gated behind Stack H. **P4 partially confirmed:** Niko C9 named Injured as confusion source on first read; Niko Q12 "Clearly weaker"; Elias Q12 "Slightly weaker / Barely noticeable" — experienced player barely registers mechanical effect. Volume:payoff ratio looks thin. Could scale up to baseline-change candidate if it plays well. |

---

## Deferred

Mechanics that may be revisited after core systems are stable.

| Mechanic | Source OQ | Deferred Since | Condition to Revisit |
|----------|-----------|---------------|----------------------|
| Damage escalation after Round X | OQ-19 | Session 1 (ADR-002) | Only if games are still 25+ rounds after Stacks A–C |
| 3 Move Slots (vs current 2) | OQ-23 | Session 1 / P1 suggestion | May be superseded by AP system (Stack G) |
| Restricted movement (straight-line only) | OQ-28 | Session 8 | Layer 6 candidate — would make Move skills stronger. Now parked indefinitely (see archive). |
| Board 8x8 with fewer pieces | OQ-1b / OQ-27 | Session 1 | Decoupled Session 22: 8x8 lives in Stack D (Board Geometry); piece count reduction lives in Stack K (Piece Count Reduction). Test independently. |
| Pool draft variant | OQ-35 | Session 8 | Test after Stack A/B accepted |
| Flexible piece placement | OQ-36 | Session 8 | Bundled with OQ-48; test after Stack A/B |
| Piece placement order (post-draft) | OQ-48 | Session 8 | Bundled with OQ-36; test after Stack A/B |
| Path obstruction Idea 2 (only opponent Guards block) | OQ-49 | Session 8 | Conditional on Stack A/B frustration data |
| Minor/Major skill action cost | OQ-50 | Session 8 | Pre-work needed: design ultimate-skill candidates first |
| Cascade trigger (+1 action on kill) | OQ-51 | Session 11 | Backpocket; test in Stack F |
| Pin / Threatened restriction | OQ-51 | Session 11 | Backpocket; needs own stack |
| Collision damage (universal push-into-piece = 1 damage) | OQ-51 | Session 11 | Conditional on standoff resolution |

---

## Withdrawn / Rejected

Mechanics that were explicitly ruled out, with reasons.

| Mechanic | Source OQ | Withdrawn | Reason |
|----------|-----------|-----------|--------|
| Bodyguard: adjacent to defender only; Guard takes damage (Stack B) | OQ-21 | Session 22 (2026-05-29) | P4 confirmed Bodyguard tracks standoff state, not the rule (0 triggers when Armor stalling returned). Defender-only adjacency is unlikely to be the right fix even if Bodyguard remains broken post-Stack-H — different solutions (e.g. simplify/remove Bodyguard per Q-C2 Framing-B watch-flag) would be on the table. OQ-21 itself remains open; the originally proposed stack does not. |
| YINSH-inspired capture penalty | OQ-19 | Session 1 (ADR-002 feedback) | Creates asymmetric cost — if one player runs out of Guards, only the other player pays the penalty. Punishes playing correctly. |
| No board (Direction B — card fighter) | — | Session 1 (ADR-001) | Loses spatial skills (push/pull/swap). "Just another card game." Designer rejected. |
| Zone/lane system (Direction C — spatial hybrid) | — | Session 1 (ADR-001) | Direction A+ chosen instead; grid preserved. |
| Terrain effects (Water/Forest/Plains/Mountains) | OQ-15 | Session 1 (ADR-001) | Confirmed overhead complexity; removed. Reversible as "map variant" expansion. |
| Linked Movement-Action (move to act) | OQ-3 | Session 1 / P1 | Unlinked preferred; likely superseded by AP system. |
| 3 HP for Champions/King (Guards stay 2 HP) | OQ-18 | Session 6 | Would extend game (first Champion kill at R26 with 2 HP). Guards at 2 HP vs Champions at 3 HP creates artificial tier. |
| Performance-based Money gain | OQ-47 | Session 8 | Forces single playstyle, constrains creative expression. Auto-economy is strategy-neutral. KPI problem — rewards symptoms, not systems. |
| Economy skills as actions | OQ-25 | Session 8 | 2-slot scarcity makes this unworkable. Could be post-v1 expansion variant. |
| 3rd skill slot per Champion | — | Session 8 | 2 slots forces specialist builds; 3 risks generalist meta. Fix for "narrow variety" is better skill design, not more slots. |
| CR-style draft picks (strict interleaving) | OQ-43 | Session 8 | Restricts free strategy with small catalogue. Variant material. |
| Ban phase in draft | OQ-44 | Session 8 | From older game version with unique fixed-skill Champions. Needs 20+ skills and a different draft model. |
| Starting player bid (hidden Money auction) | OQ-45 | Session 8 | No first-player advantage observed. If surfaces, use Go-style komi instead. |
| Path Idea 1 (only opponent pieces block) | OQ-49 | Session 7 | Creates turtle meta — cluster all pieces, use skills from safety. |
| Coordinated movement bonus (−1 Money if pieces move to same zone) | OQ-51 | Session 11 | Too easy to trigger accidentally; doesn't reward cleverness. |
| Breakthrough bonus (+1 Slot on first Champion hit) | OQ-51 | Session 11 | Subsumed into cascade trigger; arbitrary "first hit" trigger less elegant. |
| Checkmate-style win condition | OQ-19 | Session 11 | Verification burden too high — too many defensive options (heal, armor, push, LoS block) to prove "100% lost" at the table. `docs/research/checkmate-win-conditions.md`. |
| Class-based skill pools (Champion's class determines drafting) | — | Session 10 | Restricts strategy freedom for minimal complexity reduction. Contradicts blank-slate Champion design. |
| Champion pre-naming (Blacksmith, Necromancer, etc.) | — | Session 10 | Champions are blank slates — identity emerges from equipped skills. |
| Information/scouting skill (reveal Money count) | — | Session 10 | Irrelevant in a perfect-information game. |

---

## Reopened / Under Review

Mechanics that were previously withdrawn but have been re-evaluated.

| Mechanic | Source OQ | Reopened | Reason |
|----------|-----------|----------|--------|
| Hex grid | OQ-42 | Session 8 | Original Session 1 rejection was by omission: ADR-001 confirmed "grid over card-fighter," not "square over hex." Hex IS a grid variant. Trigger `/research hex vs square grid in tactical games` before scheduling a test stack. |

---

## Methodology / Design Decisions (no single mechanic)

Decisions about *how* we design or test, not specific mechanics. These shape what enters the tables above.

| Decision | Source OQ | Recorded | Reason |
|----------|-----------|----------|--------|
| Defense / Armor's role — diagnosis: late-game survival tax | OQ-11 | Session 23 (2026-05-30) | Three diagnoses tested for the late-game Armor problem. **A — Money curve too steep**: KILLED. Starving Money weakens skills as primary damage and removes fire-and-think tension. **B — HP too thin**: KILLED. Catalogue audit shows no 2-damage skills (cheapest 2-damage path costs 0/2/4/6/8 Money); raising HP just shifts the bottleneck to healing. **C — Armor's *shape* is wrong, functions as late-game survival tax / mandatory upkeep**: CONFIRMED. User verbatim: *"i 100% agree that armor is like the tax you have to pay. that they are the mandatory upkeep of pieces in the endgame."* This is the diagnosis anchor — not a candidate fix. Future Armor proposals must answer "does this turn defense into a strategic choice instead of upkeep?" Full discussion: `docs/research/path-y-defense-redesign.md`. Cross-link: backpocket "Armor — Current-Role Audit" entry. |
| Pole framing: parallel design tracks (Pole A pre-game-draft, Pole B per-turn-draft) | OQ-61 / OQ-63 | Session 23 (2026-05-30) | While core identity is unsettled, design proceeds along two parallel tracks rather than tweaking variables inside one rule set. **Pole A** = current game (skills equipped at start, fixed for the game). **Pole B** = radical alternative (skills added during play; reusable while equipped; max 12 equipped per player; shared action slots between movement and drafting; no Money activation gate; effectively infinite skill pool). User: *"i do not wanna abandon somethign that might serve a different game feel."* Pole B is an experiment-that-could-replace, not a committed parallel-forever design. Stack L (Pole B per-turn-draft prototype) is the new Active stack; Stack H deprioritised to Queued. Cross-pole fixing methodology (test fixes once or per pole?) tracked as OQ-63 — resolved on first encounter. Pairs with Principle 7 (fundamental shifts while core unsettled). Full discussion: `docs/research/path-y-defense-redesign.md`. |
