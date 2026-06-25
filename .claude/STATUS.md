# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-25 — Session 33 end (L7c authoritative-host multiplayer + L8 draft + cross-peer parity).*

---

## Current focus

**Multiplayer is now authoritative-host with cross-peer parity for effects, sounds, and greying-out.** The L7c arc (six steps + bug-fix sweep) plus L8 phases A–E shipped this session. Next likely lane: frontend follow-ups (Inspector L6.7d preview-window primitive) — or further multiplayer hardening if playtesting surfaces issues.

## Active stack

**Stack M — Game Length Cut.** Engine and digital UI are now Stack M-shaped. Still awaiting a real playtest. `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## What changed this session

1. **L8 phases A–E — engine-driven draft.** `Phase::Draft` variant + `DraftTurn` action with apply/unapply; `setup_with_loadouts()` constructor + Tauri/WASM commands; `/draft/` route engine-driven (DnD, glyphs, locked vs tentative, fixed-preset AI draft stand-in); pre-made First/Second/Third game loadouts (OQ-65 for new-player onboarding). `next_steps id=10` closed.
2. **L7c — authoritative-host multiplayer (six steps + bug-fix sweep).** New wire protocol v2 (`multiplayer-protocol-v2.ts`), role-aware engine wrapper (`multiplayer-engine.ts`) funnelling all apply traffic, joiner anti-cheat audit via mirror re-apply + Zobrist match, leader handoff (`promoteToHost`), IDB v1→v2 retro-add of `joined_codes` store, `softReconnectJoiner` preserving banner state, displaced-host probe-first Rejoin. `/draft/` + `/match/` migrated through wrapper. `next_steps id=24` records shipped scope.
3. **Cross-peer parity follow-ups.** `match.localSeat`; disconnect-on-exit + pong-age-out; bodyguard prompt where defender chooses; two-stage ally picker for Dash/Retreat retarget under Focus; effect/sound/greying-out parity (wrapper's remote `onApplied` now snapshots pre-state from `match.position` and runs full `renderApplied` so non-acting peer plays SFX, spawns effects, marks `usedThisPhase`).
4. **New OQ.** `oq-84` (high, p3) — design question on whether bodyguard-intercepted Move-Attack should still mark the attacker as used this phase.

## Immediate next action

User has been driving the multiplayer lane. Three viable picks:
1. **Frontend** — Inspector L6.7d preview window primitive (`next_steps id=12`).
2. **Multiplayer hardening** — deferred IllegalActionInHistory / draft-route-when-in-play replay bugs (snapshot codec or `phase-change` replay path).
3. **Design** — argue through `oq-84` before next Stack M digital playtest.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical + 10 high (new `oq-84` added).

## Open methodological loose ends

- **oq-69** (Skill-Phase action progression curve) — resolved in code as `2 + (round_number-1)/10`. OQ row may still be marked open in DB; verify and resolve.
- **oq-70** (Focus on Move-skills) — encoding shipped; verify OQ status against current code.

## DB sanity

`PRAGMA integrity_check` → ok. Pre-existing dangling FKs in `open_questions` rows 10, 86, 87 (created_in pointing at non-existent sessions); not introduced this session.
