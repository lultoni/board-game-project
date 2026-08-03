# Architecture

This document describes every file in the `game/` directory: what it does, what it owns, and how it connects to everything else. It is organised by crate and layer, following the engine's own five-layer stack.

---

## Table of Contents

1. [High-Level Overview](#1-high-level-overview)
2. [Workspace Layout](#2-workspace-layout)
3. [crates/core_engine — Layer 1: State](#3-cratescore_engine--layer-1-state)
4. [crates/core_engine — Layer 2: Game Logic](#4-cratescore_engine--layer-2-game-logic)
5. [crates/core_engine — Layer 3: Search](#5-cratescore_engine--layer-3-search)
6. [crates/core_engine — Layer 4: Session](#6-cratescore_engine--layer-4-session)
7. [crates/core_engine — Layer 5: Telemetry](#7-cratescore_engine--layer-5-telemetry)
8. [crates/core_engine — Public API Surface](#8-cratescore_engine--public-api-surface)
9. [crates/tauri_wrapper](#9-cratestauri_wrapper)
10. [crates/search_bench](#10-cratessearch_bench)
11. [crates/nn_trainer](#11-cratesnn_trainer)
12. [frontend](#12-frontend)
    - [12.1 Routing Structure](#121-routing-structure)
    - [12.2 Engine Boundary](#122-engine-boundary-srclibengine)
    - [12.3 Board Rendering](#123-board-rendering-srclibboard)
    - [12.4 Visual Effects Pipeline](#124-visual-effects-pipeline)
    - [12.5 Audio](#125-audio-srclibaudiosftsts)
    - [12.6 State Stores](#126-state-stores-srclibstate)
    - [12.7 The /match/ Fat Controller](#127-the-match-fat-controller)
    - [12.8 Multiplayer](#128-multiplayer-srclibmultiplayer--srclibmultiplayer)
    - [12.9 Storage / Telemetry](#129-storage--telemetry-srclibstorage)
    - [12.10 Extracted Component Directories](#1210-extracted-component-directories)
    - [12.11 Replay / Inspector Data Flow](#1211-replay--inspector-data-flow)
    - [12.12 Key Seams](#1212-key-seams)
    - [12.13 Testing](#1213-testing)
    - [12.14 Observed Extraction Opportunities](#1214-observed-extraction-opportunities)
13. [relay](#13-relay)
14. [bench](#14-bench)
15. [tools](#15-tools)
16. [Cross-Cutting Data Flows](#16-cross-cutting-data-flows)

---

## 1. High-Level Overview

The engine is a pure Rust library (`core_engine`) with no I/O, no threads, and no platform assumptions. Everything above it — the desktop app, the AI trainer, the benchmark harness — consumes it as a dependency. The five internal layers are strict: each layer may only call the layer below it.

```
┌─────────────────────────────────────────┐
│  5  telemetry          src/telemetry.rs │  per-ply log, JSON helpers
├─────────────────────────────────────────┤
│  4  session            src/session.rs   │  Match lifecycle, undo history, TT
├─────────────────────────────────────────┤
│  3  search             src/search/      │  alpha-beta, evaluator, QS, SEE, TT
├─────────────────────────────────────────┤
│  2  game_logic         src/game_logic/  │  move gen, make/unmake, skills, draft
├─────────────────────────────────────────┤
│  1  state              src/state/       │  Position, bitboards, mailbox, FEN, Zobrist, action notation
└─────────────────────────────────────────┘
```

External consumers access the engine through two façades:

- **`src/wrapper_api.rs`** — a flat, allocation-minimising set of free functions shared by the Tauri desktop wrapper and any future WASM build.
- **`src/lib.rs`** — re-exports the public types that downstream crates (`tauri_wrapper`, `search_bench`, `nn_trainer`) import directly.

Above the engine sit five additional components:

| Component | Technology | Role |
|---|---|---|
| `crates/tauri_wrapper` | Tauri 2 / Tokio | Desktop app shell; exposes engine as `invoke()` commands |
| `crates/nn_trainer` | Burn (autograd) + `wide` SIMD | Neural-net rater training and inference |
| `crates/search_bench` | Rust CLI | Search-speed benchmarking and regression checks |
| `frontend` | SvelteKit + TypeScript | UI rendered inside the Tauri webview |
| `relay` | Bun + WebSocket | Cloud relay for multiplayer sessions |

---

## 2. Workspace Layout

```
game/
├── Cargo.toml                  workspace root
├── Cargo.lock
├── crates/
│   ├── core_engine/            pure game logic, search, eval (no I/O)
│   ├── tauri_wrapper/          Tauri 2 desktop app; wraps core_engine
│   ├── nn_trainer/             NNUE training pipeline
│   └── search_bench/           CLI benchmark harness
├── frontend/                   SvelteKit + TypeScript UI
├── relay/                      Bun WebSocket relay (Fly.io)
├── bench/                      benchmark corpus + sweep scripts
├── tools/                      analysis scripts (Python)
├── plans/                      active engineering plan docs
└── runs/                       training run artefacts (gitignored)
```

The workspace `Cargo.toml` lists all four crates as `members` with no `default-members` key, so `cargo build` builds the whole workspace; individual crates are targeted with `-p`. Shared metadata (`version`, `edition`, `authors`) lives in `[workspace.package]`, and the release profile pins `lto = true` / `codegen-units = 1` / `opt-level = 3`.

---

## 3. crates/core_engine — Layer 1: State

Layer 1 owns the complete, minimal representation of a game position. Nothing here computes move legality or game outcomes — it only defines the data structures, their serialisation, and the geometry primitives that the layers above need.

### 3.1 `src/state.rs`

Module root for Layer 1. Contains no logic; its only purpose is to declare the seven child modules and to re-export the most-consumed types (`Bitboard`, `FenError`, `NotationError`, `MailboxEntry`, `EMPTY_MAILBOX_ENTRY`, `Position`, `Phase`, `Player`, `GameResult`) so callers do not need to reach into sub-modules. The ADR-005 canonical bit-layout spec for `MailboxEntry` — `[hp:2][armor:2][combo:3][skill1:4][skill2:4]` — lives here as the single authoritative comment that all other files cross-reference.

### 3.2 `src/state/bitboard.rs`

Defines `Bitboard(pub u64)`, a newtype over a 64-bit integer where bit `i = rank*8 + file` (rank 0 = P1 home row). Every spatial query in the engine ultimately resolves to bitboard arithmetic. All operators (`BitAnd`, `BitOr`, `BitXor`, `Not`) are `#[inline]` delegations to their `u64` equivalents with zero overhead. Key methods: `from_square(sq: u8)`, `contains(sq)`, `count()`, `is_empty()`, `lsb() -> Option<u8>` (lowest set bit via `trailing_zeros`). The two constants `EMPTY` (0) and `FULL` (!0) cover the common extremes.

### 3.3 `src/state/mailbox.rs`

Defines `MailboxEntry(pub u16)`, a packed bitfield encoding all per-piece mutable state for one square: HP (bits 0-1), Armor (2-3), combo counter (4-6), skill1 ID (7-10), skill2 ID (11-14). Bit 15 is reserved. Every field accessor and setter is `#[inline]`; setters follow an immutable builder pattern (`with_hp`, `with_armor`, etc.) — each returns a new `Self` rather than mutating in place, which makes the code trivially safe to diff during undo. `EMPTY_MAILBOX_ENTRY = MailboxEntry(0)` is the canonical "nothing on this square" value; zero is a valid sentinel for all fields because skill IDs use 0 as "unequipped."

### 3.4 `src/state/position.rs`

The `Position` struct is the complete game state. It holds:

- **Spatial bitboards** — `p1_pieces`, `p2_pieces`, `kings`, `champions`, `guards` (five `Bitboard` fields; occupancy is derived as their intersection/union, not stored separately).
- **Per-square data** — `mailbox: [MailboxEntry; 64]` (entries on unoccupied squares are undefined, matching Stockfish convention).
- **Resources** — `p1_money: u16`, `p2_money: u16`, `round_number: u16`, `to_move: Player`, `current_phase: Phase`, `actions_remaining: u8`.
- **Turn tracking** — `moved_this_phase: Bitboard`, `pending_modifiers: u8` (three flags: `FOCUS = 1<<0`, `CHARGE = 1<<1`, `MOVE_ATTACK_USED = 1<<2`).
- **Combo tracking** — `tracked_enemies: [u8; 16]` + `tracked_enemies_len: u8`, `tracked_casters: [u8; 8]` + `tracked_casters_len: u8` (each buffer paired with its own length), `champion_credit: u128` (a compact 8x16 cross-product bitmap).
- **Bodyguard** — `pending_bodyguard: Option<PendingBodyguard>` (mid-resolution state between a tentative Move-Attack and the defender's interception choice).
- **Hash** — `zobrist: u64` (always in sync with the above fields).
- **Terminal** — `game_result: Option<GameResult>`.

Key constructors: `setup_stack_m()` (canonical start position), `setup_stack_m_for_draft()` (opens in `Phase::Draft`), and `setup_stack_m_with_loadouts()` (skips draft by pre-assigning skill IDs). Replay-validated construction from a `Snapshot` is `Match::from_snapshot` (Layer 4), not a `Position` method — `Position` itself reconstructs from a string via `from_fen`. `Position` derives `Clone` but not `Copy` — it is too large for implicit copies on the search stack, so it is always moved or borrowed explicitly.

Also defined here: `Player { P1, P2 }`, `Phase { Draft, Move, Skill }`, `GameResult { P1Wins, P2Wins }` (no draw by design), and `PendingBodyguard { attacker_src, attacker_now, target_sq, eligible: [u8; 4], eligible_len }`.

### 3.5 `src/state/magic.rs`

All board-geometry primitives. Two categories of computation live here: sliding-attack tables (magic bitboards) and spatial helpers (BFS flood-fill, Chebyshev distance, ray stepping).

**Magic bitboards.** `SliderTables` holds per-square rook and bishop `mask/magic/shift/offset` arrays plus flat `attacks` Vecs (~0.82 MB total), built once via `OnceLock`. Magic search uses `MagicRng` — a fixed-seed xorshift64 — so tables are byte-identical across builds. PEXT (BMI2) was deliberately rejected in favour of multiply-shift magics because the primary target is aarch64.

**Static tables** (all `OnceLock`): `BETWEEN[64][64]` (squares strictly between two squares on a queen-ray), `WITHIN_RANGE[64][5]` (all squares within Chebyshev distance 1..=r), `MOVE1[64]` (8-neighbours), `CHEBY[64][64]` (pairwise Chebyshev distances).

**Key public functions:**

| Function | What it returns |
|---|---|
| `skill_attacks(sq, occ, range)` | Queen-style reachable squares, range-capped and occupancy-blocked |
| `between(a, b)` | Squares strictly between two squares on a shared ray |
| `on_ray(a, b)` | Whether two squares share a queen-ray |
| `step_toward(from, to)` | One step from `from` toward `to` along their ray |
| `step_away(pivot, at)` | One step from `at` away from `pivot` |
| `movement_targets_speed1(sq)` | 8-adjacent squares (King / Champion movement) |
| `movement_targets_speed2(sq, occ)` | Guard BFS-2 flood-fill via bitboard expansion |
| `king_expand(x)` | 8-direction bitboard dilation; shared primitive for SEE, evaluator, and Guard movement |
| `cheby_dist(a, b)` | O(1) table lookup |

The speed-2 flood-fill replaced a per-call BFS; both implementations are retained in the test module and proven equivalent by 4000-case per-square random fuzz.

### 3.6 `src/state/path.rs`

A thin `Position`-aware wrapper over `magic` primitives. Its purpose is to let `magic.rs` remain free of `Position` while giving the move generator and skill resolvers convenient higher-level queries.

- `skill_targets(pos, src, range) -> Bitboard` — occupied squares reachable from `src` within `range` (the "first blocker per ray" property of `skill_attacks`, intersected with the occupancy bitboard). Ownership filtering is left to the caller.
- `path_clear(pos, src, tgt) -> bool` — true if every square strictly between `src` and `tgt` on their shared ray is empty. Returns false if they share no ray.

### 3.7 `src/state/zobrist.rs`

Maintains `pos.zobrist` — a `u64` Zobrist hash of the full position — both incrementally (via XOR helpers called by `make_unmake`) and from scratch (via `full_recompute`).

All tables are computed at const-eval time (`static T: Tables = make_tables()`) using `splitmix64` seeded with `"BoardGam"`, so there is zero runtime initialisation cost. The table layout covers every field of `Position`: HP, armor, combo, skill IDs, piece occupancy, side-to-move, phase, actions, pending modifiers, round, money, moved-this-phase, game result, and pending-bodyguard state.

Three design decisions are worth noting. First, mailbox keys are decomposed per-field rather than per-full-`u16`, so incremental updates (`mailbox_xor(sq, prev, new)`) are cheap — typically only one or two table lookups differ. Second, unbounded scalars (money, round number) are bucketed modulo a power of two; hash collisions from this are missed transposition-table hits, not correctness bugs. Third, the turn-scoped combo-tracking fields (`tracked_enemies`, `tracked_casters`, `champion_credit`) are intentionally **not** hashed, enabling transpositions between move orderings that produce the same end-of-turn board.

### 3.8 `src/state/action_notation.rs`

Compact, human-readable encoding for `Action` values — the single canonical source of action strings used by every layer that needs to render an action as text.

`action_to_notation(action: Action, pending: Option<&PendingBodyguard>) -> String` encodes any action to a short string:

| Family | Example |
|---|---|
| Plain move | `a1-b2` |
| Move-Attack (speed-1) | `c3xd5` |
| Move-Attack (speed-2) | `c3xd5@c4` |
| Skill | `b2*d4:Tempest` |
| Skill + focus-effect | `b2*d4:Blast~` |
| Skill + focus-retarget | `b2*c3:Shield>` / `b2*d4:Dash>c3` |
| Skill + Shove direction | `b2*d4:Shove:NE` |
| EndPhase / EndTurn | `endphase` / `endturn` |
| Draft | `draft Lance@a1:1+Shield@b2:2` |
| Bodyguard decline | `bgX` |
| Bodyguard redirect | `bga5` (guard's square) |

The `pending` parameter is only meaningful for `BodyguardChoice` redirect actions. When `Some`, it resolves the guard's square from `pending.eligible[idx-1]` and emits `bg<sq>`. When `None` (all call sites except the one live path in `session.rs::try_apply_timed`), it falls back to `bg<N>` (numeric index). All other action families are unaffected by `pending`.

`notation_to_action(s: &str, pos: &Position) -> Result<Action, NotationError>` is the full inverse parser, used by test harnesses and future scenario tooling.

`NotationError` is a rich enum (8 variants) covering every parse failure. `sq_to_notation` / `notation_to_sq` are public helpers within the module but are not re-exported at the crate root — they are an implementation detail of the notation format, and no external consumer needs to format bare square indices independently.

### 3.9 `src/state/fen.rs`

Implements a custom FEN-like single-line format for losslessly round-tripping a `Position`. The format has three field-count variants: 9 fields (between-turns baseline), 12 fields (with the combo-tracker trailer), and 13 fields (with a pending-bodyguard trailer). Older saved positions load as 9-field and gain default zero values for the newer fields.

`to_fen(pos)` always emits the full current form. `from_fen(s)` accepts all three counts; `from_fen_strict(s)` additionally enforces piece-count invariants (1K+5C+6G per side, kings on different files). Both parsers recompute the `zobrist` field from scratch via `full_recompute` after parsing — the hash is never stored in the FEN string itself.

The main parsing complexity comes from the rank-separator `/` colliding with the `/` used inside per-piece brackets (`[hp/armor/combo/s1/s2]`). This is resolved by `split_ranks_respecting_brackets`, a small bracket-aware scanner. `FenError` is a rich enum with over 16 variants covering every possible parse failure.

---

## 4. crates/core_engine — Layer 2: Game Logic

Layer 2 translates the raw state representation into legal game actions and applies or reverses them. It is the only layer that mutates `Position`. The key invariant: `generator` is pure-read, `make_unmake` is the single write site, and `turn_manager` / `draft` are thin clients that re-use `make_unmake`'s Zobrist-aware helpers.

### 4.1 `src/game_logic.rs`

Module root. Declares the six child modules. Contains no logic; its doc-comment names the dependency direction: `action` and `skills` are pure data, `generator` reads state, `make_unmake` writes it, `turn_manager` and `draft` layer on top.

### 4.2 `src/game_logic/action.rs`

Defines `Action(pub u32)`, the single information bus between the generator, the resolver, and the search. The 32-bit word encodes source square (bits 0-5), target square (6-11), action kind (12-13), skill ID (14-17), choice index (18-21), focus-effect mode (bit 22), aux/approach square (23-28), presence flags (bit 29), and two tag bits: `DRAFT_TURN_TAG` (bit 30) and `BG_CHOICE_TAG` (bit 31). The three families — regular actions, `DraftTurn`, and `BodyguardChoice` — are mutually exclusive.

`ActionKind` is a `repr(u8)` enum: `Move = 0`, `Skill = 1`, `EndPhase = 2`, `EndTurn = 3`.

`Undo` is the companion record written by `make` and consumed by `unmake`. It snapshots every scalar field that any action can mutate (phase, actions remaining, side-to-move, money deltas, round number, pending modifiers, pending-bodyguard state, moved-this-phase mask), per-square mailbox snapshots for up to 16 changed squares, bitboard XOR deltas for all five piece-set fields, combo-tracking arrays, and the Zobrist delta. The `Undo` is one-per-stack-frame in the search — not one-per-position — so it only exists as long as the corresponding `make` is live.

`Action::default()` is deliberately 0, serving as the transposition-table sentinel for "no best move stored."

### 4.3 `src/game_logic/skills.rs`

The canonical skill registry: IDs, costs, ranges, categories, and target-owner contracts. Zero game-state mutation; pure lookup tables.

`Skill` is a `repr(u8)` enum with IDs 1-15: `Lance`, `Hook`, `Break`, `Steal`, `Tempest`, `Shield`, `Heal`, `Plate`, `Dash`, `Blast`, `Shove`, `Swap`, `Retreat`, `Focus`, `Charge`. ID 0 is the unequipped sentinel.

`SkillCategory` (`Strike | Shield | Move | Mystic`) and `TargetOwner` (`Enemy | Ally | Either | Empty | SelfOnly`) drive generator filtering. `SideLoadout = [(u8, u8); 6]` represents one side's six skill-bearer assignments (King at index 0, Champions 1-5 by ascending square).

Key functions: `skill_cost(s)`, `skill_default_range(s)`, `skill_category(s)`, `skill_target_owner(s)`, `validate_loadout(l)`, `mirror_loadout(l)` (reverses Champion indices 1↔5, 2↔4 for the P2 symmetric starting layout).

### 4.4 `src/game_logic/generator.rs`

Given a `Position`, produces the complete legal `Vec<Action>` for the current player and phase. The generator is pure-read: it never mutates `Position`.

`generate(pos)` is the entry point. It returns an empty vec if `game_result.is_some()` and otherwise dispatches to one of three phase handlers.

**Move phase** — if `pending_bodyguard` is `Some`, only `BodyguardChoice` actions are emitted. Otherwise, for each unmoved piece, `reachable(src, speed, occ, opp_bb)` is called (a Chebyshev BFS returning empty-reachable, attack-reachable, and per-square BFS distances). Plain moves and Move-Attacks are emitted with their `approach_sq` encoded in the action. `EndPhase` is always appended.

**Skill phase** — walks every piece x every equipped skill, checks cost, dispatches by `TargetOwner`, handles Focus-retarget and focus-effect variants. `EndPhase` is always appended.

**Draft phase** — delegates to `make_unmake::legal_draft_turns(pos)` for the full cross-product enumeration.

`bodyguard_guards_for(pos, target_sq, approach_sq)` performs the dual-adjacency check (Guards adjacent to both the approach square and the target), returning an ascending list of eligible Guard squares. Guards themselves are never eligible for protection under this rule.

### 4.5 `src/game_logic/make_unmake.rs`

The single site of all `Position` mutation. `make(pos, action) -> Undo` applies an action and records the undo information; `unmake(pos, &undo)` perfectly reverses it.

Dispatch inside `make`:
- **Draft** → `apply_draft_turn`
- **BodyguardChoice** → `apply_bodyguard_choice`
- **Move** → `apply_plain_move` or `apply_move_attack`
- **Skill** → per-skill resolver (`apply_lance`, `apply_hook`, ..., `apply_charge` — 15 total)
- **EndPhase** → `apply_end_phase`
- **EndTurn** → `turn_manager::end_turn`

The bodyguard interaction is the most structurally interesting case. `apply_move_attack` performs a tentative first hop to the approach square, then calls `generator::bodyguard_guards_for`. If eligible Guards exist it sets `pos.pending_bodyguard`, flips side-to-move (so the defender can choose), and returns without dealing damage. The defender's subsequent `BodyguardChoice` ply completes the transaction. Both plies have their own `Undo` and unwind cleanly in reverse order on the search stack.

All scalar field mutations go through Zobrist-aware helper functions (`write_mailbox`, `xor_piece`, `set_actions`, `dec_actions`, `set_phase`, `flip_to_move`, `set_round`, `set_p1_money`, `set_p2_money`, `set_pending`, `moved_set`, `moved_clear_all`, etc.). Each helper is `#[inline]`, XORs the appropriate Zobrist delta, then mutates the field. This makes it structurally impossible to modify `Position` without keeping the hash in sync.

`skill_phase_budget(round_number) -> u8` computes the Progression-table action count (`2 + (round - 1) / 10`). It is `pub(crate)` so `turn_manager` can call it without duplicating the formula.

### 4.6 `src/game_logic/turn_manager.rs`

Handles the end-of-turn transition. `end_turn(pos, undo)` performs seven steps in order: clear all combo counters on every piece (both sides), clear pending modifier bits and reset combo-tracking arrays, flip side-to-move (incrementing round number when P1 becomes active again), disburse money income to the new active player (skipped in round 1), then set phase to Move with 2 actions and clear the moved-this-phase mask.

`income_per_turn(round_number) -> u16` implements the Progression table: returns 0 for round 1, then steps up every five rounds (`2 + round / 5`).

All state changes flow through the `pub(super)` helpers exported by `make_unmake`, so the Zobrist invariant is maintained here too. `turn_manager` has no direct state-mutation code of its own.

### 4.7 `src/game_logic/draft.rs`

Two concerns: a lightweight UI-facing snapshot of the draft state, and a preset-driven draft strategy for AI seats.

`DraftState` (a `Copy` struct) encodes the current draft turn number, the active player, and a 12x2 grid of which skill slots are already filled. It is computed on demand from `Position` — nothing extra is stored.

`next_preset_draft_turn(pos, preset)` is the deterministic AI draft picker. It walks the side-to-move's skill-bearers in canonical order (King first, Champions ascending) and returns the first two unfilled slots that match the preset loadout. `DEFAULT_AI_LOADOUT` and `DEFAULT_AI_LOADOUT_P2` are curated four-category presets used for single-player and AIvAI games respectively. Full-search draft AI is a documented future work item.

---

## 5. crates/core_engine — Layer 3: Search

Layer 3 implements the AI. It is read-only with respect to `Position` between calls — it calls into Layer 2's `make`/`unmake` internally but always restores the position before returning. The public entry points are `find_best` and `find_best_with_evaluator`.

### 5.1 `src/search.rs`

Module root. Declares the six child modules (`alpha_beta`, `quiescence`, `see`, `transposition`, `counters`, `evaluator`). Contains no logic.

### 5.2 `src/search/alpha_beta.rs`

Implements iterative-deepening alpha-beta with a full complement of pruning heuristics.

**Score convention.** Scores are absolute P1-POV throughout — not negamax — which keeps mate-distance arithmetic frame-invariant regardless of which side is to move.

**Public entry points:**
- `find_best(pos, tt, time_limit_ms, max_depth) -> SearchResult` — calls `find_best_with_evaluator` with `HeuristicEvaluator`.
- `find_best_with_evaluator(pos, tt, time_limit_ms, max_depth, evaluator, on_depth) -> SearchResult` — the main loop. Handles the forced-move short-circuit, seeds a fallback move before depth-1 in case of clock abort, runs iterative deepening from depth 1 to `max_depth`, discards aborted iterations, and exits early on a mate score.

**Forced-move short-circuit.** When `generate(pos)` yields exactly one legal action — overwhelmingly the mandatory phase-end `EndPhase` once `actions_remaining == 0` — the whole tree is skipped and that action is returned with `nodes: 1`. For a meaningful score/depth it first probes the TT at ply 0 (`score_from_tt(entry.score, 0)`, P1-POV so no sign flip) and reuses the stored score + depth; on a TT miss it falls back to the static eval with `depth: 0`. (ns-55 fix: it previously returned `depth: max_depth`, which surfaced as a bogus "d64" in the UI on every phase-end ply — since the UI maps an "unlimited depth" setting to `max_depth = 64` — and reported a bare static eval that broke the surrounding mate line. Pinned by `forced_move_does_not_report_max_depth`.)

**`SearchCtx`** is the per-search shared context: references to the `TranspositionTable`, `OrderingTables` (killers and history), the `Evaluator`, an optional deadline, a node counter, an abort flag, and the `acc_stack` (an incremental-eval accumulator stack for NNUE evaluators, zero-cost for the heuristic evaluator).

**`OrderingTables`** (~128 KB) holds `killers[ply][phase][2]` and `history[side][kind][from][to]`. Move ordering is applied at `depth >= 3` (below that, the TT-move swap-to-front alone is profitable; full sorting loses NPS at shallow depth).

**Pruning techniques active:**
- Null-move pruning (NMP) in the Skill phase only (the only natural "pass" in the game). Guarded by `can_null` to prevent consecutive null moves and by `NMP_MIN_PIECES = 6` as a zugzwang guard.
- Principal variation search (PVS): null-window re-search on non-PV nodes.
- Late move reductions (LMR): `0.75 + ln(depth) * ln(idx) / 2.25`, floored at 1.
- Late move pruning (LMP): schedule `{depth 1→6, 2→9, 3→13, 4→18, 5→24}` quiet moves maximum.

Time is checked every 1024 nodes via a `TIME_CHECK_MASK` bitmask to avoid the overhead of a system-clock call per node.

All five heuristics are individually toggleable at runtime via atomic bool flags (`DISABLE_QS`, `ENABLE_NMP`, `ENABLE_PVS`, `ENABLE_LMR`, `ENABLE_LMP`) for A/B benchmarking.

### 5.3 `src/search/quiescence.rs`

Quiescence search called at the `depth <= 0` boundary. Continues through "loud" actions — Move-Attacks, Strike-category skills (`Lance`, `Hook`, `Break`, `Steal`, `Tempest`), `Blast`, and `BodyguardChoice` — until the position is quiet, resolving the horizon effect.

`is_loud(action, pos)` classifies actions. `is_king_threatened(pos, side)` is a fast bitboard approximation (no full move generation): it checks Chebyshev distance for Move-Attacks in the Move phase and skill range x money for Strike/Blast skills in the Skill phase. Over-approximation is safe because false positives only force additional search.

`quiesce(pos, alpha, beta, ply, qs_ply, ctx)` implements stand-pat with a check-evasion gate. It builds a SEE-scored `[(i32, Action); 128]` list, sorts it by insertion sort, and iterates with recursive make/unmake calls. The cap is `MAX_QS_PLY = 8` within quiescence and `MAX_PLY = 128` absolute. The QS does not write to the transposition table and does not update killer or history heuristics.

### 5.4 `src/search/see.rs`

Static Exchange Evaluation (SEE) for move ordering in quiescence. Builds a per-square attacker table once per QS node, then scores individual captures by simulating alternating least-valuable-attacker exchanges.

`AttackersTable` holds physical-attacker bitmasks and skill-attacker bitmasks per square for both sides. `build_attackers_table(pos, all_occ)` fills both physical and skill columns; `build_attackers_table_phys(pos, all_occ)` fills only the physical columns, used on the evaluator hot path to avoid skill-ray tracing overhead.

`see_capture(pos, table, src, target) -> i32` simulates the full LVA exchange from the initiator's perspective with stand-pat fold-back: neither side is forced to take a losing step. Returns net material (positive = winning for the initiator). `see_single_hit(pos, target) -> i32` is a lighter variant for ordering Strike/Blast skills in QS.

`AttackerList` is a fixed-size (`SEE_MAX_ATTACKERS = 8`) insertion-sorted cheapest-first list — heap-free and stack-allocated for the hot path.

### 5.5 `src/search/transposition.rs`

A preallocated, single-slot-per-index transposition table (TT). Entries are 24 bytes: Zobrist key (8), score (4), best move (4), depth (1), bound flag (1), generation (1), padding (1).

`TranspositionTable::with_capacity_mb(mb)` allocates the largest power-of-two entry count that fits within the requested size. `probe(key)` returns `Some(&Entry)` on an exact key match. `store(entry)` replaces if the slot is empty, older generation, same key, or new depth ≥ existing depth. `new_search()` is an O(1) soft-clear that wraps the generation byte; `clear()` is an O(n) hard-clear. The `Match` object retains the TT across AI calls within a game so move ordering stays warm.

### 5.6 `src/search/counters.rs`

Thread-local diagnostic counters that compile to zero-cost no-ops unless the `bench_counters` feature is active. `Snapshot` holds ~20 `u64` fields covering eval calls, phase-gate fires, SEE calls, attacker-list histogram, alpha-beta node counts, and QS node counts. `snapshot()` / `reset()` are the only public read/write operations. The `search_bench` crate activates this feature unconditionally; the Tauri and nn_trainer builds never pay for it.

### 5.7 `src/search/evaluator/mod.rs`

The public façade for the evaluator subsystem. Defines the `Evaluator` trait, `HeuristicEvaluator`, `AccHandle`, and the free-function entry points `evaluate` and `evaluate_report`.

`Evaluator` is the seam between the search and any concrete evaluation function:

```rust
pub trait Evaluator: Send {
    fn evaluate(&self, pos: &Position) -> i32;
    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport;
    // Accumulator hooks — default to no-ops:
    fn uses_accumulator(&self) -> bool { false }
    fn fresh_acc(&self, pos: &Position) -> AccHandle { AccHandle::none() }
    fn clone_acc(&self, h: &AccHandle) -> AccHandle { AccHandle::none() }
    fn push_acc(&self, h: &mut AccHandle, undo: &Undo, pos: &Position) {}
    fn eval_acc(&self, h: &AccHandle, pos: &Position) -> i32 { self.evaluate(pos) }
}
```

`evaluate_report(pos, detail)` (ns-53) is the single dynamic-breakdown entry point, replacing the former fixed-struct `evaluate_breakdown`. `BreakdownDetail::{Aggregate, PerPiece}` selects whether the returned `EvalReport` also carries a per-piece decomposition. The default heuristic decomposes into its terms; an NN evaluator returns a single synthetic term (it has no term structure). `AccHandle(Option<Box<dyn Any + Send>>)` is a type-erased accumulator slot. The search threads it through make/unmake without knowing the concrete type inside. `HeuristicEvaluator` is a zero-size struct that returns `false` from `uses_accumulator()`, so the entire accumulator seam is dead code for the standard heuristic path.

`MATE_SCORE: i32 = 1_000_000` is defined here as the canonical sentinel.

**Selectable evaluators (ns-54).** Besides `HeuristicEvaluator`, this module defines `ParamHeuristicEvaluator { params: EvalParams }` — same term math, custom weights (the "same terms, different balance" shape) — and a `builtin` registry submodule. `builtin::BUILTINS: &[BuiltinEvaluator]` maps a stable `id` + display `label` to a constructor `fn() -> Box<dyn Evaluator + Send>`; `builtin::make(id)` resolves one. This is the single place `core_engine`'s own concrete evaluators are enumerated for UI selection — adding a new evaluator (a fresh struct with its own term registry, a tuned `ParamHeuristicEvaluator`, whatever) is "write the struct + one entry here." NN raters are NOT in this registry (they live in `nn_trainer`, which depends on this crate); the wrapper layer unions them in. A ready-to-edit scaffold ships as `custom.rs` / `CustomEvaluator` (registry id `"custom-stub"`, label "Custom (stub)") — it delegates to the heuristic out of the box and is the intended starting point for hand-rolled experimental evaluators.

### 5.8 `src/search/evaluator/params.rs`

All tunable evaluation weights in a single `EvalParams` struct (`Copy + Clone + Serialize + Deserialize`). `EvalParams::DEFAULT` reproduces the pre-ns-43 constants exactly and is enforced by a `golden_eval_unchanged` test. It carries the full weight set for every term — not just base material. Representative defaults: `champion_value = 1000`, `guard_value = 600`, `hp_per_point = 150`, `armor_per_point = 120`, `king_material = 0` (presence/absence is captured by the MATE branch, not material), `tempo_per_action = 15`, `money_per_unit = 25`, `offensive_range_weight = 500`, `wasted_modifier_per_cost = 25`. Beyond these it also holds the mobility block (per-square weights for guards/champions/king plus the champion skill-coverage cap), the exposure/king-exposure multiplier curves, the coverage weight, the guard-isolation and champion-threat blocks, the stage-classification thresholds, the endgame-closing block, the skill-availability sigmoid parameters, the ns-53 SEE-term weights (`hanging_penalty_pct`, `king_tempo_penalty`), and a 16-entry `skill_value` table indexed by skill ID. The struct is serialisable so an offline weight-tuner can perturb and reload candidates without recompiling.

### 5.9 `src/search/evaluator/context.rs`

`EvalContext` is built once per `evaluate()` call and then borrowed by every term in the evaluation, preventing each term from redundantly recomputing occupancy, attacker tables, or availability tables.

It holds: precomputed bitboards (`all_occ`, `p1_bb`, `p2_bb`, `p1_guards`, `p2_guards`), skill-availability fixed-point tables (`p1_avail / p2_avail: [i32; 16]`), the physical-only `AttackersTable` (`atk`), a lazily-built full `AttackersTable` (`atk_full`, a `OnceCell` — see below), the `GameStage` classification (`Opening | Mid | End`), actions-per-round, money caps, the current material advantage, and a reference to `EvalParams`.

`EvalContext::new(pos, params)` deliberately calls `build_attackers_table_phys` (physical scatter only) to avoid skill-ray tracing overhead on the ~3.37M calls-per-sweep eval hot path. The full table (physical + skill scatter, needed by `see_capture`) is built lazily by `atk_full()` — a `OnceCell` populated on first use. The ns-53 SEE terms consult `atk_full()` only after the cheap physical `atk` table confirms a piece is actually attacked, so a quiet leaf position never triggers the full-table build.

`skill_availability_fp(money, cost, params) -> i32` implements the piecewise-linear sigmoid used by the Skills term. `classify_stage(total_material, round_number, params) -> GameStage` applies a round-number bias so games that drag on are treated as later-stage even if material is still high.

### 5.10 `src/search/evaluator/registry.rs`

The term registry and the two evaluation drivers.

`evaluate_scalar(pos, params) -> i32` is the search leaf hot path: no heap allocation, no `dyn` dispatch, static dispatch throughout. It calls `EvalContext::new`, then `accumulate_terms` (a single shared board pass via `score_piece_all` and `score_side_all`), then `TermSums::fold_total`.

`evaluate_report(pos, params, detail) -> EvalReport` (ns-53) is the allocation-heavy variant used by the frontend eval panel and any tuning. It reuses the SAME term math: the aggregate per-side sums come from `score_piece_all` / `score_side_all`, and — when `detail == PerPiece` — the per-piece rows are built from the *same* `score_piece_all` call in the same board pass. This is what let the old hand-duplicated `evaluate_by_square` be deleted: there is now exactly one implementation of the term math, so the per-piece view is definitionally consistent with the scalar total rather than test-enforced. `PIECE_TERM_NAMES` and `PIECE_TERM_SIGN` (parallel to the `pt::*` indices) give the report stable term names and the signs that must stay in lockstep with `fold_total`.

`default_terms` / `default_terms_static()` (the boxed term set, cached via `OnceLock`) are retained for a future weight-tuner / preset path but are no longer on the scalar or report hot paths — both accumulate through the monomorphic `score_piece_all` / `score_side_all` helpers.

`TermSums` accumulates per-piece scores (9 terms, indexed by `pt::*` constants) and side-level scores (5 terms). `fold_total` is the single point where all magnitudes are signed and summed — exposure and guard-isolation are subtracted as penalties; offensive range is weighted.

### 5.11 `src/search/evaluator/term.rs`

Defines the `EvalTerm` trait — the unit interface every evaluation term implements. Each term is a zero-size struct that implements exactly one of two shapes:

- **Per-piece** (`is_per_piece() -> true`): `score_piece(ctx, pc) -> i32`, called once per occupied square.
- **Side-level** (`is_per_piece() -> false`): `score_side(ctx) -> (i32, i32)`, returning `(p1_score, p2_score)` in one pass.

`signed_total(p1, p2, params) -> i32` is overridable — terms that are penalties override it to return `-(p1 - p2)`, while the offensive-range term multiplies by a weight. This keeps the registry term-agnostic.

`PieceContext` delivers precomputed per-square inputs to per-piece terms: square index, owner, kind flags, and the `MailboxEntry`.

### 5.12 `src/search/evaluator/terms.rs`

All 16 evaluation term implementations.

**Per-piece terms (10):**

| Term | What it measures |
|---|---|
| `Material` | Base piece value (`champion_value` / `guard_value`) |
| `Hp` | Current HP as a fraction of `hp_per_point` |
| `Armor` | Current armor x `armor_per_point` |
| `Skills` | Weighted average of equipped skill values x their availability sigmoid |
| `Mobility` | Reachable empty squares (champions/king) — changed in ns-43 from "enemies in range" to separate threat from reach |
| `Exposure` | Penalty: unshielded-attacker count (0-3) indexes `exposure_mult`; kings use the steeper `king_exposure` curve |
| `Coverage` | Guards shielding a champion/king in directions where an enemy is approaching |
| `GuardIsolation` | Penalty: a Guard with more enemies than friendlies within radius 2 |
| `ChampionThreat` | Offensive: strike/movement skills pointing at enemy targets; defensive: support skills pointing at ally targets — both soft-capped |
| `HangingPiece` | Penalty (ns-53): a non-king piece that is *currently attacked* and SEE-losing where it stands. **SEE-gated** — only pieces with an enemy in the cheap physical-attacker table run the exchange rollout; quiet pieces cost nothing |

**Side-level terms (6):**

| Term | What it measures |
|---|---|
| `Money` | Useful money (quadratic ramp to a cap, plateau above it) |
| `Tempo` | Actions remaining x `tempo_per_action` |
| `OffensiveRange` | Max strike/Shove range including optional Focus +1, weighted by `offensive_range_weight` |
| `WastedModifier` | Penalty when Focus or Charge is pending but no castable consumer exists this phase |
| `EndgameClosing` | Active only in End stage with a material lead: leader drives king pressure and escape denial; trailer maximises king safety and compactness |
| `KingTempo` | Penalty (ns-53): a side whose king is one loud enemy action from capture, via the cheap `is_king_threatened` bitboard scan (no exchange rollout) |

`Coverage` gained a threat gate during ns-43 to prevent rewarding guards that shield directions with no approaching enemy. `EndgameClosing` is intentionally asymmetric: the leading side plays to close; the trailing side plays to stall. `HangingPiece` and `KingTempo` (ns-53) are the "eval may look at forced tactics of the current moment" terms — both are gated so the eval hot path only pays for SEE where a real threat exists (the `see_tables` bench counter is ~0 on quiet positions and rises with tactical density).

### 5.13 `src/search/evaluator/breakdown.rs`

Defines the dynamic breakdown types (ns-53). `EvalReport` replaced both the old fixed 25-field `EvalBreakdown` struct AND the hand-duplicated `evaluate_by_square` per-square view.

`EvalReport` is `Clone + Serialize + Deserialize` — the wire format sent to the frontend eval panel. It carries `terms: Vec<TermEntry>` (aggregate per-piece terms, active only), `side_terms: Vec<TermEntry>` (money/tempo/offensive-range/…), `pieces: Option<Vec<PieceTermBreakdown>>` (present only when `PerPiece` was requested — one row per occupied square), a P1-POV `total`, and a `terminal` flag. `TermEntry` is `{ name: String, p1, p2, signed }` where `signed` already carries the term's sign/weight (a penalty reads negative; offensive-range is pre-multiplied). `PieceTermBreakdown` carries the square, owner, kind, HP/armor/skill IDs, its per-term rows (owner-signed), and the piece's owner-signed `piece_total`. Constructors: `EvalReport::terminal(total)` and `EvalReport::single(name, total)` (the one-term report NN evaluators return).

`EvalReport` is deliberately NOT persisted in match logs — the per-ply breakdown fields were removed as redundant (a log stores each position's FEN + scalar eval, so any term decomposition is recomputable live). For non-terminal positions the invariant `Σ pieces[*].piece_total + Σ side_terms[*].signed == total` holds and is pinned by a test.

### 5.14 `src/search/evaluator/incremental.rs`

`IncrementalEvaluator` is a stateful `Evaluator` that caches the per-square term decomposition, intended to diff only the changed squares on incremental updates rather than re-scoring the full board. The Phase-1 implementation (current) always does a full rebuild via `EvalCache::rebuild` — the cache scaffold is in place but the diff path is not yet activated (`ENABLE_INCREMENTAL_EVAL: AtomicBool` defaults to false). The cache stores per-square term magnitudes, all position scalars, and the last Zobrist hash so stale entries are detectable. Phase 2/3 will activate the `subtract old square / add new square` path. (Caveat noted in-file: the ns-53 `hanging_piece` term is not square-local, so activating the diff path must recompute it — and `king_tempo` — per changed neighbourhood rather than treating them as square-local.)

### 5.15 `src/search/evaluator/custom.rs`

`CustomEvaluator` — a ready-to-edit scaffold for a hand-rolled evaluator, registered in `builtin::BUILTINS` as `"custom-stub"` so it appears in the setup / settings dropdowns immediately. Out of the box its private `score(pos)` delegates to `HeuristicEvaluator`, so selecting it plays normally; the file's doc comment walks through the two edit points — swap `score` for your own P1-POV math, and (optionally) build a richer `EvalReport` in `evaluate_report` (pushing your own `TermEntry` / `PieceTermBreakdown` rows) so the dynamic eval panel renders your components. It is a standalone struct, not bound to the shared term registry — the intended home for genuinely different evaluation logic. Ship more variants by adding more structs + `BUILTINS` entries.

---

## 6. crates/core_engine — Layer 4: Session

### 6.1 `src/session.rs`

`Match` is the central object of Layer 4. It owns a `Position`, the full `Vec<(Action, Undo)>` history, a `Config`, a `TranspositionTable` (retained across AI calls within a game for warm move ordering), an optional `MatchLog`, and a `Box<dyn Evaluator + Send>` that can be hot-swapped to a trained rater at any point.

**Configuration.** `Config` has four named presets (`local_hvh`, `local_hvai`, `local_aivai`, `networked_hvh`) and holds `SeatKind` (Human or AI) per seat, `AiBudget` (time limit in ms + max depth) per seat, an optional step delay for AIvAI display, an undo flag, and an auto-log flag. `AiBudget::default()` is 1000 ms / depth 6.

**Construction.** `Match::new(config)` opens at the Stack-M starting position. `Match::new_with_draft` opens in `Phase::Draft`. `Match::new_with_loadouts` skips the draft entirely. `Match::from_snapshot(s)` replay-validates every action in the snapshot — if any action is illegal the load fails, acting as a tamper-detection mechanism.

**Play.** `try_apply(action)` checks legality, calls `make_unmake::make`, and appends to history. `try_apply_timed(action, thought_ms, applied_at_unix_ms, ai)` is the instrumented variant that additionally calls `telemetry::snapshot_pre`/`snapshot_post` and appends a `PlyRecord` to the `MatchLog`. `undo_last()` calls `make_unmake::unmake` and pops history.

**AI.** `request_ai_move()` runs the search but does not apply the result — the caller decides whether to apply it. `step_ai()` searches and immediately applies the best move. `request_ai_move_with_cb(on_depth)` fires a callback per iterative-deepening depth for streaming depth updates to the UI. `request_ai_move_forced()` ignores seat kind, used by the inspector.

**Notable design decisions.** `Match` never reads system time — all clocks are caller-supplied (`now_unix_ms` parameters). The engine is platform-agnostic and deterministic given the same inputs. The `NetworkTransport` trait (`send(ApplyEvent)` / `poll() -> Option<ApplyEvent>`) and its no-op `LocalTransport` implementation are the seam for multiplayer — the session layer does not know about WebSockets.

---

## 7. crates/core_engine — Layer 5: Telemetry

### 7.1 `src/telemetry.rs`

Pure data-shaping for per-ply telemetry. The engine never writes files; it only populates in-memory structures that the wrapper layers serialise.

`PlyRecord` is a per-action record: ply number, seat, timing, `ActionDecoded` (action unpacked to human-readable fields), legal-action count, pre/post Zobrist, pre/post FEN, pre/post **scalar** static eval, post-position state (phase, round, money, modifiers, combo tracking), and an optional `SearchMeta` (depth, nodes, score, mate distance). As of ns-53 it no longer stores a per-term breakdown: the former `prev_breakdown`/`post_breakdown` fields (and `SearchMeta.post_move_breakdown`) were removed as redundant — a consumer that wants the term decomposition recomputes it live from the stored FEN via `evaluate_report`.

`ActionDecoded` carries a `notation: String` field — the canonical action notation string (e.g. `"a1-b2"`, `"b2*d4:Tempest"`) populated by `session.rs::try_apply_timed` **before** `make()` is called, while `pending_bodyguard` is still live. This is the only point where `BodyguardChoice` redirects can be resolved to a guard square rather than a numeric index fallback. `#[serde(default)]` keeps legacy log JSON (no `notation` field) loading cleanly. The `notation::to_text` function uses this field directly; there is no second independent action renderer in the telemetry layer.

`MatchLog` collects all `PlyRecord`s for a game alongside metadata: engine version, start time, `Config`, a stable `config_hash` (FNV-style, not `DefaultHasher`, for cross-process stability), start FEN/Zobrist, final result/FEN/Zobrist, and aggregate node/time totals.

`Bundle = Vec<MatchLog>` is the multi-match export format written by the Tauri app when the user exports their library.

`snapshot_pre(pos)` and `snapshot_post(pos)` are called by `session::Match::try_apply_timed` to capture before-and-after fingerprints without allocating the full record until after the action resolves.

All serde fields carry `#[serde(default)]` for forward-compatibility — logs saved with an older engine version load cleanly even when new optional fields are added.

---

## 8. crates/core_engine — Public API Surface

### 8.1 `src/lib.rs`

The crate entry point. Re-exports the public surface from all five layers so downstream crates only need to `use core_engine::…`. The module-level comment documents the five-layer stack and calls out the platform-free design constraint. No logic lives here.

Action notation is re-exported at the crate root: `action_to_notation`, `notation_to_action`, and `NotationError` from `state::action_notation`. This means all four consumers (`tauri_wrapper`, `search_bench`, `nn_trainer`, `aivai_demo`) import through the same path without reaching into sub-modules.

### 8.2 `src/wrapper_api.rs`

A flat, stateless façade that both `tauri_wrapper` and any future WASM build call into. It translates their boundary types into `Match` operations and keeps both wrappers in lockstep.

Hot-path functions return flat, allocation-minimising types: `PositionView` (`#[repr(C)]`) contains the five bitboards and all scalar fields in one struct; `legal_actions_into(m, buf)` fills a caller-owned `Vec<u32>` rather than allocating. `position_mailbox(m) -> &[u16; 64]` is a zero-copy unsafe reinterpret cast relying on `MailboxEntry` being `#[repr(transparent)]` over `u16`.

Cold-path functions (snapshot, log, eval breakdown) return owned `String`. The pattern throughout is: the caller owns allocation, the wrapper borrows.

Key functions beyond the hot path: `new_match_with_draft`, `new_match_with_loadouts`, `from_snapshot_json`, `snapshot_json`, `match_log_json`, `latest_ply_json` (incremental — returns only the last ply to avoid O(n²) re-serialisation), `finalise_log`, `eval_report` (dynamic `EvalReport`; `per_piece` flag; uses the Match's installed evaluator), `current_draft_state`, `step_ai_with_cb` (with per-depth callback).

### 8.3 `src/time.rs`

A platform-conditional monotonic clock. `now_ms() -> u64` returns milliseconds since process start on native (using a `OnceLock<Instant>`) and calls an extern C import `engine_now_ms()` on `wasm32`. The search uses this to enforce time budgets without knowing which platform it is running on. On wasm32, failing to provide the import causes a load-time instantiation failure — the preferred failure mode over a mid-search panic.

---

---

## 9. crates/tauri_wrapper

The desktop app shell. Exposes `core_engine` to the SvelteKit frontend via Tauri 2 `invoke()` commands, and hosts the Training Observatory IPC surface for `nn_trainer`.

### 9.1 `src/main.rs`

Binary entry point. Sets two Linux-specific WebKit environment variables before delegating to `lib::run()`: `WEBKIT_DISABLE_DMABUF_RENDERER=1` (fixes CSS animation lag on wlroots compositors) and `WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1` (works around an AudioContext failure caused by PipeWire socket blockage — upstream WebKit bug #239682).

### 9.2 `src/lib.rs`

The entire command surface lives here. Engine state is held in `EngineRegistry` — a process-global `AtomicU64` handle counter and a `Mutex<HashMap<u64, EngineEntry>>`. Each `EngineEntry` holds a `Match` and a reusable `legal_buf: Vec<u32>`. Handles are opaque `u64` IDs issued on `create_engine` and freed on `drop_engine`.

All CPU-bound AI commands (`step_ai`, `request_ai_move_forced`, `request_ai_move_at_depth`) run inside `tokio::task::block_in_place` to avoid pinning the async executor. `step_ai` emits `"ai-depth-update"` Tauri events per iterative-deepening depth (via `step_ai_with_cb`) so the frontend can show live depth progress; `request_ai_move_at_depth` is the inspector's forced fixed-depth variant. A separate `AivaiProducerState` drives the background AI-vs-AI producer (`start_aivai_producer` / `aivai_producer_log` / `stop_aivai_producer`), which streams a self-play game to the UI (throttled `"aivai-progress"` events) without blocking the engine registry.

Training state is held in `TrainingState` — a `Mutex<TrainingInner>` with an optional stop-flag `Arc<AtomicBool>` and `JoinHandle`. `start_training_run` spawns a background thread; `stop_training_run` signals the flag; `RunEvent::ExitRequested` calls `signal_stop` for a clean final snapshot on app exit.

**Game-engine commands (selected):**

| Command | Purpose |
|---|---|
| `create_engine` / `create_engine_with_draft` / `create_engine_with_loadouts` / `create_engine_from_snapshot` | Instantiate or restore a `Match`; returns a `u64` handle |
| `drop_engine` | Free the handle |
| `position_view` / `legal_actions` / `position_fen` / `phase_state` | Read current state |
| `fen_to_position_view` | Stateless: decode a FEN string to a `PositionView` (Position Builder) |
| `action_to_notation_cmd` | Stateless: encode a raw `u32` action to canonical notation string; no handle required |
| `skill_metadata` / `game_constants_cmd` | Stateless: engine-canonical skill and rule constants for the frontend contract mirror |
| `try_apply` | Apply one action |
| `step_ai` / `request_ai_move_forced` / `request_ai_move_at_depth` | AI search + apply; `request_ai_move_at_depth` streams depth via events |
| `evaluate_draw_offer` | Engine assessment of a proposed draw |
| `snapshot_json` / `match_log_json` / `latest_ply_json` / `finalise_log` | Snapshot and log serialisation |
| `eval_report` | Eval panel data — dynamic `EvalReport` (`per_piece` flag). Uses the Match's installed evaluator, OR a transient one built from an optional `source`/`id` (the UI-eval pick, `settings.uiEvaluator`) without disturbing the AI seats |
| `draft_state` | Draft-phase view |
| `engine_version` | Engine version string for the menu |
| `start_aivai_producer` / `aivai_producer_log` / `stop_aivai_producer` | Background AI-vs-AI self-play producer lifecycle |
| `set_ai_evaluator` | Install an evaluator (builtin or trained rater) as the AI for one match |

**Training Observatory commands (selected):**

| Command | Purpose |
|---|---|
| `list_available_raters` | Unions the active run directory and `game/raters/blessed/` (NN raters only) |
| `list_evaluators` | Unified evaluator list feeding all UI dropdowns: `core_engine`'s builtin evaluators first, then `list_available_raters`'s NN raters |
| `list_backends` | Enumerate available training backends (Cpu / feature-gated GPU) |
| `default_run_dir` | Default training-run output directory |
| `set_ai_evaluator` | Dispatches to `NnueEvaluator` or `NnEvaluator` based on `model_config.input_dim` |
| `start_training_run` / `stop_training_run` | Background thread lifecycle |
| `read_training_status` / `read_training_live` | Poll snapshot and per-ply live files |
| `subscribe_training_live` / `unsubscribe_training_live` | Gate for the per-ply live writes |
| `inspect_rater` | Load + forward pass + weight stats for the Observatory inspector |
| `read_rater_index` / `read_gauntlet_matrix` | Read training artefacts |

Both `set_ai_evaluator` and `eval_report` route through the shared `build_evaluator(source, id, run_dir)` dispatch: `"builtin"` (and legacy `"heuristic"`) resolve against `core_engine::search::evaluator::builtin::make`; `"run"` / `"blessed"` load a trained rater and discriminate dense vs NNUE topology by checking `metadata.model_config.input_dim == NUM_FEATURES` (3352) versus `INPUT_DIM` (2825). This avoids matmul dimension panics when a rater trained on the old dense encoder is loaded. `build_evaluator` is also reused by the background AIvAI producer (which re-installs seat evaluators after `from_snapshot` resets them to the default heuristic).

### 9.3 `build.rs`

Calls `tauri_build::build()` to generate the Tauri context — resource embedding and capability manifests — at compile time. No custom logic.

---

---

## 10. crates/search_bench

### 10.1 `src/main.rs`

A CLI benchmark harness that calls `find_best_with_evaluator` directly against raw `Position` values, bypassing `Match` and `wrapper_api` entirely. Four modes:

- **`--depth N`** (fixed depth): runs N times per position, reports the median.
- **`--time-ms T`** (time-budgeted): runs once per position with a T-millisecond budget.
- **`--determinism`**: asserts node counts and best moves are identical across N runs; exits with code 3 on failure.
- **`--eval-only`**: pure evaluator throughput benchmark (ns/eval, geometric mean).

Supporting flags tune the run without changing mode: `--corpus <path>`, `--runs N` (repetitions per position), `--out <file>`, `--eval-iterations N` and `--determinism-runs N`, and `--eval heuristic|nnue` (selects the search-leaf evaluator for the ns-50 NNUE A/B comparison).

Each run gets a fresh 64 MB `TranspositionTable`. Median (not mean) is used for per-position timing to reject OS-scheduling outliers. `action_brief(a: Option<Action>) -> String` formats the best move in output records using `core_engine::action_to_notation(act, None)` — the same canonical notation as the engine's telemetry layer.

The corpus is loaded from `bench/corpus/corpus.txt` via `from_fen`. The `bench_counters` feature is activated unconditionally in the `search_bench` Cargo manifest, so `counters::snapshot()` is available after every timed run. Structured JSON output includes aggregate blocks (geometric mean NPS, TT hit rate, EBF, counter snapshots) and per-position records.

The `--eval-only` mode optionally builds an `NnueEvaluator` from `nn_trainer` to compare NPS against the heuristic evaluator. Correctness regressions (score drift or best-move disagreement at fixed depth) exit with code 4.

---

---

## 11. crates/nn_trainer

### 11.1 Training Pipeline Overview

The training system has two phases:

**Phase 0 — Bootstrap.** A corpus of ~100k game positions is generated via search-driven self-play (depths 2/3/4, deduped by Zobrist hash). Each position is labelled with the heuristic evaluator's centipawn score. A small MLP (topology `3352 → 128 → 32 → 32 → 1`) is trained via MSE regression to reproduce those scores. The trained float model is quantised once into a `QuantizedNet` of integer tables. This is the first champion rater.

**Phase 1 — Mutation self-play.** Each iteration takes the current champion, applies Gaussian weight perturbation, and pits the candidate against the champion in a mirrored best-of-3 series. If the candidate wins, it becomes the new champion. No gradient descent occurs in Phase 1; selection pressure comes entirely from game outcomes.

At inference time, the `NnueEvaluator` wraps the `QuantizedNet` and maintains an incremental `Accumulator` on the search stack, so each leaf evaluation costs only a handful of integer dot products rather than a full forward pass.

```
Corpus generation
  └─► label with evaluate()  ──► Phase-0 training  ──► QuantizedNet  ──► v0001 champion
                                                                              │
                                              ┌───────────────────────────────┘
                                              ▼
                                   perturb weights → candidate
                                              │
                                        mirrored BO3
                                              │
                                    win? → new champion (vNNNN)
                                    lose? → discard
```

### 11.2 `src/lib.rs`

Crate root and re-export hub. Exposes the full public API to `tauri_wrapper` and `search_bench`. Contains one integration smoke test: `end_to_end_forward_on_stack_m_start_position` encodes the start position and runs a forward pass, catching `INPUT_DIM` / topology mismatches at build time.

### 11.3 `src/backend.rs`

Backend type aliases and the `BackendChoice` runtime enum. `InferenceBackend = NdArray<f32>` is always available and used for all forward-only paths. `TrainingBackend = Autodiff<InferenceBackend>` is the CPU training backend. `WgpuTrainingBackend` and `CudaTrainingBackend` are feature-gated. GPU vs CPU is a compile-time type-system distinction in Burn; runtime dispatch requires `BackendChoice` matching into a different monomorphised `run_training_*` function. Currently only `Cpu` is supported in the mutation self-play path.

### 11.4 `src/encoding.rs`

Dense f32 position encoder. `encode_position(pos) -> Vec<f32>` produces a flat vector of length `INPUT_DIM = 2825`. Per-square block (2816 values): owner (2), kind (3), HP one-hot (3), armor one-hot (3), skill1 one-hot (16), skill2 one-hot (16), combo scalar (1). Global block (9 values): money pair, side-to-move 2-hot, phase 3-hot, round, actions remaining. This is the v1 dense encoder used by `NnEvaluator`; the NNUE path uses `sparse.rs` instead.

### 11.5 `src/sparse.rs`

Sparse binary feature encoder for the NNUE-style accumulator. Rather than a dense vector, `encode_sparse(pos, out)` fills `out: &mut Vec<u32>` with the indices of active features — the "hot" dimensions in what would be a `NUM_FEATURES = 3352` one-hot vector.

`NUM_FEATURES = 3352` breaks down as `BOARD_BLOCK = 3328` (64 squares x 52 features per square) plus `GLOBAL_BLOCK = 24`. The per-square encoding differs from the dense path: combo is an 8-bucket one-hot (8 values) rather than a scalar (1 value).

`global_indices(to_move, phase, p1_money, p2_money, round_number, actions_remaining, push)` takes raw scalars rather than a `&Position` reference so that `accumulator.rs` can reconstruct pre-make global feature indices from an `Undo` without requiring the pre-make position. This is the critical invariant that enables `Accumulator::revert`.

`ACCUM_WIDTH = 128` is declared here and must equal `MlpConfig::hidden_sizes[0]`; this is asserted in `QuantizedNet::from_mlp`.

### 11.6 `src/accumulator.rs`

The incremental first-layer accumulator — the central data structure enabling sub-microsecond NN evaluation per search node.

`FeatureTransform` holds the weights of the first linear layer in column-major layout: `weights: Vec<[i16; ACCUM_WIDTH]>`, one column per feature. `Accumulator` holds the running sum `acc: [i32; ACCUM_WIDTH]` (bias + all active feature columns) and `globals: Vec<u32>` (cached current global feature indices for diff computation).

Three operations:
- `Accumulator::refresh(pos, ft)` — full recompute oracle; independent of any prior state.
- `Accumulator::apply(&mut self, undo, pos, ft)` — incremental forward after `make`. Uses `TouchedSet` (a stack-allocated dedup structure) to find which squares changed, subtracts old feature columns, adds new ones, and diffs the global features.
- `Accumulator::revert(&mut self, undo, pos, ft)` — incremental reverse for `unmake`. Provided as an alternative to the save-and-restore clone strategy.

The correctness invariant — `apply(undo, pos, ft) == refresh(pos, ft)` at every node — is maintained structurally: both paths call the same `sparse::square_features` and `sparse::global_indices` helpers with the same index math. It is additionally verified by a golden test that walks 120 random make/unmake steps and asserts bit-identity.

The inner column update loop (`add_col_i16` / `sub_col_i16`) is a plain scalar widening add/sub of i16 into i32; the compiler autovectorises this. A manual `wide` version was measured to be slower.

### 11.7 `src/quantized.rs`

Integer forward pass for the NNUE tail layers. Bypasses all Burn/autograd overhead during search — a Burn forward pass was measured at ~382x the hand-crafted integer path.

Weights are quantised to i16 in output-major layout, padded for lane-aligned SIMD via `wide::i16x8`. The quantisation scales are `QA = 1024.0` for the feature-transform layer (fixed-point shift `QA_SHIFT = 10`) and `QW = 64.0` for the tail layers. Clipped-ReLU ceiling is `CR_MAX = 8192`.

`QuantizedNet::from_mlp(model, scales)` quantises a trained float model once into integer tables. `QuantizedNet::forward_int(acc)` is the hot path: reads the accumulator, applies clipped-ReLU dequantisation, passes through `l1 → l2 → out` with SIMD dot products, scales to centipawns, and clamps to `±MAX_NN_SCORE`.

### 11.8 `src/model.rs`

Defines `MlpConfig` and `Mlp<B: Backend>` — the configurable Burn MLP. Default hidden topology after ns-50: `→ 128 → 32 → 32 → 1`. The `input_dim` is config-driven, not fixed: the dense path defaults to `INPUT_DIM = 2825`, while the sparse/NNUE path is built with `NUM_FEATURES = 3352` — do not conflate the two. `hidden_sizes[0]` must equal `ACCUM_WIDTH = 128`. `Mlp::layer_params()` extracts raw `(weight, bias, in_dim, out_dim)` tuples for quantisation. `Mlp::weight_stats()` returns per-layer `LayerStats` for the Training Observatory network inspector.

### 11.9 `src/nn_evaluator.rs`

`NnEvaluator` wraps `Mlp<InferenceBackend>` with a calibrated centipawn scale. The dense path: `encode_position` (2825 f32) → Burn tensor → `Mlp::forward` → raw f32 → `nn_output_to_centipawns`. This path is retained for raters trained on the dense encoder but is superseded by `NnueEvaluator` for search performance.

`NnEvaluator::load_from_stem(stem)` loads the `.mpk` weight file and reads `eval_scale` from the `.json` sidecar. `inspect_fen_at_stem(stem, pos)` returns `(raw_score, scale, weight_stats)` in a single call for the Tauri rater inspector.

### 11.10 `src/nnue_evaluator.rs`

`NnueEvaluator` wraps `QuantizedNet` and implements the full `Evaluator` accumulator seam. It overrides `uses_accumulator()` to return `true` and provides `fresh_acc`, `clone_acc`, `push_acc`, and `eval_acc` so the search can maintain incremental state through make/unmake.

The fallback path (`eval_acc` on downcast failure) calls `Accumulator::refresh` per leaf — slower but always correct, never panics. The seam is verified by `search_incremental_matches_refresh_per_call`: `find_best_with_evaluator` at depths 2/4/6 must produce bit-identical `(best, score, nodes)` with and without the accumulator active.

### 11.11 `src/train.rs`

Core training utilities. `batch_to_tensors` (dense encoder) and `sparse_batch_to_tensors` (sparse encoder with `scatter_dense`) convert example batches into Burn tensors. `train_step` runs a forward pass, computes MSE loss, backpropagates, and applies an optimizer step. `train` is the epoch loop. `into_inference` strips the autograd graph via `model.valid()`.

### 11.12 `src/bootstrap.rs`

Phase-0 supervised bootstrap. `label_corpus(text)` parses a FEN corpus and labels each position with `evaluate()`. `train_scalar(corpus, config)` runs the gradient-descent loop. `bootstrap(corpus, config)` trains then immediately quantises, producing the first `QuantizedNet`. `mean_abs_error_cp(net, corpus)` reports MAE in centipawns using the integer forward path. `LABEL_DIVISOR = 1000.0` normalises centipawn labels to ~[-1, 1] for training and is folded into `QuantScales::out` so `forward_int` returns centipawns at inference.

### 11.13 `src/corpus_gen.rs`

In-process corpus generation for Phase-0. `generate_training_corpus(target, n_games, seed)` runs search-driven self-play (cycling through depths 2/3/4 across games for divergence), deduplicates by Zobrist hash and FEN view-key, and stops at `target` positions. Each game uses a seeded `ChaCha8Rng` and its own 16 MB TT, so the output is fully reproducible. `write_training_corpus_file` writes one FEN per line.

### 11.14 `src/batch.rs`

Rayon-parallel corpus generation. `generate_corpus(n_games, seed_base, rater_p1, rater_p2, max_depth)` maps game index `i` over a parallel iterator with `seed_p1 = seed_base + 2i` and `seed_p2 = seed_base + 2i+1`, calls `play_game` on each, and flat-maps the results. Output order is deterministic (Rayon preserves index order on collection).

### 11.15 `src/selfplay.rs`

Single-game self-play driver. Lower overhead than `session::Match` — no telemetry, no snapshots, no draft machinery. `play_game(rater_p1, rater_p2, loadout_p1, loadout_p2, max_depth)` uses `find_best_with_evaluator` with `time_limit_ms = 0` (fixed depth for reproducibility). Returns `None` if the game exceeds `MAX_PLIES = 250`. `GameRecord::into_labelled` broadcasts the game outcome (±1.0) to all recorded positions.

### 11.16 `src/loadout.rs`

Deterministic `SideLoadout` generator. `random_loadout(rng)` fills 12 slots in shuffled order, enforcing per-skill caps and requiring at least 3 of 4 categories and at least 1 Strike skill. This ensures self-play games have varied but realistic drafts. `random_loadout_from_seed(seed)` wraps `ChaCha8Rng` for reproducible generation.

### 11.17 `src/gauntlet.rs`

Selection layer. `accept_vs(candidate, champion, loadout_seed, time_ms) -> Acceptance` plays a mirrored BO3 series (candidate as P1 then P2 on the same loadout; tiebreaker on a fresh loadout if 1-1). Games adjudicate at `MAX_PLIES = 250` via the heuristic evaluator. `ChampionTracker::consider(id, win_rate)` updates the champion pointer if the win rate exceeds the current floor; the first candidate always wins.

### 11.18 `src/lineage.rs`

Perturbation injection and parallel training lineages. `GaussianNoiseMapper` is a Burn `ModuleMapper` that adds seeded Gaussian noise to all float parameters. `perturb_model(model, std_dev, seed, device)` applies it reproducibly. `Lineage` holds a model, a seed, and a loss history; `train_burst` runs a fixed number of gradient steps on a corpus chunk. `train_lineages` drives a population-based round structure: each round trains each lineage, generates a perturbed candidate, trains the candidate, and keeps it if its validation loss is lower. The sequential-across-lineages constraint exists because the Burn autodiff backend is not `Send`-friendly.

### 11.19 `src/lineage_checkpoint.rs`

Crash-recovery for Phase-0 gradient training. `save_lineages` persists each lineage model via `save_rater` and writes an atomic umbrella sidecar. `load_lineages` restores the population on resume, validating format version and config digest to reject stale checkpoints from a different run configuration. `quarantine_stale` renames mismatched checkpoints rather than deleting them.

### 11.20 `src/run.rs`

Top-level mutation self-play orchestrator. `run_training(config, run_dir, should_stop, backend)` is the entry point called by the Tauri command. The main loop: seed or resume the champion, then for each iteration perturb the champion, quantise the candidate to a `NnueEvaluator`, run `accept_vs_live` (mirrored BO3 with per-ply live writes), and on acceptance persist the new rater and update the index.

`accept_vs_live` fires `write_live` after each ply if the Training Observatory is subscribed, streaming the live board to the UI without blocking when nobody is watching. All IPC file writes use atomic `.tmp` + rename to prevent partial reads.

### 11.21 `src/persistence.rs`

Versioned rater save/load. A rater on disk is a pair of files: `<stem>.mpk` (Burn msgpack weight blob) and `<stem>.json` (human-readable `RaterMetadata` sidecar). `load_rater` reads the sidecar first to determine topology, initialises the model skeleton via `MlpConfig::init`, then fills weights from the recorder. `load_metadata` reads only the sidecar — used by `tauri_wrapper` to dispatch dense vs NNUE without loading the full weight blob. All `RaterMetadata` fields use `#[serde(default)]` for forward-compatibility.

### 11.22 `src/registry.rs`

Append-only `index.json` tracking all accepted raters and the current champion. `RaterIndex` enforces two invariants: entries are append-only (no deletion), and track pointers (`BTreeMap<Track, String>`) must reference existing entry IDs. `Track` has a single variant `Champion` (the three-track system was retired in ns-50).

### 11.23 `src/calibration.rs`

Slope-only OLS regression to fit a centipawn scale factor `k` for a newly trained rater. `calibrate_rater(model, heuristic, probes)` computes `k = Σ(nn_raw·cp) / Σ(nn_raw²)` over probe positions, dropping terminals and non-finite outputs. The no-intercept constraint is correct because the network should be sign-symmetric. The result is stored in `RaterMetadata::eval_scale`.

### 11.24 `src/snapshot.rs`

~1 Hz training status IPC. `StatusSnapshot` captures phase, generation, round, ETA, population members, and the currently active match. `write_snapshot` stamps `written_at_ms` and uses atomic rename. The Tauri `read_training_status` command polls this file every second for the Training Observatory status panels.

### 11.25 `src/live.rs`

Per-ply board state IPC for the Live Match View. Subscription-gated: `is_subscribed(dir)` checks for a `live.sub` sentinel file so the training loop avoids the file-write overhead when no UI is watching. `write_if_subscribed(dir, live)` writes `live.json` only when subscribed. `LivePosition` carries the current FEN, last action, ply, game index, and eval bars (challenger NN score, defender NN score, heuristic score).

### 11.26 `src/matrix.rs`

Gauntlet match-matrix persistence. `GauntletMatrix` stores a flat `Vec<MatrixEntry>` recording challenger x defender x bracket results. `record_series` accumulates into an existing cell or inserts a new one. All reads and writes use atomic rename. The matrix is displayed in the Training Observatory's Matrix tab and is also used as a data source for future tuning analysis.

---

---

## 12. frontend

The UI is a SvelteKit 5 application rendered inside the Tauri webview. SSR is disabled; all routing is client-side file-based. State is managed with Svelte 5 runes (`$state`) plus `localStorage` for settings and IndexedDB for match history and saved loadouts. Since this doc was last written, a large amount of presentational surface has been extracted out of the routes into shared component directories (`$lib/match/`, `$lib/eval/`, `$lib/multiplayer/`, `$lib/ui/`), and a new Position Builder route was added.

### 12.1 Routing Structure

| Route | LOC | Purpose |
|---|---:|---|
| `/` | 400 | Main menu — navigation cards, engine version, resume banner |
| `/setup/` | 768 | Seat kinds, draft mode, loadout selection, per-AI-seat evaluator picker (`list_evaluators`: builtins + raters). Writes `match` store, hands off to `/draft/` |
| `/draft/` | 1,442 | 12-ply alternating skill draft with drag-and-drop. MP coordination, AI drafting |
| `/match/` | 3,404 | Live game. **Fat controller** — see §12.7 |
| `/multiplayer/` | 784 | Lobby — host/join, recent sessions, rejoin flow |
| `/replay/` | 897 | MatchLog playback with scrubber and step controls |
| `/inspector/` | 1,014 | Branching position explorer with AI search |
| `/library/` | 589 | Match history (IndexedDB), filter, bulk export |
| `/loadouts/` | 842 | Custom loadout CRUD, share codes, import |
| `/training/` | 584 | Training Observatory — start/stop NN training, live status panels |
| `/position-builder/` | 524 | **NEW.** FEN-based position editor: place pieces/skills onto a board and boot an engine from a hand-built FEN. Backed by the pure `state/position-fen.ts` |

`+layout.svelte` wraps all routes with the `MpErrorBanner`, help/settings buttons, the shared `BackButton` (driven by the `back-nav` store), and the SFX-unlock handler. `+layout.ts` disables SSR globally.

### 12.2 Engine Boundary (`src/lib/engine/`)

All communication with the Rust backend goes through Tauri IPC `invoke()` calls. The engine is a singleton: `getEngine()` in `engine/index.ts` lazily constructs one `TauriClient` and caches it for the session lifetime (`resetEngine()` disposes it). There is no WASM path — it was fully removed; the app ships as Tauri-only (WASM cannot host the CUDA training backend), and only vestigial no-op comments mention it.

- **`types.ts`** — the `EngineClient` interface, now a broad surface. It covers draft (`createEngineWithDraft`, `createEngineWithLoadouts`, `draftState`), live play (`tryApply(action, turnStartedMs?, backgroundEval?)`, `stepAi`), inspector/forced AI (`requestAiMoveForced`, `requestAiMoveAtDepth`), reads (`positionView`, `legalActions`, `positionFen`), the stateless contract surface (`actionToNotation`, `skillMetadata`, `gameConstants`), eval overlays (`evalReport(perPiece)` — one dynamic `EvalReport` drives both the eval bar/term panel and the per-square hover card), persistence/resume (`snapshotJson`, `restoreFromSnapshot`, `matchLogJson`, `latestPlyJson`, `finaliseLog`), rater/eval control (`setAiEvaluator`, `evaluateDrawOffer`), the background-eval and AIvAI-producer event hooks (`onBackgroundEvalReady`, `startAivaiProducer` / `aivaiProducerLog` / `stopAivaiProducer` / `onAivaiProgress`), and lifecycle (`createEngine`, `version`, `dispose`).
- **`tauri-client.ts`** (~330 lines) — the desktop IPC implementation. Routes every method to a Rust `invoke()` command. It holds a private `#handle`; every `createEngine*` / `restoreFromSnapshot` routes through `#replaceHandle`, which drops the prior registry handle via `drop_engine` so route re-entry does not leak Rust-side `Match` records. `#requireHandle` guards handle-dependent calls.
- **`action.ts`** — u32 codec (`decodeAction`, `actionKindName`, `ActionKind`, BodyguardChoice bit 31, DraftTurn bit 30). Shared by all routes that need to inspect a raw action integer without a round-trip to the engine.
- **`skills.ts`, `mailbox.ts`, `config.ts`** — Skill metadata (a synchronous `SKILLS` mirror), mailbox decoder (`formatSquare(sq) -> "a1"`), config JSON builder.
- **`ai-hooks.ts` / `ai-service.ts`** — the AI-access seam. `ai-hooks.ts` is a shared error/timeout shell (`runAiCall`) deduping try/catch shapes; `ai-service.ts` unifies one-shot best-move requests and background-AIvAI-producer control.
- **`ply-eval.ts`** — extracts a per-ply engine assessment (`ai` / `background_eval`) from a match-log `PlyRecord`.
- **`snapshot-validator.ts`** — a shared trust gate that validates external snapshot/MatchLog JSON before `restoreFromSnapshot`.

`action-label.ts` no longer exists. Action strings are produced by the engine via `actionToNotation(raw)` (a stateless Tauri command that calls `core_engine::action_to_notation`) and stored on `PlyRecord.action.notation` in match logs. Every place that displays an action label reads one of these two sources. The `engine/` directory also carries several contract/unit tests (`skills.contract.test.ts` asserts the `SKILLS` mirror against `skillMetadata()`/`gameConstants()`; plus `ai-hooks`, `ai-service`, `ply-eval`, `snapshot-validator`, and `tauri-client` tests).

The hot path is: `create_engine` → `legal_actions` / `position_view` → `try_apply` (human) or `step_ai` (AI). `step_ai` streams per-depth updates via a Tauri event `"ai-depth-update"` for the live depth display; `request_ai_move_at_depth` is the inspector's fixed-depth forced-search variant.

### 12.3 Board Rendering (`src/lib/board/`)

- **`Board.svelte`** — SVG grid renderer. Pure-ish: parent owns `position`, `pieceIds`, all interaction state. ~15-prop surface — all interactivity arrives as callbacks.
- **`Piece.svelte`, `SkillWheel.svelte`, `SkillInfoCard.svelte`, `DirectionPicker.svelte`, `SkillGlyphDefs.svelte`** — leaf renderers.
- **`EffectsLayer.svelte`** — Canvas overlay; drains `effectQueue` and dispatches per-skill choreography from `viz/skill-effects.ts`.
- **`LoadoutBoard.svelte`** — a mini 8x8 board showing one side's King + Champions on their Stack-M squares (Guards greyed); shared by the loadout editor and draft preview.
- **`SkillPicker.svelte`** — the shared skill-catalogue chip grid (click and drag interaction modes) consumed by `/draft/` and `/loadouts/`.
- **`bodyguard-cover.ts`** — pure geometry for the always-on Bodyguard-protection indicator, mirroring the engine's `bodyguard_guards_for` (unit-tested).

### 12.4 Visual Effects Pipeline (`src/lib/board/ply-renderer.svelte.ts` + EffectsLayer)

`Effect` is a discriminated union (`dust | impact | damageNumber | shake | heal | armor`) defined in `src/lib/viz/effects.ts`, which owns the lightweight FX descriptor queue (each entry ~700ms with an optional `ttl`). Per-skill animation choreography — the attack/hold/release segments each skill plays — lives in `src/lib/viz/skill-effects.ts` and is dispatched by `EffectsLayer`.

**`ply-renderer.svelte.ts`** is a stateful driver shared by `/match/`, `/replay/`, and `/inspector/`. Both routes create one via `createPlyRenderer(eng, opts)` and call `applyAndRender(raw, applyFn)` per action. It owns `pieceIds` (stable IDs for CSS transitions), `shakingSquares`, `effectQueue`, deferred-skill-refresh state, and the current rendered `position`. The `positionSink` callback keeps `match.position` as the source of truth.

`RELOC_DELAY_MS = 260ms` governs the deferred-state-flip for skill actions with relocations or deaths: the impact animation plays on the pre-state board, then `drainPendingSkillRefresh()` flips the position. If a new action commits before the timer fires, the pending flip is cancelled.

`fastForwardTo(baseSnap, actions, target)` seeds a checkpoint every 32 plies so scrubs near a previously-visited position pay at most 31 `tryApply` round-trips instead of N.

**SFX policy:** `sfxEnabled` opts gates `sfx.play` calls. `/match/` passes `true`; `/replay/` and `/inspector/` pass `false`.

### 12.5 Audio (`src/lib/audio/sfx.ts`)

WebAudio synthesis only (no asset files). `sfx.play(event, opts?)`. Called from `ply-renderer.svelte.ts` for all match-mode sounds. No audio-engine abstraction — every caller imports `sfx` directly.

### 12.6 State Stores (`src/lib/state/`)

| Store | File | Description |
|---|---|---|
| `match` | `match-store.svelte.ts` | Mode, seats, position, legal actions, selection, telemetry IDs, pending snapshots, sandbox state |
| `mpState` | `multiplayer.svelte.ts` | WS status, code, role, pong timing, redial state, session epoch |
| `settings` | `settings.svelte.ts` | All UI preferences, persisted to localStorage — including the per-AI-seat evaluator choices (`p1Evaluator`/`p2Evaluator`) and the single global `uiEvaluator` that drives the eval bar / breakdown panel / replay / inspector |
| `inspector` | `inspector-store.svelte.ts` | Branch tree, current node, legal actions, last AI hint |
| `aiSearch` | `ai-search.svelte.ts` | Per-side think state, current `EvalReport` (+ prior-round report), background eval |

`inspector-store.svelte.ts` defines `InspectorNode`, which carries `edgeNotation: string` — the canonical notation string for the action that produced this node. It is populated at `addChild()` time (resolved via `eng.actionToNotation()` or read from `ply.action.notation` when loading from a log) so `MoveListItem` can render it without any async work.

Supporting stores: `move-targets.ts`, `skill-targets.ts` (derived legality from `PositionView` + `legalActions`), `draft.ts` (pre-made loadouts, draft geometry), `geometry.ts`, `i18n.ts` + `i18n-locale.svelte.ts` (the reactive locale holder split out because runes only work in `.svelte.ts`), `telemetry-session.ts` (incremental per-ply persistence via `latestPlyJson`), `back-nav.svelte.ts` (global back-navigation state read by the layout back button, set per-route via `setBackNav`/`clearBackNav`), and `position-fen.ts` (pure FEN-board manipulation backing the Position Builder, testable without the Tauri engine). Several stores now carry co-located tests (`match-store.svelte.test.ts`, `settings.svelte.test.ts`, `position-fen.test.ts`).

### 12.7 The `/match/` Fat Controller

~3,400 lines. It is still the app's fat controller, but a large share of its former presentational surface has been extracted into child components — `PlayerPanel` / `ProgressionPanel` / `ActionLogPanel` (`$lib/match/`), `EvalBreakdownPanel` / `SquareEvalCard` (`$lib/eval/`), and `ConnectivityPill` / `GraceBanner` / `MultiplayerStatusStrip` (`$lib/multiplayer/`). What remains is match *orchestration*. By concern:

| Lines | Concern |
|---|---|
| 1–101 | `<script>` open + imports (now including the extracted `$lib/match/`, `$lib/eval/`, `$lib/multiplayer/` components and `createPlyRenderer`) |
| ~101–410 | Reactive state declarations (mode, busy/ready flags, AIvAI producer/view-split state, preview/time-travel state, drag/cursor, skill-arming with focus/charge/retarget prefs, wheel derivations) |
| ~410–1070 | Derived selectors (move targets, wheel legality/splits, ally-pick candidates) and drag handlers |
| ~1073–1120 | Host-side peer-drop detection + resume handshake |
| ~1125–1630 | Core apply/refresh loop (`refresh`, `applyRaw`, `afterApplied`, `runAiStep`, `advanceView`) + click/drop handlers + `endPhase` |
| ~1630–1684 | Resign / draw (`confirmResign`, `offerDraw`, `respondToDrawOffer`) |
| ~1684–1844 | Wheel-slice click/hover, direction picker, keyboard handler |
| ~1844–2410 | Export / sandbox: FEN & log copy/download, sandbox enter/undo/redo/exit, preview time-travel (`selectPreviewPly`, `confirmPlayMyMoves`, `drainPlayMyMoves`, `restoreTrueLineFromSandbox`, `ensureLiveEngineOnTrueLine`), `beforeUnloadGuard` |
| 2412–~2864 | Markup |
| ~2865+ | Styles |

Concerns still juggled in one file: engine lifecycle, action-application orchestration, MP wire coordination, telemetry, AI scheduling (including the AIvAI producer/view split), drag UI, skill-arming state machine, modal choosers, sandbox/preview time-travel isolation, and export. Visual-effects rendering and SFX live in `PlyRenderer`; per-player HUD, progression, action log, and eval panels now live in their extracted components.

### 12.8 Multiplayer (`src/lib/multiplayer*` + `src/lib/multiplayer/`)

Multiplayer code lives in two places: the older top-level `multiplayer-*.ts` modules (protocol, engine, resume, handoff) and a newer `multiplayer/` subdirectory holding extracted UI components and pure helpers.

**Top-level modules:**

**`multiplayer.svelte.ts`** — reactive `$state mpState`. Subscribers attach `rawDataHandlers` for V2 game messages; a per-kind raw inbox buffers messages that arrive during route transitions.

**`multiplayer-protocol-v2.ts`** — types and codec for the V2 wire protocol. Messages: `session-hello`, `intent`, `committed` (with `seq` and `postZobrist`), `intent-rejected`, `phase-change`, `snapshot`, `request-snapshot`, `cheat-detected`, `handoff-announce`, `paused/resumed`, `game-config`, `error`.

**`multiplayer-protocol.ts`** — heartbeat (`ping`/`pong`), broker `error` frame, `generateCode` / `isValidCode` / `GRACE_MS`. Still imported by `multiplayer.svelte.ts`, the lobby route, and `GraceBanner.svelte`. Not a delete candidate.

**`multiplayer-engine.ts`** — `createMpEngine()` is the single funnel for all apply traffic. Solo: submits directly. Host: applies locally then broadcasts `committed`. Joiner: sends `intent`, awaits matching `committed`, mirrors the apply, audits Zobrist. A mismatch triggers `request-snapshot`.

**`multiplayer-resume.ts`** — `snapshotJsonFromMatchLog` rebuilds a Snapshot from a persisted log for the host-rejoin handshake. `logIsMidDraftCheap` routes to `/draft/` vs `/match/` without booting the engine. Zobrists in the log are not touched here; the V2 protocol uses the live `PositionView.zobrist` instead.

**`multiplayer-handoff.ts`** — cross-route handoff state for host handover.

**`multiplayer/` subdirectory:**

- **`websocket-transport.ts`** — the WebSocket relay transport implementing the transport interface: session routing plus the joiner-side auto-redial backoff (400ms → 1.5s → 3s → 6s → 12s → 30s).
- **`transport-config.ts`** — relay WS/HTTP URL constants resolved from env.
- **`heartbeat.ts`** — the heartbeat + now-tick timer pair (plain TS, testable), extracted from the wrapper.
- **`pill-state.ts`** — pure derivation of the connectivity-pill signal + `disconnectedSince` anchor (removes a hidden side-effect from a getter).
- **`route-lifecycle.ts`** — uniform MP teardown on route unmount (`tearDownMultiplayerOnLeave`).
- **`ConnectivityPill.svelte`, `GraceBanner.svelte`, `MpErrorBanner.svelte`, `MultiplayerStatusStrip.svelte`** — MP status UI components extracted from `/match/` and the layout.

The `match.localSeat` (game-identity, permanent) / `mpState.role` (network-identity, changes on host handoff) invariant is critical: the UI always derives "am I P1?" from `localSeat`, never from `role`.

### 12.9 Storage / Telemetry (`src/lib/storage/`)

- **`idb-backend.ts`** — IndexedDB backend. `MatchMeta` rows + per-ply `PlyEntry` records. `startMatch`, `recordPly` (incremental, not full re-serialise), `finaliseMatch`, `listMatches`, `bundleMatches`.
- **`tauri-backend.ts`** — Desktop backend (file-based). Same interface as the IDB backend.
- **`index.ts`** — Runtime backend detect.
- **`types.ts`** — Shared types crossing the backend boundary.
- **`library-handoff.ts`** — One-shot sessionStorage cell to pass a MatchLog from `/library/` → `/replay/` or `/inspector/`.
- **`position-handoff.ts`** — In-memory FEN handoff from `/replay/` or `/inspector/` → `/position-builder/`.
- **`loadout-codec.ts`** — share-code + JSON codec for custom loadouts (the `L1:<base64url>` share form).
- **`loadout-dedupe.ts`** — dedupe helpers keyed on the 12-skill tuple (names are treated as labels, not identity).

### 12.10 Extracted Component Directories

Presentational surface pulled out of the routes lives in dedicated component dirs, each a thin view over a store or an engine read:

- **`src/lib/match/`** — `PlayerPanel.svelte` (per-player HUD reading `PositionView` + settings + `ai-search`), `ProgressionPanel.svelte` (round/progression lookahead table), `ActionLogPanel.svelte` (ply-by-ply action log). All extracted from the old `/match/` monolith.
- **`src/lib/eval/`** — `EvalBreakdownPanel.svelte` (renders the `EvalReport`'s aggregate + side terms dynamically, with a round-over-round delta column, from the `ai-search` store) and `SquareEvalCard.svelte` (per-square hover card iterating `pieces[sq].terms`). Both extracted from `/match/`; both are fully term-driven (add an eval term in Rust and it appears with no frontend change).
- **`src/lib/inspector/`** — `AiHintBanner.svelte` (AI hint + notation), `MoveListItem.svelte` (one branch-tree row), and `PoiLabelDialog.svelte` (a `<dialog>`-based POI label editor replacing `window.prompt`, built on the shared `Modal`).
- **`src/lib/training/`** — the Training Observatory panels: `LiveMatchView.svelte` (with `EvalBar.svelte`), `TournamentStandings.svelte`, `LineageTree.svelte`, `NetworkInspector.svelte`, `GauntletMatrix.svelte`. `polling.ts` is a fixed-cadence Tauri-command polling store shared by all panels, and `types.ts` mirrors the trainer's serde wire shapes.
- **`src/lib/ui/`** — shared primitives: `BackButton.svelte` (boxed back button with click SFX, used by every top-level route via the `back-nav` store) and `Modal.svelte` (the unified modal all dialogs build on).
- **`src/lib/`** (top level) — `HelpModal.svelte` and `SettingsModal.svelte` (a content test, `HelpModal.content.test.ts`, guards the help copy). The settings drawer also hosts the evaluator dropdowns: a per-AI-seat picker beside each seat's think-time/depth (writes `settings.p{1,2}Evaluator`, applied on the next engine boot) plus the single global UI-eval pick (`settings.uiEvaluator`); both are populated from `list_evaluators`, the same source as the `/setup/` picker.

### 12.11 Replay / Inspector Data Flow

```
TelemetryStore (IDB or Tauri FS)
    │
    ▼
library/+page.svelte ──setPendingMatchLog──▶ sessionStorage
                                                  │
                       ┌──────────────────────────┤
                       ▼                          ▼
            replay/+page.svelte         inspector/+page.svelte
                  │                              │
                  └─snapshotJsonFromMatchLog()───┘
                                  │
                                  ▼
                  eng.restoreFromSnapshot() + tryApply loop
```

Both routes read `ply.action.notation` from the log JSON so action labels are resolved from the already-stored engine string, not re-derived in TypeScript. Where notation is absent (old logs or live inspector nodes not yet in a log), the routes fall back to `eng.actionToNotation(raw)`.

Replay drives the apply loop through `PlyRenderer` so skill effects and slide animations work the same way as in `/match/`. Inspector also drives through `PlyRenderer` (Session 33): node selection uses `renderer.fastForwardTo(baseSnap, node.actions, n)` so piece identity is preserved across sibling navigation and effects animate on the landing ply.

### 12.12 Key Seams

1. **`applyRaw` → `renderer.applyAndRender`** — caller passes an apply closure; the renderer snapshots pre-state, runs the closure (which moves the engine), renders effects/SFX, then flips position (sometimes deferred via `RELOC_DELAY_MS`).
2. **`mpEngine` wrapper** — sits between input handlers and `eng.tryApply`. Host applies directly; joiner sends intent and re-applies on committed echo.
3. **Telemetry lifecycle** — `startTelemetrySession` → `recordPly` per apply (uses `latestPlyJson`) → `finalize` / `networkLost` / `claimWinByOpponentForfeit` on terminal events.
4. **AI scheduler** — `$effect` watches `currentSeatIsAi + aiAutoPlay`, queues `runAiStep` with `aivaiStepDelayMs` delay.
5. **Sandbox** — saves `snapshotJson` on entry, restores on exit via `ensureLiveEngineOnTrueLine()`, all moves discarded.
6. **Action notation** — engine owns the string; frontend never re-implements action formatting. `PlyRecord.action.notation` carries the string in logs; `actionToNotation(raw)` via Tauri IPC covers live raw integers.

### 12.13 Testing

28 `*.test.ts` files (~343 `it`/`test` cases) under Vitest, spanning the engine boundary, stores, multiplayer, storage, and pure helpers:

| Test file(s) | Covers |
|---|---|
| `multiplayer-engine.test.ts` | Role-aware wrapper (host/joiner/solo paths, intent/committed handshake, zobrist audit) |
| `multiplayer-protocol-v2.test.ts` | Wire-format encode/decode + validation |
| `multiplayer-protocol.test.ts` | Legacy V1 heartbeat + utility helpers |
| `multiplayer-resume.test.ts` | `snapshotJsonFromMatchLog`, `logIsMidDraftCheap` |
| `multiplayer-handoff.test.ts` | Cross-route handoff state |
| `multiplayer.svelte.test.ts` | Transport wrapper + `mpState` |
| `multiplayer-sandbox.integration.test.ts` | Sandbox ↔ MP-wrapper contract (ns-37) against a route-shaped fake engine |
| `multiplayer/pill-state.test.ts`, `multiplayer/route-lifecycle.test.ts` | Pure connectivity-pill derivation + MP route teardown |
| `engine/skills.contract.test.ts` | `SKILLS` mirror vs engine `skillMetadata()` / `gameConstants()` |
| `engine/ai-hooks.test.ts`, `engine/ai-service.test.ts`, `engine/ply-eval.test.ts`, `engine/snapshot-validator.test.ts`, `engine/tauri-client.test.ts` | AI plumbing, per-ply eval extraction, snapshot trust gate, IPC client |
| `board/bodyguard-cover.test.ts`, `board/ply-renderer.test.ts` | Bodyguard geometry, ply renderer |
| `viz/skill-effects.test.ts` | Per-skill choreography |
| `state/*.test.ts` | `match-store`, `settings`, `position-fen` |
| `storage/*.test.ts` | `idb-backend`, `library-handoff`, `loadout-codec`, `loadout-dedupe` |
| `telemetry-session.test.ts` | Lifecycle (start/record/finalise/networkLost/abandon) |
| `training/polling.test.ts` | Fixed-cadence polling store |
| `HelpModal.content.test.ts` | Help-copy guard |

Coverage gaps: routes (no end-to-end); Board/EffectsLayer (visual). The route layer is verified by manual smoke + `svelte-check` + production build. No engine boundary contract test enforces cross-client parity — but with WASM removed, `tauri-client.ts` is now the only `EngineClient` implementation, so the concern is largely moot.

### 12.14 Observed Extraction Opportunities

1. **Skill targeting service.** Arm/disarm + modifier state lives in `/match/`, but legal targets are in `skill-targets.ts`. Could own the full lifecycle.
2. **Drag service.** ~30 lines of drag state (`dragSrc`, `dragTrail`, `dragHover`, `cursorXY`, `pendingApproach`) is a reusable shape.
3. **Telemetry finalizer.** Three terminal paths (finalize/networkLost/forfeit) share enough structure to collapse into one resolver.
4. **Board props grouping.** Could group ~15 props into `movePhase`, `skillPhase`, `chooser` sub-objects.

Since the last revision, several previously-listed opportunities have been realised: the MP status UI, connectivity-pill derivation, and route teardown are now extracted into `$lib/multiplayer/`; the per-player HUD, progression, action log, and eval panels are extracted into `$lib/match/` and `$lib/eval/`; and the AI-access plumbing is unified behind `$lib/engine/ai-service.ts` + `ai-hooks.ts`.

What's missing / would need a new layer: no AI player abstraction beyond `currentSeatIsAi`; no game-phase FSM (phase/turn logic is imperative `if` chains); no effect/SFX abstraction beyond the queue; no engine `undo_ply` / `seek_to_ply` (replay scrubbing is O(N) round-trips per jump because the session layer does not expose its internal undo stack).

---

## 13. relay

### 13.1 `server.ts`

A pure WebSocket relay written in Bun. It pairs two browser peers in a named session and forwards all game messages verbatim between them. No game logic runs here — the relay never inspects message content beyond the `type` field used for session-management control frames.

Each session holds host and joiner sockets, a creation timestamp, and a `bothAbsentSince` timestamp for TTL. A 10-second cleanup loop deletes sessions where both peers have been absent for more than 60 seconds.

### 13.2 Protocol Reference

**Relay control frames** (consumed by the relay, keyed by `type`):

| Sender → Relay | Message | Relay → Sender |
|---|---|---|
| Host | `{"type":"create"}` or `{"type":"create","preferCode":"XXXXXX"}` | `{"type":"created","code":"XXXXXX"}` |
| Joiner | `{"type":"join","code":"XXXXXX"}` | `{"type":"joined"}` (normal) or `{"type":"created","code":"..."}` (promoted to host) |
| — | — | `{"type":"peer-connected"}` when the second peer joins |
| — | — | `{"type":"peer-disconnected"}` on socket close |
| — | — | `{"type":"error","reason":"..."}` on invalid-code / session-full / session-gone |

Game messages (V2 protocol, keyed by `kind`) are forwarded byte-for-byte without inspection.

**HTTP endpoint:** `GET /probe/:code` → `{"live":bool,"paired":bool}`. Used by the lobby UI to show session liveness dots before a WebSocket connection is established.

### 13.3 Deployment

Deployed on Fly.io as `boardgame-relay.fly.dev` (Frankfurt region, 256 MB / 1 vCPU, `auto_stop_machines = 'stop'`). The frontend resolves the URL from `VITE_RELAY_URL` / `VITE_RELAY_HTTP_URL`; development falls back to `localhost:3001`. The Dockerfile copies only `server.ts` and `package.json` into a `oven/bun:1` base image.

---

## 14. bench

### 14.1 `run_sweep.sh`

Builds `search_bench` in release and runs five budget configurations in sequence: fixed depth 6 (median of 5 runs) and time-budgeted at 100ms / 500ms / 1000ms / 3000ms. Output goes to `bench/<prefix>-<budget>.json`. The `baseline` prefix writes to `bench/` directly; all other prefixes write to the gitignored `bench/results/`. A determinism smoke check at the end exits with code 1 on failure.

### 14.2 `compare.py`

Diffs a candidate sweep in `results/` against the committed `baseline-*.json`. Reports geometric-mean NPS delta, total wall-clock delta, and per time-budget mean depth counts. The acceptance criterion is: over-1-second positions must not increase AND total wall-clock must decrease. Prints "NET WIN" or "NET LOSS" as the final line.

### 14.3 `corpus/corpus.txt`

30 hand-curated FEN positions (corpus v2, Session 41, 2026-07-05) sampled from search-driven self-play at depths 2/3/4. Categories: `opening-with-skills`, `midgame-move`, `skill-phase-full`, `combo-loaded`, `endgame-with-skills`, `king-in-danger`. Each line contains ID, category, expected depth-N best move, expected score range, and the FEN string. The committed `baseline-*.json` files contain the search results these positions should reproduce.

---

## 15. tools

### 15.1 `tools/analyze_playtest.py`

Extracts design-relevant metrics from a `boardgame-bundle-v1` telemetry export (the "send to designer" JSON blob). Never prints raw game state; only reports derived signal.

Per-game report sections: game length, material arc (piece counts from start to end), capture timeline (round / ply / by whom, reconstructed from FEN census diffs), action balance (Move-Attack count vs skill activations with ratio), skill usage (by name, first/last round used, "used 3+ times" flag, drafted-but-never-used), draft loadouts, per-player think time and legal-action branching factor.

The `--combo-trace` flag audits combo-bonus application: for each Skill ply it reads the target's combo counter from the pre-action FEN and compares it to the actual HP/armor delta to verify the engine applied the correct bonus damage.

CLI usage:
```
analyze_playtest.py <bundle.json>
analyze_playtest.py <bundle.json> --game 2        # single game only
analyze_playtest.py <bundle.json> --combo-trace   # combo-bonus audit
analyze_playtest.py <bundle.json> --json          # machine-readable output
```

The `SKILL_NAME` / `SKILL_MONEY` dictionaries in the script are manually kept in sync with `core_engine/src/game_logic/skills.rs`.

---

## 16. Cross-Cutting Data Flows

### 16.1 Human Move: Frontend → Engine → Frontend

```
User drags piece
  └─► frontend encodes Action (raw u32) from src/target/approach_sq/skill_id
        └─► invoke("try_apply", { handle, raw_action })
              └─► tauri_wrapper: registry.with(handle, |entry| wrapper_api::try_apply(...))
                    └─► session::Match::try_apply_timed(action, thought_ms, now_unix_ms, None)
                          ├─► action_to_notation(action, pending_bodyguard) [BEFORE make()]
                          ├─► telemetry::snapshot_pre(pos)
                          ├─► generator::generate(pos)  [legality check]
                          ├─► make_unmake::make(pos, action) → Undo
                          ├─► telemetry::snapshot_post(pos)
                          └─► MatchLog::record(PlyRecord { notation, … })
              └─► StepResultDto { applied_action, score=0, depth=0, nodes=0 }
  └─► frontend calls invoke("position_view") + invoke("legal_actions")
        └─► UI re-renders board state
```

### 16.2 AI Move: Request → Search → Apply → Frontend

```
invoke("step_ai", { handle })
  └─► tauri_wrapper: block_in_place {
        wrapper_api::step_ai_with_cb(m, now_unix_ms, Some(&on_depth_cb))
          └─► session::Match::request_ai_move_with_cb(on_depth)
                └─► find_best_with_evaluator(pos, tt, time_ms, max_depth, evaluator, on_depth)
                      └─► search::alpha_beta::search(...)  [iterative deepening]
                            ├─► make_unmake::make / unmake  [tree traversal]
                            ├─► evaluator::evaluate(pos)    [at leaves]
                            │     └─► NnueEvaluator: Accumulator::apply + QuantizedNet::forward_int
                            │                    OR  HeuristicEvaluator: evaluate_scalar
                            └─► on_depth(depth, score) → Tauri event "ai-depth-update" → frontend
              └─► Match::try_apply_timed(best_action, thought_ms, now_unix_ms, Some(SearchMeta))
      }
  └─► StepResultDto { best_action, score, depth, nodes, thought_ms }
  └─► frontend re-renders
```

### 16.3 Multiplayer: Host Action → Relay → Joiner

```
Host applies action locally:
  MpEngineHandle.submitAction(action)
    └─► tryApply(action)  [host engine]
    └─► send V2 "committed" { seq, action, postZobrist } via WebSocket → relay
          └─► relay.forward(ws, session, raw)  [no inspection]
                └─► joiner receives "committed"
                      └─► MpEngineHandle.onCommitted(msg)
                            ├─► tryApply(action)  [joiner mirrors]
                            ├─► audit: pos.zobrist == postZobrist
                            │     mismatch? → send "request-snapshot" → host sends full "snapshot"
                            └─► onApplied(action)  [joiner UI re-renders]
```

### 16.4 Training: Bootstrap → Mutation Self-Play → Rater → In-Game AI

```
run_training(config, run_dir, stop_flag, Cpu)
  │
  ├─ Phase 0 (first run only)
  │    corpus_gen::generate_training_corpus
  │      └─► self-play games at depth 2/3/4 → deduped Vec<Position>
  │    bootstrap::label_corpus  →  Vec<ScalarLabelled>  (each = pos + evaluate() centipawns)
  │    bootstrap::train_scalar  →  Mlp<InferenceBackend>
  │    QuantizedNet::from_mlp   →  v0001.{mpk,json}  →  RaterIndex  (champion = v0001)
  │
  └─ Phase 1 (each iteration)
       perturb_model(champion, std_dev, seed)  →  Mlp<TrainingBackend>
       into_inference + evaluator_from_inference_model  →  NnueEvaluator
       accept_vs_live (mirrored BO3):
         play_match_with_callback:
           find_best_with_evaluator [NnueEvaluator, 100ms]
             └─► Accumulator::apply → QuantizedNet::forward_int  (per node)
           per ply: write_if_subscribed → live.json  (if Training Observatory open)
       accepted?
         yes → save_rater → vNNNN.{mpk,json} → RaterIndex::append → new champion
         no  → discard candidate

In-game use:
  invoke("list_evaluators")  →  builtin evaluators + trained raters (one unified list)
  invoke("set_ai_evaluator", { handle, source, id })
    └─► build_evaluator(source, id):
          "builtin"/"heuristic" → core_engine builtin::make(id)
          "run"/"blessed"       → load_metadata(stem) → dispatch on input_dim:
                                    3352 → NnueEvaluator::load_from_stem  (NNUE path)
                                    2825 → NnEvaluator::load_from_stem    (dense path)
    └─► Match::set_evaluator(Box<dyn Evaluator>)
  invoke("step_ai")  →  find_best_with_evaluator uses the installed evaluator
```
