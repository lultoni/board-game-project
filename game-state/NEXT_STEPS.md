# NEXT STEPS

*Prioritised action items. Update after each session.*

*Last updated: 2026-05-18 — Session 15 (Playtest 3 analysed; standard attack 1 DMG accepted into baseline; OQ-52 + OQ-53 raised; Stack B de-prioritised)*

---

## ~~Priority 0: Decide Layer 2 Topic~~ DECIDED (Session 7)

**Layer 2 = Standard Attack Nerf + Multi-Champion Combo Bonus.**

Two-game test format in one session:
- **Game 1**: Standard attack deals 1 DMG (not 2). Everything else = baseline + Layer 1 economy.
- **Game 2**: Standard attack 1 DMG + multi-Champion combo bonus (+1 DMG when a second Champion's skill hits the same target in the same turn).

Play the more disruptive change (nerf) first. Game 2 is strictly additive — delta between games = combo bonus effect.

Full reasoning: `docs/design-principles.md`

## ~~Priority 0: Write Layer 2 Rule Sheets~~ DONE (Session 7 cont.)

- [x] **Game 1 rule sheet**: `stack-a-cleverness/stack-a-game1-attack-nerf.typ` — standard attack 1 DMG with `⚡ CHANGED:` callout.
- [x] **Game 2 rule sheet**: `stack-a-cleverness/stack-a-game2-attack-nerf-combo.typ` — nerf + combo bonus with two `⚡ CHANGED:` callouts, examples, Blade Call interaction.
- [x] **Feedback form**: `stack-a-cleverness/stack-a-feedback.typ` — covers both games, tracks first Guard/Champion kill, standoff, combo attempts.
- [x] **PDFs built** via `build-pdfs.sh`.

## ~~Priority 1: Build Dynamic Testing System~~ DONE (Session 9)

(see prior log — all 12 PDFs rebuilt successfully)

## ~~Priority 1: Transcribe Latest Playtest~~ DONE (Session 15)

- [x] **`/playtest 3` complete** — `docs/research/playtest-3-analysis.md` produced. L2G1 confirmed working: standard attack 1 DMG, standoff dissolved, Bodyguard organic, Armor RPS loop functional.
- [x] **OQ verdicts updated**: OQ-37 confirmed (accept into baseline), OQ-40 resolved, OQ-21 significantly updated, OQ-11 confirmed, OQ-46 closed, OQ-34 inconclusive (Mode A/B reframe), OQ-10 inconclusive (rule clarity blocker), OQ-41 partially confirmed.
- [x] **OQ-52 + OQ-53 raised**: centre-of-board attractor problem and attrition-vs-regicide framing question.
- [x] **Backpocket updated**: 8×10 narrower board, starting-formation swap to expose King, "spec the game for a programmer", digital playtest prototype.
- [x] **mechanics-evaluated.md**: standard attack 1 DMG moved to Accepted-In-Baseline.

---

## Priority 1: Pre-Stack-A-Game-2 Prep Work (Session 16)

*Before deciding the next stack, address the issues surfaced by Playtest 3 that block clean future data.*

### 1a. Resolve baseline rule ambiguities

- [ ] **Lance Thrust + Injured Range penalty** (side-note #5; cost Elias R22). Lance Thrust is Range 1; the Injured "Range 2+ → Range 1" penalty is a no-op. Decide: ruling text — "Injured penalty does not apply to Range 1 skills" — and add to baseline.
- [ ] **Focus Strike + adjacent self-target interaction** (side-note #4). Decide: can Focus Strike extend Range 1 self-targeting skills (like Field Medic) to Range 2? Per side-note: yes (Focus Strike extends *any* skill); no, you can't magically self-target an adjacent skill via Focus Strike. Resolve and write into Focus Strike's skill text.
- [ ] **Update `ruleset-baseline.typ`** to reflect Standard Attack 1 DMG as canonical (was 2 DMG). Cascade to all Stack rule sheets that import baseline sections.

### 1b. Form / tracking sheet fixes

- [ ] **Add Standard Attack count column** to `shared/game-tracking.typ`. Mario's attack count is unknown because the form didn't ask — Playtest 3 surfaced this gap.
- [ ] **Bake Rune-gain + Skill-Slot scaling into the tracking sheet** (side-note #1). Reduces explanation burden + cognitive load at game start. Per Elias's feedback: rule explanation overhead is meaningfully eating game time.
- [ ] **Drop "cost" column** from skill draft section of tracking sheet (side-note #2 — not needed once you've drafted).
- [ ] **More writing space on feedback forms** (carried from Session 13 UX list — still relevant).

### 1c. Design-discussion brainstorms (no playtest yet)

- [ ] **OQ-52 (centre attractor) brainstorm session**. Pre-think solution candidates: central scoring tile, contested rune source, narrower board (`[TO DISCUSS]` 8×10 entry), formation rework. Constraint: no "queen" piece — use board-geometry / attractor-mechanics levers.
- [ ] **OQ-53 (King as real target) brainstorm session**. Pre-think candidates: starting-formation swap (`[TO DISCUSS]` entry), King-specific sente skills, mobility/safety asymmetry rewarding King advancement. Constraint: King must *participate*, not just *be more durable* (King Lifetime HP doesn't fix this alone).
- [ ] **OQ-52 + OQ-53 are deeply linked** — likely needs one combined brainstorm session.

### 1d. Decide: digital playtest prototype yes/no

- [ ] **Sleep-on-it status**: user wants to think on this for a few days. Decision via ADR before any implementation. Option to start with the no-build "spec the game for a programmer" exercise as a low-cost step.

### 1e. Combo-bonus scope question (blocks Stack A G2 rule sheet)

- [ ] **Resolve before printing Stack A Game 2**: should the combo bonus reward only Strike+Strike sequences, or any cross-category (Move-into-Strike, Mystic-into-Strike) coordination on the same target? Surfaced by Elias's organic Air Blast → Hook Pull at L2G1 — cross-piece, cross-category combos already happen *without* the bonus, so Game 2's purpose is now "does the bonus add to or crowd out cross-category play?" — needs a decision before the rule text is finalised.

---

## Priority 2: Run Stack A Game 2 (after pre-work above)

- [ ] **Print and play Stack A Game 2** (`stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf`) once 1a–1c are done.
  - Ideal: two experienced players (Elias + an experienced opponent — NOT a draft-asymmetric setup). The Mario inexperience confounded Playtest 3's draft data.
  - Track explicitly: Champion kill timing, combo bonus activations, *cross-category combos vs. Strike+Strike combos*, whether organic combos persist or get crowded out.
  - Watch Rune Theft for Mode B usage — confirm or deny dominance framing.

## Priority 3: De-prioritised — Stack B (Bodyguard Fix)

- [ ] **Hold Stack B.** Playtest 3 showed Bodyguard activates organically once standoff dissolves. Stack B may be solving an already-solved problem. Re-evaluate after one more experienced-player game (Stack A G2). If Bodyguard keeps triggering organically, close OQ-21 as resolved-by-side-effect and skip Stack B entirely.

## Priority 4: Decide next stack after Stack A G2

- [ ] After Stack A G2 results: follow `TESTING_PLAN.pdf` decision tree.
- [ ] Likely candidates: Stack F (Cleverness II — sente skills), or a dedicated stack addressing OQ-52 / OQ-53.
- [ ] Re-check checks: if first Champion kill creeps past R20 with combo bonus → Stack C (Pacing). If combo ceiling still low → Stack F. If Guards feel irrelevant → reopen Stack B.

## Priority 5: Skill Balance (carry-over)

- [ ] Monitor **Rune Theft** in Stack A G2 — confirm Mode A vs Mode B framing.
- [ ] Confirm **Shadow Shift** Range 2 feels right in play.
- [ ] Monitor **Blade Call** + combo bonus interaction.

---

## Backlog

- Stack C (Pacing): King Lifetime HP, Armor Decay — write when triggered
- Stack D (Board): 8×10 (new candidate from OQ-52), 8x8, hex — hex gated on `/research hex vs square grid` first (OQ-42)
- Stack E (Draft): pool draft (OQ-35) + placement order (OQ-36+48) — after Stack A G2
- Stack F (Cleverness II): cascade trigger, Pin/Threatened, sente skills — Stack A G2 results inform whether sente still urgent
- Stack G (Structure): unified AP framework — draft written, run after core stacks stabilise
- **OQ-52 / OQ-53 dedicated stack** — depending on brainstorm output, may become a Stack rather than living in F
- **Run `/research` on hex vs. square grid** (OQ-42) — before any hex layer proposed
- **Run `/research` on how board games track temporary effects on pieces** — blocks: Temp Armor, Shield duration, Active Guard-Bind
- **Brainstorm session: Adjacency synergies / piece compatibility** — connects OQ-51 (rewarding clever plays)
- **Skill catalogue expansion** — 10 new candidates staged in backpocket (Session 11). Target ~25 total. Gated on Stack A/B combat balance confirmation.
- **Improve `/playtest` skill**: add draft-pick extraction and analysis as standard step (parse drafted skills, usage frequency, must-pick/never-pick patterns). Already partially done; refine.
- First-player advantage mitigation
- In-game skill redraft (shop/auction/interval) — Layer 6+ candidate
- Armor timing asymmetry — discuss after Stack A G2

---

## Test-Scenario UX Improvements (Session 13 — still pending)

*Template/architecture changes for the next time rule sheets or feedback forms are rebuilt. Some entries below now overlap with Session 15 form-fix priorities (1b above) — fold those into a single rebuild pass.*

- [ ] **Separate .typ source from PDF output** — move all `.typ` files into a `src/` subfolder within each stack directory.
- [ ] **Rules PDFs read as "intended rules"** — move all meta-information to a detachable facilitator page at the front.
- [ ] **Feedback forms fully independent** — no cross-game references.
- [ ] **More physical writing space** — feedback forms and game-tracking sheets need larger answer areas and more whitespace.
