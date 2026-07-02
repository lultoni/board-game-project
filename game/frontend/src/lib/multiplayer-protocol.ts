// Legacy multiplayer transport types. Post-L7c, the wire is V2
// (`multiplayer-protocol-v2.ts`); this module retains only the heartbeat and
// the broker-level error frame so the transport in `multiplayer.svelte.ts`
// can keep its ping/pong loop and `session-full` kick path unchanged.
//
// `derivePillState`, `generateCode`, `isValidCode`, `GRACE_MS` live here too
// because they're orthogonal to the wire shape and the lobby/HUD import them
// from this module.

export type WireMessage =
  | { kind: "ping"; t: number }
  | { kind: "pong"; t: number }
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

const PILL_UNSTABLE_MS = 6_000;
// 30s (was 15s). JS timer throttling on backgrounded tabs and Tauri webview
// suspension can stall the tick + queue pongs for 15–25s at a time; a shorter
// window flags healthy connections as dead. See PROTOCOL_TRACE.md Part 1.
export const PILL_DISCONNECTED_MS = 30_000;
const PILL_FORFEIT_MS = 5 * 60_000;

/** How long the player has to wait before they can claim a win by opponent
 *  forfeit after the pill goes 🔴. Mirrors PILL_FORFEIT_MS so the grace-
 *  banner countdown and the pill's forfeit transition land together. */
export const GRACE_MS = 5 * 60_000;

/** How long the joiner waits after the host vanishes before the "Take over
 *  as host" CTA becomes clickable. Smaller than GRACE_MS so the joiner has a
 *  productive alternative to a 5-minute claim-win wait. Anchored on the same
 *  `disconnectedSince` timestamp as the forfeit countdown. */
export const TAKEOVER_MS = 30_000;

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
