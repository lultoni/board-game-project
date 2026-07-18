// Build a Rust `core_engine::session::Config` JSON blob from the current
// UI seat assignments and AI budgets. Passed to
// `EngineClient.createEngine(configJson)` so the engine's seat config
// matches what the UI expects - without this, `step_ai` returns NotAiTurn
// when the engine thinks the current side is Human while the UI thinks it
// is AI (or vice versa).
//
// Pure leaf: this module does NOT import from `$lib/state`. Callers pass
// the seat + budget data in. The previous direction (engine reading
// `settings` directly) made the engine layer depend on the state layer,
// which inverted the natural app stack and made `engine/` un-testable
// without booting the rune store.

export type SeatTag = "Human" | "Ai";

export interface EngineConfigInput {
  /** Seat assignments - already mapped to Rust's enum values. */
  p1: SeatTag;
  p2: SeatTag;
  /** AI budgets, one bag per side. Time limits are in milliseconds and are
   *  converted to Rust's `{secs, nanos}` Duration on the way out. */
  p1Ai: { timeLimitMs: number; maxDepth: number };
  p2Ai: { timeLimitMs: number; maxDepth: number };
  /** Inter-ply delay for AIvAI playback. Milliseconds; same conversion. */
  aivaiStepDelayMs: number;
}

function durationFromMs(ms: number): { secs: number; nanos: number } {
  const safe = Math.max(0, Math.floor(ms));
  const secs = Math.floor(safe / 1000);
  const nanos = (safe - secs * 1000) * 1_000_000;
  return { secs, nanos };
}

export function buildEngineConfigJson(input: EngineConfigInput): string {
  // 0 in the UI means "no depth limit"; map to 64 (Rust coerces > actual tree depth to effective ∞).
  const p1MaxDepth = input.p1Ai.maxDepth === 0 ? 64 : input.p1Ai.maxDepth;
  const p2MaxDepth = input.p2Ai.maxDepth === 0 ? 64 : input.p2Ai.maxDepth;
  return JSON.stringify({
    p1: input.p1,
    p2: input.p2,
    p1_ai: {
      time_limit_ms: input.p1Ai.timeLimitMs,
      max_depth: p1MaxDepth,
    },
    p2_ai: {
      time_limit_ms: input.p2Ai.timeLimitMs,
      max_depth: p2MaxDepth,
    },
    aivai_step_delay: durationFromMs(input.aivaiStepDelayMs),
    allow_undo: true,
    auto_log: true,
  });
}
