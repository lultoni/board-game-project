# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-25 — Session 33 end (L7c authoritative-host multiplayer + L8 draft + cross-peer parity shipped).*

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

### Where We Are (Session 33 end, 2026-06-25)

- **Engine complete for Stack M** + L8 draft phase shipped (`Phase::Draft`, `DraftTurn` action with apply/unapply, `setup_with_loadouts()`, pre-made First/Second/Third game loadouts).
- **Multiplayer is authoritative-host** as of L7c. One peer is AUTH (host) and owns the only engine that originates state; the other (joiner) runs a mirror that re-applies every committed action and reports Zobrist mismatches. The role-aware wrapper `createMpEngine` in `multiplayer-engine.ts` is the single funnel for all engine apply traffic (solo/host/joiner). Leader handoff in place via `promoteToHost`; displaced old-host falls through to joiner on Rejoin via `probeHost` first; IDB schema bumped to v2 with `joined_codes` store retro-added; banner state preserved across auto-redial via `softReconnectJoiner`.
- **Cross-peer parity** for effects, SFX, and greying-out shipped in-conversation post-L7c-step-6. Wrapper's remote `onApplied` snapshots pre-state from `match.position` (still pre-apply at that point) and runs the full `renderApplied` pipeline. Cross-peer bodyguard prompt + two-stage ally picker for Dash/Retreat retarget under Focus also shipped.
- **Inspector core (S31) + match-HUD export + Sandbox (S32)** unchanged.
- **Highest-leverage remaining items:** frontend follow-ups (Inspector L6.7d preview window primitive at `next_steps id=12`), and the deferred IllegalActionInHistory / draft-route-when-in-play replay bugs (snapshot codec or `phase-change` replay path).

### Immediate Next Action

User has been driving the multiplayer lane. Three viable picks — ask which:
1. **Frontend** — `next_steps id=12` (Inspector L6.7d preview window primitive). Unblocks L6.7b + L6.8.
2. **Multiplayer hardening** — investigate IllegalActionInHistory + draft-route-when-in-play replay bugs (out of scope for L7c, deferred).
3. **Design** — `oq-84` (greying-out semantics when bodyguard intercepts a Move-Attack). Argue through before next Stack M digital playtest.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be marked open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding lives in engine Focus + Move-skill resolvers; verify OQ status against current code.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id=24;` | L7c shipped scope — full file-of-record list + what is not covered |
| `SELECT body FROM next_steps WHERE id=12;` | Inspector L6.7d preview window primitive (highest-leverage frontend item) |
| `SELECT body FROM next_steps WHERE id=15;` | Inspector polish (draft handoff + radial wheel) |
| `SELECT id, priority, title FROM next_steps WHERE priority >= 20 ORDER BY priority;` | All inspector/frontend follow-ups |
| `SELECT body FROM sessions WHERE id='session-33';` | This session's narrative |
| `SELECT body FROM adrs WHERE id='adr-006';` | Multiplayer lifecycle, lobby/reconnect UX, telemetry persistence |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT body FROM open_questions WHERE id='oq-84';` | New OQ — bodyguard intercept greying-out semantics |
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
| `game/frontend/src/routes/match/+page.svelte` | Match route — wired through MP wrapper; renderApplied runs for remote commits too |
| `game/frontend/src/routes/draft/+page.svelte` | Draft route — wired through MP wrapper + engine `Phase::Draft` |
| `game/frontend/src/routes/multiplayer/+page.svelte` | Lobby — host/join/handoff/probe-first Rejoin |
| `game/frontend/src/routes/inspector/+page.svelte` | Inspector route |
| `game/frontend/src/lib/multiplayer-protocol-v2.ts` | Authoritative-host wire codec |
| `game/frontend/src/lib/multiplayer-engine.ts` | Role-aware engine wrapper (single apply funnel) |
| `game/frontend/src/lib/multiplayer.svelte.ts` | PeerJS lifecycle + softReconnectJoiner + probeHost |
| `game/frontend/src/lib/state/skill-targets.ts` | Skill target helpers (incl. two-stage ally picker) |
| `game/frontend/src/lib/state/match-store.svelte.ts` | Match-level reactive state (incl. `localSeat`, sandbox fields) |
| `game/frontend/src/lib/state/inspector-store.svelte.ts` | Tree-of-positions data model |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
