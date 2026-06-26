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

### Aspiration windows
After iteration N, set `α = score_N − δ`, `β = score_N + δ` for iteration N+1, with δ ~ 25 cp. Widen exponentially on fail. Gain: 10-20% time-to-depth. Low complexity. Don't aspirate until depth ≥ 5. Disable near mate scores.

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

## Cross-references

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
