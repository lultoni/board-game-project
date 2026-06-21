# Backpocket

*Three-part reference: (1) Design guardrails — invariants to check every change against. (2) New ideas & staged fixes — hypotheses ready to deploy when triggered. (3) Known potential issues — risks to monitor.*

*Last updated: 2026-06-21*

---

## Design Guardrails

*Every proposed change must pass these. If a change violates a guardrail, it needs explicit justification or redesign.*

### G1. Shortfall Never Closes
Players should *never* be able to spend all their actions on Money-funded activations every turn. The economy is tuned so you always want to do more than you can afford. This scarcity IS the decision engine.

### G2. Encourage Spending via Attractiveness, Not Punishment
If players hoard Money, the fix is making spending more attractive (better skills, more combo opportunities) — not forcing them to spend via caps or use-it-or-lose-it rules. "Better to invest badly than lose the money entirely" should never describe the player's situation.

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
Players must always want to do more than they can execute. Early game: Money limits action count. Mid/late game: actions limit action count while Money costs force choosing WHICH skills to fire. If either resource becomes so abundant that spending requires no tradeoff, something is broken.

---

## New Ideas & Staged Fixes

*Hypotheses ready to test when the relevant problem surfaces. Each has a trigger condition.*

---

## Session 25 Brainstorm — Post-P5 Direction-Setting (the path to the Game Length Cut)

*This entry captures the brainstorm chain that led the designer to the Game Length Cut (GLC) decision. The conclusions (GLC, Principle 8, etc.) are documented elsewhere; this entry preserves the **reasoning chain** so future sessions can see WHY decisions look the way they do, and so adjacent open questions don't get lost when the GLC work absorbs the foreground. Each item below is parked, not yet active — most are explicitly "asan: revisit after GLC."*

Items in the chain (designer's "asan" = "after sleeping a night" reflections in italics):

**1. Does this game even need to be Perfect Information?**
- *Brainstorm:* questioning whether PI is load-bearing or a chess-clone holdover.
- *asan:* PI lets players never blame the game; that's worth something but doesn't automatically make a game fun. Park: revisit after the bigger blockers (GLC) are addressed.
- **Status:** parked, not promoted to OQ. PI remains a Hard Constraint in `design-principles.md` for now. Connects to OQ-64 (felt-PI vs formal-PI).

**2. Would smart, fair luck make the game more fun?**
- *Brainstorm:* counterpart to #1. Luck only if "neither winning nor losing player feels punished by it."
- *asan:* the better/cleverer player should still win when both play their best. Park alongside #1.
- **Status:** parked. Not justifiable under current Justification Rule (no concrete problem it fixes).

**3. Win condition: do we need a King? Could it be piece-count or other?**
- *Brainstorm:* King-capture may be a chess-clone remnant. Alternative end conditions (capture-N-pieces, threshold-based, territory) would change game shape.
- *asan:* this is part of the GLC brainstorm itself — "what is the natural end condition after the first climax?" Connects to Principle 8 (single climax shape).
- **Status:** live design question, folded into the GLC work. Tracked as the win-condition question inside GLC, not as a standalone OQ until GLC clarifies the shape.

**4. Health/Armor: power curve problem (one-shot vs unkillable).**
- *Brainstorm:* pieces either fall in one hit or can't be killed. Two paths: (a) scaling power curve, (b) chess-like flat power with emergent progression.
- *asan:* third path — just don't have games long enough for the late-game wall to materialize. GLC may dissolve this.
- **Status:** parked behind GLC. Re-evaluate Armor performance after the length cut lands. Connects to OQ-11.

**5. Armor as "easy progressive escape" from creativity.**
- *Brainstorm:* players use Armor to avoid clever moves. Need balance: actually-defensive play allowed, but no always-available auto-action.
- *asan:* a useful exercise — *look at every existing system/skill from both attack AND defense perspectives* to surface "better defensive alternatives to Armor." Also: re-check Armor's performance post-GLC before committing to alternatives.
- **Status:** parked. The "attack/defense lens for every skill" is a candidate exercise for after GLC. Connects to OQ-11.

**6. The "creative/clever is too hard" problem — Go comparison.**
- *Brainstorm:* tracking burden (a) + branching factor (b) makes cleverness expensive. Go has simple rules but huge breadth — players play by feel because *most moves are low-impact*. Our game is the opposite: few high-impact moves, but currently with a long total move count, so the "few moves" claim is false in practice.
- *Sub-finding:* high-impact moves punish small oversights too harshly; current game has high move count, so a small mistake AND a long game compound the punishment.
- *Ideal shape:* at decent skill level, every turn has roughly equal impact on game progression — no "this turn is nothing" / "this turn is everything" swings. Also: a half-decent move should be reachable in ≤1 minute of thought; 10-minute thinking should be *rewarded* but not *required*.
- *asan:* "how to reduce per-turn impact variance" is a later question, after GLC.
- **Status:** parked behind GLC. The "≤1 min reachable / 10 min rewarded" ratio is a candidate feedback-form KPI for post-GLC playtests.

**7. What are players actually competing on?**
- *Brainstorm:* the fun is inventing macro tactics and reading the opponent (rock-paper-scissors flavor). But long games where outcome is decided early kill this. Solutions: (a) shorter games, (b) game structure that allows mid-game macro-strategy revision (more reactive). Current frame already has some of this.
- *asan:* current answer = "out-think the opponent on macro strategy." Some meso layer may emerge depending on changes. Revisit after GLC.
- **Status:** parked. Connects to OQ-56 generally.

**8. Money's purpose is thin.**
- *Brainstorm:* Money currently only does "save vs spend on skills." Could add depth via shops, alternative sinks, alternative faucets.
- *asan:* revisit after GLC. Don't expand the economy mid-cut.
- **Status:** parked. New entry deliberately *not* created in NEXT_STEPS — would violate Principle 7's "fundamental shifts > variable tweaking while core unsettled" if we tune Money before the shape is settled.

**9. Game economy / component map (the "übersicht" idea).**
- *Brainstorm:* designer wants a single diagram/map of all components and their interactions: piece count → HP / Armor / skills → damage economy → Money + actions → activity rate. Should expose where progression compounds.
- *asan:* this map is the next thing the designer will produce in this session.
- **Status:** in-flight as of Session 25. Will land as its own doc (location TBD by designer).

**10. Early-game pacing — leave as-is.**
- *Brainstorm:* slow opening is normal for the genre. Onboarding is the real issue, not opening pacing.
- *asan:* fine as-is. Tutorial-feeling onboarding is desired but deferred until new-player intake is the active concern (pre-made loadouts already addresses Sub-goal A).
- **Status:** parked. No action.

**11. Mid-game "high tension → 10-round low" pattern, and the single-climax shape.**
- *Brainstorm:* P4-style pattern — one big exchange, then 10 quiet rounds. Reframe: compress to *one climax = the end of the game*. Avoid the "up-down-up-down" shape where the second up tells you who won. Also solves the "decided endgame still being played" problem by ending the game while tension is still live.
- *asan:* promote this to feedback-form KPI / observable target.
- **Status:** **promoted to Principle 8** (`design-principles.md`). KPI form (single climax vs sine wave) added to the GLC-feedback-form TODO.

**12. Bodyguard removal — what's the effect?**
- *Brainstorm:* if Bodyguard goes, Champions become free move-attack targets for Guards. Worth a playtest.
- *Designer note:* lower priority than the rest of the brainstorm.
- **Status:** added as new OQ-67 (live, low priority). Connects to OQ-21 (Bodyguard trigger watch).

**13. Draw conditions — remove?**
- *Brainstorm:* current draw conditions feel like dead weight. Either smarter draw logic or remove entirely until needed.
- *asan:* fits thematically into the big GLC change — combine.
- **Status:** added as new OQ-68 (live, gated to GLC bundling).

**14. Multi-Champ combo bonus → extend to non-Strike skills.**
- *Brainstorm:* current Strike-only scope feels mathematically restrictive — narrows what players consider viable macro strategy. Should extend to non-Strike skills.
- *asan:* this is exactly the Stack A G3 target-counter scope-widening — but the designer's lean is "ship this *with* the GLC bundle, not after." Test-one-thing-at-a-time vs ship-it-all-together tension explicitly noted.
- **Status:** marked for inclusion in the GLC change-bundle (per designer direction "das kommt in der sammlung von big changes welche der GLC sind"). Move from Stack A G3 (gated-on-Stack-H) into the GLC bundle. To be reconciled with Stack A G3 when GLC stack is drafted.

---

*All "parked behind GLC" items above should be re-surfaced after the GLC stack lands and the new baseline stabilises. Until then, they are not action items — they are reasoning context for why GLC was chosen as the path forward and why adjacent improvements are deferred.*

---

## Pre-Made Loadouts for New Players (Pole A Onboarding) — Session 25

**What it fixes / improves**: OQ-56 Problem A (draft entry complexity — new players cannot evaluate skills during draft because they have no on-board experience). P5 confirmed Pole B was the *wrong* solution for this problem — it replaced "no idea what skills do" with "too much happens to plan." A targeted fix inside Pole A: new players skip drafting entirely and pick one of N curated starter loadouts. Experienced players continue to draft freely.

**Pre-thought design**:
- 2–3 curated starter loadouts per side. Candidate identities: *Aggro* (Strike-heavy), *Defense* (Armor + Heal), *Combo-focused* (cross-category combo enablers).
- New player picks a loadout (simultaneous reveal — connects to OQ-62) rather than drafting individual skills.
- Loadouts use only the existing skill catalogue — no new skills required.
- Experienced players can opt in to pre-made loadouts as a "quick game" mode or play full draft.
- "Experienced" = subjective — player chooses their own mode per game.

**Justification (per Rule)**: Solves OQ-56 Problem A without introducing the felt-PI / pure-reaction problems Pole B surfaced (OQ-64). New players get a comprehensible first game (game-feel improvement tied to the "Two minds, one puzzle" framing — they can read the puzzle from turn 1); experienced players keep full draft depth.

**Risks**:
- Loadouts may bias new players toward a specific play style, hiding variety. *Mitigation*: pick the 2–3 loadouts to span distinct strategic identities, not to teach one "correct" approach.
- Must-pick skills (Armor, Steal, Focus) become more visible if they appear in every loadout — could surface a different chassis-volume problem (OQ-11 / OQ-12).
- Defines a "starter" tier of skills — risks splitting the catalogue into "easy" and "advanced" tiers, which is its own design call (Q-E1 tiered catalogue).

**Trigger**: this is now an *active* design candidate, not waiting on a trigger. Likely the next or near-next Active stack in the Pole A revival. See OQ-65.

**Connects to**: OQ-56 (Problem A — direct fix); OQ-62 (simultaneous-reveal pairs); OQ-65 (the open question that frames this); OQ-66 (game length — short games make a "wrong" loadout less punishing, addresses OQ-56 Problem B without needing in-game switching); Q-E1 (tiered catalogue framing).

---

## Game Length 30-60 Minute Target — Multi-Lever Pacing Pass — Session 25

**What it fixes / improves**: Principle 6 (game length is itself attrition). P4 ran 2h30; P5 ran 15 rounds (~?) but felt empty because nobody planned. Target: 30-60 min *with* "overarching tactic" feel preserved. Designer verbatim: *"man will trotzdem noch das feeling haben von 'ich habe eine overarching taktik und will diese anwenden um zu gewinnen'."*

**Why this is a backpocket-entry instead of a single stack**: no single lever clearly delivers the target. Likely a stacked pass over multiple sessions. Candidate levers inventoried:
- **Stack H — Armor Trim** (queued): reduces mid-game stalling at the chassis level. Confirmed effective lever (OQ-11).
- **Stack K — Piece Count Reduction** (queued): less material to grind through.
- **Stack D — Board Geometry** (dormant): tighter board (8×8) forces faster engagement.
- **Pre-made loadouts** (Session 25 entry above): cuts pre-game time.
- **Move/Skill action retuning**: under-explored; risk of breaking combo grammar.
- **Win-condition acceleration** (Stack C dormant — King Lifetime HP): converts position into result faster.

**Risk — "short because nobody planned"**: P5 was 15 rounds *because* play collapsed to reaction, not because mechanics shortened the game cleanly. Watch flag: any length-reduction lever must preserve the multi-turn planning window. Length cuts that work by reducing decision space are anti-Principle-4 and must be rejected.

**Trigger**: integrated into Pole A revival as a *measurement axis* — every near-future Pole A stack tracks game length (rounds + wall clock) against the 30-60 min target alongside its primary measurement. Not a standalone stack until single-lever stacks have run.

**Connects to**: Principle 6; OQ-11, OQ-27, OQ-1b, OQ-66 (the open question that frames this); OQ-19 (endgame acceleration — adjacent lever).

---

## Digital Prototype Persistence — Tooling Requirement — Session 25

**What it fixes / improves**: P5 played digitally with Jonathan; Jonathan refreshed the browser mid/post-game and the game state vanished. No export, no log, no feedback artefact — only handwritten/verbal recall of insights survived. This blocks any future digital playtest from producing the analysis-grade data physical playtests do (tracking sheets, photos, transcribable notes).

**Not a game-design fix — a tooling fix**. Lives in backpocket only to document the constraint that any digital prototype work must clear before serving as a playtest surface.

**Pre-thought design** (for whoever owns the digital prototype):
- Auto-save game state to local storage (or backend) every turn.
- Export-as-JSON / export-as-PDF at any time, including post-game state.
- Per-turn log of moves / drafts / activations / damage / Armor changes.
- No reliance on browser tab persistence.

**Trigger**: any future Pole B revival or digital Pole A playtest. Until persistence ships, digital playtests cannot be analysed at the level paper playtests can. Default back to paper for any "this game matters for evaluation" run.

**Connects to**: P5 notes — `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`; OQ-61 (Pole B revival gated on this if it happens digitally).

---

## Armor — Current-Role Audit (Diagnosis Anchor) — Session 23

**What it fixes / improves**: Not a candidate fix. This is the **diagnosis anchor** for any future defense redesign — every Armor proposal should be measured against the role Armor actually plays today.

**Diagnosis** (Session 23, confirmed by user): Armor functions as a **late-game survival tax / mandatory upkeep**, not a strategic choice. User verbatim: *"i 100% agree that armor is like the tax you have to pay. that they are the mandatory upkeep of pieces in the endgame."* The role isn't "absorb hits" — it's "the only thing standing between pieces and instant death in late game." That makes it a chassis tax, not engine identity.

**Two diagnoses ruled out** (so future fixes don't re-litigate):
- *Money curve too steep* — killed: starving Money weakens skills as primary damage and removes fire-and-think tension.
- *HP too thin* — killed: catalogue audit shows no 2-damage skills exist (cheapest 2-damage path costs 0/2/4/6/8 Money); raising HP just shifts the bottleneck to healing.

The shape is wrong, not the magnitude. Future Armor proposals must answer: "does this turn defense into a strategic choice instead of upkeep?"

**Cross-link**: full discussion in `docs/research/path-y-defense-redesign.md`.

**Trigger**: any proposal touching Armor / late-game survival / defensive identity. Read this entry first.

---

## Armor Cap Scales by Round (Pole-Agnostic Candidate) — Session 23

**What it fixes / improves**: Keeps Armor's late-game survival role from compounding into the early game, where pieces should still be fragile enough for cheap kills. Targets the "compounding tax" failure mode of the current flat cap — early-game stacking of Armor reduces the kill-rate window where most strategic decisions live.

**Pre-thought design**: cap on max Armor scales with round number. Sketch: cap = 0 in rounds 1–5, 1 in rounds 6–10, 2 in rounds 11–15, 3 thereafter. Numbers illustrative — real values calibrated against P4 Armor totals (14 / 22 by end-of-game).

**Watch flag**: stacking yet another scaling rule on top of Money (already scales over rounds) and actions (already scales) may cross the cognitive-load line. User self-flagged this risk: *"too many things scaling over time may become boring."* Three round-indexed dials is a candidate ceiling.

**Trigger**: defense redesign work in either pole. Pole-agnostic — applies to Pole A and Pole B alike. If the cross-pole-fixing OQ resolves "test in each pole separately," this entry runs twice.

---

## Combo Bonus — Dual-Counter + Widened Scope (Stack A G3) — Session 22

**What it fixes / improves**: Three confirmed P4 problems at once.
1. **Cross-category crowd-out (OQ-38 G2 finding)**: Strike-only scope crowded out organic Move-into-Strike combos (P3 had them; P4 didn't). Widening the target counter to "any skill that hits an enemy piece" restores cross-category combo grammar.
2. **Late-game offensive lockout (P4 #6)**: Elias's verbatim *"i did not have any other attack champs left"*. With single-counter + Strike-only, when Strike-equipped Champs die, the bonus is locked away. The proposed **attacker counter** (one Champion hitting *different* targets in one turn) gives a single surviving offensive Champion access to the bonus by spreading hits.
3. **Exchange-pit / mid-game stickiness (OQ-58)**: P4's mid-game collapsed into one cluster; pieces removed one-by-one in a single region. Attacker counter structurally rewards **distributing pressure** across multiple fronts — the antidote to one-pit convergence.

**Pre-thought design**:
- **Target counter** (existing G2 mechanic, widened): on each enemy piece, count how many *different* friendly Champions have hit this target this turn. Bonus on 2nd+ hit. **Scope widened**: any skill that hits an enemy piece counts (Strike + hit-causing Move skills). Move-Attacks excluded (free → would over-cheapen).
- **Attacker counter** (new): on each friendly Champion, count how many *different* enemy targets this Champion has hit this turn. Bonus on 2nd+ hit on a different target.
- **Stacking**: intuitive — both counters fire if both qualify on the same hit. If 4 attack skills are made by 2 different Champs hitting 2 different targets, both bonuses can trigger; rare in practice, reward when it lands.
- **Multi-target skills (Tempest)**: tick the counter on each hit piece for now. **Watch flag** — first surgical rollback if dual-counter proves OP, since AOE + dual-counter is the highest-risk interaction.
- **Move-Attacks excluded**: still no counter ticks from Move-onto-tile. Free movement shouldn't earn bonus access.

**Teaching cost (G4 guardrail)**: dual-counter is strictly more complex than current G2. Two parallel counters per turn need a physical tracking solution — board-side trackers or per-piece tokens. Budget for this in the Stack A G3 rule sheet.

**Methodology gating**: Stack H (Armor volume) runs first. Reasoning: chassis-volume is the *confirmed* P4 problem; combo scope is solving an articulated structural problem (exchange-pit) but hasn't been isolated in a single-variable test. Test Armor first → cleaner baseline for evaluating dual-counter.

**Trigger**: Stack A G3 follows Stack H. If Stack H accidentally dissolves the exchange-pit pattern (chassis volume was masking it), dual-counter may not be needed. If exchange-pit persists post-H, dual-counter is the targeted fix.

**Session 25 narrowing note**: P5 designer reflection — the **attacker counter** ("single piece hits multiple Champs gets bonus on 2nd+ different target") felt too generous in headspace simulation. Before Stack A G3 ships, narrow the attacker counter. Candidate narrowings: require the *targets* to be in different categories (Champion vs Guard) for the second hit to qualify; require the hits to be from *different skill categories* (not e.g. Tempest hitting all targets); cap the attacker bonus at +1 (no +2 stacking). Target counter scope-widening (cross-category skills count) stays as-is — only the attacker counter is over-generous on second look. Locks in at Stack A G3 design time, not before.

---

## Plague Skill — Inflict Injured, Bypass Armor — Session 22

**What it fixes / improves**: Provides a non-killing Injured-state-as-payload skill — analogous to Break but for HP state. Currently the only path to Injured is "deal 1 damage to a non-Armored piece" or "deal 2 damage total." A skill that inflicts Injured while ignoring Armor opens combat texture P4 lacked.

**Connects to OQ-57 (Injured state) finding**: P4 showed Injured pieces "lingered briefly then died shortly afterwards." Plague would create a different Injured-pattern — pieces that get Injured *without* a kill setup, forcing the opponent to decide between healing and finishing. Adds tactical weight to the Injured state without making it more punishing mechanically.

**Pre-thought design**: Mystic skill, cost TBD (~3 Money), Range 2. Effect: target enemy piece becomes Injured (loses 1 HP, ignoring Armor). Cannot kill — if target is already at 1 HP, no effect (or alternative: target loses one Armor instead, designer's call at stack design time).

**Trigger**: Skill catalogue expansion (post-Stack-H, post-G3). Bundle with other catalogue additions; do not introduce as standalone stack.

---

## Lucky Strike / Star Strike — Mystic Targeting Anywhere — Session 22

**What it fixes / improves**: Designer note: *"a 'lucky strike' or a 'star strike' that is mystic and allows one skill to be used targeted at any opponent piece on the board (like striking from above)."* No problem-statement attached — flagged as design idea pending Justification Rule writeup.

**Possible justifications to evaluate before staging properly**:
- **Bypasses LoS / blockers** — could be a counter to the "front-pieces-block-everything" issue and add a sente-style threat element (Stack F territory).
- **Range 2 is restrictive for combo setup** — a board-wide skill expands combo geometry.
- **Risk**: too flexible — universal "hit anything anywhere" skill may dominate must-pick lists or invalidate positional play.

**Status**: idea logged, no design done. Re-evaluate when skill catalogue expansion is unblocked. Justification Rule must pass before backpocket-graduation.

---

## Focus Replacement — "Pay More for +1 Range" Mechanic — Session 22

**What it fixes / improves**: P4 confirmed Focus is must-pick (in Q10 lists) but most armies only equip Focus 1-2× *total* (Session 22 correction). Suggests Focus is a *bottleneck* rather than a mass-pick problem — players want it on at least one Champion but rarely on more. Replacing the skill-slot version with a baseline "spend +1 Money for +1 Range on any skill" mechanic frees that slot for combo variety while preserving the +Range tactical lever.

**Pre-thought design**: remove Focus from the skill catalogue. Add a baseline rule: *"any skill activation may pay +1 Money to gain +1 Range on either activation or effect range (caster's choice, same constraint as the existing Focus + Move ruling)."* Maintains current Focus mechanics, just unties them from the slot.

**Risk**: makes every skill effectively Range 3 if Money available; could destabilise the combat-distance balance the Range 2 default is calibrated for.

**Connects to**: OQ-12 (catalogue density); OQ-56 (must-pick density softer than thought, but Focus specifically still bottleneck-ish).

**Trigger**: bundle with skill catalogue expansion or as a dedicated mini-stack. Justification Rule satisfied: removes a must-pick bottleneck while preserving the tactical lever.

---

## Lance + Steal — Merge Candidate — Session 22

**What it fixes / improves**: Designer note: *"combining lance and theft."* Both skills overlap heavily — Lance (1 damage, Range−1) and Steal (1 damage + steal 1 Money, Range 2) are cheap-Strike-skill siblings. Lance is rarely picked when Steal exists; Steal is must-pick (P4 confirmed). Merging would (a) free a catalogue slot for new skills and (b) reduce must-pick concentration on Steal specifically.

**Pre-thought design**: replace both with a single skill — name TBD — that does 1 damage and *optionally* steals 1 Money (caster's choice at activation). Money steal becomes a tactical opt-in rather than a mandatory rider on Steal. Range 2 default; Lance's Range−1 dropped (or kept as a Focus-style modifier).

**Risk**: removes the "Steal is the strong one" identity that P4 confirmed players love. Need to confirm the merged skill still delivers the Mode-B tempo feel.

**Connects to**: OQ-34 (Steal Mode B confirmed); OQ-12 (catalogue density).

**Trigger**: skill catalogue expansion / rebalance pass.

---

## Money — Rename Candidate ("Money" / something more natural) — Session 22

**What it fixes / improves**: OQ-56 onboarding barrier (Niko's first game). "Money" is a custom term that needs teaching; "money" / "gold" / similar would be self-explanatory and reduce vocabulary load on first read. Pure naming change, no mechanical impact.

**Pre-thought**: candidate names: money, gold, energy, mana. Pick one that aligns with the eventual high-concept aesthetic (Phase B — game identity / visual / naming).

**Risk**: minimal. The custom term may have flavour value for veterans, but vocabulary-cost-for-newcomers is a confirmed P4 issue.

**Trigger**: bundle with the next vocabulary/naming pass. Defer until Phase B (game identity work) or until a first-timer playtest specifically benefits from it.

**Connects to**: OQ-56 (onboarding); designer Session 22 note that "a lot of the names were not used or shortened" by the teacher (Elias) during the actual game — informal renaming already happens at the table.

---

## Steal — Cost Nerf

**Problem**: Steal (3 Money: 1 damage + steal 1 Money) may be too strong with Layer 1 economy. With +2/turn income, stealing 1 Money represents ~50% of a turn's income AND deals damage. Creates aggressive "Money race" dynamics and tempo swings that may dominate decision-making.

**Pre-thought fix**: Raise cost to **4 Money**.

**Net cost analysis**:
- Current: Pay 3, deal 1 damage, steal 1 Money → net cost 2 Money + 1 damage
- Nerfed: Pay 4, deal 1 damage, steal 1 Money → net cost 3 Money + 1 damage
- Compared to Lance: Pay 2, deal 1 damage → net cost 2 Money, no steal
- At cost 4, Steal is still economically neutral (steal 1 back), but costs 4 Money up front — less spammable, requires more planning

**Alternative**: Remove the 1 damage (just theft, no damage, cost 2). Makes it a pure utility/disruption skill — likely weaker and less interesting.

**Trigger**: Test in Layer 2 if Steal is still dominant after 3 HP changes. If not dominant, defer further.

---

## Tempest — Push Direction Ambiguity

**Problem**: "Adjacent pieces pushed 1 tile away from target" is ambiguous when a piece is diagonally adjacent to the target. "Away from target" could mean:
- **Option A**: Along the attacker's Path (the line from caster through target). Pieces not on that axis are not pushed.
- **Option B**: Radially — each adjacent piece pushed directly away from the target tile (8 possible directions, one per adjacent tile).

**Current rule text**: Option B (radial, all adjacent pieces pushed).

**Pre-thought note on Option A**: Would make Tempest a more linear, directional skill — easier to block by placing a piece behind the target on the Path. Less chaotic but more readable.

**Trigger**: If Option B creates consistent confusion at the table, switch to Option A and test in the next available layer.

---

## Tempest — Blocker Chain

**Observation**: Tempest's radial push could theoretically create interesting chain interactions — a pushed piece could land on another piece's tile. Currently there is no ruling on what happens when a pushed piece lands on an occupied tile.

**Pre-thought fix**: Pushed pieces stop on the first unoccupied tile in the push direction (they don't displace other pieces). If there is no unoccupied tile in that direction (e.g., board edge or wall of pieces), the piece is not pushed.

**Trigger**: Add this ruling to Tempest's text when the edge case is first reached at the table.

---

## Charge — Extension to Movement Skills

**Current**: Charge only buffs Strike skills (+1 damage to a Strike skill this turn).

**Idea**: ~~Allow Charge to extend to movement skills as well — "one skill this turn gains +1 Range."~~ **Session 7 correction**: +1 Range duplicates Focus's effect. This idea as stated is invalid. If Charge gets a secondary mode, it needs to be something other than Range — possibly +1 push/pull distance, or allowing a Strike to ignore 1 Armor. Needs rethinking.

**Risk**: At cost 3 Money, any extension to movement skills could be very powerful. Might need cost adjustment.

**Trigger**: Revisit when the skill catalogue is more complete and a broader meta is visible. Candidate for Layer 5+ or a separate skill variant.

---

## Focus — "Skill Slave" Problem

**Problem**: A Champion equipped with Focus and a second skill becomes a "Skill Slave" — every turn it just fires Focus then the second skill. No positional agency, no interesting choices. The skill effectively hard-locks its carrier's decision space.

**Pre-thought fix**: Path Proximity rule — Focus only activates if the caster is within Range 1 of the benefiting skill's caster (i.e., the two pieces must be adjacent or very close). This forces the Focus carrier to position actively near the ally they're buffing, adding movement decisions instead of removing them.

**Alternative**: Focus enhances the *next* skill used by the *same* piece only (not any of your pieces). This eliminates the cross-piece buff entirely. Simpler, but loses the cool ally-combo feel.

**Note**: The current "any of your pieces" ruling was decided as canon. The Skill Slave problem is real but acceptable for now — the proximity fix is a deferred solution if it becomes a dominant play pattern.

**Trigger**: If Focus + one-skill-per-Champion becomes the default "correct" draft in Layer 2+, add the proximity constraint.

---

## Skill Catalogue Expansion — Staged Candidates (Session 11)

**Context**: Research (`docs/research/skill-catalogue-balance.md`) and Playtest 2 draft data show the catalogue's problem is not that defensive skills are underpicked — all 3 Shield skills are heavily used. The problem is **too few distinct strategic identities within Shield and Mystic categories**. All Shield skills are passive (add durability, no pressure). Mystic has a must-pick (Focus) and a never-pick (Charge in P2 meta). Research recommends 25-35 skills minimum for meaningful draft variety; we're at 15.

**Design principle for new skills (from Session 11 research)**: Every new skill should pass the **sente test** — does it create a situation the opponent must respond to? Skills that are purely self-serving (passive buffs with no threat) don't dissolve standoffs or create interesting decisions. Dual-purpose skills (defend + create threat) are the ideal.

**Expansion target**: ~25 skills (10 new). Distribute across categories to reach roughly 9 Strike / 6 Shield / 5 Move / 5 Mystic.

---

### Shield — New Candidates

**Thorn Armor (Shield, cost 3-4 Money)**
Grant +1 Armor to target ally/self. If that armor is destroyed by an attack, deal 1 damage to attacker.

- **Sente**: YES — opponent must choose: attack the armored piece and take retaliatory damage, or avoid it entirely.
- **Balancing needed**: As stated, this is strictly better than Plate (same armor + free damage). Needs constraints:
  - Higher cost (4 Money?) so the economy trade is real
  - Thorn only triggers when armor is FULLY broken (not on each hit)
  - Break could explicitly bypass the thorn effect (built-in counter)
  - OR: thorn replaces existing armor rather than stacking (can't have Plate + Thorn on same piece)
- **Status**: Promising concept, needs balancing design before testing.

**Runic Ward (Shield, cost 3 Money)**
Grant +1 Armor to target ally/self. If that armor absorbs damage this round, gain +2 Money at start of next turn.

- **Sente**: YES — opponent attacks into it → fuels your economy. Opponent avoids it → your piece is safe. Either outcome benefits caster.
- **Standoff connection**: Directly incentivises forward positioning — you WANT to be attacked because it's profitable. Walk forward, dare the opponent to hit you.
- **Balancing**: Net cost = 1 Money (pay 3, get 2 back IF hit). If never triggered, you paid 3 for +1 Armor (worse than Plate at 3 for +1). Self-balancing: only good in aggressive/forward positions.
- **Status**: Ready to test. Clean design, clear sente, self-balancing.

**Bulwark (Shield, cost 2 Money)**
Grant +2 Armor to self. This piece cannot use skills for the rest of this turn.

- **Sente**: No — this is a pure defensive commit. Trade: big armor but sacrifice your skill phase.
- **Use case**: King protection when under pressure. "Hunker down" option when you can't afford to both defend and attack.
- **Design note**: The self-restriction prevents it from being strictly better than Plate. It's a strategic choice: maximum defense at the cost of offense.
- **Status**: Ready to test. Simple design, clear trade-off.

---

### Mystic — New Candidates

**Bind (Mystic, cost 3 Money)**
Target enemy piece within range: that piece cannot be moved during the next Move Phase. It CAN still use its own skills.

- **Sente**: YES — pinned piece must either accept reduced mobility (can't reposition) or burn a Move skill (Money cost) to escape.
- **Counterplay**: Move skills (Dash, Swap) are the escape. Gives Move category a new defensive role.
- **Connects to**: Pin/Threatened concept (Topic 1). Bind is the active/drafted version; Pin/Threatened (if ever implemented) would be the passive/positional version.
- **Status**: Ready to test. Clear sente, clear counterplay.

**Energize (Mystic, cost 2 Money)**
Target ally within range: that piece's next skill activation (this turn OR next turn) costs −2 Money (minimum 0).

- **Sente**: Partial — enables a cheaper follow-up but opponent can ignore it.
- **Use case**: Alternative to Focus as an enabler. Focus gives +1 Range; Energize gives −2 Money cost. Different build identity: "range extension" vs "economy enabler." Breaks Focus's monopoly as the only buff Mystic.
- **Key design**: Must carry over to next turn, otherwise it's "pay 2 now, save 2 later this turn" = net zero within a turn (pointless). Carry-over makes it a setup/investment tool.
- **Tracking**: One token on the piece indicating "next skill discounted." Removed after use. Minimal overhead.
- **Status**: Ready to test. Clear alternative to Focus.

**Skill Drain (Mystic, cost 3-4 Money)**
Target enemy Champion within range: their next skill activation this turn costs +2 Money.

- **Sente**: YES — directly taxes opponent's action economy. They must either pay more or change plans entirely.
- **Mirror of Energize**: Energize helps allies, Skill Drain hurts enemies. Together they create an "economy manipulation" sub-category in Mystic.
- **Risk**: Could feel oppressive / "unfun" (opponent's plans are disrupted without counterplay beyond "have more Money"). Monitor carefully in testing.
- **Balancing**: High cost (4 Money) makes it an investment — you spend 4 to make them spend +2, net cost to you is 2 Money for a tempo disruption. Only worthwhile against expensive skills.
- **Status**: Promising but risky. Test with caution — monitor for "feels bad" feedback.

---

### Move — New Candidates

**Mini-Step (Move, cost 2 Money)**
Move self 1 tile along Path.

- **Sente**: No — pure self-repositioning. But enables sente plays (adjust LoS for follow-up Strike).
- **Priority**: LOW — luxury candidate. Only test if sente skills don't already solve game speed. Risk: if efficient, becomes auto-draft and crowds out interesting skills.
- **Use case**: Cheap LoS adjustment. Fills gap between free movement (Move Phase) and expensive Move skills (Dash = 3 Money). The "glue" skill that makes combos possible.
- **Cost decision**: 1 Money might be too spammable. 2 Money makes it economy-comparable to other options. Test at 2, reduce to 1 if underused.
- **Status**: Ready to test. Already in backpocket from Session 8.

**Swap Step (Move, cost 2 Money)**
Swap positions of two of your adjacent allied pieces.

- **Sente**: Partial — surprise LoS changes can create unexpected threats. Opponent must re-evaluate which pieces threaten what.
- **Use case**: Formation rearrangement without burning actions. Put your Strike Champion where your Guard was (and vice versa). Enables surprise combos.
- **Status**: Ready to test. Simple, clear, enables creativity.

**Ram (Move/Strike hybrid, cost 3 Money)**
Move self 1 tile toward target along Path. Push target 1 tile in same direction. If pushed piece hits another piece, that stationary piece takes 1 damage.

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
- **Disrupt (Mystic, cost 3)**: Target loses 1 action. Very powerful / potentially unfun. Superseded by Skill Drain as a gentler economy-tax version.
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
- *Shop*: Spend Money to swap one skill for another during your turn (from a shared pool).
- *Auction*: Both players bid Money for skills from a shared pool at fixed milestone rounds.
- *Opponent swap*: Exchange one skill with opponent at a negotiated interval.
- *Fixed-interval redraft*: Partial or full redraft at milestone rounds (e.g. R10, R20).

**Why interesting**: Collaborative analysis in Playtest 2 — both players evaluating "what's the best move here" together — felt more engaging than pure competition. A designed redraft moment could create a shared strategic pause beat, leaning into the "puzzle-solving together" feel rather than against it. Connects to the idea of the game as a shared experience rather than a zero-sum contest.

**Trigger**: Deferred — do not discuss further or test until Layers 1–5 are complete and core systems are stable. Flag as Layer 6+ candidate or standalone design session.

---

## Move-Attack — Retaliation Variant

**Problem**: If the Layer 2 move-attack nerf (1 damage) makes Guard clearing too slow and extends game length, an alternative approach is needed that keeps move-attacks risky without lowering their raw damage.

**Pre-thought fix**: Move-attacks deal 2 damage as before, but the **attacker takes 1 damage** (retaliation). Melee engagement becomes a mutual exchange. Skills (ranged, no retaliation) become the safe option. Cleverness is rewarded with safety; brute force is punished with self-damage. Common in tactical video games (Fire Emblem, Advance Wars).

**Trigger**: If Layer 2 testing (move-attack 1 damage) shows Guard clearing drags and game length increases. This is an alternative to the nerf, not a complement.

---

## Jump Skill — Movement Through Own Pieces

**Idea**: An ultimate movement skill (Move or Mystic category) that allows a piece to move through allied pieces. Currently all movement is blocked by all pieces (ally and enemy). A "Jump" skill would let a Champion vault over allied Guards to reposition behind enemy lines.

**Why interesting**: Would add a new movement dimension without changing the base movement rules. Especially relevant if the board shrinks (8x8) and pieces are more tightly packed.

**Trigger**: When movement feels too constrained, especially on smaller boards. Candidate for skill catalogue expansion, not a system-level change.

---

## Rewarding Risky Positioning

**Problem**: Both playtests showed a "standoff zone" — 2-3 tile gap between formations that neither player wants to cross first because entering attack range risks heavy damage. First player to commit is at a disadvantage because the opponent can react optimally (perfect information makes this worse than in games with randomness).

**Research findings (Session 11 — `docs/research/forward-positioning-incentives.md`)**:
Five mechanical patterns identified: contested Money generators (Advance Wars), one-time threshold bonuses, objective/VP scoring (Aristeia!/Kemet), sente skills (Go theory), and underdog bonuses. Key insight: the standoff is an incentive-intent gap — our game makes waiting strictly dominant. The attack nerf addresses entry risk but not the cost of passivity.

**Primary solution: Sente Skill Design (design principle, not a mechanic)**:
Rather than adding territory-control mechanics that shift the game's identity, design skills that naturally create threats requiring immediate response from forward positions. The game dissolves standoffs through its OWN systems (skills/combos) rather than bolted-on spatial incentives.

**What makes a skill "sente" (threat-forcing)**:
- Creates a state the opponent MUST respond to or suffer consequences
- Is more effective from forward/contested positions (rewards advancement implicitly)
- Doesn't require a separate tracking system — the threat IS the skill effect

**Current skills with sente properties**:
- Steal: forces economy response
- Tempest: pushes pieces out of formation, opponent must reposition
- Combo bonus (Stack A): if 2 Champions are in range of a target, opponent MUST deal with one

**Current skills WITHOUT sente properties** (opponent can ignore):
- Plate, Heal: self/ally buffs — no pressure on opponent
- Focus: setup for your own future action — opponent can wait

**G1/G8 compatibility (Session 11 — researched)**: Sente threats force *reactive* spending — the defender spends Money/actions to neutralize, not to profit. Both players still feel the shortfall (G1). The attacker's advantage is tempo (they chose when/where), not resources. This is G8-compatible because the tradeoff persists: spending to respond means NOT spending on your own plan. Sente breaks G1 only if responding generates more resources than it costs — avoid that in skill design.

**Implication for skill catalogue expansion**: Prioritize skills that create "must-respond" threats from mid-range positions. Skills that are purely self-buffing don't dissolve standoffs. See Topic 4 (skill gaps) for specific candidates.

**Fallback hierarchy (if sente skills + attack nerf don't dissolve standoff)**:
1. One-time midline crossing bonus (+1 Money per Champion crossing rank 5 for first time) — small, non-compounding
2. Contested Money generators (2-3 midfield tiles producing Money for controller) — shifts identity toward territory control, use only if desperate
3. VP scoring track (parallel win condition for forward presence) — absolute last resort, conflicts with core fantasy

**Anti-snowball safeguards (if generators ever deployed)**: Fixed-rate nodes, presence-required (piece must stay), recapturable (never locked in), "make winning cost resources."

**Trigger**: Monitor in Stack A playtest. If standoff persists despite 1 damage attack nerf, escalate. If standoff dissolves, deprioritize entire section.

---

## Action-Based Money Economy

**Idea**: Tie Money income to board engagement instead of (or in addition to) automatic time-based scaling. Examples: +1 Money for dealing damage, +2 for capturing a piece, +1/turn for occupying centre tiles.

**Why interesting**: Would reward active play and punish turtling. Creates virtuous cycle: clever play → more resources → more clever play.

**Why dangerous**: Snowball effect (winner gets more Money → wins harder). KPI problem — rewards the symptom (dealing damage) not the system (clever play). Move-attacks are free AND would be further rewarded, making them even more dominant. Saying "only skill damage counts" feels arbitrary.

**Designer's KPI analogy**: Like company KPIs that reward one metric and cause employees to optimise for it at the expense of the actual product. Must reward the entire cycle/system, not just one part.

**Trigger**: Only if move-attack nerf + combo bonus together don't fix the passive-play problem. Park until post-Layer-2 data. The existing automatic economy may self-correct when move-attacks are nerfed (skills become primary damage tool → Money spending patterns change naturally).

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
- Incentivises dealing ANY damage to the King (even 1 damage "snipes" matter over time)

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
- What carries over? (Money pools? Skill loadouts? HP states? Everything?)
- Does this change the win condition? (King capture = instant loss regardless of swap, or do you need to "win both halves"?)
- Is this a fundamental game mode, or a variant/tournament format?

**Connects to**: OQ-39 (shared-puzzle direction), OQ-45 (first-player advantage), OQ-13 (first-player advantage data).

**Trigger**: Do not design further until the core loop (Layers 1–3) is stable. This is a game-mode-level idea, not a system tweak.

---

## Cascade Trigger — +1 action on Kill

**Idea**: When one of your pieces kills an enemy piece (by any method — move-attack or skill), you gain +1 action for the remainder of that turn.

**Why interesting**: Rewards finishing a setup. The bonus is tempo (one more action THIS turn) not resources (no extra Money). Creates exciting follow-up moments: kill → reposition to safety, kill → chain into a second exposed target. Incentivises committing to an attack rather than poking safely.

**Anti-snowball properties**:
- One-turn-only (doesn't compound across rounds)
- Still costs Money to use the extra slot (early-game kills barely benefit because Money is scarce)
- Opponent lost a piece = fewer future threats anyway; the slot just lets the attacker capitalise immediately rather than waiting a turn

**Backpocketed restriction (if too easy to exploit via move-attacks)**: Limit trigger to skill-kills only. Test this restriction if playtests show free move-attack kills generating too much tempo.

**Removal condition**: If playtests show the extra slot is never used (players don't have Money to spend), remove entirely. No dead rules.

**Trigger**: Stack F (Cleverness II) or earlier if a natural test opportunity arises.

---

## Pin / Threatened Status

**Idea**: A piece that is in the Path (line of sight) of 2+ enemy Champions is "Threatened" — it cannot be moved during the Move Phase (but CAN still use its own skills, and CAN be moved by Move skills).

**Why interesting**: Rewards surrounding and multi-piece coordination without dealing damage. Creates positional "captures" — you restrict the opponent's options by clever placement. The opponent must use a Move skill (Money cost) to escape, or reposition the threatening Champions away. Connects to the "restriction as reward" pattern from Hive/Go.

**Risk**: Could feel oppressive / "control-losing" for the defending player. May make Movement skills a must-pick (interesting but constrains draft freedom).

**Counterplay**: Move skills become the escape tool (gives Move category a defensive role). Opponent can break the pin by moving one of the threatening Champions. Guard screens can block LoS to prevent pins.

**Open design questions**:
- Does the King count as a "Champion" for pin purposes? (Probably yes — it has skills.)
- Can Guards be pinned? (Probably yes — Guards in LoS of 2 Champions can't move. But Guards don't have skills, so pinning a Guard removes ALL its options except being rescued by a Move skill on an ally.)
- Does "Path" mean direct LoS or does the path need to be unblocked? (Probably unblocked — you must have a clear shot to "threaten.")

**Trigger**: As its own test layer (Stack F or later). Independent of combat/economy changes.

---

## Collision Damage — Universal Rule (speculative)

**Idea**: When a piece is pushed/pulled into a tile occupied by another piece, the stationary piece takes 1 damage. The pushed piece stops on the tile before (does not displace).

**Why interesting**: Makes ALL push/pull skills into positional combo tools. Rewards reading the board and creating "lined up" formations to exploit. Adds depth to Tempest, Blast, Shove, Maelstrom, and any future push/pull skills.

**Risk (identified in Session 11 discussion)**:
- If BOTH pieces take damage: too punishing — creates keep-away zones where nobody advances into push range.
- If only the stationary piece takes damage: could amplify standoff problem (fear of being pushed into allies for splash damage). Opponent clusters less OR stays far away.
- Makes push skills potentially very strong relative to their cost (2-3 Money for damage + displacement + collision damage).

**Why deferred**: The standoff problem must be confirmed dissolved FIRST (via Stack A results). If players ARE engaging closely after the move-attack nerf, collision damage adds exciting interactions. If standoff persists, collision damage makes it worse.

**Trigger**: Test ONLY after standoff is confirmed dissolved (post-Stack A, possibly post-Stack C). Do not test alongside standoff-fixing mechanics — evaluate independently.

---

## Collision Damage — Skill-Specific ("Ram" / "Shove")

**Idea**: A new Strike or Move skill where collision damage is the SKILL'S special property, not a universal physics rule. Example: *Ram (Strike, cost 3 Money)*: Move self 1 tile toward target along Path, push target 1 tile. If target hits another piece, that piece takes 1 damage.

**Why interesting**: Opt-in during drafting (not a universal rule everyone must account for). Counterable (don't cluster pieces). Creates board-reading moments without taxing ALL positioning decisions. Gives the skill a unique identity — "the one that punishes clusters."

**Design space**: Could be a Strike (deals damage + push + collision) or a Move (no base damage, but repositions self AND punishes target's neighbours). The Move version is more novel.

**Trigger**: When skill catalogue expands (Stack F or later). Design the full skill text before testing.

---

## New Skill Idea — Mini-Step

**Idea (Session 8)**: A cheap micro-repositioning skill. *Mini-Step (Move, cost 1–2 Money)*: Move self 1 tile along Path.

**Why interesting**: Fills the gap between free movement (Move Phase, free, up to Speed tiles) and expensive Move skills (Dash = 3 Money for 2 tiles). At 1–2 Money, it's a low-commitment tactical adjustment — nudge a piece into LoS, escape a threat, or set up a combo next turn without burning a full action.

**Design consideration**: If cost is 1 Money, it might be too spammable (essentially free repositioning via skills). Cost 2 Money makes it comparable to Lance in economy but with no damage — the trade-off is "reposition vs. deal damage."

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

**Example — Placeable Plate**: A skill places a "forge" token on a tile. Any friendly piece that moves onto or through that tile can spend Money to gain Armor (e.g., pay 2 Money → +1 Armor). Alternatively: upfront investment model (pay 5 Money to place; any ally on the tile gets +1 Armor for free, unlimited uses until destroyed).

**Why interesting**:
- Creates forward-positioning incentive — you MUST push toward (or protect) your station to benefit
- Opponent must decide: ignore it (let them armor up) or destroy it (costs 1 attack action + positioning)
- Sente property: a well-placed station forces the opponent to either contest the zone or concede the value
- Solves standoff: places something worth fighting over in the middle of the board

**Ownership model (design decision)**:
- **Player-spawned**: A skill places the station (costs Money + an action). Only your pieces benefit. Creates asymmetric board states — your station is your advantage to defend.
- **Neutral / pre-placed**: Stations exist on fixed tiles from game start (or appear at set rounds). Both players can use them. Creates contested zones — whoever controls the tile gets the benefit.
- Hybrid: neutral stations exist, but a skill lets you "claim" or "corrupt" one (flip it to your side / deny opponent access).

**Open questions**:
- Does this violate "no terrain" (ADR-001)? Or is it different because it's player-created/contestable, temporary, and destructible?
- 1 HP = one hit to destroy. Too fragile? Or correct because placement itself cost Money?
- What other station types? (Healing font, Money generator, speed boost tile, LoS blocker?)
- How does this interact with G4 (tracking)? Token on tile = fine. But if effects are conditional (e.g., "first piece each turn"), overhead increases.
- Does this change the game's identity too much toward territory control?

**Trigger**: Discuss before testing. Potentially connects to OQ-40 (standoff dissolution) and Stack D (board feel). Could be its own mini-stack if the concept survives discussion.

---

## [TO DISCUSS] Laser Beam — Line Damage Skill (Session 12 idea)

**Concept**: A high-cost Strike/Mystic skill that deals 1 damage to ALL pieces (ally and enemy?) along the Path line. Pierces through blockers — the line keeps going.

**Possible design**: *Laser Beam (Strike, cost 5-6 Money)*: Choose a direction from caster. Deal 1 damage to every piece on that line until board edge. Does NOT stop at the first piece (unlike normal Path).

**Why interesting**:
- Ultimate/expensive skill — high cost makes it a committed investment, not spammable
- Anti-stalling tool: breaks through defensive walls and "hiding behind Guards" formations
- Forces opponent to spread out rather than cluster (anti-deathball)
- Punishes predictable linear formations — creates a new positional concern

**Open questions**:
- Hits own pieces too? (More interesting but harder to use — reward is positioning your pieces OFF the line)
- Only enemies? (Safer design but less interesting positioning puzzle)
- Does it ignore Path blockage entirely? (If yes: unique mechanic. If no: it's just multi-target Lance.)
- At 5-6 Money: is it ever worth it vs. multiple cheaper targeted skills? Need to ensure there's a board state where it's the correct play.
- Charge interaction: +1 damage to ALL targets? (Probably no — Charge buffs exactly one Strike.)

**Trigger**: Discuss as part of skill catalogue expansion (Stack F). Clearly an "ultimate" tier skill — connects to OQ-50 (major skill action cost) if that's ever implemented.

---

## [TO DISCUSS] Wave Push — Line Displacement Skill (Session 12 idea)

**Concept**: A Move skill that pushes ALL pieces on the Path line 1 tile in the cast direction. Like a shockwave traveling down a corridor.

**Possible design**: *Wave Push (Move, cost 3-4 Money)*: Choose a direction from caster along Path. All pieces (ally and enemy) on that line are pushed 1 tile away from caster.

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
- What happens when a piece is pushed into the board edge? (Stays in place? Takes 1 damage from wall slam?)
- What happens when pushed into another piece? (Stop before it? Collision damage? See Ram / collision damage section.)
- Connects to existing "Push Wave" in Previously Listed Ideas (line 218) — this is a more fleshed-out version.
- Is Wave Pull too strong as a combo setup tool? (Pull 3 enemies into a cluster → Tempest AoE?)

**Trigger**: Discuss as part of skill catalogue expansion (Stack F). Supersedes the earlier "Push Wave" concept in Previously Listed Ideas.

---

---

## [TO DISCUSS] Guard "Skills" — Passive Buff Draft (Session 13 idea)

**What it fixes / improves**: Guards are currently strategically flat — all 6 are mechanically identical, so they're treated as interchangeable bodies. This idea gives Guards their own identity layer (parallel to how skills give Champions identity), so positioning them is a real decision rather than just "where do bodies go." Also opens a new draft axis without adding active-skill cognitive load (buffs are passive — no in-game Money decisions).

**Concept**: Just as Champions equip 2 active skills during draft, Guards could draft passive buffs from a separate pool. These aren't activated with Money — they're permanent traits that change how the Guard plays.

**Example buffs**:
- *Stalwart*: This Guard has +1 Armor permanently (starts with 1 Armor).
- *Flanker*: This Guard has Speed 3 (instead of 2).
- *Sentinel*: This Guard can bodyguard from 2 tiles away (not just adjacent).
- *Anchor*: This Guard cannot be pushed or pulled by enemy skills.

**Why interesting**:
- Adds draft depth without adding active-skill cognitive load (Guards don't need Money spending decisions)
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
- *Round 10 — "The Veil Lifts"*: All pieces gain +1 action for the rest of the game. (Accelerates the endgame — more actions per turn, more Money pressure.)
- *Round 15 — "Desperation"*: Move-attacks deal 2 damage again (reverting the nerf). (Forces engagement if stalling.)
- *First Champion killed — "Blood Price"*: The killer's team loses 2 Money immediately. (Anti-snowball.)
- *Midpoint — "Resupply"*: Both players gain a one-time Money bonus (e.g., +5). (Enables a big skill turn.)

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

**What it fixes / improves**: forces unambiguous rule definitions. Code has no tolerance for "we'll figure it out at the table" — writing an implementation spec exposes every ambiguous interaction (Lance + Injured, Focus + adjacent self-target, push-into-LoS-blocker, Tempest pushed-onto-occupied-tile, etc.) and forces decisions. Output: a cleaner ruleset with no hidden gaps. Doubles as a foundation for the digital prototype if that goes ahead. Concrete catalyst: Playtest 3's R22 was a wasted turn because Elias couldn't resolve an ambiguity at the table.

**Trigger condition**: anytime; scope-limited (write spec, do not build). Could be a single dedicated session, or a slow background pass while writing baseline updates.

**Note**: consider running `/research requirements engineering` first — there are established techniques (use cases, formal specs, behaviour-driven specs, decision tables, state machines) for exactly this kind of "translate a fuzzy domain into unambiguous rules" exercise. A short research pass before starting could surface the right format for our case (rule book → spec) and save us from inventing one from scratch.

**Risks**: low. Time investment, not design risk. Could surface contradictions in current rules that need resolving — that *is* the point. Risk of scope creep into "let's just build it" — keep this as a write-only exercise unless ADR-status decision is made first.

**Status**: `[TO DISCUSS]` — bookmarked exercise.

---

## [TO DISCUSS] Digital Playtest Prototype (web / iPad / Tabletop Simulator) (Session 15 idea)

**What it fixes / improves**: faster playtest iteration cycles, cleaner data capture (auto-logged rounds, attacks, armor, money — fixes the form gaps surfaced in Playtest 3), can play during travel or short windows, and forces rule-disambiguation as a by-product (see "Spec the game for a programmer" entry above). Useful as a *complement* to physical playtests, not a replacement.

**Trigger condition**: travel window with a playtest partner (mentioned: Jonathan), OR after 2+ more physical playtests when iteration speed becomes the bottleneck.

**Scope discipline**: minimum viable = drag-and-drop simulator + long-press wheel for Injured/Armor/skill-equip + side-panel money/round tracking. **No rules enforcement, no AI opponents, no polish.** Treat as a tool, not a product. Decision needs an ADR before any implementation work.

**Risks**: scope creep (polish is bottomless); risk of "the digital version becomes the game" — defeats the screen-free design intent; rule-state divergence between digital and `ruleset-baseline.typ` (digital must source from baseline, not the other way around).

**Status**: `[TO DISCUSS]` — sleep-on-it. ADR required before any building.

---

## Known Potential Issues

*Risks to monitor. Not active problems — but if the trigger conditions are met, these become real.*

---

### Pole B Variant — Skills Cost a Resource to Activate — Session 24

**What it fixes / improves**: Reintroduces an activation gate to Pole B if the free-activation Skill Phase produces feel problems — e.g. burst-turns, no-cost spam, or "I activate everything I drafted, every turn" predictability. The first Pole B prototype runs with no resource cost on activation; this entry is the "add cost back" lever if the absence of a cost reads badly.

**Pre-thought design**: revive a Money-style economy (or a fresh per-turn pool) gating skill activations. Costs would be calibrated to whatever Pole B-specific economy is built — not necessarily the Pole A Money curve. Could also be a non-Money cost (e.g. spend a future action, sacrifice an Equip Slot temporarily).

**Trigger**: Pole B prototype shows free Skill Phase feels too cheap / too predictable / too burst-prone. Re-evaluate the moment the data lands.

---

### Pole B Variant — Per-Skill-Phase Activation Cap — Session 24

**What it fixes / improves**: A simpler counter to Pole B's free Skill Phase than introducing a resource economy. Caps how many drafted skills can be activated in one Skill Phase, regardless of how many are equipped. Forces players to choose *which* skills to fire on a turn instead of dumping the whole loadout.

**Pre-thought design**: cap = 2 or 3 activations per Skill Phase. Numbers illustrative — calibrate against actual prototype data. Cleaner than a cost system because it adds no new resource to track.

**Connects to**: same problem as the "Unstoppable One-Turn Killer" entry below. The cap is the simplest counter; resource cost is the richer counter; pick one based on what the prototype actually breaks.

**Trigger**: Pole B prototype shows hoarding-then-dumping is the dominant pattern.

---

### Pole B Variant — Permanently Equipped (Non-Consumable) Drafted Skills — Session 24

**What it fixes / improves**: Alternative interpretation of the per-turn-draft mechanic — drafted skills stay equipped after activation rather than being exhausted and returning to the pool. Tests a different game-feel along the same Pole B axis: skills are *commitments* (once drafted, always available), not *charges* (drafted, used, gone).

**Pre-thought design**: same as the active Pole B prototype, but skills are not removed on activation. The 12-skill equipped cap (6 Champions × 2 slots) becomes the structural constraint instead of consumption + repool. Drafting becomes a long-term identity choice; the Skill Phase is unconstrained reuse of what you have.

**Why staged separately**: Session 24 user verdict — *"your idea is also worth keeping in the backpocket but it is not what i wanted"*. The consumable model is the active prototype; the permanent model is the alternative-axis variant for if the consumable feel reads as too churny / too pool-mediated.

**Trigger**: Pole B consumable prototype completes 2–3 games. If feel issues point toward "I can't build a coherent identity because skills keep cycling," test the permanent variant as Pole B v2.

---

### Pole B "Unstoppable One-Turn Killer" Burst — Session 23

**What it fixes / improves**: Tracks the risk that hoarding skills under Pole B's per-turn-draft rules (no Money-economy activation gate, only the 12-equipped cap) lets a player set up a single overwhelming turn — drafting passively for several turns then unloading a coordinated burst that the opponent has no read on.

**Status — potential issue, not guardrail.** User verbatim: *"a potential issue that could theoretically occur but is not confirmed to actually exist and hence also not a guard rail."* Logging it preserves the question without committing to a counter that may not be needed.

**Re-evaluate after first Pole B prototype game.** If burst-turns dominate the game-feel, candidate counters include:
- **Per-turn activation cap** (e.g. max N skill activations per turn regardless of Money availability).
- **Fatigue / cooldown** (a fired skill cannot fire again the following turn).
- **Skill-use stagger** (drafting a skill imposes a 1-turn delay before it can be activated).

Do not pre-design these. Wait for the prototype data.

**Trigger**: Pole B per-turn-draft prototype playtest results.

---

### King 3 actions → Ultimate Stay-Back Support

**Risk**: If the King ever gets 3 slots (post-v1 tuning), it could become the ultimate backline healer/buffer — never needing to advance, just stacking heal + buff + buff from safety. This makes the "capture the King" win condition harder to achieve because the King has no reason to be in danger.

**Mitigation ideas**: King-specific slot restriction (e.g., at least 1 slot must be Strike); or King gains 3rd slot only when on opponent's half of the board.

**Trigger**: If King 3 slots is ever tested.

---

### Armor Destruction Skills → Armor Becomes Dead Skill

**Risk**: If anti-Armor skills (Pocket Thief, Rüstungsbrecher) are too strong or too cheap, Armor skills (Plate, Scrap Armor) become a waste of a slot — you spend 3 Money to grant Armor, opponent spends 2 to strip it instantly.

**Mitigation ideas**: Anti-Armor skills must cost ≥ the Armor they destroy (economy-neutral at best); or Armor grants some residual benefit even when removed (e.g., the piece gets +1 Speed for 1 turn as the "Armor drops off" benefit).

**Trigger**: If anti-Armor skills are added to the catalogue.

---

### Temporary Effects Tracking Overhead

**Risk**: Any mechanic with a duration (Temp Armor, shields, speed boosts, debuffs) creates tracking overhead on a physical board. Without a solution, these become cognitive-load traps that slow the game.

**Known approaches to research**: Tokens placed on pieces, countdown dice, card sleeves with markers, turn-track markers. See `/research how board games track temporary effects on pieces`.

**Trigger**: Before any temp-duration mechanic is added to the catalogue.

---

### Move action Loss as Debuff → Feels OP

**Risk**: Restricting the opponent's actions (they move one fewer piece this turn) is an extremely powerful tempo debuff. It directly removes agency and could feel unfair / unfun regardless of balance.

**Mitigation ideas**: Very high Money cost (5+), or limited to "only when target is Injured" (conditional). Or: don't reduce actions, instead reduce Speed by 1 for that piece (softer, still feels like a debuff, has existing mechanical precedent via Injured).

**Trigger**: If this debuff type is ever proposed for the skill catalogue.

---

### Steal Dominance (existing — OQ-34)

See existing section above ("Steal — Cost Nerf"). Monitoring in Layer 2.
