# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-06-21 — Session 26 (Stack M rule sheet finalised; ready to print).*

---

## Priority 1 — Stack M — Game Length Cut (ready to print, awaiting playtest)

Rule sheet + feedback form finalised at `docs/test-scenarios/stack-m-game-length-cut/`. Six bundled simultaneous changes (intentional methodology deviation per Principle 7, designer call: *"alles auf einmal — ich will schnellen progress sehen"*):

1. Board 10×10 → **8×8**
2. Armor cap 3 → **2**
3. Injured state: penalties removed (still 2 HP tracker; no speed cap, no Range −1)
4. Draw conditions removed entirely (not replaced)
5. Steal cost 3 → **4** (both Modes)
6. Multi-Champion Combo Bonus widened on two axes:
   - Counter **ticks** on movement-causing skills (Tempest, Hook, Blast, Shove, Swap when it relocates an enemy) by a new Champion, not just Strikes.
   - Bonus damage applies to **any** skill (Strike OR movement-causing) that affects a target with counter > 0 *(Session 26 design change — movement becomes a damage vector once counter loaded)*.

Hypothesis: a single coordinated cut delivers 30-60 min length + single-climax shape (Principle 8) without breaking the combo fantasy. Structural justification: the 12-economy map (`docs/research/game-economy-map.md`) shows each lever targets a specific compounding curve.

- [ ] **Run P6 — Stack M playtest.** Two experienced players preferred. Track rounds + wall-clock + first-Champion-kill round vs 30-60 min target. Print `stack-m-game-length-cut.pdf` (rule sheet) + `stack-m-feedback.pdf` (one per player). Game-tracking sheet + skill cards unchanged.
- [ ] **Per-axis rollback routing** (full table in `stack-m-game-length-cut.typ` Facilitator Notes block + TESTING_PLAN.typ Active section). Six failure modes, six surgical rollbacks. Methodology recovers on the next stack.

## Priority 2 — Stack H — Armor Trim *(absorbed into Stack M; isolation-fallback only)*

Armor cap 3→2 is one of Stack M's six bundled changes. Stack H as a standalone is *not* the next stack — but if Stack M's routing produces "rollback Armor only" or "rollback everything except Armor", Stack H steps in as the isolation stack for the Armor lever. The Plate +1→+2 component of the originally-bundled Stack H dose is NOT in Stack M and would re-enter via Stack H if needed.

- [ ] *Conditional*: if Stack M routing demands Armor isolation, draft Stack H rule sheet in `docs/test-scenarios/stack-h-armor-trim/` at that point. Bundled lead dose (cap 3→2 + Plate +1→+2) vs smaller dose (cap-only) still applies.

## Priority 3 — Stack A G3 — Dual-Counter Combo *(Queued; gated on Stack M result)*

Stack M widens the combo bonus to movement-causing skills (Stack A G3's "widen target counter" move, partly). Stack A G3's *attacker counter* (single Champion hits multiple targets → bonus) is NOT in Stack M and remains the candidate for Stack A G3 if the exchange-pit pattern persists post-Stack-M. Session 25 narrowing note in `docs/backpocket.md`: attacker counter felt too generous on reflection — narrow before shipping.

- [ ] **Run after Stack M** — only if Stack M's combo widening does NOT auto-resolve the exchange-pit pattern (OQ-58).

## Priority 4 — Sequenced after Stack M

- [ ] **Pre-Made Loadouts (OQ-65)** — Pole A onboarding fix. Sequenced after Stack M because Stack M's game-length cut should make wrong-loadout choices less punishing — natural pairing. Design in `docs/backpocket.md` → "Pre-Made Loadouts for New Players."
- [ ] **Stack K — Piece Count Reduction** *(Queued)* — single-variable: 3 Champions + 4 Guards + 1 King. Now a Stack M follow-up isolation stack if game length is still too long without stalling.
- [ ] **Stack J — Injured Trim** *(Queued)* — already absorbed into Stack M (Injured-no-penalty is one of the six changes). Stack J as a standalone returns only if Stack M's routing demands "Injured isolation".

## Priority 5 — Pole A draft determinism (OQ-62) *(live again)*

Session 23 raised that the current sequential draft drives a "deterministic perfect game / always better to react" pathology. Proposal: simultaneous-reveal drafting. With Pole A back as Active, OQ-62 returns to live status. Pre-made loadout selection should also use simultaneous reveal — natural coupling.

- [ ] **Decide whether this becomes its own stack** or is bundled into the pre-made loadouts stack via simultaneous loadout reveal. Likely the latter for the first run.

## Priority 6 — Tooling: Digital Prototype Persistence

P5 lost its game log because Jonathan refreshed the browser. Any future digital playtest (Pole B revival, or Pole A digital) must persist state to be analysis-grade. Until persistence ships, default back to paper for any "this game matters" run. Full notes: `docs/backpocket.md` → "Digital Prototype Persistence."

- [ ] **Flag for digital-prototype owner**: auto-save per turn, export-as-JSON / PDF available at any time, per-turn log of all state changes.

## Priority 7 — Skill Balance carry-over watches

- [ ] **Steal Mode B confirmed dominant (P4)** — both players pinned it as must-pick. Hold cost increase pending Stack H.
- [ ] Confirm **Swap** Range 2 feels right in play (no P4 data).
- [ ] Monitor **Charge** + combo bonus interaction (now in baseline).

---

## Backlog (no priority — pull when triggered)

**Process / facilitation (deferred):**
- **Teacher-vocab-checklist enforcement** — DEFERRED (Session 22). Bundle with Phase B naming pass.

**Pre-playtest polish (deferred from Session 18):**
- One-page player-facing intro / pitch.
- Rule sheet ordering audit.
- ADR on tiered skill catalogue.

**Dormant stacks (waiting on triggers):**
- **Stack L — Pole B Per-Turn-Draft (consumable variant)** — PAUSED Session 25 after P5. Other Pole B variants in `docs/backpocket.md` (permanently-equipped, activation-cap, resource-cost-on-activation) may be revived if Pole A track stalls again.
- **Stack C — Pacing**: King Lifetime HP, Armor Decay — trigger: first Champion kill past R20.
- **Stack D — Board Geometry**: 8×10 (OQ-52), 8×8 (OQ-1b), hex (gated on `/research hex vs square grid`, OQ-42). Now also a game-length lever (OQ-66).
- **Stack E — Draft Flow**: pool draft (OQ-35), placement order (OQ-36+48). *May be subsumed or reshaped by OQ-62 simultaneous-reveal work bundled into pre-made loadouts.*
- **Stack F — Sente Skills**: cascade trigger, Pin/Threatened, midline pressure — trigger: only if Stack A G3 ran and exchange-pit pattern persists.
- **Stack G — Unified AP**: draft written, run after core stacks stabilise.

**Withdrawn / archived:**
- **Stack L — Pole B Per-Turn-Draft Prototype (consumable variant)** — paused Session 25 after P5. See OQ-61 resolution and `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`.
- **Stack B — Bodyguard Fix** — withdrawn Session 22; folder deleted Session 23.
- **Stack A — Cleverness/Combo** — accepted into baseline Session 22, source archived to `old-game-versions/archived-stacks/stack-a-cleverness/` Session 23.
- **Stack I — Armor Rollback** — folded into Stack H as the smaller within-stack dose.

**P4 design ideas surfaced (Session 22 — staged in `docs/backpocket.md`):**
- **Plague skill** — fixes "Injured-state-as-payload that bypasses Armor stack."
- **Lucky Strike / Star Strike** — staged for further design pass.
- **Focus replacement** — fixes catalogue must-pick density.
- **Lance + Steal merge** — fixes catalogue redundancy.
- **Resource rename "Runes" → "Money" — DONE** (Session 24 vocabulary simplification pass).
- Elias: paid Focus extension, "rusty thief" piece-design idea, small mini pre-game for new players.

**Session 23 design ideas (staged in `docs/backpocket.md`):**
- **Armor cap scales by round** (Pole-agnostic candidate) — keeps Armor's late-game role from compounding into the early game.
- **Items as defensive option** (cross-pole concern) — take a slot where a skill would otherwise sit.

**Session 25 design ideas (staged in `docs/backpocket.md`):**
- **Pre-made loadouts for new players** — primary onboarding fix in Pole A.
- **Game length 30-60 min target** — multi-lever pacing pass as measurement axis.
- **Digital prototype persistence** — tooling requirement for any digital playtest.
- **Attacker-counter narrowing** (note appended to existing dual-counter entry) — single-piece-multi-Champ felt too generous on reflection.

**OQs opened in Session 22:**
- **OQ-58** — Mid-game stickiness / "exchange pit". Watched under Stack H.
- **OQ-59** — Opening + endgame "don't know what to do" pattern.
- **OQ-60** — Cognitive load.

**OQs opened in Session 23:**
- **OQ-61** — Two-pole parallel design: Pole A vs Pole B. *PARTIALLY RESOLVED Session 25 → Pole A continues.*
- **OQ-62** — Pole A draft information (sequential vs simultaneous reveal). *Live again post-P5.*
- **OQ-63** — Cross-pole fixing methodology. *RESOLVED on first encounter Session 25.*

**OQs opened in Session 25:**
- **OQ-64** — Felt PI vs formal PI under combinatorial breadth (P5 finding).
- **OQ-65** — Pre-made loadouts for new players (Pole A onboarding fix).
- **OQ-66** — Game length 30-60 min target (Principle 6 axis).

**Research / brainstorm (deferred per Session 23 user direction):**
- How comparable games handle in-game skill acquisition.
- Defensive identity without HP/Armor walls.
- `/research` on hex vs. square grid (OQ-42).
- `/research` on how board games track temporary effects on pieces.

**Catalogue / system expansion:**
- Skill catalogue expansion — 10 new candidates staged in `docs/backpocket.md`. Target ~25 total.
- First-player advantage mitigation (OQ-13b watch).
- In-game skill redraft — note: Pole B *was* a form of in-game skill redraft and surfaced felt-PI problems (OQ-64). Pre-made loadouts + short games (OQ-65 + OQ-66) is the new candidate answer to OQ-56 Problem B without going through Pole B.

**Tooling improvements:**
- Improve `/playtest` skill: add draft-pick extraction and analysis as standard step.
- **Digital prototype persistence** (Session 25 — blocking for any digital playtest).

---

## Test-Scenario UX Improvements (open since Session 13)

*Template/architecture changes for the next time rule sheets or feedback forms are rebuilt.*

- [ ] **Separate .typ source from PDF output** — move all `.typ` files into a `src/` subfolder within each stack directory.
- [ ] **Rules PDFs read as "intended rules"** — move all meta-information to a detachable facilitator page at the front.
- [ ] **Feedback forms fully independent** — no cross-game references.
- [x] **More physical writing space** — feedback forms converted to `#v(1fr)` distribution (Session 18).

---

## Recently completed (Session 25)

- **Playtest 5 ran** — Elias (P1) vs Jonathan (P2) on the digital Pole B prototype. Jonathan won after 15 rounds. No exported game log (browser refresh wiped state).
- **Insights captured** at `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`: Armor 3 still mandatory (cross-pole), play collapsed to pure reaction, felt-PI broke under combinatorial breadth, game short but empty.
- **OQ-61 partially resolved** → Pole A continues as Active track; Pole B (consumable variant) paused.
- **OQ-63 resolved on first encounter** → per-pole-revival, not once-and-carry.
- **OQ-64 / OQ-65 / OQ-66 / OQ-67 / OQ-68 opened** — felt-PI vs formal-PI; pre-made loadouts; game length target; Bodyguard removal; draw conditions.
- **OQ-11 cross-pole confirmation logged.**
- **Principle 8 promoted** to `design-principles.md` — *"The game shape is a single climax, not a sine wave."*
- **Backpocket entries added**: Session 25 Brainstorm — Post-P5 Direction-Setting (14 threads, "asan" reflections); Pre-Made Loadouts; Game Length 30-60 min Target; Digital Prototype Persistence; Attacker-Counter Narrowing note.
- **Economy map written** — `docs/research/game-economy-map.md` documents 12 economies, quantitative end-state (Money/Armor/Damage curves), 2.2M end-to-end-moves combinatorics, stalling root cause, win-condition alternatives parked. Structural justification for Stack M.
- **Stack M — Game Length Cut drafted** — `docs/test-scenarios/stack-m-game-length-cut/` (rule sheet + feedback form + PDFs). Six bundled changes; intentional methodology deviation per Principle 7; full per-axis rollback routing documented.
- **TESTING_PLAN.typ updated** — Active = Stack M; Stack H downgraded to "absorbed into Stack M; isolation-fallback only."
- **mechanics-evaluated.md updated**: Stack L moved to Withdrawn (with reason), Methodology rows added (Pole B outcome, Cross-pole first encounter, Stack M bundled-deviation justification).

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
