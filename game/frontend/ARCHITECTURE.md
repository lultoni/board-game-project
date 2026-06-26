# Frontend Architecture (Session 33, 2026-06-26)

Living map of the SvelteKit frontend at `game/frontend/`. Updated when significant restructuring lands. Not a tutorial — assumes you've read the code at least once.

---

## Layers (top down)

### 1. Routes (`src/routes/`)

| Route | LOC | Role |
|---|---:|---|
| `+page.svelte` (root) | 142 | Menu. Backend detect (wasm/tauri). Lists resumable MP sessions. |
| `setup/+page.svelte` | 430 | Seat selection (HvH/HvAI/AIvAI), draft mode, MP role. Writes `match` store, hands off to /draft/. |
| `draft/+page.svelte` | 1,146 | 12-ply draft. Skill picker per turn, MP coordination, AI drafting. |
| `match/+page.svelte` | 1,807 | Main game loop. **Fat controller** — see §10. Effects pipeline extracted to `ply-renderer.svelte.ts`. |
| `replay/+page.svelte` | 392 | MatchLog playback. restoreFromSnapshot + tryApply loop through `PlyRenderer`. |
| `inspector/+page.svelte` | 877 | Position analysis. Same data loader as replay + AI search hooks. |
| `library/+page.svelte` | 459 | TelemetryStore browser. Filter + export + hand off to replay/inspector. |
| `multiplayer/+page.svelte` | 734 | Lobby (PeerJS code generation + join). |

**Signal**: match is ~2× draft/inspector/multiplayer. Orchestration lives there.

### 2. Engine boundary (`src/lib/engine/`)

- **types.ts** — `EngineClient` interface (18 methods). Covers L8 draft (`createEngineWithDraft`, `createEngineWithLoadouts`, `draftState`), live play (`tryApply`, `stepAi`), inspector AI hints (`requestAiMoveForced`, `requestAiMoveAtDepth`), reads (`positionView`, `legalActions`, `positionFen`), persistence (`snapshotJson`, `restoreFromSnapshot`, `matchLogJson`, `latestPlyJson`, `finaliseLog`), and lifecycle (`createEngine`, `version`, `dispose`). Returns: `PositionView`, `StepResult`, `DraftStateView`, `PendingBodyguardView`.
- **index.ts** — Runtime backend detect; singleton `getEngine()` cached for the page lifetime. `resetEngine()` for tests/HMR.
- **wasm-client.ts** / **worker.ts** — Browser path. Messages over a Worker to `wasm_wrapper.js`. Worker holds one engine; `createEngine*` overwrites in place.
- **tauri-client.ts** — Desktop path. Routes engine calls to Rust IPC commands. `#replaceHandle(...)` drops the prior registry handle on every `createEngine*` / `restoreFromSnapshot` so route re-entry doesn't leak Rust-side `Match` records.
- **action.ts** — u32 codec. `ActionKind`, BodyguardChoice (bit 31), DraftTurn (bit 30).
- **action-label.ts** — Human-readable formatting.
- **skills.ts, mailbox.ts, config.ts** — Skill metadata, mailbox decoder, config JSON builder.

### 3. Board rendering (`src/lib/board/`)

- **Board.svelte** — SVG grid renderer. Pure-ish: parent owns `position`, `pieceIds`, all interaction state. Big props surface (~15 props) — see §10 for the "what does this say" reading.
- **Piece.svelte, SkillWheel.svelte, SkillInfoCard.svelte, DirectionPicker.svelte, SkillGlyphDefs.svelte** — leaf renderers.
- **EffectsLayer.svelte** — Canvas overlay; drains `effectQueue` (see §4).

### 4. Visual effects pipeline (`src/lib/board/ply-renderer.svelte.ts` + EffectsLayer)

- `Effect` discriminated union (in `src/lib/viz/effects.ts`): `dust | impact | damageNumber | shake | heal | armor`.
- **Producer: `lib/board/ply-renderer.svelte.ts`.** Stateful driver. Both `/match/` and `/replay/` create one via `createPlyRenderer(eng, opts)` and call `applyAndRender(raw, applyFn)` per action. Owns `pieceIds`, `shakingSquares`, `effectQueue`, deferred-skill-refresh state, and the current rendered `position` (writes through to the supplied `positionSink` so `match.position` stays the source of truth for match).
- **Consumer: EffectsLayer.svelte.** Reactive canvas; effects expire on `FX_LIFETIME_MS`. Bound via `bind:queue={renderer.effectQueue}`.
- Adding a new effect (sound or visual) is a one-place edit.
- **SFX policy:** `sfxEnabled` opt gates `sfx.play` calls. Match passes `true`; replay passes `false`.

### 5. Audio (`src/lib/audio/sfx.ts`)

WebAudio synthesis only (no asset files). `sfx.play(event, opts?)`. Called from `ply-renderer.svelte.ts` (all match-mode sounds), gated behind `sfxOn()` so replay can disable.

### 6. State stores (`src/lib/state/`)

- **match-store.svelte.ts** — `match` carrier (mode, sides, position, legal, selection, lastApplied, draft mode, sandbox snapshot, telemetry id, MP role/code/seat). Re-exports telemetry lifecycle for ergonomics; actual implementation in `telemetry-session.ts`.
- **telemetry-session.ts** — `startTelemetrySession`, `recordPly`, `finalizeTelemetrySession`, `networkLostTelemetrySession`, `abandonTelemetrySession`, `claimWinByOpponentForfeit`. Uses `latestPlyJson` for incremental per-ply persistence (avoids O(n²) re-serialisation).
- **settings.svelte.ts** — Persisted user prefs (audio, locale, AI budgets, aivaiStepDelayMs).
- **i18n.ts** — Translation helper.
- **move-targets.ts, skill-targets.ts** — Derived legality from `PositionView` + `legalActions`.
- **draft.ts** — Pre-made loadouts, draft geometry helpers.
- **geometry.ts** — Board geometry helpers.
- **inspector-store.svelte.ts** — Inspector-route state (tree, current node, AI search progress).

### 7. Multiplayer (`src/lib/multiplayer*` + `src/lib/multiplayer/`)

- **multiplayer.svelte.ts** — PeerJS wrapper, `mpState` carrier, ping/pong.
- **multiplayer-protocol-v2.ts** — Current wire protocol (intent/committed/snapshot/...).
- **multiplayer-protocol.ts** — Heartbeat (`ping`/`pong`) + broker `error` frame + `generateCode` / `isValidCode` / `GRACE_MS`. Still imported by `multiplayer.svelte.ts`, the lobby route, and `GraceBanner.svelte`. Not a delete candidate.
- **multiplayer-engine.ts** — Role-aware wrapper over `EngineClient`. Host: tryApply + broadcast. Joiner: send intent, await committed, audit via mirror engine + zobrist check.
- **multiplayer-resume.ts** — Helpers for the host Rejoin handshake: `snapshotJsonFromMatchLog` rebuilds a Snapshot from a persisted log; `logIsMidDraftCheap` routes to /draft/ vs /match/ without booting WASM. Zobrists in the log are not touched here — V2 protocol uses the live `PositionView.zobrist` bigint instead.
- **multiplayer-handoff.ts** — Cross-route handoff state (which match resumes into which route).
- **multiplayer/ConnectivityPill.svelte, GraceBanner.svelte** — Status UI components.

### 8. Storage / telemetry (`src/lib/storage/`)

- **idb-backend.ts** — IndexedDB backend. `MatchMeta` rows + per-ply `PlyEntry` records. `startMatch`, `recordPly` (incremental, not full re-serialise), `finaliseMatch`, `listMatches`, `bundleMatches`.
- **tauri-backend.ts** — Desktop backend (file-based). Same interface.
- **index.ts** — Runtime backend detect (mirrors `engine/index.ts`).
- **types.ts** — Shared types crossing backend boundary.
- **library-handoff.ts** — One-shot sessionStorage cell to pass a MatchLog from /library/ → /replay/ or /inspector/.

### 9. Replay / Inspector data flow

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

Replay drives the apply loop through `PlyRenderer` (so skill effects + slide animations work the same way they do in /match/). Inspector still drives its own; migration pending.

---

## 10. The match/+page.svelte fat controller

1,807 lines, by concern (post-`PlyRenderer` extraction):

| Lines | Concern |
|---|---|
| 1–270 | Imports + state declarations (drag, approach chooser, armed skill, focus/charge prefs, toast, MP, …) |
| 270–410 | Derived state (currentSeatIsAi, moveTargets, selectable, wheelOpen, armedSkillTargets, …) |
| 410–600 | Lifecycle: engine boot via `renderer = createPlyRenderer(...)`, MP init, AI scheduler `$effect` |
| 600–680 | MP engine wrapper wiring (`onApplied`, `onSnapshotApplied`, `onHostCommitted`) |
| 680–800 | **Apply orchestration** (`applyRaw`, `afterApplied`, `runAiStep`). Delegates effect rendering to the `PlyRenderer`. |
| 800–1100 | Input handlers (square click, drag, drop, wheel slice, direction picker, skill targeting) |
| 1100–1200 | Sandbox lifecycle |
| 1200–1340 | MP state machine (resume, grace, claim-win, telemetry finalisation) + unload guards |
| 1340–1581 | Markup |
| 1581–1807 | Styles |

**Concerns juggled in one file:** engine lifecycle, action application orchestration, MP wire coordination, telemetry, AI scheduling, drag UI, skill targeting + modifiers, modal choosers, sandbox isolation, export. Visual-effects rendering and SFX now live in `PlyRenderer`.

---

## Key seams (where the layers meet)

1. **applyRaw → renderer.applyAndRender** — caller passes an apply closure; the renderer snapshots pre-state, runs the closure (which moves the engine), then renders effects/SFX, then flips position (sometimes deferred via `RELOC_DELAY_MS` so impact animation lands on the pre-state board).
2. **mpEngine wrapper** — sits between input handlers and `eng.tryApply`. Host applies directly; joiner sends intent and re-applies on committed echo.
3. **Telemetry lifecycle** — `startTelemetrySession` → `recordPly` per apply (uses `latestPlyJson`) → `finalize` / `networkLost` / `claimWinByOpponentForfeit` on terminal events.
4. **AI scheduler** — `$effect` watches `currentSeatIsAi + aiAutoPlay`, queues `runAiStep` with `aivaiStepDelayMs` delay.
5. **Sandbox** — saves `snapshotJson` on entry, restores on exit, all moves discarded.

---

## Notable patterns

- **Pure-ish Board renderer.** Board.svelte takes ~15 props but doesn't own game state. Parent computes everything (targets, drag trail, wheel state) and feeds it in.
- **Deferred state flip for skill animations.** `RELOC_DELAY_MS = 260ms` for skill actions with relocations/deaths — impact animation plays on pre-state, then position flips. `drainPendingSkillRefresh()` cancels pending flip if a new action commits first.
- **Bodyguard state lives in the engine**, not a frontend store. `PositionView.pendingBodyguard` + restricted `legalActions` buffer drive the chooser.
- **`match.position` is a cached pull from the engine, not duplicated state.** Engine remains the authority; `refresh()` and `renderer.resyncFromEngine()` write through. Code reads `match.position` after every apply has rendered — the apply pipeline is structured so every read is post-write or pre-apply.
- **Telemetry methods on the match carrier.** `startTelemetrySession()` is a method on `match` for ergonomics; implementation in `telemetry-session.ts`. Tight coupling.
- **Procedural audio.** No assets. Easy to extend, but no audio-engine abstraction — every caller imports `sfx` directly.

---

## 11. Testing

148 tests across `*.test.ts` files, all under `vitest`:

| Test file | Covers |
|---|---|
| `multiplayer-engine.test.ts` | Role-aware wrapper (host/joiner/solo paths, intent/committed handshake, zobrist audit) |
| `multiplayer-protocol-v2.test.ts` | Wire-format encode/decode + validation |
| `multiplayer-protocol.test.ts` | Legacy V1 heartbeat + utility helpers |
| `multiplayer-resume.test.ts` | `snapshotJsonFromMatchLog`, `logIsMidDraftCheap` |
| `multiplayer-handoff.test.ts` | Cross-route handoff state |
| `multiplayer.svelte.test.ts` | PeerJS wrapper + `mpState` |
| `idb-backend.test.ts` | IndexedDB telemetry CRUD |
| `library-handoff.test.ts` | One-shot sessionStorage handoff |
| `telemetry-session.test.ts` | Lifecycle (start/record/finalise/networkLost/abandon) |

**Coverage gaps:** routes (no end-to-end), `ply-renderer.svelte.ts` (no unit), Board/EffectsLayer (visual). The route layer is verified by manual smoke + `svelte-check` + production build.

---

## Observed extraction opportunities (no decisions yet)

1. ~~**Ply renderer.**~~ **DONE (Session 33).** Extracted into `lib/board/ply-renderer.svelte.ts`. Match + replay share one effects pipeline. Inspector migration still pending.
2. **Skill targeting service.** Arm/disarm + modifier state lives in match, but legal targets are in `skill-targets.ts`. Could own the full lifecycle.
3. **Drag service.** ~30 lines of drag state (dragSrc, dragTrail, dragHover, cursorXY, pendingApproach) is reusable shape.
4. **Multiplayer facade.** ~200 lines of mpEngine wiring in match. Could expose a single match-shaped API.
5. **Telemetry finalizer.** Three terminal paths (finalize/networkLost/forfeit) → one resolver.
6. **Board props grouping.** Could group ~15 props into `movePhase`, `skillPhase`, `chooser` sub-objects.

---

## What's missing / would need a new layer

- **No AI player abstraction.** `eng.stepAi()` is called directly; routing is via `currentSeatIsAi` boolean. If we ever want multiple AI personalities or an external AI process, this is the seam.
- **No game-phase state machine.** Phase/turn logic is imperative `if (position.toMove === …)`. A FSM would help, but probably overkill at current size.
- **No effect/SFX abstraction beyond the queue.** If you wanted to add e.g. screen-shake intensity scaling, declarative effect bundles, or replay-mode skipping, there's nowhere obvious to plug it.
- **No engine boundary contract test.** `wasm-client` and `tauri-client` both implement `EngineClient`, but nothing enforces semantic parity — a method's behaviour could drift on one backend without the other noticing until a desktop-build smoke test catches it.
- **No `undo_ply` / `seek_to_ply` in the engine.** Replay's `fastForwardTo` does `restoreFromSnapshot` + N-1 `tryApply` calls per scrub (O(N) per jump). The engine has `make_unmake` internally for search but doesn't retain an Undo stack at the session level. A future engine enhancement would expose `undo_ply()` / `seek_to_ply(n)`.
