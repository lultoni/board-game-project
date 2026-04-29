# Balancing Offensive, Defensive, and Utility Ability Pools in Tactical Games

## Executive Summary

No universal "ideal ratio" of offensive to defensive to utility abilities exists across successful tactical games — the split is always a design consequence of what each category *does to the game state*, not a target ratio to hit in isolation. The dominant pattern across BattleCON, Exceed, Summoner Wars, Tash-Kalar, and Flesh and Blood is that **defensive and utility abilities become competitive picks only when they generate initiative (sente), not merely mitigate incoming damage**. The failure modes — must-pick and never-pick — both stem from the same root cause: abilities that either unconditionally dominate a dimension or have no tempo impact on the game. For a 10-pick shared draft from a bespoke catalogue, available evidence and design practice suggest a minimum of **25–35 unique skills** to resist solving; with context-dependent combo potential, the ceiling shifts toward 40+.

***

## 1. Ratio Analysis: What Published Games Actually Use

### BattleCON's Embedded Ratio Logic

BattleCON does not categorise its abilities as offensive, defensive, or utility — it embeds all three functions into a combinatorial pairing system. Each player has access to **6 universal Bases** (Grasp, Drive, Strike, Shot, Burst, Dodge) and **5–6 unique Styles**. The Bases roughly map as follows:[^1]

| Base | Primary Function | Secondary Function |
|------|------------------|--------------------|
| Grasp | Offensive (fast, melee) | Positional (push/pull) |
| Drive | Offensive (mobile attack) | Utility (advance to close range) |
| Strike | Offensive (high power) | Defensive (high Stun Guard) |
| Shot | Offensive (wide range) | — |
| Burst | Defensive/Evasive (retreat + late hit) | — |
| Dodge | Utility (pure repositioning, 0 damage) | Anti-stun (allows reactive attack) |

The ratio of Bases is roughly **4 offensive : 1 defensive : 1 pure utility** — but this understates how Styles cross-pollinate functions. A character's Style can turn Dodge into a threat (by granting it damage or a powerful Start-of-Beat effect), or turn Strike into a defensive manoeuvre (by granting high Soak). The actual ratio of *functions in play* at any moment is fluid because of multiplicative pairing. Brad Talton's design principle is that "the unique ability of the character and the Style choices define the gameplan — offense defines what you're doing; defense should be universal and shared".[^2][^3][^1]

The Dodge base is a case study in utility competing with offense: it does **zero damage** but allows the player to reposition anywhere on the board and still trigger a full reactive attack (since the opponent's attack likely misses). Beginners routinely dismiss it as "doing nothing" — experienced players recognize it as initiative-preserving and combo-enabling. The design insight is that Dodge's value is **tempo recapture**: it costs your opponent their offensive beat without costing you yours.[^4][^5][^1]

Stun Guard (BattleCON's defensive stat) appears on Strike (defensive-offensive hybrid) and certain Styles. The Force Overload ante system lets any player spend 2 Force to add **+2 Stun Guard** to any attack pair — a universal defensive floor that can be layered on top of aggressive attacks. This implements the Guilty Gear "universal defense skeleton" principle: you never *have* to pick a defensive ability, but you always *can* build one.[^6][^1]

### Exceed Fighting System

Exceed's Normal set (shared by all characters) includes **Block** as the dedicated defensive card. Block has **3 Guard**, which in the system means it absorbs stunning attempts from attacks up to 5 Power without the defender becoming stunned. The design is explicit: Block exists so that no character is ever completely without a defensive response. What makes Block interesting is that it simultaneously enables the opponent to be more aggressive — knowing you have Block, they may overcommit, creating counter-attack opportunities.[^7][^8]

Each Normal card in Exceed is dual-purpose: the top 80% is an attack, the bottom yellow bar is a Boost (repositioning, buff, or setup). This means even aggressive-looking cards serve utility functions when resources are tight. The Exceed system maintains roughly a **2:1:1 offense-to-defense-to-utility split** among Normals, but character-specific cards skew heavily offensive with specialist utility — mimicking the Guilty Gear formula of unique offense on a universal defensive skeleton.[^9][^10][^11]

### Summoner Wars (2nd Edition)

Summoner Wars separates ability categories more explicitly. A standard faction deck contains:[^12][^13]
- 1 Summoner
- 3 Champion units (each with unique special abilities)
- 16 Common units
- **6 Standard Events** + 2 Epic Events

Events are the clearest category analogue to your skill catalogue. In Summoner Wars 2e, Event cards span offensive boosts (*Heroic Feat*: +2 attack to a unit), direct damage, **defensive reactions** (blocking attacks, healing), and utility/control (repositioning, disruption of the opponent's economy). The faction design breakdowns on SW-Zone rate each faction on Offense, Defense, Mobility, Reach, and Control as distinct axes — no faction scores maximum in all five.[^14][^15][^16]

Critically, Summoner Wars makes defensive events *sente* by tying them to **opponent response**. A defensive event that lets you keep your Summoner alive under pressure forces the opponent to reassess their attack plan entirely — because killing the Summoner is the win condition, any ability that threatens to prevent that is inherently sente. This is the key insight: defensive abilities become "worth picking" in Summoner Wars because the threat of using them changes how opponents sequence attacks.[^17]

### Tash-Kalar: Flares as Dual-Function Disruptors

Tash-Kalar's card taxonomy separates **Common beings** (placed to build patterns), **Legendary beings** (high-impact summoned entities), and **Flare cards** (the closest analogue to utility/disruption). Flares in Tash-Kalar are explicitly designed to be reactive and disruptive — they interrupt opponent patterns mid-setup, collapsing the "pre-condition" for summoning a powerful being. This makes them inherently sente: if the opponent is clearly building toward a powerful pattern, playing a Flare to disrupt that pattern *forces a response* (the opponent now either plays defensively or attempts an alternate route).[^18][^19][^20][^21]

The Tash-Kalar design also demonstrates an important failure mode. Flares that disrupt without creating their own forward pressure were reported as feeling "purely reactive" and were underselected in playtest drafts. Flares that both disrupted *and* set up your own patterns were selected much more frequently. The redesign for the 2019 edition updated Flare cards specifically to add proactive effects alongside their disruption.[^21]

### Codex: Designing with Explicit Resource Pools

David Sirlin's design notes and community analysis of Codex describe the game as explicitly managing **offensive capability pools** and **defensive capability pools** simultaneously. Codex's specs (spell schools) within its colour system each have offensive specialists and utility/defensive specialists. The "Balance" spec (within the Green faction) is explicitly the utility/control school — its spells manipulate the board state and resource flow rather than dealing direct damage.[^22][^23]

Sirlin's approach in Codex is to make vulnerability explicit and attackable: your heroes who cast spells, and your Tech buildings, can all be attacked. This creates an environment where defensive investment directly prevents the opponent from deploying their most powerful tools — making "I'll protect my Tech II building" a defensive act that is simultaneously an offensive strategy. This is a sophisticated implementation of the sente principle: a well-positioned defensive choice takes the initiative away from the opponent.[^24]

### Guilty Gear (Sirlin's Framework) and the Ratio Principle

Sirlin's seminal analysis of Guilty Gear's balance methodology establishes the clearest theoretical framework for the offensive/defensive ratio question. The formula is:[^10][^25]

> **Universal Defense: all characters share access to a robust suite of defensive failsafes. Unique Offense: each character has extreme, idiosyncratic offensive tools.**

Guilty Gear's universal defense toolkit includes:[^26][^10]
- **F+P move**: upper body invulnerability anti-air, available to every character
- **Faultless Defense (Green Block)**: blocks projectile chip damage, pushes opponent away, costs super meter
- **Instant Block (White Block)**: reduces blockstun on precise timing, costs no meter
- **Alpha Counter (Dead Angle Attack)**: cancels blockstun into a counter-attack, costs super meter
- **Burst**: once-per-round "get out of jail free" escape from any combo

This suite means the offensive/defensive ratio in terms of *card slots* or *ability slots* can skew heavily offensive — because the defensive floor is systemic, not slot-based. For your design, the lesson is that **some defensive capacity can be embedded in universal mechanics** (like a base-level Rune cost reduction when defending, or a "counterplay" token), freeing your catalogue slots to focus on meaningful choices rather than defensive prerequisites.

***

## 2. Why Defensive Abilities Compete with Offense: Seven Mechanical Patterns

### Pattern 1: Defensive Abilities that Generate Sente

The most powerful pattern for making defense competitive: abilities that protect you *and* create a threat the opponent must respond to. In Go and tactical card game theory, a sente move is one that forces the opponent to answer rather than pursue their own agenda. Purely passive defense is gote (the opponent ignores it and continues attacking). A defensive ability becomes sente when:[^27][^28]

- It creates a durable threat that compounds over time if not answered (e.g., a Shield that regenerates at end-of-turn, forcing the opponent to break it or the Champion gains unstoppable momentum)
- It repositions a piece into a tactically advantageous square while also providing cover (Dodge in BattleCON is the canonical example)
- It generates a resource when triggered by opponent attacks (spending the opponent's offensive turns to fuel your own economy)

### Pattern 2: Positional/Tempo Abilities as Pseudo-Defense

BattleCON's Dodge base is not a damage-soaking ability — it is a positioning ability that *happens* to avoid damage as a side effect. This reframing matters enormously. When Dodge is understood as "I place myself advantageously for next beat while denying my opponent their hit," it becomes clearly valuable without requiring a numerical defensive stat.[^29][^1]

Flesh and Blood explicitly teaches this principle: "tempo is often more important than damage. You may want to allow some damage in if it means you're going to be able to throw out a huge attack of your own and try to seize the initiative". A repositioning skill in your design — one that moves your Champion plus a friendly unit — achieves both defensive displacement and offensive setup, making it a draft pick not because it reduces damage but because it creates the right *next board state*.[^30]

### Pattern 3: The Dual-Use Pitch Mechanic

Flesh and Blood's pitch system is arguably the cleanest published solution to making defense non-sacrificial. Every attack card can also be "pitched" as a resource, and many can be used to block. This means playing offensively does not require giving up defensive options — the question is *which* offensive card you sacrifice to fuel the engine. Defense Reactions in FAB are the category that does defense *better than regular cards* (blocking for 4+ vs. typical 2–3) and also trigger useful on-play effects.[^31][^32][^33][^30]

For your design, the analogue would be: **any skill card can be discarded to generate a Rune** (even if it wasn't selected in the draft). This creates a safety valve where "wrong" draft picks become resource, and intentional defensive builds can "bank" unused aggressive options.

### Pattern 4: The Contextual Threat-Response Loop

Reddit discussions of defensive card design identify the core failure of most defensive cards: "Ignore or destroy is frustrating to the offensive player; Reduce is unappealing because it rewards a stat the opponent already invested in". The solution observed in successful designs is **conditionality**: defensive abilities that trigger specific conditions (e.g., "if opponent attacks with a Mystic skill, this Shield gains +2 and generates 1 Rune") make both the defensive pick and the offensive counterplay interesting. The opponent now has to decide whether to avoid that category, creating genuine decision-making value.[^34]

### Pattern 5: Defensive Abilities with Embedded Resource Generation

Summoner Wars' defensive events that protect the Summoner are competitive with offensive events because losing the Summoner is an immediate loss condition. In your design, Kings serve this function. A Shield skill that simply reduces damage is underwhelming; a Shield skill that reads "reduce damage by 3; if damage is reduced to 0, generate 2 Runes" creates an economy loop that rewards intelligent positioning and makes the opponent think twice before committing a full offensive combo.[^17]

### Pattern 6: Codex's "Attack the Infrastructure" Defense

Sirlin's Codex creates defensive value by making the things you protect *the source of your offensive power*. Protecting your Tech II building is not a passive act — it preserves your ability to summon your best units. This translates to your design as: **abilities that protect Champions also preserve the combo engine those Champions provide**. A Shield skill on a Champion with a powerful Mystic combo is worth drafting precisely because it keeps that Mystic combo online.[^24]

### Pattern 7: Stun-Avoidance as Asymmetric Timing Defense

BattleCON's Stun Guard system creates a specific niche for defensive investment: avoiding the "stunned" state that causes you to lose your reactive attack activation. Because being stunned costs you a full action, any ability that prevents it is worth comparing to the offensive upside of being stunned-through. This asymmetry — where stun prevention has a clear situational value floor — is a model for how to make defense feel mechanically necessary rather than numerically compensatory.[^35][^36]

***

## 3. Utility as Combinatorial Glue

### The "Glue" Mechanism

The best tactical game designers treat utility abilities not as a third category but as **interaction amplifiers** — skills that make both offense and defense more interesting by creating new interaction surfaces. In your design context, Move and Mystic categories serve this function when they create board states that change how all other skills operate.

Onitama is the purest example: all 16 movement cards are purely utility (no attack stats), yet the game's entire tension emerges from the interaction between movement options. A card that moves pieces in an L-shape is neither offensive nor defensive in isolation — it becomes either, depending on the position it creates. Onitama also demonstrates a critical combinatorial property: with only 5 cards in play from a pool of 16, you get **4,368 possible card combinations**, each creating a distinct game. This suggests that even a small card pool generates meaningful variety when the cards interact with the board state rather than simply adding/subtracting numbers.[^37][^38]

### Repositioning as Tempo Control

In Summoner Wars, the SW-Zone analysis identifies **Mobility** (ability to move and place units at odd angles) and **Control** (ability to manipulate the opponent's units or force suboptimal play) as distinct from pure Offense and Defense. Factions rated highly on Control are not necessarily strong on Offense — but their Control abilities create sente because they force the opponent to respond to threats of displacement rather than damage. A unit ability that teleports an enemy piece away from the King's guard zone is forcing a response just as effectively as a direct-damage threat.[^16]

### Utility as Combo-Enablement

In your design, where the core fantasy is "discovering and executing clever spell/skill combos," utility abilities should function as *enabling conditions* for more powerful offensive or defensive payoffs. A Move skill that repositions a Champion adjacent to two enemy pieces enables a Strike skill that hits both; a disruption Mystic that removes Runes from the opponent enables a strategic window that makes the following Strike skill hit for greater effect. This transforms utility from "extra option" into "required setup" for the most satisfying play patterns.

### The Control Category and Forced Responses

The Netrunner/Go sente literature describes utility-as-sente precisely: "A sente move has the property that the opponent *must* answer it or suffer a strategic disadvantage". In your design, a skill that forces the opponent's Champion off an optimal tile is sente utility: the opponent either uses their next action to correct the position, or they're vulnerable to the follow-up. This is more interesting than a raw damage skill because it creates a **two-move decision chain** rather than a single-pass damage trade.[^39][^27]

***

## 4. Dual-Purpose Abilities: The Design Sweet Spot

### What Makes an Ability Dual-Purpose?

The ideal skill for your design goal — "sente skills that aren't purely self-serving" — follows a specific pattern:

> **The ability is good for you regardless of whether the opponent responds. If they respond, it's still good. If they don't respond, it's even better.**

BattleCON's Burst base exemplifies this in compressed form: it retreats you while *also* hitting a specific range that only activates if the opponent fails to retreat. It is defensive (creates distance), offensive (threatens damage if they stay), and forces a decision. The opponent cannot simply ignore it.[^1]

### Specific Dual-Purpose Design Examples from Published Games

**BattleCON — Dash Base (Drive base variant):** The Before: Advance 2 effect means the attack closes range before the hit check. This functions offensively (closes range to land the attack) and as mobility utility (repositions the attacker advantageously for future beats). If the opponent retreats with their Start of Beat, they waste their defensive positioning. If they don't retreat, Drive hits. Either outcome advances the attacker's gameplan.[^2][^1]

**Exceed — Guard (Block Normal):** Block has Guard 3, which directly stops stunning attempts. But used as a Boost (its lower ability), it provides Force that enables stronger attacks next round. Playing Block is thus either "I need to not be stunned" or "I'm banking resources." Both uses are proactive, not purely reactive.[^8]

**Flesh and Blood — Defense Reactions with On-Play Text:** FAB's best defense reactions (e.g., *Springboard Somersault*, *Rootbound Carapus*) block for 4+ *and* fix your hand or generate synergy with your deck's engine. These cards are sought because they are simultaneously defensive and engine-building — defending with them advances your next turn's plan rather than merely preventing a setback.[^32][^33]

**Summoner Wars — Defensive Events that Set Up Attacks:** Summoner Wars events that create walls (structures) function as dual-purpose abilities: they block movement corridors *and* extend the reach of summoned units. Placing a wall defensively is simultaneously an offensive positioning manoeuvre, because it forces the opponent's advance into a channelled approach where your ranged units have line-of-sight advantage.[^40][^14]

**Tash-Kalar — Flares (redesigned):** The 2019 redesign explicitly added proactive effects to Flares — they now both disrupt opponent patterns *and* clear the board in ways that enable your own faster pattern completion. A Flare that destroys two opponent pieces simultaneously disrupts their summon setup while potentially completing your own pattern. Dual purpose: their disruption is your setup.[^21]

### Design Template for Dual-Purpose Abilities

For your specific context (2-player, 10x10 grid, King kill-condition, Rune economy), a dual-purpose skill pattern should follow this structure:

> **Primary effect**: Protects or repositions your Champion/King.
> **Secondary effect**: Creates a new threat or resets a powerful combo condition for your next action.

**Example template:**
- *Warding Step* (Move/Shield hybrid): Move your Champion up to 2 tiles. Then, the first offensive skill this Champion uses before their next Rune refresh costs 0 Runes. *(Defense through relocation; creates an immediate sente threat because the free offensive skill is now available.)*

- *Mirror Strike* (Shield/Strike hybrid): Reduce incoming damage by X. Then deal X damage to any unit in range. *(Scales with how aggressively the opponent attacks; the harder they hit, the more damage they bounce back.)*

- *Runic Anchor* (Shield/Mystic hybrid): Gain 2 Runes when an opponent uses a skill targeting your Champion this turn. If you have 4+ Runes, this Champion's next skill ignores the target's Soak. *(Rewards being attacked; creates economic sente.)*

***

## 5. Catalogue Size and Draft Diversity

### The Minimum Viable Catalogue

No published academic study directly addresses the minimum card pool for a 2-player shared draft with 10 picks. However, several converging data points inform the estimate:

**Onitama's evidence**: With 16 movement cards and only 5 in play per game, 4,368 possible combinations exist. This suggests that even a 16-card pool generates meaningful diversity if cards interact multiplicatively. However, Onitama's cards are symmetrical in function (all are movement); in your design, asymmetric skill categories interact less combinatorially unless cross-category combos are deliberately designed.[^37]

**Tash-Kalar's draft evidence**: Tash-Kalar's standard draft pools 18–23 cards per player from 4 factions of 18 cards each (72+ total), from which players build 18-card decks. The game's designers explicitly note that draft is only recommended for experienced players, because smaller pools lead to "solved" archetypes too quickly. However, Tash-Kalar's pool is faction-segregated; a shared pool would have different dynamics.[^20]

**Magic: The Gathering draft research**: A 2020 arXiv study on CCG drafting notes that "for a set of 250 different cards, the full landscape of a draft comprises about \(10^{700}\) starting conditions and roughly \(10^{40}\) potential deck-building trajectories". The research also confirms the "solved format" problem: when the pool is small enough that top-archetype cards appear in every draft, the format stabilises toward dominant strategies quickly. The key variable is not absolute pool size but **ratio of desirable picks to total picks**: when more than ~30% of the pool consists of "obvious" picks, the draft becomes solved.[^41][^42]

**Summoner Wars custom deck building**: Summoner Wars' custom deck format allows mixing cards from 16+ factions with a pool of hundreds of cards. The SW-Zone community notes that faction ratings on five dimensions (Offense, Defense, Mobility, Reach, Control) matter primarily because different pools excel on different axes — implying that meaningful diversity requires each pick to have meaningfully different axis implications.[^16]

### Practical Estimates for Your 10-Pick, 2-Player Draft

For a 10-skill draft (5 per player, 2 per Champion), from a **shared pool visible to both players** simultaneously, the following thresholds apply:

| Catalogue Size | Diversity Assessment |
|----------------|---------------------|
| 15 skills (your current state) | Essentially solved; 3 categories are underrepresented; players converge on Strike dominance |
| 25–30 skills | Minimum for meaningful variety; can support 3–4 viable archetypes per combination |
| 35–45 skills | Comfortable diversity; enough "second-tier" skills that early picks have meaningful signal |
| 50+ skills | Rich draft environment; requires deliberate signals and cross-category synergies to avoid "best-in-slot" simply being the rule |

**The critical mechanism**: The draft becomes interesting not primarily through pool size but through **cross-category dependencies**. If certain offensive combos require a utility skill as a prerequisite (e.g., a powerful Strike skill that reads "deals +3 damage if your Champion moved this turn"), then utility picks are not just nice-to-have but necessary infrastructure. This is the Tash-Kalar pattern: pattern-completion requires specific pieces, so no single card category is self-sufficient.

### Preventing "Must-Pick" Dominance

The must-pick problem emerges from one of three failure states:

1. **Asymmetric win condition access**: Strike skills advance the kill-the-King win condition directly; no other category does so comparably → Strike is always better. *Fix: Give Shield and Move skills paths to creating unwinnable King-protection windows or movement-based checkmate threats.*

2. **Uncapped power scaling**: A single category's abilities scale better numerically than others, creating dominant efficiency. *Fix: All skills should have diminishing returns when stacked (two Strikes is not always better than a Strike and a Move).*

3. **Pool too small for diversified strategies**: With only 15 skills, if 7 are Strike, players optimise around Strike because diversifying into a thin pool means no coherent secondary strategy exists. *Fix: Expand Shield and Mystic to at least 5–6 each, with 4–5 Move skills, ensuring each category has a full "suite" of early/mid/late utility.*

The **never-pick problem** is the inverse: defensive abilities are never picked when they have no tempo impact — they are purely reactive and add nothing to the player's own agenda. The fix is to design all Shield skills so they either generate Runes on trigger, reposition advantageously, or create a follow-up threat. A Shield skill that simply reduces damage by 3 with no secondary effect is a dead pick in most states; a Shield skill that reduces damage *and* grants the Champion a free movement, enabling a follow-up attack, is a sente defensive option.

***

## 6. Failure Mode Analysis

### Failure Mode 1: Defense as "Dump Stat"

The classic failure in tactical games where numerical offense beats numerical defense at equal investment, making defense a "dump stat". The Reddit discussion on offense vs. defense mechanics puts it directly: "if no amount of defense boosts will matter against overwhelming offense, defense becomes a dump stat and the meta will favor the naturally stronger stat". In your design, this occurs if Strike skills deal more damage than Shield skills can absorb, with no other compensating mechanism. The fix is to give defense *non-symmetric* payoffs — not more soak, but different *kinds* of outcomes that offense cannot provide (resource generation, initiative theft, position improvement).[^43]

### Failure Mode 2: Defense as Pure Tax

When defensive abilities cost the same Rune investment as offensive ones but only "not lose" rather than "win," they are taxes on survival rather than strategic choices. FAB's approach of giving defense reactions additional text effects (they don't just block — they also fix your hand, generate Arcane damage, or synergise with your class) prevents this. A Shield skill in your game should always do *something additional* when triggered — never "just absorb damage."[^33][^32]

### Failure Mode 3: Sente Defense Becomes Must-Pick

The opposite failure: if a defensive ability is so powerful that not picking it means you have no answer to the opponent's primary threat, it becomes a must-pick that crowds out diversity. The solution is to ensure **multiple defensive options** at different price points and with different secondary effects — so no single defensive pick is the only correct answer to any situation. Guilty Gear achieves this by making its defensive tools universal (free) and limiting their resource cost rather than their existence.[^44][^10]

### Failure Mode 4: Utility as "Trap" Picks

Utility skills become trap picks when they have no path to win-condition advancement. Move skills that reposition without creating a threat opportunity feel like wasted picks. The fix is the Onitama lesson: every positioning option should also create at least one tactical threat (moving your Champion adjacent to the opponent's King is both utility and offensive pressure).[^45][^37]

### Failure Mode 5: Category Lock-In During Draft

In a shared 2-player draft, the first player to pick heavily in one category signals their strategy, allowing the second player to counter-draft. This is desirable in theory, but if categories are too isolated (Strike beats everything, defense never counters Strike), counter-drafting is ineffective. The fix is **cross-category interaction design**: Shield skills that are especially effective against Strike combos, Mystic skills that disrupt Rune-heavy Strike combinations, and Move skills that reposition around the Strike's range requirements.

***

## 7. Synthesis: Design Recommendations

### On Ratio
Expand to a **minimum of 35 skills** with a distribution of approximately **12 Strike : 8 Shield : 7 Move : 8 Mystic**. This 35% / 23% / 20% / 23% split gives enough depth in each category for meaningful intra-category choice while preventing Strike from dominating the pool by volume.

### On Defensive Competitiveness
Every Shield and Mystic skill should pass the **sente test**: does this skill, when drafted and played, create a situation the opponent must respond to rather than ignore? If a Shield skill can be safely ignored (the opponent just attacks again without penalty), redesign it to trigger a resource bonus, a positional shift, or a combo-enabling condition.

### On Utility as Glue
Design at least **4–5 cross-category enabler skills** — skills that amplify other skills in different categories (e.g., a Move skill that sets up a specific Strike trigger, a Mystic skill that unlocks a Shield's secondary effect). These are the combinatorial glue that makes the draft feel like a system rather than a collection of isolated effects.

### On Dual-Purpose Design
Target **30–40% of the catalogue** (roughly 10–14 skills) as genuine dual-purpose abilities — ones that are good in both defensive and offensive postures. These should be the engine cards around which the most interesting drafts revolve. They should never be "best in all situations" but should be "excellent in at least two situations simultaneously."

### On Draft Diversity
Implement **cross-category dependencies** to prevent pure single-category builds from dominating. If the most powerful combos require at least one skill from two different categories, drafting becomes a genuine resource-allocation problem rather than "take the best Strike cards."

### On the Universal Defense Floor
Consider embedding a **universal defensive mechanism** as a game rule (not a skill): perhaps every Champion can spend 1 Rune per turn to reduce incoming damage by 1, regardless of skills. This creates the Guilty Gear "design skeleton" — a defensive floor that prevents catastrophic opening turns — while keeping skill picks focused on interesting choices rather than survival prerequisites.[^25][^10]

---

## References

1. [A Beginner’s Primer to BattleCON - Steam Solo](https://steamsolo.com/guide/a-beginner-s-primer-to-battlecon-battlecon-online/)

2. [Can someone help me understand the hype around Battlecon?](https://www.reddit.com/r/boardgames/comments/29ddbg/can_someone_help_me_understand_the_hype_around/) - Can someone help me understand the hype around Battlecon?

3. [Episode 38: Work In Progress - Brad Talton Jnr - Level 99 Games](https://www.youtube.com/watch?v=pgZXGjZisw8) - Richard is delighted to be joined by Brad Talton Jnr, who runs Level 99 Games and has brought such d...

4. [Top 5 Mistakes BattleCON Beginners Make! (BattleCHAT) - YouTube](https://www.youtube.com/watch?v=vj0sfeHVMr0) - Wanna avoid common newbie mistakes before getting into BCO? Here's how! BattleCON Online on Steam: ....

5. [Tips for getting into BattleCon, and what to think about while playing.](https://www.reddit.com/r/Battlecon/comments/9wx0br/tips_for_getting_into_battlecon_and_what_to_think/) - Tips for getting into BattleCon, and what to think about while playing.

6. [Guide :: A Beginner's Primer to BattleCON - Steam Community](https://steamcommunity.com/sharedfiles/filedetails/?id=1447481784) - Effects that automatically Stun or Ignore Stun Guard have no effect against Stun Immunity. Style: Th...

7. [[PDF] Exceed_Rules_Final.pdf](https://gamers-hq.de/media/pdf/51/58/65/Exceed_Rules_Final.pdf) - To begin playing, shuffle your complete 30-card deck together. A deck is formed from Normals and Cha...

8. [Why does Block have 3 Guard? : r/eXceed - Reddit](https://www.reddit.com/r/eXceed/comments/165ixow/why_does_block_have_3_guard/) - If you get hit by a 6 Power attack and choose to spend 0 Force, you take 4 damage (6 Power - 2 Armor...

9. [Normals — A Guide to Exceed's Basic Options - Level 99 Games](https://www.level99store.com/blogs/exceed-fighter-reveals/normals-a-guide-to-exceeds-basic-options) - These cards are meant to represent attacks and options that any character can do such as blocking or...

10. [Designing Defensively: Guilty Gear — Sirlin.Net — Game Design](https://www.sirlin.net/articles/designing-defensively-guilty-gear) - The more variety there is an asymmetric game, the harder it is to balance. When each character (or f...

11. [Multiplayer Game Balance - Part 1 -](https://evolvingdeveloper.com/multiplayer-game-balance-part-1/) - Sirlin used a fighting game called Guilty Gear as his example. ... One of the best ways to balance a...

12. [[PDF] Summoner Wars: Master Set Rulebook](https://cdn.1j1ju.com/medias/72/a5/91-summoner-wars-master-set-rulebook.pdf) - Unit Type: There are 3 different types of Unit Cards: Summoners,. Champions, and Commons. Some rules...

13. [[PDF] Summoner Wars Second Edition Rulebook](https://cdn.1j1ju.com/medias/d7/25/ba-summoner-wars-second-edition-rulebook.pdf) - To play an event card, pay its cost, resolve its effects, and then discard it. Event cards cannot be...

14. [Help with event card rules : r/Summoner_Wars - Reddit](https://www.reddit.com/r/Summoner_Wars/comments/10firyp/help_with_event_card_rules/) - The rules say that to play an event card, pay its cost, resolve its effects, and then discard it (un...

15. [[PDF] Summoner Wars Frequently Asked Questions - Plaid Hat Games](https://media.plaidhatgames.com/old_images/games/summoner-wars/swfaq2.pdf) - 1.0.0.11: But how do I know which Event Cards confer Abilities and which do not? A: Whenever an Even...

16. [Faction Summarie - SW Zone](https://www.sw-zone.com/factions/overview) - A powerfully well rounded faction with some incredibly strong units and events that can have both ex...

17. [You Don't Kill Your Own Units for Magic | SPACE-BIFF!](https://spacebiff.com/2021/04/16/summoner-wars-2/) - There's no more killing your own units for magic; your options are limited to discarding cards from ...

18. [Beings of Tash-Kalar - Northern Empire](https://tash-kalar.com/beings.html) - Tash-Kalar has four player decks, each representing one school of Tash-Kalar, and one legendary deck...

19. [Tash-Kalar Game Modes and Rules | PDF - Scribd](https://www.scribd.com/document/695252406/Tash-Kalar-Arena-of-Legends-holistic-summary-1-4) - It describes the different play modes including objectives, setup, and scoring. Players take turns p...

20. [Arena of Legends - Variants - Tash-Kalar](https://tash-kalar.com/variants.html) - With two or three players, we recommend Highland, Sylvan and one of the Imperial decks. With four pl...

21. [Tash Kalar Changes Revealed : r/boardgames - Reddit](https://www.reddit.com/r/boardgames/comments/2he504/tash_kalar_changes_revealed/) - All new cardboard components, game board and updated art. Flare cards have also been redesigned. Ess...

22. [Codex : Card -time Strategy Review - YouTube](https://www.youtube.com/watch?v=6JK2xvhPLtA) - On today's thoughts from the corner we discuss Codex Buy stuff on Amazon: https://amzn.to/3GLdgD1 Wa...

23. [The Game Design of Codex - Sirlin Games](https://forums.sirlingames.com/t/the-game-design-of-codex/332) - Manage an offensive capabilities resource pool (mostly by playing units, heroes, or spells); Manage ...

24. [Codex Design: Heroes and Tech Buildings - Sirlin.Net](https://www.sirlin.net/posts/codex-design-heroes-and-tech-buildings) - In Codex, your vulnerabilities are on the table and you have to defend them. If you want to cast any...

25. [The Gaming Den](http://www.tgdmb.com/phpBB3/viewtopic.php?t=57424)

26. [Build a knowledge base for defensive decision-making](https://www.guiltygear.se/guilty-gear-fundamentals-3-defense-part-1/)

27. [Beyond the Three Phase view of Netrunner: Sente](https://daman-asha.livejournal.com/27707.html) - LiveJournal Tags: android , lcg , netrunner If you’ve not read the first post in this (sub)series, y...

28. [Is league like chess? Which games teach lessons that translate to ...](https://www.reddit.com/r/summonerschool/comments/16thmfl/is_league_like_chess_which_games_teach_lessons/) - It's similar to chess in that you are making moves that require your opponent to answer (in Go, this...

29. [BattleCON | BattleTIPS - Beginner's Guide to Positioning - YouTube](https://www.youtube.com/watch?v=UplF7nsbiaA) - In this episode, Marco teaches you some positioning basics in BattleCON! This is a new format that w...

30. [FAB 101: Defense](https://towernumbernine.com/blog/fab-101-defense) - This post covers the basics of Flesh and Blood defense, including blocks from hand, equipment, defen...

31. [The Genius of Flesh and Blood's Pitch System - FABREC](https://fabrec.gg/articles/the-genius-of-flesh-and-bloods-pitch-system/) - On resource management, Flesh and Blood is a clear winner. FAB's pitch system coupled with creating ...

32. [The Top 10 Defense Reactions in Flesh and Blood](https://www.youtube.com/watch?v=BB_YNSMA5kQ) - Defense Reactions are a super prominent card type in Flesh and Blood, but they're not all created eq...

33. [The best generic defensive cards in Flesh and Blood](https://www.youtube.com/watch?v=Z10VQB5kYrQ) - If you're not killing your opponent quick, you might need to block. Let's look at the strongest gene...

34. [What makes for a good defensive card? : r/tabletopgamedesign](https://www.reddit.com/r/tabletopgamedesign/comments/aygxhv/what_makes_for_a_good_defensive_card/) - The defensive cards fill your deck with "okay" card, but protect you from attacks. This means that t...

35. [The Big Game Hunters #6 - Part 3 - BattleCON Tutorial - YouTube](https://www.youtube.com/watch?v=lzttAx_-O3I) - Patrick teaches BattleCON. No animation this time guys, sorry. Note: The game shown in this video is...

36. [Battlecon rules question : r/boardgames - Reddit](https://www.reddit.com/r/boardgames/comments/4g54l1/battlecon_rules_question/) - In the same sense, if an effects stuns you and then x, but you have stunguard, would x happen? Ex: "...

37. [Onitama – Review - Elusive Meeple](https://elusivemeeple.com/2017/06/22/onitama-review/) - Onitama is a short chess like game – so this will be quite a short review and I will post up a few s...

38. [Review: Onitama - One Board Family](https://oneboardfamily.com/review-onitama/) - The cards show you how your chosen pawn is allowed to move. One card may allow the piece to move lef...

39. [Designing an Android: Netrunner deck](https://daman-asha.livejournal.com/29034.html) - Pro-tip #4: Design for sente. ... Or are you giving up too much initiative in striving for the sente...

40. [Strategy Guide - The Forged - SW Zone](https://sw-zone.com/faction/the-forged/strategy-guide) - When building units, the summoner can deal a lot of damage and use that as defense while poking the ...

41. [[PDF] AI solutions for drafting in Magic: the Gathering arXiv:2009.00655v1 ...](https://www.arxiv.org/pdf/2009.00655v1.pdf)

42. [It's a big concern how quickly draft formats are getting solved - Reddit](https://www.reddit.com/r/magicTCG/comments/wz4yxg/maro_on_blogatog_its_a_big_concern_how_quickly/) - At least with commander you can intentionally build lower powered decks. With draft you either build...

43. [Offense vs Defense vs Balanced approaches, when is one preferable to the other? And what makes the meta evolve?](https://www.reddit.com/r/truegaming/comments/c1s2ic/offense_vs_defense_vs_balanced_approaches_when_is/) - Offense vs Defense vs Balanced approaches, when is one preferable to the other? And what makes the m...

44. [Ham, Ethan - Tabletop Game Design For Video Game ... - Scribd](https://www.scribd.com/document/452967640/Ham-Ethan-Tabletop-game-design-for-video-game-designers-2016-Focal-Press) - Game designers and programmers must pick and choose which aspects of real ... stale from the dominan...

45. [Onitama | Game rules - The Chess Variant Pages](https://www.chessvariants.com/rules/onitama) - Onitama is a dynamic CV on a 5x5 board, where each piece moves identically, but in a constantly chan...

