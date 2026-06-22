# HANDOVER PROMPT

*Copy-paste this entire file as your first message in a new Claude Code session to resume where you left off.*

*Last updated: 2026-06-22 — mid Session 30 (audit pass complete, pre-Slice-1).*

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

### Where We Are (mid Session 30, 2026-06-22)

- **Audit pass complete (pre-Slice-1).** Engine stubs no longer claim "cardinal" movement — Move-Phase movement is free in all 8 directions, speed = Chebyshev distance, zigzag legal (Stack M is explicit on this).
- **`Position` has two new fields**: `round_number: u16` and `moved_this_phase: Bitboard`. FEN grammar is now 9 fields; `Undo` snapshots both for reversibility.
- **Armor cap enforced to 2** in mailbox + FEN validator. **Guards-no-skills enforced** in FEN parser (new `FenError::GuardCarriesSkill`).
- **Slice -1 + Slice 0 still shipped from S29** — FEN serialisation + `Position::setup_stack_m()` + strict validator. 40/40 tests green after audit-pass edits.
- **Stub doc-comments updated** in `action.rs`, `generator.rs`, `turn_manager.rs`, `zobrist.rs` to reflect: Move-Phase reachability via Chebyshev BFS, Move-Attack Bodyguard enumeration via `choice_idx` (0=no redirect, k=k-th adjacent friendly Guard), Tempest target-not-pushed (only neighbours), round-based income (`2 + round_number/5` paid each turn).

### Immediate Next Action

**Slice 1 — Move Phase: plain movement + Move-Attack + Bodyguard enumeration.**

1. `Action(u32)` Move-kind encoding (already designed; just wire it up).
2. `make` / `unmake` for Move-kind actions:
   - Plain move: clear src mailbox+bitboards, set dest, OR dest into `moved_this_phase`, decrement `actions_remaining`.
   - Move-Attack: enemy at target takes 1 damage (Armor first, then HP, then remove if HP=0). Mover does *not* relocate. OR mover's *src* into `moved_this_phase` (since the mover stayed put — TODO confirm this semantics call; alternative is to consume the action without marking the piece moved, which would let it move again. Designer intent: Stack M says "Each piece can only be moved once per Move Phase" — Move-Attack *is* a Move action, so the attacker is marked).
   - Bodyguard: `choice_idx>0` redirects damage to the k-th adjacent friendly Guard of the *defender*.
3. Legal-move generator: Chebyshev-BFS bounded by speed (Guard=2, Champion=1, King=1), blocked by occupied intermediate squares. For each enemy-occupied target, enumerate Bodyguard-choice variants.
4. `EndPhase` becomes legal whenever `actions_remaining == 0` or no piece has a legal move/attack.

Once Slice 1 lands, `gamedbg` CLI (`show / legal / apply / trace / perft`) and the `.scenario` runner become buildable.

### Open methodological loose ends

- **OQ: Skill-Phase action progression curve.** Stack M body says "starts at 2 per turn, scaling up over the game" with no numbers. File as OQ; treat as constant 2 until resolved.
- **OQ: Focus on Move-Skills.** Stack M: "Move skills: caster chooses activation-range or effect-range, not both." Needs an action-encoding decision (extra `choice_idx` slot, or two separate skill IDs). Slice 4 concern; file as OQ now.
- **Open question: Move-Attack effect on `moved_this_phase`.** Per Slice 1 deliverable 2 above. Default position: mark the attacker. Confirm at slice start.

### Key DB Queries (instead of file paths)

| Query | Returns |
|-------|---------|
| `SELECT body FROM next_steps WHERE id='8';` | Rule-coverage matrix + slice 0–8 plan (priority 1) |
| `SELECT body FROM next_steps WHERE id='9';` | Debug harness status (FEN done; CLI + runner todo) |
| `SELECT body FROM sessions WHERE id='session-29';` | This session's narrative |
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
| `game/crates/core_engine/src/state/fen.rs` | FEN encoder/parser + strict validator |
| `game/crates/core_engine/src/state/position.rs` | `Position`, `setup_stack_m()` |
| `game/crates/core_engine/SCENARIO_FORMAT.md` | Frozen FEN + action-text + scenario-file spec |
| `game/frontend/src/` | Svelte 5 UI (App, Board, engine bridge, multiplayer stub) |
| `.claude/STATUS.md` | One-screen re-entry summary |
| `CLAUDE.md` | Orientation (points at DB; does not restate facts) |

### Open methodological loose ends

- None.
