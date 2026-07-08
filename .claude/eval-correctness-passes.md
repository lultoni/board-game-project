# Evaluator correctness passes — playbook + track log

*Living doc, separate from `eval-perf-passes.md`. That file's discipline is "make eval faster." This file's discipline is "make eval **correct in scope** — a pure position rater, no lookahead, no simulation." Perf and correctness cross-reference each other but are worked as distinct tracks.*

*Started 2026-07-08 after Pass 3 (perf) landed. Rollback anchor for D+E tracks: `b5f4b7f Pass 3 audit: delete dead threat_bb, close Group A`.*

---

## Scope rule (the mandate)

**Eval is a pure function of the current position.** It returns a P1-POV rating for one state, using only static features of that state. It MUST NOT simulate future states — no capture resolution, no attack sequences, no "if I cast, then they respond." That's search's job.

This rule was violated by MAEE (Move-Attack Exchange Evaluation), which was a mini-search embedded in eval. Every problem MAEE caused (perf cost, double-count with QS, horizon-effect band-aids, tactical accuracy risk, ordering perturbation) traces back to the scope violation. The correctness track exists to unwind this and prevent recurrence.

### What eval MAY score (position-static)

- Material — piece counts × piece value.
- HP / armor — current remaining.
- Money — with realistic reference frame (see E3).
- Skill ownership — value of having skills equipped, gated on their actual usability (see E4).
- Placement quality — structural measures (king safety ring, guard adjacency, coverage).
- Mobility — count of reachable squares, subject to review (see E7).
- Tempo — whose turn, actions_remaining, phase (see E8).
- Exposure — count of enemies who *could* attack this piece, static (see E2).
- Bodyguard coverage — structural, ratio-based (see E6).

### What eval MUST NOT do

- Simulate exchanges (MAEE-style).
- Ask "will this piece die if attack plays out."
- Enumerate response sequences.
- Weight tactical bonuses based on future capture outcomes.

Search — including quiescence, SEE for ordering, and TT — is where all future-state reasoning lives.

---

## Why we're doing this

**Play-quality symptoms observed on `b5f4b7f`:**
- Champions Dash-zooming, wasting money for no visible tactical/positional gain.
- Focus activated on the last skill slot of a turn, wasting the buff.
- Skills cast with no material or positional effect (money burned).
- Guards piling to center, dying en masse instead of holding structure.

Diagnostic mapping:
- Dash-zooming, Focus-wasting → likely eval bug (over-crediting `skill_activity`'s "can cast now" signal). Search *knows* actions_remaining; eval hands it a value function that doesn't account for whether the cast is productive.
- Skills-with-no-gain → eval bug (`skill_activity` rewards ability-to-cast, not value-of-cast).
- Guards to center → eval bug (mobility term rewards center squares regardless of tactical structure).

None of this is fixed by making eval faster. The eval is *wrong in scope*, not slow.

**Perf-track ties:**
- Group A (MAEE internals) — retroactively wasted work. No active regret; the code paid for itself as learning. E1 comments the calls out; deletion of code follows once SEE-for-ordering proves out (planned as track D).
- Group E (QS redesign) — becomes moot after E1. If eval is cheap and pure, QS calling it every node isn't a bottleneck. The MAEE-in-QS double-count problem disappears.
- Group C (Zobrist eval cache) — lower priority; cheap eval doesn't need caching as urgently.
- Group D (SEE for move ordering) — **stays open, becomes the tactical safety net** for the removal of MAEE. Runs FIRST, before E1.
- Group F (TT under perturbed scores) — much less relevant with clean eval; probably closes.

---

## Track order (locked)

### Track D — SEE for move ordering (Group D of perf backlog)

**Type:** search-scope, not eval-scope. Runs as a *perf pass* (log in `eval-perf-passes.md`), not as an eval-correctness pass. Placed here in the sequence because it's a **prerequisite** for the eval-correctness track — it establishes the tactical safety net that lets us safely remove MAEE from eval.

Content: MVV-LVA + SEE scoring for captures in QS (and optionally at AB nodes at depth ≥ 3). Uses static exchange math to order captures so QS explores good ones first, hitting alpha-beta cutoffs faster. This is the classical chess-engine technique; the exchange-simulation math that MAEE was doing at leaf gets *moved to move ordering in search*, where it belongs.

**Follow the perf-pass playbook.** Pre-commit anchor at `b5f4b7f`, scoped plan, implement, `cargo check` between chunks, `cargo test -p core_engine` at end, eval-only microbench + search d6 bench with a new label (`post-pass4-see` or similar), and a Pass 4 entry appended to `eval-perf-passes.md`.

### Track E — eval-correctness (this file)

Runs AFTER track D lands. Each item is its own pass with commit anchor + bench + playtest.

- **E1:** Comment out MAEE calls in `evaluate_breakdown`. Keep MAEE code in place (functions, `AttackersTable`, `AttackerList`, `MAEE_MAX_PLIES`) — commented calls give us zero-cost rollback if SEE-for-ordering underperforms. Delete code later once E1 proves out in playtest.
- **E2:** Exposure term. Per own piece, credit-penalty scaled by number of enemies that could attack the square (using the attacker-bitboard from Pass 3's table). King gets its own escalation curve. Static, no simulation.
- **E3:** Money rework. Dim-then-cliff with cap = `max_owned_skill_cost × actions_per_round(round)`. Draft-phase pre-skill-ownership money is *correctly* worthless — that's not an edge case, it's the intended semantics per user (2026-07-08).
- **E4:** Skill ownership value gated on money availability. A Tempest owned but unaffordable is worth less than a Focus you can cast every round. Smooth (sigmoid) so search doesn't see score cliffs at cast-affordability thresholds.
- **E5:** Delete or heavily simplify `skill_activity`. Ownership contribution rolled into whatever term E4 establishes. `skill_activity` in its current shape ("has money + range") is a scope violation — it's asking "can I cast this move" which is search's job.
- **E6:** Bodyguard coverage term. Structural, ratio-based (see below).
- **E7:** Mobility term review. Decide whether "count of reachable squares" is right, or whether structural placement terms (Champion skill-range coverage, Guard adjacency penalty, King safety ring) should replace it.
- **E8:** Tempo / action-economy term. actions_remaining, phase, whose-turn.

---

## Locked design decisions

### E3 — money rework

**Cap:** `max_owned_skill_cost × actions_per_round(round_number)`.

- `max_owned_skill_cost = max(skill_cost(s) for s in equipped_skills(this_side))`. All skills, not just Strike. Support skills cost money too.
- `actions_per_round` reads from the game's action-progression function (find during implementation — probably `Position` or the round-config module).
- Cap grows across the game as actions/round scales up.

**Shape:** diminishing returns up to cap, hard cutoff past cap.

Candidate form:
```
w(m) = max(0, 1 - m/cap)                    // linear-down weight
useful_money = MONEY_PER_UNIT × ∫₀^money w(m) dm
             = MONEY_PER_UNIT × money × (1 - money/(2·cap))    for money ≤ cap
             = MONEY_PER_UNIT × cap / 2                        for money > cap
```

**Edge case (per user, 2026-07-08):** `max_owned_skill_cost = 0` → cap = 0 → money worthless. This is *correct* — pre-draft (before any skill draft), money has no use. Do not add a floor.

### E6 — bodyguard coverage

**Applies to:** Champions and King (Guards are the shielders, not the shielded).

**Formula:**
```
coverage(piece) = n_bodyguarded_squares / n_threat_vectors
```

Where:
- `n_bodyguarded_squares` = cheby-1 squares of `piece` that are ALSO cheby-1 to an own Guard.
- `n_threat_vectors` = cheby-1 squares of `piece` that are *empty* (attackers approach via empty squares).

Denominator uses empty squares — a square occupied by own piece isn't a threat vector; a square occupied by an enemy is already engaged (attack from that square is directly counted by E2 exposure, not by this term).

**Denominator = 0 edge case** (piece fully surrounded): `coverage = 1` (fully protected — no threat vectors to protect from). Multiplied by piece value for the bonus; contributes full credit as if fully bodyguarded.

**Why not "count squares adjacent to piece that are adjacent-to-guard AND actually-attackable" (option b from discussion):** that couples coverage to threat state, which is a form of forward-looking reasoning ("if attacker approaches from square X, is X shielded?"). Coverage is a *pure structural* measure — how well-placed is this piece relative to its bodyguards, independent of current threat. Search combines it with exposure (E2) to make tactical decisions. Keeping the two signals separate keeps eval scope clean.

**If playtest shows over-clustering** (Guards packed around Champs even under zero pressure, wasting Guard mobility), that's a signal E6 is over-credited or coupled to E7 (mobility). Revisit at playtest time.

### E2 — exposure term

**Applies to:** all own non-king pieces (Champions, Guards), and King (separate curve).

**Formula:**
```
exposure_penalty(piece) = piece_value(sq) × f(unshielded_attackers(sq))
unshielded_attackers(sq) = max(0, n_enemy_attackers(sq) - n_adjacent_own_guards(sq))
```

- `n_enemy_attackers(sq)` = `popcount(attackers_table[opp][sq])`. Uses the Pass-3 attacker bitboard.
- `n_adjacent_own_guards(sq)` = `popcount(king_expand(sq) & own_guards)`. Each adjacent Guard can intercept one attack (bodyguard rule).
- `f(n)`: 0 → 0; 1 → small; 2+ → sharp (attackable-when-one-defender-dies is mate-adjacent).

**King special-casing:**
- `n_attackers > 0` on king square → large penalty.
- `n_attackers ≥ 2` → mate-adjacent flag, very large penalty.

**Where the "shield" credit could double-count E6:** E6 credits structural coverage (ratio of shielded threat vectors); E2 credits reduction of attacker count by nearby guards. Both use adjacent-own-guards. This is intentional — E2 reflects immediate defense (attacker offset by defender), E6 reflects structural resilience (position holds up even if one defender is neutralised). If playtest shows they compound incorrectly (double-credit on well-placed pieces), tune weights or merge into one term.

### E4 — skill ownership gated on money

**Formula (candidate):**
```
availability(s, side) = sigmoid((money - skill_cost(s)) / K)
skill_value(s, side) = skill_base_value(s) × availability(s, side)
```

`K` = smoothing constant, tune during implementation (probably 2–4 money units).

Sigmoid keeps the transition smooth so search doesn't see score cliffs when money crosses cast threshold. A Champion 1 money short of Tempest still gets partial credit for owning it; a Champion 5 money short gets almost none.

### E5 — skill_activity

**Likely disposition:** delete. Its current job ("has money, has range to cast") is a scope violation — it's asking "is a cast productive right now," which is a search question. Replace its contribution with:
- E4 (skill value gated on money) covers "having a usable skill is worth money."
- Search decides whether to actually cast, given ordering & alpha-beta.

If deletion causes a play-quality regression (AI stops equipping high-value skills because they don't score without the activity credit), revisit.

### D — SEE for move ordering (search side)

Not detailed here — logs into `eval-perf-passes.md` per the perf-pass playbook. Key points for E-track:

- Runs BEFORE E1 to establish tactical safety net.
- Adds MVV-LVA + SEE scoring to move ordering in QS (and optionally AB depth ≥ 3).
- Exchange-simulation math from MAEE moves here.
- After D lands, tactical resolution at low depth is preserved via ordered QS captures + alpha-beta cutoffs.

---

## Playbook for E-track passes

Different from perf-track playbook. Each E-item is:

1. **Rollback anchor.** Confirm clean tree + note the commit hash before starting.
2. **Scoped plan.** Which sub-items in, which deferred, why. Present for approval before coding.
3. **Implementation.** `cargo check -p core_engine` between chunks; `cargo test -p core_engine` at end.
4. **Bench: search d6 + instrumented.** Same corpus as perf-track for cross-comparison. New labels per-item (`search-post-E1.json`, etc.).
5. **Playtest: mandatory.** Load current build in Tauri dev (`cd game/crates/tauri_wrapper && cargo tauri dev`). Play at least 1 game against the AI. Note any behavioral changes (positive OR negative) — with FENs where possible for regressions.
6. **Endgame FEN test** at `.claude/eval-perf-passes.md` playbook — re-run each E-item.
7. **Log:** append entry to this file with date, scope, deferred, bench results, playtest observations, known regressions, follow-ups.

### Stop-and-ask triggers (E-specific)

- Playtest reveals worse behavior than baseline → stop, discuss.
- Endgame FEN test fails → stop.
- Total corpus wall time regresses by >20% (rare — eval is *cheaper* now — but if it happens, something's wrong).
- Node counts explode by >100% on multiple positions → likely eval is now under-informed and search is compensating badly.
- Any test failure that isn't a trivial fix.

### What we're NOT doing

- **Not bundling.** One E-item per pass. Same discipline as perf-track.
- **Not skipping playtest.** Correctness track ≠ perf track. Bench alone doesn't tell us whether AI plays *better*.
- **Not deleting MAEE code in E1.** Only commenting calls. Preserves rollback. Delete after track completion.

---

## Open questions to resolve before starting

### Play-baseline recording

Should we play 1-2 games against `b5f4b7f` AI and note down FENs where the bad behavior triggers, BEFORE starting D? Value: reproducible before/after comparison at track completion. Cost: ~20 min.

**User's call. Not blocking.**

### Denominator of E6 uses empty squares only

Should it also include *own-piece-occupied* squares (i.e. "protected by own piece even though enemy can't approach there because we already stand there")? Argument for: an own piece adjacent to the Champ is also a form of shielding (attacker has to go through them). Argument against: that's what E7 mobility already partially captures, and expanding the denominator lowers the ratio artificially. Keep as-is (empty only) unless playtest shows a specific case where this misvalues a position.

### Champion vs Guard exposure weighting

E2's `piece_value(sq)` uses standard piece values. Champions are worth more than Guards. Is that enough differentiation, or does Guard exposure need its own curve (e.g. Guards are expected to be more exposed as bodyguards, so shouldn't be penalised as heavily for standing in threat)? Probably fine as-is; revisit if playtest shows AI hoarding Guards defensively.

---

## Log

### 2026-07-08 — Track opened

Design pivoted from Group E perf pass to eval-correctness track after user identified play-quality symptoms (dash-zooming, wasted skills, guards to center) as *eval scope* problems, not perf problems. Full discussion captured in the "Why we're doing this" section above.

**Anchor:** `b5f4b7f`.
**Next:** Track D (SEE for move ordering) as perf pass — logs into `eval-perf-passes.md`.
