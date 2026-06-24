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
});

let peer: Peer | null = null;
let conn: DataConnection | null = null;
let pingTimer: ReturnType<typeof setInterval> | null = null;
const dataHandlers = new Set<(msg: WireMessage) => void>();
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
  return derivePillState(mpState.status, mpState.lastPongAt, nowTick);
}

/** Subscribe to incoming wire messages. Returns a disposer. Ping/pong is
 *  handled internally and never reaches subscribers. */
export function onData(cb: (msg: WireMessage) => void): () => void {
  dataHandlers.add(cb);
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
    startHeartbeat();
  });
  c.on("data", (raw: unknown) => {
    if (typeof raw !== "string") return;
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
    for (const h of dataHandlers) h(msg);
  });
  c.on("close", () => {
    mpState.status = "disconnected";
  });
  c.on("error", (e) => {
    mpState.lastError = e?.message ?? String(e);
    mpState.status = "disconnected";
  });
}

/** Host a session. Picks a random 6-digit code and registers with the
 *  PeerJS broker; retries on collision. Resolves with the chosen code. */
export function host(): Promise<string> {
  disconnect();
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

/** Join a session by 6-digit code. Resolves when the data channel opens. */
export function join(code: string): Promise<void> {
  disconnect();
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
