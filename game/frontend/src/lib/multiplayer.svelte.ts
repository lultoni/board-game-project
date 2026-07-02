// Side-effecting multiplayer wrapper. Owns the reactive `mpState` carrier,
// the message inbox/buffering, the heartbeat timers, and the V1 decode +
// ping/pong handling. PeerJS lifecycle + auto-redial ladder live in
// `./multiplayer/transport.ts` so they can be unit-tested without runes.
//
// Out of scope (deferred to L7b):
//   - Reconnect handshake / Zobrist verification
//   - Forfeit timer + Claim-win UI (banner reads `disconnectedSince` from here)

import {
  decodeMessage,
  derivePillState,
  encodeMessage,
  PILL_DISCONNECTED_MS,
  type MpStatus,
  type PillState,
  type WireMessage,
} from "./multiplayer-protocol";
import { decodeMessageV2 } from "./multiplayer-protocol-v2";
import { createWebSocketTransport, type RedialState, type TransportRole } from "./multiplayer/websocket-transport";
import { createHeartbeat } from "./multiplayer/heartbeat";

export type { WireMessage, PillState, MpStatus };

interface MpState {
  status: MpStatus;
  /** The 6-digit code (host's PeerJS ID suffix). Null until host/join resolves. */
  code: string | null;
  role: "host" | "joiner" | null;
  lastPongAt: number | null;
  lastError: string | null;
  /** Set to `Date.now()` when status flips to `"disconnected"`; cleared back
   *  to `null` when status returns to `"connected"`. GraceBanner anchors its
   *  5-minute countdown on this so the timer starts when the user actually
   *  loses the peer, not on the last successful pong. Survives `disconnect()`
   *  so a hard tear-down doesn't yank the countdown out from under the UI. */
  disconnectedSince: number | null;
  /** Latches `true` the first time we receive any traffic from the peer this
   *  session (first pong, first non-ping data). Survives a close/error —
   *  unlike `lastPongAt`, which is nulled on every drop. GraceBanner uses
   *  this to gate visibility: if we never paired up (host's pre-join
   *  rehost window), don't show the "opponent disconnected" banner.
   *  Cleared only by `disconnect()`. */
  peerEverPaired: boolean;
  /** Joiner-side auto-redial telemetry exposed for UI. Updated by the
   *  transport's `onRedialState` callback. `mode="idle"` when no retry is in
   *  flight; `"ladder"` during the bounded backoff; `"longtail"` once the
   *  ladder is exhausted (indefinite ~30s retries until host returns or the
   *  user manually disconnects). Banners read this to render
   *  "Reconnecting (attempt N, next try in Xs)". */
  redial: RedialState;
  /** Monotonic session identifier. Incremented every time we start a fresh
   *  host/join (including Rejoin variants). Routes capture this on mount and
   *  gate their own teardown by it: if the epoch has advanced, a *newer*
   *  session is live and the stale route must not tear it down.
   *
   *  Motivates the guard: SvelteKit's `onDestroy` for a route we've navigated
   *  away from can fire LATE — after the destination route has mounted and
   *  started a fresh session (e.g. joiner leaves /match/ → lobby → Rejoin
   *  → /match/, and /draft/'s onDestroy from an earlier navigation fires
   *  during the second /match/ mount, calling destroyPeerKeepState() and
   *  nuking the just-opened WS). Epoch matching turns those late teardowns
   *  into no-ops. */
  sessionEpoch: number;
}

export const mpState = $state<MpState>({
  status: "idle",
  code: null,
  role: null,
  lastPongAt: null,
  lastError: null,
  disconnectedSince: null,
  peerEverPaired: false,
  redial: { mode: "idle", attempt: 0, nextAttemptAt: null },
  sessionEpoch: 0,
});

/** Returns the current session epoch. Routes capture this on mount and pass
 *  it back to `tearDownMultiplayerOnLeave` so a stale onDestroy from a
 *  previous session can't tear down a newer one. See MpState.sessionEpoch. */
export function getSessionEpoch(): number {
  return mpState.sessionEpoch;
}

/** Route-ownership token. Monotonically increases every time a route claims
 *  ownership via `claimRouteOwnership()`. Only the token-holder is allowed
 *  to tear the session down — any prior route's late-firing onDestroy sees
 *  a mismatched token and no-ops. Zero (default) means "no route owns the
 *  session yet" (transient, between lobby and route mount).
 *
 *  Why this is stricter than sessionEpoch alone: HMR can preserve a stale
 *  route instance whose captured epoch coincidentally matches the current
 *  epoch (both zero, or both one after a single (re)join). The token
 *  bumps on every claim, so even same-epoch stale teardowns are rejected. */
let routeOwnershipToken = 0;

export function claimRouteOwnership(): number {
  routeOwnershipToken++;
  return routeOwnershipToken;
}

export function getRouteOwnershipToken(): number {
  return routeOwnershipToken;
}

const dataHandlers = new Set<(msg: WireMessage) => void>();
/** Raw-string subscribers. The role-aware wrapper (createMpEngine) reads
 *  these so it can decode v2 messages (committed, intent, snapshot, …) that
 *  the legacy WireMessage type in multiplayer-protocol.ts doesn't model.
 *  Both raw and decoded paths fire for the same inbound payload — keeping
 *  legacy `onData` subscribers unaffected during the v1→v2 cutover. */
const rawDataHandlers = new Set<(raw: string) => void>();
/** Direct callbacks for connection-lifecycle events. Fired synchronously
 *  from the transport's onOpen/onClose callbacks (plus the pong-age-out
 *  bridge and its recovery path). Consumers (createMpEngine, the lobby
 *  navigation trigger) subscribe here instead of watching `mpState.status`
 *  via `$effect` — protocol sequencing has no room for Svelte effect
 *  scheduling. See PROTOCOL_TRACE.md Part 2 §6. */
const connectedHandlers = new Set<() => void>();
const disconnectedHandlers = new Set<() => void>();
/** Single-slot-per-kind buffer for messages that arrive while no subscriber
 *  is registered. The joiner navigates from /multiplayer/ → /match/ between
 *  dispatching `resume-request` and mounting the /match/ `mpOnData` listener;
 *  a `resume-accept` arriving in that ~50ms window would otherwise be dropped
 *  and the joiner would hang on the boot screen forever. Latest-wins per
 *  kind is fine — resume-accept/reject are terminal, and pong/snapshot/ply
 *  callers are happy to skip stale buffered copies. */
const inbox = new Map<WireMessage["kind"], WireMessage>();
/** Per-kind raw buffer used by V2 subscribers (`onRawData`). The lobby's V2
 *  peek subscription unsubscribes when the joiner navigates; the destination
 *  route mounts and re-subscribes via the wrapper. Anything that arrived in
 *  that window (the first `committed`, a follow-up `snapshot`, …) needs to
 *  survive the gap. We key by the decoded kind so newer-of-same-kind wins,
 *  matching the typed inbox's semantics — committed is a special case (we
 *  could miss a ply), but the wrapper detects this via seq gaps and asks for
 *  a snapshot, which the buffer reliably delivers. Cleared on disconnect. */
const rawInbox = new Map<string, string>();
let nowTick = $state(Date.now());

// Heartbeat owns the 1Hz ping + 500ms now-tick timer handles. The callbacks
// stay in this module because they touch mpState (pong-age-out bridge writes
// `disconnectedSince` + `status`, ping emits a V1 frame via `sendData`).
//
// The now-tick doubles as the pong-age-out → status bridge: PeerJS's
// DataConnection `close` event is unreliable when the remote peer dies
// without an explicit `peer.destroy()` call. Without this bridge,
// `mpState.status` stays "connected" forever and the GraceBanner never
// appears. Once we cross the threshold, flip status so every downstream
// listener (wrapper, GraceBanner via pill, /match/ network-lost effect)
// gets the same signal a clean `conn.close` would have produced.
const heartbeat = createHeartbeat({
  onPing: () => {
    sendData({ kind: "ping", t: Date.now() });
  },
  onTick: (now: number) => {
    nowTick = now;
    if (
      mpState.status === "connected"
      && mpState.lastPongAt !== null
      && now - mpState.lastPongAt > PILL_DISCONNECTED_MS
    ) {
      // eslint-disable-next-line no-console
      console.log("[mp] pong age-out → disconnected", { age: now - mpState.lastPongAt });
      if (mpState.disconnectedSince === null) {
        mpState.disconnectedSince = now;
      }
      mpState.status = "disconnected";
      fireDisconnected();
    }
  },
});

export function pillState(): PillState {
  // Pure read. All anchor writes to `mpState.disconnectedSince` happen at
  // the source event (onClose/onError, onStatusChange, onTick age-out) —
  // this function must not mutate state, since it's read from inside
  // `$derived` blocks in the UI and Svelte 5 forbids state writes there.
  return derivePillState(mpState.status, mpState.lastPongAt, nowTick);
}

/** Subscribe to incoming wire messages. Returns a disposer. Ping/pong is
 *  handled internally and never reaches subscribers. Any messages buffered
 *  in the inbox while no subscriber was registered are drained synchronously
 *  into the new subscriber (in insertion order) before this returns. */
export function onData(cb: (msg: WireMessage) => void): () => void {
  dataHandlers.add(cb);
  if (inbox.size > 0) {
    const drained = Array.from(inbox.values());
    inbox.clear();
    for (const msg of drained) cb(msg);
  }
  return () => dataHandlers.delete(cb);
}

export function sendData(msg: WireMessage): void {
  transport.sendRaw(encodeMessage(msg));
}

/** Subscribe to raw inbound strings before they're decoded into WireMessage.
 *  Used by createMpEngine to handle v2 envelopes that the legacy decoder
 *  doesn't know about. Disposer follows the same shape as onData. Drains any
 *  V2 messages that arrived while no raw subscriber was registered (e.g. the
 *  joiner's lobby→/match/ navigation gap) into the new subscriber. */
export function onRawData(cb: (raw: string) => void): () => void {
  rawDataHandlers.add(cb);
  if (rawInbox.size > 0) {
    const drained = Array.from(rawInbox.values());
    rawInbox.clear();
    for (const raw of drained) cb(raw);
  }
  return () => rawDataHandlers.delete(cb);
}

/** Send a pre-encoded JSON string. Skips encodeMessage so the wrapper can
 *  emit v2 envelopes (committed/intent/snapshot/…) without widening the
 *  legacy WireMessage union. No-op when no peer is connected. */
export function sendRaw(raw: string): void {
  transport.sendRaw(raw);
}

/** Subscribe to connection-established events. Fires when the transport
 *  actually opens the data channel AND when the pong-age-out recovery path
 *  restores an age-outed session whose WS is still live. Callbacks fire
 *  synchronously — no microtask scheduling — so protocol sequencing is
 *  deterministic. Disposer removes the callback. */
export function onConnected(cb: () => void): () => void {
  connectedHandlers.add(cb);
  return () => connectedHandlers.delete(cb);
}

/** Subscribe to connection-lost events. Fires when the transport closes,
 *  errors, or the pong-age-out bridge trips. Disposer removes the callback. */
export function onDisconnected(cb: () => void): () => void {
  disconnectedHandlers.add(cb);
  return () => disconnectedHandlers.delete(cb);
}

function fireConnected(): void {
  for (const h of connectedHandlers) {
    try { h(); } catch { /* subscriber crash must not poison the fan-out */ }
  }
}

function fireDisconnected(): void {
  for (const h of disconnectedHandlers) {
    try { h(); } catch { /* subscriber crash must not poison the fan-out */ }
  }
}

// === Transport instantiation ============================================
//
// The transport is ignorant of mpState and the V1/V2 wire formats. It
// delivers raw strings via `onData`; we decode + fan out + handle ping/pong
// here. The role/code accessors let the transport's auto-redial loop gate
// retries on the current carrier state.

const transport = createWebSocketTransport({
  onOpen: () => {
    console.log("[mp] onOpen → status=connected");
    mpState.status = "connected";
    mpState.disconnectedSince = null;
    // The transport fires onOpen when the relay says both peers are paired
    // (peer-connected / joined / created envelopes). That is the server's
    // authoritative pairing signal — trust it as a liveness stamp so the
    // pill renders green immediately instead of sitting yellow for up to
    // 5s waiting for the first heartbeat roundtrip.
    mpState.lastPongAt = Date.now();
    heartbeat.startPings();
    fireConnected();
  },
  onData: (raw: string) => {
    // Any inbound traffic proves a peer is on the other end this session.
    // Latch for GraceBanner's visibility gate. Survives close/error.
    mpState.peerEverPaired = true;
    // Fan out the raw string to any wrapper subscribers BEFORE decoding.
    // createMpEngine consumes v2 envelopes this way without taking a dep on
    // the legacy WireMessage union.
    if (rawDataHandlers.size > 0) {
      for (const h of rawDataHandlers) h(raw);
    } else {
      // No raw subscriber yet — buffer per V2 kind so the wrapper, mounting
      // after the joiner navigates, still sees session-hello / committed /
      // snapshot that arrived in the gap.
      const v2 = decodeMessageV2(raw);
      if (v2) rawInbox.set(v2.kind, raw);
    }
    const msg = decodeMessage(raw);
    if (!msg) return;
    if (msg.kind === "ping") {
      sendData({ kind: "pong", t: msg.t });
      mpState.lastPongAt = Date.now();
      return;
    }
    if (msg.kind === "pong") {
      mpState.lastPongAt = Date.now();
      // Pong-age-out recovery: if we flipped `status="disconnected"` from
      // the tick bridge (JS timer throttling / Tauri webview suspension) but
      // the WS is actually alive — proven by this pong having just arrived —
      // restore the session in place. Without this the game bricks after any
      // long-enough JS suspension: WS stays open, no `onOpen` re-fires, and
      // `status` is stuck at "disconnected" forever. See PROTOCOL_TRACE.md
      // Part 1 §"pong-age-out bug".
      if (mpState.status === "disconnected" && transport.isActive()) {
        // eslint-disable-next-line no-console
        console.log("[mp] pong-age-out recovery — WS still open, restoring status");
        mpState.status = "connected";
        mpState.disconnectedSince = null;
        heartbeat.startPings();
        fireConnected();
      }
      return;
    }
    if (dataHandlers.size === 0) {
      inbox.set(msg.kind, msg);
      return;
    }
    for (const h of dataHandlers) h(msg);
  },
  onClose: () => {
    console.log("[mp] onClose");
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.lastPongAt = null;
    heartbeat.stopPings();
    fireDisconnected();
  },
  onError: (_message: string) => {
    console.log("[mp] onError", _message);
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.lastPongAt = null;
    heartbeat.stopPings();
    fireDisconnected();
  },
  onStatusChange: (s) => {
    console.log("[mp] onStatusChange →", s);
    mpState.status = s;
    // Anchor the disconnect timestamp at the source event, so the pure
    // `pillState()` read can stay side-effect-free (it's called from
    // $derived blocks in the UI). Match the guard used by onClose/onError:
    // only anchor when a peer was ever paired, so the host's pre-join
    // "hosting"→"disconnected" flap doesn't burn the anchor.
    if (
      (s === "disconnected")
      && mpState.disconnectedSince === null
      && mpState.peerEverPaired
    ) {
      mpState.disconnectedSince = Date.now();
    }
  },
  onCode: (code: string) => {
    mpState.code = code;
  },
  onLastError: (message: string) => {
    mpState.lastError = message;
  },
  getRole: (): TransportRole => mpState.role,
  getCode: () => mpState.code,
  onRedialState: (next: RedialState) => {
    mpState.redial = next;
  },
  onPromotedToHost: () => {
    // Relay assigned us the host role (we joined but the host slot was empty).
    mpState.role = "host";
  },
});

// === Public facade ======================================================

/** Host a session. Picks a random 6-digit code and registers with the
 *  PeerJS broker; retries on collision. Resolves with the chosen code. */
export function host(): Promise<string> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "host";
  mpState.sessionEpoch++;
  return transport.host();
}

/** Re-host a session under a specific code. Used by the lobby's Rejoin flow
 *  to reclaim the same PeerJS ID we held before the tab closed, and by the
 *  leader-handoff path where the broker may still hold the dying host's
 *  registration for several seconds. */
export function hostWithCode(code: string): Promise<string> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "host";
  mpState.sessionEpoch++;
  return transport.hostWithCode(code);
}

/** Rejoin variant of `hostWithCode` that soft-teardowns instead of hard
 *  disconnecting. Preserves `peerEverPaired`, `disconnectedSince`, and any
 *  other latched state across the rebind so GraceBanner stays visible and
 *  the pill keeps showing the code throughout. Used by the lobby's
 *  Rejoin-as-host path — the peer was paired before, so the latch should
 *  survive the rebind. */
export function hostWithCodeKeepState(code: string): Promise<string> {
  destroyPeerKeepState();
  mpState.role = "host";
  mpState.sessionEpoch++;
  return transport.hostWithCode(code);
}

/** Join a session by 6-digit code. Resolves when the data channel opens. */
export function join(code: string): Promise<void> {
  // eslint-disable-next-line no-console
  console.log("[mp] wrapper.join", code);
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "joiner";
  mpState.sessionEpoch++;
  return transport.join(code);
}

/** Rejoin variant of `join` that soft-teardowns instead of hard disconnecting.
 *  Preserves `peerEverPaired` + `disconnectedSince` across the rebind so
 *  GraceBanner stays visible and the pill keeps showing the code. Used by
 *  the lobby's Rejoin-as-joiner path. */
export function joinKeepState(code: string): Promise<void> {
  // eslint-disable-next-line no-console
  console.log("[mp] wrapper.joinKeepState", code);
  destroyPeerKeepState();
  mpState.role = "joiner";
  mpState.sessionEpoch++;
  return transport.join(code);
}

/** Tear down the peer + connection. Safe to call repeatedly. */
export function disconnect(): void {
  // eslint-disable-next-line no-console
  console.log("[mp] wrapper.disconnect");
  const wasConnected = mpState.status === "connected";
  heartbeat.stopPings();
  transport.disconnect();
  mpState.status = "idle";
  mpState.code = null;
  mpState.role = null;
  mpState.lastPongAt = null;
  mpState.peerEverPaired = false;
  inbox.clear();
  rawInbox.clear();
  heartbeat.stopTicking();
  if (wasConnected) fireDisconnected();
}

/** Soft teardown used by the leader-handoff path. Drops the PeerJS object and
 *  the open DataConnection synchronously but PRESERVES the carrier fields the
 *  takeover flow needs to remain stable: `code` (so hostWithCode can reclaim
 *  the same id), `role`/`peerEverPaired` (so GraceBanner stays visible during
 *  the swap), `disconnectedSince` (so the countdown doesn't reset).
 *
 *  Status falls back to "disconnected" — not "idle" — so the pill renders the
 *  same red dot throughout the swap. */
export function destroyPeerKeepState(): void {
  // eslint-disable-next-line no-console
  console.log("[mp] destroyPeerKeepState");
  const wasConnected = mpState.status === "connected";
  heartbeat.stopPings();
  transport.destroyPeerKeepState();
  mpState.status = "disconnected";
  mpState.lastPongAt = null;
  if (wasConnected) fireDisconnected();
}

/** True when an active session exists (host or joiner, regardless of whether
 *  the pill is currently green). Used by routes to decide whether to forward
 *  snapshots / actions to the peer. */
export function isActive(): boolean {
  return transport.isActive();
}

/** Liveness probe: open a throwaway Peer + DataConnection to `code` and
 *  resolve `true` if the channel opens AND the host doesn't kick us with a
 *  `session-full` error within a 500ms confirmation window. Else `false`.
 *  Used by the lobby to show 🟢/⚫ dots on recent-sessions cards. */
export function probeHost(code: string, timeoutMs = 2_000): Promise<boolean> {
  return transport.probeHost(code, timeoutMs);
}

/** Returns true if multiplayer is supported in this environment.
 *  WebSockets work everywhere — including Linux/webkit2gtk — so this is
 *  always true. Kept for API compatibility with existing callers. */
export function isWebRtcSupported(): boolean {
  return true;
}
