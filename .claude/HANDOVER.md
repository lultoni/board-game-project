# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-24 — end of Session 31 (Inspector core ships).*

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

### Where We Are (end Session 31, 2026-06-24)

- **Frontend Inspector core shipped.** `/inspector/` route is live with all four entry points (paste MatchLog, paste FEN, restore tree JSON, fresh draft via `/setup/`), tree-of-positions, POI bookmarks, "Play this position" handoff, and AI hint with proper iterative-deepening "Search continuously."
- **Engine track is paused at Slice 5** (last touched S30). Slice 6 (Focus + Charge wiring, end-of-turn clearance, Skill-Phase action curve, Zobrist) is still the canonical next engine work.
- **Three new Rust entry points** for inspector AI: `request_ai_move_forced` (HvH-friendly) and `request_ai_move_at_depth` (caller-driven ID with no time bound). Threaded all the way through WASM + Tauri + worker + both clients.
- **177 lib tests still green; engine clean release build; frontend `npm run check` + `npm run build` both clean.**
- **Nine follow-up `next_steps` rows captured** (priorities 20–28) covering the remaining inspector slices, frontend polish, and the engine exact-depth AI optimisation.

### Immediate Next Action

**Pick one of the two parallel tracks:**

**Track A — Engine Slice 6** (canonical next engine work, blocked since S30):
1. `apply_focus` / `apply_charge` resolvers — write `pending_modifiers` bits, debit money + decrement action.
2. Generator: consume Focus's +1 Range when iterating Strike-skill ranges (OQ noted in `generator.rs`).
3. `turn_manager::end_turn` — clear combo counters, `tracked_*`, `pending_modifiers`; advance round on flip-back to P1; disburse `2 + round_number/5` income.
4. Skill-Phase action budget by round (currently hardcoded 2 — see oq-69).
5. Zobrist hashing — populate `undo.zobrist_xor`.

**Track B — Inspector follow-up.** Highest leverage: `next_steps` id=12 (Preview window primitive, L6.7d) — unblocks both L6.7b (top-K) and L6.8 (skill tooltips). Full body: `sqlite3 design/design.db "SELECT body FROM next_steps WHERE id=12;"`.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Stack M body says "starts at 2 per turn, scaling up." Still no curve. Treat as constant 2 in Slice 6 until resolved.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Action-encoding decision pending; Slice 6 will surface it concretely when wiring Focus.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id='8';` | Rule-coverage matrix + engine slice 0–8 plan (priority 1) |
| `SELECT body FROM next_steps WHERE id=12;` | Inspector L6.7d preview window primitive (Track B starting point) |
| `SELECT id, priority, title FROM next_steps WHERE priority >= 20 ORDER BY priority;` | All inspector/frontend follow-ups |
| `SELECT body FROM sessions WHERE id='session-31';` | This session's narrative |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/session.rs` | Match API; new `request_ai_move_forced` + `request_ai_move_at_depth` live here |
| `game/crates/core_engine/src/game_logic/make_unmake.rs` | All skill resolvers (Slices 4+5) + Move-kind apply/unmake |
| `game/crates/core_engine/src/game_logic/generator.rs` | Legal-action enumeration |
| `game/frontend/src/routes/inspector/+page.svelte` | Inspector route (entry screen + tree + board + AI hint) |
| `game/frontend/src/lib/state/inspector-store.svelte.ts` | Tree-of-positions data model |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
