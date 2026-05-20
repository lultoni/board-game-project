# Session Log

*Per-session narrative log. Newest sessions at the top. `/wrapup` adds a new entry per session.*

*This used to live in `old-game-versions/README.md`. Moved to `game-state/` in Session 17 because it is active state, not archive material.*

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
