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
**Status:** Soft yes. Worth a paragraph in `design-principles.md` next time that doc is touched.
**Connected to:** Justification Rule (CLAUDE.md), Core Fantasy.

---

## Theme B — On-ramp / first-game experience

The biggest cluster — these questions all share the root cause that the high concept hasn't been confirmed for any non-designer player, and the audit found this gap is concentrated in specific systems.

### Q-B1 — Could a "starter loadout" for first-time players deliver the high concept on round 1 without changing rules?

**Origin:** Angle 2 (Skill Drafting findings, headline finding 3, Q3 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — design 2-3 candidate loadouts, test one in a future first-timer playtest. Could plug straight into Nico's 2026-05-28 session if decided in time.
**Why it matters:** The audit identified Drafting's on-ramp problem as the **lowest-cost lever with highest information content**. Newcomers draft before they understand combos; pre-built loadouts trade draft depth for combo legibility on round 1.
**Open sub-questions:**
- What does a known-good "intro combo" loadout look like? (Probably contains Focus Strike + a Strike skill + a movement skill for setup.)
- Should both players get the same starter loadout, mirrored loadouts, or the *same single loadout* (so the asymmetric-information dimension is zero on round 1)?
- Does using a starter loadout in game 1 reduce the player's combo-discovery joy in game 2 when they draft for the first time? (The "tutorial spoils the puzzle" risk.)
**Status:** Open. Highest-priority on-ramp lever from the audit.
**Connected to:** Q-B2, Q-B3, Q-D2.

### Q-B2 — Could combo hints on skill cards deliver the high concept on round 1 without changing rules?

**Origin:** Angle 2 (Skill Drafting findings, Skill System findings, Q6 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — write hints, test in playtest. Cards already exist as `images/`, this is a content edit.
**Why it matters:** Same on-ramp logic as Q-B1, lighter intervention. *"Pairs well with X"* on the card surface tells a newcomer that combos exist before they draft.
**Open sub-questions:**
- Does this risk *over-determining* the meta? If every card lists its best partner, do experienced drafts converge on the listed pairs?
- Should hints be on physical cards or only on a separate cheat sheet shown during the draft?
- Does this change the draft's strategic surface (you're now drafting against your opponent *and* against the card text's nudges)?
**Status:** Open. Combines well with Q-B1; could test together.

### Q-B3 — Should drafting be deferred until game 2?

**Origin:** Angle 2 (Skill Drafting net assessment).
**Type:** `on-ramp`
**Decidability:** `cheap` to decide as a teaching protocol; `medium` if it becomes a formal "tutorial mode" rule.
**Why it matters:** New players draft *before* they have combo intuition, so first-time drafts default to chassis-flavored picks (Mario's 2× Armorsmith). Deferring drafting to game 2 means the first game is purely about *experiencing* combos, the second game is about *building toward* them.
**Tradeoff:** Removes the strategic on-ramp's hardest decision from round 1 — but also removes a key part of the game from the first experience entirely.
**Status:** Open. Lighter version of Q-B1.

### Q-B4 — Could the Standard Attack be reframed in the rule text to plant skill-first thinking?

**Origin:** Angle 2 (Combat findings, Q4 in audit's open questions).
**Type:** `framing` / `on-ramp`
**Decidability:** `cheap` — text edit in `baseline-sections.typ`.
**Why it matters:** Standard Attack is currently presented as a sub-clause of the Movement Phase ("spend a Move Slot to attack"). That rule placement teaches "attacking is a chassis verb." The mechanic is fine post-Stack-A; the *framing* is the lever.
**Open sub-questions:**
- Could attacks become their own short section *between* Movement and Action, signposting the transition from chassis-verbs to engine-verbs?
- Or: does attack belong inside the Combat section, called out from Movement only by reference?
- Does any reframing risk *confusing* new players who currently parse "attack = move onto enemy" cleanly?
**Status:** Open. Cheapest framing-side intervention in the audit. Low risk.

### Q-B5 — Does the Injured state's teaching cost exceed its first-game relevance?

**Origin:** Angle 2 (Health & Armor findings, Q5 in audit's open questions).
**Type:** `on-ramp`
**Decidability:** `medium` — needs a playtest with Injured-hidden teaching to compare.
**Why it matters:** Injured carries non-trivial teaching cost (penalties, range modifiers, edge cases like "doesn't affect 'self'/'adjacent' skills"). For a first-time player who'll mostly see 1-DMG damage anyway, the *concept* of an Injured intermediate state may be more rules-overhead than gameplay-payoff.
**Tradeoff:** Hiding Injured for first games could simplify the on-ramp dramatically — but the moment the new player takes 1 DMG, the rule has to be explained mid-game. Or pieces just track HP without naming the state.
**Status:** Open. Speculative. Worth thinking about, not urgent.

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
- Is the right lever the cap *or* the per-application amount (e.g., Armorsmith grants +1, but maybe it should grant +2 with the cap held)?
**Status:** Open. Real test candidate.
**Connected to:** OQ-11 (Armor cap, currently in `OPEN_QUESTIONS.md`).

### Q-C2 — Is Bodyguard's strategic value combo-relevant or chess-relevant?

**Origin:** Angle 2 (Combat findings, headline finding 5, Q1 in audit's open questions).
**Type:** `design` / `framing`
**Decidability:** `medium` — needs analysis of P3 + future playtest data on *what kinds of decisions* Bodyguard generates.
**Why it matters:** Bodyguard is the most chess-coded sub-system in the game. It works strategically but speaks war-chess vocabulary. If the strategic value comes from *combo-relevant decisions* (positioning Guards to enable skill setups), it's quietly combo-serving despite the framing. If it's *chess-relevant* (defensive screening), it may be reinforcing chassis-first thinking even when working as designed.
**Open sub-questions:**
- In P3, when Bodyguard triggered "way more often" (Elias), what kinds of combo-adjacent decisions emerged from that? (Need to look at game logs.)
- Is there a way to *test* this — e.g., a stack where Bodyguard interacts with skills (not just standard attacks) to see if the combo-relevant version is preferred?
- Could Bodyguard's vocabulary be reframed without touching mechanics? (e.g., a non-chess name)
**Status:** Open. Diagnostic question; might fold into Phase B (naming/identity) work.

### Q-C3 — Is the 10x10 + 12 pieces chassis the minimum-viable host for the combo system?

**Origin:** Angle 1 (initial open questions), Angle 4 placeholder.
**Type:** `design`
**Decidability:** `expensive` — full Angle 4 work, multiple playtests.
**Why it matters:** Specificity guidepost: less chassis = combos louder. If a smaller chassis (8x8, fewer Champions, no Guards as separate pieces) supports the same combo experience, the simplification is pure win for the high concept.
**Status:** Open. The whole Angle 4 work item.
**Connected to:** Backpocket entry "8×10 narrower board variant", OQ-1 (board size).

---

## Theme D — Validation / data needs

Questions that can only be answered by getting more data.

### Q-D1 — Does the high concept land for new players in one game?

**Origin:** Angle 1 verdict (the central finding).
**Type:** `validation`
**Decidability:** `medium` — needs Nico's session (2026-05-28) plus ~3 more first-timer sessions to read a trend.
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

**Data target:** Nico (2026-05-28) + ~3 more first-timers. Rule changes between sessions are *not* frozen — but each session's rule state must be tagged with the data so trend-reading stays honest (already standard; P1/P2/P3 are tagged).

**Bias-correction mechanism — pre-game teacher checklist:** Per `feedback_designer_is_not_data.md`, the analysis is circular if the teacher seeds combo-vocabulary during teaching. The teacher (Elias) fills out `docs/test-scenarios/shared/teacher-vocab-checklist.pdf` *immediately after teaching, before the game starts*, recording which combo-coded words were used during rule explanation. After the game, the checklist is filed alongside the player's onboarding feedback. Analysis logic:
- Player said "combo" + teacher did NOT say "combo" → **strong signal** (emerged).
- Player said "combo" + teacher said "combo" repeatedly → **weak signal at most** (taught).
- Player did NOT use combo-first vocab + teacher avoided it → **real "didn't land" result** (uncontaminated negative).

**Discipline notes (carried forward from Angle 1):**
- Don't lead with the high concept during rule explanation.
- Q11 ("in your own words, what is this game about?") is the primary signal — onboarding form already asks it.
- Q14 (anchoring to other games) is secondary — chassis-coded comparisons (chess, war games) vs. combo-coded comparisons (deckbuilders, combo-card-games) help triangulate.
- Separate Elias's quotes from independent player quotes rigidly (per `feedback_designer_is_not_data.md`).

**Status:** Open. Highest-priority data need. Resolution arrives after Nico + ~3 more first-timer sessions, read against the criteria above.

### Q-D2 — Does game 2 (after the player has seen one combo) shift their language toward combo-first?

**Origin:** Angle 1 verdict (Jonathan recognised exactly one combo in P2 — late, isolated).
**Type:** `validation`
**Decidability:** `medium` — needs a returning first-timer for a second session.
**Why it matters:** The audit's pessimistic reading is "the engine isn't audible until you've heard it." If true, *one* full game might not be enough; the high concept may land on game 2. That changes how we evaluate first-game-only data.
**Status:** Open. Cheaper to answer than Q-D1 but still needs a real second-session playtest.
**Connected to:** Q-B3 (defer drafting to game 2).

### Q-D3 — Does the Multi-Champion Combo Bonus (Stack A Game 2, pending) raise the combo ceiling without crowding out cross-skill-category combos?

**Origin:** Playtest 3 analysis (already in `OPEN_QUESTIONS.md` as OQ-38), surfaced again by Angle 2 audit of Combat.
**Type:** `validation`
**Decidability:** `medium` — Stack A Game 2 playtest.
**Why it matters:** P3 saw Elias do an organic Air Blast → Hook Pull cross-piece combo *without* the bonus active. The pending bonus mechanic only triggers on Strike+Strike sequences from different Champions. Risk: the bonus creates a Strike+Strike monoculture and *crowds out* the cross-category combo behavior that already emerged organically.
**Status:** Open, formally OQ-38. Listed here because the audit reframed its purpose: this isn't just "does the bonus enable combos?" but "does the bonus's narrow scope hurt the combo system that already works?"

---

## Theme E — Skill catalogue / engine content

These ask whether the engine itself has enough content to deliver the high concept.

### Q-E1 — Does the current skill catalogue support enough combo discovery, or is it too narrow?

**Origin:** Angle 1 (initial open questions), Angle 2 (Skill System net assessment).
**Type:** `design`
**Decidability:** `medium` — needs a playtest where draft variety is the explicit watch item.
**Why it matters:** P3 saw Mario draft 2× Armorsmith and never build a combo-shaped loadout. Partly bad draft luck, partly catalogue shape: with 15 skills and only some of them being combo-shaped pairs, it's possible to draft a chassis-only army. If the catalogue is too narrow, the engine has thin content; if it's too wide, drafts have too much variance.
**Open sub-questions:**
- How many *distinct combo shapes* does the catalogue actually support? (e.g., Strike+Strike, Move-into-Strike, Buff-then-Strike, Setup-then-Combo.)
- Would adding 2-3 explicitly combo-shaped skills (e.g., a "next ally piece's skill costs −1 Rune" enabler) widen the engine without breaking balance?
**Status:** Open, parked. Content question; not urgent.
**Connected to:** OQ-12 (skill catalogue expansion, in backpocket).

### Q-E2 — Could skill cards be reshaped to communicate combo-affinity without changing rules?

**Origin:** Angle 2 (Skill System findings, Q6 in audit's open questions).
**Type:** `on-ramp` / `framing`
**Decidability:** `cheap` to design, `medium` to test.
**Why it matters:** The audit found the engine doesn't announce itself as the engine. Card layout is one place where this could change without rule changes — e.g., colored borders for combo-pair groups, an "amplifies / amplified by" line, visual icons that hint at chain structure.
**Status:** Open. Combines well with Q-B2.

---

## Theme F — Phase B / theme & identity

Forward-looking — these will become urgent when Phase B (naming, theme, art) starts.

### Q-F1 — What theme/visual language reinforces "find the elegant solution" instead of "wargame with spells"?

**Origin:** Angle 1 (forward-looking applications, point 4).
**Type:** `framing`
**Decidability:** `expensive` — Phase B itself.
**Why it matters:** The high concept rules out theming that pulls toward war-chess (Battle Chess), narrative campaigns (Gloomhaven), or character-class identity (Summoner Wars). It pulls toward something more like *puzzle-arcane* — visual language that reinforces elegance and sequence-discovery rather than combat and progression.
**Status:** Open, parked. Phase B work.
**Connected to:** `game-identity-visual-naming.md`, Q-A1 (the framing decision shapes which themes fit).

---

## Suggested next-discussion agenda

If/when this becomes a focused design discussion, the natural priority order is:

1. **Q-D1** (does the high concept land for new players?) — the blocking question. Run Nico's session designed to test it (per discipline notes).
2. **Q-A1** (framing decision: combo discovery alone vs. with-opponent-puzzle?) — relatively cheap to decide; unblocks downstream.
3. **Q-B1 + Q-B2 + Q-B4** (on-ramp interventions: starter loadouts, card hints, attack reframing) — the cheapest, highest-info levers to test the *fixability* of the on-ramp gap.
4. **Q-C1** (Armor cap 2 vs. 3) — a real candidate test stack.
5. Everything else as it becomes priority.

This order matches the audit's lever-cost analysis: cheap framing interventions first, then on-ramp tests, then chassis adjustments, with the big chassis-minimisation work (Q-C3) last.
