# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-07-11 — Session 44 end (ns-37 sandbox/MP fix + settings pass + ns-35 help modal).*

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

### Where We Are (Session 44 end, 2026-07-11)

- **Three frontend commits landed on `main` this session, all verified (svelte-check clean, 292 tests):** ns-37 sandbox/MP anti-cheat fix (`0f12282`), settings-defaults + language-selector pass (`e2b2043`), and ns-35 part A in-game help modal (`f276410`).
- **`main` is 6 commits ahead of `origin/main` — nothing is pushed.** The `fix/combo-bonus-and-preset-mirror` branch referenced in older handovers is resolved: those B1/B2 fixes are already on `main`, no stray branches exist.
- **Sequencing unchanged:** clean up UI / close UX gaps first, THEN decide the next game change for the next playtest. No release-testing this session. Design OQs untouched.

### Immediate Next Action

1. **Designer visual check first:** run `cargo tauri dev` from `game/crates/tauri_wrapper` (NOT `game/relay`) and eyeball the ns-35 help modal — Help button placement next to the gear, opens over the board & closes back with no navigation, all three tabs, live switch to Deutsch via Settings.
2. **ns-38** — unify duplicated UI components into shared primitives (skill-card primitive consumed by SkillInfoCard / SkillPicker / SquareEvalCard / HelpModal; panel/modal chrome). This is the deferred part B of ns-35. Feed ns-13/ns-14 tooltip work into the shared primitive.
3. **ns-36** is the low-priority QoL grab-bag — defer unless it shares components with ns-38.
4. **Then** (separate design mode): pick the next game change for the next playtest. Candidate levers: ns-32 (Focus 1→2), ns-34/oq-58 (first-mover tempo), oq-86 (loser-gets-money rebate).
5. Ask the designer whether to **push the 6 unpushed commits** to origin (and, separately, ns-39: what AI think-time default value they want).

### Open methodological loose ends

- **`main` 6 commits ahead of `origin/main`** — unpushed (pushing needs explicit designer OK)
- ns-35 part A manual visual verification pending in the running app
- v0.1.0 cross-platform release smoke test — still outstanding from Session 40/42 (ns-28, ns-29)
- ns-39 (AI think-time default — no value named yet), MP loadout fairness — deferred

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-44';` | This session — ns-37 + settings + ns-35 help modal |
| `SELECT body FROM next_steps WHERE id='38';` | Deferred UI-unification work (next anchor) |
| `SELECT body FROM next_steps WHERE id='35';` | ns-35 part A done / B deferred |
| `SELECT body FROM next_steps WHERE id IN ('36','39');` | QoL grab-bag + AI think-time TBD |
| `SELECT body FROM essays WHERE id='essay-playtest-6-analysis';` | Full P6 analysis |
| `SELECT body FROM open_questions WHERE id='oq-58';` | First-mover-loses standoff |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files

| Path | Purpose |
|------|---------|
| `game/frontend/src/lib/HelpModal.svelte` | NEW — help modal (ns-35 part A); inline skill list to be refactored in ns-38 |
| `game/frontend/src/lib/board/SkillInfoCard.svelte` | Skill display (wheel hover) — duplication target for ns-38 |
| `game/frontend/src/lib/board/SkillPicker.svelte` | 15-skill grid — duplication target for ns-38 |
| `game/frontend/src/lib/eval/SquareEvalCard.svelte` | Eval breakdown skill list — duplication target for ns-38 |
| `game/frontend/src/lib/ui/BackButton.svelte` | Existing shared component (model for ns-38 unification) |
| `game/frontend/src/lib/engine/skills.ts` | `SKILLS` registry + `CATEGORY_COLOR` (skill metadata source) |
| `game/frontend/src/routes/+layout.svelte` | Global chrome — Help + Settings buttons |
| `game/frontend/src/lib/multiplayer-engine.ts` | MP wrapper; `ensureLiveEngine` (ns-37 fix) |
| `design/design.db` | Source of truth (binary; committed) |
| `.claude/STATUS.md` | One-screen re-entry summary |
