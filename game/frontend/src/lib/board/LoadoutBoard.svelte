<script lang="ts">
  // Mini board for the loadout editor + draft-screen "what's drafted so far"
  // preview. Renders a small 8x8 SVG with one side's King + 5 Champions on
  // their Stack M starting squares plus 3 skill-less Guards on the adjacent
  // rank. Guards are visible but greyed and non-interactive — they're chassis
  // context, not skill targets.
  //
  // Each King/Champion square shows the two currently-assigned skill glyphs
  // stacked; the "selected" piece gets a highlight ring. Clicking a piece
  // fires onPieceClick(pieceIdx) — index 0..5 into a SideLoadout.
  //
  // Consumers:
  //   - /loadouts/ editor: interactive=true, mutable loadout, selectedPieceIdx
  //     drives the SkillPicker context.
  //   - /draft/ screen (8i): interactive=false, mailbox-derived loadout so the
  //     board mirrors what the player has drafted so far.

  import type { Owner } from "$lib/engine";
  import type { SideLoadout } from "$lib/engine";
  import { STACK_M_LOADOUT_SQUARES } from "$lib/state/draft";
  import { skillColor, SKILLS } from "$lib/engine";
  import { t } from "$lib/state/i18n";

  interface Props {
    /** Which side's starting squares to draw. Loadout data itself is side-
     *  agnostic — this only controls board orientation and piece placement. */
    side: Owner;
    /** The 6 skill pairs to display, in piece order (King @ 0, Champions 1..5).
     *  Slot value 0 renders as an empty placeholder. */
    loadout: SideLoadout;
    /** Whether clicking a piece is allowed. False = read-only preview. */
    interactive: boolean;
    /** Which piece is currently being edited (highlight ring). Null = none. */
    selectedPieceIdx: number | null;
    /** Fired on interactive click. pieceIdx is 0..5 into the loadout. */
    onPieceClick?: (pieceIdx: number) => void;
  }

  let {
    side,
    loadout,
    interactive,
    selectedPieceIdx,
    onPieceClick,
  }: Props = $props();

  // SVG geometry. Kept small (~208px total) so two boards fit side-by-side in
  // narrow panels; the container can scale it up with a CSS width if needed.
  const SIZE = 26;                       // pixel edge per square
  const BOARD = SIZE * 8;                // full board pixel size

  // Convert an engine square (0..63, file = sq & 7, rank = sq >> 3) into SVG
  // coordinates. P1 is drawn with rank 1 at the bottom (SVG y grows downward,
  // so we flip). Viewing "as P2" mirrors the board vertically so the P2 back
  // rank sits at the bottom of the mini-board — that's the frame the user
  // thinks in when authoring "P2's loadout".
  function sqToXY(sq: number): { x: number; y: number } {
    const file = sq & 7;
    const rank = (sq >> 3) & 7;
    const x = file * SIZE;
    // For "p1" view: rank 0 at bottom → y = (7 - rank) * SIZE.
    // For "p2" view: flip vertically so their back rank (7) is at bottom.
    const y = side === "p1" ? (7 - rank) * SIZE : rank * SIZE;
    return { x, y };
  }

  // Piece squares for the active side. Order matches the SideLoadout indices:
  // 0 = King, 1..5 = Champions.
  const pieceSquares = $derived(STACK_M_LOADOUT_SQUARES[side]);

  // Guard squares: rank 2 (P1) / rank 7 (P2), files b..g = files 1..6.
  const guardSquares = $derived(
    Array.from({ length: 6 }, (_, f) =>
      side === "p1" ? 8 + (f + 1) : 48 + (f + 1),
    ),
  );

  function pieceLabel(idx: number): string {
    return idx === 0 ? "K" : `C${idx}`;
  }

  function skillTitle(id: number): string {
    if (id === 0) return "—";
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.name`) : `?${id}`;
  }
</script>

<svg
  class="mini-board"
  viewBox="0 0 {BOARD} {BOARD}"
  width={BOARD}
  height={BOARD}
  aria-label="loadout mini board"
>
  <!-- Checkerboard squares -->
  {#each Array(64) as _, sq}
    {@const { x, y } = sqToXY(sq)}
    {@const file = sq & 7}
    {@const rank = (sq >> 3) & 7}
    {@const dark = (file + rank) % 2 === 0}
    <rect
      x={x}
      y={y}
      width={SIZE}
      height={SIZE}
      fill={dark ? "var(--paper-line, #d8d0be)" : "var(--paper-bg, #f6f1e0)"}
      stroke="var(--paper-line-strong, #8a7a52)"
      stroke-width="0.4"
    />
  {/each}

  <!-- Guards: rank 2 / rank 7 files b..g. Greyed, non-interactive. -->
  {#each guardSquares as sq}
    {@const { x, y } = sqToXY(sq)}
    <g class="guard" aria-hidden="true">
      <circle
        cx={x + SIZE / 2}
        cy={y + SIZE / 2}
        r={SIZE * 0.28}
        fill="var(--paper-bg, #f6f1e0)"
        stroke="var(--paper-line-strong, #8a7a52)"
        stroke-width="1"
        opacity="0.55"
      />
      <text
        x={x + SIZE / 2}
        y={y + SIZE / 2 + 2}
        text-anchor="middle"
        font-size="7"
        fill="var(--paper-ink, #333)"
        opacity="0.55"
      >
        G
      </text>
    </g>
  {/each}

  <!-- King + Champions: interactive, show skill glyphs. -->
  {#each pieceSquares as sq, idx (sq)}
    {@const { x, y } = sqToXY(sq)}
    {@const [s1, s2] = loadout[idx]}
    {@const isSelected = selectedPieceIdx === idx}
    {@const isKing = idx === 0}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <g
      class="piece"
      class:selected={isSelected}
      class:interactive
      role={interactive ? "button" : undefined}
      tabindex={interactive ? 0 : undefined}
      onclick={interactive ? () => onPieceClick?.(idx) : undefined}
      onkeydown={interactive
        ? (ev) => {
            if (ev.key === "Enter" || ev.key === " ") {
              ev.preventDefault();
              onPieceClick?.(idx);
            }
          }
        : undefined}
    >
      <!-- Piece background: highlight ring if selected. -->
      <rect
        x={x + 1}
        y={y + 1}
        width={SIZE - 2}
        height={SIZE - 2}
        rx="3"
        fill={isKing ? "color-mix(in srgb, gold 20%, var(--paper-bg, #f6f1e0))" : "var(--paper-bg, #f6f1e0)"}
        stroke={isSelected ? "var(--accent, #c94)" : "var(--paper-ink, #333)"}
        stroke-width={isSelected ? 2.2 : 0.9}
      />
      <!-- Two skill glyphs stacked (slot 1 top-left, slot 2 bottom-right). -->
      {#if s1 !== 0}
        <svg
          x={x + 2}
          y={y + 2}
          width={SIZE * 0.45}
          height={SIZE * 0.45}
          viewBox="0 0 24 24"
          color={skillColor(s1)}
        >
          <title>{skillTitle(s1)}</title>
          <use href="#skill-glyph-{s1}" />
        </svg>
      {:else}
        <text
          x={x + 5}
          y={y + 9}
          font-size="7"
          fill="var(--paper-line-strong, #8a7a52)"
        >?</text>
      {/if}
      {#if s2 !== 0}
        <svg
          x={x + SIZE - SIZE * 0.45 - 2}
          y={y + SIZE - SIZE * 0.45 - 2}
          width={SIZE * 0.45}
          height={SIZE * 0.45}
          viewBox="0 0 24 24"
          color={skillColor(s2)}
        >
          <title>{skillTitle(s2)}</title>
          <use href="#skill-glyph-{s2}" />
        </svg>
      {:else}
        <text
          x={x + SIZE - 8}
          y={y + SIZE - 3}
          font-size="7"
          fill="var(--paper-line-strong, #8a7a52)"
        >?</text>
      {/if}
      <!-- Piece letter (K / C1..C5) top-right for context. -->
      <text
        x={x + SIZE - 2}
        y={y + SIZE * 0.45}
        text-anchor="end"
        font-size="6.5"
        fill="var(--paper-line-strong, #8a7a52)"
        aria-hidden="true"
      >{pieceLabel(idx)}</text>
    </g>
  {/each}
</svg>

<style>
  .mini-board {
    display: block;
    max-width: 100%;
    height: auto;
    font-family: inherit;
  }
  .piece.interactive {
    cursor: pointer;
  }
  .piece.interactive:hover rect {
    filter: brightness(1.05);
  }
  .piece.interactive:focus {
    outline: none;
  }
  .piece.interactive:focus-visible rect {
    stroke: var(--accent, #c94);
    stroke-width: 2.2;
  }
</style>
