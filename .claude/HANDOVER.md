# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — end of Session 30 (Slices 1–5 complete).*

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

### Where We Are (end Session 30, 2026-06-22)

- **Engine is up through Slice 5.** Move Phase (plain + Move-Attack + Bodyguard), terminal detection, the 5 Strike-skill resolvers (Lance/Break/Steal/Hook/Tempest), and the 8 Shield/Move-class resolvers (Shield/Heal/Plate/Dash/Blast/Shove/Swap/Retreat) all ship in a single working commit chain: `5e4f316` → `e938569` → `04d301b` → `55be8cc` → `09db1de` → (Slice 5, this session).
- **177 lib tests green, clean release build.**
- **Eight designer decisions captured as resolved OQs** this session: oq-73 (Shield/Plate at cap illegal), oq-74 (Heal non-Injured illegal), oq-75 (Blast/Shove off-board asymmetry — Blast fizzles in resolver, Shove pre-filtered), oq-76 (all movement skills must relocate), oq-77 (Swap ally-only), oq-78 (Retreat-no-guards illegal), oq-79 (friendly-piece pushes don't tick combo), oq-80 (skill movement is queen-ray straight line, never king-zigzag).
- **Still panicking with "Slice 6"**: `Skill::Focus` and `Skill::Charge` resolvers.

### Immediate Next Action

**Slice 6 — Focus + Charge wiring + end-of-turn clearance + Skill-Phase action curve.**

1. `apply_focus` / `apply_charge` resolvers — write to `pending_modifiers` bits, debit money + decrement action.
2. Generator: consume Focus's +1 Range when iterating Strike-skill ranges (OQ noted in generator.rs).
3. `turn_manager::end_turn` — clear combo counters, `tracked_*`, `pending_modifiers`; advance round on flip-back to P1; disburse `2 + round_number/5` income.
4. Skill-Phase action budget by round (currently hardcoded 2 — see oq-69).
5. Zobrist hashing — populate `undo.zobrist_xor` so transposition tables become viable.

### Open methodological loose ends

- **oq-69 — Skill-Phase action progression curve.** Stack M body says "starts at 2 per turn, scaling up." Still no curve. Treat as constant 2 in Slice 6 until resolved.
- **oq-70 — Focus on Move-skills.** Caster chooses activation-range or effect-range. Action-encoding decision pending; Slice 6 will surface it concretely when wiring Focus.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id='8';` | Rule-coverage matrix + slice 0–8 plan (priority 1) |
| `SELECT body FROM next_steps WHERE id='9';` | Debug harness status (FEN done; CLI + runner todo) |
| `SELECT body FROM sessions WHERE id='session-30';` | This session's narrative |
| `SELECT body FROM adrs WHERE id='adr-005';` | Digital architecture decision |
| `SELECT body FROM stacks WHERE id='stack-m';` | Stack M rule substance (engine's source of truth) |
| `SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high');` | Live critical/high OQs |
| `SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;` | Active todos |

### Key Files (still on disk)

| Path | Purpose |
|------|---------|
| `design/design.db` | Source of truth (binary; committed) |
| `design/schema.sql` | 12-table schema |
| `design/inbox/{brainstorm,ai-chats,digital}/` | Designer's inbox channels |
| `game/Cargo.toml` | Rust workspace root |
| `game/crates/core_engine/src/game_logic/make_unmake.rs` | All skill resolvers (Slices 4+5) + Move-kind apply/unmake |
| `game/crates/core_engine/src/game_logic/generator.rs` | Legal-action enumeration (Move + Skill phases) |
| `game/crates/core_engine/src/state/magic.rs` | Queen-ray geometry primitives |
| `game/crates/core_engine/SCENARIO_FORMAT.md` | Frozen FEN + action-text + scenario-file spec |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |

### Open methodological loose ends

- None.
