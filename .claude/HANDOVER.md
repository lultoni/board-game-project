# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — end of Session 28.*

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

### Where We Are (end of Session 28, 2026-06-22)

- **Digital architecture is locked.** ADR-005 accepted: Rust core (Cargo workspace, 3 crates) + Svelte 5 + TS + Tauri 2 + WASM + PeerJS/WebRTC P2P with commit-reveal lockstep + local-auto-save telemetry. `SELECT body FROM adrs WHERE id='adr-005';`
- **`game/` is scaffolded and compiling.** Cargo workspace at `game/Cargo.toml`. `core_engine` Layers 1–5 stubs with bitboard newtype, bit-packed mailbox, Position struct (bitboards-authoritative), `Action(u32)` + fat `Undo` record (Stockfish convention), transposition table with `best_move` for move ordering. Svelte 5 frontend with runtime-agnostic engine bridge (`__TAURI__` detection).
- **Slice plan is the binding contract.** `next_steps` priority 1 is the full rule-coverage matrix (slice 0…slice 8) enumerating every Stack M rule and edge case. Each slice ends with a debug-harness verification step.
- **Debug harness comes first** — slice -1, `next_steps` priority 2.

### Immediate Next Action

**Slice -1 — Debug harness.** Build the engine's I/O surface so every later slice is verifiable interactively:
1. `Position::from_fen()` / `Position::to_fen()` — FEN-like single-line position format (squares + side-to-move + phase + money + actions_remaining + modifier_bits).
2. `gamedbg` binary in `core_engine`: `show <fen>`, `legal <fen>`, `apply <fen> <action>`, `trace <fen> <action>` (dumps the Undo), `perft <fen> <depth>`.
3. `.scenario` plain-text test runner under `crates/core_engine/tests/scenarios/` — load FEN + expected legal moves / expected post-state, assert match.

After slice -1, run slice 0 (`Position::setup_stack_m()` + Guard moves + roundtrip Make/Unmake). Then slice 1…8 per the matrix.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id='9';` | Slice -1 debug harness spec (priority 2) |
| `SELECT body FROM next_steps WHERE id='8';` | Full rule coverage matrix + slice 0–8 plan (priority 1) |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision (latest) |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (foundation for `game/`) |
| `SELECT body FROM sessions WHERE id='session-28';` | This session's narrative |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |
| `SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;` | Active todos |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/` | Layers 1–5 stubs + bitboard/mailbox/Action/Undo/TT |
| `game/frontend/src/` | Svelte 5 UI (App, Board, engine bridge, multiplayer stub) |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |

### Open methodological loose ends

- None.
