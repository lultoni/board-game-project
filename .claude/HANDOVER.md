# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-07-06 — Session 42 end (Task 8 shipped).*

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

### Where We Are (Session 42 end, 2026-07-06)

- **Task 8 shipped.** Custom loadout manager complete: `/loadouts/` route (list/editor/import/export), IDB v3 `loadouts` store, `L1:` base64url share codes + JSON, dedupe on skill tuple, per-side setup pickers (`sideLoadouts` refactor of match-store), draft-from-custom dropdown with compatibility filter + auto-fill, read-only mini-board preview on draft screen. Shared `BackButton.svelte` rolled out across seven routes.
- **v0.1.0 is still built and waiting.** Draft release from Session 40 has not yet been tested on real hardware. WS relay (Fly.io) still the multiplayer transport.
- **Engine unchanged this session.** No design decisions; no OQs resolved. Pure feature engineering on the frontend.

### Immediate Next Action

**Test the v0.1.0 release artefacts on actual hardware** (unchanged from Session 40 end):
1. Download from the GitHub draft release, install/run on each target platform.
2. Verify multiplayer via WS relay works end-to-end (host create → guest join → move relay → game completion).
3. Smoke-test the MP setup path (single shared picker branch — code touched by Session 42's `sideLoadouts` refactor).
4. If all platforms pass: publish the draft release.

### Open methodological loose ends

- A5: Replay page parity (PlayerPanels, turn strip) — deferred
- ETA field in status snapshot always null — not computed yet
- MP loadout fairness story (custom loadouts disabled in MP) — deferred; noted in code
- v0.1.0 draft release — needs publish after cross-platform testing

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-42';` | Task 8 — Custom Loadout Manager |
| `SELECT body FROM sessions WHERE id='session-41';` | Task batch + Phase A/B search sweep |
| `SELECT body FROM sessions WHERE id='session-40';` | WS relay + v0.1.0 release |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files

| Path | Purpose |
|------|---------|
| `game/frontend/src/routes/loadouts/+page.svelte` | Loadout editor route |
| `game/frontend/src/lib/board/LoadoutBoard.svelte` | Mini-board (viewBox-croppable) |
| `game/frontend/src/lib/board/SkillPicker.svelte` | Shared 15-skill grid (drag + click modes) |
| `game/frontend/src/lib/storage/loadout-codec.ts` | `L1:` share code + JSON codec |
| `game/frontend/src/lib/storage/loadout-dedupe.ts` | Skill-tuple dedupe |
| `game/frontend/src/lib/ui/BackButton.svelte` | Shared back button |
| `game/frontend/src/lib/state/match-store.svelte.ts` | `sideLoadouts: { p1, p2 }` |
| `game/relay/` | WS relay server (Fly.io) |
| `game/PROTOCOL_TRACE.md` | WS relay protocol |
| `.github/workflows/release.yml` | Release matrix |
| `design/design.db` | Source of truth (binary; committed) |
| `.claude/STATUS.md` | One-screen re-entry summary |
