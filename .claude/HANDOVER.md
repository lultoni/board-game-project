# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-29 — Session 38 end (NN Trainer Debug + Gauntlet Speed).*

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
4. Check `design/inbox/brainstorm/`, `design/inbox/ai-chats/`, and `design/inbox/digital/` for new dumps from the designer.
5. Check `design/raw/playtest-photos/` for any new playtest folders since last session.

### Where We Are (Session 38 end, 2026-06-29)

- **Training Observatory UX is complete.** Controls bar, status strip, eval bar colours, standings bars, lineage nodes, inspector copy, sound effects on everything — all committed and clean.
- **NN trainer is more robust.** Panic catching, phase-boundary logging, ply cap 250, heuristic adjudication, per-preset `gauntlet_think_ms` (smoke=10ms). All committed.
- **Smoke run has not completed cleanly.** Time-bounded search at 10ms/ply is still slow on CPU. The engine is doing real search work even at tiny budgets. Suspected fix: switch smoke to depth-1 fixed-depth (pass `time_ms=0`, `max_depth=1` to `find_best_with_evaluator`).
- Release workflow (`v0.1.0-rc1`) from S37 still unrun.

### Immediate Next Action

**Make the smoke gauntlet use depth-1 fixed-depth search** so it completes in seconds. In `gauntlet.rs`: when `time_ms == 0`, pass `max_depth=1` to `find_best_with_evaluator` instead of `TIME_BOUNDED_MAX_DEPTH=64`. Set `smoke.gauntlet_think_ms = 0`. Verify smoke completes in <30s and prints `[training] run finished`.

### Open methodological loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — deferred
- ETA field in status snapshot always null — not computed yet
- Release workflow — drafted S37, never run

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-38';` | This session — trainer debug + gauntlet speed |
| `SELECT body FROM sessions WHERE id='session-37';` | Backend flexibility + release pipeline |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files

| Path | Purpose |
|------|---------|
| `game/crates/nn_trainer/src/gauntlet.rs` | `play_match_with_callback`, `tier1_fitness`, `mirrored_bo3`, ply cap |
| `game/crates/nn_trainer/src/run.rs` | `RunConfig` presets, `gauntlet_think_ms`, orchestrator loop |
| `game/crates/tauri_wrapper/src/lib.rs` | `start_training_run` with `catch_unwind` |
| `game/frontend/src/routes/training/+page.svelte` | Training Observatory shell, `STALE_MS` |
| `design/design.db` | Source of truth (binary; committed) |
| `.claude/STATUS.md` | One-screen re-entry summary |
