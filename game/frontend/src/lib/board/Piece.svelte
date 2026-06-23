<script lang="ts">
  import type { BoardPiece } from "$lib/engine/mailbox";
  import { SKILLS, skillColor } from "$lib/engine/skills";

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
  }

  let {
    piece,
    size,
    used = false,
    overrideXY = null,
    animate = true,
    shake = false,
  }: Props = $props();

  const file = $derived(piece.square & 7);
  const rank = $derived((piece.square >> 3) & 7);
  const baseX = $derived(file * size);
  const baseY = $derived((7 - rank) * size);
  const x = $derived(overrideXY ? overrideXY.x : baseX);
  const y = $derived(overrideXY ? overrideXY.y : baseY);
  const transition = $derived(
    overrideXY || !animate
      ? "none"
      : "transform 280ms cubic-bezier(0.3, 0.7, 0.3, 1), opacity 240ms ease, filter 240ms ease",
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
</script>

<g
  class="piece"
  class:p1={piece.owner === "p1"}
  class:p2={piece.owner === "p2"}
  class:used
  class:dragging={overrideXY !== null}
  class:shake
  style:transform="translate({x}px, {y}px)"
  style:transition
  data-square={piece.square}
>
  <title>
    {piece.owner.toUpperCase()} {piece.kind} · HP {piece.hp}/2 · Armor {piece.armor}/2 · Combo {piece.combo}{skillNames.length
      ? "\n" + skillNames.join(", ")
      : ""}
  </title>

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
  .piece.shake :global(> *:not(.combo)) {
    animation: piece-shake 320ms ease-out;
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
</style>
