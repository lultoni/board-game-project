# Combat / Attack System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

### Standard Attack
- Move a piece **onto an enemy piece's tile** = **2 DMG** (baseline) / **1 DMG** (Layer 2 test).
- No Rune cost. Uses a Move Slot.
- If target is **removed**: attacker occupies the tile.
- If target **survives** (Armor absorbed some damage): attacker stops on the tile **immediately before** the target (along the attack path).

### Skill Attacks
- Typically 1 DMG (Injures a Normal piece; removes an Injured piece).
- Skills always hit directly — **Bodyguard does NOT intercept skills**.

### Bodyguard Rule (Baseline)
- A Guard **adjacent to both** the tile immediately before the target (along the attack path) **AND** the defender can intercept a Standard Attack on a Champion or King.
- The Guard takes the damage (not auto-removed — goes through normal HP rules).
- Attacker moves 1 tile toward the target when intercepted (stops before the Guard).
- Interception is **optional** — the defending player chooses whether to trigger it.
- **Layer 3 test**: Loosen to adjacent to defender only (removes the dual-adjacency requirement).

### Multi-Champion Combo Bonus (Layer 2, Game 2)
- When 2+ different Champions use skills that hit the same enemy piece in the same turn, each skill **after the first** deals **+1 DMG**.
- Buff skills targeting your own pieces (Focus Strike, Blade Call, Rust Shield, etc.) do NOT count.
- Blade Call stacks with combo bonus (separate effects).
- Skills do not need to be consecutive — any two or more Champions targeting the same piece in the same turn qualifies.

---

## MDA Analysis

**Inputs**: Movement, piece positions, Armor state, Guard positions, Skill Path line-of-sight.

**Outputs**: Piece removal, Armor reduction, position changes.

**Interactions**: Movement system (attacks require moving onto tile), Health system, Guard positioning, Skill System (skills are attack vectors).

**Feedback loops**: **Positive** (losing Guards = less Bodyguard protection = easier to lose Champions/King). This is the primary snowball risk in the design. Armor creates a **balancing** counterloop.

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Standard attack is immediately clear. Bodyguard adds one decision. |
| Depth | 3 | Standard attack is binary (move on or don't). Depth comes from setup and reading the board. |
| Interconnection | 4 | Movement + Health + Guards + Skills all intersect here. |
| Emotional Resonance | 5 | Capturing a piece feels decisive. Bodyguard intercepts are dramatic. |

**Critical issue identified (Session 7, ADR-003)**: Standard attack dominance. 2 DMG for free outperforms all skill combos — skills are structurally the support act despite being the stated core fantasy. Layer 2 tests 1 DMG standard attack (Game 1) + combo bonus (Game 2) to rebalance.

---

## Open Questions

- **OQ-37**: Standard attack damage 1 DMG — ready to test (Layer 2, Game 1).
- **OQ-38**: Multi-Champion combo bonus — ready to test (Layer 2, Game 2).
- **OQ-21**: Bodyguard adjacency to defender only — ready to test (Layer 3).
- **OQ-40**: Standoff / no-man's-land problem — tracking in Layer 2. Neither player wants to enter the 2-3 tile gap between formations first.
- **OQ-41**: Game length vs. damage nerf tradeoff — tracking in Layer 2.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Bodyguard never triggered (Guards died too fast / adjacency-to-both too restrictive). Standard attack dominated — players "wait and pounce" rather than using skills.

**Playtest 2 (24.04.2026, Layer 1)**: Bodyguard triggered ~2x (improvement). "Two Guards like pawns" blocking lanes at R14 — positional deadlock. Only 1 Champion kill in 26 rounds. Guards were dying throughout. Standard attack dominance confirmed via session analysis: 2 DMG free vs. skill combos costing 3–6 Runes for equivalent damage.

**Key ruling** (Session 3): Standard attack survival — attacker stops on tile immediately before target; only occupies tile if target removed.

**Key ruling** (Session 7): Bodyguard adjacency — "attacker's starting tile" was the prior description; corrected to "tile immediately before target along attack path" (handles Speed-2 attackers correctly).
