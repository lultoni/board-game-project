# UI cleanup + perf plan

*Written 2026-07-08. Anchor commit: `ec0cae4` (main). Rollback target if a phase misbehaves.*

## Status

- **PR 1 (Phases 1–3) — DONE 2026-07-08** (commit `1cc0010`). Audio pre-play resume + statechange listener + AudioNode disconnect on `ended`; renderer `dispose()` wired into `/match` and `/replay` onDestroy.
- **PR 2 (Phase 4) — DONE 2026-07-08** (commit `a794fa9`). New `lib/state/ai-search.svelte.ts` rune-store; AI-transient state (thinking / depth / score / searchStartedAt / finishedAtPly + heuristic-eval fields) extracted from `/match`; PlayerPanel and EvalBreakdownPanel now read from store instead of props; `/replay` rewired to same store for eval panel.
- **PR 3 phase 5 — DONE 2026-07-08** (commit `4ad0cdc`). `aiSearch.thinking` guard added to heuristic-eval `$effect`; poll no longer fires during search. Trailing poll on `thinking→false` transition is intended behaviour.
- **PR 3 (Phases 6–8) — pending.** Phase 8 diagnostic in progress.

svelte-check clean across all landed PRs. Test suite: 1 pre-existing failure inherited from anchor (`settings.svelte.test.ts / rejects zero max-depth`), unrelated.

---

## Open symptom (2026-07-08, post-PR-2)

**Fast-AIvAI ply-renderer stall.** During AIvAI with short think times (moves land faster than the piece animation can finish), the ply renderer visibly freezes mid-animation until the next move arrives. Not a search-thread problem — the pause happens *while* an animation is supposed to be running.

### Hypothesis

The heuristic-eval `$effect` in `/match` (currently `match/+page.svelte:905-926`) reacts to `match.position` changes. `match.position` mutates inside `renderApplied()` *before* the piece animation completes (the ply-renderer writes the post-state to the positionSink to drive `<Board>`'s SVG animate elements). So during move N's animation:

1. `renderApplied` writes new `match.position` → `<Board>` starts SVG `<animate>` slides
2. `$effect` sees `match.position` change → fires `heuristicEval()` IPC + `heuristicEvalBySquare()` IPC
3. Responses arrive on main thread → `setHeuristic()` / `setHeuristicBySquare()` writes into the store
4. `EvalBreakdownPanel`'s `rowVsPrev` $derived reruns (9 rows × several fields each)
5. On short think times, move N+1's search *result* also arrives during move N's animation window (search runs in parallel with animation, gated only by `await animationDone()` before `endSearch` / `busy=false`). Result: heavy main-thread work bunched into the animation window.

SVG `<animate>` runs on the compositor, but the JS main thread must not stall long enough to miss the frame that *writes* the new `pieceMotion` map for the next move. When a heuristic-eval response, a store write, and a downstream $derived rerun all land in the same frame, that frame can slip and the animation visually pauses.

Note that the depth-tick throttle (100 ms) and the store isolation from PR 2 already keep `<Board>` and `<EffectsLayer>` out of the search-tick re-render cascade — those helped the "AI thinking slowness" case but not the fast-AIvAI stall.

### Diagnostic before fixing

Before implementing Phase 8, add temporary `console.time` markers around:
- `renderApplied()` and `animationDone()` in `runAiStep`
- The `heuristicEval()` IPC + response handler
- `EvalBreakdownPanel`'s `rows` $derived

Record a 10 s AIvAI stall with Chrome Perf (frontend running in the browser build; Tauri devtools are limited). Confirm the frame drop lines up with the heuristic response, not the search return. Only then commit to the fix — if the profile disagrees, revisit the hypothesis.

---

## PR 3 — Gating + cancellation + cleanup sweep + fast-AIvAI fix

### Phase 5 — Gate `heuristicEval()` polling during AI search

File: `match/+page.svelte` (the `$effect` at ~905 that polls `heuristicEval` on `match.position` change).

- Add guard `if (aiSearch.thinking) return;` alongside existing early returns.
- `afterApplied()` runs after `renderApplied()` → `match.position` update naturally re-triggers the effect once when `thinking` flips back to false. That single trailing poll is the intended behaviour.

Risk: low. Complements Phase 8 — this stops the poll from firing *while the AI is thinking*, Phase 8 stops it from firing *during animation*.

### Phase 6 — AI cancellation on route exit (cooperative)

Files: `match/+page.svelte` onDestroy + `runAiStep` finally block. `lib/engine/ai-hooks.ts` already supports a cancellation predicate.

- Add `let aiCancelRequested = false;` at module top.
- In `onDestroy`: `aiCancelRequested = true;`.
- Pass `{ cancelled: () => aiCancelRequested }` into `runAiCall(() => eng!.stepAi(...))`.

Effect: search still runs to completion in the background, but result is discarded — no `renderApplied`, no `recordPly`, no state writes into a torn-down renderer.

**Rust-side hard cancellation** (interrupt the search mid-loop) is a follow-up — requires exposing an engine cancellation API through Tauri.

Risk: low. `ai-hooks.test.ts` already covers the cancellation path.

### Phase 7 — Cleanup audit sweep

Concrete gaps to fix:

1. **Match toast timer** in `match/+page.svelte`: no `clearTimeout` in `onDestroy`. Add `if (toastTimer) clearTimeout(toastTimer);`.
2. **Multiplayer inbox clearing** — `lib/multiplayer.svelte.ts` maintains module-level `inbox` and `rawInbox`. Audit the disconnect path; add `.clear()` calls if aborted lobby joins can leak buffered messages across sessions.
3. **WebSocket close path** — `lib/multiplayer/websocket-transport.ts`. Trace every `disconnect()` code path; confirm each calls `socket.close()`.
4. **Draft `pagehide` handler** in `routes/draft/+page.svelte`: verify `onDestroy` removes the listener.

Long-lived subscription inventory (kept as reference for future audits, no immediate work):

| # | Subscription | Location | Cleanup? |
|---|---|---|---|
| 1 | `visibilitychange` | `routes/+layout.svelte` | onMount return ✓ |
| 2 | `beforeunload` | `routes/match/+page.svelte` | onDestroy ✓ |
| 3 | `pagehide` | `routes/draft/+page.svelte` | **audit** |
| 4 | EffectsLayer RAF | `lib/board/EffectsLayer.svelte` | onMount return ✓ |
| 5 | PlayerPanel progress-bar RAF | `lib/match/PlayerPanel.svelte` | $effect cleanup ✓ |
| 6 | Ply-renderer scheduler timers | `lib/board/ply-renderer.svelte.ts` | `dispose()` ✓ (PR 1 wired call site) |
| 7 | Pending skill-refresh setTimeout | `ply-renderer.svelte.ts` | via `dispose()` ✓ |
| 8 | Match toast timer | `match/+page.svelte` | **NO — Phase 7 fix** |
| 9 | Heartbeat ping/now timers | `lib/multiplayer/heartbeat.ts` | via `stopPings`/`stopTicking` ✓ |
| 10 | Lobby liveness refresh | `routes/multiplayer/+page.svelte` | onDestroy ✓ |
| 11 | Lobby mp raw/connected subs | `routes/multiplayer/+page.svelte` | onDestroy ✓ |
| 12 | Match mp connected/disconnected unsubs | `match/+page.svelte` | onDestroy ✓ |
| 13 | GraceBanner countdown interval | `lib/multiplayer/GraceBanner.svelte` | onDestroy ✓ |
| 14 | MultiplayerStatusStrip interval | `lib/multiplayer/MultiplayerStatusStrip.svelte` | onDestroy ✓ |
| 15 | mp module-level Sets/Maps + inbox | `lib/multiplayer.svelte.ts` | **audit (Phase 7)** |
| 16 | WebSocket socket | `lib/multiplayer/websocket-transport.ts` | **audit (Phase 7)** |
| 17 | AudioContext | `lib/audio/sfx.ts` | app-lifetime singleton (intentional) |
| 18 | Renderer's checkpoints Map, timers Set | `ply-renderer.svelte.ts` | via `dispose()` ✓ |
| 19 | Sandbox undo stack | `match/+page.svelte` | released on route exit ✓ |

### Phase 8 — Fast-AIvAI ply-renderer stall

**Only after diagnostic confirms the hypothesis above.**

Two candidate fixes; pick one after the profile identifies the biggest offender:

**Fix A — defer heuristic poll to after animation.** Remove the `$effect` reactive dependency on `match.position` for the eval poll. Instead, call an explicit `void pollHeuristic()` at the *end* of `afterApplied()` (after `plyCount += 1`). Keep the `$effect` only to watch `settings.showEvalPanel` / `settings.showHeuristicEval` for setting-flip cases. This decouples the IPC from the render frame that starts the piece animation, so the animation gets a clean frame budget.

**Fix B — coalesce heuristic polls with a microtask.** Wrap `setHeuristic` / `setHeuristicBySquare` in a `requestIdleCallback` (fallback: `setTimeout(0)`). Cheaper to implement than Fix A but adds a small perceptual lag on the eval panel. Only choose this if Fix A ends up entangled with other consumers of the `match.position` reactivity.

**Combined with Phase 5**, the poll will:
- Skip entirely while `aiSearch.thinking` (Phase 5),
- Fire only at ply boundaries via explicit call from `afterApplied()` (Fix A),
- Never race with the animation start frame.

Risk: medium. Fix A moves the poll trigger out of the reactive graph, which changes when it fires on setting toggles (need to also poll on toggle flip; add a second small `$effect` that watches settings only). Fix B is lower-risk but doesn't eliminate the underlying re-render churn — it just shifts timing.

Verification (Phase 8): AIvAI with think time set below animation duration (e.g. `p1ThinkTimeMs=100`, `p2ThinkTimeMs=100`, `respectAnimation=on`). Piece animations should complete smoothly without visible mid-slide pauses. Chrome Perf: no long tasks >16 ms during animation windows.

---

## Approvals resolved 2026-07-08 (before starting)

1. **New store module vs extend `match-store.svelte.ts`?** — New module (`match-store` is route-persistent; AI-search state is per-run transient and semantically distinct).
2. **Rust-side AI cancellation** — defer. Phase 6 is cooperative-only; hard cancellation via Tauri is a follow-up.
3. **`AudioContext.close()` on very long idle** — no. Just `resume()` + node disconnect. Closing needs a re-create dance and burns master gain / user volume routing.

## Verification approach (for PR 3)

### Frontend re-render counts (Phase 5)
- Svelte devtools: watch `<Board>`, `<EffectsLayer>`, `<PlayerPanel>` render counters during a 3-second AIvAI search with `showAiDepth` on.
  - Board / EffectsLayer: no re-renders per depth tick (already achieved by PR 2).
  - PlayerPanel: re-renders per throttled depth tick (~10/sec) only.
  - EvalBreakdownPanel: no re-renders during search (Phase 5 gate).

### Fast-AIvAI stall (Phase 8)
- Manual: AIvAI with `p1ThinkTimeMs=100` and `respectAnimation=on`. Piece animations complete smoothly, no visible mid-slide pauses.
- Chrome Perf: 10 s recording. No long tasks >16 ms landing inside animation windows. `heuristicEval`-triggered work sits between plies, not during them.

### Cleanup (Phase 6, 7)
- AIvAI, `p1ThinkTimeMs=500`, 30 min. Heap snapshots at t=0/15/30. `PlyRenderer`, `Effect`, `Map`/`Set` retained-object counts flat between 15 and 30.
- Rapid navigation: match → replay → inspector → match ×20. `PlyRenderer` retained count exactly 1 (current route).
- Multiplayer mid-search Back button: no `[match]` telemetry warnings from torn-down route.

## Follow-ups (not in this plan)

- **Rust-side AI cancellation.** Expose a cancellation flag through Tauri so the search can be interrupted mid-loop. Requires a shared `AtomicBool` in `tauri_wrapper` and per-node checks in `search/alpha_beta.rs`.
- **Long-idle AudioContext close.** If profiling reveals context lifetime cost, add an idle-timer that closes and recreates on next SFX. Currently rejected — nice-to-have.
- **Svelte-inspector automated re-render regression test.** Would require a headless setup that counts renders per interaction. Manual check via devtools sufficient for now.

## Notes for the implementer

- Always commit before starting a phase. That commit is the rollback anchor.
- After each phase: (a) `npm test` still passes, (b) the specific behaviour improves visibly. Don't batch phases just because they're small.
- Phase 8 requires a diagnostic profile first — do not implement blind.
- Update this file's status as phases land. Append notes rather than rewriting history.
