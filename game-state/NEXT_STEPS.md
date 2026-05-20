# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-05-20 — Session 17 (Repo rework: skill repair, state-doc lifecycle, single-source-of-truth, pipeline parameterization, hygiene principles captured).*

---

## Priority 1 — Run Stack A Game 2 (READY TO PRINT)

- [ ] **Print and play Stack A Game 2** (`docs/test-scenarios/stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf`).
  - Ideal: two experienced players (Elias + experienced opponent — NOT draft-asymmetric).
  - Track explicitly: Champion kill timing, combo bonus activations, cross-category vs Strike+Strike combos, whether organic combos persist or get crowded out.
  - Watch Rune Theft for Mode B usage (OQ-34) — confirm or deny dominance framing.
  - Use updated `shared/game-tracking.pdf` — now has R+, SS, Atk columns pre-filled.

## Priority 2 — Decide: digital playtest prototype yes/no

- [ ] **Sleep-on-it status**: still pending. Decision via ADR before any implementation.

## Priority 3 — Hold Stack B (Bodyguard Fix)

- [ ] **De-prioritised.** Playtest 3 showed Bodyguard activates organically once standoff dissolves. Stack B may be solving an already-solved problem. Re-evaluate after Stack A G2 — if Bodyguard keeps triggering organically, close OQ-21 as resolved-by-side-effect and skip Stack B entirely.

## Priority 4 — Decide next stack after Stack A G2

- [ ] Follow `TESTING_PLAN.pdf` decision tree once G2 data lands.
- [ ] Likely candidates: Stack F (Cleverness II — sente skills), or a dedicated stack addressing OQ-52 / OQ-53.
- [ ] Re-check thresholds: if first Champion kill creeps past R20 → Stack C (Pacing). If combo ceiling still low → Stack F. If Guards feel irrelevant → reopen Stack B.

## Priority 5 — Skill Balance (carry-over watches)

- [ ] Monitor **Rune Theft** in Stack A G2 — confirm Mode A vs Mode B framing.
- [ ] Confirm **Shadow Shift** Range 2 feels right in play.
- [ ] Monitor **Blade Call** + combo bonus interaction.

---

## Backlog (no priority — pull when triggered)

**Stacks waiting on triggers:**
- **Stack C (Pacing)**: King Lifetime HP, Armor Decay — trigger: first Champion kill past R20 in Stack A G2.
- **Stack D (Board)**: 8×10 (from OQ-52), 8x8, hex — hex gated on `/research hex vs square grid` (OQ-42).
- **Stack E (Draft)**: pool draft (OQ-35) + placement order (OQ-36+48) — trigger: after Stack A G2.
- **Stack F (Cleverness II)**: cascade trigger, Pin/Threatened, sente skills — trigger: Stack A G2 results inform urgency.
- **Stack G (Structure)**: unified AP framework — draft written, run after core stacks stabilise.
- **OQ-52 / OQ-53 dedicated stack** — may become its own stack rather than living in F.

**Research / brainstorm:**
- `/research` on hex vs. square grid (OQ-42) — before any hex stack proposed.
- `/research` on how board games track temporary effects on pieces — blocks: Temp Armor, Shield duration, Active Guard-Bind.
- Brainstorm: Adjacency synergies / piece compatibility — connects OQ-51.

**Catalogue / system expansion:**
- Skill catalogue expansion — 10 new candidates staged in `docs/backpocket.md`. Target ~25 total. Gated on Stack A/B combat balance confirmation.
- First-player advantage mitigation (OQ-13b watch).
- In-game skill redraft (shop/auction/interval) — connects OQ-56 Problem B.
- Armor timing asymmetry — discuss after Stack A G2.

**Tooling improvements:**
- Improve `/playtest` skill: add draft-pick extraction and analysis as standard step. Partially done; refine.

---

## Test-Scenario UX Improvements (open since Session 13)

*Template/architecture changes for the next time rule sheets or feedback forms are rebuilt.*

- [ ] **Separate .typ source from PDF output** — move all `.typ` files into a `src/` subfolder within each stack directory.
- [ ] **Rules PDFs read as "intended rules"** — move all meta-information to a detachable facilitator page at the front.
- [ ] **Feedback forms fully independent** — no cross-game references.
- [ ] **More physical writing space** — feedback forms and game-tracking sheets need larger answer areas and more whitespace.

---

## Recently completed (Session 17)

- Repo rework: skills repaired (ghost refs gone, layer→stack rename complete); OPEN_QUESTIONS split into live + archive; STATUS.md added; mechanics-evaluated columns expanded; session log moved out of old-game-versions/; CLAUDE.md trimmed and gained hygiene principles section.
