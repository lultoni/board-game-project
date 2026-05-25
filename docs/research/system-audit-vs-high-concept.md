# System Audit vs. High Concept

**Date:** 2026-05-25
**Companion to:** `youtube-transcript-high-concept.md` (Angle 2 of the four work angles)
**Source data:** `docs/systems-and-mechanics.md`, `docs/test-scenarios/shared/baseline-sections.typ`, P1/P2/P3 playtest analyses.

---

## Purpose

Walk every system in `systems-and-mechanics.md`, and ask:

1. **Newcomer perspective (round 1, first ever game):** Does this system push the player toward *chassis-first thinking* (kill pieces, manage army, board control) or *combo-first thinking* (find skill sequences, set up payoffs)?
2. **Veteran perspective (5th+ game, engine visible):** Once the combo system is the recognised core, does this system *deepen* the combo experience or just *sit there* as overhead?

The High Concept under test: *"a 2-player perfect-information board game where the only thing that matters is finding clever skill combos before your opponent does."*

Each system gets two tags (one per perspective) and a verdict.

**Tag legend:**

- `combo-serving` — actively sharpens the combo experience.
- `chassis-necessary` — doesn't sharpen combos directly, but combos can't exist without it.
- `chassis-bloat` — chassis volume that exceeds what combos require. Removing or shrinking it would make combos louder, not break them.
- `dual` — does both jobs in different ways for different players.

---

## 1. Turn Structure (Movement Phase + Action Phase, 2 Move Slots, N Skill Slots)

### Newcomer perspective

The two-phase split is **legible** — P1 and P2 both rated it intuitive, no complaints about the structure itself. But the *order* (Movement first, then Action) primes a chassis-first reading: the first thing you do every turn is "pick a piece to move." That's chess-language. The Skill Slots come second, after the player has already framed the turn as "where do my pieces go."

The 2 Move Slots vs. N Skill Slots split is also signal-rich: Movement gets a fixed, simple budget (2). Skills get a *scaling* budget (2 → 5 over time). The structure tells the player "movement is the constant, skills are the thing that grows" — which is high-concept-aligned, but only legible after a few rounds when scaling is felt.

**Verdict: `dual` (slight chassis-first lean for newcomers).**

The phase order plants chassis thinking first. The slot-count asymmetry plants combo thinking second. Net: a newcomer's first-action-of-first-turn is "move a piece," not "use a skill," and that anchors how they read the game for the early rounds. **Real but small effect.**

### Veteran perspective

For experienced players, the two-phase structure becomes the combo *grammar* — Move → set up Skill Path → fire skill chain. The structure is the scaffolding combos hang on, including cross-piece sequences (P3's Air Blast → Hook Pull). The "long think times" reported in P2 are exactly the symptom of a veteran working through combo space the structure enables.

**Verdict: `combo-serving`.** Once seen, the structure *is* the combo medium.

### Net assessment

The Turn Structure is **load-bearing for combos and broadly good**, but its *opening framing* (movement before action, fixed move budget) is the single biggest reason a new player's mental model defaults to "chess with skills" instead of "skills with chess." Worth thinking about whether the *order or framing* of the phases could be tuned without changing the mechanics — e.g., naming the Action Phase first in the rules, or changing how it's introduced to new players.

---

## 2. Resource Economy (Runes)

### Newcomer perspective

Post-Layer-1, Runes are usable from Round 1 — that's the most important thing the economy now does for the high concept. Before Layer 1 (P1 evidence) the economy *forced* chassis-first play in the opening because skills were unaffordable.

But: Runes are still a *currency* with an income table, scaling tiers, and a "save vs. spend" decision. That structure looks-and-feels like a resource management game (chassis-flavored language). For a new player, the very existence of Runes adds a track to the cognitive budget — *before* they've experienced what skills can do. This is the classic problem of "you have to teach the cost system before the player understands what the cost is buying."

**Verdict: `dual`.** Without Runes, no scarcity, no "want more than I can afford" tension — combos lose their meaning as costly choices. With Runes, new players have to learn the bookkeeping before they get to the engine. Net: necessary, but heavy on the on-ramp.

### Veteran perspective

For experienced players, the Rune budget is the *tempo dimension* of the combo game. "Can I afford this combo *and* still have something for my opponent's response?" is a deeply combo-flavored question. The economy is what makes a 4-Rune skill feel like a real commitment, which is what lets combos feel earned. Working as designed.

**Verdict: `combo-serving`.**

### Net assessment

Runes are **chassis-necessary for the system to work at all, dual for newcomers, combo-serving for veterans.** The economy isn't bloat, but its *teaching* burden is real. Possibly the biggest lever for lowering the on-ramp cost without changing what the game *is* would be making the economy less prominent in the rule explanation: not "here's the income table" but "skills cost things; you'll get more over time, don't worry about the exact numbers."

---

## 3. Progression (Skill Slot scaling)

### Newcomer perspective

Skill Slot scaling is **invisible in the first few rounds** — both P1/P2 confirm slots were the limiter only once Layer 1 kicked in. For a new player in their first 5–10 rounds, Skill Slots are just "you can do 2 skills per turn" — fixed, simple. The progression table is on the cheat sheet but *operationally inert* until late.

This is a low-cost system for newcomers. It doesn't push toward chassis or combos — it sits in the background. The eventual escalation (more slots later) is something a newcomer probably won't even reach in their first game (P3 was 24 rounds, slot increase happens at Round 11).

**Verdict: `chassis-necessary`** — sits there, doesn't help or hurt the high concept landing.

### Veteran perspective

For veterans, scaling Skill Slots is what makes **late-game combos qualitatively different** from early-game combos. 2 slots = sequential one-skill-after-another; 4 slots = real multi-Champion combo turns. The progression is the engine that turns single-Champion play (early) into full combo orchestration (late). High-concept-aligned in design intent.

**Verdict: `combo-serving`.**

### Net assessment

Progression is **doing real work for veterans, invisible-and-fine for newcomers.** No changes warranted from a high-concept standpoint. The only soft flag: the existence of OQ-50 (minor/major skill slot cost) suggests Progression might absorb additional complexity later — that complexity needs to *increase combo expressiveness*, not just add bookkeeping.

---

## 4. Skill System

### Newcomer perspective

This is the engine — and it's also the most visible. Skill cards in hand, skill icons on the cheat sheet, skill names called out at the table. *In principle* this should immediately frame the game as combo-first.

In practice, P1/P3 evidence says **the Skill System's combo properties are not legible on round 1**:

- Pasco's favorite moment was a *single* skill (Shadow Shift), not a combo.
- Mario used essentially one skill (Armorsmith) for 80% of his actions. He never *experienced* a combo as such.
- Jonathan recognised exactly one combo (the famous "Shift + Focus + Blade + Medic, Turn 23") in a 26-round game. *One* combo, very late.

What new players see first is the **catalogue surface**: 15 skills with different costs and effects. They pick the cheapest one that solves the problem in front of them. That's chassis-first thinking executed *with* skill cards. The cards alone don't communicate "these are meant to combo" — that emerges only after you've seen one happen.

The Skill Path geometry (Queen-style line, blocked by all pieces) is also a hidden depth: it's *the* mechanic that makes positioning and skills interact, but it's an *implicit* combo enabler, not an explicit one. A newcomer doesn't know that "I can move a piece to enable a skill line" is a combo move until they've seen it played out.

**Verdict: `dual` heavy on chassis-first for newcomers despite being the engine.**

This is the most uncomfortable finding in the audit. The system that *is* the high concept is also the one most likely to be misread as "just a card pool" by a fresh player. The engine isn't self-advertising.

### Veteran perspective

For veterans, the Skill System is everything the design principles promise — high depth, high interconnection, high emotional resonance (5/5 across all three in `systems-and-mechanics.md`). The combo ceiling is currently *limited* (P3 noted only "buff + hit" combos exist organically, Stack A Game 2 will test multi-Champion combos), but the floor is solid.

**Verdict: `combo-serving`.**

### Net assessment

The Skill System is the engine, but **the engine doesn't yet announce itself as the engine.** Two things follow:

1. The on-ramp problem (newcomer-perspective Angle 1 finding) is *concentrated here*, not spread evenly across systems. The fix space isn't "make the chassis quieter" — it's "make the Skill System *present* itself as combo-shaped on round 1."
2. Possible levers (none of these are recommendations yet — just lever-space): explicit combo prompts on skill cards ("pairs well with X"), pre-game "intro combo" demonstrations during teaching, a tutorial draft that hands the new player a known-good combo to discover. *None of these change the rules; they change the framing.*

The other latent issue: the **catalogue is small enough that with bad draft luck a new player might never have access to a combo-shaped pair**. Mario picked 2× Armorsmith and no Rust Shield — his loadout *wasn't* combo-shaped. That's an **on-ramp issue masquerading as a draft issue**.

---

## 5. Combat / Attack (Standard Attack + Bodyguard + Combo Bonus pending)

### Newcomer perspective

The Standard Attack is the **lowest-friction action in the game**: no Rune cost, no slot, just move-onto-tile. It's described in the rules right after Movement, as a sub-clause of Movement. Mechanically and rhetorically it's chassis-first par excellence.

Pre-Stack-A this was actively bad for the high concept — 2 DMG free attacks dominated everything else (Session 7's "standard attack dominance" finding). Post-Stack-A (1 DMG) it's better: attacks now serve as *setups* (apply Injured) and skills serve as *finishers*. P3 evidence is clean on this — Elias's attack:skill ratio dropped to ~1:5.

But for a *new player*, the rules-text framing is still: *"Movement Phase: spend a Move Slot to move OR attack."* Attack is presented as a movement option, not a skill option. The system says "attacking is a chassis verb" through its rule placement.

The Bodyguard rule is a chassis-flavored layer on top: Guard positioning, adjacency requirements, defensive interception. It's strategically valuable (P3 confirmed it triggers organically once standoff dissolves) but it's *deeply chess-coded* — Guards as pawns, intercepting attacks, protecting royalty. A new player learning Bodyguard learns the game as "wargame with rules about who screens whom."

The pending Multi-Champion Combo Bonus is the **only part of the Combat system that's explicitly combo-shaped.** It hasn't been tested yet (Stack A Game 2). If it works, it's a direct rebalance of Combat away from chassis-first toward combo-first.

**Verdict: `chassis-necessary` post-Stack-A, was `chassis-bloat` pre-Stack-A.** Combo Bonus pending could shift this further combo-ward.

### Veteran perspective

For veterans, post-nerf Standard Attack is exactly the right tempo tool: it threatens just enough to force the opponent to respond, but doesn't replace skill-based kills. Bodyguard is genuinely strategic (drafting Guards, positioning, deciding when to sacrifice). Both work as designed.

The Combo Bonus, if it works, will be a veteran's combo *amplifier* — the lever that turns "use a sequence" into "use a sequence with a damage multiplier." Net positive for the high concept at the depth ceiling.

**Verdict: `combo-serving` (especially with Combo Bonus accepted).**

### Net assessment

Combat is the **system most heavily pulled toward chassis** by its rule placement and chess-genre vocabulary. Two consequences:

1. **The Standard Attack's framing in the rules is chassis-first by default**, even though its current mechanics (1 DMG, attack-as-setup) are combo-friendly. Rule-text and rule-placement are levers worth thinking about — does the Movement Phase need to mention attack at all, or could attacks be reframed as their own thing?
2. **Bodyguard is the most chess-coded thing in the game.** It works, it's strategic, but it speaks in the language of war-chess. A new player learning Bodyguard is being taught the chassis vocabulary in its purest form. Whether that's a problem depends on whether Bodyguard is generating its strategic value through *combo-relevant* decisions or through *chess-relevant* decisions.

---

## 6. Health & Armor

### Newcomer perspective

Three states (Normal → Injured → Removed) plus stackable Armor. This is HP bookkeeping — about as chassis-flavored as it gets. Every wargame, RPG, and tactics game has it. The new player slots this into "HP system" without ever asking what makes this game *special*.

Armor in particular is an entire sub-system: tokens to track, max cap of 3, granted-by-skills, removed-by-Armor-Breaker. P3 had Mario stacking ~20 Armor across the game and Elias breaking it ~6 times — that's a *lot* of cognitive load and game time spent on a sub-mechanic that, for a new player, looks like "more numbers to manage."

**Verdict: `chassis-necessary` leaning toward `chassis-bloat`.**

The case for `chassis-bloat`: Health & Armor consume real cognitive bandwidth and table time, and they don't *teach* combos — they teach resource management. Mario's experience is the strongest evidence. He stacked Armor 20 times because Armor-stacking was the most legible action available, and the whole loop (stack → break → re-stack) consumed his entire game without ever surfacing a combo.

The case against: damage *has to land somewhere*; Armor is part of the RPS loop (Armor vs. Armor Breaker) that gives skills strategic depth. Without Armor, the Strike skills become much more deterministic. So it's not pure bloat — but the *volume* of Health & Armor mechanics may exceed what combos require.

### Veteran perspective

For veterans, Health & Armor are the **stakes**. Knowing your King has 2 HP makes the King-threat real (in principle — see P3 finding that the King isn't currently a real target). The Armor RPS loop is genuine strategic depth — drafting Armor Breaker because you read the opponent's Armorsmith intent is a real combo-adjacent decision.

But: even for veterans, the *amount of bookkeeping* (token tracking, stacking up to 3, remembering which piece has how much) is heavier than it strictly needs to be to deliver the strategic experience.

**Verdict: `combo-serving` for the RPS loop, `chassis-bloat` for the bookkeeping volume.**

### Net assessment

Health & Armor are the **strongest candidate for chassis-bloat in the audit.** Two specific concerns:

1. **Armor is doing two jobs**: providing the RPS loop (combo-relevant) and providing extra HP buffer (pure attrition). The second job is what extends games and consumes cognitive load without helping combos. *Possibly the cap-of-3 is too high.* P3's data — Mario stacked Armor 20 times, and the analysis flagged "armor amounts not formally counted" — suggests Armor volume is currently un-managed.
2. **Three states (Normal/Injured/Removed) might be one state too many for a 2-HP system.** With 2 HP and Injured being a transient state often skipped (P1 evidence: 2 DMG attacks skipped Injured entirely; pre-Stack-A this was the norm), Injured was almost vestigial. Post-Stack-A it's relevant again, but the *teaching cost* of Injured (penalties, range modifiers, edge cases like "Injured doesn't affect 'self' or 'adjacent' skills") is high relative to its frequency of relevance for new players.

This isn't a "remove Health/Armor" recommendation — they're load-bearing. It's a "the volume here is the highest in the game, and the volume itself may be drowning out the engine for newcomers."

---

## 7. Skill Drafting

### Newcomer perspective

Skill Drafting is the **first thing that happens** in the game — before any movement, before any combat. Mechanically it should plant combos as the central concept: "you're picking the skills that will define your army." That's high-concept-aligned in principle.

In practice, P1/P3 evidence says it doesn't land:

- Mario: "no plan." He drafted 2× Armorsmith and never built a combo-shaped loadout.
- Jonathan (P2): "the plan came from playing the game more." He didn't *draft* a plan; he *discovered* one mid-game.
- Even P1's Pasco said the draft felt fair but didn't describe it as combo-flavored.

The problem: **the draft is high-concept-aligned for veterans but pre-combo-experience for newcomers.** A new player drafts before they've ever seen what skills *do*, and certainly before they've seen any combos. The draft asks them to make combo decisions before they have combo intuition. The result: defensive picks (cheap, legible, individually useful) dominate first-game drafts.

**Verdict: `dual` — combo-serving in design intent, chassis-first in newcomer practice.**

The draft's design fantasy is exactly the high concept. Its on-ramp delivery is chassis-first because newcomers can only evaluate skills individually, not in combination.

### Veteran perspective

For veterans, drafting is **strategy-defining** — Elias's drafts are visibly combo-shaped (Armor Breaker drafted to counter expected Armor stacking, Focus Strike + Strike pairs, etc.). Q14 in P3 has Elias listing "must-pick" skills (Blade Call, Focus Strike, heal, shield, Rune Theft) — that's a veteran reading the catalogue with combo-sensitivity.

**Verdict: `combo-serving`.**

### Net assessment

Skill Drafting is **the single strongest combo-serving system in design intent** and **the single most on-ramp-hostile system in practice** for newcomers. That's a meaningful tension.

This is probably the most fixable on-ramp problem in the audit. Levers (lever-space, not recommendations):

- **Pre-built starter loadouts** for first-game players: hand newcomers a known-good combo-shaped army instead of asking them to draft one. Trades draft depth for combo legibility on round 1.
- **Drafted-after-first-game**: explicitly tell new players "you draft from round 2 onward; round 1 we hand you a deck so you experience combos first."
- **Combo hints on skill cards**: "Lance Thrust pairs well with Focus Strike." Doesn't change the rules, just the on-ramp surface.

None of these are accepted; they're the menu of fixes that come *out* of the audit pointing here.

---

## Summary Tables

### Per-system tags

| System | Newcomer (round 1) | Veteran (5th+) | Net |
|---|---|---|---|
| 1. Turn Structure | `dual` (slight chassis lean) | `combo-serving` | OK with framing tweaks |
| 2. Resource Economy | `dual` (heavy on-ramp cost) | `combo-serving` | OK; teaching burden real |
| 3. Progression | `chassis-necessary` (invisible early) | `combo-serving` | OK |
| 4. Skill System | `dual` (engine isn't self-advertising) | `combo-serving` | **Engine, but not legible as engine** |
| 5. Combat / Attack | `chassis-necessary` (post-Stack-A) | `combo-serving` (with Combo Bonus pending) | Heavily chess-coded vocabulary |
| 6. Health & Armor | `chassis-necessary → chassis-bloat` | `combo-serving` (RPS) + `chassis-bloat` (volume) | **Strongest bloat candidate** |
| 7. Skill Drafting | `dual` (pre-combo-experience) | `combo-serving` | **On-ramp delivery is the problem, not the system** |

### Where the engine is loud vs. quiet

For newcomers, the systems that **announce combos**: none, fully. Skill System and Skill Drafting are *meant* to but require combo experience first.

For newcomers, the systems that **announce chassis**: Combat (rule placement of Standard Attack + Bodyguard's chess vocabulary), Health & Armor (volume + bookkeeping), Turn Structure (movement-first phase order).

For veterans, all seven systems are roughly combo-serving or combo-amplifying — the engine is fully audible.

**The audit's central finding: the chassis is louder than the engine *only on first contact*. After enough plays, the balance flips. The on-ramp is the problem, not the design.**

---

## Headline findings

1. **The Skill System is the engine but does not announce itself as the engine on round 1.** The catalogue surface looks like "abilities" until a player has seen one combo, and combos require either luck (right draft) or guidance to surface in a first game. This is the *single biggest concrete finding* of the audit.

2. **Health & Armor are the strongest chassis-bloat candidate.** The volume of bookkeeping (3 states, Armor cap 3, granted/removed mechanics, tokens to track) exceeds what combos require, even if both are individually load-bearing. *Possibly* Armor cap should drop to 2, and *possibly* the Injured state could be simplified for first-game teaching. Both are speculative — would need testing.

3. **Skill Drafting's on-ramp problem is fixable without rule changes.** Pre-built starter loadouts, combo-hint cards, or "draft-after-first-game" are framing tools, not rule changes. They could deliver the high concept on round 1 without redesigning the system.

4. **Standard Attack's rule placement teaches chassis-first thinking** even though its current mechanics are combo-friendly. The Movement Phase rules introducing attack as a "spend a Move Slot to attack" sub-clause anchors the player's mental model to chess, not skills. Worth thinking about whether attack could be reframed in the rule text.

5. **Bodyguard is the most chess-coded sub-system.** It works strategically but it speaks pure war-chess vocabulary. Whether this is a problem depends on whether the strategic value Bodyguard creates is *combo-relevant* or *chess-relevant* — currently unclear, worth thinking about.

6. **No system in the audit is pure bloat that should be removed.** Every system pulls weight, even the chassis-bloat candidates. The opportunity is in *volume management* and *first-contact framing*, not in cuts.

---

## What this teaches us about the next move

The on-ramp gap (Angle 1's main finding: the high concept hasn't been confirmed for any non-designer player) is **concentrated in three places** based on this audit:

1. **The Skill System / Drafting boundary** — players draft before they understand combos.
2. **Combat's chess vocabulary** — Standard Attack and Bodyguard speak in chassis terms.
3. **Health & Armor volume** — bookkeeping competes with combo attention.

Of these, **Drafting on-ramp** is the lowest-cost lever (no rule changes) and the highest-information one (it's where the player's first game-relevant decision happens). If we want to test whether *the design itself* lands the high concept versus *teaching helps it land*, the drafting on-ramp is where to experiment first.

This is **not yet a recommendation to change anything.** It's a map: here's where the chassis is loudest, here's where the engine is quietest, here's where the gap lives. The next step (Angle 4: Chassis Minimisation, or a separate on-ramp ADR) is where we'd actually decide what to do.

---

## Open questions raised by this audit

- **Q1**: Is Bodyguard's strategic value coming from combo-relevant decisions (Guard-positioning enabling skill setups) or chess-relevant decisions (defensive screening)? If chess-relevant, is that a problem for the high concept?
- **Q2**: Would a 2-cap Armor (instead of 3) reduce bookkeeping volume without breaking the Armor vs. Armor-Breaker RPS loop?
- **Q3**: Could a "starter loadout" for first-time players be a better way to deliver the high concept on round 1 than asking them to draft? What would that loadout look like?
- **Q4**: Could the Standard Attack be reframed in the rule text — perhaps as its own section rather than a sub-clause of Movement — to plant skill-first thinking before chassis-first thinking?
- **Q5**: Does the Injured state's teaching cost exceed its first-game relevance? Should it be hidden from first-game rules?
- **Q6**: Is the skill catalogue's *combo-shape legibility* a property of the cards themselves, or only of the players who've seen combos before? Could the cards be reshaped to communicate combo-affinity (without changing rules)?

These belong in a follow-up discussion, not in this audit.
