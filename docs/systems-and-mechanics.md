# Systems & Mechanics

*Per-system design documentation — how each system works, MDA analysis, interactions, design health. Last updated: 2026-05-17.*

*For player-facing rules: `docs/test-scenarios/baseline/ruleset-baseline.typ`*
*For design principles: `docs/design-principles.md`*

---

## Turn Structure

### How It Works

- **Round** = P1 Turn + P2 Turn.
- **Turn** = Movement Phase (2 Move Slots) → Action Phase (N Skill Slots).
- **Move Slots**: Move one piece per slot (speed = tiles moved). Each piece may only be moved once per Movement Phase. Can use 0, 1, or 2 slots.
- **Skill Slots**: Activate one equipped skill per slot. Must pay Rune cost.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Player decisions, available pieces, Rune supply, board state |
| **Outputs** | Board state changes, resource expenditure |
| **Interactions** | Feeds into Combat, Skill, and Resource systems |
| **Feedback loops** | None inherent — neutral framework. Movement and action are unlinked. |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Clear to new players. Two-phase separation is intuitive. |
| Depth | 3 | Limited inherent depth — mostly a container. AP system (Stack G) would change this. |
| Interconnection | 5 | Every other system operates inside this framework. |
| Emotional Resonance | 3 | Players feel the turn structure most when it constrains them. |

### Open Questions

- **OQ-26**: Unified AP system (3 AP/turn) — Stack G. Would merge Movement and Action phases.
- **OQ-23**: Move Slot count — test 3 Move Slots if AP system is not adopted.
- **OQ-45**: Starting Player Decision variants (hidden Rune bid, coin flip, mutual agreement). Currently: mutual agreement pre-game.

### Playtest Evidence

- **P1** (31.10.2025): Unlinked movement/action was appreciated — "intuitive."
- **P2** (24.04.2026): Two-phase structure held up. No complaints. Long think times noted (~R22) — depth is in decisions, not structure.

---

## Resource Economy (Runes)

### How It Works

- **Canonical numbers**: see `docs/test-scenarios/shared/baseline-sections.typ` → `section-resource-economy()`.
- **Gain timing**: Collected at the start of each player's own turn. Round 1: no collection.
- **Rune cap**: None.
- **Spending**: Runes spent to activate skills. Some skills steal opponent Runes.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Round counter, skill activations, Rune Theft |
| **Outputs** | Action tempo — how many skills per turn are affordable |
| **Interactions** | Directly gates Skill System. Rune Theft creates economy-combat crossover. |
| **Feedback loops** | Neutral (automatic gain, no performance coupling). |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Simple income table. Scaling requires reference but is learnable. |
| Depth | 4 | When to spend vs. save is the core turn-by-turn decision. |
| Interconnection | 4 | Gates Skill System directly. Rune Theft creates crossover. |
| Emotional Resonance | 4 | Saving for a big combo turn feels satisfying. |

**Status**: FIXED — Layer 1 accepted (Playtest 2, 24.04.2026). Dead opening eliminated. Skills active from Round 1.

### Open Questions

- **OQ-8/46**: Rune cap — currently none. Monitor. Cap at 8 is the candidate.
- **OQ-47**: Performance-based Rune gain — permanently closed. Forces single playstyle; auto-economy is strategy-neutral.
- **OQ-34**: Rune Theft balance — monitoring. May need cost 4.

### Playtest Evidence

- **P1**: Economy too slow — first ~6 rounds had no skill use. Pasco: "start at +2 gain."
- **P2** (Layer 1): Economy fix worked. Skills active from Round 1. Skill Slots became the action limiter — as intended.

---

## Progression (Skill Slots)

### How It Works

- **Canonical numbers**: see `docs/test-scenarios/shared/baseline-sections.typ` → `section-progression()`.
- Skill Slots do not carry over between turns.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Round counter |
| **Outputs** | Action capacity per turn |
| **Interactions** | Multiplies Resource Economy (more slots = more Rune spend). |
| **Feedback loops** | Positive (automatic escalation). Game becomes more lethal in later rounds. |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 5 | Simple table. Easy to track. |
| Depth | 3 | Automatic — no player decisions. Depth from interaction with Rune economy. |
| Interconnection | 4 | Multiplies the entire Skill System. |
| Emotional Resonance | 3 | Escalation feels good as background pressure. |

### Open Questions

- **OQ-26**: AP system (Stack G) would absorb Skill Slots into unified action-point model.
- **OQ-50**: Minor/Major skill slot cost (minor = 1 slot, major = 2 slots).

### Playtest Evidence

- **P1**: Skill Slots sufficient — Runes were the bottleneck, not Slots.
- **P2** (Layer 1): Slots became the action limiter in early rounds — exactly as intended.

---

## Skill System

### How It Works

- Skills are equipped during pre-game draft (2 per Champion/King).
- Categories: **Strike** (damage), **Shield** (defense/healing), **Move** (repositioning), **Mystic** (buffs/utility).
- All skills cost 1 Skill Slot + a Rune cost.
- A Champion may use its skills multiple times per turn — including the same skill twice.
- **Skill Path**: Queen-like straight line. Blocked by all pieces (ally and enemy).
- **Default Skill Range = 2**. Unless the skill's text explicitly names "self" (Range 0) or "adjacent" (Range 1). Skills with a Range modifier (e.g. "Range−1") are still Range 2 skills with a modifier — not treated as adjacent. Self/adjacent targeting cannot be shifted inward by Range buffs.
- Skills that cause movement do not deal damage.
- **Injured effect on range**: −1 Range when Injured. Does NOT affect skills that explicitly name "self" or "adjacent" — those always work regardless of Injured status.
- **Focus Strike note**: +1 Range to next skill. Can boost self → adjacent and adjacent → Range 2. Range and Injured penalties are calculated independently.

### Current Skill Catalogue

Canonical: see `docs/test-scenarios/shared/baseline-sections.typ` → `section-skill-reference()`.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Equipped loadout, Rune supply, Skill Slots, board geometry, line-of-sight |
| **Outputs** | Damage, healing, repositioning, buffs, Rune theft |
| **Interactions** | Core nexus — touches every system. Skill Path geometry × positioning. Rune cost × Economy. Slots × Progression. |
| **Feedback loops** | Diversity creates balancing loops (Strike ↔ Shield, Mobility ↔ positioning). |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 3 | Individual skills clear; combo interactions require experience. |
| Depth | 5 | Highest-depth system. Huge mastery ceiling. |
| Interconnection | 5 | Touches every system. The central hub. |
| Emotional Resonance | 5 | "Finding and executing clever combos" IS the core fantasy. |

**Known issues**: Low combo ceiling (only "buff + hit" combos exist). Multi-Champion combo bonus (Stack A, Game 2) targets this.

### Open Questions

- **OQ-4**: Skills per piece per turn — currently uncapped.
- **OQ-34**: Rune Theft balance — may need cost 4.
- **OQ-38**: Multi-Champion combo bonus — Stack A, Game 2.
- **OQ-49**: Skill path obstruction model (all pieces block vs. only opponents).
- **OQ-50**: Minor/Major skill slot cost.

### Playtest Evidence

- **P1**: Defensive skills underused. Players gravitated to cheap offensive/utility. Blade Call enabled burst.
- **P2** (Layer 1): Dramatic improvement. Armorsmith, Rust Shield, Field Medic all used extensively. Focus Strike + another skill pattern observed. Rune Theft flagged as soft concern.

---

## Combat / Attack

### How It Works

**Standard Attack**: Move onto enemy tile = 1 DMG *(accepted into baseline, Playtest 3)*. No Rune cost. Uses a Move Slot. If there are multiple paths toward the target, attacker may choose which to take (relevant for Bodyguard).
- If target removed: attacker occupies tile.
- If target survives: attacker stops on tile immediately before target.

**Skill Attacks**: Typically 1 DMG. Skills always hit directly — Bodyguard does NOT intercept skills.

**Bodyguard Rule** (baseline): Guard adjacent to both tile-before-target AND defender can intercept Standard Attack on Champion/King. Guard takes damage. Attacker moves 1 tile. Interception is optional.

**Multi-Champion Combo Bonus** (Stack A, Game 2 — pending test): Counter model. Each enemy piece has a combo counter (starts 0, resets end of your turn). Different Champions hitting with Strike skills increment the counter. Bonus DMG = counter value at time of hit.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Movement, positions, Armor state, Guard positions, line-of-sight |
| **Outputs** | Piece removal, Armor reduction, position changes |
| **Interactions** | Movement (attacks require moving onto tile), Health, Guard positioning, Skills |
| **Feedback loops** | Positive (losing Guards = less protection = easier losses). Armor creates balancing counterloop. |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Standard attack is clear. Bodyguard adds one decision. |
| Depth | 3 | Binary (move on or don't). Depth from setup and reading the board. |
| Interconnection | 4 | Movement + Health + Guards + Skills intersect here. |
| Emotional Resonance | 5 | Capturing a piece is decisive. Bodyguard intercepts are dramatic. |

**Critical issue** (Session 7): Standard attack dominance. 2 DMG for free outperformed all skill combos. Stack A Game 1 tested 1 DMG nerf — *accepted into baseline (Playtest 3)*. First Champion kill moved from R26 → R11. Standoff dissolved. Stack A Game 2 (combo bonus) ready to test.

### Open Questions

- **OQ-37**: Standard attack 1 DMG — ~~Stack A, Game 1~~ **ACCEPTED into baseline (P3)**.
- **OQ-38**: Combo bonus — Stack A, Game 2. **Ready to test (Session 16).**
- **OQ-21**: Bodyguard adjacency to defender only — Stack B.
- **OQ-40**: Standoff / no-man's-land — tracking in Stack A.
- **OQ-41**: Game length vs. damage nerf tradeoff.

### Playtest Evidence

- **P1**: Bodyguard never triggered. Standard attack dominated — "wait and pounce."
- **P2** (Layer 1): Bodyguard triggered ~2x. "Two Guards like pawns" blocking lanes at R14. 1 Champion kill in 26 rounds.

---

## Health & Armor

### How It Works

All pieces have **2 HP**: Normal (2) → Injured (1) → Removed (0).

**Injured effects**: Speed capped at 1 (Guards only — Champions/King already Speed 1). Skill Range −1 (affects Range 2+ only).

**Armor**: Max 3 per piece. Each point absorbs 1 DMG then destroyed. Granted by Rust Shield, Armorsmith. Removed by Armor Breaker.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Damage from attacks/skills, healing skills, armor-granting skills |
| **Outputs** | Piece state changes, mobility/capability reduction |
| **Interactions** | Combat (damage), Skills (heal/armor), Movement (speed cap) |
| **Feedback loops** | Positive (Injured = weaker = easier to finish). Armor creates balancing counterloop. |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 4 | Three states are intuitive. Armor tracking requires tokens. |
| Depth | 3 | When to heal vs. advance, how to manage Armor investment. |
| Interconnection | 4 | Intersects Combat, Skills, Movement. |
| Emotional Resonance | 3 | Improved from P1 to P2. Injured state now meaningfully felt. |

**3 HP for Champions/King**: Scrapped. Would extend game (first Champion kill at R26 with 2 HP). Guards at 2 HP vs Champions at 3 HP would create artificial tier.

### Open Questions

- **OQ-10**: Injured penalty severity — for Champions/King, only Range −1. Is that enough?
- **OQ-11**: Armor cap — keep at 3, re-evaluate after Stack A.

### Playtest Evidence

- **P1**: Injured rarely relevant — 2 DMG standard attack skips it. Armor never used meaningfully.
- **P2** (Layer 1): Dramatic improvement. Injured state: "Often" relevant. Defensive skills used extensively.

---

## Skill Drafting

### How It Works

- Pre-game alternating draft from a shared pool.
- P1 picks 2 skills, assigns freely to any Champion/King → P2 → repeat.
- Each Champion/King has 2 skill slots. Continue until all filled (12 per player).
- Duplicates allowed.

### MDA Analysis

| | |
|---|---|
| **Inputs** | Skill catalogue, strategy/meta-knowledge, opponent's picks |
| **Outputs** | Army composition and capability profile for the entire game |
| **Interactions** | Defines Skill System for the match. Affects Combat, Economy, positioning. |
| **Feedback loops** | N/A — pre-game. Draft choices create implicit commitment loops. |

### Design Health

| Dimension | Score (1–5) | Notes |
|-----------|------------|-------|
| Legibility | 2 | Picks are clear; synergy decisions require deep catalogue knowledge. |
| Depth | 4 | High mastery ceiling. Wrong picks cause mid-game regret. |
| Interconnection | 3 | Defines army composition but is pre-game. |
| Emotional Resonance | 3 | Feeling of strategic identity from draft is present. |

**Known issue**: Draft may be too deterministic. Picking a non-synergistic skill early causes regret with no recovery.

### Open Questions

- **OQ-16**: Draft fairness — "decides a lot." Too deterministic?
- **OQ-35**: Skill pool draft (draft pool first, assign after) — post-Stack B.
- **OQ-48**: Equip skills first, then place on board — post-Stack B.

### Playtest Evidence

- **P1**: Draft felt fair. Wrong-pick regret observed.
- **P2** (Layer 1): Fair. Elias suggestion: "Skill pool draft." Both defensive and offensive draft strategies validated.
