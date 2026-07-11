# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-07-11 — Session 46 end (ns-43 evaluator: 4 new terms + stage infra + per-position decider).*

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

### Where We Are (Session 46 end, 2026-07-11)

- **ns-42 done:** Stack N's three rules are implemented in the engine (Focus 2, 1 move-attack/turn, strike-moves-caster). RULES.md still marks them ⧗-staged (playtest-unconfirmed until P7).
- **ns-43 done (core):** the position evaluator (`game/crates/core_engine/src/search/evaluator/`) is now a **registry of parameterised, per-position-gated terms**. Added this session, one commit each, all golden+determinism-guarded: `guard_isolation` (**56.2%** self-play), champion `mobility`→real movement-space (**56.2%**), `champion_threat` (offensive+defensive, 52.5% — under-tuned), `endgame_closing` (asymmetric, End-stage-gated — correct but self-play never reached End so A/B was uninformative), `wasted_modifier` (blind-spot fix). Plus `EvalContext` stage infra (`advantage` + `GameStage`) and a per-position activation decider (the pre-existing `is_active` hook + safe skips on the costly terms). All tunable weights live in `evaluator/params.rs` (`EvalParams`).
- **`main` is 21 commits ahead of `origin/main`, unpushed** — the entire Stack N engine + ns-43 eval overhaul. Pushing needs explicit designer OK.

### Immediate Next Action

1. **Ask the designer whether to push** the 21 unpushed commits.
2. **P7** — playtest Stack N: does the standoff dissolve without re-inflating game length (watch mid-30s–40s round counts)? Engine now runs the rules.
3. **ns-46** — investigate search-depth futility (d12@30s ≈ d∞@1s). Flagged as possibly the highest-leverage AI fix; mostly NOT eval (move ordering / quiescence). Reproduce + profile first. `SELECT body FROM next_steps WHERE id='46';`.
4. **ns-44** — offline eval-param tuner. champion_threat weights + endgame_closing stage threshold are the measured under-tuned params. `SELECT body FROM next_steps WHERE id='44';`.

### Open methodological loose ends

- **`main` ahead of `origin/main`** — 21 commits unpushed (pushing needs explicit designer OK).
- `champion_threat` + `endgame_closing` correct but under-tuned; `endgame_closing` unproven in self-play (never reached End stage at 60ms/ply).
- v0.1.0 cross-platform release smoke test — still outstanding (ns-28, ns-29).
- ns-35 (help/tutorial + UI unification), ns-39/40/41 deferred.

### Key DB Queries

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-46';` | This session — ns-43 eval terms + infra + decider |
| `SELECT body FROM next_steps WHERE id='44';` | Offline eval-param tuner (next eval lever) |
| `SELECT body FROM next_steps WHERE id='45';` | Magic-bitboard `skill_attacks` (perf) |
| `SELECT body FROM next_steps WHERE id='46';` | Search-depth futility investigation |
| `SELECT body FROM stacks WHERE id='stack-n';` | Stack N — the three staged rules (now in engine) |
| `SELECT body FROM open_questions WHERE id='oq-58';` | First-mover-loses standoff (P7 hypothesis) |
| `SELECT id, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';` | The live parked levers |

### Key Files

| Path | Purpose |
|------|---------|
| `design/RULES.md` | **Canonical ruleset (authoritative on conflict).** Read first each session. |
| `design/design.db` | Source of truth (binary; committed) |
| `game/crates/core_engine/src/search/evaluator/` | **ns-43 eval module** — `terms.rs` (all terms), `params.rs` (`EvalParams`, tuner's search space), `context.rs` (stage/advantage), `registry.rs` (`is_active` decider), `mod.rs` (tests + goldens) |
| `game/crates/core_engine/src/state/magic.rs` | `skill_attacks` = classic ray-scan; ns-45 = convert to magic bitboards |
| `game/crates/core_engine/src/search/{alpha_beta,quiescence,transposition}.rs` | ns-46 search-depth investigation lives here |
| `game/crates/search_bench/` | `--eval-only` for eval speed; throwaway `examples/` for self-play A/B (delete after) |
| `.claude/STATUS.md` | One-screen re-entry summary |
