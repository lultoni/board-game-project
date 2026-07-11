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
4. Check `design/inbox/` for new dumps from the designer (`brainstorm-*`, `chat-*`, `digital-*`, `playtest-*-notes.md`).
5. Check `design/raw/playtest-photos/` for any new playtest folders since last session.

### Where We Are (Session 45 end, 2026-07-11)

- **Stack N is staged (design), awaiting P7 (playtest).** It targets the P6 mid-game "first-mover-loses" standoff (oq-58), root-caused this session as **ranged skill-kills being non-reciprocal** (attacker pays no positional cost). Three changes: Focus 1→2, max 1 move-attack/turn, strike-moves-caster. Full: `SELECT body FROM stacks WHERE id='stack-n';`.
- **Big structural cleanup landed (commit `be2affe`, +456/−1803):** `design/RULES.md` is now the canonical ruleset; the Typst paper-pipeline is archived to `design/_archive/`; the 3 inboxes are collapsed into one `design/inbox/`; the "one stack per experiment" methodology is retired (16 stack rows frozen for provenance, `/scenario` now parks levers in `backpocket`); 6 residual plan files deleted; all skills rewritten to query the DB / RULES.md.
- **The engine does NOT yet implement Stack N** — the three rules live in RULES.md marked ⧗-staged. ns-42 is the P7-blocking engine task.
- **`main` is unpushed** (7+ commits). No game code changed this session.

### Immediate Next Action

1. **ns-42 (P1, P7-blocking)** — implement Stack N's three rules in `game/crates/core_engine`: Focus cost 2, cap move-attacks at 1/turn, strike-moves-caster (after a Strike resolves, caster steps 1 tile toward target iff that tile is now empty; reuse existing movement resolution; Strike category only). Add regression tests for the strike-moves-caster edge cases. `SELECT body FROM next_steps WHERE id='42';`.
2. **Then P7** — playtest Stack N: does the standoff dissolve without re-inflating game length (watch for mid-30s–40s round counts)?
3. **ns-43 (P2)** — phase-aware evaluator: every eval term toggleable + parameterised; `evaluate()` detects game phase from the position, then picks which terms to compute + their per-phase weights. Independent of P7.
4. Ask the designer whether to **push** the unpushed commits, and ns-39 (AI think-time default value).

### Open methodological loose ends

- **`main` ahead of `origin/main`** — unpushed (pushing needs explicit designer OK)
- v0.1.0 cross-platform release smoke test — still outstanding from Session 40/42 (ns-28, ns-29)
- ns-39 (AI think-time default — no value named yet), ns-40 (bodyguard visual indicator), ns-41 (rendering decouple + research) — deferred

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-45';` | This session — Stack N staging + cleanup |
| `SELECT body FROM stacks WHERE id='stack-n';` | Stack N — the three staged lethality/standoff changes |
| `SELECT body FROM next_steps WHERE id='42';` | Engine implementation of Stack N (next anchor) |
| `SELECT body FROM next_steps WHERE id='43';` | Phase-aware evaluator rework |
| `SELECT body FROM open_questions WHERE id='oq-58';` | First-mover-loses standoff + non-reciprocity diagnosis |
| `SELECT id, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';` | The 3 live parked levers |
| `SELECT body FROM essays WHERE id='essay-playtest-6-analysis';` | Full P6 analysis |

### Key Files

| Path | Purpose |
|------|---------|
| `design/RULES.md` | **NEW — canonical ruleset (authoritative on conflict).** Read first each session. |
| `design/design.db` | Source of truth (binary; committed) |
| `design/inbox/` | Single fast-write staging folder (brainstorm-/chat-/digital-/playtest- prefixes) |
| `design/_archive/` | Frozen Typst paper-pipeline (historical) |
| `game/crates/core_engine/src/game_logic/` | Move/skill resolution — where ns-42 (Stack N rules) lands |
| `game/crates/core_engine/src/search/evaluator.rs` | Evaluator — where ns-43 (phase-aware rework) lands |
| `.claude/skills/scenario/SKILL.md` | Rewritten — parks levers in `backpocket`, not `stacks` |
| `.claude/STATUS.md` | One-screen re-entry summary |

