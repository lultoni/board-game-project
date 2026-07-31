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
  // Focus / Charge are SKILLS the piece may have equipped - they are NOT
  // dedicated slices. When either is currently staged on the position
  // (`focusActive` / `chargeActive`), a small badge appears *inside* the
  // wheel near the piece, hoverable for an info card.
  //
  // All coordinates are in piece-local space (0..size on each axis). The
  // parent translates by the piece's (x, y) so the wheel sits centred on it.

  import { skillColor, SKILLS } from "$lib/engine";

  /** A quarter of a split skill. "focusMode" skills (Blast/Shove) split into
   *  activation(+rng) / effect(+eff); "retarget" skills (Shield/Dash/Retreat)
   *  split into self / ally. The `variant` string is the quarter's identity. */
  export type FocusVariant = "activation" | "effect" | "self" | "ally";

  export type SliceKind =
    | { kind: "skill"; skillId: number; slot: 1 | 2 }
    | { kind: "focusBoost"; slot: 1 | 2; skillId: number; variant: FocusVariant }
    | { kind: "modifierBadge"; modifier: "focus" | "charge" };

  /** Describes how one slot's half splits into two focus quarters. The two
   *  quarters are (a, b): focusMode → (activation, effect); retarget → (self, ally). */
  export interface SplitDesc {
    kind: "focusMode" | "retarget";
    /** Legality of quarter A (activation | self) and B (effect | ally). */
    aLegal: boolean;
    bLegal: boolean;
    /** Which quarter is currently armed, or null. */
    armed: FocusVariant | null;
  }

  interface Props {
    /** Tile size in SVG units (matches Board's SIZE). */
    size: number;
    /** Skill IDs from the selected piece's mailbox slots (0 = empty). */
    skill1: number;
    skill2: number;
    /** Which skill (if any) is currently armed. Drives the "armed" glow. */
    armedSkillId: number | null;
    /** Modifier flags from position.pendingModifiers. */
    focusActive: boolean;
    chargeActive: boolean;
    /** Whether each skill sector's action is currently legal. */
    skill1Legal: boolean;
    skill2Legal: boolean;
    /** Per-slot focus split descriptor. When non-null, that slot's half-ring
     *  divides into two quarters. `kind` picks the labels (focusMode → +rng/+eff,
     *  retarget → self/ally). `aLegal`/`bLegal` grey the unavailable quarter.
     *  `armed` marks which quarter is the armed variant (null = neither). */
    split1?: SplitDesc | null;
    split2?: SplitDesc | null;
    /** Click handler. */
    onSliceClick: (slice: SliceKind) => void;
    /** Hover handler. Called with null on mouse-leave of all regions. */
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
    split1 = null,
    split2 = null,
    onSliceClick,
    onSliceHover,
  }: Props = $props();

  // Geometry. Centre of the wheel = piece centre = (size/2, size/2).
  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  /** Inner radius - leaves the piece visible at the ring's centre. */
  const rInner = $derived(size * 0.62);
  /** Outer radius - the slice's outer edge. */
  const rOuter = $derived(size * 1.05);
  /** Mid-radius where glyphs / labels sit. */
  const rMid = $derived((size * 0.62 + size * 1.05) / 2);

  // Sector angles. Two half-rings with a small gap between them.
  // Skill 1 = top half (180° centred upward): -180°+GAP to -GAP
  // Skill 2 = bottom half (180° centred downward): +GAP to +180°-GAP
  const GAP_DEG = 2;
  const skill1Start = -180 + GAP_DEG;
  const skill1End   =  -GAP_DEG;
  const skill2Start =   GAP_DEG;
  const skill2End   =  180 - GAP_DEG;

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

  const skill1Glyph = $derived(midPoint((skill1Start + skill1End) / 2));
  const skill2Glyph = $derived(midPoint((skill2Start + skill2End) / 2));

  // Glyph icon size - fits comfortably inside the sector's mid-arc band.
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

  // Split-sector helpers for the focus picker.
  // When a slot has a SplitDesc, that slot's half-ring is divided into two
  // equal quarter-rings: quarter A (activation | self) and quarter B
  // (effect | ally). The labels/variants are chosen by the split kind below.
  function splitPaths(isSlot1: boolean): { activationPath: string; effectPath: string } {
    const [start, end] = isSlot1
      ? [skill1Start, skill1End]
      : [skill2Start, skill2End];
    const mid = (start + end) / 2;
    return {
      activationPath: sectorPath(start, mid - GAP_DEG / 2, rInner, rOuter),
      effectPath: sectorPath(mid + GAP_DEG / 2, end, rInner, rOuter),
    };
  }
  const slot1Split = $derived(split1 && skill1 > 0 ? splitPaths(true) : null);
  const slot2Split = $derived(split2 && skill2 > 0 ? splitPaths(false) : null);

  /** Labels for a split's two quarters, by kind. A = activation|self, B = effect|ally. */
  function quarterLabels(kind: "focusMode" | "retarget"): { a: string; b: string } {
    return kind === "focusMode" ? { a: "+rng", b: "+eff" } : { a: "self", b: "ally" };
  }
  /** The FocusVariant identity of each quarter, by kind. */
  function quarterVariants(kind: "focusMode" | "retarget"): { a: FocusVariant; b: FocusVariant } {
    return kind === "focusMode"
      ? { a: "activation", b: "effect" }
      : { a: "self", b: "ally" };
  }
</script>

<g class="skill-wheel" pointer-events="auto">
  <!-- Skill 1 sector (top half-ring) -->
  {#if skill1 > 0}
    {@const armed = armedSkillId === skill1}
    {#if slot1Split && split1}
      <!-- Split into two focus quarters (A / B). Clicking a quarter arms this
           skill with that variant. Labels/variants depend on split kind. -->
      {@const lbl = quarterLabels(split1.kind)}
      {@const va = quarterVariants(split1.kind)}
      {@const aArmed = split1.armed === va.a}
      {@const bArmed = split1.armed === va.b}
      <g
        class="sector {aArmed ? 'armed' : ''} {split1.aLegal ? '' : 'disabled'}"
        onpointerdown={(e) => { e.stopPropagation(); if (split1.aLegal) onSliceClick({ kind: "focusBoost", slot: 1, skillId: skill1, variant: va.a }); }}
        onpointerenter={() => onSliceHover({ kind: "focusBoost", slot: 1, skillId: skill1, variant: va.a })}
        onpointerleave={() => onSliceHover(null)}
        role="button" tabindex="0" aria-label={lbl.a}
      >
        <path d={slot1Split.activationPath} fill={aArmed ? skillColor(skill1) : "#fefcf3"}
          stroke={skillColor(skill1)} stroke-width="2.4" stroke-linejoin="round" />
        <text x={midPoint((skill1Start + (skill1Start + skill1End) / 2) / 2).x}
              y={midPoint((skill1Start + (skill1Start + skill1End) / 2) / 2).y + 4}
              text-anchor="middle" font-size={glyphSize * 0.55} font-weight="700"
              fill={aArmed ? "#fefcf3" : skillColor(skill1)} pointer-events="none">{lbl.a}</text>
      </g>
      <g
        class="sector {bArmed ? 'armed' : ''} {split1.bLegal ? '' : 'disabled'}"
        onpointerdown={(e) => { e.stopPropagation(); if (split1.bLegal) onSliceClick({ kind: "focusBoost", slot: 1, skillId: skill1, variant: va.b }); }}
        onpointerenter={() => onSliceHover({ kind: "focusBoost", slot: 1, skillId: skill1, variant: va.b })}
        onpointerleave={() => onSliceHover(null)}
        role="button" tabindex="0" aria-label={lbl.b}
      >
        <path d={slot1Split.effectPath} fill={bArmed ? skillColor(skill1) : "#fefcf3"}
          stroke={skillColor(skill1)} stroke-width="2.4" stroke-linejoin="round" />
        <text x={midPoint(((skill1Start + skill1End) / 2 + skill1End) / 2).x}
              y={midPoint(((skill1Start + skill1End) / 2 + skill1End) / 2).y + 4}
              text-anchor="middle" font-size={glyphSize * 0.55} font-weight="700"
              fill={bArmed ? "#fefcf3" : skillColor(skill1)} pointer-events="none">{lbl.b}</text>
      </g>
    {:else}
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
    {/if}
  {:else}
    <!-- Empty slot 1 - render placeholder so the wheel reads as full. -->
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
    {#if slot2Split && split2}
      {@const lbl = quarterLabels(split2.kind)}
      {@const va = quarterVariants(split2.kind)}
      {@const aArmed = split2.armed === va.a}
      {@const bArmed = split2.armed === va.b}
      <g
        class="sector {aArmed ? 'armed' : ''} {split2.aLegal ? '' : 'disabled'}"
        onpointerdown={(e) => { e.stopPropagation(); if (split2.aLegal) onSliceClick({ kind: "focusBoost", slot: 2, skillId: skill2, variant: va.a }); }}
        onpointerenter={() => onSliceHover({ kind: "focusBoost", slot: 2, skillId: skill2, variant: va.a })}
        onpointerleave={() => onSliceHover(null)}
        role="button" tabindex="0" aria-label={lbl.a}
      >
        <path d={slot2Split.activationPath} fill={aArmed ? skillColor(skill2) : "#fefcf3"}
          stroke={skillColor(skill2)} stroke-width="2.4" stroke-linejoin="round" />
        <text x={midPoint((skill2Start + (skill2Start + skill2End) / 2) / 2).x}
              y={midPoint((skill2Start + (skill2Start + skill2End) / 2) / 2).y + 4}
              text-anchor="middle" font-size={glyphSize * 0.55} font-weight="700"
              fill={aArmed ? "#fefcf3" : skillColor(skill2)} pointer-events="none">{lbl.a}</text>
      </g>
      <g
        class="sector {bArmed ? 'armed' : ''} {split2.bLegal ? '' : 'disabled'}"
        onpointerdown={(e) => { e.stopPropagation(); if (split2.bLegal) onSliceClick({ kind: "focusBoost", slot: 2, skillId: skill2, variant: va.b }); }}
        onpointerenter={() => onSliceHover({ kind: "focusBoost", slot: 2, skillId: skill2, variant: va.b })}
        onpointerleave={() => onSliceHover(null)}
        role="button" tabindex="0" aria-label={lbl.b}
      >
        <path d={slot2Split.effectPath} fill={bArmed ? skillColor(skill2) : "#fefcf3"}
          stroke={skillColor(skill2)} stroke-width="2.4" stroke-linejoin="round" />
        <text x={midPoint(((skill2Start + skill2End) / 2 + skill2End) / 2).x}
              y={midPoint(((skill2Start + skill2End) / 2 + skill2End) / 2).y + 4}
              text-anchor="middle" font-size={glyphSize * 0.55} font-weight="700"
              fill={bArmed ? "#fefcf3" : skillColor(skill2)} pointer-events="none">{lbl.b}</text>
      </g>
    {:else}
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
    {/if}
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
