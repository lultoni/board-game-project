// Pure decision-matrix tests for the shared route teardown helper. The helper
// picks between "no-op / hard disconnect / soft teardown" based on two flags
// AND a route-ownership token (stale-teardown guard); mocking the wrapper
// lets us assert exactly which primitive is called.

import { beforeEach, describe, expect, it, vi } from "vitest";

const { mpDisconnect, destroyPeerKeepState, getRouteOwnershipToken } = vi.hoisted(() => ({
  mpDisconnect: vi.fn(),
  destroyPeerKeepState: vi.fn(),
  getRouteOwnershipToken: vi.fn(() => 0),
}));

vi.mock("../multiplayer.svelte", () => ({
  disconnect: mpDisconnect,
  destroyPeerKeepState,
  getRouteOwnershipToken,
}));

// Import AFTER the mock so the module resolves to our doubles.
import { tearDownMultiplayerOnLeave } from "./route-lifecycle";

describe("tearDownMultiplayerOnLeave", () => {
  beforeEach(() => {
    mpDisconnect.mockReset();
    destroyPeerKeepState.mockReset();
    getRouteOwnershipToken.mockReset();
    getRouteOwnershipToken.mockReturnValue(1);
  });

  it("no-op when navigatingForward=true (the next route inherits the connection)", () => {
    tearDownMultiplayerOnLeave({ navigatingForward: true, telemetryFinalised: false, ownershipToken: 1 });
    expect(mpDisconnect).not.toHaveBeenCalled();
    expect(destroyPeerKeepState).not.toHaveBeenCalled();
  });

  it("no-op when navigatingForward=true, even if telemetry is finalised", () => {
    // navigatingForward takes precedence — a forward handoff shouldn't tear
    // down even at the end of the pipeline. (Setup→match with preMade mode
    // isn't strictly a natural end, but the flag combination shouldn't cause
    // a hard disconnect either.)
    tearDownMultiplayerOnLeave({ navigatingForward: true, telemetryFinalised: true, ownershipToken: 1 });
    expect(mpDisconnect).not.toHaveBeenCalled();
    expect(destroyPeerKeepState).not.toHaveBeenCalled();
  });

  it("hard disconnect when telemetry has finalised (natural game-end)", () => {
    tearDownMultiplayerOnLeave({ navigatingForward: false, telemetryFinalised: true, ownershipToken: 1 });
    expect(mpDisconnect).toHaveBeenCalledTimes(1);
    expect(destroyPeerKeepState).not.toHaveBeenCalled();
  });

  it("soft teardown when leaving mid-match (default fallback)", () => {
    tearDownMultiplayerOnLeave({ navigatingForward: false, telemetryFinalised: false, ownershipToken: 1 });
    expect(destroyPeerKeepState).toHaveBeenCalledTimes(1);
    expect(mpDisconnect).not.toHaveBeenCalled();
  });

  it("no-op when the ownership token is stale (a newer route has claimed ownership)", () => {
    // Simulates the failing pattern: /draft/'s onDestroy fires during a later
    // /match/ mount after Rejoin. /match/ has claimed a newer token by then;
    // the stale teardown must not touch the fresh session.
    getRouteOwnershipToken.mockReturnValue(3);
    tearDownMultiplayerOnLeave({ navigatingForward: false, telemetryFinalised: false, ownershipToken: 2 });
    expect(destroyPeerKeepState).not.toHaveBeenCalled();
    expect(mpDisconnect).not.toHaveBeenCalled();
  });

  it("stale-token guard also blocks the hard-disconnect branch", () => {
    getRouteOwnershipToken.mockReturnValue(5);
    tearDownMultiplayerOnLeave({ navigatingForward: false, telemetryFinalised: true, ownershipToken: 3 });
    expect(mpDisconnect).not.toHaveBeenCalled();
    expect(destroyPeerKeepState).not.toHaveBeenCalled();
  });
});
