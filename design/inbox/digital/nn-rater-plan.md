# NN Position Rater — Lock-In Plan

*Session 35 brainstorm output (2026-06-26). Companion to `next_steps id=25`.*
*Status: aligned on shape, not yet ADR-ed. Next concrete step is the ADR comparing learning architectures (Path 1 vs Path 2 vs Path 3), then a search-speed pass, then implementation.*

---

## 1. Framing — decisions locked in

- **Parallel evaluator, not a replacement.** The existing hand-coded `evaluate()` in `core_engine/src/search/evaluator.rs` stays untouched and remains the default. The NN is a second `Evaluator` implementation, selectable at runtime. Multiple NN versions also coexist behind the same trait.
- **Goal: a rater that out-plays the hand-coded eval in head-to-head matches.** Not a design-signal tool (yet). Not a draft-phase rater (yet — draft scoring is coupled to play scoring in ways we want to untangle separately).
- **Search improvements continue in parallel.** "Search less, smarter" (move ordering, TT, quiescence — oq-81 territory) is orthogonal to "score the leaves better" (NN). Both tracks advance independently.
- **Search-speed pass happens before training begins.** Training spends most of its time inside the search loop; making search faster pays compounding dividends across millions of self-play games. See §7.
- **Inputs are raw, not derived.** Feeding hand-engineered features (king-safety scalar, mobility count, etc.) biases the rater toward the same blind spots as the hand-coded eval. We trust the net to learn its own intermediate features from raw bits.
- **Terminals bypass the NN entirely.** A captured King or N-ply-mate position short-circuits to ±MATE_SCORE via the existing eval; the NN never sees them. Keeps the MATE_SCORE convention and mate-distance math intact.

## 2. Inputs — "every bit a Position can hold"

For Stack M on 8×8, roughly:

- **Per-square occupancy planes** (one-hot): P1-king, P1-champion, P1-guard, P2-king, P2-champion, P2-guard → 6 × 64 = 384 binary inputs.
- **Per-square mailbox state**: hp, armor, skill1 id, skill2 id, charged-flag, focus-flag, injured-flag (if Stack M retains it), combo-counter. Categorical fields (skill IDs especially) want one-hot expansion, not raw integer encoding — confirm via /research, see §8.
- **Global state**: P1 money, P2 money, side-to-move, phase, round number, action-budget-remaining, P1/P2 skill-phase actions used this round.

Estimate: ~1500–2000 input bits. No derived features.

## 3. Output

Single scalar in the same units and sign convention as `evaluate()`:
- P1-POV (positive = P1 advantage).
- Non-terminal magnitude roughly bounded by ±36k (current material+HP+armor+skills+money sum).
- Terminals never routed through the NN.

## 4. Learning architecture — Path 3 with perturbation injection

**Decided:** gradient descent on self-play labels (Path 3), with periodic random weight perturbations layered on top to escape local minima.

### Why Path 3

Every strong modern engine (Stockfish-NNUE, AlphaZero, Leela) is gradient-trained. Gradients tell you the *direction* of improvement — blind mutation only tells you whether a random jump was lucky. With ~100k weights in a 1500-input dense architecture, the search space is far too big for evolution alone to compete. Path 1 (pure mutation) was considered as a baseline-first stepping stone but rejected — we go straight to Path 3 and accept the dependency cost.

### Mechanics

- **Topology fixed.** A pre-set layer structure (e.g. 1500 → 256 → 64 → 1; final shape TBD during implementation). All learning happens via weight updates.
- **Labels.** Two candidate signals, both viable; the ADR (or first implementation pass) picks one:
  - Game outcome (AlphaZero-style): each position in a self-play game is labelled with the eventual game result. Cheap per label, slow to converge.
  - Deep-search score on the position: cheaper per game (more labels per game) but biased toward agreeing with the current search.
  - Decision deferred to implementation — likely start with outcome labels, add deep-search bootstrapping if convergence is slow.
- **Optimiser.** Standard (Adam or SGD with momentum). Decide during implementation.
- **Backprop.** Pull in a Rust autograd crate (`burn`, `candle`, or `dfdx`). Hand-rolling backprop in Rust is annoying for diminishing return. Crate choice decided when implementation starts.

### Perturbation injection (local-minimum escape)

Layered on top of standard gradient descent:

1. **Periodic perturbation.** Every K training steps (K to be tuned — likely 1000s of steps), spawn M perturbed copies of the current best weights by adding Gaussian noise. Continue gradient training each in parallel for a short burst. If any perturbed copy ends up stronger than the unperturbed line after the burst, it becomes the new best.
2. **Population maintained.** Keep a small population of training lineages running in parallel (e.g. 4 lineages). Each lineage runs its own gradient descent with periodically perturbed restarts. The best across lineages is the candidate for the gauntlet.
3. **Restart strategy.** If a lineage plateaus for a long stretch (no win-rate improvement against the prior champion), fork it with heavier perturbation — high-magnitude noise pushing it further from its current basin.

This is the "Path 3 + semi-random mutations" you asked for. It's a known hybrid — sometimes called "gradient + ES" or "noise injection." Pure gradient descent gets stuck in local minima; pure mutation can't climb gradients efficiently. The hybrid does both.

### What this does NOT include

Topology mutation (NEAT-style). The structure is fixed; only weights move.

## 5. Selection — gauntlet, two-tier

### Two tiers (cheap filter → expensive acceptance)

**Tier 1 — fitness filter:**
Each candidate plays a mini-gauntlet against the top-K (K=3) of the previous generation at the **fast think-time only (100 ms/ply)**. Fitness = win-rate in that mini-gauntlet. Cheap; runs every training milestone.

**Tier 2 — acceptance gauntlet:**
Top fitness performers go to the full acceptance test:
- **Best-of-three** mirrored matches against every previously-accepted version.
- Three think-time brackets per opponent: **100 ms / 300 ms / 500 ms per search per ply.** Best-of-three is the per-bracket decider — clean 2/3 winner.
- Mirrored loadouts: random-but-legal loadout, both sides play both colours with the same loadout, draft luck cancels.
- A candidate becomes "accepted" iff it wins the best-of-three against the immediate predecessor at all three brackets AND achieves ≥45 % win-rate against every prior accepted version at all three brackets (non-regression).

### Three champion tracks

We maintain three tracked "champions" simultaneously:
- **best-fast** — leader of the 100 ms bracket.
- **best-slow** — leader of the 500 ms bracket.
- **best-overall** — weighted aggregate across all three brackets, **slow bracket weighted higher than medium, medium weighted higher than fast** (rationale: in real games players have time to think, so the slow bracket reflects real-world performance better). Concrete weights TBD — likely something like 1.0 (slow) / 0.6 (medium) / 0.4 (fast).

A new version is "accepted" into the gauntlet membership list if it qualifies for *any* of these three tracks (i.e. it beats the predecessor in best-of-three at its bracket AND meets the non-regression bar). The three tracks can diverge — best-fast and best-slow may be different rater versions. That's fine and expected.

### Why three brackets

Raters can specialise. A rater that's great at 100 ms may be worse at 500 ms (different search trees). Three brackets keeps us honest and produces the best-fast / best-slow split above.

### Loadout generator

A standalone, legality-checked random-loadout generator. Same generator feeds Tier 1 and Tier 2. Eventually overlaps with oq-83 (AI draft strategy) but is owned by the trainer for now.

## 6. Avoiding local minima

Combination of two layers, both already specified above:

1. **Path 3 + perturbation injection** (§4). Periodic Gaussian-noise perturbations of the best lineage, parallel training lineages, plateau-triggered heavy-noise restarts. This is the primary defence — built into the training loop itself.
2. **Population-based selection** via the two-tier gauntlet (§5). Multiple training lineages → multiple candidates per acceptance window → the gauntlet picks among them.

### Parked research questions

The /research call described in §8 is **parked**, not required. The plan is internally consistent and ready to implement without it. If we hit a wall during implementation — particularly on label choice, optimiser tuning, or perturbation cadence — we run /research then. Until then, we proceed.

## 7. Search-speed pass — happens FIRST

Before training begins, profile and tighten the search loop. Training time is dominated by search nodes-per-second × games-per-generation × generations. A 2× speedup in search ≈ 2× more training in the same wall-clock.

Targets (informed by oq-81):
- Move ordering (captures first, threats, then quiet) — biggest alpha-beta speedup, no correctness impact.
- Zobrist hashing wired (already on the Slice-6 TODO).
- Transposition table populated.
- Killer-move + history heuristics.
- Quiescence search past Strike-skill chains.

Not all of these need to land before training starts — but **at least move ordering and a working TT must.** Otherwise the first hundred generations of training are bottlenecked on search inefficiency, not eval quality.

## 8. Parked research questions

These are stored for later — they would refine implementation choices but are not required to begin. Plan as written is internally consistent; we proceed without research and run /research only if implementation hits a wall on one of these.

1. **Architecture for tiny game-eval NNs.** NNUE-style (incremental update + sparse halfkp-equivalent) vs. dense MLP vs. small CNN over piece-occupancy planes. For ~1500 inputs and sub-millisecond eval budget on an 8×8 board with skill state, what's the current state-of-the-art trade-off?
2. **Training methodology refinements.** Best practice for Path 3 with perturbation injection in 2-player perfect-info game evaluators — perturbation magnitude schedules, population size, lineage-merge policies.
3. **Local-minimum escape.** Additional techniques beyond perturbation injection (novelty search, fitness sharing, hall-of-fame gaunttlets) used in published game-AI work.
4. **Categorical input encoding.** For small categoricals like skill-IDs in a tiny dense NN, one-hot vs. learned embedding vs. raw integer — which works best at our scale?
5. **Label choice.** Game-outcome labels vs. deep-search-score labels vs. hybrid (TD-leaf, bootstrap from search) for engine-style evaluators at our scale.

If run, this should be a single consolidated /research call with full context (this document attached). Either in a fresh focused conversation (preferred, keeps thread context clean) or as the opening action of a future session.

## 9. Backend integration

### Evaluator trait seam (lands first, regardless of ADR outcome)

```rust
// Sketch — final form during implementation
pub trait Evaluator {
    fn evaluate(&self, pos: &Position) -> i32;
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown;
}

pub struct HeuristicEvaluator;  // wraps today's free functions
pub struct NnEvaluator { weights: NnWeights, /* … */ }
```

`SearchCtx` gains `evaluator: &dyn Evaluator`. The single call in `alpha_beta.rs:search()` becomes `ctx.evaluator.evaluate(pos)`. Default wires `HeuristicEvaluator` — zero behaviour change.

Lands BEFORE any NN code. Small, safe, isolates the integration risk.

### Training crate

New crate: `game/crates/nn_trainer/` (final name TBD). Depends on `core_engine`. Native-only — no WASM in the training loop. Owns:

- `Weights` type and shape descriptor.
- `forward(pos, &weights) -> i32` — inference path also used inside `NnEvaluator`.
- `backward(...)` — gradient computation via chosen autograd crate.
- `train_step(batch, &mut weights, optimiser_state)` — one optimiser step on a labelled batch.
- `perturb(parent, magnitude, &mut rng) -> Weights` — Gaussian noise injection for the perturbation-injection layer.
- `play_match(rater_a, rater_b, loadout, think_ms) -> Outcome`.
- `lineage_round(lineages) -> NewLineages` — runs one round of parallel-lineage training, applies perturbation injection schedule, runs the gauntlet for any candidates that hit the acceptance milestone.
- **Multithreaded** via `rayon` — self-play games are embarrassingly parallel, each is a pure function of (rater_a, rater_b, loadout_seed). A 16-core machine runs 16 games at once; training time scales close to linearly with core count.

### Weight storage and versioning

- Each accepted version: a single binary blob (`raters/v0042.bin`) plus a JSON sidecar (lineage ID, parent ID, training step count, perturbation history, win-rate against predecessor per bracket, hyperparameters, git SHA, date).
- Index file (`raters/index.json`) lists accepted versions in order — this is the gauntlet membership list. Each entry tags which track(s) it leads: best-fast / best-slow / best-overall.
- Committed to repo. Blobs are KB-scale, no LFS needed.
- WASM build embeds a chosen version via `include_bytes!` at compile time. Default is best-overall. No runtime fetch.
- Gauntlet result matrix (every version vs. every version, per think-time bracket) stored separately as a CSV or JSON for the observatory UI.

### IPC — local file/shared-memory polling, no network, UI never blocks training

The trainer is the source of truth. UI is a passive observer.

- **Transport.** Local-only. Trainer writes status snapshots to a file (likely JSON for summary state, binary for live-position state) or a memory-mapped region at a configurable cadence (e.g. summary every 1 s, live-position every ply when subscribed).
- **UI polls.** UI reads on its own timer (e.g. 1 Hz for summary panels, 4 Hz when Live Match View is focused). No callbacks, no streams, no websockets.
- **Subscription mechanism.** When Live Match View is focused/visible, UI writes a sentinel file or flag indicating "subscribed." Trainer checks the flag once per ply and writes live-position state only while subscribed. When unfocused, no live-position writes — full speed resumes.
- **No interference.** UI cannot pause, cancel, or otherwise control training while it's running. (Stopping training is a separate manual operation — kill the process or send a signal via CLI, not the UI.)
- Concrete file paths and serialisation format decided during implementation.

## 10. UI — Training Observatory

**Primary principle: the UI is opt-in observability. Default state = headless training at full speed on all cores. Open the page = subscribe to one live game + summary stats. Close the page = unsubscribe, full speed resumes.** UI must never throttle training.

### Route: `/training`

#### Panel 1 — Live Match View
- Reuse the existing board renderer from `match/+page.svelte`.
- Current self-play game from the active tournament round: position, last action.
- **Three eval bars** at the side: current candidate NN, current opponent NN, AND the hand-coded heuristic (the heuristic isn't driving the game — it's a control signal showing how all three differ on the same position).
- Header: "Rater v42 (challenger) vs Rater v36 (defender) — game 3 of 6, ply 47, think budget 300 ms."
- Speed control: pause / play / step-ply / 1× / 4× / 16× / fast-as-possible.
- **Render only when focused/visible.** When the panel is hidden or off-screen the trainer stops streaming live-game ply events to the UI. Summary events (game-completed, generation-completed) still flow.
- "Skip to end" button → jumps to result, advances to next game.

#### Panel 2 — Tournament Standings
- Table of the current generation's population: rater ID, parent ID, generation number, W-L-D against opponents played so far this round, predicted Elo, alive/eliminated state.
- Highlight which two are playing right now.
- Footer: "Generation 17, round 3 of 5, ETA 4 m 12 s."

#### Panel 3 — Lineage Tree
- Tree view of accepted-version history. Root at top (v0). Each accepted version is a node; edges show parent → child.
- Hover a node → tooltip with metadata (generation, mutation magnitude, win-rate vs predecessor, think-time brackets cleared).
- Click → loads weights into Panel 4.
- Main line of accepted versions runs down the centre; rejected siblings branch off and fade.

#### Panel 4 — Network Inspector
- For the selected rater:
  - **Weight heatmap** per layer — spots pathologies (collapsed layers, NaN drift).
  - **Live forward pass** on Panel 1's current position — layer 1 activations rendered as 6 × 8 × 8 grids (one per occupancy plane), interpretable.
  - **Per-square contribution overlay** — perturb the position by removing one piece at a time, re-run forward pass, plot Δscore per square. Sanity check: "what does this rater think each piece is worth, given the rest of the position." If v42 thinks the King is worth 12 and a Guard is worth 800, we have a bug.

#### Panel 5 — Gauntlet Matrix (separate tab)
- N×N grid of every accepted version vs. every accepted version.
- Cell colour = win-rate of row vs. column. Diagonal blank.
- Dropdown per think-time bracket: {100 ms / 300 ms / 500 ms}.
- Should be roughly upper-triangular if progress is real. Chaotic matrix = something is wrong.

### What this UI is NOT
- Not a debugger for individual games. Live View is for watching, not stepping. To dissect a specific self-play game, add a "load this game into Inspector" button that routes out of the observatory and into the existing Inspector tools.
- Not real-time during training of large networks. For our scale this is fine; if architecture grows we decouple training (background, headless) from observation (load checkpoints into the observatory).

### Polling model (see §9 IPC for transport detail)
- Training binary runs games at full speed in worker threads; never blocks on UI.
- Trainer writes summary status snapshots at a low cadence (~1 Hz) regardless of UI state.
- Trainer writes live-position state per ply only while UI is subscribed (Panel 1 focused and visible) — sentinel flag controls this.
- UI polls the snapshots on its own timer; never pushes commands to the trainer while training runs.
- UI cannot pause, cancel, or otherwise interfere with the live training process. Stopping training is a CLI / signal operation, not a UI button.

## 11. Execution order

1. **Search-speed pass** — move ordering, Zobrist + TT wiring, killer/history heuristics, quiescence. (§7) Top priority — every speedup compounds across millions of training games.
2. **`Evaluator` trait seam** in `core_engine` — small, zero-behaviour-change refactor. (§9)
3. **Stub `nn_trainer` crate** — empty crate, dependency wiring, no logic. Lands the workspace structure. Choose autograd crate (`burn` / `candle` / `dfdx`) at this point.
4. **Implement Path 3 training loop** — forward, backward, optimiser step, label generation, parallel lineages, perturbation injection schedule. (§4)
5. **Implement gauntlet protocol** — two-tier, best-of-three per bracket, three-track champions, mirrored loadouts. (§5)
6. **Weight storage + IPC plumbing** — versioned blobs, index file, status snapshot writer, sentinel-flag live-position subscription. (§9)
7. **Training Observatory UI** — `/training` route, 5 panels (§10). Can land in parallel with steps 4-6 once the snapshot format is defined.
8. **First training run** — accept v1 if it beats the heuristic; iterate. /research (§8) only if implementation stalls.

Steps 1-3 are independent and can happen in any order. Steps 4-6 should be in order. Step 7 can begin as soon as step 6 has a snapshot format defined.

No ADR is required — Path 3 + perturbation is locked in. An ADR may still be worth writing as a record of the decision (especially the Path 1/2 rejection rationale) but it is not blocking implementation.

## 12. Cross-references

- `next_steps id=25` — original NN-rater idea seed. This document is its expansion.
- `oq-81` — AI search branching factor + strategy plan. The "search less, smarter" track running parallel to this one.
- `oq-83` — AI draft strategy. Currently a placeholder. NN rater might eventually feed it; not in scope here.
- `core_engine/src/search/evaluator.rs` — current hand-coded eval, header comment carries load-bearing eval philosophy.
- `core_engine/src/search/alpha_beta.rs` — search loop; single point where `evaluate()` is called per leaf.
- `adr-005` — digital architecture (Rust core + Svelte/Tauri + P2P MP). Establishes the host environment for the NN rater.
