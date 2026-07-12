# Alpha-Beta Search Optimisation Techniques — Catalogue

*Session 35 (2026-06-26). Companion to `search-speed-benchmark-plan.md`.*
*Source: web-search synthesis across chessprogramming.org wiki, Stockfish, and amateur engine writeups (CCRL-rated engines: Weiss, Ethereal, Smallbrain).*
*Status: reference material. Each technique is a candidate for the benchmark's optimisation queue.*

---

## Engine context (recap)

- Rust, native + WASM, straight alpha-beta (not negamax), iterative deepening, Zobrist TT with BoundFlag.
- Time-check via node-count mask every 1024 nodes.
- Static eval is sub-ms (material + HP + armor + skills + money).
- Game mechanics that interact with chess-derived techniques:
  - Multi-phase turns (Move → Skill → end-of-round). Each ply is one atomic action.
  - `EndPhase` is a legal action, not the null move (it advances state).
  - No "capture" primitive; HP-reducing skills with money costs are the analog of forcing moves.
  - Score absolute P1-POV, ±MATE_SCORE with mate-distance encoding.

**Definitions used throughout:**
- **Loud** = HP-reducing skill cast or King-threatening positioning.
- **Quiet** = repositioning without HP impact, `EndPhase`, cost-only skills with no board impact.
- **Forcing** = move that materially reduces opponent's legal-response set (the "check" analog).

---

## 1. Move Ordering

Highest-leverage area. Ideal ordering shrinks tree toward `sqrt(N)`. Modern engines achieve fail-high-on-first-move ~90% of the time at cut-nodes.

| Technique | What it does | Speedup | Complexity | Correctness | Fits our game? |
|---|---|---|---|---|---|
| **TT-move first** | Try the TT entry's stored move first. | 2-5× at depth-N from depth-(N-1) data. Biggest single gain. | Low | Yes (ordering only) | Yes — but legality-check the TT move against current phase before trusting it. |
| **PV-move** | Order previous iteration's PV move first. | Subsumed by TT-move when TT is reliable. | Low | Yes | Yes; often redundant with TT-move. |
| **MVV-LVA** | Captures ordered by victim-value / attacker-value. | Significant in chess. | Low | Heuristic | **Adapt.** No captures. Substitute: HP-reducing skills ordered by `(damage_dealt × target_value) / cost`. King-threatening skills get top priority. |
| **SEE** | Simulates recapture cascade on a square to label captures +/-/=. | Major in QS and cut-nodes. | Medium | Heuristic-ish | **Limited fit.** No attacker-stack on squares; ranged skills with shared money pool break the model. See §3 for our adaptation. |
| **Killer moves** | Two quiet moves per ply that caused beta-cutoffs at siblings. | 10-30% node reduction. | Low | Heuristic | Yes — keep separate killer slots per phase (Move vs Skill), since move sets are disjoint. |
| **History heuristic** | `[side][from][to] += depth²` on cutoffs; order quiet moves by score. | 10-20% on top of killers. | Low | Heuristic | Yes — index by `[side][action_kind][from][to]` where `action_kind` covers move/skill_id/EndPhase. Let `EndPhase` accrue history too. |
| **Butterfly tables** | Like history but normalised: `cutoff_count / try_count`. | Small but consistent. | Low | Heuristic | Yes. |
| **Countermove heuristic** | `counter[prev_move] = best_reply`. | 5-10% on top of killers+history. | Low | Heuristic | Yes — especially useful across phase boundaries. |
| **Continuation history** | n-ply history (Stockfish uses 1/2/3/4/6-ply). | Cumulative ~50-100 Elo on Stockfish. | Medium | Heuristic | Yes — 1-ply and 2-ply are the sweet spot for an amateur engine. Caveat: "n plies ago" crosses phase boundaries cleanly since plies are atomic. |
| **Correction history** | `[side][slow-changing-hash] → eval_correction`. Stockfish 2023+. | ~10-25 Elo on FishTest. | Medium | Heuristic (affects RFP/razoring/NMP margins via eval) | Maybe. Replace chess's pawn-hash index with a hash of our slow-changing feature — likely the **equipment / skill loadout state**, since pieces and skills are sticky across many plies. |

**Pitfalls.**
- History/killer tables must be cleared (or aged) between independent searches.
- Don't trust TT-move without legality-check in current phase.
- **Phase must be in the Zobrist key.** Same board state, different phase = different position. Already flagged in TT section §6.

---

## 2. Pruning

| Technique | What it does | Gain | Complexity | Correctness | Fits? |
|---|---|---|---|---|---|
| **Null-move pruning (NMP)** | Hypothetically pass; depth−R reduced search; if β-cutoff, prune. R typically 2-4. | 2-3×. Biggest pruning gain in chess. | Low-medium | Heuristic; fails in zugzwang | **See special caveats below.** |
| **Reverse futility pruning (RFP) / Static NMP** | If `static_eval − margin×depth ≥ β` at low depth and non-PV, return `static_eval`. | 30-80 Elo in amateur engines. | Low | Heuristic | Yes. Disable when King is directly threatened. |
| **Futility pruning** | At depth 1-2, skip quiet moves where `static_eval + max_gain + margin < α`. | 20-40 Elo. | Low | Heuristic | Yes. Tune `max_gain` for our damage range (much more compressible than chess piece values). |
| **Late Move Reductions (LMR)** | Reduce depth for moves past the first few. `R = base + ln(depth)·ln(move_idx) / divisor`. Re-search at full depth on fail-high. | "Effective branching factor < 2." 100+ Elo in modern engines. | Medium | Heuristic with re-search safety net | **One of our highest-leverage adds** given Skill-Phase branching. Don't reduce: PV node, in-check, King-threatening, TT-move, killers. |
| **Late Move Pruning (LMP) / Move-count pruning** | At low depth and non-PV, skip moves with index above `threshold(depth)` entirely. | 20-50 Elo. | Low | Heuristic (more aggressive than LMR) | Yes — particularly valuable for `EndPhase`-padded tails of skill-phase move lists. |
| **Razoring** | If `static_eval + margin < α` at depth ≤ 3, drop to QS; if still fail-low, return that score. | 10-20 Elo, marginal in modern engines. | Low | Heuristic | Maybe. Only valuable if QS is well-defined first. |
| **ProbCut** | Statistical NMP: at depth d, do depth d' < d with raised β. Requires regression-tuned `(a, b, σ)`. | Real Elo in Stockfish; more in Othello. | High (offline regression) | Heuristic | Premature. Revisit after engine is stable. |
| **Multi-cut** | At cut-nodes, search first M moves at reduced depth; if C cause β-cutoff, prune. | Modest, superseded by SE+LMR. | Medium | Heuristic | Skip. |

### RFP / Static NMP — Session 36 result, rejected (standalone)

Implemented as a post-TT-probe early return: at low depth, if `static_eval ± margin*depth` is past the cutoff bound, return `static_eval` directly. Gated `ply > 0` and `!is_mate(α) && !is_mate(β)`.

Margin sweep at fixed depth 6:
1. `margin=120, depth∈[1,3]` — Δnodes -0.1%, Δtime **+6.4%**. **1 score regression** (endgame-sparse-05: -3950 → -3775) with best-move disagreement. Static eval and 6-ply minimax disagree by ~175cp on that position; that margin doesn't gate it out.
2. `margin=200, depth∈[1,3]` — Δnodes -0.0% (zero pruning), Δtime **+6.1%**. No regressions but strictly worse than baseline: pure overhead from per-node `evaluate()` with no offsetting cutoffs.
3. `margin=150, depth∈[1,2]` — Δnodes -0.1%, Δtime +6.2%. Same regression as config 1.

**Root cause.** The +6% time overhead is from calling `evaluate(pos)` at every internal node at depths 1-2 (or 1-3). After killers+history, those nodes are already getting cut on the TT-move or killer-1, so RFP almost never finds a position to prune that wasn't already pruned. When the margin is loose enough to fire (≤150), it occasionally returns a stale static_eval where 3-ply minimax disagrees → correctness regression. When the margin is tight enough to be safe (≥200), nothing prunes. There is no useful margin band.

**Mechanistic difference from LMP, but same outcome.** RFP doesn't poison TT/history at deeper iterations the way LMP does (it returns `static_eval` rather than changing which leaves get visited). But its dependence on static-eval accuracy at "decision-relevant boundary" positions means it inherits a related problem from a different angle: static eval is noisy at depth-1-2 nodes in the mid-skill-exchange regime that QS would clean up.

Methodology note: see the methodological note at the bottom of the LMP entry below — baselines are deterministic, regressions are real algorithmic deltas.

**Revisit conditions:**
- After **QS** lands. QS would make the static eval at depth-1-2 nodes reliable (eval'd at quiet leaves, not mid-skill-exchange) — the endgame-sparse-05 regression specifically would likely not recur because static eval there would be post-quiesce.
- After **incremental eval** lands (if pursued). The +6% time cost would drop substantially if `evaluate()` were O(1) updates on make/unmake rather than O(N) recomputation.
- Could be a **fixed-depth-only** training-pipeline optimisation if we find a margin that's both safe at depth 6 AND useful at depth-12+ (the regression at depth 6 doesn't tell us about deeper iterations — it's plausible the static-eval gap closes at depth 10+).

---

### LMP — Session 36 result, rejected (standalone)

Implemented LMP with several threshold configurations. **All standalone configurations regressed at least one position at one time budget. Rejected per strict no-regression protocol.**

Configurations tried (all gate `lmp_active = lmp_threshold > 0 && !is_mate(α) && !is_mate(β)`, never prunes index 0):
1. `{1→8, 2→12, 3→18}` — depth-6 -43% wall-clock, -55% nodes, BUT **3 score regressions at fixed depth** (midgame-low-skill-01/03/05 each lost ~600 score units). Far too aggressive.
2. `{1→12, 2→24}` — depth-6 -35%, 1 score regression (midgame-low-skill-03: 825 → 75). Still unsafe.
3. `{1→16, 2→32}` — depth-6 -34%, 1 score regression remained (midgame-low-skill-03: 825 → 225).
4. `{1→16}` only (depth-1 LMP, depth ≥ 2 untouched) — **depth-6 clean: -33% wall-clock, -45% nodes, zero correctness regressions, zero best-move disagreements.** Time-budget sweep: 16W:5L across 100/500/1000/3000ms.
5. `{1→16}` + skip-history-record-at-depth-1 — depth-6 still clean (-32%, no regressions). Time-mode improved at 1000ms (opening-02 was 9 now 10 vs baseline 11) but 500ms regression unchanged.

**Root-cause diagnosis (opening-02, the worst regressor):**
- Baseline 500ms: depth **10**, 2.2M nodes, EBF 4.31.
- LMP v5 500ms: depth **8**, 85K nodes, EBF 4.13.
- LMP v5 *fixed-depth-9*: 4.46M nodes, 830ms, **EBF 5.48** — i.e. LMP makes the effective branching factor *worse* on this position at the depth that would have fit baseline's budget.
- Mechanism: LMP's depth-1 pruning changes the leaf-set, which changes TT contents at deeper iterations, which changes move ordering at non-leaf depths. On positions where one move is dominant and standard AB already cuts on index-0, LMP's leaf-pruning has nothing positive to contribute but its TT/history side-effects poison deeper iterations.

**Methodological note (carries over to every entry in this catalogue):**
- Baselines are **deterministic** across 3-run and 10-run sweeps at every time budget (verified Session 36, 0/20 positions disagreed across budgets). Time-mode "regressions" are real algorithmic deltas, not measurement noise. The "median-by-nodes" reduction in the bench binary is stable because every position reaches the same depth across repeated runs.
- Therefore: per-position regressions at any time budget must be explained, not waved away as noise.

**Revisit conditions:**
- After **better move ordering** matures (e.g., MVV/LVA-style scoring for skill captures, SEE-equivalent). LMP's regressions come from poor deeper-iteration ordering; better ordering may close the gap.
- After **QS** lands. QS limits the volatility at depth 0, which reduces the cost of LMP's leaf-pruning getting "the wrong answer".
- Maybe valuable as a **fixed-depth-only optimization** for the training pipeline (gate by `time_limit_ms == 0`). -33% on depth-6 with zero regressions is a real win for the NN-rater training loop.

---

### NMP — critical adaptation for our `EndPhase` action

The conceptual basis of NMP is "if I do nothing, my position is still ≥ β." In our game, `EndPhase` is a legal move — it transitions Move → Skill → end-of-round, **advancing state**. So:

- **The real "null move" is NOT `EndPhase`.** It's "give opponent two consecutive turns without changing any state" — flip side-to-move + advance ply hash, no state change.
- **Implement NMP and `EndPhase` as distinct operations.** Searching `EndPhase` at depth−R does not give NMP-equivalent information.
- **Zugzwang analog.** If we ever add "you take damage at end of your turn" effects (poison, bleed, upkeep), disable NMP near end-of-round. Stack M has no such effects today but `bp-plague` (Plague skill backpocket) would introduce it.
- **NMP-refutation TT-move trick still works.** On NMP fail-low, the depth−R−1 search returns a move from opponent's perspective — extract from TT to seed move ordering.

---

## 3. Quiescence Search and Variants

**The big adaptation.** Chess QS exists because captures+promotions+checks are roughly the full set of forcing moves with bounded material. Our analog:

- **Loud actions to search in QS:** HP-reducing skills; King-threat moves; skills that change armor (mediates HP loss).
- **Quiet actions to NOT search in QS:** Repositioning without HP impact; `EndPhase`; cost-only skills.
- **"Check" analog:** King is in any opponent skill's threat zone next ply. When true, QS searches ALL legal moves (no stand-pat), like check-evasions. **This is the most load-bearing extension to QS.**

### Stand-pat
- Compute static eval at QS entry.
- If `eval ≥ β`, return β (fail-high).
- Else raise `α = max(α, eval)`.
- **Disable stand-pat when King is threatened** — otherwise side-to-move can "stand still" while King gets captured next ply.

### Delta pruning adaptation
- Chess: skip captures where `eval + victim_value + margin < α`.
- Ours: skip HP-skills where `eval + max_HP_swing_this_skill + margin < α`.
- `max_HP_swing` = `damage_after_armor × value_of_most_valuable_piece_in_range`. Cheap precomputed table.

### SEE adaptation — the hard part

Chess SEE assumes square X is repeatedly attacked by a stack of pieces in ascending value order. Ranged skills break this: no positional attacker-stack, money is global. Two viable adaptations:

1. **Skill-Exchange Evaluation (single-step, no recursion).** Compute net HP swing for one skill cast against a target, including immediate counter-cast if target survives and has a damaging skill within range/cost. Truncate at depth 2 (cast, counter). Captures ~80% of what SEE buys without trying to recurse.
2. **Cost-aware filter.** Skill costs M, deals D damage. "Winning" iff `D × target_value − M × own_money_value > 0`. Cheapest filter, sufficient for QS pruning.

**Caveat.** Don't port chess SEE pseudocode literally. The mismatch will produce subtle bugs (treating money like an attacker in the stack).

### Generating checking moves in QS
Most strong engines do this for the first 1-2 plies of QS. Our analog: generate King-threatening positioning moves in QS even if they don't reduce HP immediately. Cap at QS depth 2 to avoid explosion.

---

## 4. Extensions

| Extension | When | Gain | Notes |
|---|---|---|---|
| **Check extension** | Move puts opponent King in immediate threat. +1 ply. | 20-40 Elo in chess. | Our "check" = King in opponent's next-ply skill threat. Conservative trigger — broad threat definitions cause explosion. |
| **Singular extension** | TT-move score exceeds all alternatives at reduced depth by a margin. +1 ply. | 30-80 Elo; biggest "modern" extension. | Requires an extra reduced search at SE-trigger node. Worth it once basics are stable. |
| **Recapture extension** | Move recaptures piece just lost. +1 ply. | Marginal (~10 Elo). | **Adapt or skip.** Our analog: "skill cast in immediate response to opponent's last HP-skill, same target/attacker." Often subsumed by good QS. |
| **Threat extension** | Move was good only because it parries a null-move-detected threat. | Mostly historical; modern engines fold this into LMR-not-reducing. | Skip. |

---

## 5. Aspiration Windows and PVS

### PVS vs straight alpha-beta
Drop-in replacement: first move with `[α, β]`, rest with null window `[α, α+1]`. On null-window > α, re-search with `[score, β]`. Gain: 5-15% in chess given good move ordering. **Worth switching** — composes cleanly with LMR (the LMR re-search is itself usually null-window). Almost all modern engines are PVS, not straight AB.

**Session 36 result — rejected (standalone).** Implemented PVS with absolute-POV framing (P1 null-window `[α, α+1]`, P2 null-window `[β-1, β]`), gated by `depth >= 2`. Outcome at depth-6: +0.8% wall-clock (essentially a no-op — killers+history orders so well that PVS finds nothing left to save). Time-mode budgets: 500/1000/3000ms each gained 1-4 positions, but 100ms lost 2 positions (opening-02 8→7, midgame-low-skill-06 9→8) and 3000ms lost 1 (endgame-sparse-06 18→17). Regressions are deterministic (see LMP entry's methodological note). Hypothesis: PVS's win materialises when null-window re-searches are *cheap* — that requires LMR to be reducing the re-search depth. Standalone PVS atop strong move ordering has almost nothing to save. **Revisit alongside LMR** — PVS is the canonical re-search target for LMR, so they should be implemented together and graded as a pair.

### Aspiration windows
After iteration N, set `α = score_N − δ`, `β = score_N + δ` for iteration N+1, with δ ~ 25 cp. Widen exponentially on fail. Gain: 10-20% time-to-depth. Low complexity. Don't aspirate until depth ≥ 5. Disable near mate scores.

**Session 36 result — rejected (standalone).** Tried at δ=50/150/300 with 4× exponential widening, MAX_WIDENINGS=4, MIN_DEPTH=5. Outcome: depth-6 wall-clock improved ~25%, BUT 1-3 corpus positions regressed by 1 ply at the extreme time budgets (100ms and 3000ms) regardless of δ. δ=300 also regressed depth-6 wall-clock (+3.8%). No δ achieves strict no-regression across the multi-budget sweep. Regressions are deterministic, not noise (baseline 3-run vs 10-run sweeps agreed on every position at every budget — see LMP entry above for the methodological note). Hypothesis: at very tight budgets the re-search after a fail-high/fail-low burns enough time to fall short of the next depth. **Revisit after LMR/PVS** — those make the re-search itself cheaper and may eliminate the time-mode regressions. Don't retry the same parameter range alone.

---

## 6. TT Refinements

Already have Zobrist + BoundFlag. Recommended layers:

| Refinement | What | Complexity | Notes |
|---|---|---|---|
| **TT-move ordering** | Already implied (§1). | Low | — |
| **Two-tier (depth-preferred + always-replace)** | Each slot holds two entries: one keeps deepest, one keeps freshest. | Medium | Classic Thompson/Condon scheme; robust default. |
| **Bucketed (4-way set-associative)** | Each cache line holds 4 entries; replace lowest-depth on store. | Medium | Modern preferred. Matches L1 cache line (64B) for 4 × 16B entries. |
| **Aging** | Tag entries with a "search generation" counter; prefer replacing older. | Low | Essential — stale entries from prior root searches outlive their usefulness. |
| **Mate-score adjustment on store/probe** | Store as "distance from this node"; on probe, add current ply back. | Low | Skip this and mate-distance reporting breaks. We already do this — verify. |

### TT pitfalls specific to our game
- **Phase must be in Zobrist.** Same board + different phase = different position.
- **Money and HP must be in Zobrist.** Collisions otherwise produce illegal TT-move suggestions.
- Equipment is in Zobrist if mailbox state is hashed (verify).

---

## 7. Iterative Deepening Refinements

| Refinement | What | Notes |
|---|---|---|
| **Internal Iterative Deepening (IID)** | When no TT-move at PV node, search at depth−2 first to get a TT-move, then proceed at full depth. | Low gain in modern engines (TT hit rate is high); cheap to add. |
| **Internal Iterative Reductions (IIR)** | Inverse: when no TT-move at non-PV cut-nodes, reduce depth by 1. | Stockfish 2020+. ~10-20 Elo. Lower complexity than IID; replaces it in some engines. |
| **ID + aspiration** | Covered §5. | — |
| **Time-management coupling** | Per-iteration time check; abort iteration if next is unlikely to finish in budget. | Our node-mask check is fine; add "iteration time grew Nx" predictor for early termination. |

---

## 8. Parallelism

Native + WASM constraint: WASM threads need `SharedArrayBuffer` with COOP/COEP headers. Native is straightforward.

| Algorithm | Approach | Scaling | Complexity | Verdict |
|---|---|---|---|---|
| **Lazy SMP** | N threads search same root with shared TT; nondeterministic ordering causes diverging paths; TT shares work. | Reasonable to 8+ cores. | Low (needs concurrent TT) | **Recommended.** Stockfish switched from YBWC → Lazy SMP in 2016. |
| **YBWC** | Sequential first move, parallel siblings. | Better time-to-depth than Lazy SMP. | High (split points, helpers, abort propagation) | Skip — complexity not worth it for amateur engine. |
| **ABDADA** | Mark TT entries "being searched" to avoid duplicate work. | Modest. | Medium | Skip — Lazy SMP dominates. |

### WASM caveat
TT in `SharedArrayBuffer`. Lockless reads via Hyatt xor-trick: store `key XOR data` so a torn read produces an invalid key. Native can use atomics or unsynchronised reads (torn reads are rare and self-correct).

**For our trainer:** training runs native-only, so Lazy SMP is straightforward. WASM threading is a separate plumbing job for the in-browser AI play feature — can defer.

---

## 9. Modern / Less-Known

- **Continuation history (§1)** — Stockfish family; highest-Elo addition past basics.
- **Correction history (Stockfish 2023+)** — small correction added to static eval. Reduces noise for RFP/razoring/NMP margins. Index on our equipment/loadout hash.
- **Static-eval-derived LMR adjustments** — reduce more when `static_eval < α`; reduce less when improving.
- **History pruning at low depth** — prune (not just reduce) moves with sufficiently negative history at depth ≤ 2-3.
- **Conthist-based move ordering** — order quiet moves by `history + 1-ply-conthist + 2-ply-conthist`. Stockfish standard.
- **Singular Extension with double-extend** — large SE singular margin → extend by 2 plies. Stockfish standard since ~2021.
- **Leela hybrid AB attempts** — Lc0 experiments with AB sub-searches inside MCTS rollouts. Not relevant unless we go neural; ignore for now.

---

## Techniques That Don't Port Cleanly — Flag List

| Technique | Why it doesn't port |
|---|---|
| **MVV-LVA** | No "captures." Substitute damage-weighted skill ordering. |
| **SEE recursion** | No attacker-stack on a square; shared money pool. Substitute single-step Skill-Exchange Evaluation. |
| **Recapture extension** | No recaptures. Substitute "immediate damaging counter-skill" or skip. |
| **QS on captures only** | Chess heuristic "only captures in QS" maps to "only HP-reducing skills" — but must also include King-threat moves and (briefly) checking moves. |
| **Pawn-hash for correction history** | No pawns. Use equipment/loadout hash. |
| **Razoring depth-3-to-QS** | Only meaningful if QS is well-defined. Get QS solid first. |
| **Verification null-move** | Empirically inconclusive in chess (Hyatt). Not worth complexity in a new engine. |

---

## Recommended Implementation Order

Ordered by Elo-per-engineering-hour for our specific engine. Reasoning follows CCRL-rated amateur engines (Weiss, Ethereal early commits, Smallbrain).

1. **PVS** (1-2 h). Almost free conversion from straight AB. Composes with everything. Every modern engine is PVS.
2. **TT-move ordering hook into move generation** (if not already wired). Biggest single move-ordering gain.
3. **Aspiration windows** with exponential widening. Cheap, 10-20% time-to-depth.
4. **Killer moves + history heuristic** (2-3 h). Foundational quiet-move ordering. LMR is much weaker without these.
5. **LMR** (4-8 h, including formula tuning). Often biggest single Elo jump. Pair with PVS — shared null-window re-search structure. **Critical for our high-branching Skill phase.**
6. **Reverse Futility Pruning** (1 h). Cheap, low-risk, 30-80 Elo.
7. **Adapt Quiescence Search** to our loud/quiet distinction. Stand-pat + delta pruning + King-threat extension. Use single-step Skill-Exchange Evaluation, not chess-SEE port.
8. **Null-move pruning** with zugzwang awareness. Disable if any "damage on end-of-round" mechanic is added. 2-3× speedup when working.
9. **Check extension** (King-threat analog). Conservative trigger. ~20-40 Elo.
10. **Futility pruning + LMP** at low depth. Cheap on top of RFP.
11. **Countermove + 1-ply continuation history.** Diminishing returns but cumulative.
12. **Singular extensions.** Medium complexity, real gain. Only after 1-11 are stable and TT-move is reliable.
13. **Lazy SMP** if going multi-threaded. Native first; WASM threading is separate plumbing.
14. **Correction history, 2+ ply continuation history, ProbCut.** Stockfish-class polish. Revisit only after engine is competitive.

Items 1-8 will produce a strong tactical engine on their own. Items 12+ are the dividing line between "decent amateur engine" and "FishTest-class engine."

---

## How to use this catalogue

- This is the optimisation queue for `search-speed-benchmark-plan.md`. Each technique is a candidate benchmark run.
- The order above is a default, not a hard sequence. The benchmark's data may suggest reordering — e.g. if move-ordering is already excellent, LMR's payoff drops; if QS is poorly defined, razoring is pointless.
- Each landing commit should include: benchmark before, benchmark after, technique name, baseline update.

---

## Session 37 retrospective — the evaluator is the bottleneck, not the search

Implemented full QS (catalogue §3 spec, minus SEE) over the session. After it landed: re-attempted RFP atop QS (both `evaluate`- and `quiesce`-eval variants), re-attempted aspiration windows atop QS. All three rejected at the multi-budget sweep.

**Then ran a head-to-head play match** (QS engine vs no-QS engine, 1000 ms/move, 3 games, swapping sides). Outcome: 1 win for QS, 0 for baseline, 2 ply-cap stalemates at 600 plies. The win-rate was secondary — the diagnostic was the **action breakdown** of each game:

| Game | Move actions | Skill actions | EndPhase actions |
|---|---|---|---|
| 1 (decisive) — 194 plies | 74 | **0** | 120 |
| 2 (cap) — 600 plies      | 181 | **0** | 419 |

Then `aivai_demo --max-plies 80`: **80 consecutive `EndPhase` plies, no piece ever moves.** The evaluator score drifts 0→150 across those 80 plies — purely from asymmetric round-by-round money accumulation. The search is alive (30 k–67 k nodes per ply, depth 6), but every legal Move-action is eval-neutral so the engine prefers the cheap `EndPhase` cutoff.

**Root cause.** `evaluate()` is purely material + HP + armor + skills + money. There is no positional term — no piece-square table, no king-pressure / proximity-to-enemy-king, no centre-control. A piece on `c2` and the same piece on `f7` evaluate identically, so every Move action has Δeval = 0 and the search has no gradient to climb. The only Move actions that have non-zero Δeval are Move-Attacks already in range — which is why QS appears to find tactics at leaves: those are the only moves the eval rewards.

**Implications for every catalogue entry tried so far (Session 36–37):**
- RFP / aspiration / PVS / LMP rejections at the depth-reached protocol are **uninterpretable** as algorithmic signals. The corpus consists of self-play positions generated by an engine that doesn't move pieces — many corpus positions are effectively "money-accumulating starting positions" where any search optimisation amounts to fine-tuning which `EndPhase` line the engine prefers.
- QS's "horizon-effect" wins at depth 6 (`midgame-low-skill-01: -750 → -300` etc.) are real but conditional: they only fire when a tactical position exists at the corpus seed. In actual self-play, neither side reaches that range without piece movement.
- **The multi-budget no-regression protocol is not invalid — it just doesn't grade what matters.** Depth-reached compares engines that play the same eval-neutral wandering deeper. Play-strength is what we care about.

**Action items for next time around:**
1. **Add positional terms to `evaluate()` before retrying any search optimisation.** Minimum viable:
   - King-attack-distance: bonus for own piece Chebyshev-close to enemy king (mirrors what `quiescence::is_king_threatened` already computes).
   - Piece-square table or centre-control: distinguish `c2` from `f7`.
   - Optional: tempo bonus to break ties toward forward motion.
2. **Re-grade QS, RFP, aspiration, PVS, LMP atop the new eval.** All rejections to date are provisional — the regressions may have been eval-noise artefacts.
3. **Replace the corpus** once the eval moves pieces. Current corpus (random self-play) inherits the no-movement pathology.
4. **The head-to-head match harness (`examples/qs_match.rs`) is the new accept/reject test**, not depth-reached. Pit candidate-engine vs baseline-engine, 3+ games, swap sides, count decisive wins. Once eval supports real play, this scales naturally.

**What stays banked from Session 36–37:**
- QS module itself is correct (380/380 tests, including mate-distance invariants and unmake-symmetry). Keep it hooked; it can only help once the eval moves pieces.
- `is_king_threatened` bitboard fast path (Session 37 v2): 79% wall-clock vs 151% for the generator-based v1. Real engineering win, independent of the eval issue.
- `DISABLE_QS` runtime flag in `alpha_beta.rs` and the match harness in `examples/qs_match.rs`. Both are general-purpose A/B tooling — any future "X vs no-X" grading reuses them.
- Catalogue rejections of RFP/aspiration/PVS/LMP remain as Session 36 entries, but **with the caveat that they were graded against a non-playing engine.** Don't treat them as final verdicts.

## Session 41 retrospective — corpus v2 + Phase B sweep atop positional eval

Session 37 concluded that every Session 36 rejection (RFP, LMP, PVS, aspiration) was **provisional**: the corpus was generated by an engine that didn't move pieces, so `depth-reached` graded eval-neutral wandering. Sessions 38–40 landed:
- **Positional evaluator** — magic-table piece-square + Guard/Champion mobility (`state/mobility.rs`) + rank-advance bonus. Pieces now move, `evaluate()` differentiates a corner Guard from a centred one, and self-play produces board activity instead of 200-ply `EndPhase` streams.
- **Corpus v2** — regenerated `corpus.txt` via search-driven play at depths 2/3/4 (cycled), `MAX_PER_STM_PER_GAME=2`, view-key dedup, 30 hand-selected positions across 6 categories (opening-with-skills, midgame-move, skill-phase-full, combo-loaded, endgame-with-skills, king-in-danger). Positions come from real search, not random rollouts.
- **Bench sweep tooling** — `game/bench/run_sweep.sh` runs the full 5-budget grid (`depth6` + `time100/500/1000/3000ms`) + determinism check as one command. Results go to `bench/results/<prefix>-<budget>.json`; baselines live at `bench/baseline-<budget>.json`.

Session 37's revisit condition ("re-grade every rejection atop the new eval") is now the entry criterion for this session. Ran the full Phase B set of low-complexity pruning/ordering candidates against the corpus-v2 baseline.

### B1: Null-move pruning — **ACCEPTED (2026-07-05)**

Implemented as `alpha_beta::ENABLE_NMP` gate. Depth reduction R=2, gated `depth ≥ 3 && !is_mate(α) && !is_mate(β) && !in_check`. Real null move (flip side-to-move + advance ply hash, no `EndPhase`), per the catalogue's §2 caveat.

Sweep vs corpus-v2 baseline (NMP off):
- Depth-6 wall-clock: **-9.4%**, nodes -8.7%, no score drifts, no best-move disagreements.
- 100ms: mean depth +0.10 plies, 3 deeper / 27 same / 0 shallower.
- 500ms: mean depth +0.13 plies, 4 deeper / 26 same / 0 shallower.
- 1000ms: mean depth **+0.27** plies, geom-mean NPS **+18.6%**, 8 deeper / 22 same / 0 shallower.
- 3000ms: mean depth +0.20 plies, 6 deeper / 24 same / 0 shallower.

Zero regressions at any budget. Committed and baseline moved.

### B2: Delta pruning in quiescence — REJECTED (correctness drift)

Implemented in `quiescence.rs` under `ENABLE_DELTA_PRUNE` with `DELTA_MARGIN` set to the max HP-swing bound. Skip loud actions where `static_eval + max_swing + margin < α` (mirror for maximiser).

- `margin=2500`: pruned aggressively, correctness looked OK at first pass — but `combo-loaded-03` at depth 6 swung score `275 → 157` (Δ=-118) at same depth. The evaluator was being denied lines it demonstrably cared about. Also cosmetic mate-distance drift on `king-in-danger-01/02` (`-999996 → -999995`).
- `margin=1500`: score drifts persisted (same positions, similar magnitude). Tightening further gave nothing to prune.

**Root cause.** Our eval is not material-dominated the way chess eval is; positional + mobility + skill-reach are a large chunk of the score. Delta pruning's bounding assumption ("only material-equivalent HP swings matter at this leaf") doesn't hold. The margin needed to be safe was so wide that nothing pruned.

**Revisit condition:** if we ever add a material-heavy eval mode, or if incremental eval makes per-node eval cheap enough that a wide-margin filter is still cost-negative. Not until then.

Rolled back completely — no lingering `ENABLE_DELTA_PRUNE`/`DELTA_MARGIN` in tree.

### B3: Move-ordering sort at depth ≥ 1 / ≥ 2 — REJECTED (sort overhead > cutoff gain)

Baseline sort guard was `depth ≥ 3`. Tried lowering to catch shallow nodes:
- **`depth ≥ 1`**: -10% geom-mean NPS at 1000ms, mean depth **-0.33 plies at 500ms**, 6/30 positions shallower. Sort cost at shallow nodes exceeded cutoff savings — leaves are cheap enough that a stable sort is measurable overhead per node.
- **`depth ≥ 2`**: essentially flat NPS, zero positions gained depth vs baseline. Never improved anything.

**Root cause.** The TT-move swap already runs unconditionally at every node (it's a single index probe + swap-to-front, not a sort). That plus killer/history for depth ≥ 3 captures most of the achievable first-move-cutoff rate. Sorting quiet tails at d1/d2 costs more than it saves because our branching factor at Skill-Phase is already narrowed by killers-at-front logic elsewhere.

Restored `depth ≥ 3` guard. Comment updated in `alpha_beta.rs` referencing this sweep.

### B4: TT-move-first + killer-promote only, no full sort — REJECTED (skill-phase blows up)

Idea: skip the history-sort entirely; only swap TT-move to index 0 and promote the two killers to indices 1–2. Cheap `killer_pair(...)` accessor. Depth ≥ 3 gate unchanged so the sort was replaced by promotion, not added atop it.

- Depth-6 aggregate: -4% wall-clock overall — but the by-category breakdown was disqualifying:
  - `skill-phase-full` bucket: **+55% nodes / +58% time** vs baseline.
  - Other buckets flat or slightly better.

**Root cause.** History-based sorting of the tail is load-bearing in Skill-Phase, where legal action counts are largest and TT/killer coverage is thinnest. Random tail order → poor cutoff rates on deep branches → exponential node blowup at exactly the positions the engine spends most time on.

Rolled back completely. Removed `ENABLE_KILLER_ONLY_ORDER` gate and the `killer_pair` accessor.

### What the sweep does and does not settle

**Confirmed valid atop positional eval:**
- **NMP is a genuine speedup.** All budgets improve, zero regressions. The Session 37 caveat is now discharged for NMP specifically.
- **The corpus-v2 + multi-budget-no-regression protocol is a working accept/reject gate now that the eval moves pieces.** Session 37's "depth-reached graded eval-neutral wandering" pathology is resolved.

**Still provisional from Session 36:**
- **PVS, aspiration windows, LMP, RFP** were not re-graded this session. The Session 36 rejections stand as-is, but the Session 37 caveat still applies — they need the same corpus-v2 sweep before final verdict. Most useful direction: LMR + PVS as a pair (per §5 note), since PVS's null-window re-search is what LMR's reduced-depth re-search rides on.

**Reinforced from Session 36 methodology:**
- Multi-budget sweep is mandatory. B2 looked fine at fixed-depth (-3% time) but the mate-distance / combo-loaded score drifts wouldn't have surfaced without the depth breakdown.
- Category-by-category node breakdown is what caught B4. Aggregate metrics would have accepted it.

### What lives in tree after Session 41

- `alpha_beta.rs`: `ENABLE_NMP=true` (accepted), sort guard restored to `depth ≥ 3` with an explanatory comment referencing this session's sweep, no delta-prune / killer-only remnants.
- `quiescence.rs`: unchanged from pre-B2 state.
- `bench/baseline-{depth6,time100ms,time500ms,time1000ms,time3000ms}.json`: regenerated with NMP=on.
- `bench/corpus/corpus.txt` + `raw_corpus.txt`: corpus v2 (search-driven).
- `bench/run_sweep.sh`: the sweep runner.

### Next in queue

Per the catalogue's recommended order, remaining Phase B / early-Phase-C items to grade atop the current baseline:
1. **LMR + PVS paired** — the catalogue's §5 note says they must be tested together. Highest expected Elo of anything remaining.
2. **Check extension** (King-threat +1 ply). Simple; test alone.
3. **Countermove heuristic + 1-ply continuation history.**
4. Re-grade **RFP** atop QS + positional eval + NMP (Session 37 revisit condition still open — was queued behind QS-lands and never re-run).

---

## Session 48 retrospective — Phase 1 eval throughput (search-deepening plan)

### Phase 1: monomorphic allocation-free scalar leaf path — **ACCEPTED (2026-07-12)**

The ns-43 term registry rebuilt itself on every `evaluate()`: 14 `Box<dyn
EvalTerm>` allocations (`default_terms`), an `active`/`per_piece`/`side_level`/
`acc`/`entries` `Vec` chain, `dyn` dispatch per term × square, and a full
`DynBreakdown::to_legacy()` projection to the frontend struct — all pure
overhead in the search leaf, which only wants an `i32`.

Two behaviour-preserving changes (commit `60dbc1e`), gated by `golden_eval_unchanged`
+ `eval_is_deterministic` staying byte-identical:
- `registry::default_terms_static()` — build the boxed term set once (`OnceLock`)
  and borrow it. Terms are stateless ZSTs reading `ctx.params`, so a single
  static set is correct; kills the 14 boxes/call on the breakdown paths too.
- `registry::evaluate_scalar()` — a monomorphic fold of the same term
  set/order/`is_active` gates/signs into one running `i32`: no `DynBreakdown`,
  no `acc`/`entries` Vecs, no `to_legacy`. `evaluate()` (hence the search leaf
  via `HeuristicEvaluator::evaluate`) routes through it.

Sweep vs the fresh post-ns-43 baseline (determinism 30/30):
- **depth6: geo-NPS 587K → 1,153K (+96.5%), total d6 22,496ms → 11,221ms
  (−50.1%), positions-over-1s 5 → 3.** midgame-move-05 (1519→836ms) and
  skill-phase-full-04 (1008→501ms) crossed under 1s. Every position ~50% faster;
  zero score/best-move regressions.
- time budgets: deeper at all four, **0 shallower anywhere** (100ms +0.43 plies /
  9 deeper; 500ms +0.47 / 14; 1000ms +0.37 / 10; 3000ms +0.50 / 14).

**The `--eval-only` microbench was misleading — read this before trusting one
again.** eval-only ran ~2270 ns/eval before AND after (flat, warm allocator,
predictable branch pattern in a tight loop). It completely hid a ~2× *search*
NPS gain. The win is allocator + cache pressure amortised across 5M+ real search
nodes with interleaved make/unmake, not per-call eval latency. **Grade
eval-throughput refactors on the search sweep, not eval-only.**

**Phase 1c ("≤150 ns/eval") is moot and was not pursued.** eval latency is
dominated by `EvalContext::new` (the `build_attackers_table` scatter + the two
`[i32;16]` availability tables + the value/material board pass), which every
path shares unchanged — not the registry glue Phase 1 removed. Phase 1a/1b/1c
collapsed into the single scalar-path commit; the scalar path is already
heap-allocation-free (all accumulators are stack scalars), which satisfies 1c's
actual intent. Further eval-throughput gains would need to attack
`EvalContext::new` itself (or incremental eval, 1d) — deferred; the remaining
3 over-1s positions are a tree-shape problem for Phase 2 (LMR+PVS).

Remaining d6-over-1s after Phase 1: opening-with-skills-03 (~2.7s),
midgame-move-03 (~2.4s), skill-phase-full-03 (~1.7s).

---

### Phase 2: LMR + PVS paired — **ACCEPTED (2026-07-12)**

Implemented together per the §5 note (commit `a9a734f`), both behind
`ENABLE_PVS` / `ENABLE_LMR` toggles (default on):
- **PVS** (absolute-POV): first move full window `[α,β]`; siblings null-window
  (max `[α,α+1]`, min `[β-1,β]`); full re-search on an in-window raise.
  **Proven exact** — a PVS-only build (NMP + LMR off) matched plain alpha-beta
  scores byte-for-byte across the corpus at depth 6. So the fixed-depth score
  drifts observed with the full stack are NMP + LMR heuristic behaviour, NOT a
  PVS re-search bug. (This finally discharges the Session 36 PVS rejection: it
  was rejected standalone because strong ordering left it nothing to save; atop
  LMR's reduced re-searches it pulls its weight.)
- **LMR**: reduce late (idx≥3), quiet (`!is_loud`), non-check, depth≥3 moves by
  `R = 0.75 + ln(depth)·ln(idx)/2.25`; re-search full depth on a reduced-probe
  raise. Never reduces the first move, in-check nodes, or loud/King-threatening
  actions (`is_loud` covers Move-Attacks + Strike/Blast + BodyguardChoice).

Sweep vs the Phase-1 baseline (determinism 30/30):
- **depth6: total 11,221ms → 9,567ms (−14.7%), over-1s 3 → 2**
  (skill-phase-full-03 1691→591ms crossed under). geo-NPS −4.5% (LMR re-searches
  lower the raw node rate even as the tree shrinks). Skill-phase/opening tails
  −40…−78%.
- time budgets deeper at all four: 100ms +0.37 plies (7 deeper/0 shallower),
  500ms +0.60 (14/1), 1000ms +0.67 (12/0), 3000ms +0.70 (16/3).

**The documented trade (accept-gate call).** The two widest-EBF positions
(opening-with-skills-03 ebf 11.9, midgame-move-03 ebf 12.2) got ~26%/12% SLOWER
at d6 and lose a ply at 3s (plus skill-phase-full-04). On trees that wide LMR's
reduced probes fail high and re-search often enough that the overhead outweighs
the pruning — the same "LMR hurts the worst case" pattern the Session-36 LMP
retro saw. They were already over 1s (not newly crossed over), over-1s dropped
3→2, and the aggregate improves at every budget → net win, accepted.

**Tuning tried:** a gentler formula (`0.5 + ln·ln/3.0`, min_idx 4) halved the
speedup (−7.2% vs −14.7%) and did NOT remove the midgame-move-01 fixed-depth
sign flip (LMR reduces the true-best line there regardless of formula
gentleness) — so conservatism bought no correctness, only lost speed. Kept the
aggressive setting.

Remaining d6-over-1s after Phase 2: opening-with-skills-03 (~3.5s),
midgame-move-03 (~2.8s) — both high-EBF, both slightly worse under LMR. These
are the Phase 3 / tree-shape targets (aspiration windows, LMP re-grade, or an
LMR "don't reduce / reduce-less at very high branching" refinement).

---

### Phase 3a: Late Move Pruning re-graded — **ACCEPTED (2026-07-12)**

LMP behind `ENABLE_LMP` (default on, commit `270a22a`): at non-PV, non-check
nodes with `depth ≤ 5`, quiet (`!is_loud`) moves whose ordering index exceeds a
depth-indexed threshold are skipped entirely (no reduced search). Guarded off at
mate boundaries. Schedule **{1→6, 2→9, 3→13, 4→18, 5→24}**.

**Why it extends to depth 5** (Session 36's clean config was depth-1-only
`{1→16}`): the two EBF-12 offenders spend most of their nodes in the depth-4/5
interior of a d6 search; a `depth≤3` schedule never touches that mass. Extending
through depth 5 is what actually bends their EBF down.

**Why it's clean now when Session 36 rejected it standalone:** Session 36's LMP
regressed because its leaf-set changes poisoned deeper-iteration TT/history
ordering against a slow eval and no other pruning. It now sits atop fast-eval +
QS + LMR/PVS, which absorb that churn.

Sweep vs the Phase-2 baseline (determinism 30/30):
- **depth6: total 9,567ms → 5,199ms (−45.7%), geo-NPS +1.7%, over-1s held at 2.**
  opening-with-skills-03 3475→1591ms (ebf 11.9→10.6), midgame-move-03
  2762→1521ms (ebf 12.2→11.0). Skill-phase/opening tails −70…−87%.
- time budgets deeper at all four: 100ms +0.47 plies (10 deeper/1 shallower),
  500ms +0.47 (13/1), 1000ms +0.60 (13/0), 3000ms +0.87 (12/1).

**Trade:** a handful of sparse-endgame / skill-phase positions lose a ply at one
budget (LMP mis-prunes in low-material tactical spots). Fixed-depth d6 shows ~5
best-move changes vs true minimax — heuristic; real play reaches d18-22 where
these shallow prunes are far from the root. Aggregate depth up everywhere,
over-1s unchanged, d6 nearly halved → net win.

**Schedule tuning:** a gentler low-depth variant `{1→8,2→12,3→16,4→22,5→30}` was
*worse* on the two targets (1982/1602ms vs 1591/1521ms) for the same ~5 bm
changes — the bm-change count is dominated by the near-root depth-4/5 pruning,
not the low-depth thresholds, so low-depth aggression is free-ish and helps the
goal. Kept the aggressive schedule.

Remaining d6-over-1s after Phase 3a: opening-with-skills-03 (~1.6s),
midgame-move-03 (~1.5s) — the last two, both ~1.5× over. Next levers: aspiration
windows (narrow root window → fewer top-of-tree nodes) or a further LMP nudge.

---

### Phase 3b: Aspiration windows — REJECTED again (root re-search cost)

Re-graded atop fast-eval + LMR/PVS + LMP (the Session 36 revisit condition).
`ENABLE_ASPIRATION`, `ASP_MIN_DEPTH=5`, `ASP_DELTA=40`, ×3 exponential widen,
keep-the-other-bound on a fail. Determinism held 30/30, but depth6 **regressed
+25.8%** (5,199ms → 6,543ms) and both remaining offenders got *worse*
(opening-with-skills-03 1591→2238ms, midgame-move-03 1521→2234ms).

Same mechanism as the Session 36 rejection: on these high-EBF positions the d5
score is unstable, so the narrow d6 window fails (low or high) and the root
re-search — now at a *bigger* tree thanks to LMR/PVS/LMP reaching deeper — costs
more than the narrow window ever saved. LMR/PVS made the re-search *structurally*
cheaper per node but there are more nodes under it, so the net is still a loss at
d6. Rolled back completely (no lingering toggle/const). Not worth re-trying at
this eval; would only pay off with a much more stable eval or a tighter δ that
then saves nothing.

---


### Phase 4: Eval throughput — physical-only attackers table + ray-skip LANDED; champion_threat vectorization + NN-eval REJECTED

Session 48 (ns-49). Goal was the last 2 over-1s d6 positions. After the search
wins (Phases 1–3), profiling turned to the eval itself.

**What landed (−2.5% to −3.2% d6, byte- AND node-identical, 448 tests green):**

1. **Physical-only attackers table on the eval path.** `EvalContext::new` built
   the full `build_attackers_table` (physical scatter + skill-scatter). But the
   evaluator only ever reads `p1_of`/`p2_of` (physical) via `any_attackers_of`
   — the skill-scatter fields (`*_skill_of`, `*_skill_kind`) are consumed ONLY
   by `see_capture`, never by eval. Added `build_attackers_table_phys` (skips
   `scatter_skills_side`, the queen-ray-per-champion/king pass) and pointed
   `EvalContext::new` at it. ~3.37M eval-path builds/sweep stop tracing skill
   rays they discard. Byte-identical: eval reads only what's unchanged.
2. **Ray-skip in `champion_threat`.** Skills whose `TargetOwner` is `Empty` /
   `SelfOnly` (Dash, Retreat, Shield, Focus, Charge) score nothing in the term,
   yet the loop traced their `skill_attacks` ray before discarding it. Moved the
   ray trace inside the branches that read it. Small (corpus champions mostly
   carry offensive skills) but free.

**What was rejected — vectorizing champion_threat (negative result):**

The initial profile *looked* like `champion_threat` was a 10× hot spot
(2314 ns/board-pass in opening-with-skills-03 vs ~50–90 ns for every other term).
**That was a microbench artifact** — timing the term through the per-term probe's
`PieceContext` dispatch loop. Timed *directly*, champion_threat is ~260 ns,
comparable to exposure (92 ns) / coverage (89 ns) / mobility (84 ns). **There is
no single dominant eval hot spot.** The op-counter confirmed it: the two positions
process near-identical work (~14 rays, 9–16 hits).

The vectorized rewrite (a `FAR_RAY[sq][dir]` octant table so all distance-≥2
strike hits in one direction share ONE landing-safety lookup, + masked-popcount
value sums replacing the per-hit loop) was proven byte-identical against a
verbatim reference over the corpus + ~1800 random positions — but **regressed the
search sweep +1.9%**. The common case is 0–2 hits per champion, where the simple
per-hit loop beats the popcount/closure machinery. Reverted completely (FAR_RAY
table + all probes removed). Lesson: **grade eval micro-opts on the search sweep,
and beware the per-term microbench** (it inflated a non-hot-spot ~9×).

**NN eval as a throughput lever — REJECTED (wrong tool).** Measured a single
`NnEvaluator` forward pass (burn `NdArray`, dense `2825→256→64→32→1`) at
**~394 µs/call vs ~1 µs for the hand-crafted eval — ~382× slower.** A single NN
pass is not faster; it is dramatically slower per call. NN is a *strength* lever,
not a *throughput* one, and only via a ground-up NNUE-style redesign (sparse
per-piece-per-square features + an incrementally-updated accumulator wired into
make/unmake + int quantization). Scoped separately in `nnue-rework-plan.md`.

**Net:** eval throughput is near its byte-identical floor. The remaining 2
over-1s positions are node-count-bound; the next lever is search-side (node
count), not eval cost per node.

---


- `search-speed-benchmark-plan.md` — the benchmark infrastructure that grades each technique.
- `nn-rater-plan.md` §7 — search-speed pass is step 1 of NN-rater execution; this catalogue feeds it.
- `oq-81` — search branching factor + strategy plan. This catalogue expands the mitigation list in oq-81's Layer 2.
- `core_engine/src/search/alpha_beta.rs` — current straight-alpha-beta implementation.
- `core_engine/src/search/transposition.rs` — TT (already exists; needs verification of bucketing/aging schemes).

## Sources

- [Move Ordering — chessprogramming wiki](https://www.chessprogramming.org/Move_Ordering)
- [Null Move Pruning — chessprogramming wiki](https://www.chessprogramming.org/Null_Move_Pruning)
- [Late Move Reductions — chessprogramming wiki](https://www.chessprogramming.org/Late_Move_Reductions)
- [Principal Variation Search — chessprogramming wiki](https://www.chessprogramming.org/Principal_Variation_Search)
- [Quiescence Search — chessprogramming wiki](https://www.chessprogramming.org/Quiescence_Search)
- [Transposition Table — chessprogramming wiki](https://www.chessprogramming.org/Transposition_Table)
- [Futility Pruning — chessprogramming wiki](https://www.chessprogramming.org/Futility_Pruning)
- [Reverse Futility Pruning — chessprogramming wiki](https://www.chessprogramming.org/Reverse_Futility_Pruning)
- [Aspiration Windows — chessprogramming wiki](https://www.chessprogramming.org/Aspiration_Windows)
- [Extensions — chessprogramming wiki](https://www.chessprogramming.org/Extensions)
- [Singular Extensions — chessprogramming wiki](https://www.chessprogramming.org/Singular_Extensions)
- [Internal Iterative Deepening — chessprogramming wiki](https://www.chessprogramming.org/Internal_Iterative_Deepening)
- [Lazy SMP — chessprogramming wiki](https://www.chessprogramming.org/Lazy_SMP)
- [ProbCut — chessprogramming wiki](https://www.chessprogramming.org/ProbCut)
- [Razoring — chessprogramming wiki](https://www.chessprogramming.org/Razoring)
- [History Heuristic — chessprogramming wiki](https://www.chessprogramming.org/History_Heuristic)
- [Alpha-Beta Search — chessprogramming wiki](https://www.chessprogramming.org/Search)
