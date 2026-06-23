<script lang="ts">
  import { onMount } from "svelte";
  import { type Effect, FX_LIFETIME_MS } from "$lib/viz/effects";

  interface Props {
    /** SVG viewBox edge in pixels. Same value as Board. */
    viewBox: number;
    /** Effect queue — drained internally as effects expire. */
    queue: Effect[];
  }

  let { viewBox, queue = $bindable() }: Props = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let raf = 0;

  // Dust particles per dust effect.
  type Particle = { x: number; y: number; vx: number; vy: number; life: number; max: number; size: number };
  const particlesByEffect = new WeakMap<Effect, Particle[]>();

  function squareCenter(sq: number, size: number): { x: number; y: number } {
    const file = sq & 7;
    const rank = (sq >> 3) & 7;
    return { x: file * size + size / 2, y: (7 - rank) * size + size / 2 };
  }

  function ensureDustParticles(eff: Effect, size: number): Particle[] {
    let ps = particlesByEffect.get(eff);
    if (ps) return ps;
    ps = [];
    if (eff.kind !== "dust") return ps;
    // Sprinkle particles along each segment of the path.
    for (let i = 0; i < eff.path.length - 1; i++) {
      const a = squareCenter(eff.path[i], size);
      const b = squareCenter(eff.path[i + 1], size);
      const n = 7;
      for (let k = 0; k < n; k++) {
        const t = (k + 0.5) / n;
        const px = a.x + (b.x - a.x) * t + (Math.random() - 0.5) * size * 0.18;
        const py = a.y + (b.y - a.y) * t + (Math.random() - 0.5) * size * 0.18;
        ps.push({
          x: px,
          y: py,
          vx: (Math.random() - 0.5) * 6,
          vy: -Math.random() * 8 - 2,
          life: 0,
          max: 350 + Math.random() * 250,
          size: size * (0.05 + Math.random() * 0.04),
        });
      }
    }
    particlesByEffect.set(eff, ps);
    return ps;
  }

  function frame(now: number) {
    raf = requestAnimationFrame(frame);
    const c = canvas;
    if (!c) return;
    // Resize-aware: keep the canvas pixel-buffer matching the rendered size,
    // but draw in SVG-coordinate space so square sizes match.
    const rect = c.getBoundingClientRect();
    const dpr = window.devicePixelRatio ?? 1;
    const wantW = Math.max(1, Math.floor(rect.width * dpr));
    const wantH = Math.max(1, Math.floor(rect.height * dpr));
    if (c.width !== wantW || c.height !== wantH) {
      c.width = wantW;
      c.height = wantH;
    }
    const ctx = c.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, c.width, c.height);
    // Map SVG viewBox coords (0..viewBox+24) to canvas pixel coords.
    const scaleX = c.width / viewBox;
    const scaleY = c.height / (viewBox + 24);
    const scale = Math.min(scaleX, scaleY);
    ctx.scale(scale, scale);
    const size = viewBox / 8;

    if (queue.length === 0) return;

    // Iterate, render, drop expired.
    let writeIdx = 0;
    for (let i = 0; i < queue.length; i++) {
      const eff = queue[i];
      const age = now - eff.startedAt;
      const ttl = FX_LIFETIME_MS[eff.kind];
      const alive = age < ttl;
      if (alive) {
        renderEffect(ctx, eff, age, size);
        queue[writeIdx++] = eff;
      } else {
        // particle map auto-clears via WeakMap on GC
      }
    }
    queue.length = writeIdx;
  }

  function renderEffect(ctx: CanvasRenderingContext2D, eff: Effect, age: number, size: number) {
    if (eff.kind === "dust") {
      const ps = ensureDustParticles(eff, size);
      const dt = 16; // approx ms/frame (advance by a frame each draw)
      for (const p of ps) {
        p.life += dt;
        p.x += p.vx * (dt / 16);
        p.y += p.vy * (dt / 16);
        p.vy += 0.4; // gravity
        const a = Math.max(0, 1 - p.life / p.max);
        ctx.fillStyle = `rgba(140, 122, 88, ${a * 0.55})`;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fill();
      }
    } else if (eff.kind === "impact") {
      const t = age / FX_LIFETIME_MS.impact;
      const c = squareCenter(eff.at, size);
      const r = size * 0.18 + t * size * 0.45;
      ctx.strokeStyle = `rgba(196, 74, 58, ${(1 - t) * 0.9})`;
      ctx.lineWidth = size * (0.06 * (1 - t * 0.5));
      ctx.beginPath();
      ctx.arc(c.x, c.y, r, 0, Math.PI * 2);
      ctx.stroke();
      // Inner flash
      const innerA = Math.max(0, 1 - t * 2.2);
      if (innerA > 0) {
        const grd = ctx.createRadialGradient(c.x, c.y, 0, c.x, c.y, size * 0.4);
        grd.addColorStop(0, `rgba(255, 240, 200, ${innerA * 0.8})`);
        grd.addColorStop(1, "rgba(255, 240, 200, 0)");
        ctx.fillStyle = grd;
        ctx.beginPath();
        ctx.arc(c.x, c.y, size * 0.4, 0, Math.PI * 2);
        ctx.fill();
      }
    } else if (eff.kind === "damageNumber") {
      const t = age / FX_LIFETIME_MS.damageNumber;
      const c = squareCenter(eff.at, size);
      const y = c.y - size * 0.2 - t * size * 0.6;
      const a = Math.max(0, 1 - t);
      ctx.fillStyle = `rgba(204, 58, 42, ${a})`;
      ctx.strokeStyle = `rgba(28, 26, 23, ${a * 0.9})`;
      ctx.lineWidth = size * 0.04;
      ctx.font = `bold ${size * 0.36}px ui-rounded, system-ui, sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const text = `-${eff.amount}`;
      ctx.strokeText(text, c.x, y);
      ctx.fillText(text, c.x, y);
    }
  }

  onMount(() => {
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
</script>

<canvas bind:this={canvas} class="fx" aria-hidden="true"></canvas>

<style>
  .fx {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
