# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-28 — Session 37 end (Backend flexibility + clean install/release pipeline).*

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

### Where We Are (Session 37 end, 2026-06-28)

- **Trainer backend is runtime-switchable.** `nn_trainer::backend::BackendChoice` (Cpu / Wgpu / Cuda); features additive (default ndarray + wgpu, CUDA opt-in). `run_training` matches on `BackendChoice` and dispatches into the right `B: AutodiffBackend` monomorphisation. `NnEvaluator` stays CPU regardless; the cross-backend hop uses burn's wire-compatible `.mpk` recorder.
- **Tauri IPC + frontend wired.** `list_backends` command, Training Observatory has a Backend dropdown (preselection = build default, last choice in localStorage).
- **Release workflow drafted but unproven** — `.github/workflows/release.yml` matrix on `v*` tags produces macOS arm64 `.dmg`, Linux x86_64 `.AppImage`, and Linux x86_64 + CUDA `.AppImage`. Has never run.
- **Repo cleanup landed** — stale `archive/migrators/` + `archive/old-game-versions/` deleted; `archive/paper-pipeline/` relocated to `design/raw/paper-pipeline-archive/`.
- 36 commits ahead of `origin/main`; nothing pushed yet.

### Immediate Next Action

**Push, then tag `v0.1.0-rc1` and watch the release workflow.** First run will surface anything wrong with the CUDA toolkit action, AppImage glob patterns, or 22.04 build deps. Fix iteratively in `.github/workflows/release.yml` until all three artefacts land in a draft Release; install on Mac + CUDA box; verify the backend dropdown lists what each build supports.

Once rc1 is healthy, return to the **NN position-rater** thread (still the primary design/engineering focus per S35–S36 — eval is the bottleneck, S36 sweep results all provisional pending real eval).

### Open methodological loose ends (carried from S36)

- **oq-69 — Skill-Phase action progression curve.** Resolved in code (`make_unmake.rs:982-985`); OQ row status may still be open — verify.
- **oq-70 — Focus on Move-skills.** Encoding shipped. Verify OQ status.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-37';` | This session — backend flexibility + release pipeline |
| `SELECT body FROM sessions WHERE id='session-36';` | QS + evaluator bottleneck finding |
| `SELECT body FROM next_steps WHERE id=15;` | rc1 smoke-test checklist |
| `SELECT body FROM next_steps WHERE id=16;` | A3 cross-backend save/load test (deferred) |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `.github/workflows/release.yml` | Three-job release matrix (unproven) |
| `game/crates/nn_trainer/src/backend.rs` | `BackendChoice` + type aliases |
| `game/crates/nn_trainer/src/run.rs` | `run_training` dispatcher + `run_training_with::<B>` |
| `game/crates/tauri_wrapper/src/lib.rs` | `list_backends` + `start_training_run` (with `backend` arg) |
| `game/crates/tauri_wrapper/tauri.cuda.conf.json` | CUDA variant overlay |
| `game/frontend/src/routes/training/+page.svelte` | Training Observatory shell (backend dropdown) |
| `CONTRIBUTING.md` | Branch convention + pre-PR checks + release-cut steps |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
