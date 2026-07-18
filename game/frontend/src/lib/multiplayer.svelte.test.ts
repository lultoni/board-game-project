// State-machine tests for the joiner-side auto-redial path. Exercises the
// carrier-state fields the GraceBanner depends on (peerEverPaired,
// disconnectedSince, code, role) without needing a live relay server.
//
// Regression target: pre-fix, every auto-redial called `join()` → `disconnect()`
// → cleared peerEverPaired + disconnectedSince. The banner vanished on the
// first redial. This file asserts the soft path preserves those fields across
// retries while the hard path (lobby disconnect / explicit join) still resets.

import { beforeEach, describe, expect, it, vi } from "vitest";

// Import AFTER the mock so the production module picks up FakePeer.
import {
  mpState,
  join,
  disconnect,
  destroyPeerKeepState,
  hostWithCodeKeepState,
  joinKeepState,
} from "./multiplayer.svelte";

describe("multiplayer joiner state - hard reset path", () => {
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

  it("join() clears disconnectedSince + peerEverPaired synchronously, then sets status to 'joining'", async () => {
    mpState.disconnectedSince = 1_700_000_000_000;
    mpState.peerEverPaired = true;

    // Capture the promise so its eventual rejection has a handler, but
    // never await it: the fake WebSocket never opens, so join stays pending.
    const joinPromise = join("123456");
    joinPromise.catch(() => { /* no WebSocket in test env; expected */ });

    // Synchronous slice - the pre-join reset happens inside join() before
    // the WebSocket is created.
    expect(mpState.disconnectedSince).toBeNull();
    expect(mpState.peerEverPaired).toBe(false);
    expect(mpState.code).toBe("123456");
    expect(mpState.role).toBe("joiner");
    expect(mpState.status).toBe("joining");
  });
});

// The soft-reconnect helper isn't exported, but its core teardown primitive -
// `destroyPeerKeepState` - is. Asserting its behaviour directly is sufficient
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

  it("destroyPeerKeepState is idempotent - second call doesn't clobber preserved fields", () => {
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

// Rejoin variants: hostWithCodeKeepState + joinKeepState use the soft
// teardown internally so peerEverPaired + disconnectedSince survive the
// rebind. This is what makes GraceBanner stay visible while the lobby's
// Rejoin button re-anchors the transport.
describe("multiplayer rejoin (keep-state variants)", () => {
  beforeEach(() => disconnect());

  it("hostWithCodeKeepState preserves peerEverPaired + disconnectedSince", () => {
    mpState.role = "host";
    mpState.code = "424242";
    mpState.peerEverPaired = true;
    mpState.disconnectedSince = 1_700_000_000_000;
    mpState.status = "disconnected";

    const p = hostWithCodeKeepState("424242");
    p.catch(() => { /* no relay in test env */ });

    // Synchronous slice: destroyPeerKeepState + role write happen before the
    // WebSocket dial. The banner-critical fields must not have been cleared
    // by a hard reset.
    expect(mpState.role).toBe("host");
    expect(mpState.peerEverPaired).toBe(true);
    expect(mpState.disconnectedSince).toBe(1_700_000_000_000);
  });

  it("joinKeepState preserves peerEverPaired + disconnectedSince", () => {
    mpState.role = "joiner";
    mpState.code = "424242";
    mpState.peerEverPaired = true;
    mpState.disconnectedSince = 1_700_000_000_000;
    mpState.status = "disconnected";

    const p = joinKeepState("424242");
    p.catch(() => { /* no relay in test env */ });

    expect(mpState.role).toBe("joiner");
    expect(mpState.peerEverPaired).toBe(true);
    expect(mpState.disconnectedSince).toBe(1_700_000_000_000);
  });

  // The transport's auto-redial ladder is now symmetric across roles: a host
  // whose WS drops re-binds via `bindJoiner`, and the relay + onPromotedToHost
  // path restores the host role. There isn't a WebSocket harness in this test
  // suite yet, so we can't drive a full close/rebind cycle here - but the
  // wrapper's public shape guarantees the entry point exists for both roles:
  // hostWithCodeKeepState and joinKeepState both call the same transport path.
  it("hostWithCodeKeepState and joinKeepState both leave role writable for the transport's onPromotedToHost callback", () => {
    // Start as host; call joinKeepState - role should flip to joiner (matches
    // what the relay does when a session promotion happens the other way).
    mpState.role = "host";
    const p1 = joinKeepState("424242");
    p1.catch(() => { /* expected */ });
    expect(mpState.role).toBe("joiner");

    // Start as joiner; call hostWithCodeKeepState - role should flip to host.
    mpState.role = "joiner";
    const p2 = hostWithCodeKeepState("424242");
    p2.catch(() => { /* expected */ });
    expect(mpState.role).toBe("host");
  });
});