// AI-transient search state. Held in one rune-store so that depth-tick
// updates from `stepAi()` don't cascade re-renders into every sibling
// component that happens to sit near a consumer. Owners of this state
// (routes/match) call `beginSearch()` / `updateDepth()` / `endSearch()`;
// panels that display the state (PlayerPanel, EvalBreakdownPanel) read
// directly from the store instead of receiving props from the route.
//
// The 100 ms depth-tick throttle is baked in here so every consumer
// automatically benefits — previously it lived in match/+page.svelte and
// callers who wanted the value had to re-derive.

import type { EvalBreakdown, EvalBreakdownBySquare } from "../engine/types";

interface AiSearchState {
  thinking: boolean;
  lastDepth: number | null;
  lastScore: number | null;
  searchStartedAt: number | null;
  finishedAtPly: number | null;
  heuristicEvalBreakdown: EvalBreakdown | null;
  heuristicEvalBySquare: EvalBreakdownBySquare | null;
  prevRoundBreakdown: EvalBreakdown | null;
  lastRoundSeen: number | null;
}

const state = $state<AiSearchState>({
  thinking: false,
  lastDepth: null,
  lastScore: null,
  searchStartedAt: null,
  finishedAtPly: null,
  heuristicEvalBreakdown: null,
  heuristicEvalBySquare: null,
  prevRoundBreakdown: null,
  lastRoundSeen: null,
});

// Non-reactive throttle bookkeeping — kept outside `state` so writes here
// don't trigger consumer re-renders.
let lastDepthUpdateMs = 0;

export const aiSearch = {
  get thinking() { return state.thinking; },
  get lastDepth() { return state.lastDepth; },
  get lastScore() { return state.lastScore; },
  get searchStartedAt() { return state.searchStartedAt; },
  get finishedAtPly() { return state.finishedAtPly; },
  get heuristicEvalBreakdown() { return state.heuristicEvalBreakdown; },
  get heuristicEvalBySquare() { return state.heuristicEvalBySquare; },
  get prevRoundBreakdown() { return state.prevRoundBreakdown; },
  get lastRoundSeen() { return state.lastRoundSeen; },
};

export function beginSearch(): void {
  state.thinking = true;
  state.searchStartedAt = Date.now();
  state.lastDepth = null;
  state.lastScore = null;
  lastDepthUpdateMs = 0;
}

/** Depth-tick writer. Coalesces to at most one write per 100 ms so a
 *  fast-iterating search doesn't fire per-depth re-renders into every
 *  consumer. The final depth reached will still be captured because
 *  `endSearch()` doesn't touch depth/score — the last throttled write
 *  before search-end is authoritative. */
export function updateDepth(depth: number, score: number): void {
  const now = Date.now();
  if (now - lastDepthUpdateMs < 100) return;
  lastDepthUpdateMs = now;
  state.lastDepth = depth;
  state.lastScore = score;
}

export function endSearch(atPly: number): void {
  state.thinking = false;
  state.finishedAtPly = atPly;
}

/** Write the final depth reached, bypassing the throttle. Called from the
 *  search-completion path so the last (deepest) iteration reached is always
 *  visible in the linger badge, even if the throttle skipped its callback. */
export function setFinalDepth(depth: number): void {
  state.lastDepth = depth;
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
  state.thinking = false;
  state.lastDepth = null;
  state.lastScore = null;
  state.searchStartedAt = null;
  state.finishedAtPly = null;
  state.heuristicEvalBreakdown = null;
  state.heuristicEvalBySquare = null;
  state.prevRoundBreakdown = null;
  state.lastRoundSeen = null;
  lastDepthUpdateMs = 0;
}
