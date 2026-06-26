import { describe, it, expect, vi } from "vitest";
import { runAiCall, AiCallError } from "./ai-hooks";

describe("runAiCall", () => {
  it("returns the engine result when fn resolves before the timeout", async () => {
    const result = await runAiCall(async () => ({ score: 42 }));
    expect(result).toEqual({ score: 42 });
  });

  it("rejects with AiCallError('timeout') when fn outlasts timeoutMs", async () => {
    vi.useFakeTimers();
    try {
      const slow = new Promise<number>((resolve) => setTimeout(() => resolve(7), 1000));
      const p = runAiCall(() => slow, { timeoutMs: 50 });
      vi.advanceTimersByTime(60);
      await expect(p).rejects.toBeInstanceOf(AiCallError);
      await expect(p).rejects.toMatchObject({ reason: "timeout" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("rejects with AiCallError('cancelled') when cancelled() returns true on resolve", async () => {
    let cancelled = false;
    const fn = async () => {
      cancelled = true;
      return 99;
    };
    await expect(runAiCall(fn, { cancelled: () => cancelled })).rejects.toMatchObject({
      reason: "cancelled",
    });
  });

  it("wraps a thrown engine error as AiCallError('engine')", async () => {
    const fn = async () => {
      throw new Error("worker died");
    };
    await expect(runAiCall(fn)).rejects.toMatchObject({
      reason: "engine",
      message: "worker died",
    });
  });

  it("does not fire the timeout when fn resolves quickly even with a timeout set", async () => {
    const result = await runAiCall(async () => 1, { timeoutMs: 10000 });
    expect(result).toBe(1);
  });
});
