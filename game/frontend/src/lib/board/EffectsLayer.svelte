<script lang="ts">
  import { onMount } from "svelte";
  import { type Effect, FX_LIFETIME_MS } from "$lib/viz/effects";

  interface Props {
    /** SVG viewBox edge in pixels. Same value as Board. */
    viewBox: number;
    /** Padding around the 8×8 grid baked into the SVG's viewBox so the
     *  radial wheel can render outside the grid. Same value as Board's
     *  WHEEL_PAD. The canvas fills the SVG element's outer box, so we
     *  use this to map from grid-local coords to canvas pixels. */
    wheelPad?: number;
    /** Effect queue — read-only from this component's perspective. The
     *  caller (PlyRenderer) owns the array and resets it at match-reset
     *  boundaries. We track expiry internally via `expired` below so we
     *  never mutate the caller's `$state` array (which would trigger
     *  `ownership_invalid_mutation`). */
    queue: Effect[];
  }

  let { viewBox, wheelPad = 0, queue }: Props = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let raf: number | null = null;
  let running = false;

  // Dust particles per dust effect.
  type Particle = { x: number; y: number; vx: number; vy: number; life: number; max: number; size: number };
  const particlesByEffect = new WeakMap<Effect, Particle[]>();
  // Effects whose lifetime has elapsed. Skipped in future frames. WeakSet so
  // the caller's queue reset (queue.length = 0) drops the references and lets
  // GC reclaim entries here too — no manual cleanup needed.
  const expired = new WeakSet<Effect>();
  // Smallest index in `queue` that might still contain a live effect. Advanced
  // as prefix effects expire so we don't rescan them every frame. Reset to 0
  // whenever the caller shrinks the queue (match reset).
  let scanStart = 0;
  let lastLen = 0;

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
    raf = null;
    const c = canvas;
    if (!c) {
      running = false;
      return;
    }
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
    if (!ctx) {
      running = false;
      return;
    }
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, c.width, c.height);
    // Map the SVG's outer viewBox (which includes wheelPad on each side)
    // to canvas pixel coords, then translate so grid (0,0) lines up with
    // the visible board origin.
    const outerW = viewBox + 2 * wheelPad;
    const outerH = viewBox + 24 + 2 * wheelPad;
    const scaleX = c.width / outerW;
    const scaleY = c.height / outerH;
    const scale = Math.min(scaleX, scaleY);
    ctx.scale(scale, scale);
    ctx.translate(wheelPad, wheelPad);
    const size = viewBox / 8;

    // Detect caller reset (length shrank). Reset our scan cursor so the
    // next batch starts at 0.
    if (queue.length < lastLen) scanStart = 0;
    lastLen = queue.length;

    // Iterate, render, mark expired. Do NOT mutate `queue` — it's the
    // caller's $state array; mutating it here triggers Svelte's
    // ownership_invalid_mutation. The caller resets it at match boundaries.
    let alive = 0;
    let advancePrefix = true;
    for (let i = scanStart; i < queue.length; i++) {
      const eff = queue[i];
      if (expired.has(eff)) {
        // Contiguous-expired prefix can be skipped forever in later frames.
        if (advancePrefix) scanStart = i + 1;
        continue;
      }
      advancePrefix = false;
      const age = now - eff.startedAt;
      const ttl = FX_LIFETIME_MS[eff.kind];
      if (age < ttl) {
        renderEffect(ctx, eff, age, size);
        alive++;
      } else {
        expired.add(eff);
        // particle map auto-clears via WeakMap on GC
      }
    }

    // P3: re-schedule only while there's work to do. The reactive $effect
    // below restarts the loop when the queue receives a new push.
    if (alive > 0) {
      raf = requestAnimationFrame(frame);
    } else {
      running = false;
    }
  }

  function start() {
    if (running) return;
    running = true;
    raf = requestAnimationFrame(frame);
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
    } else if (eff.kind === "heal") {
      // Soft green ring + + glyph rising. Lifetime 720ms.
      const ttl = FX_LIFETIME_MS.heal;
      const t = age / ttl;
      const c = squareCenter(eff.at, size);
      // Expanding ring.
      const r = size * 0.18 + t * size * 0.42;
      ctx.strokeStyle = `rgba(80, 158, 96, ${(1 - t) * 0.85})`;
      ctx.lineWidth = size * (0.07 * (1 - t * 0.5));
      ctx.beginPath();
      ctx.arc(c.x, c.y, r, 0, Math.PI * 2);
      ctx.stroke();
      // Soft inner glow.
      const innerA = Math.max(0, 1 - t * 1.8);
      if (innerA > 0) {
        const grd = ctx.createRadialGradient(c.x, c.y, 0, c.x, c.y, size * 0.42);
        grd.addColorStop(0, `rgba(170, 230, 180, ${innerA * 0.65})`);
        grd.addColorStop(1, "rgba(170, 230, 180, 0)");
        ctx.fillStyle = grd;
        ctx.beginPath();
        ctx.arc(c.x, c.y, size * 0.42, 0, Math.PI * 2);
        ctx.fill();
      }
      // Rising "+" glyph (or +N if amount > 1).
      const a = Math.max(0, 1 - t);
      const y = c.y - size * 0.18 - t * size * 0.5;
      ctx.fillStyle = `rgba(56, 124, 70, ${a})`;
      ctx.strokeStyle = `rgba(28, 26, 23, ${a * 0.8})`;
      ctx.lineWidth = size * 0.04;
      ctx.font = `bold ${size * 0.34}px ui-rounded, system-ui, sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const text = eff.amount > 1 ? `+${eff.amount}` : "+";
      ctx.strokeText(text, c.x, y);
      ctx.fillText(text, c.x, y);
    } else if (eff.kind === "armor") {
      // Steel-blue shield rune that pulses then fades. Negative amount =
      // armor stripped (Break) → coppery flash instead of blue.
      const ttl = FX_LIFETIME_MS.armor;
      const t = age / ttl;
      const c = squareCenter(eff.at, size);
      const positive = eff.amount > 0;
      const stroke = positive
        ? `rgba(94, 130, 168, ${(1 - t) * 0.9})`
        : `rgba(176, 96, 48, ${(1 - t) * 0.9})`;
      const fill = positive ? "rgba(160, 190, 220, " : "rgba(220, 168, 124, ";
      // Hexagonal shield outline expanding outward.
      const r = size * 0.22 + t * size * 0.18;
      ctx.strokeStyle = stroke;
      ctx.lineWidth = size * (0.07 * (1 - t * 0.4));
      ctx.beginPath();
      for (let i = 0; i < 6; i++) {
        const ang = (Math.PI / 3) * i - Math.PI / 2;
        const px = c.x + Math.cos(ang) * r;
        const py = c.y + Math.sin(ang) * r;
        if (i === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
      ctx.closePath();
      ctx.stroke();
      // Soft fill flash.
      const innerA = Math.max(0, 1 - t * 1.8);
      if (innerA > 0) {
        const grd = ctx.createRadialGradient(c.x, c.y, 0, c.x, c.y, size * 0.35);
        grd.addColorStop(0, `${fill}${innerA * 0.5})`);
        grd.addColorStop(1, `${fill}0)`);
        ctx.fillStyle = grd;
        ctx.beginPath();
        ctx.arc(c.x, c.y, size * 0.35, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }

  onMount(() => {
    // Initial RAF kick only if there's already content. The $effect below
    // handles the common case (queue receives push → loop starts).
    if (queue.length > 0) start();
    return () => {
      if (raf !== null) cancelAnimationFrame(raf);
      raf = null;
      running = false;
    };
  });

  // Reactive restart: any time the queue length transitions from 0 → N (a
  // producer pushed an effect), kick the RAF if it isn't already running.
  // Reads `queue.length` so Svelte 5 tracks the $state-backed array.
  $effect(() => {
    if (queue.length > 0 && !running) start();
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
