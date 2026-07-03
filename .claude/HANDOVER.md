# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-07-03 — Session 40 end (WS relay + v0.1.0 release).*

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

### Where We Are (Session 40 end, 2026-07-02)

- **v0.1.0 is built and waiting.** GH Actions Release workflow ran successfully on the `v0.1.0` tag. All 5 matrix artefacts exist: macOS .dmg, Linux .AppImage (CPU + CUDA), Windows .msi/.exe (CPU + CUDA). It is a draft release — not yet published.
- **Multiplayer now uses a WS relay.** PeerJS/WebRTC replaced with a custom WS relay hosted on Fly.io (`boardgame-relay.fly.dev`). Works on all platforms including Linux. Protocol documented in `game/PROTOCOL_TRACE.md`.
- **Engine is stable.** Combo bonus self-grant bug fixed (session 39). Bodyguard DTO projected into position snapshots. Linux audio/animation workarounds in place.

### Immediate Next Action

**Test the v0.1.0 release artefacts on actual hardware.** Download from the GitHub draft release, install/run on each target platform, and verify:
1. App launches and plays a normal game.
2. Multiplayer via WS relay works end-to-end (host create → guest join → move relay → game completion).
3. If all platforms pass: publish the draft release.

### Open methodological loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — deferred
- ETA field in status snapshot always null — not computed yet
- v0.1.0 draft release — needs publish after cross-platform testing

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-40';` | WS relay + v0.1.0 release |
| `SELECT body FROM sessions WHERE id='session-39';` | Stabilisation (engine bugs, Linux, RC2) |
| `SELECT body FROM sessions WHERE id='session-38';` | NN Trainer Debug + Gauntlet Speed |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files

| Path | Purpose |
|------|---------|
| `game/relay/` | WS relay server (hosted on Fly.io) |
| `game/PROTOCOL_TRACE.md` | Full WS relay protocol documentation |
| `game/frontend/src/lib/multiplayer/websocket-transport.ts` | New WS transport (replaces transport.ts) |
| `game/frontend/src/lib/multiplayer/route-lifecycle.ts` | Route connect/disconnect lifecycle |
| `.github/workflows/release.yml` | Release matrix (injects relay secrets) |
| `design/design.db` | Source of truth (binary; committed) |
| `.claude/STATUS.md` | One-screen re-entry summary |
