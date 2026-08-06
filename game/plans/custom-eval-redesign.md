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

## Two distinct safety axes: support ≠ exposure (designer insight, 2026-08)

A key clarification from reviewing C1 on a live position. **Over-extension (support)
and exposure (vulnerability) are two different things and both are valuable factors —
they must NOT be collapsed into one.**

- **Support / over-extension** (`overextension_rate`, surfaced as `factor_overextension`):
  measures *"are your pieces clustered or pushed out alone?"* Its strategic message is
  **"move pieces together."** It is gated on *some* enemy being within R4, but it says
  nothing about whether a specific piece can actually be hit/killed next turn. Standing
  alone is not inherently bad.

- **Exposure / vulnerability** (C4, not yet built): measures *"can the opponent
  actually reach and kill this piece, and can I retaliate/defend?"* It is about
  reachability (Move-Attack / Strike), effective health (hp + armor vs incoming), AND
  the economy to punish/answer the trade. In the reference position the champs standing
  *in front of their own guards* were **free food**: reachable + no armor + no money to
  retaliate. `overextension_rate` was blind to all three (it saw friends in the rings →
  "supported" → safe).

**How they relate:** as long as you are NOT vulnerable, you don't care about exposure.
But the moment you ARE vulnerable, you wish you'd prepared — and *that* preparation is
what support/over-extension guides ("moving pieces together"). So support is the
cheap-insurance axis; exposure is the did-the-risk-come-due axis. They compose
multiplicatively: a piece that is BOTH unsupported AND vulnerable is worst.

**Naming fix (done in C1):** the C1 factor was mis-named `factor_exposure`; it wraps
`overextension_rate` (support/clustering). Renamed to `factor_overextension`, freeing
"exposure" for the real vulnerability factor C4 adds.

**Orthogonal to both — move with purpose.** Support only captures the *"move pieces
together"* strategic angle. It does NOT replace the AI's need to move pieces with
**purpose**: activating "dead" pieces (a wounded piece doing nothing in a corner is
worth little even if perfectly safe) and pushing forward with attacking intent (the
combo-overlap / offensive-pressure axis). Support/exposure are the safety axes; combo
and offense (C1/C2/C5) are the purpose axes. A good eval needs both — safety without
purpose is turtling (principle 5), purpose without safety is free food.

---

## Playtest observations — replay analysis (2026-08-06)

Two matches watched (P1 = heuristic, P2 = custom). Raw observations mapped to existing
changes or flagged as new vectors. **Nothing here is a new numbers-guess — it's the
symptom → cause → which change owns the fix.** FENs kept verbatim for regression positions.

### Symptoms that EXISTING changes already own (sharpened, not new)

- **"Champs standing exposed, about to die, not seen as such."** Recurs constantly: the
  focus-dashed champ on c6 that dies next turn but reads +186; the exposed champs in the
  `gc[...]k[...]` positions; the champ dashed forward then taken 2 plies later. → **C4
  (exposure)** — the core missing axis. This is the single most-repeated complaint and
  confirms C4 is the highest-impact change. See the new C4 sub-requirements below (true
  Move-Attack + bodyguard, self-protection, injured-piece decay).

- **"Down offensively, but rushes in to grab the last few pieces instead of retreating /
  saving pieces to strike back decisively."** (the `3k[...]/1g1gg3` position; "generally
  I see it rushing in when it's down offensively"). → **C2 (offense utilisation) + C4**:
  the non-switch utilisation term (P2) is *exactly* this — a side that is behind on offense
  should be nudged to play cool / preserve, not trade down. Currently absent, so it trades
  into a lost position. Verify the utilisation term makes "preserve when behind" out-score
  "grab a guard."

- **"Wastes money on no-op skills"**: king Dash 1 tile sideways (−3 money, did nothing);
  `f5*d3:Blast +64` that did nothing but cost money; the expensive focus-dash that just
  hung a champ. → these are **search/eval interaction**, but the eval half is: a skill that
  changes nothing meaningful must not read as +64. Two owners: (a) **combo double-count** —
  see the Dash question below; (b) a move/skill that ends with the piece MORE exposed should
  net *negative* via C4, not positive. Flag: audit why Blast reads +64 with no combo and no
  material change (likely territory or a stale combo tick — verify on that exact FEN).

- **"King in the back row turn 1 just loses 2 money for shielding a non-strategic piece."**
  → **C4 + a new sub-point**: self-protection (Shield/Plate/armor) should only be *rewarded*
  in proportion to the protected piece's strategic value. Armoring filler = pure money loss
  (correct that it's punished by the economy), armoring a strategic piece = rewarded. C4
  already says "don't reward random armor"; this makes it concrete — armor's value is gated
  by the base value it's protecting (which is the whole-product rule again: armor raises
  effective health → raises exposure factor → scaled by the piece's own worth). No new term.

### NEW vectors this analysis surfaced (fold into the owning change)

- **N1 — Territory must use TRUE piece speed, and the bonus doubled.** The flood
  (`Territory::compute`) grows both sides with `king_expand` (uniform 1-ring waves) — it
  does NOT model that Guards move 2 and Champions/King move 1. A Guard should flood territory
  *twice as fast*. Designer also wants the control bonus **doubled** (base bonus PLUS the
  near-king bonus, not near-king replacing base). This reframes **C7** from a pure weight
  sweep into a *correctness + weight* change: (a) speed-aware flood (Guard fronts advance 2
  rings/wave, Champ/King 1), (b) additive king bonus (base + near-king) instead of the
  current multiplicative-only. Ordering unchanged (after C2/C4) but scope grew — noted in C7.

- **N2 — "Doubling guards / shuffling guards in the back" instead of advancing.** Guards sit
  inert in the back "almost the entire game," and the AI stacks a champ behind doubled guards
  rather than pushing a final guard forward to overload. Suspected cause: `factor_overextension`
  rewards clustering with no counter-pressure to *advance*, and guards have ONLY that factor
  (they get no purpose axis). → two owners: (a) the **guards-beyond-overextension** step (still
  TBD, but this is now concrete evidence guards need a *presence/advance* factor, not just
  clustering); (b) territory (N1) — if forward guard-presence were properly valued via
  speed-aware territory, advancing would out-score shuffling. Do N1 first; it may dissolve
  much of N2 before we design guard factors.

- **N3 — "Retreating guards, ceding board control."** (`b7-a8 AI -11`, `c7-a7 AI +18` —
  pulling guards to the back rank read as *better*). Board control is leaking because
  territory is under-weighted AND speed-blind (N1). This is the same root as N2. N1's doubled,
  speed-aware territory is the direct fix; re-check these exact FENs after N1.

- **N4 — Standoff / breakthrough behaviour (systemic).** The `2c[...]k[...]` standoff: the
  designer wants the AI to either break through OR never enter a dead standoff in the first
  place. This is **not purely an eval knob** — it's the same family as the turtle canary (P3):
  if the eval faithfully values control + safety and the AI *still* prefers a frozen standoff,
  that's a signal the **ruleset under-rewards initiative/commitment** (a core-design lever, not
  an eval hack). Record it as a design-watch item, not a factor. Revisit after C2+C4+N1 land —
  if breakthroughs still never happen, it's a RULES conversation.

- **N5 — Does not clear deep-lying enemy guards from its own ranks.** A deep enemy guard has
  outsized presence (screens, blocks paths, enables the enemy's champs to follow). The AI
  never prioritises removing it. → this is the flip side of N2's "presence" idea: enemy
  presence in *my* territory should register as a threat worth removing. Likely falls out of
  N1 (speed-aware territory: a deep enemy guard contests a lot of my squares) + a guard
  presence factor. Watch after N1; don't build a bespoke term yet.

### One correctness question to answer immediately — ✅ FIXED (2026-08-06)

- **Q — Is Dash counted in combo overlap? It must NOT be. — CONFIRMED BUG, now FIXED.**
  `CustomCtx::combo_reach_of` counted a skill as combo-ticking if its category was
  `Strike | Move`, wrongly including **self-movement** (Dash, Retreat) and ally-relocation
  (Swap). **Fix landed:** added `skill_ticks_combo(s)` — Strike (always), or a Move skill
  whose `TargetOwner` is `Enemy | Either` (Blast, Shove); everything else (Dash/Retreat →
  `Empty`, Swap/Heal/Plate → `Ally`, Shield/Focus/Charge → `SelfOnly`) returns false.
  `combo_reach_of` now uses it, so a Dash-only champ has zero combo reach and the partner
  reach cache is corrected in the same pass. Test: `combo_reach_excludes_self_move_skills`
  (Dash → 0 points; Blast → >0). No sign-off needed (per designer). 15 tests green.

---

## The changes

Ordering matters: each builds on the previous, and each is gated on the Non-Regression check
before the next lands. All edits are in `custom.rs` unless noted.

### C1. `combo_overlap_bonus` → `combo_multiplier` (add-on becomes a weight) — ✅ DONE (2026-08)

**Landed and signed off ("keep C1 as is for now").** Implemented as a factor-chain:
`score_champion = BASE × Π factor`, where factors are registered in `CHAMP_FACTORS`
(add a factor = one fn + one line, never touch the fold). Two factors so far:
`factor_overextension` (`1.0 − overextension_rate`, clustering) and
`factor_combo` (`1.0 + 0.5 × min(1, points/50)`, ceiling ×1.5). `combo_overlap_bonus`
renamed `combo_overlap_points` (raw points feeding the factor, tiers 25/10/5). Because
combo multiplies the base, an unsupported champ's combo reward shrinks with its support
factor — no longer a flat add-on (principle 1/2).

*Known gap deferred to C4:* the reference c6 champ still reads high because its rings
are supported (`factor_overextension` ≈ 1) even though it's killable next turn — that
"free food" signal is exposure/vulnerability, which C4 owns. See the two-axes section.

*Historical spec (as-implemented above):* the original C1 called for a `≥ 1.0`
combo factor applied to the exposure-scaled base, conditioned on a genuine second
combo-ticker and weighted by target quality (enemy champ/king > guard > empty, via the
existing proximity tiers). The board-wide skill-control-map alternative (C5) was
deferred — the local multiplier was prototyped first and signed off, so C5 is only
promoted if the local form can't express joint control cleanly.

### C1b. Move-Attack reachability fix — ✅ DONE (2026-08)

Correctness fix surfaced while discussing exposure. "Enemy within R2 → can Move-Attack"
is WRONG: a Move-Attack needs the attacker to actually reach the target. Only a **Guard**
(speed 2) threatens from R2; a **Champion/King** (speed 1) threatens only from R1. Added
`CustomCtx::enemy_can_move_attack(is_p1, sq)` = (enemy Guard within R2) OR (enemy
Champion/King within R1), and rewired `king_danger_malus`'s gate to use it (was raw R2).
The C4 exposure factor will reuse the same helper so danger and exposure agree on what
"reachable" means. Test: `move_attack_reach_guard_r2_champ_r1`.

### C1c. King is a Champion under the hood — score it as one (P0, do before C2)

`score_king` currently returns ONLY `-king_danger_malus`, discarding everything else. But
the king has 2 equip slots, carries skills, can Strike, can tick a combo, controls
territory, occupies space — it IS a champion. A skill-carrying king reads as pure liability
today. **Change:** the king runs the FULL champion factor chain (`CHAMP_FACTORS`:
overextension, combo, offense, exposure) PLUS `king_danger_malus` on top. Do this before C2
— every per-piece factor we add is silently missing on the king until it's fixed. Watch the
sign path: the malus stays a subtraction from the king-owner's total; the chain value is a
positive contribution like any champion. Re-dump and confirm a skill-carrying king now
reads as a valuable-but-protected piece, not pure downside.

### C2. Piece value scales with offensive strength (hypo-2 core)

The AI hangs offensive champions because an offensive champ is worth ~the same as filler.
Make an offensive piece genuinely worth more so its loss registers as a real
advantage→disadvantage move.

**C2a. `factor_offense` — ✅ DONE & signed off (2026-08-06).** New champion factor (King
included: C1c). Per-piece worth keyed on the side's Strike-carrier count via a hand-authored
curve `1→2.0, 2→1.75, 3→1.57, 4→1.43, 5→1.31, 6→1.22`, tuned so the COST to lose one attacker
is strictly larger the fewer you hold (200/150/120/100/85/75 — no bounce-back; pinned as a
hard test assertion). Strike carrier → full curve; Move-only carrier (Blast/Shove, gated on a
Strike existing) → half the bonus-above-1.0, does NOT change the count; neither → 1.0.
`SideInfo` now counts the King in `strike_champs`/`move_champs` and its combo reach. Verified
no perverse "gain from losing a piece" (total offense mass strictly rises with count). Known
gap: does NOT yet interact with exposure — a doomed 1-hp attacker still reads high; C4 scales
it down. Test: `offense_factor_rewards_last_attacker_most`.

**C2b. Reframe `term_offense_capable` → non-switch utilisation term — ✅ DONE & signed off (2026-08-06).**
No longer a raw attacker count. Now penalises a side ONLY when it is ahead on offense and not
converting, scaled by convertibility — returns a non-positive magnitude (never rewards), so
the double-count with `factor_offense` is gone. Three multiplicative gates (continuous, no
switch): (1) **advantage** `adv = my_potential − enemy_potential`; `adv ≤ 0` → 0 (free when
behind/even, designer's rule); (2) **realisation** weighted by best enemy target an attacker's
reach covers — King 1.0 / Champion 0.7 / Guard 0.3 — gap = `1 − realising_frac`;
(3) **takeability gate (PLACEHOLDER)** = fraction of my attackers not `enemy_can_move_attack`-able
→ REWIRE to read `factor_exposure` at C4. `penalty = −WEIGHT × adv × (1−realising_frac) × safe_frac`,
`WEIGHT = 33` (≈ −100 for 3 fully-unrealised safe attackers, designer-set).
**Correctness fix landed alongside:** `move_champs` and `piece_offense_role` now use
`skill_ticks_combo` (enemy-moving Move only) instead of raw `SkillCategory::Move`, so a
Dash/Retreat-only piece is no longer miscounted as offense (it inflated `adv` and wrongly got
the `factor_offense` half-bonus). Test: `offense_utilisation_penalises_unconverted_advantage`.

**Do-now audit before C2 (from the 2026-08 replay, question Q):** verify Dash is NOT counted
in `combo_overlap_points`. Dash is self-movement → per RULES it never ticks a combo counter,
so it must not feed combo. The no-op king-Dash and `f5*d3:Blast +64` reading as gains point at
either a combo mis-credit or missing "real target / partner present" gating. Fix any
mis-credit here first — C2's utilisation term reasons about "realising offense," and it must
not build on a combo signal that fires for no-ops.

### C3 + C4 build together — one shared "survivability" race, two consumers — ✅ DONE (2026-08-06)

C3 (king danger) and C4 (piece exposure) ask the SAME question — *is this piece about to be
lost, and can its owner do anything about it?* — so they were built as ONE standardized
mechanism (designer: "standardise it"). **Implemented:** `CustomCtx::survivability_severity`
returns bounded `[0,1]`, consumed by THREE callers: `king_danger_malus` (C3),
`factor_exposure` (C4), AND the C2b utilisation gate (the placeholder is retired — the gate now
uses the mean `factor_exposure` of the side's attackers). Live details:
  - **Two reach vectors:** Move-Attack (`enemy_can_move_attack` minus `bodyguard_fully_covers`,
    checked against every open approach square) + Strike (`enemy_strike_reaches`, range+1, clear
    path, distance-scaled via `nearest_enemy_striker_dist`; bodyguard does NOT save vs Strikes).
  - **Race:** imminent incoming (2-round window, NOT the old 5) vs owner's Shield/Plate/Heal
    (only on the owner's turn) + eff-health; retaliation damp ×0.6 if we can Strike back.
  - **Certain-death boost:** lethal incoming + no possible defense floors severity at 0.85.
  - **King malus:** `CAP(6000) × severity²` — bounded, escalating, strictly below MATE_SCORE.
    FIXES the old unbounded `netto × 400` that produced −1200. Attacker rewarded via owner-diff
    (verified: pressured P2 king lifts P1-POV total to +4823 in the dump).
  - **`factor_exposure`:** maps severity to `[floor .. 1.0]`, floor 0.1 (1hp) → 0.5 (2hp+2armor)
    so a 1-hp dead-to-rights piece bottoms near ×0.1.
  - Tests: `exposure_scales_with_vulnerability`, `exposure_fires_on_strike_only`,
    `king_exposure_certain_death_is_capped`, plus the updated `king_danger_gates_and_penalises`
    and `offense_utilisation_penalises_unconverted_advantage`. 19 custom tests green.

**Original design notes (as-implemented above):**

C3 (king danger) and C4 (piece exposure) ask the SAME question — *is this piece about to be
lost, and can its owner do anything about it?* — so they are built as ONE standardized
mechanism (designer: "standardise it"). Structure:

**Step 1 — extract a shared, BOUNDED survivability helper.** One function estimates a small
exchange for any piece: incoming (reachable, affordable) vs the owner's ability to shore it up
+ its effective health, and returns a **bounded severity in `[0.0, 1.0]`** (0 = safe,
1 = dead-to-rights). Both the king malus and `factor_exposure` consume this — they agree by
construction on what "about to die" means.

The two reach vectors feeding "incoming" (BOTH now):
  - **Move-Attack vector:** `enemy_can_move_attack` (Guard@R2 / Champ-King@R1) **minus
    bodyguard cover**. Bodyguard cover must be computed **against the opponent's actual
    approach squares** — the Rule needs a friendly Guard adjacent to *both the pre-target tile
    (along that approach) and the defended piece*. A piece is safe on this vector only if every
    viable enemy approach is covered; mirror the `showBodyguardCover` geometry, don't just
    check "a guard is next to me."
  - **Strike vector:** an enemy Strike hits it — true range of the enemy's Strike (max range
    if two different Strikes) **+1**, straight-line path must be clear (skills hit direct →
    bodyguard does NOT save here). **Scales with distance:** a point-blank Strike threat
    contributes more than a far one.

Survivability inputs:
  - **Incoming** = affordable enemy strikes/Move-Attack that actually reach (money ÷ strike
    cost, capped by actions). Tighten the horizon to IMMINENT lethality (this turn + next),
    NOT a 5-round spend — the 5-round sum is what over-counted "incoming" (see the −1200 note).
  - **Defense (heal-vs-damage race)** = the OWNER's Shield/Plate/Heal, but only usable **on the
    owner's own turn** (on the enemy's turn the defender can't heal — gate the heal side by
    whose turn it is). Can the piece be protected faster than it can be damaged?
  - **Retaliation economy** = a piece is LESS exposed if, after the trade, the enemy can't be
    struck back next turn (they lack the Strike skills or the money). Retaining the means to
    punish the trade reduces exposure — this is the c6 "free food" fix.

**Step 2 — the KING maps severity to an escalating-but-CAPPED malus (C3).** The current malus
is `netto × PER_DAMAGE` with `PER_DAMAGE = 400`, LINEAR and UNBOUNDED over a 5-round window —
that is exactly how a merely-pressured king read **−1200** (`netto ≈ 3 × 400`): the 5-round
`affordable_casts` over-counted one-turn lethality, and 400/unit has no ceiling. **Fix:** drive
the malus from the bounded severity instead — an escalating curve that is big when the king is
genuinely dead-to-rights but **capped strictly below `MATE_SCORE`** (near-death ≈ a large
fraction of mate, never exceeding a real forced mate, so the search never prefers "probably
dead" over an actual capture). Also loosen the *gate* so a genuinely pressured king fires more
than 1/17 — but severity, not the gate, carries the magnitude now.

  - **Attacker reward (unchanged requirement):** `score_king` returns `−malus`; the driver
    signs by owner, so a pressured P2 king already lifts the P1-POV total. Verify end-to-end.
  - This is the **close-out driver** (principle 3/4): symmetric "enemy king in danger" pressure
    rewarding activation of ANY offensive piece against the enemy king, regardless of distance.

**Step 3 — PIECES map the same severity to `factor_exposure` (C4).** See the C4 block below;
it multiplies the whole chain by `[~0.1 .. 1.0]`, amplified by low effective health.

### C4. Exposure / vulnerability — the missing safety axis (hypo-3)

**This is the `factor_exposure` the C1 rename freed up** — the *second* safety axis,
distinct from `factor_overextension`. See the two-axes section above. Today hp/armor are
invisible to the score (a hangable champ reads the same as a safe one), and support
(clustering) can't tell "free food" from "fine". C4 adds the vulnerability read.

Reference failure it must fix: champs standing *in front of their own guards* — rings
supported (so `factor_overextension` ≈ 1.0, no penalty) yet reachable + no armor + no money to
retaliate = free food. That is exactly what this factor must catch.

Do NOT reward random armor. Instead:

- The factor is `1.0` when the piece is not actually vulnerable (unreachable, or safe
  enough that healing it is worth ~nothing — the corner example). It drops below `1.0`
  only when the piece is genuinely reachable AND its effective health (hp + armor) is
  low against the incoming AND the side can't cheaply retaliate/answer the trade.
- **Consumes the shared survivability severity (C3 step 1).** Reachability (both vectors,
  bodyguard-aware), heal-vs-damage race, and retaliation economy all live in the shared
  helper; `factor_exposure` maps its bounded severity `[0,1]` to a multiplier `[~0.1 .. 1.0]`
  (severity 0 → ×1.0, severity 1 → deep cut). Do NOT re-derive the race here.
- Implement as another entry in `CHAMP_FACTORS` (`factor_exposure`). Because it multiplies
  the accumulated `BASE × overextension × combo × offense`, it **scales the whole piece
  value** — so whatever makes a piece valuable (offense, combo, being the last
  Heal/Plate/Mystic carrier) makes losing it hurt more. It is NOT offense-specific; it
  protects "the piece holding the most strategic value" without its own value calc
  (designer point 3). A piece that is BOTH over-extended AND vulnerable is worst,
  automatically.
- Keep it disjoint from `factor_overextension` (matrix P1): overextension = clustering
  (empty rings); exposure = concrete killability (reachable + eff-health-vs-incoming +
  can't-retaliate). They multiply; they must not re-measure the same thing.
- **Reachability must be TRUE Move-Attack reachability, minus bodyguard cover (2026-08).**
  Reuse `enemy_can_move_attack` (Guard@R2 / Champ-King@R1), but a piece protected by the
  **Bodyguard Rule** is NOT reachable via that Move-Attack — a friendly Guard adjacent to
  both the pre-target tile and the piece removes that attackability entirely. Subtract
  bodyguard-covered attack vectors before deciding "reachable", or exposure will punish
  pieces that are actually screened. (There's a `showBodyguardCover` legibility aid already;
  mirror its geometry.)
- **Self-protection counts (2026-08).** A piece's effective health includes its own armor
  AND its castable self-defense (Shield → +1 armor, a reachable Plate/Heal ally). A piece
  that can Shield itself before the incoming lands is less exposed. Fold this into the
  effective-health-vs-incoming input, not a new term.
- **Presence, then durability (designer framing, 2026-08).** The deeper ask underneath
  exposure: a piece's *presence* (the control/threat it projects onto squares) should show
  in the score, and THEN be modulated by how hard it is to remove from that square —
  effective hp+armor vs the enemy's ways to take it OR push it away (Blast/Shove/Swap, not
  just damage). Exposure is the "how removable" half; the "how much presence" half overlaps
  territory (N1) and the guard-presence idea (N2). Keep exposure focused on removability;
  don't smuggle a presence map in here.
- **Injured pieces amplify exposure (resolved 2026-08-06 — lives HERE, not a separate term).**
  Low effective health doesn't just feed the race — it deepens how far exposure can drag the
  multiplier. Designer anchor: a **1-hp, unsupported piece fully surrounded by enemies → ×0.1**
  (worth ~10%). A 1-hp piece with no reachable Heal is a permanent haircut (it can't recover);
  fold this as the low-health amplifier on the exposure multiplier rather than a standalone
  `factor_wounded`. Keep it disjoint from overextension (clustering).

- **Guard against turtling (principle 5) — with a design escape hatch:** exposure must be
  dominated by C3 (enemy-king danger) and by material swings from trades; explicitly test
  that a favourable trade or king-hunt out-scores the piece-safety penalty it incurs. **But
  if, after honest tuning, the AI still prefers to turtle rather than convert, do NOT chase
  it with eval knobs** — that passivity is the eval faithfully reporting that the *ruleset*
  under-rewards commitment, which is a core game-design signal to act on (matrix P3). The
  eval must not fake an incentive the game doesn't provide.

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

### C7. Territory — speed-aware flood + bigger base (parts 1+2 ✅ DONE & signed off 2026-08-06)

Grew from a pure weight sweep into a **correctness + weight** change (2026-08 replay N1/N2/N3).
Three parts:

1. **Speed-aware flood (correctness) — ✅ DONE.** Added a **guard pre-flood** to
   `Territory::compute`: before the shared champion-speed wave loop, each side's guards
   claim their full **R2 footprint** (two `king_expand` rings, each blocked by pieces/walls
   via `& empty` so no jumping), P1-vs-P2 tie-resolved on that whole footprint and folded
   into `claimed`. Only then does the 1-ring/wave main loop run, so a champion can only *tie*
   a square a guard already reached in one turn, never steal it. Test:
   `territory_guard_speed_beats_champion`. On the N3 case, advancing a guard now beats
   retreating it (+36 territory swing) — directly targets guard inertia (N2/N3/N5).
2. **Bigger base (NOT additive — designer corrected) — ✅ DONE.** Base worth per plain
   controlled square raised `1 → 2` (`Territory::BASE_SQUARES = 2`); contested still halves.
   The near-enemy-king bonus **stays MULTIPLICATIVE** (R1 ×3, R2 ×2) on top of the doubled
   base — the earlier "additive" framing was a misread. So an R1 square = 2×3 = 6, plain = 2.
3. **Weight sweep (original) — PENDING.** Sweep `BASE_SQUARES` / call-site weight and watch
   behaviour. Deferred to AFTER C2/C4 land so we're not tuning against a moving eval.

Watch the territory×pieces overlap (matrix) — don't let it re-pay what piece activity already
pays.


### Guards — factors beyond overextension (TBD, do not lock in)

Guards should get more than just `factor_overextension` (defensive covering, offensive
presence, …), but the specific factors are **deliberately undecided** — not the first
instinct. Revisit as a designed step, not a reflex. (Skill-only factors combo/offense
correctly stay 1.0 for guards.)

**2026-08 evidence (do N1 first, then decide):** guards sit inert in the back all game
(N2), retreating them reads as fine (N3), and the AI never clears deep enemy guards from
its ranks despite their screening presence (N5). This is concrete evidence guards need a
*presence/advance* signal, not just clustering. BUT the leading hypothesis is that
speed-aware, doubled territory (C7/N1) already makes forward guards worth far more —
possibly dissolving most of N2/N3/N5 without a bespoke guard factor. So: **land C7/N1
first, re-watch these positions, and only then design guard factors for whatever gap
remains.** Don't lock in a guard-presence factor before N1's effect is measured.


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
  `search_bench --report --eval custom --fen "…"` (single) or
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

**Done so far:** C1 ✅ + C1b ✅ (reachability) → C1c ✅ (king-as-champion) → Q ✅ (Dash-combo
filter) → C7 parts 1+2 ✅ (guard pre-flood + base 1→2) → C2a ✅ (`factor_offense`) → C2b ✅
(`offense_capable` → utilisation term, with the Dash-miscount fix) → C3+C4 ✅ (shared
survivability severity → capped king malus + `factor_exposure`; C2b gate rewired off its
placeholder to read exposure). All signed off 2026-08-06.

**Next (after commit + a performance-optimisation pass on the eval):** → re-watch guard
behaviour (C7/N1 may already have dissolved inertia N2/N3/N5) → design guard factors for any
residual gap, run the C7 part-3 weight sweep, consider C5/C6. N4 (standoff/breakthrough) is a
design-watch item, revisited only if C3+C4 don't produce breakthroughs (then a RULES
conversation, not an eval knob).

Each sign-off is a hard stop: the assistant shows the piece values (`search_bench --report
--eval custom …`), the designer decides keep / rebalance / redesign, and the sequence
advances only on approval. If a change adds noise rather than meaning, revert it. Commit
only when the designer asks. See `custom-eval-interaction-matrix.md` for the term
interactions (P0–P5) that drive this ordering.
