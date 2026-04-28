# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-04-28 — Session 9 (dynamic stack system, composable Typst rule sheets, playtest skill improvements)*

---

## ~~Priority 0: Decide Layer 2 Topic~~ DECIDED (Session 7)

**Layer 2 = Standard Attack Nerf + Multi-Champion Combo Bonus.**

Two-game test format in one session:
- **Game 1**: Standard attack deals 1 DMG (not 2). Everything else = baseline + Layer 1 economy.
- **Game 2**: Standard attack 1 DMG + multi-Champion combo bonus (+1 DMG when a second Champion's skill hits the same target in the same turn).

Play the more disruptive change (nerf) first. Game 2 is strictly additive — delta between games = combo bonus effect.

Full reasoning: `docs/decisions/ADR-003-rewarding-cleverness.md`

## ~~Priority 0: Write Layer 2 Rule Sheets~~ DONE (Session 7 cont.)

- [x] **Game 1 rule sheet**: `stack-a-cleverness/stack-a-game1-attack-nerf.typ` — standard attack 1 DMG with `⚡ CHANGED:` callout.
- [x] **Game 2 rule sheet**: `stack-a-cleverness/stack-a-game2-attack-nerf-combo.typ` — nerf + combo bonus with two `⚡ CHANGED:` callouts, examples, Blade Call interaction.
- [x] **Feedback form**: `stack-a-cleverness/stack-a-feedback.typ` — covers both games, tracks first Guard/Champion kill, standoff, combo attempts.
- [x] **PDFs built** via `build-pdfs.sh`.

## ~~Priority 1: Build Dynamic Testing System~~ DONE (Session 9)

- [x] **Composable Typst section functions** — `docs/test-scenarios/shared/baseline-sections.typ` with 16 parameterized functions. Layer files now ~50 lines each.
- [x] **All layer files refactored** — baseline, Stack A (G1+G2), Stack B, Stack G all use composable sections.
- [x] **Stack-based folder naming** — `stack-a-cleverness/`, `stack-b-guards/`, `stack-g-structure/`, `accepted-layer-1-economy/`.
- [x] **TESTING_PLAN.typ** — new PDF with 6+ stack definitions, Mermaid decision tree, entry conditions table, accepted layers table.
- [x] **`/scenario` and `/wrapup` skills updated** to use composable pattern and maintain TESTING_PLAN.
- [x] **`/playtest` skill improved** — Block B behavioral patterns, multi-agent independent transcription, OQ Metric Evaluation step.
- [x] **Feedback forms improved** — OQ-monitoring pattern in `feedback-baseline.typ`; Layer 2 and Layer 3 feedback forms updated with OQ-10, OQ-11, OQ-34, OQ-46 monitoring questions.
- [x] **All 12 PDFs rebuilt** successfully.

## Priority 1: Run Stack A Playtest

- [ ] **Print and play Stack A** — two games in one session.
  - Game 1 (`stack-a-cleverness/stack-a-game1-attack-nerf.pdf`): standard attack nerf only.
  - Game 2 (`stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf`): nerf + combo bonus.
  - Feedback form: `stack-a-cleverness/stack-a-feedback.pdf` — fill out after Game 2.
  - Tracking: `shared/game-tracking.pdf` — 1 per player per game.
  - Layer 1 economy carried forward (6 start Runes, +2/turn, +1 every 5 rounds).
  - **Track explicitly**: rounds to first Guard kill, rounds to first Champion kill, combo attempts, Rune totals.

## Priority 2: Run Stack B Playtest (Bodyguard Fix)

- [ ] **Print and play Stack B**: `stack-b-guards/stack-b-bodyguard-fix.pdf` + feedback + tracking.
  - Independent of Stack A — can run any time.
  - Only change: Bodyguard adjacency to defender only.

## Priority 3: Evaluate Stack A + B and Choose Next Stack

- [ ] After results in: follow `TESTING_PLAN.pdf` decision tree to pick highest-value next stack.
- [ ] If first Champion kill still past R20 → **Stack C (Pacing)** becomes Priority 1.
- [ ] If combo ceiling still low → **Stack F (Cleverness II)** next.
- [ ] If Guards feel irrelevant → extend Stack B evaluation.

## Priority 4: Skill Balance

- [ ] Monitor **Rune Theft** in Stack A — if dominant, test cost 4 (see `docs/backpocket.md`).
- [ ] Confirm **Shadow Shift** Range 2 feels right in play.
- [ ] Monitor **Blade Call** + combo bonus interaction.

---

## Backlog

- Stack C (Pacing): checkmate win condition, board/piece count — write when triggered
- Stack D (Board): 8x8 or hex — hex gated on `/research hex vs square grid` first (OQ-42)
- Stack E (Draft): pool draft (OQ-35) + placement order (OQ-36+48) — after Stack B accepted
- Stack F (Cleverness II): OQ-51 mechanical levers — after Stack A combo data
- Stack G (Structure): unified AP framework — draft written, run after core stacks stabilise
- Skill pool draft variant (OQ-35)
- Flexible piece placement variant (OQ-36)
- **Run `/research` on hex vs. square grid** (OQ-42) — before any hex layer proposed
- First-player advantage mitigation
- In-game skill redraft (shop/auction/interval) — Layer 6+ candidate (see backpocket)
- Armor timing asymmetry — discuss after Stack A
