// Heartbeat + now-tick timer pair. Plain TS module (no runes) so it can be
// driven from tests with a fake timer if needed in the future. Owns the two
// setInterval handles and their lifecycle; the wrapper module owns the
// state mutations triggered on each tick.
//
// Why split these two timers into one module: they're both lifecycle-bound
// to the same "we have a live connection" concept and tend to start/stop
// together. The wrapper would otherwise have to coordinate two unrelated
// `setInterval` ids; this module makes the lifecycle a single concern.

export interface HeartbeatCallbacks {
  /** Called once per 5s while the heartbeat is running. Wrapper sends a
   *  V1 ping frame here. The module never references the wire format. */
  onPing(): void;
  /** Called once per 500ms while the now-tick is running, with the current
   *  `Date.now()`. Wrapper updates its `nowTick` $state and runs the
   *  pong-age-out → status bridge. */
  onTick(now: number): void;
}

export interface Heartbeat {
  /** Begin the 1Hz ping loop AND ensure the 500ms now-tick is running.
   *  Restarting clears a prior ping timer (matching the wrapper's old
   *  startHeartbeat behaviour). */
  startPings(): void;
  /** Stop only the ping loop; leaves the now-tick running so pillState
   *  recomputes even after a drop (the bridge needs to keep firing). */
  stopPings(): void;
  /** Idempotently start the 500ms now-tick if it isn't already running. */
  ensureTicking(): void;
  /** Stop the now-tick. Used by `disconnect()` to fully tear down. */
  stopTicking(): void;
}

export function createHeartbeat(cbs: HeartbeatCallbacks): Heartbeat {
  let pingTimer: ReturnType<typeof setInterval> | null = null;
  let nowTimer: ReturnType<typeof setInterval> | null = null;

  function ensureTicking(): void {
    if (nowTimer) return;
    nowTimer = setInterval(() => {
      cbs.onTick(Date.now());
    }, 500);
  }

  function stopTicking(): void {
    if (nowTimer) {
      clearInterval(nowTimer);
      nowTimer = null;
    }
  }

  function startPings(): void {
    if (pingTimer) clearInterval(pingTimer);
    ensureTicking();
    // Fire the first ping immediately so the pong roundtrip confirms
    // liveness right away — waiting 5s means the pill sits yellow after
    // every (re)connect even though the relay already told us both peers
    // are paired.
    try { cbs.onPing(); } catch { /* subscriber crash must not stop the loop */ }
    pingTimer = setInterval(() => {
      cbs.onPing();
    }, 5_000);
  }

  function stopPings(): void {
    if (pingTimer) {
      clearInterval(pingTimer);
      pingTimer = null;
    }
  }

  return { startPings, stopPings, ensureTicking, stopTicking };
}
