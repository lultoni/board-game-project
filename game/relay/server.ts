// WebSocket relay server for 2-player board game sessions.
//
// Peers connect to /ws and exchange relay envelopes to set up a session,
// then send game messages which are forwarded verbatim to the other peer.
//
// Relay control frames use a `type` field; game messages (V1/V2) use `kind`,
// so there is zero collision risk and no game message is ever consumed here.
//
// Session lifecycle:
//   1. Host connects to /ws, sends {"type":"create"} (or {"type":"create","preferCode":"XXXXXX"})
//   2. Relay replies {"type":"created","code":"XXXXXX"}
//   3. Joiner connects to /ws, sends {"type":"join","code":"XXXXXX"}
//   4. Relay replies {"type":"joined"} to joiner; {"type":"peer-connected"} to host
//   5. Both peers exchange game messages freely — relay forwards verbatim
//   6. On disconnect: relay notifies remaining peer with {"type":"peer-disconnected"}
//
// HTTP endpoints:
//   GET /probe/:code  — liveness probe used by the lobby's status dots

const PORT = parseInt(process.env.PORT ?? "3001", 10);

// Session cleanup: if both peers are absent for this long, delete the session.
const SESSION_TTL_MS = 60_000;

interface Session {
  code: string;
  host: ServerWebSocket<WsData> | null;
  joiner: ServerWebSocket<WsData> | null;
  // Timestamp of the last time BOTH peers were absent. Used for TTL cleanup.
  bothAbsentSince: number | null;
  createdAt: number;
}

interface WsData {
  sessionCode: string | null;
  role: "host" | "joiner" | null;
  peerId: string;
  openedAt: number;
}

const sessions = new Map<string, Session>();

// --- Cleanup loop ------------------------------------------------------------

setInterval(() => {
  const now = Date.now();
  for (const [code, session] of sessions) {
    const hasHost = session.host !== null;
    const hasJoiner = session.joiner !== null;
    if (!hasHost && !hasJoiner) {
      if (session.bothAbsentSince === null) {
        session.bothAbsentSince = now;
      } else if (now - session.bothAbsentSince > SESSION_TTL_MS) {
        sessions.delete(code);
        console.log(`[relay] session ${code} expired`);
      }
    } else {
      session.bothAbsentSince = null;
    }
  }
}, 10_000);

// --- Code generation ---------------------------------------------------------

function generateCode(): string {
  // 6-digit code matching the frontend's isValidCode regex: /^[1-9][0-9]{5}$/
  return String(100000 + Math.floor(Math.random() * 900000));
}

function generateUniqueCode(): string {
  let code = generateCode();
  let attempts = 0;
  while (sessions.has(code) && attempts < 20) {
    code = generateCode();
    attempts++;
  }
  return code;
}

function generatePeerId(): string {
  // Short random ID for correlating log lines to a specific peer socket.
  // 4 hex chars = 16 bits — plenty for a relay whose peer count is bounded
  // by concurrent sessions.
  return Math.floor(Math.random() * 0x10000).toString(16).padStart(4, "0");
}

// --- Relay helpers -----------------------------------------------------------

type RelayMsg = Record<string, unknown>;

function send(ws: ServerWebSocket<WsData>, msg: RelayMsg): void {
  try {
    ws.send(JSON.stringify(msg));
  } catch {
    // Socket already closed — ignore.
  }
}

function forward(from: ServerWebSocket<WsData>, session: Session, raw: string): void {
  const peer = from === session.host ? session.joiner : session.host;
  if (peer) {
    try {
      peer.send(raw);
    } catch {
      // Peer socket closed — the close handler will fire shortly and notify.
    }
  }
}

// --- Message handler ---------------------------------------------------------

function handleMessage(ws: ServerWebSocket<WsData>, raw: string): void {
  const peer = ws.data.peerId;
  let msg: RelayMsg;
  try {
    const parsed = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) throw new Error("not object");
    msg = parsed as RelayMsg;
  } catch {
    // Not valid JSON or not an object — this is a game message that slipped
    // through before session setup. Drop it.
    return;
  }

  // Already in a session? Forward game messages, handle peer-level controls.
  const code = ws.data.sessionCode;
  if (code) {
    const session = sessions.get(code);
    if (!session) {
      console.log(`[relay][peer=${peer}] reject: session-gone (code=${code})`);
      send(ws, { type: "error", reason: "session-gone" });
      return;
    }
    // Any message whose `type` is NOT a relay keyword is forwarded verbatim.
    const t = msg.type;
    const isRelayMsg = t === "create" || t === "join";
    if (!isRelayMsg) {
      forward(ws, session, raw);
      return;
    }
    // Peer tried to create/join while already in a session — ignore.
    console.log(`[relay][peer=${peer}] ignored ${String(t)} while in session ${code}`);
    return;
  }

  // Not yet in a session — handle setup envelopes.
  const type = msg.type;

  if (type === "create") {
    // Optionally honour a preferred code (for hostWithCode / rejoin path).
    const prefer = typeof msg.preferCode === "string" ? msg.preferCode : null;
    console.log(`[relay][peer=${peer}] recv create${prefer ? ` preferCode=${prefer}` : ""}`);
    let code: string;
    if (prefer && /^[1-9][0-9]{5}$/.test(prefer)) {
      const existing = sessions.get(prefer);
      if (!existing || (existing.host === null && existing.joiner === null)) {
        // Slot is free — honour the request.
        code = prefer;
        if (existing) {
          // Reuse the existing empty session object.
          existing.host = ws;
          existing.bothAbsentSince = null;
          ws.data.sessionCode = code;
          ws.data.role = "host";
          send(ws, { type: "created", code });
          console.log(`[relay] session ${code} sent created → peer=${peer} (host, reclaimed empty)`);
          return;
        }
      } else {
        // Preferred code is taken — fall through to generate a new one.
        console.log(`[relay][peer=${peer}] preferCode=${prefer} taken — allocating new`);
        code = generateUniqueCode();
      }
    } else {
      code = generateUniqueCode();
    }
    sessions.set(code, {
      code,
      host: ws,
      joiner: null,
      bothAbsentSince: null,
      createdAt: Date.now(),
    });
    ws.data.sessionCode = code;
    ws.data.role = "host";
    send(ws, { type: "created", code });
    console.log(`[relay] session ${code} sent created → peer=${peer} (host, fresh)`);
    return;
  }

  if (type === "join") {
    const joinCode = typeof msg.code === "string" ? msg.code : null;
    console.log(`[relay][peer=${peer}] recv join code=${joinCode}`);
    if (!joinCode || !/^[1-9][0-9]{5}$/.test(joinCode)) {
      console.log(`[relay][peer=${peer}] reject: invalid-code`);
      send(ws, { type: "error", reason: "invalid-code" });
      return;
    }
    const session = sessions.get(joinCode);

    // No session exists for this code — the original host left and the session
    // expired (or was never created). Treat the first joiner as the new host
    // so both sides can reconnect by just pressing "Join" with the old code.
    if (!session) {
      sessions.set(joinCode, {
        code: joinCode,
        host: ws,
        joiner: null,
        bothAbsentSince: null,
        createdAt: Date.now(),
      });
      ws.data.sessionCode = joinCode;
      ws.data.role = "host";
      // Reply with "created" so the transport's bindHost resolver fires and
      // the peer is put in the correct host state.
      send(ws, { type: "created", code: joinCode });
      console.log(`[relay] session ${joinCode} sent created → peer=${peer} (host, recreated-via-join)`);
      return;
    }

    // Session exists — check if the host slot is free or stale.
    const hostAlive = session.host !== null && (session.host as ServerWebSocket<WsData>).readyState === 1;
    if (!hostAlive) {
      // Host slot is empty or stale — this peer becomes the host.
      const displacedPeer = session.host?.data.peerId ?? null;
      if (session.host) { try { session.host.close(); } catch { /* noop */ } }
      session.host = ws;
      session.bothAbsentSince = null;
      ws.data.sessionCode = joinCode;
      ws.data.role = "host";
      send(ws, { type: "created", code: joinCode });
      console.log(`[relay] session ${joinCode} sent created → peer=${peer} (host, took stale slot${displacedPeer ? ` from peer=${displacedPeer}` : ""})`);
      if (session.joiner && (session.joiner as ServerWebSocket<WsData>).readyState === 1) {
        const joinerPeer = session.joiner.data.peerId;
        send(session.joiner, { type: "peer-connected" });
        send(ws, { type: "peer-connected" });
        console.log(`[relay] session ${joinCode} sent peer-connected → peer=${joinerPeer} (joiner)`);
        console.log(`[relay] session ${joinCode} sent peer-connected → peer=${peer} (new host)`);
      }
      return;
    }

    // Host slot is live — this peer is the joiner.
    const joinerAlive = session.joiner !== null && (session.joiner as ServerWebSocket<WsData>).readyState === 1;
    if (joinerAlive) {
      console.log(`[relay][peer=${peer}] reject: session-full (code=${joinCode}, existing joiner=${session.joiner!.data.peerId})`);
      send(ws, { type: "error", reason: "session-full" });
      return;
    }
    // Accept (or replace a stale joiner slot).
    const displacedPeer = session.joiner?.data.peerId ?? null;
    if (session.joiner) { try { session.joiner.close(); } catch { /* noop */ } }
    session.joiner = ws;
    session.bothAbsentSince = null;
    ws.data.sessionCode = joinCode;
    ws.data.role = "joiner";
    send(ws, { type: "joined" });
    send(session.host!, { type: "peer-connected" });
    const hostPeer = session.host!.data.peerId;
    console.log(`[relay] session ${joinCode} sent joined → peer=${peer} (joiner${displacedPeer ? `, replaced stale peer=${displacedPeer}` : ""})`);
    console.log(`[relay] session ${joinCode} sent peer-connected → peer=${hostPeer} (host)`);
    return;
  }

  // Unknown envelope — drop silently.
  console.log(`[relay][peer=${peer}] recv unknown kind=${String(type)}`);
}

// --- Disconnect handler ------------------------------------------------------

function handleClose(ws: ServerWebSocket<WsData>): void {
  const peer = ws.data.peerId;
  const code = ws.data.sessionCode;
  const elapsedMs = Date.now() - ws.data.openedAt;
  if (!code) {
    console.log(`[relay][peer=${peer}] disconnected (no session, elapsed=${elapsedMs}ms)`);
    return;
  }
  const session = sessions.get(code);
  if (!session) return;

  const role = ws.data.role;
  if (role === "host") {
    // Only act if this socket is still the current host — a rejoining peer
    // may have already replaced the stale slot (and closed the old socket),
    // which would fire this handler for the old socket. Sending
    // peer-disconnected for a socket that's no longer in the session would
    // be a spurious drop notification to the joiner.
    if (session.host !== ws) {
      console.log(`[relay][peer=${peer}] disconnected (stale host socket for ${code}, elapsed=${elapsedMs}ms)`);
      return;
    }
    session.host = null;
    if (session.joiner) {
      send(session.joiner, { type: "peer-disconnected" });
    }
    console.log(`[relay] session ${code} host disconnected → peer=${peer} (elapsed=${elapsedMs}ms)`);
  } else if (role === "joiner") {
    if (session.joiner !== ws) {
      console.log(`[relay][peer=${peer}] disconnected (stale joiner socket for ${code}, elapsed=${elapsedMs}ms)`);
      return;
    }
    session.joiner = null;
    if (session.host) {
      send(session.host, { type: "peer-disconnected" });
    }
    console.log(`[relay] session ${code} joiner disconnected → peer=${peer} (elapsed=${elapsedMs}ms)`);
  }
}

// --- HTTP probe handler ------------------------------------------------------

function handleProbe(code: string): Response {
  const session = sessions.get(code);
  if (!session) {
    return new Response(JSON.stringify({ live: false }), {
      status: 404,
      headers: { "Content-Type": "application/json" },
    });
  }
  const paired = session.host !== null && session.joiner !== null;
  return new Response(JSON.stringify({ live: true, paired }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

// --- Bun server --------------------------------------------------------------

const server = Bun.serve<WsData>({
  port: PORT,
  fetch(req, server) {
    const url = new URL(req.url);

    // CORS preflight (for fetch-based probe from browser).
    if (req.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: corsHeaders(),
      });
    }

    // Liveness probe endpoint.
    if (req.method === "GET" && url.pathname.startsWith("/probe/")) {
      const code = url.pathname.slice("/probe/".length);
      const res = handleProbe(code);
      // Attach CORS headers so the browser's fetch() can read the response.
      const headers = new Headers(res.headers);
      for (const [k, v] of Object.entries(corsHeaders())) {
        headers.set(k, v);
      }
      return new Response(res.body, { status: res.status, headers });
    }

    // WebSocket upgrade.
    if (url.pathname === "/ws") {
      const upgraded = server.upgrade(req, {
        data: {
          sessionCode: null,
          role: null,
          peerId: generatePeerId(),
          openedAt: Date.now(),
        } satisfies WsData,
      });
      if (upgraded) return undefined;
      return new Response("WebSocket upgrade failed", { status: 400 });
    }

    return new Response("Not found", { status: 404 });
  },
  websocket: {
    open(ws) {
      console.log(`[relay][peer=${ws.data.peerId}] ws.open`);
    },
    message(ws, message) {
      if (typeof message !== "string") return;
      handleMessage(ws, message);
    },
    close(ws) {
      handleClose(ws);
    },
  },
});

function corsHeaders(): Record<string, string> {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
  };
}

console.log(`[relay] listening on port ${server.port}`);
