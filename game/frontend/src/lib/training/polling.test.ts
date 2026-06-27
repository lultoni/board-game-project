// Vitest for the polling store. Uses a fake `@tauri-apps/api/core` invoke
// via vi.mock so we can sequence return values and confirm the store ticks.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { createPollingStore, type PollState } from "./polling";

function collect<T>(
  store: ReturnType<typeof createPollingStore<T>>,
): { values: PollState<T>[]; unsub: () => void } {
  const values: PollState<T>[] = [];
  const unsub = store.subscribe((v) => values.push(v));
  return { values, unsub };
}

describe("createPollingStore", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("emits initial empty state synchronously on subscribe", () => {
    invokeMock.mockResolvedValue({ k: 1 });
    const store = createPollingStore<{ k: number }>({
      invokeCmd: "noop",
      args: {},
      intervalMs: 1000,
      pollImmediately: false,
    });
    const { values, unsub } = collect(store);
    expect(values[0]).toEqual({ data: null, error: null, lastUpdated: null });
    unsub();
  });

  it("polls immediately by default and parses results", async () => {
    invokeMock.mockResolvedValueOnce({ k: 7 });
    const store = createPollingStore<{ k: number; doubled: number }>({
      invokeCmd: "fake_cmd",
      args: { x: 1 },
      intervalMs: 1000,
      parser: (raw) => {
        const obj = raw as { k: number };
        return { k: obj.k, doubled: obj.k * 2 };
      },
    });
    const { values, unsub } = collect(store);
    // Flush the immediate tick's promise chain.
    await vi.advanceTimersByTimeAsync(0);
    expect(invokeMock).toHaveBeenCalledWith("fake_cmd", { x: 1 });
    const last = values[values.length - 1];
    expect(last.data).toEqual({ k: 7, doubled: 14 });
    expect(last.error).toBeNull();
    expect(typeof last.lastUpdated).toBe("number");
    unsub();
  });

  it("ticks again after intervalMs and overwrites prior state", async () => {
    invokeMock
      .mockResolvedValueOnce({ k: 1 })
      .mockResolvedValueOnce({ k: 2 });
    const store = createPollingStore<{ k: number }>({
      invokeCmd: "cmd",
      args: {},
      intervalMs: 500,
    });
    const { values, unsub } = collect(store);
    await vi.advanceTimersByTimeAsync(0);
    await vi.advanceTimersByTimeAsync(500);
    expect(invokeMock).toHaveBeenCalledTimes(2);
    const last = values[values.length - 1];
    expect(last.data).toEqual({ k: 2 });
    unsub();
  });

  it("captures invoke errors as string in `error`", async () => {
    invokeMock.mockRejectedValueOnce(new Error("trainer offline"));
    const store = createPollingStore({
      invokeCmd: "cmd",
      args: {},
      intervalMs: 1000,
    });
    const { values, unsub } = collect(store);
    await vi.advanceTimersByTimeAsync(0);
    const last = values[values.length - 1];
    expect(last.data).toBeNull();
    expect(last.error).toBe("trainer offline");
    unsub();
  });

  it("treats null/undefined returns as data: null without an error", async () => {
    invokeMock.mockResolvedValueOnce(null);
    const store = createPollingStore({
      invokeCmd: "cmd",
      args: {},
      intervalMs: 1000,
    });
    const { values, unsub } = collect(store);
    await vi.advanceTimersByTimeAsync(0);
    const last = values[values.length - 1];
    expect(last.data).toBeNull();
    expect(last.error).toBeNull();
    expect(last.lastUpdated).not.toBeNull();
    unsub();
  });

  it("stops ticking after the last subscriber unsubscribes", async () => {
    invokeMock.mockResolvedValue({ k: 0 });
    const store = createPollingStore({
      invokeCmd: "cmd",
      args: {},
      intervalMs: 500,
    });
    const { unsub } = collect(store);
    await vi.advanceTimersByTimeAsync(0); // immediate
    unsub();
    invokeMock.mockClear();
    await vi.advanceTimersByTimeAsync(5000);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("skips the immediate tick when pollImmediately=false", async () => {
    invokeMock.mockResolvedValue({ k: 0 });
    const store = createPollingStore({
      invokeCmd: "cmd",
      args: {},
      intervalMs: 1000,
      pollImmediately: false,
    });
    const { unsub } = collect(store);
    await vi.advanceTimersByTimeAsync(0);
    expect(invokeMock).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1000);
    expect(invokeMock).toHaveBeenCalledTimes(1);
    unsub();
  });
});
