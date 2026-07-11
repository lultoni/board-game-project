// Persisted user settings. Backed by localStorage with a JSON blob under
// the key `game-settings`. Defaults apply when no entry exists. Each rune
// write triggers a persist via $effect at the call site.

import { setLocale } from "./i18n";

const STORAGE_KEY = "game-settings";


export type AnimationSpeed = "off" | "normal" | "fast" | "cinematic";

export interface Settings {
  /** Always-on: paint tiles where the skill can legally be cast. */
  showLegalTargets: boolean;
  /** Path tiles for projectile skills (Lance, Hook, Break, ...). */
  showProjectilePath: boolean;
  /** Tiles inside the skill's geometric range but with a wrong-owner
   *  piece (e.g. enemy skill range over an empty tile). Teaching aid. */
  showIllegalOwner: boolean;
  /** Projectile paths that are blocked by a friendly piece (red X). */
  showBlockedByFriendly: boolean;
  /** Always-on: mark the tile edges from which a Champion/King is currently
   *  protected by a friendly Guard (Bodyguard Rule). Teaching / legibility aid. */
  showBodyguardCover: boolean;
  /** Master audio volume, 0..1. */
  audioVolume: number;
  /** UI language. Falls back to en when key missing. */
  locale: "en" | "de";
  /** AI think budgets (used when a seat is AI). Mirror Rust AiBudget defaults. */
  p1ThinkTimeMs: number;
  p2ThinkTimeMs: number;
  p1MaxDepth: number;
  p2MaxDepth: number;
  /** Delay between AI plies in AIvAI mode (ms). Frontend-paced; engine does
   *  not sleep on its own. */
  aivaiStepDelayMs: number;
  /** Per-seat evaluator pick. `source` distinguishes built-in heuristic vs
   *  a trained rater from a run dir (`"run"`) or the curated `"blessed"`
   *  collection. `id` names the rater within that source; ignored when
   *  source is `"heuristic"`. */
  p1Evaluator: EvaluatorChoice;
  p2Evaluator: EvaluatorChoice;
  /** Piece slide animation speed. "off" disables transitions entirely. */
  animationSpeed: AnimationSpeed;
  /** Replay step delay, independent of aivaiStepDelayMs. */
  replayStepDelayMs: number;
  /** Loop replay to start when it reaches the end. */
  replayLoopOnEnd: boolean;
  /** Wait for the piece animation to finish before the next AI ply / replay
   *  step advances. Applies globally to AI turns and replay auto-play. When
   *  false, AI/replay proceed on their own delay floors regardless of how
   *  long the piece walk + lunge takes. */
  respectAnimation: boolean;
  /** Show depth counter alongside AI spinner. */
  showAiDepth: boolean;
  /** Show the per-seat AI think-progress bar (rAF-driven fill under each
   *  PlayerPanel). Turn off to skip the rAF loop entirely when the bar is
   *  hidden — noticeable on lower-end machines with many panels visible. */
  showThinkProgressBar: boolean;
  /** Show heuristic eval bar + score in the match UI. */
  showHeuristicEval: boolean;
  /** Show the full heuristic eval breakdown side panel (per-component,
   *  per-side, colour-coded). Intended for analysis / tuning. Independent
   *  of `showHeuristicEval` — both may be on. */
  showEvalPanel: boolean;
}

export type EvaluatorSource = "heuristic" | "run" | "blessed";

export interface EvaluatorChoice {
  source: EvaluatorSource;
  id: string | null;
}

const ANIMATION_SPEEDS: ReadonlyArray<AnimationSpeed> = ["off", "normal", "fast", "cinematic"];

const DEFAULTS: Settings = {
  showLegalTargets: true,
  showProjectilePath: true,
  showIllegalOwner: false,
  showBlockedByFriendly: true,
  showBodyguardCover: true,
  audioVolume: 0.6,
  locale: "en",
  p1ThinkTimeMs: 1000,
  p2ThinkTimeMs: 1000,
  p1MaxDepth: 6,
  p2MaxDepth: 6,
  aivaiStepDelayMs: 300,
  p1Evaluator: { source: "heuristic", id: null },
  p2Evaluator: { source: "heuristic", id: null },
  animationSpeed: "normal",
  replayStepDelayMs: 300,
  replayLoopOnEnd: false,
  respectAnimation: true,
  showAiDepth: true,
  showThinkProgressBar: false,
  showHeuristicEval: false,
  showEvalPanel: false,
};

const EVAL_SOURCES: ReadonlyArray<EvaluatorSource> = ["heuristic", "run", "blessed"];

const LOCALES: ReadonlyArray<Settings["locale"]> = ["en", "de"];

// Per-field validation. localStorage is user-writable, so a tampered or
// stale blob can carry NaN, negative budgets, or unknown locale strings
// that the engine then has to defend against downstream. Each picker
// silently falls back to DEFAULTS on a bad value — no error surfacing,
// since the user didn't ask for this read to fail.
function pickBool(v: unknown, fallback: boolean): boolean {
  return typeof v === "boolean" ? v : fallback;
}
function pickFiniteNonNeg(v: unknown, fallback: number): number {
  return typeof v === "number" && Number.isFinite(v) && v >= 0 ? v : fallback;
}
function pickClamped01(v: unknown, fallback: number): number {
  return typeof v === "number" && Number.isFinite(v)
    ? Math.min(1, Math.max(0, v))
    : fallback;
}
function pickPosInt(v: unknown, fallback: number): number {
  return typeof v === "number" && Number.isInteger(v) && v > 0 ? v : fallback;
}
function pickNonNegInt(v: unknown, fallback: number): number {
  return typeof v === "number" && Number.isInteger(v) && v >= 0 ? v : fallback;
}
function pickLocale(v: unknown, fallback: Settings["locale"]): Settings["locale"] {
  return typeof v === "string" && (LOCALES as readonly string[]).includes(v)
    ? (v as Settings["locale"])
    : fallback;
}
function pickAnimationSpeed(v: unknown, fallback: AnimationSpeed): AnimationSpeed {
  return typeof v === "string" && (ANIMATION_SPEEDS as readonly string[]).includes(v)
    ? (v as AnimationSpeed)
    : fallback;
}
function pickEvaluator(v: unknown, fallback: EvaluatorChoice): EvaluatorChoice {
  if (!v || typeof v !== "object") return { ...fallback };
  const o = v as Record<string, unknown>;
  const source: EvaluatorSource =
    typeof o.source === "string" && (EVAL_SOURCES as readonly string[]).includes(o.source)
      ? (o.source as EvaluatorSource)
      : fallback.source;
  const id = typeof o.id === "string" && o.id.length > 0 ? o.id : null;
  // Heuristic ignores id; non-heuristic without an id falls back.
  if (source === "heuristic") return { source: "heuristic", id: null };
  if (id === null) return { ...fallback };
  return { source, id };
}

function validate(raw: unknown): Settings {
  if (!raw || typeof raw !== "object") return { ...DEFAULTS };
  const r = raw as Record<string, unknown>;
  return {
    showLegalTargets: pickBool(r.showLegalTargets, DEFAULTS.showLegalTargets),
    showProjectilePath: pickBool(r.showProjectilePath, DEFAULTS.showProjectilePath),
    showIllegalOwner: pickBool(r.showIllegalOwner, DEFAULTS.showIllegalOwner),
    showBlockedByFriendly: pickBool(r.showBlockedByFriendly, DEFAULTS.showBlockedByFriendly),
    showBodyguardCover: pickBool(r.showBodyguardCover, DEFAULTS.showBodyguardCover),
    audioVolume: pickClamped01(r.audioVolume, DEFAULTS.audioVolume),
    locale: pickLocale(r.locale, DEFAULTS.locale),
    p1ThinkTimeMs: pickFiniteNonNeg(r.p1ThinkTimeMs, DEFAULTS.p1ThinkTimeMs),
    p2ThinkTimeMs: pickFiniteNonNeg(r.p2ThinkTimeMs, DEFAULTS.p2ThinkTimeMs),
    p1MaxDepth: pickNonNegInt(r.p1MaxDepth, DEFAULTS.p1MaxDepth),
    p2MaxDepth: pickNonNegInt(r.p2MaxDepth, DEFAULTS.p2MaxDepth),
    aivaiStepDelayMs: pickFiniteNonNeg(r.aivaiStepDelayMs, DEFAULTS.aivaiStepDelayMs),
    p1Evaluator: pickEvaluator(r.p1Evaluator, DEFAULTS.p1Evaluator),
    p2Evaluator: pickEvaluator(r.p2Evaluator, DEFAULTS.p2Evaluator),
    animationSpeed: pickAnimationSpeed(r.animationSpeed, DEFAULTS.animationSpeed),
    replayStepDelayMs: pickFiniteNonNeg(r.replayStepDelayMs, DEFAULTS.replayStepDelayMs),
    replayLoopOnEnd: pickBool(r.replayLoopOnEnd, DEFAULTS.replayLoopOnEnd),
    // Backwards-compat: old blobs used `replayRespectAnimation` (replay-only).
    // Fall through to the old key so existing users keep their preference.
    respectAnimation: pickBool(
      r.respectAnimation ?? r.replayRespectAnimation,
      DEFAULTS.respectAnimation,
    ),
    showAiDepth: pickBool(r.showAiDepth, DEFAULTS.showAiDepth),
    showThinkProgressBar: pickBool(r.showThinkProgressBar, DEFAULTS.showThinkProgressBar),
    showHeuristicEval: pickBool(r.showHeuristicEval, DEFAULTS.showHeuristicEval),
    showEvalPanel: pickBool(r.showEvalPanel, DEFAULTS.showEvalPanel),
  };
}

function load(): Settings {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    return validate(JSON.parse(raw));
  } catch {
    return { ...DEFAULTS };
  }
}

// Exported for tests; not part of the public API.
export const _validateSettings = validate;

export const settings = $state<Settings>(load());

// Apply the persisted locale to the i18n layer at module load, before any
// component renders — otherwise the first paint would be English regardless
// of the saved preference. initSettingsPersistence() keeps it in sync after.
setLocale(settings.locale);

function persist() {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    /* quota / private mode — ignore */
  }
}

/** ms for the piece slide transition at each speed tier. */
export const SLIDE_DURATION_MS: Record<AnimationSpeed, number> = {
  off: 0,
  fast: 140,
  normal: 280,
  cinematic: 700,
};

/** ms to wait after renderApplied before the caller may advance (AIvAI / replay).
 *  Gives the CSS transition time to complete. Zero when animations are off. */
export function slideDurationMs(): number {
  return SLIDE_DURATION_MS[settings.animationSpeed];
}

/** Multiplier applied to baseline FX_LIFETIME_MS values so per-skill choreography,
 *  impact rings, damage numbers, spotlights, etc. all scale with the user's
 *  animation-speed pick.
 *
 *  Ratios chosen to match the walk-speed ratios (slide-duration / 280 ms normal):
 *    off       0        → skip animations entirely
 *    fast      0.5      → half-time
 *    normal    1        → baseline
 *    cinematic 2.5      → slow, exaggerated flourishes matching the slow walk
 *
 *  `off` returns 0 so callers can shortcut and simply skip pushing the effect
 *  (a zero-lifetime effect would expire on the first frame anyway). */
export function fxSpeedMultiplier(): number {
  switch (settings.animationSpeed) {
    case "off": return 0;
    case "fast": return 0.5;
    case "normal": return 1;
    case "cinematic": return 2.5;
  }
}
export function initSettingsPersistence() {
  $effect(() => {
    // Touch every key so $effect tracks them all.
    void settings.showLegalTargets;
    void settings.showProjectilePath;
    void settings.showIllegalOwner;
    void settings.showBlockedByFriendly;
    void settings.showBodyguardCover;
    void settings.audioVolume;
    void settings.locale;
    void settings.p1ThinkTimeMs;
    void settings.p2ThinkTimeMs;
    void settings.p1MaxDepth;
    void settings.p2MaxDepth;
    void settings.aivaiStepDelayMs;
    void settings.p1Evaluator;
    void settings.p2Evaluator;
    void settings.animationSpeed;
    void settings.replayStepDelayMs;
    void settings.replayLoopOnEnd;
    void settings.respectAnimation;
    void settings.showAiDepth;
    void settings.showThinkProgressBar;
    void settings.showHeuristicEval;
    void settings.showEvalPanel;
    // Mirror the persisted locale into the i18n layer so switching the
    // language dropdown re-renders all t() text live.
    setLocale(settings.locale);
    persist();
  });
}
