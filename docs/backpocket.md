# Backpocket

*Three-part reference: (1) Design guardrails — invariants to check every change against. (2) New ideas & staged fixes — hypotheses ready to deploy when triggered. (3) Known potential issues — risks to monitor.*

*Last updated: 2026-05-17*

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

### G8. Spending Tension
Players must always want to do more than they can execute. Early game: Runes limit action count. Mid/late game: Skill Slots limit action count while Rune costs force choosing WHICH skills to fire. If either resource becomes so abundant that spending requires no tradeoff, something is broken.

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

## Skill Catalogue Expansion — Staged Candidates (Session 11)

**Context**: Research (`docs/research/skill-catalogue-balance.md`) and Playtest 2 draft data show the catalogue's problem is not that defensive skills are underpicked — all 3 Shield skills are heavily used. The problem is **too few distinct strategic identities within Shield and Mystic categories**. All Shield skills are passive (add durability, no pressure). Mystic has a must-pick (Focus Strike) and a never-pick (Blade Call in P2 meta). Research recommends 25-35 skills minimum for meaningful draft variety; we're at 15.

**Design principle for new skills (from Session 11 research)**: Every new skill should pass the **sente test** — does it create a situation the opponent must respond to? Skills that are purely self-serving (passive buffs with no threat) don't dissolve standoffs or create interesting decisions. Dual-purpose skills (defend + create threat) are the ideal.

**Expansion target**: ~25 skills (10 new). Distribute across categories to reach roughly 9 Strike / 6 Shield / 5 Move / 5 Mystic.

---

### Shield — New Candidates

**Thorn Armor (Shield, cost 3-4 Runes)**
Grant +1 Armor to target ally/self. If that armor is destroyed by an attack, deal 1 DMG to attacker.

- **Sente**: YES — opponent must choose: attack the armored piece and take retaliatory damage, or avoid it entirely.
- **Balancing needed**: As stated, this is strictly better than Armorsmith (same armor + free damage). Needs constraints:
  - Higher cost (4 Runes?) so the economy trade is real
  - Thorn only triggers when armor is FULLY broken (not on each hit)
  - Armor Breaker could explicitly bypass the thorn effect (built-in counter)
  - OR: thorn replaces existing armor rather than stacking (can't have Armorsmith + Thorn on same piece)
- **Status**: Promising concept, needs balancing design before testing.

**Runic Ward (Shield, cost 3 Runes)**
Grant +1 Armor to target ally/self. If that armor absorbs damage this round, gain +2 Runes at start of next turn.

- **Sente**: YES — opponent attacks into it → fuels your economy. Opponent avoids it → your piece is safe. Either outcome benefits caster.
- **Standoff connection**: Directly incentivises forward positioning — you WANT to be attacked because it's profitable. Walk forward, dare the opponent to hit you.
- **Balancing**: Net cost = 1 Rune (pay 3, get 2 back IF hit). If never triggered, you paid 3 for +1 Armor (worse than Armorsmith at 3 for +1). Self-balancing: only good in aggressive/forward positions.
- **Status**: Ready to test. Clean design, clear sente, self-balancing.

**Bulwark (Shield, cost 2 Runes)**
Grant +2 Armor to self. This piece cannot use skills for the rest of this turn.

- **Sente**: No — this is a pure defensive commit. Trade: big armor but sacrifice your skill phase.
- **Use case**: King protection when under pressure. "Hunker down" option when you can't afford to both defend and attack.
- **Design note**: The self-restriction prevents it from being strictly better than Armorsmith. It's a strategic choice: maximum defense at the cost of offense.
- **Status**: Ready to test. Simple design, clear trade-off.

---

### Mystic — New Candidates

**Bind (Mystic, cost 3 Runes)**
Target enemy piece within range: that piece cannot be moved during the next Movement Phase. It CAN still use its own skills.

- **Sente**: YES — pinned piece must either accept reduced mobility (can't reposition) or burn a Move skill (Rune cost) to escape.
- **Counterplay**: Move skills (Quick Dash, Shadow Shift) are the escape. Gives Move category a new defensive role.
- **Connects to**: Pin/Threatened concept (Topic 1). Bind is the active/drafted version; Pin/Threatened (if ever implemented) would be the passive/positional version.
- **Status**: Ready to test. Clear sente, clear counterplay.

**Energize (Mystic, cost 2 Runes)**
Target ally within range: that piece's next skill activation (this turn OR next turn) costs −2 Runes (minimum 0).

- **Sente**: Partial — enables a cheaper follow-up but opponent can ignore it.
- **Use case**: Alternative to Focus Strike as an enabler. Focus gives +1 Range; Energize gives −2 Rune cost. Different build identity: "range extension" vs "economy enabler." Breaks Focus Strike's monopoly as the only buff Mystic.
- **Key design**: Must carry over to next turn, otherwise it's "pay 2 now, save 2 later this turn" = net zero within a turn (pointless). Carry-over makes it a setup/investment tool.
- **Tracking**: One token on the piece indicating "next skill discounted." Removed after use. Minimal overhead.
- **Status**: Ready to test. Clear alternative to Focus Strike.

**Skill Drain (Mystic, cost 3-4 Runes)**
Target enemy Champion within range: their next skill activation this turn costs +2 Runes.

- **Sente**: YES — directly taxes opponent's action economy. They must either pay more or change plans entirely.
- **Mirror of Energize**: Energize helps allies, Skill Drain hurts enemies. Together they create an "economy manipulation" sub-category in Mystic.
- **Risk**: Could feel oppressive / "unfun" (opponent's plans are disrupted without counterplay beyond "have more Runes"). Monitor carefully in testing.
- **Balancing**: High cost (4 Runes) makes it an investment — you spend 4 to make them spend +2, net cost to you is 2 Runes for a tempo disruption. Only worthwhile against expensive skills.
- **Status**: Promising but risky. Test with caution — monitor for "feels bad" feedback.

---

### Move — New Candidates

**Mini-Step (Move, cost 2 Runes)**
Move self 1 tile along Skill Path.

- **Sente**: No — pure self-repositioning. But enables sente plays (adjust LoS for follow-up Strike).
- **Priority**: LOW — luxury candidate. Only test if sente skills don't already solve game speed. Risk: if efficient, becomes auto-draft and crowds out interesting skills.
- **Use case**: Cheap LoS adjustment. Fills gap between free movement (Move Phase) and expensive Move skills (Quick Dash = 3 Runes). The "glue" skill that makes combos possible.
- **Cost decision**: 1 Rune might be too spammable. 2 Runes makes it economy-comparable to other options. Test at 2, reduce to 1 if underused.
- **Status**: Ready to test. Already in backpocket from Session 8.

**Swap Step (Move, cost 2 Runes)**
Swap positions of two of your adjacent allied pieces.

- **Sente**: Partial — surprise LoS changes can create unexpected threats. Opponent must re-evaluate which pieces threaten what.
- **Use case**: Formation rearrangement without burning Move Slots. Put your Strike Champion where your Guard was (and vice versa). Enables surprise combos.
- **Status**: Ready to test. Simple, clear, enables creativity.

**Ram (Move/Strike hybrid, cost 3 Runes)**
Move self 1 tile toward target along Skill Path. Push target 1 tile in same direction. If pushed piece hits another piece, that stationary piece takes 1 DMG.

- **Sente**: YES — displacement + potential collision damage. Opponent must consider clustering risk.
- **Connects to**: Collision damage concept (Topic 1). This is the skill-specific version (opt-in via draft, not universal physics).
- **Design note**: Dual-purpose — repositions you forward, pushes enemy back, AND punishes clusters. The ultimate "aggressive utility" skill.
- **Status**: Ready to test. Collision damage as skill property (not universal rule).

---

### Previously Listed Ideas (retained, lower priority)

The following ideas from earlier sessions remain in the pool but are lower priority than the above candidates:

- **Ultimate Heal (Shield, cost 4)**: Heal self fully. High-cost sustain. Not sente (opponent ignores it). Lower priority.
- **Ultimate Shield (Shield, cost 5)**: Grant +2 Armor. Not sente. Superseded by Runic Ward / Thorn Armor as more interesting designs.
- **Push Wave (Move, cost 3)**: Push all pieces in a 3×1 corridor. Area denial. Needs more design work on targeting rules.
- **Deflect (Shield/Mystic, cost 2)**: Negate next skill on target. Tracking problem (G4) — "which piece has deflect active?" Creates uncertainty that may not be fun.
- **Warding Stone (Shield, cost 2)**: Place 1-turn barrier on tile. Temp effect tracking (G4 blocker — needs research from backlog).
- **Speed Surge (Mystic, cost 2)**: +1 Speed this turn. Not sente. Functional but not exciting.
- **Disrupt (Mystic, cost 3)**: Target loses 1 Skill Slot. Very powerful / potentially unfun. Superseded by Skill Drain as a gentler economy-tax version.
- **Gravity Well (Move, cost 3)**: Pull all pieces within 2 tiles of target tile 1 tile toward it. Affects own pieces too. Sente (formation disruption). Needs more design work — edge cases around tie-breaking, targeting clarity. Move category, not Mystic.
- **Line Pull / Strömungsruf**: Pull all enemies on a line toward midpoint. Needs elegant rule formulation.

**Trigger for expansion**: Stack F (Cleverness II) or dedicated skill catalogue session after Stack A/B results confirm combat balance is stable. Do not expand mid-combat-testing — introduces confounding variable.

---

## ~~New Skill Ideas — Ultimate Heal/Shield~~

*(Superseded by expanded candidate list above — see "Previously Listed Ideas")*

---

## ~~New Skill — Push Wave~~

*(Superseded by expanded candidate list above — see "Previously Listed Ideas")*

---

## ~~Skill Gaps — Shield and Mystic Categories~~

*(Superseded by expanded candidate list above — Session 11 research provides full analysis)*

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

**G1/G8 compatibility (Session 11 — researched)**: Sente threats force *reactive* spending — the defender spends Runes/slots to neutralize, not to profit. Both players still feel the shortfall (G1). The attacker's advantage is tempo (they chose when/where), not resources. This is G8-compatible because the tradeoff persists: spending to respond means NOT spending on your own plan. Sente breaks G1 only if responding generates more resources than it costs — avoid that in skill design.

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

## [TO DISCUSS] Terrain Objects — Placeable Skill Stations (Session 12 idea)

**Concept**: Terrain "effects" are permanent objects placed on tiles with 1 HP (destructible). Pieces walk to them to use their effect. Unlike removed terrain (ADR-001), these are player-created via skills — not map features.

**Example — Placeable Armorsmith**: A skill places a "forge" token on a tile. Any friendly piece that moves onto or through that tile can spend Runes to gain Armor (e.g., pay 2 Runes → +1 Armor). Alternatively: upfront investment model (pay 5 Runes to place; any ally on the tile gets +1 Armor for free, unlimited uses until destroyed).

**Why interesting**:
- Creates forward-positioning incentive — you MUST push toward (or protect) your station to benefit
- Opponent must decide: ignore it (let them armor up) or destroy it (costs 1 attack action + positioning)
- Sente property: a well-placed station forces the opponent to either contest the zone or concede the value
- Solves standoff: places something worth fighting over in the middle of the board

**Ownership model (design decision)**:
- **Player-spawned**: A skill places the station (costs Runes + a Skill Slot action). Only your pieces benefit. Creates asymmetric board states — your station is your advantage to defend.
- **Neutral / pre-placed**: Stations exist on fixed tiles from game start (or appear at set rounds). Both players can use them. Creates contested zones — whoever controls the tile gets the benefit.
- Hybrid: neutral stations exist, but a skill lets you "claim" or "corrupt" one (flip it to your side / deny opponent access).

**Open questions**:
- Does this violate "no terrain" (ADR-001)? Or is it different because it's player-created/contestable, temporary, and destructible?
- 1 HP = one hit to destroy. Too fragile? Or correct because placement itself cost Runes?
- What other station types? (Healing font, Rune generator, speed boost tile, LoS blocker?)
- How does this interact with G4 (tracking)? Token on tile = fine. But if effects are conditional (e.g., "first piece each turn"), overhead increases.
- Does this change the game's identity too much toward territory control?

**Trigger**: Discuss before testing. Potentially connects to OQ-40 (standoff dissolution) and Stack D (board feel). Could be its own mini-stack if the concept survives discussion.

---

## [TO DISCUSS] Laser Beam — Line Damage Skill (Session 12 idea)

**Concept**: A high-cost Strike/Mystic skill that deals 1 DMG to ALL pieces (ally and enemy?) along the Skill Path line. Pierces through blockers — the line keeps going.

**Possible design**: *Laser Beam (Strike, cost 5-6 Runes)*: Choose a direction from caster. Deal 1 DMG to every piece on that line until board edge. Does NOT stop at the first piece (unlike normal Skill Path).

**Why interesting**:
- Ultimate/expensive skill — high cost makes it a committed investment, not spammable
- Anti-stalling tool: breaks through defensive walls and "hiding behind Guards" formations
- Forces opponent to spread out rather than cluster (anti-deathball)
- Punishes predictable linear formations — creates a new positional concern

**Open questions**:
- Hits own pieces too? (More interesting but harder to use — reward is positioning your pieces OFF the line)
- Only enemies? (Safer design but less interesting positioning puzzle)
- Does it ignore Skill Path blockage entirely? (If yes: unique mechanic. If no: it's just multi-target Lance Thrust.)
- At 5-6 Runes: is it ever worth it vs. multiple cheaper targeted skills? Need to ensure there's a board state where it's the correct play.
- Blade Call interaction: +1 DMG to ALL targets? (Probably no — Blade Call buffs exactly one Strike.)

**Trigger**: Discuss as part of skill catalogue expansion (Stack F). Clearly an "ultimate" tier skill — connects to OQ-50 (major skill slot cost) if that's ever implemented.

---

## [TO DISCUSS] Wave Push — Line Displacement Skill (Session 12 idea)

**Concept**: A Move skill that pushes ALL pieces on the Skill Path line 1 tile in the cast direction. Like a shockwave traveling down a corridor.

**Possible design**: *Wave Push (Move, cost 3-4 Runes)*: Choose a direction from caster along Skill Path. All pieces (ally and enemy) on that line are pushed 1 tile away from caster.

**Why interesting**:
- Mass displacement — disrupts entire formations in one action
- Creates chain reactions if pieces are pushed into each other (connects to collision damage concept / Ram skill)
- Strategic depth: you might push your OWN pieces forward as a feature, not a bug
- Anti-stalling: breaks apart defensive clusters, forces re-evaluation of positions

**Mirror skill — Wave Pull**: Same mechanic, opposite direction. All pieces on the line are pulled 1 tile toward caster.
- More dangerous to use (pulls enemies closer to you)
- Enables: pull enemy piece into your combo kill zone
- Creates interesting pair: draft Push for anti-stalling, draft Pull for aggressive combo setups

**Open questions**:
- Affects both ally and enemy? (More interesting, harder to use, higher skill ceiling)
- What happens when a piece is pushed into the board edge? (Stays in place? Takes 1 DMG from wall slam?)
- What happens when pushed into another piece? (Stop before it? Collision damage? See Ram / collision damage section.)
- Connects to existing "Push Wave" in Previously Listed Ideas (line 218) — this is a more fleshed-out version.
- Is Wave Pull too strong as a combo setup tool? (Pull 3 enemies into a cluster → Blade Tempest AoE?)

**Trigger**: Discuss as part of skill catalogue expansion (Stack F). Supersedes the earlier "Push Wave" concept in Previously Listed Ideas.

---

---

## [TO DISCUSS] Guard "Skills" — Passive Buff Draft (Session 13 idea)

**What it fixes / improves**: Guards are currently strategically flat — all 6 are mechanically identical, so they're treated as interchangeable bodies. This idea gives Guards their own identity layer (parallel to how skills give Champions identity), so positioning them is a real decision rather than just "where do bodies go." Also opens a new draft axis without adding active-skill cognitive load (buffs are passive — no in-game Rune decisions).

**Concept**: Just as Champions equip 2 active skills during draft, Guards could draft passive buffs from a separate pool. These aren't activated with Runes — they're permanent traits that change how the Guard plays.

**Example buffs**:
- *Stalwart*: This Guard has +1 Armor permanently (starts with 1 Armor).
- *Flanker*: This Guard has Speed 3 (instead of 2).
- *Sentinel*: This Guard can bodyguard from 2 tiles away (not just adjacent).
- *Anchor*: This Guard cannot be pushed or pulled by enemy skills.

**Why interesting**:
- Adds draft depth without adding active-skill cognitive load (Guards don't need Rune spending decisions)
- Differentiates Guards from each other — currently all 6 Guards are identical, which is strategically flat
- Creates formation-building decisions: which buffs go where in your lineup?
- Connects to OQ-51 (rewarding clever plays) — Guard buffs could reward specific positioning patterns

**Open questions**:
- How many buffs per Guard? (1 each seems right — 6 Guards × 1 buff = 6 draft picks from Guard pool)
- Separate draft phase or combined with Champion skill draft?
- Pool size: how many distinct Guard buffs exist? (8-12 seems right for meaningful choice with 6 picks)
- Does the opponent see your Guard buffs? (Perfect information says yes — but does that make Guards too readable?)
- Does this violate "Guards are simple" or enhance it? (Buff is passive = no in-game decisions, just draft decision)

**Risk**: Could make Guards too important relative to Champions. The core fantasy is Champion combos — Guard buffs must support that, not compete with it.

**Trigger**: Discuss after Stack A/B results confirm basic balance. Could be its own mini-stack or bundled with Stack E (Draft).

---

## [TO DISCUSS] Mid-Game Events / Inflection Points (Session 13 idea)

**What it fixes / improves**: The game currently feels linear — players "play out the strategy they decided at draft" with little reason to evolve mid-game. This breaks the engagement curve: once your plan is set, the rest is execution. Inflection points create natural pacing beats that force re-evaluation, give the late game a different feel from the early game, and prevent stalling by escalating pressure over time.

**Concept**: At set points during the game, something "shifts" — a rule changes, a resource appears, or a constraint activates. Creates natural game phases with distinct feels.

**Example events**:
- *Round 10 — "The Veil Lifts"*: All pieces gain +1 Skill Slot for the rest of the game. (Accelerates the endgame — more actions per turn, more Rune pressure.)
- *Round 15 — "Desperation"*: Standard attacks deal 2 DMG again (reverting the nerf). (Forces engagement if stalling.)
- *First Champion killed — "Blood Price"*: The killer's team loses 2 Runes immediately. (Anti-snowball.)
- *Midpoint — "Resupply"*: Both players gain a one-time Rune bonus (e.g., +5). (Enables a big skill turn.)

**Why interesting**:
- Creates natural pacing beats — the game feels different at Round 5 vs Round 15
- Can address the "game too long" problem by escalating pressure over time
- Players can plan AROUND events (setup before, capitalise after)
- Deterministic (no randomness) — both players know when events fire

**Open questions**:
- Fixed-round triggers vs. state-based triggers (first kill, first King damage)?
- How many events per game? (1-2 seems right — more creates tracking overhead / G4 violation)
- Are events symmetric (affect both players equally)?
- Does this conflict with "small number of interlocking systems" north star?
- Could events be DRAFTED? (Each player picks 1 event card that fires at a time they choose — adds a hidden-info element… but we're perfect information. So: open event picks during draft?)

**Risk**: Could add complexity without depth. Events that are just "numbers change" don't create interesting decisions. Events must create NEW decision points, not just shift existing ones.

**Connects to**: King Lifetime HP (irreversible game clock), Cascade Trigger (+1 slot on kill), Pacing (Stack C).

**Trigger**: Discuss after core stacks (A/B/C) tested. This is a game-mode-level concept — don't prototype until base systems are stable.

---

## [TO DISCUSS] Private Draft + Trade Phase (Session 13 idea)

**What it fixes / improves**: Open reactive drafting (current model) drifts toward a single meta over time — the game becomes "counter-pick the opponent's picks" rather than "express a creative strategy." Private drafts with simultaneous reveal break this collapse: you commit to a real plan instead of just countering. The trade phase adds a social/negotiation layer and lets players adjust at the seam without devolving into pure counter-picking. Goal: protect strategy diversity across many matches between the same players.

**Concept**: Modify the skill draft to include a trading/negotiation phase. Instead of purely sequential drafting from a shared pool, each player first receives a private allocation, then a simultaneous-reveal trade window opens.

**How it could work**:
1. **Split phase**: The 6 copies of each skill are randomly (or by rule) split 3-3 between players. Each player now has a private pool of skill copies.
2. **Trade phase**: Players simultaneously reveal trade offers ("I give you Skill X if you give me Skill Y"). Both must agree for a trade to happen. Limited rounds of offers (e.g., 3 rounds max).
3. **Equip phase**: After trades resolve, each player equips skills from their final pool onto Champions/King as normal.

**Why interesting**:
- Creates a pre-game social/negotiation layer — "the draft IS a mini-game"
- Both players have imperfect information about opponent's intentions during trade
- Trade refusal is information ("they really want to keep that skill — why?")
- Could create asymmetric loadouts that feel more personal/expressive

**Open questions**:
- Does this violate perfect information? (During the trade phase, yes — but by game start, all equipped skills are visible. Could argue trade phase is a "setup" separate from the game itself.)
- Random initial split vs. deterministic (alternating picks, then trade)? Random adds a luck element we've explicitly banned. Deterministic split (e.g., player A takes first copy of each odd-numbered skill, player B takes first of each even) is predictable but boring.
- Is trading actually fun with only 2 players? (Works better with 3+ — with 2, every trade is zero-sum. If I give you something good, I'm helping you directly. Might devolve into "no trades ever" equilibrium.)
- Time pressure: does negotiation slow the game down? (A 2-player game shouldn't spend 10 minutes haggling before play starts.)
- Simultaneous reveal: both players secretly write offers and reveal at once? Or alternating open offers?

**Risk**: The 2-player zero-sum problem is real. In a 2-player game, any trade that benefits your opponent directly hurts you. This might make the entire trade phase degenerate (no trades happen, or only trades where both players mis-evaluate). Works much better in 3+ player games where you can trade with a non-direct-rival.

**Alternative that preserves the feel**: Private draft with simultaneous reveal of equipped skills. Each player drafts skills in private (from a shared pool, but secretly), then all equipped skills are revealed before play starts. This gives the "surprise" element without the zero-sum trade problem. But it adds a hidden-information phase to a perfect-information game.

**Trigger**: Discuss alongside Stack E (Draft). This is a draft-variant concept — evaluate against current sequential open draft first.

---

---

---

## [TO DISCUSS] 8×10 Narrower Board Variant (Session 15 idea)

**What it fixes / improves**: shrinks the "spread to the flanks" runway. Pieces can't fan as far before hitting the edge → potentially less flank-drift at opening, more incentive to engage centrally. Addresses OQ-52 (centre attractor) directly via geometry rather than via added mechanics. Same height as 10×10 (preserves opening distance), narrower width.

**Trigger condition**: when OQ-52 reaches an active design-discussion phase, OR alongside a Stack D (Board) test.

**Risks**: increases piece density per column → standoff risk could re-emerge (the problem we just solved with the attack nerf — would erase Stack A gains). Rectangular not square — changes skill-range and LoS feel asymmetrically. Hard to isolate effect from other variables; must test as a single-variable change. Might require formation rework (current `--GGGGGG-- / --CCKCCC--` is centred for a 10-wide board).

**Status**: `[TO DISCUSS]` — staged option for OQ-52 / Stack D.

---

## [TO DISCUSS] 6×6 Board + 3C+4G+1K — Extreme Chassis Minimisation (Session 21 idea)

**What it fixes / improves**: Same hypothesis as Stack K (8×8 + piece reduction), pushed one step further. Does shrinking the board and army to their minimum practical size produce a more compact, combo-focused game where players spend less time navigating space and more time discovering and executing skill combinations? Specifically: reduces option-overwhelm (fewer pieces = fewer slots to evaluate), shortens game length, and tightens the decision density so both players are in "interesting choices" territory more of the time.

**Coupling note**: Board size and piece count are NOT independent at 6×6 — the full 12-piece army doesn't fit a 6×6 board without overcrowding the setup. This is why they must be bundled (unlike 8×8 where either variable can be tested alone). The coupling is deliberate and documented; this is not a methodology violation but an accepted constraint.

**Piece count rationale (4G+3C+1K)**: More Guards relative to Champions keeps the bodyguard/screen function meaningful while reducing the combo engine to 3 Champions — fewer skill slots in play, less catalogue knowledge required at once, same team-identity feel.

**Trigger / gating**: Strictly contingent on 8×8 (Stack K G1) AND 8×8 + 3C+4G+1K (Stack K G2) both showing positive returns (denser play, shorter game, better combo focus). Do not test if either prior step shows neutral or negative results — it would not be informative. Operationalised as Stack K G3.

**Risks**: At 6×6, even 8 pieces per side may feel overcrowded at opening. Formation design would need revisiting (current `--GGGG--/--CCKCC--` layout assumes a wider board). Game may feel more like a puzzle-box than a tactical game — evaluate after G2 data.

**Status**: `[TO DISCUSS]` — staged follow-up to Stack K. See OQ-1b (follow-up note) and OQ-27 for context.

---

## [TO DISCUSS] Starting-Formation Swap to Expose King (Session 15 idea)

**What it fixes / improves**: addresses OQ-53 (King isn't a real target) by changing the *starting* geometry so the King is more open from turn 1 — without changing what the King *is*. Specifically: swap the centre 2 Champions with the Guards in front of them, OR swap King + adjacent Champion with their fronting Guards, OR similar formation tweaks that reduce the King's screen. Lightweight to test (no rule changes — only initial setup).

**Trigger condition**: as part of an OQ-53 design discussion, or as a one-off setup variant during any non-formation-dependent stack.

**Risks**: swapped formations may unbalance opening (the player who's better at exploiting an exposed King wins reliably). Could push too far and make the King die too quickly, killing the "long game" feel. Test as one of several formation variants, not as a single fix. Bundles awkwardly with OQ-36 (flexible placement) — confirm which question is being answered.

**Status**: `[TO DISCUSS]` — needs brainstorming as part of OQ-53.

---

## [TO DISCUSS] "Spec the Game for a Programmer" Exercise (Session 15 idea)

**What it fixes / improves**: forces unambiguous rule definitions. Code has no tolerance for "we'll figure it out at the table" — writing an implementation spec exposes every ambiguous interaction (Lance Thrust + Injured, Focus Strike + adjacent self-target, push-into-LoS-blocker, Blade Tempest pushed-onto-occupied-tile, etc.) and forces decisions. Output: a cleaner ruleset with no hidden gaps. Doubles as a foundation for the digital prototype if that goes ahead. Concrete catalyst: Playtest 3's R22 was a wasted turn because Elias couldn't resolve an ambiguity at the table.

**Trigger condition**: anytime; scope-limited (write spec, do not build). Could be a single dedicated session, or a slow background pass while writing baseline updates.

**Note**: consider running `/research requirements engineering` first — there are established techniques (use cases, formal specs, behaviour-driven specs, decision tables, state machines) for exactly this kind of "translate a fuzzy domain into unambiguous rules" exercise. A short research pass before starting could surface the right format for our case (rule book → spec) and save us from inventing one from scratch.

**Risks**: low. Time investment, not design risk. Could surface contradictions in current rules that need resolving — that *is* the point. Risk of scope creep into "let's just build it" — keep this as a write-only exercise unless ADR-status decision is made first.

**Status**: `[TO DISCUSS]` — bookmarked exercise.

---

## [TO DISCUSS] Digital Playtest Prototype (web / iPad / Tabletop Simulator) (Session 15 idea)

**What it fixes / improves**: faster playtest iteration cycles, cleaner data capture (auto-logged rounds, attacks, armor, runes — fixes the form gaps surfaced in Playtest 3), can play during travel or short windows, and forces rule-disambiguation as a by-product (see "Spec the game for a programmer" entry above). Useful as a *complement* to physical playtests, not a replacement.

**Trigger condition**: travel window with a playtest partner (mentioned: Jonathan), OR after 2+ more physical playtests when iteration speed becomes the bottleneck.

**Scope discipline**: minimum viable = drag-and-drop simulator + long-press wheel for Injured/Armor/skill-equip + side-panel rune/round tracking. **No rules enforcement, no AI opponents, no polish.** Treat as a tool, not a product. Decision needs an ADR before any implementation work.

**Risks**: scope creep (polish is bottomless); risk of "the digital version becomes the game" — defeats the screen-free design intent; rule-state divergence between digital and `ruleset-baseline.typ` (digital must source from baseline, not the other way around).

**Status**: `[TO DISCUSS]` — sleep-on-it. ADR required before any building.

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
