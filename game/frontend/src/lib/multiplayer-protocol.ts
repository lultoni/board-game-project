// Pure helpers for the L7a multiplayer wire protocol. No PeerJS, no DOM, no
// runes — kept vanilla TS so vitest can exercise them directly.
//
// The wire is a JSON-encoded discriminated union over a single PeerJS
// DataConnection. See multiplayer.svelte.ts for the side-effecting layer.

export type ResumeRejectReason =
  | "zobrist-mismatch"
  | "no-such-session"
  | "host-not-in-match";

/** L8 Phase F — pre-game draft mode broadcast by the host so the joiner can
 *  route to /draft/ (custom) or /match/ (preMade) without its own setup pass.
 *  `loadoutId` is required when `mode === "preMade"` and ignored otherwise. */
export type DraftModeWire = "custom" | "preMade";
export type PreMadeLoadoutIdWire = "firstGame" | "secondGame" | "thirdGame";

export type WireMessage =
  | { kind: "ping"; t: number }
  | { kind: "pong"; t: number }
  | { kind: "snapshot"; snapshotJson: string }
  | { kind: "ready" }
  | { kind: "action"; raw: number }
  | { kind: "error"; reason: string }
  // L8 Phase F — multiplayer draft.
  // - draft-mode: host → joiner at /setup/ commit; selects custom-vs-preMade.
  // - draft-ready: peer → peer once /draft/ has booted its local engine.
  //                Both sides exchange this before the first DraftTurn so
  //                neither acts on a half-mounted opponent.
  // - draft-turn: peer → peer on every commit. `raw` is the packed u32
  //                produced by encodeDraftTurn — both engines apply it via
  //                tryApply and the determinism guarantee keeps state aligned.
  | { kind: "draft-mode"; mode: DraftModeWire; loadoutId?: PreMadeLoadoutIdWire }
  | { kind: "draft-ready" }
  | { kind: "draft-turn"; raw: number }
  // Resume handshake. `zobrist` is a decimal string because the engine's
  // PositionView.zobrist is a bigint that JSON cannot natively encode.
  // plyCount: 0 + zobrist: "0" means "I have no engine yet — send me a fresh
  // snapshot if your code matches".
  | { kind: "resume-request"; code: string; plyCount: number; zobrist: string }
  | { kind: "resume-accept"; snapshotJson: string }
  | { kind: "resume-reject"; reason: ResumeRejectReason };

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
    case "action": {
      const raw = (m as { raw?: unknown }).raw;
      // Engine actions are u32. Reject anything outside [0, 2^32 - 1] or
      // non-integer values — otherwise a malformed peer message would call
      // `tryApply` with garbage and trip an engine panic.
      return (
        typeof raw === "number"
        && Number.isInteger(raw)
        && raw >= 0
        && raw <= 0xffffffff
      )
        ? (m as WireMessage)
        : null;
    }
    case "error":
      return typeof (m as { reason?: unknown }).reason === "string"
        ? (m as WireMessage)
        : null;
    case "draft-mode": {
      const r = m as { mode?: unknown; loadoutId?: unknown };
      if (r.mode === "custom") {
        return { kind: "draft-mode", mode: "custom" };
      }
      if (r.mode === "preMade") {
        const id = r.loadoutId;
        if (id === "firstGame" || id === "secondGame" || id === "thirdGame") {
          return { kind: "draft-mode", mode: "preMade", loadoutId: id };
        }
        return null;
      }
      return null;
    }
    case "draft-ready":
      return { kind: "draft-ready" };
    case "draft-turn": {
      const raw = (m as { raw?: unknown }).raw;
      // Same u32 guard as `action` — reject negatives, non-integers, and
      // anything beyond 2^32 - 1. Engine tryApply panics on out-of-range raw.
      return (
        typeof raw === "number"
        && Number.isInteger(raw)
        && raw >= 0
        && raw <= 0xffffffff
      )
        ? (m as WireMessage)
        : null;
    }
    case "resume-request": {
      const r = m as {
        code?: unknown;
        plyCount?: unknown;
        zobrist?: unknown;
      };
      if (
        typeof r.code === "string"
        && Number.isInteger(r.plyCount)
        && (r.plyCount as number) >= 0
        && typeof r.zobrist === "string"
      ) {
        return m as WireMessage;
      }
      return null;
    }
    case "resume-accept":
      return typeof (m as { snapshotJson?: unknown }).snapshotJson === "string"
        ? (m as WireMessage)
        : null;
    case "resume-reject": {
      const r = (m as { reason?: unknown }).reason;
      if (
        r === "zobrist-mismatch"
        || r === "no-such-session"
        || r === "host-not-in-match"
      ) {
        return m as WireMessage;
      }
      return null;
    }
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

/** How long the player has to wait before they can claim a win by opponent
 *  forfeit after the pill goes 🔴. Mirrors PILL_FORFEIT_MS so the grace-
 *  banner countdown and the pill's forfeit transition land together. */
export const GRACE_MS = 5 * 60_000;

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
