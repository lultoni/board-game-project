// Tiny i18n. Dictionaries are flat-namespaced JSON; `t('skills.lance.name')`
// resolves dotted keys, with {param} interpolation. Falls back to English when
// the active locale is missing a key.

import en from "../i18n/en.json";
import de from "../i18n/de.json";

export type Locale = "en" | "de";

const dicts: Record<Locale, unknown> = { en, de };

let active: Locale = "en";

export function setLocale(loc: Locale): void {
  active = loc;
}
export function getLocale(): Locale {
  return active;
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
  const hit = resolve(dicts[active], key) ?? resolve(dicts.en, key);
  if (hit === undefined) return key;
  return interpolate(hit, params);
}
