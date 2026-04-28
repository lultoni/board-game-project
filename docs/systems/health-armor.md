# Health & Armor System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

### HP States
All pieces have **2 HP**:
- **Normal** (2 HP) → 1 DMG → **Injured** (1 HP) → 1 DMG → **Removed** (0 HP).

### Injured Effects
- **Speed**: Capped at 1. (This only affects Guards, who are Speed 2 at full health. Champions and King are already Speed 1.)
- **Skill Range**: −1 Range (affects Range 2+ skills only). Range 0 (self) and Range 1 (adjacent) always work regardless of Injured status.
- **Healing**: Injured can be removed by Field Medic ("Remove Injured from one adjacent ally").

### Armor
- **Cap**: 3 Armor points per piece.
- Each Armor point absorbs 1 instance of damage (then destroyed).
- Armor is granted by skills: Rust Shield (+1 self), Armorsmith (+1 to adjacent ally), Forest terrain entry (+1 temporary, lost on leaving).
- Armor is removed by: Armor Breaker skill (remove 1 Armor from target).

---

## MDA Analysis

**Inputs**: Damage from attacks/skills, healing skills, armor-granting skills, terrain position (terrain removed from current design — see `docs/systems/skill-drafting.md`).

**Outputs**: Piece state changes, mobility/capability reduction.

**Interactions**: Combat system (damage sources), Skill System (healing/armor skills), Movement (Injured speed cap).

**Feedback loops**:
- **Positive** (Injured = weaker = easier to finish off). Snowball for the attacker.
- Armor creates a **balancing** counterloop (defensive investment can protect pieces).

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Three states (Normal/Injured/Removed) are intuitive. Armor tracking requires tokens. |
| Depth | 3 | Depth comes from deciding when to heal vs. advance, and how to manage Armor investment. |
| Interconnection | 4 | Intersects Combat (damage sources), Skill System (heal/armor skills), Movement (speed cap). |
| Emotional Resonance | 3 | Improved from P1 to P2. Injured state now meaningfully felt. |

**Status**: Improved. 3 HP for Champions/King was scrapped (would extend game; Guards at 2 HP would feel like "cheap kills" by design). Current 2 HP baseline maintained.

---

## Open Questions

- **OQ-10**: Injured penalty severity — speed penalty is Guard-only. For Champions/King, only effect is Range −1 on Range 2+ skills. Is that punishing enough? Alternatives: +1 Rune cost on skills when Injured; −1 Skill Slot when Injured.
- **OQ-11**: Armor cap — keep at 3, re-evaluate after Layer 2.
- **OQ-18**: 3 HP scrapped — see OPEN_QUESTIONS.md for full reasoning.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Injured state rarely relevant — standard attack (2 DMG) skips Injured entirely. Armor never observed in meaningful use (players chose offensive skills). 3 HP suggestion from Pasco.

**Playtest 2 (24.04.2026, Layer 1)**: Dramatic improvement. Injured state: "Often" relevant (both players). Defensive skills Armorsmith and Rust Shield used extensively. Field Medic used. The Injured Range ruling (−1 affects Range 2+ only; adjacent always works) was a key clarification that made the state feel fair rather than punishing.

**Withdrawn**: 3 HP for Champions/King — scrapped before testing in Session 6. First Champion kill was R26 with 2 HP; 3 HP would push that even later. Guards at 2 HP vs. Champions at 3 HP would create an artificial tier ("Guards = cheap kills").
