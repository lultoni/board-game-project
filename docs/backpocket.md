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

**Problem**: Both playtests showed a "standoff zone" — 2-3 tile gap between formations that neither player wants to cross first because entering attack range risks heavy damage. First player to commit is at a disadvantage because the opponent can react optimally (perfect information makes this worse than in games with randomness).

**Research findings (Session 11 — `docs/research/forward-positioning-incentives.md`)**:
Five mechanical patterns identified: contested Rune generators (Advance Wars), one-time threshold bonuses, objective/VP scoring (Aristeia!/Kemet), sente skills (Go theory), and underdog bonuses. Key insight: the standoff is an incentive-intent gap — our game makes waiting strictly dominant. The attack nerf addresses entry risk but not the cost of passivity.

**Primary solution: Sente Skill Design (design principle, not a mechanic)**:
Rather than adding territory-control mechanics that shift the game's identity, design skills that naturally create threats requiring immediate response from forward positions. The game dissolves standoffs through its OWN systems (skills/combos) rather than bolted-on spatial incentives.

**What makes a skill "sente" (threat-forcing)**:
- Creates a state the opponent MUST respond to or suffer consequences
- Is more effective from forward/contested positions (rewards advancement implicitly)
- Doesn't require a separate tracking system — the threat IS the skill effect

**Current skills with sente properties**:
- Rune Theft: forces economy response
- Blade Tempest: pushes pieces out of formation, opponent must reposition
- Combo bonus (Stack A): if 2 Champions are in range of a target, opponent MUST deal with one

**Current skills WITHOUT sente properties** (opponent can ignore):
- Armorsmith, Field Medic: self/ally buffs — no pressure on opponent
- Focus Strike: setup for your own future action — opponent can wait

**Implication for skill catalogue expansion**: Prioritize skills that create "must-respond" threats from mid-range positions. Skills that are purely self-buffing don't dissolve standoffs. See Topic 4 (skill gaps) for specific candidates.

**Fallback hierarchy (if sente skills + attack nerf don't dissolve standoff)**:
1. One-time midline crossing bonus (+1 Rune per Champion crossing rank 5 for first time) — small, non-compounding
2. Contested Rune generators (2-3 midfield tiles producing Runes for controller) — shifts identity toward territory control, use only if desperate
3. VP scoring track (parallel win condition for forward presence) — absolute last resort, conflicts with core fantasy

**Anti-snowball safeguards (if generators ever deployed)**: Fixed-rate nodes, presence-required (piece must stay), recapturable (never locked in), "make winning cost resources."

**Trigger**: Monitor in Stack A playtest. If standoff persists despite 1 DMG attack nerf, escalate. If standoff dissolves, deprioritize entire section.

---

## Action-Based Rune Economy

**Idea**: Tie Rune income to board engagement instead of (or in addition to) automatic time-based scaling. Examples: +1 Rune for dealing damage, +2 for capturing a piece, +1/turn for occupying centre tiles.

**Why interesting**: Would reward active play and punish turtling. Creates virtuous cycle: clever play → more resources → more clever play.

**Why dangerous**: Snowball effect (winner gets more Runes → wins harder). KPI problem — rewards the symptom (dealing damage) not the system (clever play). Standard attacks are free AND would be further rewarded, making them even more dominant. Saying "only skill damage counts" feels arbitrary.

**Designer's KPI analogy**: Like company KPIs that reward one metric and cause employees to optimise for it at the expense of the actual product. Must reward the entire cycle/system, not just one part.

**Trigger**: Only if standard attack nerf + combo bonus together don't fix the passive-play problem. Park until post-Layer-2 data. The existing automatic economy may self-correct when standard attacks are nerfed (skills become primary damage tool → Rune spending patterns change naturally).

---

## ~~Checkmate-Style Win Condition~~ — KILLED (Session 11)

**Original idea**: Game ends when a player creates an inescapable lethal position against the King.

**Why killed**: Our game has too many defensive options (heal, armor, push, LoS blocking, 6+ Champions with 2 skills each) to ever formally prove "this position is 100% lost" at the table. Chess checkmate works because the defender's options are extremely limited (move/block/capture). In our game, verification burden is closer to Shogi's brinkmate — impractical without a computer. Research confirmed this (`docs/research/checkmate-win-conditions.md`).

**What remains**: King capture is the only formal win condition. Either player may resign at any time (informal convention — no rule needed).

**Replaced by**: King Lifetime HP (see below) as the mechanical endgame accelerator, IF the problem manifests in playtests.

---

## King Lifetime HP (Endgame Accelerator)

**Idea**: The King has a separate **Lifetime HP** track (number TBD — likely 4–8). Every point of damage the King takes from any source is permanently marked on this track, regardless of healing or armor. When Lifetime HP reaches 0, the King is removed and the game ends. Normal HP (2: Normal → Injured → Removed) still exists alongside — the King can still die through the normal route.

**Why interesting**:
- Creates an irreversible game clock — the game MUST end eventually because King damage accumulates permanently
- Healing becomes "delay" not "undo" — strategically richer (aligns with G1: shortfall never closes)
- Zero verification burden (one counter per player, tracked on game-tracking sheet)
- No arguments about "is this decidable?" — the King simply dies when the counter runs out
- Incentivises dealing ANY damage to the King (even 1 DMG "snipes" matter over time)

**Open design questions**:
- **Armor interaction — Model A**: Armor damage does NOT count toward Lifetime HP. Only real HP damage ticks the counter. This means armor is a true shield — extends lifetime. Risk: infinite armor cycling loops remain possible.
- **Armor interaction — Model B**: ALL damage counts (including armor). "Snipe hits" over many rounds eventually kill the King even through armor. Risk: needs a higher Lifetime HP number to feel fair. Upside: no infinite loop possible.
- **The number**: Must be high enough that "accidental" early King damage doesn't create a snowball, but low enough that games can't stall past ~25 rounds. Needs playtest data on average King damage per game to calibrate.
- **Tracking**: Single counter per player (e.g., a token track on the game-tracking sheet, or a small dial). Minimal overhead.

**Risk**: If playtests show Kings rarely take damage anyway (Playtest 2: ~0-2 King damage in 26 rounds), this mechanic doesn't fire and doesn't solve the length problem. The real fix may need to come from elsewhere (fewer pieces, smaller board, pacing stack).

**Trigger**: Only deploy if playtests show the King is specifically unkillable (armor/heal loops prevent capture) despite the game being strategically decided. NOT an active proposal — a backpocketed response.

---

## Armor Decay (Lifetime Armor Cap)

**Idea (speculative)**: Each piece has a maximum lifetime armor absorption (e.g., 6-8 total armor points across the whole game). Once a piece has absorbed that much armor damage cumulatively, no further armor can be applied to it. Piece becomes permanently "exposed."

**Why interesting**: Prevents infinite armor cycling in late game. Creates natural "wear and tear" — pieces degrade over time. Adds strategic depth to armor timing (use it early vs. save for when you really need it).

**Tracking problem**: Requires a per-piece counter (up to 12 per player). Same overhead issue as all-piece Lifetime HP. Likely only viable for Champions + King (6 per player) if at all.

**Connects to**: King Lifetime HP (same philosophy — irreversible accumulation), OQ-11 (armor cap), G4 (cognitive load).

**Trigger**: Only if armor cycling becomes a degenerate stalling strategy in playtests. Very speculative — park until observed.

---

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

## Cascade Trigger — +1 Skill Slot on Kill

**Idea**: When one of your pieces kills an enemy piece (by any method — standard attack or skill), you gain +1 Skill Slot for the remainder of that turn.

**Why interesting**: Rewards finishing a setup. The bonus is tempo (one more action THIS turn) not resources (no extra Runes). Creates exciting follow-up moments: kill → reposition to safety, kill → chain into a second exposed target. Incentivises committing to an attack rather than poking safely.

**Anti-snowball properties**:
- One-turn-only (doesn't compound across rounds)
- Still costs Runes to use the extra slot (early-game kills barely benefit because Runes are scarce)
- Opponent lost a piece = fewer future threats anyway; the slot just lets the attacker capitalise immediately rather than waiting a turn

**Backpocketed restriction (if too easy to exploit via standard attacks)**: Limit trigger to skill-kills only. Test this restriction if playtests show free standard-attack kills generating too much tempo.

**Removal condition**: If playtests show the extra slot is never used (players don't have Runes to spend), remove entirely. No dead rules.

**Trigger**: Stack F (Cleverness II) or earlier if a natural test opportunity arises.

---

## Pin / Threatened Status

**Idea**: A piece that is in the Skill Path (line of sight) of 2+ enemy Champions is "Threatened" — it cannot be moved during the Movement Phase (but CAN still use its own skills, and CAN be moved by Move skills).

**Why interesting**: Rewards surrounding and multi-piece coordination without dealing damage. Creates positional "captures" — you restrict the opponent's options by clever placement. The opponent must use a Move skill (Rune cost) to escape, or reposition the threatening Champions away. Connects to the "restriction as reward" pattern from Hive/Go.

**Risk**: Could feel oppressive / "control-losing" for the defending player. May make Movement skills a must-pick (interesting but constrains draft freedom).

**Counterplay**: Move skills become the escape tool (gives Move category a defensive role). Opponent can break the pin by moving one of the threatening Champions. Guard screens can block LoS to prevent pins.

**Open design questions**:
- Does the King count as a "Champion" for pin purposes? (Probably yes — it has skills.)
- Can Guards be pinned? (Probably yes — Guards in LoS of 2 Champions can't move. But Guards don't have skills, so pinning a Guard removes ALL its options except being rescued by a Move skill on an ally.)
- Does "Skill Path" mean direct LoS or does the path need to be unblocked? (Probably unblocked — you must have a clear shot to "threaten.")

**Trigger**: As its own test layer (Stack F or later). Independent of combat/economy changes.

---

## Collision Damage — Universal Rule (speculative)

**Idea**: When a piece is pushed/pulled into a tile occupied by another piece, the stationary piece takes 1 DMG. The pushed piece stops on the tile before (does not displace).

**Why interesting**: Makes ALL push/pull skills into positional combo tools. Rewards reading the board and creating "lined up" formations to exploit. Adds depth to Blade Tempest, Air Blast, Precision Thrust, Maelstrom, and any future push/pull skills.

**Risk (identified in Session 11 discussion)**:
- If BOTH pieces take damage: too punishing — creates keep-away zones where nobody advances into push range.
- If only the stationary piece takes damage: could amplify standoff problem (fear of being pushed into allies for splash damage). Opponent clusters less OR stays far away.
- Makes push skills potentially very strong relative to their cost (2-3 Runes for damage + displacement + collision damage).

**Why deferred**: The standoff problem must be confirmed dissolved FIRST (via Stack A results). If players ARE engaging closely after the standard attack nerf, collision damage adds exciting interactions. If standoff persists, collision damage makes it worse.

**Trigger**: Test ONLY after standoff is confirmed dissolved (post-Stack A, possibly post-Stack C). Do not test alongside standoff-fixing mechanics — evaluate independently.

---

## Collision Damage — Skill-Specific ("Ram" / "Shove")

**Idea**: A new Strike or Move skill where collision damage is the SKILL'S special property, not a universal physics rule. Example: *Ram (Strike, cost 3 Runes)*: Move self 1 tile toward target along Skill Path, push target 1 tile. If target hits another piece, that piece takes 1 DMG.

**Why interesting**: Opt-in during drafting (not a universal rule everyone must account for). Counterable (don't cluster pieces). Creates board-reading moments without taxing ALL positioning decisions. Gives the skill a unique identity — "the one that punishes clusters."

**Design space**: Could be a Strike (deals damage + push + collision) or a Move (no base damage, but repositions self AND punishes target's neighbours). The Move version is more novel.

**Trigger**: When skill catalogue expands (Stack F or later). Design the full skill text before testing.

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
