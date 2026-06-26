// PeerJS transport — owns the WebRTC/broker lifecycle and the joiner-side
// auto-redial ladder. Plain TS (no Svelte runes) so it can be unit-tested
// without the test harness pulling in $state.
//
// The transport is intentionally ignorant of:
//   - `mpState` and its shape (we expose callbacks instead)
//   - the V1 `WireMessage` union and the V2 envelope kinds (we deliver raw
//     strings; the wrapper module decodes them)
//   - ping/pong (the wrapper handles those after decoding)
//
// What the transport DOES own:
//   - The singleton `peer`/`conn` PeerJS objects for the live session
//   - host / hostWithCode / join + their broker-retry policies
//   - The soft `destroyPeerKeepState` teardown used by leader handoff
//   - The auto-redial ladder for the joiner side of a drop
//   - `probeHost` (a throwaway PeerJS instance — never touches the singleton)
//
// Everything visible to the wrapper goes through `TransportCallbacks`; the
// wrapper threads role/code back via `getRole` / `getCode` so the transport
// can gate the redial loop without owning a copy of the state.

import Peer, { type DataConnection } from "peerjs";

/** PeerJS namespace prefix. Default matches the L7a wire format — must NOT
 *  change without coordinating a wire-format bump, since two peers on
 *  different prefixes will never find each other on the broker. */
const DEFAULT_ID_PREFIX = "boardgame-l7a-";

/** Joiner-side auto-redial backoff ladder. First slot is intentionally short
 *  (~400ms) — when the host vanishes and returns quickly via `hostWithCode`,
 *  the joiner's broker registration drops almost immediately. A near-instant
 *  first redial lands as soon as the host's reclaim resolves; the longer
 *  slots take over if the host stays away. Once exhausted, retries continue
 *  indefinitely at `LONG_TAIL_DELAY_MS` (see below) so a slow lobby Rejoin
 *  still finds the joiner alive. */
const DEFAULT_REDIAL_DELAYS = [400, 1_500, 3_000, 6_000, 12_000, 30_000];

/** Long-tail retry cadence after the ladder is exhausted. Indefinite — only
 *  stops on a successful `open` or an explicit `disconnect()` that clears
 *  role/code. 30s keeps broker chatter low while still feeling responsive. */
const LONG_TAIL_DELAY_MS = 30_000;

/** Per-attempt watchdog timeout for joiner-side redial. PeerJS's `p.connect()`
 *  can silently hang if the target id was registered but the WebRTC
 *  negotiation gets lost (broker says "ok" but the offer/answer never
 *  finalizes) — no `c.on("open")`, no `c.on("error")`, just nothing. Without
 *  a watchdog, the redial loop dead-ends on that single hung attempt. 5s is
 *  more than enough for a healthy negotiation; anything longer almost
 *  certainly won't recover. */
const REDIAL_ATTEMPT_TIMEOUT_MS = 5_000;

/** hostWithCode broker-eviction retry budget. ~8.8s total — the broker can
 *  hold the previous registration for several seconds after the old peer
 *  dies, so we retry through that window for the leader-handoff path. */
const DEFAULT_HOST_RETRY_DELAYS = [800, 1_500, 2_500, 4_000];

/** Broad pattern matching transient host-side errors that warrant a retry
 *  on hostWithCode. Includes broker "id taken" plus WebRTC/WebSocket
 *  transport errors that flap during fast handoffs. */
const TRANSIENT_HOST_ERROR = /taken|unavailable-id|network|socket-(?:error|closed)|disconnected/i;

export type TransportStatus =
  | "hosting"
  | "joining"
  | "connecting"
  | "connected"
  | "disconnected"
  | "error";

export type TransportRole = "host" | "joiner" | null;

/** Joiner-side redial state for UI display. `mode` is "idle" when no retry is
 *  scheduled (either we never dropped or we're already connected), "ladder"
 *  during the bounded backoff (attempts 1..ladderLength), "longtail" once the
 *  ladder is exhausted and we're retrying every LONG_TAIL_DELAY_MS forever. */
export interface RedialState {
  mode: "idle" | "ladder" | "longtail";
  /** 1-based attempt counter for the current drop. 0 when idle. */
  attempt: number;
  /** Wall-clock ms timestamp of the next scheduled retry, or null when idle. */
  nextAttemptAt: number | null;
}

export interface TransportOpts {
  /** PeerJS ID namespace. Defaults to "boardgame-l7a-". */
  idPrefix?: string;
  /** Joiner-side auto-redial backoff schedule (ms). */
  redialDelays?: number[];
  /** hostWithCode broker-eviction retry schedule (ms). */
  hostRetryDelays?: number[];
  /** Diagnostic sink. Defaults to console.log so production keeps the same
   *  visibility we had before the split. Tests pass a no-op. */
  log?: (event: string, detail?: unknown) => void;
}

export interface TransportCallbacks {
  /** Conn opened end-to-end. Wrapper flips mpState.status → "connected",
   *  clears disconnect anchors, and starts the heartbeat. */
  onOpen(): void;
  /** Raw string delivered by the peer. Wrapper decodes V1+V2 from this. */
  onData(raw: string): void;
  /** Conn closed cleanly. Wrapper flips status and runs auto-redial gating. */
  onClose(): void;
  /** Conn errored. Wrapper surfaces lastError (unless suppression is on),
   *  flips status, and runs auto-redial gating. */
  onError(message: string): void;
  /** Status transition (hosting/joining/connecting/connected/disconnected/error).
   *  Wrapper mirrors this onto mpState.status. */
  onStatusChange(s: TransportStatus): void;
  /** Final code emitted after broker registration. Wrapper writes mpState.code. */
  onCode(code: string): void;
  /** Broker-level error message. Wrapper writes mpState.lastError — but only
   *  when redial suppression is OFF. The transport already gates this internally;
   *  the wrapper just records the value. */
  onLastError(message: string): void;
  /** Live role accessor — read at every auto-redial tick so a manual
   *  disconnect() between scheduling and firing cancels the retry. */
  getRole(): TransportRole;
  /** Live code accessor — read when the auto-redial soft-reconnect fires. */
  getCode(): string | null;
  /** Joiner-side redial telemetry. Fires every time the state transitions
   *  (scheduled → fired, ladder → longtail, retry → idle on success). The
   *  wrapper mirrors this onto mpState so banners can render "Reconnecting
   *  (attempt N, next try in Xs)". */
  onRedialState?(state: RedialState): void;
}

export interface Transport {
  host(): Promise<string>;
  hostWithCode(code: string): Promise<string>;
  join(code: string): Promise<void>;
  disconnect(): void;
  destroyPeerKeepState(): void;
  sendRaw(raw: string): void;
  probeHost(code: string, timeoutMs?: number): Promise<boolean>;
  isActive(): boolean;
}

export function createPeerJsTransport(
  cbs: TransportCallbacks,
  opts: TransportOpts = {},
): Transport {
  const idPrefix = opts.idPrefix ?? DEFAULT_ID_PREFIX;
  const redialDelays = opts.redialDelays ?? DEFAULT_REDIAL_DELAYS;
  const hostRetryDelays = opts.hostRetryDelays ?? DEFAULT_HOST_RETRY_DELAYS;
  const log = opts.log ?? ((event, detail) => {
    // eslint-disable-next-line no-console
    console.log(`[mp] ${event}`, detail);
  });

  // === Session-singleton state ===========================================

  let peer: Peer | null = null;
  let conn: DataConnection | null = null;
  // Per-drop counter for joiner auto-redial attempts. Reset on every successful
  // `open` and on `disconnect()`. Ladder phase covers attempts 1..redialDelays.length
  // with increasing backoff; after that we switch to indefinite long-tail at
  // LONG_TAIL_DELAY_MS so a slow lobby Rejoin still finds the joiner alive.
  let redialAttempts = 0;
  // Guard against duplicate scheduling. Both `conn.close` AND `conn.error` can
  // fire for the same drop (and `bindJoinerPeer`'s inner error handler used to
  // fire a second time when a softReconnect failed inside bindConnection); each
  // path called `maybeAutoRedialJoiner`, halving the effective budget. The
  // guard ensures one retry per failure window. Cleared when the scheduled
  // attempt fires (or is bailed out of) so a real subsequent drop can still
  // schedule.
  let redialPending = false;
  // Mirror lives in the wrapper (mpState); transport just emits transitions.
  // While the joiner's auto-redial is in flight, transient PeerJS errors
  // (`Could not connect to peer …`, network blips) are normal — the retry
  // will mask them. Suppress writing them to lastError so the banner doesn't
  // surface a misleading toast during recovery. Cleared on successful
  // reconnect (bindConnection's "open") or on disconnect().
  let suppressingRedialErrors = false;

  /** True while we're inside a `softReconnectJoiner` attempt (the retry loop)
   *  rather than the user's initial `join()`. Initial-join failures must NOT
   *  trigger another redial — the lobby UI expects a one-shot reject and the
   *  user is sitting on the "choose" view; spinning retries in the background
   *  would silently re-dial dead codes. */
  let inRedialAttempt = false;

  function emitRedialState(next: RedialState): void {
    cbs.onRedialState?.(next);
  }

  // === Helpers ============================================================

  function bindConnection(c: DataConnection): void {
    conn = c;
    log("bindConnection", { peerId: c.peer });
    c.on("open", () => {
      log("conn.open", { peerId: c.peer });
      // Reset the per-drop auto-redial state — a healthy open earns a fresh
      // ladder the next time we drop.
      redialAttempts = 0;
      redialPending = false;
      emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
      // Recovery succeeded — let any subsequent unrelated errors surface again.
      suppressingRedialErrors = false;
      cbs.onStatusChange("connected");
      cbs.onOpen();
    });
    c.on("data", (raw: unknown) => {
      if (typeof raw !== "string") return;
      cbs.onData(raw);
    });
    c.on("close", () => {
      log("conn.close", { peerId: c.peer });
      cbs.onStatusChange("disconnected");
      cbs.onClose();
      maybeAutoRedialJoiner();
    });
    c.on("error", (e) => {
      const msg = e?.message ?? String(e);
      log("conn.error", { peerId: c.peer, error: msg });
      if (!suppressingRedialErrors) {
        cbs.onLastError(msg);
      }
      cbs.onStatusChange("disconnected");
      cbs.onError(msg);
      maybeAutoRedialJoiner();
    });
  }

  /** Joiner-side auto-redial after a drop. Two phases:
   *
   *   1. Ladder (attempts 1..redialDelays.length) — bounded backoff so a
   *      quickly-returning host pairs up fast.
   *   2. Long-tail (attempt redialDelays.length+1 onwards) — indefinite
   *      retries at LONG_TAIL_DELAY_MS until either `open` succeeds or
   *      `disconnect()` clears role/code.
   *
   *  Skipped when this peer is the host (host stays put and waits for the
   *  joiner to dial back). The `redialPending` guard prevents the same drop
   *  from scheduling twice when both `conn.close` and `conn.error` fire. */
  function maybeAutoRedialJoiner(): void {
    if (cbs.getRole() !== "joiner") return;
    if (cbs.getCode() === null) return;
    if (redialPending) return;
    const inLadder = redialAttempts < redialDelays.length;
    const delay = inLadder ? redialDelays[redialAttempts] : LONG_TAIL_DELAY_MS;
    const code = cbs.getCode();
    if (code === null) return;
    redialAttempts += 1;
    redialPending = true;
    // From here until either a successful `open` or an explicit disconnect,
    // suppress lastError writes — every mid-recovery `Could not connect to
    // peer …` is expected and would otherwise toast misleadingly.
    suppressingRedialErrors = true;
    emitRedialState({
      mode: inLadder ? "ladder" : "longtail",
      attempt: redialAttempts,
      nextAttemptAt: Date.now() + delay,
    });
    setTimeout(() => {
      redialPending = false;
      // Bail if the user manually navigated away / disconnected in the meantime.
      if (cbs.getRole() !== "joiner") return;
      if (cbs.getCode() === null) return;
      // bindConnection's close/error handlers will call back into
      // maybeAutoRedialJoiner if this attempt fails — no .catch chain needed
      // to schedule the next attempt. We still swallow the rejection so
      // bindJoinerPeer's reject path doesn't surface as an unhandled promise.
      softReconnectJoiner(code).catch(() => { /* handled via close/error */ });
    }, delay);
  }

  /** Soft reconnect used by the auto-redial loop. Drops the dying peer
   *  synchronously but preserves carrier state up in the wrapper (the wrapper
   *  decides what survives — transport just calls destroyPeerKeepState to
   *  tear down the socket without firing `disconnect()`'s status=idle reset).
   *
   *  Caller invariant: getRole() === "joiner" and getCode() === code at call. */
  function softReconnectJoiner(code: string): Promise<void> {
    destroyPeerKeepStateInternal();
    cbs.onStatusChange("joining");
    inRedialAttempt = true;
    return bindJoinerPeer(code).finally(() => {
      inRedialAttempt = false;
    });
  }

  /** Inner joiner PeerJS handshake. Shared by `join()` (which resets state in
   *  the wrapper first) and `softReconnectJoiner()` (which preserves it).
   *
   *  Error paths during a redial loop (host not yet rehosted):
   *    - `peer-unavailable` is a peer-level error → `p.on("error")` fires
   *      (the broker tells us the target id isn't registered). Conn-level
   *      handlers in bindConnection never fire because the conn was never
   *      established. We must call `maybeAutoRedialJoiner` here ourselves.
   *    - Mid-session drops where a conn WAS open and then closed fire
   *      `c.on("close")`/`c.on("error")` in bindConnection, which already
   *      call `maybeAutoRedialJoiner`.
   *
   *  Once `c.on("open")` has fired (the conn is live), peer-level errors
   *  (`disconnected`, `network`, …) from PeerJS's broker socket DO NOT kill
   *  the conn — they just mean the broker handshake hiccupped. Don't touch
   *  status in that case; the conn's own handlers will report a real drop.
   *
   *  Silent-hang case: PeerJS sometimes returns a DataConnection from
   *  `p.connect()` to an unregistered target and then never fires any callback
   *  on it (`open`/`error`/`close` are all silent). The watchdog
   *  (REDIAL_ATTEMPT_TIMEOUT_MS) detects that and treats the attempt as failed
   *  so the redial loop can move on. */
  function bindJoinerPeer(code: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const myId = idPrefix + "j-" + Math.random().toString(36).slice(2, 10);
      const p = new Peer(myId);
      let connOpened = false;
      let settled = false;
      // Watchdog handle for the PeerJS silent-hang case: `p.connect()` returned
      // a DataConnection object, `bindConnection` registered handlers, but
      // none of `open`/`error`/`close` ever fire. Without this timeout the
      // redial loop dead-ends on that one hung attempt. Cleared when conn opens
      // OR when peer-level error kills the attempt outright.
      let attemptTimer: ReturnType<typeof setTimeout> | null = null;
      const clearAttemptTimer = (): void => {
        if (attemptTimer !== null) {
          clearTimeout(attemptTimer);
          attemptTimer = null;
        }
      };
      const failAttempt = (e: Error | unknown, reason: string): void => {
        if (settled) return;
        settled = true;
        clearAttemptTimer();
        const msg = (e as Error)?.message ?? String(e);
        log(reason, { error: msg });
        if (!suppressingRedialErrors) {
          cbs.onLastError(msg);
        }
        cbs.onStatusChange("disconnected");
        try { p.destroy(); } catch { /* noop */ }
        if (inRedialAttempt) {
          maybeAutoRedialJoiner();
        }
        reject(e instanceof Error ? e : new Error(msg));
      };
      p.on("open", () => {
        peer = p;
        cbs.onStatusChange("connecting");
        const c = p.connect(idPrefix + code, { reliable: true });
        bindConnection(c);
        // Start the silent-hang watchdog now that we've handed off to PeerJS.
        attemptTimer = setTimeout(() => {
          attemptTimer = null;
          if (connOpened || settled) return;
          failAttempt(new Error("redial attempt timed out"), "joiner attempt timed out (no open/error)");
        }, REDIAL_ATTEMPT_TIMEOUT_MS);
        c.on("open", () => {
          connOpened = true;
          settled = true;
          clearAttemptTimer();
          resolve();
        });
        c.on("error", (e) => {
          if (settled) return;
          settled = true;
          clearAttemptTimer();
          reject(e);
        });
      });
      p.on("error", (e) => {
        const msg = e?.message ?? String(e);
        if (connOpened) {
          // Broker-side hiccup after the conn was already live — log it but
          // don't touch status or kill anything. The conn's own close/error
          // handlers will fire if it actually drops.
          log("peer.error after conn opened (ignored)", { error: msg });
          return;
        }
        // Pre-open peer-level error (peer-unavailable, id-taken, network
        // before the conn opened). Initial-join failures must NOT spin retries
        // — `inRedialAttempt` gates that inside failAttempt.
        failAttempt(e, "joiner peer.error pre-open");
      });
    });
  }

  /** Internal soft teardown — does NOT reset the redial budget or clear the
   *  suppression flag (the wrapper's destroyPeerKeepState calls this, and the
   *  auto-redial loop expects state to persist across the swap). */
  function destroyPeerKeepStateInternal(): void {
    if (conn) {
      try { conn.close(); } catch { /* noop */ }
      conn = null;
    }
    if (peer) {
      try { peer.destroy(); } catch { /* noop */ }
      peer = null;
    }
  }

  // === Public API =========================================================

  function host(): Promise<string> {
    log("host begin");
    disconnect();
    cbs.onStatusChange("hosting");
    return new Promise((resolve, reject) => {
      const tryOne = (attemptsLeft: number) => {
        const code = generateCodeLocal();
        const p = new Peer(idPrefix + code);
        let peerOpened = false;
        p.on("open", () => {
          peerOpened = true;
          peer = p;
          cbs.onCode(code);
          p.on("connection", (c) => {
            log("host: incoming connection", { fromPeer: c.peer, hadActiveConn: conn !== null && conn.open });
            if (conn && conn.open) {
              // Reject any third peer.
              c.on("open", () => {
                try {
                  c.send(JSON.stringify({ kind: "error", reason: "session-full" }));
                } finally {
                  try { c.close(); } catch { /* noop */ }
                }
              });
              return;
            }
            cbs.onStatusChange("connecting");
            bindConnection(c);
          });
          resolve(code);
        });
        p.on("error", (e) => {
          const msg = e?.message ?? String(e);
          if (peerOpened) {
            // Broker hiccup after registration succeeded — don't kill the
            // peer or flip status. The conn's own handlers will report a
            // real drop. PeerJS sometimes emits transient `disconnected` /
            // `network` errors even when the conn is fine.
            log("host: peer.error after open (ignored)", { error: msg });
            return;
          }
          if (attemptsLeft > 0 && /taken|unavailable-id/i.test(msg)) {
            try { p.destroy(); } catch { /* noop */ }
            tryOne(attemptsLeft - 1);
            return;
          }
          cbs.onLastError(msg);
          cbs.onStatusChange("error");
          try { p.destroy(); } catch { /* noop */ }
          reject(e);
        });
      };
      tryOne(5);
    });
  }

  function hostWithCode(code: string): Promise<string> {
    log("hostWithCode begin", { code });
    disconnect();
    cbs.onStatusChange("hosting");
    return new Promise((resolve, reject) => {
      const tryOne = (attemptIdx: number): void => {
        const p = new Peer(idPrefix + code);
        let peerOpened = false;
        p.on("open", () => {
          peerOpened = true;
          peer = p;
          cbs.onCode(code);
          p.on("connection", (c) => {
            log("hostWithCode: incoming connection", { fromPeer: c.peer, hadActiveConn: conn !== null && conn.open });
            if (conn && conn.open) {
              c.on("open", () => {
                try {
                  c.send(JSON.stringify({ kind: "error", reason: "session-full" }));
                } finally {
                  try { c.close(); } catch { /* noop */ }
                }
              });
              return;
            }
            cbs.onStatusChange("connecting");
            bindConnection(c);
          });
          resolve(code);
        });
        p.on("error", (e) => {
          const msg = e?.message ?? String(e);
          if (peerOpened) {
            // Broker hiccup after registration succeeded — see host() for rationale.
            log("hostWithCode: peer.error after open (ignored)", { error: msg });
            return;
          }
          if (attemptIdx < hostRetryDelays.length && TRANSIENT_HOST_ERROR.test(msg)) {
            try { p.destroy(); } catch { /* noop */ }
            setTimeout(() => tryOne(attemptIdx + 1), hostRetryDelays[attemptIdx]);
            return;
          }
          cbs.onLastError(msg);
          cbs.onStatusChange("error");
          try { p.destroy(); } catch { /* noop */ }
          reject(e);
        });
      };
      tryOne(0);
    });
  }

  function join(code: string): Promise<void> {
    log("join begin", { code });
    disconnect();
    cbs.onStatusChange("joining");
    cbs.onCode(code);
    return bindJoinerPeer(code);
  }

  function disconnect(): void {
    log("disconnect", { hadPeer: peer !== null, hadConn: conn !== null });
    if (conn) {
      try { conn.close(); } catch { /* noop */ }
      conn = null;
    }
    if (peer) {
      try { peer.destroy(); } catch { /* noop */ }
      peer = null;
    }
    redialAttempts = 0;
    redialPending = false;
    inRedialAttempt = false;
    suppressingRedialErrors = false;
    emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
  }

  function destroyPeerKeepState(): void {
    destroyPeerKeepStateInternal();
  }

  function sendRaw(raw: string): void {
    if (!conn || !conn.open) return;
    try {
      conn.send(raw);
    } catch (e) {
      cbs.onLastError((e as Error)?.message ?? String(e));
    }
  }

  function isActive(): boolean {
    return peer !== null && cbs.getRole() !== null;
  }

  /** Liveness probe: open a throwaway Peer + DataConnection to `code` and
   *  resolve `true` if the channel opens AND the host doesn't kick us with a
   *  `session-full` error within a 500ms confirmation window. Else `false`.
   *  Used by the lobby to show 🟢/⚫ dots on recent-sessions cards.
   *
   *  Why the confirmation window: a host that's already paired with another
   *  joiner will accept the open() and THEN send `session-full` before
   *  closing. Resolving `true` on open alone gives false positives for hosts
   *  that wouldn't accept this user. 500ms covers typical broker latency.
   *
   *  Does NOT touch the singleton `peer`/`conn`. Always tears down its own
   *  Peer in the finally path so probe traffic never lingers on the broker. */
  function probeHost(code: string, timeoutMs = 2_000): Promise<boolean> {
    return new Promise((resolve) => {
      const myId = idPrefix + "probe-" + Math.random().toString(36).slice(2, 10);
      let p: Peer | null = null;
      let c: DataConnection | null = null;
      let settled = false;
      let confirmTimer: ReturnType<typeof setTimeout> | null = null;
      const finish = (result: boolean): void => {
        if (settled) return;
        settled = true;
        log("probeHost result", { code, result });
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
          c = p.connect(idPrefix + code, { reliable: true });
          c.on("open", () => {
            clearTimeout(timer);
            // Watch for a session-full kick from the paired host before
            // declaring victory.
            c?.on("data", (raw: unknown) => {
              if (typeof raw !== "string") return;
              try {
                const parsed = JSON.parse(raw) as { kind?: string; reason?: string };
                if (parsed?.kind === "error" && parsed?.reason === "session-full") {
                  finish(false);
                }
              } catch { /* not JSON — ignore */ }
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

  return {
    host,
    hostWithCode,
    join,
    disconnect,
    destroyPeerKeepState,
    sendRaw,
    probeHost,
    isActive,
  };
}

// Local copy of the 6-digit-code generator so the transport doesn't depend on
// the V1 protocol module. This is the host-code minting only — V1 wire encoders
// live elsewhere.
function generateCodeLocal(): string {
  return String(Math.floor(100000 + Math.random() * 900000));
}
