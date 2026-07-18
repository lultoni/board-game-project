// Polling store - reads a Tauri command at a fixed cadence into a Svelte store.
//
// Used by every panel in the Training Observatory. The trainer is the source
// of truth; the UI just polls a JSON file at low rate. No exponential
// backoff: failures usually mean "trainer isn't running yet", and a single
// failed poll shouldn't change cadence.

import { readable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface PollState<T> {
  data: T | null;
  error: string | null;
  lastUpdated: number | null;
}

export interface PollingOptions<T> {
  /** Tauri command name (e.g. `"read_training_status"`). */
  invokeCmd: string;
  /** Arguments object passed verbatim to `invoke`. */
  args: Record<string, unknown>;
  /** Poll cadence in milliseconds. */
  intervalMs: number;
  /** Optional parser/validator. Identity by default. */
  parser?: (data: unknown) => T;
  /** If true, start polling immediately on subscribe. Default: true. */
  pollImmediately?: boolean;
}

/**
 * Build a `Readable<PollState<T>>` that polls a Tauri command. The store
 * starts ticking on first `subscribe` and stops when the last subscriber
 * drops, so a panel that mounts and unmounts cleanly leaves no orphan timer.
 */
export function createPollingStore<T>(opts: PollingOptions<T>): Readable<PollState<T>> {
  const parse = opts.parser ?? ((x: unknown) => x as T);
  return readable<PollState<T>>(
    { data: null, error: null, lastUpdated: null },
    (set) => {
      let cancelled = false;
      let timer: ReturnType<typeof setInterval> | null = null;

      const tick = async () => {
        try {
          const raw = await invoke(opts.invokeCmd, opts.args);
          if (cancelled) return;
          set({
            data: raw === null || raw === undefined ? null : parse(raw),
            error: null,
            lastUpdated: Date.now(),
          });
        } catch (e: unknown) {
          if (cancelled) return;
          set({
            data: null,
            error: e instanceof Error ? e.message : String(e),
            lastUpdated: Date.now(),
          });
        }
      };

      if (opts.pollImmediately !== false) {
        void tick();
      }
      timer = setInterval(tick, opts.intervalMs);

      return () => {
        cancelled = true;
        if (timer !== null) clearInterval(timer);
      };
    },
  );
}
