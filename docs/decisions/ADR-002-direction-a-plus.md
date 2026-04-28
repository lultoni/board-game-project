# ADR-002: Direction A+ — Streamlined Grid, Spells as Star

*Date: 2026-04-17*
*Status: PARTIALLY ACCEPTED — direction confirmed, implementation via incremental layers (not monolithic)*
*Supersedes: ADR-001 (architecture choice resolved → Direction A+)*

---

## Design Constraints (locked in)

From designer feedback and playtest:
- **Perfect information**: No dice, no hidden cards, no randomness. All information is open.
- **Grid-based**: Spatial positioning stays. The board makes spells interesting.
- **Spells are the core fantasy**: Discovering and executing clever combos is THE experience.
- **Multiple viable strategies**: Econ, aggro, control should all be competitive. No single dominant strategy.
- **Cognitive sweet spot**: 4-5 decision axes max per turn.
- **No drawn-out endgames**: The game must accelerate toward conclusion.

---

## The Three Problems A+ Must Solve

From Playtest 1 and designer reflection:

1. **Dead opening** — 6+ rounds of positioning before spells matter
2. **Spell bandwidth** — too many non-spell things competing for brain space
3. **Binary combat** — alive/dead with no gradient, making the game "wait and pounce"

---

## Accepted Ideas (to be tested incrementally)

### Economy Fix (Layer 1 — test first)
- Start with 6 Runes instead of 4, +2 gain per round instead of +1, scaling +1 every 5 rounds.
- No Rune cap initially. Add one only if hoarding is observed.

### 3 HP for Champions/King (Layer 2)
- Champions and King get 3 HP. Guards stay at 2 HP.
- **Simplified to 2-state only**: Normal / Injured. No "Critical" state. Injured = has taken any damage and has ≤2 HP remaining. Effects: Speed capped at 1, Skill Range -1.
- Standard attack (2 DMG) Injures a Champion (doesn't kill). Skills (1 DMG) are meaningful chip damage.

### Bodyguard Rule Simplification (Layer 3)
- Guard adjacent to **defender only** can intercept. Attacker position doesn't matter.
- Guard takes the damage (not auto-removed — if Guard has Armor, it could survive).
- Defender chooses which Guard intercepts.

### Unified AP System (Layer 4 — test after 1-3 are stable)
- 3 AP per turn, each spent on Move / Skill / Attack.
- Multiple constraint models to test (see Layer 4 rule sheet).

### Board Size & Piece Count (Layer 5 — test after AP is settled)
- 8x8 board, reduced army (exact composition TBD based on earlier layer results).
- No terrain.

---

## Rejected / Withdrawn Ideas

### YINSH-inspired Capture Penalty — WITHDRAWN
**Reason**: Creates asymmetric cost. If one player runs out of Guards first, only the other player pays the penalty for capturing. The player with no Guards attacks freely while the opponent is still "taxed." This feels like being punished for playing correctly rather than an elegant balancing mechanic.

### Economy Skills as Skill Slots — DEFERRED
**Reason**: With only 2 skill slots per Champion, equipping an economy skill makes that Champion a "one-trick pony." The opportunity cost is too high — you sacrifice combat versatility for an economic investment. This is a structural problem: 2 slots is too few to afford dedicating one to non-combat function.
**Future exploration**: Economy could work as a system-level mechanic (e.g., Rune gain from board position, captures, or a separate "economy action" that doesn't consume skill slots) rather than as a skill-slot commitment. Revisit after core systems are stable.

### Damage Escalation After Round X — DEFERRED
**Reason**: Feels arbitrary ("aus der Luft gegriffen"). May not be needed once economy, HP, and board changes are in place. Only revisit if games are still too long after Layers 1-3.

---

## Designer Feedback: Specific Concern Resolutions

### "Piece count asymmetry — is 3+1 vs 3 intentional?"
Not yet decided. The 3 Champion + 3 Guard proposal was for the Layer 5 test. In Layers 1-3, the current piece count (5 Champions + 6 Guards + 1 King) is preserved. Piece count is the last thing to change, and the exact ratio (symmetric vs asymmetric) is an open question for that layer.

### "Guards shouldn't be obligatory first kills"
Agreed. The ADR-002 phrasing "Guard screen melts, then Champions duke it out" described a tendency, not a design goal. Guards surviving into endgame should be viable and tactically interesting (they're fast — speed 2 — good for chasing, blocking, screening). The design should not force a "kill Guards first" sequence.

### "AP system allows a piece to march 3 tiles and rush the King"
Valid concern. A piece spending all 3 AP on movement could traverse 3 tiles (or 6 for a Guard with speed 2). This could enable King-rushing that bypasses strategic play. Multiple constraint models are proposed for testing in Layer 4:
- **Model A**: 1-skill-per-piece-per-turn limit (original proposal). Doesn't prevent movement rushing.
- **Model B**: Each piece can only receive 1 AP per turn. Forces spreading across army.
- **Model C**: Uncapped normally, but unlock multi-AP-per-piece only when you have ≤2 pieces left (comeback/desperation mode).
- **Model D**: Hybrid — 2 AP can go to one piece maximum, 3rd must go elsewhere.

### "Changing too many systems at once is risky"
Fully agreed. This is now enshrined in CLAUDE.md as a mandatory methodology rule. ADR-002's monolithic proposal is replaced by the incremental Layer 1-5 test plan (see `docs/test-scenarios/`).

---

## Implementation: Incremental Test Layers

Changes are decomposed into testable layers. Each layer has a full standalone rule sheet in `docs/test-scenarios/`.

| Layer | Change | Independent? | Tests what? |
|-------|--------|-------------|-------------|
| 1 | Economy fix (6 start, +2/round) | Yes | Does faster economy fix the dead opening and shorten the game? |
| 2 | 3 HP for Champions/King | Yes | Does combat gradient fix binary alive/dead and make defensive skills viable? |
| 3 | Bodyguard simplification | Yes | Does "adjacent to defender only" make Bodyguard actually trigger? |
| 4 | Unified AP system | Coupled with piece freedom model | Does collapsing phases into AP improve decision quality? |
| 5 | Board 8x8 + fewer pieces + no terrain | Coupled with AP | Does a smaller, simpler board make spells the star? |

**Test order**: 1 → 2 → 3 → (evaluate) → 4 → 5

Full rule sheets: `docs/test-scenarios/layer-N-*.md`
