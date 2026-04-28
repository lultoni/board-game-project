# ADR-003: Rewarding Cleverness Over Attrition

*Date: 2026-04-27*
*Status: PARTIALLY ACCEPTED — design principles agreed, test plan defined (Layer 2), multiple ideas deferred*
*Builds on: ADR-002 (incremental testing methodology), Playtest 2 data*

---

## The Problem

A system-by-system audit of the current design reveals that the game consistently rewards attrition and grinding over clever, creative play — despite the stated core fantasy being "discovering and executing clever spell/skill combos."

### Evidence

**Standard attack dominance (Critical)**:
- Standard attack: 2 DMG, 0 Runes, 1 Move Slot. Efficiency: infinite DMG/Rune.
- Lance Thrust: 1 DMG, 2 Runes. Efficiency: 0.5 DMG/Rune.
- Best 3-skill combo (Focus Strike + Blade Call + Lance Thrust): 2 DMG, 6 Runes, 3 Skill Slots. Efficiency: 0.33 DMG/Rune.
- The "best combo in the game" matches what a free standard attack does. Skills are structurally outperformed.

**Standoff / no-man's-land problem (Critical)**:
- Both playtests showed a zone between formations that neither player wants to enter first.
- Entering attack range = risking instant death (2 DMG = kill). No incentive to commit first.
- First player to commit is at disadvantage (opponent reacts to overextension).
- Runes accumulate regardless of action — no economic incentive to break the standoff.
- Playtest 2: "two guards like pawns" blocking lanes at R14; standoff felt like a wall.

**Flat economy (High)**:
- Rune income is automatic, time-based, identical for both players.
- A player who executes a brilliant combo and a player who does nothing receive the same Runes next turn.
- No economic reward for clever play. Optimal strategy: hoard, wait, spend efficiently.

**Low combo ceiling (High)**:
- 2 Skill Slots early game → max combo length of 2.
- Only two combo enablers exist: Focus Strike (+1 Range) and Blade Call (+1 DMG). Every combo follows the same "buff + hit" structure.
- No conditional triggers, chain reactions, cross-turn setups, or positional synergies in the skill catalogue.
- Depth is evaluative (comparing 3-5 known options), not generative (discovering unexpected interactions).
- Multi-Champion coordination ("gang-ups") is theoretically possible but not incentivised — players use 1 Champion at a time because it's easier to set up.

**Win condition reinforces grinding (High)**:
- Capture the King = grind through Guards → Champions → King. Linear attrition path.
- No alternative scoring path, no way cleverness bypasses the sequence.
- Playtest 2: 1 Champion kill in 26 rounds. Guards were dying but the defensive screen held.

### System Audit Summary

| System | Rewards Cleverness? | Rewards Attrition? | Severity |
|--------|--------------------|--------------------|----------|
| Combat/Attack | No | **Yes (dominant)** | Critical |
| Standoff dynamics | No | Yes (first-mover disadvantage) | Critical |
| Resource Economy | No | Yes (passive income) | High |
| Skill System | Partially (positional) | Partially (grinding LoS) | High |
| Combo Ceiling | Low | N/A | High |
| Health/Armor | Neutral (cost-balanced) | Slightly (timing advantage for defense) | Medium |
| Turn Structure | Neutral | Slightly | Low |
| Progression | No (time-based) | Yes (survival) | Medium |
| Bodyguard | Too unreliable to matter | N/A | Medium |
| Skill Drafting | Pre-game knowledge | N/A | Low |

---

## The Shared-Puzzle Discovery

Playtest 2 produced an emergent behaviour: both players spontaneously began collaborating — analysing board states together, discussing "what's the best move here?" This felt more engaging than pure competitive play.

Research (`docs/research/cooperative-feel-competitive-games.md`) found this is a known phenomenon in perfect-information games, called **mutual epistemic exploration**. It occurs when both players share identical access to board state and the puzzle is deep enough that the opponent transforms from adversary into co-interpreter.

Games that deliberately engineer this: Onitama (shared move pool), Twilight Struggle (DEFCON as shared threat), Go (kifu review culture), Tak (aesthetic framing), Tigris & Euphrates (symmetric scarcity).

**Key insight**: The shared-puzzle feel comes from perfect information + depth, not from removing competition. The game doesn't need to become cooperative — it needs to reward cleverness visibly enough that both players can appreciate each other's plays.

**Design framing established**: "Winning means 'I found the better solution to the puzzle,' not 'I crushed you.'" Competition stays. Stakes stay. But individual moves should feel satisfying to execute and appreciate — for both players.

---

## Design Principles Established

These emerged from multi-round discussion between designer and architect. They are load-bearing — future design decisions should be tested against them.

1. **Every strategy archetype should have a moment where it's the best option on the board.** Multi-Champion combos should be devastating when set up; single-Champion precision should be bread-and-butter; defensive formation play should be viable. No single strategy should dominate.

2. **Don't reward symptoms, reward the system.** (The KPI principle.) Rewarding "dealing damage" with bonus Runes incentivises the symptom (hitting things) rather than the system (clever play). Like company KPIs: setting wrong targets leads to good-on-paper results with a worse product. Reward the entire cycle, not one metric.

3. **Players should be allowed to play how they want.** Expand viable strategies, don't shrink them. Turtling should be a viable option — just not the only one. Don't ban strategies; make alternatives competitive.

4. **Cleverness = multi-turn positional setup rewarded with a payoff that exceeds what grinding achieves.** If a 3-turn setup and a stumble-into-position produce the same result, the game doesn't reward cleverness. The combo bonus is designed to make coordinated setups definitively better than brute force.

5. **The shared-puzzle feel is a byproduct of good design, not a mechanic.** Don't engineer it directly. Make cleverness visible, make both players' puzzles legible, and the shared exploration emerges naturally from perfect information + depth.

---

## Accepted for Testing: Layer 2

### Test Format: Two-Game Session

Time constraint: designer rarely has time for playtests. Solution: two games in one session (forenoon + afternoon, or back-to-back). Game 1 tests the more disruptive change alone; Game 2 adds the second change on top.

### Game 1: Standard Attack Nerf

**Change**: Standard attack deals **1 DMG** (not 2). Everything else = baseline + Layer 1 economy (6 start Runes, +2/turn, scaling every 5 rounds).

**Rationale**: At 1 DMG, standard attacks Injure but don't kill. Skills and attacks are on equal footing — both deal 1 DMG, but skills cost Runes and offer range/utility. Skills become worth their cost because there's no free alternative that outperforms them. Entering the no-man's-land is less fatal (Injured, not dead), so players should commit more readily.

**Rejected alternative**: Standard attacks cost 1 Rune. Designer feedback: players will perceive this as "another skill" rather than a fundamentally different dimension of play. Muddies the attack/skill distinction.

**Risk**: Guards take twice as many hits to remove (1 DMG instead of 2 = two attacks to kill). May extend game length. Guard removal was already happening in Playtest 2 — this slows it down. Designer explicitly wants Guards to have late-game presence, so Guard HP stays at 2 — do NOT differentiate Guard/Champion HP.

**Track explicitly**: Rounds until first Guard kill. Rounds until first Champion kill. Compare to Playtest 2 baseline (Guards dying throughout; first Champion kill at R26).

### Game 2: Standard Attack Nerf + Multi-Champion Combo Bonus

**Additional change on top of Game 1**: When two or more different Champions use skills that target the same enemy piece in the same turn, each skill after the first deals **+1 DMG**.

**Rationale**: Incentivises multi-Champion coordination ("gang-ups"). Creates a spatial puzzle: getting 2+ Champions with LoS to the same target on a crowded board is hard, so the bonus is a reward for overcoming that difficulty. Makes coordinated skill play definitively better than grinding with standard attacks.

**Clarifications**:
- "Target the same enemy piece" = the skills must hit the same enemy. Buff skills targeting your own pieces (Focus Strike, Blade Call, Rust Shield, etc.) do not count toward the combo.
- The bonus is per-target, per-turn. Two Champions hitting two different enemies get no bonus.
- Blade Call stacks with the combo bonus: if Blade Call is active, the second Champion's Strike deals +1 DMG (combo) +1 DMG (Blade Call) = +2 total.

**Why bundled with Game 1 (not tested independently)**: The combo bonus only matters if standard attacks are nerfed. At 2-DMG standard attacks, even a combo-boosted skill chain barely competes with free standard attacks. The two changes are coupled — one without the other doesn't shift the meta. Testing in sequence (same day, game 2 builds on game 1) lets us attribute the delta between games specifically to the combo bonus.

---

## Deferred Ideas (with reasoning)

### Action-Based Economy
Tying Rune income to engagement (damage dealt, combos executed, pieces captured). **Deferred because**: snowball risk (winner gets more Runes → wins more). KPI problem (rewards the symptom, not the system). Also: if standard attacks are nerfed and skills become the primary damage tool, the existing automatic economy may self-correct. **Trigger**: revisit only if standard attack nerf + combo bonus don't fix the passive-play problem.

### Checkmate-Style Win Condition
Game ends when lethal damage to King is inescapable — both players jointly verify. Cuts drawn-out endgame. The moment both players are jointly analysing "is this escapable?" IS the shared-puzzle peak. **Deferred because**: independent from the damage economy changes. Should be its own layer. Needs anti-stalling/draw rules. **Trigger**: after Layer 2-3 data.

### Own Pieces Don't Block Skill Paths
**Withdrawn.** Would let Champions hide behind Guard walls and snipe with zero positional risk. Creates a turtle meta that's worse than the current standoff. The current blocking rule forces Champions to break formation and expose themselves to fire skills — that exposure moment IS the tension. Playtest 2 already showed "standoff with 2-3 spaces in between" — shooting through your own Guards would make this worse, not better.

### 3rd Skill Slot Per Champion
**Withdrawn.** 2 skill slots force meaningful draft choices ("offensive or defensive?"). A 3rd slot dissolves that tension — every Champion becomes a "walking army" that can do a bit of everything. Designer explicitly rejected this.

### Guard HP Differentiation
Reducing Guards to 1 HP to compensate for the standard attack nerf. **Withdrawn.** Designer wants Guards to have late-game presence and not be trivially cleared. Guards stay at 2 HP.

### In-Game Skill Redraft
Changing skills mid-game (shop, auction, interval draft, or starting half-equipped). **Deferred because**: core combat systems need to be stable first. Layer 6+ candidate. See `docs/backpocket.md`.

### Standard Attack Retaliation Variant
Attacker takes 1 DMG when standard attacking (melee risk). Alternative to the 1-DMG nerf if that proves too slow. **Staged in backpocket** — trigger if Layer 2 shows 1-DMG attacks make Guard clearing drag.

### Rewarding Risky Positioning
How to systematically reward putting pieces in exposed positions without rewarding reckless play. **Deferred** — design question, not ready for mechanical testing. Trigger: after Layer 2 data if standoff persists.

### Emergent Combo Systems Research
Research thread: "In deterministic strategy games, what mechanical patterns create emergent combos from rule interactions?" Separate from the cooperative-feel research. **Parked** — trigger when combo ceiling becomes the active design focus.

---

## Key Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| 1-DMG attacks extend game length | High | Track Guard/Champion kill timing explicitly. Compensating mechanisms ready: fewer pieces (Layer 5), smaller board, checkmate. |
| Combo bonus makes Blade Call obsolete | Medium | Blade Call stacks with combo bonus (+2 total on buffed second hit). Monitor in test. |
| Combo bonus makes multi-Champion the only viable strategy | Medium | Setup cost is naturally high (LoS constraints on crowded board). Single-Champion plays remain bread-and-butter. Monitor. |
| Two changes in one game (Game 2) violates incremental testing | Accepted | Changes are coupled. Two-game format in one session isolates the delta. Documented as an exception with rationale. |
| Game 2 contaminated by learning from Game 1 | Low-Medium | Play the more disruptive change (nerf) first. Game 2 is strictly additive. Document in feedback form. |

---

## References

- Playtest 1 analysis: `docs/research/playtest-1-analysis.md`
- Playtest 2 analysis: `docs/research/playtest-2-analysis.md`
- Cooperative feel research: `docs/research/cooperative-feel-competitive-games.md`
- ADR-001: `docs/decisions/ADR-001-game-architecture-direction.md`
- ADR-002: `docs/decisions/ADR-002-direction-a-plus.md`
- Backpocket staged fixes: `docs/backpocket.md`
