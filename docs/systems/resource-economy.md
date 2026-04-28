# Resource Economy (Runes) System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

- **Starting Runes**: 6 (Layer 1 accepted, Playtest 2).
- **Gain timing**: Collected at the **start of each player's own turn**. Round 1: no collection — use starting Runes only.
- **Gain rate** (Layer 1 — accepted):

| Rounds | Gain per player turn |
|--------|----------------------|
| R1 | 0 (use starting Runes) |
| R2–6 | +2 |
| R7–11 | +3 (+1 every 5 rounds) |
| R12–16 | +4 |
| R17+ | continues (+1 every 5 rounds) |

- **Rune cap**: None. Players naturally spend down; hoarding not observed as a problem.
- **Spending**: Runes spent to activate skills. Some skills steal opponent Runes.

---

## MDA Analysis

**Inputs**: Round counter, skill activations, Rune Theft skills.

**Outputs**: Determines action tempo — how many skills per turn are affordable.

**Interactions**: Directly gates the Skill System. Interacts with Combat through skill costs. Interacts with Progression (Skill Slots scale too).

**Feedback loops**: Currently **neutral** (automatic gain, no performance coupling). A *Performance-Based* variant (OQ-47) would create **positive feedback** (capturing pieces → more Runes → more skills → more captures).

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Simple income table. Scaling requires a reference sheet but is learnable. |
| Depth | 4 | Rune management — when to spend vs. save — is the core turn-by-turn decision. |
| Interconnection | 4 | Gates the Skill System directly. Rune Theft creates economy-combat crossover. |
| Emotional Resonance | 4 | Saving for a big combo turn feels satisfying. |

**Status**: FIXED — Layer 1 accepted (Playtest 2, 24.04.2026). Dead opening eliminated. Skills active from Round 1.

---

## Open Questions

- **OQ-8**: Rune cap — currently none. Monitor as economy speeds up. Cap at 8 is the candidate (OQ-46).
- **OQ-46**: `[System: Resource Economy] [Affects: Skill System]` — Rune cap at 8 vs. no cap.
- **OQ-47**: `[System: Resource Economy] [Affects: Combat, Progression]` — Performance-based Rune gain (capture → +2, occupy centre → +1 gain, King advanced → +1 gain). Deferred — snowball risk.
- **OQ-34**: Rune Theft balance — monitoring in Layer 2. May need cost 4 (see `docs/backpocket.md`).

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Economy too slow — first ~6 rounds had almost no skill use. Players just positioned and shielded. Elias: "start at +2 gain."

**Playtest 2 (24.04.2026, Layer 1)**: Economy fix worked. Skills active from Round 1. Skill Slots became the action limiter in early game — as intended. Neither player felt Rune-starved. Overall: "Much better than Playtest 1."
