# Path Y — Defense redesign and the two-pole game framing

*Created Session 23 (2026-05-30). Captures the full design discussion that began as a Stack H re-discussion and expanded into a structural reframing of the project.*

---

## Origin

Session 22 closed with a designer flag on Stack H — Armor Trim: *"revisit the bundled-dose framing, scope, and entry conditions in the next session before any rule sheet work."* Session 23 opened by honouring that gate.

Within the re-discussion the user pulled on a deeper thread — *"i feel like i do not really know the reason of existence for armor or the role of armor right now. like why does it exist?"* — and the conversation expanded past Stack H scope into:

1. A diagnosis of what Armor actually does in the current rules.
2. Two new design principles.
3. A two-pole framing of the project (parallel game versions rather than one game with stacked variables).
4. A radical alternative chassis (Pole B — per-turn-draft).
5. Several cross-cutting design candidates (items, scaling Armor cap by round, simultaneous-reveal drafting).

This document is the canonical writeup of that discussion. Everything else (`backpocket.md`, `TESTING_PLAN.typ`, OQs, `mechanics-evaluated.md`) cross-references this file rather than restating its content.

---

## Stack H — Armor Trim: status after the re-discussion

Stack H still has merit. User: *"yes i think it could still be worthwhile for us to explore this, but i also think that we should definitly try and find a solution to 'its hard to think up different combos'. i mean part of it is down to having no game knowledge. its the same with chess: unexperienced players are not able to think moves in the future but pros easily do +5 moves."*

The chassis-volume hypothesis (OQ-11 / Q-C1) is not abandoned — it is **deprioritised** under the new fundamental-shifts principle (see below). Stack H is **Queued**, not Withdrawn, not Dormant. It runs after Pole B prototype work, or sooner if Pole B falls over. The bundled dose remains the lead variant when it does run.

### Specific decisions about Stack H content

- **Rust Shield** stays unchanged for the bundled dose. User: *"i don't think it is teribly bad and we should try it with +2 first. maybe in the future if it lands and rust shield is too nieche or whatever we just remove it and change armor smith to allow self targeting on focus (mabye, but this would open up a whole nother discussion so we only note down that the idea exists before anything else."* The "self-target Armorsmith on Focus" idea is logged as a future option only.

- **Bundled dose stays.** User: *"i think lets just go with the risk - if the games no fun we just stop playing sooner and move on."* The original Session 22 reasoning (cap 3→2 + Armorsmith +1→+2 together, so Armorsmith stays viable inside a tighter cap) is accepted.

- **"Build cheaper than break" risk is bigger, not smaller, than I had framed.** User: *"if it is way easier to stack armor then it is to get rid of it (currently the issue if you do not have armor breaker or standard attacks) the change can exponetiallise this even more."* User's catalogue audit lands the point: if your Armor Breaker champion dies, your only Armor-removal options are 4-Rune Blade Tempest (wasted on full-armor pieces) or Standard Attacks. Within-stack rollback (cap-only, Armorsmith unchanged) is the right contingency *if* Stack H runs and Armor totals climb past the P4 baseline (14 Elias / 22 Niko).

- **Abort signal — deferred.** User: *"i feel like this is arguing about symptoms instead of thinking about the root - lets discuss the other things first before we come back to this."* Root-cause discussion took over the rest of the session; the abort signal is still owed to Stack H whenever it returns to active.

---

## What Armor actually does today

Three role-statements explored in turn, with user pushback.

**1. "Allow defensive strategy."** User agreed with the conclusion that Armor is *a* defensive option, but not necessarily *the right* one. Defense could equally be positional (formations, screens, LoS-blocking), recovery (Field Medic), disruption (Rune Theft), or threat-based (sente skills). Armor is the HP-bar version — closest to D&D / video-game defense, the simplest model, but not necessarily the one that serves the Core Fantasy of combo discovery.

**2. "Pieces feel vulnerable."** User: *"currently i would say that armor 3 really does make a piece feel like number 3 - it cannot die without exessive investment from the opponent (ie you get a major tempo advantage or just major presence from the piece jsut having armor). and the issue is with unprotected pieces: if we get into the later stages of the game (later mid game and end game) it is almost trivial to deal 2 damage, like you get the runes to do that every single turn so in the later stages you need to have your pieces protected to have them not just fly off the board."*

This is the diagnosis. Armor is **a late-game survival prerequisite, not a strategic choice.** In early game pieces can be unarmored because nobody has Rune-fueled offense yet. In mid-late game, an unarmored piece is one that dies on the opponent's next turn. So players don't *choose* defense; they *must* armor up to keep pieces alive long enough to do anything. Armorsmith and Rust Shield aren't strategic picks — they're mandatory upkeep.

User confirmation: *"i 100% agree that armor is like the tax you have to pay. that they are the mandatory upkeep of pieces in the endgame."* And: *"Armor is the only thing standing between pieces and instant death in late game. That's the role."*

That role-statement explains:
- Why both P4 players ran near-identical Armor arcs (R1–5 build, R15–21 re-build) — both paying the same tax at the same time, not strategy convergence.
- Why must-pick density centres on Focus + Armor + Rune Theft. Focus and Theft are offense; Armor is required to survive offense. The catalogue lacks alternative survival mechanisms.
- Why mid-game stalls collapse into Armor clusters when offensive Rune economy outpaces engagement opportunities.
- Why removing Armor entirely without replacement would be catastrophic — pieces would evaporate in late game.

**3. "Pre-battle prep."** User landed cleanly: *"yes this is exactly what i wanted. i wanted players to decide with their draft what kinda srtategy they will do. so like 'i choose movement because i want to flank and zoom around the board' or 'i choose armor (bad example but it shows defense for this point) so i stay back and wait for the opponent to attack me so i can try and counter instead' and so on and so forth - all emergent just based on the skills they choose. i don't want skills to be like 'yeah i had to choose this as this is meta, otherwise i will just be in disadvantage all the time'."*

Defense should be a **draft identity**, not a mid-game upkeep tax.

---

## The three diagnoses (A / B / C)

Three competing fixes were tested. Two were killed; one confirmed.

### Diagnosis A — The Rune economy curves too fast (KILLED)

Hypothesis: the +1 every 5 rounds compounding makes late-game offense overwhelm any reasonable HP system. Flatten the Rune curve and pieces would survive on raw HP alone.

User pushback (kills A): *"if we do not give a lot of runes however it would make executing skills happen less often and make the skills in turn weaker as you only have so many activations and really have to think 'is it worth it to fire my skills now and not be able to do that again for like 3 turns?' so we have to be carefull to not remove using skills entirely as that is still the core mechani in the game."*

Starving Runes guts the engine. Spending tension (G8) and skill-as-primary-damage source both depend on the current scale. A is a non-starter.

### Diagnosis B — HP is too thin (2 is too few) (KILLED)

Hypothesis: 2 HP means any 2-DMG skill = instant kill. If pieces had 3 or 4 HP, Armor's "extends life" job becomes redundant.

User pushback (kills B), with catalogue audit: *"there are no 2 dmg skills. what does exist is either damage buffing or just using the same skill twice. so dealing 2 damage costs 0 (2 standard attacks), 2 (1 attack, 1 lance), 4 (2 lance, really rare tho), 6 (2 normal strike skills) or 8 (2 times blade tempest)."*

And the second blow: *"yes it might make the pieces more damage proof for late game, but in early game you essentially cannot really kill pieces then - do we really want that? that would force games to be so much longer and this could also then create the issue that healing now becomes the new bottleneck."*

Raising HP makes early kills near-impossible (extending games — violates the new game-length principle below) and shifts the bottleneck to healing. The problem isn't HP/Armor magnitude; it is *shape*. B dies.

### Diagnosis C — Armor is the wrong shape (CONFIRMED)

Hypothesis: even at the right curve and HP, the kind of defense being offered is wrong. Armor is passive HP+ — the least interesting kind of defense (no decisions, no positioning, no reading the opponent). The fix isn't to balance Armor; it is to *replace* it with defenses that require strategic engagement.

User confirmation: *"yesss this is exactly the correct call. we do not want to have boring 'oh yeah i dont know what to do so i will just armor up and then i can't die anyways'. we want clever poitioning and skill usage and whatever so the strategic mind of players will be used more (can be taxing again tho ofc)."*

C is the call. Defensive identity should come from drafting choices, not from upkeep.

---

## Two new design principles

Both emerged in this session and are promoted into `design-principles.md`.

### Game length is itself a form of attrition

User: *"i feel like if we in general try and cut down on the game length as the overarching current goal we can note down is 'simplify the game to allow more thinking about combos' and 'shorten the game for players being able to test more strategies and not being burned out by having to play out a single strat over long arduous games' (so the game itself is mental attrition and as we do not want the game itself to have attrition we neither want it in the players brains)."* And later: *"this is a crucial design principle and should be treated as such. every design decision should be viewed from this angle as this now has been the main concern for 3 games/playtests in a row: the game is too long."*

This is a sharper version of "chassis bloat." It says game length is a kind of meta-attrition: a 2h30 game where the winner is "whoever didn't burn out first" is attrition at the player layer even if the in-game economy is clever. It pairs with ADR-003 Principle 4 (cleverness > attrition) and the chassis/engine lens.

### While the core identity is unsettled, prefer fundamental shifts over variable tweaking

User: *"i also generally feel like it should be smarter to make fundamental shifts more often when we still have not figgured out the core identity of the game instead of hyperoptimising the game for the current rules (so instead of 'we have this set of variables - how to we adjust them to produce better results' it should be 'do we even need these variables and could we add new ones in place of them?')."*

This is a **conditional** principle. Once the core identity is settled, the existing Incremental Testing Methodology (one variable per stack) resumes primacy. Until then, take bigger swings — *new variables in place of old ones, not new values for old variables*.

This is the principle that deprioritises Stack H from Active, and promotes Pole B to next-up.

---

## The two poles

Rather than one game with stacked tweaks, the project now carries two parallel game versions.

### Pole A — pre-game-draft

The current game. Skills are equipped during a pre-game draft and remain fixed for the full game. Strategy lock-in via the draft.

Open issue: today's draft is **sequential and reactive** — you read your opponent's pick and counter, so "lock-in" doesn't happen cleanly. User: *"we are currently running toward a 'deterministic perfect game' with how the game draft looks and feels. there is no fundamental strategy picking as it is always better to react instead of doubling down (because then the oppoentn can counter you instead)."*

Pole A's leading internal fix candidate: **simultaneous-reveal drafting.** *"both players pick 2 skills at the same time when both are ready and repeat."* User explicitly accepts a small perfect-information loss *only in the pre-game draft window*: *"i accept losing a tiny bit of perfect information in the 'pre game part' if we uphold perfect information later on."* During play, perfect information stays a hard constraint.

### Pole B — per-turn-draft

The radical alternative. Skills are added to pieces *during* play, not all at once at the start.

**Authoritative mechanics** (corrected to match user pushback):

- **Skills still fire once per use, but are reusable while equipped.** The constraint is *equipped count*, not uses-per-skill. (Earlier misread "skills are removed after use" was rejected: *"they still fire once but you JUST CANT FUCKING EQUIP MORE THAN 12 SKILLS AS YOU PHYSICALLY CANT HAVE MORE ON YOUR PIECES."*)

- **Equipped count cap = 12 skills per player.** 6 Champions × 2 slots each. The King's slots don't change the count. User: *"you have 6 champs (or 5 and 1 king, but does not matter). so you have 12 skills per player."*

- **Shared action slots.** "Movement phase" and "action phase" become **phase 1** and **skill phase** (placeholder names — needs renaming later). Action slots are spent on either *moving a piece* or *drafting a new skill onto a piece*. Choosing to draft is a tempo cost paid against movement. User: *"you get x action slots and can either use them to move pieces or draft skills."*

- **No Rune-economy activation gate.** Activate as many equipped skills as you want per turn. User: *"activte as many as you want - i don't care. you will just then not have them in the following turns. so you can hoard them if you feel like it but of course run the risk of losing pieces."* Hoarding-into-burst (the "unstoppable one-turn killer" risk) is a known potential issue, not a guardrail — see Open risks below.

- **Effectively infinite skill pool for drafting.** *"i would for now just assume that there are infinite skills for equiping, but the amount of skills you do get is the limiting factor."* The 12-equipped cap is the constraint, not pool exhaustion.

### Why both stay alive

User: *"i do not wanna abandon somethign that might serve a different game feel. so like if we see that both rule versions create different feels and stuff then we should maybe think about having 2 modes for the game so that we offer more variety or something and do not lose a conceptually cool game (/game mode)."*

Status framing: **Pole B is an experiment that could replace Pole A later, OR could land alongside it as a second mode.** Not committed-parallel-forever. The decision is open until we have data — see OQ on two-pole parallel design.

### Naming reservation

User confirmed names: Pole A = `"pre-game-draft"`, Pole B = `"per-turn-draft"`. Letter labels (A / B) are stable cross-reference keys; the descriptive names are the human-facing labels.

---

## Cross-cutting concerns (apply across both poles)

These are *not* Pole-B features. They live across both poles as separate design threads.

### Items as a defensive option

Items take a slot where a skill would otherwise sit. A piece could be loaded with "1 skill + 1 item" or "2 items" or stay at "2 skills." Example: a Horse item granting +1 Speed.

**Acquisition is open.** User: *"items do not directly have to be drafted (they are rn, but this is ofc subject to change) - they could also be bought or picked during the game. this is all very possible if we just change the systems around it."* A shop / mid-game-acquisition option is on the table but not committed; per Incremental Testing Methodology, items-as-draft-only would be tested first, with the shop layer added only after.

Why items matter: defense becomes a **draft decision**, not a turn decision — directly serving the game-length principle. You don't spend mid-game turns or Runes on Armorsmith; your loadout *is* your defensive identity.

Risks (acknowledged by user):
- **Cognitive load** — more identities to track per piece. *"yes, real."*
- **Catalogue size** — formally doubles, but bounded in practice. User: *"not directly, we can only do x items for now and just expand later on if we see gaps (same goes for skills tho)."*
- **Draft regret** — exists if items are draft-only and unusable mid-game. The shop / swap mechanics are the answer to this if items go that direction.
- **Items might trivialise positioning** — user view: *"yeah that might happen but i mean the same can be said about any op thing in the game, this is not a item exclusive risk."*

### Armor cap scales by round (Thread 5)

User idea (from the deep-dive section): *"max armor scales with point in the game (so for example: first 5 turn no armor, 6-10 max 1, 11-15 max 2 and so on (runs risk tho of too many things scaling over time maybe and this becoming boring or something?))."*

Property: accepts Armor's late-game survival role and ties it explicitly to the game timeline. Early game becomes lethal (engagement happens fast), late-game survival kicks in when offense is overwhelming. Directly serves the game-length principle.

Risks (called out by user themselves):
- Stacking another scaling rule on top of Runes and Skill Slots = three things scaling = cognitive load.
- More rules, not fewer — opposite of trimming chassis.
- May not solve "Armor as easy default" — once cap is reached, players still default to filling it.

Status: **backpocket candidate.** Triggered by defense redesign work in either pole.

### Defensive identity from drafting choices

This is the underlying philosophy that ties items, Armor's diagnosis, and Pole B together. User: *"i wanted players to decide with their draft what kinda srtategy they will do."* All three threads share the goal of making defense a chosen identity, not an upkeep tax.

### Positional zones — ruled out

User explicit ruling: *"positional zones or terrain stations are not the way to go. this forces players to play a certain way: 'i have to move here because if i don't i just lose' desipte them not wanting to really potentially."*

The argument is symmetric to Armor-as-tax: a forced action rather than a chosen one. Out for the same reason Armor-as-tax is out.

### HP redesign — ruled out

Diagnosis B above. Killed.

---

## Sente-skills — moved out of immediate consideration

Sente threats remain on the table as a Stack F concept (already Dormant in `TESTING_PLAN.typ`), but no Session-23 movement on them. Mentioned here only because they came up as a candidate Armor-replacement in the diagnosis discussion; the user did not engage with them as a primary path.

---

## Open risks (tracked, not blocking)

- **Pole B "unstoppable one-turn killer" turn.** Hoarding equipped skills with no Rune gate could let a player set up a single overwhelming turn. User stance: *"a potential issue that could theoretically occur but is not confirmed to actually exist and hence also not a guard rail."* In `backpocket.md` as a known potential issue. Re-evaluate after first Pole B prototype game. Possible counters if it surfaces: per-turn activation cap, fatigue, skill-use cooldown.

- **Pole B cognitive load.** Drafting *and* playing *and* tracking the opponent's likely future picks (poker-like read). User flagged this as real complexity. G4 guardrail watch.

- **Items doubling cognitive load** if added on top of Pole B (not yet on the active path).

- **Cross-pole fixing methodology.** Open question. User lean: *"its cleaner if we have to run it twice, once per pole, but i also say that it could be confusng if we do not clearly seperate both poles from one another in the testing or actually all the design docs."* Tracked as an OQ; resolved on first encounter, not pre-decided.

- **Pole A draft-determinism.** Sequential draft → "always react" pathology. Simultaneous-reveal drafting is the candidate fix. Tracked as an OQ.

---

## What we are deliberately not doing now

- **No `/research` prompts fired this session.** User: *"i would say to not get distractd by research in this exact ciurrent moment. we leave the concrete research promtps for later and just focus on defining the direction and writing down the insights and ideas and such first rn and stuff."* See "Research to schedule" below.

- **No design-space "axis map."** I had drafted a 4-axis (skill-commitment / activation-gate / defensive-role / information) framing. User scrapped it: *"i don't understand you axis idea and i want you to scrap it. we have 2 game versions. pole a and pole b. in those poles we can change rules that live in pole-based stacks that try and imporve different parts of their pole-game. there are no axies."* Two poles, internal stacks per pole, cross-cutting concerns. No axes.

- **No Pole B rule sheet yet.** That's the next piece of work after the documentation in this plan lands. Lives in `docs/test-scenarios/stack-?-per-turn-draft/` (letter ID to be assigned).

- **No item design pass yet.** Items are a cross-pole concern, scheduled after we see how a bare Pole B prototype plays.

---

## Research to schedule (when ready)

Deferred per user, listed here so they aren't lost:

1. **How comparable 2-player tactical games provide defensive variety without HP/Armor mechanics.** Targets Diagnosis C and the items idea. Looking for examples where defense is positional, active, or draft-time rather than HP-bar.

2. **Commit-and-execute vs adapt-and-respond strategy in tactical board games.** Targets the underlying Pole A / Pole B tension. When does mid-game adaptation work? When does it dilute strategic commitment? What do hybrid models look like?

3. **Equipment / loadout systems in 2-player tactical games** — Aristeia!, Summoner Wars, Star Wars: Rebellion. Practical implementation of items. Defer until research 1 has framed whether items are the right path at all.

---

## Cross-references

- `docs/design-principles.md` — game-length-as-attrition, fundamental-shifts-over-variable-tweaking.
- `docs/backpocket.md` — Armor diagnosis anchor, scaling Armor cap candidate, Pole B one-turn-killer potential issue.
- `game-state/OPEN_QUESTIONS.md` — OQ on Pole A vs Pole B parallel design, OQ on Pole A draft information, OQ on cross-pole fixing, OQ-11 status update.
- `docs/test-scenarios/TESTING_PLAN.typ` — Stack H Queued, Pole B prototype Active.
- `docs/mechanics-log/mechanics-evaluated.md` — Session 23 entries.
- `docs/research/playtest-4-analysis.md` — P4 evidence used throughout this discussion.
- `docs/research/high-concept-open-questions.md` — Q-C1 framing.
