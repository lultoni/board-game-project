// Side-effecting multiplayer layer for L7a. Wraps PeerJS to:
//   - host a session with a 6-digit code
//   - join a session by code
//   - exchange `WireMessage` envelopes over a single DataConnection
//   - run a 1Hz ping/pong heartbeat that powers the HUD pill
//   - reject any third peer that tries to connect
//
// The reactive `mpState` carrier here is a `$state` rune — that's why this
// file uses the `.svelte.ts` extension. Pure helpers (code generation,
// wire encoding, pill derivation) live in `multiplayer-protocol.ts` so
// they can be unit-tested without runes.
//
// Out of scope for L7a (deferred to L7b):
//   - Reconnect handshake / Zobrist verification
//   - Forfeit timer + Claim-win UI
//   - State sync after a mid-match disconnect

import Peer, { type DataConnection } from "peerjs";
import {
  decodeMessage,
  derivePillState,
  encodeMessage,
  generateCode,
  type MpStatus,
  type PillState,
  type ResumeRejectReason,
  type WireMessage,
} from "./multiplayer-protocol";

export type { WireMessage, PillState, MpStatus };

interface MpState {
  status: MpStatus;
  /** The 6-digit code (host's PeerJS ID suffix). Null until host/join resolves. */
  code: string | null;
  role: "host" | "joiner" | null;
  lastPongAt: number | null;
  lastError: string | null;
  /** Set to true on the host once the joiner confirms snapshot received. */
  opponentReady: boolean;
  /** When set, the joiner sends a `resume-request` immediately after its
   *  DataConnection opens instead of waiting passively for a snapshot. The
   *  lobby's Rejoin flow populates this before calling `join(code)`. */
  pendingResume: { code: string; plyCount: number; zobrist: string } | null;
  /** Sticky reason set when the host's last `resume-reject` came in. The
   *  lobby reads this on mount and surfaces it as a user-facing error. */
  resumeFailed: ResumeRejectReason | null;
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
}

// Namespace the PeerJS ID so two random apps don't clash on the same broker.
const ID_PREFIX = "boardgame-l7a-";

export const mpState = $state<MpState>({
  status: "idle",
  code: null,
  role: null,
  lastPongAt: null,
  lastError: null,
  opponentReady: false,
  pendingResume: null,
  resumeFailed: null,
  disconnectedSince: null,
  peerEverPaired: false,
});

let peer: Peer | null = null;
let conn: DataConnection | null = null;
let pingTimer: ReturnType<typeof setInterval> | null = null;
// Per-session counter for joiner auto-redial attempts after a drop. Reset
// on every successful `open` and on `disconnect()`. Bounded by REDIAL_DELAYS
// to avoid hammering the broker forever — after the budget is exhausted the
// user falls back to manual lobby Rejoin.
let redialAttempts = 0;
const REDIAL_DELAYS = [1_500, 3_000, 6_000, 12_000, 30_000];
const dataHandlers = new Set<(msg: WireMessage) => void>();
/** Single-slot-per-kind buffer for messages that arrive while no subscriber
 *  is registered. The joiner navigates from /multiplayer/ → /match/ between
 *  dispatching `resume-request` and mounting the /match/ `mpOnData` listener;
 *  a `resume-accept` arriving in that ~50ms window would otherwise be dropped
 *  and the joiner would hang on the boot screen forever. Latest-wins per
 *  kind is fine — resume-accept/reject are terminal, and pong/snapshot/ply
 *  callers are happy to skip stale buffered copies. */
const inbox = new Map<WireMessage["kind"], WireMessage>();
let nowTick = $state(Date.now());

// Drive a coarse "now" tick so $derived pillState recomputes every 500ms
// even when no messages arrive. The tick is cheap (one Date.now + assign).
let nowTimer: ReturnType<typeof setInterval> | null = null;
function ensureNowTimer(): void {
  if (nowTimer) return;
  nowTimer = setInterval(() => (nowTick = Date.now()), 500);
}
function stopNowTimer(): void {
  if (nowTimer) {
    clearInterval(nowTimer);
    nowTimer = null;
  }
}

export function pillState(): PillState {
  const p = derivePillState(mpState.status, mpState.lastPongAt, nowTick);
  // Anchor disconnectedSince from the pill itself, not just from PeerJS
  // close/error events. The DataConnection's close/error handlers don't
  // fire reliably when the remote peer destroys its Peer object — pong
  // age-out is often the only signal we get. Without this, the GraceBanner
  // sees `disconnectedSince === null`, falls into its `deadline === null
  // ? GRACE_MS` branch, and freezes at 5:00 indefinitely. We also want the
  // anchor for `forfeit` so claim-win timing stays sensible even if we
  // landed there without a close event.
  if ((p === "disconnected" || p === "forfeit") && mpState.disconnectedSince === null) {
    // Only anchor when an opponent was actually present at some point —
    // otherwise the host's pre-join "hosting" state would mint a stale
    // anchor. peerEverPaired latches on first inbound traffic and survives
    // close/error, so it's reliable across reconnect cycles too.
    if (mpState.peerEverPaired) {
      mpState.disconnectedSince = nowTick;
    }
  }
  return p;
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
  if (!conn || !conn.open) return;
  try {
    conn.send(encodeMessage(msg));
  } catch (e) {
    mpState.lastError = (e as Error)?.message ?? String(e);
  }
}

/** Tear down the peer + connection. Safe to call repeatedly. */
export function disconnect(): void {
  if (pingTimer) {
    clearInterval(pingTimer);
    pingTimer = null;
  }
  if (conn) {
    try { conn.close(); } catch { /* noop */ }
    conn = null;
  }
  if (peer) {
    try { peer.destroy(); } catch { /* noop */ }
    peer = null;
  }
  mpState.status = "idle";
  mpState.code = null;
  mpState.role = null;
  mpState.lastPongAt = null;
  mpState.opponentReady = false;
  mpState.pendingResume = null;
  mpState.peerEverPaired = false;
  inbox.clear();
  redialAttempts = 0;
  // Note: resumeFailed is intentionally NOT cleared here — the lobby needs
  // to read it AFTER the failed connection has been torn down. The lobby
  // clears it when entering "choose" view.
  stopNowTimer();
}

function startHeartbeat(): void {
  if (pingTimer) clearInterval(pingTimer);
  ensureNowTimer();
  pingTimer = setInterval(() => {
    if (!conn || !conn.open) return;
    sendData({ kind: "ping", t: Date.now() });
  }, 1_000);
}

function bindConnection(c: DataConnection): void {
  conn = c;
  c.on("open", () => {
    mpState.status = "connected";
    // Clear any prior disconnect anchor — peer is back.
    mpState.disconnectedSince = null;
    // Reset the per-session auto-redial budget — a healthy open earns a
    // fresh set of retries the next time we drop.
    redialAttempts = 0;
    startHeartbeat();
    // If the joiner staged a resume request before dialling, fire it now so
    // the host can validate state before sending a fresh snapshot.
    if (mpState.role === "joiner" && mpState.pendingResume) {
      const r = mpState.pendingResume;
      sendData({
        kind: "resume-request",
        code: r.code,
        plyCount: r.plyCount,
        zobrist: r.zobrist,
      });
    }
  });
  c.on("data", (raw: unknown) => {
    if (typeof raw !== "string") return;
    // Any inbound traffic proves a peer is on the other end this session.
    // Latch this for GraceBanner's visibility gate, which must distinguish
    // "we never had a peer" (host's pre-join rehost) from "we lost the peer
    // we had". Survives close/error — unlike lastPongAt, which is reset.
    mpState.peerEverPaired = true;
    const msg = decodeMessage(raw);
    if (!msg) return;
    if (msg.kind === "ping") {
      sendData({ kind: "pong", t: msg.t });
      // Treat any inbound traffic as fresh — peer is clearly alive.
      mpState.lastPongAt = Date.now();
      return;
    }
    if (msg.kind === "pong") {
      mpState.lastPongAt = Date.now();
      return;
    }
    if (msg.kind === "ready") {
      mpState.opponentReady = true;
    }
    if (dataHandlers.size === 0) {
      // Nobody listening yet — buffer the latest message of this kind so
      // it survives the /multiplayer/→/match/ navigation gap.
      inbox.set(msg.kind, msg);
      return;
    }
    for (const h of dataHandlers) h(msg);
  });
  c.on("close", () => {
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.status = "disconnected";
    // Pull lastPongAt back to null so the pill stops reporting "live" against
    // a peer we've just lost. derivePillState falls through to its disconnected
    // branch once status is disconnected AND lastPongAt is null.
    mpState.lastPongAt = null;
    maybeAutoRedialJoiner();
  });
  c.on("error", (e) => {
    mpState.lastError = e?.message ?? String(e);
    if (mpState.disconnectedSince === null) {
      mpState.disconnectedSince = Date.now();
    }
    mpState.status = "disconnected";
    mpState.lastPongAt = null;
    maybeAutoRedialJoiner();
  });
}

/** Bounded retry after a joiner-side drop. Tries up to REDIAL_DELAYS.length
 *  times with increasing backoff (1.5s, 3s, 6s, 12s, 30s — ~52s total) so the
 *  host has wall-clock time to navigate back through home → multiplayer →
 *  Rejoin and re-claim the same code. The counter resets on a successful
 *  `open` and on `disconnect()`. Skipped when we're mid-resume-handshake
 *  (the resume flow has its own semantics) or when the host is the one whose
 *  side dropped (host stays put and waits for the joiner to dial back). If
 *  the budget runs out, the user falls back to manual lobby Rejoin. */
function maybeAutoRedialJoiner(): void {
  if (mpState.role !== "joiner") return;
  if (mpState.pendingResume !== null) return;
  if (mpState.code === null) return;
  if (redialAttempts >= REDIAL_DELAYS.length) return;
  const delay = REDIAL_DELAYS[redialAttempts];
  const code = mpState.code;
  redialAttempts += 1;
  setTimeout(() => {
    // Bail if the user manually navigated away / disconnected in the meantime.
    if (mpState.role !== "joiner") return;
    if (mpState.status === "connected" || mpState.status === "connecting") return;
    join(code).catch(() => maybeAutoRedialJoiner());
  }, delay);
}

/** Host a session. Picks a random 6-digit code and registers with the
 *  PeerJS broker; retries on collision. Resolves with the chosen code. */
export function host(): Promise<string> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "host";
  mpState.status = "hosting";
  return new Promise((resolve, reject) => {
    const tryOne = (attemptsLeft: number) => {
      const code = generateCode();
      const p = new Peer(ID_PREFIX + code);
      p.on("open", () => {
        peer = p;
        mpState.code = code;
        // Wait for an incoming DataConnection.
        p.on("connection", (c) => {
          if (conn && conn.open) {
            // Reject any third peer.
            c.on("open", () => {
              try {
                c.send(encodeMessage({ kind: "error", reason: "session-full" }));
              } finally {
                try { c.close(); } catch { /* noop */ }
              }
            });
            return;
          }
          mpState.status = "connecting";
          bindConnection(c);
        });
        resolve(code);
      });
      p.on("error", (e) => {
        const msg = e?.message ?? String(e);
        // "ID is taken" → retry with a fresh code.
        if (attemptsLeft > 0 && /taken|unavailable-id/i.test(msg)) {
          try { p.destroy(); } catch { /* noop */ }
          tryOne(attemptsLeft - 1);
          return;
        }
        mpState.lastError = msg;
        mpState.status = "error";
        try { p.destroy(); } catch { /* noop */ }
        reject(e);
      });
    };
    tryOne(5);
  });
}

/** Re-host a session under a specific code. Used by the lobby's Rejoin flow
 *  to reclaim the same PeerJS ID we held before the tab closed. Unlike
 *  `host()`, this does NOT retry on collision — if the code is already taken,
 *  someone else grabbed it while we were away, and the caller surfaces that
 *  to the user. */
export function hostWithCode(code: string): Promise<string> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "host";
  mpState.status = "hosting";
  return new Promise((resolve, reject) => {
    const p = new Peer(ID_PREFIX + code);
    p.on("open", () => {
      peer = p;
      mpState.code = code;
      p.on("connection", (c) => {
        if (conn && conn.open) {
          c.on("open", () => {
            try {
              c.send(encodeMessage({ kind: "error", reason: "session-full" }));
            } finally {
              try { c.close(); } catch { /* noop */ }
            }
          });
          return;
        }
        mpState.status = "connecting";
        bindConnection(c);
      });
      resolve(code);
    });
    p.on("error", (e) => {
      const msg = e?.message ?? String(e);
      mpState.lastError = msg;
      mpState.status = "error";
      try { p.destroy(); } catch { /* noop */ }
      reject(e);
    });
  });
}

/** Join a session by 6-digit code. Resolves when the data channel opens. */
export function join(code: string): Promise<void> {
  disconnect();
  mpState.disconnectedSince = null;
  mpState.role = "joiner";
  mpState.status = "joining";
  mpState.code = code;
  return new Promise((resolve, reject) => {
    // Use a random PeerJS ID for the joiner; we never need to be dialled.
    const myId = ID_PREFIX + "j-" + Math.random().toString(36).slice(2, 10);
    const p = new Peer(myId);
    p.on("open", () => {
      peer = p;
      mpState.status = "connecting";
      const c = p.connect(ID_PREFIX + code, { reliable: true });
      bindConnection(c);
      c.on("open", () => resolve());
      c.on("error", (e) => {
        mpState.lastError = e?.message ?? String(e);
        mpState.status = "error";
        reject(e);
      });
    });
    p.on("error", (e) => {
      const msg = e?.message ?? String(e);
      mpState.lastError = msg;
      mpState.status = /peer-unavailable/i.test(msg) ? "error" : "error";
      try { p.destroy(); } catch { /* noop */ }
      reject(e);
    });
  });
}

/** True when an active session exists (host or joiner, regardless of whether
 *  the pill is currently green). Used by routes to decide whether to forward
 *  snapshots / actions to the peer. */
export function isActive(): boolean {
  return peer !== null && mpState.role !== null;
}

/** Liveness probe: open a throwaway Peer + DataConnection to `code` and
 *  resolve `true` if the channel opens AND the host doesn't kick us with a
 *  `session-full` error within a 500ms confirmation window. Else `false`.
 *  Used by the lobby to show 🟢/⚫ dots on recent-sessions cards.
 *
 *  Why the confirmation window: a host that's already paired with another
 *  joiner will accept the open() (PeerJS auto-opens the data channel) and
 *  THEN send `{ kind: "error", reason: "session-full" }` before closing.
 *  Resolving `true` on open alone gives false positives for hosts that
 *  wouldn't accept this user. 500ms covers typical broker latency; longer
 *  delays will register as a false 🟢 but the worst outcome is a hint —
 *  Rejoin still works either way.
 *
 *  Does NOT touch the singleton `peer`/`conn` used by host()/join(). Always
 *  tears down its own Peer in the finally path so probe traffic never lingers
 *  on the broker.
 */
export function probeHost(code: string, timeoutMs = 2_000): Promise<boolean> {
  return new Promise((resolve) => {
    const myId = ID_PREFIX + "probe-" + Math.random().toString(36).slice(2, 10);
    let p: Peer | null = null;
    let c: DataConnection | null = null;
    let settled = false;
    let confirmTimer: ReturnType<typeof setTimeout> | null = null;
    const finish = (result: boolean): void => {
      if (settled) return;
      settled = true;
      if (confirmTimer) clearTimeout(confirmTimer);
      if (c) { try { c.close(); } catch { /* noop */ } }
      if (p) { try { p.destroy(); } catch { /* noop */ } }
      resolve(result);
    };
    const timer = setTimeout(() => finish(false), timeoutMs);
    try {
      p = new Peer(myId);
      p.on("open", () => {
        if (!p) return;
        c = p.connect(ID_PREFIX + code, { reliable: true });
        c.on("open", () => {
          clearTimeout(timer);
          // Watch for a session-full kick from the paired host before
          // declaring victory.
          c?.on("data", (raw: unknown) => {
            if (typeof raw !== "string") return;
            const msg = decodeMessage(raw);
            if (msg && msg.kind === "error" && msg.reason === "session-full") {
              finish(false);
            }
          });
          confirmTimer = setTimeout(() => finish(true), 500);
        });
        c.on("error", () => {
          clearTimeout(timer);
          finish(false);
        });
      });
      p.on("error", () => {
        clearTimeout(timer);
        finish(false);
      });
    } catch {
      clearTimeout(timer);
      finish(false);
    }
  });
}
