// Uniform multiplayer teardown for route unmounts. Every route (/setup/,
// /draft/, /match/) hands control back to another route (forward navigation
// like draft→match) or to the lobby. The route shouldn't have to know
// whether it's the host or joiner leaving, or which layer of state to
// preserve - it just declares whether it's going forward and whether the
// telemetry row has been finalised.
//
// Decision matrix:
//   navigatingForward=true                 → no-op. The next route inherits
//                                            the live connection.
//   telemetryFinalised=true                → hard `mpDisconnect()`. Natural
//                                            game end - both peers leave
//                                            together and the wrapper resets.
//   else (mid-match leave / route swap)    → `destroyPeerKeepState()`. Drops
//                                            the WS so the peer sees the
//                                            drop, but preserves code, role,
//                                            peerEverPaired, disconnectedSince
//                                            for a later Rejoin.
//
// Stale-teardown guard (route-ownership token). SvelteKit's `onDestroy` can
// fire LATE - after the destination route has mounted and started a fresh
// mp session. Concrete symptom: joiner leaves /match/ → lobby → Rejoin →
// /match/, and an OLD route's onDestroy fires during the SECOND /match/
// mount, calling destroyPeerKeepState() and nuking the just-opened WS.
//
// The mechanism: every route claims ownership on mount (bumping a monotonic
// token in the wrapper) and passes the claimed token back here on teardown.
// Only the current token-holder is permitted to tear down; any prior route's
// stale teardown sees a mismatched token and no-ops. This is stricter than
// a per-session epoch - HMR can preserve a stale instance whose captured
// epoch coincidentally matches the current epoch, but its token cannot
// match once a newer mount has claimed ownership.

import {
  disconnect as mpDisconnect,
  destroyPeerKeepState,
  getRouteOwnershipToken,
} from "../multiplayer.svelte";

export interface RouteLeaveOpts {
  /** True when the current route is handing off to another route in the same
   *  match flow (setup→draft, setup→match, draft→match). No teardown. */
  navigatingForward: boolean;
  /** True when the game reached a natural conclusion and telemetry has been
   *  finalised. Triggers a hard disconnect so both peers exit cleanly. */
  telemetryFinalised: boolean;
  /** Ownership token this route claimed at mount via `claimRouteOwnership()`.
   *  Only the current token-holder may tear down - if a newer route has
   *  claimed ownership since, this teardown is stale and becomes a no-op. */
  ownershipToken: number;
}

export function tearDownMultiplayerOnLeave(opts: RouteLeaveOpts): void {
  if (opts.navigatingForward) return;
  const currentToken = getRouteOwnershipToken();
  if (currentToken !== opts.ownershipToken) {
    // eslint-disable-next-line no-console
    console.log("[mp] tearDown skipped - stale ownership token", { held: opts.ownershipToken, current: currentToken });
    return;
  }
  if (opts.telemetryFinalised) {
    mpDisconnect();
    return;
  }
  destroyPeerKeepState();
}
