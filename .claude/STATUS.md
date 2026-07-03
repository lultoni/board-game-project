# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-03 — Session 40 end (WS relay + v0.1.0 release).*

---

## Current focus

**Testing the v0.1.0 release artefacts.** The GH Actions release workflow ran successfully (all 5 matrix jobs). Draft release at GitHub Releases has macOS .dmg, Linux .AppImage (CPU+CUDA), Windows .msi/.exe (CPU+CUDA). Need to test the prebuilt binaries on actual hardware across platforms and verify multiplayer (WS relay, Fly.io) end-to-end in the shipped builds.

## Active stack

**Stack M — Game Length Cut.** Engine + UI are Stack M-shaped. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed since session 38

- **Session 39 (2026-06-30):** Stabilisation. Cargo.lock pinned (CI time-crate breakage). Engine bug fixed: combo bonus self-grant when same piece hooks twice (`relocate_piece` now updates dedup arrays). Linux fixes: AudioContext sandbox workaround, animation lag (DMABUF), "not available on Linux" notice for MP. Bodyguard DTO fix: `pending_bodyguard` now projected into `PositionViewDto`. RC2 release workflow succeeded.
- **Session 40 (2026-07-02):** PeerJS/WebRTC replaced with custom WS relay (`game/relay/`) hosted on Fly.io. New frontend transport layer: `websocket-transport.ts`, `transport-config.ts`, `route-lifecycle.ts`, `MultiplayerStatusStrip.svelte`. Protocol documented in `game/PROTOCOL_TRACE.md`. Release workflow now injects relay secrets. `v0.1.0` tag pushed — all 5 matrix artefacts produced (draft release).

## Immediate next action

Test the v0.1.0 release artefacts:
1. Download and run each platform's build (macOS .dmg, Linux .AppImage, Windows .msi).
2. Verify multiplayer over the WS relay works end-to-end in the prebuilt versions (not just in dev).
3. If all good, publish the draft release.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — unchanged this session (design OQs, not digital implementation).

## Open loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — logged, not started
- ETA field in status snapshot always null — not yet computed
- v0.1.0 is a draft release — needs publish after testing

## DB sanity

Sessions 39 + 40 rows inserted. `PRAGMA integrity_check` → ok.
