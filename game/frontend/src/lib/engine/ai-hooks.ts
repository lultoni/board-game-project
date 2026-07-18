// Shared error/timeout shell for engine AI calls. The two existing call sites
// (`routes/match/+page.svelte` stepAi, `routes/inspector/+page.svelte`
// requestAiMoveAtDepth in the deepening loop) had near-identical try/catch
// shapes - same `(e as Error).message` stringification, no timeout, no
// uniform cancellation hook. Funnel through `runAiCall` so future work
// (e.g., a global "engine wedged" toast or a settings-driven timeout) has
// one place to land.
//
// IMPORTANT: There is no engine-side cancellation API. A `timeoutMs` rejection
// drops the result on the floor; the engine keeps searching until it finishes
// naturally. `cancelled` is *cooperative* - it is polled at well-known points
// (today: only on resolve, since the engine doesn't yield mid-search). The
// outer iterative-deepening loop in inspector remains the authoritative
// cancellation channel for that flow.

export type AiCallReason = "timeout" | "cancelled" | "engine";

export class AiCallError extends Error {
  readonly reason: AiCallReason;
  constructor(reason: AiCallReason, message?: string) {
    super(message ?? reason);
    this.name = "AiCallError";
    this.reason = reason;
  }
}

export interface AiCallOpts {
  /** Wall-clock timeout in ms. If the engine call doesn't resolve in time,
   *  the returned promise rejects with `AiCallError("timeout")`. */
  timeoutMs?: number;
  /** Cooperative-cancellation flag. Checked after `fn` resolves; if true the
   *  result is discarded and the promise rejects with `AiCallError("cancelled")`.
   *  We deliberately don't poll mid-flight - the engine doesn't yield. */
  cancelled?: () => boolean;
}

export async function runAiCall<T>(fn: () => Promise<T>, opts: AiCallOpts = {}): Promise<T> {
  const { timeoutMs, cancelled } = opts;

  let timer: ReturnType<typeof setTimeout> | null = null;
  const callP = fn();

  if (timeoutMs !== undefined && timeoutMs > 0) {
    const timeoutP = new Promise<never>((_, reject) => {
      timer = setTimeout(() => reject(new AiCallError("timeout")), timeoutMs);
    });
    try {
      const result = await Promise.race([callP, timeoutP]);
      if (cancelled?.()) throw new AiCallError("cancelled");
      return result;
    } catch (e) {
      if (e instanceof AiCallError) throw e;
      throw new AiCallError("engine", (e as Error)?.message ?? String(e));
    } finally {
      if (timer !== null) clearTimeout(timer);
    }
  }

  try {
    const result = await callP;
    if (cancelled?.()) throw new AiCallError("cancelled");
    return result;
  } catch (e) {
    if (e instanceof AiCallError) throw e;
    throw new AiCallError("engine", (e as Error)?.message ?? String(e));
  }
}
