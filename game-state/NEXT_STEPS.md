# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-05-31 — Session 24 close (project-wide vocabulary simplification; Pole B rule sheet written; PDF template redesigned).*

---

## Priority 1 — Stack L — Pole B Per-Turn-Draft Prototype (Active)

Session 23 design discussion crystallised a two-pole framing: *Pole A* (current pre-game-draft) vs *Pole B* (per-turn-draft, radical alternative). Session 24 wrote the standalone rule sheet — the prototype is now playable with a printable ruleset. With the 3-week vacation window opening (Jonathan + digital prototype), Stack L claims the active slot to test whether Pole B produces a different game-feel and dissolves the Armor-as-late-game-tax problem at the structural level.

Pole B rule sheet: `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.pdf` (standalone — does not import baseline-sections). Three-phase turn (Move → Draft → Skill); Move and Draft share a 4-action pool; Skill Phase is free with consumable activations; Bodyguard between Move and Draft.

- [x] **Rule sheet written** (Session 24).
- [ ] **Run the first Pole B per-turn-draft prototype game digitally** with Jonathan (3-week vacation window).
- [ ] **After 2–3 games, compare game-feel vs Pole A** and route per Stack L's *Routing on result* in TESTING_PLAN.
- [ ] **Watch flags**: Pole B "unstoppable one-turn killer" burst (hoarding into single overwhelming turn — staged in backpocket as potential issue, not guardrail), cognitive load (drafting + playing + reading opponent's likely future picks).

## Priority 2 — Stack H — Armor Trim *(Queued; deprioritised Session 23)*

Stack H still has merit for Pole A (chassis-volume problem from P4 OQ-11 / Q-C1 remains valid), but Pole B may dissolve the Armor-as-tax problem at the source. Stack H runs only if Pole B prototype reveals Pole A is worth keeping as a parallel mode and chassis volume is the dominant remaining issue.

- [ ] **Draft Stack H rule sheet** in `docs/test-scenarios/stack-h-armor-trim/` *only after Pole B prototype data lands*. Bundled lead dose: Armor cap 3→2 *and* Plate +1→+2.
- [ ] **Build cheaper than break** risk noted as bigger than originally framed (user verbatim Session 23). Track Armor totals vs P4 baseline (14/22) when Stack H runs.
- [ ] **Within-stack rollback** if bundled dose stalls: cap 3→2 only, Plate unchanged.

## Priority 3 — Stack A G3 — Dual-Counter Combo *(Queued; gated on Stack H)*

Unchanged from Session 22. Design summary in `docs/backpocket.md` → "Combo Bonus — Dual-Counter + Widened Scope". Solves cross-category crowd-out (#3 P4) and late-game offensive lockout (#6 P4). Teaching-cost flag (G4) — two parallel counters; OQ-60 watches whether cognitive load is acceptable.

- [ ] **Draft Stack A G3 rule sheet** in `docs/test-scenarios/stack-a-combo-bonus/` after Stack H result lands. Include teaching aids (counter tokens, examples).
- [ ] **Run after Stack H** — only if Stack H trim does NOT auto-resolve the exchange-pit pattern (OQ-58).

## Priority 4 — Other Queued stacks

- [ ] **Stack K — Piece Count Reduction** *(Queued)* — single-variable: 3 Champions + 4 Guards + 1 King on current 10×10 board.
- [ ] **Stack J — Injured Trim** *(Queued)* — gated on Stack H. P4 partially confirmed OQ-57.

## Priority 5 — Pole A draft determinism (OQ-62)

Session 23 raised that the current sequential draft drives a "deterministic perfect game / always better to react" pathology. Proposal: simultaneous-reveal drafting (both players pick 2 at the same time, repeat). User accepts limited PI loss *only* in the pre-game window.

- [ ] **Decide whether this becomes its own stack** or gets bundled into a future Pole A revival run. Wait until after Pole B prototype data lands.

## Priority 6 — Skill Balance carry-over watches

- [ ] **Steal Mode B confirmed dominant (P4)** — both players pinned it as must-pick. Hold cost increase pending Pole B data.
- [ ] Confirm **Swap** Range 2 feels right in play (no P4 data).
- [ ] Monitor **Charge** + combo bonus interaction (now in baseline).

---

## Backlog (no priority — pull when triggered)

**Process / facilitation (deferred):**
- **Teacher-vocab-checklist enforcement** — DEFERRED (Session 22). Bundle with Phase B naming pass.
- **Resource rename "Runes" → "Money" — DONE** (vocabulary simplification pass).

**Cross-pole methodology (OQ-63):**
- When a fix targets a problem present in both poles, decide whether to test it in each pole separately or once-and-carry. User lean: twice for cleanness. Resolved on first encounter.

**Pre-playtest polish (deferred from Session 18):**
- One-page player-facing intro / pitch.
- Rule sheet ordering audit.
- ADR on tiered skill catalogue.

**Dormant stacks (waiting on triggers):**
- **Stack C — Pacing**: King Lifetime HP, Armor Decay — trigger: first Champion kill past R20.
- **Stack D — Board Geometry**: 8×10 (OQ-52), 8×8 (OQ-1b), hex (gated on `/research hex vs square grid`, OQ-42).
- **Stack E — Draft Flow**: pool draft (OQ-35), placement order (OQ-36+48). *May be subsumed or reshaped by OQ-62 simultaneous-reveal work.*
- **Stack F — Sente Skills**: cascade trigger, Pin/Threatened, midline pressure — trigger: only if Stack A G3 ran and exchange-pit pattern persists.
- **Stack G — Unified AP**: draft written, run after core stacks stabilise.

**Withdrawn / archived:**
- **Stack B — Bodyguard Fix** — withdrawn Session 22; folder deleted Session 23.
- **Stack A — Cleverness/Combo** — accepted into baseline Session 22, source archived to `old-game-versions/archived-stacks/stack-a-cleverness/` Session 23.
- **Stack I — Armor Rollback** — folded into Stack H as the smaller within-stack dose.

**P4 design ideas surfaced (Session 22 — staged in `docs/backpocket.md`):**
- **Plague skill** — fixes "Injured-state-as-payload that bypasses Armor stack."
- **Lucky Strike / Star Strike** — staged for further design pass.
- **Focus replacement** — fixes catalogue must-pick density.
- **Lance + Steal merge** — fixes catalogue redundancy.
- **Resource rename "Runes" → "Money" — DONE** (vocabulary simplification pass).
- Elias: paid Focus extension, "rusty thief" piece-design idea, small mini pre-game for new players.

**Session 23 design ideas (staged in `docs/backpocket.md`):**
- **Armor cap scales by round** (Pole-agnostic candidate) — keeps Armor's late-game role from compounding into the early game. Watch flag: yet another scaling rule on top of Money/actions may cross the cognitive-load line.
- **Items as defensive option** (cross-pole concern) — take a slot where a skill would otherwise sit; could be drafted, bought, or picked mid-game.

**OQs opened in Session 22:**
- **OQ-58** — Mid-game stickiness / "exchange pit". Watched under Stack H.
- **OQ-59** — Opening + endgame "don't know what to do" pattern.
- **OQ-60** — Cognitive load.

**OQs opened in Session 23:**
- **OQ-61** — Two-pole parallel design: Pole A vs Pole B (modes vs experiment-that-could-replace).
- **OQ-62** — Pole A draft information (sequential vs simultaneous reveal).
- **OQ-63** — Cross-pole fixing methodology.

**Research / brainstorm (deferred per Session 23 user direction):**
- How comparable games handle in-game skill acquisition.
- Defensive identity without HP/Armor walls.
- `/research` on hex vs. square grid (OQ-42).
- `/research` on how board games track temporary effects on pieces.

**Catalogue / system expansion:**
- Skill catalogue expansion — 10 new candidates staged in `docs/backpocket.md`. Target ~25 total.
- First-player advantage mitigation (OQ-13b watch).
- In-game skill redraft (shop/auction/interval) — note: Pole B *is* a form of in-game skill redraft; OQ-56 Problem B may be partially answered by Pole B prototype data.

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

## Recently completed (Session 24)

- **Project-wide vocabulary simplification** — broad rename pass across top-level docs, design-principles, systems, backpocket, mechanics-evaluated, research, game-state, images README, skill files, and Typst rule sheets. 6 commits. Aligns terminology between files and reduces jargon for both designer and new players.
- **Pole B (Stack L) standalone rule sheet** — `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.typ/.pdf`. Three-phase turn (Move → Draft → Skill); Move + Draft share a 4-action pool; Bodyguard between Move and Draft; Skill Phase free with consumable activations. Standalone (does not import baseline-sections).
- **Backpocket: three Pole B variants staged** — skills-cost-a-resource, per-Skill-Phase activation cap, permanently-equipped (non-consumable) drafted skills.
- **PDF template redesigned** (canonical `shared/template.typ` rebuilt). Inter typography throughout; H1 = 28pt display title (eyebrow dropped); H2 = numbered presence with calmer teal numerals + tight SECTION/title pair, sticky to following content; H3 = small-caps teal eyebrow; tables = cool grey header + charcoal hairline + light alt rows; new `sk("Lance")` chip helper for in-text skill references; callouts redesigned (note teal / changed amber, no longer red / designer muted).
- **Pagination fixes** — outer `breakable: false` wraps removed from `baseline-sections.typ` (12 wraps) and `stack-l` (10 wraps) — they were forcing half-empty pages. H2 heading kept `breakable: false, sticky: true` so SECTION + title never strand. Lists/enums set to `block(breakable: false)` so bullets don't split mid-content.
- **`#hr` separators removed from rule docs** (kept in feedback forms / game-tracking where they separate fillable form sections).
- **Scratch `template-experiments/` directory removed** after convergence; build script SKIP list trimmed back.

## Recently completed (Session 23)

- **`docs/research/path-y-defense-redesign.md` written** — full canonical writeup of the Session 23 defense + game-shape redesign discussion. Three diagnoses tested (A: Money curve — killed; B: HP magnitude — killed; C: Armor-as-late-game-tax — confirmed). Pole A vs Pole B framing. Pole B mechanics corrected and locked. Open risks tracked.
- **Two new design principles** added to `design-principles.md`: (6) game length is itself a form of attrition; (7) while core identity is unsettled, prefer fundamental shifts over variable tweaking (conditional — incremental methodology resumes primacy once core is settled).
- **Three new OQs**: OQ-61 (two-pole framing), OQ-62 (Pole A draft determinism), OQ-63 (cross-pole fixing methodology). OQ-11 status updated to Queued.
- **Three new backpocket entries**: Armor diagnosis anchor, Armor cap scales by round, Pole B one-turn-killer potential issue.
- **TESTING_PLAN.typ restructured**: Stack L — Pole B Per-Turn-Draft Prototype is the new Active. Stack H deprioritised to Queued. Per-stack routing preserved.
- **Multi-Champion Combo Bonus migrated into baseline** (`baseline-sections.typ` → `section-multi-champion-combo()`). BASELINE_VERSION → 2026-05-30. Quick Reference table updated.
- **`mechanics-evaluated.md` updated**: combo migration logged; new "Methodology / Design Decisions" section with Diagnosis A/B/C verdict and Pole framing.
- **Repo housekeeping**: deleted `stack-b-guards/` (withdrawn never-played), archived `stack-a-cleverness/` to `old-game-versions/archived-stacks/`, switched all Typst imports to root-relative form (`/docs/test-scenarios/shared/...`) so files survive future folder moves.
- **WHAT_TO_PRINT.md, README.md, HANDOVER.md, scenario skill** all cleaned of stale Stack A/B references.

## Recently completed (Session 22)

- Playtest 4 analysed end-to-end: 11 transcription files, `docs/research/playtest-4-analysis.md` synthesis, OQ verdicts cascaded, mechanics-evaluated.md updated.
- "Nico" → "Niko" project-wide rename (44 occurrences).
- OQ-11 confirmed via P4 evidence; Stack H promoted to Priority 1 (later deprioritised Session 23).
- OQ-21 confirmed as covariate; OQ-19 not triggered; OQ-41 closed; Q-D1 reading downgraded to contaminated.
- Post-analysis design discussion: OQ-38 reframed; dual-counter combo design drafted; Path A methodology decision; 3 new OQs (OQ-58/59/60); 6 new backpocket entries.
- TESTING_PLAN.typ rewritten: state lifecycle introduced (Active / Queued / Dormant / Resolved); decision tree replaced with per-stack routing.

## Recently completed (Session 21)

- All 3 feedback forms audited and updated for high-concept alignment.
- WHAT_TO_PRINT.md added; README sanitised; Section G in prototype feedback form brought to parity with paper.

## Recently completed (Session 20)

- All 11 high-concept open questions (Q-A1 → Q-F1) resolved.
- ADR-004 written and accepted: "Two minds, one puzzle" (Framing B) becomes canonical design intent.
- Q-B4 baseline change shipped (Move-Attack reframed as "a Move that ends on an enemy tile"). BASELINE_VERSION → 2026-05-26 (now 2026-05-30 with combo migration).
- `shared/teacher-vocab-checklist.typ/pdf` shipped.
