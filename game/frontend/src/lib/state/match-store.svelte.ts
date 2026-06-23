// Match-level reactive state. Populated by routes/match/+page.svelte and the
// engine bridge. This slice creates the skeleton; later slices fill in
// selection/legal/effects.

import type { PositionView } from "../engine/types";

export type SeatKind = "human" | "ai";
export type MatchMode = "idle" | "hvh" | "hvai" | "aivai" | "replay" | "sandbox";

export interface MatchState {
  mode: MatchMode;
  side: { p1: SeatKind; p2: SeatKind };
  position: PositionView | null;
  legal: Uint32Array;
  /** Square (0..63) currently selected by the human, if any. */
  selection: number | null;
  /** The raw action just applied, for "what just happened" effects. */
  lastApplied: number | null;
  /**
   * Pre-built engine snapshot stashed by the draft route. When set, the
   * match route restores from this snapshot instead of creating a fresh
   * engine. Cleared after consumption.
   */
  pendingSnapshotJson: string | null;
}

export const match = $state<MatchState>({
  mode: "idle",
  side: { p1: "human", p2: "human" },
  position: null,
  legal: new Uint32Array(),
  selection: null,
  lastApplied: null,
  pendingSnapshotJson: null,
});

export function resetMatchState(): void {
  match.mode = "idle";
  match.side = { p1: "human", p2: "human" };
  match.position = null;
  match.legal = new Uint32Array();
  match.selection = null;
  match.lastApplied = null;
  match.pendingSnapshotJson = null;
}
