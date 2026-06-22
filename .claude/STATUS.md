# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — end of Session 28.*

---

## Current focus

**Digital engine implementation.** Architecture (ADR-005) is locked; `game/` is a real Cargo workspace + Svelte 5 project that compiles clean. Next session's work is **slice -1**: the debug harness (FEN-like position I/O + `gamedbg` CLI + `.scenario` test runner) so every subsequent rule slice can be verified interactively before moving on.

## Active stack

**Stack M — Game Length Cut.** Stack M's body is the rule source the engine will execute. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed in Session 28

1. **ADR-005 accepted** — Rust core + Svelte 5 + TS + Tauri 2 + WASM + PeerJS/WebRTC P2P with commit-reveal + local-auto-save telemetry. `SELECT body FROM adrs WHERE id='adr-005';`
2. **`game/` scaffolded** — Cargo workspace (`core_engine`, `wasm_wrapper`, `tauri_wrapper`) + Svelte 5 frontend. `core_engine` compiles; mailbox + Action roundtrip tests pass.
3. **Audit pass** — analysis agent reviewed scaffold vs. Perplexity plan + Stack M rules. 6 corrections folded in: thin `Action(u32)` + fat `Undo` record (Stockfish convention), per-turn `tracked_enemies` for combo credit, `modifier_bits` for Focus/Charge, money widened to `u16`, bitboards-authoritative invariant documented, TT carries `best_move` for move ordering.
4. **Slice plan locked in** as `next_steps` priority 1: full rule-coverage matrix (slice 0…slice 8) with every Stack M rule + edge case enumerated. Binding contract for upcoming sessions.
5. **Debug harness** locked in as `next_steps` priority 2 (slice -1): FEN-like position format + `gamedbg` CLI + `.scenario` plain-text test runner.

## Immediate next action

**Slice -1 — Debug harness.** Implement `Position::from_fen()` / `to_fen()`, build a `gamedbg` binary in `core_engine` with `show / legal / apply / trace / perft` subcommands, and a `.scenario` runner that loads plain-text test files and asserts expected outcomes. Once that's in place, slice 0 (`Position::setup_stack_m()` + Guard moves) becomes verifiable from the command line.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"`

## Open methodological loose ends

- None outstanding.

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok.
- `session-28` row written; ADR-005's `decided_in='session-28'` FK satisfied.
- `next_steps` priorities contiguous (1, 2, 3, 4, 5, 6, 7, 8, 10).
