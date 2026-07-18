<script lang="ts">
  // Direction picker for Shove (skill 11). After the player has armed Shove
  // and clicked a target tile, this component renders 8 arrows around the
  // target - one per cardinal/diagonal direction - and the player picks the
  // push direction. Only directions backed by a legal action variant render
  // as enabled (the rest are dimmed but still visible so the user can see
  // which pushes the rules forbid).
  //
  // Rendered as a child of Board.svelte's <svg> so it shares the same
  // coordinate space. Position is computed from the target square index.

  import type { SkillVariant } from "$lib/state/skill-targets";

  interface Props {
    /** SVG square edge in pixels (Board's SIZE). */
    size: number;
    /** The target square the player clicked (file/rank packed: rank*8 + file). */
    target: number;
    /** All legal variants for the armed (src, skill) at this target. Direction
     * comes from `choiceIdx` (0..=7 = N, NE, E, SE, S, SW, W, NW). */
    variants: SkillVariant[];
    /** Click handler - passes the chosen raw u32 action. */
    onPick?: (raw: number) => void;
    /** Cancel handler - clicking the centre tile or any non-arrow region. */
    onCancel?: () => void;
  }

  let { size, target, variants, onPick, onCancel }: Props = $props();

  // Engine direction order: 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW.
  // (Matches core_engine::magic::neighbour_in_dir.)
  // Note: rank increases upward on the board (P1 home = rank 0 = bottom of
  // the SVG). The SVG y-axis points DOWN, so N (rank+1) is y - SIZE in SVG.
  type DirInfo = { dx: number; dy: number; label: string };
  const DIRS: DirInfo[] = [
    { dx:  0, dy: -1, label: "N"  }, // 0
    { dx:  1, dy: -1, label: "NE" }, // 1
    { dx:  1, dy:  0, label: "E"  }, // 2
    { dx:  1, dy:  1, label: "SE" }, // 3
    { dx:  0, dy:  1, label: "S"  }, // 4
    { dx: -1, dy:  1, label: "SW" }, // 5
    { dx: -1, dy:  0, label: "W"  }, // 6
    { dx: -1, dy: -1, label: "NW" }, // 7
  ];

  const tFile = $derived(target & 7);
  const tRank = $derived((target >> 3) & 7);
  // Target tile's top-left in SVG coords.
  const tx = $derived(tFile * size);
  const ty = $derived((7 - tRank) * size);
  // Target tile centre.
  const cx = $derived(tx + size / 2);
  const cy = $derived(ty + size / 2);

  // Variants keyed by direction. There should be at most one variant per
  // (target, direction); if Focus introduces a focusMode duplicate we just
  // pick the first match (parent gates by focusMode before rendering us).
  const byDir = $derived.by(() => {
    const m = new Map<number, SkillVariant>();
    for (const v of variants) {
      if (!m.has(v.choiceIdx)) m.set(v.choiceIdx, v);
    }
    return m;
  });

  /** Push distance per direction, in tiles. `focus_mode=1` Shove variants
   *  push 2 tiles instead of 1; the generator only emits those when both
   *  intermediate and final squares are empty + on-board, so an enabled
   *  arrow in effect-mode IS a legal 2-tile push. Variants without
   *  focus_mode=1 always push 1 (default Shove + activation-buff Shove). */
  function pushDistance(v: SkillVariant): number {
    return v.focusMode ? 2 : 1;
  }

  const ARROW_HEAD = $derived(size * 0.18);
  const HIT_R = $derived(size * 0.32);
</script>

<g class="direction-picker" pointer-events="auto">
  <!-- Backdrop: absorbs clicks outside the arrows (= cancel). Sized to
       comfortably enclose 2-tile arrows for focus-effect Shove. -->
  <rect
    x={cx - size * 2.4}
    y={cy - size * 2.4}
    width={size * 4.8}
    height={size * 4.8}
    fill="rgba(0, 0, 0, 0.08)"
    role="presentation"
    onpointerdown={(ev) => {
      // Only cancel on direct backdrop hits (not bubbled from an arrow).
      if (ev.target === ev.currentTarget) {
        ev.stopPropagation();
        onCancel?.();
      }
    }}
  />

  <!-- Centre marker on the target tile so the player sees the anchor. -->
  <circle
    {cx}
    {cy}
    r={size * 0.14}
    fill="var(--accent, #c79b3a)"
    fill-opacity="0.6"
    stroke="var(--accent, #c79b3a)"
    stroke-width="2"
    pointer-events="none"
  />

  <!-- 8 direction arrows. Each is a line + head + invisible hit-circle. -->
  {#each DIRS as dir, i (i)}
    {@const variant = byDir.get(i)}
    {@const legal = variant !== undefined}
    <!-- Arrow length scales with the variant's push distance: 1 tile by
         default, 2 tiles for focus-effect Shove. Illegal directions render
         at 1-tile length so the dimmed silhouette still reads consistently. -->
    {@const dist = variant ? pushDistance(variant) : 1}
    {@const len = size * (0.55 + dist * 0.7)}
    <!-- For diagonals, scale so arrow tips form a circle (not a square). -->
    {@const norm = Math.sqrt(dir.dx * dir.dx + dir.dy * dir.dy)}
    {@const ux = dir.dx / norm}
    {@const uy = dir.dy / norm}
    {@const tipX = cx + ux * len}
    {@const tipY = cy + uy * len}
    {@const tailX = cx + ux * size * 0.32}
    {@const tailY = cy + uy * size * 0.32}
    <!-- Perpendicular for the arrowhead wings. -->
    {@const px = -uy}
    {@const py =  ux}
    {@const headBaseX = tipX - ux * ARROW_HEAD}
    {@const headBaseY = tipY - uy * ARROW_HEAD}
    {@const wing1X = headBaseX + px * ARROW_HEAD * 0.6}
    {@const wing1Y = headBaseY + py * ARROW_HEAD * 0.6}
    {@const wing2X = headBaseX - px * ARROW_HEAD * 0.6}
    {@const wing2Y = headBaseY - py * ARROW_HEAD * 0.6}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <g
      class="dir"
      class:legal
      class:illegal={!legal}
      role={legal ? "button" : "presentation"}
      aria-label={legal ? `push ${dir.label}` : undefined}
      tabindex={legal ? 0 : undefined}
      onpointerdown={(ev) => {
        if (!legal || !variant) return;
        ev.stopPropagation();
        onPick?.(variant.raw);
      }}
      onkeydown={(ev) => {
        if (!legal || !variant) return;
        if (ev.key === "Enter" || ev.key === " ") {
          ev.preventDefault();
          onPick?.(variant.raw);
        }
      }}
    >
      <!-- Shaft + head. Stroke colour from class. -->
      <line x1={tailX} y1={tailY} x2={headBaseX} y2={headBaseY} class="shaft" />
      <path
        d="M {tipX} {tipY} L {wing1X} {wing1Y} L {wing2X} {wing2Y} Z"
        class="head"
      />
      <!-- Invisible hit-region sized for fingers, centred on the arrowhead. -->
      <circle cx={tipX} cy={tipY} r={HIT_R} class="hit" />
    </g>
  {/each}
</g>

<style>
  .direction-picker { animation: fade-in 120ms ease-out; }
  @keyframes fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
  .dir .shaft {
    stroke-width: 4.5;
    stroke-linecap: round;
    fill: none;
  }
  .dir .head { stroke-linejoin: round; }
  .dir.legal .shaft,
  .dir.legal .head {
    stroke: var(--accent, #c79b3a);
    fill: var(--accent, #c79b3a);
  }
  .dir.legal .head { stroke-width: 2; }
  .dir.legal .hit { fill: transparent; cursor: pointer; }
  .dir.legal:hover .shaft,
  .dir.legal:hover .head {
    stroke: #a37416;
    fill: #a37416;
  }
  .dir.legal:hover .head {
    transform-box: fill-box;
    transform-origin: center;
  }
  .dir.illegal .shaft,
  .dir.illegal .head {
    stroke: var(--paper-ink-soft, #6a6055);
    fill: var(--paper-ink-soft, #6a6055);
    opacity: 0.25;
  }
  .dir.illegal .hit { fill: transparent; pointer-events: none; }
</style>
