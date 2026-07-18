// Reactive holder for the active UI locale. Split out of `i18n.ts` because
// runes (`$state`) only work in `.svelte`/`.svelte.ts` modules. `t()` reads
// `activeLocale.value` during render, so any component that calls `t()`
// re-renders when the locale changes - no reload needed.

export type Locale = "en" | "de";

export const activeLocale = $state<{ value: Locale }>({ value: "en" });
