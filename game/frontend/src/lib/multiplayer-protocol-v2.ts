// Authoritative-host wire protocol (v2). See `.claude/plans/twinkling-questing-quiche.md`.
//
// One peer is AUTH (host); the other is MIRROR (joiner). Host owns the only
// engine that originates state. Joiner runs a mirror engine for anti-cheat
// audit. Every committed action is sequenced; out-of-order arrivals trigger
// a snapshot request rather than partial application.
//
// Pure types + codec. No PeerJS, no DOM, no runes. Sister module
// `multiplayer-protocol.ts` still holds the legacy union while routes are
// migrated; this file is the eventual replacement. Both files re-export
// `derivePillState`, `generateCode`, `isValidCode`, `GRACE_MS` — those are
// orthogonal to the wire layer and stay in the v1 file as the canonical
// home.

export type WirePhase = "draft" | "play";

export type IntentRejectReason =
  | "illegal"          // engine try_apply returned Err
  | "out-of-turn"      // not joiner's seat
  | "phase-mismatch"   // intent.phase !== host's current phase
  | "paused";          // host is paused (waiting for reconnect)

export type SnapshotRequestReason =
  | "audit-mismatch"   // mirror's Zobrist disagreed with host's
  | "reconnect"        // joiner just reopened the channel
  | "stale";           // mirror's seq is behind host's

export type WireMessageV2 =
  // Heartbeat. Unchanged shape from v1 for wire-level continuity.
  | { kind: "ping"; t: number }
  | { kind: "pong"; t: number }

  // Host → joiner immediately after the DataConnection opens. Tells the
  // joiner who's authoritative, which IDB row id the host has adopted, what
  // phase play is in, and the current committed seq. If `seq > 0` the joiner
  // follows up with `request-snapshot{reason:"reconnect"}` to fetch state.
  | { kind: "session-hello"; matchId: string; phase: WirePhase; seq: number; code: string }

  // Joiner → host. Joiner wants to apply action `raw`. Host validates with
  // try_apply on the AUTH engine. `nonce` is opaque to host; joiner uses it
  // to correlate the eventual `committed` (matching `originNonce`) or
  // `intent-rejected` reply, and to ignore late duplicates after a drop.
  | { kind: "intent"; phase: WirePhase; nonce: string; raw: number }

  // Host → joiner. Authoritative commit. Emitted after host's AUTH engine
  // accepts an action — whether host-originated or accepted from a joiner
  // `intent`. Joiner re-runs `raw` on its mirror; if the mirror rejects, the
  // joiner emits `cheat-detected`. If it accepts but post-state Zobrist
  // differs from `postZobrist`, the joiner emits `request-snapshot`.
  //   seq:         monotonic u32, starts at 1 for the first committed action.
  //                Spans both draft and play phases — never resets.
  //   postZobrist: decimal string of the engine's u64 zobrist after applying
  //                this action (engines use bigint internally; JSON can't
  //                round-trip those without loss).
  //   originNonce: echoes the joiner's `intent.nonce` when this commit was
  //                accepting a joiner intent; null for host-originated moves.
  | {
      kind: "committed";
      seq: number;
      phase: WirePhase;
      raw: number;
      postZobrist: string;
      originNonce: string | null;
    }

  // Host → joiner. Joiner's intent was refused. Reason tells the joiner
  // whether to retry (paused → wait for resumed), surface to the user
  // (illegal → "move not allowed"), or quietly drop (out-of-turn,
  // phase-mismatch — usually means joiner's UI lagged).
  | { kind: "intent-rejected"; nonce: string; reason: IntentRejectReason }

  // Host → joiner. Phase transition driven by host (draft → play). Includes a
  // full snapshot so the mirror can re-anchor without replaying the draft
  // sequence. `seq` is the seq of the LAST committed draft action.
  | { kind: "phase-change"; from: "draft"; to: "play"; snapshotJson: string; seq: number }

  // Host → joiner. Full state push. Issued on first connect (joiner has no
  // engine yet), in reply to `request-snapshot`, and after a reconnect when
  // the host's seq is ahead of the joiner's. Idempotent — receiving a
  // snapshot for a seq the joiner already has is a no-op.
  | { kind: "snapshot"; snapshotJson: string; seq: number; phase: WirePhase; matchId: string }

  // Joiner → host. Joiner needs a fresh snapshot. `mySeq` lets the host
  // decide whether the joiner is merely stale (push delta — not implemented
  // here; we always send a full snapshot) or wildly diverged.
  | { kind: "request-snapshot"; mySeq: number; reason: SnapshotRequestReason }

  // Joiner → host. Anti-cheat fault. Joiner's mirror engine rejected an
  // action that host claimed was legal. Both peers transition to forfeit
  // and surface the fault to the user. `seq` + `raw` identify which commit
  // triggered the detection so post-game inspection is possible.
  | { kind: "cheat-detected"; seq: number; raw: number }

  // (Old joiner, now host) → old host. Announces that this peer has
  // promoted itself to AUTH and adopted a new IDB row. The receiving peer
  // is expected to follow up by treating the announcer as host: it switches
  // role to joiner, replaces its engine with the announcer's snapshot, and
  // sends `request-snapshot{reason:"reconnect"}`. Carries the new matchId
  // so the old host can record it for later library-sync.
  | { kind: "handoff-announce"; matchId: string; seq: number }

  // Host → joiner. Host has refused to act locally because the joiner is
  // disconnected (anti-tamper guard from the user's spec: "during no
  // connection to the client the host is not allowed to make any actions").
  // Mirror displays a "paused — waiting for opponent" indicator. Cleared by
  // `resumed`.
  | { kind: "paused" }
  | { kind: "resumed" }

  // Reserved for the broker's third-peer kick path (kept compatible with v1
  // so the host can still send `session-full` to a fourth tab dialling in).
  | { kind: "error"; reason: string }

  // Attacker → defender (relayed through the data channel; symmetric — the
  // recipient is just "the other peer"). Stack M says the DEFENDER chooses
  // which of the dual-adjacent Guards intercepts a Move-Attack. The attacker
  // computes whether bodyguard variants exist locally, then defers to the
  // defender by sending this prompt. Defender opens its own bodyguard chooser
  // (recomputed locally from `src` + `target` + `approach` + the mirrored
  // position), picks a variant, and submits the chosen raw via the normal
  // `intent` path. The attacker freezes input until a `committed` for that
  // move lands.
  | { kind: "bodyguard-prompt"; src: number; target: number; approach: number };

/** JSON-encode a wire message. */
export function encodeMessageV2(m: WireMessageV2): string {
  return JSON.stringify(m);
}

/** JSON-decode and validate. Returns null on any structural issue — callers
 *  treat null as "drop the message" rather than throwing, since a malformed
 *  payload from a peer is a network event, not a programmer bug. */
export function decodeMessageV2(s: string): WireMessageV2 | null {
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
      return typeof (m as { t?: unknown }).t === "number" ? (m as WireMessageV2) : null;

    case "session-hello": {
      const r = m as {
        matchId?: unknown;
        phase?: unknown;
        seq?: unknown;
        code?: unknown;
      };
      if (
        typeof r.matchId === "string"
        && r.matchId.length > 0
        && isWirePhase(r.phase)
        && isU32(r.seq)
        && typeof r.code === "string"
        && r.code.length > 0
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "intent": {
      const r = m as { phase?: unknown; nonce?: unknown; raw?: unknown };
      if (
        isWirePhase(r.phase)
        && typeof r.nonce === "string"
        && r.nonce.length > 0
        && isU32(r.raw)
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "committed": {
      const r = m as {
        seq?: unknown;
        phase?: unknown;
        raw?: unknown;
        postZobrist?: unknown;
        originNonce?: unknown;
      };
      if (
        isU32(r.seq)
        && (r.seq as number) >= 1
        && isWirePhase(r.phase)
        && isU32(r.raw)
        && typeof r.postZobrist === "string"
        && /^\d+$/.test(r.postZobrist)
        && (r.originNonce === null || (typeof r.originNonce === "string" && r.originNonce.length > 0))
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "intent-rejected": {
      const r = m as { nonce?: unknown; reason?: unknown };
      if (
        typeof r.nonce === "string"
        && r.nonce.length > 0
        && isIntentRejectReason(r.reason)
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "phase-change": {
      const r = m as {
        from?: unknown;
        to?: unknown;
        snapshotJson?: unknown;
        seq?: unknown;
      };
      if (
        r.from === "draft"
        && r.to === "play"
        && typeof r.snapshotJson === "string"
        && r.snapshotJson.length > 0
        && isU32(r.seq)
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "snapshot": {
      const r = m as {
        snapshotJson?: unknown;
        seq?: unknown;
        phase?: unknown;
        matchId?: unknown;
      };
      if (
        typeof r.snapshotJson === "string"
        && r.snapshotJson.length > 0
        && isU32(r.seq)
        && isWirePhase(r.phase)
        && typeof r.matchId === "string"
        && r.matchId.length > 0
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "request-snapshot": {
      const r = m as { mySeq?: unknown; reason?: unknown };
      if (isU32(r.mySeq) && isSnapshotRequestReason(r.reason)) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "cheat-detected": {
      const r = m as { seq?: unknown; raw?: unknown };
      if (isU32(r.seq) && (r.seq as number) >= 1 && isU32(r.raw)) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "handoff-announce": {
      const r = m as { matchId?: unknown; seq?: unknown };
      if (
        typeof r.matchId === "string"
        && r.matchId.length > 0
        && isU32(r.seq)
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    case "paused":
      return { kind: "paused" };

    case "resumed":
      return { kind: "resumed" };

    case "error":
      return typeof (m as { reason?: unknown }).reason === "string"
        ? (m as WireMessageV2)
        : null;

    case "bodyguard-prompt": {
      const r = m as { src?: unknown; target?: unknown; approach?: unknown };
      if (
        typeof r.src === "number"
        && Number.isInteger(r.src)
        && r.src >= 0
        && r.src < 64
        && typeof r.target === "number"
        && Number.isInteger(r.target)
        && r.target >= 0
        && r.target < 64
        && typeof r.approach === "number"
        && Number.isInteger(r.approach)
        && r.approach >= 0
        && r.approach < 64
      ) {
        return m as WireMessageV2;
      }
      return null;
    }

    default:
      return null;
  }
}

function isU32(v: unknown): v is number {
  return typeof v === "number" && Number.isInteger(v) && v >= 0 && v <= 0xffffffff;
}

function isWirePhase(v: unknown): v is WirePhase {
  return v === "draft" || v === "play";
}

function isIntentRejectReason(v: unknown): v is IntentRejectReason {
  return v === "illegal" || v === "out-of-turn" || v === "phase-mismatch" || v === "paused";
}

function isSnapshotRequestReason(v: unknown): v is SnapshotRequestReason {
  return v === "audit-mismatch" || v === "reconnect" || v === "stale";
}

/** Generate a joiner-side nonce for an intent. Opaque, unique-enough per
 *  session. Format: `i-{base36 random}` — short enough not to bloat the wire,
 *  random enough that two consecutive intents won't collide. */
export function newIntentNonce(): string {
  return "i-" + Math.random().toString(36).slice(2, 10) + Date.now().toString(36).slice(-4);
}
