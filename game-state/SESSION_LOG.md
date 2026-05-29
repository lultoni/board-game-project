# Session Log

*Per-session narrative log. Newest sessions at the top. `/wrapup` adds a new entry per session.*

*This used to live in `old-game-versions/README.md`. Moved to `game-state/` in Session 17 because it is active state, not archive material.*

---

### May 29, 2026 — Session 22: Playtest 4 — Stack A G2 + Niko's first game

Niko (P1, first-time player) beat Elias (P2) on 2026-05-28 in a 28-29 round Stack A G2 game (~2h30). Elias surrendered on T29 after Niko's third consecutive Strike+Strike kill round.

Project-wide rename **Nico → Niko** (44 occurrences) executed first via sed: README, STATUS, OPEN_QUESTIONS, NEXT_STEPS, SESSION_LOG, HANDOVER, research docs, plus memory file and pointer in MEMORY.md. User finalised image transcriptions across 11 files in `playtest-results/elias-vs-niko-28_05_26/`. `/playtest` skill executed end-to-end with multi-agent isolation: two independent agents transcribed each player's materials separately to prevent cross-contamination of behavioural reads.

**Synthesis written**: `docs/research/playtest-4-analysis.md`. Combo bonus (OQ-38) **confirmed in mechanics**: Elias R11 margin "1. ever combo!!" (his first multi-Champion combo across all 4 playtests); Niko's R26-R28 = textbook 3-turn Strike+Strike kill loop (Blade Tempest + Rune Theft + Focus). **Weak in feel**: neither player rated the bonus "Very rewarding" — read it as a normal damage modifier. Cross-category crowd-out (Q-D3 risk) **partially confirmed** — Elias circled Rarely AND Never, Niko circled Sometimes. Bonus stays into baseline pending one more experienced-player game.

**Mid-game stalling returned despite Stack A nerf** — both players ran identical Armor-stack arcs (R15-R21 Niko / R15-R18 Elias), no Atk during the cluster. Elias D-notes: *"a lot of turns where we both didn't really know what to do."* OQ-11 / Q-C1 chassis-volume hypothesis received its **best evidence yet**: Elias Q13 "Yes, a lot" mental focus + game "Slowed noticeably" + verbatim *"armor was a part of combo calcs but it just felt like you were not able to do your combos because of it."* Niko's split read (Q13 "Not really" + "Slightly extended") suggests cost is asymmetric across skill levels.

**Stack H promoted to Priority 1.** Decision-tree literal output was Stack F (standoff persists), but the standoff observed is *Armor-driven mid-game stalling*, not opening-engagement standoff that Stack F's sente skills target. Stack H's pre-condition test ("does combo bonus auto-resolve Q-C1?") returned NO — Niko's combos overran Armor only after a 7-round stall. Chassis volume is the lever.

**Q-D1 reading contaminated.** Elias did not honour the teacher-vocab-checklist commitment ("I just used the words to make it clear what the game is about"; pitch box: "explained a lot of my experience and good combos"). Niko's engine vocabulary ("combinations", "skill combos", "stacking of skills") may be borrowed. The ≥2/4 strong-signal threshold is NOT met by this session alone. **Process fix required before next first-timer session**: either pre-game initials per word OR read-rules-from-document protocol. Tracked in `NEXT_STEPS.md` as Priority 2.

Other findings: Niko's Q15 favourite moment — *"killing powerful champion by surprise and also stealing runes to prevent skills next turn"* — is exact cleverness-over-attrition language, ADR-003 Principle 4 landing for a first-time player. OQ-21 Bodyguard regressed to 0 triggers (covariate with stalling, not independent signal). OQ-19 not triggered (first Champion kill R13 < R20). OQ-41 closes — length problem is chassis volume, not damage nerf. OQ-34 Mode B confirmed dominant.

Cascade updates: OPEN_QUESTIONS.md (8 OQ verdicts), NEXT_STEPS.md (Stack H Priority 1, Q-D1 process fix Priority 2), mechanics-evaluated.md (combo bonus + Stack H evidence pointers), systems-and-mechanics.md (P4 evidence on Combat / Skill / Health-Armor sections), STATUS.md (re-entry update).

**Post-analysis design discussion (Session 22 second half).** Designer pushed back on the initial synthesis: combo-bonus Q3 softness ("Somewhat/Neutral", "Bit of both") is **design-aligned**, not a problem — the bonus is by design a few-times-a-game payoff, not "do or lose." The lever is **scope, not strength**. Walked through every analysis finding for designer reaction; multiple reframes landed:

- **#6 reframed**: Elias's late-game Blade Tempest spam was not a behavioural choice between "use the bonus loop" vs "single-Champ burst" — his Strike-equipped Champions were dead. *"I did not have any other attack champs left."* Structural offensive lockout, not preference.
- **Must-pick density softer than initial read**: Focus is equipped 1/2 across the army (not per-Champion); Armor 2/3 across the army. The catalogue pressure is per-loadout, not per-piece. Initial framing was overstated.
- **Q-D1 teacher-vocab-checklist deferred**, not made Priority 2. Lower-priority lever surfaced: skill names were shortened or replaced with natural words at the table — vocabulary barrier could be reduced by **renaming**, not just by stricter teaching. Bundled with Phase B naming pass.
- **"Don't know what to do" pattern decomposed** into two distinct windows: (a) early-game positioning (no Strike skills firing in opening — only Defense), and (b) post-mid-game-exchange endgame (after first big exchange, neither player knows how to convert position into win). Two sub-problems, not one.

**Dual-counter combo design drafted.** Replaces the simpler scope-widening (Option A, Move-into-Strike inclusion) with a richer mechanic: two parallel counters per turn — **target counter** (different friendly Champions hitting the same enemy target, current rule kept) plus **attacker counter** (same friendly Champion hitting different enemy targets). Both counters live in parallel; if a hit qualifies for both, both fire (intuitive stacking — rare in real play, reward when it lands). Multi-target skills (Blade Tempest) tick the counter on every hit piece, with a watch flag for OP rollback. Standard Attacks excluded from both counters. Scope widened to any skill that hits an enemy piece (not Strike-only). Justifications: cross-category crowd-out (#3), late-game offensive lockout (#6), and the **exchange-pit** mid-game pattern (one cluster, pieces taken one-by-one — attacker counter rewards distributing pressure across multiple fronts).

**Path A methodology decision.** Stack H ships first. Stack A G3 (dual-counter combo + widened scope) is queued behind it. Reason: incremental testing methodology requires changing one structural variable at a time. Combo scope and Armor volume both affect mid-game pacing; bundling them blurs the read. If Stack H trims Armor and the exchange-pit pattern persists, dual-counter is the targeted fix. If Stack H accidentally fixes the exchange-pit pattern (chassis volume was masking it), dual-counter may not be needed in this form. Designer agreed with the methodology argument: *"yes i agree that it is very complex, especially at the current game size so i agree we first lower the complexity of the game and then think about this."*

**Three new OQs opened (full text in `game-state/OPEN_QUESTIONS.md`):**
- **OQ-58 — Mid-game stickiness / "exchange pit"**: once an exchange starts, all action concentrates in one cluster, pieces taken one-by-one. Watched under Stack H.
- **OQ-59 — Opening + endgame "don't know what to do" pattern**: 59a opening chassis-skew (no Strike skills firing in opening), 59b endgame conversion gap.
- **OQ-60 — Cognitive load**: real concern or acceptable? G4 guardrail watch — informs how complex Stack A G3 dual-counter can afford to be.

**Six new backpocket entries** (each with Justification Rule writeup): Combo Bonus Dual-Counter (Stack A G3), Plague skill (Mystic, Range 2, ~3 Runes, inflicts Injured ignoring Armor, no kill), Lucky/Star Strike (Mystic, target anywhere on board), Focus replacement ("any skill +1 Rune for +1 Range" baseline rule, removing the Focus skill), Lance Thrust + Rune Theft merge (single skill: 1 DMG + optional Rune steal), "Runes" rename candidate (Phase B bundle).

Cascade updates (second half): OPEN_QUESTIONS.md (OQ-38 reframe, OQ-11 designer note re Armor Breaker died early, OQ-12 must-pick softer note, OQ-58/59/60 added), backpocket.md (6 new entries), NEXT_STEPS.md (Priority 2 → Stack A G3 dual-counter gated on Stack H, teacher-vocab-checklist moved to deferred backlog, Recently Completed expanded with discussion outputs).

**TESTING_PLAN.typ rewritten (Session 22 third pass).** Designer flagged the doc's structure as confusing: ordering signals contradicted, gate logic scattered across three sections, Stack A G2 status stale, post-P4 routing missing, Stack A G3 unrepresented. Rewrite outcomes: stacks renamed for legibility (H = Armor Trim, A G3 = Dual-Counter Combo, K = Piece Count Reduction, F = Sente Skills) with letter IDs preserved as stable cross-reference keys; **Stack I dropped** (folded into Stack H as the smaller within-stack dose — rollback is contingency, not a separate stack); **Stack B withdrawn** (defender-only adjacency — P4 confirmed Bodyguard tracks standoff state, different solutions on the table even if Bodyguard remains broken post-Stack-H); **Stack K decoupled from Stack D** (K owns piece-count reduction; D owns board geometry — independent variables); **Stack F sequenced after Stack A G3** (both target the exchange-pit pattern via different mechanisms); state lifecycle introduced (Active / Queued / Dormant / Resolved — exactly one Active at a time); Phase 1/Phase 2 decision tree replaced with per-stack *Routing on result* blocks. Cascaded to OPEN_QUESTIONS.md (OQ-11 / OQ-21 / OQ-27 / OQ-1b cross-refs), mechanics-evaluated.md (Bodyguard adjacency moved to Withdrawn, Stack I rollback note collapsed, board-8x8 deferred row updated), NEXT_STEPS.md and STATUS.md (new stack names + dormant/withdrawn lists).

**Skill sweep.** Updated `/playtest` Step 5 (Decision Tree Routing → "Routing — Which Stack Next?", Active/Queued/Dormant/Resolved structure, per-stack routing rules); `/scenario` Step 5 (TESTING_PLAN section now requires placing the new stack in the appropriate state section with the per-stack required fields); `/wrapup` Step 2 TESTING_PLAN block (decision tree tables → state lifecycle moves + per-stack *Routing on result* updates). `/research` and `/adr` verified clean (no stack vocabulary).

**Stack H re-discussion gate.** Designer flagged Stack H bundled-dose framing for re-discussion before any rule sheet work begins. NEXT_STEPS.md Priority 1 and STATUS.md Next Action both now lead with the re-discussion item; rule sheet drafting is explicitly downstream of that conversation.

---

### May 26, 2026 — Session 21: Feedback Forms — High-Concept Alignment

Audited all three feedback forms (`feedback-onboarding.typ`, `stack-a-feedback.typ`, `feedback-baseline.typ`) against the high-concept investigation findings (Sessions 19–20). Identified gaps: no Q-D1 combo-discovery signal questions, no Framing B parallel-puzzle coverage, stale question framings (Armor only asked about pacing, not attentional cost; Bodyguard asked frequency not chess-vs-combo type; draft asked "did you have a plan" not "were pairings in mind"), no chassis-vs-engine confusion distinction.

`feedback-onboarding.typ` fully restructured: new Q6 (aha discovery moment), Q9/Q10 split into chassis-confusion vs engine-confusion, Q12 expanded with "describe what happened", new Q14 (Framing B), facilitator reminder to staple teacher-vocab-checklist. `stack-a-feedback.typ`: 5 targeted fixes (combo planned/coincidence, draft pairing framing, Bodyguard chess/combo distinction, Armor attentional cost second line, new Framing B question). `feedback-baseline.typ` template updated — changes propagate to all future stacks H/I/J/K. Prototype feedback form (Section G + Sections C/D/E) brought to parity with paper forms; `notice` render type added for section headers; Section G notice instructs both players to fill together.

`WHAT_TO_PRINT.md` added to repo root (print checklist for all game scenarios). `README.md` sanitised: live state removed, project structure tree corrected, prototype/ and all new shared/ files added.

---

### May 26, 2026 — Session 20: High-Concept Open Questions Sweep — All 11 Resolved

Worked through every question in `docs/research/high-concept-open-questions.md` (Q-A1 through Q-F1) with discussion-then-decision per question. ADR-004 written and accepted: high-concept framing is **"Two minds, one puzzle" (Framing B)** — 2-player nature is load-bearing, opponent is a fellow puzzle-solver, asymmetry biased against. New `§ High-Concept Framing` and `§ Chassis and Engine` sections added to `design-principles.md`; chassis/engine becomes canonical project vocabulary as a companion lens to the Justification Rule.

Q-B4 executed in baseline: Standard Attack reworded as *"a Move that ends on an enemy tile"*; Movement Phase intro and survival-stop rule strengthened with explicit attacker-speed cases. BASELINE_VERSION bumped to 2026-05-26. Q-B1 design intent set (single shared loadout for both players in game 1, gated on Niko's data); Q-B2 (combo hints on cards) rejected to preserve emergent discovery; Q-B3 resolved as entailment from Q-B1.

Q-D1 resolution criteria locked in (signal definitions + ≥2/4 strong-signal threshold + teacher-vocab-checklist as bias correction); `shared/teacher-vocab-checklist.typ/pdf` shipped. ADR-004's reversal criterion updated via Q-D2 to read against combined Q-D1 + Q-D2 result.

Five new test artefacts queued in `TESTING_PLAN.typ`: **Stack H** (Armor cap 3→2 + Armorsmith +1→+2 bundled, gated on Stack A G2), **Stack I** (Armor rollback if H stalls), **Stack J** (Injured downsides removal, gated on G2 + H), **Stack K** (two-game chassis-minimisation session: 8×8 then 8×8 + 3+3+1 pieces). OQ-11 reopened from archive under chassis-volume framing; OQ-57 (Injured chassis volume) added; OQ-21, OQ-27, OQ-1b, OQ-12, OQ-38, OQ-39 cross-linked to the new resolutions. Q-C2 finding logged: Bodyguard is chess-coded behaviorally (Mario's P3 usage = trade-up screening), Framing-B watch-flag added.

---

### May 25, 2026 — Session 19: Digital Prototype PWA + iOS Touch Fix

Built and deployed a single-file offline PWA prototype at GitHub Pages (repo made public). The prototype covers the full game loop: 10×10 board with drag-and-drop piece movement, piece state tracking (armor pips, injured dot, 2 skill slots with icons), rune tracking per player, end-turn notes panel, post-game feedback form (multi-select scale questions + notes), and JSON export per game. All 15 skill icons are base64-embedded so the file runs fully offline once cached. Board and all piece content scale dynamically via `--cell` CSS custom property.

iOS touch handling was broken (instant drag on any touch, modal unreachable). Rewrote from Touch Events API to Pointer Events API: event delegation on `#board` via `pointerdown`, `setPointerCapture` for reliable routing, combined 10px distance + 100ms time threshold before committing to drag, `requestAnimationFrame` wrapping for iOS 17.4 DOM repaint compatibility, and `elementFromPoint` with `pointer-events:none` on the drag clone for accurate hit-testing. User confirmed it works on iPad.

---

### May 24, 2026 — Session 18: Pre-Playtest Tooling — Skill Cards, Onboarding Form, Focus+Move Ruling

Two artifacts shipped under `docs/test-scenarios/shared/` ahead of the 2026-05-28 playtest with Niko (first-time player). `skill-cards.typ/pdf` is a printable A4 sheet of 15 physical reference cards — one per skill, color-coded by category, with a 2×2 range matrix per card showing reach in four states (Default / +Focus / Injured / Inj.+Focus) and per-skill Focus footnotes on Move cards. Resolves OQ-56 problem C1 (in-game lookup friction). `feedback-onboarding.typ/pdf` is a 2-page first-game-only feedback form covering rules absorption, draft thinking, in-game confusion, and player anchoring — independent of stack, fills before the standard stack form.

Major ruling: Focus Strike on Move skills lets the caster choose, at activation, whether the +1 applies to activation range OR effect range (not both). Documented in `baseline-sections.typ` (Skill Reference + Quick Reference), on every Move skill card, and in `mechanics-evaluated.md`. Resolves the ambiguity that surfaced while building the cards.

Lance Thrust effective Range 0 while Injured ruled "cannot fire" — the chained derivation from baseline rules IS the ruling, not ambiguity to be resolved. User correction: never reframe a derivation as "let's make a ruling — it already is one." Memory `feedback_baseline_is_authoritative.md` written. OQ-54 (Lance Thrust wording rewrite) closed: keep "Range−1" — the modifier preserves a real interaction the "Adjacent" rewrite would obscure.

Decision logged for 2026-05-28: Niko plays the standard baseline draft. No starter loadout, no simplified pool, no rule changes for first-time players. Onboarding data comes from the new form, not from changing the game. Memory `project_niko_first_game.md`.

Forms cleanup: Stack A, Stack B, and feedback-baseline forms converted from fixed `#v(2.7cm)` to `#v(1fr)` distribution (eliminates dead-zone empty pages). Build script gained zsh guard. Hygiene principle 7 expanded with the `1fr`-over-`#v(Ncm)` sub-rule. BASELINE_VERSION bumped to 2026-05-24.

Deferred from end-of-session: one-page player-facing intro, rule sheet ordering audit, tiered-catalogue ADR. Backlogged in NEXT_STEPS until Niko's onboarding feedback lands and informs reorder priorities.

---

### May 20, 2026 — Session 17: Repo Rework — Skills, State Lifecycle, Single Source of Truth, Hygiene Principles

Five-agent architecture audit ran first to surface drift and friction across documentation, test-scenario pipeline, skill workflow, state-doc lifecycle, and repo hygiene. Findings were consolidated into a single rework plan executed in one pass.

Skill workflow repair: ghost references to `CURRENT_DESIGN.md`, `docs/decisions/`, and `docs/brainstorm/session-log.md` repointed across `/research`, `/scenario`, `/adr`, `/playtest`, `/build-pdfs`. The layer→stack vocabulary rename (incomplete since Session 9) was finished in the `/scenario` skill, `/build-pdfs` skill, and skill description text.

State-doc lifecycle: `OPEN_QUESTIONS.md` split into a live file (open / inconclusive / deferred only, sorted by status) and `OPEN_QUESTIONS_ARCHIVE.md` for resolved / closed / scrapped / parked entries. Partially-resolved items kept in live as residual entries (OQ-1b, OQ-3b, OQ-13b, OQ-20b) pointing back to the archive for full history. `STATUS.md` introduced as a one-screen re-entry doc. `mechanics-evaluated.md` schema extended with Source OQ + Evidence columns. Session log moved out of `old-game-versions/README.md` into `game-state/SESSION_LOG.md`.

Single source of truth for rule numbers: `BASELINE_VERSION` constant added to `baseline-sections.typ`. Numeric tables in `CLAUDE.md` "Key Game Systems" and `docs/systems-and-mechanics.md` trimmed to pointers into the canonical Typst baseline.

Test-scenario pipeline parameterization: `section-quick-reference()` accepts `overrides:` for changed rows; existing stack files migrated to use it. `feedback-baseline.typ` converted to importable functions with auto-numbered `#fq[]`. `build-pdfs.sh` rewritten to discover `.typ` files via `find` instead of a hardcoded list.

Repo hygiene: `.DS_Store` sweep; PDF commit policy documented; `playtest-results/README.md` records the photo-size policy decision; `images/README.md` added; `balde_call.jpg` typo fixed.

Hygiene principles section added to `CLAUDE.md` so future sessions avoid the same drift patterns: pointers not restatements, lifecycle for state docs, cross-link by ID, atomic vocabulary renames, real skill paths, immutable memory, parameterization over copy-paste, discovery-based build scripts, archive-before-piling, CLAUDE.md is for orientation not facts.

Memory cleanup: stale memory entries (Session 3 rulings, Playtest 1 single-event) reframed as immutable historical claims. MEMORY.md index refreshed.

---

### May 19, 2026 — Session 16: Pre-Stack-A-G2 Prep Complete — Stack A G2 Ready to Print

Full housekeeping session before Playtest 4. Rule clarifications accepted into baseline: Range system refined (all skills default to Range 2 unless text explicitly names "self"/"adjacent"; Range modifiers apply from default; self/adjacent targeting cannot be shifted inward by buffs even with Focus Strike); Focus Strike note added. Tracking sheet redesigned with pre-filled `R+` and `SS` columns (scaling values shown on change rounds, `|` otherwise) plus new `Atk` column; cost column dropped. Rule document restructured with new Introduction + Simple Overview pages and dependency-correct section order (Skill Drafting early; Bodyguard after Standard Attack; Progression next to Economy; Health/Armor last). Designer-box style added to stack files so facilitator notes fade into background for players. OQ-54 (Lance Thrust wording) and OQ-55 (Blade Call broader interaction) logged. Combo-bonus scope decided: Strike-only for Game 2, cross-category reconsidered after data. Stack A G2 rule sheets updated and ready to print.

---

### May 18, 2026 — Session 15: Playtest 3 Analysed — Stack A G1 Confirmed

Transcribed and analysed Playtest 3 (Elias vs Mario 17.05.26, Stack A Game 1: standard attack nerfed to 1 DMG). Key findings: standoff dissolved (first Champion kill R11 vs P2's R26), Bodyguard activated organically without the Stack B adjacency fix (may obsolete Stack B), Armor / Armor-Breaker RPS loop functioned as designed, and a cross-piece Move-into-Strike combo (Air Blast → Hook Pull) emerged organically without the combo-bonus layer. **Standard attack 1 DMG accepted into baseline.** Two new design questions raised as OQ-52 (centre-of-board has no attractor — flank-drift, cramped opening, King stays back) and OQ-53 (attrition vs regicide — King is currently incidental, not a target). Reframed Rune Theft (OQ-34) into Mode A (opponent at 0 Runes = normal Strike) vs Mode B (cheap damage + disable, time-dependent); P3 burst was Mode A, not dominance. Four new backpocket entries: 8×10 narrower board, starting-formation swap to expose King, "spec the game for a programmer" exercise (with note to research requirements engineering first), and digital playtest prototype (sleep-on-it, ADR required). NEXT_STEPS restructured with Session 16 priorities: rule clarifications (Lance Thrust + Injured, Focus Strike + adjacent self), form fixes (standard-attack count, bake-in Rune scaling), OQ-52/53 brainstorm, combo-bonus scope decision before Stack A G2.

---

### May 17, 2026 — Session 14: Idea Capture & Justification Rule

Captured three new design ideas from a fresh post-playtest conversation (Elias vs Mario 17.05.26): Guard passive-buff drafts, mid-game inflection events, and private-draft-plus-trade. Amended `design-principles.md` to clarify that strategy-specific economy is acceptable if multiple paths exist and are balanced, and to add nuance that the 4-axis cognitive-load model is aspirational (experienced play) not descriptive (early experience). Added a new mandatory **Justification Rule** to `CLAUDE.md`: every new idea must answer "what current problem / uncoolheit does this fix, OR what specific aspect of game feel does this improve?" — variety alone is not justification. Logged four test-scenario UX issues (separate src/PDF folders, facilitator-page pattern, independent feedback forms, more writing space) to NEXT_STEPS.

---

### May 2026 — Session 13: Repository Consolidation

Restructured the entire repository. Eliminated overlapping documentation: ADRs dissolved (principles extracted into `design-principles.md`, history into the timeline), 7 per-system files merged into one `systems-and-mechanics.md`, session-log absorbed into the timeline. Result: each document has one clear job, no redundancy.

---

### April 29 — Sessions 10–12: Research Loaded, Ready to Play

**Session 10**: All old xlsx/pptx files converted. 21 ideas extracted from old versions, triaged into 80-item assessment. Designer confirmed: Champions must remain blank slates. Design Guardrails G1–G7 established. `design-language.md` created for future visual identity phase.

**Session 11**: Four Perplexity research threads completed (clever-play levers, checkmate win conditions, forward positioning, skill catalogue balance). Checkmate win condition killed (verification burden impossible with ranged + heals + armor). Sente skill design chosen as primary standoff solution. Ten new skill candidates staged. G8 (Spending Tension) guardrail established.

**Session 12**: TESTING_PLAN audited and fixed (stale since Session 9). Decision tree rebuilt as tables. Three new backpocket ideas. All 12 PDFs rebuilding cleanly.

---

### April 28 — Sessions 8–9: Infrastructure Complete

**Session 8**: All 7 per-system design docs populated. All ideas from old "Systems to Test" document triaged. Hex grid reopened (original rejection was by omission, not evaluation). Performance-based Rune gain permanently closed (forces single playstyle). 3rd skill slot closed by principle (2 slots forces specialist builds).

**Session 9**: Linear test layers replaced with dynamic stack system. Composable Typst section functions built. All rule sheets refactored. `TESTING_PLAN.pdf` created. Eight stacks defined (A through G). `/scenario` and `/playtest` skills rewritten.

---

### April 27 — Session 7: Cleverness vs Attrition (The Turning Point)

A full system-by-system audit revealed: the game consistently rewards attrition over clever play. Standard attack: 2 DMG, 0 Runes, infinite efficiency. Best skill combo in the game: 2 DMG, 6 Runes, 3 Skill Slots. The "best combo" merely matches what a free attack does.

Research on cooperative feel in competitive games found "mutual epistemic exploration" — both players co-interpreting the same puzzle. This was already happening naturally in Playtest 2.

Five design principles established (now in `docs/design-principles.md`). Layer 2 redefined: standard attack nerf (1 DMG) + multi-Champion combo bonus. Two-game test format: Game 1 tests nerf alone, Game 2 adds combo bonus. Rule sheets written, feedback forms created, all 15 skill icons added.

---

### April 27 — Session 6: Layer 2 Scrapped, Baseline Overhauled

The 3 HP proposal (Layer 2) was killed: first Champion kill was R26 with 2 HP — 3 HP would push that later. Guards at 2 HP vs Champions at 3 HP would create an artificial tier. Baseline ruleset rewritten with all Playtest 2 clarifications. `docs/backpocket.md` created.

---

### April 25 — Session 5: Playtest 2 Analysis

Full transcription and analysis of all materials. Layer 1 marked ACCEPTED. The standard attack dominance problem identified in the data: 2 DMG free vs. skill combos costing 3-6 Runes for equivalent damage. Skills are structurally the supporting act despite being the stated core fantasy.

---

### April 24 — Playtest 2: Elias vs Jonathan (Layer 1 Economy Fix)

Economy fix confirmed working. Skills active from Round 1. Skill Slots now the real limiter. Injured state relevant ("Often" — both players). Defensive skills used meaningfully for the first time. Bodyguard triggered twice. Overall enjoyment: 4–5 out of 5. Jonathan: "Mid to late game Bombe — 6 out of 5."

But the game still ran four hours, ending as a draw at Round 26. Only one Champion kill in 26 rounds.

**Layer 1 accepted.** Economy changes (6 start Runes, +2/turn, +1 every 5 rounds) carry forward permanently.

---

### April 19 — Session 3: Rules Audit & Typst Migration

All rule ambiguities resolved via explicit designer rulings: Rune timing (start of own turn), movement pathing (free route ≤ speed, blocked by all pieces), attack resolution (attacker stops before target if it survives), Bodyguard scope (Standard Attacks only), healing cap (none), Skill Path blockers (all pieces). Canonical baseline ruleset created in Typst. Build system established.

---

### Session 4: Feedback Forms & Tracking

Per-player in-game tracking sheet (35-round log). Section C (systems feel) added to all feedback forms. `mechanics-evaluated.md` created as the living decision registry. `feedback-baseline.typ` template for future layers.

---

### April 18 — Session 2: Tooling

Six custom Claude Code skills built (`/start`, `/wrapup`, `/research`, `/playtest`, `/scenario`, `/adr`). Discovery: skill directories must be unhyphenated.

---

### April 17, 2026 — Session 1: The Claude Code Partnership Begins

Semester break. A decision to approach the design more systematically — not just "try a change and see what happens," but a real methodology: incremental layers, isolated variables, documented decisions, living documents that survive between sessions.

Enter Claude Code as AI co-creator. The first session established the project infrastructure: `game-state/`, `docs/`, Typst rule sheets, feedback forms. All six baseline rules documents read, 8 interlocking systems identified, 16 open questions documented.

Three research threads ran: wizard-chess genre landscape (Onitama, War Chest, Tash-Kalar), competitive card fighters (BattleCon, Flesh and Blood, Yomi), and cognitive load in game design (3-5 variables per decision = sweet spot).

The first architecture decision: three directions proposed for the game's structural identity. Direction A (Streamlined Tactical Grid, Onitama model) chosen. Direction B rejected ("just another card game"). Perfect information confirmed as non-negotiable.

Then the monolithic overhaul was proposed: change board size, HP, turn structure, economy, piece count, bodyguard simultaneously. Elias caught it: **"Don't change everything at once."** This became the mandatory incremental testing methodology.

---

### October 2025 — Playtest 1: Elias vs Pasco

The first real physical playtest. Pieces on a board. Two people across a table. Handwritten feedback forms.

The results were honest: game too long (~30 rounds), Rune economy too slow (first 6 rounds were skill-less — you just moved pieces and waited), Bodyguard never triggered, Injured state almost never reached because a standard 2 DMG attack killed a piece outright and skipped Injured entirely.

These findings became the entire testing programme. Every stack, every layer, every open question in this repo traces its origin to what Elias and Pasco found that evening.

---

*Pre-Session-1 history (v1 Realm of Elements 2023, v2 Project ROE 2024, v3 First Board Game 2025) lives in `old-game-versions/README.md`.*
