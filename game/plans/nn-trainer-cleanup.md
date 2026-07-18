# NN-Trainer Cleanup - pre-GPU work

Eight tasks identified after the first real launch of the Training Observatory
(session 38). Goal: take the NN-training subsystem from "smoke-test that runs"
to "system that produces meaningful lineages from a long run," then prep the
backend type seams for a GPU port.

Recommended order: **1, 2, 3, 5, then 4 (GPU prep), then a real run on the
GPU box, then 6 + 7.** Task 8 is a nice-to-have that can land anywhere after 4.

Each section below was expanded by a planning agent with concrete file paths,
line numbers, and design decisions. Treat them as implementation briefs, not
final designs - open questions and trade-offs are flagged.

---

## 1. Stop the run when the user closes the window

**Why first**: hit twice already in session 38. Without it, every "exit and
restart" leaks CPU and triggers the next launch's "training already running"
error, requiring `kill -9` to recover. It's also the only task that prevents
the developer from iterating quickly - every other item assumes a clean
shutdown story.

**What we know**:
- `TrainingState` (`game/crates/tauri_wrapper/src/lib.rs:432-441`) already
  holds the `Arc<AtomicBool>` stop flag and a `JoinHandle<()>` for the
  orchestrator thread. The plumbing exists; nothing is wired to the
  window/app lifecycle.
- `start_training_run` (`lib.rs:611-632`) clones the `Arc` into the spawned
  thread and stores the original in `TrainingState.inner.stop`.
- `stop_training_run` (`lib.rs:634-646`) sets the flag, then drops both
  `stop` and `handle` from the inner state *without* joining. The comment is
  explicit: "blocking the IPC thread is rude. The orchestrator writes a
  final phase=Idle snapshot on the way out."
- The orchestrator (`game/crates/nn_trainer/src/run.rs:151-371`) checks
  `should_stop.load(Ordering::Relaxed)` at four points: top of the
  generation loop (`run.rs:169`), after `train_lineages` (`run.rs:237`),
  inside the per-candidate Tier-1 loop (`run.rs:263`), and via the early
  `None` return from `run_tier2_with_live` (`run.rs:319-322`). Latency is
  bounded by one full Tier-1 BO3 (worst case), which at smoke-test scale is
  seconds and at production scale could be tens of seconds.
- The orchestrator always writes a final `StatusSnapshot::idle()`
  (`run.rs:369`) before returning - even on stop-flag exit - so the on-disk
  `status.json` ends up clean regardless of how we exit.
- Tauri v2 exposes both `WindowEvent::CloseRequested` (per-window, fires on
  red-light / Cmd+W) and `RunEvent::ExitRequested` (app-level, fires on
  Cmd+Q before the runtime actually tears down). Both run on the main
  thread; either can call `.prevent_exit()` / `api.prevent_close()` if we
  want to block.
- `tauri::Builder::on_window_event` and `tauri::App::run(|handle, event|
  ...)` are the two registration points. The latter is invoked from inside
  `run()` at `lib.rs:653-692`.

**Design decisions**:

1. **Signal-and-let-go, not block-until-drain.** Blocking the close until
   the orchestrator winds down would freeze the OS-level "Quit" affordance
   for up to a full BO3, which feels broken (the user will Cmd+Q again,
   then `kill -9`, which is exactly the bug we're fixing). The orchestrator
   already exits cleanly on the stop flag and writes a final idle snapshot.
   We signal and exit immediately.
2. **Hook `RunEvent::ExitRequested`, not `WindowEvent::CloseRequested`.**
   Cmd+Q on macOS bypasses the window-close event and goes straight to
   app-exit; conversely, the red-light on macOS just hides the window - we
   *don't* want to stop training when the user accidentally closes the
   window.
3. **No `prevent_exit()`.** Consistent with (1). The stop flag is set
   synchronously before the runtime's exit path runs, so the orchestrator
   thread observes it on its next checkpoint.
4. **Tolerate "no run in progress".** The hook fires on every quit. The
   handler reads `TrainingState` and is a no-op if `inner.stop` is `None`.
5. **Accept the worst-case latency (~one BO3) for v1.** Tightening it
   requires plumbing the stop flag deeper - into
   `play_match_with_callback` / `mirrored_bo3_live` so the per-ply callback
   at `run.rs:520-522` can short-circuit. Strictly larger change; file a
   follow-up if production runs make per-BO3 latency painful.

**Implementation sketch**:

Factor flag-setting out of `stop_training_run` into a shared helper:

```rust
fn signal_stop(state: &TrainingState) {
    let mut inner = state.inner.lock().unwrap();
    if let Some(flag) = inner.stop.as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    inner.stop = None;
    inner.handle = None;
}

#[tauri::command]
fn stop_training_run(state: State<'_, TrainingState>) -> Result<(), String> {
    signal_stop(&state);
    Ok(())
}
```

Wire the runtime hook inside `run()` at `lib.rs:653-692` by swapping the
convenience `.run(...)` for `.build(...).run(closure)`:

```rust
let app = tauri::Builder::default()
    .manage(EngineRegistry::default())
    .manage(TrainingState::default())
    .invoke_handler(tauri::generate_handler![ /* unchanged */ ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

app.run(|app_handle, event| {
    if let tauri::RunEvent::ExitRequested { .. } = event {
        let state = app_handle.state::<TrainingState>();
        signal_stop(&state);
    }
});
```

**Test strategy**:
- Unit: extract `signal_stop` body; test that calling it on a `TrainingState`
  with a populated `inner.stop` sets the flag true (verified via a cloned
  `Arc`) and clears `stop`/`handle`. Mirror at `lib.rs:698-928` test module.
- Unit (no-op): `signal_stop` on a `TrainingState` with `inner.stop = None`
  must not panic.
- Manual / integration: start a run, Cmd+Q, relaunch, confirm
  `start_training_run` no longer errors with "training already running".

**Effort**: 30 min. Real risk is the `.build().run(closure)` form vs `.run()`
convenience wrapper - one compile cycle to confirm the `RunEvent` import path.

---

## 2. Tier-2 plays against accepted predecessors, not just the heuristic

**Why second**: this is the gauntlet's whole point. Until it lands,
"accepted" is meaningless past generation 1 - every candidate from gen 2
onwards is judged against the same trivial heuristic the gen-1 winner
already beat, so the lineage cannot meaningfully improve. Task 1 is a 30-min
unblock; this is the first task that changes what the system actually
*does*. It must precede tasks 3/5 because there is no point letting users
crank `n_generations=20` on the GPU if the gauntlet itself is a no-op.

**What we know**:
- `RaterIndex::entries` (in `registry.rs`) is the authoritative list of
  accepted raters in acceptance order, each carrying a `stem: PathBuf`
  relative to the raters directory. `RaterIndex::latest()` returns the
  immediate predecessor.
- `IndexEntry` already has a `parent_id: Option<String>` field, populated
  at `run.rs:339`, but nothing reads it back yet.
- `persistence::load_rater::<InferenceBackend>(stem, &device)` returns
  `(Mlp<InferenceBackend>, RaterMetadata)`. The stem expected is *absolute*
  (or relative to CWD) - the caller has to do
  `raters_dir(run_dir).join(&entry.stem)` itself.
- `NnEvaluator::new(model)` wraps an `Mlp<InferenceBackend>` into an
  `Evaluator` implementor. `evaluate_fen_at_stem` exists but reloads from
  disk on every call - wrong granularity for the gauntlet. We want one
  load per predecessor per generation.
- `tier2_acceptance(&dyn Evaluator, &[&dyn Evaluator], u64) ->
  AcceptanceReport` already accepts an arbitrary predecessor list. So does
  the live-aware sibling `run_tier2_with_live` in `run.rs:414`. No API
  change needed downstream.
- The "loading rater blobs would be expensive" comment is wrong. A single
  forward pass is microseconds. Each Tier-2 BO3 plays 2-3 full games with
  depth-N search, so the search dominates by orders of magnitude - load
  cost is in the noise.
- Bootstrap (empty index) still needs the heuristic - the existing
  fallback is correct for gen 1.

**Design decisions**:

1. **Load all predecessor blobs up front, once per generation, not once
   per match.** Loading happens off the hot path; an MLP at the current
   topology is tens of KB, so holding 100 raters in memory is single-digit
   MB. Streaming would re-deserialise each blob inside the gauntlet loop.
2. **Cap predecessor count at `MAX_PREDECESSORS = 16`** (configurable via
   `RunConfig` later). Take the *most recent* 16 - they're the strongest
   opponents and the ones we most care about not regressing against. Cost
   is `O(N)` per generation: at 20 generations × 20 predecessors × 3
   brackets × 3 games that's 3600 games/gen, too much.
3. **Parent resolution: candidate must beat the *immediate predecessor*
   AND meet the non-regression bar against every other predecessor in the
   window.** Same semantics `tier2_acceptance` already encodes -
   `predecessors[last]` is the immediate predecessor with the 45 % bar
   checked against all others.
4. **Lifetime structure: own the evaluators in a local
   `Vec<NnEvaluator>`, then borrow into `Vec<&dyn Evaluator>` for the
   call.** Two-vector pattern. For the heuristic-bootstrap path, allocate
   one stack-local `HeuristicEvaluator`.

**Implementation sketch**:

Helper near the top of `run.rs`:

```rust
const MAX_PREDECESSORS: usize = 16;

fn load_predecessor_evaluators(
    index: &RaterIndex,
    raters_dir: &Path,
) -> Vec<NnEvaluator> {
    let device: <InferenceBackend as burn::tensor::backend::Backend>::Device =
        Default::default();
    let take_from = index.entries.len().saturating_sub(MAX_PREDECESSORS);
    let window = &index.entries[take_from..];
    let mut owned = Vec::with_capacity(window.len());
    for entry in window {
        let stem = raters_dir.join(&entry.stem);
        match crate::persistence::load_rater::<InferenceBackend>(&stem, &device) {
            Ok((model, _meta)) => owned.push(NnEvaluator::new(model)),
            Err(e) => eprintln!("nn_trainer: skipping predecessor {}: {}", entry.id, e),
        }
    }
    owned
}
```

At the Tier-2 call site (lines 296-302), replace the `vec![&baseline]`:

```rust
let owned: Vec<NnEvaluator> = load_predecessor_evaluators(&index, &raters_dir(run_dir));
let heuristic_fallback = HeuristicEvaluator;
let predecessors: Vec<&dyn Evaluator> = if owned.is_empty() {
    vec![&heuristic_fallback]
} else {
    owned.iter().map(|e| e as &dyn Evaluator).collect()
};
```

Delete the misleading "loading rater blobs would be expensive" comment.

**Test strategy**:
1. Unit test for `load_predecessor_evaluators`: save two raters via
   `save_rater`, build a `RaterIndex` with two entries, call the helper,
   assert `len() == 2` and `evaluate` returns finite i32.
2. Cap test: 20 entries → helper returns 16, last entry is most recent.
3. Corruption-skip: one valid + one malformed sidecar → returns 1 + logs.
4. Integration extension to the smoke test: bump to 2 generations with a
   contrived config that's almost certain to accept; assert gen-2 Tier-2's
   `per_predecessor.len() == 1`.
5. Manual: 3-generation run, watch `matrix.json` accumulate cells keyed on
   `pred-0`, `pred-1`, … rather than only `pred-0`.

**Effort**: 1-2 hr.

---

## 3. `RunConfig` plumbed from the UI to `start_training_run`

**Why third**: today `start_training_run` hard-codes `RunConfig::default()`
(2 gen / 4 lineage / 4 corpus / depth-2). Every knob that determines
whether a run is meaningful is baked in at compile time. The whole point of
tasks 1+2 is to make a long run cancellable and substantive; without task 3
you still need `cargo build --release` per parameter sweep. The smallest
unblock for the first real GPU session.

**What we know**:
- `RunConfig` (`run.rs:69-96`), `LineageConfig` (`lineage.rs:188`),
  `TrainingConfig` (`train.rs:29`) are all `#[derive(Clone, Debug)]` only.
  **No serde.**
- `TrainingConfigSnapshot` (a flat serde mirror) already lives in
  `persistence.rs:115` with a `From<&TrainingConfig>` impl. **That is the
  cross-boundary precedent - do not put serde on the live structs.**
- `MlpConfig` uses burn's `#[derive(Config, Debug)]` macro which implements
  `Serialize`/`Deserialize`. Already wire-ready.
- `nn_trainer` already depends on `serde` and `serde_json`.
- `tauri_wrapper/src/lib.rs` already uses `#[serde(rename_all =
  "camelCase")]` DTOs (e.g. `WeightStats`, `RaterInspection`).
- `run_training` does *not* validate `RunConfig` - bad input today crashes
  the orchestrator thread silently (the spawned thread's panic is
  swallowed).
- Per-rater metadata already contains `model_config: MlpConfig` and
  `training_config: TrainingConfigSnapshot` - half of the "persist run
  config" pattern is already in the code.

**Design decisions**:

1. **IPC arg vs `<run_dir>/config.json` - do both, IPC authoritative.**
   The IPC payload carries the full config every call. On run start, the
   orchestrator writes a verbatim copy to `<run_dir>/config.json`. The
   on-disk copy is the audit trail and prefills the UI form when reopening
   an old run dir.
2. **Validation at the IPC boundary, not inside `run_training`.**
   `run_training` keeps assuming well-formed configs (matches its current
   contract). The IPC command runs `validate_run_config(&cfg) ->
   Result<(), String>` and surfaces errors via the existing `startError`
   path.
3. **UI shape: presets first, JSON textarea behind a disclosure, no
   per-field form yet.** Three preset buttons (Smoke / Medium / Long), a
   one-line summary, and a collapsed "Advanced (JSON)" textarea for
   overrides. Presets cover ~95% of the workflow; per-field inputs can
   graduate later.
4. **Flat `RunConfigDto` over exposing nested config directly.** New DTO
   in `tauri_wrapper/src/lib.rs` with `#[serde(rename_all = "camelCase")]`
   that mirrors `RunConfig`'s shape but composes from `MlpConfig` (already
   serde) and `TrainingConfigSnapshot` (already serde). `From`/`Into`
   conversions to/from `nn_trainer::RunConfig`. Keeps the wire format
   decoupled from the live structs.
5. **`seed_root: u64` quirk:** JS can't safely round-trip integers > 2^53.
   Recommend serialising as a hex string via `#[serde(with = "hex_u64")]`,
   or constrain to `u32` at the DTO layer with a `wrapping_into()`.

**Validation rules** (minimum): `n_generations ∈ [1, 1000]`, `corpus_games
∈ [1, 10_000]`, `corpus_max_depth ∈ [1, 8]`, `n_lineages ∈ [1, 64]`,
`n_rounds ≥ 1`, `steps_per_burst ≥ 1`, `perturb_std` finite and `> 0`,
`learning_rate` finite and `> 0`, `batch_size ≥ 1`, `hidden_sizes`
non-empty with all entries `≥ 1`, total params `< 50M`.

**Recommended Long preset** (first GPU session):
- `n_generations: 10`, `corpus_games: 64`, `corpus_max_depth: 4`
- `lineage.n_lineages: 8`, `n_rounds: 10`, `steps_per_burst: 100`,
  `steps_per_candidate: 50`, `perturb_std: 0.03`
- `training.learning_rate: 1e-3`, `batch_size: 128`, `epochs: 5`
- `model.hidden_sizes: [256, 64, 32]`

Medium ≈ half of these. Smoke = current `RunConfig::default()`.

**Implementation sketch**:

1. In `nn_trainer/src/lib.rs`: re-export `TrainingConfigSnapshot` from
   crate root.
2. In `tauri_wrapper/src/lib.rs`:
   - Add `RunConfigDto` + `LineageConfigDto` + `From`/`Into`.
   - Add `validate_run_config(cfg: &RunConfigDto) -> Result<(), String>`.
   - Modify `start_training_run` to accept `config: RunConfigDto`;
     validate, write `<run_dir>/config.json`, convert, spawn.
   - Add `read_run_config(run_dir) -> Result<RunConfigDto, String>` for UI
     prefill.
3. In `frontend/src/lib/training/runConfig.ts`: type defs, preset
   constants, JS-side validator (UX nicety; Rust validator is
   authoritative).
4. In `frontend/src/routes/training/+page.svelte`: replace
   `runRequested` start path with preset/textarea-driven config; add a
   "Run Config" sub-section with three preset buttons + collapsible
   textarea. On mount, call `read_run_config` to prefill.

**Test strategy**:
- Rust: `RunConfigDto` ↔ `RunConfig` round-trips losslessly;
  `validate_run_config` rejects each rule (one test per rule); presets all
  validate; serde JSON round-trip preserves equality.
- Integration: parameterised version of the smoke test loads a tiny DTO
  from a JSON fixture, asserts `<run_dir>/config.json` is written by the
  wrapper (not by `run_training` - confirms the layering).
- Manual: each preset → confirm `config.json` reflects values, status
  shows correct generation count, edit textarea → re-Start → values
  updated.

**Effort**: 1.5-2 hr. Cut to 1 hr if you punt the JSON textarea ("presets
only" v1).

---

## 4. Backend type aliases become feature-gated `pub type`s

**Why fourth**: tasks 1–3 stabilise the orchestrator API; task 5 changes
depth without touching types. Doing the backend-aliasing sweep *after*
those means we sweep once, not twice, and we can validate the alias on a
config-driven real run instead of the smoke-test defaults. Must land
*before* the GPU box session because every hardcoded `NdArray<f32>` is a
recompile-edit otherwise.

**What we know** (re-grep'd from current tree):

| File | Line | Hardcoded type |
|------|------|----------------|
| `run.rs` | 64 | `type AutodiffB = Autodiff<NdArray<f32>>;` |
| `nn_evaluator.rs` | 40 | `pub type InferenceBackend = NdArray<f32>;` |
| `lib.rs` (tests) | 70, 74 | `type B = NdArray<f32>;` |
| `lineage.rs` (tests) | 326, 329 | `type B = Autodiff<NdArray<f32>>;` |
| `model.rs` (tests) | 163, 166 | `type B = NdArray<f32>;` |
| `persistence.rs` (tests) | 228, 232 | `type B = NdArray<f32>;` |
| `train.rs` (tests) | 129, 132, 180, 185 | mixed Autodiff/Ndarray |

Production code already speaks `B: AutodiffBackend` or `B: Backend`. Only
two concrete-type sites worth aliasing; everything else is a mechanical
test-side `use` swap.

Current `Cargo.toml`:
```toml
burn = { version = "0.21", default-features = false, features = ["ndarray", "train", "std"] }
```

`.mpk` persistence should round-trip across backends because burn's
recorder is parameterised by precision settings, not backend - but **worth
a one-shot save-on-A / load-on-B test once Wgpu compiles.**

`Send`/`Sync` situation: `Autodiff<NdArray<f32>>` is not fully `Send`
today (note in `lineage.rs:244-248`). Burn 0.21's `Wgpu` and `Cuda`
backends *are* `Send + Sync` (`Arc`-backed device buffers). Rayon-parallel
lineages on GPU become plausible and likely a bigger win than per-op
throughput.

**Design decisions**:

1. **Three explicit features (`backend-ndarray`, `backend-wgpu`,
   `backend-cuda`) over one `gpu` umbrella.** Single-`gpu` is ambiguous on
   a machine with both Metal and Cuda; explicit features make the build
   matrix legible; future-proofs for LibTorch / Candle.
2. **Default = `backend-ndarray`.** Keeps `cargo build` working from a
   fresh clone without GPU toolchains; CI on hosted Linux runners keeps
   working.
3. **Features mutually exclusive, enforced with `compile_error!`.**
   Violates Cargo's "features are additive" guideline in spirit, but a
   single `TrainingBackend` `pub type` cannot resolve to two concrete
   types. Document in ADR.
4. **New `backend.rs` module** centralises the two aliases plus a
   `default_device()` constructor.

**Implementation sketch**:

`Cargo.toml`:
```toml
[features]
default = ["backend-ndarray"]
backend-ndarray = ["burn/ndarray"]
backend-wgpu    = ["burn/wgpu"]
backend-cuda    = ["burn/cuda"]

[dependencies]
burn = { version = "0.21", default-features = false, features = ["train", "std"] }
```

New file `nn_trainer/src/backend.rs`:
```rust
#[cfg(all(feature = "backend-ndarray", feature = "backend-wgpu"))]
compile_error!("nn_trainer: features `backend-ndarray` and `backend-wgpu` are mutually exclusive");
#[cfg(all(feature = "backend-ndarray", feature = "backend-cuda"))]
compile_error!("nn_trainer: features `backend-ndarray` and `backend-cuda` are mutually exclusive");
#[cfg(all(feature = "backend-wgpu", feature = "backend-cuda"))]
compile_error!("nn_trainer: features `backend-wgpu` and `backend-cuda` are mutually exclusive");
#[cfg(not(any(feature = "backend-ndarray", feature = "backend-wgpu", feature = "backend-cuda")))]
compile_error!("nn_trainer: one of `backend-ndarray`, `backend-wgpu`, `backend-cuda` must be enabled");

#[cfg(feature = "backend-ndarray")]
mod inner {
    use burn::backend::{Autodiff, NdArray};
    pub type InferenceBackend = NdArray<f32>;
    pub type TrainingBackend  = Autodiff<NdArray<f32>>;
    pub fn default_device() -> <InferenceBackend as burn::tensor::backend::Backend>::Device { Default::default() }
}
// (analogous mod inner for backend-wgpu and backend-cuda)

pub use inner::{default_device, InferenceBackend, TrainingBackend};
```

Sweep call sites:
- `nn_evaluator.rs`: drop `use burn::backend::NdArray;`; replace
  `InferenceBackend` with `pub use crate::backend::InferenceBackend;`.
- `run.rs`: drop `use burn::backend::{Autodiff, NdArray};` + local
  `AutodiffB`; add `use crate::backend::TrainingBackend as AutodiffB;`.
- Tests in `lib.rs:74`, `lineage.rs:329`, `model.rs:166`,
  `persistence.rs:232`, `train.rs:132`: replace
  `use burn::backend::NdArray;` with `use crate::backend::{InferenceBackend,
  TrainingBackend};`, swap `type B` defs accordingly.

**Test strategy**:
1. Per-backend `cargo build` smoke in CI matrix.
2. Existing tests run on every backend.
3. Mutual-exclusion guard: `cargo build --features "backend-ndarray
   backend-wgpu"` must fail with the `compile_error!` (CI expect-fail).
4. **Cross-backend persistence round-trip** (new). Save with `TrainingBackend
   = Autodiff<Wgpu>`, load with `NdArray<f32>` via dev-dependencies
   feature, probe one position on each side, assert outputs within epsilon.
5. `into_inference` keeps working - existing `train.rs` test covers this.

**CI matrix** (add when CI exists):
```yaml
matrix:
  backend: [backend-ndarray, backend-wgpu, backend-cuda]
  os: [ubuntu-latest, macos-latest]
  exclude:
    - { backend: backend-cuda, os: macos-latest }
    - { backend: backend-wgpu, os: ubuntu-latest } # no adapter
```
Cuda = `cargo check`-only in CI; functional verification on GPU box.

**Effort**: 30 min for mechanical aliasing + 30-60 min once `backend-wgpu`
is on (the `Wgpu<f32, i32>` generic shape will expose any place we
accidentally pinned types) + 15 min Cargo features + 30 min CI matrix.
**Total realistic: 1.5-2 hr** including cross-backend round-trip test.

**Companion ADR-008** ("Three explicit backend features over one umbrella
`gpu`"): captures decisions 1-3 + additive-features-violation rationale,
default rationale, the fact that `tauri_wrapper` doesn't re-export the
features (downstream selection at workspace root via
`--features nn_trainer/backend-wgpu`), and the open follow-up of runtime
backend selection (a `dyn Backend` design, not feature-flag).

**Send/Sync follow-up** (out of scope for this task but worth flagging):
once `backend-wgpu` lands, the comment block in `lineage.rs:244-248` is no
longer accurate - `train_lineages` can become `par_iter_mut`-based with
rayon. **Bench first, parallelise second**: an MLP this size is
dispatch-bound on Wgpu, but four concurrent training bursts saturating the
device queue should recover useful throughput.

---

## 5. Self-play depth is configurable + defaults higher

**Why fifth**: the corpus is the foundation of the lineage layer - every
accepted rater is downstream of one. Depth-2 self-play produces
near-random labels (the heuristic at depth-2 plays badly enough that game
outcomes barely correlate with positional strength). Without a deeper
default, training nine generations on top of depth-2 noise wastes GPU
time. Lands fifth because it only matters once (3) gives the UI a way to
set the knob.

**What we know**:

Two distinct depth use-sites:
1. **Corpus generation** - `selfplay::play_game` (called by
   `batch::generate_corpus`, called by `run::build_corpus`). Uses
   `find_best_with_evaluator` with `time_limit_ms = 0` and `max_depth =
   config.corpus_max_depth`. Fixed-depth: reproducibility per
   `selfplay.rs:84-86`. Parallel across games via rayon (`batch.rs:42`).
   Default today: `2`.
2. **Gauntlet play** - `gauntlet::play_match`. Uses time-bounded search
   (`bracket.time_limit_ms()` of 100/300/500ms, with
   `TIME_BOUNDED_MAX_DEPTH = 64` cap). Wall-clock-bounded by design - the
   three brackets are the whole point of plan §5.

So the two sites have different cost/quality contracts: corpus is
**depth-bounded** (reproducibility wins over wall-clock parity); gauntlet
is **time-bounded** (consistent thinking time wins over node-count
parity).

Session 36 killer/history landed in `core_engine` (-19.5 % wall-clock at
depth 6). QS is also already merged (`alpha_beta.rs:201-205`, default-on,
kill-switch only used by the `qs_match` example for A/B grading).

**Design decisions**:

1. **Two depth knobs, not one.** Forcing one knob (e.g. "depth 6" applied
   to both) would either degrade the gauntlet (bracket semantics are
   load-bearing) or distort the corpus. Recommendation:
   - Rename `corpus_max_depth: u8` → `corpus_search_depth`.
   - Add `gauntlet_time_per_ply_ms: Option<[u64; 3]>` (Fast/Medium/Slow).
     `None` → defaults `[100, 300, 500]`.
   - Optional `gauntlet_max_depth_cap: u8` (default 64).
2. **QS stays always-on, do not expose.** QS makes static eval at
   depth-1/2 nodes reliable (session 36 finding). That argues *for*
   relying on QS at the depth-2 smoke-test default. If someone later wants
   to A/B QS for corpus generation specifically, lift `DISABLE_QS` to a
   thread-local - don't add it to `RunConfig`.
3. **Two `RunConfig` constructors.** `Default` stays as is (depth-2, 4
   games, 2 generations) for the smoke test and CI. Introduce
   `RunConfig::real_run() -> Self` with the depth-6 / 10-generation /
   8-lineage shape for GPU sessions.

**Recommended defaults for `real_run()`** (subject to benchmark below):
- `corpus_search_depth: 6`, `corpus_games: 64`
- `n_generations: 10`, `lineage.n_lineages: 8`, `n_rounds: 5`,
  `steps_per_burst: 50`
- Gauntlet times unchanged from plan §5.

**Implementation sketch**:

1. Rename + extend `RunConfig` in `run.rs:69-96` (corpus_search_depth,
   gauntlet_time_per_ply_ms, gauntlet_max_depth_cap, add serde derives -
   required by task 3 anyway).
2. Add `impl RunConfig { pub fn real_run() -> Self { ... } }`. Keep
   `Default` as is.
3. Thread `gauntlet_time_per_ply_ms` through to `Bracket` via a new
   `BracketBudget { fast, medium, slow, max_depth_cap }` struct passed
   into `play_match` / `mirrored_bo3*`. Keeps `Bracket` as a pure tag -
   smaller blast radius than refactoring `Bracket` to carry ms inline.
4. `build_corpus` reads `config.corpus_search_depth`. No structural change.
5. No changes to `core_engine`.

**Test strategy**:
1. Determinism regression: `generate_corpus(..., depth=4)` called twice
   returns identical labels. Existing test at `batch.rs:76-85` covers
   depth-2; add a depth-4 variant.
2. `real_run()` shape test: assert depth ≥ 4 and generations ≥ 4.
3. `BracketBudget` wiring: custom budget `{fast: 50, ...}` makes wall-clock
   roughly proportional (loose bound - 5× difference is outside flake).
4. Smoke test unchanged - must continue to complete in seconds.

**Benchmark plan: depth-4 vs depth-6 wall-clock**

We have `crates/search_bench` for single-position depth-bounded
measurements. The question for (5) is "what does corpus generation cost
end-to-end" - a *game-level* number.

Procedure (one-off measurement, not CI):
1. Add `--mode game-corpus` to `search_bench` that calls
   `nn_trainer::batch::generate_corpus` with `n_games ∈ {16, 64}`,
   `max_depth ∈ {2, 4, 5, 6, 7}`, fixed seed. Reports wall-clock, total
   plies, total positions, per-ply mean ms. Single-thread and rayon.
   Persists to `game/bench/results/corpus-depth-sweep.json`.
2. Run on dev box (current baseline) and GPU box (CPU dominates corpus
   generation).
3. Decision rule: pick smallest depth `d ≥ 4` where total wall-clock for
   `corpus_games=64, n_generations=10` is ≤ 30 minutes on the target box.
4. Also measure game-length-vs-depth: deeper search may produce shorter,
   more decisive games - partially offsets per-ply cost.
5. **Label-quality probe (stretch)** - generate corpora at d2/d4/d6,
   train one lineage on each (same seed/topology), run Tier-1 against
   `HeuristicEvaluator`. Higher-depth corpora should produce higher
   win-rates. If d6 isn't stronger than d4, default to d4.

**Effort**: ~3 hr active work plus benchmark runtime. Cheap, but blocked
on (3) for the UI to set the new fields.

---

## 6. `EVAL_SCALE` calibration

**Why sixth**: gated on real data. The whole point of calibration is to
map a *converged* rater's `[-1, +1]` output onto centipawn-scale so it
composes with `HeuristicEvaluator`. Until tasks 3 (`RunConfig` plumbed)
and 5 (depth-≥4 self-play) actually produce useful raters, every fitted
scalar is curve-fitting against undertrained noise. Ship the
*infrastructure* once task 4 has produced trainable raters; defer the
*blessed value* until a real lineage exists.

Calibration also unblocks task 7 (NN into the AI engine). Once a player
picks "NN" or "Heuristic", the two outputs MUST live in the same units. A
miscalibrated `EVAL_SCALE` either underweights the NN (always loses ties)
or overweights it (every NN hint looks like a near-mate). Both fail
silently in alpha-beta.

**What we know**:
- Training labels are `{-1, +1}` (P1/P2 outcomes from `selfplay.rs`). A
  converged final-layer activation (tanh-like) saturates near those ends.
- `EVAL_SCALE = 3000.0` lives at `nn_evaluator.rs:43`, re-exported from
  `lib.rs:43`. The doc-comment at lines 14-21 explicitly defers final
  tuning to "the gauntlet."
- `HeuristicEvaluator` scoring range: `MATE_SCORE = 1_000_000` (terminal),
  `CHAMPION_VALUE = 1000`, `GUARD_VALUE = 600`, `HP_PER_POINT = 150`,
  `ARMOR_PER_POINT = 120`. Realistic mid-game `|total|` lands roughly in
  `[-3000, +3000]`. Three Champions of headroom (the current 3000) is
  plausible *only* if "definitely winning" maps to "all three Champions
  vs zero" - a guess until measured.
- `MAX_NN_SCORE = MATE_SCORE - 1` clamping leaves three orders of
  magnitude of unused dynamic range. The floor on `EVAL_SCALE` is "tall
  enough that NN preferences outvote heuristic noise," not the ceiling.
- `RaterMetadata` (`persistence.rs:67-109`) is explicitly designed
  append-only - adding `#[serde(default)] eval_scale: f32` is a
  non-breaking schema bump.
- `LabelledPosition` corpus already exists per generation and is the
  obvious probe set (FEN-serialisable, statistically representative).
- Burn's `forward` at inference is autograd-free (~5k FENs well under a
  second).

**Design decisions**:

1. **Calibration metric: slope-only linear regression** of NN output →
   heuristic output. Reject match-equity (alpha-beta only cares about
   *order* within wide bands - same outcome for many `k`; expensive). Reject
   `mean(|delta|)` (rewards a rater that has *learned the heuristic*, not
   one that has learned the game). Slope-only OLS (`y ≈ k·x`, force `b=0`)
   weighted toward large-magnitude points naturally ignores positions
   where the NN sees something subtle the heuristic misses. A non-zero
   intercept would mean a sign-symmetric model bug - fix it, don't paper
   over.
2. **Per-rater storage, not global constant.** Different topologies
   saturate differently; undertrained raters need a larger scale. Add
   `eval_scale: f32` to `RaterMetadata` with `#[serde(default)]`. The
   constant `DEFAULT_EVAL_SCALE = 3000.0` survives as the bootstrap
   default for old/uncalibrated raters.
3. **Probe set source: current generation's self-play corpus**, with a
   held-out 10% slice for paranoia. Statistically aligned with the
   rater's training distribution; zero work since the corpus already
   exists.
4. **When does calibration run?** At rater-acceptance time, inside
   `run_training` between Tier-2 acceptance and `save_rater`. The rater
   has just been judged worthy and the corpus is in scope.

**Implementation sketch**:

New module `nn_trainer/src/calibration.rs`:
- `pub struct CalibrationReport { scale: f32, n_probes: usize,
  median_abs_residual: f32, r_squared: f32 }`
- `pub fn calibrate_rater<B: Backend>(model, device, probes: &[Position])
  -> CalibrationReport`:
  - For each pos: skip terminals; `x_i = forward(model, pos)`, `y_i =
    heuristic.evaluate(pos) as f32`.
  - `k = Σ(x_i · y_i) / Σ(x_i²)`. Guard `Σ(x_i²) > ε`.
  - Report `median(|r_i|)` and `r²`.
- `calibrate_from_corpus`, `calibrate_fens` convenience wrappers.

Schema bump in `persistence.rs`: `#[serde(default)] pub eval_scale: f32`
in `RaterMetadata` (0.0 sentinel = not yet calibrated). No
`RATER_FORMAT_VERSION` increment needed.

Refactor `nn_evaluator.rs`: `NnEvaluator` grows a `scale: f32` field;
`with_scale(model, scale)` constructor; `output_to_centipawns` reads
`self.scale`. Rename const to `DEFAULT_EVAL_SCALE`.

In `run.rs`, between `run_tier2_with_live` returning acceptance and
`save_rater`:
```rust
let report = calibration::calibrate_from_corpus(&candidate.model, &device, &held_out_probes);
metadata.eval_scale = report.scale;
// log report.r_squared, report.median_abs_residual
save_rater(&candidate.model, &stem, &metadata)?;
```

Hold-out slice in `batch.rs` / `selfplay.rs`: when `generate_corpus`
produces `corpus`, seeded shuffle splits into `(train_corpus,
calibration_probes)` with ~10% to probes.

Load path in `evaluate_fen_at_stem` / `inspect_fen_at_stem`: read
`meta.eval_scale`, plumb through to the constructed evaluator.

**Test strategy**:
- Unit: hand-built tiny model with known weights such that
  `forward(x) = 0.5 * x[0]`; feed positions with known heuristic scores;
  assert fitted `k` matches closed-form.
- Property: random `(x_i, y_i)` from a target `k_true` + noise → `|k_fit
  − k_true| < tol`.
- Roundtrip: `RaterMetadata { eval_scale: 1234.5 }` survives save/load.
  Hand-rolled JSON without the field deserialises with `eval_scale ==
  0.0`.
- Behaviour: `with_scale(model, 0.0)` falls back to `DEFAULT_EVAL_SCALE`.
  `with_scale(model, 6000.0)` doubles all non-terminal outputs vs 3000.
- Integration: mini-run → resulting sidecar has non-zero `eval_scale`,
  `evaluate_fen_at_stem` produces finite centipawns within bounds.
- Diagnostic: print `r²`. Real long run should produce `r² > 0.6` once
  the NN has learned the dominant material terms. `r² < 0.2` is "this
  rater hasn't learned anything yet" alarm.
- **Defer to real run**: validate on a post-GPU lineage that ran ≥5
  generations at depth ≥4.

**Effort**: 1.5 hr code + 30 min real-data validation = **2 hr** total
(gated on tasks 3+5).

---

## 7. Wire NN selection into the actual game/AI engine

**Why seventh**: the downstream consumer for everything tasks 1-6
produce. A trained `.mpk` rater sitting in `runs/active/raters/` is dead
weight until the player can actually *play against it*. Strictly speaking
this isn't NN-training work - it's UX plumbing on the player-facing side -
but it's what makes the whole training pipeline a closed loop instead of
a one-way write to disk. Ordering after 1-6 means wiring against the
final shape of a rater (post task-2 gauntlet, post task-3 real
`RunConfig`, post task-6 calibrated `EVAL_SCALE`).

**What we know**:
- `Evaluator` is already a `Sync` trait at
  `core_engine/src/search/evaluator.rs:142`. `HeuristicEvaluator` is a
  unit struct; `NnEvaluator` implements the same trait and the comment at
  `evaluator.rs:140` explicitly anticipates this hookup.
- The search already has a generic seam: `find_best_with_evaluator(pos,
  tt, time_limit_ms, max_depth, &dyn Evaluator)` exists at
  `alpha_beta.rs:321`. `SearchCtx::evaluator: &'a dyn Evaluator` is the
  only call site for `evaluator.evaluate(pos)` in the hot path.
- `Match` at `core_engine/src/session.rs:200` currently has **no
  evaluator field**. Its three search entry points (`request_ai_move`
  line 491, `request_ai_move_forced` line 530, `request_ai_move_at_depth`
  line 543) all call `find_best` directly - implicitly pinning the
  heuristic.
- The wrapper surface (`tauri_wrapper/src/lib.rs`) exposes one engine per
  `u64` handle via `EngineRegistry`. Already-precedent for rater-aware
  IPC: `inspect_rater` loads from `<run_dir>/raters/<rater_id>` via
  `NnEvaluator::inspect_fen_at_stem`.
- Rater discovery infrastructure exists: `RaterIndex` at
  `nn_trainer/src/registry.rs:91` already round-trips over IPC
  (`read_rater_index` at `tauri_wrapper/src/lib.rs:534`).
- AI difficulty UX lives in
  `frontend/src/lib/state/settings.svelte.ts` (`p1ThinkTimeMs`,
  `p1MaxDepth`, …) and `frontend/src/routes/setup/+page.svelte`. The
  Inspector route only ever calls `defaultConfigJson()` - no per-seat
  evaluator option.
- **`NnEvaluator` is not `Sync` as written.** Its `Mlp<NdArray<f32>>`
  field is generally not `Sync`, and the `Evaluator` trait requires
  `Sync`. Today this is masked because `NnEvaluator` is only constructed
  inside a single thread per tier-2 match.

**Design decisions**:

1. **Injection site: `Box<dyn Evaluator + Send>` field on `Match`.**
   - Add `evaluator: Box<dyn Evaluator + Send>` to `Match` (default
     `Box::new(HeuristicEvaluator)`). All three `request_ai_move*` route
     through `find_best_with_evaluator(..., &*self.evaluator)`.
   - **Why trait object, not generic**: `Match<E: Evaluator>` would
     force every caller into generic-land, and `EngineRegistry`'s
     `HashMap<u64, EngineEntry>` can't live with `Match<E>`. Trait
     object localises dynamic dispatch to one indirection per leaf -
     `alpha_beta` already pays this today.
   - **Sync surgery**: relax `Evaluator: Sync` to `Evaluator: Send` in
     the trait. Re-add `+ Sync` *only* at the tier-2 predecessor-list
     call site via `Vec<&(dyn Evaluator + Sync)>` - one-line bound
     relaxation. Both `HeuristicEvaluator` and properly-constructed
     `NnEvaluator` satisfy `Sync + Send` once audited.
   - **Snapshot/serde**: `evaluator` is *not* serialised; restoring from
     snapshot rebuilds the default heuristic. If the frontend wants the
     rater to follow save/load, the rater ID lives in `Config`.
2. **Rater discovery: run-dir paths + blessed convention.**
   - Keep `run_dir` as the load primitive. Add a `game/raters/blessed/`
     convention with the same `index.json` layout. New IPC takes
     `(source: "run" | "blessed", id: string)`.
   - Defer creating the directory until first promotion; an empty
     `blessed/` is invisible by design.
   - Why not promote-only: forces a UX flow before the player has any
     way to know whether the rater is worth playing.
3. **AI-difficulty composition: NN composes with depth, doesn't
   replace.** `AiBudget` continues to bound *search*, not *eval*. A
   reasonable preset table for the frontend: "Easy = Heuristic@2, Medium
   = Heuristic@6, Hard = Heuristic@8, Master = Blessed-NN@6" - but
   that's a frontend table, not engine policy.
4. **Blended evaluator (NN + heuristic, phase-weighted): defer.** Tempting
   but premature: heuristic is the calibration reference for
   `EVAL_SCALE` (blending un-calibrated signals propagates error
   multiplicatively); phase weighting presumes the NN has *learned*
   phase-relevant features (mooted by current corpus depth); the blended
   `Evaluator` impl is trivial to add later. Track as a future ADR
   triggered by the first concrete observation of phase-specific
   losses.

**Implementation sketch**:

1. **Relax `Evaluator: Sync` to `Evaluator: Send`** in
   `core_engine/src/search/evaluator.rs`. Add `+ Sync` back at the tier-2
   predecessor list call site.
2. **Add `evaluator` field to `Match`** in `core_engine/src/session.rs`:
   ```rust
   pub struct Match {
       /* existing fields */
       evaluator: Box<dyn Evaluator + Send>,
   }
   ```
   `Match::new*` constructors default to `Box::new(HeuristicEvaluator)`.
   `pub fn set_evaluator(&mut self, e: Box<dyn Evaluator + Send>)`. All
   three `request_ai_move*` route through `find_best_with_evaluator`.
3. **`core_engine::wrapper_api`** helper:
   `pub fn set_match_evaluator(m: &mut Match, eval: Box<dyn Evaluator +
   Send>)`.
4. **New IPC commands** in `tauri_wrapper/src/lib.rs`:
   - `list_available_raters() -> Vec<RaterListing>` - walks both
     `default_run_dir()/raters/` and `game/raters/blessed/`. Returns
     `{ source, id, accepted_at, parent_id, bracket_results }`. Empty
     blessed dir returns empty list, not error.
   - `set_ai_evaluator(handle: u64, source: "heuristic"|"run"|"blessed",
     id: Option<String>)` - resolves stem, loads via
     `persistence::load_rater::<InferenceBackend>`, wraps in
     `NnEvaluator::new`, calls `set_match_evaluator`.
   - `promote_rater(run_dir, rater_id)` - copies blob+sidecar into
     `game/raters/blessed/`, appends to its `RaterIndex`. Optional v1.
5. **Frontend**:
   - `settings.svelte.ts`: `p1Evaluator: { source: "heuristic"|"run"|
     "blessed", id: string | null }` + P2 mirror.
   - `setup/+page.svelte`: source select + (when non-heuristic) rater
     select populated from `list_available_raters`.
   - `match-store.svelte.ts`: after `createEngine` returns, call
     `set_ai_evaluator` per AI seat (skip if heuristic).
   - `inspector/+page.svelte`: third evaluator picker; install after
     `entryFromFen` / `entryFromMatchLog` / `entryFromTree` boots.
6. **Wasm story**: `wasm_wrapper` always gets heuristic - burn isn't
   bundled. Document as Tauri-only for now; cross-target = future ADR.

**Test strategy**:
- Unit (core_engine): mock `Evaluator` records calls in a thread-local;
  `request_ai_move` increments the counter.
  Default-evaluator-preserves-heuristic-score regression test.
- Unit (nn_trainer): load + box freshly-initialised rater → install on
  Match → `request_ai_move_at_depth(2)` → no-panic.
- Wrapper smoke: empty run dir → `list_available_raters` returns `[]`
  (not error). Unknown rater ID → `Err(_)` and match unchanged.
- End-to-end (manual, gated on §1-§6): train ≥1 acceptance → restart →
  pick "P2 = NN(blessed/v0001)" → play a quick HvAI match → confirm AI
  thinks, toggling produces visibly different moves.

**Effort**: **2-3 hr** realistic. 30 min `Match::evaluator` field + Sync
surgery; 30 min `NnEvaluator` Send/Sync audit + boxed-loader helper; 45
min new IPCs + smoke tests; 45 min setup/inspector UI + settings; 30 min
slack for cross-cutting issues.

---

## 8. Checkpoint/resume of `RaterIndex` mid-run

**Why eighth** (optional): the acceptance path already persists every
accepted rater plus the updated `index.json` at end-of-generation - so
killing the process between generations costs nothing beyond whatever
wall-clock was spent on the dropped generation. On clean shutdown that's
already a no-op; on `SIGKILL` we lose the in-progress generation's
corpus, `train_lineages` output, and unjudged candidate pool. On a
CPU-only smoke run that's seconds; on a long GPU run, hours. Worth doing
once the rest of the cleanup has shaken out, but not load-bearing.

**What we know**:
- `RaterIndex::save` already does the right thing: append-only JSON,
  `load` returns `Default::default()` on missing - so resume already
  works at the generation boundary for accepted state.
- `run_training` does, per generation: (1) build corpus, (2)
  `train_lineages_with_progress` → `Vec<Lineage<AutodiffB>>`, (3)
  Tier-1 fitness, (4) Tier-2 acceptance, (5) `save_matrix` + optional
  `index.append`/`save`. Acceptance + matrix save is the natural
  checkpoint boundary.
- `Lineage<B>` is `{id, model: Mlp<B>, seed, loss_history}`. Only the
  model isn't trivially `Serialize` - `persistence::save_rater` already
  writes `Mlp<B: Backend>` to `<stem>.mpk` + JSON sidecar with
  `MlpConfig`. The autodiff wrapper has to be stripped (the
  `into_inference` helper in `train.rs` already does it; orchestrator
  uses it on the winning lineage at line 336).
- `ChampionTracker` holds per-track best scores in memory, rebuilt fresh
  every process start. Resume needs it recovered from the index, or the
  first post-resume acceptance uses stale comparators.
- Determinism reality: `train_lineages` is deterministic *up to rayon's
  parallel order*; `generate_corpus` is not bit-reproducible across
  hosts/threads. "Re-run from `seed_root` to recover the dropped
  generation" is not safe in general.

**Design decisions**:

1. **Granularity: per-generation, not mid-generation.** Mid-generation
   has three painful failure modes: (a) `train_lineages` doesn't expose
   an interior cancellation point; we'd have to plumb a callback through
   it and serialise Adam optimiser moments; (b) the corpus is held
   entirely in memory and is large; persisting per lineage multiplies
   disk traffic; (c) one gradient step costs a fraction of a second -
   negligible upside. Per-generation costs at most one generation of
   work; the marginal addition is **one file**: an "in-progress
   generation" snapshot written after `train_lineages` returns and
   before Tier-1 begins.
2. **Resume determinism: best-effort, not bit-identical.** We can't
   guarantee `generate_corpus` is bit-identical across hosts. Demanding
   it would force us to also checkpoint the corpus, doubling checkpoint
   size for nothing. Contract: *if* resume succeeds, training proceeds
   as if the kill had not happened *from the saved lineage pool
   onward*. Numbers downstream may differ from an uninterrupted run,
   but gauntlet comparators are on disk, so candidates that would have
   been accepted get a fair shake.
3. **Dependency on Task 4.** Lineages in memory are
   `Lineage<AutodiffB>`. The resumed process must round-trip them
   through disk; `save_rater` is generic over `B: Backend` but loading
   back into `Autodiff<NdArray>` for further training requires the
   inference-side model to be lifted into the autodiff backend - a
   Task-4-shaped concern. **Do this AFTER Task 4.**
4. **Checkpoint location and atomicity.** New file
   `<run_dir>/raters/in_progress.json` + sibling directory
   `<run_dir>/raters/in_progress/`. JSON sidecar lists generation index,
   gen seed, run-config digest, and per-lineage `{id, seed,
   loss_history, stem}`. Write-temp-then-rename for atomicity. On
   successful acceptance at end-of-generation, delete the directory +
   sidecar. On resume, presence of the sidecar means "skip corpus build
   + train_lineages, hydrate lineages from disk, jump to Tier-1 with
   gen_idx set from the sidecar".
5. **Bound on resume scope via config digest.** Serialise a
   `RunConfig::digest() -> [u8; 32]` into the in-progress sidecar; on
   resume, refuse to continue if the caller passed a different config
   (rename stale checkpoint aside, start fresh).

**Implementation sketch**:

1. **New module `lineage_checkpoint.rs`** with:
   - `save_lineages<B: AutodiffBackend>(&[Lineage<B>], dir, seed,
     gen_idx, model_cfg)` - writes umbrella sidecar + `lin-{i}.{mpk,
     json}` per lineage via `into_inference` + `save_rater`.
   - `load_lineages<B: AutodiffBackend>(dir, device) -> Option<InProgress<B>>`
     - lifts inference model back into autodiff backend (Task 4
     dependency).
   - `clear_lineages(dir)` - recursive delete on successful
     end-of-generation.
2. **Wire into `run_training`**: at loop entry, attempt
   `load_lineages`; on match, skip corpus + `train_lineages` for that
   gen_idx, jump to Tier-1. On normal path, save checkpoint *before*
   Tier-1 starts. After `index.save(...)` clear the checkpoint -
   regardless of acceptance.
3. **`ChampionTracker::from_index(&RaterIndex) -> Self`** - walk
   `index.entries`, replay each `bracket_results` into the tracker's
   per-track best. Independent of lineage checkpoint and useful on its
   own.
4. **`RunConfig::digest()`** - stable SHA-256 of `serde_json::to_vec` of
   self. Hex-encode into the sidecar.
5. **Cancellation handshake (optional polish)** - when `should_stop`
   fires after `train_lineages` but before acceptance, write the
   in-progress checkpoint on the way out so next run can resume from
   Tier-1. Not strictly required.

**Test strategy**:
- Unit: `lineage_checkpoint` round-trip - save, load, forward pass on
  each loaded lineage matches original within 1e-6.
- Unit: stale checkpoint with mismatched config digest is renamed aside,
  not honoured. Assert resume returns `None` and original preserved
  under `.stale-{ts}` suffix.
- Integration: orchestrator-smoke-style test runs 2 generations, kill
  simulated after gen-2's `train_lineages` returns. Verify in-progress
  file exists. Second `run_training` invocation completes gen-2 without
  redoing `train_lineages` (mock corpus builder panics if called twice
  for same `gen_idx`).
- Integration: resume across processes - gen 1 to completion → exit →
  separate process invocation with same `run_dir` picks up gen-2.
- Negative: corrupted sidecar → log, ignore, run from scratch. Never
  crash on bad checkpoint.
- Tracker restore: hand-build `RaterIndex` with three entries across
  tracks → `ChampionTracker::from_index` → `consider(...)` with a
  comparator that should *not* trigger; assert no update fires.

**Effort**:
- `lineage_checkpoint` + round-trip tests: ~1 day *assuming Task 4 has
  landed*. Without Task 4: ~2-3 days (autodiff ↔ inference round-trip
  solved here and re-solved when Task 4 arrives).
- `ChampionTracker::from_index` + tests: ~0.5 day.
- `run_training` wiring + integration test: ~0.5 day.
- Config digest + stale-checkpoint quarantine: ~0.5 day.
- **Realistic total: 2.5-3 days after Task 4, 4-5 days before. Defer
  until Task 4 is in.**
