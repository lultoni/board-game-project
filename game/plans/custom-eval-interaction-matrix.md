# Custom Evaluator — Term Interaction Matrix

Companion to `custom-eval-redesign.md`. Purpose: map how every eval term affects every
other so we can (a) find pain points, (b) see which terms touch them, and (c) pick the
highest-impact change while avoiding noise/double-counting bloat.

Snapshot: after C1 (factor chain landed) + the Move-Attack reachability fix
(`enemy_can_move_attack`: Guard@R2 OR Champ/King@R1, now gating `king_danger_malus`).
Before P0 (king-as-champion) and C2. Update as factors land.

---

## Two kinds of "affect each other" — keep them separate

1. **Compositional (×):** terms that combine *within one piece's score* — one term
   scales another. This is REAL interaction (the point of the per-piece model). Today
   only the champion factor chain: `piece = BASE × factor_overextension × factor_combo`.
2. **Additive (+):** terms that land in the *same total* but never change each other's
   magnitude — they just sum (per-piece totals + all side terms + king malus). They
   don't interact mathematically, but they can still **conflict in intent**:
   double-count one signal, or pull opposite directions. That intent-conflict is the
   real bloat risk, not the arithmetic.

A healthy eval: strong compositional interaction inside pieces (context matters),
minimal *intent overlap* across the additive terms (each says something new).

---

## Term inventory (current + planned)

| # | Term | Kind | Scope | Reads | Encodes |
|---|------|------|-------|-------|---------|
| T1 | `factor_overextension` | × | champ/guard/king | own rings 1–3, enemy ≤R4 gate | clustering |
| T2 | `factor_combo` | × | champ/king | combo-reach, partner reaches, enemy proximity, path | combo set-up |
| T3 | `king_danger_malus` | + | king only | Move-Attack reach (Guard@R2/Champ-King@R1), strikes, money/actions, defense, king hp/arm | king kill-race |
| T4 | `term_skill_capacity` | + | side | money, income, actions, max skill cost | payable throughput |
| T5 | `term_offense_capable` | + | side | strike/move champ counts | attacker count → **utilisation gap (P2)** |
| T6 | `term_territory` | + | side | flood control, contested, enemy-king proximity | board control |
| **C2** | `factor_offense` (planned) | × | champ/king | strike/move skills, side attacker count | offensive worth |
| **C4** | `factor_exposure` (planned) | × | all pieces | reachability, eff. health vs incoming, retaliation econ | vulnerability (scales whole product) |
| **C5** | skill-control map (candidate) | + or substrate | side/champ | skill-threatened tiles | restriction |

Scope note: T1/T2/C2/C4 are the per-piece factor chain. The KING must run that chain too
(P0) — it is a champion under the hood — with T3 (`king_danger_malus`) added on top as the
king-specific term. "king only" on T3 means the malus is king-specific, not that the king
skips the chain.

---

## Matrix A — compositional (×): within-piece factor interaction

Rows scale columns (and vice versa — multiplication is symmetric). ✓ = they multiply
into the same piece score, so each is felt *in proportion to* the other. `—` = different
piece scope (never on the same piece) or one is additive (see Matrix B).

|                | overext | combo | offense(C2) | exposure(C4) |
|----------------|:-------:|:-----:|:-----------:|:------------:|
| **overext**    |    —    |   ✓   |     ✓       |      ✓       |
| **combo**      |    ✓    |   —   |     ✓       |      ✓       |
| **offense(C2)**|    ✓    |   ✓   |     —       |      ✓       |
| **exposure(C4)**|   ✓    |   ✓   |     ✓       |      —       |

All four champion factors multiply on the same `BASE`, so they fully interact by
construction. Reading the consequences:

- **overext × combo** (live): an over-extended (alone) champ's combo reward shrinks. ✓ working.
- **exposure scales the WHOLE product** (C4): because it multiplies the accumulated
  `BASE × overext × combo × offense`, whatever makes a piece valuable makes losing it
  hurt more. So it protects "the piece holding the most strategic value" — often the
  attacker, but equally the last Heal/Plate/Mystic carrier — WITHOUT needing its own
  value calc. This is the designer's point 3: don't single out offensive pieces; scale
  the whole value.
- **combo × exposure** (THE key pair, designer): a champ whose value is mostly combo
  set-up must not sit exposed — exposure damps its combo-inflated value. Resolves the c6
  reference case.
- **offense × exposure** (C2 × C4): a high-offense champ that's takeable loses a lot
  (big base × low exposure) — one instance of the whole-product rule, the "don't hang
  your attacker" case.
- **overext × exposure**: disjoint inputs (clustering vs killability), both multiply in.
  No conflict — just hold the discipline that they measure different things (P1).

Guards: over-extension applies today; the OTHER factors guards should get (defensive
covering, offensive presence, etc.) are **not yet decided** — do not lock in first
instincts (designer). The skill-only factors (combo/offense) correctly stay 1.0 since
guards carry no skills. The KING, by contrast, is a champion under the hood and must run
the full chain (see P0).

---

## Matrix B — additive (+): intent overlap across summed terms

NOT arithmetic interaction — these just sum into the total. A cell flags whether two
terms share a signal (**DOUBLE** = same thing counted twice, must resolve; `overlap` =
partial shared input, watch magnitudes) or pull in **tension** (one rewards what the
other punishes — fine, but must be ORDERED so the right one wins). Blank = independent.
Each live cell's *mechanism* is spelled out below the grid (a label is not an
explanation).

|                    | pieces | king_danger | skill_capacity | offense_capable | territory | ctrl-map(C5) |
|--------------------|:------:|:-----------:|:--------------:|:---------------:|:---------:|:------------:|
| **pieces**         |   —    |   tension   |                |  **DOUBLE(C2)** |  overlap  |   overlap    |
| **king_danger**    | tension|      —      |    overlap     |     overlap     |           |   overlap    |
| **skill_capacity** |        |   overlap   |       —        |     overlap     |           |              |
| **offense_capable**|**DBL** |   overlap   |    overlap     |       —         |           |   overlap    |
| **territory**      | overlap|             |                |                 |     —     |  **DOUBLE**  |
| **ctrl-map(C5)**   | overlap|   overlap   |                |     overlap     |**DOUBLE** |      —       |

**Mechanisms (why each cell is what it is):**

- **pieces × king_danger (tension):** king_danger lowers the king-owner's total when the
  king is threatened (protect it); a king-hunt lowers the *enemy* king's owner total,
  i.e. rewards the attacker for pushing pieces INTO danger to threaten. So per-piece
  safety (don't lose my pieces) opposes the king-hunt reward (spend pieces to threaten
  the enemy king). Intentional — must be ordered so the king-hunt wins (see P3).
- **pieces × offense_capable (DOUBLE, arrives with C2):** today `offense_capable` counts
  attacker champs at the side level (±10 each). C2's `factor_offense` puts attacker worth
  *into the piece score*. Same fact — "this side has attackers" — then lands in two places
  and the search sees attacker value doubled. Resolve in C2 (see P2, now reframed).
- **pieces × territory (overlap):** an active, forward piece both raises its own score
  (via combo/offense proximity) and expands the flood-fill control around it. Partial —
  territory is per-empty-square, pieces is per-piece — but a forward piece is credited on
  both axes. Watch that territory's weight doesn't re-pay what activity already paid.
- **king_danger × skill_capacity (overlap):** both read the same money/actions model
  (`affordable_casts`). king_danger uses it for a *specific* race (can enemy afford enough
  strikes before I defend); skill_capacity uses it for *general* throughput potential.
  Shared machinery, different question — acceptable, not a double-count (see P5).
- **king_danger × offense_capable (overlap):** both reward "having offense". king_danger
  is the *concrete* payoff (offense aimed at the enemy king); offense_capable is the
  *abstract* count. Once offense_capable becomes the utilisation term (P2-new), this
  sharpens rather than overlaps: king_danger is one way to *realise* the potential the
  utilisation term says you're sitting on.
- **skill_capacity × offense_capable (overlap):** more money → more skill casts → more
  realisable offense. Both partly measure "can this side do stuff." Different units
  (money-throughput vs attacker-count); keep magnitudes apart.
- **territory × ctrl-map C5 (DOUBLE):** both are board-control maps over tiles. A C5
  *side term* would largely re-say territory (flood reachability vs skill-threat tiles).
  Gates C5 (see P4).
- **ctrl-map C5 × {pieces, king_danger, offense_capable} (overlap):** if C5 exists, a
  skill-threatening piece feeds C5, its own piece score, and (if aimed at the king) the
  king race — three credits for one aggressive placement. Another reason C5 stays a
  candidate until it's proven to add signal, not echo.

---

## Pain points (ranked)

### P0 — the King is scored as pure liability, not a Champion (CORRECTNESS BUG)
The king has 2 equip slots, carries skills, can Strike, can tick a combo, controls
territory, occupies space — it IS a champion under the hood. But `score_king` returns
*only* `-king_danger_malus`, discarding all of that. A skill-carrying king reads as pure
downside. **Resolution:** the king runs the FULL champion factor chain (overextension,
combo, offense, exposure) PLUS the king-specific danger malus on top. Do this before/with
C2 — every per-piece factor we add is silently missing on the king until it's fixed.

### P1 — over-extension vs exposure double-count (must stay disjoint, NOT a blocker)
`factor_overextension` (clustering) and `factor_exposure` (C4, killability) both live on
the safety side. They MUST key off disjoint inputs — overextension = empty friendly rings
(clustering only); exposure = concrete killability (reachable via `enemy_can_move_attack`
+ eff-health-vs-incoming + can't-retaliate). This is not a hard design tension (the
designer was explicit: "of course they target different things and they will") — it's just
a discipline to hold when writing C4. Noted, not blocking.

### P2 — offense_capable → a NON-SWITCH utilisation term (RESOLVE WITH C2)
Not "delete or shrink" — **reframe**. `offense_capable` becomes an aggression/utilisation
signal: measure the GAP between a side's offensive potential and what it's realising. High
potential + not converting → rate it worse (wasted advantage, push to use it); weaker side
→ nudge toward equalising / playing cool. This is DIFFERENT from `factor_offense` (per-piece
worth), so the double-count dissolves.

**Critical constraint (designer):** it must NOT be a hard switch, or we reintroduce the
single-ply score-flip disease. The aggression incentive is itself **gated by whether the
pieces that would carry out the advantage are attackable/takeable**: if the attackers are
safe → full "go convert" incentive; if they're exposed/takeable → scale the incentive DOWN
smoothly (a takeable attacker can't safely convert). Continuous, and tied back into
exposure. Build the continuous version with C2, verify no single-ply flip.

### P3 — king_danger vs safety factors: tension ordered, WITH a design escape hatch
king_danger (±400, close-out / turtle-breaker) opposes the safety factors (protect my
pieces). They MUST be ordered so a king-hunt or favourable trade OUT-scores a piece-safety
penalty (principle 5). **But the deeper risk (designer):** if we gate *everything* behind
exposure, the AI's safest line is to never expose anything → passive turtle that never
converts. Magnitude discipline (king_danger swing >> any one exposure loss; trades win) is
the first defense. The escape hatch: **if, after honest tuning, the AI still prefers to
turtle rather than convert, that is evidence the RULESET under-rewards commitment — a
core game-design problem to fix, NOT something the eval should brute-force with knobs.**
The eval must not fake an incentive the game doesn't actually provide.

### P4 — territory (T6) vs control-map (C5): near-identical (GATES C5)
Both are board-control/restriction maps over tiles. C5 as a *side term* would largely
re-say T6. **Resolution:** only build C5 if it measures something T6 can't (skill-threat
tiles vs flood reachability) OR fold it in as the combo substrate (per-piece), not a
second territory term. This is exactly why C5 is "candidate, not committed."

### P5 — skill_capacity (T4) overlaps king_danger & offense economy (acceptable)
T4, T3-defense, and T3-incoming all read the same money/actions model (`affordable_casts`).
Shared *machinery*, not double-counting (T4 = throughput potential, T3 = a specific race),
so acceptable — but if T4's magnitude grows it can drown the per-piece signal. Designer
said ±30 is fine and low-priority. Leave as-is; just don't inflate it.

### Territory weight — worth a sweep (T6, low-risk experiment)
Designer wants to try making territory worth MORE and watch how the AI acts. Cheap
tunable (the `PER`-style weight at the `term_territory` call site / `Territory::SCALE`).
Not a structural change — sweep it after the C2/C4 pair lands so we're not tuning against
a moving eval.

---

## Highest-impact read

- **P0 (king-as-champion) first** — it's a correctness bug, and every factor C2/C4 add is
  missing on the king until fixed. Cheap and unblocks correct measurement of everything
  else.
- **C4 (exposure) is the highest-impact factor** — the missing multiplier that makes the
  two valuable compositional pairs work. It scales the WHOLE piece value (so whatever makes
  a piece valuable — offense, combo, being the last support/mystic carrier — makes losing
  it hurt), not just offensive pieces. Keep overextension/exposure disjoint (P1).
- **C2 (offense) is the enabler** — exposure has little to bite on until pieces are worth
  more than filler. Sequence P0 → C2 → C4 stands; `offense_capable` becomes the non-switch
  utilisation term with C2 (P2).
- **Hold C5** until P4 is answered. **Sweep territory** (T6) after the pair lands.
- Net: king fixed to full chain, two new per-piece factors (offense, exposure), one side
  term *reframed* (offense_capable → utilisation), zero new side terms, one tunable sweep
  (territory). Fewer moving parts, more interaction — not bloat.

## The turtle risk is a design canary, not just an eval bug

Recorded because it reframes the whole exposure effort: gating value behind exposure is
correct, but if it makes the AI passive, do NOT chase it with eval knobs. That passivity
is the eval faithfully reporting that the *game* doesn't reward commitment enough — which
is a core-design signal to act on (RULES / mechanics), per the designer.

---

## 2026-08-06 replay analysis — new findings folded in

Two matches (P1 heuristic, P2 custom). Full symptom→owner mapping lives in
`custom-eval-redesign.md` (Playtest observations section). Matrix-relevant deltas:

- **Confirmed bug FIXED (2026-08-06): Dash counted in combo.** `combo_reach_of` treated all
  `SkillCategory::Move` as combo-ticking; now uses `skill_ticks_combo` = Strike, or Move with
  `TargetOwner::Enemy | Either` (Blast/Shove). Self-move (Dash/Retreat → Empty) and Swap/Heal/
  Plate (Ally) no longer grant phantom combo reach. Sharpened T2; no matrix cell changed.

- **T6 (territory) reshape LANDED (2026-08-06, signed off).** Guard pre-flood (full R2
  footprint claimed before the champion-speed loop, blocked + tie-resolved) + base square
  `1 → 2` (near-king ×3/×2 kept multiplicative). This strengthens the **pieces × territory**
  overlap: a forward guard now legitimately dominates more squares (that's the point — fixes
  N2/N3/N5). Watch it doesn't *double*-pay when a guard-presence piece factor lands later:
  territory pays square-control, the piece score pays piece-worth. Part 3 (weight sweep) still
  pending, deferred until after C2/C4.

- **C4 exposure gained three inputs (still one factor, still disjoint from overext):**
  (1) reachability must subtract **bodyguard cover** (a screened piece isn't Move-Attackable);
  (2) **self-protection** (own Shield/armor, reachable Plate/Heal) raises effective health;
  (3) **injured-no-healer decay** — a 1-hp piece with no reachable Heal is permanently worth
  less. All three feed the eff-health-vs-incoming input; none re-measure overextension.

- **N4 standoff/breakthrough = another design canary** (like P3): if C2+C4+N1 don't yield
  breakthroughs, the ruleset under-rewards initiative — a RULES fix, not an eval knob.

- **Presence vs removability split:** the designer's "show a piece's presence, then modulate
  by how removable it is" splits cleanly across existing terms — *presence* = territory
  (N1) + (future) guard-presence factor; *removability* = C4 exposure. Don't build a third
  presence map inside exposure.
