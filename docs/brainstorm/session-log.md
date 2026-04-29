# Session Log

## Session 12 — 2026-04-29 — TESTING_PLAN Overhaul + New Ideas

**Goal**: Audit whether all test stacks have up-to-date timelines/entry conditions in the TESTING_PLAN document; fix any staleness. Capture new design ideas.

**What was done**:
- Deep audit of TESTING_PLAN.typ revealed multiple stale entries from Sessions 10-11 decisions not being reflected.
- Fixed Stack C description (removed killed checkmate, replaced with King Lifetime HP / Armor Decay).
- Fixed Stack F description (replaced "Ultimate skills" with cascade trigger / Pin/Threatened / sente skills).
- Fixed Stack D description (added piece count scenario).
- Updated all entry conditions (Stack C, F corrected; Stack G added — was missing entirely).
- Added "Current Priority Sequence" section (P1–P4 table mirroring NEXT_STEPS.md).
- Replaced unreadable Mermaid diagrams with table-based decision tree covering all 7 stacks in two phases.
- Added Session 11 context note-box summarising key design decisions.
- Removed `mmdr` package dependency (Mermaid diagrams replaced by tables).
- All 12 PDFs rebuilt cleanly.
- Added 3 new `[TO DISCUSS]` ideas to backpocket.md: terrain objects/placeable skill stations, laser beam (piercing line damage), wave push/pull (mass line displacement).

**Key findings**:
- TESTING_PLAN.typ had been stale since Session 9 — Sessions 10-11 made material design decisions (checkmate killed, sente chosen, G8 added) that weren't reflected in the document.
- Mermaid diagrams via `mmdr:0.2.1` render too small for complex flowcharts (~40 nodes). Table-based format is far more readable for printed reference.
- Stack G had no entry condition documented anywhere in TESTING_PLAN — oversight since Session 9.

**Decisions made**:
- Decision tree format: tables over Mermaid diagrams for this document (readability for printed reference).
- All stack descriptions and entry conditions now match CURRENT_DESIGN.md and NEXT_STEPS.md as single source of truth.

**Open items for next session**:
- **Print and play Stack A** — still the top priority (unchanged since Session 9).
- Discuss 3 new `[TO DISCUSS]` ideas (terrain objects, laser beam, wave push/pull) when appropriate.
- After Stack A results: use staged research in backpocket to respond quickly.

---

## Session 11 — 2026-04-29 — Research & Brainstorm: Playtest Response Toolkit

**Goal**: Pre-load design knowledge for rapid responses to upcoming Stack A/B playtest results. Research 4 topics via Perplexity, discuss findings, stage concrete candidates.

**What was done**:
- Researched OQ-51 (mechanical levers for rewarding clever play): identified 4 patterns — threat-as-reward, environmental multiplier, restriction-as-reward, one-time action economy. Staged Cascade Trigger (+1 Skill Slot on kill), Pin/Threatened (2+ LoS = can't move), Collision Damage (as Ram skill).
- Researched checkmate-style win conditions: killed checkmate (verification burden impossible with ranged + heals + armor). Replaced with King Lifetime HP and Armor Decay as Stack C candidates.
- Researched forward positioning incentives: sente skill design chosen as primary standoff solution. Preserves game identity (uses existing skill system) over territory mechanics. Fallback hierarchy staged.
- Researched skill catalogue balance: minimum 25-35 skills needed. 10 new candidates designed (Thorn Armor, Runic Ward, Bulwark, Bind, Energize, Skill Drain, Mini-Step, Swap Step, Ram, Gravity Well) with sente test applied to each.
- Ran conflict check: 10 potential clashes between new skills and existing guardrails. User resolved all — most "test it and see." Sente vs G1 tension researched and resolved.
- Added G8 (Spending Tension) guardrail to backpocket and CLAUDE.md. Researched via Perplexity.
- Mini-Step deprioritized (luxury candidate, gated on sente results).

**Key findings**:
- Checkmate is fundamentally wrong for this game — can't detect "inescapable lethal" with ranged attacks + heals + armor.
- Sente and G1 coexist: forced reactive spending IS compatible with "shortfall never closes" because the tradeoff persists (respond = can't execute own plan).
- The economy naturally transitions: early=Rune-scarce, mid=Slot-limited, late=opportunity-rich. G8 codifies this.
- Cascade trigger works on any kill (not "kill via combo") — simpler, less snowbally, actually fires.
- Skill catalogue problem is VARIETY of effects, not underpicking of categories (Playtest 2 showed all Shield skills heavily drafted).

**Decisions made**:
- Checkmate win condition KILLED — replaced by King Lifetime HP concept (backpocket).
- Sente skill design = primary standoff approach (not territory mechanics).
- G8 guardrail established: "Players must always want to do more than they can execute."
- Mini-Step deprioritized to luxury/post-sente candidate.
- 10 skill candidates staged in backpocket with full sente analysis and conflict notes.

**Open items for next session**:
- Print and play Stack A (still top priority — unchanged).
- After Stack A results: use staged research to respond quickly (candidates ready in backpocket).
- `/playtest` skill improvement (draft-pick extraction) when next playtest happens.

---

## Session 10 — 2026-04-29 — Old Versions Triage + Design Language

**Goal**: Convert old xlsx/pptx files to readable markdown, extract all possible design ideas from old game versions, triage them against current design state, collect user feedback on every idea, and process that feedback into the correct project locations.

**What was done**:
- Converted `ROE-goofing-around.xlsx` (v2) and `Project ROE Skills.xlsx` / `quick list skills export gemini.xlsx` (v3) from garbled auto-conversions into clean readable markdown.
- Extracted 21 new idea entries into `docs/research/old-versions-ideas.md` (Champion roster concepts, terrain modifiers, 13 skill ideas, economy insights, draft constraints).
- Created `docs/mechanics-log/old-versions-triage.md` — ~80 ideas sorted into New/Possible, Deferred, Archived with reasoning.
- Created feedback form (`old-versions-triage-feedback.md`), user annotated all items.
- Restructured `docs/backpocket.md` — added 7 Design Guardrails (G1-G7), Known Potential Issues section, new staged ideas (Draw if only Kings, Line pull).
- Created `docs/design-language.md` — future visual/identity direction (blank-slate Champions, emergent skill identity, naming direction, physical design notes).
- Updated triage with final dispositions: 9 items → Archived, 7 items → Deferred (based on user feedback).
- Added 3 backlog items to NEXT_STEPS: temp effect tracking research, adjacency synergies brainstorm, skill catalogue expansion roadmap.
- Added `design-language.md` to README navigation.

**Key findings**:
- User confirmed: Champions must remain blank slates — identity emerges from equipped skills, not pre-naming.
- "Shortfall never closes" is an explicit economy principle (now guardrail G1).
- Cognitive load from tracking temporary effects is the primary blocker for several skill ideas (Temp Armor, Shield duration, Guard-Bind). Research needed.
- User explicitly excited about adjacency synergies / piece compatibility — connects to OQ-51.

**Decisions made**:
- Design Guardrails established (G1-G7 in backpocket.md) — invariants every proposed change must pass.
- Information/scouting skills archived — irrelevant in perfect-information game.
- Class-based skill pools archived — restricts strategy freedom for minimal complexity reduction.
- Champion naming archived — contradicts blank-slate design value.
- Visual identity work deferred to Phase B (~2027) — design-language.md collects insights until then.

**Open items for next session**:
- Print and play Stack A (unchanged from Session 9 — still the immediate priority).
- Run `/research how board games track temporary effects on pieces` when ready to unblock triage items.
- Brainstorm adjacency synergies when Stack F becomes active.

---

## Session 9 — 2026-04-28 — Dynamic Stack System + Composable Rule Sheets

**Goal**: Replace the fixed linear test layer queue with a dynamic, evidence-driven stack system. Build composable Typst rule sheets so baseline changes propagate automatically. Improve the `/playtest` skill for deeper analysis.

**What was done**:
- Built `docs/test-scenarios/shared/baseline-sections.typ` — 16 parameterized `#let` section functions covering the entire ruleset. Layer files now ~50 lines each (down from ~250).
- Refactored all existing rule sheets to use composable sections: `ruleset-baseline.typ`, Stack A Game 1 + Game 2, Stack B bodyguard fix, Stack G unified AP.
- Created `docs/test-scenarios/TESTING_PLAN.typ` — compiles to PDF; contains 6+ stack definitions table, Mermaid decision tree (`@preview/mmdr:0.2.1`), entry conditions, accepted layers table.
- Renamed all `layer-N-*` folders/files to stack-based naming: `stack-a-cleverness/`, `stack-b-guards/`, `stack-g-structure/`, `accepted-layer-1-economy/`.
- Updated `build-pdfs.sh` with new paths; all 12 PDFs rebuild cleanly.
- Rewrote `/scenario` skill — now uses composable pattern and updates TESTING_PLAN.typ.
- Updated `/wrapup` skill — added TESTING_PLAN.typ maintenance step.
- Improved `/playtest` skill — added Block B behavioral pattern extraction, multi-agent independent transcription (per-player isolation), OQ Metric Evaluation step with Verdict blocks.
- Improved `feedback-baseline.typ` — added OQ-monitoring comment block and 4 standard OQ-monitor questions (OQ-10, OQ-11, OQ-34, OQ-46) to Section C skeleton.
- Updated Stack A and Stack B feedback forms with those 4 OQ monitoring questions, armor totals grid, and kill timing fields.
- Fixed internal references in renamed `.typ` files (feedback form pointers, heading titles, note-box text).
- Added Stack G (Structure) to TESTING_PLAN stacks table.

**Key findings**:
- Typst reserved keyword `show` cannot be used as a parameter name — renamed `section-combo-bonus(show:)` to `section-combo-bonus(enabled:)`.
- `baseline-sections.typ` must `#import "./template.typ": *` internally to access `skill-table`, `skill-icon`, `changed-box`, `note-box` helpers.
- The dynamic stack model is now the canonical testing methodology — stacks are independent experience-outcome groups, not a fixed sequence.

**Decisions made**:
- Testing methodology migrated from fixed linear queue to dynamic stack selection (pick highest-value stack after each playtest).
- Stack G created for radical structure redesign (unified AP framework) — separate from combat/Guard stacks.
- `/scenario` skill now always creates `stack-X-<slug>/` folders, never `layer-N-<desc>/`.

**Open items for next session**:
- **Print and play Stack A** — materials fully ready in `stack-a-cleverness/`.
- Stack G (`stack-g-structure/stack-g-unified-ap.typ`) could use skill icons and rule clarifications — low priority.
- Stacks C, D, E, F not yet written — write on demand when triggered.

---

## Session 8 — 2026-04-28 — Project Cleanup, Ideas Migration, Deferred-Items Triage

**Goal**: Migrate all ideas from `baseline-rules/md-converted/Systems to Test.md` into the living document system; populate `docs/systems/` with per-system files; clean up project structure; triage all vague-trigger deferred items; retroactive analysis of implicitly closed design topics.

**What was done**:
- Populated `docs/systems/` with 7 per-system files: turn-structure.md, resource-economy.md, progression.md, skill-system.md, combat-attack.md, health-armor.md, skill-drafting.md. Each has How It Works, MDA Analysis, Design Health, Open Questions, Playtest Evidence sections.
- Converted `CURRENT_DESIGN.md` from monolith to summary index pointing to per-system files.
- Migrated all ideas from Systems to Test into OPEN_QUESTIONS.md (OQ-42–50) with `[System: X] [Affects: Y]` tags.
- Reopened hex grid (OQ-42) — moved from Withdrawn to Reopened in mechanics-evaluated.md. Original Session 1 rejection was by omission, not evaluation.
- Deep retroactive analysis of implicitly closed design topics — identified 11 findings (performance economy, hex, 3rd skill slot, Guard HP, terrain, restricted movement, Bodyguard attacker-only variant, etc.)
- Full triage of all deferred items: 6 closed permanently (moved to Withdrawn), 4 parked indefinitely (v1 out of scope), 6 given concrete re-entry triggers, 3 given monitoring criteria.
- Added OQ-51: Mechanical levers for rewarding clever plays (cascade triggers, positional payoffs, coordinated movement bonuses, checkmate-style threat creation).
- Added 3 new backpocket ideas: Mini-Step skill (1-2 Runes, move 1 tile), Reveal-style simultaneous placement, Mid-game side swap (CS-style board rotation).
- Updated CLAUDE.md architecture section (docs/systems/ populated, stale paths fixed, system descriptions corrected to current values).
- Noted 3rd-skill-slot verdict in mechanics-evaluated.md (closed by principle: 2 slots forces specialist builds → draft tension).
- Noted performance-based economy full reasoning in mechanics-evaluated.md (forces playstyle, auto is strategy-neutral, combo bonus is the correct lever).

**Key findings**:
- **11 implicitly closed design topics identified**: Architecture decisions (ADR-001/002) collapsed multiple branches into one path without evaluating each individually. Key reopening: hex grid.
- **"Deferred to Layer X" = structural closure risk**: Several items had vague triggers that would never fire. Fixed by assigning concrete conditions or marking v1 out of scope.
- **Performance-based economy definitively closed**: Forces one playstyle; auto-economy is strategy-neutral; combo bonus rewards cleverness of execution not outcomes. KPI principle from ADR-003 applies.
- **3rd skill slot closed by principle**: 2 slots → specialists → rock-paper-scissors draft dynamics → meaningful tradeoffs. Fix for "narrow variety" is better skills, not more slots.
- **New design question emerged (OQ-51)**: Beyond combo bonus, what mechanical levers reward clever multi-turn setups? Cascade triggers, positional payoffs, and checkmate threats are candidates.
- **Mini-Step skill identified as gap-filler**: 1-2 Runes for 1-tile repositioning fills gap between free movement and expensive Move skills (Quick Dash = 3 Runes).
- **Reveal-style placement solves infinite-adjustment problem**: Simultaneous commitment avoids reactive counter-positioning loop in OQ-36/48.

**Decisions made**:
- Hex grid reopened (OQ-42) — needs `/research hex vs square grid in tactical games` before any layer proposed
- Performance-based Runes closed permanently — forces single playstyle
- Economy skills as slot closed permanently — 2-slot scarcity makes it unworkable
- Minor/Major slot cost deferred with pre-work: design ultimate skills first, then test in Layer 4+
- Restricted movement parked indefinitely — not solving a real problem, dissolves if hex adopted
- CR-draft, ban phase, starting player bid all parked — v1 out of scope
- OQ-35 (skill pool draft) trigger: post-Layer-3 acceptance
- OQ-36+48 (placement) trigger: post-Layer-3, reveal-style approach
- OQ-27+1 (piece count + board size) trigger: if first Champion kill still past R15 after Layer 2/3
- OQ-19 (endgame acceleration) trigger: if first Champion kill past R20 after Layer 2
- OQ-50 (slot cost) trigger: design ultimate skills first, then Layer 4+
- First-player advantage fix if needed: Go-style komi (fewer starting Runes), not bidding
- Side-swap idea captured in backpocket (raw, unformed — literally rotate board 180° and play opponent's pieces)

**Open items for next session**:
- **Print and play Layer 2** — materials unchanged and ready
- **Run `/research hex vs square grid in 2-player tactical board games`** (OQ-42) — if designer wants to explore before next playtest
- Explore OQ-51 (mechanical levers for clever plays) — research how comparable games reward multi-turn setups
- Layer 4 rule sheet still needs skill icons and clarifications (low priority)

---

## Session 7 — 2026-04-27 — Layer 2 Decided, Cleverness vs Attrition Analysis, Shared-Puzzle Research

**Goal**: Decide Layer 2 topic; explore "play against the game together" design direction; improve custom skill usage.

**What was done**:
- Created `SessionStart` hook (`.claude/hooks/session-start.sh` + `.claude/settings.local.json`) — injects skill-trigger reminders at every session start.
- Recorded in-game skill redraft idea (shop/auction/interval/swap variants) in `docs/backpocket.md` as a deferred Layer 6+ concept.
- Ran `/research` skill: "cooperative feel in competitive 2-player tactical games." Full research saved to `docs/research/cooperative-feel-competitive-games.md`. Found the "mutual epistemic exploration" phenomenon — both players co-interpreting the same puzzle in perfect-information games. Mapped 6 design patterns from published games (Onitama, Twilight Struggle, Go, Tak, T&E, Race for the Galaxy).
- Deep multi-round design discussion: system-by-system audit of cleverness vs attrition reward structure. Identified standard attack dominance (2 DMG free vs. skill combos costing 3-6 Runes for same damage) and standoff/no-man's-land problem as critical issues.
- Created `docs/decisions/ADR-003-rewarding-cleverness.md` — full audit, design principles, test plan, deferred ideas with reasoning.
- **Layer 2 topic decided**: Standard attack nerf (1 DMG) + multi-Champion combo bonus (+1 DMG on second Champion hit to same target). Two-game test format in one session.
- Added 5 new backpocket entries: retaliation variant, jump skill, risky positioning, action-based economy, checkmate win condition. Fixed Blade Call entry (+1 Range duplicates Focus Strike).
- Added 5 new open questions: OQ-37 (standard attack damage), OQ-38 (combo bonus), OQ-39 (shared-puzzle direction), OQ-40 (standoff problem), OQ-41 (game length tradeoff).
- Updated all living docs: CURRENT_DESIGN.md (test plan, target experience, health check), NEXT_STEPS.md (Layer 2 decided, backlog expanded), OPEN_QUESTIONS.md, HANDOVER.md.

**Key findings**:
- **Standard attack dominance is the critical problem**: 2 DMG for free outperforms every skill combo in the game. Skills are structurally the support act despite being the stated core fantasy.
- **Standoff / no-man's-land**: Both playtests showed a 2-3 tile gap between formations that neither player wants to enter. Caused by lethal standard attacks (2 DMG = instant kill) + no economic incentive to commit.
- **Low combo ceiling**: Only "buff + hit" combos exist. No emergent interactions, no multi-Champion coordination incentive. Depth is evaluative (comparing known options), not generative.
- **Shared-puzzle feel is real and engineered in great games**: Onitama, Twilight Struggle, Go all deliberately produce collaborative analysis in competitive settings. The game already has the infrastructure (perfect info, shared skill pool, symmetric scarcity).
- **Guard kill data correction**: Playtest 2's "only 1 kill in 26 rounds" refers to Champions specifically. Guards were dying throughout. Standard attack nerf will specifically slow Guard clearing — key risk for game length.

**Design principles established (ADR-003)**:
1. Every strategy archetype should have a moment where it's the best option on the board
2. Don't reward symptoms, reward the system (KPI principle)
3. Players should play how they want — expand viable strategies, don't shrink them
4. Cleverness = multi-turn setup rewarded with a payoff exceeding what grinding achieves
5. Shared-puzzle feel comes from perfect info + depth, not from removing competition

**Decisions made**:
- Layer 2 = standard attack 1 DMG (Game 1) + multi-Champion combo bonus (Game 2)
- Two-game test format (same session, back-to-back). Nerf first, combo bonus additive.
- Guard HP stays at 2 — designer wants Guards to have late-game presence
- 3rd skill slot per Champion withdrawn — dissolves draft tension
- Own-pieces-don't-block-LoS withdrawn — creates turtle meta
- Economy changes parked until post-nerf data
- In-game redraft, checkmate, emergent combo research all parked for later

**Open items for next session**:
- ~~Write Layer 2 rule sheets~~ DONE
- ~~Write Layer 2 feedback forms~~ DONE
- ~~Update Layer 3 rule sheet~~ DONE (icons, clarifications)
- Layer 4 rule sheet still needs skill icons and clarifications
- **Print and play Layer 2** (two-game format)

### Session 7 cont. — Rule Sheet Production & Clarifications

**What was done (cont.)**:
- Wrote Layer 2 Game 1 rule sheet (`layer-2-game1-attack-nerf.typ`) — standard attack 1 DMG, full standalone with Layer 1 economy.
- Wrote Layer 2 Game 2 rule sheet (`layer-2-game2-attack-nerf-combo.typ`) — nerf + combo bonus with detailed rules, examples, Blade Call interaction.
- Wrote Layer 2 feedback form — two-game format covering both games, with kill-tracking and standoff questions.
- Added skill icons (15 JPGs in `images/`) to template (`skill-icon()` helper at 1.8em) and all rule sheet skill tables (5-column with icon).
- Reordered sections: Progression moved before Skill Reference; page break so Skill Reference + Quick Reference share the last page.
- Updated `build-pdfs.sh`: renamed Layer 2 folder paths, added `--root` flag for image resolution.
- Removed old `layer-2-three-hp` files.

**Rule clarifications applied across all sheets**:
- **Bodyguard adjacency**: "attacker's starting tile" → "tile immediately before the target (along the attack path)" — handles Speed 2 attackers correctly.
- **Focus Strike**: "One skill used by any of your pieces" → "The next skill" — distinguishes from Blade Call's "any one Strike this turn" timing.
- **Skill reuse**: "A Champion may activate both of its equipped skills" → "may use its skills multiple times — including the same skill twice."
- **Combo bonus details**: skills don't have to be consecutive; Blade Call boosts exactly one Strike then is spent; "different piece" → "different attacking piece."
- **Progression/Economy tables**: added continuation pattern notes ("(+1 every 10 rounds)", "(+1 every 5 rounds)").
- Updated `CURRENT_DESIGN.md` Focus Strike text and Bodyguard description to match.

---

## Session 6 — 2026-04-27 — Rule Clarifications Applied, Layer 2 Scrapped, Baseline Overhauled

**Goal**: Apply all rule clarifications from Playtest 2 to the baseline and layer sheets; restructure the test-scenarios folder; create backpocket.md; decide Layer 2 topic.

**What was done**:
- Rewrote `baseline/ruleset-baseline.typ` with all decided rule changes: per-piece-once movement, Bodyguard (Guard takes damage, attacker moves 1 tile, optional), default Range 2 + numbering, Injured Range (affects Range 2+ only — adjacent and self always work), movement-via-skills no damage, double activation allowed, all corrected skill texts (Lance Thrust Range−1, Field Medic "Remove Injured", Blade Tempest caster unaffected, Precision Thrust Range+1, Shadow Shift default Range 2, Retreat Plan "Range+1", Focus Strike any of your pieces, Blade Call fixed cost 3), corrected economy table (2–4/5–9/10–14/15+), corrected progression table (R1–10/R11–20, shifted by 1, with "(etc.)"), free skill assignment in draft.
- Rebuilt `docs/test-scenarios/` folder structure from flat `typ-files/`+`pdf-files/` into per-layer subfolders: `shared/`, `baseline/`, `layer-1-economy-fix/`, `layer-2/`, `layer-3-bodyguard-fix/`, `layer-4-unified-ap/`. All `.typ` import paths updated. `build-pdfs.sh` rewritten.
- Rewrote `layer-3-bodyguard-fix/layer-3-bodyguard-fix.typ` from scratch: carries all baseline clarifications forward; the ONLY marked change is adjacency to defender only.
- Fixed `layer-4-unified-ap/layer-4-unified-ap.typ` import path and added version header.
- Created `docs/backpocket.md`: pre-thought fixes for Rune Theft nerf, Blade Tempest direction ambiguity, Blade Tempest blocker chain, Blade Call extension to movement skills, Focus Strike Skill Slave problem, new skill ideas (Ultimate Heal/Shield, Push Wave), skill gap notes (Shield/Mystic).
- Updated all living docs: CURRENT_DESIGN.md (skill catalogue, rules, economy/progression tables, Layer 2 scrapped status), OPEN_QUESTIONS.md (OQ-18 scrapped, OQ-20/21/29–33 resolved, Injured Range rule clarified), NEXT_STEPS.md (Priority 0 done, Layer 2 topic decision as new Priority 0), CLAUDE.md (new folder structure), HANDOVER.md.
- **Updated `/playtest` skill (SKILL.md)**: Added "common misread patterns" section with explicit guidance on all error types encountered in Playtest 2 (circle position, "gut so" as design affirmation, soft vs hard flags, crossed-out questions, "?" answers, German handwriting traps). Added side-notes exhaustive checklist. Added "Final round played" and "Post-game annotations" fields.

**Key findings**:
- **Layer 2 (3 HP) scrapped**: Would make game even longer (already only 1 kill in 26 rounds with 2 HP). Guards at 2 HP vs Champions at 3 HP would make Guards feel like cheap kills by design — conflicts with design intent.
- **Injured speed penalty is Guard-only**: Champions and King are already Speed 1. The Injured effect on Champions/King is purely Range 2+ → Range 1. Worth monitoring whether this is punishing enough.
- **Injured Range ruling**: −1 affects only Range 2+. Adjacent (Range 1) and self (Range 0) always work, Injured or not.

**Decisions made**:
- Injured Range −1 affects only Range 2+. Adjacent and self always work.
- Retreat Plan: "Range+1" (not "Skill Range+1" — redundant).
- Blade Call: confirmed fixed cost 3 Runes, +1 DMG built in, no extra payment.
- Layer 2 topic: **still to decide** — candidates are smaller board (8x8), fewer pieces, 3 Move Slots, Rune Theft nerf, or other.

**Open items for next session**:
- Decide Layer 2 topic (see Priority 0 in NEXT_STEPS.md).
- Write Layer 2 rule sheet once topic decided.
- Rebuild all PDFs.

---

## Session 5 — 2026-04-25 — Playtest 2 Analysis

**Goal**: Analyse Playtest 2 (Elias vs Jonathan, 24.04.2026), update all living documents.

**What was done**:
- Transcribed and analysed all Playtest 2 materials: both game logs (Elias + Jonathan), both 2-page feedback forms, old skill overview page, and Elias's side-notes.
- Created `docs/research/playtest-2-analysis.md` — full structured analysis with tracking data, raw transcriptions, key findings, and comparison to Playtest 1.
- **Layer 1 accepted**: Economy fix confirmed working. Skills active from Round 1. Dead opening eliminated.
- Updated `game-state/CURRENT_DESIGN.md`: Layer 1 marked ACCEPTED; Playtest 2 evidence added; Design Health scores updated.
- Updated `game-state/OPEN_QUESTIONS.md`: OQ-17 resolved; OQ-21 updated with P2 data; 7 new OQs added (OQ-29–35: rules clarifications and new design questions).
- Updated `game-state/NEXT_STEPS.md`: Priority 0 added (apply rule clarifications before next playtest); priorities reshuffled.

**Key findings**:
- Layer 1 economy fix is a clear success — both players rated experience "Better/Much better" than Playtest 1.
- Injured state now relevant ("Often" — both players). Defensive skills used. Bodyguard triggered 2x.
- Game still too long (~26–30 rounds, ended as Draw by time pressure at 23:40).
- 6+ rules ambiguities surfaced mid-game — must be codified before Layer 2.
- Rune Theft flagged as potentially too strong with faster economy.
- Jonathan: "Spielfeld kleiner → Schneller Action" — aligns with Layer 5 hypothesis.

**Decisions made**:
- Layer 1 accepted. Economy changes (6 start, +2/turn, +1 every 5 rounds) carry forward into all future layers.
- Priority 0 established: apply all rule clarifications to Layer 2 rule sheet before next playtest.
- Next playtest: Layer 2 (3 HP for Champions/King).

**Open items for next session**:
- Update `layer-2-three-hp.typ` with all rule clarifications (see Priority 0 in NEXT_STEPS.md).
- Rebuild PDFs and prepare Layer 2 print packet.

---



**Goal**: Build a per-player in-game tracking sheet, improve feedback forms to cover all game systems (not just the layer being tested), establish a feedback form baseline template, update the `/scenario` and `/playtest` skills to use them, and eliminate stale file references throughout the project.

**What was done**:
- Created `docs/test-scenarios/typ-files/game-tracking.typ` — per-player in-game tracking sheet (header + 35-round log: Runes, skills used, events/notes). Compiled to `pdf-files/game-tracking.pdf`.
- Added **Section C — Systems & Overall Feel** to all three feedback forms (layers 1–3): 7 questions covering skill drafting, turn flow, skill balance/combos, Bodyguard, one carry-over system monitor, best/worst moment, plus rating rows. Identical across all layers for cross-layer comparability.
- Created `docs/test-scenarios/typ-files/feedback-baseline.typ` — generic feedback form template with `[LAYER: ...]` placeholders. Compiled to `pdf-files/feedback-baseline.pdf`. Added template banner so it can't be accidentally used as a player form.
- Updated `/scenario` skill (Step 5): now says copy `feedback-baseline.typ` and fill in placeholders, instead of describing the form structure inline. Also fixed stale Typst preamble instruction (was `#let horizontalrule`, now `#import "template.typ": *`).
- Updated `/playtest` skill: added Step 2.5 — Tracking Sheet Analysis. When tracking sheets exist, extracts structured data blocks per player (Rune economy, skill usage frequency table, captures/events log) before synthesis. Added `## Tracking Data` section to analysis doc structure.
- `build-pdfs.sh` updated to compile `feedback-baseline.typ` — now produces 10 PDFs.
- Standardised Section A heading across all feedback forms to "A — Observational Data" (layer-1 was inconsistently named "A — Layer: Economy Fix").
- **CLAUDE.md full architecture update**: corrected architecture tree to show actual Typst layout (`typ-files/`, `pdf-files/`, all new files), updated Key Game Systems to current values (10x10, no terrain, Normal→Injured→Removed), fixed all `.md` references in conventions and methodology to `.typ`, clarified baseline-rules as outdated historical reference.
- Fixed stale `docs/ruleset-baseline.md` references in `HANDOVER.md` (3 instances) — now correctly point to `.typ` source / `.pdf` printable.
- Created `docs/mechanics-log/mechanics-evaluated.md` — pre-populated running log of all mechanics considered, accepted (baseline), accepted (pending test in layers), deferred, and withdrawn/rejected. Drawn from ADRs, session log, and OPEN_QUESTIONS.md.

**Key findings**:
- `docs/ruleset-baseline.md` (Markdown) never existed — only the `.typ` equivalent does. CLAUDE.md and HANDOVER.md had been pointing to a ghost file since Session 3. Now fixed.
- Three empty folders (`docs/core-loops/`, `docs/mechanics-log/`, `docs/systems/`) were in the architecture. Decision: populate `mechanics-log` only; the other two stay reserved.
- Audit confirmed `ruleset-baseline.typ` IS the single source of truth for rule text — no conflicts with other files (legacy baseline-rules excluded as intentionally outdated).

**Decisions made**:
- Tracking form = objective in-game data only. Feedback form = subjective + systems feel. No overlap.
- `feedback-baseline.typ` is the canonical starting point for all new layer feedback forms going forward.
- `docs/mechanics-log/mechanics-evaluated.md` is the living log for mechanic status; update whenever a mechanic is proposed, resolved, or withdrawn.
- `docs/systems/` and `docs/core-loops/` left empty — content lives in `CURRENT_DESIGN.md` for now.

**Open items for next session**:
- Run Layer 1 (Economy Fix) playtest — print `pdf-files/layer-1-economy-fix.pdf` + 2 copies of `pdf-files/layer-1-feedback.pdf` + 2 copies of `pdf-files/game-tracking.pdf`

---

## Session 3 — 2026-04-19 — Rules Audit, Baseline Ruleset & Typst Migration

**Goal**: Audit all existing rules for ambiguities and gaps, establish a canonical player-facing baseline ruleset, overhaul test scenario rule sheets to be proper standalones, add per-layer feedback forms, and migrate all documents to Typst for PDF generation.

**What was done**:
- Resolved all core rule ambiguities via explicit user rulings (Rune timing, movement pathing, attack resolution, Bodyguard scope, healing cap, Skill Path blockers)
- Created `docs/ruleset-baseline.md` and `docs/test-scenarios/typ-files/ruleset-baseline.typ` — canonical player-facing rules, all rulings applied
- Rewrote Layers 1–3 as full standalone rule sets (complete copy of baseline with only the changed section replaced), in Typst
- Created separate printable feedback forms per layer (`.typ` files): `layer-1-feedback.typ`, `layer-2-feedback.typ`, `layer-3-feedback.typ`
- Built shared `template.typ` (Helvetica Neue, A4, compact tables, navy H1, rule-above-H2 styling, `changed-box`, `note-box`, `skill-table`, `fq`, `rating-row` helpers)
- Created `build-pdfs.sh` and `/build-pdfs` skill — one command compiles all PDFs to `pdf-files/`
- Removed old Markdown test scenario files; `.typ` is now the single source for all scenario documents
- Fixed page-break issues: Movement Phase, Progression, Quick Reference all wrapped `breakable: false`; Skill Reference table at 8.5pt to fit 15 rows without page split
- Fixed Layer 1 Rune income table: dropped confusing "Cumulative if unspent" column, simplified to income-per-player-turn ranges
- Redesigned feedback form layout: numbered questions with `fq()` helper, rating scales on separate lines, header info in 2-column grid
- Updated `/scenario` skill to write `.typ` files and create companion feedback form
- Updated `CLAUDE.md` architecture section to reflect Typst workflow and `typ-files/` layout

**Key rulings made (now canonical)**:
- Rune income: start of each player's own turn. Round 1: no collection.
- Standard attack survival: attacker stops on tile before target. Only occupies tile if target removed.
- Movement: free pathing (any route ≤ speed). Cannot pass through any piece.
- Bodyguard: Standard Attacks only. Skills always hit directly.
- Healing: no cap. Rune cap: none.
- Skill Path: blocked by all pieces (ally and opponent).

**Decisions made**:
- All scenario documents are `.typ` (Typst). No more Markdown for printable rule sheets or feedback forms.
- H2 headings have a rule line *above* them (not below).
- Feedback forms use `fq()` numbered questions — each question gets its own breathing room.

**Open items for next session**:
- Run Layer 1 (Economy Fix) playtest — print `pdf-files/layer-1-economy-fix.pdf` and `pdf-files/layer-1-feedback.pdf`

---

## Session 2 — 2026-04-18 — Tooling & Workflow

**Goal**: Set up custom Claude Code skills for recurring session workflows.

**What was done**:
- Created 6 custom skills in `.claude/skills/`: `start`, `wrapup`, `research`, `playtest`, `scenario`, `adr`
- Wrote full SKILL.md content for each, with correct frontmatter (user-only for `start`/`wrapup`, auto-triggerable for the rest)
- Updated `CLAUDE.md`: replaced "Session Workflow" section with a skills reference table, updated research protocol convention to reference `/research` skill
- Fixed skill directory naming: removed hyphens from `wrap-up`, `playtest-analysis`, `test-scenario` (Claude Code requires unhyphenated names)

**Key findings**:
- Claude Code custom skills must have directories and `name` fields without hyphens to resolve correctly
- Skills placed in project-level `.claude/skills/` are not detected until Claude Code is restarted

**Decisions made**:
- Only `/start` and `/wrapup` are user-only (`disable-model-invocation: true`)
- `/research`, `/playtest`, `/scenario`, `/adr` auto-trigger based on conversation context

**Open items for next session**:
- Run Layer 1 (Economy Fix) playtest — rule sheet is in `docs/test-scenarios/layer-1-economy-fix.md`

---

## Session 1 — 2026-04-17 — Initialisation

**Goal**: Read all baseline rules, create project infrastructure, populate living documents.

**What was done**:
- Read all 6 files in `baseline-rules/md-converted/`
- Created full project folder structure (`docs/`, `game-state/`)
- Created `CLAUDE.md` with project overview and conventions
- Created `game-state/CURRENT_DESIGN.md` — populated with all 8 identified systems, core loop, piece roster, design health scores, and target experience
- Created `game-state/OPEN_QUESTIONS.md` — 16 open questions across 4 priority tiers
- Created `game-state/NEXT_STEPS.md` — prioritised action items in 5 tiers + backlog

**Key findings**:
- The game has 8 interlocking systems: Turn Structure, Resource Economy, Progression, Skill System, Combat/Attack, Health/Armor, Terrain, and Skill Drafting
- The Skill System is the central hub — it touches every other system
- Combat has a **positive feedback loop** (losing Guards = weaker defense = more losses) that is the primary snowball risk
- The "Systems to Test" document reveals significant unresolved design space: at least 5 critical variants need playtesting
- Extended skill catalogue (ROE Skills) has ~30+ skills with German names and variant options — needs consolidation with the English rulebook list

**Most pressing design questions**:
1. Board size (OQ-1) — affects pacing fundamentally
2. Movement-Action link (OQ-3) — changes the entire feel of turns
3. Rune economy model (OQ-2) — determines whether the game rewards tactical play with resources

---

### Playtest 1 Analysis (added mid-session)

**Playtest**: Elias vs Pasco, 31.10.2025. Variants: 10x10, Automatic Runes, Unlinked.

**Transcribed and analysed**: Both feedback sheets (handwritten, German+English), both game logs (handwritten Rune/capture/slot tracking over ~28-35 rounds).

**Top findings**:
1. Game too long (~30 rounds, both rated 4/5). Endgame dragged 10+ rounds.
2. Rune economy too slow — first 6 rounds were skill-less. Pasco: "start at +2 gain."
3. Bodyguard Rule never triggered — Guards died too fast, adjacency-to-both too restrictive.
4. Shadow Shift (global swap) flagged as OP by Pasco.
5. Injured state almost never reached (2 DMG attacks skip it). Pasco suggested 3HP.
6. Defensive skills (Field Medic, Armorsmith) never used.
7. "Wait and pounce" feel — players saved up, then went all-in. Lacked moment-to-moment tension.

**What changed in the design**:
- Partially resolved: OQ-1 (10x10 baseline), OQ-3 (Unlinked baseline), OQ-5 (all pieces block)
- New critical questions: OQ-17 (Rune start rate), OQ-18 (3HP), OQ-19 (endgame accel), OQ-20 (Shadow Shift), OQ-21 (Bodyguard redesign)
- Updated priority: **3HP decision** is now the highest-leverage design choice — it cascades into 4+ other problems.

Full analysis: `docs/research/playtest-1-analysis.md`

---

### Architecture Exploration (mid-session)

**Core fantasy identified**: Spell/skill combos. "Discovering cool combos and finding the winning move" — both Elias and Pasco agreed this was the best part.

**Three Perplexity research threads run**:
1. `docs/research/wizard-chess-genre-landscape.md` — Onitama, War Chest, Tash-Kalar, Summoner Wars, etc.
2. `docs/research/competitive-card-fighters-landscape.md` — BattleCon, FaB, Yomi, Ashes Reborn, etc.
3. `docs/research/cognitive-load-game-design.md` — 3-5 variables per decision, meaningful vs overhead complexity.

**Three architecture directions proposed** (ADR-001):
- A: Streamlined Grid (Onitama/War Chest model)
- B: Card Fighter (BattleCon/FaB model)
- C: Spatial Hybrid (Summoner Wars/Ashes model)

**Elias feedback**: Direction A preferred. Direction B rejected ("just another card game" / "too deep into the core value"). Direction C lukewarm ("positioning spells don't have enough choice"). No dice/luck — perfect information only.

**Fourth research thread**: `docs/research/perfect-info-tactical-games.md` — Onitama, Hive, Arimaa, YINSH, Tash-Kalar, The Duke. How they handle escalation, dominant strategies, and endgame.

---

### ADR-002: Direction A+ Proposal and Feedback (late session)

**Proposed**: Monolithic overhaul — 8x8 board, 3 HP, unified AP, fewer pieces, YINSH penalty, economy skills.

**Elias feedback (critical)**:
- **Don't change everything at once** — can't attribute effects to individual changes. This is now a mandatory methodology rule in CLAUDE.md.
- YINSH capture penalty withdrawn — unfair when asymmetric (one player has Guards, other doesn't).
- Economy skills as slots withdrawn — 2 slots per Champion is too few to waste on economy.
- Damage escalation deferred — arbitrary, may not be needed.
- AP piece-freedom concern — a piece could rush the King with 3 AP. Multiple constraint models proposed.
- Guards shouldn't be obligatory first kills — they should be useful in endgame too.

**Result**: Direction A+ accepted as the DIRECTION, but implemented via **5 incremental test layers**, not a monolithic change.

### Test Layer Rule Sheets Written (session end)

- `docs/test-scenarios/layer-1-economy-fix.md` — Full standalone rule sheet. Only change: 6 start Runes, +2/round.
- `docs/test-scenarios/layer-2-three-hp.md` — Full standalone rule sheet. Only change: Champions/King at 3 HP.
- `docs/test-scenarios/layer-3-bodyguard-fix.md` — Full standalone rule sheet. Only change: adjacent to defender only.
- `docs/test-scenarios/layer-4-unified-ap.md` — Framework with 4 constraint models (A/B/C/D). Awaits L1-3 results.
- Layer 5 (board/pieces) — placeholder, awaits L4 results.

### Session end state

All living documents updated. Incremental testing methodology added to CLAUDE.md. Handover prompt written. Ready for next session to begin Layer 1 playtest.
