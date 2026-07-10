// Tiny i18n. Dictionaries are flat-namespaced JSON; `t('skills.lance.name')`
// resolves dotted keys, with {param} interpolation. Falls back to English when
// the active locale is missing a key.
//
// The active locale lives in `i18n-locale.svelte.ts` as a `$state` rune so
// that `t()` is reactive: any component calling `t()` re-renders when the
// locale changes. This module stays a plain `.ts` so the `t` import path is
// unchanged app-wide.

import en from "../i18n/en.json";
import de from "../i18n/de.json";
import { activeLocale, type Locale } from "./i18n-locale.svelte";

export type { Locale };

const dicts: Record<Locale, unknown> = { en, de };

export function setLocale(loc: Locale): void {
  activeLocale.value = loc;
}
export function getLocale(): Locale {
  return activeLocale.value;
}

function resolve(dict: unknown, key: string): string | undefined {
  const parts = key.split(".");
  let cur: unknown = dict;
  for (const p of parts) {
    if (cur && typeof cur === "object" && p in (cur as Record<string, unknown>)) {
      cur = (cur as Record<string, unknown>)[p];
    } else {
      return undefined;
    }
  }
  return typeof cur === "string" ? cur : undefined;
}

function interpolate(s: string, params?: Record<string, string | number>): string {
  if (!params) return s;
  return s.replace(/\{(\w+)\}/g, (_, k) => {
    const v = params[k];
    return v === undefined ? `{${k}}` : String(v);
  });
}

export function t(key: string, params?: Record<string, string | number>): string {
  const active = activeLocale.value;
  const hit = resolve(dicts[active], key) ?? resolve(dicts.en, key);
  if (hit === undefined) return key;
  return interpolate(hit, params);
}
