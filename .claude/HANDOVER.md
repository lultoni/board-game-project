# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-26 — Session 34 end (Phase 6 frontend remediation shipped — Inspector→PlyRenderer + replay perf).*

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

### Where We Are (Session 34 end, 2026-06-26)

- **Frontend remediation plan complete.** All 6 phases of `game/frontend/REMEDIATION_PLAN.md` are shipped (T5, P4, S1-residual closed in S34). Only 8 explicitly-deferred items remain — see `REMEDIATION_PLAN.md` "Out of scope". None block anything.
- **Inspector now uses PlyRenderer.** `syncEngineToNode` drives `renderer.fastForwardTo(baseSnap, node.actions, len)`; piece identity slides between sibling nodes; effects animate on landing ply. POI labels via native `<dialog>` (no more `window.prompt`).
- **Replay scrubbing has stride-32 snapshot checkpoints.** 200-ply re-scrub: was N round-trips, now ≤ 31. Cache lives in the `PlyRenderer` factory closure; invalidates on base change; cleared by `reset()` / `dispose()`.
- **Shared AI-call shell at `lib/engine/ai-hooks.ts`.** `runAiCall` + `AiCallError { reason: "timeout" | "cancelled" | "engine" }` adopted at match `stepAi` + inspector `requestAiMoveAtDepth`. No timeouts wired today — future-proof seam.
- **Engine complete for Stack M** + L8 draft phase shipped (S33). Multiplayer is authoritative-host with cross-peer parity for effects/SFX/greying-out (S33). Inspector core (S31) + match-HUD export + Sandbox (S32) unchanged.

### Immediate Next Action

No lane forced by S34. Pre-existing picks from S33 still valid — ask which:
1. **Frontend** — `next_steps id=12` (Inspector L6.7d preview window primitive). Unblocks L6.7b + L6.8.
2. **Multiplayer hardening** — investigate IllegalActionInHistory + draft-route-when-in-play replay bugs (deferred during L7c).
3. **Design** — `oq-84` (greying-out semantics when bodyguard intercepts a Move-Attack). Argue through before next Stack M digital playtest.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be marked open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding lives in engine Focus + Move-skill resolvers; verify OQ status against current code.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-34';` | This session's narrative (Phase 6 remediation) |
| `SELECT body FROM next_steps WHERE id=12;` | Inspector L6.7d preview window primitive (highest-leverage frontend item) |
| `SELECT body FROM next_steps WHERE id=15;` | Inspector polish (draft handoff + radial wheel) |
| `SELECT id, priority, title FROM next_steps WHERE priority >= 20 ORDER BY priority;` | All inspector/frontend follow-ups |
| `SELECT body FROM adrs WHERE id='adr-006';` | Multiplayer lifecycle, lobby/reconnect UX, telemetry persistence |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT body FROM open_questions WHERE id='oq-84';` | New OQ (S33) — bodyguard intercept greying-out semantics |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/session.rs` | Match API (incl. `request_ai_move_forced` / `_at_depth`) |
| `game/crates/core_engine/src/game_logic/make_unmake.rs` | Skill resolvers + Move-kind apply/unmake |
| `game/crates/core_engine/src/game_logic/generator.rs` | Legal-action enumeration |
| `game/frontend/REMEDIATION_PLAN.md` | Frontend remediation tracker — all 6 phases shipped, 8 items deferred |
| `game/frontend/ARCHITECTURE.md` | Frontend layering doc (§9 updated S34) |
| `game/frontend/src/lib/board/ply-renderer.svelte.ts` | PlyRenderer factory (checkpoint cache lives here) |
| `game/frontend/src/lib/engine/ai-hooks.ts` | Shared AI-call error/timeout shell |
| `game/frontend/src/lib/inspector/PoiLabelDialog.svelte` | POI label modal (replaces window.prompt) |
| `game/frontend/src/routes/match/+page.svelte` | Match route — MP wrapper + runAiCall(stepAi) |
| `game/frontend/src/routes/draft/+page.svelte` | Draft route — wired through MP wrapper + engine `Phase::Draft` |
| `game/frontend/src/routes/multiplayer/+page.svelte` | Lobby — host/join/handoff/probe-first Rejoin |
| `game/frontend/src/routes/inspector/+page.svelte` | Inspector route (now drives via PlyRenderer) |
| `game/frontend/src/routes/replay/+page.svelte` | Replay route (passes plyHint for checkpoint seeding) |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
