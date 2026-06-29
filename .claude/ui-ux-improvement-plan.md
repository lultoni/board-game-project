# UI/UX Improvement Plan

*Iteratively updated. Each section reflects the state of discussion at that point.*
*Last updated: 2026-06-29 — Batch 3 answered, ready to implement.*

---

## What this plan is

A structured improvement plan for the game's frontend. Built by auditing current state and resolving design questions before implementing. Phase A = game screen. Phase C = feedback states. Phase B = non-game screens.

---

## All Q&A

### Batch 1 (2026-06-29)
**Q1 — Priority:** A (game screen) → C (feedback) → B (non-game screens).
**Q2 — Illegal click:** Shake the clicked square. No tinting. Visual language = expressive when active, simple otherwise.
**Q3 — AI thinking:** Per-player placement. Keep spinner, add depth counter (max 100ms update rate).
**Q4 — Breathing:** Restore for active player's pieces. Fade in/out. Show turn ownership via board visuals.
**Q5 — Menu:** More stylised — "not packing vibes". Exact direction TBD.
**Q6 — Animations:** Slide everywhere (live, replay, inspector). Drag excluded (add drag_start sound). Settings must prevent interval cutting off animations. Settings menu needed.
**Q7 — Training Observatory:** Out of scope. Top bar clips on small windows — note for later.

### Batch 2 (2026-06-29)
**Q8 — Guards bug:** Every first load, any mode. Only Move fixes it — Skill/EndPhase do not. Confirms pieceIds race.
**Q9 — Scaling:** Both vertical and horizontal overflow at 1024×768. Full-screen fine.
**Q10 — Turn indicator:** Coloured accent strip on active player's board edge. Must be clearly visible.
**Q11 — Settings:** Modal/overlay from gear icon anywhere. Contextual settings at top (route-detected), global below.
**Q12 — What animates:** Everything — captures, shove, bodyguard intercept, skill-caused moves. Click-to-move attacks also snap currently (bug). Approach chooser missing for click-to-move.

### Batch 3 (2026-06-29)
**Q13 — A0 fix approach:** Write tests first, then fix; confirm in Tauri.
**Q14 — Scaling approach:** Use full screen width. HUD stretches across full window width below board (not just 720px). Board fills available height clamped by aspect ratio.
**Q15 — P1/P2 colours:** Use same as piece fill for now. Refactor to CSS vars later.
**Q16 — AI depth indicator placement:** Above/below board depending on player. Use space beside board for player info (captured pieces, etc.) — inspired by lichess/chess.com layout.
**Q17 — Settings scope:** Add AI settings and colour settings. Run explore agents per mode/feature to find all candidate settings (done — see settings inventory below).

---

## Settings inventory (from audit)

### Existing (in `settings.svelte.ts`, persisted to localStorage `game-settings`)
- `audioVolume` (0–1)
- `locale` ("en" | "de")
- `p1ThinkTimeMs`, `p2ThinkTimeMs`
- `p1MaxDepth`, `p2MaxDepth`
- `aivaiStepDelayMs` (also drives replay speed — should split)
- `p1Evaluator`, `p2Evaluator`
- `showLegalTargets`, `showProjectilePath`, `showIllegalOwner`, `showBlockedByFriendly`

### Hardcoded values to promote to settings
- Replay step delay — currently shares `aivaiStepDelayMs` — needs its own key
- Piece animation speed multiplier (currently baked into 280ms constant)
- Toast duration (2200ms in match page)
- Training poll interval (1000ms in training page)
- Training backend choice (currently separate localStorage key, not in settings store)

### New settings for day-one modal (prioritised)
**Visual:**
- Animation speed: off / normal / fast (controls piece slide duration multiplier)
- Board colour scheme (light/dark — future, but add the slot)

**Sound:**
- Master sound on/off (already exists as `audioVolume`, but needs an on/off toggle)

**Match:**
- AI move indicator style (spinner only / spinner + depth)

**Replay:**
- Step delay (independent of `aivaiStepDelayMs`)
- Loop on end
- Respect animation timing (bool — if true, waits for slide to finish before next step)

**AI:**
- Think time per seat (already exists in setup but not in settings modal)
- Max depth per seat (same)

**Accessibility/Visual aids:**
- Show legal targets (already exists)
- Show projectile path (already exists)

---

## Planned changes

### Phase A — Game screen

#### A0 — Bug: P1 King/Champion render as Guards on initial load (Tauri only)
- **Status: FIXED** (commit a6b11bc)
- **Root cause (confirmed):** `u64` bitboard values for champions (~7.9×10¹⁸) and kings (~1.15×10¹⁸) both exceed JavaScript's f64 safe integer limit (2⁵³ ≈ 9×10¹⁵). Serde serialised them as JSON numbers; `JSON.parse` silently rounded them, making `bitboardHas` return false for every square. All pieces fell through to the "guard" default branch in `readPieces`.
- **Fix:** Changed `PositionViewDto.bitboards` from `[u64; 5]` to `[String; 5]` and `zobrist` from `u64` to `String` in `game/crates/tauri_wrapper/src/lib.rs`. All construction sites updated to call `.to_string()`. The WASM client was unaffected (passes bitboards through `BigUint64Array` which preserves full u64 precision).
- **Tests added:** Two regression tests in `ply-renderer.test.ts` — `pieceIds` populated immediately after `resyncFromEngine()`, and stable ids preserved for squares that remain occupied across two calls. 229 frontend tests pass.

#### A1 — Full-width layout: board + player panels side-by-side
- **Layout redesign (lichess-inspired):** Instead of board centred with HUD below, use a side-by-side layout:
  - Left column: Player 2 info panel (top) + Board + Player 1 info panel (bottom)
  - Right column: Controls + HUD stats + End Phase + export
- Board width = `min(100vw - RIGHT_PANEL_WIDTH, 100vh - P_PANEL_HEIGHT * 2 - HEADER_HEIGHT)` — fills height first, respects width second.
- Player info panels (above/below board): player name/colour indicator, captured pieces (chess.com style), AI thinking indicator when it's their turn, money display.
- Right panel: phase, round, actions remaining, end phase button, focus mode toggles, export buttons.
- On narrow screens (<820px): right panel drops below, player panels collapse into rows above/below board.
- Board SVG already has `width: 100%; height: auto` — just needs the container to be correctly sized.

#### A2 — Piece animations everywhere
- Replay step-forward already works (goes through `applyAndRender`). Snap issue is on scrub: `fastForwardTo` → `resyncFromEngine` clears `pieceIds` → DOM remount = no slide. Fix: same differential reconcile from A0 fix propagates here.
- Inspector: same path, same fix.
- Skills that move pieces: `renderApplied` traces move events in `emitMoveEvents` — verify this runs for skill-caused displacements. If a skill relocates a piece but the engine doesn't produce a Move action (it does a deferred refresh instead), the pieceId update happens in the deferred timer's apply function — check it calls `reconcilePieceIds()` after position update.

#### A3 — Illegal move: shake on clicked square
- Add `shakingTargetSquares: Set<number>` to Board props.
- When match page detects an illegal click (piece selected, but clicked square is not in `moveTargets` and not another selectable piece), add that square to a `shakingTargets` set for 300ms.
- In Board SVG, render a CSS `@keyframes` shake on the square's `<rect>` when it's in `shakingTargetSquares`.
- No shake for clicking empty space with no selection (not an "illegal action", just a deselect).

#### A4 — Whose-turn board strip + breathing fade
- **Breathing:** Replace `dormant` binary with `--breathe-amplitude` CSS var: 1.0 for active player's pieces, 0.3 for inactive. Transition the var change over 400ms. Keeps board alive, communicates turn without killing it.
- **Turn strip:** Render a `<rect>` in the Board SVG: full width, 5px tall, at `y = viewBox` (bottom edge, P1's side) or `y = -WHEEL_PAD` (top edge, P2's side) depending on `position.toMove`. Use piece fill colour for P1 (blue) and P2 (red). Fade between positions with a 300ms opacity/transform transition. The strip lives inside the board SVG so it always scales with the board.

#### A5 — Per-player AI thinking indicator + depth counter
- Remove the single `.thinking` overlay from the board-stack.
- Place thinking indicators in the player info panels (from A1): inline next to the player's name row.
- Add `thinkingDepth: number` state (0 while idle). When `aiThinking` becomes true, start a `setInterval(100ms)` that increments a displayed depth counter. On `aiThinking` becoming false, snap to the real final depth from `StepResult`, then clear after 1.5s.
- Indicator: spinner glyph + "depth N" label, styled per player colour.

#### A6 — Click-to-move: approach chooser + animation
- Approach chooser: `tryCommitMoveTo` already shows `pendingApproach` when candidates > 1. Verify this path is reached for all click-to-move targets (it is — `handleSquareClick` calls `tryCommitMoveTo`). The issue may be that click-to-move with exactly 1 candidate silently picks it without animating. `commitMoveTargetApproach` → `applyRaw` → `renderer.applyAndRender` — the animation *should* fire. If it doesn't, add logging to confirm `applyAndRender` is called.
- For the snap bug: drag sets `press.dragging = true` which makes `overrideForPiece` return the cursor position (suppresses CSS transition). On drop, `press` is cleared. The position update from `applyAndRender` happens *after* the drop handler returns — by which point `overrideForPiece` returns null and the transition is restored. So the piece should slide. If it snaps, the issue is that Svelte flushes the position update before the `press` clear in the same tick. Fix: ensure `press = null` is set before `await applyRaw(...)`.

#### A7 — Settings modal
- New `Settings.svelte` component: full-screen overlay (not a page route), triggered by a gear `⚙` icon in the site header.
- Structure:
  - **Contextual section** (top): reads `$page.url.pathname`, renders relevant settings only. Replay page → step delay + loop + respect-animation-timing. Match page → sound toggle + animation speed. Inspector → (future). Default → nothing.
  - **Global section**: animation speed (off/normal/fast), master sound toggle, show legal targets, show projectile path.
  - **AI section**: P1/P2 think time + max depth (mirrors setup page sliders, but accessible mid-game).
- Settings store: extend `settings.svelte.ts` with new keys (`replayStepDelayMs`, `replayLoopOnEnd`, `replayRespectAnimation`, `animationSpeed`, `pieceColourP1`, `pieceColourP2`). Migrate training backend key into store. Validate new keys in `_validateSettings`.
- Gear icon: in `+layout.svelte` header, right-aligned.

---

## Phase C — Feedback states (after Phase A)

*(Scoped, not yet detailed — will add a batch of questions when Phase A is underway.)*

- Loading states on evaluator list fetch (setup page)
- Loading state on library mount
- Success toasts on export operations
- Confirmation on match delete and game abandon
- "Waiting for opponent" indicator in multiplayer match (between your moves)

---

## Phase B — Non-game screens (after Phase C)

*(TBD — menu styling direction still open.)*

---

## Implementation order within Phase A

1. **A0** — Guards bug (test + fix, self-verifiable) ✓
2. **A7** — Settings modal skeleton ✓
3. **A1** — Full-width layout (biggest structural change; do before visual polish)
4. **A3** — Illegal click shake (small, independent)
5. **A4** — Turn strip + breathing fade (visual, depends on A1 layout)
6. **A5** — Per-player AI indicator (depends on A1 player panels)
7. **A2** — Animation everywhere (depends on A0 fix propagating to replay/inspector)
8. **A6** — Click-to-move approach + animation fix

---

## Known broken / not yet done

### BUG: Move animations are instant (all cases)
- **Status: FIXED** (session 2026-06-29)
- **Root cause:** `pieceIds` and `position` were both written in the same Svelte reactive flush. The CSS transition on `transform` needs a painted "before" state — but both writes batched together meant the DOM element was always created/updated at the new coordinates, giving the browser no opportunity to interpolate. Attack animations (lunge) also failed because the lunge timer was set after the position was already updated.
- **Fix:** In `renderApplied` (ply-renderer.svelte.ts), for Move actions: wait one `requestAnimationFrame` (skipped when `animationSpeed = "off"`) before fetching fresh engine state. Then upfront-detect kills from the fresh position, and update `pieceIds` + `setPosition` synchronously in a single block — both land in one Svelte flush. The browser sees the DOM element at its old transform for one frame, then the new transform triggers the CSS transition. Test environment uses `setTimeout(0)` fallback since `requestAnimationFrame` is unavailable in Node/jsdom. All 229 tests pass.

### BUG: Lunge animation not playing on non-kill attacks
- The lunge keyframes (`piece-lunge-1`, `piece-lunge-2`) and timing delay are wired up but the animation does not visually fire. The slide transition works; the lunge after it does not.
- Infrastructure in place: `triggerLunge` delays by `slideDurationMs()`, fires inner `scheduleTimer` to add to `lungeSquares`, `dist` passed through to `Piece.svelte`, two keyframes defined in CSS. Something in the chain is still broken — needs a targeted debug session (check that `lungeSquares` actually gets populated in the browser, that the CSS `animation` property is being applied to `.lunge-wrap`, and that `forwards` fill-mode isn't interfering).

### BUG: Sandbox discard confirmation dialog renders below content
- The `<dialog open>` used for the sandbox exit confirmation appears at the bottom of the page rather than centred/overlaying the board. Needs `showModal()` pattern (like `PoiLabelDialog`) or CSS `position: fixed` centering, plus a backdrop. Low priority — it works, just looks bad.
- Replay mode shows the draft phase in a raw/ugly way. Needs a dedicated view or better layout for the drafting portion of a replay. Not yet designed — needs discussion before implementation.
