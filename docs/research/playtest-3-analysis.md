# Playtest 3 Analysis

**Date**: 2026-05-17
**Players**: Elias (P1) vs Mario (P2). Mario is a first-time player.
**Variant tested**: L2G1 — Stack A, Game 1 only (Standard Attack Nerf, 1 DMG instead of 2). No combo bonus.
**Game length**: 24 rounds (Mario gave up). Rule explanation ~14:30–15:00; game start 15:35; game end 18:05; feedback complete 18:28. **~2h30 of game time.**
**Source**: `playtest-results/elias-vs-mario-17_05_26/transcripts-DRAFT.md` (corrected by user).

This document deliberately stops at *insights and issues*. Solutions / next-stack discussion follow in a subsequent session per the user's directive.

---

## 1. Tracking Data

### Block A — Elias (P1, winner)

**Rune economy**
- Starting Runes: 6
- First round a skill was used: Round 1 (Rust Shield + Armorsmith)
- Rounds where end-of-turn unspent Runes exceeded 6: none (peak end-Rune = 6, on R7 and R22)
- Rounds where all Runes were spent: R10, R11, R14, R15, R16, R17, R20, R21 (8 rounds). Plus R18, R23, R24 ending at 0.
- Largest single-turn Rune spend: 9 Runes — R23 (3× Rune Theft) and R24 (3× Rune Theft + move)

**Captures & key events** (from log)
| Round | Event |
|-------|-------|
| R10 | First Guard kill — Hook Pull on the Guard he had healed |
| R11 | First Champion kill — Attack + Focus + Hook + Hook ("good!!") |
| R12 | Killed another Guard on right flank (Attack + Hook) |
| R16 | Killed a Guard (zigzag attack into Blade Call + Armor Breaker) |
| R18 | Killed a Champion (push into Hook Pull range) |
| R19 | Killed a Guard (attack + Rune Theft) |
| R23 | Killed last Strike Champion (3× Rune Theft) |
| R24 | Killed last Champion (3× Rune Theft + move) — **Mario gave up** |

- Final round played: 24
- Total captures made (self-reported): 9
- Total captures suffered (self-reported): 6
- Post-game annotations: enjoyment 4/5; "had to explain a lot of rules at the start which moved the focus from finding cool shit"

### Block A — Mario (P2)

**Rune economy**
- Starting Runes: 6
- First round a skill was used: Round 1 (Armorsmith on King)
- Rounds where unspent Runes exceeded 6: none (peak end-Rune = 6, R6/R7/R8/R9/R13)
- Rounds where all Runes were spent: R12, R17, R18 (–1 from theft), R19, R20, R21, R22, R23 (8 rounds at 0)
- Largest single-turn Rune spend: 6 Runes (multiple rounds — 2× Armorsmith / 2× Strike / Quick Dash + Armorsmith)

**Captures & key events** (from log + Elias's 9/6 score)
| Round | Event |
|-------|-------|
| R1–R5 | Stacked Armor on King (4× across 5 rounds) |
| R14 | 2× Rune Theft — first offensive burst |
| R16 | 1 standard attack on a Guard |
| R17 | 1 Strike skill (likely Lance Thrust) |
| R18 | Lost 1 Rune to Elias's Rune Theft |
| R20 | Field Medic + Armorsmith (only heal of the game) |
| R21 | 2× Strike — second offensive burst |
| R24 | **Gave up.** No skills logged. |

- Final round played: 24 (forfeit)
- Total captures made: not self-reported; **inferred ~6** (mirror of Elias's "captures suffered: 6")
- Total captures suffered: **inferred ~9** (mirror of Elias)
- Post-game annotations: enjoyment 4/5; "wenig genutzt" (Bodyguard rarely used by him)

---

## 2. Behavioral Patterns

### Block B — Elias

**Skill usage frequency**
| Skill | Times used | Rounds | Typical context |
|-------|-----------|--------|-----------------|
| Armorsmith | ~5 | R1, R4, R5, R8, R13 (~) | Building Guard presence early/mid |
| Rust Shield | ~2 | R1, R2, R7 | Defensive setup on flanker |
| Hook Pull | ~5 | R9, R10, R11 (×2), R12, R18 | Finishers on Injured pieces; pulling Champions into kill range |
| Focus Strike | ~3 | R9, R11, R13, R21 (×2) | Range/damage extender for kill turns |
| Lance Thrust | ~2 | R17 (×2) | Following double-move into stand-off Guard |
| Air Blast | 1 | R15 | Push to set up subsequent kill |
| Field Medic | 1 | R15 | Heal own injured Guard |
| Blade Call | 1 | R16 | Combined with Armor Breaker for kill |
| Armor Breaker | ~6 | R14 (×3), R15, R16, R19, R20, R21 (×2) | High-frequency mid/late, removing Mario's Armor stacks |
| Quick Dash | 1 | R20 | Reposition |
| Rune Theft | ~6 | R19, R20, R23 (×3), R24 (×3) | Late-game burst tool — last-Champion finisher (because i did not have those pieces with the skill positioned to do smth prior (i also had no chance to as i was "forced" to keep putting on all the preassure/action the right flank where i did not have this skill (instead having hook pull which i then used instead))) |

- Most-used skills: **Armor Breaker** (~6) and **Rune Theft** (~6)
- Skills used 3+ times: Hook Pull, Focus Strike, Armor Breaker, Rune Theft, Armorsmith
- Never used: Shadow Shift, Blade Tempest (not drafted, but the same goes for precision blast and retreat plan tho)

**Attack vs. skill balance**
- Standard attacks logged in events: ~5 (R11, R12, R18 ×2, R19) — likely under-counted (form didn't ask)
- Skill activations logged: ~28+
- **Attack-to-skill ratio is very low** (~1:5). The nerf made attacks the *setup* for skill kills, not the killer themselves.

**Combo attempts** (Layer 2 — Game 1 only, no combo bonus active)
- Multi-Champion combos attempted: not formally tracked (combo bonus inactive). However, Elias used multi-skill *single-target* sequences from one Champion in single turns (e.g., R11: Attack + Focus + Hook + Hook). These are single-Champion bursts, not multi-Champion.
- it is to be noted that i once pushed a target (air blast) into the range of another piece (after attacking it with movement) so that i could then finish it off - so i did a combo, just not a strike skill one.
  - could be worth exploring to maybe try and encourage such combos as well?

**Positioning and movement patterns**
- Rounds before first forward movement: R3 ("advanced right flank")
  - this is misleading as before that like in a chess game i just "opened up my position to then able such flanking initiatives"
- First contact (first DMG to opponent piece): around R9–R10 (first Guard kill R10)
- Evidence of standoff: **No.** Q4 answer: "much less standoff." Engagement felt earlier and more continuous.
- Guard behavior: Used as advancing front line on right flank. Armorsmith stacked early — Guards screened, didn't die en masse.
- King: only moved at the very end "because he needed it for a kill."
  - i say this is also down to strategy, but having a "i must protect this piece" in the middle of course makes players just draw attention to the flanks (because you have no "queen in the middle" to reward your risk (**this does not mean i want a queen piece**))

**Armor usage**
- Total armor granted: not tracked (Elias: "→ I don't want to count it")
- Qualitative comment Q17: "start: basically only armor stacking, afterwards it was used to prevent 'instant death' + it gave presence." Slight extension / well balanced.

### Block B — Mario

**Skill usage frequency**
| Skill | Times used | Rounds | Typical context |
|-------|-----------|--------|-----------------|
| Armorsmith | ~20 | R1, R2, R3, R5, R7, R8, R10 (×2), R11 (×2), R12 (×2), R15 (×2), R19 (×2), R20, R22, R23 (×2) | Default action almost every turn - also because he had no other real good options because he drafted poorly (because he had no game knowledge) but also because he did not want me to overly gobble up his unprotected pieces (i guess he did not really have any "presence pieces"?) |
| Field Medic | 1 | R20 | One heal in 24 rounds |
| Rune Theft | 2 | R14 (×2) | First offensive burst |
| Strike (Lance Thrust?) | 1 | R17 | Single Strike use |
| Strike (×2) | 2 | R21 | Second offensive burst |
| Quick Dash | 1 | R22 | Reposition late |
| Standard attack | ≥1 | R16 ("1 Angriff Guard") | Form did not prompt — likely undercounted |

- Most-used skill: **Armorsmith (~20 uses)** — by an enormous margin
- Skills used 3+ times: only Armorsmith
- Never used / unclear: Hook Pull, Air Blast, Blade Tempest, Blade Call, Armor Breaker, Focus Strike, Rust Shield, Shadow Shift (depends on his draft, but no log entries)
  - same as above: this this matter tho?
  -  also: he picked both focus and blade call and also used focus here and there

**Attack vs. skill balance**
- Standard attacks logged: ≥1 confirmed (R16). Form did not prompt for them, so true count unknown but believed > 1 given Elias's 6 captures suffered.
  - can confirm: it was used a lot by him because pieces were almost always close together hence it would have been stupid not to attack
- Skill activations logged: ~28 (≥20 of which were Armorsmith)
- **Skill mix is heavily skewed defensive.** Offensive output = ~3 Strike skills + 2 Rune Thefts + at least 1 standard attack across 24 rounds.

**Combo attempts**
- None logged.

**Positioning and movement patterns**
- Rounds before first forward movement: unclear (no movement notes). Likely later — King "never moved meaningfully" per side-notes.
  - i would say same as elias
- First contact: Mario was reacting, not initiating. First time he visibly engaged offensively was R14 (Rune Thefts).
- Evidence of standoff: He answered Q4 with no option marked / "?". He did not play P2, so the comparative question doesn't apply, but his behavior was passive — he was being approached, not approaching.
  - i would say this again is because he did not have any "game confidence" (which comes from game knowledge/understanding)
- Guard behavior: Armor-stacked screens, slow to commit forward.
- King: Side-notes #6 — "Mario: never. King was a 'mystic skill slave'." King sat in back row purely as a skill-activation hub.

**Armor usage**
- Approx. 20 Armorsmith activations × +1 Armor each = up to ~20 armor points granted across the game.
- This is a huge defensive investment that absorbed substantial damage but did not translate to a winning position.
  - because i also had a counter in armor breaker (which he also verbally complained about me "eating up his armor")
  - i just countered his strategy well (he had 2 armor smiths, no rust shields and i had armor breaker and used it well and often)

### Cross-Player Synthesis

1. **Hyper-asymmetric skill mix**: Elias used a balanced suite of 8+ skills; Mario used essentially one skill (Armorsmith) until R14. This is a first-time-player effect, but it's also a *signal that defensive stacking is the easy default* with no in-game pressure to break out of it.
2. **The nerf created a skill-first combat texture**: Elias's attack:skill ratio is ~1:5. Standard attacks became *setups* (apply Injured) and skills became *finishers*. This matches the OQ-37 / OQ-38 design intent — skills now feel worth their Rune cost.
3. **Both players' Rune economy converged on the same shape**: peaks of 6 Runes mid-game, frequent 0-Rune turns late. Neither hoarded. Both said "Always wanted more." The +2/turn economy held under heavier skill usage.
4. **Engagement timing was much faster than P2**: First Guard kill R10, first Champion kill R11. Compared to Playtest 2's first Champion kill at R26, this is a ~15-round acceleration.
5. **Asymmetric data quality**: Mario didn't fill captures, didn't track standard attacks, gave many blank/?'s. This is partly skill-level (first game), partly form design. Elias's data is rich; Mario's is sparse. (this also came from mario not being well versed in english + having other things occupy his mind (open todo's which is understandable))

---

## 3. Raw Transcriptions

Locked in `playtest-results/elias-vs-mario-17_05_26/transcripts-DRAFT.md` (Sections 2–5). Treated as ground truth for this analysis. Highlights re-quoted as needed below.

---

## 4. Key Findings

### 4A. Confirmed positives

1. **The standard-attack nerf works as designed (behaviorally observed)**.
   - First Champion kill at R11 vs P2's R26.
   - Elias: combat feel "Better / Much Better".
   - Both: "Felt right" on Q2 (1 DMG attacks not frustrating).
   - Skill use density is high — Elias fired ~28+ skills, Mario ~28 (mostly defensive).
   - No standoff: Elias Q4 "Much less standoff"; Q5 "Not reluctant — I knew I would not die immediately if I did". *(Self-reported AND behaviorally observed: first contact moved from ~R7+stalemate in P2 to active engagement by R10.)*

2. **Bodyguard activated organically (self-reported and behaviorally observed)**.
   - Elias Q15: "with less standoff it (body guard rule) happened way more often" + repositioned **Yes**.
   - This is the first playtest where Bodyguard was a live mechanic during play. **OQ-21's hypothesis (need an adjacency fix) may already be partially solved by the standoff fix.**

3. **Rune economy held under heavier skill load (behaviorally observed)**.
   - Both players reached 0 Runes regularly without it feeling oppressive.
   - Both Q18: "Always wanted more" — G1 (shortfall never closes) preserved.
   - Layer 1 economy + Layer 2 nerf is balanced — confirms not over-tuned.

4. **Armor felt well-balanced from Elias's side (self-reported)**.
   - Q17: "Slightly extended / well balanced. Start: basically only armor stacking, afterwards it was used to prevent 'instant death' + it gave presence."
   - Cap of 3 not flagged. (OQ-11 partial confirmation.)

5. **Turn flow was intuitive (self-reported)**.
   - Both Q12: "Very intuitive". No changes needed to Move/Skill Slot structure.

6. **A non-Strike multi-piece combo emerged organically (behaviorally observed, your note)**.
   - Elias used Air Blast on one piece to push a target into Hook Pull range of *another* of his pieces, which then finished the kill.
   - This is a **multi-piece combo without the combo bonus being active**, and it's *not* a Strike+Strike combo — it's a Move-skill setup chained into a Strike-skill payoff across two different Champions.
   - Confirms that the design fantasy of "discovering and executing clever spell/skill combos" is being executed naturally even at L2G1, **without** any explicit combo-incentive layer.
   - **Open design question** (your note): worth exploring whether to encourage non-Strike-only combos explicitly — e.g., does the eventual combo bonus reward only Strike+Strike or any multi-piece sequence on a target?

### 4B. Confirmed problems / unhappinesses

1. **R22 was a fully wasted turn due to a rule ambiguity (behaviorally observed)**.
   - End-of-turn Runes = 6, no skills fired. Elias's note: "Injured effect on Lance Thrust unclear".
   - The ambiguity (does Range-2+ Injured penalty apply to a Range-1 skill?) caused the *winning* player to pause mid-game, decide it was too risky to act, and burn a full turn. This is more impactful than the side-note framing suggests — it's a concrete data point that **rule clarity issues cost real game time at high-skill play**.
   - Side-notes #4 and #5 already flag the underlying rule. This is a clarity-bug in the rule sheet, not a balance issue.

2. **Mario's skill mix was extreme (~80% Armorsmith), but the cause is more nuanced than "turtling-as-default" (behaviorally observed, with caveats from your edits)**.
   - Across 24 rounds, ~20 Armorsmith activations vs ~3 Strikes + 2 Rune Thefts logged.
   - **However, his standard-attack count is substantially higher than the form shows** — your note: "pieces were almost always close together hence it would have been stupid not to attack." So Mario *was* attacking; he just wasn't logging it (form gap, see point 3).
   - The Armorsmith spam itself has multiple causes:
     - **Draft weakness**: Mario picked 2× Armorsmith and no Rust Shield. With limited shield variety in his loadout, Armorsmith was his only defensive lever.
     - **No "presence pieces"**: he didn't have skills that project threat onto the board, so unprotected pieces would have been picked off → stacking Armor was a rational reaction to Elias's pressure, not pure passivity.
     - **No game confidence**: he didn't know which combos were viable yet, so he leaned on the one action whose outcome he could predict.
   - **Elias drafted and executed the counter** (Armor Breaker, ~6 uses, with Mario verbally complaining about it) — the rock-paper-scissors loop *is functioning*. Armor was not "free defense"; it was countered.
   - **Reframed signal**: this is not "turtling has no in-game cost." Turtling *was* costed (Armor Breaker hard-countered it, Mario lost). The signal is closer to: "a new player with a poor draft defaults to the most legible action, and the legible action happens to be Armorsmith." That's a draft-experience / skill-catalogue / on-ramp issue, not a turtling-balance issue.
   - King: never moved meaningfully. See point 4 for the structural reading.

3. **The form does not capture standard attacks (process issue)**.
   - Mario logged exactly one ("R16 — 1 Angriff Guard") because it happened to feel notable. The actual count is unknown but >1 (Elias's 6 captures suffered ≈ Mario's 6 attacks landed implies he was hitting things).
   - Without standard-attack counts, we cannot answer questions like "did Mario attack at all" or "what's the actual attack:skill ratio?"
   - **This is a tracking-form design bug**, not a game-design bug. Affects all future analyses.

4. **The King is mechanically inert — and there's a structural reason (cross-player observation, your framing)**.
   - Mario: never moved his King meaningfully.
   - Elias: only moved his King at the very end and only because forced.
   - Side-notes #6: "No mechanic incentivises using all pieces actively — King especially."
   - **Your structural read**: there's nothing in the centre of the board worth fighting over. Players drift to the flanks because flanks offer flanking initiatives without a central attractor. A "queen in the middle"-style central reward piece would change this — but you explicitly said **you do not want a queen piece**. The point is just that the *absence* of a central pull explains the King's static role.
   - **Reframed signal**: King-as-skill-slave is not just an incentive gap on the King itself. It's a board-geometry / central-tension gap. The fix space is "make the centre matter," not "make Kings move."

5. **Rule explanation ate ~30 minutes of session time and bled into game enjoyment (self-reported)**.
   - Elias overall enjoyment 4/5 with the explicit comment: "had to explain a lot of the rules at the start which moved the focus from finding cool shit."
   - Mario Q11 (Skill Drafting): "no plan" — possibly meaning "I don't know what this means / I forgot."
   - Side-note #1: tracking sheet should bake in Rune gain + Skill Slot scaling so it doesn't need to be taught from scratch.
   - **Teaching friction is now a real cost**, not just an annoyance.

6. **Game length straddles "just right / a bit long"**.
   - Elias: 3/4 split. Mario: 4 (a bit long). 24 rounds, ~2h30 game time.
   - This is *significantly better* than P2 (~4h, ~R30+) but still not in "comfortable" territory for two-game sessions.
   - Compounding factor: rule explanation overhead means total session time is even higher than game time alone.

### 4C. Soft flags / deferred

1. **Lance Thrust + Injured range penalty rule** — needs explicit baseline clarification (side-note #5). Not a balance issue, just rule-text.
2. **Focus Strike + adjacent-self interaction** — needs explicit clarification: cannot magically self-target adjacent skills like Field Medic, but CAN extend a Range-1 skill to Range 2 (side-note #4).
3. **"Skill cost" column in tracking sheet is unnecessary** (side-note #2).
4. **New idea: replace "10 turns no kill = draw" with "push pieces 1 toward centre"** — side-note #3, parked as backpocket idea.
5. **Mario didn't fill ratings for Q4 (standoff vs P2)** — he hadn't played P2. Form should make this kind of question conditional / opt-out clearly.

### 4D. Open questions raised

1. Did the multi-Champion combo bonus (Game 2) need to be tested in this session? (It wasn't — only one game played, ~2h30 was the budget.) Future: do we still need a Game 2 with combo bonus, or does the L2G1 result alone close the question?
2. **Should the combo bonus reward Move-into-Strike or other non-Strike-only combos?** Elias's Air Blast → Hook Pull cross-piece sequence was a real combo *without* the bonus active. If Game 2's combo bonus only activates on Strike+Strike, this kind of clever play gets no extra reward.
3. **Is the absence of a central attractor on the board a design problem?** The King-as-skill-slave pattern + flank-only play seem to share the same root: nothing in the centre is worth fighting over. Not solving this layer; flagging the question.
4. Mario's offensive output is hard to read because of form gaps — but cross-referencing with your note ("pieces always close, would have been stupid not to attack") and Elias's 9 captures suggests he was attacking frequently, just not logging it. Form fix needed before next playtest.

---

## 5. Answers to Test Scenario Questions (Stack A Game 1)

Drawn from Elias's feedback form. Mario's form is sparse — note: this is partly a **language-barrier effect** (English forms, German speaker) plus open todos in his head, not disengagement with the game itself. His blank fields should not be read as "this didn't matter to me".

| # | Question | Answer | Source |
|---|----------|--------|--------|
| Q1 | Injured pieces had meaningful time before dying | Elias: Sometimes ("not injured for very long here / fewer injuries that meant something"). Mario: Rarely. | Both forms |
| Q2 | Standard Attacks only Injure — frustrating? | Both: **Felt right** | Both |
| Q3 | Used Strike skills to finish off Injured pieces | Elias: Often (Hook Pull, Rune Theft). Mario: Sometimes. | Both |
| Q4 | Standoff vs Playtest 2 | Elias: **Much less standoff**. Mario: blank ("?", didn't play P2). | Elias only |
| Q5 | Reluctant to move forward | Both: **Not reluctant**. Elias: "I knew I would not die immediately if I did." | Both |
| Q11 | Skill Drafting | Elias: "optimal strategy will come from experience, otherwise it felt good." Mario: "no plan." | Both |
| Q12 | Turn flow intuitive | Both: **Very intuitive** | Both |
| Q13 | Skill balance / best combo | Elias: best combo "attack + Blade Call + Armor Breaker to kill"; "all skills felt balanced this game → rune theft okay because rarely runes left". Mario: "Quick Dash useful only with Strikes". | Both |
| Q14 | Must-pick skills | Elias: **Yes** — Blade Call, Focus Strike, heal, shield, Rune Theft. Mario: **No**. | Both (disagree) |
| Q15 | Bodyguard | Elias: triggered "?" times, **repositioned Yes**, "with less standoff it happened way more often". Mario: "wenig genutzt" (rarely used by him). | Both |
| Q16 | Injured Champions feel weaker | Elias: blank, "did not notice it - both in a good way or bad way". Mario: **Not different at all**. | Both |
| Q17 | Armor | Elias: **Slightly extended / well balanced**. Mario: **slowed game noticably**. | Both (disagree) |
| Q18 | Rune spending | Elias: **Always wanted more / balanced**. Mario: **Always wanted more**. | Both |
| Q19 | Rune economy with 1-DMG attacks | Elias: **Somewhat more spending**. Mario: **About the same** (no comparison point). | Both |
| Q20 | Favorite moment | Elias: "much more active game with stuff happening". Mario: blank. | Elias only |
| Q21 | Most confusing / frustrating | Elias: "Focus Strike + Lance Thrust" rule ambiguity. Mario: blank. | Elias only |
| Game length rating | | Elias: 3/4 (just right ↔ a bit long). Mario: 4 (a bit long). | Both |
| Combat feel vs P2 | | Elias: **Better / Much Better**. Mario: n/a. | Elias only |
| Overall enjoyment | | Both: **4/5** | Both |

---

## 6. OQ Evaluations

Going through every OQ marked TRACKING / with evaluation criteria for Layer 2 (L2G1).

### OQ-10: Injured Penalty Severity

**Hypothesis**: Range 2+ → Range 1 may be too soft for Champions/King (they're already Speed 1, so the speed cap does nothing).
**Evidence**:
- Elias Q16: blank, "did not notice it — both in a good way or bad way".
- Mario Q16: "Not different at all".
- R22 wasted turn was *because* the Injured penalty rule is unclear, not because it's too soft.
**Verdict**: **Inconclusive**. Players didn't notice Injured Champions feeling weaker, but the rule ambiguity prevents clean evaluation.
**Recommended action**: Resolve the rule clarity (side-note #5) first. Re-test in next layer.

### OQ-11: Armor Cap

**Hypothesis**: Cap at 3 may be too high if armor extends games.
**Evidence**:
- Elias Q17: slightly extended / well balanced. Comment: armor used early to stack, later to prevent insta-death.
- Mario: "slowed game noticably."
- Mario stacked ~20 Armorsmith uses; Elias countered with ~6 Armor Breaker uses. **The RPS loop functioned** — Elias drafted the counter, used it well, Mario verbally complained about it.
- Armor amounts were not formally counted (Elias Q-A: "→ ich will nicht zählen").
**Verdict**: **Confirmed working as designed at cap 3**. The "slowed game" complaint is real but pairs with a working counter — the game wasn't slow because armor is unbreakable, it was slow because both players invested heavily in the armor/anti-armor loop.
**Recommended action**: Keep cap at 3. Add formal armor tracking to next tracking sheet (count granted + count broken) so the loop's tempo cost can be measured. No design change yet.

### OQ-21: Bodyguard Rule

**Hypothesis (Layer 3)**: Adjacent-to-defender-only would make Bodyguard trigger more often.
**Evidence**:
- **Bodyguard already triggered "way more often" in this playtest under the baseline adjacency rule** because the standoff dissolved.
- Elias Q15: triggered "?" times, **repositioned Yes**.
- Mario: "wenig genutzt" — but per your reframing, this likely reflects Mario's lack of game confidence / knowledge (he didn't know to engineer Bodyguard situations) rather than a structural Bodyguard problem.
**Verdict**: **Significantly updated**. Standoff fix (Stack A) appears to indirectly solve the Bodyguard-dead problem by closing engagement distance. Mario's low engagement does not contradict this — it's draft/knowledge limited, not rule-limited.
**Recommended action**: **Reconsider whether Stack B (Bodyguard adjacency fix) is still high-priority.** It may be solved-by-side-effect. Confirm in another playtest with two engaged players before fully de-prioritising.

### OQ-34: Rune Theft Balance

**Hypothesis**: Cost 3 may be too cheap; consider raising to 4.
**Evidence**:
- Elias used Rune Theft ~6 times, including 3× back-to-back on R23 and R24 to finish the game.
- **Per your edit**, the late-game Rune Theft burst was *forced by positioning*, not saved tempo — Elias's right-flank pressure didn't have Rune-Theft-equipped pieces in position, so he used Hook Pull there instead and only fired Rune Theft once the situation allowed it. So R23/R24's 3× Rune Theft burst is NOT evidence of Rune Theft being a "saved game-ending tempo weapon".
- Elias Q13: "all skills felt balanced this game → rune theft okay because rarely runes left."
- Q14 must-pick: lists Rune Theft as a must-pick.
- Mario used Rune Theft only twice (R14 burst).

**Reframing (designer's read after Playtest 3)**: Rune Theft is **state-dependent**, not flatly over- or under-tuned. It has two modes:
- **Mode A — opponent at 0 Runes**: behaves as a normal Strike skill (1 DMG, default range). Lance-Thrust-equivalent.
- **Mode B — opponent has Runes**: cheap damage + opponent-disable. The disable value is *time-dependent*:
  - **Early game**: stealing 1 Rune blocks a planned skill (~50% of a turn's gain). High impact.
  - **Late game**: both sides can roll major-combo Runes every ~2 turns, so −1 Rune barely dents the opponent. Low impact.
- This explains why Elias felt it was "balanced this game" (Mario was usually at low Runes → Mode A all game) AND why Q14 marks it must-pick (Mode B is genuinely strong when it triggers).

**Verdict**: **Inconclusive on the flat cost question, but the right design question has shifted**. The lever to evaluate is no longer "is 3 Runes too cheap" but "is early-game Mode B disproportionately punishing relative to a 3-Rune cost?"
**Recommended action**: Continue monitoring. Don't nerf yet. In the next playtest with two experienced players, watch specifically: (a) does early-game Rune Theft block opener plans, (b) does late-game Rune Theft feel pointless or still tempo-relevant. If (a) is yes, the cost question reopens. If (b) is "pointless", Rune Theft might need a redesign rather than a cost tweak (e.g., scaling effect, or theft amount tied to opponent pool).

### OQ-37: Standard Attack Damage (1 DMG)

**Hypothesis**: Nerfing standard attacks to 1 DMG makes skills worth their Rune cost and dissolves the standoff.
**Evidence**:
- First Guard kill R10, first Champion kill R11 (vs P2's R26).
- Both Q2: "Felt right".
- Elias Q5: "I knew I would not die immediately if I did."
- Combat feel: Better / Much Better.
- Skill activation rate ~28/player/24 rounds.
**Verdict**: **Confirmed.**
**Recommended action**: **Accept the nerf into baseline** for all future stacks. Update `ruleset-baseline.typ` accordingly when next stack is drafted.

### OQ-40: Standoff / No-Man's-Land

**Hypothesis**: Attack nerf reduces entry risk → standoff dissolves.
**Evidence**:
- Elias Q4: "Much less standoff."
- First contact ~R10 vs P2's standoff at R14+.
- Q5 both "Not reluctant".
**Verdict**: **Confirmed.**
**Recommended action**: Standoff was the leading symptom of the combat-balance problem. With the nerf, it's gone for now. Keep monitoring in future playtests with two experienced players (Mario was first-time, so the standoff might re-emerge with two cautious veterans).

### OQ-41: Game Length vs. Damage Nerf Tradeoff

**Hypothesis**: 1-DMG attacks may extend the game by making Guards harder to remove.
**Evidence**:
- Game ran 24 rounds, ~2h30. Earlier first kills than P2 but more total kills (9 vs ~1 in P2).
- Length ratings: Elias 3/4, Mario 4. Both lean toward "a bit long".
- Mario's heavy Armor stacking + Elias's Armor Breaker counter created a meta-game that consumed turns.
**Verdict**: **Partially confirmed (game still slightly long, but better than P2).**
**Recommended action**: Don't fix this in isolation yet. The asymmetric Mario-defense pattern is a confounder. Re-test with two engaged players. If still long → Stack C (pacing) or Stack D (smaller board / fewer pieces) becomes priority.

### OQ-46: Rune Cap

**Hypothesis**: Players may hoard if cap absent. Test cap at 8 if hoarding observed.
**Evidence**:
- Peak end-of-turn Runes for either player: 6 (well below any proposed cap).
- Both: "Always wanted more."
- Both spent down to 0 frequently.
**Verdict**: **Hoarding NOT observed.**
**Recommended action**: No cap needed. Close OQ-46 from monitoring (or downgrade priority) — current economy + spending pressure naturally prevents hoarding.

---

## 7. Implications for Game Feel and Issues / Unhappinesses

*Per user directive: insights only. Solutions discussed in subsequent session.*

### Game feel — what changed for the better
- The game now *feels* like skills matter. Standard attacks became setup tools, skills became finishers. This is the design fantasy (clever spell combos) starting to express itself mechanically.
- **Cross-piece combos emerged organically without a combo-bonus layer** — Elias's Air Blast (push) → Hook Pull (kill) on a different Champion shows the system already supports the design fantasy at L2G1. This is a positive signal *before* Game 2 even ran.
- Engagement happens early (~R10) rather than late (~R20+). Combat texture is denser.
- Bodyguard is alive without a rule change. The standoff fix is doing double duty.
- Rune economy holds — players spend, want more, feel constrained but not starved.
- **The Armor / Armor-Breaker RPS loop functioned** — Elias drafted the counter and used it well, validating that the skill catalogue contains real strategic responses to defensive stacking.

### Game feel — what's still not quite right
- Rule clarity issues are now a measurable cost (R22 wasted turn). This wasn't a problem in earlier playtests because skills were used less.
- The game still runs ~2h30 with rule explanation. Rule explanation is biting into the experience explicitly (Elias's enjoyment comment).
- **King has no role in active play, and the cause is structural — there's no central attractor on the board.** Both players gravitated to flanks because the centre offers no reward worth contesting. (Not solving in this layer; flagging the design space.)
- **Mario's monotonous strategy was draft + experience-driven, not a "turtling-too-good" signal.** Counter existed and worked. But: a new player's first game converging to "Armorsmith every turn" is still a teaching/onramp problem worth flagging.

### Unhappinesses surfaced
1. R22-style rule-ambiguity stalls (clarity issue).
2. **First-time player onramp**: Mario drafted poorly and defaulted to one skill because it was the most legible. The game's complexity is hard to parse without prior exposure. (Not a balance issue.)
3. Tracking sheet doesn't ask about standard attacks (form issue) — Mario's true attack count is hidden.
4. Tracking sheet doesn't bake in Rune gain / Skill Slot scaling (teaching friction).
5. Game-length perception: "a bit long" from Mario, on the edge from Elias.
6. **Centre of the board has no attractor** — see deep-dive below. Real issue.
7. **King is not a strategic target; the game is attrition-driven by default** — see deep-dive below. Real issue.

### Deep-dive issue #1: centre-cramp / no central attractor

**Observation**: Both players naturally drifted to the flanks at the start; the centre stayed empty; the King never moved.

**Cause** (designer's read, after Playtest 3):

1. The starting formation is cramped in the centre 6 columns of a 10-wide board. Opening play is therefore about *getting out of the centre* to claim more of your half — which means moving to the flanks.
2. The opponent's pieces are *also* clustered in the middle, so engaging the centre means engaging their strongest concentration of force first. That's the worst place to commit early.
3. The King is in the centre but is functionally just-another-Champion (a skill carrier). "Keep this safe" therefore means "keep it away from action" → away from the centre.

All three forces push outward at the same time. Action drifts back toward the centre or across to the other flank only later, once positions are committed and threats clarify.

**Why this matters**: the centre is doubly anti-attractive in the opening (cramped on entry + dense with enemy pieces on the way through). There's nothing in the centre that *rewards* being there, so players go around. This shrinks the strategic space of the opening.

**Status**: Real found issue. Not solving here; flagged for design discussion. New OQ proposed (see Section 12).

### Deep-dive issue #2: attrition vs. regicide — the King isn't a real target

**Observation**: Mario gave up when his last Champion died — not when his King was actually threatened. Elias never went for Mario's King; he ground out the army. The King was a passive skill-activation hub all game.

**Cause** (designer's read): the *formal* win condition is King capture, but the *real* victory path is attrition — grind down Champions and Guards until the opponent has no offensive output left. King capture follows automatically once the army is gone. So the strategic texture of the game is "manage your army," not "threaten the King."

This is partly behavioral (both players took the easier attrition path because regicide is risky), and partly structural:
- The King has no pull toward the centre (issue #1 above).
- The King is just-another-piece — it doesn't broadcast threat or invite contestation.
- There's no in-game reward for putting your King forward; only risk.

**Designer's intent (your call this turn)**: you want the King to be a real threatenable target. Right now it isn't.

**Important nuance**: King Lifetime HP (currently in backpocket) would NOT fix this on its own. Adding HP to a piece that never enters the action just makes a static piece harder to kill. The actual problem is **getting the King to participate** — only then does the game enter "the part that gets real fun."

**Status**: Real found issue. Not solving here; flagged for design discussion. New OQ proposed (see Section 12). Connected to issue #1 — both share the root cause "centre is mechanically inert + King has no incentive to move."

### Things specifically validated as no-longer-broken
- Standoff / no-man's-land → dissolved.
- Bodyguard dead → triggered organically.
- Skills not worth their cost → no longer true; skill density is high.
- Hoarding → not observed.
- **Defensive stacking has no counter → false; Armor Breaker hard-counters it and was used effectively.**
- **Combo fantasy requires the combo-bonus layer to manifest → false; cross-piece combos already happen at L2G1 organically.**

---

## 8. Comparison to Previous Playtests

| Metric | P1 (31.10.25, Elias vs Pasco) | P2 (24.04.26, Elias vs Jonathan) | **P3 (17.05.26, Elias vs Mario, L2G1)** |
|---|---|---|---|
| Layer tested | baseline | Layer 1 (Rune economy) | **Layer 2 Game 1 (1-DMG nerf)** |
| First Guard kill | early (~R5–R7) | early (~R10) | **R10** |
| First Champion kill | n/a (game didn't reach) | **R26** | **R11** |
| Total Champion kills | ~0 | 1 | **~3 (Elias's count = 9 captures across all pieces)** |
| Standoff observed | yes | yes ("two guards like pawns" at R14) | **no — "much less standoff"** |
| Bodyguard triggers | ~0 | 2 | **"way more often" (organic, count not formally tracked)** |
| Rune hoarding | yes (slow gain) | no | **no** |
| Game length | "way too long" (didn't finish) | ~4h, ~R30+ | **~2h30, R24 (forfeit)** |
| Combat feel rating | n/a | n/a | Better / Much Better (Elias) |
| Overall enjoyment | poor | mixed | **4/5 both** |
| New player? | yes - elias first time this format, pasco first time in general | yes - jonathan first time this format | **yes — Mario first-time** |

---

## 9. Decision Tree Routing

**Key metrics for routing:**
- First Champion kill: **R11** → before R15 (strong)
- Standoff observed: **No** — dissolved
- Bodyguard triggers: ≥several, organically (Elias Q15: "way more often" + repositioned Yes)
- Combo attempts (formal Strike+Strike): 0 — combo bonus inactive
- **Combo attempts (organic, cross-piece, Move-into-Strike): ≥1** — Air Blast → Hook Pull
- RPS loop (Armor vs Armor Breaker): functioning

**Per `TESTING_PLAN.typ` — Phase 1 entry conditions:**
- Stack A entry: combat balance / standoff / first-kill timing — **all green**.
- Stack B entry: Bodyguard dead — **may already be solved by side-effect**, contingent on confirmation with experienced players.
- Stack C entry: kill timing past R20 — **NOT triggered**.
- Stack D entry: board feel / piece count — not triggered, but Mario's "a bit long" is a soft signal.
- Stack F entry: combo ceiling unexplored — **arguable trigger** since combo bonus was never tested AND organic combos already showing without it.

**Decision tree path**: Stack A → Game 1 results clean → standoff & engagement healthy → Bodyguard triggered organically → cross-piece combos already emerging without bonus → next branch is **Stack A Game 2 (combo bonus on top of nerf)** to isolate whether the bonus *adds* to an already-functional combo system or just decorates it.

**Reasoning**: We tested only half of Stack A (the nerf). The combo bonus question (OQ-38) is still open. The Air Blast → Hook Pull observation reframes Game 2's purpose: it's not "does this enable combos?" — combos already exist — it's "does the bonus mechanic raise the ceiling of clever play, or does it create a Strike+Strike monoculture and *crowd out* the kind of cross-skill-category combo Elias just showed?" That's a more interesting question than the original Game 2 hypothesis.

**Recommendation (for the subsequent solution discussion, per your directive — not to be acted on yet):**
1. Plan Stack A Game 2 next, with two experienced players (so the data isn't draft-knowledge confounded).
2. **Re-examine whether Game 2's combo bonus should be Strike+Strike-only or any-skill-into-any-skill** — this is now an active design question, not a write-up question.
3. Before the next playtest, fix the form (add standard-attack count + bake Rune gain into the tracking sheet).
4. Resolve Lance Thrust + Injured rule ambiguity in the baseline rule sheet.
5. Defer Stack B — re-evaluate after one more experienced-player playtest.
6. Park "centre of board has no attractor" as a backpocket discussion (linked to King-inertness), not an active layer.

---

## 10. Triggered Backpocket Items

From `docs/backpocket.md` — items that this playtest's results activate or update:

- **Rune Theft cost nerf (G7 backpocket)**: NOT triggered. Per your edit, Elias's late Rune Theft burst was forced by positioning, not strategic save — the "dominant tempo weapon" framing weakens. Keep on watch list for a higher-Rune meta.
- **Skill catalogue expansion (OQ-12)**: Mario's draft converged to Armorsmith partly because of game inexperience, partly because his loadout had no presence pieces and no Rust Shield. This is a softer signal than "shield category lacks variety" — it's more "draft onramp for new players is rough." Keep gated on more data.
- **Sente skill design / Stack F**: Standoff dissolved naturally this game. Sente-property skills are still wanted long-term to keep standoff dissolved with two experienced players, but not urgent. **However**, the cross-piece Air Blast → Hook Pull combo shows there's already organic emergence of "setup → payoff" multi-piece play — Stack F should be designed to *reward* this kind of play, not just Strike+Strike sequences.
- **King Lifetime HP**: NOT triggered. Game length is acceptable.
- **King mobility / central attractor**: **NEW backpocket entry warranted** — both players said the King is static, and your structural reading ("no queen in the middle, no central reward, players drift to flanks") points at a board-geometry / central-tension design space. Justification: improves game-feel by giving the centre of the board strategic weight. Park as `[TO DISCUSS]`.
- **Cascade trigger / Pin / Threatened**: Not triggered. Wait for Stack A Game 2 data.
- **Combo bonus scope question**: **NEW open question** — should the combo bonus reward cross-skill-category sequences (Move-into-Strike, Mystic-into-Strike) or only Strike+Strike? Surfaced by Elias's Air Blast → Hook Pull. Resolve before drafting Stack A Game 2's final ruleset.
- **[TO DISCUSS] Guard buffs / Mid-game events / Private draft+trade**: Mario's Guard-screen + Armor pattern is still relevant input to the Guard-buff brainstorm. Worth flagging in the discussion phase.

---

## 11. Summary for User

Top findings (one sentence each):

1. **Standard attack nerf works** — first Champion kill R11 vs P2's R26; both players said combat felt right. *(Self-reported + behavioral.)*
2. **Bodyguard activates organically when standoff dissolves** — Elias "way more often, repositioned yes." This may obsolete Stack B. *(Self-reported + behavioral.)*
3. **Cross-piece combos already emerge organically without the combo-bonus layer** — Air Blast → Hook Pull on a different Champion. Reframes Stack A Game 2's purpose. *(Behavioral.)*
4. **The Armor / Armor-Breaker RPS loop functions** — Elias drafted the counter and used it well. Mario's Armor stacking was draft-and-experience-driven, not a "turtling-too-good" signal. *(Behavioral, with your reframing.)*
5. **R22 was a fully wasted turn** because of a Lance Thrust + Injured rule ambiguity — concrete data point that rule clarity costs game time. *(Behavioral, not in feedback form.)*
6. **The board has no central attractor** — King-inertness + flank-only play share the same root: nothing in the centre is worth fighting over (you explicitly noted you do *not* want a queen piece). *(Behavioral + your structural read.)*

OQ verdict changes:
- OQ-37 → **Confirmed / Accept into baseline.**
- OQ-40 → **Confirmed / standoff dissolved.**
- OQ-21 → **Significantly updated** — Bodyguard works without the adjacency fix. May deprioritise Stack B.
- OQ-11 → **Confirmed working** — RPS loop functions; cap 3 is fine.
- OQ-46 → No hoarding observed; close from monitoring.
- OQ-34 → Inconclusive (Rune Theft burst was forced-late, not strategic-late — weakens the dominance framing).
- OQ-10 → Inconclusive; rule clarity blocking clean evaluation.

Behavioral patterns not in any feedback question:
- **Mario's standard-attack count is hidden** by form gap; per your note, he was attacking frequently because pieces were close. Form needs a "standard attacks made" counter.
- **Cross-skill-category combo (Move → Strike) is already happening at L2G1**, which raises a new design question for Game 2's combo bonus scope.

Decision tree recommendation:
- **Next playtest**: Stack A Game 2 (nerf + combo bonus) with two experienced players. Stack B drops in priority. Stack F still long-term.
- **Pre-Game-2 design question**: should the combo bonus reward cross-category combos or only Strike+Strike? Resolve before drafting the rule sheet.

---

**End of analysis.** Ready for the subsequent solutions / next-playtest-prep discussion when you are.

---

## 12. Cascade Applied (Session 15 → Session 16)

The proposed entries below were applied to the living docs on 2026-05-18. This section is preserved for traceability — what was proposed and where it landed.

### Proposed new OQs for `game-state/OPEN_QUESTIONS.md`

#### OQ-52 (proposed): Centre of the Board Has No Attractor
`[System: Board/Spatial] [Affects: Movement, King role, Opening dynamics]`

**The centre of the board is mechanically inert and players naturally flank-drift at the start.**
- Three reinforcing causes (Playtest 3): (a) starting formation crams pieces into centre 6 columns of 10-wide board → opening play is about spreading to flanks; (b) opponent pieces are clustered in the centre → engaging centre means engaging strongest concentration first; (c) King is centre-positioned but functionally a back-row skill carrier → "keep safe" pulls it backward.
- Centre is doubly anti-attractive in the opening (cramped + dense with enemies). Nothing rewards being there.
- **Status**: Open. Real found issue. Solution space includes: central rune/scoring tile, contested resource at centre, narrower board (8×10 — see backpocket), formation rework that opens centre lanes, or accepting it and designing around it.
- **Connected to**: OQ-53 (King-as-target), OQ-1 (board size — partially resolved, but 8×10 not yet evaluated).
- **Re-entry trigger**: Address before or during Stack F (Cleverness II), since sente skills assume there are valuable squares/positions to threaten — without a centre attractor, sente threats may stay at the flanks.

#### OQ-53 (proposed): Attrition vs. Regicide — Should the King Be a Real Target?
`[System: Win condition / Strategic texture] [Affects: King role, Pacing, Endgame, Sente design]`

**The formal win condition is King capture, but the real victory path is attrition. The game is currently played as "wear down the army," not "threaten the King."**
- Playtest 3 evidence: Mario surrendered when his last Champion died, not when his King was threatened. Elias never targeted the King — he ground out the army. King capture would have been an inevitable consequence, not a parallel strategic axis.
- **Designer's intent (Session 14)**: The King should be a real threatenable target — "getting the King to be an active part of the game" is when the game becomes "real fun."
- **Critical clarification**: King Lifetime HP (in backpocket) does NOT fix this alone. Adding HP to a static piece just makes a static piece harder to kill. The lever is *making the King participate*, not *making the King durable*.
- **Solution space (to brainstorm, not decide here)**: starting-formation swaps that expose the King (see backpocket); central-attractor mechanics that pull the King forward (overlaps with OQ-52); sente threat skills that target the King specifically; mobility/safety asymmetry that rewards King advancement.
- **Status**: Open. Real found issue, designer-intent driven.
- **Connected to**: OQ-52 (centre attractor), OQ-19 (endgame acceleration), OQ-51 (mechanical levers for clever play — sente skills are sharper when there's a high-value target to threaten).
- **Re-entry trigger**: Brainstorm session before Stack F, OR a dedicated King-role design session.

### Proposed new backpocket entries for `docs/backpocket.md`

#### Backpocket: 8×10 narrower board variant
- **What it fixes / improves**: shrinks the "spread to the flanks" runway. Pieces can't fan as far before hitting the edge → potentially less flank-drift at opening, more incentive to engage centrally. Addresses OQ-52 (centre attractor) directly via geometry rather than via added mechanics.
- **Trigger condition**: when OQ-52 reaches an active design-discussion phase, OR alongside a Stack D (Board) test.
- **Risks**: increases piece density per column → standoff risk could re-emerge (the problem we just solved with the attack nerf). Asymmetric (rectangular not square) board changes skill-range and LoS feel. Hard to isolate effect from other variables — must test as a single-variable change.
- **Status**: `[TO DISCUSS]` — staged option for Stack D.

#### Backpocket: Starting-formation swap to expose the King
- **What it fixes / improves**: addresses OQ-53 (King isn't a real target) by changing the *starting* geometry so the King is more open from turn 1 — without changing what the King *is*. Specifically: swap the centre 2 Champions with the Guards in front of them, OR swap King + adjacent Champion with their fronting Guards, OR similar formation tweaks that reduce the King's screen.
- **Trigger condition**: as part of an OQ-53 design discussion. Lightweight to test (no rule changes — only initial setup changes).
- **Risks**: swapped formations may unbalance opening (the player who's better at exploiting an exposed King wins reliably). Could push too far and make the King die too quickly. Test as one of several formation variants, not as a single fix.
- **Status**: `[TO DISCUSS]` — needs brainstorming.

#### Backpocket: "Spec the game for a programmer" exercise
- **What it fixes / improves**: forces unambiguous rule definitions. Code has no tolerance for "we'll figure it out at the table" — writing an implementation spec exposes every ambiguous interaction (Lance Thrust + Injured, Focus Strike scope, push-into-LoS-blocker, etc.) and forces decisions. Output: a cleaner ruleset with no hidden gaps. Doubles as a foundation for the iPad/web prototype if that goes ahead.
- **Trigger condition**: anytime; scope-limited (write spec, do not build). Could be a single dedicated session.
- **Risks**: low. Time investment, not design risk. Could surface contradictions in current rules that need resolving (which is the point).
- **Status**: `[TO DISCUSS]` — bookmarked exercise.

#### Backpocket: Digital playtest prototype (web / iPad / Tabletop Simulator)
- **What it fixes / improves**: faster playtest iteration cycles, cleaner data capture (auto-logged rounds, attacks, armor, runes), can play during travel or short windows, and forces rule-disambiguation as a by-product (see "Spec the game for a programmer" entry above). Useful as a *complement* to physical playtests, not a replacement.
- **Trigger condition**: travel window with playtest partner, OR after 2+ more physical playtests when iteration speed becomes the bottleneck.
- **Scope discipline**: minimum viable = drag-and-drop simulator + long-press wheel for Injured/Armor/skill-equip + side-panel rune/round tracking. **No rules enforcement, no AI opponents, no polish.** Treat as a tool, not a product.
- **Risks**: scope creep (polish is bottomless); risk of "the digital version becomes the game" defeating the screen-free design intent; rule-state divergence between digital and `ruleset-baseline.typ`. Decision needs an ADR before any implementation work.
- **Status**: `[TO DISCUSS]` — sleep-on-it. ADR required before any building.

---

**Cascade applied 2026-05-18**: OQ-52 + OQ-53 added to `OPEN_QUESTIONS.md`; four backpocket entries (8×10 narrower board, starting-formation swap, "spec for a programmer", digital playtest prototype) added to `backpocket.md`; standard attack 1 DMG moved to Accepted-In-Baseline in `mechanics-evaluated.md`; OQ verdicts updated (OQ-37, OQ-40, OQ-21, OQ-11, OQ-46, OQ-34 with Mode A/B + early/late time-dependence, OQ-10, OQ-41); `NEXT_STEPS.md` restructured for Session 16 with pre-Game-2 prep priorities.
