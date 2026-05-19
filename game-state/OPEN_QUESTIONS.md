# OPEN QUESTIONS

*Living document — update as questions arise or get resolved.*

---

## Resolved / Partially Resolved

### OQ-1: Board Size — PARTIALLY RESOLVED
**10x10 confirmed as viable.** Both players said size was generally right. Early game still has too much empty space (first contact at ~Round 7). 12x12 likely worse. Hex untested.
- **Remaining question**: Would 8x8 be better? Deferred to Layer 5 testing.
- **Status**: 10x10 is the baseline for Layers 1-3. Board size change is Layer 5.

### OQ-3: Movement-Action Link — PARTIALLY RESOLVED
**Unlinked works and is liked.** May be superseded by Unified AP system (Layer 4), which merges the concept entirely.
- **Status**: Unlinked is the baseline. AP system is a Layer 4 test.

### OQ-5: Skill Path Blockage — RESOLVED
**Blocked by all pieces.** Both players confirmed.

### OQ-9: Starting Piece Placement — RESOLVED
**Current placement is balanced.** Both players confirmed.
- **Note**: Becomes important again if piece count or board size changes (Layer 5). On 8x8 with fewer pieces, placement matters a lot more — will need re-evaluation.

### OQ-13: First Player Advantage — PARTIALLY RESOLVED
**No strong advantage observed** in Playtest 1. Low priority.

### OQ-19: Endgame Acceleration — RESEARCHED (Session 11, concrete trigger unchanged)
- YINSH-inspired capture penalty **withdrawn** — creates unfair asymmetry when one player has no Guards left.
- Damage escalation after Round X **deferred** — feels arbitrary.
- Checkmate-style win condition **killed** (Session 11) — verification burden too high for our game. Too many defensive options to formally prove "100% lost" at the table. Research: `docs/research/checkmate-win-conditions.md`.
- **Playtest 2 update**: Game ran 4+ hours; natural finish estimated ~R40. Layers 1–3 have not fixed game length. The problem appears to be piece durability (first kill at R26) + long think times + positional deadlock, not just Rune speed.
- **New leading candidate**: King Lifetime HP — separate irreversible damage track on the King. Healing delays but cannot prevent death. Creates natural game clock without verification burden. See `docs/backpocket.md`. Only deploy if King specifically proves unkillable through armor/heal loops.
- **Secondary candidates**: Fewer pieces (Stack D), smaller board (Stack D), anti-repetition rule (threefold repetition = draw).
- **Anti-repetition rule (from research)**: If same board position occurs 3 times → draw. Prevents infinite stalling loops. Simple, no blame assignment needed. Add to any layer where game-length is being tested.
- **Re-entry trigger**: After Stack A playtest — if first Champion kill is still past R20, endgame acceleration becomes Priority 1. If first kill moves earlier, deprioritize.
- **Status**: Researched. Checkmate killed. King Lifetime HP staged in backpocket. Trigger unchanged: Stack A kill timing data.

### OQ-25: Economy Skills as Skill Slots — CLOSED (Session 8)
- With only 2 skill slots per Champion, equipping an economy skill makes that piece a "one-trick pony" — sacrifices combat versatility entirely.
- **Session 8 verdict**: Closed permanently. 2-slot scarcity makes this unworkable; performance-based economy is also closed. The core idea (ultra-defense → late-game payoff strategy) is interesting as a post-v1 expansion variant, not a core system change.
- **Status**: Closed. Could become an expansion variant once core game is perfected.

---

## Critical (blocks next playtest)

### OQ-17: Rune Start Rate — RESOLVED (Layer 1, Playtest 2, 24.04.2026)
**Start with 6 Runes, +2 gain/turn, scaling +1 every 5 rounds. ACCEPTED.**
- Both players used skills from Round 1. Dead opening eliminated.
- Skill Slots became the action limiter in early game — as intended.
- Economy is fast enough without being overwhelming.
- **Status**: Accepted. Carry forward into all future layers.

### OQ-18: Health System — SCRAPPED (Layer 2 topic TBD)
**3 HP for Champions/King was the proposed test, but it has been scrapped before testing.**
- Reason: 3 HP would make the game even longer (first kill was already at R26 with 2 HP). Guards at 2 HP with Champions at 3 HP creates an artificial rank — Guards become "easy kills" by design, which doesn't match the design intent.
- The underlying problem (combat too coarse, Injured state bypassed by standard attacks) needs a different solution.
- **Status**: Scrapped. Layer 2 topic needs rethinking. See NEXT_STEPS.md.

### OQ-20: Shadow Shift Balance — PARTIALLY RESOLVED
**Shadow Shift now uses default Range 2** (not a special fixed cap). No separate range exception needed.
- The prior "Range 3" fix was a misunderstanding — Shadow Shift simply uses the default Skill Range (2), which is now explicitly defined in the rules.
- Side-note alternative (Range+1) not needed: default Range 2 is correct and consistent.
- **Status**: Resolved as Range 2 (default). Carry forward into all layers. Watch whether Range 2 feels limiting in practice.

### OQ-21: Bodyguard Rule — SIGNIFICANTLY UPDATED (Playtest 3, Stack B may be obsolete)
**Adjacent to defender only.** Guard takes the damage (this is already the baseline rule — only the adjacency requirement changes in Stack B).
- Full standalone rule sheet: `docs/test-scenarios/stack-b-guards/stack-b-bodyguard-fix.typ`
- **Hypothesis**: Bodyguard triggers more frequently. Guards become genuinely useful as screens.
- **P2 update**: Bodyguard triggered 2x in Playtest 2 under baseline adjacency rule.
- **P3 update (Session 15)**: Elias Q15 — "with less standoff it (Bodyguard rule) happened way more often" + repositioned **Yes**. **Bodyguard organically activated under Stack A nerf** without any adjacency-rule change. The dead-Bodyguard problem may be a *symptom* of the standoff problem, not an independent issue. Stack B may be solving an already-solved problem.
- **Attacker movement on intercept**: RULED — attacker moves 1 tile toward target (stops before Guard). Applied to baseline.
- **Contingent variant (Session 8)**: If "defender-only" overshoots (triggers too often / breaks positioning), test "attacker-only" adjacency as a middle ground between "both" (too restrictive) and "defender-only" (possibly too permissive).
- **Status**: De-prioritise Stack B. Re-evaluate after one more experienced-player playtest (Stack A G2). If Bodyguard keeps triggering organically across multiple games, close OQ-21 as resolved-by-side-effect.

---

## New (from Playtest 2, 24.04.2026)

### OQ-29: Move Slots per Piece — RESOLVED (applied to baseline)
**Each piece may only be moved once per Movement Phase.** Ruled mid-game Playtest 2. Now explicit in all rule sheets.

### OQ-30: Default Skill Range — RESOLVED (applied to baseline)
**Default Skill Range = 2.** Range 0 = self, Range 1 = adjacent, Range 2 = 2 tiles along Skill Path. Now defined in all rule sheets.
- Adjacent-while-injured edge case: **resolved** — minimum Range is 0 (self). A piece at Range 0 may always target itself.

### OQ-31: Focus Strike / Blade Call Scope — RESOLVED (applied to baseline)
**Focus Strike buffs the *next* skill used by any of your pieces that turn. Blade Call buffs any *one* Strike skill used by any of your pieces that turn (before or after).** This is the key timing distinction: Focus Strike must come first, Blade Call can be declared retroactively. Each Blade Call activation boosts exactly one Strike, then is spent.

### OQ-32: Movement-via-Skills Damage — RESOLVED (applied to baseline)
**Skills that move pieces do NOT deal damage.** General rule now in Skill System section of baseline.

### OQ-33: Blade Tempest Self-Affect — RESOLVED (applied to baseline)
**Blade Tempest does NOT affect the caster.** Only the target takes 1 DMG; only pieces adjacent to the *target* are pushed. Now in skill text.

### OQ-34: Rune Theft Balance — INCONCLUSIVE (Playtest 3, reframe)
**Rune Theft has two operating modes — verdict depends on which one shows up in a given game.**
- **Mode A** — opponent at 0 Runes: Rune Theft is functionally a normal-range Strike skill (1 DMG, no theft effect). Comparable to Lance Thrust at full board range. Not dominant.
- **Mode B** — opponent has Runes, mid-game: cheap damage + opponent disabling. Tempo weapon. **Disable value is time-dependent**: early-game, stealing 1 Rune blocks a planned skill (~50% of a turn's gain — high impact); late-game, both sides roll major-combo Runes every ~2 turns, so −1 Rune barely dents the opponent (low impact). So Mode B itself splits into "early-Mode-B = punishing" vs. "late-Mode-B = mostly cosmetic."
- **Playtest 3 evidence**: Elias's late Rune Theft burst (R23–R24, 6 activations across 2 turns) happened after Mario was already mostly out of Runes — Mode A territory. Per user's reframe: "the late burst was forced by positioning, not strategic save." Does NOT confirm dominance.
- **Playtest 2 (P2)**: Earlier flagging was Mode B framing (mid-game, both with Runes).
- **Open question becomes**: in a typical experienced-player game, which mode dominates? If Mode B is rare because the economy keeps both players spending, Rune Theft is fine at cost 3.
- **Hypothesis (unchanged)**: Raise cost to 4 Runes if Mode B is prevalent in next experienced-player game.
- **What to watch in Stack A G2**: (a) does early-game Rune Theft block opponent's opener plans? (b) does late-game Rune Theft feel pointless or still tempo-relevant? If (a) → cost question reopens. If (b) is "pointless" → Rune Theft might need a redesign rather than a cost tweak (scaling effect, or theft amount tied to opponent pool).
- **Status**: Inconclusive. Continue monitoring in Stack A Game 2.

### OQ-46: Rune Cap — CLOSE FROM MONITORING (Playtest 3)
`[System: Resource Economy] [Affects: Skill System]`

**No hoarding observed in Playtest 3 — neither player accumulated unspent Runes meaningfully.** Both players' max unspent end-of-turn Rune count was small; both players said "always wanted more / balanced." Confirms G2 (encourage spending via attractiveness) is working. Close from monitoring unless a future game shows different behaviour.
- **Status**: Close from monitoring. No cap needed.

### OQ-35: Skill Pool Draft Variant — DEFERRED (concrete trigger)
**Alternative draft: each player drafts a pool of N skills first, then assigns skills to Champions.**
- Proposed by Elias. Potentially reduces "accidental" synergies from round-by-round assignment and increases strategic intent.
- Might make the draft feel more like deck-building and increase replayability.
- **Re-entry trigger**: Test after Layer 3 is accepted. Independent of combat/economy changes — can be the first non-combat test layer.
- **Status**: Deferred. Re-entry: post-Layer-3 acceptance.

### OQ-36: Flexible Piece Placement — DEFERRED (concrete trigger, bundled with OQ-48)
**Alternative: players choose starting positions within constraints** (e.g., back 2 rows) rather than fixed `--GGGGGG-- / --CCKCCC--` formation.
- Proposed by Elias. Could increase strategic variety and personalise openings.
- Risk: increases setup time and cognitive load pre-game. Also risk of infinite counter-positioning (player A places, player B reacts, repeat).
- **Session 8 note**: The first shuffling turns serve as an on-ramp for new players (learn mechanics before being punished). Removing them removes that buffer. Explore **reveal-style simultaneous pick** (both commit, then reveal) to avoid reactive loops. See backpocket.
- **Re-entry trigger**: Test after Layer 3 accepted, bundled with OQ-48. Independent of combat changes.
- **Status**: Deferred. Re-entry: post-Layer-3, bundled with OQ-48.

---

## High Priority (affects system interactions)

### OQ-2: Rune Economy Model
**Automatic confirmed.** Layer 1 accepted — automatic faster economy works well. Performance-based is still in the backlog but not needed now.

### OQ-4: Skills Per Piece Per Turn
**Uncapped for now.** Blade Call burst is a skill-specific balance issue, not a structural one.

### OQ-6: Skill Slot Cost (Small vs Big)
*(Unchanged — not directly tested.)*

### OQ-8: Rune Cap
**No cap initially.** Players naturally spent down in Playtest 2 as well. Not a problem yet. Re-evaluate after Layer 2.

### OQ-22: Defensive Skill Viability
**Improved in Playtest 2.** Armorsmith, Rust Shield, Field Medic all used meaningfully. Injured state now relevant ("Often"). Layer 2 (3HP) should deepen this further. Re-evaluate after Layer 2.

### OQ-23: Move Slot Count
**May be superseded by AP system (Layer 4).** If staying with current turn structure, test 3 Move Slots. If moving to AP, this question dissolves.

### OQ-26: AP System Piece Freedom — NEW (from ADR-002 feedback)
**How many AP can one piece receive per turn?**
With unified AP, a single piece could spend all 3 AP on movement and rush the King. Four constraint models proposed:
- **Model A**: 1-skill-per-piece limit. Doesn't prevent movement rushing.
- **Model B**: 1-AP-per-piece limit. Forces spreading across army. Most restrictive.
- **Model C**: Uncapped normally, multi-AP unlocks when ≤2 pieces left (comeback mechanic).
- **Model D**: Max 2 AP per piece, 3rd must go elsewhere. Hybrid.
- **Status**: Test all models in Layer 4. Start with Model D as default, test others as variants.

---

## Medium Priority (balance and polish)

### OQ-10: Injured Penalty Severity — INCONCLUSIVE (Playtest 3, blocked by rule clarity)
- **Speed penalty is Guard-only**: Champions/King are already Speed 1, so Injured's "Speed capped at 1" does nothing for them. The only Champion/King Injured effect is Range 2+ → Range 1.
- Question: Is that range reduction punishing enough for Champions/King? Or do Injured Champions feel largely unaffected until they're killed?
- Potential alternative effects: +1 Rune cost on all skills when Injured; −1 Skill Slot when Injured; or something else.
- **P3 update (Session 15)**: Elias Q1 "Sometimes" — injured pieces had meaningful time before dying. Q16 ("did Injured Champions feel weaker?") **blank**. The rule itself is also ambiguous: side-note #5 (Lance Thrust + Injured Range penalty) cost Elias an entire turn (R22) — he stalled rather than risk a misplay. Until the rule is unambiguous, "is the penalty enough?" can't be evaluated.
- **Action required**: resolve the Lance Thrust + Injured rule ambiguity in baseline before next playtest. Then re-evaluate severity question.
- **Status**: Inconclusive. Rule clarity is the blocker — fix that first.

### OQ-11: Armor Cap — CONFIRMED WORKING (Playtest 3, RPS loop functions)
- Keep at 3.
- **P3 update (Session 15)**: Mario stacked Armor heavily (~20 Armorsmith activations). Elias drafted Armor Breaker as the counter and used it effectively (multiple R14, R15, R20, R21, R23 activations). The RPS loop **functions as designed**: Armor stacking exists, the counter exists, the counter wins when picked. Elias Q17: Armor "slightly extended / well balanced; start: only armor stacking, afterwards prevented instant death + gave presence." This is the intended Armor experience.
- Mario's heavy stacking was draft-and-experience-driven (no Rust Shield in his pool, no presence pieces, reactive defensive learning curve), not an Armor-too-cheap signal.
- **Status**: Confirmed working at cap 3. Watch for re-emergence with two experienced players (does Armor still meaningfully resist a competent Armor Breaker draft?). Close monitoring otherwise.

### OQ-12: Skill Catalogue Completeness — RESEARCHED (Session 11)
**Is the catalogue large enough and balanced enough across categories?**
- Research (`docs/research/skill-catalogue-balance.md`): Minimum 25-35 skills needed for meaningful draft variety. At 15, most skills are picked by someone — no real draft tension.
- Playtest 2 data shows: all 3 Shield skills heavily used (category exhausted in variety, not copies), Focus Strike is must-pick (Mystic monopoly), Blade Call never-picked in P2 meta.
- **Real problem**: Too few distinct strategic identities within Shield (3 passive buffs) and Mystic (1 must-pick + 1 never-pick). Not a "nobody picks defense" problem.
- **Design principle**: New skills must pass the sente test (create situations opponent must respond to). Passive-only skills don't create interesting decisions.
- **10 new skill candidates staged** in `docs/backpocket.md`: Thorn Armor, Runic Ward, Bulwark (Shield); Bind, Energize, Skill Drain (Mystic); Mini-Step, Swap Step, Ram (Move); Gravity Well (Move, needs design work).
- **Target**: ~25 skills total (current 15 + 10 new). Distribution: ~9 Strike / 6 Shield / 5 Move / 5 Mystic.
- **Trigger for expansion**: Stack F or dedicated session after Stack A/B confirm combat balance is stable. Do not expand mid-combat-testing.
- **Status**: Researched. Candidates staged. Expansion gated on Stack A/B results.

### OQ-24: Skill Card/Rule Clarity
**Review all skill descriptions for clarity before next playtest.**

---

## Low Priority / Future

### OQ-14: Draw Conditions
*(Unchanged.)*

### OQ-15: Terrain System
**Confirmed as overhead complexity.** Removed in A+ direction. Reversible — can return as "map variant" expansion if missed.

### OQ-16: Skill Drafting Fairness
- Fair but "decides a lot." Future question: is draft too deterministic?

### OQ-27: Piece Count and Ratio — DEFERRED (concrete trigger)
**What's the right number and ratio of Champions to Guards?**
- Current: 5 Champions + 6 Guards + 1 King = 12 per player.
- Proposed (Layer 5): 3 Champions + 3 Guards + 1 King = 7 per player. But is 3:3 (+ King as 4th skill-carrier) the right ratio?
- Guards surviving into endgame should be viable and cool, not a forced-first-kill obligation.
- **Session 8 note**: Bundle with board size (OQ-1). Both are "scale it down" changes. BUT: fewer pieces + smaller board automatically makes Runes worth more and skill slots more scarce — risks damaging game experience. Frame as "same feel, shorter games" — must preserve per-turn decision depth.
- **Re-entry trigger**: After Layer 2/3 playtest — if first Champion kill is still past R15, piece count + board size becomes Priority 1. If first kill moves to R10–15, deprioritize to post-v1.
- **Status**: Deferred. Re-entry: post-Layer-2/3 based on kill timing data.

### OQ-28: Restricted vs. Free Movement — PARKED INDEFINITELY (v1 out of scope)
**Should pieces move freely (any path ≤ speed) or in a straight line only (Skill Path model)?**
- Current baseline: free pathing (any route up to speed, cannot pass through pieces).
- Hypothesis: Restricting movement to straight-line-only would make Move skills meaningfully stronger.
- **Session 8 verdict**: Parked. Not solving a real problem — no one has complained about movement feeling too free. Restriction punishes Guards (Speed 2 in a straight line is terrible for navigation) and is meaningless for Champions (Speed 1 = 1 tile in any direction regardless). Dissolves entirely if hex grid (OQ-42) is adopted (hex has no diagonal/orthogonal distinction).
- **Status**: Parked indefinitely. v1 out of scope. Only revisit if Move skills feel weak AND hex is not adopted.

---

## New (from Session 7, 2026-04-27)

### OQ-37: Standard Attack Damage — CONFIRMED (Playtest 3, accept into baseline)
**Standard attacks deal 1 DMG instead of 2.**
- Playtest 3 (L2G1): first Champion kill **Round 11** (vs P2's R26). Both players Q2 "Felt right." Combat feel "Better / Much Better" (Elias). Standoff dissolved. No "1 DMG too weak" complaints.
- Risk that Guards become hard to remove — **not observed**. First Guard kill R10. Guards behaved as screens rather than damage sponges.
- **Status**: **Accepted into baseline.** Move to `mechanics-evaluated.md` accepted-in-baseline. Update `ruleset-baseline.typ` so 1 DMG is the canonical Standard Attack rule.

### OQ-38: Multi-Champion Combo Bonus — READY TO TEST (Layer 2, Game 2)
**Should coordinated multi-Champion attacks on the same target get +1 DMG on the second+ hit?**
- When 2+ different Champions use skills that target the same enemy piece in the same turn, each skill after the first deals +1 DMG.
- Buff skills targeting your own pieces (Focus Strike, Blade Call, Rust Shield, etc.) do NOT count.
- Incentivises "gang-ups" — multi-Champion coordination that is currently rare because single-Champion plays are easier.
- Blade Call stacks with combo bonus (separate effects).
- **Risk**: could make multi-Champion combos the only viable strategy. Mitigated by naturally high setup cost (LoS constraints on crowded board).
- **Status**: Ready to test in Layer 2 (Game 2, on top of OQ-37 nerf). See ADR-003.

### OQ-39: Shared-Puzzle Design Direction — OPEN
**Should the game lean into "rewarding cleverness / mutual epistemic exploration" as a deliberate design direction?**
- Playtest 2 emergent behaviour: both players collaboratively analysing board states felt more engaging than pure competition.
- Research (`docs/research/cooperative-feel-competitive-games.md`) shows this is a known phenomenon in perfect-information games. Engineered deliberately in Onitama, Twilight Struggle, Go, Tak.
- Key finding: the shared-puzzle feel comes from perfect information + depth, not from removing competition. Winning = "I found the better solution," not "I crushed you."
- Not a mechanical commitment yet — more a framing/identity question that influences how rules are written, skills are named, and the experience is presented.
- Design principles agreed (see ADR-003): expand viable strategies, reward cleverness visibly, let both players appreciate each other's plays.
- **Status**: Open. Principles agreed, not yet a mechanical commitment. Revisit after Layer 2 data. See also OQ-51 (mechanical levers for rewarding clever plays).

### OQ-40: Standoff / No-Man's-Land Problem — CONFIRMED RESOLVED (Playtest 3, primary lever worked)
**The standard attack nerf (OQ-37) dissolved the standoff in Playtest 3.**
- Both players Q4: "Much less standoff" (Elias) / "?" Mario, but Mario didn't play P2 so has no comparison frame. Q5: both said "Not reluctant to move forward." Elias: "I knew I would not die immediately if I did."
- First Guard kill R10, first Champion kill R11. Forward movement was active from early rounds; not the 2–3 tile gap of P2.
- **Confirms primary research finding**: lower entry risk + sente design dissolves standoff. The combo bonus (Game 2) is no longer needed *to fix the standoff* — it remains worth testing for combo-ceiling reasons (OQ-38), but standoff is solved.
- **Sente skills** (Stack F) remain worthwhile long-term to keep standoff dissolved against more experienced opposition, but not urgent.
- **Status**: Confirmed resolved by Stack A Game 1. Sente design (Stack F) is no longer urgent. Watch for re-emergence with two experienced players.

### OQ-41: Game Length vs. Damage Nerf Tradeoff — PARTIALLY CONFIRMED (Playtest 3)
**1-DMG attacks did NOT noticeably extend the game.** Game ended at Round 24 (Mario surrendered) — comparable or shorter than P2 (~26 rounds when P2 had not yet finished). First Guard kill R10. No game-length blow-up from the nerf alone.
- Caveat: this was a draft-asymmetric game (Mario inexperienced) — pacing data is influenced by Mario's passive Armor stacking, not just the rule change.
- **Status**: Partially confirmed. Re-evaluate with two experienced players in Stack A Game 2.

---

## New (from Session 8, 2026-04-28 — migrated from baseline-rules/md-converted/Systems to Test.md)

### OQ-42: Hex Grid Board — REOPENED
`[System: Board/Spatial] [Affects: Skill System (push/pull/path geometry), Movement, Bodyguard]`

**Should we evaluate a hexagonal grid as an alternative to the square 10x10?**
- Listed as "Withdrawn" in mechanics-evaluated.md (Session 1, ADR-001), but the rejection was by omission — the ADR confirmed "grid over card-fighter," not "square over hex." Hex IS a grid.
- **Potential upside**: Eliminates diagonal ambiguity in Skill Path (queen-moves are cleaner on a hex grid — all 6 directions are equidistant). Push/pull becomes geometrically unambiguous. Bodyguard adjacency clearer.
- **Potential downside**: Completely changes spatial intuition. Piece counts, board size, and formation thinking all need reevaluation. All push/pull/range rules must be reimagined. High design risk.
- **Research needed**: How do published tactical games (Hive, Summoner Wars hex variants, BattleCON) handle hex vs. square? What does hex do to combo geometry?
- **Status**: Reopened. Add to backlog. Trigger `/research hex vs square grid in 2-player tactical board games` before any test layer is proposed.

---

### OQ-43: CR-Style Draft Picks — PARKED INDEFINITELY (v1 out of scope)
`[System: Skill Drafting] [Affects: Skill System (combo visibility during draft)]`

**Should skill drafting use "one for me, one for you" alternating pairs instead of the current alternating singles?**
- Current: P1 picks 2 skills (assigns freely), then P2, repeating.
- CR style: P1 picks 1, P2 picks 1, P1 picks 1, P2 picks 1 — stricter interleaving.
- **Session 8 verdict**: Parked. Restricts free strategy picking — with a small catalogue, counter-picking leads to "correct" picks that reduce variety. Feels more like a tournament variant than the main game. Only relevant with a much larger catalogue (20+ skills).
- **Status**: Parked indefinitely. v1 out of scope. Variant/expansion material.

---

### OQ-44: Ban Phase in Skill Draft — PARKED INDEFINITELY (v1 out of scope)
`[System: Skill Drafting] [Affects: Skill System]`

**Should players ban 1–2 skills each before picks begin?**
- Original proposal from Systems to Test (pre-project).
- **Session 8 verdict**: Parked. This is from an older game version with unique Champions that had fixed skills — players chose Champions from a pool. The current system (shared skills, free assignment) makes banning less meaningful. Needs 20+ skills AND a different draft model to be viable.
- **Status**: Parked indefinitely. v1 out of scope.

---

### OQ-45: Starting Player Decision — PARKED INDEFINITELY
`[System: Turn Structure] [Affects: Resource Economy (bid costs Runes)]`

**How should the first player for the first round be determined?**
- Current: mutual agreement / not specified.
- Options: coin flip (contradicts no-luck), hidden Rune bid, mutual agreement.
- **Session 8 verdict**: Parked. No first-player advantage observed in 2 playtests. If it ever surfaces over many games (accounting for skill level), the fix is Go-style komi (P1 starts with fewer Runes as compensation), not a bidding war.
- **Status**: Parked indefinitely. Only revisit if consistent first-player win rate observed across many games.

---

### OQ-47: Performance-Based Rune Gain — CLOSED (Session 8)
`[System: Resource Economy] [Affects: Combat, Progression]`

**Should Rune gain be tied to board performance rather than (or in addition to) automatic time-based scaling?**
- Options from Systems to Test: Capture bonus (+2 Runes), occupy centre tile 1 full round (+1 gain), King advanced 2 rows above start (+1 gain), per 2 pieces taken (+1 gain), max gain cap 5.
- **Session 8 verdict**: Closed permanently. Performance-based income forces players toward whichever strategy earns Runes fastest, constraining creative expression and making every game feel the same ("you must play this way to gain Runes"). Auto-economy is strategy-neutral — supports aggro, control, and defensive archetypes equally. The combo bonus (Layer 2) is the correct lever: it rewards *cleverness of execution* (hard to set up, high coordination cost), not *which action you performed*. This avoids the KPI problem.
- **Status**: Closed. Not revisiting.

---

### OQ-48: Piece Placement Order — DEFERRED (concrete trigger, bundled with OQ-36)
`[System: Skill Drafting] [Affects: Movement, Combat]`

**Should players place pieces on the board after the skill draft (informed by loadout) rather than before it?**
- Current: fixed starting formation (Guards in front row, Champions + King in second row, middle of the board).
- Proposed: after equipping a Champion with skills, place it on the board before picking the next Champion's skills. Or: equip all, then place freely within the starting zone.
- Potential upside: Draft-informed placement. A Champion with ranged skills can start on a flank; a melee Champion can start centre.
- Connects to OQ-36 (flexible piece placement — Elias suggestion) and OQ-9 (placement is "balanced" at current fixed positions).
- **Session 8 note**: Bundled with OQ-36. Explore reveal-style simultaneous placement to avoid infinite counter-positioning. See `docs/backpocket.md`.
- **Re-entry trigger**: Test after Layer 3 accepted, bundled with OQ-36. Independent of combat changes.
- **Status**: Deferred. Re-entry: post-Layer-3, bundled with OQ-36.

---

### OQ-49: Skill Path Obstruction Model — DEFERRED (concrete trigger)
`[System: Skill System] [Affects: Combat, Bodyguard]`

**What should count as an obstruction to the Skill Path?**
- Current (baseline): **all pieces** (ally and enemy) block the Skill Path.
- **Idea 1**: Only opponent pieces block. Own pieces are transparent to your skills.
  - Upside: Removes the need to reposition allies to clear LoS. More skills usable per turn.
  - Risk: Enables "turtle" formation — cluster all pieces together, use skills from safety. Withdrawn in Session 7 for this reason.
- **Idea 2**: Only opponent Guards block (not Champions/King, not own pieces).
  - Upside: Guards become active LoS controllers. Defensive formation matters more.
  - Risk: Creates "Guard wall" dominant strategy — stack Guards to block all enemy skill paths.
- **Note**: Idea 1 was explicitly discussed and withdrawn in Session 7 (creates turtle meta). Idea 2 not yet evaluated.
- **Re-entry trigger**: After Layer 2 — if own-piece LoS blockage is consistently frustrating (players repeatedly can't use skills because their own pieces are in the way), test Idea 2. Otherwise park.
- **Status**: Deferred. Idea 1 withdrawn. Idea 2 conditional on Layer 2 frustration data.

---

### OQ-50: Minor/Major Skill Slot Cost — DEFERRED (concrete trigger)
`[System: Skill System] [Affects: Resource Economy, Progression]`

**Should skills cost different numbers of Skill Slots (minor = 1, major = 2) rather than all costing 1 slot?**
- Current: all skills cost 1 Skill Slot + their Rune cost.
- Proposed: "minor" skills (simple effects) cost 1 slot; "major" skills (complex/high-impact) cost 2 slots.
- **Session 8 discussion**: With 2 slots/turn, 2-cost skills are traps (no one takes them). BUT — if we design intentionally powerful "ultimate" skills worth the commitment, the mechanic becomes interesting: one player strategically saves toward a single game-turning play. Could serve as endgame acceleration (powerful late-round plays that break deadlock). Must ensure that in later rounds (3+ slots), these don't become spammable.
- **Re-entry trigger**: Design 2-3 candidate "ultimate" skills (2 slots + high Rune cost) that would make the mechanic worth testing. Only evaluate as part of Layer 4+ (when Skill Slots expand to 3/turn). Connected to endgame acceleration (OQ-19).
- **Status**: Deferred. Pre-work needed: design ultimate skill candidates first.

---

### OQ-51: Mechanical Levers for Rewarding Clever Plays — RESEARCHED (Session 11)
`[System: Cross-system] [Affects: Combat, Skill System, Progression]`

**Beyond the combo bonus, what other mechanical levers can reward clever play?**
- The combo bonus (Layer 2) rewards multi-Champion coordination. But it's one lever. What else can the game use to make cleverness *mechanically* rewarded (not just aesthetically satisfying)?
- **Existing levers**: Combo bonus (+1 DMG for coordination), Blade Call (+1 DMG for buff setup), Focus Strike (+1 Range for buff setup).

**Research findings (Session 11 — `docs/research/mechanical-reward-clever-play.md`)**:
Four mechanical patterns identified from published games:
1. **Threat = reward (Hive, Chess)**: Creating an unavoidable threat forces opponent into reactive play, draining their action economy. Tempo gain without resource gain.
2. **Environmental multiplier (Into the Breach)**: Board state amplifies your action (knockback into hazards = free damage). Situational, single-use, not compounding.
3. **Restriction as reward (Hive pinning, Go territory)**: You don't gain stats — you remove opponent options. Positional "captures."
4. **One-time action economy (anti-snowball principle)**: Give tempo advantages (extra actions THIS turn), not resource advantages (more Runes forever).

**Evaluated candidates**:

| Candidate | Verdict | Reasoning |
|-----------|---------|-----------|
| **Cascade trigger** (+1 Skill Slot on kill, any method) | **Staged in backpocket** | Anti-snowball (one-turn, still costs Runes). Rewards finishing setups. If too easy via standard attacks → restrict to skill-kills. If never used → remove. |
| **Pin/Threatened** (2+ enemy LoS = can't move) | **Staged in backpocket** | Restriction-as-reward pattern. Risk: oppressive. Counterplay: Move skills as escape. Needs own test layer. |
| **Collision damage — universal** (push into piece = 1 DMG) | **Backpocketed (conditional)** | Only test after standoff dissolved. Risk: amplifies keep-away if standoff persists. |
| **Collision damage — skill-specific** ("Ram" skill) | **Skill catalogue candidate** | Opt-in via draft. Safer to test than universal rule. Design when catalogue expands. |
| **Positional payoff** (forward deployment bonus) | **Deferred to OQ-40 / Topic 3** | Overlaps with "rewarding risky positioning." Explore there. |
| Coordinated movement bonus (−1 Rune if pieces move same zone) | **Killed** | Too easy to trigger accidentally. Doesn't reward cleverness. |
| Breakthrough bonus (+1 Slot on first Champion hit) | **Subsumed into cascade** | Arbitrary "first hit" trigger less elegant than general "on kill." |

- **Design constraint**: Must reward *cleverness of execution* (hard to set up, requires multi-turn planning), not *outcomes* (easy to measure, creates snowball). The KPI principle from ADR-003 applies. Anti-snowball key: one-time tempo advantages, not permanent resource engines.
- **Status**: Researched. Three promising candidates staged in backpocket. Next step: test cascade trigger in Stack F (or earlier if opportunity arises). Pin/Threatened needs its own layer. Collision damage gated on standoff resolution.
- **Connected to**: OQ-39 (shared-puzzle), OQ-19 (endgame acceleration via checkmate), OQ-50 (ultimate skills), OQ-40 (standoff — positional payoff deferred there).

---

## New (from Playtest 3, 2026-05-17 / Session 15)

### OQ-54: Lance Thrust — Should Its Text Say "Adjacent" Instead of "Range−1"? — OPEN
`[System: Skill System] [Affects: Skill clarity, Injured interaction]`

**Lance Thrust's effect text reads "Target within Range−1 takes 1 DMG." Under the accepted Range ruling, this makes it a Range 2 skill with a −1 modifier (effective Range 1 = adjacent). The question is whether the text should be rewritten to say "adjacent" explicitly.**
- Argument for rewriting to "adjacent": clearer at the table, no mental arithmetic, consistent with how Field Medic uses "adjacent."
- Argument for keeping "Range−1": preserves the design signal that the reduced range is a deliberate tradeoff (hence the lower cost of 2 Runes), not just a fixed property. Keeps the modifier language visible.
- Injured interaction under current ruling: Injured reduces Lance Thrust to Range 0 (self) — unusable against enemies. This cost Elias a full turn in Playtest 3 (R22).
- **Status**: Open. Do not decide now — watch whether the Range−1 language causes confusion in further playtests. If it does, rewrite to "adjacent."

### OQ-55: Blade Call — Broader Skill Interaction? — OPEN
`[System: Skill System] [Affects: Skill combo depth, Mystic category identity]`

**Blade Call currently boosts one Strike skill by +1 DMG. Could it interact with a wider range of skills — e.g. also boosting the Armor gained by Shield skills (+1 Armor instead of +1 DMG)?**
- The core question: should Blade Call become a general "amplifier" for any skill, or stay as a Strike-specific damage booster?
- A broader Blade Call would deepen the Mystic category's identity as a "combo enabler" — using Mystic skills to amplify whatever strategy you're running that turn, not just the Strike path.
- Risk: if Blade Call can amplify Armor gain, it becomes a universal "make your best skill better" button — possibly too flexible, crowding out other Mystic skills.
- Alternative framing: rather than changing Blade Call, introduce a second Mystic skill that amplifies Shield/Move skills (a separate amplifier archetype).
- **Connected to**: OQ-12 (skill catalogue completeness — Mystic category currently has too few distinct identities), OQ-51 (mechanical levers for clever play).
- **Status**: Open. No action until skill catalogue expansion is unblocked (post-Stack A/B combat confirmation). Justification required before any change: what specific problem does broader Blade Call fix?

### OQ-52: Centre of the Board Has No Attractor — OPEN
`[System: Board/Spatial] [Affects: Movement, King role, Opening dynamics]`

**The centre of the board is mechanically inert and players naturally flank-drift at the start.**
- Three reinforcing causes (Playtest 3): (a) starting formation crams pieces into centre 6 columns of 10-wide board → opening play is about spreading to flanks; (b) opponent pieces are clustered in the centre → engaging centre means engaging strongest concentration first; (c) King is centre-positioned but functionally a back-row skill carrier → "keep safe" pulls it backward.
- Centre is doubly anti-attractive in the opening (cramped + dense with enemies). Nothing rewards being there.
- **Designer constraint (explicit)**: NOT introducing a "queen" piece that pulls action centrally — the lever must be board-geometry, attractor mechanics, or formation, not piece power.
- **Solution space (to brainstorm, not decide here)**: central rune/scoring tile; contested resource at centre; narrower board (8×10 — see backpocket); formation rework that opens centre lanes; or accepting it and designing around it.
- **Connected to**: OQ-53 (King-as-target), OQ-1 (board size — partially resolved, but 8×10 not yet evaluated), OQ-40 (sente skills assume threatenable territory).
- **Status**: Open. Real found issue from Playtest 3.
- **Re-entry trigger**: Address before or during Stack F (Cleverness II), since sente skills assume there are valuable squares/positions to threaten — without a centre attractor, sente threats may stay at the flanks.

### OQ-53: Attrition vs. Regicide — Should the King Be a Real Target? — OPEN
`[System: Win condition / Strategic texture] [Affects: King role, Pacing, Endgame, Sente design]`

**The formal win condition is King capture, but the real victory path is attrition. The game is currently played as "wear down the army," not "threaten the King."**
- Playtest 3 evidence: Mario surrendered when his last Champion died, not when his King was threatened. Elias never targeted the King — he ground out the army. King capture would have been an inevitable consequence, not a parallel strategic axis.
- Both players' Kings barely moved (side-note #6): Mario's King "mystic skill slave," Elias's King moved only at game end out of necessity.
- **Designer's intent (Session 14)**: The King should be a real threatenable target — "getting the King to be an active part of the game" is when the game becomes "real fun."
- **Alternate (or compounding) cause to consider**: this might also be a *player-strategy* artefact — both players see attrition as the easier and lower-risk path, so neither pushes for "risky sniping the King." It may not be purely a mechanical lack of incentive; it may also be that the mechanical incentive *exists* but is dominated by the easier attrition path. Brainstorm should distinguish: is the attrition default a mechanics problem, a player-knowledge problem, or both?
- **Critical clarification**: King Lifetime HP (in backpocket) does NOT fix this alone. Adding HP to a static piece just makes a static piece harder to kill. The lever is *making the King participate*, not *making the King durable*.
- **Solution space (to brainstorm, not decide here)**: starting-formation swaps that expose the King (see backpocket); central-attractor mechanics that pull the King forward (overlaps with OQ-52); sente threat skills that target the King specifically; mobility/safety asymmetry that rewards King advancement.
- **Connected to**: OQ-52 (centre attractor), OQ-19 (endgame acceleration), OQ-51 (mechanical levers for clever play — sente skills are sharper when there's a high-value target to threaten).
- **Status**: Open. Real found issue from Playtest 3, designer-intent driven.
- **Re-entry trigger**: Brainstorm session before Stack F, OR a dedicated King-role design session.
