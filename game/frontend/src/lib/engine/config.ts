// Build a Rust `core_engine::session::Config` JSON blob from the current
// UI seat assignments and persisted AI settings. Passed to
// `EngineClient.createEngine(configJson)` so the engine's seat config
// matches what the UI expects — without this, `step_ai` returns NotAiTurn
// when the engine thinks the current side is Human while the UI thinks it
// is AI (or vice versa).

import { settings } from "$lib/state/settings.svelte";
import type { SeatKind } from "$lib/state/match-store.svelte";

function durationFromMs(ms: number): { secs: number; nanos: number } {
  const safe = Math.max(0, Math.floor(ms));
  const secs = Math.floor(safe / 1000);
  const nanos = (safe - secs * 1000) * 1_000_000;
  return { secs, nanos };
}

function seatTag(s: SeatKind): "Human" | "Ai" {
  return s === "ai" ? "Ai" : "Human";
}

export function buildEngineConfigJson(side: {
  p1: SeatKind;
  p2: SeatKind;
}): string {
  return JSON.stringify({
    p1: seatTag(side.p1),
    p2: seatTag(side.p2),
    p1_ai: {
      time_limit_ms: settings.p1ThinkTimeMs,
      max_depth: settings.p1MaxDepth,
    },
    p2_ai: {
      time_limit_ms: settings.p2ThinkTimeMs,
      max_depth: settings.p2MaxDepth,
    },
    aivai_step_delay: durationFromMs(settings.aivaiStepDelayMs),
    allow_undo: true,
    auto_log: true,
  });
}
