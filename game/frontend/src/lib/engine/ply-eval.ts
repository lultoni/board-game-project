// Per-ply engine assessment extracted from a match-log `PlyRecord` (B3).
//
// The engine records two kinds of search readout per ply (Change 5):
//   - `ai`               — set for AI plies (the search that CHOSE the move).
//   - `background_eval`   — set for human plies by the time-bounded background
//                           eval (Change 5 Part B): the engine's own read of
//                           the position the human created.
// Both share the `SearchMeta` shape and now carry `post_move_breakdown`.
//
// Match-log JSON is serialised by the Rust telemetry layer, so its field names
// are snake_case (`score_cp`, `was_mate`, `mate_in`, `post_move_breakdown`),
// NOT the camelCase used by the hot-path `StepResult` DTO. This module reads
// the snake_case log shape.

import type { EvalBreakdown } from "./types";

/** Snake_case mirror of `core_engine::telemetry::SearchMeta` as it appears in
 *  serialised match-log JSON. All fields optional for defensive parsing of
 *  legacy / partial logs. */
export interface SearchMetaLog {
  depth?: number;
  nodes?: number;
  raw_score?: number;
  was_mate?: boolean;
  mate_in?: number | null;
  score_cp?: number | null;
  post_move_breakdown?: EvalBreakdown | null;
}

/** The subset of a match-log `PlyRecord` this module reads. */
export interface PlyRecordEvalView {
  ai?: SearchMetaLog | null;
  background_eval?: SearchMetaLog | null;
}

/** A display-ready summary of a ply's engine assessment. */
export interface PlyEval {
  /** "ai" = the search that chose an AI move; "background" = the engine's
   *  post-hoc read of a human move. */
  source: "ai" | "background";
  depth: number;
  /** Centipawn-style score (P1-POV, positive = P1 ahead). Null when mate. */
  scoreCp: number | null;
  /** Plies-to-mate when the search found a forced mate; null otherwise. */
  mateIn: number | null;
  wasMate: boolean;
  breakdown: EvalBreakdown | null;
}

/** Extract the engine's assessment for one ply. Prefers `ai` (AI plies) and
 *  falls back to `background_eval` (human plies). Returns null when the ply
 *  carries neither — e.g. a legacy log, or a human ply whose background eval
 *  never ran (game over / draft / annotation disabled). */
export function plyEvalOf(ply: PlyRecordEvalView | null | undefined): PlyEval | null {
  if (!ply) return null;
  const meta = ply.ai ?? ply.background_eval ?? null;
  if (!meta) return null;
  const source: "ai" | "background" = ply.ai ? "ai" : "background";
  return {
    source,
    depth: meta.depth ?? 0,
    scoreCp: meta.score_cp ?? null,
    mateIn: meta.mate_in ?? null,
    wasMate: meta.was_mate ?? false,
    breakdown: meta.post_move_breakdown ?? null,
  };
}

/** Compact one-line label for the ply-info UI, e.g. "engine +120 (d8)" or
 *  "engine mate in 3 (d10)". Returns null when there's no assessment. */
export function formatPlyEval(e: PlyEval | null): string | null {
  if (!e) return null;
  const tag = e.source === "ai" ? "AI" : "engine";
  if (e.wasMate && e.mateIn !== null) {
    return `${tag} mate in ${e.mateIn} (d${e.depth})`;
  }
  const cp = e.scoreCp ?? 0;
  const sign = cp > 0 ? "+" : "";
  return `${tag} ${sign}${cp} (d${e.depth})`;
}
