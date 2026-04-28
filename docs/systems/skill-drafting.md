# Skill Drafting System

*Last updated: 2026-04-28 — Session 8. Extracted from CURRENT_DESIGN.md.*

---

## How It Works

- Pre-game draft: players alternately pick skills from a **shared pool** and assign them freely to any of their Champions or King.
- **Pick structure**: P1 picks 2 skills and assigns them to any piece (or splits across pieces), then P2, repeating until all Champions and the King have 2 skills each.
- Each Champion/King has **2 skill slots**. The King fills last.
- **Duplicate skills**: Allowed for both players. Skill pool uses tokens (up to 6 copies per skill).
- **No banning phase**: Skill pool is currently too small to support a ban phase without starving options.

---

## MDA Analysis

**Inputs**: Skill catalogue, player strategy/meta-knowledge, opponent's picks (read-react).

**Outputs**: Each player's army composition and capability profile for the entire game.

**Interactions**: Defines the entire Skill System for the match. Affects Combat (which attacks are available), Economy (which Rune costs are being planned for), and board positioning (skills inform piece placement).

**Feedback loops**: N/A — pre-game. But draft choices create implicit commitment loops: picking Focus Strike requires pairing it well, committing a piece to a support role.

---

## Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 2 | Alternating picks are clear; synergy decisions require deep catalogue knowledge. |
| Depth | 4 | High mastery ceiling. Wrong picks cause regret mid-game (designer observed). |
| Interconnection | 3 | Defines army composition but is pre-game — doesn't interact with live systems. |
| Emotional Resonance | 3 | Fair. Elias noted "decides a lot." Feeling of strategic identity from draft is present. |

**Known issue**: Draft may be too deterministic. Picking a skill that doesn't synergise with later picks causes regret with no recovery option.

---

## Open Questions

- **OQ-16**: Draft fairness — fair, but "decides a lot." Is draft too deterministic?
- **OQ-35**: Skill pool draft variant — draft a pool of N skills first, then assign to Champions (Elias suggestion). Increases strategic intent, reduces accidental synergies.
- **OQ-43**: `[System: Skill Drafting] [Affects: Skill System]` — CR-style picks ("one for me, one for you" alternating pairs vs. current alternating singles).
- **OQ-44**: `[System: Skill Drafting] [Affects: Skill System]` — Ban phase (1–2 bans each before picks). Viable when skill pool is larger.
- **OQ-48**: `[System: Skill Drafting] [Affects: Movement, Combat]` — Piece placement order: equip skills first, then place on board. Lets draft inform starting positions.
- Backpocket: In-game skill redraft (shop/auction/interval/swap) — Layer 6+ candidate. See `docs/backpocket.md`.

---

## Playtest Evidence

**Playtest 1 (31.10.2025)**: Draft felt fair. Both players confirmed. Designer noted wrong-skill-pick regret mid-game.

**Playtest 2 (24.04.2026, Layer 1)**: Draft fair. Elias suggestion: "Skill pool draft" (pick pool first, assign after). Jonathan's army composition: heavy defensive skills (Armorsmith, Rust Shield). Both approaches valid — confirms multiple viable draft strategies.

**Key ruling** (Session 6): Free skill assignment during draft — players may assign picked skills to any Champion/King, not just the next one in sequence.
