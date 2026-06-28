# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-28 — Session 37 end (Backend flexibility + clean install/release pipeline).*

---

## Current focus

**Cut a v0.1.0 release.** Eight infrastructure commits landed in S37 to make the trainer backend runtime-switchable (CPU vs GPU from the UI) and produce signed-or-at-least-bundled `.dmg` / `.AppImage` artefacts from a tag push. The release workflow is unproven — first push of `v0.1.0-rc1` is the smoke test.

Underlying design / NN-rater work from S35–S36 (evaluator bottleneck, QS retest pending real eval) is unchanged and still queued.

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped, awaiting a real playtest. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

- **F1** — Repo cleanup: deleted `archive/migrators/` + `archive/old-game-versions/`; relocated `archive/paper-pipeline/` → `design/raw/paper-pipeline-archive/`.
- **A1+A2** — `nn_trainer::backend` rewritten; backend Cargo features now additive (`backend-ndarray` + `backend-wgpu` default; `backend-cuda` opt-in). `run_training` is generic over `B: AutodiffBackend`; a top-level `match BackendChoice` dispatcher monomorphises into the right backend. Cross-backend acceptance hop uses burn's `.mpk` recorder; `NnEvaluator` stays CPU regardless.
- **B1** — Tauri `list_backends` command; `start_training_run` gains `backend` arg.
- **C1** — `/training` top-bar has a Backend dropdown (default from `BackendChoice::default_choice`, last choice persists in localStorage).
- **D1** — `.github/workflows/release.yml`: three-job matrix on `v*` tags (macOS arm64 `.dmg`, Linux x86_64 `.AppImage`, Linux x86_64 + CUDA `.AppImage`). `tauri.cuda.conf.json` overlay for the CUDA variant.
- **E1** — Root README Download section; `game/README.md` backend feature matrix; new `CONTRIBUTING.md`.

No design knowledge changed. No OQs resolved. No engine logic touched.

## Immediate next action

Push (`git push` — 36 commits ahead of `origin/main`), then tag `v0.1.0-rc1` and watch the Actions run. Expect the first run to fail somewhere — fix glob patterns / CUDA Toolkit action version / 22.04 deps iteratively until all three artefacts land in a draft Release. Then download + install on Mac and CUDA box and click through the backend dropdown.

After rc1 settles, return to the NN-rater work (S36's deferred evaluator-bottleneck thread) — see `design/inbox/digital/nn-rater-plan.md`.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical + 10 high. Unchanged this session.

## DB sanity

`PRAGMA integrity_check` → ok. Pre-existing dangling FKs in `open_questions` rows 10, 86, 87 (created_in pointing at non-existent sessions); not introduced this session.
