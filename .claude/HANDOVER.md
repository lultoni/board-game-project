# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — end of Session 29.*

---

## Instructions for Claude: How to Maintain This Handover Prompt

**When to update**: At the end of every session (or when the user says "wrap up"), update this file with:
1. Current session number and date.
2. 2-3 sentence summary of what was accomplished.
3. Current "Where We Are" section (overwrite, don't append).
4. Current "Immediate Next Action".

**What NOT to put here**: Full design details, rule text, or long explanations. This is a pointer document — it tells you WHERE to look, not WHAT the answers are. Keep it under 80 lines.

---

## The Prompt

You are my board game design co-creator and systems architect. We are working on a 2-player tactical board game (working title: "(GAME NAME)") inside this repository.

### How to start this session

1. Read `CLAUDE.md` (orientation; tells you the DB owns the facts).
2. Read `.claude/STATUS.md` (one-screen re-entry doc).
3. Query the DB for current focus — example one-liners in CLAUDE.md "Working with the DB" section.
4. Check `design/inbox/brainstorm/`, `design/inbox/ai-chats/`, and `design/inbox/digital/` for new dumps from the designer. Mine load-bearing content into the DB.
5. Check `design/raw/playtest-photos/` for any new playtest folders since last session.

### Where We Are (end of Session 29, 2026-06-22)

- **Slice -1 + Slice 0 shipped.** Engine has FEN serialisation (`Position::to_fen()` / `from_fen()` / `from_fen_strict()`), the canonical Stack M starting position (`Position::setup_stack_m()`), and 30 passing tests. Rust toolchain on 1.96; Tauri compiles.
- **Spec doc frozen** at `crates/core_engine/SCENARIO_FORMAT.md`: FEN grammar, action-text grammar, `.scenario` file format, strict-vs-lax parse rules.
- **Inbox is clean** — evaluator philosophy folded into `crates/core_engine/src/search/evaluator.rs`.
- **Slice plan stays binding.** `next_steps` id=8 (rule-coverage matrix) carries a slice-status header; -1 and 0 are ticked.

### Immediate Next Action

**Slice 1 — Move Phase: plain movement.** Three deliverables:
1. `Action(u32)` movement encoding (origin square + dest square; phase = Move; piece type implicit via lookup).
2. `make(&mut Position, Action) -> Undo` + `unmake(&mut Position, Undo)` for movement actions only (Stockfish-style fat Undo; cleared mailbox slot on origin, populated on dest, bitboard flips).
3. Legal-move generation for Guard (speed 2) / Champion / King (speed 1) — cardinal movement, blocked by any piece, no off-board, one move per piece per phase, 2 actions per phase.

Edge cases (per `next_steps` id=8 Slice 1): cannot move 3 with Guard, cannot move 2 with Champion/King, cannot move through ally/enemy, cannot reuse a piece in the same phase, `EndPhase` becomes legal when actions hit 0.

**Designer decision needed before this slice ships**: is diagonal movement legal in the Move Phase? Stack M body is silent. File an OQ at slice start if not pre-resolved; default to NO until designer says otherwise.

Once Slice 1 lands, `gamedbg` CLI (`show / legal / apply / trace / perft`) and the `.scenario` runner become buildable — `next_steps` id=9 deliverables 2 + 3.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id='8';` | Rule-coverage matrix + slice 0–8 plan (priority 1) |
| `SELECT body FROM next_steps WHERE id='9';` | Debug harness status (FEN done; CLI + runner todo) |
| `SELECT body FROM sessions WHERE id='session-29';` | This session's narrative |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |
| `SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;` | Active todos |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/state/fen.rs` | FEN encoder/parser + strict validator |
| `game/crates/core_engine/src/state/position.rs` | `Position`, `setup_stack_m()` |
| `game/crates/core_engine/SCENARIO_FORMAT.md` | Frozen FEN + action-text + scenario-file spec |
| `game/frontend/src/` | Svelte 5 UI (App, Board, engine bridge, multiplayer stub) |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |

### Open methodological loose ends

- None.
