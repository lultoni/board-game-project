<script lang="ts">
  import { SKILLS, skillColor, type BoardPiece } from "$lib/engine";
  import { settings, SLIDE_DURATION_MS } from "$lib/state/settings.svelte";

  interface Props {
    piece: BoardPiece;
    /** Square pixel size (e.g. 80 for an 800px board). */
    size: number;
    /** Piece has used its Move action this Move Phase — render dim/grey. */
    used?: boolean;
    /** When set, render the piece at this SVG (x,y) instead of its square,
     * skipping the slide-tween. Used during pointer-drag. */
    overrideXY?: { x: number; y: number } | null;
    /** When true (default) the piece slides smoothly to its square. Skipped
     * for the initial render and during drag. */
    animate?: boolean;
    /** Run a brief "hit" shake animation. Toggled by the parent for ~320ms
     * after this piece was damaged. */
    shake?: boolean;
    /** Pause idle breathing while effects are mid-play (keeps eyes on action). */
    effectsActive?: boolean;
    /** When true, this piece belongs to the side whose turn is NOT current —
     *  suppress idle breathing so the active side reads as "alive". */
    dormant?: boolean;
    /** When set, play a lunge-and-recoil animation by (dx, dy) SVG pixels.
     *  Used for non-kill attacks: attacker lives at auxSq but visually lunges
     *  toward the target and bounces back. */
    lunge?: { dx: number; dy: number } | null;
  }

  let {
    piece,
    size,
    used = false,
    overrideXY = null,
    animate = true,
    shake = false,
    effectsActive = false,
    dormant = false,
    lunge = null,
  }: Props = $props();

  const file = $derived(piece.square & 7);
  const rank = $derived((piece.square >> 3) & 7);
  const baseX = $derived(file * size);
  const baseY = $derived((7 - rank) * size);
  const x = $derived(overrideXY ? overrideXY.x : baseX);
  const y = $derived(overrideXY ? overrideXY.y : baseY);
  const slideDur = $derived(SLIDE_DURATION_MS[settings.animationSpeed]);
  const transition = $derived(
    overrideXY || !animate || slideDur === 0
      ? "none"
      : `transform ${slideDur}ms cubic-bezier(0.3, 0.7, 0.3, 1), opacity ${Math.round(slideDur * 0.86)}ms ease, filter ${Math.round(slideDur * 0.86)}ms ease`,
  );
  // Lunge-recoil animation: piece at its resting square lunges toward the
  // attacked square and snaps back. Duration = one full slide cycle.
  const lungeStyle = $derived(
    lunge && slideDur > 0
      ? `--lunge-dx:${lunge.dx}px;--lunge-dy:${lunge.dy}px;animation:piece-lunge ${slideDur * 2}ms cubic-bezier(0.25,0.46,0.45,0.94) forwards`
      : "",
  );

  const ownerColor = $derived(
    piece.owner === "p1" ? "var(--p1, #2b4a8a)" : "var(--p2, #a13a2a)",
  );
  const ink = "var(--paper-ink, #1c1a17)";

  // Frame geometry inside a unit square. Pieces with skills (King, Champion)
  // get the full skill-bearing frame. Guards get a compact tile because they
  // carry no skills (Stack M).
  const hasSkills = $derived(piece.kind !== "guard");

  // Frame margins (fraction of `size`).
  // Vertical: leave ~14% above for armor pips, ~14% below for HP pips/combo.
  // Horizontal: ~16% each side for breathing room.
  const FRAME_LEFT = 0.16;
  const FRAME_RIGHT = 0.84;
  const FRAME_TOP = 0.18;
  const FRAME_BOTTOM = 0.78;
  const FRAME_W = FRAME_RIGHT - FRAME_LEFT;
  const FRAME_H = FRAME_BOTTOM - FRAME_TOP;

  const fx = $derived(size * FRAME_LEFT);
  const fy = $derived(size * FRAME_TOP);
  const fw = $derived(size * FRAME_W);
  const fh = $derived(size * FRAME_H);

  // For Kings, the top edge of the frame gets two crown notches.
  // For Champions, the top edge is gently pointed.
  // For Guards: small inset rectangle, no glyphs.

  // Skill glyph slots: two stacked rows inside the frame, equal height.
  const slotH = $derived(fh / 2);
  const slotSize = $derived(Math.min(slotH, fw) * 0.78);
  const slot1X = $derived(fx + fw / 2 - slotSize / 2);
  const slot1Y = $derived(fy + slotH / 2 - slotSize / 2);
  const slot2X = $derived(fx + fw / 2 - slotSize / 2);
  const slot2Y = $derived(fy + slotH + slotH / 2 - slotSize / 2);

  // Pip row: both HP and armor sit on the piece's bottom edge.
  // Armor occupies the left half of the bottom edge, HP the right half.
  // Each pair is centred within its half.
  const pipBaseY = $derived(fy + fh); // bottom edge of the frame
  const halfW = $derived(fw / 2);

  // HP dots
  const hpR = $derived(size * 0.05);
  const hpGap = $derived(hpR * 2 + size * 0.04);
  const hpCenterX = $derived(fx + fw * 0.75); // centre of right half
  const hpXs = $derived([
    hpCenterX - hpGap / 2,
    hpCenterX + hpGap / 2,
  ]);

  // Armor squares — sized to match HP dot diameter so the two groups read
  // as one row.
  const armSide = $derived(size * 0.085);
  const armGap = $derived(armSide + size * 0.04);
  const armCenterX = $derived(fx + fw * 0.25); // centre of left half
  const armXs = $derived([
    armCenterX - armGap / 2 - armSide / 2,
    armCenterX + armGap / 2 - armSide / 2,
  ]);

  // Combo badge: hugs the right side of the frame, just above the pip row.
  const comboCX = $derived(fx + fw + size * 0.05);
  const comboCY = $derived(fy + fh - size * 0.15);

  const skillNames = $derived(
    [piece.skill1, piece.skill2]
      .filter((id) => id > 0)
      .map((id) => SKILLS[id]?.key ?? `?${id}`),
  );

  // Stagger the idle-breathe animation by a deterministic offset derived from
  // the piece's square so neighbours don't pulse in lockstep. 3.6s loop.
  const breatheDelay = $derived(-((piece.square * 137) % 3600) + "ms");
  // Centre of the piece's local square coords. Drives the SVG transform-origin
  // for the breathing animation so the whole body scales around its centre.
  const centerX = $derived(size / 2);
  const centerY = $derived(size / 2);
</script>

<g
  class="piece"
  class:p1={piece.owner === "p1"}
  class:p2={piece.owner === "p2"}
  class:used
  class:dragging={overrideXY !== null}
  class:shake
  class:effects-active={effectsActive}
  class:dormant
  style:transform="translate({x}px, {y}px)"
  style:transition
  style:--breathe-delay={breatheDelay}
  data-square={piece.square}
>
  <title>
    {piece.owner.toUpperCase()} {piece.kind} · HP {piece.hp}/2 · Armor {piece.armor}/2 · Combo {piece.combo}{skillNames.length
      ? "\n" + skillNames.join(", ")
      : ""}
  </title>

  <!-- Lunge wrapper: applies lunge-recoil animation independently of the
       positional slide on the outer <g>. No-op when lungeStyle is empty. -->
  <g class="lunge-wrap" style={lungeStyle}>

  <g
    class="body"
    style:transform-origin="{centerX}px {centerY}px"
  >

  {#if piece.kind === "king"}
    <!--
      King frame: skill-bearing body with crown notches along the top edge.
      Built as a single closed path so stroke and fill are coherent.
    -->
    <path
      d="
        M {fx} {fy + size * 0.04}
        L {fx} {fy + fh}
        L {fx + fw} {fy + fh}
        L {fx + fw} {fy + size * 0.04}
        L {fx + fw - size * 0.05} {fy + size * 0.04}
        L {fx + fw - size * 0.08} {fy - size * 0.04}
        L {fx + fw - size * 0.16} {fy + size * 0.04}
        L {fx + fw / 2 + size * 0.04} {fy + size * 0.04}
        L {fx + fw / 2} {fy - size * 0.06}
        L {fx + fw / 2 - size * 0.04} {fy + size * 0.04}
        L {fx + size * 0.16} {fy + size * 0.04}
        L {fx + size * 0.08} {fy - size * 0.04}
        L {fx + size * 0.05} {fy + size * 0.04}
        Z
      "
      fill={ownerColor}
      stroke={ink}
      stroke-width="2.4"
      stroke-linejoin="round"
    />
  {:else if piece.kind === "champion"}
    <!-- Champion frame: skill-bearing body with a pointed top edge. -->
    <path
      d="
        M {fx} {fy + size * 0.06}
        L {fx} {fy + fh}
        L {fx + fw} {fy + fh}
        L {fx + fw} {fy + size * 0.06}
        L {fx + fw / 2} {fy - size * 0.04}
        Z
      "
      fill={ownerColor}
      stroke={ink}
      stroke-width="2.4"
      stroke-linejoin="round"
    />
  {:else}
    <!-- Guard: compact tile, no skills. Smaller frame. -->
    <rect
      x={fx + size * 0.05}
      y={fy + size * 0.05}
      width={fw - size * 0.1}
      height={fh - size * 0.05}
      rx={size * 0.06}
      ry={size * 0.06}
      fill={ownerColor}
      stroke={ink}
      stroke-width="2.2"
    />
  {/if}

  {#if hasSkills}
    <!-- Slot dividers: thin line halving the interior -->
    <line
      x1={fx + size * 0.02}
      x2={fx + fw - size * 0.02}
      y1={fy + slotH}
      y2={fy + slotH}
      stroke={ink}
      stroke-width="0.8"
      stroke-opacity="0.4"
    />

    <!-- Skill glyphs (slot 1 top, slot 2 bottom). The glyph defs live in
         the parent SVG as <symbol id="skill-glyph-N">. Each glyph is drawn
         twice: a wider white pass underneath (paper-cream halo *along the
         strokes only*), then the category-coloured pass on top. -->
    {#if piece.skill1 > 0}
      <use
        href="#skill-glyph-{piece.skill1}"
        x={slot1X}
        y={slot1Y}
        width={slotSize}
        height={slotSize}
        color="#fefcf3"
        stroke-width="6"
      />
      <use
        href="#skill-glyph-{piece.skill1}"
        x={slot1X}
        y={slot1Y}
        width={slotSize}
        height={slotSize}
        color={skillColor(piece.skill1)}
        stroke-width="3"
      />
    {/if}
    {#if piece.skill2 > 0}
      <use
        href="#skill-glyph-{piece.skill2}"
        x={slot2X}
        y={slot2Y}
        width={slotSize}
        height={slotSize}
        color="#fefcf3"
        stroke-width="6"
      />
      <use
        href="#skill-glyph-{piece.skill2}"
        x={slot2X}
        y={slot2Y}
        width={slotSize}
        height={slotSize}
        color={skillColor(piece.skill2)}
        stroke-width="3"
      />
    {/if}
  {/if}

  <!-- HP pips riding the bottom edge of the frame, right half, centred -->
  {#each Array.from({ length: 2 }) as _, i}
    <circle
      cx={hpXs[i]}
      cy={pipBaseY}
      r={hpR}
      fill={i < piece.hp ? "#cc3a2a" : "var(--paper-bg, #f3ecd9)"}
      stroke={ink}
      stroke-width="1.4"
    />
  {/each}

  <!-- Armor pips riding the bottom edge of the frame, left half, centred -->
  {#each Array.from({ length: 2 }) as _, i}
    <rect
      x={armXs[i]}
      y={pipBaseY - armSide / 2}
      width={armSide}
      height={armSide}
      fill={i < piece.armor ? "#9c9486" : "var(--paper-bg, #f3ecd9)"}
      stroke={ink}
      stroke-width="1.4"
    />
  {/each}
  </g>

  <!-- Combo badge, only when > 0 -->
  {#if piece.combo > 0}
    <g class="combo">
      <circle
        cx={comboCX}
        cy={comboCY}
        r={size * 0.1}
        fill="var(--paper-bg, #f3ecd9)"
        stroke={ink}
        stroke-width="1.6"
      />
      <text
        x={comboCX}
        y={comboCY}
        text-anchor="middle"
        dominant-baseline="central"
        font-size={size * 0.14}
        font-weight="700"
        fill={ink}
      >{piece.combo}</text>
    </g>
  {/if}
  </g><!-- /lunge-wrap -->
</g>

<style>
  .piece {
    transform-box: fill-box;
    transform-origin: center;
    will-change: transform, filter, opacity;
  }
  .piece.used {
    filter: grayscale(1) brightness(0.85);
    opacity: 0.55;
  }
  .piece.dragging {
    filter: drop-shadow(0 6px 4px rgba(0, 0, 0, 0.25));
  }
  /* Subtle idle breathing: gentle scale pulse on the body group, around the
     centre of the piece's local square coords (set via inline transform-origin).
     Paused while a hit-shake is mid-flight, while dragging, while a piece is
     "used", and while the parent has signalled that effects are mid-play. */
  .piece .body {
    animation: piece-breathe 3.6s ease-in-out infinite;
    animation-delay: var(--breathe-delay, 0ms);
  }
  .piece.dragging .body,
  .piece.used .body,
  .piece.effects-active .body,
  .piece.dormant .body {
    animation: none;
  }
  .piece.shake .body {
    animation: piece-shake 320ms ease-out;
  }
  @keyframes piece-breathe {
    0%, 100% { transform: scale(1) rotate(0deg); }
    50%      { transform: scale(1.06) rotate(0.6deg); }
  }
  @keyframes piece-shake {
    0%   { transform: translate(0, 0) rotate(0deg); }
    15%  { transform: translate(-3px, 1px) rotate(-2deg); }
    30%  { transform: translate(3px, -1px) rotate(2deg); }
    45%  { transform: translate(-2px, 2px) rotate(-1.5deg); }
    60%  { transform: translate(2px, -2px) rotate(1.5deg); }
    80%  { transform: translate(-1px, 0px) rotate(-0.5deg); }
    100% { transform: translate(0, 0) rotate(0deg); }
  }
  @media (prefers-reduced-motion: reduce) {
    .piece .body {
      animation: none;
    }
  }
  /* Lunge-and-recoil: piece surges toward the attacked square then snaps back.
     Driven by --lunge-dx / --lunge-dy set inline when lunge prop is active.
     40% forward, 100% back — feels like a quick jab. */
  .lunge-wrap {
    /* default: no transform when not lunging */
  }
  @keyframes piece-lunge {
    0%   { transform: translate(0, 0); }
    35%  { transform: translate(var(--lunge-dx, 0px), var(--lunge-dy, 0px)); }
    100% { transform: translate(0, 0); }
  }
</style>
