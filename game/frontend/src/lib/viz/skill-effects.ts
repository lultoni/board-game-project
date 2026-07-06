// Per-skill choreography for the SkillEffect kind. Each skill has its own
// draw function; EffectsLayer dispatches on skillId. Timing is expressed as
// three segments (attack / hold / release) so per-skill tuning stays local.
//
// See .claude/plans/skill-animations.md for the design brief.

import type { SkillEffect } from "./effects";
import { skillColor } from "$lib/engine/skills";

/** Convert #rrggbb + alpha into an rgba() string. Assumes 6-digit hex. */
export function withAlpha(hex: string, a: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

export function squareCenter(sq: number, size: number): { x: number; y: number } {
  const file = sq & 7;
  const rank = (sq >> 3) & 7;
  return { x: file * size + size / 2, y: (7 - rank) * size + size / 2 };
}

/** Deterministic small wobble sign for a src/target pair — so bezier control
 *  offsets feel intentional rather than random. Same pair always produces the
 *  same wobble. */
export function pairSign(from: number, to: number): 1 | -1 {
  const h = (from * 73856093) ^ (to * 19349663);
  return (h & 1) ? 1 : -1;
}

/** Perpendicular offset vector for a bezier control point mid-way between two
 *  squares. `perpPx` is the offset magnitude in canvas px. */
export function perpOffset(
  from: { x: number; y: number },
  to: { x: number; y: number },
  perpPx: number,
): { x: number; y: number } {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy) || 1;
  return { x: -dy / len * perpPx, y: dx / len * perpPx };
}

/** Quadratic bezier point at t. */
export function bezierPt(
  a: { x: number; y: number },
  ctrl: { x: number; y: number },
  b: { x: number; y: number },
  t: number,
): { x: number; y: number } {
  const u = 1 - t;
  return {
    x: u * u * a.x + 2 * u * t * ctrl.x + t * t * b.x,
    y: u * u * a.y + 2 * u * t * ctrl.y + t * t * b.y,
  };
}

/** Phase-map: given t (0..1) with attack + hold + release fractions, return
 *  `{ phase, local }` where phase is "attack" | "hold" | "release" and local
 *  is 0..1 within that phase. */
export function phaseOf(
  t: number,
  attackFrac: number,
  holdFrac: number,
): { phase: "attack" | "hold" | "release"; local: number } {
  if (t < attackFrac) return { phase: "attack", local: t / attackFrac };
  if (t < attackFrac + holdFrac) return { phase: "hold", local: (t - attackFrac) / holdFrac };
  const rel = 1 - attackFrac - holdFrac;
  return { phase: "release", local: (t - attackFrac - holdFrac) / rel };
}

/** Nib-pressure stroke width: baseWidth * (1 + amp * sin(t * PI)) so the
 *  middle of the stroke is thicker than the ends. */
export function nibWidth(baseWidth: number, t: number, amp = 0.6): number {
  return baseWidth * (1 + amp * Math.sin(t * Math.PI));
}

// === Per-skill render dispatch ==============================================

export function renderSkill(
  ctx: CanvasRenderingContext2D,
  eff: SkillEffect,
  age: number,
  size: number,
): void {
  const dispatch = SKILL_RENDERERS[eff.skillId];
  if (!dispatch) return;
  dispatch(ctx, eff, age, size);
}

type SkillRenderer = (
  ctx: CanvasRenderingContext2D,
  eff: SkillEffect,
  age: number,
  size: number,
) => void;

// === Skill 1: Lance — the stab ==============================================
// A thin spear-mark grows from caster toward target, punches 8px past target
// center, then retracts. Timing: 140ms extend + 60ms hold + 200ms retract +
// 200ms fade = 600ms.

const LANCE_TTL = 600;

const renderLance: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= LANCE_TTL) return;
  const t = age / LANCE_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len, uy = dy / len;
  const punchPx = size * 0.08;

  // Phase fractions (of 600ms): 140/60/(200+200)
  const attackFrac = 140 / LANCE_TTL;
  const holdFrac = 60 / LANCE_TTL;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  let tipX: number, tipY: number, alpha: number;
  if (phase === "attack") {
    // Grows from caster to target-plus-punch.
    tipX = from.x + ux * (len + punchPx) * local;
    tipY = from.y + uy * (len + punchPx) * local;
    alpha = 0.95;
  } else if (phase === "hold") {
    tipX = to.x + ux * punchPx;
    tipY = to.y + uy * punchPx;
    alpha = 0.95;
  } else {
    // Retract + fade combined: tip pulls back to caster while alpha fades.
    tipX = from.x + (to.x + ux * punchPx - from.x) * (1 - local);
    tipY = from.y + (to.y + uy * punchPx - from.y) * (1 - local);
    alpha = 0.9 * (1 - local);
  }
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.045;
  ctx.lineCap = "round";
  ctx.beginPath();
  ctx.moveTo(from.x, from.y);
  ctx.lineTo(tipX, tipY);
  ctx.stroke();
  // Spear tip: a small filled triangle at the tip during attack + hold.
  if (phase !== "release") {
    const tipSize = size * 0.06;
    ctx.fillStyle = withAlpha(color, alpha);
    ctx.beginPath();
    ctx.moveTo(tipX + ux * tipSize, tipY + uy * tipSize);
    ctx.lineTo(tipX + -uy * tipSize * 0.5, tipY + ux * tipSize * 0.5);
    ctx.lineTo(tipX - -uy * tipSize * 0.5, tipY - ux * tipSize * 0.5);
    ctx.closePath();
    ctx.fill();
  }
};

// === Skill 2: Hook — the pull ===============================================
// Curved hook-line draws from caster to target (200ms, pronounced drop
// mid-path). Catches (80ms hold). Line pulls taut over 180ms while target
// piece slides. Fade 200ms. Total ~660ms.

const HOOK_TTL = 660;

const renderHook: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= HOOK_TTL) return;
  const t = age / HOOK_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);
  // Bezier control offset — sags "downward" perpendicular to the ray, plus a
  // deterministic sign so left-to-right and right-to-left casts curve
  // opposite ways rather than always the same side.
  const sign = pairSign(eff.from, eff.to);
  const perp = perpOffset(from, to, size * 0.55 * sign);
  const midX = (from.x + to.x) / 2 + perp.x;
  const midY = (from.y + to.y) / 2 + perp.y;

  // Phase fractions of 660: 200/80/(180+200). Break the release into "pull
  // taut" (180) then "fade" (200) manually via a nested local.
  const attackFrac = 200 / HOOK_TTL;
  const holdFrac = 80 / HOOK_TTL;
  const pullFrac = 180 / HOOK_TTL;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  // Sag amount: full during attack + hold; interpolated to 0 during pull;
  // stays at 0 during fade. Alpha holds during attack/hold/pull; fades in
  // the final segment.
  let sag: number;
  let alpha: number;
  if (phase === "attack") {
    // Draw sweep across the curve — we truncate the bezier at t=local.
    sag = 1;
    alpha = 0.9;
  } else if (phase === "hold") {
    sag = 1;
    alpha = 0.9;
  } else {
    // release: split into pull and fade
    const relStart = attackFrac + holdFrac;
    const withinRelease = t - relStart;
    if (withinRelease < pullFrac) {
      const pullLocal = withinRelease / pullFrac;
      sag = 1 - pullLocal;
      alpha = 0.9;
    } else {
      const fadeLocal = (withinRelease - pullFrac) / (1 - relStart - pullFrac);
      sag = 0;
      alpha = 0.9 * (1 - fadeLocal);
      // local silences the "unused" complaint via reference
      void local;
    }
  }

  // Actual bezier control: mix straight midpoint with sagged midpoint.
  const straightMidX = (from.x + to.x) / 2;
  const straightMidY = (from.y + to.y) / 2;
  const ctrlX = straightMidX + (midX - straightMidX) * sag;
  const ctrlY = straightMidY + (midY - straightMidY) * sag;

  // Determine the segment of the curve currently drawn. Attack grows the
  // stroke from t=0 to t=1 along the curve; hold + release show the full
  // curve.
  const drawEnd = phase === "attack" ? local : 1;

  // Rasterize the curve as a series of short segments so we can taper width.
  const steps = 24;
  const drawSteps = Math.max(1, Math.floor(steps * drawEnd));
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineCap = "round";
  ctx.beginPath();
  for (let i = 0; i <= drawSteps; i++) {
    const tt = i / steps;
    const p = bezierPt(from, { x: ctrlX, y: ctrlY }, to, tt);
    if (i === 0) ctx.moveTo(p.x, p.y);
    else ctx.lineTo(p.x, p.y);
  }
  ctx.lineWidth = nibWidth(size * 0.04, drawEnd);
  ctx.stroke();

  // Hook barb at the target end during attack + hold. Small triangle
  // perpendicular to the tangent at t=1.
  if (phase !== "release") {
    const near = bezierPt(from, { x: ctrlX, y: ctrlY }, to, 0.94);
    const dx = to.x - near.x;
    const dy = to.y - near.y;
    const len = Math.hypot(dx, dy) || 1;
    const ux = dx / len, uy = dy / len;
    const barb = size * 0.08;
    ctx.fillStyle = withAlpha(color, alpha);
    ctx.beginPath();
    ctx.moveTo(to.x + ux * barb * 0.4, to.y + uy * barb * 0.4);
    ctx.lineTo(to.x + -uy * barb, to.y + ux * barb);
    ctx.lineTo(to.x - ux * barb * 0.4, to.y - uy * barb * 0.4);
    ctx.closePath();
    ctx.fill();
  }
};

// === Skill 6: Shield (self) — the brace =====================================
// A heater-shield silhouette draws itself over the piece top-down (140ms),
// holds (200ms), fades (180ms). Timing total ~520ms — we hold a bit longer
// than a strike because "becoming armored" reads slower than "stabbing".

const SHIELD_TTL = 620;

const renderShieldSelf: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= SHIELD_TTL) return;
  const t = age / SHIELD_TTL;
  const color = skillColor(eff.skillId);
  const c = squareCenter(eff.from, size);
  const attackFrac = 140 / SHIELD_TTL;
  const holdFrac = 300 / SHIELD_TTL;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  // Shield silhouette: rounded-top rectangle tapering to a point at the
  // bottom (heater-shield outline). Width ~size*0.32, height ~size*0.44.
  const w = size * 0.32;
  const h = size * 0.44;
  // Top of shield sits slightly above piece center.
  const topY = c.y - h * 0.55;

  // "Draw itself top-down" — reveal via clipping. During attack, the shield
  // is revealed from top down; during hold it's fully visible; during
  // release it fades in alpha but stays fully drawn.
  const reveal = phase === "attack" ? local : 1;
  const alpha = phase === "release" ? 0.9 * (1 - local) : 0.9;

  ctx.save();
  ctx.beginPath();
  ctx.rect(c.x - w, topY, w * 2, h * reveal);
  ctx.clip();
  // Shield outline path.
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.045;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  ctx.beginPath();
  // Start at top-left, arc across the top, straight down to point.
  ctx.moveTo(c.x - w, topY + h * 0.15);
  ctx.quadraticCurveTo(c.x - w, topY, c.x, topY);
  ctx.quadraticCurveTo(c.x + w, topY, c.x + w, topY + h * 0.15);
  ctx.lineTo(c.x + w * 0.7, topY + h * 0.7);
  ctx.quadraticCurveTo(c.x, topY + h, c.x, topY + h);
  ctx.quadraticCurveTo(c.x, topY + h, c.x - w * 0.7, topY + h * 0.7);
  ctx.closePath();
  ctx.stroke();
  ctx.restore();
};

// === Skill 14: Focus — the sharpen ==========================================
// Four crosshair ticks converge on piece from N/S/E/W (140ms). Hold with a
// subtle outline glow (300ms). Fade (200ms).

const FOCUS_TTL = 640;

const renderFocus: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= FOCUS_TTL) return;
  const t = age / FOCUS_TTL;
  const color = skillColor(eff.skillId);
  const c = squareCenter(eff.from, size);

  const attackFrac = 140 / FOCUS_TTL;
  const holdFrac = 300 / FOCUS_TTL;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  const outerR = size * 0.32;
  const innerR = size * 0.22;
  const tickLen = size * 0.08;

  let tickAt: number;
  let alpha: number;
  if (phase === "attack") {
    // ticks travel from outerR toward innerR
    tickAt = outerR + (innerR - outerR) * local;
    alpha = 0.9;
  } else if (phase === "hold") {
    tickAt = innerR;
    alpha = 0.9;
  } else {
    tickAt = innerR;
    alpha = 0.9 * (1 - local);
  }

  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.03;
  ctx.lineCap = "round";
  // 4 ticks: pointing inward at N/S/E/W.
  const dirs = [
    { dx: 0, dy: -1 },
    { dx: 1, dy: 0 },
    { dx: 0, dy: 1 },
    { dx: -1, dy: 0 },
  ];
  for (const d of dirs) {
    const startX = c.x + d.dx * (tickAt + tickLen);
    const startY = c.y + d.dy * (tickAt + tickLen);
    const endX = c.x + d.dx * tickAt;
    const endY = c.y + d.dy * tickAt;
    ctx.beginPath();
    ctx.moveTo(startX, startY);
    ctx.lineTo(endX, endY);
    ctx.stroke();
  }

  // During hold + release: subtle circular outline around the piece.
  if (phase !== "attack") {
    const outlineA = phase === "release" ? 0.5 * (1 - local) : 0.5;
    ctx.strokeStyle = withAlpha(color, outlineA);
    ctx.lineWidth = size * 0.02;
    ctx.beginPath();
    ctx.arc(c.x, c.y, size * 0.32, 0, Math.PI * 2);
    ctx.stroke();
  }
};

// === Skill 15: Charge — the wind-up =========================================
// Spiral coil draws outward from piece center (220ms, 1.5 turns, r=14px).
// Holds with a slow ~10° clockwise rotation (240ms). Fades (180ms).

const CHARGE_TTL = 640;

const renderCharge: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= CHARGE_TTL) return;
  const t = age / CHARGE_TTL;
  const color = skillColor(eff.skillId);
  const c = squareCenter(eff.from, size);

  const attackFrac = 220 / CHARGE_TTL;
  const holdFrac = 240 / CHARGE_TTL;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  const maxR = size * 0.28;
  const turns = 1.5;

  let alpha: number;
  let rotationOffset: number;
  let drawFrac: number;

  if (phase === "attack") {
    drawFrac = local;
    alpha = 0.9;
    rotationOffset = 0;
  } else if (phase === "hold") {
    drawFrac = 1;
    alpha = 0.9;
    // Slow clockwise rotation during hold — ~10° over the hold.
    rotationOffset = (Math.PI / 18) * local;
  } else {
    drawFrac = 1;
    alpha = 0.9 * (1 - local);
    rotationOffset = Math.PI / 18;
  }

  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineCap = "round";
  const steps = 60;
  const drawSteps = Math.max(1, Math.floor(steps * drawFrac));
  ctx.beginPath();
  for (let i = 0; i <= drawSteps; i++) {
    const tt = i / steps;
    const theta = tt * turns * Math.PI * 2 + rotationOffset;
    const r = tt * maxR;
    const x = c.x + Math.cos(theta) * r;
    const y = c.y + Math.sin(theta) * r;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  // Slight width taper: thicker near center, thinner at outer end.
  ctx.lineWidth = size * 0.028;
  ctx.stroke();
};

// === Fallback / unimplemented ================================================
// Skills without a dedicated renderer yet: quiet placeholder so F1's plumbing
// still has a visible signal. Replaced skill-by-skill as we work through the
// build order. Draws a hairline stroke from caster to target (or a faint ring
// for self-cast) in the category color.

const renderFallback: SkillRenderer = (ctx, eff, age, size) => {
  const ttl = 500;
  if (age >= ttl) return;
  const t = age / ttl;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);
  if (eff.from === eff.to) {
    const r = size * 0.28 + t * size * 0.08;
    ctx.strokeStyle = withAlpha(color, (1 - t) * 0.55);
    ctx.lineWidth = size * 0.04;
    ctx.beginPath();
    ctx.arc(from.x, from.y, r, 0, Math.PI * 2);
    ctx.stroke();
    return;
  }
  const drawT = t < 0.4 ? t / 0.4 : 1;
  const alpha = t < 0.55 ? 0.7 : 0.7 * (1 - (t - 0.55) / 0.45);
  const endX = from.x + (to.x - from.x) * drawT;
  const endY = from.y + (to.y - from.y) * drawT;
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.03;
  ctx.lineCap = "round";
  ctx.beginPath();
  ctx.moveTo(from.x, from.y);
  ctx.lineTo(endX, endY);
  ctx.stroke();
};

// === Skill 3: Break — the shatter ===========================================
// Short thick chisel-mark stamps down (100ms), radial crack-lines emanate
// from impact (60ms draw, 250ms fade). Total ~410ms.

const BREAK_TTL = 410;

const renderBreak: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= BREAK_TTL) return;
  const t = age / BREAK_TTL;
  const color = skillColor(eff.skillId);
  const to = squareCenter(eff.to, size);
  const from = squareCenter(eff.from, size);

  // Phase: strike 100 / cracks-draw 60 / fade 250. Combine strike+cracks-draw
  // as one attack; then release.
  const attackFrac = 160 / BREAK_TTL;
  const holdFrac = 0;
  const { phase, local } = phaseOf(t, attackFrac, holdFrac);

  // Thick chisel-mark: short stroke coming down onto target from caster
  // direction. Only draws during first ~60% of attack.
  if (phase === "attack" && local < 0.63) {
    const strikeLocal = local / 0.63;
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const len = Math.hypot(dx, dy) || 1;
    const ux = dx / len, uy = dy / len;
    // Chisel starts ~0.4 size out from target and moves in.
    const chiselLen = size * 0.4;
    const tipDist = chiselLen * (1 - strikeLocal);
    const tipX = to.x - ux * tipDist;
    const tipY = to.y - uy * tipDist;
    const tailX = tipX - ux * (chiselLen * 0.35);
    const tailY = tipY - uy * (chiselLen * 0.35);
    ctx.strokeStyle = withAlpha(color, 0.95);
    ctx.lineWidth = size * 0.075;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(tailX, tailY);
    ctx.lineTo(tipX, tipY);
    ctx.stroke();
  }

  // Crack lines: 5 short jagged strokes from target center outward. Draw
  // starting at ~60% of attack, fade over release.
  if (t > 0.15) {
    const crackT = phase === "attack"
      ? (local - 0.63) / 0.37
      : 1;
    const alpha = phase === "release" ? 0.9 * (1 - local) : 0.9;
    if (crackT > 0) {
      ctx.strokeStyle = withAlpha(color, alpha);
      ctx.lineWidth = size * 0.025;
      ctx.lineCap = "round";
      const nCracks = 5;
      // Deterministic angle spread from src+target hash so cracks feel
      // intentional per-cast, not jittery per-frame.
      const seed = (eff.from * 2654435761 ^ eff.to * 40503) >>> 0;
      for (let i = 0; i < nCracks; i++) {
        const baseAng = (i / nCracks) * Math.PI * 2;
        const jitter = ((seed >> (i * 3)) & 0x7) / 0x7 - 0.5;
        const ang = baseAng + jitter * 0.4;
        const maxLen = size * (0.18 + ((seed >> (i * 3 + 8)) & 0x7) / 0x7 * 0.1);
        const len = maxLen * Math.min(1, crackT);
        // Draw a slight zigzag: midpoint kinked by ~15° in a deterministic dir.
        const midR = len * 0.55;
        const kinkAng = ang + (((seed >> (i * 2 + 16)) & 1) ? 0.25 : -0.25);
        const midX = to.x + Math.cos(kinkAng) * midR;
        const midY = to.y + Math.sin(kinkAng) * midR;
        const endX = to.x + Math.cos(ang) * len;
        const endY = to.y + Math.sin(ang) * len;
        ctx.beginPath();
        ctx.moveTo(to.x, to.y);
        ctx.lineTo(midX, midY);
        ctx.lineTo(endX, endY);
        ctx.stroke();
      }
    }
  }
};

// === Skill 4: Steal — the pickpocket ========================================
// Thin dashed line darts caster→target (120ms). Coin flies back caster→target
// (200ms) after grab. Caster scale-pulse can't be done from Canvas without
// coupling to Piece — instead, we add a small "gotcha" burst at caster on
// arrival. Total ~600ms.

const STEAL_TTL = 700;

const renderSteal: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= STEAL_TTL) return;
  const t = age / STEAL_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);

  // Segments (of 700ms): reach 120 / grab 100 / return 200 / burst 100 / fade 180
  const reachEnd = 120 / STEAL_TTL;
  const grabEnd = 220 / STEAL_TTL;
  const returnEnd = 420 / STEAL_TTL;
  const burstEnd = 520 / STEAL_TTL;

  // Reach line: dashed stroke from caster to target, drawn during reach.
  // Persists faintly through grab, then fades.
  ctx.setLineDash([size * 0.06, size * 0.05]);
  ctx.lineCap = "butt";
  ctx.lineWidth = size * 0.028;
  let reachAlpha: number;
  let reachT: number;
  if (t < reachEnd) {
    reachAlpha = 0.85;
    reachT = t / reachEnd;
  } else if (t < grabEnd) {
    reachAlpha = 0.85;
    reachT = 1;
  } else if (t < returnEnd) {
    // fades during return
    reachAlpha = 0.85 * (1 - (t - grabEnd) / (returnEnd - grabEnd));
    reachT = 1;
  } else {
    reachAlpha = 0;
    reachT = 1;
  }
  if (reachAlpha > 0.01) {
    ctx.strokeStyle = withAlpha(color, reachAlpha);
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(from.x + (to.x - from.x) * reachT, from.y + (to.y - from.y) * reachT);
    ctx.stroke();
  }
  ctx.setLineDash([]);

  // Coin: after grab (t > grabEnd), a filled disk travels target → caster.
  if (t > grabEnd && t < returnEnd) {
    const coinT = (t - grabEnd) / (returnEnd - grabEnd);
    const cx = to.x + (from.x - to.x) * coinT;
    const cy = to.y + (from.y - to.y) * coinT;
    ctx.fillStyle = withAlpha(color, 0.9);
    ctx.strokeStyle = withAlpha("#3a2a1a", 0.9);
    ctx.lineWidth = size * 0.015;
    ctx.beginPath();
    ctx.arc(cx, cy, size * 0.065, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  }

  // Caster "gotcha" burst: 4 short radial ticks at the caster during burst
  // phase, fading through the tail.
  if (t > returnEnd) {
    const localT = t < burstEnd
      ? (t - returnEnd) / (burstEnd - returnEnd)
      : 1;
    const fadeT = t < burstEnd ? 0 : (t - burstEnd) / (1 - burstEnd);
    const alpha = 0.9 * (1 - fadeT);
    if (alpha > 0.01) {
      const spokeLen = size * 0.14 * localT;
      ctx.strokeStyle = withAlpha(color, alpha);
      ctx.lineWidth = size * 0.025;
      ctx.lineCap = "round";
      for (let i = 0; i < 4; i++) {
        const ang = (i / 4) * Math.PI * 2 + Math.PI / 4;
        const innerR = size * 0.14;
        const x1 = from.x + Math.cos(ang) * innerR;
        const y1 = from.y + Math.sin(ang) * innerR;
        const x2 = from.x + Math.cos(ang) * (innerR + spokeLen);
        const y2 = from.y + Math.sin(ang) * (innerR + spokeLen);
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
      }
    }
  }
};

// === Skill 5: Tempest — the shock-burst =====================================
// Mechanic: target takes 1 damage, all 8 neighbours of the target get pushed
// 1 tile outward. Animation: quick strike-line caster → target (140ms),
// then an expanding shock-ring around the target with 8 short push-ticks
// pointing outward in the cardinal/diagonal directions.

const TEMPEST_TTL = 720;

const renderTempest: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= TEMPEST_TTL) return;
  const t = age / TEMPEST_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);

  // Segments: strike 140 / shock 300 / fade 280
  const strikeEnd = 140 / TEMPEST_TTL;
  const shockEnd = 440 / TEMPEST_TTL;

  // Strike-line: fast red stroke from caster to target.
  if (t < strikeEnd) {
    const localT = t / strikeEnd;
    const endX = from.x + (to.x - from.x) * localT;
    const endY = from.y + (to.y - from.y) * localT;
    ctx.strokeStyle = withAlpha(color, 0.9);
    ctx.lineWidth = nibWidth(size * 0.045, localT);
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(endX, endY);
    ctx.stroke();
  } else if (t < shockEnd) {
    // Faint residual strike-line fading.
    const fadeT = (t - strikeEnd) / (shockEnd - strikeEnd);
    ctx.strokeStyle = withAlpha(color, 0.5 * (1 - fadeT));
    ctx.lineWidth = size * 0.03;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(to.x, to.y);
    ctx.stroke();
  }

  // Shock: expanding ring around target + 8 outward push-ticks.
  if (t >= strikeEnd) {
    const shockT = t < shockEnd
      ? (t - strikeEnd) / (shockEnd - strikeEnd)
      : 1;
    const fadeT = t < shockEnd ? 0 : (t - shockEnd) / (1 - shockEnd);
    const shockAlpha = 0.9 * (1 - fadeT);

    // Expanding ring: r goes 0.35 → 1.15 tile radii.
    const r = size * (0.35 + shockT * 0.8);
    ctx.strokeStyle = withAlpha(color, shockAlpha * (1 - shockT * 0.5));
    ctx.lineWidth = size * 0.035 * (1 - shockT * 0.4);
    ctx.beginPath();
    ctx.arc(to.x, to.y, r, 0, Math.PI * 2);
    ctx.stroke();

    // 8 push-ticks pointing outward at each cardinal/diagonal direction.
    // They lengthen as the ring expands, giving a "pieces being pushed out"
    // read.
    const tickInnerR = size * 0.55;
    const tickLen = size * 0.18 * shockT;
    if (tickLen > 1) {
      ctx.lineWidth = size * 0.03 * (1 - shockT * 0.3);
      for (let i = 0; i < 8; i++) {
        const ang = (i / 8) * Math.PI * 2;
        const x1 = to.x + Math.cos(ang) * tickInnerR;
        const y1 = to.y + Math.sin(ang) * tickInnerR;
        const x2 = to.x + Math.cos(ang) * (tickInnerR + tickLen);
        const y2 = to.y + Math.sin(ang) * (tickInnerR + tickLen);
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
      }
    }
  }
};

// === Skill 7: Heal — the mending thread =====================================
// Twin thread caster → ally. At ally, thread wraps into a closed loop. Loop
// pulses once as +HP appears. Total ~660ms.

const HEAL_TTL = 660;

const renderHeal: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= HEAL_TTL) return;
  const t = age / HEAL_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);

  // Segments: thread 200 / wrap 120 / pulse 100 / fade 240
  const threadEnd = 200 / HEAL_TTL;
  const wrapEnd = 320 / HEAL_TTL;
  const pulseEnd = 420 / HEAL_TTL;

  const sign = pairSign(eff.from, eff.to);
  const perp = perpOffset(from, to, size * 0.2 * sign);
  const midX = (from.x + to.x) / 2 + perp.x;
  const midY = (from.y + to.y) / 2 + perp.y;

  // Thread: bezier stroke drawn from caster to ally.
  if (t < wrapEnd + 0.1) {
    const drawEnd = t < threadEnd ? t / threadEnd : 1;
    const alpha = 0.75;
    ctx.strokeStyle = withAlpha(color, alpha);
    ctx.lineCap = "round";
    ctx.lineWidth = size * 0.025;
    const steps = 20;
    const drawSteps = Math.max(1, Math.floor(steps * drawEnd));
    ctx.beginPath();
    for (let i = 0; i <= drawSteps; i++) {
      const tt = i / steps;
      const p = bezierPt(from, { x: midX, y: midY }, to, tt);
      if (i === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    }
    ctx.stroke();
  }

  // Wrap loop: closed circle around the ally, drawn once threadEnd passes.
  if (t > threadEnd) {
    const localT = t < wrapEnd
      ? (t - threadEnd) / (wrapEnd - threadEnd)
      : 1;
    let loopR = size * 0.24;
    let loopAlpha: number;
    if (t < wrapEnd) {
      loopAlpha = 0.85 * localT;
    } else if (t < pulseEnd) {
      const pulseLocal = (t - wrapEnd) / (pulseEnd - wrapEnd);
      // Grow then shrink: 0.24 → 0.28 → 0.24
      const pulse = Math.sin(pulseLocal * Math.PI) * (size * 0.04);
      loopR = size * 0.24 + pulse;
      loopAlpha = 0.85;
    } else {
      const fadeLocal = (t - pulseEnd) / (1 - pulseEnd);
      loopAlpha = 0.85 * (1 - fadeLocal);
    }
    ctx.strokeStyle = withAlpha(color, loopAlpha);
    ctx.lineWidth = size * 0.028;
    ctx.beginPath();
    if (t < wrapEnd) {
      // Draw partial loop: arc from angle 0 to 2π * localT.
      ctx.arc(to.x, to.y, loopR, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * localT);
    } else {
      ctx.arc(to.x, to.y, loopR, 0, Math.PI * 2);
    }
    ctx.stroke();
  }
};

// === Skill 8: Plate — the shield handed over ================================
// Small shield-glyph travels caster → ally along arc, settles onto ally.

const PLATE_TTL = 620;

const renderPlate: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= PLATE_TTL) return;
  const t = age / PLATE_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);

  // Segments: travel 220 / settle 80 / fade 200 (+ trailing residue)
  const travelEnd = 220 / PLATE_TTL;
  const settleEnd = 300 / PLATE_TTL;

  // Trajectory: gentle arc.
  const sign = pairSign(eff.from, eff.to);
  const perp = perpOffset(from, to, size * 0.35 * sign);
  const midX = (from.x + to.x) / 2 + perp.x;
  const midY = (from.y + to.y) / 2 + perp.y;

  let cx: number, cy: number, scale: number, alpha: number;
  if (t < travelEnd) {
    const localT = t / travelEnd;
    const p = bezierPt(from, { x: midX, y: midY }, to, localT);
    cx = p.x;
    cy = p.y;
    scale = 1;
    alpha = 0.9;
  } else if (t < settleEnd) {
    const localT = (t - travelEnd) / (settleEnd - travelEnd);
    cx = to.x;
    cy = to.y;
    // Shrink from 1 → 0.65 as it "settles" onto the piece.
    scale = 1 - localT * 0.35;
    alpha = 0.9;
  } else {
    const fadeLocal = (t - settleEnd) / (1 - settleEnd);
    cx = to.x;
    cy = to.y;
    scale = 0.65;
    alpha = 0.9 * (1 - fadeLocal);
  }

  // Heater-shield outline centered at (cx, cy).
  const w = size * 0.24 * scale;
  const h = size * 0.32 * scale;
  const topY = cy - h * 0.5;
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.fillStyle = withAlpha("#f8f1de", alpha * 0.4); // paper-bg fill inside
  ctx.lineWidth = size * 0.03 * scale;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  ctx.beginPath();
  ctx.moveTo(cx - w, topY + h * 0.15);
  ctx.quadraticCurveTo(cx - w, topY, cx, topY);
  ctx.quadraticCurveTo(cx + w, topY, cx + w, topY + h * 0.15);
  ctx.lineTo(cx + w * 0.7, topY + h * 0.7);
  ctx.quadraticCurveTo(cx, topY + h, cx, topY + h);
  ctx.quadraticCurveTo(cx, topY + h, cx - w * 0.7, topY + h * 0.7);
  ctx.closePath();
  ctx.fill();
  ctx.stroke();
};

// === Skill 9: Dash — no dedicated fx (dust trail already carries it) ========
// Fall through to fallback (which is quiet).

// === Skill 10: Blast — the leap-and-strike ==================================
// Piece slide (dust trail) already handles the movement; we add a red radial
// burst at the LANDING square.

const BLAST_TTL = 460;

const renderBlast: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= BLAST_TTL) return;
  const t = age / BLAST_TTL;
  const color = skillColor(eff.skillId);
  // Landing square = eff.to (the enemy target the caster leapt at).
  const to = squareCenter(eff.to, size);

  // Segments: burst-draw 180 / hold 60 / fade 220
  const drawEnd = 180 / BLAST_TTL;
  const holdEnd = 240 / BLAST_TTL;

  let spokeLen: number;
  let alpha: number;
  if (t < drawEnd) {
    const localT = t / drawEnd;
    spokeLen = size * 0.22 * localT;
    alpha = 0.9;
  } else if (t < holdEnd) {
    spokeLen = size * 0.22;
    alpha = 0.9;
  } else {
    const fadeLocal = (t - holdEnd) / (1 - holdEnd);
    spokeLen = size * 0.22 + size * 0.05 * fadeLocal;
    alpha = 0.9 * (1 - fadeLocal);
  }
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.035;
  ctx.lineCap = "round";
  const innerR = size * 0.16;
  const nSpokes = 6;
  for (let i = 0; i < nSpokes; i++) {
    const ang = (i / nSpokes) * Math.PI * 2 + Math.PI / nSpokes;
    const x1 = to.x + Math.cos(ang) * innerR;
    const y1 = to.y + Math.sin(ang) * innerR;
    const x2 = to.x + Math.cos(ang) * (innerR + spokeLen);
    const y2 = to.y + Math.sin(ang) * (innerR + spokeLen);
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
  }
};

// === Skill 11: Shove — the push =============================================
// Thick arrow-stroke from caster's edge toward target (100ms), then arrow
// follows the piece as it slides. We don't have access to the slide start
// time directly, so we render the arrow-stroke fully within the effect
// budget: quick wind-up + short "push follow-through" + fade.

const SHOVE_TTL = 520;

const renderShove: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= SHOVE_TTL) return;
  const t = age / SHOVE_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len, uy = dy / len;

  // Segments: draw 120 / follow 180 / fade 220
  const drawEnd = 120 / SHOVE_TTL;
  const followEnd = 300 / SHOVE_TTL;

  // Arrow originates at caster's edge and extends toward target.
  const startDist = size * 0.28;
  const arrowStart = { x: from.x + ux * startDist, y: from.y + uy * startDist };

  let tipDist: number;
  let alpha: number;
  if (t < drawEnd) {
    const localT = t / drawEnd;
    tipDist = startDist + (len - startDist) * 0.35 * localT;
    alpha = 0.95;
  } else if (t < followEnd) {
    const localT = (t - drawEnd) / (followEnd - drawEnd);
    // Extend further as if following the shoved piece.
    tipDist = startDist + (len - startDist) * (0.35 + 0.5 * localT);
    alpha = 0.95;
  } else {
    const fadeLocal = (t - followEnd) / (1 - followEnd);
    tipDist = startDist + (len - startDist) * 0.85;
    alpha = 0.95 * (1 - fadeLocal);
  }
  const tipX = from.x + ux * tipDist;
  const tipY = from.y + uy * tipDist;
  // Shaft tapers from thick at start to thin at tip.
  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineCap = "round";
  ctx.lineWidth = size * 0.06;
  ctx.beginPath();
  ctx.moveTo(arrowStart.x, arrowStart.y);
  ctx.lineTo(tipX, tipY);
  ctx.stroke();
  // Arrow head at tip (small chevron).
  const headSize = size * 0.09;
  const perpXn = -uy, perpYn = ux;
  ctx.fillStyle = withAlpha(color, alpha);
  ctx.beginPath();
  ctx.moveTo(tipX + ux * headSize * 0.6, tipY + uy * headSize * 0.6);
  ctx.lineTo(tipX + perpXn * headSize * 0.5, tipY + perpYn * headSize * 0.5);
  ctx.lineTo(tipX - perpXn * headSize * 0.5, tipY - perpYn * headSize * 0.5);
  ctx.closePath();
  ctx.fill();
};

// === Skill 12: Swap — the exchange ==========================================
// Two ally pieces exchange squares. Signature = two interlocking curved
// strokes meeting at midpoint + a small purple diamond glyph at the
// crossing.

const SWAP_TTL = 640;

const renderSwap: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= SWAP_TTL) return;
  const t = age / SWAP_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);

  // Segments: draw 260 / diamond 120 / fade 260
  const drawEnd = 260 / SWAP_TTL;
  const diamondEnd = 380 / SWAP_TTL;

  // Two arcs curving opposite directions, each starts at one endpoint,
  // curls toward the midpoint, and stops at the OTHER endpoint (broken
  // lemniscate — the arcs cross visually at midpoint).
  const perp1 = perpOffset(from, to, size * 0.4);
  const perp2 = perpOffset(from, to, -size * 0.4);
  const straightMid = { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 };

  const drawT = t < drawEnd ? t / drawEnd : 1;
  let alpha: number;
  if (t < diamondEnd) alpha = 0.85;
  else {
    const fadeLocal = (t - diamondEnd) / (1 - diamondEnd);
    alpha = 0.85 * (1 - fadeLocal);
  }

  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.03;
  ctx.lineCap = "round";
  // Arc 1: from → to via perp1
  const ctrl1 = { x: straightMid.x + perp1.x, y: straightMid.y + perp1.y };
  const ctrl2 = { x: straightMid.x + perp2.x, y: straightMid.y + perp2.y };
  const steps = 22;
  const drawSteps = Math.max(1, Math.floor(steps * drawT));
  ctx.beginPath();
  for (let i = 0; i <= drawSteps; i++) {
    const tt = i / steps;
    const p = bezierPt(from, ctrl1, to, tt);
    if (i === 0) ctx.moveTo(p.x, p.y);
    else ctx.lineTo(p.x, p.y);
  }
  ctx.stroke();
  ctx.beginPath();
  for (let i = 0; i <= drawSteps; i++) {
    const tt = i / steps;
    const p = bezierPt(to, ctrl2, from, tt);
    if (i === 0) ctx.moveTo(p.x, p.y);
    else ctx.lineTo(p.x, p.y);
  }
  ctx.stroke();

  // Diamond glyph at the crossing (drawn midpoint), appears after arcs are
  // mostly drawn.
  if (t > drawEnd * 0.7 && t < diamondEnd + 0.15) {
    const dLocal = Math.min(1, (t - drawEnd * 0.7) / ((diamondEnd + 0.15) - drawEnd * 0.7));
    const dAlpha = alpha * dLocal;
    const dSize = size * 0.08 * Math.min(1, dLocal * 1.3);
    ctx.fillStyle = withAlpha(color, dAlpha);
    ctx.strokeStyle = withAlpha(color, dAlpha);
    ctx.lineWidth = size * 0.02;
    ctx.beginPath();
    ctx.moveTo(straightMid.x, straightMid.y - dSize);
    ctx.lineTo(straightMid.x + dSize, straightMid.y);
    ctx.lineTo(straightMid.x, straightMid.y + dSize);
    ctx.lineTo(straightMid.x - dSize, straightMid.y);
    ctx.closePath();
    ctx.fill();
  }
};

// === Skill 13: Retreat — the pull-back ======================================
// Piece slides with dust trail (already handled elsewhere); we add a short
// trailing "arrow-tail" at the origin square.

const RETREAT_TTL = 440;

const renderRetreat: SkillRenderer = (ctx, eff, age, size) => {
  if (age >= RETREAT_TTL) return;
  const t = age / RETREAT_TTL;
  const color = skillColor(eff.skillId);
  const from = squareCenter(eff.from, size);
  const to = squareCenter(eff.to, size);
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len, uy = dy / len;

  // Three parallel ticks perpendicular to the retreat direction, positioned
  // just past the origin along the retreat path. They fade quickly, giving
  // a "getting out of there" read.
  const drawEnd = 100 / RETREAT_TTL;

  let drawT: number;
  let alpha: number;
  if (t < drawEnd) {
    drawT = t / drawEnd;
    alpha = 0.85;
  } else {
    drawT = 1;
    const fadeLocal = (t - drawEnd) / (1 - drawEnd);
    alpha = 0.85 * (1 - fadeLocal);
  }

  ctx.strokeStyle = withAlpha(color, alpha);
  ctx.lineWidth = size * 0.025;
  ctx.lineCap = "round";
  const perpXn = -uy, perpYn = ux;
  const tickLen = size * 0.08;
  for (let i = 0; i < 3; i++) {
    // Ticks sit at 15%, 25%, 35% along the retreat path.
    const along = 0.15 + i * 0.1;
    const baseX = from.x + ux * len * along;
    const baseY = from.y + uy * len * along;
    const currentLen = tickLen * drawT;
    ctx.beginPath();
    ctx.moveTo(baseX - perpXn * currentLen, baseY - perpYn * currentLen);
    ctx.lineTo(baseX + perpXn * currentLen, baseY + perpYn * currentLen);
    ctx.stroke();
  }
};

// Registry keyed by skill id. Undefined entries fall through to renderFallback.
const SKILL_RENDERERS: Record<number, SkillRenderer> = {
  1: renderLance,
  2: renderHook,
  3: renderBreak,
  4: renderSteal,
  5: renderTempest,
  6: renderShieldSelf,
  7: renderHeal,
  8: renderPlate,
  9: renderFallback,   // dash — dust already covers it
  10: renderBlast,
  11: renderShove,
  12: renderSwap,
  13: renderRetreat,
  14: renderFocus,
  15: renderCharge,
};
