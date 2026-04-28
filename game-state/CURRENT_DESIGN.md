# CURRENT DESIGN - (GAME NAME)

*Last updated: 2026-04-28 — Session 9 (dynamic stack system built; composable Typst sections; all layer files refactored; TESTING_PLAN.pdf created)*

---

## Game Concept / Working Title

**(GAME NAME)** — working title, unnamed.

A 2-player tactical abstract game. Two players command armies across a 10x10 grid, equipping Champions with skills and spending Runes to activate them. Victory by capturing the enemy King. The game targets the intersection of chess-like spatial tactics and CCG-style build customisation.

---

## Target Experience

**What should players FEEL?**

- **Strategic mastery**: "I won because I outthought you, not because I got lucky."
- **Build identity**: "My army composition reflects MY plan — it's different every game."
- **Escalating tension**: The game starts positional, becomes increasingly lethal as Runes and Skill Slots scale up.
- **Agonising decisions**: "I have 3 good moves but can only do 2 — which do I sacrifice?"
- **Read-and-react**: "I see what you're building toward and I need to counter it NOW."
- **Shared discovery**: "Both players jointly exploring the same puzzle; winning means 'I found the better solution,' not 'I crushed you.'" (Session 7 — informed by `docs/research/cooperative-feel-competitive-games.md`)

---

## Piece Roster

| Piece | Count per Player | Base Speed | Skills? | HP | Notes |
|-------|-----------------|------------|---------|-----|-------|
| King | 1 | 1 | Yes (2 slots) | 2 | Capture = loss condition |
| Champion | 5 | 1 | Yes (2 slots) | 2 | Primary skill carriers |
| Guard | 6 | 2 | No | 2 | Fast screens, Bodyguard-eligible |

All pieces have 2 HP: Normal → Injured → Removed.

---

## Identified Systems

Each system is documented in full in `docs/systems/`. This section is an index — follow the links for rules, MDA analysis, design health scores, open questions, and playtest evidence per system.

| # | System | Status | File |
|---|--------|--------|------|
| 1 | **Turn Structure** | Stable. Consider 3 Move Slots or AP system (Layer 4). | [`docs/systems/turn-structure.md`](../docs/systems/turn-structure.md) |
| 2 | **Resource Economy (Runes)** | FIXED — Layer 1 accepted. Skills from R1. Skill Slots are now the limiter. | [`docs/systems/resource-economy.md`](../docs/systems/resource-economy.md) |
| 3 | **Progression (Skill Slots)** | Stable. May be superseded by AP system (Layer 4). | [`docs/systems/progression.md`](../docs/systems/progression.md) |
| 4 | **Skill System** | Low combo ceiling identified (Session 7). Layer 2 targets this. | [`docs/systems/skill-system.md`](../docs/systems/skill-system.md) |
| 5 | **Combat / Attack** | CRITICAL — standard attack dominance identified. Layer 2 testing 1 DMG + combo bonus. | [`docs/systems/combat-attack.md`](../docs/systems/combat-attack.md) |
| 6 | **Health & Armor** | Improved — Injured now "Often" relevant (Playtest 2). | [`docs/systems/health-armor.md`](../docs/systems/health-armor.md) |
| 7 | **Terrain** | Removed — confirmed overhead complexity. Reversible as "map variant" expansion. | No file — decision in ADR-001/002 |
| 8 | **Skill Drafting** | Fair but possibly too deterministic. In-game redraft idea parked in backpocket. | [`docs/systems/skill-drafting.md`](../docs/systems/skill-drafting.md) |

---

## Current Core Loop

```
ROUND START
  |
  v
[P1 TURN]
  |-> Movement Phase: Move 0-2 pieces (positioning, attacking)
  |-> Action Phase: Activate 0-N skills (damage, heal, reposition, buff)
  |
[P2 TURN]
  |-> (same structure)
  |
  v
[ROUND END]
  |-> Both players gain Runes
  |-> Round counter advances (progression checkpoints at R7, R10, R14, R20...)
  |
  v
ROUND START (repeat)
```

**Short-term loop** (within a turn): "Where do I move? What skills do I spend Runes on?" — Positional + economic decisions.

**Medium-term loop** (across rounds): "I'm saving Runes for a big play next turn" / "I need to trade pieces to weaken their Guard screen" — Resource management + attrition planning.

**Long-term loop** (game arc): "The game is escalating — Runes and Skill Slots are increasing. I need to position for the kill before they do" — Tempo + inevitability pressure.

---

## Open Design Questions

See `game-state/OPEN_QUESTIONS.md` for the full list. The most critical:

1. **Rune economy speed** (OQ-17): Start 6 Runes, +2/round — **ACCEPTED** (Layer 1, Playtest 2)
2. **Standard attack damage** (OQ-37): Standard attack 1 DMG — ready to test (Layer 2, Game 1)
3. **Multi-Champion combo bonus** (OQ-38): +1 DMG on coordinated hits — ready to test (Layer 2, Game 2)
4. **Bodyguard adjacency** (OQ-21): Adjacent to defender only — ready to test (Layer 3)
5. **Unified AP system** (OQ-26): 3 AP per turn replacing two phases — deferred to Layer 4
6. **Board size / piece count**: 8x8 with fewer pieces — deferred to Layer 5
7. **Hex board** (OQ-42): Reopened (Session 8) — needs research before scheduling
8. **Shared-puzzle direction** (OQ-39): Lean into "rewarding cleverness" as design identity — open

Resolved: Board 10x10 (baseline), Unlinked movement (baseline), Skill path blocked by all pieces, No terrain.
Withdrawn: YINSH capture penalty (unfair when asymmetric), Economy skills as slots (slot-tax too high), Damage escalation after Round X (arbitrary), 3 HP for Champions/King (would extend game, Guards artificially weaker).

---

## Architecture Direction (decided Session 1)

**Direction A+: Streamlined Grid, Spells as Star.** (ADR-001, ADR-002)

- Grid stays — it makes spells interesting. Removing it makes spells into math.
- Spells/skill combos are THE core fantasy. Everything else serves them.
- Perfect information, no luck, no dice.
- Multiple viable strategies (aggro, control, defensive).
- Changes to be tested via dynamic stacks (pick highest-value stack after each playtest), not a fixed linear queue.

Full decision records: `docs/decisions/ADR-001-game-architecture-direction.md`, `docs/decisions/ADR-002-direction-a-plus.md`

---

## Incremental Test Plan

Testing is now stack-based (not linear). See `docs/test-scenarios/TESTING_PLAN.pdf` for the full decision tree and entry conditions.

| Stack | Experience Outcome | Scenarios | Status |
|-------|--------------------|-----------|--------|
| **Accepted** | Economy fix (Layer 1) | 6 start Runes · +2/turn · +1 every 5 rounds | **ACCEPTED** (Playtest 2, 24.04.2026) |
| **A — Cleverness** | Make skill combos dominant strategy | G1: 1 DMG attack · G2: + combo bonus | **Ready to print** (`stack-a-cleverness/`) |
| **B — Guards** | Make Guards strategically useful | Bodyguard: adjacent to defender only | **Ready to print** (`stack-b-guards/`) |
| **C — Pacing** | Shorten games / accelerate kills | Checkmate win condition · board/piece count | Not yet written |
| **D — Board** | Optimise board feel and scale | 8x8 · Hex (gated on `/research` first) | Not yet written |
| **E — Draft** | Improve pre-game drafting | Pool draft (OQ-35) · Placement order (OQ-36+48) | Not yet written |
| **F — Cleverness II** | More levers for clever plays | OQ-51: cascade triggers, positional payoffs | Not yet written |
| **G — Structure** | Radical turn structure redesign | Unified AP framework (no separate phases) | Draft written (`stack-g-structure/`) |

All rule sheets use the composable section system (`docs/test-scenarios/shared/baseline-sections.typ`). Baseline changes propagate automatically — no manual copy-paste across files.

---

## Playtest Evidence

### Playtest 1: Elias vs Pasco (31.10.2025) — Baseline

**Variants tested**: 10x10 board, Automatic Runes (baseline), Unlinked Movement/Action

#### Confirmed Working
- 10x10 board size is generally right (both players agreed)
- Unlinked movement/action is intuitive and appreciated
- Skill Drafting feels fair
- Piece placement is balanced
- Skill Slots are sufficient — Runes are the bottleneck (as intended)

#### Confirmed Problems
1. **Game too long** (~28-35 rounds, both rated 4/5 too long). Endgame dragged 10+ rounds with outcome already decided.
2. **Rune economy too slow at start** — first ~6 rounds had almost no skill use. Players just positioned and shielded.
3. **Bodyguard Rule never triggered** — Guards died too fast or weren't positioned correctly.
4. **Shadow Shift is OP** — no range limit on position swap.
5. **Injured state rarely relevant** — Standard attack (2 DMG) skips Injured entirely.
6. **Defensive skills underused** — players gravitated toward cheap offensive/utility skills.
7. **Blade Call enables burst combos** — ranged instant kills possible.
8. **"Wait and pounce" feel** — game lacked moment-to-moment tension.

*Full analysis: `docs/research/playtest-1-analysis.md`*

---

### Playtest 2: Elias vs Jonathan (24.04.2026) — Layer 1: Economy Fix

**Variants tested**: Layer 1 (6 start Runes, +2/turn, scaling every 5 rounds). Result: **Draw** (~26–30 rounds, ended by time).

#### Confirmed Working (improvements from Playtest 1)
- **Skills active from Round 1** — economy fix works, dead opening eliminated
- **Skill Slots now the action limiter** (not Runes) in early game — as intended
- **Injured state relevant** — both players: "Often." Injured pieces still threaten via Focus Strike
- **Defensive skills used** — Armorsmith and Rust Shield used extensively; Field Medic used
- **Bodyguard Rule triggered** (~2x — improvement from P1's zero triggers)
- **Tension maintained** — neither player felt the outcome was decided until the end
- **Overall enjoyment**: 4–5 (Jonathan: "Mid to late game Bombe 6 out of 5"). "Much better" than Playtest 1.

#### Remaining Problems
1. **Game ended at Round 26 — session ran 4+ hours.** Draw. Estimated ~10 more rounds needed for a win condition → natural finish ~R36.
2. **Only one piece kill in 26 rounds** — first (and only) Champion kill at R26, "unexpected target." Guards were dying throughout.
3. **Positional deadlock replaced Rune-starvation dead opening** — Guards blocking attack lanes at R14.
4. **Long think times — caused by combo depth.** Elias: "still very long think times" at R22.
5. **6+ rules ambiguities surfaced** (resolved in Sessions 6–7).
6. **Rune Theft is a soft flag, deferred.** Monitor in Layer 2.

#### Player Suggestions
- Smaller board for faster action (Jonathan)
- Skill pool draft (draft pool first, assign to Champions after) — Elias
- Flexible piece placement (not fixed row) — Elias
- Air Blast / Precision Thrust: allow targeting own pieces for repositioning

*Full analysis: `docs/research/playtest-2-analysis.md`*

---

## Design Health Summary

For full scores and notes, see the per-system files in `docs/systems/`.

| System | Status |
|--------|--------|
| Turn Structure | Stable |
| Resource Economy | FIXED (Layer 1 accepted) |
| Progression | Stable |
| Skill System | Low combo ceiling — Layer 2 targets this |
| Combat/Attack | CRITICAL — standard attack dominance (Layer 2 tests fix) |
| Health/Armor | Improved — Injured now relevant |
| Terrain | Removed (stable) |
| Bodyguard Rule | Improved — Layer 3 ready |
| Skill Drafting | Fair — may be too deterministic |
