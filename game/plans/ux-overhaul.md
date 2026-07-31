# UX Overhaul — Testing Checklist

Status key: ✅ works · ❌ does not work · ❔ check if this exists

> **Deep-analysis pass (2026-07-31):** full root-cause map + fix plan appended at the
> bottom under "── Analysis & Fix Plan ──". The original checklist below is preserved as
> the intent record. Several items marked ❌ were found already-done; see analysis.

---

## Principles (apply everywhere)

1. **Symmetric-by-default** — one action mirrors to both sides; asymmetric is opt-in
2. **Surface, don't hide** — loadouts, options, key actions visible directly
3. **Low-click entry** — minimum steps to start a game
4. **Coexistent context** — rules, action log, tools live *beside* the board; never cover it
5. **No dead ends** — every screen connects to adjacent relevant ones
6. **Everything is reviewable and re-enterable** — every game is resumable, inspectable, forkable

---

## P1 — Critical gameplay correctness

### P1-A · Shove now does not allow to actually pick the direction ❌
- you broke it so that you cannot choose which direction you want to shove a piece (WITH SHOVE SKILL!) like it was previously possible
- you broke the code. it was working before
- make sure you did not fuck this up somewhere else. and i mean REALLY make sure

### P1-B · Resign + Draw actions ❌
- HvAI: AI uses `evaluate_draw_offer()` in Rust engine — accepts when its seat-relative score ≤ 50 centipawns (positive = P1 winning, score is negated for P2 AI)
- MP: draw offer sent to peer; resign terminates immediately

### P1-C · Local game resumability ❌
- Abandoned / in-progress games resumable from library does not work with aivai
- Hub resume banner for local games does not show up all the time
  - it would also need to be discuess how and when it should trigger and for which games and then also with which informations shown

---

## P2 — Board orientation + piece interaction

### P2-A · Board flip at match start ❌
- HvAI local: human seat always rendered at bottom — **not done**
- MP: your seat always at bottom — **not done**
- HvH: no flip — **should not flip**

### P2-B · Speed-1 drag: suppress intermediate squares ❌
- Intermediate step squares still shown for speed-1 pieces during drag — **not done**

### P2-C · Click-to-move: show approach squares on enemy hover ❌
- Hovering an enemy piece WITH YOUR MOUSE in click-to-move mode should show approach squares
  - i realtalk want the same effect of the approach squares showing like as if you were dragging a piece but instead of the piece dragging being tracked it tracks the mouse
  - it already picks the correct square for when you perform a click, now we just need the visual indicator

### P2-D · Generalise approach/direction to angle(origin→mouse) ❔
- `pickApproachByCursor` helper exists in `state/move-targets.ts`
- Used for both drag and click-to-move path resolution

### P2-E · On Load of a Position in Ply renderer it shows which pieces already moved in movement phase as greyed out ❌

### P2-F · Last-action highlight stays on end-phase ❌
- EndPhase no longer clears the highlight

---

## P3 — HUD / information display

### P3-A · Money display: expense/skill cost indicator indicator inline ❌
- Money preview for skill costs clears on EndPhase and after skill use

### P3-B · Phase indicator: action counts ❌
- Historical/future count shown with an actual correct number greyed when phase is inactive
  - should be calculated via engine api points if no direct one is available

### P3-D · Taken-pieces counter: init from custom positions ❌
- Detects actual piece count when launching from custom position
  - it should really count the amount of pieces per piece type to know how many boxes to show (if less than the normal number: still show normal amount of boxes, but fill the ones out that are more) (if more pieces than normal: create more boxes)

### P3-E · In-game action log panel ❌
- Allows moving back and forth between made moves just like the step buttons in the replay screen do
  - find a way to decouple the ply renderer in the match screen from the match state itself, so that you are able to have this moving around like in replay
  - maybe you can copy the homework from how replay does it? maybe there is a way to unify the architecture a bit so it works better?
- Clicking past move in action log to preview that position — not done
- it is read only for now and does not actually touch the game state so it can work independantloy on seat types or local/mp
- we need a "jump to present" button when we are not already there that appears at the top dynamically

---

## P4 — Skill wheel

### P4-B · Skill tooltip: 1s delay, beside piece ✅ FIXED
- its not next to the piece, but the delay works
- FIXED (session 6): info-anchor now positioned from `wheelOpen.square` beside the piece.

### P4-C · Focus/boost picker inside the wheel ✅ FIXED
- there is no actual way to pick the two different versions of how focus affects your skill
- every skill should be split in hald in the wheel if they have a different effect with focus
  - the first "split quarter" then on activation triggers "version 1" and the second quarter the second
  - so i really want instad of the two hald circles to then have 1 half cirecle and 2 quarters OR 4 quarters displayed (ofc dynmailcally done inside the skill wheel based on if focus is active and ofc also which skills are picked)
- FIXED (session 6): split driven at wheel-open from `focusActive` + per-slot `hasFocusModeChoice`;
  clicking a quarter arms the skill with that focus mode. Was previously dead code (wheel closed on arm).

---

## P5 — Panels and overlays

### P5-B · Sandbox: "Play My Moves" inside sandbox ✅ FIXED
- Inside sandbox, when moves have been staged, a **"▶ Play moves"** button appears
- Clicking it shows a confirm dialog → commits moves to the real game incrementally
- Bodyguard choice mid-playback: if a bodyguard decision (i mean bg accepted/declined) arises for a staged move that differs from what was planned, playback stops at that point (engine will return the chooser UI naturally still, but the playback stops as soon as the "wring choice" was done and the playback hence cannot be continued)

---

## P7 — Draft screen

### P7-A · Piece order matches board placement ❌
- Pieces in draft UI not yet ordered as they appear on the board

### P7-B · Unify draft and loadout editor visual language ❌
- Draft screen and loadout creator still differ in layout/style
- Premade loadouts still hidden behind dropdown

---

## P8 — Replay + Inspector

### P8-A · Inspector merged into replay as "Inspector Insights" panel ❌
- Toggleable "Inspector Insights" side panel in replay
- Heuristic eval breakdown shown when open
- Shallow AI search (depth 4) runs when panel is open, shows best move — **only fires when panel is open, does not block interaction**
- Fix: eval was gated on `settings.showEvalPanel`; now also fires when `showInsights` is open
- End-of-game result shown next to last ply (e.g. "End of game — P1 wins")
- **Missing:** POI marking not yet ported from old inspector
- **the issue currently is that the inspector panel only shows the eval breakdown and nothing else PLUS the eval breakdown itself is too wide for the small space shown there**

---

## P9 — Position Builder

### P9-A · New route `/position-builder` ❌
- Click a piece → edit panel (HP 1–2, Armor 0–2, Skill slots for Champions/Kings) does not work yet
- Drag a piece → repositions it (FEN round-trip) gives this error: snapshot restore error: BadFen("BadDecimal { field: \"moved_this_phase\" }")
- you do not see the current fen of the position constantly, you only see it when you click on copy and put it in the field
- the input field is small and does not allow for easy editing (redundant if you get the visual editing to work)

---

## P10 — Game history library

### P10-B · Tag games ❌

### P10-D · Hide / clean up incomplete games ❌
- "Hide incomplete" toggle in filters now exists, but it should remember which choice the user picked

---

## Back button standardisation ❌

All secondary screens (replay, library, position-builder, setup, draft, match) now use the unified `BackButton` component in the top-left Header row (like settings and help) across all screens.

---

## Bug backlog (found during testing)

### BUG-1 · Ply renderer: wrong damage numbers on strike-moves-caster skills ✅ FIXED

**Repro FEN:**
```
1c[2/0/0/1/10]1k[2/2/0/6/9]2c[2/2/0/6/10]1/gg1g1c[2/0/0/1/9]2/1c[2/2/0/1/6]3g2/8/GC[2/2/0/5/6]1g4/2GC[2/2/0/2/6]Gc[1/0/0/1/9]1C[2/2/0/5/6]/2G1K[2/1/0/14/15]G2/4C[2/2/0/3/6]1C[2/2/0/4/6]1 P1 S 3 8 4 4 12 0x0
```
**Action:** `h3*f3:Tempest`

**What happened:** A heal effect appeared on g3 (the caster's landing square after the post-strike step). The damage/kill effect on f3 (the target that was killed) was not shown.

**Root cause area:** `ply-renderer.svelte.ts` — `extractCasterMove` strips the caster's src→dest move from `diff.moves` and `diff.stayed`. However `emitImpactEvents` uses `diff.stays` and `diff.moves` to render damage numbers. When the killed piece ends up in `diff.deaths` (handled separately by `emitRelocationAndDeathEvents`), it gets a generic impact+damageNumber at its origin square — but if `extractCasterMove` incorrectly leaves the destination square in `diff.stayed`, `emitImpactEvents` reads the pre-mailbox of the caster's destination as a "piece that healed" (because the caster was not there pre-skill). The fix must not be skill-specific; `emitImpactEvents` should be correct for any skill that involves caster relocation after a strike.

**Fix direction:** In `extractCasterMove`, verify that `diff.stayed` entries whose square equals `casterMove.to` are also stripped (the caster's destination may linger in `stayed` if the diff pairing logic doesn't account for the caster arriving there). Alternatively, ensure `emitImpactEvents` skips `stayed` entries whose pre-mailbox is empty (the caster wasn't there before the skill).

**ACTUAL root cause (traced 2026-07-31):** the earlier partial fix stripped `diff.moves` by `m.from === src` and `diff.stayed` by `s === dest` — but the repro is the **equidistant-tie** case, which produces neither shape. Trace of `h3*f3:Tempest`: caster h3(23) ranged-kills victim f3(21), steps to g3(22, empty). `diffSkillMailbox` sees vacated {f3, h3}, arrived {g3}; g3 is chebyshev-dist 1 from BOTH, and the nearest-vacated loop breaks the tie toward the LOWER square index → pairs g3 with the **victim f3** (move f3→g3, read as +1 HP = spurious heal on g3) and lists the **caster h3** as a death. The old strip (`m.from === src=h3`) never touched the bogus `f3→g3` move, so the heal survived and the real kill rendered on h3 (the caster's origin) instead of f3.

**Fix applied:** `extractCasterMove` now strips the arrival at `dest` by its **`to === dest`** (identity-blind, robust to whichever `from` the tie chose), and when that stripped move's `from` is a stranded victim (≠ `src`, occupied pre-skill) it re-registers it in `diff.deaths` so `emitRelocationAndDeathEvents` fires the kill on the correct tile. Point-blank-kill (caster steps onto victim tile → `dest` in `stayed`) and point-blank-non-kill (caster doesn't move) paths are unchanged. Regression test added in `ply-renderer.test.ts` ("Strike ranged KILL … equidistant … BUG-1 repro") — verified it FAILS on the old strip logic (spurious heal present) and PASSES with the fix. Full suite 372 pass, check 0/0.

---

### BUG-2 · AIvAI match: "thinking…" depth/score pill not shown for either player ❌

**What happened:** In HvAI mode the thinking pill (depth + score badge in PlayerPanel) appears correctly. In AIvAI mode neither panel shows it.

**Root cause area:** `routes/match/+page.svelte` — in HvAI, `runAiStep` calls `beginSearch(side)` / `updateDepth(side, d, s)` / `endSearch(side, plyCount)` which write into the `aiSearch` store that `PlayerPanel` reads. In AIvAI the view loop (`advanceView`) replays producer-computed raws via `applyAndRender` — it never calls `beginSearch` / `updateDepth` / `endSearch`. The `aiSearch` store slots for both seats therefore stay at their initial state (`thinking: false`, `lastDepth: null`) throughout playback.

**Fix direction:** In `advanceView`, before calling `applyAndRender`, read the per-ply `SearchMeta` from the producer log (it records depth + score for each AI ply). After applying, call `setFinalDepth(side, depth)` + `endSearch(side, plyCount)` with those values so the panels show the depth/score linger badge. The producer log already carries `ai.depth` and `ai.score` per ply — these are parsed in `loadFromJson` in replay; the same parse can be done here from `producerRaws` metadata.

---

### BUG-3 · Resuming an AIvAI match loads default start position instead of saved state ❌

**What happened:** Clicking Resume on an AIvAI "Abandoned" row in the library navigates to `/match/` but shows the default starting position with no skills drafted, as if a fresh game started.

**Root cause area:** The resume path in `routes/match/+page.svelte` calls `getTelemetryStore().getResumeSnapshot(matchId)` which should return the stored mid-game snapshot. Two likely causes:
1. The AIvAI session never writes an incremental snapshot to IDB (the producer runs on a background thread and the view engine may not be snapshotted mid-game the way HvH/HvAI does), so `getResumeSnapshot` returns null and the code falls back to `eng.createEngine()` (fresh default).
2. The `match.side` set in the library resume function is `{ p1: "ai", p2: "ai" }` which is correct, but `modeFromSeats` will derive `"aivai"` — and the AIvAI boot path in `onMount` immediately tries to start the producer rather than waiting for a snapshot restore, potentially racing with or overwriting the restored state.

**Fix direction:** Check whether `idb-backend.ts` writes a resume snapshot for AIvAI matches during the session, and whether `getResumeSnapshot` finds it. If the snapshot is missing, the AIvAI session needs to periodically persist the *view engine* snapshot (not the producer) so resume has something to restore. If the snapshot exists but is being overwritten by the producer start, guard the producer start so it only fires when `resumeMatchId` is null.


---


---

# ── Work Log & Status (updated 2026-07-31, session 2) ──

Verification model (important): the game **engine is Tauri-only** (`invoke` IPC) — it does
NOT run under the `npm run dev` browser. So:
- Pure UI / layout / navigation → verified headlessly with **Playwright** (dev-only dep,
  never shipped; browser binary in ~/Library/Caches). Scripts in `frontend/tests-e2e/`.
- Engine logic → verified with **Rust unit tests** and **TS vitest** unit tests.
- Engine-dependent UI in the live window → user verifies via `cargo tauri dev` (checklists below).

Test state at last checkpoint: frontend `npm run check` = 0 errors/0 warnings; `vitest` = 363 pass.
Rust `core_engine` + `tauri_wrapper` compile; draw + FEN Rust tests pass.

## ✅ DONE (code complete + tested/typechecked; ⟳ = also needs a live Tauri eyeball)

- **P1-A Shove direction picker** — removed cursor auto-resolve in `openDirectionPicker`
  (match/+page.svelte); arrow overlay always shows when >1 legal dir. User confirmed "shove works".
- **Back button = global header** — `+layout.svelte` renders a fixed top-left pill (like
  Settings/Help), hidden on hub, route-context destinations (replay→library), teardown for
  multiplayer via `setBackNav`/`clearBackNav` (`lib/state/back-nav.svelte.ts`). Removed all
  per-route `<BackButton>`. 13/13 Playwright checks.
- **Back button overlap (#29)** — `.app-body.has-back` wrapper + global rule pads route header
  left 92px / right 172px so content clears the fixed pills. 6/6 Playwright. ⟳ match board.
- **P9-A Position Builder FEN** — `moved_this_phase` zeroed + turn-scoped trailers dropped in
  `mutateBoardToStaticFen` (`lib/state/position-fen.ts`). Rust regression test + TS tests.
- **Pos-builder click-to-edit (#24)** — Board `clickPieceOnTap` prop; tap opens editor. Removed
  redundant 2nd FEN input; live-FEN textarea (see+edit+load) + Copy.
- **Pos-builder KingCount BUG (#31)** — root cause: `parseBoardSection` split ranks on `/` which
  also appears INSIDE mailbox brackets (`G[1/0/0/0/0]`), shattering ranks and dropping the king.
  Fixed with `splitRanks` (bracket-aware). 10 TS tests.
- **Pos-builder edit UX (#27)** — HP/Armor as segmented toggles, skills as NAME dropdowns,
  auto-apply on change (no Apply btn), panel stays open until Close/click-away. ⟳ live.
- **P1-B MP resign** — added `{kind:"resign",seat}` to protocol-v2 (+decode+tests), sendResign/
  onResign in multiplayer-engine, confirmResign sends it, onResign→resignGame. 45 protocol tests.
  Draw path (HvAI/MP) was already correct. ⟳ live (MP two-window).
- **P1-B draw uses SEARCH not static (#28)** — `evaluate_draw_offer` now runs
  `request_ai_move_forced` (real search at seat budget), rotates stm→P1→AI-relative. 3 Rust tests
  prove searched≠static. ⟳ live.
- **Draw threshold (#33)** — 50 → **100** cp. (Note: searched start-pos tempo ≈86cp, so AI now
  accepts draws from a balanced start; 100 is the forgiving value user asked for.)
- **P1-C / BUG-3 resumability** — AIvAI now persists a view-engine resume snapshot each
  `advanceView` ply (was skipped). `seatsFromMode()` helper (unit-tested) recovers seats incl
  aivai. ⟳ live (start aivai, leave, resume → mid-game not blank).
- **Resume banner label (#30)** — `matchModeLabel()`; aivai reads "AI vs AI" not "vs AI".
- **Resume banner dismiss (#25→#32)** — hub ✕ now `markAbandoned` (NOT delete): game leaves the
  banner but stays in library + resumable. Banner query = in-progress only. ⟳ live.
- **Abandoned ply count (#26)** — library shows ply count whenever `totalPlies` set (not
  "in progress") for abandoned/network-lost rows.
- **P8-A replay Inspector** — fixed `result.notation` type error (convert `appliedAction` via
  `actionToNotation`) → build now 0 errors; best-move works; insights panel width clamp;
  end result already shows beside last ply. POI deferred (#23).

## ⟳ LIVE-VERIFY CHECKLIST (run in `cargo tauri dev`)
1. Position builder: click piece → toggles/dropdowns, live update, panel stays open; edit a
   GUARD's HP → no KingCount error, kings preserved.
2. AIvAI: depth/score pills show for BOTH seats (**STILL BROKEN — see #19**).
3. Draw: HvAI offer draw → AI accepts unless clearly winning (>100cp).
4. MP resign (2 windows): resign → both windows end, resigner loses.
5. Resume: start aivai/hvai/hvh, leave, resume from library → restores mid-game.
6. Hub ✕ → game leaves banner, still in library.
7. Back button not covering board/game elements.

## ❌ STILL OPEN (pending) — in rough priority order

- **#19 BUG-2 AIvAI depth pills** — ✅ FIXED (session 3), ⟳ needs Tauri eyeball. Root cause CONFIRMED:
  `advanceView` (the AIvAI producer/view log-player) rendered plies but NEVER drove the `aiSearch`
  store that `PlayerPanel` reads (`beginSearch`/`updateDepth`/`endSearch`) — only HvAI's `runAiStep`
  did. Both panels' seat slots therefore stayed `thinking:false, lastDepth:null` the whole game →
  no pill. FIX: (1) new `producerMetaFromLog()` in `ai-service.ts` extracts per-ply `{depth,scoreCp}`
  from the producer log's `plies[].ai` SearchMeta, index-aligned with `producerRawsFromLog` (same
  truncation, null for plies lacking meta → pill still shows via `thinking` flag = resilient).
  (2) `+page.svelte` keeps `producerMetas` beside `producerRaws`, refreshed in the SAME
  `onAivaiProgress` handler + initial pull. (3) `advanceView` now captures `side` before apply,
  calls `beginSearch(side)` + `updateDepth`/`setFinalDepth` (score flipped to seat-POV for P2), and
  `endSearch(side, plyCount)` in `finally`. Both panels show the depth/score badge + linger, exactly
  like HvAI. `showAiDepth` toggle now works because the pill container is finally driven. 5 new
  vitest cases (`producerMetaFromLog`); 368 pass; check 0/0. ⟳ VERIFY: start aivai → both panels
  show `thinking d… +…` then a `done d…` linger each ply.
- **#11 P3-E action-log preview isolation** — ✅ FIXED (session 3), ⟳ needs Tauri eyeball. Root
  cause CONFIRMED: `selectPreviewPly` mutated the ONE live engine in place (`restoreFromSnapshot`+
  `tryApply`); AI loops / auto-end-phase / MP apply didn't gate on preview → scrubbing could corrupt
  the live game + write a bad AIvAI resume snapshot. FIX (per user decision: live game must KEEP
  RUNNING in ALL modes — HvAI AI keeps moving, AIvAI keeps auto-playing, MP peer moves keep landing;
  looking at the past costs you time, it is not a pause): preview now runs on a SEPARATE isolated
  engine (`new TauriClient()` → its own `EngineRegistry` handle) + a silent `createPlyRenderer`,
  fast-forwarded via the replay `fastForwardTo` pattern. The live `eng` is only read (`matchLogJson`)
  never mutated. `previewing` derived swaps the Board's render source (position/pieceIds/motion/
  shaking/lastApplied/toMove) to the preview renderer and forces `interactive=false`; live loops
  stay UNGATED so the game advances off-screen. Preview stays FROZEN (base line re-read only on an
  explicit ply click). `ActionLogPanel` gained a sticky "⏭ Jump to latest (N)" catch-up bar shown
  when `selectedPly < entries.length`, and its auto-scroll is gated on `selectedPly===null` so new
  live moves don't yank the list. Preview engine disposed on return-to-live + onDestroy (no handle
  leak). check 0/0; 368 vitest pass; Rust compiles. FOLLOWUP (user tested): (a) muted live SFX
  during preview via `sfxEnabled: () => !previewing` on the live renderer; (b) EffectsLayer now binds
  to the PREVIEW renderer's queue while previewing (live pulses no longer play over the frozen
  board); (c) `teardownPreview` resyncs the live renderer on return-to-present to drop the effect
  backlog that accumulated undrained during preview (no burst on catch-up); (d) visual indicators in
  ActionLogPanel — selected ply gets accent fill + left bar + "viewing" pill; the live-head ply you
  jumped from gets a "left here" pill + dashed left bar; "Present" row relabeled "Back to present".
  ⟳ VERIFY: HvAI/AIvAI click a past ply → board freezes, AI keeps moving in log, NO sound/effects
  from live moves, "viewing"/"left here" tags visible, "jump to latest" appears, click → snaps to
  live cleanly (no effect burst); MP no desync; AIvAI resume snapshot still true-line. Future
  per-player clock noted (memory project_future_player_clock).
- **#12 P2 cluster** — ✅ FIXED (session 4), ⟳ needs Tauri eyeball. Per user "never trust the
  code, only trust me": treated all sub-items as open; fixed the concrete defects provable by
  reading and left repro steps for the live-only ones.
  - **P2-E grey-on-load** — root cause: `usedThisPhase` was tracked incrementally via the
    renderer's `onMoveLanding` callback, so a resume/snapshot/preview LOAD showed nothing greyed
    (no move history). The engine has the authoritative `moved_this_phase` **Bitboard** on
    `Position` but it was NOT projected into `PositionView`. FIX: added `moved_this_phase: u64`
    to `PositionView` (`wrapper_api.rs`) + `movedThisPhase: String` to `PositionViewDto`
    (`tauri_wrapper/src/lib.rs`, both the `position_view` command and `fen_to_position_view`) +
    `movedThisPhase: bigint` to the TS `PositionView` + nullish-guarded decode in
    `normalisePositionView` (`toBigInt(dto.movedThisPhase ?? 0)`, so older backends degrade to
    "nothing greyed"). `+page.svelte` now DERIVES the greyed set from the bitboard
    (`movedSquares = new Set(bitsOf(match.position.movedThisPhase))`, `previewMoved` for the
    frozen preview) and retired the incremental `onMoveLanding` writer + the `usedThisPhase = new
    Set()` phase-reset (the engine clears the bitboard on the phase flip). Correct on ALL entry
    paths now. 3 new vitest cases (movedThisPhase round-trip / 64-bit / absent→0n); Rust
    round-trip test asserts `v.moved_this_phase == p.moved_this_phase.0`.
  - **P2-C enemy-hover approach** — TWO root causes (fixed session 5 after user retest):
    (1) the hover ring + landing crosshair in `Board.svelte` were gated on `isDragging`, so a
    SELECTED piece hovering an enemy in click-to-move showed no drag-like preview; (2) the REAL
    blocker — `hoverSq` / `cursorX` / `cursorY` (which `clickLanding` / `hoverApproachChoices` /
    `clickHoverTarget` all read) were ONLY set by `handleDragMove` during a drag; the mouse-move
    handler wrote to a DIFFERENT variable (`hoveredSq`, for the eval card) with client (not SVG)
    coords. So in click mode those derivations saw a stale null and nothing rendered. FIX:
    `Board.onSquareHover` now also emits SVG coords `(svgX, svgY)` (computed via `clientToSvg`);
    `+page.svelte`'s handler mirrors the pointer into `hoverSq` + `cursorX/cursorY` when not
    dragging. Plus new `clickHover` prop + unified `activeHover` drives the ring/crosshair in both
    modes. Now a selected piece hovering a legal target previews exactly as if dragged (approach
    squares + landing crosshair), for single- and multi-approach.
  - **P2-A board flip** — went through several wrong attempts; FINAL correct model (session 5c):
    the flip is a **single SVG transform on an inner `.flip-layer` `<g>`** wrapping all board
    content: `rotate(180 viewBox/2 viewBox/2)` (grid centre) when flipped. This is NOT a CSS
    transform on the `<svg>` — that was the root of the churn, because `getScreenCTM()` did not
    fold CSS transforms in reliably in the WebView, so pointer→coord math had two mismatched
    frames (piece placed in one, hit-tested in another → mirror/whole-board-offset). With the
    inner-`<g>`, `getScreenCTM()` stays a pure viewBox map and there is ONE frame: `clientToSvg`
    reflects the cursor through the grid centre (`viewBox - x, viewBox - y`) into layer space, and
    the dragged-piece override, square resolution, and the parent's approach-picker cursor coords
    ALL use those same coords. Counter-rotations keep glyphs upright: Piece `.body` (about its own
    centre), the skill wheel (about its piece centre), and each coordinate-label GROUP
    (`rotate(180 viewBox/2 viewBox/2)` — composes with the flip-layer to identity, so labels stay
    pinned at their normal screen spots and upright; only the VALUE flips: files `h..a`, ranks
    `8..1`). EffectsLayer is a separate canvas → CSS-rotated 180° with per-draw text
    counter-rotation. Outside the SVG, `.board-column.flipped { flex-direction: column-reverse }`
    swaps the PlayerPanels. DirectionPicker arrows intentionally rotate WITH the layer (spatial).
  - **P2-B speed-1 trail** — root cause: trail was gated on `selectionIsSpeed2` (true if ANY
    target of the piece is speed-2), so a speed-2-capable piece dragged onto a *speed-1* target
    still drew intermediate squares. FIX: gate the trail on the LIVE hover — show it only when
    `dragLanding !== dragHover` (this specific approach is genuinely speed-2). Removed the now-
    dead `selectionIsSpeed2` derived.
  - **P2-F last-action highlight on end-phase** — traced: the renderer already preserves
    `lastApplied` through EndPhase and nothing in `afterApplied` nulls it. No code change; ⟳
    verify live and report if it clears (would patch `afterApplied`).
  - check 0/0; 371 vitest pass; core_engine + tauri_wrapper compile; Rust round-trip test passes.
  ⟳ VERIFY (cargo tauri dev): (E) play a move → resume/reload → moved piece greyed on load;
  preview a past ply → greying matches that ply. (C) select a piece, hover an enemy →
  crosshair + approach preview like a drag, single- and multi-approach. (B) drag speed-2 piece
  onto a speed-1 target → no trail; onto a speed-2 target → trail. (A) HvAI-as-P2: the WHOLE board
  is rotated 180° — your pieces at the bottom, YOUR banner under the board, pieces/skill-glyphs/
  labels upright, square colours unchanged, effects + turn-strip on the correct edges, wheel +
  bodyguard highlights correctly placed. (F) move then end phase → last-action highlight persists.
- **#13 P3 HUD**: ✅ FIXED + user-confirmed live (2026-07-31). Root causes + fixes:
  - **P3-A money/skill-cost preview** — traced as ALREADY CORRECT. `pendingSkillCost`
    (match/+page.svelte) derives purely from live `hoveredSlice`/`armedSkill`; it is only
    nulled in `afterApplied()` on a **phase-key change** (to-move/phase flip), not on every
    apply. That is the desired "clears only on phase flip" behavior. No code change — live-verify:
    hover/arm a skill → `−cost` shows next to money; it should persist across intra-phase hovers
    and clear when the phase ends.
  - **P3-B phase action counts** — the *inactive* phase box showed a dash (`movePhaseActionsHistory
    ?? "-"` for Move, `"-"` for Skill). Now both boxes always show the ruleset-derived budget when
    inactive: `movePhaseBudget = 2`, `skillPhaseBudget = 2 + floor((roundNumber-1)/10)` (greyed).
    Retired the now-dead `movePhaseActionsHistory` state + its snapshot block in `afterApplied`.
  - **P3-D taken-pieces box count** — boot detection used P1's *actual* counts as the box count, so
    a custom position with fewer pieces showed fewer boxes (and asymmetric armies used the wrong
    side). Now `baselinePieces` = `max(STANDARD {1K,5C,6G}, actual-P1, actual-P2)` per type: custom
    positions with fewer pieces still show the standard box count (surplus boxes pre-fill as
    "captured" via PlayerPanel's `max(0, baseline-alive)` — matches spec), more pieces grow the row,
    symmetric across both panels.
  - VERIFY (cargo tauri dev): (A) hover a skill → money shows `−cost`, survives re-hover, clears on
    phase end. (B) In Move phase the Skill box shows its future count greyed (e.g. `2` at round 1,
    `3` at round 11+); in Skill phase the Move box shows `2` greyed. (D) Launch from a custom
    position with e.g. 3 champions → still 5 champ boxes, 2 pre-filled; launch with 7 champs → 7
    boxes.
- **#14 P4 skill wheel** ✅ FIXED (session 6). Both sub-items:
  - **P4-B tooltip beside piece** — root cause: `.info-anchor` was `position:absolute; top:0;
    left:calc(100%+0.6rem)` — pinned to the board-stack's top-right corner, nowhere near the
    hovered piece. FIX: the info-anchor now derives its position from `wheelOpen.square` — file/rank
    → `%` of the (square) board-stack (`colFrac`/`rowFrac`, mirrored when `boardFlipped`), and a
    `.to-right`/`.to-left` class (chosen by which board half the piece is on, so the card never runs
    off-screen) translates it beside the piece, vertically centred. Delay (1s) unchanged.
  - **P4-C focus split picker** — root cause: the split was DEAD CODE. The `SkillWheel` split only
    rendered when `focusModeChoice` was passed, and the parent only passed it when
    `armedHasFocusModeChoice && armedSkill` — but `wheelOpen` returned `null` the instant
    `armedSkill !== null`, so the wheel (and its split) vanished before it could ever show. SECOND
    root cause (found after user retest — "never splits, even for movement skills"): even once the
    split was decoupled from arming, gating it on `hasFocusModeChoice(legal, …)` required BOTH
    focus_mode variants to be *currently legal* (an enemy in effect-range) — so with no target in
    range, Blast/Shove didn't split. THIRD: the user wants self/defensive skills to split too, but
    those don't use the focus_mode bit at all — their two variants are self-cast vs ally-retarget
    (Focus's +1 Range lets the caster channel Shield/Dash/Retreat onto an adjacent ally).
    FIX (final model): the split is now driven by skill TYPE via `focusSplitKind(skillId)`
    (`$lib/engine/skills`): Blast/Shove → `"focusMode"` (+rng activation / +eff effect quarters);
    Shield/Dash/Retreat → `"retarget"` (self / ally quarters); everything else → null. A slot splits
    whenever `focusActive && focusSplitKind !== null`, INDEPENDENT of whether both quarters are
    currently castable — each quarter's legality (`split{1,2}.aLegal/bLegal`, computed from the
    engine's actual variants via `skillVariantsFor`) greys the unavailable side. `SkillWheel` now
    takes per-slot `split1`/`split2` `SplitDesc {kind, aLegal, bLegal, armed}` (replacing the old
    `focusModeChoice`/`splitSlot`/`armedMode` props) and renders quarter labels by kind
    (+rng/+eff or self/ally). Clicking a quarter sets `focusModePref` (focusMode) or
    `focusRetargetPref` (retarget) AND arms that slot's skill so the next click is the target
    (per user: "arm the skill with that mode"); a self-cast quarter with no retarget ambiguity fires
    immediately. Picking a quarter arms AND CLOSES the wheel (same as a normal skill half) so its
    chrome can't intercept target-tile clicks; the armed skill stays cancelable via ✕ Cancel / Escape
    (re-pick the other variant by cancelling + reopening). `SliceKind.focusBoost` now carries
    `{skillId, variant}` (variant = activation|effect|self|ally); `SkillInfoCard` renders a
    focus-variant badge (new i18n keys `wheel.focusActivation/focusEffect/focusSelf/focusAlly` in
    en+de). New `focusSplitKind` contract test (373 total; was 372).
  - check 0/0; 373 vitest pass. ⟳ VERIFY (cargo tauri dev): (B) hover a wheel slice → tooltip
    appears BESIDE the piece (right of it on left-half pieces, left of it on right-half), after ~1s;
    flipped board (HvAI-as-P2) → still correctly beside the piece. (C) stage Focus, open the wheel on
    a piece with Blast/Shove → that skill's half splits into +rng / +eff quarters; on a piece with
    Shield/Dash/Retreat → that skill's half splits into self / ally quarters (both halves split if
    both slots are focus-eligible); the split shows EVEN with no legal target for one quarter (that
    quarter greyed); click a quarter → skill arms with that variant AND the wheel closes (like a
    normal half), board shows that variant's targets; ✕ Cancel / Escape disarms (re-pick a variant by
    cancelling + reopening). Non-focus-eligible skills and empty slots render as before.
- **#18 BUG-1 ply-renderer heal glitch** ✅ FIXED. Root cause was NOT the earlier
  `from===src`/`stayed` strip — it was the equidistant-tie in `diffSkillMailbox`'s identity-blind
  nearest-vacated pairing: the caster's arrival tile is dist-1 from both its origin and the ranged
  victim, tie breaks to the lower square index → arrival mispaired with the victim (spurious heal),
  caster origin listed as death. Fix: `extractCasterMove` strips the arrival by `to===dest` and
  restores the stranded victim `from` to `deaths`. Regression test verified to fail on old code.
- **#21 P5-B sandbox Play-My-Moves** ✅ FIXED + user-confirmed live (2026-07-31). Rebuilt as a
  **frontend ply queue** (`playMyMovesQueue`) drained through the normal apply path.
  - **True root cause (found via full console trace, user-reported "nothing plays / NotAiTurn"):**
    the match-log ply's `action` field is an **`ActionDecoded` object** (`{raw, kind, src, …}`), not a
    raw u32. The code mapped `p.action` and filtered `typeof === "number"` → dropped every entry →
    the queue was ALWAYS empty → the drain no-oped. All observed symptoms were downstream of the
    empty queue: nothing committed; the auto-end-phase effect then fired a stray EndPhase; and in the
    multi-turn case the AI scheduler fired on the restored AI-turn → `NotAiTurn`. Fix: read
    `p.action?.raw`.
  - **Design (also fixes the multi-turn semantics):** `confirmPlayMyMoves` gathers the staged raws,
    loads `playMyMovesQueue` BEFORE the mode flip / awaits (so the gated effects see it), restores the
    true line, exits sandbox, kicks `drainPlayMyMoves()`. The driver applies one ply at a time and
    STOPS (clearing the queue) when: queue empty; the next ply's turn belongs to a non-local/AI seat
    (→ AI scheduler resumes and plays its REAL move); `pending_bodyguard` is set (notice, user
    resolves); the user intervenes (press/click clears the queue); or an apply is refused. The HvAI
    auto-step scheduler AND the auto-end-phase effect are both gated `if (playMyMovesQueue.length > 0)
    return` so neither races the drain. Non-error notice auto-clears when the bodyguard resolves.
  - **User-confirmed live:** (1) own-turn moves commit with the delay; (2) 2 moves commit then the
    phase auto-ends correctly (after the drain, not prematurely); (3) moves into the AI's turn commit
    up to the handoff, drain stops at `not local human turn`, AI plays for real — NO NotAiTurn.
    check 0/0, 372 tests pass.
- **#20 P7-A draft King label + dead code** ✅ FIXED. Root cause: `boardOrder()` sorts squares by
  file so pieces read left-to-right, but the template computed `isKing = i === 0` on the SORTED
  array — after sorting, P1 King (d1=sq3) lands at index 2, so the leftmost Champion (b1) got
  labeled "King" and the real King labeled a Champion. Fix: `boardOrder()` now returns
  `{sq, isKing, championIdx}` objects, deriving `isKing` from the loadout's King-first square
  identity and numbering Champions 1..5 in board order; template destructures these. Also removed
  dead `showCustomDropdown` state + never-called `toggleCustomDropdown`. Verified: P1 → b1=C1 c1=C2
  d1=KING e1=C3 f1=C4 g1=C5; P2 → b8=C1 … e8=KING …. check 0/0, 372 tests pass.
  ⟳ **P7-B (unify draft/loadout visual language + un-hide premade loadouts from dropdown)** remains
  — that's a layout/style redesign, not part of this bug-scope; deferred to a dedicated pass.
- **#22 P10-B/D library**: tag games (tags?:string[] on MatchMeta + updateTags + UI); remember
  hide-incomplete toggle (persist via settings.svelte.ts).
- **#23 POI in replay** (deferred): old inspector POI is tree-based, doesn't map onto replay's
  linear scrubber — needs design (bookmark plies?).

## OPEN DESIGN QUESTIONS (need user decision)
- **Abandoned vs in-progress lifecycle**: match teardown currently auto-marks games `abandoned`
  on navigate-away. Under the new "banner = in-progress only" model, the banner then only shows
  crash/hard-reload-interrupted games. If you want navigate-away to KEEP a game `in-progress`
  (so the banner shows games you left but didn't dismiss), teardown's `markAbandoned` must be
  removed/changed. Decide before finishing #32.
- **Draw threshold**: now 100cp. The searched eval sees a real side-to-move tempo (~86cp at start).
  Revisit if the AI feels too eager/reluctant to draw.

## PRINCIPLE PROOF-CHECK (from session-1 audit; still valid)
- Weakest = "No dead ends": match END-STATE has zero navigation (no replay/rematch/library). Highest
  single UX lever, still open. Setup is asymmetric-by-default; mirrorSeats omits evaluator.

## Dev-only test assets (never shipped)
- `frontend/package.json` devDependencies: `playwright`. Chromium in ~/Library/Caches/ms-playwright.
- `frontend/tests-e2e/verify-backbtn.mjs`, `verify-pill-overlap.mjs` (run against a live vite :5173).
- Unit tests added: `lib/state/position-fen.test.ts`, `match-store.svelte.test.ts` (seatsFromMode),
  protocol-v2 resign cases; Rust: fen.rs `position_builder_moved_bit_roundtrip`,
  wrapper_api.rs `evaluate_draw_offer_*`.
