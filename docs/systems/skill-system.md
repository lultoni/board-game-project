# Skill System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

- Skills are **equipped during pre-game draft** (2 per Champion/King).
- Categories: **Strike** (damage), **Shield** (defense/healing), **Move** (repositioning), **Mystic** (buffs/utility).
- All skills cost **1 Skill Slot** to activate (plus a Rune cost).
- A Champion may use its skills multiple times per turn — including the same skill twice.
- **Skill Path**: Queen-like straight line (horizontal, vertical, or diagonal). **Blocked by all pieces** (ally and enemy).
- **Default Skill Range = 2**. Range 0 = self, Range 1 = adjacent, Range 2 = 2 tiles along Skill Path.
- Skills that cause movement (Quick Dash, Shadow Shift, Retreat Plan) do **not** deal damage.
- **Injured effect on range**: −1 Range when Injured, but Range 0 (self) and Range 1 (adjacent) always work regardless.

### Current Skill Catalogue

| Category | Name | Cost | Effect |
|----------|------|------|--------|
| Strike | Lance Thrust | 2 | 1 DMG at Range−1 (1 tile in front of caster along Skill Path) |
| Strike | Hook Pull | 3 | 1 DMG + pull target 1 tile closer along Skill Path |
| Strike | Armor Breaker | 2 | Remove 1 Armor from target |
| Strike | Rune Theft | 3 | 1 DMG + steal 1 Rune from opponent |
| Strike | Blade Tempest | 4 | 1 DMG to target; all pieces adjacent to target pushed 1 tile away from target (radially, not along Skill Path). Caster unaffected. |
| Shield | Rust Shield | 2 | Self: +1 Armor |
| Shield | Field Medic | 3 | Remove Injured from one adjacent ally |
| Shield | Armorsmith | 3 | Adjacent ally: +1 Armor |
| Move | Quick Dash | 3 | Self: move up to 2 tiles along Skill Path (no damage) |
| Move | Air Blast | 2 | Push target enemy 1 tile directly away from caster |
| Move | Precision Thrust | 3 | Push target enemy 1 tile in any direction (caster chooses). Range+1 (Range 3 at default). |
| Move | Shadow Shift | 4 | Swap position with an allied piece. Default Range 2. Requires unobstructed Skill Path. |
| Move | Retreat Plan | 4 | Self: teleport to adjacent to one of your Guards. Range+1. |
| Mystic | Focus Strike | 1 | The *next* skill used by any of your pieces this turn gains +1 Range. |
| Mystic | Blade Call | 3 | One Strike skill used by any of your pieces this turn deals +1 DMG. Fixed cost: 3 Runes. Boosts exactly one Strike, then spent. Can be declared retroactively (before or after the Strike). |

*Extended skill catalogue: `baseline-rules/md-converted/Project ROE Skills.md`. Backpocket ideas: `docs/backpocket.md`.*

---

## MDA Analysis

**Inputs**: Equipped skill loadout, Rune supply, Skill Slots, board geometry, Skill Path line-of-sight.

**Outputs**: Damage, healing, repositioning, buffs, Rune theft.

**Interactions**: Core interaction nexus — touches every other system. Skill Path geometry interacts with board positioning. Rune cost interacts with Economy. Skill Slots interact with Progression.

**Feedback loops**: Skill diversity creates **balancing loops** (Strike countered by Shield, Mobility counters positioning advantage). No inherent positive feedback — the combo bonus (Layer 2) is designed to reward coordination without auto-snowballing.

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 3 | Individual skills are clear; combo interactions require experience. |
| Depth | 5 | The highest-depth system in the game. Huge mastery ceiling. |
| Interconnection | 5 | Touches every other system. The central hub. |
| Emotional Resonance | 5 | "Finding and executing clever combos" is THE core fantasy. |

**Known issues**:
- Low combo ceiling: only "buff + hit" combos exist (Focus Strike/Blade Call + Strike). No emergent multi-Champion interactions. Multi-Champion combo bonus (Layer 2, Game 2) directly targets this.
- Rune Theft may be too strong with faster economy (OQ-34, monitor Layer 2).

---

## Open Questions

- **OQ-4**: Skills per piece per turn — currently uncapped. Blade Call burst concern.
- **OQ-6**: Skill slot cost — currently all 1 slot.
- **OQ-20**: Shadow Shift balance — Range 2 (default). Monitor whether Range 2 feels limiting.
- **OQ-34**: Rune Theft balance — may need cost 4. See `docs/backpocket.md`.
- **OQ-38**: Multi-Champion combo bonus — ready to test (Layer 2, Game 2).
- **OQ-49**: `[System: Skill System] [Affects: Combat, Bodyguard]` — Skill path obstruction model. Currently: all pieces block. Alternatives: only opponent pieces block (Idea 1); only opponent Guards block (Idea 2).
- **OQ-50**: `[System: Skill System] [Affects: Resource Economy, Progression]` — Minor/Major skill slot cost distinction.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Skill Drafting felt fair. Defensive skills underused (Armorsmith, Field Medic never used). Players gravitated toward cheap offensive/utility skills. Blade Call enabled burst combos — ranged instant kills possible.

**Playtest 2 (24.04.2026, Layer 1)**: Dramatic improvement. Armorsmith and Rust Shield used extensively. Field Medic used. Injured state now relevant ("Often" — both players). Focus Strike + another skill pattern observed. Rune Theft flagged as a soft concern by Elias. Low combo ceiling identified in Session 7 analysis — only buff+hit combos, no emergent multi-Champion coordination.
