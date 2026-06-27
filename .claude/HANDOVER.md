# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-27 — Session 36 end (Quiescence Search + head-to-head match reveals evaluator bottleneck).*

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

### Where We Are (Session 36 end, 2026-06-27)

- **QS module shipped** at `game/crates/core_engine/src/search/quiescence.rs`. Catalogue §3 minus SEE. 380/380 tests pass. Hooked at `depth <= 0` in `alpha_beta.rs`.
- **Bitboard `is_king_threatened`** replaces generator-based first cut. Chebyshev + skill cost/range gating.
- **`DISABLE_QS` AtomicBool** + **`qs_match` example** ship a runtime A/B kill-switch and head-to-head match harness.
- **RFP and aspiration retested on top of QS** — still failed multi-budget sweep. Reverted.
- **Critical finding**: 3-game head-to-head match at 1000 ms/move showed **0 skills cast across 105 rounds**, EndPhase-dominated. Root cause: `search/evaluator.rs` has no positional terms (eval = material + HP + armor + skills + money), so Move actions have Δeval=0. The entire S36 grading protocol was measuring a non-playing engine. All S36 rejections (RFP, aspiration, PVS, LMP) are provisional pending eval fix.

### Immediate Next Action

**NN position-rater** per `design/inbox/digital/nn-rater-plan.md`:
1. Native-only training crate (rayon-parallel).
2. Path 3 (gradient descent) + perturbation injection.
3. Two-tier gauntlet (best-of-three at 100/300/500 ms, mirrored loadouts, three champion tracks).
4. Opt-in observability UI via local-file polling.

After eval supports real play, **re-run the full S36 sweep** (QS, RFP, aspiration, PVS, LMP) AND the `qs_match` head-to-head harness on top of the new evaluator before accepting/rejecting any technique.

### Banked wins from S36 (kept hooked)

- QS module — correct, ready to grade once eval supports real play.
- Bitboard `is_king_threatened` — load-bearing primitive.
- `DISABLE_QS` + `qs_match` — play-strength accept/reject path independent of corpus depth-reached.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Resolved in code as `2 + (round_number-1)/10` (`make_unmake.rs:982-985`). OQ row may still be marked open in DB — verify and resolve if so.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Encoding shipped. Verify OQ status.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM sessions WHERE id='session-36';` | This session's narrative (QS + evaluator bottleneck finding) |
| `SELECT body FROM sessions WHERE id='session-35';` | Previous session — NN-rater scope + bench plan + AB catalogue |
| `SELECT body FROM next_steps WHERE id=25;` | NN position rater idea + S35 scoping pointer |
| `SELECT body FROM open_questions WHERE id='oq-81';` | AI search branching-factor + strategy plan |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/inbox/digital/nn-rater-plan.md` | NN-rater full scope (next session focus) |
| `design/inbox/digital/alpha-beta-optimisation-catalogue.md` | AB catalogue + S37-retrospective (DB Session 36) |
| `design/inbox/digital/search-speed-benchmark-plan.md` | Benchmark infrastructure plan |
| `game/crates/core_engine/src/search/quiescence.rs` | QS module (shipped S36) |
| `game/crates/core_engine/examples/qs_match.rs` | Head-to-head match harness |
| `game/crates/core_engine/src/search/alpha_beta.rs` | Main search; carries `DISABLE_QS` kill-switch |
| `game/crates/core_engine/src/search/evaluator.rs` | **The bottleneck.** Material/HP/armor/skills/money only — no positional terms |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |
