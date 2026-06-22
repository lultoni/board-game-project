# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — end of Session 29.*

---

## Current focus

**Digital engine implementation — Stack M ruleset.** Slice -1 (FEN serialisation) and Slice 0 (`setup_stack_m()` + setup-invariant validator) are in. Next session opens Slice 1: Move Phase plain movement + the first cut of `make` / `unmake`.

## Active stack

**Stack M — Game Length Cut.** The engine executes Stack M's body as written. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed in Session 29

1. **Slice -1 shipped** (commit `07e7dbd`) — `Position::to_fen()` / `from_fen()` + tagged `FenError` + 16 tests. Frozen spec at `crates/core_engine/SCENARIO_FORMAT.md`. Rust toolchain bumped to 1.96 so Tauri's `edition2024` deps compile. `pending_modifiers` spec corrected in `next_steps` id=9 body (single field, side-to-move-owned).
2. **Slice 0 shipped** — `Position::setup_stack_m()` produces the canonical Stack M start (P1 King@d1, P2 King@e8, 1K+5C+6G per side, money 6). New `from_fen_strict()` enforces setup invariants (piece counts + Kings on different files); plain `from_fen` stays structural-only so mid-game positions parse. 11 new tests; full `cargo test -p core_engine` is 30/30 green.
3. **Inbox cleanup** — two pre-existing `design/inbox/digital/` notes (evaluator philosophy + engine plan distillation) folded directly into `crates/core_engine/src/search/evaluator.rs` as module-doc comments, then deleted from the inbox. Knowledge lives adjacent to the code, not in orphan essays.
4. **Slice-status block added to `next_steps` id=8** — the rule-coverage matrix now carries an at-a-glance checklist of slice -1…slice 8.

## Immediate next action

**Slice 1 — Move Phase plain movement.** Introduce `make` / `unmake`, the movement-flavoured `Action(u32)` encoding, and legal-move generation for Guard (speed 2) / Champion / King (speed 1). Edge cases enumerated in `next_steps` id=8. Designer decision needed at slice start: **is diagonal movement legal in the Move Phase?** Stack M body is silent — file as an OQ if not pre-resolved. After slice 1, `gamedbg` CLI + `.scenario` runner (`next_steps` id=9 deliverables 2–3) become buildable.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"`

## Open methodological loose ends

- None outstanding.

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok.
- `session-29` row written.
- `next_steps` priorities contiguous (1, 2, 3, 4, 5, 6, 7, 8, 10).
