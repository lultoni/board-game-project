# Backpocket

*Three-part reference: (1) Design guardrails — invariants to check every change against. (2) New ideas & staged fixes — hypotheses ready to deploy when triggered. (3) Known potential issues — risks to monitor.*

*Last updated: 2026-04-29*

---

## Design Guardrails

*Every proposed change must pass these. If a change violates a guardrail, it needs explicit justification or redesign.*

### G1. Shortfall Never Closes
Players should *never* be able to fill all their skill slots with Rune-funded activations every turn. The economy is tuned so you always want to do more than you can afford. This scarcity IS the decision engine.

### G2. Encourage Spending via Attractiveness, Not Punishment
If players hoard Runes, the fix is making spending more attractive (better skills, more combo opportunities) — not forcing them to spend via caps or use-it-or-lose-it rules. "Better to invest badly than lose the money entirely" should never describe the player's situation.

### G3. Skill Cost is Feel, Not Math
Don't calibrate skill costs by spreadsheet. Calibrate by playtesting: players should never feel "I can't do anything" (too expensive) or "I have no reason to plan" (too cheap). The right cost creates agonising tradeoffs.

### G4. Cognitive Load is Real Cost
Any mechanic that requires tracking state (temp effects, turn timers, tile conditions) must have a physical tracking solution that doesn't break flow. If you can't explain how it's tracked on the table, the mechanic isn't ready.

### G5. Strategy Freedom > Simplification
Never restrict player choice just to reduce complexity. If a system is complex, find ways to make it *learnable* (better onboarding, clearer rules text, reference cards) rather than *smaller*. The space of possibilities is the game's value proposition.

### G6. No Single Strategy Dominance
Don't ban strategies — make more strategies viable. If one approach dominates, the fix is strengthening alternatives, not nerfing the dominant one into uselessness.

### G7. Core Fantasy First
"Does this make skill combos more interesting?" is the test for every system. If a mechanic doesn't serve the combo/cleverness fantasy, it needs to justify its existence on other grounds.

---

## New Ideas & Staged Fixes

*Hypotheses ready to test when the relevant problem surfaces. Each has a trigger condition.*

---

## Rune Theft — Cost Nerf

**Problem**: Rune Theft (3 Runes: 1 DMG + steal 1 Rune) may be too strong with Layer 1 economy. With +2/turn income, stealing 1 Rune represents ~50% of a turn's income AND deals damage. Creates aggressive "Rune race" dynamics and tempo swings that may dominate decision-making.

**Pre-thought fix**: Raise cost to **4 Runes**.

**Net cost analysis**:
- Current: Pay 3, deal 1 DMG, steal 1 Rune → net cost 2 Runes + 1 DMG
- Nerfed: Pay 4, deal 1 DMG, steal 1 Rune → net cost 3 Runes + 1 DMG
- Compared to Lance Thrust: Pay 2, deal 1 DMG → net cost 2 Runes, no steal
- At cost 4, Rune Theft is still economically neutral (steal 1 back), but costs 4 Runes up front — less spammable, requires more planning

**Alternative**: Remove the 1 DMG (just theft, no damage, cost 2). Makes it a pure utility/disruption skill — likely weaker and less interesting.

**Trigger**: Test in Layer 2 if Rune Theft is still dominant after 3 HP changes. If not dominant, defer further.

---

## Blade Tempest — Push Direction Ambiguity

**Problem**: "Adjacent pieces pushed 1 tile away from target" is ambiguous when a piece is diagonally adjacent to the target. "Away from target" could mean:
- **Option A**: Along the attacker's Skill Path (the line from caster through target). Pieces not on that axis are not pushed.
- **Option B**: Radially — each adjacent piece pushed directly away from the target tile (8 possible directions, one per adjacent tile).

**Current rule text**: Option B (radial, all adjacent pieces pushed).

**Pre-thought note on Option A**: Would make Blade Tempest a more linear, directional skill — easier to block by placing a piece behind the target on the skill path. Less chaotic but more readable.

**Trigger**: If Option B creates consistent confusion at the table, switch to Option A and test in the next available layer.

---

## Blade Tempest — Blocker Chain

**Observation**: Blade Tempest's radial push could theoretically create interesting chain interactions — a pushed piece could land on another piece's tile. Currently there is no ruling on what happens when a pushed piece lands on an occupied tile.

**Pre-thought fix**: Pushed pieces stop on the first unoccupied tile in the push direction (they don't displace other pieces). If there is no unoccupied tile in that direction (e.g., board edge or wall of pieces), the piece is not pushed.

**Trigger**: Add this ruling to Blade Tempest's text when the edge case is first reached at the table.

---

## Blade Call — Extension to Movement Skills

**Current**: Blade Call only buffs Strike skills (+1 DMG to a Strike skill this turn).

**Idea**: ~~Allow Blade Call to extend to movement skills as well — "one skill this turn gains +1 Range."~~ **Session 7 correction**: +1 Range duplicates Focus Strike's effect. This idea as stated is invalid. If Blade Call gets a secondary mode, it needs to be something other than Range — possibly +1 push/pull distance, or allowing a Strike to ignore 1 Armor. Needs rethinking.

**Risk**: At cost 3 Runes, any extension to movement skills could be very powerful. Might need cost adjustment.

**Trigger**: Revisit when the skill catalogue is more complete and a broader meta is visible. Candidate for Layer 5+ or a separate skill variant.

---

## Focus Strike — "Skill Slave" Problem

**Problem**: A Champion equipped with Focus Strike and a second skill becomes a "Skill Slave" — every turn it just fires Focus Strike then the second skill. No positional agency, no interesting choices. The skill effectively hard-locks its carrier's decision space.

**Pre-thought fix**: Skill Path Proximity rule — Focus Strike only activates if the caster is within Range 1 of the benefiting skill's caster (i.e., the two pieces must be adjacent or very close). This forces the Focus Strike carrier to position actively near the ally they're buffing, adding movement decisions instead of removing them.

**Alternative**: Focus Strike enhances the *next* skill used by the *same* piece only (not any of your pieces). This eliminates the cross-piece buff entirely. Simpler, but loses the cool ally-combo feel.

**Note**: The current "any of your pieces" ruling was decided as canon. The Skill Slave problem is real but acceptable for now — the proximity fix is a deferred solution if it becomes a dominant play pattern.

**Trigger**: If Focus Strike + one-skill-per-Champion becomes the default "correct" draft in Layer 2+, add the proximity constraint.

---

## New Skill Ideas — Ultimate Heal/Shield

**Observation gap**: The Shield category currently has only 3 skills (Rust Shield, Field Medic, Armorsmith), all at cost 2–3 Runes. There is no high-cost / high-impact defensive skill.

**Idea A — Ultimate Heal (cost 4)**: Heal *self* fully (restore to Normal regardless of damage taken). Comparable power to a 2-DMG standard attack reversed. High-cost self-sustain tool.

**Idea B — Ultimate Shield (cost 5)**: Grant +2 Armor to an adjacent ally (or self at cost 4). Would require an Armor Breaker response to neutralise.

**Note**: These are expensive by design — they should feel like a meaningful investment, not a free cycle. At cost 4–5 Runes, you're sacrificing multiple standard skills or a full turn's income.

**Trigger**: When the Shield skill category feels thin relative to Strike. Candidate for Layer 3+ or skill catalogue expansion.

---

## New Skill — Push Wave

**Idea**: A new Mystic or Move skill. *Push Wave (cost 3)*: Push all pieces in a 3×1 line (along the Skill Path, centered on target) 1 tile directly away from caster. Differs from Blade Tempest (which pushes radially around a single target) — Push Wave affects a corridor.

**Design note**: Creates area denial and formation disruption. Rewards positional thinking. Could be a natural counter to Guard screens (break through a line rather than going around).

**Trigger**: When the Move/Mystic category feels like it lacks area tools. Candidate for Layer 4+ or a dedicated skill expansion test.

---

## Skill Gaps — Shield and Mystic Categories

**Observation**: The current skill catalogue is Strike/Move heavy. Known gaps:

- **Shield**: No ranged defensive skill (current max range = adjacent). No "preemptive" or "conditional" defense (e.g., something like a reactive ward).
- **Mystic**: Only 2 skills (Focus Strike, Blade Call). No movement-buffing Mystic, no positional Mystic, no debuff/crowd control Mystic.

**Ideas worth exploring**:
- *Deflect (Mystic, cost 2)*: Negate the next skill used against one of your pieces this turn.
- *Warding Stone (Shield, cost 2)*: Place a 1-turn barrier on an adjacent empty tile (blocks Skill Path for one round).
- *Speed Surge (Mystic, cost 2)*: One of your pieces gains +1 Speed this turn.
- *Disrupt (Mystic, cost 3)*: Target opponent's piece loses 1 Skill Slot this turn.

**Trigger**: When the meta solidifies enough to see what archetype (control, aggro, combo) is missing a tool. Candidate for Layer 4+ or dedicated skill catalogue session.

---

## In-Game Skill Redraft

**Idea (early stage — not a staged fix, just captured for future consideration)**: Allow skills to be changed during the game rather than being locked at draft time. Possible formats:
- *Shop*: Spend Runes to swap one skill for another during your turn (from a shared pool).
- *Auction*: Both players bid Runes for skills from a shared pool at fixed milestone rounds.
- *Opponent swap*: Exchange one skill with opponent at a negotiated interval.
- *Fixed-interval redraft*: Partial or full redraft at milestone rounds (e.g. R10, R20).

**Why interesting**: Collaborative analysis in Playtest 2 — both players evaluating "what's the best move here" together — felt more engaging than pure competition. A designed redraft moment could create a shared strategic pause beat, leaning into the "puzzle-solving together" feel rather than against it. Connects to the idea of the game as a shared experience rather than a zero-sum contest.

**Trigger**: Deferred — do not discuss further or test until Layers 1–5 are complete and core systems are stable. Flag as Layer 6+ candidate or standalone design session.

---

## Standard Attack — Retaliation Variant

**Problem**: If the Layer 2 standard attack nerf (1 DMG) makes Guard clearing too slow and extends game length, an alternative approach is needed that keeps standard attacks risky without lowering their raw damage.

**Pre-thought fix**: Standard attacks deal 2 DMG as before, but the **attacker takes 1 DMG** (retaliation). Melee engagement becomes a mutual exchange. Skills (ranged, no retaliation) become the safe option. Cleverness is rewarded with safety; brute force is punished with self-damage. Common in tactical video games (Fire Emblem, Advance Wars).

**Trigger**: If Layer 2 testing (standard attack 1 DMG) shows Guard clearing drags and game length increases. This is an alternative to the nerf, not a complement.

---

## Jump Skill — Movement Through Own Pieces

**Idea**: An ultimate movement skill (Move or Mystic category) that allows a piece to move through allied pieces. Currently all movement is blocked by all pieces (ally and enemy). A "Jump" skill would let a Champion vault over allied Guards to reposition behind enemy lines.

**Why interesting**: Would add a new movement dimension without changing the base movement rules. Especially relevant if the board shrinks (8x8) and pieces are more tightly packed.

**Trigger**: When movement feels too constrained, especially on smaller boards. Candidate for skill catalogue expansion, not a system-level change.

---

## Rewarding Risky Positioning

**Design question (not a fix)**: How to systematically reward putting pieces in exposed/forward positions without rewarding reckless "yolo" play where you sacrifice pieces for no gain.

**Context**: The standoff / no-man's-land problem from Playtests 1-2 shows players avoid committing pieces to forward positions. The standard attack nerf (Layer 2) lowers the risk of commitment. The combo bonus rewards coordination. But there may be a deeper structural incentive needed — some kind of positional advantage for holding forward ground.

**Ideas to explore**: Board-zone bonuses (more Runes/Range in centre), forward-deployment bonus (Champion in opponent's half gets +1 Range), "territory" scoring as alternative win condition component. All speculative — needs research and discussion.

**Trigger**: After Layer 2 data, if the standoff problem persists despite the standard attack nerf and combo bonus.

---

## Action-Based Rune Economy

**Idea**: Tie Rune income to board engagement instead of (or in addition to) automatic time-based scaling. Examples: +1 Rune for dealing damage, +2 for capturing a piece, +1/turn for occupying centre tiles.

**Why interesting**: Would reward active play and punish turtling. Creates virtuous cycle: clever play → more resources → more clever play.

**Why dangerous**: Snowball effect (winner gets more Runes → wins harder). KPI problem — rewards the symptom (dealing damage) not the system (clever play). Standard attacks are free AND would be further rewarded, making them even more dominant. Saying "only skill damage counts" feels arbitrary.

**Designer's KPI analogy**: Like company KPIs that reward one metric and cause employees to optimise for it at the expense of the actual product. Must reward the entire cycle/system, not just one part.

**Trigger**: Only if standard attack nerf + combo bonus together don't fix the passive-play problem. Park until post-Layer-2 data. The existing automatic economy may self-correct when standard attacks are nerfed (skills become primary damage tool → Rune spending patterns change naturally).

---

## Checkmate-Style Win Condition

**Idea**: Game ends when a player can deal lethal damage to the opponent's King and the opponent has no legal response to prevent it. Both players jointly verify the position is inescapable. The King is not actually captured — the game ends on the *setup*, not the execution.

**Why interesting**: Cuts drawn-out endgame where outcome is decided but execution takes 5-10 more rounds. Rewards clever positioning and combo setup over grinding through remaining pieces. The moment both players jointly analyse "is this escapable?" IS the shared-puzzle peak — the game literally asks "can both players agree this position is decided?"

**Needs**: Anti-stalling/draw rules to prevent infinite loops (threefold repetition → draw, or N rounds without damage → draw). Don't ban stalling as a strategy — make it strategically suboptimal through escalation pressure.

**Risk**: "Only saves 1-2 turns" in mechanical terms. The value is experiential (game ends on the clever move, not the cleanup) rather than length-reducing. True game length fix likely requires fewer pieces or smaller board.

**Trigger**: Independent layer after Layer 2-3 results. Not coupled with the damage economy changes.

---

## Mid-Game Side Swap (Counter-Strike Halftime)

**Raw idea (Session 8 — unformed, not yet a mechanic)**: At a set point during the game, rotate the board 180° — players continue playing but now using the opponent's pieces/position. Like Counter-Strike's side-swap at halftime.

**Why interesting**:
- Eliminates first-player positional advantage (you play both sides)
- Invites the "playing together / shared puzzle" feeling — you literally inherit and must understand what the other player built
- Creates a natural halftime beat / strategic reset moment
- Tests whether your strategy works from both sides of the board

**Completely open questions** (not yet explored):
- When does the swap happen? (Fixed round? Triggered by event? Mutual agreement?)
- What carries over? (Rune pools? Skill loadouts? HP states? Everything?)
- Does this change the win condition? (King capture = instant loss regardless of swap, or do you need to "win both halves"?)
- Is this a fundamental game mode, or a variant/tournament format?

**Connects to**: OQ-39 (shared-puzzle direction), OQ-45 (first-player advantage), OQ-13 (first-player advantage data).

**Trigger**: Do not design further until the core loop (Layers 1–3) is stable. This is a game-mode-level idea, not a system tweak.

---

## New Skill Idea — Mini-Step

**Idea (Session 8)**: A cheap micro-repositioning skill. *Mini-Step (Move, cost 1–2 Runes)*: Move self 1 tile along Skill Path.

**Why interesting**: Fills the gap between free movement (Move Phase, free, up to Speed tiles) and expensive Move skills (Quick Dash = 3 Runes for 2 tiles). At 1–2 Runes, it's a low-commitment tactical adjustment — nudge a piece into LoS, escape a threat, or set up a combo next turn without burning a full Move Slot.

**Design consideration**: If cost is 1 Rune, it might be too spammable (essentially free repositioning via skills). Cost 2 Runes makes it comparable to Lance Thrust in economy but with no damage — the trade-off is "reposition vs. deal damage."

**Trigger**: When the skill catalogue is expanded. Candidate for inclusion alongside other gap-filling skills (Ultimate Heal, Push Wave, etc.).

---

## Reveal-Style Simultaneous Placement

**Idea (Session 8)**: Alternative to sequential piece placement that avoids the infinite counter-positioning problem identified in OQ-36/48.

**How it could work**: Both players secretly choose a starting formation (from a set of option cards, or freely within a starting zone), then reveal simultaneously. No reactive loop — both commit blind.

**Why interesting**: Eliminates the "I place, you react, I adjust, you adjust" problem. Adds a mind-reading/prediction layer (what formation will my opponent pick?). Could use pre-made formation cards for speed, or free placement within constraints for depth.

**Open questions**: What are the constraints? (Back 2 rows only? Any tile in your half?) How many formation options? (Pre-made deck of 5-6 options? Or free placement?) Does this interact with the skill draft (place after drafting, informed by loadout)?

**Connects to**: OQ-36 (flexible placement), OQ-48 (placement order).

**Trigger**: Test after Layer 3 accepted, bundled with OQ-36/48. Design the formation options first.

---

## Draw if Only Kings Remain

**Idea**: If every piece except the two Kings is removed → draw. Forces endgame resolution before losing all army.

**Why interesting**: A natural draw condition that prevents an endgame of two naked Kings chasing each other. Also gives losing players a comeback path — if you can trade down to Kings-only, you draw rather than lose.

**Trigger**: If only-Kings-left endgames become common and don't feel fun. Backpocketed until observed.

---

## Line Pull — Strömungsruf

**Idea**: Choose a line (LoS). Pull all enemies on that line 1 tile toward its centre. Unlike Maelstrom (pulls toward caster), this collapses enemies inward from both ends.

**Why interesting**: Compresses an opponent's formation, sets up AoE-like multi-target situations, blocks retreat routes. Genuinely novel geometry in the current skill set.

**Implementation constraint**: Must be formulatable as a single simple rule of thumb — "all enemies on the line move 1 tile toward the line's midpoint." No edge case exceptions.

**Trigger**: When skill catalogue expands (Stack F or later). Needs elegant rule formulation before testing.

---

---

## Known Potential Issues

*Risks to monitor. Not active problems — but if the trigger conditions are met, these become real.*

---

### King 3 Skill Slots → Ultimate Stay-Back Support

**Risk**: If the King ever gets 3 slots (post-v1 tuning), it could become the ultimate backline healer/buffer — never needing to advance, just stacking heal + buff + buff from safety. This makes the "capture the King" win condition harder to achieve because the King has no reason to be in danger.

**Mitigation ideas**: King-specific slot restriction (e.g., at least 1 slot must be Strike); or King gains 3rd slot only when on opponent's half of the board.

**Trigger**: If King 3 slots is ever tested.

---

### Armor Destruction Skills → Armor Becomes Dead Skill

**Risk**: If anti-Armor skills (Pocket Thief, Rüstungsbrecher) are too strong or too cheap, Armor skills (Armorsmith, Scrap Armor) become a waste of a slot — you spend 3 Runes to grant Armor, opponent spends 2 to strip it instantly.

**Mitigation ideas**: Anti-Armor skills must cost ≥ the Armor they destroy (economy-neutral at best); or Armor grants some residual benefit even when removed (e.g., the piece gets +1 Speed for 1 turn as the "Armor drops off" benefit).

**Trigger**: If anti-Armor skills are added to the catalogue.

---

### Temporary Effects Tracking Overhead

**Risk**: Any mechanic with a duration (Temp Armor, shields, speed boosts, debuffs) creates tracking overhead on a physical board. Without a solution, these become cognitive-load traps that slow the game.

**Known approaches to research**: Tokens placed on pieces, countdown dice, card sleeves with markers, turn-track markers. See `/research how board games track temporary effects on pieces`.

**Trigger**: Before any temp-duration mechanic is added to the catalogue.

---

### Move Slot Loss as Debuff → Feels OP

**Risk**: Restricting the opponent's Move Slots (they move one fewer piece this turn) is an extremely powerful tempo debuff. It directly removes agency and could feel unfair / unfun regardless of balance.

**Mitigation ideas**: Very high Rune cost (5+), or limited to "only when target is Injured" (conditional). Or: don't reduce Move Slots, instead reduce Speed by 1 for that piece (softer, still feels like a debuff, has existing mechanical precedent via Injured).

**Trigger**: If this debuff type is ever proposed for the skill catalogue.

---

### Rune Theft Dominance (existing — OQ-34)

See existing section above ("Rune Theft — Cost Nerf"). Monitoring in Layer 2.
