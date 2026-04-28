# Progression System (Skill Slots)

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

- **Starting Skill Slots**: 2 per turn.
- **Scaling**: +1 Skill Slot every 10 rounds.

| Rounds | Skill Slots per turn |
|--------|----------------------|
| R1–10 | 2 |
| R11–20 | 3 |
| R21–30 | 4 |
| R31+ | continues (+1 every 10 rounds) |

Skill Slots are the per-turn action capacity. Each slot allows activating one equipped skill (paying its Rune cost). Skill Slots do not carry over between turns.

---

## MDA Analysis

**Inputs**: Round counter.

**Outputs**: Action capacity per turn — how many skills a player can activate.

**Interactions**: Multiplies the Resource Economy (more slots = more Rune spend per turn). Interacts with the Skill System (all skills cost 1 slot + Runes).

**Feedback loops**: **Positive** (automatic escalation). Combined with Rune scaling, the game becomes dramatically more lethal in later rounds. This is likely intentional — prevents draws by forcing endgame resolution.

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 5 | Simple table. Easy to track. |
| Depth | 3 | Automatic — no player decisions involved. Its depth comes from how it interacts with Rune economy and skill costs. |
| Interconnection | 4 | Multiplies the entire Skill System. |
| Emotional Resonance | 3 | Escalation feels good as a background pressure but isn't a moment-to-moment feeling. |

---

## Open Questions

- **OQ-26**: AP system (Layer 4) would likely replace or absorb Skill Slots into a unified action-point model.
- **OQ-23**: 3 Move Slots variant — may be superseded by AP system.
- **OQ-50**: `[System: Skill System] [Affects: Resource Economy, Progression]` — Minor/Major skill slot cost (minor = 1 slot, major = 2 slots). Currently all skills cost 1 slot + Rune cost.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Skill Slots sufficient — Runes were the bottleneck, not Slots. Confirmed as working.

**Playtest 2 (24.04.2026, Layer 1)**: After economy fix, Skill Slots became the action limiter in early rounds — exactly as intended. Both players noted they had more Runes than they could always spend (capped by Slots). Behaved as designed.
