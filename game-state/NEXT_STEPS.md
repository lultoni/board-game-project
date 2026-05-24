# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-05-24 — Session 18 (Skill cards + onboarding feedback form shipped; Focus Strike + Move ruling; Nico-no-changes decision).*

---

## Priority 1 — Print packet for Nico's first game (2026-05-28)

- [ ] **Print for Stack A G2 + Nico onboarding (28.05.26)**:
  - `docs/test-scenarios/stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf` ×2
  - `docs/test-scenarios/shared/skill-cards.pdf` ×2 (one per player)
  - `docs/test-scenarios/shared/feedback-onboarding.pdf` ×1 (Nico only, fill first)
  - `docs/test-scenarios/stack-a-cleverness/stack-a-feedback.pdf` ×2 (both players, fill after onboarding form)
  - `docs/test-scenarios/shared/game-tracking.pdf` ×2
- [ ] **No rule changes for Nico** — full standard baseline draft. Decision logged in memory (`project_nico_first_game.md`).
- [ ] Track explicitly: Champion kill timing, combo bonus activations, cross-category vs Strike+Strike combos.
- [ ] Watch Rune Theft for Mode B (OQ-34).

## Priority 2 — Decide: digital playtest prototype yes/no

- [ ] **Sleep-on-it status**: still pending. Decision via ADR before any implementation.

## Priority 3 — Hold Stack B (Bodyguard Fix)

- [ ] **De-prioritised.** Playtest 3 showed Bodyguard activates organically once standoff dissolves. Re-evaluate after Stack A G2.

## Priority 4 — Decide next stack after Stack A G2

- [ ] Follow `TESTING_PLAN.pdf` decision tree once G2 data lands.
- [ ] Likely candidates: Stack F (Cleverness II — sente skills), or a dedicated stack addressing OQ-52 / OQ-53.

## Priority 5 — Skill Balance (carry-over watches)

- [ ] Monitor **Rune Theft** in Stack A G2 — confirm Mode A vs Mode B framing.
- [ ] Confirm **Shadow Shift** Range 2 feels right in play.
- [ ] Monitor **Blade Call** + combo bonus interaction.

---

## Backlog (no priority — pull when triggered)

**Pre-playtest polish (deferred from Session 18):**
- One-page player-facing intro / pitch (#2 from Session 18 list). Open question: does the intro replace `section-introduction()` in the rule sheet or sit alongside it? Revisit after Nico's onboarding feedback lands.
- Rule sheet ordering audit (#3 from Session 18 list). Deferred until Nico's data informs the reorder.
- ADR on tiered skill catalogue (#5 from Session 18 list). Important but not time-bound.

**Stacks waiting on triggers:**
- **Stack C (Pacing)**: King Lifetime HP, Armor Decay — trigger: first Champion kill past R20 in Stack A G2.
- **Stack D (Board)**: 8×10 (from OQ-52), 8x8, hex — hex gated on `/research hex vs square grid` (OQ-42).
- **Stack E (Draft)**: pool draft (OQ-35) + placement order (OQ-36+48) — trigger: after Stack A G2.
- **Stack F (Cleverness II)**: cascade trigger, Pin/Threatened, sente skills — trigger: Stack A G2 results inform urgency.
- **Stack G (Structure)**: unified AP framework — draft written, run after core stacks stabilise.
- **OQ-52 / OQ-53 dedicated stack** — may become its own stack rather than living in F.

**Research / brainstorm:**
- `/research` on hex vs. square grid (OQ-42) — before any hex stack proposed.
- `/research` on how board games track temporary effects on pieces.
- Brainstorm: Adjacency synergies / piece compatibility — connects OQ-51.

**Catalogue / system expansion:**
- Skill catalogue expansion — 10 new candidates staged in `docs/backpocket.md`. Target ~25 total. Gated on Stack A/B combat balance confirmation.
- First-player advantage mitigation (OQ-13b watch).
- In-game skill redraft (shop/auction/interval) — connects OQ-56 Problem B.
- Armor timing asymmetry — discuss after Stack A G2.

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

## Recently completed (Session 18)

- Physical skill cards shipped (`shared/skill-cards.pdf`): 15 cards on A4, 2×2 range matrix per card showing Default / +Focus / Injured / Inj.+Focus, per-skill Focus footnotes on Move cards.
- First-game onboarding feedback form shipped (`shared/feedback-onboarding.pdf`).
- **Ruling**: Focus Strike on Move skills — caster chooses activation OR effect range (not both). Documented in baseline-sections, skill cards, mechanics-evaluated.
- Lance Thrust effective Range 0 while Injured = cannot fire (derivation, not ambiguity). Memory written.
- Stack A + Stack B feedback forms + feedback-baseline converted from fixed `#v(2.7cm)` to `#v(1fr)`.
- Build script gained zsh guard.
- Hygiene principle 7 expanded with `1fr` over `#v(Ncm)` sub-rule.
- Decision logged: Nico plays standard baseline draft on 2026-05-28; onboarding data via the new form only.
