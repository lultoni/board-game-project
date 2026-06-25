// Leader-handoff orchestrator. When the host vanishes mid-match, the joiner
// can promote itself to host by reclaiming the same PeerJS code and flipping
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
//   2. startTelemetrySession() with role=host — mint a fresh `matches` row
//      under the new host's ownership. Joiner had no IDB row.
//
//   3. checkpointMatchLog(newMatchId, engine.matchLogJson()) — baseline the
//      new row with the engine state we already have. Without this, a
//      subsequent tab close before the next ply would leave an empty row.
//
//   4. mpEngine.promoteToHost({matchId, code}) — flip wrapper role locally.
//      MUST happen before hostWithCode resolves so that when the old host
//      dials back in as joiner, the wrapper is already authoritative and
//      `notifyConnectionOpen` emits `session-hello` (host path) rather than
//      `request-snapshot` (joiner path).
//
//   5. set `match.multiplayerRole = "host"`, `match.multiplayerCode = code`.
//
//   6. await hostWithCode(code) — register the new peer with the broker.
//      Retries 3× on `peer-unavailable-id` for broker eviction (handled
//      inside `multiplayer.svelte.ts`).
//
// If any step fails after (1), the carrier is partially mutated and the user
// must either retry (which is idempotent for steps 2–5 thanks to the
// `multiplayerRole === "host"` skip in startTelemetrySession) or navigate
// to the lobby and start a fresh session.

import type { EngineClient } from "./engine/types";
import type { MpEngineHandle } from "./multiplayer-engine";
import type { MatchMode } from "./state/match-store.svelte";
import type { startTelemetrySession as StartTelemetrySession } from "./state/telemetry-session";

/** Minimal subset of the reactive match carrier the orchestrator mutates.
 *  Receives the live $state in the runtime; tests pass a plain object. */
export interface HandoffCarrier {
  mode: MatchMode;
  telemetryMatchId: string | null;
  multiplayerRole: "host" | "joiner" | null;
  multiplayerCode: string | null;
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

/** Injection seam for tests — pass overrides; real imports are resolved
 *  lazily so the test file can avoid pulling in `$state`-bearing modules. */
export interface TakeoverHooks {
  destroyPeerKeepState?: () => void;
  hostWithCode?: (code: string) => Promise<string>;
  startTelemetrySession?: typeof StartTelemetrySession;
  checkpointMatchLog?: (matchId: string, logJson: string) => Promise<void>;
  /** Carrier override — runtime passes the live $state, tests pass a stub. */
  carrier?: HandoffCarrier;
}

export async function takeoverAsHost(
  deps: TakeoverDeps,
  hooks: TakeoverHooks = {},
): Promise<TakeoverResult> {
  // Resolve defaults lazily — pulling `multiplayer.svelte` / `match-store.svelte`
  // at module-eval time would force every importer (including tests) through
  // Svelte's `$state` rune, which isn't available in plain vitest.
  const destroyPeer = hooks.destroyPeerKeepState
    ?? (await import("./multiplayer.svelte")).destroyPeerKeepState;
  const hostWithCode = hooks.hostWithCode
    ?? (await import("./multiplayer.svelte")).hostWithCode;
  const startTelemetry = hooks.startTelemetrySession
    ?? (await import("./state/telemetry-session")).startTelemetrySession;
  const checkpoint = hooks.checkpointMatchLog
    ?? (async (id: string, log: string): Promise<void> => {
      const { getTelemetryStore } = await import("./storage");
      await getTelemetryStore().checkpointMatchLog(id, log);
    });
  const carrier: HandoffCarrier = hooks.carrier
    ?? (await import("./state/match-store.svelte")).match;

  if (!deps.code) {
    return { ok: false, reason: "no-code" };
  }

  // 1 — Kill the inbound socket synchronously.
  destroyPeer();

  // 2 — Mint the new matches row under the new host.
  let newMatchId: string | null;
  try {
    newMatchId = await startTelemetry(carrier, carrier.mode, {
      multiplayerCode: deps.code,
      multiplayerRole: "host",
    });
  } catch (e) {
    return { ok: false, reason: "telemetry-failed", error: e as Error };
  }
  if (!newMatchId) {
    return { ok: false, reason: "telemetry-failed" };
  }

  // 3 — Baseline the new row with the engine state we already have.
  try {
    const logJson = await deps.eng.matchLogJson();
    if (logJson) {
      await checkpoint(newMatchId, logJson);
    }
  } catch (e) {
    return { ok: false, reason: "engine-failed", error: e as Error };
  }

  // 4 — Flip wrapper role locally BEFORE the new peer is up, so any inbound
  // DataConnection in the race window finds us in host role.
  deps.mpEngine.promoteToHost({ matchId: newMatchId, code: deps.code });

  // 5 — Update the carrier so route-level $effects see the new role.
  carrier.multiplayerRole = "host";
  carrier.multiplayerCode = deps.code;

  // 6 — Bring up the new peer. Retries on broker eviction happen inside.
  try {
    await hostWithCode(deps.code);
  } catch (e) {
    return { ok: false, reason: "rehost-failed", error: e as Error };
  }

  return { ok: true };
}
