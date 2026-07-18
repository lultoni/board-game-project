// Dedupe helpers for custom loadouts.
//
// Custom loadouts are equal when their 12 skill IDs (in canonical piece +
// slot order) match. Names are user-facing labels, not identity - the same
// skill tuple saved twice under different names is still a duplicate.
//
// `loadoutKey` produces a stable string form of a SideLoadout. `findDuplicate`
// linearly scans a list of existing rows and returns the first match, if any.
// Both are pure and side-effect-free so callers can use them from the save
// button's reactive guard and from the import handler's skip-and-report loop.

import type { SideLoadout } from "$lib/engine";
import type { SavedLoadout } from "./types";

/** Canonical string form of a loadout - 12 skill IDs joined with
 *  separators. Format: `"s1,s2|s1,s2|s1,s2|s1,s2|s1,s2|s1,s2"`. Slot 0
 *  (empty) is preserved so incomplete loadouts don't accidentally match
 *  complete ones. */
export function loadoutKey(l: SideLoadout): string {
  return l.map(pair => `${pair[0]},${pair[1]}`).join("|");
}

/** First existing row whose skills match `target`, or `null`. Names are
 *  ignored; only the skill tuple counts. */
export function findDuplicate(
  target: SideLoadout,
  existing: readonly SavedLoadout[],
): SavedLoadout | null {
  const key = loadoutKey(target);
  for (const row of existing) {
    if (loadoutKey(row.loadout) === key) return row;
  }
  return null;
}
