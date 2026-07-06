# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-06 — Session 42 end (Task 8 shipped).*

---

## Current focus

**Testing the v0.1.0 release artefacts + MP prebuilt smoke** — unchanged from Session 40/41 end. Task 8 (custom loadout manager) is now fully shipped and doesn't gate release, but the two testing blockers below still do.

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed since session 41

- **Session 42 (2026-07-06):** Task 8 fully shipped in nine commits (`40f284d` … `9bbf6db`). IDB v3 loadouts store, share-code (`L1:` base64url) + JSON export/import, dedupe on skill tuple, `/loadouts/` editor with mini-board + skill picker, per-side loadout pickers in setup (`sideLoadouts: { p1, p2 }` refactor of `match-store`), draft-from-custom dropdown with compatibility filter + randomised auto-fill, read-only mini-board preview on draft screen. Follow-up UX pass: shared `BackButton` component across seven routes, 3-rank board cutout in editor via viewBox crop, vertical editor stack, orientation toggle removed.

## Immediate next action

Unchanged from Session 40 end — Task 8 was interleaved work, not a shift in priorities:

1. Download and run each platform's v0.1.0 build (macOS .dmg, Linux .AppImage, Windows .msi).
2. Verify WS-relay multiplayer end-to-end in the prebuilt versions (not just in dev).
3. Test the MP setup-shared-picker branch once (unchanged code path, but now shares `sideLoadouts` refactor plumbing — worth a smoke test).
4. Publish the draft release once cross-platform smoke passes.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (design OQs, not digital implementation).

## Open loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — deferred
- ETA field in status snapshot always null — not computed yet
- MP loadout fairness story (blocks custom loadouts in MP) — deferred; noted in code
- v0.1.0 draft release — still awaiting cross-platform smoke test + publish

## DB sanity

Session 42 row inserted. `PRAGMA integrity_check` → ok.
