# Search Depth-Cliff — Root Cause & Fixes (Phase 2: search)

Status: **INVESTIGATED — direction changed. QS turned OFF for our work; blocked on Phase 1.**
Do the eval redesign (`custom-eval-redesign.md`, Phase 1) FIRST; then re-decide QS on the NEW eval.
HARD constraint: fixes must improve BOTH evaluators (heuristic AND custom) — the solution is
the cleanest overall one, NOT tailored to either. Benchmark both before/after. No quality loss.

---

## ⇒ CURRENT STANDING (2026-08, read this first)

The original plan (S1–S4 below) assumed we'd *tame* quiescence (QS). Investigation changed the
direction. Summary of what happened and where we are:

**What was done and kept:**
- **S4 (Shove-is-loud) — DONE & committed to the working tree.** The `is_loud_support_skills_false`
  test was failing on `main` (Shove classified loud in code, not-loud in header+test). Designer's
  call: keep Shove **always loud** ("the core problem won't be solved by dampening one skill; the
  solution has to be more fundamental"). Fixed the header (`quiescence.rs` `is_loud` doc) + the test
  to match the code. All 7 `is_loud` tests green.
- **Bench harness extended to dual-eval + cliff corpus — DONE.** `run_sweep.sh` now takes
  `[eval-id] [corpus-path] [--time-only]`; `game/bench/corpus/cliff.txt` wraps the 17
  `critique_fens.txt` positions in corpus format (cliff-08 = the ebf-50 blowup, cliff-09 = calm).
  QS-ON cliff baselines saved at `game/bench/results/baseline-cliff-{heur,custom}-time*.json`.
- **`--no-qs` flag added to `search_bench`** (sets `alpha_beta::DISABLE_QS`) — the QS on/off A/B lever.

**What the QS on/off measurement showed (both evaluators, cliff + main corpus):**
- **The root problem is NOT a ply-cap issue.** There is *no "check" in this game* — you win by
  capturing the King; nothing forces a response to a King-threat. `is_king_threatened` /"in check"
  is a chess metaphor for "the enemy could damage the King next action." One player's TURN is ~6
  plies (2 Move actions + N Skill actions + phase transitions), so a small QS-ply cap cuts off
  mid-defense. Discarded that approach.
- **The real flaw:** `quiescence.rs:242` (`if !in_check && !is_l { continue; }`) — when the King is
  threatened, QS abandons the loud-move filter and searches the FULL ~40–80-action skill list as
  "evasion." Because King-threat is common in endgames and does NOT shrink the reply set (no forced
  response), this is a full-width undisciplined search (no TT, no ordering). That is the entire cliff:
  cliff-08 is **99.8% QS nodes** (custom, 3000ms: 6.19M of 6.20M, depth 4, ebf ~50).
- **QS-OFF wins the cliff decisively:** +1.5–2 plies deeper on essentially every cliff position at
  every time budget, both evals (e.g. cliff-08 custom @1000ms: d4/ebf38 → **d7/ebf8.6, same best
  move**; king-in-danger-02: QS-on returns a *pass at depth 1* reading a forced loss, QS-off searches
  to depth 4).
- **QS-OFF's only cost is horizon-effect blunders on tactical midgame positions — and only with the
  weak HEURISTIC eval** (midgame-move-03/04 flip +800 → −4000 at the same depth). The **custom eval
  barely moves** (score swings ~80 vs the heuristic's ~4000). QS exists solely to cover an eval that
  misreads mid-exchange positions; the custom eval already largely doesn't need it, and Phase-1 **C4
  (hp/armor-vs-exposure)** builds exactly that awareness.

**Designer decision (the new direction):**
1. **Keep QS OFF for our custom-eval work** (via `--no-qs` in experiments) and go do Phase 1 first.
2. **Make the custom evaluator the DEFAULT everywhere**, and **drop the "stub" naming**
   (`custom-stub` → `custom`; "Custom (stub)" → "Custom"). Rename surface (verified): registry
   `evaluator/mod.rs:233-234`, doc `custom.rs:4`, `search_bench/src/main.rs` (4 usage strings),
   `ARCHITECTURE.md` (2 mentions), and these plan files. (NOT "stub" in `nn_trainer/lineage_checkpoint.rs`
   `LineageStub` or the stale `skills.rs` comment — those are unrelated.)
3. **Preferred long-term fix — per-evaluator search settings.** Rather than a global `DISABLE_QS`,
   let each `Evaluator` declare the search settings it wants (QS on/off first; flags+params later).
   Heuristic keeps QS (needs horizon cover); custom runs QS-off. This fully avoids the global
   compromise. This is a multi-crate change (Evaluator trait + `find_best_with_evaluator` reads the
   settings + production callers in `tauri_wrapper`/`nn_trainer` + frontend default in
   `SettingsModal.svelte`) — scope it as its own plan/task before implementing; do NOT freehand it.

**Consequence for S1/S2/S3:** with QS off (or per-eval-off), the cliff is already fixed for the
custom eval, so **S1/S2/S3 below are likely unnecessary.** Do NOT build them pre-emptively. After
Phase 1, re-run the QS on/off A/B on the NEW eval; only revisit S1/S2/S3 if a residual cliff remains.

**Immediate next step:** compact, then work `custom-eval-redesign.md` (Phase 1). Come back here to
decide QS default + the per-eval-settings mechanism once the eval handles exposure.

---


## Context

Measured: the alpha-beta search reaches depth 9–13 in early/mid-game but collapses to depth
3–5 once a king is threatened (Skill phase / endgames). This is the highest-value AI problem:
endgames MUST reliably find the finishing/mate sequence, and today the search actively gets
*shallower* on exactly those lines.

Hard data (2 s fixed-time per position, custom eval, `search_bench --time-ms 2000`):
- Calm mid-game positions: depth 9–13, ebf ~3–9.
- King-danger skill-phase position (pos-8 in `game/tools/critique_fens.txt`): depth 4,
  **ebf 39.4**.

Fixed-depth-5 A/B on pos-8 (danger) vs pos-9 (calm) isolates the mechanism:

| position | ab-nodes | qs-nodes | SEE calls | ebf |
|---|---|---|---|---|
| pos-8 (king danger) | 48,089 | **11,768,154** | **12,558,084** | 25.97 |
| pos-9 (calm) | 2,653 | 1,882 | 1 | 5.39 |

**Quiescence is 99.6% of the tree** in the danger position (11.7 M QS nodes; 12.5 M SEE
calls) vs. essentially zero in the calm one. That is the cliff.

---

## Root cause (three compounding mechanisms; all confirmed in code)

Anchors from a read-only search-code investigation:

1. **QS abandons loud-move filtering when a king is threatened.**
   `game/crates/core_engine/src/search/quiescence.rs:242` (`if !in_check && !is_l { continue; }`):
   when the STM king is threatened, QS does NOT filter to loud moves — it generates the full
   skill-phase move list (40–80 actions/node) and recurses on *every* action down to
   `MAX_QS_PLY = 8` (line 52), with **no TT and no killers/history** (lines 24–25). Stand-pat is
   also skipped in check (lines 202–210). This is the width bomb.

2. **No extensions anywhere; and LMR/LMP are switched OFF in check.**
   `alpha_beta.rs` has zero extension logic (no check/king-danger/recapture extension). Worse,
   `node_in_check` (line 463) *disables* LMR (line 496 `&& !node_in_check`) and LMP (lines
   467–471 `lmp_thresh = None`). So at the highest-branching nodes the search reverts to nearly
   plain alpha-beta over a huge move list — strictly *less* selectivity where it needs more.

3. **Move-ordering sort gated to depth ≥ 3, but the cliff lives at depth 1–3.**
   `alpha_beta.rs:433` (`if ply < MAX_PLY && depth >= 3`): below depth 3 only TT-move-first runs
   (no killer/history sort). Once depth collapses, interior nodes run with the weak ordering the
   code's own comment (lines 444–447) warns "blew up skill-phase-full nodes by +55%". Poor
   first-move cutoffs keep ebf ≈ raw branching factor. Also: **no SEE ordering in the main
   search** — SEE exists (`see.rs`) but is used only inside QS (`quiescence.rs:245–274`).

Skill-phase fan-out itself is intrinsically large and grows with money/round/Focus
(`generator.rs:213–390`: Shove 8-dir at 329–337, Focus multipliers at 251–379; budget scales
`make_unmake.rs:1168`, income `turn_manager.rs:106`). That's the raw branching the above
mechanisms fail to contain.

**Secondary bug found:** `Shove` is coded as loud in QS (`quiescence.rs:84`) but the module
header + the `is_loud_support_skills_false` test (lines ~373–380) assert it is NOT loud. Live
inconsistency — changes LMR/LMP eligibility and QS inclusion for every Shove. Resolve as part
of this work (decide the correct answer, fix code + test together).

---

## The fixes (highest-value first; land + benchmark ONE at a time)

Each is independent; each must be A/B'd on both evaluators before the next. Do NOT batch.

### S1. Bounded check / king-danger extension (biggest missing piece)

Add a search extension: when a node is in check (`is_king_threatened`), extend +1 ply, capped
per branch (e.g. a total extension budget, or only re-extend on genuinely new threats) to avoid
search explosions on perpetual-threat lines. This directly restores depth on forced king lines
— the intended-goal lever for "always find the finishing sequence". Standard chess-engine fix
for a check-driven cliff.
- Risk: unbounded extension → its own blow-up. Cap it; measure node cost.

### S2. Tame the in-check QS expansion

`quiescence.rs:242` — when in check, restrict replies to *king-safety-relevant* moves (capture
or block the threatening piece, or move the king) instead of the full generator output; and/or
lower `MAX_QS_PLY` for skill-phase-in-check nodes. This caps the 11.7 M-node leaf explosion at
its source. Pairs naturally with S1 (S1 gives depth back in the main tree; S2 stops QS from
eating the time budget).
- Risk: dropping a real defensive resource from QS → tactical blindness. Validate against the
  corpus tactical assertions (see below) that no known-best move regresses.

### S3. Move-ordering at shallow depth + SEE in the main search

- Enable the killer/history sort at depth ≥ 1–2 for the Skill phase (the cliff's depth band),
  not just ≥ 3 (`alpha_beta.rs:433`). The comment there warns of a +55% blow-up if the sort is
  dropped — the cliff is precisely running in that regime.
- Add SEE-based ordering of Strike / Move-Attack actions in the main tree (SEE already in
  `see.rs`, used only by QS today). Better first-move cutoffs are what actually shrinks ebf.
- Risk: sort cost at every shallow node. Measure NPS; the win is fewer nodes, must net positive.

### S4. Fix the `Shove`-is-loud inconsistency

Decide the correct classification, fix `quiescence.rs:84` + the header + the test in one
change. Small, but it perturbs S2/S3 (loud moves are never reduced/pruned), so do it early so
later measurements are clean.

---

## Benchmarking protocol (MANDATORY — both evaluators, no tailoring)

The existing harness: `game/bench/run_sweep.sh <prefix>` runs the 5-budget grid
(`depth6` + `time100/500/1000/3000ms`) over `game/bench/corpus/corpus.txt`, plus a determinism
smoke gate; `game/bench/compare.py` diffs runs; accepted baselines live at `game/bench/*.json`.

**Gap to close first:** `run_sweep.sh` currently benchmarks only the heuristic (the harness
predates `--eval` routing, which this session added to `search_bench`). Before any S-fix:
1. Extend the sweep/measurement to run each budget under BOTH `--eval heuristic` and
   `--eval custom-stub` (either parameterize `run_sweep.sh` or run the binary twice with
   distinct out-prefixes). Capture a **baseline for each evaluator**.
2. Add the king-danger / endgame positions (the 17 in `game/tools/critique_fens.txt`, plus any
   mate-in-N the corpus has) to the corpus or a companion corpus, so the cliff is actually
   *in* the measured set — the current corpus may under-represent it.

**For every S-fix, on BOTH evaluators:**
- Run the full sweep; `compare.py` vs baseline. Report: depth reached (esp. on king-danger
  positions), nodes, NPS, ebf, and — critically — the **corpus tactical assertions** (best-move
  + score-range regressions must be ZERO; that's the correctness gate).
- Determinism check must pass (the sweep runs it; fail loudly otherwise).
- Accept a fix only if it helps the cliff positions AND does not regress either evaluator on the
  calm positions (nodes/NPS within noise, no tactical regressions). If a change helps custom but
  hurts heuristic (or vice-versa), it is NOT clean enough — rework toward the neutral solution.

## Files (expected)

- `game/crates/core_engine/src/search/alpha_beta.rs` — extension (S1), ordering/sort-gate + SEE
  (S3).
- `game/crates/core_engine/src/search/quiescence.rs` — in-check QS scope + `MAX_QS_PLY` (S2),
  Shove classification + test (S4).
- `game/bench/run_sweep.sh` (+ maybe `compare.py`) — dual-evaluator sweep; corpus additions.
- No evaluator-logic edits (that's Phase 1).

## Sequencing

Extend the bench to dual-eval + add cliff positions → baseline both → S4 (clean the noise) →
S1 → benchmark both → S2 → benchmark both → S3 → benchmark both. Commit only when the designer
asks; never tag/push without the release-versioning rule.

**Sequencing update (2026-08, after baselining).** S4 done first as written. The dual-eval
baseline on the cliff corpus (`game/bench/corpus/cliff.txt`; results at
`game/bench/results/baseline-cliff-{heur,custom}-time*.json`) showed the worst king-danger
position (`cliff-08` = pos-8) is **99.8% quiescence nodes** (custom eval, 3000ms: 6.19M qs of
6.20M total, depth 4, ebf ~50). QS starvation — not main-tree depth — is the dominant mechanism
there. So the designer chose **S2 before S1**: tame the in-check QS blowup first (the biggest
lever), benchmark, then S1 restores depth on the now-affordable forced lines. Revised order:
**S4 → S2 → S1 → S3**, each A/B'd on both evaluators. Cliff A/B uses the time budgets only
(fixed depth6 is pathological on ebf-50 positions); diff with `game/bench/cliff_compare.py`.
