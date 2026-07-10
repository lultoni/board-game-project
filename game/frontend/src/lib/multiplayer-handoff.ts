// Leader-handoff orchestrator. When the host vanishes mid-match, the joiner
// can promote itself to host by reclaiming the same relay session code and flipping
// its wrapper role in place. This module owns the ordering of operations —
// kept out of the banner component so the policy is testable with fakes.
//
// Operations are issued in this order, intentionally:
//
//   1. destroyPeerKeepState()  — tear down the dying socket synchronously so
//      no stale `committed` frames from the old host can land mid-promotion.
//      Preserves `mpState.code`/`role`/`peerEverPaired` so the banner stays
//      mounted and `hostWithCode` has the same id to reclaim.
//
//   2. Update the existing matches row to `multiplayerRole: "host"` in place.
//      Both peers now maintain their own row via symmetric telemetry, so the
//      joiner already owns a row with the live matchLogJson — only the role
//      changes. As a defensive fallback (row missing), mint a fresh one and
//      baseline it from the engine's current MatchLog.
//
//   3. mpEngine.promoteToHost({matchId}) — flip wrapper-internal state
//      (matchId, paused, pending intents). The wrapper's `getRole()` /
//      `getCode()` deps read live `mpState`, so the role/code flip happens in
//      step 4 below and the wrapper picks it up without a local mutation.
//      MUST run BEFORE step 4's role flip: `promoteToHost` early-outs unless
//      it still sees "joiner".
//
//   4. mpState.role = "host"; mpState.code = code; — single-source flip.
//      After this, `multiplayerRole` / `multiplayerCode` (the $derived
//      constants in match-store) read "host" / code reactively across the UI.
//
//   5. await hostWithCode(code) — register the new peer with the broker.
//      Retries 4× on transient transport errors for broker eviction.
//      Idempotent w.r.t. mpState.role/code (it re-writes the same values).
//
// If any step fails after (1), the carrier is partially mutated and the user
// must either retry (which is idempotent for steps 2–4) or navigate to the
// lobby and start a fresh session.

import type { EngineClient } from "./engine";
import type { MpEngineHandle } from "./multiplayer-engine";
import { destroyPeerKeepState, hostWithCode, mpState } from "./multiplayer.svelte";
import { _telemetrySession, match, type MatchMode } from "./state/match-store.svelte";
import { getTelemetryStore } from "./storage";
import type { TelemetrySession } from "./state/telemetry-session";

/** Minimal subset of the reactive match carrier the orchestrator mutates.
 *  Receives the live $state in the runtime; tests pass a plain object.
 *  Note: MP role/code now live in `mpState` (single source) and are NOT
 *  carried here — the orchestrator writes them via the mpState import. */
export interface HandoffCarrier {
  mode: MatchMode;
  telemetryMatchId: string | null;
}

export interface TakeoverDeps {
  eng: EngineClient;
  mpEngine: MpEngineHandle;
  code: string;
}

export type TakeoverFailReason =
  | "no-code"
  | "telemetry-failed"
  | "engine-failed"
  | "rehost-failed";

export type TakeoverResult =
  | { ok: true }
  | { ok: false; reason: TakeoverFailReason; error?: Error };

/** Injection seam for tests — pass overrides; production resolves to the
 *  static imports at the top of this file. The cycle that originally forced
 *  dynamic imports (state-store ↔ wrapper) was retired in Phase 3c. */
export interface TakeoverHooks {
  destroyPeerKeepState?: () => void;
  hostWithCode?: (code: string) => Promise<string>;
  startTelemetrySession?: TelemetrySession["startTelemetrySession"];
  checkpointMatchLog?: (matchId: string, logJson: string) => Promise<void>;
  updateMultiplayerRole?: (matchId: string, role: "host" | "joiner") => Promise<void>;
  /** Carrier override — runtime passes the live $state, tests pass a stub. */
  carrier?: HandoffCarrier;
}

export async function takeoverAsHost(
  deps: TakeoverDeps,
  hooks: TakeoverHooks = {},
): Promise<TakeoverResult> {
  const destroyPeer = hooks.destroyPeerKeepState ?? destroyPeerKeepState;
  const rehost = hooks.hostWithCode ?? hostWithCode;
  const startTelemetry = hooks.startTelemetrySession ?? _telemetrySession.startTelemetrySession;
  const checkpoint = hooks.checkpointMatchLog
    ?? ((id: string, log: string) => getTelemetryStore().checkpointMatchLog(id, log));
  const updateRole = hooks.updateMultiplayerRole
    ?? ((id: string, role: "host" | "joiner") => getTelemetryStore().updateMultiplayerRole(id, role));
  const carrier: HandoffCarrier = hooks.carrier ?? match;

  if (!deps.code) {
    return { ok: false, reason: "no-code" };
  }

  // 1 — Kill the inbound socket synchronously.
  destroyPeer();

  // 2 — Promote the row's role. With symmetric telemetry the joiner already
  // owns a row; flip its `multiplayerRole` in place. If the row is somehow
  // missing (defensive — telemetry disabled, or IDB failure earlier in the
  // session), mint a fresh one and baseline it from the engine's MatchLog.
  let matchId = carrier.telemetryMatchId;
  if (matchId) {
    try {
      await updateRole(matchId, "host");
    } catch (e) {
      return { ok: false, reason: "telemetry-failed", error: e as Error };
    }
  } else {
    let minted: string | null;
    try {
      minted = await startTelemetry(carrier, carrier.mode, {
        multiplayerCode: deps.code,
        multiplayerRole: "host",
      });
    } catch (e) {
      return { ok: false, reason: "telemetry-failed", error: e as Error };
    }
    if (!minted) {
      return { ok: false, reason: "telemetry-failed" };
    }
    try {
      const logJson = await deps.eng.matchLogJson();
      if (logJson) {
        await checkpoint(minted, logJson);
      }
    } catch (e) {
      return { ok: false, reason: "engine-failed", error: e as Error };
    }
    matchId = minted;
  }

  // 3 — Flip wrapper-internal state (matchId, paused, pending intents) BEFORE
  // mpState.role is mutated. The wrapper's `getRole()` early-out inside
  // `promoteToHost` needs to still see "joiner".
  deps.mpEngine.promoteToHost({ matchId });

  // 4 — Single-source role/code flip. After this point, `multiplayerRole` and
  // `multiplayerCode` (the $derived constants in match-store) read "host" /
  // code reactively across the UI, and the wrapper's `getRole()`/`getCode()`
  // deps pick the new values up on every send/decide-branch.
  console.log(`[mp] role write: ${mpState.role} → host (source: handoff.takeoverAsHost)`);
  mpState.role = "host";
  mpState.code = deps.code;

  // 5 — Bring up the new peer. Retries on broker eviction happen inside.
  try {
    await rehost(deps.code);
  } catch (e) {
    return { ok: false, reason: "rehost-failed", error: e as Error };
  }

  return { ok: true };
}
