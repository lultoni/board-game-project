# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-26 — Session 34 end (Phase 6 frontend remediation — Inspector→PlyRenderer + replay perf).*

---

## Current focus

**Frontend remediation plan complete.** All 6 phases of `game/frontend/REMEDIATION_PLAN.md` are shipped; only 8 deliberately-deferred items remain (3 need ADRs, 1 is engine-side, 1 is a test harness, 3 are cosmetic). No design-side work this session.

## Active stack

**Stack M — Game Length Cut.** Engine and digital UI are Stack M-shaped. Still awaiting a real playtest. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

1. **Phase 6c — PlyRenderer ply checkpoints (P4).** Stride-32 snapshot cache; `fastForwardTo` restores from nearest checkpoint when savings ≥ 4 plies. 200-ply scrub: was N round-trips, now ≤ 31.
2. **Phase 6d — `lib/engine/ai-hooks.ts`.** `runAiCall` + `AiCallError` (reason: timeout|cancelled|engine). Adopted at match `stepAi` + inspector `requestAiMoveAtDepth`.
3. **Phase 6a — Inspector → PlyRenderer migration (T5).** Inspector no longer reimplements pieceIds bookkeeping. `syncEngineToNode` drives `renderer.fastForwardTo`; piece identity slides between sibling nodes (was: teleport); effects animate on landing ply.
4. **Phase 6b — POI label modal.** Native `<dialog>` replaces `window.prompt`.
5. Updated `ARCHITECTURE.md §9` + `REMEDIATION_PLAN.md` Phase 6 section. Deleted all 12 stale plan files from `~/.claude/plans/`. Verification: 220/220 vitest, 0 svelte-check errors, prod build clean.

## Immediate next action

No lane forced by this session. Pre-existing picks from S33 still valid:
1. **Frontend** — Inspector L6.7d preview window primitive (`next_steps id=12`). Unblocks L6.7b + L6.8.
2. **Multiplayer hardening** — deferred IllegalActionInHistory / draft-route-when-in-play replay bugs.
3. **Design** — argue through `oq-84` (bodyguard-intercept greying-out) before next Stack M digital playtest.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical + 10 high. Unchanged this session.

## Open methodological loose ends

- **oq-69** (Skill-Phase action progression curve) — resolved in code as `2 + (round_number-1)/10`. OQ row may still be marked open in DB; verify and resolve.
- **oq-70** (Focus on Move-skills) — encoding shipped; verify OQ status against current code.

## DB sanity

`PRAGMA integrity_check` → ok. Pre-existing dangling FKs in `open_questions` rows 10, 86, 87 (created_in pointing at non-existent sessions); not introduced this session.
