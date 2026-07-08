# UI cleanup + perf plan

*Written 2026-07-08. Anchor commit: `ec0cae4` (main). Rollback target if a phase misbehaves. See `.claude/eval-perf-passes.md` for the same doc-pattern used on the engine side.*

## Motivation

While AI is thinking, the UI slows down considerably — most visibly the ply renderer. Separately, sound effects stop working after the window has been open for a long time. Investigation (4 parallel Explore agents, 2026-07-08) established:

- **Threading is fine.** `tauri_wrapper/src/lib.rs:375` uses `tokio::task::block_in_place`; the search does not block the invoke handler. Event volume from search is 1–8 depth events per search, throttled to 100 ms on the frontend (`match/+page.svelte:977`). Not the bottleneck.
- **Ply renderer** itself (`lib/board/ply-renderer.svelte.ts`) is well-designed with proper `dispose()` — but the routes never call it, and depth-tick writes fan out to sibling components (`EvalBreakdownPanel`'s `$derived`, `PlayerPanel` re-renders) causing re-render cascades that visually manifest as ply-renderer stutter.
- **Audio** dies because `AudioContext` gets suspended on macOS WKWebView after long inactivity with no `statechange` listener and no pre-play resume check. Also: every SFX creates fresh oscillator/gain/buffer-source nodes that are stopped but never `.disconnect()`'d — zombie graph nodes accumulate.

Guiding principle from the user: **"clean up well after myself"** — window should minimize resources it holds, especially over long sessions.

## Scope: three PRs, seven phases

Ordered smallest/safest first. Each PR ships independently.

### PR 1 — Audio + renderer leak (low-risk) — **DONE 2026-07-08**

Landed together. svelte-check clean; test suite: no new failures (one pre-existing settings test failure inherited from `ec0cae4`, unrelated).

#### Phase 1 — Audio pre-play resume + statechange listener — **DONE**

Files: `game/frontend/src/lib/audio/sfx.ts`, `game/frontend/src/routes/+layout.svelte`.

- `sfx.ts:41-54` (`ensureCtx`): after creating the context, attach `ctx.onstatechange`. When the context transitions to `suspended` while `document.visibilityState === "visible"`, call `void ctx.resume()`. Guard with a small backoff (skip if last resume attempt was <1 s ago) to prevent re-entrant loops.
- `sfx.ts:93-99` (`playToneAt`): at the top, if `c.state === "suspended"`, fire `void c.resume()` before scheduling. First tone after resume may drop ~30 ms — acceptable.
- `+layout.svelte:20-24`: keep the existing `visibilitychange` handler; sanity-check that its `onMount`-returned cleanup is still wired correctly (Svelte 5 pattern).

Risk: minimal — `resume()` on a running context is a no-op.

#### Phase 2 — Audio node disconnection — **DONE**

File: `sfx.ts:97-160` (`playToneAt`).

- For each voice's node chain (env gain, osc, osc2, v2env, noise buffer source, nGain, hp filter), attach an `ended` handler on the *terminating source* nodes (oscillator, bufferSource — only source-shaped nodes fire `ended`) that calls `.disconnect()` on the source and every downstream gain/filter it feeds. Bundle each voice's chain into a small local array so one `ended` handler iterates the array.
- Wrap `.disconnect()` in try/catch — already-disconnected nodes throw in some engines.
- **No pooling.** JS engines GC unreferenced AudioNode wrappers once the graph releases them via `.disconnect()`. Revisit only if profiling still shows churn.

Risk: low. No tests today for `sfx.ts`; keep it that way.

#### Phase 3 — Renderer disposal on route exit — **DONE**

Files: `game/frontend/src/routes/match/+page.svelte:1574-1627` (existing `onDestroy`), `game/frontend/src/routes/replay/+page.svelte` (no `onDestroy` today).

- match: extend the existing `onDestroy` to call `renderer?.dispose(); renderer = null;` before the mp teardown block. `dispose()` already exists at `ply-renderer.svelte.ts:1157` and cancels all shake/deferred-skill timers and empties `effectQueue`.
- replay: add a full `onDestroy` block doing the same.
- inspector already does exactly this at `inspector/+page.svelte:349-353` — pattern is well-established.

Risk: near-zero.

---

### PR 2 — AI-transient state extraction — **DONE 2026-07-08**

Landed together. svelte-check clean; test suite: no new failures (same pre-existing `settings` test failure inherited from anchor). Extended to also rewire `/replay/` so `EvalBreakdownPanel` reads from the store uniformly across routes.

#### Phase 4 — New dedicated store — **DONE**

Files:
- **New:** `game/frontend/src/lib/state/ai-search.svelte.ts`
- Edited: `game/frontend/src/routes/match/+page.svelte`, `game/frontend/src/lib/match/PlayerPanel.svelte`, `game/frontend/src/lib/eval/EvalBreakdownPanel.svelte`.

New module exports a `aiSearch` rune-store with fields:
- `thinking: boolean`
- `lastDepth: number | null`
- `lastScore: number | null`
- `searchStartedAt: number | null`
- `finishedAtPly: number | null`
- `heuristicEvalBreakdown: EvalBreakdown | null`
- `heuristicEvalBySquare: EvalBreakdownBySquare | null`
- `prevRoundBreakdown: EvalBreakdown | null`
- `lastRoundSeen: number | null`

Plus helpers:
- `beginSearch()` — sets `thinking=true`, `searchStartedAt=now`.
- `updateDepth(d, s)` — 100 ms throttle **baked into the store** (currently at `match/+page.svelte:977`, moving it here so every consumer benefits automatically).
- `endSearch(atPly)` — sets `thinking=false`, `finishedAtPly=atPly`.
- `setHeuristic(...)` / `setHeuristicBySquare(...)` — writers used by the polling `$effect`.
- `resetAiSearch()` — for route teardown or new match.

Changes to `match/+page.svelte`:
- Remove `aiLastDepth`, `aiLastScore`, `aiSearchStartedAt`, `aiThinking`, `aiFinishedAtPly`, `heuristicEvalBreakdown`, `heuristicEvalBySquare`, `prevRoundBreakdown`, `lastRoundSeen`, `lastDepthUpdateMs` (lines 79-110). Keep `plyCount`.
- Depth callback at 974-980 becomes a single `aiSearch.updateDepth(d, s)` call.
- Heuristic-eval polling at 927-940 writes to `aiSearch.setHeuristic(...)` / `setHeuristicBySquare(...)`.
- Panel props at 1669-1681, 1751-1763, 1939 no longer pass these — panels read from the store directly.

Changes to `PlayerPanel.svelte`:
- Strip `aiLastDepth`, `aiLastScore`, `aiThinking`, `aiSearchStartedAt`, `aiFinishedAtPly` from `Props` and template. Read from `aiSearch.*`. Compute `p1Thinking` / `p2Thinking` inside the panel as `aiSearch.thinking && position?.toMove === (player === "p1" ? 0 : 1)`.

Changes to `EvalBreakdownPanel.svelte`:
- Strip `breakdown` and `prevBreakdown` props (57, 63). Read from `aiSearch.heuristicEvalBreakdown` and `.prevRoundBreakdown`. Prop surface narrows to just `player`.

Board and EffectsLayer already don't consume these props (verified at match:1684-1734, 1736). **No change needed** — this is what decouples them from search-tick reactivity.

Risk: medium. Prop surface reduction touches multiple consumers. Before merging: grep `aiLastDepth\|heuristicEvalBreakdown` across the tree to catch any third reader.

---

### PR 3 — Gating + cancellation + cleanup sweep

#### Phase 5 — Gate `heuristicEval()` polling during AI search

File: `match/+page.svelte:918-941`.

The `$effect` at 918 re-fires on every `match.position` change. During AI iterative-deepening `match.position` doesn't change mid-search (engine applies atomically at the end), so this is already low-frequency between plies — but rapid AIvAI with short think time can fire it back-to-back.

- Add guard `if (aiSearch.thinking) return;` at line 923 alongside existing early returns.
- `afterApplied()` runs after `renderApplied()` on the AI ply → `match.position` update naturally follows the search end and re-triggers the effect once when `thinking` flips back to false. This is the intended behaviour.

Risk: low.

#### Phase 6 — AI cancellation on route exit (cooperative)

Files: `match/+page.svelte:1574` (onDestroy), `1030` (finally block of `runAiStep`). `lib/engine/ai-hooks.ts` already supports a cancellation predicate.

- Add `let aiCancelRequested = false;` at module top.
- In `onDestroy`: `aiCancelRequested = true;`.
- In `runAiStep` at 974: `runAiCall(() => eng!.stepAi(...), { cancelled: () => aiCancelRequested })`.

Effect: search still runs to completion in the background, but result is discarded — no `renderApplied`, no `recordPly`, no state writes into a torn-down renderer. Matches the inspector pattern.

**Rust-side hard cancellation** (interrupt the search mid-loop) requires exposing an engine cancellation API through Tauri. Out of scope for this pass — flagged as a follow-up.

Risk: low. `ai-hooks.test.ts` already covers the cancellation path.

#### Phase 7 — Cleanup audit sweep

Two-part: (a) fix concrete gaps, (b) document the inventory so future work can spot new leaks.

##### Concrete gaps to fix

1. **Match toast timer** at `match/+page.svelte:1366`: no `clearTimeout` in `onDestroy`. Add `if (toastTimer) clearTimeout(toastTimer);`.
2. **Multiplayer inbox clearing** — `lib/multiplayer.svelte.ts:107-138` maintains module-level `inbox` and `rawInbox`. Audit the disconnect path; add `.clear()` calls if aborted lobby joins can leak buffered messages across sessions.
3. **WebSocket close path** — `lib/multiplayer/websocket-transport.ts:191`. Trace every `disconnect()` code path; confirm each calls `socket.close()`.
4. **Draft `pagehide` handler** at `routes/draft/+page.svelte:807, 852`: verify `onDestroy` at :859 removes the listener.

##### Long-lived subscription inventory (documented, no immediate fix needed)

Kept as reference for future audits. Format: subscription — file:line — has cleanup?

| # | Subscription | Location | Cleanup? |
|---|---|---|---|
| 1 | `visibilitychange` | `routes/+layout.svelte:22` | onMount return :23 ✓ |
| 2 | `beforeunload` | `routes/match/+page.svelte:1570` | onDestroy :1578 ✓ |
| 3 | `pagehide` | `routes/draft/+page.svelte:807, 852` | onDestroy :859 — **audit** |
| 4 | EffectsLayer RAF | `lib/board/EffectsLayer.svelte:144, 153` | onMount return :312-316 ✓ |
| 5 | PlayerPanel progress-bar RAF | `lib/match/PlayerPanel.svelte:101, 103` | $effect cleanup :104-106 ✓ |
| 6 | Ply-renderer scheduler timers | `lib/board/ply-renderer.svelte.ts:403` | `dispose()` ✓ — **Phase 3 fixes call site** |
| 7 | Pending skill-refresh setTimeout | `ply-renderer.svelte.ts:487-492` | via `dispose()` — same as #6 |
| 8 | Match toast timer | `match/+page.svelte:1366` | **NO — Phase 7 fix** |
| 9 | Heartbeat ping/now timers | `lib/multiplayer/heartbeat.ts:41, 61` | via `stopPings`/`stopTicking` from tearDown ✓ |
| 10 | Lobby liveness refresh | `routes/multiplayer/+page.svelte:415` | onDestroy :443 ✓ |
| 11 | Lobby mp raw/connected subs | `routes/multiplayer/+page.svelte:417-418` | onDestroy :434-441 ✓ |
| 12 | Match mp connected/disconnected unsubs | `match/+page.svelte:744-746` | onDestroy :1609-1612 ✓ |
| 13 | GraceBanner countdown interval | `lib/multiplayer/GraceBanner.svelte:42` | onDestroy :43 ✓ |
| 14 | MultiplayerStatusStrip interval | `lib/multiplayer/MultiplayerStatusStrip.svelte:30` | onDestroy :31 ✓ |
| 15 | mp module-level Sets/Maps + inbox | `lib/multiplayer.svelte.ts:107-138` | unsubscribers ✓, inbox clear — **audit (Phase 7)** |
| 16 | WebSocket socket | `lib/multiplayer/websocket-transport.ts:191` | close on disconnect — **audit (Phase 7)** |
| 17 | AudioContext | `lib/audio/sfx.ts:38-54` | app-lifetime singleton (intentional) |
| 18 | Renderer's checkpoints Map, timers Set | `ply-renderer.svelte.ts:403, 411` | via `dispose()` — same as #6 |
| 19 | Sandbox undo stack | `match/+page.svelte:131` | released on route exit ✓ |

##### Route transitions

Existing route-ownership-token pattern (`route-lifecycle.ts`, `multiplayer.svelte.ts:80`) defends against late-onDestroy vs early-onMount races for multiplayer teardown. Extend the same discipline to renderer disposal (Phase 3) — no ownership token needed since the renderer is owned by the route instance, not module-global.

---

## Approvals resolved 2026-07-08 (before starting)

1. **New store module vs extend `match-store.svelte.ts`?** — **New module** (`match-store` is route-persistent; AI-search state is per-run transient and semantically distinct). Confirmed.
2. **Rust-side AI cancellation** — **defer.** Phase 6 is cooperative-only (search finishes, result discarded). Hard cancellation via Tauri exposed engine API is a follow-up.
3. **`AudioContext.close()` on very long idle** — **no.** Just `resume()` + node disconnect. Closing needs a re-create dance and burns master gain / user volume routing.

## Verification approach

### Frontend re-render counts (Phase 4, 5)
- Svelte devtools: watch `<Board>`, `<EffectsLayer>`, `<PlayerPanel>` render counters during a 3-second AIvAI search with `showAiDepth` on.
  - Board / EffectsLayer: should re-render only on `match.position` change or `effectQueue` push. Not per depth tick.
  - PlayerPanel: re-renders per throttled depth tick (~10/sec).
  - Baseline (pre-fix): full cascade every 100 ms.
- Chrome Perf: record 10 s of AIvAI. Frame budget stable under 4 ms/frame outside ply-boundary spikes.

### Audio (Phase 1, 2)
- Manual: launch match, trigger a SFX, background window 15 min (reported failure window on macOS WKWebView), foreground, immediately click a piece. Should play with no user-visible latency.
- Chrome DevTools > Memory: heap snapshot before + after 500 SFX plays. `GainNode` / `OscillatorNode` retained-object counts should not grow monotonically — may plateau near currently-scheduled tail (~5–10 nodes).

### Cleanup (Phase 3, 6, 7)
- AIvAI, `p1ThinkTimeMs=500`, run 30 min. Heap snapshots at t=0/15/30. `PlyRenderer`, `Effect`, `Map`/`Set` retained-object counts flat between 15 and 30 (allowing noise from live state).
- Rapid navigation: match → replay → inspector → match ×20. `PlyRenderer` retained count exactly 1 (current route), not 20.
- Multiplayer mid-search Back button: engine's depth callbacks stop within seconds (natural search completion), no `[match]` telemetry warnings from torn-down route.

## PR sequencing

1. **PR 1 (Phases 1–3):** audio robustness + renderer leak fix. Low-risk, ships first.
2. **PR 2 (Phase 4):** AI-transient store extraction. Larger surface — merge once PR 1 is stable.
3. **PR 3 (Phases 5–7):** gating + cancellation + cleanup sweep. Small individual changes; can be one PR or split further if any phase surprises.

## Files touched (summary)

Heavy edits (~5 files):
- `game/frontend/src/routes/match/+page.svelte`
- `game/frontend/src/lib/audio/sfx.ts`
- `game/frontend/src/lib/match/PlayerPanel.svelte`
- `game/frontend/src/lib/eval/EvalBreakdownPanel.svelte`
- **New:** `game/frontend/src/lib/state/ai-search.svelte.ts`

Small edits:
- `game/frontend/src/routes/replay/+page.svelte` (add onDestroy)
- `game/frontend/src/routes/+layout.svelte` (statechange sanity check)
- `game/frontend/src/lib/board/ply-renderer.svelte.ts` — no change; existing `dispose()` is called from routes
- `game/frontend/src/lib/multiplayer.svelte.ts` (inbox clear on disconnect, pending audit)
- `game/frontend/src/lib/multiplayer/websocket-transport.ts` (socket close audit)
- `game/frontend/src/routes/draft/+page.svelte` (pagehide handler removal audit)

## Follow-ups (not in this plan)

- **Rust-side AI cancellation.** Expose a cancellation flag through the Tauri command surface so the search can be interrupted mid-loop rather than running to completion after route exit. Would require a shared `AtomicBool` in `tauri_wrapper` and per-node checks in `search/alpha_beta.rs`.
- **Long-idle AudioContext close.** If profiling reveals that keeping the context alive for hours is itself a resource cost, add an idle-timer that fully closes and recreates on next SFX. Currently rejected — nice-to-have.
- **Svelte-inspector-based automated re-render regression test.** Would require a headless setup that counts component renders per interaction. Manual check via devtools sufficient for now.

## Notes for the implementer

- Always commit before starting a phase. That commit is the rollback anchor.
- After each phase, verify: (a) tests still pass (`cd game/frontend && npm test`), (b) the specific behaviour improves visibly. Don't batch phases just because they're small.
- Update this file's status ("Phase X — DONE") as phases land. Don't rewrite — append notes so the timeline is preserved.
