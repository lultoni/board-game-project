# Custom Evaluator — Redesign & Calibration (Phase 1: eval)

Status: DRAFT / not started. Sibling plan: `custom-eval-search-cliff.md` (Phase 2, do AFTER this).
Owner: designer writes the eval; assistant implements precise designs + benchmarks.

---

## Context

The custom evaluator (`game/crates/core_engine/src/search/evaluator/custom.rs`) is a
per-piece contextual scorer: `score_piece(ctx, p)` returns one value per piece (base 100,
bent by interacting factors), plus a few write-once side terms; the driver signs by owner
and sums. Full per-piece + side-term breakdowns are now dumpable via
`search_bench --report --eval custom-stub --fen "…"` (or `--corpus`).

Running that report over 17 real game positions (saved at `game/tools/critique_fens.txt`)
exposed that **~95% of every position is flat ±100 material**. The interacting factors barely
move: `combo_overlap_bonus` adds a small flat +5..+60, `overextension_rate` subtracts
−16..−50, and `king_danger_malus` (−400) fired in **1 of 17** positions. The eval therefore
cannot tell a well-set-up, defended, active piece from a loose or hanging one — which is the
root of all three observed play defects.

The designer diagnosed three failures from watching the AI play; the fixes below trace to
those. Crucially, this is a **calibration pass, not a term-adding spree**: we only keep a
change if it makes the eval *more* meaningful (see the Non-Regression Principle below).

---

## Load-bearing principles (the custom eval's north star)

These are the design goals, read between the lines of the designer's guidance. Encode them as
the file's guiding doctrine.

1. **The piece score IS the whole model — no additive bonuses.** Every factor is a
   multiplier/weight on the `100` base. A piece's value must always reflect its *full context*
   (activity, combo potential, restriction it imposes, exposure, hp/armor-vs-threat). We do
   NOT want inflated scores from tacked-on bonuses; we want `piece = 100 × f(context)`.
   *Consequence:* losing an offensive champion must hurt because that champion was genuinely
   worth more, not because a bonus vanished.

2. **Context interaction is the point.** A factor that is +60 in isolation must be *scaled
   down* when the piece is itself overextended/exposed. Designer's canonical example: a wounded
   piece in the corner doing nothing → healing it barely raises its value (it still does
   nothing); advancing it raises value (more active) but exposes it to bigger loss if it
   becomes threatened. The eval must express both directions on the SAME piece.

3. **Advantage-keeping = don't lose the pieces that give you the advantage.** When ahead, the
   eval should make *hanging an offensive piece* a real drop, and should let the AI *sack
   filler* to preserve the advantage. Offensive strength should scale so having more attackers
   is worth more (and losing one is felt).

4. **Reward restriction, not proximity — with SAFE pieces.** Good play = removing the
   opponent's control / threatening with well-placed pieces that are *themselves defended* (not
   glass cannons). The restriction value of a piece must therefore interact with its own safety:
   a piece that restricts the enemy but is itself hanging/overextended should NOT be credited as
   if the restriction were secure. Explicitly NOT "I have forces nearby" as a close-out trigger —
   activation of ANY offensive piece to overload the enemy (even from a corner) is the close-out
   driver.

5. **Never over-punish progression.** Exposure/threat weighting must not force retreats or
   forbid tempo/trade/king-hunt sacrifices. A trade win, tempo win, or endangering the enemy
   king must be able to *outweigh* a piece-safety penalty. If a safety term makes the AI
   turtle, it's mis-tuned.

6. **Non-Regression Principle.** A good eval moves in ONE direction under best play; it should
   not swing winning→losing on a single ply (the heuristic eval's failure mode). Every change is
   validated against this: adjacent plies of best play produce monotone/smooth eval, not sign
   flips, on the sample positions. A term that adds noise without adding *aussage* (meaning) is
   reverted, not kept.

7. **The DESIGNER is the tuning authority — this is THE acceptance gate.** Numbers are never
   "correct by construction". After EVERY change, the assistant re-dumps the per-piece +
   side-term breakdown on the example positions and SHOWS the designer exactly how much each
   piece is now worth. The designer then decides one of: (a) good as-is → keep; (b) needs
   further balancing → adjust weights/curves and re-show; (c) the approach doesn't match the
   vision → more fundamental redesign of that factor. **No change is accepted, and the sequence
   does not advance, until the designer signs off on the shown values.** The assistant does not
   self-certify a factor as "done" — it proposes, shows, and tunes to the designer's expectation.
   The Non-Regression check (principle 6) and the tooling below are *inputs* to that judgment,
   not a substitute for it.

---

## The changes

Ordering matters: each builds on the previous, and each is gated on the Non-Regression check
before the next lands. All edits are in `custom.rs` unless noted.

### C1. `combo_overlap_bonus` → `combo_multiplier` (add-on becomes a weight)

Today `score_champion = base + combo_overlap_bonus` (flat additive, lines ~112–117, 135–187).
Change to a **multiplier on the piece's contextual base**, so a combo-set-up champion that is
itself weak/exposed does NOT get the full combo value.

- Compute a combo *factor* `≥ 1.0` (e.g. `1.0 + k·combo_strength`) rather than a point bonus.
- Apply it to the SAME base that exposure already scaled: conceptually
  `champ = 100 × (1 − overextension_rate) × combo_factor` — so an overextended champ (low base)
  gets a proportionally smaller combo reward. This directly implements principle 2 (the +60
  can't survive on a piece only worth 20).
- **Conditioning** (designer chose "require a real follow-up" + "weight by target quality"):
  - Only count overlap where a *different* combo-ticking champion can realistically land the
    follow-up (a genuine second ticker), not any two reaches crossing an empty square the
    champion would never occupy.
  - Weight the factor by *what is on/near the target* (enemy champion/king > guard > empty),
    reusing the existing proximity tiers but as a multiplier input, not points.
- **Open sub-question to resolve during impl:** whether combo potential should instead be
  measured by a board-wide **skill-control map** (see C5) and the champion multiplied by *its
  contribution* to that map. Prototype the local multiplier first (cheaper, less risk); only
  promote to the map form if the local version can't express "two pieces jointly controlling a
  tile" cleanly.

### C2. Piece value scales with offensive strength (hypo-2 core)

The AI hangs offensive champions because an offensive champ is worth ~the same as filler.
Make an offensive piece genuinely worth more so its loss registers as a real
advantage→disadvantage move.

- A champion carrying a **Strike** (and, when a Strike-priming partner exists, a Move) skill
  gets a higher contextual base than a passive piece — folded into the per-piece multiplier,
  NOT a flat add-on (principle 1).
- Scaling should be **super-linear in count** enough that losing your *last* attacker hurts
  most (the "oops my last strike piece is gone" failure). Consider deriving the per-piece
  offensive weight partly from the side's attacker count so the marginal attacker is felt.
- This replaces the weak `term_offense_capable` side term (currently ±10/champ, often 0) with
  per-piece value the search actually defends. Decide during impl whether the side term stays
  as a small tie-breaker or is removed to avoid double-counting.

### C3. `king_danger_malus` — make it fire, and make it reward the attacker

The malus already models **both** Move-Attacks (R2 gate, `+1`) and Strikes (R4 + clear path);
that part of my earlier critique was wrong. The real problems:

- **It fires far too rarely (1/17).** The gate is too binary/conservative. Loosen so a
  genuinely pressured king (e.g. enemy force adjacent, or multiple attackers converging)
  registers, without firing on every distant piece. Re-measure the fire-rate on the 17
  positions after loosening — target: fires when a human would say "that king is in trouble."
- **Confirm the attacker actually benefits.** `score_king` returns `−malus` for the endangered
  king's own side; the driver signs by owner, so a P2-king malus already becomes +P1. Verify
  this end-to-end on a position where P2's king is pressured and confirm the P1-POV total rises
  — if it doesn't net through, fix the sign path. This is the "reward the other player
  indirectly for triggering it" requirement.
- This is also the **close-out driver** (principle 3/4): a symmetric, always-on "enemy king in
  danger" pressure that rewards activating ANY offensive piece against the enemy king,
  regardless of distance — NOT gated on "forces nearby".

### C4. hp/armor weighting in relation to exposure (hypo-3)

Today hp/armor are invisible to the score (a hangable champ reads the same as a safe one).
Do NOT reward random armor. Instead:

- Make effective health matter *only in proportion to how threatened/exposed the piece is*.
  A safe piece: hp/armor ≈ no effect (healing it is worth ~nothing — matches the corner
  example). A threatened/exposed piece: low effective health *reduces* its contextual value
  (it's about to be lost), high effective health *preserves* it.
- Implement as another multiplier input on the same base, interacting with `overextension_rate`
  / the threat gate — so it composes with C1/C2, not stacks additively.
- **Guard against turtling (principle 5):** this must be dominated by C3 (enemy-king danger)
  and by material swings from trades. Explicitly test that a favourable trade or a king-hunt
  still scores better than the piece-safety penalty it incurs.

### C5. (CANDIDATE, not committed) skill-control / restriction map

A territory-like term counting tiles controlled/threatened by *skills* (offensive-pressure
map), rewarding restriction of the opponent (principle 4). Two possible roles: (a) a side term
like `territory`, and/or (b) the substrate for the combo multiplier (C1).

**Do NOT build this until C1–C4 are calibrated and pass the Non-Regression gate.** It is the
most likely to add noise. Gate its inclusion on: does it make the eval *more* directional on
the sample set, or just louder? If louder, drop it.

### C6. `overextension_rate` gate tweak (small)

Currently: enemy in R3 → full weight, only R4 → half, else 0 (lines 202–213). Designer leans
toward a pure R3 gate. Decide during impl: drop the R4 half-tier (cleaner, "only real R3
pressure counts") vs keep it. Low-stakes; fold into the C1 rework since both touch the champ
base.

### Explicitly NOT doing now

- Guard/bodyguard *blocking* term (screening enemy skill targets / intercepting Move-Attacks):
  designer flagged it as an interesting candidate but does NOT want speculative terms.
  Revisit only if C1–C4 leave a clear, demonstrable gap.
- `skill_capacity` money-conversion refinement: designer says better finances are just better;
  the ±30 swings are fine and not a priority. Leave as-is.

---

## Verification (per change, and overall)

The gate is the **designer's sign-off on the shown per-piece values** (principle 7). The tools
below produce what is shown and are inputs to that judgment — not an auto-accept.

- **Per-position breakdown (what the designer reviews):** after every change, re-dump
  `search_bench --report --eval custom-stub --fen "…"` (single) or
  `--corpus game/tools/critique_fens.txt` (all 17) and PRESENT it to the designer: which pieces
  are now worth what, and why. The designer decides keep / rebalance / redesign (principle 7).
  Do not proceed until they sign off.
- **Non-Regression check (principle 6) — an input to the designer's call.** For the before/after
  pairs (pos-9→10, pos-12→13, pos-14→15, pos-16→17) and a few self-play plies, confirm the eval
  moves smoothly/monotonically under best play and does NOT flip winning↔losing on one ply.
  Consider a small deterministic unit test pinning "no single-ply sign flip" on scripted best-play
  sequences.
- **Consistency invariant:** the report's CHECK line (`Σ piece + Σ side = total = evaluate()`)
  must always hold — it's already asserted in tests; keep it green.
- **Cheap compile/logic loop:** `cargo check` between edits; `cargo test -p core_engine --lib
  custom` at each change boundary (WIP exact-value tests may be loosened per designer — they are
  not the gate; the Non-Regression behaviour is).
- **Do NOT** run `cargo test -p nn_trainer` (30+ min, per project memory).

## Files

- `game/crates/core_engine/src/search/evaluator/custom.rs` — all eval changes.
- `game/tools/critique_fens.txt` — the 17-position sample (already saved) used for every
  before/after eyeball and the Non-Regression gate.
- No search-side edits in this plan (see the sibling search-cliff plan).

## Sequencing

C1 → (designer sign-off) → C2 → sign-off → C3 → sign-off → C4 → sign-off → *then* consider
C5/C6. Each sign-off is a hard stop: the assistant shows the piece values, the designer decides
keep / rebalance / redesign, and the sequence advances only on approval. If a change adds noise
rather than meaning, revert it. Commit only when the designer asks.
