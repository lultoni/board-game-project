// Unified AI-access seam (Change 6).
//
// The user wanted every frontend AI interaction to funnel through one module,
// with two clearly-separated call sites:
//
//   (a) ONE-SHOT — "search this position for the best move / a rating, once".
//       Used by the inspector's "Ask AI" (single + iterative-deepening) and any
//       rating read. Wraps `runAiCall` (timeout / cooperative-cancel shell) over
//       the engine's `requestAiMoveForced` / `requestAiMoveAtDepth`.
//
//   (b) CONTINUOUS — "play a whole AI-vs-AI game to completion in the
//       background". Thin wrappers over the producer commands plus a helper the
//       match route's log-player uses to pull raw actions from the producer log.
//
// Keeping both behind this module means the two engines/threads a match may
// touch (the interactive view engine and the background producer) are reached
// through one documented surface rather than scattered `invoke`/`eng.*` calls.

import type { EngineClient, StepResult } from "./types";
import { runAiCall, type AiCallOpts } from "./ai-hooks";

// === (a) One-shot search ====================================================

/** One-shot: search the current position and return the engine's best move +
 *  score, WITHOUT applying it. Runs regardless of seat kind (so it also works
 *  in HvH positions — the inspector's "what would the AI play here?"). Wrapped
 *  in `runAiCall` so callers get uniform timeout / cancellation semantics. */
export function requestBestMove(
  eng: EngineClient,
  opts: AiCallOpts = {},
): Promise<StepResult> {
  return runAiCall(() => eng.requestAiMoveForced(), opts);
}

/** One-shot iterative-deepening step: search to exactly `maxDepth` with no time
 *  bound. The caller drives the deepening loop (incrementing `maxDepth`) and
 *  polls cancellation between calls via `opts.cancelled`. The engine's shared
 *  transposition table makes successive depths progressively cheaper. */
export function requestBestMoveAtDepth(
  eng: EngineClient,
  maxDepth: number,
  opts: AiCallOpts = {},
): Promise<StepResult> {
  return runAiCall(() => eng.requestAiMoveAtDepth(maxDepth), opts);
}

// === (b) Continuous AIvAI producer ==========================================

export interface AivaiEvaluatorChoice {
  source: "builtin" | "heuristic" | "run" | "blessed";
  id?: string | null;
}

/** Start the background producer that plays the whole AIvAI game to completion
 *  from `viewSnapshotJson` (so the producer and the frontend's view engine
 *  share an identical start position + config). Both seat evaluators are
 *  re-installed engine-side (the snapshot restore resets them to heuristic). */
export function startAivaiProducer(
  eng: EngineClient,
  viewSnapshotJson: string,
  p1: AivaiEvaluatorChoice,
  p2: AivaiEvaluatorChoice,
): Promise<void> {
  return eng.startAivaiProducer(viewSnapshotJson, p1, p2);
}

/** Abort + join the producer, returning its final authoritative MatchLog JSON
 *  (or null if none was running). Awaited on leaving an AIvAI match. */
export function stopAivaiProducer(eng: EngineClient): Promise<string | null> {
  return eng.stopAivaiProducer();
}

/** Non-joining read of the producer's currently-published MatchLog JSON. */
export function aivaiProducerLog(eng: EngineClient): Promise<string | null> {
  return eng.aivaiProducerLog();
}

/** Subscribe to `aivai-progress`. The callback fires with the producer's
 *  current ply count (the ceiling the log-player advances toward) and whether
 *  the producer has finished. Returns an unlisten fn. */
export function onAivaiProgress(
  eng: EngineClient,
  cb: (plies: number, done: boolean) => void,
): Promise<() => void> {
  return eng.onAivaiProgress(cb);
}

/** Extract the ordered raw-action list from a producer MatchLog JSON string.
 *  Mirrors the `snapshotJsonFromMatchLog` parse in `multiplayer-resume.ts`:
 *  `action.raw` is a u32 that survives the JSON round-trip (unlike the zobrist
 *  fields, which overflow Number precision and we don't read here). Returns an
 *  empty array on a null/malformed log. */
export function producerRawsFromLog(matchLogJson: string | null): number[] {
  if (!matchLogJson) return [];
  try {
    const log = JSON.parse(matchLogJson) as {
      plies?: Array<{ action?: { raw?: number } }>;
    };
    const raws: number[] = [];
    for (const ply of log.plies ?? []) {
      const raw = ply.action?.raw;
      if (typeof raw !== "number" || !Number.isInteger(raw) || raw < 0) break;
      raws.push(raw >>> 0);
    }
    return raws;
  } catch {
    return [];
  }
}

/** Per-ply search readout pulled from a producer MatchLog, positionally aligned
 *  with `producerRawsFromLog` (same index = same ply). `depth`/`scoreCp` come
 *  from the ply's `ai` SearchMeta (the search that CHOSE that move). `scoreCp`
 *  is P1-POV (positive = P1 ahead), matching the log's convention; the caller
 *  flips sign per seat for display. Entries are `null` when a ply carries no
 *  `ai` meta (legacy log, or a not-yet-flushed tail) — the AIvAI pill then still
 *  shows via the `thinking` flag, just without a depth badge. Truncates at the
 *  same point `producerRawsFromLog` does so the two arrays stay index-aligned. */
export interface ProducerPlyMeta {
  depth: number;
  scoreCp: number | null;
}

export function producerMetaFromLog(matchLogJson: string | null): (ProducerPlyMeta | null)[] {
  if (!matchLogJson) return [];
  try {
    const log = JSON.parse(matchLogJson) as {
      plies?: Array<{
        action?: { raw?: number };
        ai?: { depth?: number; score_cp?: number | null } | null;
      }>;
    };
    const metas: (ProducerPlyMeta | null)[] = [];
    for (const ply of log.plies ?? []) {
      const raw = ply.action?.raw;
      // Mirror producerRawsFromLog's truncation so indices stay aligned.
      if (typeof raw !== "number" || !Number.isInteger(raw) || raw < 0) break;
      const ai = ply.ai;
      if (ai && typeof ai.depth === "number") {
        metas.push({ depth: ai.depth, scoreCp: ai.score_cp ?? null });
      } else {
        metas.push(null);
      }
    }
    return metas;
  } catch {
    return [];
  }
}

/** Count the actions baked into an engine SNAPSHOT JSON (`{ start_fen, actions,
 *  config }`). This is the number of plies already applied to an engine
 *  restored from that snapshot — the AIvAI log-player uses it as the starting
 *  offset so it does NOT re-apply plies the view engine already holds (e.g. the
 *  12 draft plies carried over from /draft/ in a Move-phase snapshot). Returns
 *  0 on a null/malformed snapshot or a missing/!array `actions`. */
export function snapshotActionCount(snapshotJson: string | null): number {
  if (!snapshotJson) return 0;
  try {
    const snap = JSON.parse(snapshotJson) as { actions?: unknown };
    return Array.isArray(snap.actions) ? snap.actions.length : 0;
  } catch {
    return 0;
  }
}
