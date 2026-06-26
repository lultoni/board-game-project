// Persisted user settings. Backed by localStorage with a JSON blob under
// the key `game-settings`. Defaults apply when no entry exists. Each rune
// write triggers a persist via $effect at the call site.

const STORAGE_KEY = "game-settings";

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
}

const DEFAULTS: Settings = {
  showLegalTargets: true,
  showProjectilePath: true,
  showIllegalOwner: false,
  showBlockedByFriendly: true,
  audioVolume: 0.6,
  locale: "en",
  p1ThinkTimeMs: 1000,
  p2ThinkTimeMs: 1000,
  p1MaxDepth: 6,
  p2MaxDepth: 6,
  aivaiStepDelayMs: 300,
};

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
function pickLocale(v: unknown, fallback: Settings["locale"]): Settings["locale"] {
  return typeof v === "string" && (LOCALES as readonly string[]).includes(v)
    ? (v as Settings["locale"])
    : fallback;
}

function validate(raw: unknown): Settings {
  if (!raw || typeof raw !== "object") return { ...DEFAULTS };
  const r = raw as Record<string, unknown>;
  return {
    showLegalTargets: pickBool(r.showLegalTargets, DEFAULTS.showLegalTargets),
    showProjectilePath: pickBool(r.showProjectilePath, DEFAULTS.showProjectilePath),
    showIllegalOwner: pickBool(r.showIllegalOwner, DEFAULTS.showIllegalOwner),
    showBlockedByFriendly: pickBool(r.showBlockedByFriendly, DEFAULTS.showBlockedByFriendly),
    audioVolume: pickClamped01(r.audioVolume, DEFAULTS.audioVolume),
    locale: pickLocale(r.locale, DEFAULTS.locale),
    p1ThinkTimeMs: pickFiniteNonNeg(r.p1ThinkTimeMs, DEFAULTS.p1ThinkTimeMs),
    p2ThinkTimeMs: pickFiniteNonNeg(r.p2ThinkTimeMs, DEFAULTS.p2ThinkTimeMs),
    p1MaxDepth: pickPosInt(r.p1MaxDepth, DEFAULTS.p1MaxDepth),
    p2MaxDepth: pickPosInt(r.p2MaxDepth, DEFAULTS.p2MaxDepth),
    aivaiStepDelayMs: pickFiniteNonNeg(r.aivaiStepDelayMs, DEFAULTS.aivaiStepDelayMs),
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

function persist() {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    /* quota / private mode — ignore */
  }
}

/** Call from a +layout.svelte $effect to start auto-persisting on changes. */
export function initSettingsPersistence() {
  $effect(() => {
    // Touch every key so $effect tracks them all.
    void settings.showLegalTargets;
    void settings.showProjectilePath;
    void settings.showIllegalOwner;
    void settings.showBlockedByFriendly;
    void settings.audioVolume;
    void settings.locale;
    void settings.p1ThinkTimeMs;
    void settings.p2ThinkTimeMs;
    void settings.p1MaxDepth;
    void settings.p2MaxDepth;
    void settings.aivaiStepDelayMs;
    persist();
  });
}
