// Global back-navigation state. The back button lives in the root layout
// (top-left, mirroring the Settings/Help pills) so every secondary screen
// gets a consistent control in the same place — it is NOT re-declared per
// route. Routes that need a context-specific destination (e.g. replay ->
// library) or custom teardown (e.g. leaving a multiplayer lobby) register an
// override on mount and clear it on unmount via `setBackNav` / `clearBackNav`.

export interface BackNavOverride {
  /** Destination href. Defaults to the hub ("/") when unset. */
  href?: string;
  /** Extra work to run on click (teardown, confirm, etc.). Runs after the
   *  click SFX. If it calls `ev.preventDefault()` the layout skips its own
   *  navigation and leaves routing to the handler. */
  onclick?: (ev: MouseEvent) => void;
  /** Override the visible label. Defaults to the shared "← back" string. */
  label?: string;
}

// A single `$state` object the layout reads reactively. `current` is null on
// the hub and on any route that hasn't registered an override (which then
// falls back to href "/" — the hub).
export const backNav = $state<{ current: BackNavOverride | null }>({ current: null });

/** Register this route's back-button behaviour. Call in an $effect so it
 *  re-registers on navigation and the returned cleanup clears it on unmount. */
export function setBackNav(override: BackNavOverride): void {
  backNav.current = override;
}

/** Clear any registered override (back button reverts to hub default). */
export function clearBackNav(): void {
  backNav.current = null;
}
