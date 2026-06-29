# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-29 — Session 38 end (NN Trainer Debug + Gauntlet Speed).*

---

## Current focus

**NN trainer smoke test.** The Training Observatory UX is complete and committed. The trainer compiles and runs but a full smoke run has not completed cleanly — the gauntlet phase takes too long on CPU even at 10ms/ply (time-bounded search at any budget is slower than depth-bounded).

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

- **Training Observatory UX** fully shipped: controls bar (2-row), status strip with coloured phase dot, EvalBar `color` prop, Standings win-rate bars + state chips + active-row icon, Lineage node sub-line WR%/games/date, NetworkInspector idle copy. Sound effects on all buttons/sliders/selects/back links/drag handles across all pages. Click pitch lowered (580Hz); tick voice added.
- **NN trainer reliability**: `catch_unwind` around `run_training` (panics now print to terminal); phase boundary `eprintln!` logging throughout `run.rs`.
- **Gauntlet speed**: `MAX_PLIES` 1000→250; heuristic adjudication at ply cap (P1 wins on score ≥0); `gauntlet_think_ms` per preset (smoke=10ms, medium/long=100ms); `play_match_with_callback` takes `time_ms: u64` directly; `Bracket::scaled_time_limit_ms(base)` for ratios.
- **UI staleness fix**: `STALE_MS` 5000→600000ms; Gauntlet snapshot written before each Tier-1 series + pre-series `ActiveMatch` populated.

## Immediate next action

Delete `game/runs/active`, restart `cargo tauri dev`, run smoke preset. Verify `[training] run finished` appears in <2 minutes. If still too slow: switch smoke gauntlet to depth-1 fixed-depth (0ms time limit, `max_depth=1`) rather than time-bounded — that removes the search budget entirely.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session.

## Open loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — logged, not started
- ETA field in status snapshot always null — not yet computed
- Release workflow (`v0.1.0-rc1`) — drafted in S37, never run

## DB sanity

Session 38 row inserted. `PRAGMA integrity_check` → ok.
