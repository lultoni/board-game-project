# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — mid Session 30 (audit pass, pre-Slice-1).*

---

## Current focus

**Digital engine implementation — Stack M ruleset.** Slice -1 (FEN serialisation) and Slice 0 (`setup_stack_m()` + setup-invariant validator) shipped in Session 29. Session 30 ran an audit pass: armor cap fixed to 2, Guards-no-skills enforced in FEN, `moved_this_phase` and `round_number` added to `Position` and FEN, stub doc-comments corrected for Move-Phase movement geometry and Bodyguard enumeration. Next: implement Slice 1.

## Active stack

**Stack M — Game Length Cut.** The engine executes Stack M's body as written. Full substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`.

## Movement geometry (Move Phase) — corrected

- Movement is **free in all 8 directions** per Stack M. Speed = Chebyshev distance (diagonal step costs 1).
- A Guard (speed 2) may reach any empty square within Chebyshev distance 2 via *any* path of single-tile steps — zigzag is legal — provided every intermediate square is empty.
- Move-Attack: same reachability, but the destination is an enemy square. Mover does NOT enter the tile; enemy takes 1 damage.
- "Each piece can only be moved once per Move Phase" — tracked by the new `moved_this_phase` bitboard.
- **Previous STATUS/HANDOVER incorrectly said "cardinal movement". This was wrong.**

## What changed in Session 30 (audit pass)

1. **Armor cap enforced to 2** — `mailbox::with_armor` debug-asserts ≤2, FEN validator rejects armor=3 with `MailboxFieldOutOfRange`. Bit-layout doc fixed.
2. **Guards-no-skills enforced** — new `FenError::GuardCarriesSkill { rank_idx_from_top, slot }`. Plain and strict parsers both reject `G[…]` with non-zero skill1/skill2.
3. **`Position::round_number`** added (u16, starts at 1, increments on flip-back to P1). Income disbursed per-turn: `2 + round_number / 5`.
4. **`Position::moved_this_phase`** added (Bitboard, cleared on Move→Skill). Tracked destination squares of pieces already moved this phase.
5. **FEN grammar extended** to 9 fields: `… <pending_modifiers> <round_number> <moved_this_phase>`. The new fields are `decimal 1..=65535` and `0x<hex>` respectively. SCENARIO_FORMAT.md updated.
6. **Stub doc-comments rewritten** in `action.rs`, `generator.rs`, `turn_manager.rs`, `zobrist.rs` to reflect: Chebyshev movement, Move-Attack Bodyguard enumeration via `choice_idx`, Tempest target-not-pushed, round-based progression. `Undo` got two new snapshot fields (`prev_moved_this_phase`, `prev_round_number`).
7. **All 40 `core_engine` tests green.**

## Immediate next action

**Slice 1 — Move Phase plain movement + Move-Attack + Bodyguard.** Deliverables:
1. `make` / `unmake` for `ActionKind::Move` actions (both plain and Move-Attack variants), maintaining `moved_this_phase`.
2. Legal-move generation (Chebyshev-BFS, blocked by occupied intermediate squares).
3. Move-Attack target enumeration with Bodyguard `choice_idx` branching.

Edge cases per `next_steps` id=8 Slice 1.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"`

## DB sanity

- 12 tables. `PRAGMA foreign_key_check` + `PRAGMA integrity_check` both ok at start of Session 30.
