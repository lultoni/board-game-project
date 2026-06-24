// Placeholder SFX. WebAudio synthesis only — no asset files. Each event is
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
  | "pickup"        // piece selected / lifted
  | "drop"          // piece dropped without moving (release back into place)
  | "wheelOpen"    // skill wheel opens
  | "phaseEnd";     // phase or turn boundary

export interface PlayOpts {
  /** For move/attack: tiles travelled. Stretches the release. */
  tiles?: number;
}

let ctx: AudioContext | null = null;
let master: GainNode | null = null;

function ensureCtx(): AudioContext | null {
  if (ctx) return ctx;
  if (typeof window === "undefined") return null;
  try {
    const C = window.AudioContext ?? (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    ctx = new C();
    master = ctx.createGain();
    master.gain.value = settings.audioVolume;
    master.connect(ctx.destination);
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
  const c = ensureCtx();
  if (!c || !master) return;
  if (settings.audioVolume <= 0) return;
  const t0 = c.currentTime;
  const dur = spec.attack + spec.release;

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
  // Click: short paper-tap.
  click: () => playTone({
    freq: 1400, glideTo: 1200, type: "square",
    attack: 0.002, release: 0.05, gain: 0.08, noise: 0.4,
  }),
  // Pickup: soft paper-lift — short low triangle with a tiny rise.
  pickup: () => playTone({
    freq: 320, glideTo: 420, type: "triangle",
    attack: 0.004, release: 0.08, gain: 0.12, noise: 0.18,
  }),
  // Drop: short low thud — release of a held piece without moving.
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
  // Phase end: soft page-turn — gentle paper rustle, no harsh edge.
  // Low triangle with a brief breath of high-pass noise, no second voice.
  phaseEnd: () => playTone({
    freq: 200, glideTo: 140, type: "sine",
    attack: 0.03, release: 0.32, gain: 0.12, noise: 0.32,
  }),
};

export function play(event: SfxEvent, opts?: PlayOpts): void {
  const voice = VOICES[event];
  if (voice) voice(opts);
}

export const sfx = { play, unlock: unlockAudio, applyVolume: applyMasterVolume };
