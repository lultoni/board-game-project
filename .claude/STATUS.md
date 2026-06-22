# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — end of Session 30 (Slices 1–5 complete).*

---

## Current focus

**Digital engine implementation — Stack M ruleset.** Session 30 absorbed the audit pass plus Slices 1–5 of the engine. The core engine now generates and applies the full Move Phase (plain moves + Move-Attack + Bodyguard), terminal-state detection, all 5 Strike-skill resolvers, and all 8 Shield/Move-class resolvers. 177 lib tests green; clean release build. Next: Slice 6 — Focus + Charge wiring, end-of-turn clearance, action-budget curve.

## Active stack

**Stack M — Game Length Cut.** The engine executes Stack M's body as written. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

1. **Audit pass** (commit `5e4f316`) — armor cap fixed to 2, Guards-no-skills enforced in FEN, `moved_this_phase` + `round_number` added to `Position` and FEN (9-field grammar), stub doc-comments rewritten.
2. **Slice 1** (commit `e938569`) — Move Phase plain movement + Move-Attack + Bodyguard. Chebyshev-BFS reachability, `make`/`unmake` for `ActionKind::Move`, `moved_this_phase` bookkeeping.
3. **Slice 2** (commit `04d301b`) — King-capture game-over signal + Bodyguard edge cases. `GameResult` + `Position::game_result`; generator emits empty on terminal states.
4. **Slice 3** (commit `55be8cc`) — Path/Range/Block primitives (`state::magic`, `state::path`) + Skill-Action framework (Action encoding, generator enumeration scaffold).
5. **Slice 4** (commit `09db1de`) — Strike-skill resolvers: Lance, Break, Steal, Hook, Tempest. Combo counter, pending_modifiers (Charge), champion_credit dedup.
6. **Slice 5** (this session, not yet committed at briefing time) — Shield-class + Move-class resolvers: Shield, Heal, Plate, Dash, Blast, Shove, Swap, Retreat. 8 designer decisions captured as **oq-73…oq-80** (Shield/Plate at cap illegal, Heal non-Injured illegal, Blast/Shove off-board asymmetry, all-movement-must-relocate, Swap ally-only, Retreat-no-guards illegal, friendly-push no combo, queen-ray geometry).

## Immediate next action

**Slice 6 — Focus + Charge wiring, end-of-turn clearance, Skill-Phase action-budget curve.** Deliverables:
1. `apply_focus` / `apply_charge` resolvers — set `pending_modifiers` bits.
2. Generator: consume Focus's +1 Range buff before iterating Strike-skill ranges (OQ noted at generator.rs).
3. `turn_manager::end_turn` — clear combo counters, `tracked_*`, `pending_modifiers`; advance round; disburse income.
4. Skill-Phase action budget by round (currently hardcoded 2).
5. Zobrist hashing — populate `undo.zobrist_xor`.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` (12 critical + 8 high as of session end; oq-73…oq-80 are resolved).

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok at session-30 end.
