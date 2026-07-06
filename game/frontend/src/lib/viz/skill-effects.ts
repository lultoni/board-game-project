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

// Registry keyed by skill id. Undefined entries fall through to renderFallback.
const SKILL_RENDERERS: Record<number, SkillRenderer> = {
  1: renderLance,
  2: renderHook,
  3: renderFallback,   // break
  4: renderFallback,   // steal
  5: renderFallback,   // tempest
  6: renderShieldSelf,
  7: renderFallback,   // heal
  8: renderFallback,   // plate
  9: renderFallback,   // dash (dust already covers)
  10: renderFallback,  // blast
  11: renderFallback,  // shove
  12: renderFallback,  // swap
  13: renderFallback,  // retreat
  14: renderFocus,
  15: renderCharge,
};
