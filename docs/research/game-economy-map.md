# Game Economy Map

*Quantitative analysis of the game's component-level economies and their progression curves. Captures the designer's Session 25 hand-mapped overview of every economy that interacts in a game of (GAME NAME), with end-of-game scaling math and three insights that informed the Game Length Cut (Stack M).*

*Source: `brainstorm-images/IMG_1083.jpg` (economies list), `brainstorm-images/IMG_1084.jpg` (quantitative end-state).*

*Established Session 25, 2026-06-21. Reflects baseline as of `BASELINE_VERSION = 2026-05-30` (current 10x10 board, baseline economy, Multi-Champion Combo Bonus accepted).*

---

## Why this doc exists

Before the GLC stack was drafted, the designer wanted to see *every* economy in the game at once — what compounds, what is bounded, what scales linearly, what scales geometrically. The economy map is the structural justification for the GLC's specific change set: each change targets a specific compounding curve that pushes games past the 30-60 min target.

This is a **reference snapshot**, not a living document. When baseline changes (Stack M lands, etc.), recompute or annotate — do not silently mutate.

---

## The 12 Economies (Bild IMG_1083)

The designer's brainstormed list of every economy the game has. Right column shows the *axes* each economy moves along.

| # | Economy | Notes |
|---|---------|-------|
| 1 | Win economy | "slow/2nd order" — rock-paper-scissors layer; macro strategy |
| 2 | Tile economy | progression der economies im rundenverlauf; |
| 3 | Health economy | und auch was man max haben könnte theoretisch jeweils, |
| 4 | Armor economy | was sind drains/faucets |
| 5 | Money economy | |
| 6 | Skills-available economy | |
| 7 | Action economy | |
| 8 | Piece-count economy | |
| 9 | Damage economy | |
| 10 | Piece-progression economy | |
| 11 | Movement economy | |
| 12 | Movement-variations-in-turn economy | |
| (12+) | Skill-variations-in-turn economy | |

Three lenses to apply per economy (designer notation, "wie zudieinen / was sind faucets"):
- **Progression of the economy through the round sequence** — how does it scale?
- **Theoretical max** — what's the ceiling?
- **Drains and faucets** — what adds to it, what removes from it?

---

## Quantitative End-of-Game State (Bild IMG_1084)

Designer-computed end-state numbers — what the economy curves look like at R1, R5, R10, R15, and beyond. Numbers in parentheses indicate the cumulative-over-game total (e.g., "5 (10)" = at round 5, the cumulative total is 10).

### Tile economy
- **Total tiles**: 100 (10×10 baseline)
- **Occupied tiles**: ~24
- **Occupation ratio**: 0.24 (vs. chess 0.5 — board feels emptier)
- **Reachable tiles**: ~40 (the area pieces can physically reach)
- **Reachable ratio**: 0.8 (vs. chess 1.0)

### Health
- **Max HP per piece**: 48 hp(?) — `3(14) money for +1` (cost to heal +1 HP, with max prog +2 per piece)
- *Reading: HP economy is essentially flat — bounded healing rate, bounded ceiling.*

### Armor
- **Min/start**: 0
- **Max**: 72 (theoretical total Armor that could exist across all pieces if everyone capped)
- **Cost**: `2/3 (14) for +1` (varies by skill: Shield at 2, Plate at 3, Armor as state)
- **Max prog**: 8 (per piece, theoretically)
- *Reading: Armor compounds the fastest of any health-related economy and is the primary chassis-volume problem.*

### Money
- **Start**: 6
- **Round-by-round cumulative**: 8, 10, 12, 15, 18, 21, 24, 27, 30, 33, 35¹, 39, 43, 47, 52, 57, 62, 67, 72, 78, 84, 90, 96, 102, 109…
- *Reading: Money compounds linearly with a +1/5-rounds bonus kicking in. By R15 a player has earned 56-65 total Money; by R25 it's 100+.*
- *Note ¹: numbers are designer's hand-tracked, may contain a small transcription slip around R11.*

### Actions
- **R1**: 1 → cumulative (1, 2, 3)
- **R5**: 3 → cumulative (3, 5, 7)
- **R10**: ~7 → cumulative (7, 10, 15)
- **R15**: ~13 → cumulative (13, 17, ...)
- **Doubling reads**: R5→R10 = 2× actions, R10→R15 = ~2×, R15→R20 = ~2×, R20→R25 = ~3× (last one rough). Cumulative end-totals R5 → R10 → R15 → R20 → R25 ≈ 2 (2), 2 (10), 42 (20), ~43 (35).

**Insight #1**: *"Money is the limiting factor most of the time, but when you do have the money, actions become the limiting factor."* → **Implication for testing**: a future stack should test with max-actions, or with **unified actions** (collapsing Move + Skill into one pool). Currently parked as a follow-up to Stack M.

### Damage
- **Move-Attack damage cumulative**: R1: 1 (2), R5: 5 (10), R10: 10 (20), R15: 15 (30)
- **+Skill damage cumulative**: R1: 1 (5), R5: 5 (17), R10: 10 (35), R15: 15 (56)
- **+Max combo damage cumulative**: R1: 1 (6), R5: 5 (20), R10: 10 (42), R15: 15 (65)

*Reading: damage compounds geometrically when combos fire. By R15 a player can theoretically deal 65 cumulative damage from one Champion's combo line — more than enough to obliterate the opponent's full piece pool (48 max HP × ~? pieces).*

**Insight #2**: *"It's easier to deal damage than to heal or armor up. Both costs money AND has possible drawback → losing spiral for the losing player (if flipped, game would be dragged out tho)."* → **Implication**: the asymmetric damage-vs-heal economy is what produces both the (a) one-shot vulnerability problem AND (b) the dragged-out endgame, depending on who's winning. Two-sided pathology, not a clean lever.

---

## Insight #3 — Stalling Root Cause (Bild IMG_1085, IMG_1086)

Bild 1085 frames it as a separate frage: *"woher kommt stalling?"*

- **Symptom**: figuren bewegen sich oder progressieren (Armor stacken / heilen) weil es nichts einfacheres oder attraktiveres gibt
- **Mechanik**: investment in piece-state-progression statt board-action = man opfert tempo um seine figuren wieder auf vordermann zu bringen
- **Insight verbatim**: *"man muss sich aktuell entscheiden zwischen 'figuren verlieren' oder 'tempo verlieren'"*
- **Deeper insight**: *stalling an sich IST tempo verlieren / keiner progress machen, weil es nichts besseres zu tun gibt (ohne 30 Jahre zu denken)*

**Implication**: we must always offer the player a way to make progress — but the progress option should be *more attractive* than stalling. Stalling appears when (a) no board activity (fight/push) is directly possible, OR (b) the activity is too complex to plan.

**Designer's own framing**: *"meine aufgabe ist die komplexität der angriffe zu verringern und immer die möglichkeit für aktivität zu bieten."*

This is the structural argument for the GLC's combined cuts:
- **Smaller board (8×8)** = pieces always close → board activity more often directly available
- **Lower Armor cap** = less "easy progressive escape" via Armor-stacking
- **Wider combo (movement also stacks)** = makes engagement more rewarding than stalling
- **No injury debuff** = removed pieces don't become slow-targets that have to be re-positioned
- **No draw conditions** = removed the "everyone stalls until draw" trap

---

## Move-Space Combinatorics (Bild IMG_1088, IMG_1089)

The designer counted possible end-to-end moves in one mid-game position to quantify the felt-PI problem (OQ-64).

### Per-piece move counts (one example mid-game position, black player)
- 11 pieces movable. Per-piece raw move options: **16, 6, 4, 11, 6, 6, 18, 6, 19, 15, 4**
- Designer notes: *"this will not be super accurate and hence the true count is higher than this"*, since actual moves don't track skill-path-dependent combo moves.

### Final calc
- Movement = 1 + 11·9 + 11·9² = **14,281** distinct movement sequences (2 pieces moved from pool of 11 options each)
- Skill activations = 1 + 12 + 12² = **157** distinct skill combinations (2 skills, 12 available, with the +money-cost branching)
- Total end-to-end move count: **T = M × S = 2,242,117**

**Implication**: ~2.2M distinct legal end-to-end moves in one mid-game position. This is the quantitative footprint of OQ-64 (felt-PI breaks under combinatorial breadth). It is also the reason for *"die komplexität der angriffe verringern"* as a stated GLC sub-goal — the 8×8 cut + lower Armor cap + removed Injury debuff all reduce the per-turn option fan-out.

### Skill usage tally (this position)
- ✦ (Strike): 9 uses
- Σ (Move/buff): 2 uses
- ⊕ (Mystic): 1 use
- *Note: red (Strike) skill usage way higher than other categories.*

---

## Win-condition alternatives (Bild IMG_1089, parked)

Designer's parked alternatives to King-capture (not adopted for Stack M):
- "Andere seite des boards erreichen" (reach opposite side)
- "Eventuell: könig auf die andere seite oder spezifisch eine figur zu einem punkt bringen" (move King or specific piece to a target tile)
- "50% der champs sterben → verlieren" (lose half your Champions = you lose)

**Insight**: *"wir müssen vermeiden, dass man einfach eine figur hinten parkt ohne sie je zu bewegen (wie es beim king oder generell einem mystic slave der fall wäre)."* → Any win-condition redesign must structurally prevent the "park King in back corner forever" degenerate.

**Status**: not adopted for Stack M. Will be revisited as a standalone design pass *after* GLC stabilises. Tracked as a future direction, not an open OQ.

---

## Injury Debuff Removal — Reasoning (Bild IMG_1085)

The designer's reasoning for removing the Injury debuff (one of Stack M's six changes):

- *"Do we even need injury debuff? I mean the piece being close to dying is already a good incentive to retreat. A nerf/debuff in the first place..."*
- **AND**: *"having a 'double cripple' forces the player to decide if they want to save money and such or heal and lose out on other actions — basically only high value pieces will be healed, which allows opp to set a lot more tempo like this. (because it basically works)"*
- **So**: injury → letting piece die → long mid-piece exchanges → healing piece → opponent has more long-term threat / can tempo spend.
- **Decision**: playtest *without* injury debuff (after GLC) — but actually folded INTO Stack M itself.

---

## Skill Brainstorm (Bild IMG_1091)

Out-of-scope for Stack M; reference image only. Roughly 25 candidate skill modifiers and ~10 new skill effects (skill-paths that push, all-target-to-caster, skill switching, transfer-skill-slot, etc.). Reserved for a future skill-catalogue-expansion pass after Stack M's results land. **No transcription needed** — refer to `brainstorm-images/IMG_1091.jpg` directly when designing new skills.

One image-1091 detail relevant to Stack M: *"Steal R2"* note in the corner — confirms designer's intention that Steal's range stays at 2. *"Move R3"* note — observation only, no change.

One image-1091 detail relevant to a future stack: *"cheaper but nerfed (only armor, lessening)"* — Plate-style buffs at lower cost, paired with reduced Armor — possible follow-up to Stack M if Armor 2 cap proves too tight.

One image-1090 detail parked: *"monster skill"* — a skill that lets pieces ignore skill-path blocks. Staged for after Stack M (or after the skill catalogue expansion pass).

---

## How this map informed Stack M

Each of Stack M's six bundled changes maps to one or more economies above:

| Change | Targets economy/insight |
|--------|------------------------|
| Board 10×10 → 8×8 | Tile economy (occupation 0.24 → higher); Movement combinatorics (per-piece options shrink); stalling root cause (closer pieces, more direct board activity) |
| Armor cap 3 → 2 | Armor economy (max 72 → ~48); chassis-volume (OQ-11); "easy progressive escape" stalling |
| Injury debuff removed | Damage economy (heal/save tradeoff dissolved); piece-progression economy (no "rehab piece" tempo loss) |
| Draw conditions removed | Win economy (single-climax shape, Principle 8); stalling-until-draw trap |
| Steal cost 3 → 4 (both modes) | Money economy (faucet/drain rebalance); skill-meta (OQ-34 must-pick) |
| Combo bonus extended to movement-causing skills | Damage economy (combo trigger surface widens — engagement more attractive than stalling); skill-meta scope (combo legibility per OQ-38) |

---

## Open questions raised by this map (not actioned in Stack M)

- The **damage > heal asymmetry (Insight #2)** is a two-sided pathology — Stack M doesn't directly address it. Watch in Stack M playtests whether the losing-spiral shape softens or worsens with the smaller board + lower Armor.
- The **unified actions / max actions** follow-up (Insight #1) is queued behind Stack M; it would be the next stack if Stack M's Money curve still feels off.
- The **piece-count cut** (Stack K) is queued behind Stack M; designer's note: "Auch piece count" als folgender follow-up test.
- The **2.2M end-to-end moves** (combinatorics) is a measurement of OQ-64 — watch whether the smaller board materially reduces this figure in Stack M playtests.
