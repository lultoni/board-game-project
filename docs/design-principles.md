# Design Principles

*The rules we design by. Every proposed change must pass these. Last updated: 2026-05-17.*

---

## Core Fantasy

**Discovering and executing clever spell/skill combos.** Every system must serve this. If a mechanic doesn't make spell combos better, cut it.

## North Star

"A small number of interlocking systems that generate surprising, meaningful decisions."

Winning means "I found the better solution to the puzzle," not "I crushed you." Competition stays. Stakes stay. But individual moves should feel satisfying to execute and appreciate — for both players.

---

## The Five Principles

*Established Session 7 (ADR-003). These are load-bearing — future design decisions are tested against them.*

### 1. Every strategy archetype should have a moment where it's the best option on the board.

Multi-Champion combos should be devastating when set up; single-Champion precision should be bread-and-butter; defensive formation play should be viable. No single strategy should dominate.

### 2. Don't reward symptoms, reward the system (The KPI Principle).

Rewarding "dealing damage" with bonus Runes incentivises the symptom (hitting things) rather than the system (clever play). Like company KPIs: setting wrong targets leads to good-on-paper results with a worse product. Reward the entire cycle, not one metric.

### 3. Players should be allowed to play how they want.

Expand viable strategies, don't shrink them. Turtling should be a viable option — just not the only one. Don't ban strategies; make alternatives competitive.

### 4. Cleverness = multi-turn positional setup rewarded with a payoff that exceeds what grinding achieves.

If a 3-turn setup and a stumble-into-position produce the same result, the game doesn't reward cleverness. The combo bonus is designed to make coordinated setups definitively better than brute force.

### 5. The shared-puzzle feel is a byproduct of good design, not a mechanic.

Don't engineer it directly. Make cleverness visible, make both players' puzzles legible, and the shared exploration emerges naturally from perfect information + depth.

---

## Hard Constraints

| Constraint | Source |
|------------|--------|
| **Perfect information** — no dice, no hidden cards, no randomness. All information is open. | Session 1, designer mandate |
| **Grid-based** — spatial positioning stays. The board makes spells interesting. | Session 1 (ADR-001/002) |
| **No terrain effects** — confirmed overhead complexity. Board is uniform. | Session 1 (ADR-001/002) |
| **2 skill slots per Champion/King** — forces specialist builds, creates draft tension. 3 slots risks generalist meta. | Session 8 analysis |
| **Champions are blank slates** — identity emerges from equipped skills, not pre-naming. Mental freedom to do anything. | Session 10, designer feedback |
| **Guards have late-game presence** — Guards stay at 2 HP. Design should not force a "kill Guards first" sequence. | Session 7, designer mandate |

---

## Spending Tension (G8)

Players must always want to do more than they can execute. The economy naturally transitions:
- **Early game**: Rune-scarce (can't afford all desired skills)
- **Mid game**: Slot-limited (have Runes but not enough Skill Slots)
- **Late game**: Opportunity-rich (both resources available, decisions about WHICH skills to fire)

If either resource becomes so abundant that spending requires no tradeoff, something is broken. Sente (forced reactive spending) is compatible with this because the tradeoff persists — responding means you can't execute your own plan.

---

## Economy Philosophy

- **Encourage spending through attractive options, not by punishing saving.** The cleanest fix for Rune hoarding is making skill usage effective and rewarding, not capping Runes or making them expire.
- **Auto-economy is strategy-neutral.** Performance-based income (captures → Runes) forces one playstyle, constraining creative expression. The combo bonus is the better lever — it rewards cleverness of execution, not which action you chose.
- **Shortfall never closes** (G1) — players can never fill all skill slots with maximum-cost skills every turn. There's always more they want to do than they can afford.

---

## Incremental Testing Methodology

Never propose changing multiple interacting systems at once.

1. **Decompose** proposed changes into independent layers.
2. **Identify coupling**: If change A affects how change B plays out, they are coupled. Coupled changes can be bundled but must be documented.
3. **Order layers** from most independent / highest impact to most dependent.
4. **One layer per playtest.** Evaluate results before proceeding.
5. **Document which layer produced which result.** Never attribute an effect to a bundled change unless you can isolate the cause.

Testing is **dynamic stack-based** — the highest-value stack is selected after each playtest, not a fixed linear sequence.

---

## Cognitive Load

Players can hold 3–5 simultaneous decision variables before paralysis (Sweller, Koster). Current design targets:

- Spell combo selection (1)
- Positioning / movement (2)
- Resource budget (3)
- Opponent prediction (4)

That's 4 axes — right in the sweet spot. Every new system added must either replace an existing axis or be absorbed into one.
