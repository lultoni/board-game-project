# Design Principles

*The rules we design by. Every proposed change must pass these. Last updated: 2026-05-17.*

---

## Core Fantasy

**Discovering and executing clever spell/skill combos.** Every system must serve this. If a mechanic doesn't make spell combos better, cut it.

## High-Concept Framing

*Established Session 19 (ADR-004, 2026-05-26).*

The Core Fantasy is delivered under the **"Two minds, one puzzle"** framing: two players race to discover and execute clever skill combos in a *shared* combinatorial space. The 2-player nature is **load-bearing** — replace the opponent with an AI and the core experience dies, because the parallel-solving *is* the experience. The opponent is a fellow puzzle-solver, not a constraint generator.

Practical consequences:
- Combo legibility must work in both directions — caster *and* observer must read the elegance.
- The shared draft pool is a load-bearing chassis feature, not decoration.
- Phase B (theme/identity) is briefed under "two minds reading the same combinatorial space" — rules out soloist-wizard themes and faction-versus-faction war themes; pulls toward mirroring / parallel-discovery imagery.
- Asymmetry (factions, starting conditions) is biased against — symmetric or near-symmetric drafts and setups preferred.
- Future mechanical decisions are evaluated partly on whether they support the framing (soft preference for B-aligned over A-aligned when otherwise equal).

This is design intent, not a mechanical mandate. No immediate rule changes follow from it. See ADR-004 in `mechanics-log/mechanics-evaluated.md` for the alternatives considered and the reversal criterion.

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
- **Base auto-economy is strategy-neutral.** The guaranteed passive income (+2/turn) must not force a playstyle. However, strategy-specific economy sources (e.g., bonuses tied to aggression, control, or combo play) ARE acceptable — provided (a) multiple strategies each have a viable economy path, and (b) those paths are balanced against each other. The ban is on a single-strategy economy funnel, not on economy variety.
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

**Important nuance**: This 4-axis model is *aspirational* — it describes what experienced players engage with, not what new players do. Early-experience players primarily process their own available actions ("what can I do?") and check the Rune budget *after* identifying a desired action, not before. Opponent prediction emerges with game knowledge (like chess: low-Elo players play reactively, grandmasters read positions and calculate 5+ turns ahead). Design should support both modes: the game must be playable with just axes 1–3 while axis 4 provides the depth ceiling.
