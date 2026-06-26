# Frontend Remediation Plan

Six commit-sized phases, sequenced so correctness/safety fixes land before any architectural moves, DI seams precede unit tests, and engine-boundary direction is fixed before new abstractions are introduced. Each phase is one PR. Total: ~33 findings closed; 8 deliberately deferred (see "Out of Scope").

Sequencing principle baked in throughout: phases 1-2 are safety nets, phase 3 fixes the engine boundary direction, phase 4 introduces DI seams and tests, phase 5 carves up `multiplayer.svelte.ts`, phase 6 migrates the inspector. The big T1 (multiplayer god module) and T5 (inspector migration) moves intentionally sit at the END so they ride on top of the safety net and DI work — otherwise they'd be high-risk refactors of code with no test floor.

Findings referenced by IDs from the 8-angle architecture audit (T*=top cross-cutting, M*=modularity, D*=data flow, E*=error handling, T_test*=testability, P*=performance, X*=extensibility, S*=security).

---

## Phase 1 — Untrusted-input gate + silent-failure surfacing

**Goal:** Make every place that ingests external bytes validate them, and stop swallowing the failure modes that today produce invisible desync/wedge.

**Findings addressed:** T2, T3, S1, S3, S4, E3, E4, E5, T6 (partial — silent wedge surfacing only)

### Stage 1a — Snapshot validator (T2, T3, S1)

Add `src/lib/engine/snapshot-validator.ts` exporting a single typed gate:

```ts
export interface SnapshotValidationOpts {
  maxActions: number;          // hard cap on actions[].length
  maxJsonBytes: number;        // hard cap on input string length
  requireConfig: boolean;      // false when entry point is "fresh start"
  source: "host-snapshot" | "joiner-paste" | "library-handoff" | "idb-resume" | "phase-change";
}
export type ValidatedSnapshot = { json: string; actionCount: number };
export function validateSnapshot(raw: unknown, opts: SnapshotValidationOpts): ValidatedSnapshot;
// throws SnapshotValidationError with `.source` + `.reason` for UI mapping
```

Validates: `typeof raw === "string"`, `raw.length <= maxJsonBytes` (suggested 4 MiB), JSON parse, `start_fen` is a string matching `^[1-8KkRrFfSsBbAaWwGgMmDd/]+\s\w\s.*$` (loose; engine still arbitrates), `actions` is an Array of `{raw:number}` shape with `Number.isInteger(raw) && raw >= 0 && raw <= 0xffffffff`, `actions.length <= maxActions` (suggested 4096 for resume / 1024 for paste), `config` shape sanity-checked when `requireConfig`.

Call sites to retrofit (file:line):
- `multiplayer-engine.ts:204` (host→joiner snapshot)
- `multiplayer-engine.ts:221` (host→joiner phase-change)
- `routes/inspector/+page.svelte:142,197,244` (paste / library handoff)
- `routes/match/+page.svelte:490` (pendingSnapshotJson — IDB resume)
- `routes/match/+page.svelte:1227` (sandbox restore — trusted, but cap still helps catch corruption)
- `routes/replay/+page.svelte:90` (handoff)

On validation failure: surface a typed error on `mpState.lastError` (consumed by the new banner — see stage 1b), do NOT silently fall through. For joiner: send `request-snapshot` with reason `"audit-mismatch"` so host can retry.

### Stage 1b — Surface `mpState.lastError`, clearable `bootError`, retry budget for joiner snapshot-request loop (T3, E3, E5)

Currently `mpState.lastError` is written from many catches and read by nobody. Add an HUD banner in `+layout.svelte` (or extend `ConnectivityPill`) that displays last MP error with a dismiss button — also clears the value. Replace `/* noop */` catches that hide protocol failures with a typed `warn("stage", e)` call into the renamed `mpState.lastError`.

For T3: in `multiplayer-engine.ts:288-307`, after sending `request-snapshot`, set a 10s timer; on expiry retry once, then on second expiry fire `onCheatDetected`-equivalent (or new `onResyncFailed`) so UI surfaces "lost sync with host" instead of looping forever.

`bootError` in `routes/match/+page.svelte:1361-1362` — make the banner dismissible, and auto-clear on successful re-applied action.

### Stage 1c — AI wedge detection (T6 surface only — full latch fix in phase 2)

`match/+page.svelte:771-774` and `draft/+page.svelte:371`: when `stepAi` returns `appliedAction === 0` AND `gameResult === 0`, set a UI flag `aiWedged = true`, surface a toast "AI returned no move — pausing", and `aiAutoPlay = false`. Don't try to recover here — make the wedge visible.

### Stage 1d — Settings.load type-checked (S3)

`settings.svelte.ts:48-54` — replace `{ ...DEFAULTS, ...parsed }` with a per-field validator: `audioVolume` clamped to `[0,1]`, `locale` against an allowlist, `pNThinkTimeMs` `Number.isFinite` and `>= 0`, etc. Tampered fields fall back to default silently.

### Stage 1e — Drop deprecated `bodyguard-prompt` decoder branch (S4)

`multiplayer-protocol-v2.ts:290-309` — delete. Update the route subscription in `match/+page.svelte:589-595` to remove the `bodyguard-prompt` no-op. One release of explicit cutover communicated by `mpState.lastError` if an old client sends it.

### Stage 1f — Drop async work from `BeforeUnloadEvent` (E4)

`match/+page.svelte:1269-1277` — replace async finalize with synchronous `markNetworkLost` write via `navigator.sendBeacon` (or accept the loss). Move the real finalize into the existing onDestroy path.

**Verification:** vitest (new tests: `snapshot-validator.test.ts` covering each failure mode + each source); svelte-check; manual smoke for paste-into-inspector, library→replay handoff, joiner mid-match drop, AI wedge toast.

**Risks:**
- Validator over-tight could break legitimate resumes from older log formats — keep validation generous on `config` shape, strict on `actions[].raw`.
- Surfacing `lastError` will reveal noise we currently swallow; budget ~30 min for tuning banner thresholds.

**Out of scope:** S5 (seq overflow) deferred to phase 2 with the seq arithmetic.

---

## Phase 2 — AI scheduler + WASM-death + seq arithmetic correctness

**Goal:** Fix the three hard-correctness bugs that produce silent wedges or unrecoverable engine state.

**Findings addressed:** T6 (latch race + scoped timeout), E1, E2, S5, S2, P1

### Stage 2a — AI scheduler refactor (T6)

Replace the `aiScheduled` boolean + bare `setTimeout` in `match/+page.svelte:428-448` and `draft/+page.svelte:371` with a single owned handle:

```ts
let aiTimer: ReturnType<typeof setTimeout> | null = null;
function cancelAiTimer() { if (aiTimer) { clearTimeout(aiTimer); aiTimer = null; } }
$effect(() => {
  if (!shouldQueueAi()) return cancelAiTimer();
  if (busy || aiTimer) return;
  aiTimer = setTimeout(async () => {
    aiTimer = null;
    if (!shouldQueueAi() || busy) return;
    await runAiStep();
  }, delay);
});
onDestroy(cancelAiTimer);
```

The microtask window closes because `aiTimer` is set synchronously and only cleared inside the timer callback or by teardown. `busy` flag is no longer the latch.

### Stage 2b — WASM worker-death recovery (E1, P1)

`engine/wasm-client.ts:36-42` — on `worker.onerror`, set an internal `#dead = true` flag AND clear the cached engine in `engine/index.ts`. New `#call` checks `#dead` and rejects synchronously. Add `getEngine()` re-create path. In `+layout.svelte` HMR hook (or vite plugin handler), call `resetEngine()` + worker terminate.

### Stage 2c — TauriClient handle-zero guard (E2)

`tauri-client.ts:90-211` — every method that uses `#handle` checks `if (this.#handle === 0) throw new Error("engine not initialized");` rather than only `dispose()`.

### Stage 2d — Seq overflow + per-intent rate cap (S5, S2)

`multiplayer-engine.ts:280` — change `m.seq !== seq + 1` to handle wraparound explicitly OR cap `seq` at `0xfffffff0` and emit a forced `snapshot` to reset both sides to seq=0 on rollover. Add a `lastIntentMs` field; if more than 30 intents arrive in 1 second from joiner, throttle with `intent-rejected` reason `"rate-limit"` (new reason — extend `IntentRejectReason` union).

**Verification:** vitest (new tests: AI scheduler unit-shaped via $effect.root harness; seq wraparound; intent flood); svelte-check; manual smoke for AIvAI long run, force-kill worker via devtools, mid-match joiner re-dial.

**Risks:**
- WASM re-create path may not preserve pendingSnapshotJson — make sure resume hooks call `restoreFromSnapshot` after re-init.
- Rate cap could refuse legitimate fast play on flaky links — tune threshold.

**Out of scope:** Full EngineClient contract test (defer to global out-of-scope — see end).

---

## Phase 3 — Engine-boundary direction fix + state-store cleanup

**Goal:** Engine layer becomes leaf-with-respect-to-state, not the other way around. Telemetry session globals become per-instance.

**Findings addressed:** T4, T7, T8, M1, M3, D1, D2

### Stage 3a — Eliminate engine→state imports (T4 down→up)

- `engine/config.ts:8-9` imports `settings` and `SeatKind` from `$lib/state/...`. Invert: caller passes the data in.
  ```ts
  export function buildEngineConfigJson(input: {
    p1: "Human"|"Ai"; p2: "Human"|"Ai";
    p1Ai: { timeLimitMs: number; maxDepth: number };
    p2Ai: { timeLimitMs: number; maxDepth: number };
    aivaiStepDelayMs: number;
  }): string
  ```
  Two callers (`match/+page.svelte:484`, `inspector/+page.svelte:141` — via `defaultConfigJson()`) read settings + side from `match`/`settings` and pass the bag.
- `storage/types.ts` imports `MatchMode` from `state/match-store` — move `MatchMode` definition into `storage/types.ts` (or a new `lib/types/mode.ts`) and have `match-store` re-export. Engine + storage become pure leaves.

### Stage 3b — Route imports via engine barrel only (T4 up→down)

`engine/index.ts` already exists; add re-exports for `mailbox`, `config`, `action`, `action-label`, `skills`, and the new `snapshot-validator`. Routes/components stop reaching into `engine/<subfile>` directly. Eslint rule (`no-restricted-imports`) blocks `$lib/engine/(mailbox|config|action|action-label|skills)` outside `lib/engine` and `lib/board`.

### Stage 3c — Single source of truth for MP role/code (T7)

Today: `mpState.role/code`, `match.multiplayerRole/multiplayerCode`, wrapper-internal `role/codeRef`. Move authoritative role+code into `mpState` ONLY; `match` reads via `$derived` (or a function call); `createMpEngine` reads through `deps` rather than holding `codeRef`. The handoff path (`promoteToHost`) becomes a single `mpState.role = "host"` write — wrapper picks it up via dep.

This unblocks future spectator role (X1, X2) — though we don't open the union yet, we stop multiplying writers.

### Stage 3d — Telemetry-session per-instance state (T8)

`telemetry-session.ts:19` — wrap module in a factory:
```ts
export function createTelemetrySession(): {
  startTelemetrySession, recordPly, finalize..., abandon..., networkLost..., reset()
}
```
`match-store` instantiates once on module load and re-exports; tests instantiate fresh. `telemetryDisabledForSession` becomes closed-over per instance, NOT module-scoped, and gets reset on every `abandon`/`networkLost`/`startTelemetrySession`.

### Stage 3e — Untangle store/UI writes (M1, M3, D1, D2)

- `claimWinByOpponentForfeit` (match-store.svelte.ts:121-179) — move engine IO + `match.position` write into `routes/match/+page.svelte`'s teardown logic. The store function returns the resultByte; caller orchestrates.
- `GraceBanner.svelte` — strip state writes; emit events / accept callbacks. Parent (match route or layout) owns the writes.
- `pendingLocalRaw` (D1) — replace with a `Set<number>` of inflight raws OR encode dedup at the wrapper level (`originNonce` already exists; piggyback). Eliminates same-raw-twice race.
- `onApplied` joiner stale-read (D2) — make pre-state snapshotting explicit and type-enforced: wrapper passes `{ raw, prePositionView }` to `onApplied`. Caller no longer relies on call-order discipline.

**Verification:** vitest (all 139 existing pass + new factory tests for telemetry-session); svelte-check; prod build (eslint import-restriction errors caught here); manual smoke for HvAI, multiplayer host+joiner, claim-win.

**Risks:**
- **State-write reactivity changes:** Moving `claimWinByOpponentForfeit` engine writes out of the store changes `$state` reactivity. Verify the terminal banner still fires by writing through `match.position` in the new location.
- Refactor surface is big — split into individual commits within the PR if review velocity matters.
- ARCHITECTURE.md §6, §7, §8 need updating in same PR.

---

## Phase 4 — DI seams + PlyRenderer unit tests

**Goal:** Make PlyRenderer testable WITHOUT writing tests against globals. Add the seams first, then the tests.

**Findings addressed:** T_test1, T_test2, T_test3, P2, P3

### Stage 4a — PlyRenderer DI seams (T_test1, P2)

Extend `PlyRendererOpts` (`lib/board/ply-renderer.svelte.ts:48`):
```ts
clock?: { now(): number };                            // default: performance
sfxImpl?: { play: typeof sfx.play };                  // default: import sfx
warn?: (stage: string, detail?: unknown) => void;
scheduler?: { setTimeout, clearTimeout };             // default: window
```
All `performance.now()` calls (290, 305, 338, 470) route through `opts.clock.now()`. All `sfx.play` (~10 sites) go through `opts.sfxImpl`. The `triggerShake` setTimeout owns its handle in a `Set<ReturnType<typeof setTimeout>>` that `dispose()` clears — fixes P2.

### Stage 4b — EffectsLayer RAF gating (P3)

`EffectsLayer.svelte` — stop the RAF loop when `effectQueue.length === 0` AND no shake is in flight; restart on next `effectQueue.push` via a reactive `$effect` watching `queue.length > 0`.

### Stage 4c — PlyRenderer unit tests (T_test1)

New `lib/board/ply-renderer.test.ts` (~10-15 tests). Inject fake clock + fake sfx; drive `applyAndRender` with a stub `EngineClient`. Covered: piece-id reconciliation, deferred-skill-refresh drain, effect emission for damage/heal/armor/death, fastForwardTo restore+replay shape, shake timer cleanup.

### Stage 4d — IDB-backend test ULID spacing (T_test2)

`idb-backend.test.ts:158,160` — replace real `setTimeout` with a deterministic ULID factory injection. If `idb-backend.ts` doesn't already accept a ULID factory, add one via constructor / module-level setter.

### Stage 4e — multiplayer.svelte.test.ts await ordering (T_test3)

Replace `void join(...).catch()` (line 64-78) with awaited boundaries on the wrapper's open/error promises. Refactor the test to expose explicit resolution points rather than racing the timeout.

**Verification:** vitest; ~10-15 new tests pass; existing 139 still pass; svelte-check; no manual smoke needed (pure test scaffolding).

**Risks:**
- DI seams change PlyRenderer construction signature — match/replay/inspector all need a default-args update.
- ARCHITECTURE.md §4 ("Adding a new effect is a one-place edit") still holds but mention DI.

---

## Phase 5 — Multiplayer god-module split

**Goal:** Carve `multiplayer.svelte.ts` (635 LOC) into single-concern modules. Done after phases 1-4 so the safety net is in place and DI exists.

**Findings addressed:** T1, X1 (partial), M2

This is the riskiest phase. Stage carefully; the wire layer is load-bearing across two routes + grace UI. Do NOT change wire-format in this phase (would break `pendingSnapshotJson` cross-route handoff).

### Stage 5a — Extract transport layer (PeerJS + reconnect ladder) into `lib/multiplayer/transport.ts`

```ts
export interface Transport {
  host(): Promise<string>;
  hostWithCode(code: string): Promise<string>;
  join(code: string): Promise<void>;
  disconnect(): void;
  destroyPeerKeepState(): void;
  sendRaw(raw: string): void;
  onRawData(cb: (raw: string) => void): () => void;
  probeHost(code: string, timeoutMs?: number): Promise<boolean>;
}
export function createPeerJsTransport(opts: {
  idPrefix?: string;            // X1 seam — no longer hard-coded
  redialDelays?: number[];      // policy injected
  clock?: { now(): number };
  log?: (event: string, detail?: unknown) => void;
}): Transport;
```

All `console.log` (13 sites) route through `opts.log`. `REDIAL_DELAYS` becomes a parameter, not a const. `ID_PREFIX` becomes a parameter. X1's first half lands here.

### Stage 5b — Extract pill state machine into `lib/multiplayer/pill-state.ts`

`pillState()` (line 142-162) is a getter with side effects. Extract as a pure function:
```ts
export function derivePillStateWithAnchor(input: {
  status, lastPongAt, now, peerEverPaired, disconnectedSince
}): { pill: PillState; nextDisconnectedSince: number | null };
```
The reactive `mpState` $effect computes both and assigns `disconnectedSince` in the effect body, not inside the getter. Worst-SoC violation closed.

### Stage 5c — Extract heartbeat/now-tick into `lib/multiplayer/heartbeat.ts`

Owns `pingTimer` + `nowTimer` + pong-age-out bridge (line 115-140). Returns `{ start, stop, lastPongAt: () => number }`. mpState's pong-age-out subscribes to this.

### Stage 5d — Slim `multiplayer.svelte.ts` to its real job: the `mpState` $state + thin facade

After 5a-c, `multiplayer.svelte.ts` shrinks to ~150 LOC: the rune-backed state plus exported `host/join/disconnect` wrappers that call transport. `lastError` is now meaningfully consumed by the banner from phase 1b.

### Stage 5e — Remove `multiplayer-handoff.ts` dynamic-import indirection (M2)

`multiplayer-handoff.ts` uses `await import(...)` to break a cycle. After phase 3's boundary cleanup, the cycle is gone; convert to static imports so dependency analyzers see the truth.

**Verification:** vitest (existing `multiplayer*.test.ts` should still pass — fakes can target new module boundaries); svelte-check; **mandatory manual smoke**: solo→host start, host+joiner full match, joiner mid-match drop+auto-redial, host tab-crash+rejoin, grace-banner countdown, claim-win.

**Risks:**
- **MP wire-format invariance:** This phase must NOT change wire format. Re-grep for `encodeMessage`/`decodeMessage`/`encodeMessageV2`/`decodeMessageV2` after the refactor — call sites untouched.
- **Cross-route handoff:** `pendingSnapshotJson` flows multiplayer → /match/. Verify with a manual rebuild (paper checklist) that a joiner who connects in the lobby and navigates to /match/ during the handoff window still drains the rawInbox.
- ARCHITECTURE.md §7 needs near-total rewrite.

**Out of scope:** Spectator role / broadcast topology (X1 full, X2). Defer to ADR.

---

## Phase 6 — Inspector → PlyRenderer migration + replay perf (✅ SHIPPED — Session 33)

**Goal:** Inspector stops reimplementing PlyRenderer in parallel. Replay scrubbing gets cheaper.

**Findings addressed:** T5, P4, S1 (residual — paste limit)

**Status:** All four stages complete. 220 tests passing (5 new ai-hooks, 4 new checkpoint), 0 svelte-check errors, prod build clean.

This is the biggest atomic move in the plan. Phases 1 (paste cap), 3 (engine barrel), and 4 (DI tested renderer) all set it up.

### Stage 6a — Migrate inspector boot/restore-per-node to PlyRenderer ✅

Inspector's local `pieceIds` / `nextPieceId` / `refreshPieceIds` removed. `syncEngineToNode` now drives `renderer.fastForwardTo(baseSnap, node.actions, node.actions.length)`; `applyActionToCurrent` routes through `renderer.applyAndRender`. Board template binds `renderer.pieceIds` + `renderer.shakingSquares`; `EffectsLayer` mounted inside `.board-wrap`. The boot-time tree-build loops (`entryFromMatchLog`, `entryFromSnapshotJson`) deliberately remain outside the renderer — no UI is up during boot, and the user's first node selection hydrates via `fastForwardTo` with checkpoint caching from 6c.

### Stage 6b — Replace `window.prompt` for POI labels ✅

`src/lib/inspector/PoiLabelDialog.svelte` — native `<dialog>`-based modal with focus/ESC trap. Mounted at the route root in `inspector/+page.svelte`; replaces `window.prompt` at the POI-mark handler.

### Stage 6c — Replay fastForwardTo checkpoints (P4) ✅

`ply-renderer.svelte.ts` now keeps a module-internal `Map<plyIndex, snapshotJson>` keyed by `(baseSnapshotJson, ply) → snap`, stride 32. Two write paths: (1) captured inside `fastForwardTo`'s silent inner loop at stride multiples; (2) captured opportunistically by `applyAndRender` when callers pass `{ plyHint, plyHintBase }` (replay's `stepForward` does this). Read path: `fastForwardTo` invalidates on base change, restores from nearest checkpoint < target if savings ≥ 4 plies. Cache cleared by `reset()` and `dispose()`. 200-ply scrub: was N round-trips, now ≤ 31.

### Stage 6d — Inspector AI iterative-deepening hook unification ✅

`src/lib/engine/ai-hooks.ts` — `runAiCall<T>(fn, opts?)` shell with `AiCallError` (`reason: "timeout" | "cancelled" | "engine"`). Adopted at two call sites: `routes/match/+page.svelte` `stepAi` and `routes/inspector/+page.svelte` `requestAiMoveAtDepth` (inside the deepening loop, with `cancelled: () => aiCancelRequested`). Inspector's catch block swallows `AiCallError("cancelled")` — user-driven stops don't surface as errors. No timeouts wired today; the hook is the future-proof seam.

**Verification:** vitest 220/220 ✅; svelte-check 0/0 ✅; prod build ✅. **Manual smoke (pending):** paste a match log, scrub through a 200-ply replay, label POIs, navigate library → inspector → replay → /match/ resume.

**Risks resolved:**
- Inspector regression surface: Checkpoint caching from 6c lands first, so deep-tree node selection (~60 plies in) only replays ~30 plies after restoring from the nearest cached checkpoint.
- ARCHITECTURE.md §9 updated to reflect Inspector-now-uses-PlyRenderer.

---

## Phase summary

| # | Goal | Findings closed | Size | Verification depth |
|---|---|---:|---|---|
| 1 | Trust gate + surface silent failures | T2,T3,S1,S3,S4,E3,E4,E5,T6(part) — 9 | M | vitest + svelte-check + manual smoke |
| 2 | AI scheduler + WASM-death + seq | T6,E1,E2,S5,S2,P1 — 6 | M | vitest + manual smoke |
| 3 | Engine-boundary + state cleanup | T4,T7,T8,M1,M3,D1,D2 — 7 | L | vitest + svelte-check + prod build + smoke |
| 4 | DI seams + PlyRenderer tests | T_test1,T_test2,T_test3,P2,P3 — 5 | M | vitest only |
| 5 | Multiplayer god-module split | T1,M2,X1(part) — 3 | L | vitest + extensive manual smoke |
| 6 | Inspector migration + replay perf | T5,P4,S1(residual) — 3 | L | vitest + extensive manual smoke |

Total closed: ~33 distinct findings across 6 PRs.

---

## Global risks

1. **ARCHITECTURE.md drift.** Each phase MUST update the doc in the same commit/PR. Particularly: phase 3 changes §6/§7/§8, phase 5 rewrites §7, phase 6 updates §4 and §9.
2. **State-write reactivity changes during store refactors (phase 3).** Svelte 5 runes track at read-site, not write-site — moving a write across modules can silently lose reactivity. Mitigation: every store-write move includes a deliberate read-back test in the route.
3. **MP wire-format stability through phase 5.** A protocol-shape change during the god-module split would break `pendingSnapshotJson` cross-route handoff AND any tab that resumed mid-deploy. Grep verification at end of phase 5.
4. **Cross-platform parity drift (wasm vs tauri).** Phase 2's WASM recovery has no parallel in tauri-client. Accept asymmetry but document.
5. **Test scaffolding leaking into prod bundles.** Phase 4's DI seams default to current globals — verify tree-shake.

---

## Out of scope (deliberately deferred)

- **AI player abstraction** (no finding ID — implied by X1/X2). Feature decision; needs ADR before any debt fix.
- **Game-phase FSM.** Same. Architecture.md §"What's missing" already lists.
- **EngineClient contract test** (T_test4). Cost: write parity harness driving both clients through identical scripted inputs. Benefit: catches drift between WASM and Tauri engines. Verdict: **defer** — the desktop-build smoke test catches drift in practice, and the harness would re-invent the engine's own integration tests in JS. Revisit if a drift bug ships.
- **`undo_ply`/`seek_to_ply` engine APIs.** Engine-side work, not a frontend remediation. Phase 6's checkpoint scheme is the right frontend bet until that lands.
- **Opening the `Role` union for spectator** (X1 full, X2). Phase 3 stops multiplying writers; the actual union expansion is a feature, not debt. ADR before doing it.
- **Board.svelte 15-prop grouping.** Cosmetic; defer.
- **Drag service / skill-targeting service extraction.** Listed in ARCHITECTURE.md as opportunities, not bugs.
- **Audio engine abstraction.** Currently a single `sfx.play` call; DI seam in phase 4 covers test needs. A real abstraction is overkill for current scope.
