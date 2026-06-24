// Pure helpers for the L7a multiplayer wire protocol. No PeerJS, no DOM, no
// runes — kept vanilla TS so vitest can exercise them directly.
//
// The wire is a JSON-encoded discriminated union over a single PeerJS
// DataConnection. See multiplayer.svelte.ts for the side-effecting layer.

export type WireMessage =
  | { kind: "ping"; t: number }
  | { kind: "pong"; t: number }
  | { kind: "snapshot"; snapshotJson: string }
  | { kind: "ready" }
  | { kind: "action"; raw: number }
  | { kind: "error"; reason: string };

/** 6-digit code in [100000, 999999]. Used as the PeerJS host ID so the
 *  joiner can dial it without a discovery service. */
export function generateCode(): string {
  return String(100000 + Math.floor(Math.random() * 900000));
}

export function isValidCode(s: string): boolean {
  return /^[1-9][0-9]{5}$/.test(s);
}

export function encodeMessage(m: WireMessage): string {
  return JSON.stringify(m);
}

export function decodeMessage(s: string): WireMessage | null {
  let obj: unknown;
  try {
    obj = JSON.parse(s);
  } catch {
    return null;
  }
  if (typeof obj !== "object" || obj === null) return null;
  const m = obj as { kind?: unknown };
  switch (m.kind) {
    case "ping":
    case "pong":
      return typeof (m as { t?: unknown }).t === "number"
        ? (m as WireMessage)
        : null;
    case "snapshot":
      return typeof (m as { snapshotJson?: unknown }).snapshotJson === "string"
        ? (m as WireMessage)
        : null;
    case "ready":
      return { kind: "ready" };
    case "action":
      return Number.isInteger((m as { raw?: unknown }).raw)
        ? (m as WireMessage)
        : null;
    case "error":
      return typeof (m as { reason?: unknown }).reason === "string"
        ? (m as WireMessage)
        : null;
    default:
      return null;
  }
}

/** Network-status enum maintained by the side-effecting layer. */
export type MpStatus =
  | "idle"
  | "hosting"
  | "joining"
  | "connecting"
  | "connected"
  | "disconnected"
  | "error";

/** Display state of the connectivity pill — derived purely from `status`
 *  and how recently we last heard back from the peer. The pill is the user
 *  signal, kept separate from the underlying PeerJS state so the UI can
 *  show "🟡 unstable" before the peer is formally disconnected.
 *
 *  Thresholds (ADR-006):
 *    - 🟢 live:         fresh pong (<2s) and status is connected
 *    - 🟡 unstable:     pong stale 2s–10s but still connected
 *    - 🔴 disconnected: pong stale ≥10s, OR status is disconnected, within the
 *                       5-minute forfeit grace window
 *    - ⚫ forfeit:      grace window expired
 */
export type PillState = "live" | "unstable" | "disconnected" | "forfeit";

const PILL_UNSTABLE_MS = 2_000;
const PILL_DISCONNECTED_MS = 10_000;
const PILL_FORFEIT_MS = 5 * 60_000;

export function derivePillState(
  status: MpStatus,
  lastPongAt: number | null,
  now: number,
): PillState {
  if (status !== "connected" && status !== "disconnected") {
    // hosting/joining/connecting/idle/error — no peer yet, so nothing to
    // display in the HUD. Callers should hide the pill in these states.
    return "disconnected";
  }
  if (lastPongAt === null) {
    // Connected but never heard a pong — show as disconnected until the
    // first heartbeat lands.
    return status === "connected" ? "unstable" : "disconnected";
  }
  const age = now - lastPongAt;
  if (status === "connected") {
    if (age < PILL_UNSTABLE_MS) return "live";
    if (age < PILL_DISCONNECTED_MS) return "unstable";
    if (age < PILL_FORFEIT_MS) return "disconnected";
    return "forfeit";
  }
  // status === "disconnected"
  if (age < PILL_FORFEIT_MS) return "disconnected";
  return "forfeit";
}
