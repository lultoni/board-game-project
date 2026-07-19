# Architecture Cleanup Plan

> **Working style:** Do the work fully. No half-measures, no "future work" placeholders
> that defer real decisions. Every change here has an exact file, exact line, and exact
> outcome. When implementing, write the complete solution — not the cheap approximation.
> The codebase is small enough that every one of these changes can be done correctly on
> the first pass.

---

## Context & Goals

A full architecture scan was run across the entire codebase (2026-07-19). The findings
fall into four clean categories. This plan addresses all of them in dependency order.

**Root problems identified:**

1. The frontend contains game-semantic knowledge that belongs exclusively in the engine —
   magic skill IDs hardcoded in TypeScript, skill metadata duplicated from Rust, phase
   constants copy-pasted, self-cast rules re-implemented.

2. The frontend has structural duplication in its own modules — `skill-targets.ts` has
   eight independent scan loops with an identical 3-line preamble each; `moveTargetsFor`
   is called a third time inline inside an event handler when the `$derived` already has
   it; `rawForSelfCast` is a one-off inline scan in the route.

3. The AI loop is driven by the frontend via a blocking `invoke("step_ai")` — the engine
   cannot run ahead of the frontend, the frontend must wait for each search to complete
   before rendering, and there is no decoupling between engine speed and display rate.

4. Telemetry is incomplete: `thought_ms = 0` for every human ply (no start-of-turn
   timestamp), and `PlyRecord.ai` is `None` for human plies (no engine assessment of
   positions the human created).

---

## Change 1 — Engine exposes static skill + game-constants metadata API

### Problem
`frontend/src/lib/engine/skills.ts` contains a full duplicate of `core_engine/src/game_logic/skills.rs`:
- All 15 skill records with cost, defaultRange, category, targetOwner (lines 16–32)
- Phase constants PHASE_MOVE=0, PHASE_SKILL=1 (lines 46–47)
- Modifier bit constants MODIFIER_FOCUS=0x01, MODIFIER_CHARGE=0x02 (lines 42–43)
- Player constants PLAYER_P1=0, PLAYER_P2=1 (lines 50–51)
- GameResult constants (lines 54–56)

These can silently drift from the engine. There is no contract test. Magic IDs 10 (Blast)
and 11 (Shove) are hardcoded in `skill-targets.ts:91` for Focus-mode rules, and skill ID
11 is hardcoded in `match/+page.svelte:1124` for the DirectionPicker gate. `isSelfCast()`
at `skills.ts:77` re-implements `TargetOwner::SelfOnly` from the metadata table rather
than from the action set.

### Solution

**In `core_engine/src/game_logic/skills.rs`:**

Add a `SkillMetadata` struct (Serialize/Deserialize) that mirrors the TypeScript `SkillInfo`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub id:            u8,
    pub key:           &'static str,   // e.g. "lance"
    pub category:      &'static str,   // "strike" | "shield" | "move" | "mystic"
    pub cost:          u8,
    pub default_range: u8,
    pub target_owner:  &'static str,   // "enemy" | "ally" | "either" | "empty" | "self"
    /// True iff this skill has distinct focus_mode=0 and focus_mode=1 variants
    /// under Focus. Currently Blast (10) and Shove (11). Exposing this flag
    /// eliminates the hardcoded ID check in the frontend.
    pub has_focus_mode_choice: bool,
    /// True iff this skill opens the direction picker (choice_idx encodes push
    /// direction). Currently only Shove (11).
    pub needs_direction_pick: bool,
}

pub fn all_skill_metadata() -> &'static [SkillMetadata; 15] { ... }
```

Add a `GameConstants` struct (Serialize/Deserialize):

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConstants {
    pub phase_move:       u8,   // 0
    pub phase_skill:      u8,   // 1
    pub phase_draft:      u8,   // 2
    pub modifier_focus:   u8,   // 0x01
    pub modifier_charge:  u8,   // 0x02
    pub modifier_move_attack_used: u8,  // 0x04
    pub player_p1:        u8,   // 0
    pub player_p2:        u8,   // 1
    pub game_ongoing:     u8,   // 0
    pub game_p1_wins:     u8,   // 1
    pub game_p2_wins:     u8,   // 2
    pub skill_count:      u8,   // 15
}

pub fn game_constants() -> GameConstants { ... }
```

Re-export both from `src/lib.rs`.

**In `crates/tauri_wrapper/src/lib.rs`:**

Add two new Tauri commands:

```rust
#[tauri::command]
fn skill_metadata() -> Vec<serde_json::Value> { ... }

#[tauri::command]
fn game_constants_cmd() -> serde_json::Value { ... }
```

Register both in `tauri::Builder`.

**In `frontend/src/lib/engine/tauri-client.ts`:**

Add `skillMetadata(): Promise<SkillInfo[]>` and `gameConstants(): Promise<GameConstants>`
to the `EngineClient` interface and implement them as `invoke()` calls.

**In `frontend/src/lib/engine/skills.ts`:**

- Replace the hardcoded `SKILLS` record with a module-level cache loaded once on startup
  via `getEngine().skillMetadata()`. The shape stays the same (`Record<number, SkillInfo>`)
  so all call sites work unchanged.
- Replace `PHASE_MOVE`, `PHASE_SKILL`, `MODIFIER_FOCUS`, etc. with values from
  `getEngine().gameConstants()` loaded once at startup and cached.
- `isSelfCast(id)` becomes `SKILLS[id]?.targetOwner === "self"` — same as now, but now
  the data comes from the engine.
- Add `SKILL_BLAST`, `SKILL_SHOVE` named constants derived from the loaded metadata
  (`metadata.find(s => s.key === "blast").id`, etc.) so no magic integers leak anywhere.

**In `frontend/src/lib/state/skill-targets.ts:91`:**

Replace `if (skillId !== 10 && skillId !== 11) return false;` with:
```typescript
import { skillById } from "$lib/engine/skills";
if (!skillById(skillId)?.hasFocusModeChoice) return false;
```

**In `frontend/src/routes/match/+page.svelte:1124`:**

Replace `armedSkill.skillId === 11` with:
```typescript
import { skillById } from "$lib/engine/skills";
skillById(armedSkill.skillId)?.needsDirectionPick === true
```

**Startup initialisation:**

In `frontend/src/lib/engine/index.ts` (or wherever the singleton engine is constructed),
call `skillMetadata()` and `gameConstants()` eagerly and cache results before any route
renders. Both are pure reads with no game state; one round-trip at startup is free.

### Files changed
- `game/crates/core_engine/src/game_logic/skills.rs` — add structs + functions
- `game/crates/core_engine/src/lib.rs` — re-export
- `game/crates/tauri_wrapper/src/lib.rs` — two new commands
- `game/frontend/src/lib/engine/tauri-client.ts` — two new methods
- `game/frontend/src/lib/engine/skills.ts` — replace static table with engine-loaded cache
- `game/frontend/src/lib/state/skill-targets.ts:91` — remove magic ID guard
- `game/frontend/src/routes/match/+page.svelte:1124` — remove magic ID 11

---

## Change 2 — `skill-targets.ts` internal deduplication

### Problem
Every exported function in `skill-targets.ts` (8 functions, lines 40–234) has the
identical loop preamble:

```typescript
for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    // ... then function-specific logic
}
```

This is copy-pasted 8 times. Adding any filtering logic (e.g. checking a new flag on
decoded actions) requires updating 8 places.

Also: `rawForSelfCast` at `match/+page.svelte:1210–1221` is a one-off inline scan of
`match.legal` to find a self-cast raw action, reimplementing logic that belongs in
`skill-targets.ts`. And `moveTargetsFor` at `match/+page.svelte:1179` is called inline
in `handlePieceDrop` when `$derived moveTargets` already holds the same value when
`src === match.selection`.

### Solution

**In `skill-targets.ts`:**

Add a single private primitive at the top of the file:

```typescript
/** Pre-filters `legal` to only Skill actions from `src` with `skillId`.
 *  All exported functions are thin wrappers over this. O(n) single pass. */
function filterSkillActions(
  legal: Uint32Array,
  src: number,
  skillId: number,
): SkillVariant[] {
  const out: SkillVariant[] = [];
  for (let i = 0; i < legal.length; i++) {
    const raw = legal[i];
    const a = decodeAction(raw);
    if (a.kind !== ActionKind.Skill) continue;
    if (a.src !== src) continue;
    if (a.skillId !== skillId) continue;
    out.push({ raw, target: a.target, choiceIdx: a.choiceIdx,
               focusMode: a.focusMode, auxSq: a.auxSq, hasAux: a.hasAux });
  }
  return out;
}
```

Rewrite all 8 exported functions as wrappers:

```typescript
export function skillTargetsFor(legal, src, skillId): SkillTargetSet {
  const variants = filterSkillActions(legal, src, skillId);
  const squares = new Set<number>();
  const byTarget = new Map<number, number[]>();
  const variantsByTarget = new Map<number, SkillVariant[]>();
  for (const v of variants) {
    squares.add(v.target);
    // ... same logic, now over `variants` instead of raw legal array
  }
  return { squares, byTarget, variantsByTarget };
}

export function skillIsCastable(legal, src, skillId): boolean {
  return filterSkillActions(legal, src, skillId).length > 0;
}

// ... etc. Each function is now 3–8 lines, zero boilerplate.
```

**Add a new export `rawForSelfCast`:**

```typescript
/** Find the raw u32 for a self-cast action (target === src, no aux).
 *  Returns null if none exists (skill not castable or no self variant). */
export function rawForSelfCast(
  legal: Uint32Array,
  src: number,
  skillId: number,
): number | null {
  const variants = filterSkillActions(legal, src, skillId);
  const v = variants.find(v => !v.hasAux || v.auxSq === src);
  return v?.raw ?? null;
}
```

**In `match/+page.svelte:1210–1221`:** Replace the inline scan with:
```typescript
import { rawForSelfCast } from "$lib/state/skill-targets";
const raw = rawForSelfCast(match.legal, sq, armedSkill.skillId);
```

**In `match/+page.svelte:1179` (`handlePieceDrop`):** Replace:
```typescript
const targets = moveTargetsFor(match.legal, src);
```
with:
```typescript
// moveTargets $derived already holds moveTargetsFor(match.legal, match.selection)
// and src was just set to match.selection above; use the cached value.
const targets = moveTargets;
```

### Files changed
- `game/frontend/src/lib/state/skill-targets.ts` — add `filterSkillActions`, rewrite 8 functions + add `rawForSelfCast`
- `game/frontend/src/routes/match/+page.svelte` — lines 1179, 1210–1221

---

## Change 3 — `ply-renderer` stops writing to match store

### Problem
`ply-renderer.svelte.ts` writes `match.position` and `match.legal` directly via the
`positionSink` interface (lines 562–575). A rendering module is the sole writer of
authoritative reactive match state. This is the data flow backwards: the visual layer
should be a consumer of state, not its owner.

Concretely: if you want to understand where `match.position` comes from, you must trace
through the renderer rather than looking at the store. This makes the store's state
ownership ambiguous and creates an invisible coupling between the renderer and the store.

### Solution

**In `ply-renderer.svelte.ts`:**

Remove the `positionSink` option entirely. Replace it with an `onStateUpdate` callback:

```typescript
export interface PlyRendererOpts {
  eng: EngineClient;
  onStateUpdate?: (position: PositionView, legal: Uint32Array) => void;
  // ... other opts unchanged
}
```

Inside `setPosition` and `setLegal`, call `opts.onStateUpdate(position, legal)` when set,
and keep the `localPosition` / `localLegal` fallback for when it is not set (replay,
inspector modes that own their own state).

**In `match/+page.svelte` (renderer construction, lines 575–582):**

Replace:
```typescript
renderer = createPlyRenderer(eng, { positionSink: match, ... });
```
with:
```typescript
renderer = createPlyRenderer(eng, {
  onStateUpdate: (pos, legal) => {
    match.position = pos;
    match.legal = legal;
  },
  ...
});
```

The match store now explicitly owns its own writes. The renderer calls a callback;
what happens to the data is the store's decision.

### Files changed
- `game/frontend/src/lib/board/ply-renderer.svelte.ts` — replace `positionSink` with `onStateUpdate` callback
- `game/frontend/src/routes/match/+page.svelte` — update renderer construction

---

## Change 4 — Human think-time recorded in telemetry

### Problem
`PlyRecord.thought_ms` is always 0 for human plies. `session.rs:try_apply_timed` takes
`thought_ms` as a caller-supplied `u32`, but the Tauri `try_apply` command
(`tauri_wrapper/src/lib.rs:357–368`) does not know when the player's turn started — it
only knows `now`. The frontend has no `turnStartedAt` timestamp anywhere.

The result: telemetry has zero decision-time data for human players. This is a gap for
any replay analysis of player behavior.

### Solution

**In `match-store.svelte.ts`:**

Add `turnStartedMs: number = 0` to the match state. Set it whenever the phase changes
to Move or Skill (i.e., when it becomes a new action opportunity for the current player).
This is driven from `afterApplied()` in `match/+page.svelte` which already detects phase
transitions — add `match.turnStartedMs = Date.now()` at that point.

Also set it on match start / engine boot.

**In `frontend/src/lib/engine/tauri-client.ts`:**

The `tryApply` method signature becomes:
```typescript
tryApply(raw: number, turnStartedMs: number): Promise<StepResult>
```

It passes `turn_started_ms` to the Tauri invoke:
```typescript
invoke("try_apply", { handle, raw_action: raw, turn_started_ms: turnStartedMs })
```

**In `crates/tauri_wrapper/src/lib.rs` (`try_apply` command):**

```rust
#[tauri::command]
fn try_apply(
    handle:          u64,
    raw_action:      u32,
    turn_started_ms: u64,   // new parameter; defaults to 0 for old callers via serde
    registry:        State<'_, EngineRegistry>,
) -> Result<StepResultDto, String> {
    let now = unix_ms_now();
    let thought_ms = now.saturating_sub(turn_started_ms).min(u32::MAX as u64) as u32;
    registry.with(handle, |e| {
        api::try_apply_with_thought(&mut e.m, raw_action, thought_ms, now)
            .map(StepResultDto::from)
            .map_err(|err| format!("{err:?}"))
    })?
}
```

**In `crates/core_engine/src/wrapper_api.rs`:**

Add `try_apply_with_thought(m, raw_action, thought_ms, now_unix_ms)` that calls
`m.try_apply_timed(action, thought_ms, now_unix_ms, None)`. The existing `try_apply`
wrapper calls it with `thought_ms = 0` for backwards compatibility.

**In all call sites for `tryApply` in the frontend:**

Pass `match.turnStartedMs` as the second argument. This covers:
- `match/+page.svelte` human action apply path
- `multiplayer-engine.ts` `tryApply` calls (human multiplayer moves)

### Files changed
- `game/frontend/src/lib/state/match-store.svelte.ts` — add `turnStartedMs`
- `game/frontend/src/routes/match/+page.svelte` — set `turnStartedMs` on phase transitions, pass it to `tryApply`
- `game/frontend/src/lib/multiplayer/multiplayer-engine.ts` — pass `turnStartedMs` to `tryApply`
- `game/frontend/src/lib/engine/tauri-client.ts` — add `turn_started_ms` parameter
- `game/crates/tauri_wrapper/src/lib.rs` — update `try_apply` command
- `game/crates/core_engine/src/wrapper_api.rs` — add `try_apply_with_thought`

---

## Change 5 — AI eval recorded on every ply (both AI moves and human moves)

### Problem

**Part A — AI eval on AI plies (what was missing):**
`PlyRecord.ai: Option<SearchMeta>` already exists and is populated for AI plies via
`step_ai`. What is missing is that `SearchMeta` only stores `depth`, `nodes`,
`raw_score`, `was_mate`, `mate_in`, `score_cp`. It does NOT store the full
`EvalBreakdown` for the position the AI chose — only the final alpha-beta score. When
replaying, you can see the AI's score but not the full breakdown of why it valued that
position (material, mobility, threat, etc.). The pre-action and post-action
`prev_breakdown` / `post_breakdown` fields in `PlyRecord` give the static heuristic
breakdown, but not the search's own assessment of the chosen move's terminal node.

Extend `SearchMeta` with the eval breakdown of the best move's resulting position:

```rust
pub struct SearchMeta {
    pub depth:      u8,
    pub nodes:      u64,
    pub raw_score:  i32,
    pub was_mate:   bool,
    pub mate_in:    Option<i32>,
    pub score_cp:   Option<i32>,
    // NEW: heuristic breakdown of the position after the best move was applied.
    // Gives full per-term visibility in replay analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_move_breakdown: Option<EvalBreakdown>,
}
```

In `session.rs::step_ai` (line 575–588), after the search completes and before
`try_apply_timed`:
1. Apply the best move temporarily to a cloned position.
2. Call `evaluate_breakdown(&clone)`.
3. Unmake (or just drop the clone).
4. Pass the breakdown into `SearchMeta`.

Alternatively (cheaper): since `post_breakdown` is already captured by
`try_apply_timed` → `snapshot_post`, extend `SearchMeta::from_search` to accept an
optional `EvalBreakdown` and thread it through the call chain. The cleanest path:
`step_ai` clones the position, applies the move, snapshots the breakdown, unclones,
then proceeds to `try_apply_timed` with the breakdown already computed.

**Part B — Background AI eval on human plies:**
After a human applies an action, fire a shallow background search (depth 4, no time
limit) against the resulting position to get the engine's assessment. Store the result
in a new field in `PlyRecord`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub background_eval: Option<SearchMeta>,
```

This is filled AFTER `try_apply_timed` returns (so it doesn't block the apply pipeline)
via a new `Match::annotate_last_ply_with_background_eval(depth: u8)` method that:
1. Runs a shallow search on the current position (post-human-move).
2. Takes only the score + breakdown, not the best move.
3. Patches `log.plies.last_mut()` with the result.

In the Tauri layer (`try_apply` command), after a successful human apply, spawn a
`tokio::task::spawn_blocking` to run `annotate_last_ply_with_background_eval(4)` and
emit a new Tauri event `"background-eval-ready"` when complete. The frontend can
optionally listen to this event to refresh the eval display mid-game without blocking
the action apply response.

This gives the replay inspector the engine's assessment of every human position with
zero impact on human turn latency (the apply responds immediately; eval arrives ~50ms
later via event).

### Files changed
- `game/crates/core_engine/src/telemetry.rs` — extend `SearchMeta` with `post_move_breakdown` and `background_eval` field on `PlyRecord`
- `game/crates/core_engine/src/session.rs` — `step_ai` computes post-move breakdown; add `annotate_last_ply_with_background_eval`
- `game/crates/tauri_wrapper/src/lib.rs` — `try_apply` command spawns background eval task + emits event
- `game/frontend/src/lib/engine/tauri-client.ts` — optional `"background-eval-ready"` listener
- `game/frontend/src/lib/state/ai-search.svelte.ts` — optional: update eval display on background-eval event

---

## Change 6 — Non-blocking AI background loop

### Problem
`step_ai` is a blocking `invoke` — the frontend sends the request, the Rust search runs,
and the frontend waits for the result before it can render anything or respond to any
input. The engine cannot run ahead of the frontend. For AI-vs-AI games this forces an
artificial delay (`aivai_step_delay`) to make the board visible between moves. There is
no way to let the engine run at full speed while the frontend renders at its own pace.

The training system already solves this correctly: the engine writes results to a file
(`live.json`) and the frontend polls. The same model should apply to in-game AI.

### Solution

**New Tauri state: `AiJobSlot`**

In `tauri_wrapper/src/lib.rs`, add alongside `EngineRegistry`:

```rust
struct AiJob {
    handle:     u64,
    result:     Option<StepResultDto>,  // None while running
    abort_flag: Arc<AtomicBool>,
    thread:     Option<JoinHandle<()>>,
}

struct AiJobSlot(Mutex<Option<AiJob>>);
```

**New commands:**

```rust
/// Start an AI search in the background. Returns immediately.
/// Emits "ai-step-ready" when the result is available.
/// Emits "ai-depth-update" per iterative-deepening depth (same as before).
#[tauri::command]
async fn start_ai_step(handle: u64, app: AppHandle, registry: State<EngineRegistry>, slot: State<AiJobSlot>)
```

```rust
/// Poll for a completed AI result. Returns None while still running,
/// Some(StepResultDto) when done. The result is consumed on first poll.
#[tauri::command]
fn poll_ai_result(handle: u64, slot: State<AiJobSlot>) -> Option<StepResultDto>
```

```rust
/// Cancel a running AI search (sets abort flag). Safe to call even when idle.
#[tauri::command]
fn cancel_ai_step(slot: State<AiJobSlot>)
```

`start_ai_step` spawns a `std::thread` (not Tokio task — the search is pure CPU with no
async) that:
1. Locks the registry, clones the `Position` (or takes the engine slot under a scoped
   lock for the duration of search — same as current `block_in_place` approach).
2. Runs `find_best_with_evaluator` with the abort flag wired to the deadline check.
3. When complete, writes `StepResultDto` into `AiJobSlot` and emits `"ai-step-ready"`.

**In the frontend (`match/+page.svelte` AI scheduler `$effect`):**

Replace:
```typescript
const result = await eng.stepAi(depthCallback);
// render result
```
with:
```typescript
eng.startAiStep(depthCallback);
// $effect re-fires when "ai-step-ready" event arrives
// (or poll on short interval as fallback)
```

Add a Tauri event listener for `"ai-step-ready"` that calls `eng.pollAiResult()` and
feeds the result into the existing `renderApplied` pipeline.

**AI-vs-AI at full speed:**

The `aivai_step_delay` hint in `Config` is now a frontend-only display rate setting.
The engine can run as fast as it wants; the frontend renders at whatever `stepDelayMs`
the user configured. The engine does not wait for the frontend to ack each move — it
just keeps the latest result in `AiJobSlot` and the frontend picks it up on its own
schedule. For display purposes at full speed (0ms delay), the frontend renders every
move as the events arrive.

**Thread safety note:**

The `EngineRegistry` `Mutex<HashMap<u64, EngineEntry>>` already serialises all access.
The background AI thread must hold the lock for the duration of the search (same as
`block_in_place` does now), so concurrent `try_apply` calls during an AI search are
queued by the mutex. This is identical to the current behaviour and correct — you cannot
apply a human move while the AI is thinking on the same handle anyway.

### Files changed
- `game/crates/tauri_wrapper/src/lib.rs` — add `AiJobSlot` state, `start_ai_step`, `poll_ai_result`, `cancel_ai_step` commands
- `game/frontend/src/lib/engine/tauri-client.ts` — add `startAiStep`, `pollAiResult`, `cancelAiStep`
- `game/frontend/src/routes/match/+page.svelte` — replace blocking `stepAi` call with event-driven flow
- `game/frontend/src/lib/state/ai-search.svelte.ts` — handle `"ai-step-ready"` event

---

## Change 7 — End-to-end multiplayer integration tests

### Problem
Multiplayer correctness is currently verified only by manual smoke testing. The
host/joiner Zobrist audit catches state desync in production, but there is no automated
test covering the full set of session lifecycle events:
- Normal play (both players make moves, game completes)
- Player leaves mid-game and rejoins (snapshot resync)
- Player leaves during sandbox mode
- Player leaves during the draft phase
- Host-side forfeit
- Joiner-side forfeit
- Both players leave simultaneously (session expiry)
- Reconnection during Bodyguard intercept (pending state mid-ply)

### Solution

**Test harness design:**

Create `game/frontend/src/lib/multiplayer/multiplayer.e2e.test.ts` (Vitest, no browser
needed — pure TypeScript logic test).

The harness spawns two in-process `MpEngineHandle` instances backed by a
`LocalRelay` — an in-process implementation of the relay's message-forwarding logic:

```typescript
class LocalRelay {
  private hostHandler: ((msg: unknown) => void) | null = null;
  private joinerHandler: ((msg: unknown) => void) | null = null;

  connectHost(handler: (msg: unknown) => void): LocalRelaySocket { ... }
  connectJoiner(handler: (msg: unknown) => void): LocalRelaySocket { ... }
  // Delivers messages synchronously or with configurable delay.
  // Can inject packet loss, reorder, or disconnect.
}
```

`LocalRelaySocket` implements the same interface as `WebSocket` (send/close/onmessage).
Both `MpEngineHandle` instances use `LocalRelay` sockets instead of real WebSockets.

Each test:
1. Creates a `LocalRelay`.
2. Boots two engine handles (host + joiner) with test configs.
3. Runs the scenario as a series of `applyAction(raw)` calls and relay events.
4. Asserts final state (both sides agree on Zobrist, `position.game_result`, etc.).

**Test scenarios (one `it()` block each):**

```typescript
describe("multiplayer lifecycle", () => {
  it("both players complete a full game", ...)
  it("joiner leaves and rejoins mid-game, state resyncs", ...)
  it("host leaves and rejoins mid-game, joiner becomes host, state resyncs", ...)
  it("joiner leaves during sandbox, sandbox state discarded on rejoin", ...)
  it("player leaves during draft phase, rejoins, draft resumes correctly", ...)
  it("host forfeits: joiner receives game-over", ...)
  it("joiner forfeits: host receives game-over", ...)
  it("both disconnect simultaneously, session expires after TTL", ...)
  it("zobrist mismatch triggers snapshot request and resync", ...)
  it("pending bodyguard state survives reconnect", ...)
});
```

Each scenario uses real engine state (via `wrapper_api` or `Match` directly depending
on what's accessible from TypeScript tests), so the assertions are against real game
positions, not mocks. This is a true integration test — not a unit test of message
codec.

**Connection state visual contract:**

The tests also assert that `mpState` (the reactive store) reflects the correct
`ConnectionStatus` at each point in the scenario. This ensures the frontend's visual
representation of connection state (banners, indicators) is driven by accurate state.
Each `it()` that involves a disconnect asserts:
- Before disconnect: `mpState.status === "connected"`
- During grace period: `mpState.status === "grace"`
- After TTL: `mpState.status === "disconnected"` (or session expired)
- After rejoin: `mpState.status === "connected"` and Zobrist matches

### Files changed
- `game/frontend/src/lib/multiplayer/local-relay.ts` — new: in-process relay stub
- `game/frontend/src/lib/multiplayer/multiplayer.e2e.test.ts` — new: all scenarios above

---

## Change 8 — Bitwise pass: replace loop-over-squares with `iter_bits`

### Problem
The scan identified three `for sq in 0..64` loops in `make_unmake.rs`. After checking
actual line numbers, all three are inside the test module (`#[cfg(test)]`). They are not
hot-path production code and are not correctness issues.

However, a targeted search of the **production** code (outside `#[cfg(test)]`) for
loop patterns that could be bitboard operations is still warranted. Known candidates
from the architecture scan and prior eval-perf passes:

**`generator.rs`:** The `reachable` function uses BFS and bitboard expansion — already
uses the correct `iter_bits` / `trailing_zeros` idiom. No change needed.

**`terms.rs`:** The two inner loops in `champion_threat_score` and `coverage_score`
already use `while hits != 0 { let s = hits.trailing_zeros(); hits &= hits - 1; }`.
This is the correct bitboard iteration pattern. No change needed.

**`magic.rs:movement_targets_speed2`:** The speed-2 flood-fill uses bitboard expansion
correctly. No change needed.

**`session.rs:aivai_draft_runs_to_completion_and_starts_move_phase` (test, line 925):**
Uses `for sq in 0..64u8` — test only, not production. Low priority to clean up but
is a good hygiene fix: replace with:
```rust
let mut bb = pos.kings.0 | pos.champions.0;
while bb != 0 {
    let sq = bb.trailing_zeros() as u8;
    bb &= bb - 1;
    // ... assert skill slots filled
}
```

**Systematic scan:**
Run `grep -n "for sq in 0..64" game/crates/core_engine/src/**/*.rs` excluding
`#[cfg(test)]` blocks to confirm no production loops were missed. If any surface, apply
the `iter_bits` pattern.

For the current known state of the codebase: no production `for sq in 0..64` loops exist
in core_engine. The bitwise foundation is already solid. This change is a verification
pass + cleanup of the test-module loops for hygiene.

### Files changed
- `game/crates/core_engine/src/session.rs` (test module) — minor: replace `for sq in 0..64` with `iter_bits`
- `game/crates/core_engine/src/game_logic/make_unmake.rs` (test module) — same

---

## Implementation Order

Dependencies run strictly top-down. Changes 2 and 3 are independent of Change 1 but
Change 1 makes Change 2 cleaner (no magic ID guards to replace). Change 4 and 5 are
independent. Change 6 builds on nothing but should come after 4+5 are in so telemetry
is correct before the background loop lands. Change 7 is independent of all others.
Change 8 is independent and can be done at any point.

```
Change 1  (engine metadata API)
    └── Change 2  (skill-targets dedup — drops magic ID guards)
    └── Change 3  (renderer state ownership — independent)

Change 4  (human think-time)
Change 5  (AI eval on all plies)
    └── Change 6  (non-blocking AI loop — land after telemetry is complete)

Change 7  (MP e2e tests — independent)
Change 8  (bitwise pass — independent)
```

Suggested batches:
- **Batch A:** Changes 1, 2, 3 — all frontend/boundary cleanup, no engine logic touched
- **Batch B:** Changes 4, 5 — telemetry completeness, purely additive schema changes
- **Batch C:** Change 6 — architectural change to AI loop
- **Batch D:** Changes 7, 8 — tests and cleanup, can run in parallel

---

## What is NOT changing

- The 5-layer `core_engine` stack — the layering discipline is correct and tight.
- `make_unmake.rs` skill dispatch — the 15 per-skill handlers are correctly separated.
- The `Evaluator` trait and `AccHandle` seam — already the right abstraction for
  swapping heuristic vs NNUE.
- The relay — pure forwarder, no game logic, correct as-is.
- `multiplayer-engine.ts` host/joiner/solo dispatch — clean, well-tested.
- The `Snapshot` / `from_snapshot` replay-validation mechanism — tamper detection works.
- The NNUE accumulator and quantised forward pass — already bitwise-optimal.
- The transposition table — correct design, warm across AI calls within a game.
- Any existing tests — no test is deleted. New tests are additive.

---

## Schema compatibility note

All new fields added to `PlyRecord` (`background_eval`, extended `SearchMeta`) use
`#[serde(default, skip_serializing_if = "Option::is_none")]`. All existing saved logs
continue to load without modification. The frontend uses `#[serde(default)]` compatible
JSON parsing throughout. No migration required.
