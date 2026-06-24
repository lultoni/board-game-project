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

function load(): Settings {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw);
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

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
