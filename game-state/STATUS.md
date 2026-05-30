# STATUS

*One-screen re-entry doc. Read this first after a gap. Updated by `/wrapup`.*

*Last updated: 2026-05-31 — Session 24 (close).*

## Current focus

**Stack L — Pole B Per-Turn-Draft Prototype is Active.** Standalone rule sheet now exists at `docs/test-scenarios/stack-l-per-turn-draft/`; ready to play digitally during the 3-week vacation window with Jonathan. *Pole A* = pre-game-draft (current game). *Pole B* = per-turn-draft (radical alternative — skills added during play, reusable while equipped, 12-equipped cap, shared action slots, no Money activation gate). Stack H — Armor Trim deprioritised to Queued. Multi-Champion Combo Bonus is in baseline.

## Active OQs (top 4)

1. **OQ-61 (two-pole framing)** — Pole A vs Pole B as parallel design tracks. Resolution: after first 2–3 Pole B prototype games, compare game-feel vs Pole A.
2. **OQ-11 (Queued; Stack H deprioritised)** — bundled dose remains the lead variant when Stack H runs. Re-evaluated after Pole B prototype data lands.
3. **OQ-62 (Pole A draft determinism)** — sequential draft drives "always better to react" pathology. Simultaneous-reveal proposal accepts limited PI loss in pre-game window only.
4. **OQ-63 (cross-pole fixing methodology)** — when a problem exists in both poles, test fixes once or per pole? Resolved on first encounter; user lean: twice for cleanness.

## Last session

2026-05-31 (Session 24): three goals tackled in order. (1) **Project-wide vocabulary simplification** — broad pass across docs/skills/Typst, 6 commits. (2) **Pole B rule sheet written** at `docs/test-scenarios/stack-l-per-turn-draft/` — standalone (no baseline imports), three-phase turn (Move → Draft → Skill), Move + Draft share a 4-action pool, Bodyguard between Move and Draft. (3) **PDF template redesign** — canonical `shared/template.typ` rebuilt: Inter typography, numbered H2 with calmer teal numerals, new `sk("Lance")` skill-chip helper, redesigned callout boxes (note teal / changed amber / designer muted), pagination fixed (outer `breakable: false` wraps removed from `baseline-sections.typ` + `stack-l`; lists/enums now block-unbreakable so sentences don't split). `#hr` separators removed from rule docs. All PDFs rebuild clean.

## Next action

**Run the first Pole B per-turn-draft prototype game digitally** with Jonathan during the 3-week vacation window. Use `docs/test-scenarios/stack-l-per-turn-draft/stack-l-per-turn-draft.pdf` as the rule sheet. After 2–3 games, compare game-feel vs Pole A and route per Stack L's *Routing on result* in TESTING_PLAN.
