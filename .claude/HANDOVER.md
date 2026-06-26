# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-26 — Session 35 end (NN-rater scoping + search-speed benchmark + AB optimisation catalogue).*

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

### Where We Are (Session 35 end, 2026-06-26)

- **Three new plan docs in `design/inbox/digital/`.** No code changes this session — pure scoping for the next work tranche.
  - `nn-rater-plan.md` — full NN position-rater scope. Path 3 + perturbation, two-tier gauntlet, three champion tracks, opt-in observability UI. No blocking ADR.
  - `search-speed-benchmark-plan.md` — benchmark infra; FEN-driven; two modes (fixed depth + fixed time); doubles as correctness regression test; manual-run-only.
  - `alpha-beta-optimisation-catalogue.md` — 9 categories of AB techniques with game-specific adaptations. Critical flags: `EndPhase` ≠ null-move; no chess-SEE port; QS loud/quiet redefined for HP-skills.
- **`next_steps id=25`** body appended with pointer to all three.
- **Engine and UI unchanged** from S34. Stack M still awaiting a real playtest.

### Immediate Next Action

**Begin search-speed work** per `search-speed-benchmark-plan.md` §"Execution order":
1. Scaffold the bench binary (native, FEN-driven, structured output).
2. Build the 20-50-position FEN corpus including ≥1 known-result tactical position.
3. Verify determinism (same-position-same-result-N-times).
4. Generate initial baseline at `game/bench/baseline.json`.
5. Land optimisations one at a time per `alpha-beta-optimisation-catalogue.md` order (PVS first, then TT-move, aspiration, killers+history, LMR, ...).

Pre-existing alternative lanes still valid if you prefer:
- **Frontend** — Inspector L6.7d preview window primitive (`next_steps id=12`).
- **Multiplayer hardening** — deferred IllegalActionInHistory + draft-route-when-in-play replay bugs.
- **Design** — argue through `oq-84` (bodyguard-intercept greying-out) before next Stack M digital playtest.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be marked open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding shipped in engine Focus + Move-skill resolvers; verify OQ status against current code.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-35';` | This session's narrative (NN-rater scope + bench plan + AB catalogue) |
| `SELECT body FROM next_steps WHERE id=25;` | NN position rater idea + S35 scoping pointer |
| `SELECT body FROM open_questions WHERE id='oq-81';` | AI search branching-factor + strategy plan (informs the catalogue) |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/inbox/digital/nn-rater-plan.md` | NN-rater full scope (S35) |
| `design/inbox/digital/search-speed-benchmark-plan.md` | Benchmark infrastructure plan (S35) |
| `design/inbox/digital/alpha-beta-optimisation-catalogue.md` | AB technique catalogue with implementation order (S35) |
| `game/crates/core_engine/src/search/alpha_beta.rs` | Search loop under optimisation |
| `game/crates/core_engine/src/search/evaluator.rs` | Current hand-coded eval (header carries load-bearing eval philosophy) |
| `game/crates/core_engine/src/search/transposition.rs` | TT infrastructure |
| `game/crates/core_engine/src/state/fen.rs` | `to_fen` / `from_fen` for corpus loading |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
