# OPEN QUESTIONS

*Live design questions only. Resolved/closed/scrapped/parked items: see `OPEN_QUESTIONS_ARCHIVE.md`.*

*Order: most actionable first. Each entry should have a status, a re-entry trigger (if deferred), and a short evaluation criterion.*

---

## Critical (blocks or shapes next playtest)

### OQ-21: Bodyguard Rule — DE-PRIORITISED (organic activation observed)
**Hypothesis**: Bodyguard triggers more frequently with adjacent-to-defender-only rule.
- Full standalone rule sheet: `docs/test-scenarios/stack-b-guards/stack-b-bodyguard-fix.typ`
- **P2 update**: Bodyguard triggered 2x in Playtest 2 under baseline adjacency rule.
- **P3 update (Session 15)**: Bodyguard organically activated under Stack A nerf without any adjacency-rule change. The dead-Bodyguard problem may be a *symptom* of the standoff problem (now solved). Stack B may be solving an already-solved problem.
- **Attacker movement on intercept**: RULED — attacker moves 1 tile toward target (stops before Guard). Applied to baseline.
- **Re-entry trigger**: Re-evaluate after Stack A G2. If Bodyguard keeps triggering organically, close as resolved-by-side-effect.

### OQ-34: Rune Theft Balance — INCONCLUSIVE
**Mode A** (opponent at 0 Runes): Rune Theft is a normal-range Strike skill (1 DMG, no theft effect). Not dominant.
**Mode B** (opponent has Runes): cheap damage + opponent disabling. Tempo weapon. Disable value time-dependent: early-game high impact, late-game low impact.
- **Hypothesis (unchanged)**: Raise cost to 4 Runes if Mode B is prevalent in next experienced-player game.
- **What to watch in Stack A G2**: (a) does early-game Rune Theft block opponent's opener plans? (b) does late-game Rune Theft feel pointless or still tempo-relevant?
- **Status**: Continue monitoring in Stack A G2.

### OQ-38: Multi-Champion Combo Bonus — READY TO TEST (Stack A Game 2)
**+1 DMG on the second+ Strike hit when 2+ different Champions target the same enemy this turn.**
- Buff skills targeting your own pieces do NOT count.
- Blade Call stacks with combo bonus (separate effects).
- Strike-only for Stack A G2 (decided Session 16). Cross-category scope reconsidered after G2 data.
- **Status**: Stack A Game 2.

---

## High Priority (affects system interactions)

### OQ-19: Endgame Acceleration — RESEARCHED, RE-ENTRY GATED
- Checkmate-style win condition **killed** (Session 11). Research: `docs/research/checkmate-win-conditions.md`.
- **Leading candidate**: King Lifetime HP — separate irreversible damage track. See `docs/backpocket.md`.
- **Secondary candidates**: Fewer pieces (Stack D), smaller board (Stack D), threefold repetition rule (anti-stalling, simple).
- **Re-entry trigger**: After Stack A G2 — if first Champion kill is still past R20, becomes Priority 1. If kill timing is healthy, deprioritize.

### OQ-26: AP System Piece Freedom — Stack 4 candidate
**How many AP can one piece receive per turn?**
Four constraint models proposed (A: 1-skill-per-piece; B: 1-AP-per-piece; C: uncapped, comeback unlock; D: max 2 AP per piece). Test all in Stack 4. Start with Model D as default.

### OQ-52: Centre of the Board Has No Attractor — OPEN
`[System: Board/Spatial] [Affects: Movement, King role, Opening dynamics]`
**The centre is mechanically inert; players naturally flank-drift at the start.**
- Three reinforcing causes: cramped opening formation, enemies clustered centre, King "stays safe" backward.
- **Designer constraint (explicit)**: NOT introducing a "queen" piece that pulls action centrally — the lever must be board-geometry, attractor mechanics, or formation, not piece power.
- **Solution space**: central rune/scoring tile; contested resource at centre; narrower board (8×10); formation rework; or accepting it.
- **Re-entry trigger**: Address before or during Stack F (Cleverness II), since sente skills assume threatenable territory.

### OQ-53: Attrition vs. Regicide — Should the King Be a Real Target? — OPEN
`[System: Win condition / Strategic texture]`
**Formal win = King capture; real victory path = attrition.**
- Playtest 3: Mario surrendered when last Champion died, not when King was threatened. Both Kings barely moved.
- **Designer's intent (Session 14)**: King should be a real threatenable target.
- **Critical clarification**: King Lifetime HP alone does NOT fix this — it makes the King durable, not participating.
- **Solution space**: starting-formation swaps (backpocket); central-attractor mechanics (overlaps OQ-52); sente threat skills targeting the King; mobility/safety asymmetry.
- **Re-entry trigger**: Brainstorm session before Stack F, OR a dedicated King-role design session.

### OQ-56: Draft Entry Complexity + Skill Permanence — OPEN
`[System: Skill Drafting] [Affects: New player onboarding, Mid-game adaptability]`
**Three linked problems**:
- **A — Draft entry complexity**: New players cannot evaluate skills during draft (no on-board experience).
- **B — Permanent draft decisions**: Mid-game realisation that loadout doesn't fit board state, no way to adapt.
- **C — In-game reference load**: Constant board↔skill-list switching. Cards on the table help reduce lookup friction but don't build the mental model.
- **Design constraint**: Solution must not reduce strategic depth for experienced players.
- **Solution candidates**: in-game skill switching; fewer/deeper skills; physical skill cards (UX fix, low risk); try-before-lock draft phase; simplified starter set.
- **Re-entry trigger**: After Stack A G2. UX fix (skill cards) can happen any time. Decision deferred until Stack A/B combat balance is confirmed.

---

## Medium Priority (balance and polish)

### OQ-2: Rune Economy Model
**Automatic confirmed.** Stack A baseline accepted. Performance-based is closed (OQ-47). Status: live but no action — kept open as the policy anchor.

### OQ-4: Skills Per Piece Per Turn
**Uncapped for now.** Blade Call burst is a skill-specific balance issue, not a structural one.

### OQ-6: Skill Slot Cost (Small vs Big)
*(Unchanged — not directly tested.)* See OQ-50 for the active proposal.

### OQ-8: Rune Cap — superseded
**No cap initially.** Players naturally spend down. See OQ-46 in archive (closed from monitoring P3). Kept open only as a watch-flag.

### OQ-10: Injured Penalty Severity — INCONCLUSIVE (rule-clarity blocker)
- Speed penalty is Guard-only (Champions/King are already Speed 1). The only Champion/King Injured effect is Range 2+ → Range 1.
- **P3 update**: Lance Thrust + Injured rule ambiguity cost Elias an entire turn (R22). Until rule is unambiguous, severity question can't be evaluated.
- **Action required**: resolve the Lance Thrust + Injured rule ambiguity in baseline before next playtest. Then re-evaluate severity.

### OQ-12: Skill Catalogue Completeness — RESEARCHED
**Real problem**: Too few distinct strategic identities within Shield (3 passive buffs) and Mystic (1 must-pick + 1 never-pick).
- **10 new skill candidates staged** in `docs/backpocket.md`.
- **Target**: ~25 skills total. Distribution: ~9 Strike / 6 Shield / 5 Move / 5 Mystic.
- **Trigger for expansion**: Stack F or dedicated session after Stack A/B confirm combat balance is stable. Do not expand mid-combat-testing.

### OQ-22: Defensive Skill Viability
**Improved in Playtest 2.** Armorsmith, Rust Shield, Field Medic all used meaningfully. Re-evaluate after future stacks if defense feels unviable again.

### OQ-23: Move Slot Count
**May be superseded by AP system.** If staying with current turn structure, test 3 Move Slots. If moving to AP, this question dissolves.

### OQ-24: Skill Card/Rule Clarity
**Review all skill descriptions for clarity before next playtest.** Connects to OQ-56 Problem C (skill cards on the table).

### OQ-39: Shared-Puzzle Design Direction — OPEN
**Should the game lean into "rewarding cleverness / mutual epistemic exploration"?**
- Research (`docs/research/cooperative-feel-competitive-games.md`): the shared-puzzle feel comes from perfect information + depth, not from removing competition.
- Design principles agreed (see ADR-003): expand viable strategies, reward cleverness visibly, let both players appreciate each other's plays.
- Not a mechanical commitment yet — framing/identity question.
- **Status**: Revisit after more stack data. See OQ-51 for mechanical levers.

### OQ-41: Game Length vs. Damage Nerf Tradeoff — PARTIALLY CONFIRMED (Playtest 3)
**1-DMG attacks did NOT noticeably extend the game** — Round 24 finish, comparable or shorter than P2.
- Caveat: draft-asymmetric game (Mario inexperienced).
- **Status**: Re-evaluate with two experienced players in Stack A Game 2.

---

## Deferred (concrete trigger required)

### OQ-27: Piece Count and Ratio — DEFERRED
**Current: 5 Champions + 6 Guards + 1 King.** Proposed (Stack D): 3 Champions + 3 Guards + 1 King.
- Bundled with board size. Both are "scale it down" changes. Must preserve per-turn decision depth.
- **Re-entry trigger**: After Stack A/B — if first Champion kill is still past R15, becomes Priority 1.

### OQ-1b: 8×8 Board Test — DEFERRED (residual of OQ-1)
**10×10 confirmed viable; 8×8 not yet tested. Would tighter geometry reduce empty-board feel in opening?**
- Bundled with OQ-27 (piece count) — both are "scale it down" levers.
- **Re-entry trigger**: Stack D scope. Same trigger as OQ-27.
- **History**: see OQ-1 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-35: Skill Pool Draft Variant — DEFERRED
**Each player drafts a pool of N skills first, then assigns skills to Champions.**
- Could reduce "accidental" synergies and increase strategic intent. Connects to OQ-56 Problem A.
- **Re-entry trigger**: Test after Stack A/B accepted. Independent of combat/economy changes.

### OQ-36: Flexible Piece Placement — DEFERRED (bundled with OQ-48)
**Players choose starting positions within constraints** rather than fixed formation.
- Risk: increases setup time and reactive counter-positioning. Explore reveal-style simultaneous pick (backpocket).
- **Re-entry trigger**: Test after Stack A/B accepted, bundled with OQ-48.

### OQ-42: Hex Grid Board — REOPENED
`[System: Board/Spatial]`
**Should we evaluate a hexagonal grid as an alternative to the square 10x10?**
- Original ADR-001 confirmed "grid over card-fighter," not "square over hex."
- **Research needed**: How do published tactical games handle hex vs. square? What does hex do to combo geometry?
- **Status**: Trigger `/research hex vs square grid in 2-player tactical board games` before any test stack is proposed.

### OQ-48: Piece Placement Order — DEFERRED (bundled with OQ-36)
**Place pieces after the skill draft, informed by loadout.**
- **Re-entry trigger**: After Stack A/B accepted, bundled with OQ-36.

### OQ-49: Skill Path Obstruction Model — DEFERRED
**Idea 1** (only opponent pieces block) **withdrawn Session 7** (creates turtle meta).
**Idea 2** (only opponent Guards block) — not yet evaluated. Risk: Guard wall dominant strategy.
- **Re-entry trigger**: After Stack A/B — if own-piece LoS blockage is consistently frustrating, test Idea 2. Otherwise park.

### OQ-50: Minor/Major Skill Slot Cost — DEFERRED
**Skills cost different numbers of Skill Slots (minor=1, major=2).**
- With 2 slots/turn, 2-cost skills are traps unless designed as "ultimate" skills worth the commitment.
- **Re-entry trigger**: Design 2-3 candidate "ultimate" skills first. Only evaluate as part of Stack 4+ (Skill Slots expand to 3/turn). Connected to OQ-19 (endgame acceleration).

### OQ-51: Mechanical Levers for Rewarding Clever Plays — RESEARCHED
**Three promising candidates staged in backpocket** (Cascade trigger, Pin/Threatened, Collision damage).
- **Cascade trigger**: anti-snowball one-turn tempo bonus on kill. Test in Stack F or earlier.
- **Pin/Threatened**: restriction-as-reward. Needs own stack.
- **Collision damage**: gated on standoff resolution.
- **Status**: Researched. Next step: test cascade trigger in Stack F.

---

## Open (no clear trigger yet)

### OQ-14: Draw Conditions
*(Unchanged.)*

### OQ-3b: AP System Replaces Movement-Action Link — OPEN (residual of OQ-3)
**Unlinked Movement/Action confirmed working. The remaining question: should the AP-system stack supersede it entirely, or do they coexist?**
- Tied to OQ-26 (AP system constraint models).
- **Status**: No action until AP-system stack is on the roadmap. Decision happens when that stack is designed.
- **History**: see OQ-3 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-13b: First-Player Advantage — DORMANT WATCH (residual of OQ-13)
**No advantage observed across 3 playtests. Watch only.**
- **Re-entry trigger**: A consistent first-player win rate across many games (5+) with similar player skill.
- **If triggered**: Apply Go-style komi (P1 starts with fewer Runes), per OQ-45 reasoning.
- **History**: see OQ-13 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-20b: Default Skill Range 2 — Practical Feel Watch (residual of OQ-20)
**Range 2 is the canonical default; question is whether it feels limiting in practice.**
- No "Range 2 too short" feedback in P2 or P3.
- **Re-entry trigger**: Player feedback in any future playtest that Range 2 skills feel too constrained.
- **History**: see OQ-20 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-16: Skill Drafting Fairness
- Fair but "decides a lot." Future question: is draft too deterministic? See also OQ-56.

### OQ-54: Lance Thrust — Should Its Text Say "Adjacent" Instead of "Range−1"?
**Effect text reads "Target within Range−1 takes 1 DMG" — under accepted Range ruling, effective Range 1 = adjacent.**
- For rewriting: clearer at the table, no mental arithmetic.
- For keeping "Range−1": preserves the design signal that reduced range is a deliberate tradeoff.
- **Status**: Watch whether the Range−1 language causes confusion in further playtests.

### OQ-55: Blade Call — Broader Skill Interaction?
**Could Blade Call interact with a wider range of skills (e.g. boost Armor gained by Shield skills)?**
- Broader Blade Call deepens Mystic identity as a "combo enabler."
- Risk: universal "make your best skill better" button — possibly too flexible.
- Alternative: introduce a second Mystic skill that amplifies Shield/Move (separate amplifier archetype).
- **Status**: No action until skill catalogue expansion is unblocked. Justification required: what specific problem does broader Blade Call fix?
