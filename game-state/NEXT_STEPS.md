# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-05-29 — Session 22 close (Playtest 4 analysed; TESTING_PLAN.typ rewritten; skill sweep done; Stack H re-discussion gate added before rule-sheet drafting).*

---

## Priority 1 — Stack H — Armor Trim (Active)

P4 confirmed OQ-11 / Q-C1: Armor↔Armor-Breaker loop draws attention away from the combo loop. Elias verbatim: *"armor was a part of combo calcs but it just felt like you were not able to do your combos because of it."* Combo bonus did not auto-resolve — Niko's R26-R28 winning loop overran Armor only after a 7-round mid-game Armor cluster.

- [ ] **Re-discuss Stack H — Armor Trim before drafting** (designer flag, Session 22): revisit the bundled-dose framing, scope, and entry conditions in the next session before any rule sheet work. Don't start writing the rule sheet until this conversation has happened.
- [ ] **Write the Stack H rule sheet** (after re-discussion): `docs/test-scenarios/stack-h-armor-trim/` (folder pending). Bundled lead dose: Armor cap 3→2 *and* Armorsmith +1→+2.
- [ ] **Build print packet** for next experienced-player session: Stack H rule sheet + `shared/skill-cards.pdf` ×2 + `stack-h-feedback.pdf` ×2 + `shared/game-tracking.pdf` ×2.
- [ ] **Two experienced players required** — sample size for chassis-volume read needs both players able to plan combos.
- [ ] **Within-stack rollback** (Session 22 restructure): if the bundled dose stalls (Armor build cheaper than break), the next iteration of Stack H runs the smaller dose — cap 3→2 only, Armorsmith unchanged. Same OQ, same hypothesis, smaller variable. (Previously tracked as a separate Stack I — folded into Stack H.)

## Priority 2 — Stack A G3 — Dual-Counter Combo *(Queued; gated on Stack H)*

**Designer decision (Session 22, Path A)**: Stack H ships first. Stack A G3 is queued behind it because incremental testing methodology requires changing one structural variable at a time, and chassis volume is the confirmed P4 problem against which combo scope changes must be evaluated.

Design summary (full writeup in `docs/backpocket.md` → "Combo Bonus — Dual-Counter + Widened Scope"):

- **Target counter** (current rule, kept): different friendly Champions hitting the same enemy target tick the counter; bonus on 2nd+ hit.
- **Attacker counter** (new): same friendly Champion hitting different enemy targets ticks an attacker counter; bonus on 2nd+ hit.
- **Scope widened**: any skill that hits an enemy piece counts (not Strike-only). Standard Attacks excluded.
- **Multi-target skills** (Blade Tempest): tick the counter on every hit piece. Watch flag — first rollback if dual-counter proves OP.
- **Stacking**: intuitive / both-trigger. If a hit qualifies for both counters, both fire. Rare in real play; reward it when it lands.
- **Justification**: solves (a) cross-category crowd-out (#3 P4), (b) late-game offensive lockout when Strike-Champs die (#6 P4 — Elias verbatim "I did not have any other attack champs left"), (c) "exchange pit" mid-game pattern (one cluster, pieces taken one-by-one) by rewarding distributed pressure.
- **Teaching-cost flag (G4 guardrail)**: two parallel counters is strictly more complex than current Stack A G2. Niko named Injured (one counter on one state) as a confusion source. Stack A G3 will likely need physical tokens or board-side trackers. OQ-60 watches whether cognitive load is acceptable.

- [ ] **Stage in backpocket** (DONE — Session 22).
- [ ] **Draft Stack A G3 rule sheet** in `docs/test-scenarios/stack-a-combo-bonus/` after Stack H result lands. Include teaching aids (counter tokens, examples).
- [ ] **Run after Stack H** — only if Stack H trim does NOT auto-resolve the exchange-pit pattern (OQ-58).

## Priority 3 — Other Queued stacks (post-Stack-H)

- [ ] **Stack K — Piece Count Reduction** *(Queued)* — single-variable: 3 Champions + 4 Guards + 1 King on current 10×10 board. *Decoupled from board geometry Session 22 — Stack D owns 8×8 etc. independently.* Trigger: post-Stack-H, two experienced players, full session.
- [ ] **Stack J — Injured Trim** *(Queued)* — gated on Stack H. P4 partially confirmed OQ-57: experienced player barely notices Injured's mechanical effect; new player gets confusion + weak-piece feel. Volume:payoff ratio looks thin.

## Priority 4 — Combo bonus scope follow-up (Session 22 reframe)

**Reframe**: Q3 reward-feel softness (Elias "Somewhat/Neutral", Niko "Bit of both") is design-aligned, not a problem — the bonus is by design a few-times-a-game payoff, not "do or lose." The lever is **scope, not strength**. Cross-category crowd-out (#3) and late-game offensive lockout (#6) are the actual problems.

- [ ] **Stack A G3 dual-counter** (see Priority 2) is the staged solution — gated behind Stack H.
- [ ] **Re-evaluate after Stack H lands**: chassis trim may shift exchange-pit dynamics enough that dual-counter complexity isn't justified.
- [ ] **OQ-58 (exchange-pit) watch** under Stack H — if mid-game stickiness persists post-trim, dual-counter is the targeted fix.

## Priority 5 — Skill Balance carry-over watches

- [ ] **Rune Theft Mode B confirmed dominant (P4)** — both players pinned it as must-pick / favourite-moment. Hold cost increase until after Stack H. If combat speeds up post-trim, Mode-B value drops naturally.
- [ ] Confirm **Shadow Shift** Range 2 feels right in play (no P4 data — Shift used once by Niko R20, no signal).
- [ ] Monitor **Blade Call** + combo bonus interaction (Elias never used Blade Call despite naming it must-pick; Niko used in winning loop).

---

## Backlog (no priority — pull when triggered)

**Process / facilitation (deferred):**
- **Teacher-vocab-checklist enforcement** — DEFERRED (Session 22 designer call). P4 Q-D1 contamination acknowledged but reading not re-attempted. Lower-priority lever: simplify skill names ("Armorsmith", "Lance Thrust" etc. were shortened or replaced with natural words at the table — vocabulary barrier could be reduced by renaming, not just by stricter teaching). Bundle with Phase B naming pass.
- **"Runes" rename candidate** — staged in backpocket; Phase B item.

**Pre-playtest polish (deferred from Session 18):**
- One-page player-facing intro / pitch (#2 from Session 18 list). Open question: does the intro replace `section-introduction()` in the rule sheet or sit alongside it? Revisit after next first-timer session lands.
- Rule sheet ordering audit (#3 from Session 18 list). Deferred.
- ADR on tiered skill catalogue (#5 from Session 18 list). Important but not time-bound.

**Dormant stacks (waiting on triggers):**
- **Stack C — Pacing**: King Lifetime HP, Armor Decay — trigger: first Champion kill past R20. P4 R13 → not triggered.
- **Stack D — Board Geometry**: 8×10 (OQ-52), 8×8 (OQ-1b — moved here from Stack K Session 22), hex (gated on `/research hex vs square grid`, OQ-42).
- **Stack E — Draft Flow**: pool draft (OQ-35), placement order (OQ-36+48) — trigger: after Stack A G3 lands.
- **Stack F — Sente Skills**: cascade trigger, Pin/Threatened, midline pressure — *trigger: only if Stack A G3 ran and exchange-pit pattern persists.* Sequenced after A G3 per Session 22 designer call (different mechanism for the same problem).
- **Stack G — Unified AP**: draft written, run after core stacks stabilise (post H + A G3 + J + K).
- **OQ-52 / OQ-53 dedicated stack** — may become its own stack rather than living in F.

**Withdrawn stacks (Session 22):**
- **Stack B — Bodyguard Fix** (defender-only adjacency). Withdrawn because P4 confirmed Bodyguard tracks standoff state, not the rule. Different solutions would be on the table even if Bodyguard remains broken post-Stack-H. OQ-21 stays open; the originally proposed stack does not.
- **Stack I — Armor Rollback**. Folded into Stack H as the smaller within-stack dose.

**P4 design ideas surfaced (Session 22 — staged in `docs/backpocket.md` with Justification Rule writeups):**
- **Combo Bonus Dual-Counter + Widened Scope** (Stack A G3) — staged. Priority 2 above.
- **Plague skill** (Mystic, Range 2, ~3 Runes, inflicts Injured ignoring Armor, no kill) — fixes "Injured-state-as-payload that bypasses Armor stack."
- **Lucky Strike / Star Strike** (Mystic, target anywhere on board) — justification TBD; staged for further design pass.
- **Focus replacement** ("any skill +1 Rune for +1 Range" baseline rule, removing the Focus skill) — fixes catalogue must-pick density.
- **Lance Thrust + Rune Theft merge** (single skill: 1 DMG + optional Rune steal) — fixes catalogue redundancy and per-loadout pressure.
- **"Runes" rename** — vocab/barrier-of-entry pass; Phase B bundle.
- Elias: **paid Focus extension** (+1 Rune to widen activation/effect range). Justification open.
- Elias: **"rusty thief" piece-design idea**. No justification yet.
- Elias: **small mini pre-game for new players**. Connects to OQ-56 Problem A.

**New OQs opened in Session 22 (full text in `game-state/OPEN_QUESTIONS.md`):**
- **OQ-58 — Mid-game stickiness / "exchange pit"**: once an exchange starts, all action concentrates in one cluster, pieces taken one-by-one. Watched under Stack H.
- **OQ-59 — Opening + endgame "don't know what to do" pattern** (sub-problems 59a opening chassis-skew = no Strike skills firing in opening, only Defense; 59b endgame conversion gap after first big mid-game exchange).
- **OQ-60 — Cognitive load**: real concern or acceptable? G4 guardrail watch — informs how complex Stack A G3 dual-counter can afford to be.

**Research / brainstorm:**
- `/research` on hex vs. square grid (OQ-42) — before any hex stack proposed.
- `/research` on how board games track temporary effects on pieces.
- Brainstorm: Adjacency synergies / piece compatibility — connects OQ-51.

**Catalogue / system expansion:**
- Skill catalogue expansion — 10 new candidates staged in `docs/backpocket.md`. Target ~25 total. Gated on Stack H combat balance confirmation.
- First-player advantage mitigation (OQ-13b watch — P4 added one P1-win + Niko's own E16 flag, still 1-of-1).
- In-game skill redraft (shop/auction/interval) — connects OQ-56 Problem B.

**Tooling improvements:**
- Improve `/playtest` skill: add draft-pick extraction and analysis as standard step.

---

## Test-Scenario UX Improvements (open since Session 13)

*Template/architecture changes for the next time rule sheets or feedback forms are rebuilt.*

- [ ] **Separate .typ source from PDF output** — move all `.typ` files into a `src/` subfolder within each stack directory.
- [ ] **Rules PDFs read as "intended rules"** — move all meta-information to a detachable facilitator page at the front.
- [ ] **Feedback forms fully independent** — no cross-game references.
- [x] **More physical writing space** — feedback forms converted to `#v(1fr)` distribution (Session 18).

---

## Recently completed (Session 22)

- Playtest 4 analysed end-to-end: 11 transcription files, `docs/research/playtest-4-analysis.md` synthesis, OQ verdicts cascaded to OPEN_QUESTIONS.md, mechanics-evaluated.md updated.
- "Nico" → "Niko" project-wide rename (44 occurrences across README, STATUS, OPEN_QUESTIONS, NEXT_STEPS, SESSION_LOG, HANDOVER, research docs, memory).
- OQ-11 confirmed via P4 evidence; Stack H promoted to Priority 1.
- OQ-21 confirmed as covariate (tracks standoff state, not the rule).
- OQ-19 not triggered (first Champion kill R13). OQ-41 closed (length problem is chassis volume, not damage nerf).
- Q-D1 reading downgraded to contaminated; teacher-vocab-checklist process fix DEFERRED per designer (lower-priority natural-naming lever instead).
- **Post-analysis design discussion (Session 22)**:
  - **OQ-38 reframed**: combo bonus Q3 softness is design-aligned (few-times-a-game payoff, not "do or lose"). Lever is **scope, not strength**.
  - **Dual-counter combo design** drafted: target counter + attacker counter, both stack, multi-target skills tick all hits, Standard Attacks excluded. Staged as Stack A G3.
  - **Path A methodology decision**: Stack H first, Stack A G3 after (one structural variable per stack).
  - **3 new OQs opened**: OQ-58 (exchange-pit), OQ-59 (opening+endgame don't-know-what-to-do), OQ-60 (cognitive load watch).
  - **6 new backpocket entries**: Combo Bonus Dual-Counter (Stack A G3), Plague skill, Lucky/Star Strike, Focus replacement, Lance/Rune-Theft merge, Runes rename.
  - **#6 reframe** (Elias's late-game): not behavioural-choice but structural lockout — Strike-equipped Champs were dead. Justifies attacker counter.
  - **Must-pick density softer** than initial read: Focus 1/2 across army, Armor 2/3 across army (not per-Champion). Catalogue pressure is per-loadout, not per-piece.
- **TESTING_PLAN.typ rewritten** (Session 22 second pass):
  - Stacks renamed for legibility: H = Armor Trim, A G3 = Dual-Counter Combo, K = Piece Count Reduction, F = Sente Skills, etc. Letter IDs preserved as stable cross-reference keys.
  - **Stack I dropped** (Armor rollback) — folded into Stack H as the smaller dose. Same OQ, same hypothesis.
  - **Stack B withdrawn** (defender-only adjacency) — P4 confirmed Bodyguard tracks standoff state, not the rule. Different solutions would be on the table; the originally drafted stack is not the right fix.
  - **Stack K decoupled from Stack D** — K owns Piece Count Reduction (3+3+1); D owns Board Geometry (8×10, 8×8, hex). Independent variables.
  - **Stack F sequenced after Stack A G3** — both target the exchange-pit pattern via different mechanisms; A G3 first, F only if A G3 doesn't dissolve it.
  - State lifecycle introduced: Active / Queued / Dormant / Resolved.
  - Phase 1 / Phase 2 decision tree replaced with per-stack routing rules.

## Recently completed (Session 21)

- All 3 feedback forms (`feedback-onboarding.typ`, `stack-a-feedback.typ`, `feedback-baseline.typ`) audited and updated for high-concept alignment.
- New questions: aha discovery moment, chassis-vs-engine confusion split, Framing B parallel-puzzle, Bodyguard chess-vs-combo distinction.
- WHAT_TO_PRINT.md added; README sanitised; Section G in prototype feedback form brought to parity with paper.

## Recently completed (Session 20)

- All 11 high-concept open questions (Q-A1 → Q-F1) resolved with discussion-then-decision per question.
- ADR-004 written and accepted: "Two minds, one puzzle" (Framing B) becomes canonical design intent.
- New `§ High-Concept Framing` and `§ Chassis and Engine` sections in `design-principles.md`.
- Q-B4 baseline change shipped: Standard Attack reframed as "a Move that ends on an enemy tile"; survival-stop strengthened. BASELINE_VERSION → 2026-05-26.
- Q-D1 resolution criteria locked in (≥2/4 strong-signal threshold + teacher-vocab-checklist as bias correction).
- `shared/teacher-vocab-checklist.typ/pdf` shipped.
- Stacks H (Armor bundle), I (Armor rollback), J (Injured-downsides removal), K (chassis-minimisation session) queued.
- OQ-11 reopened (chassis-volume framing); OQ-57 added; OQ-21, OQ-27, OQ-1b, OQ-12, OQ-38 cross-linked.

## Recently completed (Session 19)

- Digital prototype PWA built and deployed to GitHub Pages (`prototype/index.html`).
- Full game loop: 10×10 board, drag-and-drop, piece state (armor/injured/skill icons), rune tracking, end-turn notes, post-game feedback form, JSON export.
- All 15 skill icons base64-embedded — fully offline once cached.
- iOS touch rewritten: Pointer Events API, combined distance+time threshold, `setPointerCapture`, `requestAnimationFrame` for iOS 17.4 compatibility. Confirmed working on iPad.

## Recently completed (Session 18)

- Physical skill cards shipped (`shared/skill-cards.pdf`): 15 cards on A4, 2×2 range matrix per card showing Default / +Focus / Injured / Inj.+Focus, per-skill Focus footnotes on Move cards.
- First-game onboarding feedback form shipped (`shared/feedback-onboarding.pdf`).
- **Ruling**: Focus Strike on Move skills — caster chooses activation OR effect range (not both). Documented in baseline-sections, skill cards, mechanics-evaluated.
- Lance Thrust effective Range 0 while Injured = cannot fire (derivation, not ambiguity). Memory written.
- Stack A + Stack B feedback forms + feedback-baseline converted from fixed `#v(2.7cm)` to `#v(1fr)`.
- Build script gained zsh guard.
- Hygiene principle 7 expanded with `1fr` over `#v(Ncm)` sub-rule.
- Decision logged: Niko plays standard baseline draft on 2026-05-28; onboarding data via the new form only.
