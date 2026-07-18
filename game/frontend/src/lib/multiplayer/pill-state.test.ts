// Unit coverage for derivePillStateWithAnchor. Locks in the anchor-write
// invariant (the GraceBanner depends on `disconnectedSince` being seeded
// once and only once when an opponent was actually present).

import { describe, expect, it } from "vitest";
import { derivePillStateWithAnchor } from "./pill-state";

const T_NOW = 1_000_000;

describe("derivePillStateWithAnchor - pill branch coverage", () => {
  it("status='connected' with fresh pong → live, no anchor proposed", () => {
    const out = derivePillStateWithAnchor({
      status: "connected",
      lastPongAt: T_NOW - 500,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("live");
    expect(out.nextDisconnectedSince).toBeNull();
  });

  it("status='connected' with pong in 6–15s window → unstable, no anchor", () => {
    const out = derivePillStateWithAnchor({
      status: "connected",
      lastPongAt: T_NOW - 8_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("unstable");
    expect(out.nextDisconnectedSince).toBeNull();
  });

  it("status='connected' with stale pong → disconnected", () => {
    const out = derivePillStateWithAnchor({
      status: "connected",
      lastPongAt: T_NOW - 30_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("disconnected");
  });

  it("status='disconnected' past forfeit window → forfeit", () => {
    const out = derivePillStateWithAnchor({
      status: "disconnected",
      lastPongAt: T_NOW - 6 * 60_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: T_NOW - 6 * 60_000,
    });
    expect(out.pill).toBe("forfeit");
  });

  it("transient statuses (hosting/joining/connecting) → disconnected (UI hides pill)", () => {
    for (const status of ["hosting", "joining", "connecting", "idle"] as const) {
      const out = derivePillStateWithAnchor({
        status,
        lastPongAt: null,
        now: T_NOW,
        peerEverPaired: false,
        disconnectedSince: null,
      });
      expect(out.pill).toBe("disconnected");
    }
  });
});

describe("derivePillStateWithAnchor - anchor invariants", () => {
  it("anchors when pill goes disconnected AND peerEverPaired AND no prior anchor", () => {
    const out = derivePillStateWithAnchor({
      status: "disconnected",
      lastPongAt: T_NOW - 30_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("disconnected");
    expect(out.nextDisconnectedSince).toBe(T_NOW);
  });

  it("does NOT anchor when peerEverPaired is false (host's pre-join window)", () => {
    const out = derivePillStateWithAnchor({
      status: "disconnected",
      lastPongAt: null,
      now: T_NOW,
      peerEverPaired: false,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("disconnected");
    expect(out.nextDisconnectedSince).toBeNull();
  });

  it("does NOT re-anchor when disconnectedSince is already set (idempotent)", () => {
    const out = derivePillStateWithAnchor({
      status: "disconnected",
      lastPongAt: T_NOW - 30_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: T_NOW - 10_000,
    });
    expect(out.pill).toBe("disconnected");
    expect(out.nextDisconnectedSince).toBeNull();
  });

  it("also anchors when pill is forfeit and no prior anchor (defensive)", () => {
    const out = derivePillStateWithAnchor({
      status: "disconnected",
      lastPongAt: T_NOW - 6 * 60_000,
      now: T_NOW,
      peerEverPaired: true,
      disconnectedSince: null,
    });
    expect(out.pill).toBe("forfeit");
    expect(out.nextDisconnectedSince).toBe(T_NOW);
  });

  it("never anchors when pill is live/unstable", () => {
    for (const lastPongAt of [T_NOW - 500, T_NOW - 5_000]) {
      const out = derivePillStateWithAnchor({
        status: "connected",
        lastPongAt,
        now: T_NOW,
        peerEverPaired: true,
        disconnectedSince: null,
      });
      expect(out.nextDisconnectedSince).toBeNull();
    }
  });
});
