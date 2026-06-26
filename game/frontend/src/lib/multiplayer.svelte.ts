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
  encodeMessage,
  PILL_DISCONNECTED_MS,
  type MpStatus,
  type PillState,
  type WireMessage,
} from "./multiplayer-protocol";
import { decodeMessageV2 } from "./multiplayer-protocol-v2";
import { createPeerJsTransport, type RedialState, type TransportRole } from "./multiplayer/transport";
import { derivePillStateWithAnchor } from "./multiplayer/pill-state";
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
});

const dataHandlers = new Set<(msg: WireMessage) => void>();
/** Raw-string subscribers. The role-aware wrapper (createMpEngine) reads
 *  these so it can decode v2 messages (committed, intent, snapshot, …) that
 *  the legacy WireMessage type in multiplayer-protocol.ts doesn't model.
 *  Both raw and decoded paths fire for the same inbound payload — keeping
 *  legacy `onData` subscribers unaffected during the v1→v2 cutover. */
const rawDataHandlers = new Set<(raw: string) => void>();
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
    }
  },
});

export function pillState(): PillState {
  const out = derivePillStateWithAnchor({
    status: mpState.status,
    lastPongAt: mpState.lastPongAt,
    now: nowTick,
    peerEverPaired: mpState.peerEverPaired,
    disconnectedSince: mpState.disconnectedSince,
  });
  // The pure derivation tells us *whether* to anchor; the wrapper is the one
  // place that actually writes mpState. Idempotent: the derivation returns
  // null whenever the anchor is already set.
  if (out.nextDisconnectedSince !== null) {
    mpState.disconnectedSince = out.nextDisconnectedSince;
  }
  return out.pill;
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

// === Transport instantiation ============================================
//
// The transport is ignorant of mpState and the V1/V2 wire formats. It
// delivers raw strings via `onData`; we decode + fan out + handle ping/pong
// here. The role/code accessors let the transport's auto-redial loop gate
// retries on the current carrier state.

const transport = createPeerJsTransport({
  onOpen: () => {
    mpState.status = "connected";
    mpState.disconnectedSince = null;
    heartbeat.startPings();
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
      return;
    }
    if (dataHandlers.size === 0) {
      inbox.set(msg.kind, msg);
      return;
    }
    for (const h of dataHandlers) h(msg);
  },
  onClose: () => {
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.lastPongAt = null;
    heartbeat.stopPings();
  },
  onError: (_message: string) => {
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.lastPongAt = null;
    heartbeat.stopPings();
  },
  onStatusChange: (s) => {
    mpState.status = s;
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
});

// === Public facade ======================================================

/** Host a session. Picks a random 6-digit code and registers with the
 *  PeerJS broker; retries on collision. Resolves with the chosen code. */
export function host(): Promise<string> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "host";
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
  return transport.hostWithCode(code);
}

/** Join a session by 6-digit code. Resolves when the data channel opens. */
export function join(code: string): Promise<void> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "joiner";
  return transport.join(code);
}

/** Tear down the peer + connection. Safe to call repeatedly. */
export function disconnect(): void {
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
  heartbeat.stopPings();
  transport.destroyPeerKeepState();
  mpState.status = "disconnected";
  mpState.lastPongAt = null;
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
