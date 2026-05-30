# Open Questions — High Concept Investigation

**Date:** 2026-05-25
**Source:** Surfaced during Angle 1 (`youtube-transcript-high-concept.md`) and Angle 2 (`system-audit-vs-high-concept.md`).
**Purpose:** Single staging doc for questions raised by the high-concept work, ready to feed into a follow-up design discussion. Not yet tied to OQ-numbered entries in `game-state/OPEN_QUESTIONS.md` — these are *pre-OQ*: candidates for promotion if and when one becomes a design priority.

---

## How to read this doc

Each question is tagged with:

- **Origin** — which angle / section surfaced it
- **Type** — `framing` (about how we describe the game), `design` (about what the game *is*), `on-ramp` (about first-game experience), `validation` (needs more data), or `meta` (about how we make decisions)
- **Decidability** — `cheap` (decide now from existing data), `medium` (one focused discussion or 1 playtest), `expensive` (multiple playtests or large rework)
- **Connected to** — adjacent questions or existing OQs

Questions are grouped by theme, not by source angle, so related ones sit next to each other.

---

## Theme A — High concept framing

These questions ask *what the high concept is*, not *whether it's working*.

### Q-A1 — Is the high concept "clever combo discovery" or "clever combo discovery *against an opponent solving the same puzzle*"?

**Origin:** Angle 1 (initial relevance section), Angle 1 verdict (implications section).
**Type:** `framing`
**Decidability:** `medium` — likely an ADR, or a one-paragraph decision after one more independent playtest.
**Why it matters:** The two framings yield different design choices. Framing A treats the opponent as a constraint generator; Framing B makes the parallel-solving experience load-bearing.
**Current evidence:** The strongest evidence pointing to Framing B was Elias's "thinking and **together**" quote from P2 — but that's been excluded from independent data per the designer-as-data-point bias. So we currently have **no independent evidence** for either framing.

**Status: RESOLVED 2026-05-26 — ADR-004, Framing B chosen.**

The decision was made from design intent + architectural fit, not from playtest data (none exists yet for either framing per the designer-bias correction). Three load-bearing reasons, full text in ADR-004 (`docs/mechanics-log/mechanics-evaluated.md`):

1. The chassis is already partially Framing-B-aligned (shared draft pool, perfect info + open loadouts, Principle 5). Framing B turns those features from "features" into "core."
2. Existing principles already imply it (Principle 5 names "shared-puzzle feel"; North Star says "appreciate — for both players").
3. Framing B produces a sharper, more distinctive game. Framing A puts us in a crowded category; B puts us in a smaller, more interesting one.

**What was committed:** Framing B as design intent. Q-D1 signal definitions read against B. Phase B briefed under B. Future mechanical decisions evaluated partly on B-alignment (soft preference, not mandate).

**What was NOT committed:** No immediate rule changes. No commitment to add simultaneous-reveal phases, shared resource pools, or designed analysis moments. No commitment that the chassis as-it-is delivers Framing B in one game (Q-D1 tests that).

**Reversal criterion:** if Q-D1 returns "doesn't land" across the validation window AND on-ramp interventions (Q-B1, Q-B2, Q-B4) have been tested without improving the result, revisit ADR-004.

**Connected to:** Q-D1 (signal definitions depend on this framing), Q-F1 (Phase B brief depends on this), Q-B1 / Q-B2 (on-ramp choices shaped by this — see notes in those questions).

### Q-A2 — Does the chassis vs. engine distinction itself need to live in the design docs?

**Origin:** Angle 2 (system audit framing).
**Type:** `framing` / `meta`
**Decidability:** `cheap` — yes/no decision, half a session of work to write up.
**Why it matters:** The audit found the distinction useful for diagnosis (which systems pull which way for which players). If the project keeps using this lens, it should be a named concept in `design-principles.md` rather than living only in research notes.
**Status:** RESOLVED 2026-05-26 (Session 20) — section added to `design-principles.md` (§ Chassis and Engine). ~150 words, defines chassis as infrastructure-for-skills and engine as the skill+combo system, with worked example (Q-C1 = chassis-volume reduction). Lens now canonical project vocabulary; companion to Justification Rule.
**Connected to:** Justification Rule (CLAUDE.md), Core Fantasy.

---

## Theme B — On-ramp / first-game experience

The biggest cluster — these questions all share the root cause that the high concept hasn't been confirmed for any non-designer player, and the audit found this gap is concentrated in specific systems.

### Q-B1 — Could a "starter loadout" for first-time players deliver the high concept on round 1 without changing rules?

**Origin:** Angle 2 (Skill Drafting findings, headline finding 3, Q3 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — design 2-3 candidate loadouts, test one in a future first-timer playtest. Could plug straight into Niko's 2026-05-28 session if decided in time.
**Why it matters:** The audit identified Drafting's on-ramp problem as the **lowest-cost lever with highest information content**. Newcomers draft before they understand combos; pre-built loadouts trade draft depth for combo legibility on round 1.
**Open sub-questions:**
- What does a known-good "intro combo" loadout look like? (Probably contains Focus + a Strike skill + a movement skill for setup.)
- Should both players get the same starter loadout, mirrored loadouts, or the *same single loadout* (so the asymmetric-information dimension is zero on round 1)?
- Does using a starter loadout in game 1 reduce the player's combo-discovery joy in game 2 when they draft for the first time? (The "tutorial spoils the puzzle" risk.)

**Status: PARTIALLY RESOLVED 2026-05-26 — design intent set, execution deferred until after Niko.**

**Decided shape (per ADR-004 Framing B):**
- **Single shared loadout for both players in game 1.** Both players solve literally the same combinatorial puzzle on their first contact. (Pre-ADR-004, mirrored-but-different was a candidate; post-ADR-004, single-shared is the higher-leverage option because Framing B makes parallel-solving load-bearing.)
- Loadout content target: at minimum Focus + 1 Strike skill + 1 movement skill that sets up combos (Blast or Hook). Final composition deferred until execution.

**Design constraint when the loadout is eventually written:** the loadout must teach the *grammar* of combos (Focus enables a Strike) without handing players the *jokes* — emergent combo discovery in game 1 must remain possible. The starter is scaffolding, not a solution book.

**Use-trigger (when this becomes a "wenn ich gegen neuen Spieler spiele, das nehmen" rule):**
- After Niko's session (2026-05-28). If Niko lands strong-signal Q-D1, starter loadouts may be unnecessary — keep deferred.
- If Niko lands weak/no signal, write the loadout and use it in the next first-timer session.

**Connected to:** Q-B2 (originally intended to test in conjunction; now Q-B2 is rejected — see below), Q-D1 (the trigger), ADR-004.

### Q-B2 — Could combo hints on skill cards deliver the high concept on round 1 without changing rules?

**Origin:** Angle 2 (Skill Drafting findings, Skill System findings, Q6 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — write hints, test in playtest. Cards already exist as `images/`, this is a content edit.
**Why it matters:** Same on-ramp logic as Q-B1, lighter intervention. *"Pairs well with X"* on the card surface tells a newcomer that combos exist before they draft.
**Open sub-questions:**
- Does this risk *over-determining* the meta? If every card lists its best partner, do experienced drafts converge on the listed pairs?
- Should hints be on physical cards or only on a separate cheat sheet shown during the draft?
- Does this change the draft's strategic surface (you're now drafting against your opponent *and* against the card text's nudges)?

**Status: REJECTED 2026-05-26.**

Hints would prescribe what the player should *discover*. The whole point of the high concept (per ADR-004 Framing B) is that *"clever combo discovery"* is **emergent** — players figure out combos themselves, feel clever, recognise elegance in each other's plays. Card text that telegraphs "this pairs with X" hands players the punchline before they get to the joke.

**The over-determination concern from the original sub-questions is the deciding factor here, not just one consideration among many.** Under Framing B, narrowing the combinatorial space (drafts converging on listed pairs) actively damages the design — the shared puzzle gets smaller, and "discovering" a printed combo isn't really discovery.

**Possible distant-future re-emergence:** if at some point we publish "advanced rules / hints" content for experienced players who want to read about catalogue depth (after they've had their organic discovery moments), card-style references could live there. Not a live question; not on the agenda.

**Connected to:** Q-B1 (originally bundled — Q-B1 still proceeds, Q-B2 does not), ADR-004.

### Q-B3 — Should drafting be deferred until game 2?

**Origin:** Angle 2 (Skill Drafting net assessment).
**Type:** `on-ramp`
**Decidability:** `cheap` to decide as a teaching protocol; `medium` if it becomes a formal "tutorial mode" rule.
**Why it matters:** New players draft *before* they have combo intuition, so first-time drafts default to chassis-flavored picks (Mario's 2× Plate). Deferring drafting to game 2 means the first game is purely about *experiencing* combos, the second game is about *building toward* them.
**Tradeoff:** Removes the strategic on-ramp's hardest decision from round 1 — but also removes a key part of the game from the first experience entirely.
**Status:** RESOLVED 2026-05-26 (Session 20) — entailment from Q-B1. When the starter loadout ships, the draft is skipped in game 1 by construction (both players get the same preset, nothing to draft). Teaching protocol: *mention* drafting exists during game 1 rules explanation, signpost it as "this is what you'll do in game 2 with the full system." Reasons: (a) respects the player's intelligence, (b) creates anticipation for game 2 — which intersects Q-D2's question about whether the engine becomes audible after one full game. Q-B3 will not be tracked as an independent question going forward.

### Q-B4 — Could the Move-Attack be reframed in the rule text to plant skill-first thinking?

**Origin:** Angle 2 (Combat findings, Q4 in audit's open questions).
**Type:** `framing` / `on-ramp`
**Decidability:** `cheap` — text edit in `baseline-sections.typ`.
**Why it matters:** Move-Attack is currently presented as a sub-clause of the Move Phase ("spend an action to attack"). That rule placement teaches "attacking is a chassis verb." The mechanic is fine post-Stack-A; the *framing* is the lever.
**Open sub-questions:**
- Could attacks become their own short section *between* Movement and Action, signposting the transition from chassis-verbs to engine-verbs?
- Or: does attack belong inside the Combat section, called out from Movement only by reference?
- Does any reframing risk *confusing* new players who currently parse "attack = move onto enemy" cleanly?

**Status: RESOLVED 2026-05-26 — executed.**

**Decision:** Keep Move-Attack inside its existing standalone section, but reword both the Move Phase intro and the Move-Attack opening to make the move-attack unity *explicit* and signal the chassis→engine transition without introducing internal jargon.

**Why this shape (not "promote to its own section between Movement and Action"):** the user pushed back on the "own section" option because attacking *is* done via movement — splitting them risks players parsing them as separate verbs and getting confused about the survival-stop rule (Guard speed 2 attacking a survivor only moves 1 tile; Champion speed 1 attacking a survivor doesn't move at all). Unity of the verb is more important than pedagogical signposting. Reword instead.

**What changed in `baseline-sections.typ` (BASELINE_VERSION 2026-05-26):**
- Move Phase intro now says: *"Spend 1 action to move one piece — either into empty space (normal movement) or into an enemy tile (a Move-Attack — see next section)."*
- Move-Attack section opens with: *"A Move-Attack is a Move that ends on an enemy tile."*
- Survival-stop rule strengthened with the explicit attacker-speed cases (Guard speed 2 → 1 tile moved; Champion / King speed 1 → 0 tiles moved; damage dealt either way).
- Added a closing italic line in the Move-Attack section (in player-facing language, not internal chassis/engine vocabulary): *"Move-Attacks are how pieces deal damage with movement alone. Skills — activated in the Skill Phase — are the other way pieces affect each other, and use different rules (Path, Range, Money)."*

**Effect on Q-D1 reading:** This is one of the few accepted changes specifically aimed at *first-game framing*. If Niko's session and subsequent first-timer sessions show a shift in Q11 vocabulary (more skill-first language) compared to P1/P2/P3, this reframing is plausibly part of that — but separating its effect from Q-B1 (when that ships) and from natural variance across players will require multiple data points.

**Connected to:** ADR-004 (skill-first framing serves Framing B), Q-D1 (validation reading).

### Q-B5 — Does the Injured state's teaching cost exceed its first-game relevance?

**Origin:** Angle 2 (Health & Armor findings, Q5 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — needs a playtest with Injured-hidden teaching to compare.
**Why it matters:** Injured carries non-trivial teaching cost (penalties, range modifiers, edge cases like "doesn't affect 'self'/'adjacent' skills"). For a first-time player who'll mostly see 1-damage damage anyway, the *concept* of an Injured intermediate state may be more rules-overhead than gameplay-payoff.
**Tradeoff:** Hiding Injured for first games could simplify the on-ramp dramatically — but the moment the new player takes 1 damage, the rule has to be explained mid-game. Or pieces just track HP without naming the state.
**Status:** RESOLVED 2026-05-26 (Session 20) — reframed and parked as Stack H candidate.

**Reframing:** The original framing was "teaching protocol" (hide Injured for game 1 only). User pushed back: the right test isn't *explained vs. hidden*, it's *with vs. without*. Cleaner experiment, and the result generalises beyond onboarding — if Injured-downsides-removed plays well for experienced players, it becomes a baseline-change candidate, not just a first-game variant.

**Stack J scope:** Remove Injured's mechanical downsides entirely. The state still exists as HP-tracking (Normal → Injured → Removed = 2 HP → 1 HP → 0 HP), but Injured pieces have no speed cap and no Range −1 penalty.

**Justification (chassis/engine lens):** Injured carries non-trivial chassis volume — speed cap, Range modifier, the "doesn't affect self/adjacent" carve-out, the chained derivation for Range−1 skills on Injured pieces. Tests whether that volume is paying for itself in game-feel terms or whether it's chassis bloat.

**What it tests:**
- Game length / pacing (no Injured speed cap on Guards = faster mid-game repositioning).
- Combo grammar effects (Focus + Injured Range−1 interaction disappears — some combo-enabler shapes simplify, some lose texture).
- First-game teaching-cost reduction (real, but a side effect — not the experiment's purpose).

**Trigger / gating:** Park behind Stack A G2 *and* Stack H. Reasons: (a) Stack A G2 changes combo lethality, which interacts with whether Injured pieces still threaten meaningfully at full range; (b) Stack H reduces chassis volume on the Armor side, so Stack J's chassis-volume reduction reads cleaner against an already-trimmed baseline.

**Recognised risk:** Stack H might prove too much — if it plays well, it becomes a baseline-change candidate, which is a larger decision than the original onboarding-flavoured Q-B5 framing. That's an acceptable risk; it would just mean Q-B5's resolution scales up, not that Stack H was misframed.

---

## Theme C — Chassis bloat / volume management

These ask whether parts of the game are louder than they need to be.

### Q-C1 — Should the Armor cap drop from 3 to 2?

**Origin:** Angle 2 (Health & Armor findings, headline finding 2, Q2 in audit's open questions).
**Type:** `design`
**Decidability:** `medium` — single-variable test stack.
**Why it matters:** The audit flagged Health & Armor as the strongest chassis-bloat candidate. P3 saw Mario stack ~20 Armor, Elias break ~6, the loop consumed real game time. A cap of 2 (instead of 3) would shrink the loop's volume without breaking the Armor vs. Armor-Breaker RPS structure.
**Open sub-questions:**
- Does a cap-of-2 still create the "presence" effect (P3: Armor "gave presence")?
- Does it weaken Armor too much against multi-skill combo turns?
- Is the right lever the cap *or* the per-application amount (e.g., Plate grants +1, but maybe it should grant +2 with the cap held)?
**Status:** RESOLVED 2026-05-26 (Session 20) — parked as Stack H candidate, gated on Stack A G2 results. OQ-11 reopened under chassis-volume framing.
**Decision:** Stack H will bundle C1b (cap 3→2 *and* Plate +1→+2). Bundled because the two changes are coupled — C1b's volume-reduction effect depends on the cap, and the Plate change depends on the cap to constrain it. Documented as a legitimate coupling per `design-principles.md` § Incremental Testing Methodology. If Stack H shows Armor stalling becomes dominant (build cheaper than break with no faster-than-Armor kill path), rollback to C1a (cap 3→2 only, Plate unchanged) as Stack I. Trigger: after Stack A G2 confirms whether the multi-Champion combo bonus reliably creates faster-than-Armor kill paths. If combos overrun Armor reliably, the chassis-volume problem may auto-reduce and Q-C1 may dissolve.
**Connected to:** OQ-11 (reopened under chassis-volume framing in `OPEN_QUESTIONS.md`).

### Q-C2 — Is Bodyguard's strategic value combo-relevant or chess-relevant?

**Origin:** Angle 2 (Combat findings, headline finding 5, Q1 in audit's open questions).
**Type:** `design` / `framing`
**Decidability:** `medium` — needs analysis of P3 + future playtest data on *what kinds of decisions* Bodyguard generates.
**Why it matters:** Bodyguard is the most chess-coded sub-system in the game. It works strategically but speaks war-chess vocabulary. If the strategic value comes from *combo-relevant decisions* (positioning Guards to enable skill setups), it's quietly combo-serving despite the framing. If it's *chess-relevant* (defensive screening), it may be reinforcing chassis-first thinking even when working as designed.
**Open sub-questions:**
- In P3, when Bodyguard triggered "way more often" (Elias), what kinds of combo-adjacent decisions emerged from that? (Need to look at game logs.)
- Is there a way to *test* this — e.g., a stack where Bodyguard interacts with skills (not just Move-Attacks) to see if the combo-relevant version is preferred?
- Could Bodyguard's vocabulary be reframed without touching mechanics? (e.g., a non-chess name)
**Status:** RESOLVED 2026-05-26 (Session 20) — sub-q1 answered, sub-q2 deferred to Phase B, new watch-flag surfaced.

**Sub-q1 (combo-relevant or chess-relevant?):** *Chess-relevant.* User-reported P3 observation: Mario used Bodyguard to protect Champions from damage — the trade-up logic (Guard worth less than Champion = intended trade). That's classic chess sacrifice/screening, not combo-positioning. Bodyguard is *behaviourally* chassis-coded, not just vocabularily.

**Sub-q2 (vocabulary):** Deferred to Phase B / Q-F1 (game identity & naming work). Renaming is theme work, not mechanics.

**Sub-q3 (test combo-relevant variant — Bodyguard intercepts skills too):** Implicitly de-prioritised. Sub-q1 says the chess-relevant version is what's actually happening; testing a combo-relevant variant would be inventing a feature, not fixing one.

**New watch-flag (surfaced today):** Under Framing B's "soft preference for B-aligned over A-aligned" (ADR-004), Bodyguard is the most chess-coded mechanic in the chassis (now confirmed in both name *and* behaviour). Not actionable today. If a future stack proposes simplifying or removing Bodyguard, Framing B is one of the arguments for it. Logged to OQ-21 (existing Bodyguard tracking).

**Connected to:** Q-F1 (Phase B naming), OQ-21 (Bodyguard rule tracking), ADR-004.

### Q-C3 — Is the 10x10 + 12 pieces chassis the minimum-viable host for the combo system?

**Origin:** Angle 1 (initial open questions), Angle 4 placeholder.
**Type:** `design`
**Decidability:** `expensive` — full Angle 4 work, multiple playtests.
**Why it matters:** Specificity guidepost: less chassis = combos louder. If a smaller chassis (8x8, fewer Champions, no Guards as separate pieces) supports the same combo experience, the simplification is pure win for the high concept.
**Status:** RESOLVED 2026-05-26 (Session 20) — mapped to existing OQs and concretised as a session plan.

**Mapping:** Q-C3 was the umbrella question; its operational form is OQ-1b (board size) and OQ-27 (piece count), both already in `OPEN_QUESTIONS.md` as deferred candidates.

**Concrete plan added (Stack K in `TESTING_PLAN.typ`):** A two-game playtest session that unbundles OQ-1b and OQ-27 in time but keeps them in the same session for clean comparison:
- **Game 1**: 8×8 board, current piece count (5 Champions + 6 Guards + 1 King). Single-variable change from baseline = board size.
- **Game 2**: 8×8 board, reduced piece count (3 Champions + 3 Guards + 1 King). Single-variable change from G1 = piece count.

**Why session-shaped, not stack-shaped:** Stacks are single rule-states; Stack K is a *plan* that uses two rule-states sequentially in one session. Same-session control: same players, same loadouts (or fresh draft), so effects can be attributed cleanly to each variable.

**Trigger:** Post-Stack-A G2 (combo bonus data informs whether smaller chassis still hosts the engine well). Needs two experienced players for a full session.

**Connected to:** OQ-1b, OQ-27, backpocket entry "8×10 narrower board variant", Stack K in `TESTING_PLAN.typ`.

---

## Theme D — Validation / data needs

Questions that can only be answered by getting more data.

### Q-D1 — Does the high concept land for new players in one game?

**Origin:** Angle 1 verdict (the central finding).
**Type:** `validation`
**Decidability:** `medium` — needs Niko's session (2026-05-28) plus ~3 more first-timer sessions to read a trend.
**Why it matters:** This is the question Angle 1 set out to answer and *couldn't* — three independent data points (Pasco, Jonathan, Mario), 0.5 / 3 positive. Genuinely unknown.

#### Resolution criteria (decided 2026-05-25)

The question can't be *answered* in discussion — it's empirical. What was decided here is **how it gets answered**: signal definitions, data target, and the bias-correction mechanism. The question stays open until the data is in.

**Signal strength definitions** (used to read each first-timer session):
- **Strong signal — unprompted combo-first description.** Player describes the game (e.g., onboarding form Q11, Q12) using "combo" / "setup" / "chain" / "sequence" / "synergy" / "build" *without those words being used in rule explanation*. Native vocabulary emergence.
- **Medium signal — combo as Q12 favorite moment.** Player names a multi-skill sequence as their best moment, even if their general description (Q11) is chassis-flavoured.
- **Weak signal — recognising one combo in retrospect.** Player notices that *something* combo-shaped happened (Jonathan P2 Turn 23 was this) but doesn't reach for combo-first vocabulary outside that moment.

**Decision thresholds** (across the validation window):
- **Lands:** ≥ 2 of 4 next first-timers show *strong* signal.
- **Doesn't land:** 0 of 4 show even *medium* signal.
- **Unclear:** anything in between → triggers a separate decision about whether to extend the validation window, change the on-ramp, or revisit the high concept itself.

**Data target:** Niko (2026-05-28) + ~3 more first-timers. Rule changes between sessions are *not* frozen — but each session's rule state must be tagged with the data so trend-reading stays honest (already standard; P1/P2/P3 are tagged).

**Bias-correction mechanism — pre-game teacher checklist:** Per `feedback_designer_is_not_data.md`, the analysis is circular if the teacher seeds combo-vocabulary during teaching. The teacher (Elias) fills out `docs/test-scenarios/shared/teacher-vocab-checklist.pdf` *immediately after teaching, before the game starts*, recording which combo-coded words were used during rule explanation. After the game, the checklist is filed alongside the player's onboarding feedback. Analysis logic:
- Player said "combo" + teacher did NOT say "combo" → **strong signal** (emerged).
- Player said "combo" + teacher said "combo" repeatedly → **weak signal at most** (taught).
- Player did NOT use combo-first vocab + teacher avoided it → **real "didn't land" result** (uncontaminated negative).

**Discipline notes (carried forward from Angle 1):**
- Don't lead with the high concept during rule explanation.
- Q11 ("in your own words, what is this game about?") is the primary signal — onboarding form already asks it.
- Q14 (anchoring to other games) is secondary — chassis-coded comparisons (chess, war games) vs. combo-coded comparisons (deckbuilders, combo-card-games) help triangulate.
- Separate Elias's quotes from independent player quotes rigidly (per `feedback_designer_is_not_data.md`).

**Status:** Open. Highest-priority data need. Resolution arrives after Niko + ~3 more first-timer sessions, read against the criteria above.

### Q-D2 — Does game 2 (after the player has seen one combo) shift their language toward combo-first?

**Origin:** Angle 1 verdict (Jonathan recognised exactly one combo in P2 — late, isolated).
**Type:** `validation`
**Decidability:** `medium` — needs a returning first-timer for a second session.
**Why it matters:** The audit's pessimistic reading is "the engine isn't audible until you've heard it." If true, *one* full game might not be enough; the high concept may land on game 2. That changes how we evaluate first-game-only data.
**Status:** RESOLVED 2026-05-26 (Session 20) — linked to Q-D1's resolution criteria; ADR-004 reversal criterion updated.

**Resolution:** Q-D2 uses the same signal definitions and teacher-vocab-checklist mechanism as Q-D1, applied to game 2 data instead of game 1 data. A returning first-timer's game 2 produces a Q-D2 datapoint; the same Q11/Q12 + checklist analysis logic determines strong/medium/weak signal.

**Key analytical point added:** A weak game-1 signal followed by a strong game-2 signal is *not* "doesn't land" — it's "lands at game 2 cadence." Pessimistic-audit reading ("the engine isn't audible until you've heard it") could be true and Framing B could still hold.

**ADR-004 update:** Reversal criterion now reads against the *combined* Q-D1 + Q-D2 result. ADR-004 will be reversed only if Q-D1 returns "doesn't land" *and* Q-D2 also returns "doesn't land" across the validation window, AND on-ramp interventions (Q-B1, Q-B4) have been tested without improving either result.

**Q-D2 will not be tracked as an independent question going forward.**

**Connected to:** Q-D1, Q-B3 (game 1 starter loadout sets up the game 2 first-draft experience), ADR-004.

### Q-D3 — Does the Multi-Champion Combo Bonus (Stack A Game 2, pending) raise the combo ceiling without crowding out cross-skill-category combos?

**Origin:** Playtest 3 analysis (already in `OPEN_QUESTIONS.md` as OQ-38), surfaced again by Angle 2 audit of Combat.
**Type:** `validation`
**Decidability:** `medium` — Stack A Game 2 playtest.
**Why it matters:** P3 saw Elias do an organic Blast → Hook cross-piece combo *without* the bonus active. The pending bonus mechanic only triggers on Strike+Strike sequences from different Champions. Risk: the bonus creates a Strike+Strike monoculture and *crowds out* the cross-category combo behavior that already emerged organically.
**Status:** RESOLVED 2026-05-26 (Session 20) — sharpened framing added to OQ-38 in `OPEN_QUESTIONS.md`. Q-D3 will not be tracked as an independent question. Stack A G2 feedback will include a counterfactual probe: "Did the Strike+Strike bonus draw you away from cross-category combos that you'd otherwise have built?"

---

## Theme E — Skill catalogue / engine content

These ask whether the engine itself has enough content to deliver the high concept.

### Q-E1 — Does the current skill catalogue support enough combo discovery, or is it too narrow?

**Origin:** Angle 1 (initial open questions), Angle 2 (Skill System net assessment).
**Type:** `design`
**Decidability:** `medium` — needs a playtest where draft variety is the explicit watch item.
**Why it matters:** P3 saw Mario draft 2× Plate and never build a combo-shaped loadout. Partly bad draft luck, partly catalogue shape: with 15 skills and only some of them being combo-shaped pairs, it's possible to draft a chassis-only army. If the catalogue is too narrow, the engine has thin content; if it's too wide, drafts have too much variance.
**Open sub-questions:**
- How many *distinct combo shapes* does the catalogue actually support? (e.g., Strike+Strike, Move-into-Strike, Buff-then-Strike, Setup-then-Combo.)
- Would adding 2-3 explicitly combo-shaped skills (e.g., a "next ally piece's skill costs −1 Money" enabler) widen the engine without breaking balance?
**Status:** RESOLVED 2026-05-26 (Session 20) — parked with symptom trigger.

**Trigger:** "Experienced players report games becoming repetitive / combo space feeling exhausted." User-facing symptom-based trigger, per Justification Rule. Not a timing trigger.

**Why symptom-trigger and not timing-trigger:** Adding catalogue content is mainly beneficial to experienced players, of which we currently have one (designer = excluded as data per `feedback_designer_is_not_data.md`). New-player overwhelm is the *current* problem; catalogue expansion would worsen it without a clear corresponding benefit. Park until experienced players exist *and* report exhaustion.

**Intervention-type distinction (added for future-us):** When the trigger fires, the right lever isn't necessarily "add more skills." Two distinct interventions:
- *Replace-for-breadth*: swap an underperforming current skill (e.g. Mystic's "1 must-pick + 1 never-pick" problem flagged in OQ-12 research) for a skill that implements a *new combo shape*. Catalogue count stays flat; combo-shape coverage widens. Lower newcomer cost.
- *Expand-catalogue*: add new skills on top of existing ones (OQ-12's ~25-skill target). Count goes up; newcomer cost goes up.

When the symptom-trigger fires, evaluate which intervention the symptom actually calls for. Don't default to expansion just because OQ-12 is parked there.

**Connected to:** OQ-12 (skill catalogue expansion to ~25, in backpocket — keeps its existing parking, but should reference Q-E1's intervention-type distinction when triggered).

### Q-E2 — Could skill cards be reshaped to communicate combo-affinity without changing rules?

**Origin:** Angle 2 (Skill System findings, Q6 in audit's open questions).
**Type:** `on-ramp` / `framing`
**Decidability:** `cheap` to design, `medium` to test.
**Why it matters:** The audit found the engine doesn't announce itself as the engine. Card layout is one place where this could change without rule changes — e.g., colored borders for combo-pair groups, an "amplifies / amplified by" line, visual icons that hint at chain structure.
**Status:** RESOLVED 2026-05-26 (Session 20) — split into three parts.

**Combo-affinity visual signaling: REJECTED.** Same reasoning as Q-B2 — visual hints (colored borders, "amplifies / amplified by" icons, chain-structure visual cues) still prescribe what should be *discovered*. Pattern-matching by color = telegraphing the punchline in a different medium. Under Framing B, narrowing the discoverable space is a damage, not a feature.

**Card legibility (rune cost prominent, range visible, effect summarised): ALREADY SHIPPED.** Physical skill cards delivered Session 18 (`shared/skill-cards.pdf`). Cards organize by category (Strike / Shield / Move / Mystic) via existing labels — that's organizing by archetype, not pre-announcing combos.

**Anything beyond legibility (theme art, naming, full visual identity): PHASE B WORK.** Belongs in Q-F1's territory.

**Connected to:** Q-B2 (rejected for same reasons), Q-F1 (Phase B), OQ-56 (skill cards UX, C1 shipped).

---

## Theme F — Phase B / theme & identity

Forward-looking — these will become urgent when Phase B (naming, theme, art) starts.

### Q-F1 — What theme/visual language reinforces "find the elegant solution" instead of "wargame with spells"?

**Origin:** Angle 1 (forward-looking applications, point 4).
**Type:** `framing`
**Decidability:** `expensive` — Phase B itself.
**Why it matters:** The high concept rules out theming that pulls toward war-chess (Battle Chess), narrative campaigns (Gloomhaven), or character-class identity (Summoner Wars). It pulls toward something more like *puzzle-arcane* — visual language that reinforces elegance and sequence-discovery rather than combat and progression.
**Status:** CONSOLIDATED 2026-05-26 (Session 20) — now functions as a Phase B brief, awaiting Phase B start.

**Brief inputs (linked, not restated):**
- **ADR-004** (`docs/mechanics-log/mechanics-evaluated.md`) — framing constraints: rules out war-chess, narrative campaigns, character-class identity. Pulls toward parallel-discovery / mirroring / puzzle-arcane imagery. Soft preference for B-aligned over A-aligned themes.
- **Chassis/engine lens** (`docs/design-principles.md` § Chassis and Engine) — visual treatment should make the *engine* (skills, combos) feel central; chassis (board, pieces, basic combat) should feel like supportive infrastructure, not the main character.
- **Q-C2 finding** (Bodyguard is chess-coded behaviorally + vocabularily) — Bodyguard rename is a live Phase B candidate. Whether to remove or rename is a Phase B decision informed by chassis-volume work.
- **Q-E2 residual** — anything beyond card-legibility (theme art, full visual identity, faction/no-faction question) lives here.

**Phase B trigger:** Separate from this question. Currently parked behind Phase A (mechanics) reaching stability. We're not there yet.

**Not done today:** No Phase B work. No mock-ups, no name candidates, no imagery direction. Just consolidation of the inputs so when Phase B starts the brief is already linked.

**Connected to:** ADR-004, Q-C2, Q-E2, OQ-21 (Bodyguard), `docs/game-identity-visual-naming.md`.

---

## Suggested next-discussion agenda

If/when this becomes a focused design discussion, the natural priority order is:

1. **Q-D1** (does the high concept land for new players?) — the blocking question. Run Niko's session designed to test it (per discipline notes).
2. **Q-A1** (framing decision: combo discovery alone vs. with-opponent-puzzle?) — relatively cheap to decide; unblocks downstream.
3. **Q-B1 + Q-B2 + Q-B4** (on-ramp interventions: starter loadouts, card hints, attack reframing) — the cheapest, highest-info levers to test the *fixability* of the on-ramp gap.
4. **Q-C1** (Armor cap 2 vs. 3) — a real candidate test stack.
5. Everything else as it becomes priority.

This order matches the audit's lever-cost analysis: cheap framing interventions first, then on-ramp tests, then chassis adjustments, with the big chassis-minimisation work (Q-C3) last.
