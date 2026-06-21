# Session Log

*Per-session narrative log. Newest sessions at the top. `/wrapup` adds a new entry per session.*

*This used to live in `old-game-versions/README.md`. Moved to `game-state/` in Session 17 because it is active state, not archive material.*

---

### June 21, 2026 — Session 26: Holiday-insights gathering + Stack M rule sheet finalised

Designer returned from holiday with a backlog of post-P5 insights and the Stack M draft from Session 25 sitting open. Session worked in two layers. **Layer 1**: holiday-insights gathering — surfacing accumulated brainstorm threads not yet captured in living docs. **Layer 2**: Stack M rule sheet went through multiple correction passes. First pass identified two structural mistakes from Session 25's initial draft — missing onboarding sections (Introduction + Simple Overview) and wrong section order vs baseline. Second pass identified a stylistic problem — over-use of `changed-box` callouts (5 vs Stack G=2, Stack L=0) and diff-style "_(Baseline: X)_" annotations that made the sheet read as a changelog rather than a playable ruleset. Third pass took 10 specific issues from the designer: wrong numbers in Simple Overview (introduced when I called the baseline function instead of inlining); the Introduction's "What makes it deep" paragraph spoon-feeding the experience; setup piece placement (player does NOT choose tiles — Kings sit mid-back-row but not directly opposite each other, with a 2+3 Champion split and one Guard directly in front of each Champion+King); in-text skill mentions not using the `sk()` chip helper; a substantive design change to the combo bonus; the Skill System still containing baseline's "Injured Range penalty" text; Health & Armor over-explained; skill table Name column too wide; armor-cap annotations leaking into skill effect text; Facilitator Notes visible on the player-facing PDF.

**Substantive design change inside Stack M's combo bonus** (originated this session): bonus damage now applies to *any* skill — Strike OR movement-causing — that affects a target with counter > 0, not only Strike hits. This unlocks damage strategies without Strike skills, making movement skills a damage vector once a target is counter-loaded. Designer rationale verbatim: *"i mean, wenn du eine figur mit einem counter von 1 mit einem movement skill bewegst, dann sollte doch die figur auch diesen einen extra bonus damage nehmen, no? das würde es ermöglichen auch ohne strike skills damage zu machen, was mehr unility gibt für strategien und spieler."* This expands Stack M's combo widening from "movement skills tick the counter" (Session 25) to "movement skills also deal +counter bonus damage on a counter-loaded target." Quick Reference row and Skill System carve-out updated to match.

**Working norm reinforced**: when the designer raises a list of problems, *analyse first before fixing* — produce a diagnostic before touching code. Session ran the diagnostic step under explicit direction (*"du sollst als erstes selber sagen was du denkst die probleme bei den sektionen sind, sodass du erstmal verstehen kannst was der korrekte weg wäre bevor du ihn gehst"*) and that step caught the design intent of the combo bonus change correctly on the second read after a first miscoded fix.

Stack M PDF rebuilt clean; all 11 PDFs in the build script regenerated. No new OQs. No changes to mechanics-evaluated, design-principles, or backpocket — this session was a finalisation pass on existing work, not new ground.

### June 21, 2026 — Session 25: Playtest 5 (Pole B digital) + Pole A revival as Active track

User returned from holiday with P5 in hand: Elias (P1) vs Jonathan (P2) on the digital Pole B per-turn-draft prototype. Jonathan won after 15 rounds; first Guard kill ~R7, first Champion kill ~R9-10. **No exported game log** — Jonathan refreshed the browser and the digital prototype had no persistence, wiping all state. Insights captured at `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md` from designer recall plus mid-game reasoning.

**Three structural problems surfaced in Pole B**, and together they pause Pole B as the Active track:

1. **Armor 3 still felt mandatory cross-pole** — Jonathan flagged unprompted that "3 armor ist zu viel". OQ-11 chassis-volume hypothesis gets a *cross-pole confirmation*: the Armor problem survives the radical structural switch from pre-game-draft to per-turn-draft, which says the issue lives in the Armor numbers themselves, not in Pole A's compounding-stack dynamic. OQ-11 graduates from "Pole-A symptom" to "load-bearing across both poles."
2. **Play collapsed to pure reaction.** Per-turn drafting removed the multi-turn planning horizon: you can't plan a 3-turn combo when your future skill set is decided turn-by-turn. The engine that powers cleverness-over-attrition (ADR-003 / Principle 4) needs forward planning to fire, and Pole B's structure starved it.
3. **Felt-PI broke even though formal-PI held.** Both players had full information access (open boards, visible skill pool, no hidden state) but the *combinatorial breadth* of per-turn skill choice was too wide to internalise — neither player could meaningfully evaluate "what will the opponent do next turn given their full possible draft set." Formal Perfect Information was preserved; the *feel* of PI was not. New OQ-64 captures this distinction as a design-time smell-test for any future option-space expansion.

**Trade-off read.** Pole B replaced one problem (Pole A's draft-determinism + compounding-Armor late-game tax) with another (felt-PI breakdown + reactive-only play). It is *not* a net win, and going further down Pole B without addressing the breadth problem would be a sunk-cost-driven move. Pole B is **paused, not killed** — other Pole B variants in `docs/backpocket.md` (permanently-equipped non-consumable, per-Skill-Phase activation cap, resource-cost-on-activation) may be revived if Pole A track stalls again.

**Designer call: return to Pole A with two concrete sub-goals.** Game short but empty (15 rounds, ~? wall-clock) reframed: shortness alone is not the goal — *short AND meaningful* is. Two sub-goals replace "fix Pole A's standoff" as the working frame:

- **Sub-goal A — Onboard new players better (OQ-65).** Pre-made loadouts: 2-3 curated starter sets per side. New players skip drafting and get a coherent kit; experienced players keep the full draft. Directly addresses OQ-56 Problem A (drafting cognitive load for first-timers) without the radical structural switch Pole B attempted.
- **Sub-goal B — Drastically shorten the game (OQ-66, Principle 6).** Target 30-60 minutes. Multi-lever pacing pass, not a single stack — Stack H Armor trim is the biggest lever; Stack K piece reduction and Stack D board geometry are secondary levers. Game-length measurement (rounds + wall-clock + first-Champion-kill round) becomes a tracked axis on every near-future stack, not a separate experiment.

**OQ-63 resolved on first encounter.** Cross-pole fixing methodology = **per-pole-revival**, not once-and-carry. Pole A and Pole B are different enough structurally that re-validation is required when a fix moves between them; the cost of testing twice is lower than the risk of a single read masking a pole-specific dynamic.

**OQ-62 (Pole A draft determinism) goes live again.** With Pole A back as Active, the simultaneous-reveal proposal returns to active status — and naturally couples to pre-made loadout selection (both players reveal loadout choices simultaneously). Likely bundled into the first Pole A onboarding stack rather than its own stack.

**Stack A G3 narrowing note.** On reflection (no fresh evidence — designer review), the **attacker counter** ("same friendly Champion hits different enemy targets → bonus on 2nd+ hit") felt too generous as drafted. Target counter scope-widening (any skill that hits an enemy piece counts) stays as-is. Narrow attacker counter at design time when Stack A G3 rule sheet is written.

**Cascade updates performed mid-session** (under explicit user direction "schreibe erstmal alle diese sachen in die orte in den docs"):
- `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md` — P5 record created from recall (no exported log)
- `docs/mechanics-log/mechanics-evaluated.md` — 2 new Methodology rows (Pole B prototype outcome → Pole A revival; cross-pole-first-encounter); Stack L moved to Withdrawn with paused-not-killed wording
- `game-state/OPEN_QUESTIONS.md` — OQ-61 partially resolved (Pole A continues), OQ-63 resolved (per-pole-revival), OQ-11 cross-pole confirmation logged, OQ-62 reactivated, OQ-64/65/66 opened
- `docs/backpocket.md` — 3 new top entries (Pre-Made Loadouts, Game Length 30-60 min, Digital Prototype Persistence) + attacker-counter narrowing note appended to existing dual-counter entry
- `game-state/NEXT_STEPS.md` — full restructure: Priority 1 = Pole A revival (two sub-goals); Priority 2 = Stack H promoted; Stack L → Dormant + Withdrawn
- `game-state/STATUS.md` — rewritten for Session 25; top-5 OQs = OQ-65, OQ-66, OQ-11, OQ-64, OQ-62
- `docs/test-scenarios/TESTING_PLAN.typ` — Active = "Stack TBD — Pole A Revival" placeholder with three candidate next-Active options; Q1 = Stack H (promoted, cross-pole confirmed); Stack L → Dormant (paused, with revival triggers) + Resolved row

**Tooling flag.** Any future digital playtest (Pole B revival, or Pole A digital) must persist state — auto-save per turn, export-as-JSON/PDF available at any time, per-turn state log. Until persistence ships, default back to paper for any "this game matters" run.

**Next action:** Decide on the next Active stack for the Pole A revival track — pre-made loadouts stack (OQ-65) OR Stack H — Armor Trim (OQ-11 + OQ-66) OR a bundled stack. User flagged that more holiday insights are still coming, so the next Active stack is NOT yet locked.

**Mid-session continuation — post-P5 brainstorm batch 1 (the path to the Game Length Cut).** After the P5 cascade landed, user shared a structured brainstorm dump covering 14 threads of post-game reflection (each annotated with "asan" — after sleeping a night). Most threads converged on a single conclusion: the next stack is a **Game Length Cut (GLC)** — a bundled change that compresses the game to the 30-60 min target while simultaneously reshaping end conditions to deliver a single-climax game shape (no up-down-up-down, no decided-but-still-playing tail).

The brainstorm chain (parked in full in `docs/backpocket.md` → "Session 25 Brainstorm — Post-P5 Direction-Setting"): (1) does the game need Perfect Information? — parked behind GLC; (2) would smart fair luck help? — parked, no justification; (3) win condition: King vs piece-count — folded into the GLC end-condition work; (4) HP/Armor power curve problem — parked, may dissolve with shorter games; (5) Armor as "easy progressive escape" — parked, re-check post-GLC; (6) Go-comparison on impact density (few high-impact moves punish small mistakes too hard; "≤1 min reachable / 10 min rewarded" KPI proposed) — parked; (7) what are players competing on? — parked; (8) Money's purpose is thin — explicitly NOT pursued now (would violate Principle 7); (9) **game economy / component map** — designer flagged as the next output coming in this session; (10) early-game pacing — leave as-is; (11) **single-climax game shape** — promoted to Principle 8; (12) Bodyguard removal — opened as OQ-67 (low prio); (13) draw conditions — opened as OQ-68 (gated into GLC bundle); (14) multi-Champ combo widening to non-Strike — folded into the GLC bundle (overrides the standalone Stack A G3 sequencing — to be reconciled when GLC stack is drafted).

**Principle 8 promoted to `design-principles.md`** — *"The game shape is a single climax, not a sine wave."* Pairs with Principle 6 (game length as attrition): short games + single climax = the shape now being optimized for. Practical consequence: end conditions must be reachable *at* the climax, not 10 turns after; game-shape becomes a tracked KPI in feedback forms.

**Cascade updates from brainstorm batch 1:**
- `docs/design-principles.md` — Principle 8 added; date bumped to 2026-06-21
- `docs/backpocket.md` — new top entry "Session 25 Brainstorm — Post-P5 Direction-Setting" with 14 enumerated threads (preserves the reasoning chain so future sessions see WHY GLC was chosen)
- `game-state/OPEN_QUESTIONS.md` — OQ-67 (Bodyguard removal) and OQ-68 (draw conditions, GLC-bundled) opened

**Next action (updated):** designer's next output is the game economy / component map (the "übersicht" diagram). After that, the GLC stack will be drafted. More brainstorm batches still incoming — do not lock the GLC stack yet.

**Phase 3 continuation — economy map written + Stack M (GLC) drafted.** Designer shared 9 brainstorm images (`brainstorm-images/IMG_1083.jpg` through `IMG_1091.jpg`) covering the hand-mapped 12-economy overview, quantitative end-state numbers, stalling root cause, move-space combinatorics, win-condition alternatives, and skill brainstorm. Full transcription written to `docs/research/game-economy-map.md` — a reference snapshot covering: (1) the 12 economies (win, tile, health, armor, money, skills-available, action, piece-count, damage, piece-progression, movement, movement-variations-in-turn, +skill-variations-in-turn); (2) quantitative end-state at R1/R5/R10/R15/R20/R25 — tile occupation 0.24 vs chess 0.5, Money cumulative 6→109 over rounds, Armor theoretical max 72, max combo damage 65 cumulative by R15; (3) Insight #1 (Money is limiting until you have it, then actions are) → unified-action follow-up parked; (4) Insight #2 (damage > heal asymmetry → two-sided pathology) → not directly addressed in Stack M, watched; (5) Insight #3 (stalling root cause: piece-state-progression more attractive than board-action when board-action is too complex or unavailable) → structural argument for Stack M's combined cuts; (6) move-space combinatorics: ~2.2M end-to-end distinct moves in one mid-game position — quantitative footprint of OQ-64; (7) win-condition alternatives (reach other side / move King or specific piece to target tile / 50% Champion deaths) **parked as future direction, not adopted for Stack M** per designer call; (8) Injury debuff removal reasoning ("piece close to dying is already incentive to retreat; double-cripple forces save-vs-heal lose-lose"); (9) skill brainstorm IMG_1091 referenced only — out-of-scope, reserved for future skill catalogue expansion.

**Stack M — Game Length Cut drafted.** Rule sheet + feedback form at `docs/test-scenarios/stack-m-game-length-cut/` (PDFs built). Six bundled simultaneous changes:
1. Board 10×10 → **8×8**
2. Armor cap 3 → **2**
3. Injured state: penalties removed (still 2 HP tracker; no speed cap, no Range −1)
4. Draw conditions removed entirely (not replaced)
5. Steal cost 3 → **4** (both Modes)
6. Multi-Champion Combo Bonus also ticks on **movement-causing skills** (Tempest push, Hook pull, Blast push, Shove, Swap when relocating enemy). Move-Attacks and self-movement (Dash, Retreat) still excluded. Movement-only triggers tick the counter but deal no bonus damage.

Designer call verbatim on the bundling: *"alles auf einmal — ich will schnellen progress sehen ... das soll intetionally alles auf einmal gemacht werden, es ist mir egal das es gegen unser prinzip verstößt."* Stack M intentionally violates the Incremental Testing Methodology; the deviation is documented loud in the rule sheet's "Why we are running this bundled" section, justified by Principle 7 (fundamental shifts while core unsettled) and the schedule cost of 6+ sequential isolation stacks. Per-axis rollback routing provides surgical recovery: if length/shape/no-stalling all land → accept; if combo widening dominates → roll back movement-counter only; if length still too long without stalling → next is Stack K piece reduction; if cleverness gone → roll back combo OR Steal; if Injured feels disposable → roll back Injured only; if no-draws causes infinite games → restore only-Kings-remain. Methodology recovers on the next stack.

**Cascade updates from Phase 3:**
- `docs/research/game-economy-map.md` — created. Reference snapshot of designer's hand-mapped economies + quantitative end-state + 2.2M-move combinatorics + win-condition alternatives parked.
- `docs/test-scenarios/stack-m-game-length-cut/stack-m-game-length-cut.typ/.pdf` — rule sheet created. Uses `section-resource-economy()`, `section-skill-phase()`, `section-skill-system()` etc.; inlines the changed sections (Goal / Setup / Multi-Champion Combo Bonus / Health & Armor / Skill Reference) with `#changed-box` callouts; overrides quick-reference table for Armor, Injured penalty, Combo Bonus, Board, Draws, Steal cost.
- `docs/test-scenarios/stack-m-game-length-cut/stack-m-feedback.typ/.pdf` — feedback form created. 18 questions targeting game length, single-climax shape, stalling pattern, board size, Armor cap, Injured-no-penalty feel, no-draws environment, Steal cost 4, combo widening reception, per-turn complexity, plus standard systems + free-notes sections.
- `docs/test-scenarios/TESTING_PLAN.typ` — Active = Stack M (full bundled description, hypothesis, watching axes, routing, cross-refs); Q1 Stack H demoted to "absorbed into Stack M; isolation-fallback only".
- `game-state/STATUS.md` — current focus rewritten: Stack M drafted as new Active; Top-6 OQs re-ranked.
- `game-state/NEXT_STEPS.md` — Priority 1 = Stack M drafted; Stack H = absorbed (P2); Stack A G3 = post-Stack-M (P3); pre-made loadouts + Stack K + Stack J = post-M sequencing.
- `game-state/OPEN_QUESTIONS.md` — OQ-68 resolved (removed via Stack M); OQ-11 / OQ-34 / OQ-38 / OQ-57 / OQ-66 marked "addressed/absorbed via Stack M, pending playtest result".

**Next action (final):** wait for next designer input. Stack M is print-ready. User flagged more brainstorm batches still incoming (*"ich habe danach noch weiter gebrainstormt, das teile ich aber erst danach"*) — these may amend Stack M before P6. Do NOT pursue tangential work.

---

### May 31, 2026 — Session 24: Pole B rules + project-wide vocabulary + template redesign

Three sequential goals tackled. **(1) Project-wide vocabulary simplification.** Broad rename pass across top-level docs, design-principles, systems, backpocket, mechanics-evaluated, research files, game-state, images README, skill files, and Typst rule sheets — 6 commits. Aligns terminology between files and removes jargon for both the designer and new players.

**(2) Pole B rule sheet written.** Standalone rule sheet for Stack L now lives at `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.typ/.pdf`. Fully inline (does not reference `baseline-sections.typ`) so the prototype reads as its own ruleset. Vocabulary: "Build Phase" → "Draft Phase". Three-phase turn: *Move Phase → Draft Phase → Skill Phase*. Move and Draft share a 4-action pool; Skill Phase is free with consumable activations (drafted skills exhaust on activation and return to the shared pool). Bodyguard sits between Move Phase and Draft Phase. Backpocket entries added for the three obvious Pole B variants — skills-cost-a-resource, per-Skill-Phase activation cap, permanently-equipped (non-consumable) drafted skills.

**(3) PDF template redesign.** User unhappy with prior template; ran a 2-round template-experiment process (3 fresh variants A/B/C, then 3 refined variants D/E/F based on round-1 feedback) before converging. Canonical `shared/template.typ` rebuilt with merged choices: H1 = 28pt Inter Display title (eyebrow dropped — was static "Rule sheet"); H2 = numbered presence (big numeral + "SECTION" eyebrow + bold title) with calmer teal numerals so headings support rather than dominate the rules; H3 = small-caps teal eyebrow; tables = cool grey header + charcoal hairline + light alt rows; new `sk("Lance")` chip helper for in-text skill references — light tinted pill with category-color outline + visible icon, vertically centered; callouts = note (teal) / changed (amber, no longer red) / designer (muted grey).

**Pagination fix.** Outer `#block(breakable: false)[...]` wraps were forcing half-empty pages when long sections didn't fit the remaining space. Removed from `baseline-sections.typ` (12 wraps) and `stack-l` (10 wraps) via Python script. H2 heading block kept `breakable: false, sticky: true` so SECTION + title never split and never get stranded. Lists/enums set to `block(breakable: false)` so bullets and sentences don't split mid-content. `#hr` separators removed from rule docs (kept in feedback forms / game-tracking where they separate fillable form sections). Scratch experimentation directory `template-experiments/` deleted; build script SKIP list trimmed back.

**Next action**: Run the first Pole B per-turn-draft prototype game digitally with Jonathan during the 3-week vacation window. Use `stack-l-per-turn-draft.pdf` as the rule sheet.

---

### May 30, 2026 — Session 23: Defense redesign + two-pole game framing

Stack H re-discussion gate (set at Session 22 close) opened the session and quickly expanded into a defense + game-shape redesign. Three diagnoses for the late-game Armor problem were tested: (A) Rune curve too steep — **killed** by user (starving Runes removes skill tension); (B) HP magnitude too thin — **killed** by user (no 2-DMG skills exist; raising HP just shifts the bottleneck); (C) Armor's *shape* is wrong, functioning as a **late-game survival tax / mandatory upkeep** — **confirmed**. User verbatim: *"i 100% agree that armor is like the tax you have to pay."*

This crystallised a **two-pole framing** that becomes load-bearing for the project's near-term direction. **Pole A — pre-game-draft** = current game (with possible incremental fixes). **Pole B — per-turn-draft** = radical alternative where skills are added to pieces during play. Pole B mechanics locked: skills are *reusable while equipped* (not removed after use); 12-skill equipped cap per player (6 Champions × 2 slots); shared action slots between moving and drafting; no Rune-economy activation gate. Skill pool effectively infinite — the cap is the constraint.

**Stack L — Pole B Per-Turn-Draft Prototype** becomes the new Active stack, claiming the slot for the 3-week vacation digital-prototype window with Jonathan. **Stack H — Armor Trim** moves to Queued; the bundled dose remains the lead variant, and the "build cheaper than break" risk is bigger than originally framed. Two new design principles promoted: **(6) game length is itself a form of attrition** (long games burn attention budget before the interesting decisions); **(7) while core identity is unsettled, prefer fundamental shifts over variable tweaking** (conditional — once core settles, incremental methodology resumes primacy).

**Pole A draft determinism** raised as OQ-62: sequential drafting drives an "always better to react" pathology. Proposal: simultaneous-reveal drafting (both pick 2 at once, repeat). User accepts limited PI loss *only* in the pre-game window. **Cross-pole fixing methodology** raised as OQ-63: when a fix targets a problem in both poles, test once or per pole? User lean: per pole for cleanness; resolved on first encounter.

Cascade edits performed mid-session per user direction (no edits before plan approval): `docs/research/path-y-defense-redesign.md` written as the canonical writeup; `design-principles.md` (+2 principles, date bumped); `backpocket.md` (+3 entries — Armor diagnosis anchor, Armor-cap-by-round, Pole B one-turn-killer potential); `OPEN_QUESTIONS.md` (+OQ-61/62/63, OQ-11 status updated to Queued); `TESTING_PLAN.typ` (Stack L Active; Stack H Queued; renumbered queued list); `baseline-sections.typ` (combo bonus migrated to baseline as `section-multi-champion-combo()`, BASELINE_VERSION → 2026-05-30, Quick Reference row added); `mechanics-evaluated.md` (combo migration row + new "Methodology / Design Decisions" section).

**Repo housekeeping.** Deleted `stack-b-guards/` (withdrawn Session 22, never played). Archived `stack-a-cleverness/` to `old-game-versions/archived-stacks/` (accepted into baseline). Switched all Typst imports from `../shared/...` to root-relative `/docs/test-scenarios/shared/...` form so files survive future folder moves — the build script already passes `--root $PROJECT_ROOT`. Cleanup sweep updated WHAT_TO_PRINT.md, README.md, HANDOVER.md, scenario skill, baseline-sections docstring.

**Next action:** run the first Pole B per-turn-draft prototype game digitally during the 3-week vacation window. After 2–3 games, compare game-feel vs Pole A and route per Stack L's *Routing on result*.

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
