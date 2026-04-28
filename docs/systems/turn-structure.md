# Turn Structure System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

- **Round** = P1 Turn + P2 Turn.
- **Turn** = Movement Phase (2 Move Slots) → Action Phase (N Skill Slots).
- **Move Slots**: Move one piece per slot (speed = tiles moved). Each piece may only be moved once per Movement Phase. Can use 0, 1, or 2 slots.
- **Skill Slots**: Activate one equipped skill per slot. Must pay Rune cost.

---

## MDA Analysis

**Inputs**: Player decisions, available pieces, Rune supply, board state.

**Outputs**: Board state changes, resource expenditure.

**Interactions**: Feeds into Combat, Skill, and Resource systems.

**Feedback loops**: None inherent — this is a neutral framework. Movement and action are unlinked (no requirement to move before acting or to act only with pieces that moved).

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Clear to new players. Two-phase separation is intuitive. |
| Depth | 3 | Limited inherent depth — mostly a container. AP system (Layer 4) would change this. |
| Interconnection | 5 | Every other system operates inside this framework. |
| Emotional Resonance | 3 | Neutral. Players feel the turn structure most when it constrains them. |

---

## Open Questions

- **OQ-26**: Unified AP system (3 AP/turn) — deferred to Layer 4. Would merge Movement and Action phases into a single resource.
- **OQ-23**: Move Slot count — test 3 Move Slots if AP system is not adopted.
- **OQ-45**: `[System: Turn Structure] [Affects: Resource Economy]` — Starting Player Decision variants (hidden Rune bid, coin flip, mutual agreement). Currently: P1 is determined before the game by mutual agreement.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Unlinked movement/action was appreciated by both players — "intuitive."

**Playtest 2 (24.04.2026)**: Two-phase structure held up. No complaints about the framework itself. Long think times noted (~R22) — the depth is in the decision space, not the structure.

**Key ruling** (Session 3): Each piece may only be moved once per Movement Phase (resolved mid-Playtest 2).
