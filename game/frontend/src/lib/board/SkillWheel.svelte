<script lang="ts">
  // Radial skill wheel that pops around a selected piece. Slices are
  // positioned in *piece-local* coordinates (piece is already translated by
  // its (x,y); the wheel <g> renders inside that same translate, so we work
  // in 0..100 piece-local units regardless of where the piece sits on the
  // board).
  //
  // The two skill slices use the same glyph symbols as the piece itself
  // (#skill-glyph-N), animated outward via a CSS transform on the slice
  // group. Modifier slices and End-Phase use simple text + colour.

  import { skillColor, SKILLS } from "$lib/engine/skills";

  export type SliceKind =
    | { kind: "skill"; skillId: number; slot: 1 | 2 }
    | { kind: "modifier"; modifier: "focus" | "charge" }
    | { kind: "endphase" };

  interface Props {
    /** Tile size in SVG units (matches Board's SIZE). */
    size: number;
    /** Skill IDs from the selected piece's mailbox slots (0 = empty). */
    skill1: number;
    skill2: number;
    /** Which skill (if any) is currently armed. Drives the "armed" glow. */
    armedSkillId: number | null;
    /** Modifier toggles staged for the next cast. */
    focusActive: boolean;
    chargeActive: boolean;
    /** Whether each slice's action is currently legal (greyed out if not). */
    skill1Legal: boolean;
    skill2Legal: boolean;
    focusLegal: boolean;
    chargeLegal: boolean;
    endPhaseLegal: boolean;
    /** Click handler — single sink for every slice. */
    onSliceClick: (slice: SliceKind) => void;
    /** Hover handler. Called with `null` on mouse-leave of all slices. */
    onSliceHover: (slice: SliceKind | null) => void;
  }

  let {
    size,
    skill1,
    skill2,
    armedSkillId,
    focusActive,
    chargeActive,
    skill1Legal,
    skill2Legal,
    focusLegal,
    chargeLegal,
    endPhaseLegal,
    onSliceClick,
    onSliceHover,
  }: Props = $props();

  // Slice positions in piece-local units (piece occupies 0..size).
  // Ring centre = piece centre; ring radius ~= 0.7 * size.
  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  const ring = $derived(size * 0.78);

  // Slice radius (size of each pop-out icon).
  const sliceR = $derived(size * 0.22);

  // Five slice positions around the piece.
  const skill1Pos = $derived({ x: cx, y: cy - ring }); // top
  const skill2Pos = $derived({ x: cx, y: cy + ring }); // bottom
  const focusPos = $derived({ x: cx - ring, y: cy });  // left
  const chargePos = $derived({ x: cx + ring, y: cy }); // right
  const endPos = $derived({ x: cx + ring * 0.7, y: cy + ring * 0.7 }); // bottom-right

  function sliceClasses(legal: boolean, armed: boolean): string {
    return [
      "slice",
      legal ? "" : "disabled",
      armed ? "armed" : "",
    ].filter(Boolean).join(" ");
  }
</script>

<g class="skill-wheel" pointer-events="auto">
  <!-- Faint connecting arcs from piece centre to each slice (hand-drawn paper feel). -->
  {#if skill1 > 0}
    <line
      x1={cx} y1={cy}
      x2={skill1Pos.x} y2={skill1Pos.y}
      stroke="#7a6a4a"
      stroke-width="1.2"
      stroke-opacity="0.35"
      stroke-dasharray="3 3"
      pointer-events="none"
    />
  {/if}
  {#if skill2 > 0}
    <line
      x1={cx} y1={cy}
      x2={skill2Pos.x} y2={skill2Pos.y}
      stroke="#7a6a4a"
      stroke-width="1.2"
      stroke-opacity="0.35"
      stroke-dasharray="3 3"
      pointer-events="none"
    />
  {/if}

  <!-- Skill 1 slice (top) -->
  {#if skill1 > 0}
    {@const armed = armedSkillId === skill1}
    <g
      class={sliceClasses(skill1Legal, armed)}
      transform="translate({skill1Pos.x}, {skill1Pos.y})"
      onpointerdown={(e) => { e.stopPropagation(); if (skill1Legal) onSliceClick({ kind: "skill", skillId: skill1, slot: 1 }); }}
      onpointerenter={() => onSliceHover({ kind: "skill", skillId: skill1, slot: 1 })}
      onpointerleave={() => onSliceHover(null)}
      role="button"
      tabindex="0"
      aria-label={SKILLS[skill1]?.key ?? "skill 1"}
    >
      <circle r={sliceR} fill="#fefcf3" stroke={skillColor(skill1)} stroke-width="2.5" />
      {#if armed}
        <circle r={sliceR + 4} fill="none" stroke={skillColor(skill1)} stroke-width="2" stroke-opacity="0.5">
          <animate attributeName="r" values="{sliceR + 2};{sliceR + 6};{sliceR + 2}" dur="1.2s" repeatCount="indefinite" />
        </circle>
      {/if}
      <use
        href="#skill-glyph-{skill1}"
        x={-sliceR * 0.75}
        y={-sliceR * 0.75}
        width={sliceR * 1.5}
        height={sliceR * 1.5}
        color={skillColor(skill1)}
        stroke-width="2.4"
      />
    </g>
  {/if}

  <!-- Skill 2 slice (bottom) -->
  {#if skill2 > 0}
    {@const armed = armedSkillId === skill2}
    <g
      class={sliceClasses(skill2Legal, armed)}
      transform="translate({skill2Pos.x}, {skill2Pos.y})"
      onpointerdown={(e) => { e.stopPropagation(); if (skill2Legal) onSliceClick({ kind: "skill", skillId: skill2, slot: 2 }); }}
      onpointerenter={() => onSliceHover({ kind: "skill", skillId: skill2, slot: 2 })}
      onpointerleave={() => onSliceHover(null)}
      role="button"
      tabindex="0"
      aria-label={SKILLS[skill2]?.key ?? "skill 2"}
    >
      <circle r={sliceR} fill="#fefcf3" stroke={skillColor(skill2)} stroke-width="2.5" />
      {#if armed}
        <circle r={sliceR + 4} fill="none" stroke={skillColor(skill2)} stroke-width="2" stroke-opacity="0.5">
          <animate attributeName="r" values="{sliceR + 2};{sliceR + 6};{sliceR + 2}" dur="1.2s" repeatCount="indefinite" />
        </circle>
      {/if}
      <use
        href="#skill-glyph-{skill2}"
        x={-sliceR * 0.75}
        y={-sliceR * 0.75}
        width={sliceR * 1.5}
        height={sliceR * 1.5}
        color={skillColor(skill2)}
        stroke-width="2.4"
      />
    </g>
  {/if}

  <!-- Focus slice (left) -->
  <g
    class={sliceClasses(focusLegal, focusActive)}
    transform="translate({focusPos.x}, {focusPos.y})"
    onpointerdown={(e) => { e.stopPropagation(); if (focusLegal) onSliceClick({ kind: "modifier", modifier: "focus" }); }}
    onpointerenter={() => onSliceHover({ kind: "modifier", modifier: "focus" })}
    onpointerleave={() => onSliceHover(null)}
    role="button"
    tabindex="0"
    aria-label="focus"
  >
    <circle r={sliceR} fill="#fefcf3" stroke="#8a4abd" stroke-width="2.5" />
    {#if focusActive}
      <circle r={sliceR + 4} fill="none" stroke="#8a4abd" stroke-width="2" stroke-opacity="0.5">
        <animate attributeName="r" values="{sliceR + 2};{sliceR + 6};{sliceR + 2}" dur="1.2s" repeatCount="indefinite" />
      </circle>
    {/if}
    <text x="0" y="3" text-anchor="middle" font-size={sliceR * 0.9} font-weight="700" fill="#8a4abd">+1</text>
  </g>

  <!-- Charge slice (right) -->
  <g
    class={sliceClasses(chargeLegal, chargeActive)}
    transform="translate({chargePos.x}, {chargePos.y})"
    onpointerdown={(e) => { e.stopPropagation(); if (chargeLegal) onSliceClick({ kind: "modifier", modifier: "charge" }); }}
    onpointerenter={() => onSliceHover({ kind: "modifier", modifier: "charge" })}
    onpointerleave={() => onSliceHover(null)}
    role="button"
    tabindex="0"
    aria-label="charge"
  >
    <circle r={sliceR} fill="#fefcf3" stroke="#8a4abd" stroke-width="2.5" />
    {#if chargeActive}
      <circle r={sliceR + 4} fill="none" stroke="#8a4abd" stroke-width="2" stroke-opacity="0.5">
        <animate attributeName="r" values="{sliceR + 2};{sliceR + 6};{sliceR + 2}" dur="1.2s" repeatCount="indefinite" />
      </circle>
    {/if}
    <text x="0" y="3" text-anchor="middle" font-size={sliceR * 0.7} font-weight="700" fill="#8a4abd">⚡</text>
  </g>

  <!-- End-Phase slice (bottom-right) -->
  <g
    class={sliceClasses(endPhaseLegal, false)}
    transform="translate({endPos.x}, {endPos.y})"
    onpointerdown={(e) => { e.stopPropagation(); if (endPhaseLegal) onSliceClick({ kind: "endphase" }); }}
    onpointerenter={() => onSliceHover({ kind: "endphase" })}
    onpointerleave={() => onSliceHover(null)}
    role="button"
    tabindex="0"
    aria-label="end phase"
  >
    <circle r={sliceR * 0.85} fill="#fefcf3" stroke="#5a4a3a" stroke-width="2.5" />
    <text x="0" y="3" text-anchor="middle" font-size={sliceR * 0.7} font-weight="700" fill="#5a4a3a">⏵</text>
  </g>
</g>

<style>
  .slice {
    cursor: pointer;
    transition: transform 0.12s ease-out;
  }
  .slice:hover {
    transform-box: fill-box;
    transform-origin: center;
  }
  .slice.disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .skill-wheel {
    animation: wheel-pop 220ms cubic-bezier(0.34, 1.56, 0.64, 1);
    transform-origin: center;
  }
  @keyframes wheel-pop {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
</style>
