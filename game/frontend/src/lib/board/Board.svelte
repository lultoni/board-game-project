<script lang="ts">
  import { readPieces, type PositionView } from "$lib/engine";
  import Piece from "./Piece.svelte";
  import SkillWheel, { type SliceKind } from "./SkillWheel.svelte";
  import DirectionPicker from "./DirectionPicker.svelte";
  import type { SkillVariant } from "$lib/state/skill-targets";

  interface Props {
    position: PositionView | null;
    /** Stable per-piece identity, keyed by current square. Used as the
     * `{#each}` key so the Piece element keeps DOM identity across moves
     * and the CSS slide transition runs. */
    pieceIds?: Map<number, number>;
    /** SVG viewBox edge in pixels. Default 800 → 100px squares. */
    viewBox?: number;
    /** Square currently selected by the player, or null. */
    selection?: number | null;
    /** Squares reachable as Move-action targets from the selection. */
    moveTargets?: Set<number>;
    /** Squares that can be selected (i.e. own a piece with at least one legal action). */
    selectable?: Set<number>;
    /** Subset of `selectable` whose piece can be picked up for a drag-to-move
     *  gesture. Outside the Move Phase this is typically empty. */
    draggable?: Set<number>;
    /** Squares whose piece has already used its Move this phase — greyed out. */
    usedSquares?: Set<number>;
    /** Approach-square chooser: when set, render highlights on these squares
     * and trigger `onApproachChoice` instead of normal clicks. */
    approachChoices?: number[];
    /** Bodyguard chooser: when non-null, render special "choose" highlights
     *  on the defender + each eligible Guard tile. Other tiles render
     *  non-interactive (clicks elsewhere are ignored by the parent). */
    bodyguardChoice?: {
      defender: number;
      guards: number[];
    } | null;
    /** Last-applied action: a pair of {src, target} squares for the "what just happened" hint. */
    lastApplied?: { src: number; target: number } | null;
    /** Squares whose piece is currently mid-hit-shake. */
    shakingSquares?: Set<number>;
    /** Whether the human can act right now. */
    interactive?: boolean;
    /** Pause piece idle-breathing while overlay effects are mid-play. */
    effectsActive?: boolean;
    /** Pointer-up handler — passes the clicked square index (0..63) and the
     * cursor position within the SVG (used to pick a sub-tile approach for
     * multi-path Move-Attacks). */
    onSquareClick?: (square: number, x: number, y: number) => void;
    /**
     * Drag-drop callback. Fires on pointerup if a drag actually crossed
     * to a different square. `path` is the ordered list of distinct squares
     * the pointer hovered over (including src as path[0], drop as path[-1]).
     * `(x, y)` is the cursor position in SVG coords at the moment of drop.
     */
    onPieceDrop?: (src: number, path: number[], x: number, y: number) => void;
    /** Click on an approach-chooser square. */
    onApproachChoice?: (approach: number) => void;
    /** Pointer pressed down on a selectable piece (before drag threshold). */
    onPressStart?: (src: number) => void;
    /**
     * Live drag updates. `overSq` is the square under the cursor (or null
     * outside the board); `path` is the ordered list of distinct squares
     * the cursor has crossed since press. `(x, y)` is the live cursor
     * position in SVG coords (used by the parent to choose a sub-tile
     * approach for multi-path Move-Attacks). Reset to (0, null, [], 0, 0)
     * on pointerup/cancel.
     */
    onDragMove?: (src: number, overSq: number | null, path: number[], x: number, y: number) => void;
    /**
     * Squares to render with a faint trail tint (the inferred drag path).
     * Parent computes this from the live drag state.
     */
    dragTrail?: number[];
    /** Square currently under the cursor during a drag — render an extra ring. */
    dragHover?: number | null;
    /** Whether the dragHover square is a legal drop. Affects the ring colour. */
    dragHoverLegal?: boolean;
    /** Inferred attacker landing square for the current drag (approach_sq
     * on Move-Attack, target on plain Move). Null when ambiguous. */
    dragLanding?: number | null;
    /** When non-null, render the radial skill wheel around this square.
     *  All other props in this group are read while the wheel is open. */
    wheelOpen?: {
      square: number;
      skill1: number;
      skill2: number;
    } | null;
    /** Currently-armed skill id (drives the pulsing ring on the relevant slice). */
    armedSkillId?: number | null;
    /** Focus / Charge modifier toggles staged for the next cast. */
    focusActive?: boolean;
    chargeActive?: boolean;
    /** Legality of each interactive sector (engine-derived). Disabled
     *  sectors render greyed. */
    wheelLegality?: {
      skill1Legal: boolean;
      skill2Legal: boolean;
      endPhaseLegal: boolean;
    };
    onWheelSliceClick?: (slice: SliceKind) => void;
    onWheelSliceHover?: (slice: SliceKind | null) => void;
    /** Direction picker (Shove). When non-null, render an arrow ring on
     *  `target` and pass clicks back via the handlers below. */
    directionPicker?: {
      target: number;
      variants: SkillVariant[];
    } | null;
    onDirectionPick?: (raw: number) => void;
    onDirectionCancel?: () => void;
    /** Per-square lunge offsets (SVG px). Drives the lunge-recoil CSS animation
     *  on non-kill attacks: piece at `sq` lunges by (dx, dy) then recoils. */
    lungeSquares?: Map<number, { dx: number; dy: number }>;
  }

  let {
    position,
    pieceIds = new Map<number, number>(),
    viewBox = 800,
    selection = null,
    moveTargets = new Set<number>(),
    selectable = new Set<number>(),
    draggable = new Set<number>(),
    usedSquares = new Set<number>(),
    approachChoices = [],
    bodyguardChoice = null,
    shakingSquares = new Set<number>(),
    lastApplied = null,
    interactive = true,
    effectsActive = false,
    onSquareClick,
    onPieceDrop,
    onApproachChoice,
    onPressStart,
    onDragMove,
    dragTrail = [],
    dragHover = null,
    dragHoverLegal = false,
    dragLanding = null,
    wheelOpen = null,
    armedSkillId = null,
    focusActive = false,
    chargeActive = false,
    wheelLegality = {
      skill1Legal: false,
      skill2Legal: false,
      endPhaseLegal: false,
    },
    onWheelSliceClick,
    onWheelSliceHover,
    directionPicker = null,
    onDirectionPick,
    onDirectionCancel,
    lungeSquares = new Map<number, { dx: number; dy: number }>(),
  }: Props = $props();

  const SIZE = $derived(viewBox / 8);
  /** Pad around the 8×8 grid inside the SVG's viewBox so the radial
   *  skill wheel (which extends ~1.05 × SIZE beyond a piece's tile)
   *  can render AND receive pointer events without spilling outside
   *  the SVG element's hit-box. Using a negative-origin viewBox keeps
   *  all square coordinates (0..viewBox) unchanged. */
  const WHEEL_PAD = $derived(SIZE * 0.6);

  const pieces = $derived(
    position ? readPieces(position.bitboards, position.mailbox) : [],
  );

  // 64 square rects. Rank 0 (P1 home) renders at the bottom.
  const squares = $derived(
    Array.from({ length: 64 }, (_, sq) => {
      const file = sq & 7;
      const rank = (sq >> 3) & 7;
      const x = file * SIZE;
      const y = (7 - rank) * SIZE;
      const light = (file + rank) % 2 === 1;
      return { sq, x, y, light };
    }),
  );

  const fileLabels = "abcdefgh".split("");

  // Pieces occupy squares — we use this to differentiate Move-target tint
  // between "empty target" (relocation) and "occupied target" (Move-Attack
  // or, on the rare swap-like cases, an ally).
  const occupied = $derived(new Set(pieces.map((p) => p.square)));

  // Hovered approach square — tracked while the approach chooser is active
  // so moving the cursor highlights which path would be selected on click.
  let approachHovered = $state<number | null>(null);
  $effect(() => {
    // Clear hover when the chooser is dismissed.
    if (approachChoices.length === 0) approachHovered = null;
  });

  // --- Drag state -----------------------------------------------------------
  // Pointerdown on a selectable piece starts a "press". The press only becomes
  // a drag once the pointer moves past DRAG_THRESHOLD_PX (in SVG coords) — this
  // way a still-pointer click works exactly like click-to-move and there's no
  // unwanted jitter when the user just taps.

  const DRAG_THRESHOLD_PX = 6;

  let svgEl: SVGSVGElement | undefined = $state();
  let press = $state<null | {
    src: number;
    pointerId: number;
    /** Cursor x/y at pointerdown, in SVG coords. */
    startX: number;
    startY: number;
    /** Live cursor x/y, in SVG coords. Updated on every pointermove. */
    x: number;
    y: number;
    /** True once we've crossed DRAG_THRESHOLD_PX — drag visuals are on. */
    dragging: boolean;
    /** Path of distinct squares visited (src first). Only meaningful when dragging. */
    path: number[];
    /** Square the cursor is currently over (or null if outside the board). */
    overSq: number | null;
    /** False for presses started on non-selectable squares — these become
     * taps only, never drags, and skip the parent's onPressStart. */
    draggable: boolean;
    /** True if the piece was already the active selection when pressed.
     * Used so a second tap toggles selection off (otherwise onPressStart
     * just re-selects the same square and a tap-up no-op leaves it stuck). */
    wasSelected: boolean;
  }>(null);

  const isDragging = $derived(press !== null && press.dragging);

  function clientToSvg(clientX: number, clientY: number): { x: number; y: number } {
    const svg = svgEl;
    if (!svg) return { x: 0, y: 0 };
    const pt = svg.createSVGPoint();
    pt.x = clientX;
    pt.y = clientY;
    const ctm = svg.getScreenCTM();
    if (!ctm) return { x: 0, y: 0 };
    const local = pt.matrixTransform(ctm.inverse());
    return { x: local.x, y: local.y };
  }

  function svgToSquare(x: number, y: number): number | null {
    if (x < 0 || y < 0 || x >= viewBox || y >= viewBox) return null;
    const file = Math.floor(x / SIZE);
    const rank = 7 - Math.floor(y / SIZE);
    if (file < 0 || file > 7 || rank < 0 || rank > 7) return null;
    return (rank << 3) | file;
  }

  function handleSquarePointerDown(sq: number, ev: PointerEvent) {
    if (!interactive) return;
    if (ev.button !== undefined && ev.button !== 0) return;
    // When the approach chooser is open, taps on highlighted squares are
    // routed via `handleSquareClickInternal` on pointerup; taps on anything
    // else dismiss the chooser via the parent. Either way we still want a
    // press → pointerup tap to fire, so we record a non-draggable press.
    const draggableHere =
      draggable.has(sq) &&
      approachChoices.length === 0 &&
      bodyguardChoice === null;
    const wasSelected = selection === sq;
    const { x, y } = clientToSvg(ev.clientX, ev.clientY);
    press = {
      src: sq,
      pointerId: ev.pointerId,
      startX: x,
      startY: y,
      x,
      y,
      // `dragging` is locked off for non-piece taps. Pointer-up will fire
      // a click on the same square the user pressed.
      dragging: false,
      path: [sq],
      overSq: sq,
      draggable: draggableHere,
      wasSelected,
    };
    if (draggableHere) onPressStart?.(sq);
    // Capture on the SVG root, not the rect — the rect can be re-rendered
    // out from under us mid-drag and lose the capture, freezing the piece.
    svgEl?.setPointerCapture?.(ev.pointerId);
  }

  function handlePointerMove(ev: PointerEvent) {
    if (!press || ev.pointerId !== press.pointerId) return;
    const { x, y } = clientToSvg(ev.clientX, ev.clientY);
    press.x = x;
    press.y = y;
    if (!press.draggable) return; // tap-only press; ignore movement
    if (!press.dragging) {
      const dx = x - press.startX;
      const dy = y - press.startY;
      if (dx * dx + dy * dy >= DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) {
        press.dragging = true;
      } else {
        return;
      }
    }
    const overSq = svgToSquare(x, y);
    press.overSq = overSq;
    if (overSq !== null && press.path[press.path.length - 1] !== overSq) {
      // If the cursor has revisited an earlier square, truncate the path
      // back to that index — backtracking should retract the trail, not
      // extend it. This also keeps the trail free of duplicates, which
      // matters because the keyed `{#each dragTrail}` block crashes on
      // duplicate keys and the freeze cascades into a stuck sprite.
      const existing = press.path.indexOf(overSq);
      if (existing >= 0) {
        press.path = press.path.slice(0, existing + 1);
      } else {
        press.path = [...press.path, overSq];
      }
    }
    onDragMove?.(press.src, overSq, press.path, x, y);
  }

  function handlePointerUp(ev: PointerEvent) {
    if (!press || ev.pointerId !== press.pointerId) return;
    const { x, y } = clientToSvg(ev.clientX, ev.clientY);
    const dropSq = svgToSquare(x, y);
    const { src, dragging, path, draggable, wasSelected } = press;
    press = null;
    onDragMove?.(src, null, [], 0, 0); // clear hover state in parent

    // Draggable tap (no drag crossed). If the piece was already selected
    // when pressed, toggle it off by firing the click (the parent's
    // `onSquareClick` does the toggle). If it's a fresh selection,
    // `onPressStart` already set it at pointerdown — bail.
    if (draggable && !dragging) {
      if (wasSelected) handleSquareClickInternal(src, x, y);
      return;
    }

    if (!draggable) {
      // Tap on a non-selectable square (move target, empty, etc.). Route
      // the click on whichever square the pointer was released over (so a
      // quick press-and-release on a Move-target square still commits).
      const clickSq = dropSq !== null ? dropSq : src;
      handleSquareClickInternal(clickSq, x, y);
      return;
    }
    const finalPath = path.slice();
    if (dropSq !== null && dropSq !== finalPath[finalPath.length - 1]) {
      finalPath.push(dropSq);
    }
    if (dropSq === null) return;
    if (dropSq === src) {
      handleSquareClickInternal(src, x, y);
      return;
    }
    onPieceDrop?.(src, finalPath, x, y);
  }

  function handlePointerCancel(ev: PointerEvent) {
    if (!press || ev.pointerId !== press.pointerId) return;
    press = null;
    onDragMove?.(0, null, [], 0, 0);
  }

  function handleSquareClickInternal(sq: number, x: number, y: number) {
    if (!interactive) return;
    if (approachChoices.length > 0) {
      if (approachChoices.includes(sq)) onApproachChoice?.(sq);
      else onSquareClick?.(sq, x, y); // let parent clear chooser on outside-tap
      return;
    }
    onSquareClick?.(sq, x, y);
  }

  function overrideForPiece(sq: number): { x: number; y: number } | null {
    if (press && press.dragging && press.src === sq) {
      return { x: press.x - SIZE / 2, y: press.y - SIZE / 2 };
    }
    return null;
  }
</script>

<svg
  bind:this={svgEl}
  class="board"
  class:interactive
  viewBox="{-WHEEL_PAD} {-WHEEL_PAD} {viewBox + 2 * WHEEL_PAD} {viewBox + 24 + 2 * WHEEL_PAD}"
  xmlns="http://www.w3.org/2000/svg"
  role="img"
  aria-label="game board"
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerCancel}
>
  <defs>
    <symbol id="skill-glyph-1" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 4 20 L 18 6" />
        <path d="M 18 6 L 14 6 L 18 6 L 18 10" />
        <path d="M 5 19 L 8 19 L 5 19 L 5 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-2" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 18 4 L 18 14 A 6 6 0 0 1 6 14" />
        <path d="M 6 14 L 4 12" />
        <path d="M 6 14 L 4 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-3" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 6 5 L 18 5 L 17 14 L 12 19 L 7 14 Z" />
        <path d="M 12 5 L 9 11 L 13 13 L 11 19" />
      </g>
    </symbol>
    <symbol id="skill-glyph-4" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="16" cy="8" r="3" />
        <path d="M 4 18 L 8 14 L 14 18 L 10 22 Z" />
      </g>
    </symbol>
    <symbol id="skill-glyph-5" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2.5" />
        <path d="M 12 4 L 12 7" />
        <path d="M 12 17 L 12 20" />
        <path d="M 4 12 L 7 12" />
        <path d="M 17 12 L 20 12" />
        <path d="M 6 6 L 8 8" />
        <path d="M 16 16 L 18 18" />
        <path d="M 6 18 L 8 16" />
        <path d="M 16 8 L 18 6" />
      </g>
    </symbol>
    <symbol id="skill-glyph-6" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 12 3 L 19 7 L 18 15 L 12 21 L 6 15 L 5 7 Z" />
      </g>
    </symbol>
    <symbol id="skill-glyph-7" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="8" />
        <path d="M 12 7 L 12 17" />
        <path d="M 7 12 L 17 12" />
      </g>
    </symbol>
    <symbol id="skill-glyph-8" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 5 8 L 19 8" />
        <path d="M 6 13 L 18 13" />
        <path d="M 8 18 L 16 18" />
      </g>
    </symbol>
    <symbol id="skill-glyph-9" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 4 12 L 14 12" />
        <path d="M 10 7 L 15 12 L 10 17" />
        <path d="M 14 7 L 19 12 L 14 17" />
      </g>
    </symbol>
    <symbol id="skill-glyph-10" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="6" cy="12" r="2.5" />
        <path d="M 11 12 L 21 12" />
        <path d="M 18 8 L 22 12 L 18 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-11" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2.2" />
        <path d="M 10 5 L 12 3 L 14 5" />
        <path d="M 10 19 L 12 21 L 14 19" />
        <path d="M 5 10 L 3 12 L 5 14" />
        <path d="M 19 10 L 21 12 L 19 14" />
      </g>
    </symbol>
    <symbol id="skill-glyph-12" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 5 9 L 18 9" />
        <path d="M 15 5 L 19 9 L 15 13" />
        <path d="M 19 15 L 6 15" />
        <path d="M 9 11 L 5 15 L 9 19" />
      </g>
    </symbol>
    <symbol id="skill-glyph-13" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <!-- Retreat: U-turn arrow pointing back to start, semantically
             "step backwards along your own path". Distinct from Hook
             (which curves once, no return). -->
        <path d="M 6 6 L 6 11 A 5 5 0 0 0 16 11 L 16 18" />
        <path d="M 13 15 L 16 18 L 19 15" />
      </g>
    </symbol>
    <symbol id="skill-glyph-14" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2" />
        <circle cx="12" cy="12" r="6" />
        <circle cx="12" cy="12" r="10" stroke-dasharray="2 3" />
      </g>
    </symbol>
    <symbol id="skill-glyph-15" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 13 3 L 6 14 L 11 14 L 9 21 L 18 9 L 13 9 Z" />
      </g>
    </symbol>
  </defs>

  <!-- Paper-tone square fills -->
  <g class="squares">
    {#each squares as { sq, x, y, light } (sq)}
      <rect
        {x}
        {y}
        width={SIZE}
        height={SIZE}
        fill={light ? "var(--paper-square-light, #ece2c8)" : "var(--paper-square-dark, #d8c89e)"}
        stroke="var(--paper-line, #b8a87c)"
        stroke-width="1"
      />
    {/each}
  </g>

  <!-- Last-applied source + target hints (drawn under the pieces so the
       piece itself reads cleanly). -->
  {#if lastApplied}
    {@const src = lastApplied.src}
    {@const tgt = lastApplied.target}
    <g class="last-applied">
      <rect
        x={(src & 7) * SIZE}
        y={(7 - ((src >> 3) & 7)) * SIZE}
        width={SIZE}
        height={SIZE}
        fill="var(--accent, #c79b3a)"
        fill-opacity="0.16"
        pointer-events="none"
      />
      <rect
        x={(tgt & 7) * SIZE}
        y={(7 - ((tgt >> 3) & 7)) * SIZE}
        width={SIZE}
        height={SIZE}
        fill="var(--accent, #c79b3a)"
        fill-opacity="0.28"
        pointer-events="none"
      />
    </g>
  {/if}

  <!-- Move-target highlights -->
  {#if moveTargets.size > 0}
    <g class="move-targets">
      {#each [...moveTargets] as tgt (tgt)}
        {@const file = tgt & 7}
        {@const rank = (tgt >> 3) & 7}
        {@const x = file * SIZE}
        {@const y = (7 - rank) * SIZE}
        {@const isAttack = occupied.has(tgt)}
        {#if isAttack}
          <!-- Move-Attack: red ring -->
          <rect
            {x}
            {y}
            width={SIZE}
            height={SIZE}
            fill="none"
            stroke="#c44a3a"
            stroke-width="3"
            stroke-opacity="0.85"
            pointer-events="none"
          />
          <circle
            cx={x + SIZE / 2}
            cy={y + SIZE / 2}
            r={SIZE * 0.42}
            fill="#c44a3a"
            fill-opacity="0.08"
            pointer-events="none"
          />
        {:else}
          <!-- Empty target: dot -->
          <circle
            cx={x + SIZE / 2}
            cy={y + SIZE / 2}
            r={SIZE * 0.16}
            fill="#3a6a4a"
            fill-opacity="0.32"
            pointer-events="none"
          />
        {/if}
      {/each}
    </g>
  {/if}

  <!-- Selection ring -->
  {#if selection !== null}
    {@const file = selection & 7}
    {@const rank = (selection >> 3) & 7}
    <rect
      x={file * SIZE + 2}
      y={(7 - rank) * SIZE + 2}
      width={SIZE - 4}
      height={SIZE - 4}
      fill="none"
      stroke="var(--accent, #c79b3a)"
      stroke-width="3.5"
      pointer-events="none"
    />
  {/if}

  <!-- Drag trail — faint tint on the squares the cursor has crossed since
       press. Helps the player see what path their drag has claimed. -->
  {#if isDragging && dragTrail.length > 1}
    <g class="drag-trail">
      {#each dragTrail.slice(1) as sq (sq)}
        {@const file = sq & 7}
        {@const rank = (sq >> 3) & 7}
        <rect
          x={file * SIZE + 4}
          y={(7 - rank) * SIZE + 4}
          width={SIZE - 8}
          height={SIZE - 8}
          fill="var(--accent, #c79b3a)"
          fill-opacity="0.10"
          pointer-events="none"
        />
      {/each}
    </g>
  {/if}

  <!-- Drag-hover ring — the square currently under the cursor, sized so it
       stands above the move-target dots/rings. Green if legal drop, red if not. -->
  {#if isDragging && dragHover !== null}
    {@const file = dragHover & 7}
    {@const rank = (dragHover >> 3) & 7}
    <rect
      x={file * SIZE + 1}
      y={(7 - rank) * SIZE + 1}
      width={SIZE - 2}
      height={SIZE - 2}
      fill="none"
      stroke={dragHoverLegal ? "#3a6a4a" : "#a94b3b"}
      stroke-width="4"
      stroke-dasharray="6 4"
      pointer-events="none"
    />
  {/if}

  <!-- Drag landing marker — the square the attacker would actually end up on
       if the drag were released right now. Distinct from `dragHover` because
       on Move-Attack the attacker stops one tile short of the defender. -->
  {#if isDragging && dragLanding !== null && dragLanding !== dragHover}
    {@const file = dragLanding & 7}
    {@const rank = (dragLanding >> 3) & 7}
    {@const cx = file * SIZE + SIZE / 2}
    {@const cy = (7 - rank) * SIZE + SIZE / 2}
    <g pointer-events="none">
      <rect
        x={file * SIZE + 1}
        y={(7 - rank) * SIZE + 1}
        width={SIZE - 2}
        height={SIZE - 2}
        fill="none"
        stroke="#3a6a4a"
        stroke-width="3"
        stroke-dasharray="3 3"
      />
      <!-- Footprint crosshair so the marker reads as "land here", not "target here". -->
      <circle
        {cx}
        {cy}
        r={SIZE * 0.22}
        fill="#3a6a4a"
        fill-opacity="0.18"
      />
      <path
        d="M {cx - SIZE * 0.14} {cy} L {cx + SIZE * 0.14} {cy} M {cx} {cy - SIZE * 0.14} L {cx} {cy + SIZE * 0.14}"
        stroke="#3a6a4a"
        stroke-width="2.4"
        stroke-linecap="round"
      />
    </g>
  {/if}

  <!-- File labels along the bottom edge -->
  <g class="labels" font-size={SIZE * 0.22} fill="var(--paper-ink-soft, #6a6055)">
    {#each fileLabels as letter, i}
      <text
        x={i * SIZE + SIZE / 2}
        y={viewBox + 18}
        text-anchor="middle"
        dominant-baseline="alphabetic"
      >{letter}</text>
    {/each}
  </g>

  <!-- Rank labels inside the left edge of each row -->
  <g class="labels" font-size={SIZE * 0.18} fill="var(--paper-ink-soft, #6a6055)">
    {#each Array.from({ length: 8 }) as _, r}
      <text
        x={4}
        y={(7 - r) * SIZE + SIZE * 0.22}
        text-anchor="start"
        dominant-baseline="hanging"
      >{r + 1}</text>
    {/each}
  </g>

  <!-- Approach-square chooser highlights. Drawn beneath pieces so the
       defender at `target` still reads cleanly. -->
  {#if approachChoices.length > 0}
    <g class="approach-choices">
      {#each approachChoices as ap (ap)}
        {@const file = ap & 7}
        {@const rank = (ap >> 3) & 7}
        {@const x = file * SIZE}
        {@const y = (7 - rank) * SIZE}
        {@const hovered = approachHovered === ap}
        <rect
          {x}
          {y}
          width={SIZE}
          height={SIZE}
          fill="var(--accent, #c79b3a)"
          fill-opacity={hovered ? "0.42" : "0.22"}
          stroke="var(--accent, #c79b3a)"
          stroke-width={hovered ? "4" : "3"}
          pointer-events="none"
        />
        <circle
          cx={x + SIZE / 2}
          cy={y + SIZE / 2}
          r={SIZE * (hovered ? 0.24 : 0.18)}
          fill="var(--accent, #c79b3a)"
          fill-opacity={hovered ? "0.8" : "0.55"}
          pointer-events="none"
        />
      {/each}
    </g>
  {/if}

  <!-- Bodyguard chooser highlights. Defender tile gets a red "take the hit"
       ring; eligible Guards get a blue "intercept" ring. -->
  {#if bodyguardChoice}
    <g class="bodyguard-choices">
      <rect
        x={(bodyguardChoice.defender & 7) * SIZE}
        y={(7 - ((bodyguardChoice.defender >> 3) & 7)) * SIZE}
        width={SIZE}
        height={SIZE}
        fill="#cc3a2a"
        fill-opacity="0.18"
        stroke="#cc3a2a"
        stroke-width="3"
        pointer-events="none"
      >
        <animate
          attributeName="fill-opacity"
          values="0.12;0.32;0.12"
          dur="1.4s"
          repeatCount="indefinite"
        />
      </rect>
      {#each bodyguardChoice.guards as gSq (gSq)}
        <rect
          x={(gSq & 7) * SIZE}
          y={(7 - ((gSq >> 3) & 7)) * SIZE}
          width={SIZE}
          height={SIZE}
          fill="#3a7acc"
          fill-opacity="0.18"
          stroke="#3a7acc"
          stroke-width="3"
          pointer-events="none"
        >
          <animate
            attributeName="fill-opacity"
            values="0.12;0.32;0.12"
            dur="1.4s"
            repeatCount="indefinite"
          />
        </rect>
      {/each}
    </g>
  {/if}

  <!-- Pieces -->
  <g class="pieces">
    {#each pieces as piece (pieceIds.get(piece.square) ?? `sq-${piece.square}`)}
      <Piece
        {piece}
        size={SIZE}
        used={usedSquares.has(piece.square)}
        overrideXY={overrideForPiece(piece.square)}
        shake={shakingSquares.has(piece.square)}
        lunge={lungeSquares.get(piece.square) ?? null}
        {effectsActive}
        dormant={position
          ? (piece.owner === "p1" ? 0 : 1) !== position.toMove
          : false}
      />
    {/each}
  </g>

  <!-- Hit-test overlay — invisible rects on top to catch pointer events.
       Pointer-down on a selectable square begins a drag; up routes to drop
       (if the cursor crossed squares) or click (if it stayed put). -->
  <g class="hits" onpointerleave={() => { approachHovered = null; }}>
    {#each squares as { sq, x, y } (sq)}
      {@const isMoveTarget = moveTargets.has(sq)}
      {@const isSelectable = selectable.has(sq)}
      {@const isSelected = selection === sq}
      {@const isApproach = approachChoices.includes(sq)}
      {@const isHot = interactive && (isMoveTarget || isSelectable || isSelected || isApproach)}
      <rect
        {x}
        {y}
        width={SIZE}
        height={SIZE}
        fill="transparent"
        role="button"
        tabindex="-1"
        aria-label={`square ${sq}`}
        class:hot={isHot}
        class:grab={interactive && draggable.has(sq) && !usedSquares.has(sq)}
        onpointermove={() => { if (approachChoices.length > 0 && approachChoices.includes(sq)) approachHovered = sq; else if (approachChoices.length > 0) approachHovered = null; }}
        onpointerdown={(e) => handleSquarePointerDown(sq, e)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const file = sq & 7;
            const rank = (sq >> 3) & 7;
            const cx = file * SIZE + SIZE / 2;
            const cy = (7 - rank) * SIZE + SIZE / 2;
            handleSquareClickInternal(sq, cx, cy);
          }
        }}
      />
    {/each}
  </g>

  <!-- Radial skill wheel. Rendered last so its slices paint on top of the
       hit-test overlay and absorb pointer events before they reach the
       underlying square rects. -->
  {#if wheelOpen}
    {@const wFile = wheelOpen.square & 7}
    {@const wRank = (wheelOpen.square >> 3) & 7}
    {@const wX = wFile * SIZE}
    {@const wY = (7 - wRank) * SIZE}
    <g transform="translate({wX}, {wY})">
      <SkillWheel
        size={SIZE}
        skill1={wheelOpen.skill1}
        skill2={wheelOpen.skill2}
        {armedSkillId}
        {focusActive}
        {chargeActive}
        skill1Legal={wheelLegality.skill1Legal}
        skill2Legal={wheelLegality.skill2Legal}
        endPhaseLegal={wheelLegality.endPhaseLegal}
        onSliceClick={(s) => onWheelSliceClick?.(s)}
        onSliceHover={(s) => onWheelSliceHover?.(s)}
      />
    </g>
  {/if}

  <!-- Shove direction picker. Rendered last so its arrows paint above pieces
       and squares; absorbs pointer events on its backdrop + arrows. -->
  {#if directionPicker}
    <DirectionPicker
      size={SIZE}
      target={directionPicker.target}
      variants={directionPicker.variants}
      onPick={(raw) => onDirectionPick?.(raw)}
      onCancel={() => onDirectionCancel?.()}
    />
  {/if}
</svg>

<style>
  .board {
    width: 100%;
    height: auto;
    display: block;
    user-select: none;
    -webkit-tap-highlight-color: transparent;
    touch-action: none;
  }
  .board :global(*) {
    -webkit-tap-highlight-color: transparent;
  }
  .board :global(*:focus),
  .board :global(*:focus-visible) {
    outline: none;
  }
  .hits rect { cursor: default; }
  .board.interactive .hits rect.hot { cursor: pointer; }
  .board.interactive .hits rect.grab { cursor: grab; }
  .board.interactive .hits rect.grab:active { cursor: grabbing; }
</style>
