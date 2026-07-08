// AI-transient search state. Held in one rune-store so that depth-tick
// updates from `stepAi()` don't cascade re-renders into every sibling
// component that happens to sit near a consumer. Owners of this state
// (routes/match) call `beginSearch(side)` / `updateDepth(side, ...)` /
// `endSearch(side, ...)`; panels that display the state read directly.
//
// Per-side split: P1 and P2 each own a slot so a mid-game P2 search
// cannot overwrite P1's just-completed linger depth/score. The heuristic
// eval fields are position-level (not per-search), so they stay shared.

import type { EvalBreakdown, EvalBreakdownBySquare } from "../engine/types";

export type Side = "p1" | "p2";

interface SideSearchState {
  thinking: boolean;
  lastDepth: number | null;
  lastScore: number | null;
  searchStartedAt: number | null;
  finishedAtPly: number | null;
}

interface AiSearchState {
  p1: SideSearchState;
  p2: SideSearchState;
  heuristicEvalBreakdown: EvalBreakdown | null;
  heuristicEvalBySquare: EvalBreakdownBySquare | null;
  prevRoundBreakdown: EvalBreakdown | null;
  lastRoundSeen: number | null;
}

function emptySide(): SideSearchState {
  return {
    thinking: false,
    lastDepth: null,
    lastScore: null,
    searchStartedAt: null,
    finishedAtPly: null,
  };
}

const state = $state<AiSearchState>({
  p1: emptySide(),
  p2: emptySide(),
  heuristicEvalBreakdown: null,
  heuristicEvalBySquare: null,
  prevRoundBreakdown: null,
  lastRoundSeen: null,
});

// Non-reactive throttle bookkeeping, per side.
const lastDepthUpdateMs = { p1: 0, p2: 0 };

export const aiSearch = {
  get p1() { return state.p1; },
  get p2() { return state.p2; },
  get heuristicEvalBreakdown() { return state.heuristicEvalBreakdown; },
  get heuristicEvalBySquare() { return state.heuristicEvalBySquare; },
  get prevRoundBreakdown() { return state.prevRoundBreakdown; },
  get lastRoundSeen() { return state.lastRoundSeen; },
  /** True iff either side has an in-flight search. Kept for consumers
   *  that gate on "any AI is thinking" (poll suppression, button disable). */
  get anyThinking() { return state.p1.thinking || state.p2.thinking; },
};

export function beginSearch(side: Side): void {
  const s = state[side];
  s.thinking = true;
  s.searchStartedAt = Date.now();
  s.lastDepth = null;
  s.lastScore = null;
  lastDepthUpdateMs[side] = 0;
}

/** Depth-tick writer. Coalesces to at most one write per 100 ms per side. */
export function updateDepth(side: Side, depth: number, score: number): void {
  const now = Date.now();
  if (now - lastDepthUpdateMs[side] < 100) return;
  lastDepthUpdateMs[side] = now;
  state[side].lastDepth = depth;
  state[side].lastScore = score;
}

export function endSearch(side: Side, atPly: number): void {
  state[side].thinking = false;
  state[side].finishedAtPly = atPly;
}

/** Write the final depth reached, bypassing the throttle. */
export function setFinalDepth(side: Side, depth: number): void {
  state[side].lastDepth = depth;
}

export function setHeuristic(breakdown: EvalBreakdown | null): void {
  state.heuristicEvalBreakdown = breakdown;
}

export function setHeuristicBySquare(bySquare: EvalBreakdownBySquare | null): void {
  state.heuristicEvalBySquare = bySquare;
}

export function setPrevRoundBreakdown(breakdown: EvalBreakdown | null): void {
  state.prevRoundBreakdown = breakdown;
}

export function setLastRoundSeen(round: number | null): void {
  state.lastRoundSeen = round;
}

/** Full reset — route teardown or entering a new match. */
export function resetAiSearch(): void {
  state.p1 = emptySide();
  state.p2 = emptySide();
  state.heuristicEvalBreakdown = null;
  state.heuristicEvalBySquare = null;
  state.prevRoundBreakdown = null;
  state.lastRoundSeen = null;
  lastDepthUpdateMs.p1 = 0;
  lastDepthUpdateMs.p2 = 0;
}
