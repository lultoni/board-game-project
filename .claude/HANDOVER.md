# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-07-10 — Session 43 end (P6 analysis + B1/B2 fixes + idea-capture sweep).*

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

### Where We Are (Session 43 end, 2026-07-10)

- **P6 (Elias vs Dorian) is analysed and its ideas are all captured.** Combo widening accepted and combo mechanic is now **done/closed**. Game length substantially met. New live pattern: mid-game **first-mover-loses** standoff (oq-58) — a tempo problem, NOT a draw problem.
- **Two engine bugs fixed** on branch `fix/combo-bonus-and-preset-mirror` (committed, **NOT merged, NOT pushed**): B1 combo-bonus ruling, B2 preset P2 loadout mirror.
- **Focus is now the sequencing pivot:** the designer wants to **clean up broken/bugged UI first**, THEN decide the next game change for the next playtest. This session did no release-testing.

### Immediate Next Action

**UI cleanup first, in priority order:**
1. **ns-37** — fix the sandbox-in-MP false anti-cheat "engine disagreed" bug. Sandbox exploration must be isolated from the authoritative MP-validation engine (or re-sync from last `committed`/`snapshot` on sandbox exit). Surface: `multiplayer-engine.ts`, sandbox entry/exit in `/match/`.
2. **ns-35** — build the in-game help/reference surface (button next to settings, reachable from any screen) AND unify duplicated UI components (skill cards/tooltips/buttons). Feed ns-13/ns-14 tooltip work into shared primitives.
3. **ns-36** is the low-priority QoL grab-bag — defer unless it shares components with ns-35.
4. **Then** (separate design mode): pick the next game change for the next playtest. Candidate levers: ns-32 (Focus 1→2), ns-34/oq-58 (first-mover tempo), oq-86 (loser-gets-money rebate).
5. Ask the designer whether to merge/push `fix/combo-bonus-and-preset-mirror`.

### Open methodological loose ends

- Branch `fix/combo-bonus-and-preset-mirror` — committed, not merged, not pushed (awaiting designer OK)
- v0.1.0 cross-platform release smoke test — still outstanding from Session 40/42
- A5 replay parity; ETA field null; MP loadout fairness — all still deferred

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-43';` | This session — P6 + B1/B2 + idea sweep |
| `SELECT body FROM essays WHERE id='essay-playtest-6-analysis';` | Full P6 analysis |
| `SELECT body FROM next_steps WHERE id IN ('35','36','37');` | UI cleanup work items |
| `SELECT body FROM open_questions WHERE id='oq-86';` | Loser-gets-money design question |
| `SELECT body FROM open_questions WHERE id='oq-58';` | First-mover-loses standoff |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files

| Path | Purpose |
|------|---------|
| `game/frontend/src/lib/multiplayer-engine.ts` | Role-aware MP wrapper (anti-cheat validation path — ns-37) |
| `game/frontend/src/routes/match/+page.svelte` | Match route; sandbox entry/exit lives here |
| `game/frontend/src/lib/ui/BackButton.svelte` | Existing shared component (model for unification — ns-35) |
| `game/frontend/src/lib/board/SkillPicker.svelte` | 15-skill grid (candidate shared primitive) |
| `game/crates/core_engine/src/game_logic/skills.rs` | `mirror_loadout` (B2 fix) |
| `game/crates/core_engine/src/game_logic/make_unmake.rs` | combo-bonus ruling (B1 fix) |
| `game/frontend/src/lib/state/draft.ts` | `mirrorLoadout` parity + presets |
| `design/design.db` | Source of truth (binary; committed) |
| `.claude/STATUS.md` | One-screen re-entry summary |
