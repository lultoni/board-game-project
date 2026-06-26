// State-machine tests for the joiner-side auto-redial path. Mocks the `peerjs`
// module so we can drive the PeerJS lifecycle synchronously and assert on the
// carrier fields the GraceBanner depends on (peerEverPaired, disconnectedSince,
// code, role).
//
// Regression target: pre-fix, every auto-redial called `join()` → `disconnect()`
// → cleared peerEverPaired + disconnectedSince. The banner vanished on the
// first redial. This file asserts the soft path preserves those fields across
// retries while the hard path (lobby disconnect / explicit join) still resets.

import { beforeEach, describe, expect, it, vi } from "vitest";

// vi.mock is hoisted; its factory cannot reference module-scope variables.
// Define the fake Peer constructor *inside* the factory so it's self-contained.
vi.mock("peerjs", () => {
  const { EventEmitter } = require("events");
  class FakeDataConnection extends EventEmitter {
    open = false;
    send = (..._args: unknown[]) => { /* noop */ };
    close = () => { this.open = false; this.emit("close"); };
  }
  class FakePeer extends EventEmitter {
    destroyed = false;
    outboundConn: FakeDataConnection | null = null;
    constructor(public id: string) {
      super();
      queueMicrotask(() => { if (!this.destroyed) this.emit("open", id); });
    }
    connect(_targetId: string, _opts?: unknown): FakeDataConnection {
      this.outboundConn = new FakeDataConnection();
      return this.outboundConn;
    }
    destroy(): void {
      this.destroyed = true;
      this.removeAllListeners();
    }
  }
  return { default: FakePeer };
});

// Import AFTER the mock so the production module picks up FakePeer.
import { mpState, join, disconnect, destroyPeerKeepState } from "./multiplayer.svelte";

describe("multiplayer joiner state — hard reset path", () => {
  beforeEach(() => {
    disconnect();
  });

  it("disconnect() fully resets all carrier fields", () => {
    mpState.role = "joiner";
    mpState.code = "999999";
    mpState.peerEverPaired = true;
    mpState.disconnectedSince = Date.now();
    mpState.lastPongAt = Date.now();

    disconnect();

    expect(mpState.role).toBeNull();
    expect(mpState.code).toBeNull();
    expect(mpState.peerEverPaired).toBe(false);
    expect(mpState.lastPongAt).toBeNull();
  });

  it("join() clears disconnectedSince + peerEverPaired synchronously, then transitions to 'connecting' once PeerJS opens", async () => {
    mpState.disconnectedSince = 1_700_000_000_000;
    mpState.peerEverPaired = true;

    // Capture the promise so its eventual rejection has a handler, but
    // never await it: FakePeer doesn't fire the DataConnection "open" or
    // "error" events, so the join promise stays pending. The assertions
    // below check the wrapper's STATE TRANSITIONS at two well-defined
    // points (synchronous slice + post-Peer-open microtask) rather than
    // racing the wrapper's internal timeouts as the prior version did.
    const joinPromise = join("123456");
    joinPromise.catch(() => { /* fake DataConnection never opens; expected */ });

    // Synchronous slice — the pre-PeerJS reset happens inside join() before
    // bindJoinerPeer returns the promise.
    expect(mpState.disconnectedSince).toBeNull();
    expect(mpState.peerEverPaired).toBe(false);
    expect(mpState.code).toBe("123456");
    expect(mpState.role).toBe("joiner");
    expect(mpState.status).toBe("joining");

    // FakePeer's constructor queues a microtask that fires "open". Awaiting
    // microtask yields drains that queue and lets the wrapper's p.on("open")
    // handler run, which advances status to "connecting" and calls
    // p.connect(...). We do not depend on the DataConnection's "open" event.
    await Promise.resolve();
    await Promise.resolve();

    expect(mpState.status).toBe("connecting");
  });
});

// The soft-reconnect helper isn't exported, but its core teardown primitive —
// `destroyPeerKeepState` — is. Asserting its behaviour directly is sufficient
// to lock in the GraceBanner-preservation invariant: after this runs, all the
// fields the banner reads survive.
describe("multiplayer soft teardown (used by auto-redial)", () => {
  beforeEach(() => disconnect());

  it("destroyPeerKeepState preserves code/role/peerEverPaired/disconnectedSince", () => {
    mpState.role = "joiner";
    mpState.code = "424242";
    mpState.peerEverPaired = true;
    mpState.disconnectedSince = 1_700_000_000_000;
    mpState.lastPongAt = 1_700_000_005_000;
    mpState.status = "connected";

    destroyPeerKeepState();

    // Status flips to "disconnected" so the pill renders red, but the carrier
    // fields the GraceBanner reads stay intact.
    expect(mpState.status).toBe("disconnected");
    expect(mpState.role).toBe("joiner");
    expect(mpState.code).toBe("424242");
    expect(mpState.peerEverPaired).toBe(true);
    expect(mpState.disconnectedSince).toBe(1_700_000_000_000);
    // lastPongAt is intentionally nulled so derivePillState's stale-pong
    // logic doesn't keep reporting "live".
    expect(mpState.lastPongAt).toBeNull();
  });

  it("destroyPeerKeepState is idempotent — second call doesn't clobber preserved fields", () => {
    mpState.role = "joiner";
    mpState.code = "424242";
    mpState.peerEverPaired = true;
    mpState.disconnectedSince = 1_700_000_000_000;

    destroyPeerKeepState();
    destroyPeerKeepState();

    expect(mpState.role).toBe("joiner");
    expect(mpState.code).toBe("424242");
    expect(mpState.peerEverPaired).toBe(true);
    expect(mpState.disconnectedSince).toBe(1_700_000_000_000);
  });
});