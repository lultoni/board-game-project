# NN Evaluator Plan — NNUE-Style Rework (authoritative)

*Session 48 (ns-49, 2026-07-12). **Supersedes and absorbs `nn-rater-plan.md`** (Session 35), which is retired.*
*Status: single authoritative NN-eval plan. Not yet ADR-ed. Architecture decision (NNUE accumulator over dense MLP) is settled by the Session-48 latency measurement below.*

---

## 0. Why this document exists (the measurement that forced it)

The prior `nn-rater-plan.md` (Session 35) speced the *training loop, selection
gauntlet, and observability UI* for a learned evaluator — but assumed a **dense
MLP** and **parked** the NNUE-vs-dense architecture question. Session 48 measured
that parked question decisively, and the answer changes the architecture:

- **Hand-crafted `evaluate_scalar`: ~200–1000 ns/call** (varies with position +
  bench overhead; ~260 ns in isolation).
- **Current `NnEvaluator` (burn `NdArray`, dense `2825→256→64→32→1`): ~394 µs/call
  — ~382× slower.** Batch-size-1 dense inference through a generic f32 tensor
  framework is pathologically slow per-call: it re-encodes a 2825-float vector
  and recomputes the full 2825×256 first layer every node.

A leaf evaluator that costs 382× the current eval cannot run inside the search at
the node rates training needs (millions of nodes across millions of self-play
games). **The dense design is a non-starter for search-speed inference**,
independent of how well it could be trained.

### What actually makes a learned eval fast (the NNUE trick)

Stockfish-NNUE searches tens of millions of nodes/sec with a *bigger* net than
ours because it never evaluates its first (huge) layer from scratch:

1. **Sparse binary features.** Inputs are "piece of {type,color} on {square}
   (relative to king)". A position activates ~N_pieces of ~tens-of-thousands of
   features. All other features are 0.
2. **The accumulator.** The first-layer output (a few hundred–thousand ints) is
   kept as running state. A move flips only a few input features, so the
   accumulator is updated by **adding/subtracting a handful of weight columns**
   on make, and reversing them on unmake — **O(features changed) ≈ O(1) per
   node**, not O(input × hidden). Only the tiny remaining layers run fully.
3. **Integer quantization + SIMD.** int8/int16 accumulator with hand-written
   AVX, ~an order of magnitude over generic f32.

This is the *same* incremental idea ns-49 rejected for the hand-crafted eval
(our terms depend on the global attackers table, which shifts board-wide per
move) — **NNUE makes it work by designing the *inputs* to be per-piece-local, so
they ARE incrementally updatable, and pushing all global reasoning into the
learned weights of later layers.**

**Consequence for us:** even done perfectly, an accumulator eval is *competitive
with* (not dramatically faster than) our ~260 ns hand-crafted eval. The payoff is
**strength at similar cost**, not raw speed. NN is a strength lever. This scope is
about making a learned eval *fast enough to use*, so the strength work in
`nn-rater-plan.md` can actually pay off in-search.

---

## 1. Scope — this is the whole plan now

`nn-rater-plan.md` is retired; its still-valid pieces are absorbed below with the
Session-48 architecture decision and the Session-48 direction changes applied.
This document owns the full NN-eval effort:

- **Architecture** (§3): sparse, binary, per-piece-per-square features;
  incrementally-updatable accumulator; integer quantization; a non-tensor-framework
  in-search inference path.
- **Phase 0 — bootstrap to the current eval** (§2, §4): supervised regression to
  `evaluate_scalar` via **gradient descent**, before any self-play. The de-risking
  slice.
- **Phase 1 — self-play refinement** (§4): **large randomized mutation + gauntlet
  selection** (not fine gradient steps — compute is scarce, so bet on lucky-stroke
  jumps), seeded from the Phase-0 net.
- **Selection** (§5): a **single fast (100 ms/ply) track**, best-of-three per
  matchup, mirrored loadouts from the **corpus builder's** realistic generator.
- **Backend infra** (§5): versioned weight blobs, index, IPC snapshots.
- **UI** (§6): a Training Observatory — **built only after the backend works**,
  and never allowed to throttle training.

### What changed from the retired plan (decisions, Session 48)

1. **Three champion tracks → ONE fast track (100 ms).** *(Elias.)* The old plan
   maintained best-fast / best-slow / best-overall across 100/300/500 ms brackets.
   We drop that: train and select at the **fast bracket only**. Rationale: forcing
   the net to predict positions well at *shallow* search is the hard part; that
   skill snowballs into larger advantages the deeper/longer we later search. This
   **directs a code simplification** — `gauntlet.rs` currently implements the
   three-bracket `Bracket`/`ChampionTracker`/`tier2_acceptance` machinery (built to
   the old plan); collapse it to a single-bracket gauntlet.
2. **Self-play = large randomized mutation, not gradient + small perturbation.**
   *(Elias.)* With limited compute for self-play games, bet on **big random weight
   changes** to occasionally hit a large improvement, filtered by the gauntlet —
   rather than the old plan's fine gradient descent + Gaussian perturbation
   population. Gradient descent is retained ONLY for the Phase-0 supervised
   bootstrap (where it's the right tool for a known target).
3. **Loadouts come from the corpus builder.** *(Elias.)* Self-play uses the
   corpus builder's *realistic* weighted-incremental loadout generator
   (`core_engine/examples/build_corpus.rs::random_loadout` — strike + diverse
   categories), NOT the trainer's separate uniform `loadout::random_loadout_from_seed`.
   These two generators currently diverge — **unifying them (one realistic,
   reproducible, seed-based generator shared by corpus build + self-play) is a
   task this plan adds.** Mirrored loadouts (same loadout, both sides play both
   colours) stay, wrapped at the gauntlet layer.
4. **UI after backend.** The 5-panel Observatory is deferred until the training
   backend is working and validated; then it must show every available version +
   all active runs, as a passive observer that cannot throttle training.

### Already landed (reused, not re-speced)

- The **`Evaluator` trait seam** and a working `NnEvaluator` exist in
  `core_engine` / `nn_trainer` (the seam the old plan §9 called for).
- The **search-speed pass** the old plan §7 required "before training" is
  substantially done (ns-46…ns-49: fast scalar eval, LMR/PVS, LMP, the eval
  throughput pass). Training is no longer bottlenecked on an un-optimized search.
- `nn_trainer` scaffolding — `selfplay.rs`, `gauntlet.rs`, `batch.rs`, `train.rs`,
  `lineage.rs`, `loadout.rs`, `calibration.rs`, `snapshot.rs`, `persistence`,
  `lineage_checkpoint.rs` — exists and is reusable, but was built to the OLD
  (dense, three-track, gradient) plan. The rework re-points it: dense `encoding`
  → sparse+accumulator; three-track gauntlet → single fast track; gradient
  self-play → mutation self-play.

**Invariant kept from the old plan:** terminals bypass the NN entirely (a captured
King / N-ply mate short-circuits to ±MATE_SCORE via the existing eval; the NN never
sees them, preserving mate-distance math).

---

## 2. The de-risking insight — bootstrap from the current eval FIRST

*(Elias, Session 48.)* Don't start training from random weights against self-play
outcomes (slow, noisy, hard to tell if the *architecture* even works). Instead:

**Phase 0 = supervised regression to the current hand-crafted eval.** Generate a
large corpus of positions, label each with `evaluate_scalar(pos)`, and train the
NNUE net to *reproduce those scores*. Success criterion: the net matches the
hand-crafted eval within a small error band across the corpus.

Why this is the right first milestone:
- **Proves the architecture + inference path end-to-end** (encoding → accumulator
  → quantized forward → centipawns) on a target with a *known correct answer*,
  before adding the uncertainty of self-play.
- **Gives a non-random starting point** for the self-play phase — the net already
  "plays like" the current eval, so gradient descent refines a competent player
  instead of climbing out of noise. Faster convergence, and the first gauntlet
  entry is guaranteed ≈ heuristic-strength rather than a coin-flip.
- **A clean regression harness.** "Does the net reproduce the hand-crafted eval to
  within ε?" is a crisp, cheap, repeatable test — the analogue of
  `golden_eval_unchanged` for the learned path. It catches encoding bugs,
  quantization drift, and accumulator/full-recompute mismatches immediately.
- **Cheap labels.** No self-play games needed — `evaluate_scalar` is ~260 ns, so
  millions of labels cost seconds, not CPU-days.

Phase 0 does NOT need self-play, the gauntlet, or the UI. It is the smallest
end-to-end slice that de-risks the whole architecture.

---

## 3. Feature & architecture design (the rework proper)

*All concrete numbers below are starting points to confirm during implementation
— the point is the SHAPE, not the exact dims.*

### 3.1 Sparse binary features (replaces the dense 2825-f32 vector)

- **Per-piece-per-square occupancy**, one plane per (owner, piece-kind):
  {P1,P2} × {King, Champion, Guard} × 64 = 384 base binary features. A move sets
  ≤ a few of these.
- **Per-square mailbox state as binary buckets** (NOT raw ints): hp, armor as
  one-hot/threshold buckets; skill1/skill2 as one-hot over the 15 skill IDs;
  charged/focus/injured/combo flags as bits. Categorical → one-hot so a skill
  change flips exactly two features (old id off, new id on) — keeping the
  accumulator delta small. (The retired plan parked one-hot-vs-embedding for
  categoricals; for accumulator-friendliness, **one-hot wins** — decision made here.)
- **Global state** (money, side-to-move, phase, round, action budget) — these are
  NOT per-square and change rarely; fold into a small dense side-input that runs
  each node (cheap) OR bucket them into features too. Decide in Phase 0.
- **King-relative indexing** (HalfKP-style) is the classic NNUE feature multiplier.
  **Open question — do we need it?** Chess uses it because king safety dominates;
  our board is 8×8 with a king too. Start WITHOUT king-relative (simpler, smaller
  feature set); add it only if Phase-0 accuracy or Phase-1 strength demands it.
  Logged as a decision to revisit, not a day-1 commitment.

### 3.2 Topology

- `sparse_features → accumulator (e.g. 2×256 or 512) → 32 → 1`, with a
  clipped-ReLU (int-friendly) between layers. Final scalar in P1-POV centipawns,
  same sign/scale convention as `evaluate_scalar` (Phase 0 target).
- Accumulator width is the main size/strength knob; start small (256), grow if
  Phase-1 strength stalls.

### 3.3 The accumulator + make/unmake seam (the load-bearing rework)

This is the part that does not exist today and is the whole point:

- `Accumulator` = the running first-layer sum, held in the search stack alongside
  the position (or in `SearchCtx`).
- On `make(move)`: compute the set of feature indices that turned on/off (piece
  left A, arrived B, skill/hp/armor/flag deltas), and `acc += W[:,on] − W[:,off]`.
- On `unmake`: reverse (either recompute the delta, or snapshot-and-restore the
  accumulator — snapshot is simpler and O(width), decide by measurement).
- **A full-recompute path must exist** (`refresh(pos) -> Accumulator`) both for the
  root and as the correctness oracle.

**Correctness invariant (the golden test for this path):** for any position
reached by any make-sequence, the incrementally-updated accumulator must be
**bit-identical** to `refresh(pos)`. This is the exact analogue of the
`incremental_matches_scalar_over_playout` test — and it's the failure mode ns-49
hit with the hand-crafted incremental attempt, so it is a *first-class,
must-pass* test here, driven over random playouts + the corpus, asserting on
EVERY node including after unmakes.

### 3.4 Integer quantization + inference

- Train in f32 (burn/candle/dfdx — already `burn` in `nn_trainer`); **quantize to
  int8/int16 for the in-search inference path.** The search must NOT call burn
  per node — inference is a hand-written integer forward pass over the accumulator
  + small tail layers.
- Quantization introduces its own rounding; the Phase-0 regression harness must
  grade the **quantized** path against the eval, not just the f32 net.

---

## 4. Execution phases

**Phase 0 — Bootstrap architecture proof (no self-play).** *The de-risking slice.*
1. Design + implement the sparse feature encoder (`encode_sparse(pos) -> feature
   set`). Unit-test feature counts + that a `make` flips the expected small set.
2. Implement `Accumulator`, `refresh`, incremental update on make/unmake, and the
   `acc == refresh(pos)` playout invariant test. **This must be green before any
   training** — it's the architecture's correctness foundation.
3. Implement the quantized integer forward pass (accumulator → tail → centipawns).
4. Generate a labelled corpus: N positions (self-play *rollouts with the current
   eval* or random-legal walks), each labelled `evaluate_scalar(pos)`.
5. Train (f32, in `nn_trainer`) to regress the labels; quantize; measure the
   quantized net's error vs. the hand-crafted eval across a held-out corpus.
6. **Milestone:** quantized net reproduces `evaluate_scalar` within ε (target band
   TBD — e.g. mean |Δ| < a few % of typical score) AND the in-search integer
   inference is measured at ≲ a small multiple of the hand-crafted eval's ns/call.
   If inference is still >~5× the hand-crafted eval, the architecture needs
   rework before proceeding (revisit width/quantization/accumulator).

**Phase 1 — Self-play refinement (large randomized mutation + gauntlet).** The
Phase-0 net becomes the *initial weights*. Then, instead of fine gradient descent:
1. **Mutate big.** Produce candidate nets by adding *large* randomized weight
   changes to the current champion (heavy Gaussian noise / coarse jumps), betting
   that with limited self-play compute an occasional large jump lands a real
   improvement. (Contrast with the retired plan's fine gradient + small-perturbation
   population — dropped because we can't afford the games that approach needs.)
2. **Gauntlet-filter** each candidate at the single fast track (§5). A candidate is
   accepted only if it clears the bar vs. the current champion.
3. **Iterate.** The accepted champion becomes the next mutation parent.
First acceptance target: **beat the hand-crafted eval head-to-head** at 100 ms/ply.

**Phase 2 — Optimization (only if warranted).** King-relative features, wider
accumulator, SIMD on the integer path. Each graded on gauntlet strength +
in-search ns/call, never on microbench.

**Phase 3 — Training Observatory UI (only after the backend works).** §6.

---

## 5. Selection — single fast-track gauntlet + backend infra

### 5.1 One track: fast (100 ms/ply)

*(Decision, Session 48 — supersedes the retired plan's three-track design.)* We
train and select at the **100 ms/ply bracket only**. No best-slow / best-overall,
no 300/500 ms brackets.

- **Rationale (Elias):** the hard, valuable skill is predicting positions well at
  *shallow* search. A net forced to be right at 100 ms carries that advantage into
  any deeper/longer search we run later — the benefit snowballs rather than needing
  a separate slow-bracket champion.
- **Code impact:** `gauntlet.rs` was built to the old plan — it has a three-value
  `Bracket` enum, `ChampionTracker` (best-fast/slow/overall), and a
  `tier2_acceptance` that tests all three brackets. **Collapse to one bracket:** a
  single champion, acceptance decided at 100 ms only. Remove the slow/overall
  tracks and the multi-bracket loop.

### 5.2 Match protocol (kept from the retired plan)

- **Best-of-three per matchup**, to damp single-game luck.
- **Mirrored loadouts:** the same loadout is played by both sides across the games
  so draft luck cancels; mirroring is wrapped at the gauntlet layer over the
  loadout generator.
- **Acceptance:** a candidate is accepted iff it wins the best-of-three against the
  current champion at 100 ms. (Optional non-regression vs. a small hall-of-fame of
  prior champions can be added if mutation causes cycling — decide if it happens.)

### 5.3 Loadouts — unify on the corpus builder's generator

*(Decision, Session 48.)* Self-play loadouts must use the **corpus builder's
realistic** generator (`core_engine/examples/build_corpus.rs::random_loadout` —
weighted-incremental, guarantees a strike + diverse categories, `validate_loadout`-
clean), NOT the trainer's separate **uniform** `loadout::random_loadout_from_seed`.
The two currently diverge.

- **Task this plan adds:** extract ONE realistic, reproducible, seed-based loadout
  generator (ChaCha8 or StdRng — pick one) into a shared location both the corpus
  builder and the trainer call. Realistic loadouts matter: uniform-random loadouts
  produce unrealistic games, so a net tuned on them mis-learns.
- Reproducibility invariant (kept): a corpus / self-play set built from seed N is
  always regenerable bit-exact.

### 5.4 Weight storage + versioning (kept)

- Each accepted version: a binary weight blob (`raters/vNNNN.bin`) + a JSON sidecar
  (lineage/parent id, training step or mutation history, win-rate vs. predecessor
  at 100 ms, quantization params, git SHA, date).
- `raters/index.json` lists accepted versions in order — the gauntlet membership /
  champion history. (Single track now, so no per-track tags.)
- Committed to repo (KB-scale blobs, no LFS). WASM build embeds the current champion
  via `include_bytes!`; no runtime fetch.

### 5.5 IPC (kept, simplified)

Trainer is the source of truth; UI is a passive observer that **never** throttles
training. Trainer writes summary snapshots at a low cadence (~1 Hz) regardless of
UI; writes live-position state per ply only while the UI signals it is subscribed
(sentinel flag). UI polls on its own timer and cannot pause/cancel/interfere.
Concrete paths/format decided during implementation.

---

## 6. UI — Training Observatory (build AFTER the backend works)

*(Decision, Session 48.)* The UI is **deferred until the training backend is
working and validated.** It is opt-in observability: default = headless training at
full speed; open the page = subscribe to one live game + summary stats; close =
unsubscribe, full speed resumes. **UI must never throttle training.**

Requirements when built:
- Show **every available version** (champion history / lineage) and **all active
  runs** at a glance — the two things Elias explicitly wants visible.
- Live match view (reuse the existing board renderer), current self-play game +
  eval bars (candidate NN, opponent NN, and the hand-crafted eval as a control).
- Version/lineage list with per-version metadata (win-rate vs. predecessor at
  100 ms, mutation magnitude, date).
- Network inspector (weight heatmap, live forward pass, per-square Δscore overlay)
  — nice-to-have, not required for v1 of the UI.

Panels beyond "versions + active runs + one live game" are optional polish. The
single hard constraint is **passive observation that cannot slow training**.

---

## 7. Risks & honest caveats

- **The payoff is strength, not speed.** If Phase 0 shows the accumulator eval is,
  say, 3× the hand-crafted eval's cost, the search does ~3× less work per second —
  which is only a net win if the net is enough *smarter* to reach equal decisions
  at shallower depth. That's a bet on trainability, validated only in Phase 1's
  gauntlet. Phase 0 proves *feasibility*, not *superiority*.
- **Mutation-only self-play is a gamble on luck.** Large randomized changes with a
  best-of-three filter can improve, but with scarce compute it can also stall for
  long stretches (most big jumps are worse). Phase 0's bootstrap is what makes this
  tolerable — we start from a competent net, so we're refining, not searching from
  noise. If mutation stalls badly, revisit adding gradient descent back into
  Phase 1 (the retired plan's approach) rather than treating mutation as sacred.
- **Quantization drift is a real correctness surface.** The int path must be
  graded, not assumed. A net that regresses the eval well in f32 but poorly after
  quantization is a Phase-0 failure, not a Phase-1 problem.
- **Accumulator/refresh mismatch is the ns-49 trap again.** The bit-identity
  playout test is non-negotiable and must run on every node in test builds.
- **Scope discipline.** Multi-week build. Phase 0 alone (encoder + accumulator +
  quantized inference + regression harness) is the honest first deliverable; do not
  start the gauntlet-simplification or UI until Phase 0's milestone is met.

---

## 8. Cross-references

- `alpha-beta-optimisation-catalogue.md` Phase 4 — the Session-48 measurement (NN
  ~382× the hand-crafted eval; why incremental doesn't work for the hand-crafted
  eval but DOES for NNUE features). The evidence behind the architecture decision.
- `crates/nn_trainer/` — existing NN crate, built to the retired plan. This rework
  re-points it: dense `encoding.rs` (INPUT_DIM=2825) → sparse + accumulator;
  three-track `gauntlet.rs` → single fast track; gradient self-play (`train.rs`)
  → mutation self-play; `loadout.rs` uniform generator → unified with the corpus
  builder's realistic one. `selfplay.rs` / `lineage.rs` / `snapshot.rs` /
  `persistence` / `lineage_checkpoint.rs` scaffolding is reusable.
- `core_engine/examples/build_corpus.rs` — the corpus builder + its realistic
  `random_loadout`; the shared generator target for §5.3.
- `core_engine/src/search/evaluator/` — hand-crafted eval; `evaluate_scalar` is the
  Phase-0 regression target and the ns/call yardstick. Header comment carries the
  load-bearing eval philosophy.
- `core_engine/src/game_logic/make_unmake.rs` — where the accumulator update hooks
  into make/unmake.
- `oq-81` — search "less, smarter" track, parallel to this eval-strength track.
- `next_steps id=25` — the original NN-rater idea seed (from the retired plan).
- `adr-005` — digital architecture (Rust core + Svelte/Tauri + P2P MP); host
  environment for the trainer + UI.

## 9. Sources

- [NNUE — chessprogramming wiki](https://www.chessprogramming.org/NNUE)
- [Efficiently Updatable Neural Network (original Shogi NNUE)](https://www.chessprogramming.org/NNUE#History)
- [Stockfish NNUE architecture docs](https://github.com/official-stockfish/nnue-pytorch/blob/master/docs/nnue.md)
