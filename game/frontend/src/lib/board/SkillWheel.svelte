<script lang="ts">
  // Radial skill wheel that pops around the selected piece. The wheel is a
  // single ring divided into three sectors:
  //
  //   ┌──────────── Skill 1 (top, ~150°) ────────────┐
  //   │                                              │
  //   │           ┌──────────────┐    ┌── End Phase ─┤
  //   │           │              │    │   (right,    │
  //   │           │   PIECE      │    │    ~60°)     │
  //   │           │              │    └──────────────┤
  //   │                                              │
  //   └──────────── Skill 2 (bottom, ~150°) ─────────┘
  //
  // Slot empty (skill1 / skill2 == 0): that sector renders as a faded
  // placeholder. The End-Phase sector is always present.
  //
  // Focus / Charge are SKILLS the piece may have equipped — they are NOT
  // dedicated slices. When either is currently staged on the position
  // (`focusActive` / `chargeActive`), a small badge appears *inside* the
  // wheel near the piece, hoverable for an info card.
  //
  // All coordinates are in piece-local space (0..size on each axis). The
  // parent translates by the piece's (x, y) so the wheel sits centred on it.

  import { skillColor, SKILLS } from "$lib/engine";

  export type SliceKind =
    | { kind: "skill"; skillId: number; slot: 1 | 2 }
    | { kind: "endphase" }
    | { kind: "modifierBadge"; modifier: "focus" | "charge" };

  interface Props {
    /** Tile size in SVG units (matches Board's SIZE). */
    size: number;
    /** Skill IDs from the selected piece's mailbox slots (0 = empty). */
    skill1: number;
    skill2: number;
    /** Which skill (if any) is currently armed. Drives the "armed" glow. */
    armedSkillId: number | null;
    /** Modifier flags from position.pendingModifiers. Drive whether the
     *  Focus / Charge badge inside the wheel renders. */
    focusActive: boolean;
    chargeActive: boolean;
    /** Whether each skill sector's action is currently legal (greyed out
     *  if not). */
    skill1Legal: boolean;
    skill2Legal: boolean;
    endPhaseLegal: boolean;
    /** Click handler — single sink for every interactive region. */
    onSliceClick: (slice: SliceKind) => void;
    /** Hover handler. Called with `null` on mouse-leave of all regions. */
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
    endPhaseLegal,
    onSliceClick,
    onSliceHover,
  }: Props = $props();

  // Geometry. Centre of the wheel = piece centre = (size/2, size/2).
  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  /** Inner radius — leaves the piece visible at the ring's centre. */
  const rInner = $derived(size * 0.62);
  /** Outer radius — the slice's outer edge. */
  const rOuter = $derived(size * 1.05);
  /** Mid-radius where glyphs / labels sit. */
  const rMid = $derived((size * 0.62 + size * 1.05) / 2);

  // Sector angles in degrees. SVG y-axis is flipped (down=positive), so we
  // compute with the convention 0°=right, 90°=down (matches Math.cos/sin
  // for SVG). Angles are measured clockwise from the +x axis.
  //
  // End-Phase = thin slice centred on the right (angle 0):
  //   from -15° to +15°  (30° wide)
  // Skill 1 = top half-ring centred upward (angle -90°):
  //   from -165° to -15° (150° wide), i.e. left of End-Phase, going up & around.
  // Skill 2 = bottom half-ring centred downward (angle +90°):
  //   from +15° to +165° (150° wide), i.e. right of nothing, going down & around.
  // Small ~0° gaps between sectors are introduced via slight inset (+1° each
  // side) so the slices read as separate.
  const GAP_DEG = 1;
  const endStart = -15 + GAP_DEG;
  const endEnd = 15 - GAP_DEG;
  const skill1Start = -165 + GAP_DEG;
  const skill1End = -15 - GAP_DEG;
  const skill2Start = 15 + GAP_DEG;
  const skill2End = 165 - GAP_DEG;

  function deg2rad(d: number): number {
    return (d * Math.PI) / 180;
  }

  /** Build an SVG arc-ring sector path between two angles (degrees). */
  function sectorPath(
    a0: number,
    a1: number,
    rInnerLocal: number,
    rOuterLocal: number,
  ): string {
    const r0 = deg2rad(a0);
    const r1 = deg2rad(a1);
    const x0o = cx + rOuterLocal * Math.cos(r0);
    const y0o = cy + rOuterLocal * Math.sin(r0);
    const x1o = cx + rOuterLocal * Math.cos(r1);
    const y1o = cy + rOuterLocal * Math.sin(r1);
    const x1i = cx + rInnerLocal * Math.cos(r1);
    const y1i = cy + rInnerLocal * Math.sin(r1);
    const x0i = cx + rInnerLocal * Math.cos(r0);
    const y0i = cy + rInnerLocal * Math.sin(r0);
    const large = a1 - a0 > 180 ? 1 : 0;
    return [
      `M ${x0o} ${y0o}`,
      `A ${rOuterLocal} ${rOuterLocal} 0 ${large} 1 ${x1o} ${y1o}`,
      `L ${x1i} ${y1i}`,
      `A ${rInnerLocal} ${rInnerLocal} 0 ${large} 0 ${x0i} ${y0i}`,
      "Z",
    ].join(" ");
  }

  /** Position on the mid-arc at angle `aDeg`. */
  function midPoint(aDeg: number, rad: number = rMid): { x: number; y: number } {
    const r = deg2rad(aDeg);
    return { x: cx + rad * Math.cos(r), y: cy + rad * Math.sin(r) };
  }

  // Pre-computed slice paths + glyph anchors.
  const skill1Path = $derived(sectorPath(skill1Start, skill1End, rInner, rOuter));
  const skill2Path = $derived(sectorPath(skill2Start, skill2End, rInner, rOuter));
  const endPath = $derived(sectorPath(endStart, endEnd, rInner, rOuter));

  const skill1Glyph = $derived(midPoint((skill1Start + skill1End) / 2));
  const skill2Glyph = $derived(midPoint((skill2Start + skill2End) / 2));
  const endGlyph = $derived(midPoint((endStart + endEnd) / 2));

  // Glyph icon size — fits comfortably inside the sector's mid-arc band.
  const glyphSize = $derived((rOuter - rInner) * 0.55);

  // Modifier badges sit just inside the inner ring, on the left (focus) and
  // right (charge) of the piece. Only rendered when the corresponding
  // modifier is currently active on the position.
  const focusBadgePos = $derived(midPoint(180, rInner * 0.55));
  const chargeBadgePos = $derived(midPoint(0, rInner * 0.55));
  const badgeR = $derived(size * 0.13);

  function sectorClasses(legal: boolean, armed: boolean): string {
    return [
      "sector",
      legal ? "" : "disabled",
      armed ? "armed" : "",
    ].filter(Boolean).join(" ");
  }
</script>

<g class="skill-wheel" pointer-events="auto">
  <!-- Skill 1 sector (top half-ring) -->
  {#if skill1 > 0}
    {@const armed = armedSkillId === skill1}
    <g
      class={sectorClasses(skill1Legal, armed)}
      onpointerdown={(e) => { e.stopPropagation(); if (skill1Legal) onSliceClick({ kind: "skill", skillId: skill1, slot: 1 }); }}
      onpointerenter={() => onSliceHover({ kind: "skill", skillId: skill1, slot: 1 })}
      onpointerleave={() => onSliceHover(null)}
      role="button"
      tabindex="0"
      aria-label={SKILLS[skill1]?.key ?? "skill 1"}
    >
      <path
        d={skill1Path}
        fill="#fefcf3"
        stroke={skillColor(skill1)}
        stroke-width="2.4"
        stroke-linejoin="round"
      />
      {#if armed}
        <path
          d={skill1Path}
          fill="none"
          stroke={skillColor(skill1)}
          stroke-width="3.5"
          stroke-opacity="0.55"
        >
          <animate attributeName="stroke-opacity" values="0.25;0.7;0.25" dur="1.2s" repeatCount="indefinite" />
        </path>
      {/if}
      <use
        href="#skill-glyph-{skill1}"
        x={skill1Glyph.x - glyphSize / 2}
        y={skill1Glyph.y - glyphSize / 2}
        width={glyphSize}
        height={glyphSize}
        color={skillColor(skill1)}
        stroke-width="2.4"
        pointer-events="none"
      />
    </g>
  {:else}
    <!-- Empty slot 1 — render placeholder so the wheel reads as full. -->
    <path
      d={skill1Path}
      fill="#f3ecd9"
      stroke="#b0a47a"
      stroke-width="1.8"
      stroke-dasharray="4 3"
      stroke-linejoin="round"
      opacity="0.55"
      pointer-events="none"
    />
  {/if}

  <!-- Skill 2 sector (bottom half-ring) -->
  {#if skill2 > 0}
    {@const armed = armedSkillId === skill2}
    <g
      class={sectorClasses(skill2Legal, armed)}
      onpointerdown={(e) => { e.stopPropagation(); if (skill2Legal) onSliceClick({ kind: "skill", skillId: skill2, slot: 2 }); }}
      onpointerenter={() => onSliceHover({ kind: "skill", skillId: skill2, slot: 2 })}
      onpointerleave={() => onSliceHover(null)}
      role="button"
      tabindex="0"
      aria-label={SKILLS[skill2]?.key ?? "skill 2"}
    >
      <path
        d={skill2Path}
        fill="#fefcf3"
        stroke={skillColor(skill2)}
        stroke-width="2.4"
        stroke-linejoin="round"
      />
      {#if armed}
        <path
          d={skill2Path}
          fill="none"
          stroke={skillColor(skill2)}
          stroke-width="3.5"
          stroke-opacity="0.55"
        >
          <animate attributeName="stroke-opacity" values="0.25;0.7;0.25" dur="1.2s" repeatCount="indefinite" />
        </path>
      {/if}
      <use
        href="#skill-glyph-{skill2}"
        x={skill2Glyph.x - glyphSize / 2}
        y={skill2Glyph.y - glyphSize / 2}
        width={glyphSize}
        height={glyphSize}
        color={skillColor(skill2)}
        stroke-width="2.4"
        pointer-events="none"
      />
    </g>
  {:else}
    <path
      d={skill2Path}
      fill="#f3ecd9"
      stroke="#b0a47a"
      stroke-width="1.8"
      stroke-dasharray="4 3"
      stroke-linejoin="round"
      opacity="0.55"
      pointer-events="none"
    />
  {/if}

  <!-- End-Phase sector (thin right slice) -->
  <g
    class={sectorClasses(endPhaseLegal, false)}
    onpointerdown={(e) => { e.stopPropagation(); if (endPhaseLegal) onSliceClick({ kind: "endphase" }); }}
    onpointerenter={() => onSliceHover({ kind: "endphase" })}
    onpointerleave={() => onSliceHover(null)}
    role="button"
    tabindex="0"
    aria-label="end phase"
  >
    <path
      d={endPath}
      fill="#fefcf3"
      stroke="#5a4a3a"
      stroke-width="2.4"
      stroke-linejoin="round"
    />
    <text
      x={endGlyph.x}
      y={endGlyph.y + glyphSize * 0.18}
      text-anchor="middle"
      font-size={glyphSize * 0.9}
      font-weight="700"
      fill="#5a4a3a"
      pointer-events="none"
    >⏵</text>
  </g>

  <!-- Modifier badges. Rendered only when staged. Sit inside the ring near
       the piece. Hoverable so the info card explains what's about to fire. -->
  {#if focusActive}
    <g
      class="modifier-badge"
      onpointerenter={() => onSliceHover({ kind: "modifierBadge", modifier: "focus" })}
      onpointerleave={() => onSliceHover(null)}
      role="img"
      aria-label="focus active"
    >
      <circle
        cx={focusBadgePos.x}
        cy={focusBadgePos.y}
        r={badgeR}
        fill="#fefcf3"
        stroke="#8a4abd"
        stroke-width="2"
      >
        <animate attributeName="r" values="{badgeR};{badgeR + 1.5};{badgeR}" dur="1.6s" repeatCount="indefinite" />
      </circle>
      <text
        x={focusBadgePos.x}
        y={focusBadgePos.y + badgeR * 0.32}
        text-anchor="middle"
        font-size={badgeR * 1.0}
        font-weight="700"
        fill="#8a4abd"
        pointer-events="none"
      >+1</text>
    </g>
  {/if}
  {#if chargeActive}
    <g
      class="modifier-badge"
      onpointerenter={() => onSliceHover({ kind: "modifierBadge", modifier: "charge" })}
      onpointerleave={() => onSliceHover(null)}
      role="img"
      aria-label="charge active"
    >
      <circle
        cx={chargeBadgePos.x}
        cy={chargeBadgePos.y}
        r={badgeR}
        fill="#fefcf3"
        stroke="#8a4abd"
        stroke-width="2"
      >
        <animate attributeName="r" values="{badgeR};{badgeR + 1.5};{badgeR}" dur="1.6s" repeatCount="indefinite" />
      </circle>
      <text
        x={chargeBadgePos.x}
        y={chargeBadgePos.y + badgeR * 0.32}
        text-anchor="middle"
        font-size={badgeR * 0.9}
        font-weight="700"
        fill="#8a4abd"
        pointer-events="none"
      >⚡</text>
    </g>
  {/if}
</g>

<style>
  .sector {
    cursor: pointer;
    transition: filter 0.12s ease-out;
  }
  .sector:hover {
    filter: brightness(0.98) drop-shadow(0 0 4px rgba(0, 0, 0, 0.12));
  }
  .sector.disabled {
    opacity: 0.32;
    cursor: not-allowed;
  }
  .modifier-badge {
    cursor: help;
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
