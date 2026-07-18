// Placeholder SFX. WebAudio synthesis only - no asset files. Each event is
// a short procedural tone shaped to roughly fit its on-screen meaning. The
// API is Howler-shaped (`play("name")`) so swapping in real foley later is
// a one-file change.
//
// The AudioContext is created lazily on the first `play()` call because
// browsers require a user-gesture before they'll start one. Until that
// first gesture, `play()` is a silent no-op.

import { settings } from "$lib/state/settings.svelte";

export type SfxEvent =
  | "move"
  | "attack"        // move-attack landing on enemy
  | "damage"        // skill-damage hit
  | "heal"          // hp restored
  | "armor"         // armor gained
  | "armorBreak"    // armor stripped
  | "death"         // piece removed
  | "skillFire"     // any skill cast (button-press timbre)
  | "click"         // wheel / button click
  | "tick"          // slider drag / select change (lighter than click)
  | "pickup"        // piece selected / lifted
  | "drop"          // piece dropped without moving (release back into place)
  | "wheelOpen"     // skill wheel opens
  | "phaseEnd"      // phase or turn boundary
  | "victory"       // game won by this client
  | "defeat"        // game lost by this client
  | "gameEnd"       // neutral game-end (draw, or used when side unknown)
  | "sandboxEnter"  // entering sandbox / analysis mode
  | "draftPick";    // skill placed onto a piece slot in the draft

export interface PlayOpts {
  /** For move/attack: tiles travelled. Stretches the release. */
  tiles?: number;
}

let ctx: AudioContext | null = null;
let master: GainNode | null = null;
// Last time we auto-called ctx.resume() in response to a `statechange` event.
// WKWebView on macOS will occasionally suspend the context silently after a
// long idle window; the `statechange` listener re-resumes but we back off to
// avoid a tight loop if resume() itself keeps failing.
let lastAutoResumeMs = 0;

function ensureCtx(): AudioContext | null {
  if (ctx) return ctx;
  if (typeof window === "undefined") return null;
  try {
    const C = window.AudioContext ?? (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    ctx = new C();
    master = ctx.createGain();
    master.gain.value = settings.audioVolume;
    master.connect(ctx.destination);
    // WKWebView (Tauri on macOS) can transition the context to "suspended"
    // after long inactivity without firing `visibilitychange`. Watch for it
    // and re-resume while the tab is visible.
    ctx.addEventListener("statechange", () => {
      if (!ctx) return;
      if (ctx.state !== "suspended") return;
      if (typeof document !== "undefined" && document.visibilityState !== "visible") return;
      const now = Date.now();
      if (now - lastAutoResumeMs < 1000) return;
      lastAutoResumeMs = now;
      void ctx.resume().catch(() => { /* wait for next user gesture */ });
    });
    return ctx;
  } catch {
    return null;
  }
}

/** Resume the context after the first user gesture. Call from a one-shot
 *  pointerdown / keydown handler at the top of `match` / `+layout`. */
export function unlockAudio(): void {
  const c = ensureCtx();
  if (c && c.state === "suspended") {
    void c.resume();
  }
}

/** Push the current settings.audioVolume to the master gain. Call after
 *  settings change. (Settings persistence already wires this via $effect
 *  in the layout.) */
export function applyMasterVolume(): void {
  if (master) master.gain.value = settings.audioVolume;
}

// --- Voice helpers ---------------------------------------------------------

interface ToneSpec {
  /** Carrier frequency in Hz at attack peak. */
  freq: number;
  /** Optional pitch glide (Hz) at the end of the envelope. */
  glideTo?: number;
  /** Oscillator waveform. */
  type: OscillatorType;
  /** Attack time, seconds. */
  attack: number;
  /** Release time, seconds. */
  release: number;
  /** Peak gain (multiplied by master). */
  gain: number;
  /** Optional band of noise mixed in at the same envelope (gain 0..1). */
  noise?: number;
  /** Optional second voice (e.g. a fifth above) at the given relative gain. */
  voice2?: { ratio: number; gain: number };
}

function playTone(spec: ToneSpec): void {
  playToneAt(spec, 0);
}

function playToneAt(spec: ToneSpec, delaySeconds: number): void {
  const c = ensureCtx();
  if (!c || !master) return;
  if (settings.audioVolume <= 0) return;
  // If the context suspended itself (WKWebView idle behaviour), wake it up
  // before scheduling. resume() is idempotent on a running context. The
  // first tone after resume may drop ~30 ms while the context spins up.
  if (c.state === "suspended") {
    void c.resume().catch(() => { /* fall through - schedule anyway */ });
  }
  const t0 = c.currentTime + delaySeconds;
  const dur = spec.attack + spec.release;

  // Every voice bundles its nodes into a single chain; the source's `ended`
  // event tears the chain down. Without this, stopped oscillators / gains /
  // biquads sit in the AudioContext graph forever, and long sessions leak
  // hundreds of zombie nodes.
  function tearDown(chain: AudioNode[]): void {
    for (const n of chain) {
      try { n.disconnect(); } catch { /* already disconnected */ }
    }
  }

  const env = c.createGain();
  env.gain.setValueAtTime(0, t0);
  env.gain.linearRampToValueAtTime(spec.gain, t0 + spec.attack);
  env.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
  env.connect(master);

  const osc = c.createOscillator();
  osc.type = spec.type;
  osc.frequency.setValueAtTime(spec.freq, t0);
  if (spec.glideTo !== undefined) {
    osc.frequency.exponentialRampToValueAtTime(Math.max(20, spec.glideTo), t0 + dur);
  }
  osc.connect(env);
  osc.start(t0);
  osc.stop(t0 + dur + 0.02);
  const oscChain: AudioNode[] = [osc, env];
  osc.addEventListener("ended", () => tearDown(oscChain));

  if (spec.voice2) {
    const osc2 = c.createOscillator();
    osc2.type = spec.type;
    osc2.frequency.setValueAtTime(spec.freq * spec.voice2.ratio, t0);
    if (spec.glideTo !== undefined) {
      osc2.frequency.exponentialRampToValueAtTime(
        Math.max(20, spec.glideTo * spec.voice2.ratio), t0 + dur,
      );
    }
    const v2env = c.createGain();
    v2env.gain.setValueAtTime(0, t0);
    v2env.gain.linearRampToValueAtTime(spec.gain * spec.voice2.gain, t0 + spec.attack);
    v2env.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
    v2env.connect(master);
    osc2.connect(v2env);
    osc2.start(t0);
    osc2.stop(t0 + dur + 0.02);
    const osc2Chain: AudioNode[] = [osc2, v2env];
    osc2.addEventListener("ended", () => tearDown(osc2Chain));
  }

  if (spec.noise !== undefined && spec.noise > 0) {
    const noiseLen = Math.ceil(c.sampleRate * dur);
    const buf = c.createBuffer(1, noiseLen, c.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < noiseLen; i++) data[i] = Math.random() * 2 - 1;
    const src = c.createBufferSource();
    src.buffer = buf;
    const nGain = c.createGain();
    nGain.gain.setValueAtTime(0, t0);
    nGain.gain.linearRampToValueAtTime(spec.gain * spec.noise, t0 + spec.attack);
    nGain.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
    // High-pass to keep noise crisp not rumbly.
    const hp = c.createBiquadFilter();
    hp.type = "highpass";
    hp.frequency.value = 800;
    src.connect(hp);
    hp.connect(nGain);
    nGain.connect(master);
    src.start(t0);
    src.stop(t0 + dur + 0.02);
    const noiseChain: AudioNode[] = [src, hp, nGain];
    src.addEventListener("ended", () => tearDown(noiseChain));
  }
}

// --- Event → tone mapping --------------------------------------------------

const VOICES: Record<SfxEvent, (opts?: PlayOpts) => void> = {
  // Soft wood-on-paper slide. Length scales with distance so a 1-tile move
  // is a single short thump and a 7-tile slide reads as a longer scrape.
  move: (opts) => {
    const tiles = Math.max(1, opts?.tiles ?? 1);
    // 1 tile ≈ 0.18s release; +0.12s per additional tile, capped at ~0.9s.
    const release = Math.min(0.9, 0.18 + (tiles - 1) * 0.12);
    playTone({
      freq: 180, glideTo: 90, type: "triangle",
      attack: 0.005, release, gain: 0.22, noise: 0.18,
    });
  },
  // Sharper landing thump with a little crack. Slight distance scaling for
  // the run-up (shorter than a plain slide because the impact dominates).
  attack: (opts) => {
    const tiles = Math.max(1, opts?.tiles ?? 1);
    const release = Math.min(0.65, 0.22 + (tiles - 1) * 0.08);
    playTone({
      freq: 240, glideTo: 80, type: "sawtooth",
      attack: 0.003, release, gain: 0.28, noise: 0.4,
    });
  },
  // Skill-damage: percussive snap.
  damage: () => playTone({
    freq: 520, glideTo: 200, type: "square",
    attack: 0.002, release: 0.18, gain: 0.22, noise: 0.45,
  }),
  // Heal: rising chime + soft fifth.
  heal: () => playTone({
    freq: 560, glideTo: 880, type: "sine",
    attack: 0.02, release: 0.45, gain: 0.18,
    voice2: { ratio: 1.5, gain: 0.6 },
  }),
  // Armor gain: metallic shimmer (high sine + noise).
  armor: () => playTone({
    freq: 980, glideTo: 1320, type: "sine",
    attack: 0.01, release: 0.35, gain: 0.18, noise: 0.18,
    voice2: { ratio: 2, gain: 0.4 },
  }),
  // Armor break: short metallic crack falling.
  armorBreak: () => playTone({
    freq: 1200, glideTo: 320, type: "sawtooth",
    attack: 0.003, release: 0.22, gain: 0.22, noise: 0.55,
  }),
  // Death: descending hollow boom.
  death: () => playTone({
    freq: 220, glideTo: 55, type: "triangle",
    attack: 0.01, release: 0.55, gain: 0.32, noise: 0.25,
    voice2: { ratio: 0.5, gain: 0.7 },
  }),
  // Skill fire: airy whoosh-blip.
  skillFire: () => playTone({
    freq: 420, glideTo: 720, type: "triangle",
    attack: 0.01, release: 0.18, gain: 0.17, noise: 0.22,
  }),
  // Click: short paper-tap. Mid-low pitch so it reads as tactile not tinny.
  click: () => playTone({
    freq: 580, glideTo: 420, type: "square",
    attack: 0.002, release: 0.055, gain: 0.1, noise: 0.35,
  }),
  // Tick: very short lighter tap for sliders/selects - quieter than click so
  // rapid drag doesn't feel like machine-gun fire.
  tick: () => playTone({
    freq: 480, glideTo: 380, type: "triangle",
    attack: 0.001, release: 0.03, gain: 0.06, noise: 0.2,
  }),
  // Pickup: soft paper-lift - short low triangle with a tiny rise.
  pickup: () => playTone({
    freq: 320, glideTo: 420, type: "triangle",
    attack: 0.004, release: 0.08, gain: 0.12, noise: 0.18,
  }),
  // Drop: short low thud - release of a held piece without moving.
  drop: () => playTone({
    freq: 220, glideTo: 140, type: "triangle",
    attack: 0.003, release: 0.1, gain: 0.14, noise: 0.22,
  }),
  // Wheel open: airy upward swoosh + soft fifth (decision moment).
  wheelOpen: () => playTone({
    freq: 380, glideTo: 720, type: "sine",
    attack: 0.012, release: 0.22, gain: 0.14, noise: 0.08,
    voice2: { ratio: 1.5, gain: 0.5 },
  }),
  // Phase end: soft page-turn - gentle paper rustle, no harsh edge.
  // Low triangle with a brief breath of high-pass noise, no second voice.
  phaseEnd: () => playTone({
    freq: 200, glideTo: 140, type: "sine",
    attack: 0.03, release: 0.32, gain: 0.12, noise: 0.32,
  }),

  // Victory: bright rising 3-note major arpeggio (root → major third → fifth).
  // Each note is a short sine with a harmonising fifth, staggered by 120ms.
  victory: () => {
    const base = 440; // A4
    const notes = [base, base * 1.25, base * 1.5]; // root, M3, P5
    notes.forEach((freq, i) => {
      playToneAt({
        freq, type: "sine",
        attack: 0.01, release: 0.55, gain: 0.16,
        voice2: { ratio: 1.5, gain: 0.45 },
      }, i * 0.13);
    });
    // Final shimmer: all three notes together 400ms later
    notes.forEach((freq, i) => {
      playToneAt({
        freq, type: "sine",
        attack: 0.015, release: 0.9, gain: 0.1,
        voice2: { ratio: 2, gain: 0.25 },
      }, 0.4 + i * 0.04);
    });
  },

  // Defeat: two descending minor notes - a falling minor third.
  // Hollow triangle, slow release for a mournful tone.
  defeat: () => {
    playToneAt({
      freq: 330, glideTo: 220, type: "triangle",
      attack: 0.02, release: 0.7, gain: 0.18,
      voice2: { ratio: 0.5, gain: 0.5 },
    }, 0);
    playToneAt({
      freq: 277, glideTo: 185, type: "triangle",
      attack: 0.025, release: 0.9, gain: 0.14,
      voice2: { ratio: 0.5, gain: 0.4 },
    }, 0.25);
  },

  // Game end (draw / result without known winner): neutral hollow chime.
  gameEnd: () => playTone({
    freq: 370, glideTo: 280, type: "sine",
    attack: 0.015, release: 0.65, gain: 0.14,
    voice2: { ratio: 1.333, gain: 0.4 },
  }),

  // Sandbox / analysis mode enter: a soft "mode-shift" - rising shimmer
  // suggesting stepping outside normal play. Airy sine with a high overtone.
  sandboxEnter: () => {
    playTone({
      freq: 480, glideTo: 780, type: "sine",
      attack: 0.02, release: 0.38, gain: 0.12, noise: 0.06,
      voice2: { ratio: 3, gain: 0.2 },
    });
    playToneAt({
      freq: 640, glideTo: 960, type: "sine",
      attack: 0.015, release: 0.28, gain: 0.08,
    }, 0.12);
  },

  // Draft pick: crisp card-placement snap. Short square transient + noise,
  // slightly higher pitch than a plain click to read as "something placed".
  draftPick: () => playTone({
    freq: 900, glideTo: 600, type: "square",
    attack: 0.002, release: 0.09, gain: 0.1, noise: 0.35,
  }),
};

export function play(event: SfxEvent, opts?: PlayOpts): void {
  const voice = VOICES[event];
  if (voice) voice(opts);
}

export const sfx = { play, unlock: unlockAudio, applyVolume: applyMasterVolume };
