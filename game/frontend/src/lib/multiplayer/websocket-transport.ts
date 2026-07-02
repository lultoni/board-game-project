// WebSocket relay transport — drop-in replacement for the PeerJS transport.
//
// Implements the same `Transport` interface. The relay server owns session
// routing; this module owns the WebSocket lifecycle, relay-envelope handling,
// and the joiner-side auto-redial ladder (identical policy to the PeerJS
// version).
//
// Relay control frames use `type`; game messages use `kind` — zero collision.
// Control frames are consumed here and never reach the wrapper's onData.

import { RELAY_WS_URL, RELAY_HTTP_URL } from "./transport-config";

export type TransportStatus =
  | "hosting"
  | "joining"
  | "connecting"
  | "connected"
  | "disconnected"
  | "error";

export type TransportRole = "host" | "joiner" | null;

export interface RedialState {
  mode: "idle" | "ladder" | "longtail";
  attempt: number;
  nextAttemptAt: number | null;
}

export interface TransportOpts {
  idPrefix?: string;
  redialDelays?: number[];
  hostRetryDelays?: number[];
  log?: (event: string, detail?: unknown) => void;
}

export interface TransportCallbacks {
  onOpen(): void;
  onData(raw: string): void;
  onClose(): void;
  onError(message: string): void;
  onStatusChange(s: TransportStatus): void;
  onCode(code: string): void;
  onLastError(message: string): void;
  getRole(): TransportRole;
  getCode(): string | null;
  onRedialState?(state: RedialState): void;
  /** Called when the relay promotes a joining peer to host (because the host
   *  slot was empty). The wrapper should flip mpState.role to "host". */
  onPromotedToHost?(): void;
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

/** Joiner-side auto-redial backoff ladder (ms). First slot is short so a
 *  quickly-returning host pairs up fast; long-tail continues indefinitely. */
const DEFAULT_REDIAL_DELAYS = [400, 1_500, 3_000, 6_000, 12_000, 30_000];
const LONG_TAIL_DELAY_MS = 30_000;

/** hostWithCode retry schedule for the "prefer code" reclaim path. */
const DEFAULT_HOST_RETRY_DELAYS = [800, 1_500, 2_500, 4_000];

export function createWebSocketTransport(
  cbs: TransportCallbacks,
  opts: TransportOpts = {},
): Transport {
  const relayWs = RELAY_WS_URL;
  const relayHttp = RELAY_HTTP_URL;
  const redialDelays = opts.redialDelays ?? DEFAULT_REDIAL_DELAYS;
  const log = opts.log ?? ((event, detail) => {
    // eslint-disable-next-line no-console
    console.log(`[mp] ${event}`, detail);
  });

  // === Session-singleton state ================================================

  let ws: WebSocket | null = null;
  // True once the session is fully paired (both peers present).
  // Reset on disconnect().
  let paired = false;

  let redialAttempts = 0;
  let redialPending = false;
  let suppressingRedialErrors = false;
  let inRedialAttempt = false;

  function emitRedialState(next: RedialState): void {
    cbs.onRedialState?.(next);
  }

  // === Relay envelope helpers =================================================

  type RelayInbound =
    | { type: "created"; code: string }
    | { type: "joined" }
    | { type: "peer-connected" }
    | { type: "peer-disconnected" }
    | { type: "error"; reason: string };

  function parseRelay(raw: string): RelayInbound | null {
    let obj: unknown;
    try { obj = JSON.parse(raw); } catch { return null; }
    if (typeof obj !== "object" || obj === null) return null;
    const m = obj as Record<string, unknown>;
    switch (m.type) {
      case "created":
        return typeof m.code === "string" ? { type: "created", code: m.code } : null;
      case "joined":
        return { type: "joined" };
      case "peer-connected":
        return { type: "peer-connected" };
      case "peer-disconnected":
        return { type: "peer-disconnected" };
      case "error":
        return typeof m.reason === "string" ? { type: "error", reason: m.reason } : null;
      default:
        return null;
    }
  }

  function sendEnvelope(socket: WebSocket, msg: Record<string, unknown>): void {
    if (socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(msg));
    }
  }

  // === Auto-redial (symmetric across roles) ==================================
  //
  // The relay is role-agnostic on rejoin: sending {type:"join", code} promotes
  // the sender to host if the session is dead or the host slot is empty, and
  // attaches them as joiner otherwise. So we always rebind via bindJoiner and
  // let the relay + onPromotedToHost callback sort out the role.

  function maybeAutoRedial(): void {
    if (cbs.getRole() === null) return;
    if (cbs.getCode() === null) return;
    if (redialPending) return;
    const inLadder = redialAttempts < redialDelays.length;
    const delay = inLadder ? redialDelays[redialAttempts] : LONG_TAIL_DELAY_MS;
    const code = cbs.getCode();
    if (code === null) return;
    redialAttempts += 1;
    redialPending = true;
    suppressingRedialErrors = true;
    emitRedialState({
      mode: inLadder ? "ladder" : "longtail",
      attempt: redialAttempts,
      nextAttemptAt: Date.now() + delay,
    });
    setTimeout(() => {
      redialPending = false;
      if (cbs.getRole() === null) return;
      if (cbs.getCode() === null) return;
      softReconnect(code).catch(() => { /* handled via close/error */ });
    }, delay);
  }

  function softReconnect(code: string): Promise<void> {
    destroyWsKeepState();
    cbs.onStatusChange("joining");
    inRedialAttempt = true;
    return bindJoiner(code).finally(() => {
      inRedialAttempt = false;
    });
  }

  // === Core WebSocket builders ================================================

  /** Open a new WebSocket to the relay and resolve/reject via callbacks.
   *
   *  `onOpen`  — fires after WS open AND after the relay confirms the session
   *              (either "created" for host, or "joined" for joiner).
   *  `onMsg`   — called for every relay control message received on this socket.
   *              Returns `true` if the message was consumed (relay control);
   *              `false` means it's a game message and should be forwarded.
   *  `onClose` / `onError` — forwarded to reject + cleanup.
   */
  function openWs(
    onRelayOpen: (socket: WebSocket) => void,
    onRelayMsg: (socket: WebSocket, env: RelayInbound) => boolean,
    onFail: (reason: string) => void,
  ): WebSocket {
    const socket = new WebSocket(relayWs);
    ws = socket;

    socket.onopen = () => {
      log("ws.open");
      onRelayOpen(socket);
    };

    socket.onmessage = (ev) => {
      const raw = typeof ev.data === "string" ? ev.data : null;
      if (!raw) return;
      log("ws.message", { raw });
      const relay = parseRelay(raw);
      if (relay) {
        log("relay.envelope", { type: relay.type });
        const consumed = onRelayMsg(socket, relay);
        if (consumed) return;
      }
      // Game message — forward to wrapper.
      cbs.onData(raw);
    };

    socket.onclose = () => {
      log("ws.close");
      if (ws !== socket) return; // stale socket from a previous attempt
      ws = null;
      paired = false;
      cbs.onStatusChange("disconnected");
      cbs.onClose();
      maybeAutoRedial();
    };

    socket.onerror = () => {
      log("ws.error");
      if (ws !== socket) return;
      const msg = "WebSocket connection error";
      if (!suppressingRedialErrors) cbs.onLastError(msg);
      cbs.onStatusChange("disconnected");
      cbs.onError(msg);
      ws = null;
      paired = false;
      maybeAutoRedial();
    };

    return socket;
  }

  /** Host handshake. Resolves with the session code once "created" arrives.
   *  Subsequent peer-connected / peer-disconnected events are handled via
   *  the same onRelayMsg callback (openWs routes ALL relay messages through it). */
  function bindHost(preferCode?: string): Promise<string> {
    return new Promise((resolve, reject) => {
      let settled = false;
      let createdCode: string | null = null;
      const fail = (reason: string) => {
        if (settled) return;
        settled = true;
        cbs.onLastError(reason);
        cbs.onStatusChange("error");
        reject(new Error(reason));
      };

      openWs(
        (socket) => {
          const msg: Record<string, unknown> = { type: "create" };
          if (preferCode) msg.preferCode = preferCode;
          sendEnvelope(socket, msg);
        },
        (_socket, env) => {
          if (env.type === "created") {
            createdCode = env.code;
            cbs.onCode(env.code);
            if (!settled) {
              settled = true;
              resolve(env.code);
            }
            return true;
          }
          if (env.type === "peer-connected" && createdCode !== null) {
            paired = true;
            redialAttempts = 0;
            redialPending = false;
            suppressingRedialErrors = false;
            emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
            cbs.onStatusChange("connected");
            cbs.onOpen();
            return true;
          }
          if (env.type === "peer-disconnected" && createdCode !== null) {
            paired = false;
            cbs.onStatusChange("disconnected");
            cbs.onClose();
            // Host stays put — joiner will redial back.
            return true;
          }
          if (env.type === "error") {
            fail(`relay error: ${env.reason}`);
            return true;
          }
          return false;
        },
        fail,
      );
    });
  }

  /** Joiner handshake. Resolves once "joined" arrives (session exists on relay). */
  function bindJoiner(code: string): Promise<void> {
    return new Promise((resolve, reject) => {
      let settled = false;
      const fail = (reason: string, raw?: string) => {
        if (settled) return;
        settled = true;
        log("bindJoiner fail", { reason });
        if (!suppressingRedialErrors) cbs.onLastError(reason);
        cbs.onStatusChange("disconnected");
        cbs.onError(reason);
        if (inRedialAttempt) maybeAutoRedial();
        reject(new Error(raw ?? reason));
      };

      openWs(
        (socket) => {
          sendEnvelope(socket, { type: "join", code });
        },
        (_socket, env) => {
          if (env.type === "joined") {
            if (settled) return true;
            settled = true;
            paired = true;
            redialAttempts = 0;
            redialPending = false;
            suppressingRedialErrors = false;
            emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
            cbs.onStatusChange("connected");
            cbs.onOpen();
            resolve();
            return true;
          }
          // Relay promoted this peer to host (host slot was empty).
          // Switch into host mode: notify wrapper, then wait for peer-connected.
          if (env.type === "created") {
            if (settled) return true;
            settled = true;
            cbs.onCode(env.code);
            cbs.onPromotedToHost?.();
            // Now behave like bindHost: wait for peer-connected on this same socket.
            resolve();
            return true;
          }
          if (env.type === "peer-connected") {
            // Arrives after promotion — we're now paired as the host.
            paired = true;
            redialAttempts = 0;
            redialPending = false;
            suppressingRedialErrors = false;
            emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
            cbs.onStatusChange("connected");
            cbs.onOpen();
            return true;
          }
          if (env.type === "peer-disconnected") {
            // Host side: joiner left, stay connected and wait for redial.
            paired = false;
            cbs.onStatusChange("disconnected");
            cbs.onClose();
            return true;
          }
          if (env.type === "error") {
            fail(`relay error: ${env.reason}`, env.reason);
            return true;
          }
          return false;
        },
        fail,
      );
    });
  }

  // === Soft teardown (no state reset) ========================================

  function destroyWsKeepState(): void {
    if (ws) {
      // Null out first so the onclose/onerror handlers see a stale socket and
      // skip their disconnect/redial logic — we're driving that ourselves.
      const old = ws;
      ws = null;
      try { old.close(); } catch { /* noop */ }
    }
    paired = false;
  }

  // === Public API =============================================================

  function host(): Promise<string> {
    log("host begin");
    disconnect();
    cbs.onStatusChange("hosting");
    return bindHost();
  }

  function hostWithCode(code: string): Promise<string> {
    log("hostWithCode begin", { code });
    disconnect();
    cbs.onStatusChange("hosting");

    // Retry a few times — the relay may still hold the session slot briefly
    // after the old host disconnected (TTL cleanup runs every 10s).
    const delays = opts.hostRetryDelays ?? DEFAULT_HOST_RETRY_DELAYS;
    let attemptIdx = 0;

    const tryOne = (): Promise<string> =>
      bindHost(code).catch((e: Error) => {
        if (attemptIdx < delays.length && /session-gone|session-full/i.test(e.message)) {
          attemptIdx++;
          return new Promise<string>((res, rej) =>
            setTimeout(() => tryOne().then(res, rej), delays[attemptIdx - 1])
          );
        }
        throw e;
      });

    return tryOne();
  }

  function join(code: string): Promise<void> {
    log("join begin", { code });
    disconnect();
    cbs.onStatusChange("joining");
    cbs.onCode(code);
    return bindJoiner(code);
  }

  function disconnect(): void {
    log("disconnect", { hadWs: ws !== null, stack: new Error().stack?.split("\n").slice(1, 5).join(" | ") });
    if (ws) {
      const old = ws;
      ws = null;
      try { old.close(); } catch { /* noop */ }
    }
    paired = false;
    redialAttempts = 0;
    redialPending = false;
    inRedialAttempt = false;
    suppressingRedialErrors = false;
    emitRedialState({ mode: "idle", attempt: 0, nextAttemptAt: null });
  }

  function destroyPeerKeepState(): void {
    destroyWsKeepState();
  }

  function sendRaw(raw: string): void {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    try {
      ws.send(raw);
    } catch (e) {
      cbs.onLastError((e as Error)?.message ?? String(e));
    }
  }

  function isActive(): boolean {
    return ws !== null && cbs.getRole() !== null;
  }

  function probeHost(code: string, timeoutMs = 2_000): Promise<boolean> {
    const url = `${relayHttp}/probe/${encodeURIComponent(code)}`;
    return fetch(url, { signal: AbortSignal.timeout(timeoutMs) })
      .then((r) => {
        if (!r.ok) return false;
        return r.json().then((j: unknown) => {
          if (typeof j === "object" && j !== null && "live" in j) {
            return (j as { live: boolean }).live === true;
          }
          return false;
        });
      })
      .catch(() => false);
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
