// Role-aware multiplayer engine wrapper.
//
// One funnel for all engine apply traffic in a multiplayer (or solo) match.
// Routes call `submitAction(raw)`; the wrapper does the right thing based on
// role:
//
//   solo   → tryApply locally, recordPly.
//   host   → tryApply on the AUTH engine, broadcast `committed`, recordPly.
//            Refuses local actions while paused (joiner disconnected).
//   joiner → send `intent`; wait for `committed{originNonce: nonce}`. On
//            receipt apply on the MIRROR engine and verify postZobrist
//            matches the host's. On mismatch request a snapshot.
//
// Joiner also re-runs every host-originated `committed` action on its mirror
// for anti-cheat audit (host can't fabricate a legal-looking action that the
// joiner's engine would accept and that produces the host's claimed Zobrist).
//
// Dependencies are injected (engine, send, onData, telemetry store, the
// reactive match carrier) so the test file can drive the wrapper with fakes
// and the runtime file can wire it to the real WebSocket relay/WASM/IDB.

import {
  SNAPSHOT_BUDGETS,
  SnapshotValidationError,
  validateSnapshot,
  type EngineClient,
  type PositionView,
} from "./engine";
import {
  encodeMessageV2,
  newIntentNonce,
  type WireMessageV2,
  type WirePhase,
} from "./multiplayer-protocol-v2";

export type Role = "host" | "joiner" | "solo";

export interface SubmitResult {
  accepted: boolean;
  /** Set when `accepted` is false. Stable string codes for UI. */
  reason?: string;
}

/** Metadata accompanying every `onApplied` callback fire. */
export interface OnAppliedMeta {
  /** Engine `PositionView` captured by the wrapper BEFORE `tryApply`. Routes
   *  use it for pre-state diffing (mailbox/bodyguard effects) without having
   *  to re-read engine state - at the time `onApplied` fires the engine has
   *  already moved, so a fresh `positionView()` would return post-state. */
  prePositionView: PositionView;
  /** True when this `onApplied` fires for an action originated locally on
   *  THIS peer (solo `submitAction`, host `submitAction`, or joiner's intent
   *  that the host just acked). False for remote-originated actions (host
   *  receiving a joiner's intent, joiner mirroring a host-originated commit).
   *  Routes use this to gate local-only side effects such as telemetry's
   *  `recordPly`. */
  isLocalEcho: boolean;
}

export interface MpEngineDeps {
  /** The booted engine. AUTH on host/solo; MIRROR on joiner. */
  eng: EngineClient;
  /** Send a wire message. Wrapper assumes this is best-effort - if the
   *  channel is closed, the caller's implementation is responsible for
   *  swallowing the failure. */
  send: (m: WireMessageV2) => void;
  /** Subscribe to inbound wire messages. Returns a disposer. The wrapper
   *  will register exactly one handler and unsubscribe on `dispose`. */
  subscribe: (cb: (m: WireMessageV2) => void) => () => void;
  /** Called after every successfully-applied action so callers can refresh
   *  reactive UI state (`match.position`, `match.legal`, draft view, etc).
   *  Wrapper does not touch the reactive carrier itself - keeps it pure.
   *  `meta.prePositionView` is the engine's view captured BEFORE `tryApply`;
   *  `meta.isLocalEcho` distinguishes locally-submitted from remotely-driven
   *  applies. See `OnAppliedMeta` for full semantics. */
  onApplied: (raw: number, phase: WirePhase, meta: OnAppliedMeta) => Promise<void> | void;
  /** Snapshot was just restored end-to-end (engine + matchId update). UI
   *  should re-pull position/legal. Distinct from `onApplied` because no
   *  single action triggered it. */
  onSnapshotApplied: (phase: WirePhase) => Promise<void> | void;
  /** Host has signalled a draft→play phase transition. Route should
   *  `goto("../match/")`. */
  onPhaseChange: (to: "play") => Promise<void> | void;
  /** Host or joiner detected the other side cheating. Forfeit UI. */
  onCheatDetected: (info: { seq: number; raw: number; side: "host" | "joiner" }) => Promise<void> | void;
  /** Joiner-only: the resync handshake exhausted its retry budget without the
   *  host returning a usable snapshot. Caller should surface "lost sync with
   *  host" and offer a manual rejoin. Wrapper does NOT loop further after
   *  this fires. */
  onResyncFailed?: (info: { reason: string; attempts: number }) => Promise<void> | void;
  /** Called by the wrapper whenever the host pauses / resumes (joiner sees
   *  `paused`/`resumed` messages; host triggers them on disconnect/reconnect
   *  events delivered by the runtime). UI surfaces a "paused" indicator. */
  onPausedChange: (paused: boolean) => void;
  /** Called when the peer sends a draw offer. The route shows a prompt;
   *  calling the returned `reply(accepted)` sends the draw-response. */
  onDrawOffer?: () => void;
  /** Called when the peer responds to our draw offer. accepted=true means
   *  both sides agreed; the route should call agreeDrawGame. */
  onDrawResponse?: (accepted: boolean) => void;
  /** Called when the peer resigns (their seat is passed). The route should
   *  terminate the game with the OTHER seat as the winner. */
  onResign?: (seat: 0 | 1) => void;
  /** Called by the wrapper after every committed action ON THE HOST so the
   *  caller can write to the host's IDB row. Joiner never writes its own
   *  row - caller's implementation should no-op when `role === "joiner"`. */
  onHostCommitted?: () => Promise<void> | void;
  /** Live role accessor. The wrapper reads this on every send/decide-branch
   *  so role changes (the handoff path writes `mpState.role = "host"`) take
   *  effect without re-instantiating the wrapper. */
  getRole: () => Role;
  /** Live code accessor. Used by `session-hello` emission. The handoff path
   *  writes `mpState.code = newCode` and this read sees it on the next emit. */
  getCode: () => string | null;
  /** Called by the wrapper immediately BEFORE it touches the shared engine
   *  with INCOMING REAL traffic (host: a joiner `intent`; joiner: a host
   *  `committed`/`snapshot`/`phase-change`). The route uses it to guarantee
   *  the shared engine is on the true authoritative line before validation -
   *  specifically, to auto-exit sandbox (restore the entry snapshot) so an
   *  in-progress local exploration can't mis-validate the opponent's real
   *  move (the false "engine disagreed" anti-cheat bug, ns-37). Awaited; must
   *  resolve before the wrapper calls tryApply / restoreFromSnapshot. Expected
   *  to be a cheap no-op when not in sandbox. Optional - defaults to a no-op
   *  for solo and for tests that don't wire it. */
  ensureLiveEngine?: () => Promise<void> | void;
  /** Live accessor for the local player's start-of-turn timestamp
   *  (`match.turnStartedMs`, a `Date.now()` reading). Passed to
   *  `eng.tryApply(raw, turnStartedMs)` on the local player's OWN moves
   *  (solo + host self-submit) so the engine records human decision time.
   *  Not applied to opponent moves (their think-time isn't ours to measure).
   *  Optional - defaults to 0 (no timing) for tests / solo callers that
   *  don't wire it. */
  getTurnStartedMs?: () => number;
  /** Live accessor for whether background eval annotation should run after a
   *  local human move. Only true when the eval display is visible — the eval
   *  holds the engine mutex for ~1 s and stalls all subsequent IPC if enabled
   *  without a consumer. Defaults to false. */
  getBackgroundEval?: () => boolean;
}

export interface MpEngineOpts {
  /** Active draft/play phase at construction time. Host updates this on
   *  `phase-change`; joiner inherits from `session-hello`/`snapshot`. */
  phase: WirePhase;
  /** Host owns the IDB row; pass its id. Joiner: null until the first
   *  `session-hello` arrives, then updated via `setMatchId`. Solo: pass the
   *  telemetry id or null if not logging. */
  matchId: string | null;
  /** Optional clock for nonces / debugging. Defaults to Math.random + Date.now. */
  nonceFactory?: () => string;
  /** Diagnostics - captures internal warnings without spamming console.
   *  Defaults to console.warn. */
  warn?: (stage: string, detail?: unknown) => void;
}

export interface MpEngineHandle {
  /** Local user wants to apply `raw`. On host/solo, applies immediately
   *  and resolves with `accepted: true`. On joiner, sends an intent and
   *  resolves when the host's `committed` (matching nonce) lands. Rejection
   *  cases (illegal, out-of-turn, paused, peer dropped, host refused)
   *  resolve with `accepted: false`. */
  submitAction(raw: number): Promise<SubmitResult>;

  /** Host: tell the wrapper that the channel just opened / closed so it can
   *  send `session-hello`, pause/resume, etc. Joiner: notify of reconnect so
   *  it re-requests a snapshot. The runtime calls this from its WebSocket relay
   *  lifecycle handlers; tests call it directly. */
  notifyConnectionOpen(): void;
  notifyConnectionLost(): void;

  /** Host-only: send a fresh snapshot to the joiner. Called explicitly by
   *  the lobby after a reconnect, or implicitly when the joiner sends a
   *  `request-snapshot`. */
  hostSendSnapshot(reason?: "explicit" | "reply"): Promise<void>;

  /** Joiner-only: replace the matchId carried by the wrapper (e.g. after a
   *  handoff). On host/solo, no-op. */
  setMatchId(id: string | null): void;

  /** Flip wrapper-internal state in place for the joiner→host handoff. Caller
   *  (`multiplayer-handoff.ts`) invokes this BEFORE writing `mpState.role =
   *  "host"`; the wrapper's own `getRole()` read still sees "joiner" at entry
   *  (pre-flip check). After this returns, the caller flips `mpState.role`
   *  and `mpState.code`; the wrapper's deps pick those up reactively on the
   *  next send/decide-branch - there's no internal `role`/`codeRef` to keep
   *  in sync. Pending intents are rejected with reason "promoted" since they
   *  targeted an authority that no longer exists. Preserves `seq`, the engine
   *  reference, and the wrapper's subscription. No-op on host/solo. */
  promoteToHost(opts: { matchId: string }): void;

  /** The latest committed seq we know about. */
  getSeq(): number;

  /** Send a draw offer to the peer. No-op in solo mode. */
  sendDrawOffer(): void;
  /** Reply to a draw offer from the peer. No-op in solo mode. */
  sendDrawResponse(accepted: boolean): void;

  /** Notify the peer that `seat` has resigned. The peer terminates the game
   *  with the other seat as the winner. No-op in solo mode. */
  sendResign(seat: 0 | 1): void;

  /** Tear down: unsubscribe, reject any in-flight intents. Idempotent. */
  dispose(): void;
}

export function createMpEngine(opts: MpEngineOpts, deps: MpEngineDeps): MpEngineHandle {
  // --- internal mutable state ------------------------------------------------
  // Role and code are NOT held here - they live in `mpState` (the single
  // source) and are read through `deps.getRole()` / `deps.getCode()` on every
  // branch. This lets `promoteToHost` (or any future role/code mutator) flip
  // them via mpState without re-instantiating the wrapper.
  let phase: WirePhase = opts.phase;
  let matchId: string | null = opts.matchId;
  let seq = 0;
  let paused = false; // host-side: pause while joiner is gone
  let disposed = false;

  // --- seq overflow guard ----------------------------------------------------
  // The wire protocol pins seq to u32. At 1 ply/sec that's 136 years, so this
  // is mostly belt-and-braces - but a malicious/buggy actor could send a near-
  // max seq in `session-hello` and trip undefined behaviour on the next
  // increment. We cap at `SEQ_CAP` (well below 0xffffffff) and refuse to
  // increment past it: the host's `submitAction` returns `{accepted:false,
  // reason:"seq-overflow"}`, joiner's `committed` handler emits a resync
  // request and stops applying. A future protocol revision can introduce a
  // seq-reset frame; for now, this is a hard fault rather than silent wrap.
  const SEQ_CAP = 0xfffffff0;
  function bumpSeqOrFail(): boolean {
    if (seq >= SEQ_CAP) return false;
    seq += 1;
    return true;
  }

  // --- intent rate cap (joiner-side flood protection) -----------------------
  // A malicious/buggy joiner could flood the host with thousands of intents
  // per second, pinning the AUTH engine's `tryApply` and starving host-local
  // moves. Track a sliding 1s window of incoming intents; if it exceeds
  // RATE_CAP_PER_SEC, reject with the `rate-limit` reason. The window resets
  // naturally as old entries fall off the front.
  const RATE_CAP_PER_SEC = 30;
  const RATE_WINDOW_MS = 1000;
  const intentTimestamps: number[] = [];
  function intentExceedsRateCap(now: number): boolean {
    const cutoff = now - RATE_WINDOW_MS;
    while (intentTimestamps.length > 0 && intentTimestamps[0] < cutoff) {
      intentTimestamps.shift();
    }
    if (intentTimestamps.length >= RATE_CAP_PER_SEC) return true;
    intentTimestamps.push(now);
    return false;
  }

  /** Local player's decision time for the move being submitted: now − the
   *  start-of-turn clock. Mirrors the Rust guard — a 0/unset start means "no
   *  timing available" and yields 0 rather than a nonsense-large delta. */
  function computeLocalThoughtMs(): number {
    const started = deps.getTurnStartedMs?.() ?? 0;
    if (started === 0) return 0;
    return Math.max(0, Date.now() - started);
  }

  // Joiner: in-flight intents waiting for `committed` or `intent-rejected`.
  const pendingIntents = new Map<string, {
    raw: number;
    resolve: (r: SubmitResult) => void;
    timer: ReturnType<typeof setTimeout> | null;
  }>();

  const nonceFactory = opts.nonceFactory ?? newIntentNonce;
  const warn = opts.warn ?? ((stage: string, detail?: unknown) => {
    // eslint-disable-next-line no-console
    console.warn(`[mp-engine] ${stage}`, detail);
  });

  /** Reject and clear every in-flight joiner intent with the given reason.
   *  Used by dispose (reason: "disposed"), notifyConnectionLost (peer-lost),
   *  and promoteToHost (promoted). Idempotent. */
  function clearPendingIntents(reason: string): void {
    for (const [n, p] of pendingIntents) {
      if (p.timer) clearTimeout(p.timer);
      p.resolve({ accepted: false, reason });
      pendingIntents.delete(n);
    }
  }

  // Resync retry budget. Joiner: after sending `request-snapshot`, arm a
  // 10s timer. If no snapshot/phase-change arrives in that window, retry
  // once. If the retry also times out, fire `onResyncFailed` and stop -
  // we don't want to hammer the host forever. Successful snapshot or
  // phase-change clears the timer and counter via `clearResyncBudget`.
  const RESYNC_TIMEOUT_MS = 10_000;
  const RESYNC_MAX_ATTEMPTS = 2;
  let resyncAttempts = 0;
  let resyncTimer: ReturnType<typeof setTimeout> | null = null;
  let lastResyncReason: string | null = null;

  function clearResyncBudget(): void {
    if (resyncTimer) {
      clearTimeout(resyncTimer);
      resyncTimer = null;
    }
    resyncAttempts = 0;
    lastResyncReason = null;
  }

  function requestResync(reason: "reconnect" | "stale" | "audit-mismatch", mySeq: number): void {
    if (deps.getRole() !== "joiner") return;
    resyncAttempts += 1;
    lastResyncReason = reason;
    deps.send({ kind: "request-snapshot", mySeq, reason });
    if (resyncTimer) clearTimeout(resyncTimer);
    resyncTimer = setTimeout(() => {
      resyncTimer = null;
      if (disposed) return;
      if (resyncAttempts >= RESYNC_MAX_ATTEMPTS) {
        const attempts = resyncAttempts;
        const r = lastResyncReason ?? reason;
        resyncAttempts = 0;
        lastResyncReason = null;
        warn("resync-exhausted", { reason: r, attempts });
        if (deps.onResyncFailed) {
          void deps.onResyncFailed({ reason: r, attempts });
        }
        return;
      }
      // Retry once with the same reason. Use the current seq, which may
      // have advanced if a stray committed slipped in; the host treats
      // request-snapshot idempotently.
      requestResync(reason, seq);
    }, RESYNC_TIMEOUT_MS);
  }

  // --- wire handler ----------------------------------------------------------
  const unsubscribe = deps.subscribe((m) => {
    if (disposed) return;
    void handleWire(m);
  });

  async function handleWire(m: WireMessageV2): Promise<void> {
    switch (m.kind) {
      case "ping":
      case "pong":
      case "error":
        // Owned by the runtime layer, not us.
        return;

      case "session-hello": {
        if (deps.getRole() !== "joiner") {
          warn("session-hello-as-host");
          return;
        }
        matchId = m.matchId;
        phase = m.phase;
        // Whatever seq the host says, we accept - we'll resync via snapshot
        // before applying anything. If the host's seq is 0, no snapshot is
        // needed; we'll wait for the first `committed`.
        seq = m.seq;
        if (m.seq > 0) {
          requestResync("reconnect", 0);
        }
        return;
      }

      case "snapshot": {
        if (deps.getRole() !== "joiner") {
          warn("snapshot-as-host");
          return;
        }
        try {
          validateSnapshot(m.snapshotJson, {
            maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
            maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
            requireConfig: true,
            source: "host-snapshot",
          });
          // Auto-exit any local sandbox fork before we overwrite the engine,
          // so the sandbox carrier state (match.mode etc.) stays consistent
          // and we never restore onto a mid-exploration fork (ns-37).
          if (deps.ensureLiveEngine) await deps.ensureLiveEngine();
          await deps.eng.restoreFromSnapshot(m.snapshotJson);
          matchId = m.matchId;
          phase = m.phase;
          seq = m.seq;
          clearResyncBudget();
          await deps.onSnapshotApplied(phase);
        } catch (e) {
          if (e instanceof SnapshotValidationError) {
            warn("snapshot-validation-failed", e.reason);
            requestResync("audit-mismatch", seq);
          } else {
            warn("snapshot-restore-failed", e);
          }
        }
        return;
      }

      case "phase-change": {
        if (deps.getRole() !== "joiner") {
          warn("phase-change-as-host");
          return;
        }
        try {
          validateSnapshot(m.snapshotJson, {
            maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
            maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
            requireConfig: true,
            source: "phase-change",
          });
          // Auto-exit any local sandbox fork before overwriting the engine
          // (ns-37) - same rationale as the `snapshot` case.
          if (deps.ensureLiveEngine) await deps.ensureLiveEngine();
          await deps.eng.restoreFromSnapshot(m.snapshotJson);
          phase = m.to;
          seq = m.seq;
          clearResyncBudget();
          await deps.onSnapshotApplied(phase);
          await deps.onPhaseChange("play");
        } catch (e) {
          if (e instanceof SnapshotValidationError) {
            warn("phase-change-validation-failed", e.reason);
            requestResync("audit-mismatch", seq);
          } else {
            warn("phase-change-restore-failed", e);
          }
        }
        return;
      }

      case "intent": {
        if (deps.getRole() !== "host") {
          warn("intent-as-non-host");
          return;
        }
        if (paused) {
          deps.send({ kind: "intent-rejected", nonce: m.nonce, reason: "paused" });
          return;
        }
        if (m.phase !== phase) {
          deps.send({ kind: "intent-rejected", nonce: m.nonce, reason: "phase-mismatch" });
          return;
        }
        // Rate-limit BEFORE the engine round-trip - a flood of illegal intents
        // would otherwise still pin tryApply.
        if (intentExceedsRateCap(Date.now())) {
          deps.send({ kind: "intent-rejected", nonce: m.nonce, reason: "rate-limit" });
          return;
        }
        // Validate via tryApply - engine's deterministic rule check.
        // First ensure the shared engine is on the true authoritative line:
        // if this host is exploring a sandbox fork, auto-exit it now, else
        // tryApply would validate the joiner's real intent against the
        // sandbox-altered position and falsely reject it (ns-37).
        if (deps.ensureLiveEngine) await deps.ensureLiveEngine();
        let prePositionView: PositionView;
        let postZobrist: bigint;
        try {
          prePositionView = await deps.eng.positionView();
          // Record the JOINER's own think-time (shipped in the intent) into
          // the canonical log. `tryApply` takes a start-of-turn timestamp and
          // the engine computes `now − turnStartedMs`; reconstruct that start
          // as `now − thoughtMs` so the engine recovers the joiner's value.
          // The Rust `now` fires ~sub-ms after this Date.now(), well within
          // tolerance for a human decision time measured in seconds. A missing
          // / zero thoughtMs (pre-B2 joiner) yields turnStartedMs=0 → the
          // engine records 0 (unknown), exactly the prior behaviour.
          const turnStartedMs = m.thoughtMs && m.thoughtMs > 0 ? Date.now() - m.thoughtMs : 0;
          await deps.eng.tryApply(m.raw, turnStartedMs);
          const view = await deps.eng.positionView();
          postZobrist = view.zobrist;
        } catch {
          deps.send({ kind: "intent-rejected", nonce: m.nonce, reason: "illegal" });
          return;
        }
        if (!bumpSeqOrFail()) {
          // Engine has accepted the action, but we can't sequence it. Rather
          // than emit a committed with overflowed seq, we surface the fault.
          // The joiner's mirror is now ahead of any committed we'd send -
          // there's no clean recovery short of a fresh match.
          deps.send({ kind: "intent-rejected", nonce: m.nonce, reason: "seq-overflow" });
          warn("seq-overflow", { seq });
          return;
        }
        const committed: WireMessageV2 = {
          kind: "committed",
          seq,
          phase,
          raw: m.raw,
          postZobrist: postZobrist.toString(),
          originNonce: m.nonce,
        };
        deps.send(committed);
        try {
          await deps.onApplied(m.raw, phase, { prePositionView, isLocalEcho: false });
          if (deps.onHostCommitted) await deps.onHostCommitted();
        } catch (e) {
          warn("onApplied-host-intent", e);
        }
        await maybeEmitPhaseChange();
        return;
      }

      case "committed": {
        if (deps.getRole() !== "joiner") {
          warn("committed-as-non-joiner");
          return;
        }
        if (m.seq !== seq + 1) {
          // Out-of-order or duplicate. Drop and request fresh snapshot.
          // Duplicate (m.seq <= seq) → silent; gap (m.seq > seq+1) → resync.
          if (m.seq > seq + 1) {
            requestResync("stale", seq);
          }
          return;
        }
        // Capture `isLocalEcho` BEFORE the pendingIntents.delete below - once
        // we delete, the .has() check would return false and we'd misreport
        // joiner-originated commits as remote-driven.
        const isLocalEcho = !!(m.originNonce && pendingIntents.has(m.originNonce));
        // Ensure the mirror engine is on the true authoritative line before
        // the audit re-apply: if this joiner is exploring a sandbox fork,
        // auto-exit it now, else tryApply below runs against the sandbox-
        // altered state and produces a divergent Zobrist → a false
        // cheat-detected / resync (ns-37).
        if (deps.ensureLiveEngine) await deps.ensureLiveEngine();
        // Audit: re-apply on the mirror engine.
        let prePositionView: PositionView;
        let mirrorZobrist: bigint;
        try {
          prePositionView = await deps.eng.positionView();
          await deps.eng.tryApply(m.raw);
          const view = await deps.eng.positionView();
          mirrorZobrist = view.zobrist;
        } catch {
          // Mirror rejected an action host claimed legal. Host is cheating
          // OR engine versions disagree. Either way: forfeit.
          deps.send({ kind: "cheat-detected", seq: m.seq, raw: m.raw });
          await deps.onCheatDetected({ seq: m.seq, raw: m.raw, side: "host" });
          return;
        }
        if (mirrorZobrist.toString() !== m.postZobrist) {
          // Accidental divergence - ask host for a snapshot to re-anchor.
          // We've already mutated the mirror; the incoming snapshot will
          // restore it.
          requestResync("audit-mismatch", seq);
          return;
        }
        seq = m.seq;
        // Resolve the matching intent if this came from us.
        if (m.originNonce && pendingIntents.has(m.originNonce)) {
          const p = pendingIntents.get(m.originNonce);
          pendingIntents.delete(m.originNonce);
          if (p?.timer) clearTimeout(p.timer);
          p?.resolve({ accepted: true });
        }
        try {
          await deps.onApplied(m.raw, phase, { prePositionView, isLocalEcho });
        } catch (e) {
          warn("onApplied-mirror", e);
        }
        return;
      }

      case "intent-rejected": {
        if (deps.getRole() !== "joiner") {
          warn("intent-rejected-as-non-joiner");
          return;
        }
        const p = pendingIntents.get(m.nonce);
        if (!p) return;
        pendingIntents.delete(m.nonce);
        if (p.timer) clearTimeout(p.timer);
        p.resolve({ accepted: false, reason: m.reason });
        return;
      }

      case "request-snapshot": {
        if (deps.getRole() !== "host") {
          warn("request-snapshot-as-non-host");
          return;
        }
        await hostSendSnapshot("reply");
        return;
      }

      case "cheat-detected": {
        // Joiner has detected host cheating; surface to host UI too.
        if (deps.getRole() !== "host") {
          warn("cheat-detected-as-non-host");
          return;
        }
        await deps.onCheatDetected({ seq: m.seq, raw: m.raw, side: "joiner" });
        return;
      }

      case "handoff-announce": {
        // Caller (lobby) handles this via its own subscription - wrapper
        // ignores it because handoff requires re-creating the wrapper with
        // a flipped role.
        return;
      }

      case "paused":
        if (deps.getRole() === "joiner") {
          paused = true;
          deps.onPausedChange(true);
        }
        return;

      case "resumed":
        if (deps.getRole() === "joiner") {
          paused = false;
          deps.onPausedChange(false);
        }
        return;

      case "draw-offer":
        deps.onDrawOffer?.();
        return;

      case "draw-response":
        deps.onDrawResponse?.(m.accepted);
        return;

      case "resign":
        deps.onResign?.(m.seat);
        return;
    }
  }

  // --- public API ------------------------------------------------------------
  /** Engine `currentPhase` values come from `wrapper_api.rs` - 2 means
   *  Phase::Draft; anything else (Move=0, Skill=1, Ended=3) means play. The
   *  only wire-relevant transition is draft→play, so any post-apply phase
   *  that is NOT 2 while our wrapper's `phase` is still "draft" is the cue
   *  to broadcast `phase-change` and fire `onPhaseChange("play")`. */
  const PHASE_DRAFT = 2;

  /** Host-side: detect a draft→play transition driven by the action that
   *  just landed. Broadcasts the `phase-change` envelope and fires the local
   *  `onPhaseChange("play")` so the route navigates regardless of whether
   *  the trigger action came from the host or from an accepted joiner intent.
   *
   *  Pre-this-refactor, only the manual `hostTransitionToPlay()` API drove
   *  this - and route code only called it when the host's OWN local commit
   *  filled the last draft slot. If the joiner placed the last pick, the
   *  host's engine moved to play but no broadcast fired and the host stayed
   *  on /draft/ (the softlock).
   *
   *  Solo callers also reach this path; we fire `onPhaseChange` for them too
   *  so solo /draft/ can navigate via the same callback rather than the
   *  separate `finishAndForward → goto` flow. The `deps.send` call still
   *  runs for solo but is a no-op (no peer subscribes).
   *
   *  Safe to call after every apply: gated on phase actually crossing. */
  async function maybeEmitPhaseChange(): Promise<void> {
    if (phase !== "draft") return;
    let postPhase: number;
    try {
      const view = await deps.eng.positionView();
      postPhase = view.currentPhase;
    } catch (e) {
      warn("phase-detect-failed", e);
      return;
    }
    if (postPhase === PHASE_DRAFT) return;
    phase = "play";
    if (deps.getRole() === "host") {
      try {
        const snapshotJson = await deps.eng.snapshotJson();
        deps.send({
          kind: "phase-change",
          from: "draft",
          to: "play",
          snapshotJson,
          seq,
        });
      } catch (e) {
        // Snapshot serialisation failure - joiner will resync via the next
        // `request-snapshot`. Local navigation still proceeds.
        warn("phase-change-snapshot-failed", e);
      }
    }
    try {
      await deps.onPhaseChange("play");
    } catch (e) {
      warn("onPhaseChange-local", e);
    }
  }

  async function submitAction(raw: number): Promise<SubmitResult> {
    if (disposed) return { accepted: false, reason: "disposed" };
    const role = deps.getRole();
    if (role === "solo") {
      let prePositionView: PositionView;
      try {
        prePositionView = await deps.eng.positionView();
        await deps.eng.tryApply(raw, deps.getTurnStartedMs?.() ?? 0, deps.getBackgroundEval?.() ?? false);
      } catch (e) {
        return { accepted: false, reason: (e as Error)?.message ?? "illegal" };
      }
      try {
        await deps.onApplied(raw, phase, { prePositionView, isLocalEcho: true });
      } catch (e) {
        warn("onApplied-solo", e);
      }
      await maybeEmitPhaseChange();
      return { accepted: true };
    }
    if (role === "host") {
      if (paused) {
        return { accepted: false, reason: "paused" };
      }
      let prePositionView: PositionView;
      let postZobrist: bigint;
      try {
        prePositionView = await deps.eng.positionView();
        await deps.eng.tryApply(raw, deps.getTurnStartedMs?.() ?? 0, deps.getBackgroundEval?.() ?? false);
        const view = await deps.eng.positionView();
        postZobrist = view.zobrist;
      } catch (e) {
        return { accepted: false, reason: (e as Error)?.message ?? "illegal" };
      }
      if (!bumpSeqOrFail()) {
        // Engine applied, but we can't sequence. Surface as a refused submit.
        // Caller will treat this as an illegal move from the user's POV.
        warn("seq-overflow", { seq });
        return { accepted: false, reason: "seq-overflow" };
      }
      const committed: WireMessageV2 = {
        kind: "committed",
        seq,
        phase,
        raw,
        postZobrist: postZobrist.toString(),
        originNonce: null,
      };
      deps.send(committed);
      try {
        await deps.onApplied(raw, phase, { prePositionView, isLocalEcho: true });
        if (deps.onHostCommitted) await deps.onHostCommitted();
      } catch (e) {
        warn("onApplied-host", e);
      }
      await maybeEmitPhaseChange();
      return { accepted: true };
    }
    // joiner
    const nonce = nonceFactory();
    return new Promise<SubmitResult>((resolve) => {
      // Timeout after 15s - if host hasn't replied, surface a failure so the
      // UI doesn't hang forever. Joiner can retry.
      const timer = setTimeout(() => {
        if (pendingIntents.has(nonce)) {
          pendingIntents.delete(nonce);
          resolve({ accepted: false, reason: "timeout" });
        }
      }, 15_000);
      pendingIntents.set(nonce, { raw, resolve, timer });
      // Include the joiner's OWN think-time so the host records it in the
      // canonical log (the host can't observe our start-of-turn clock). 0 when
      // unwired (tests / no timing available).
      const thoughtMs = computeLocalThoughtMs();
      deps.send({ kind: "intent", phase, nonce, raw, thoughtMs });
    });
  }

  function notifyConnectionOpen(): void {
    if (disposed) return;
    const role = deps.getRole();
    if (role === "host") {
      // Resume host-side operation: clear pause, announce session.
      paused = false;
      deps.onPausedChange(false);
      deps.send({
        kind: "session-hello",
        matchId: matchId ?? "",
        phase,
        seq,
        code: deps.getCode() ?? "",
      });
      // If seq > 0 the joiner will request a snapshot; we wait passively.
      // If seq === 0 (fresh match) the first `committed` is enough.
    } else if (role === "joiner") {
      // Reconnect path - pull a fresh snapshot to be safe.
      deps.send({ kind: "request-snapshot", mySeq: seq, reason: "reconnect" });
    }
  }

  function notifyConnectionLost(): void {
    if (disposed) return;
    const role = deps.getRole();
    if (role === "host") {
      paused = true;
      deps.onPausedChange(true);
      // Best-effort broadcast - channel may already be gone. The runtime's
      // send-on-closed-channel handler swallows.
      deps.send({ kind: "paused" });
    } else if (role === "joiner") {
      // Reject in-flight intents so the UI doesn't hang.
      clearPendingIntents("peer-lost");
    }
  }

  async function hostSendSnapshot(_reason?: "explicit" | "reply"): Promise<void> {
    if (disposed || deps.getRole() !== "host") return;
    if (!matchId) {
      warn("hostSendSnapshot-no-matchId");
      return;
    }
    const snapshotJson = await deps.eng.snapshotJson();
    deps.send({
      kind: "snapshot",
      snapshotJson,
      seq,
      phase,
      matchId,
    });
  }

  function setMatchId(id: string | null): void {
    matchId = id;
  }

  function promoteToHost(promoteOpts: { matchId: string }): void {
    if (disposed) return;
    // Pre-flip check: caller MUST NOT have mutated `mpState.role` yet. The
    // role/code flip happens in the orchestrator AFTER this call returns, so
    // `deps.getRole()` still reads "joiner" here.
    if (deps.getRole() !== "joiner") return;
    // Any joiner intent that hasn't received a `committed` yet was aimed at
    // an authority that no longer exists. Reject so the route's awaiters can
    // clean up their optimistic UI / retry against the new self-host.
    clearPendingIntents("promoted");
    matchId = promoteOpts.matchId;
    // Fresh host is by definition not paused - we just acquired the role.
    paused = false;
    // seq stays put; new host continues the sequence from the last committed
    // action the mirror saw. The next `committed` we broadcast will be
    // seq + 1, matching the natural flow.
  }

  function getSeq(): number {
    return seq;
  }

  function sendDrawOffer(): void {
    if (disposed) return;
    deps.send({ kind: "draw-offer" });
  }

  function sendDrawResponse(accepted: boolean): void {
    if (disposed) return;
    deps.send({ kind: "draw-response", accepted });
  }

  function sendResign(seat: 0 | 1): void {
    if (disposed) return;
    deps.send({ kind: "resign", seat });
  }

  function dispose(): void {
    if (disposed) return;
    disposed = true;
    unsubscribe();
    clearPendingIntents("disposed");
    clearResyncBudget();
  }

  return {
    submitAction,
    notifyConnectionOpen,
    notifyConnectionLost,
    hostSendSnapshot,
    setMatchId,
    promoteToHost,
    getSeq,
    sendDrawOffer,
    sendDrawResponse,
    sendResign,
    dispose,
  };
}

// `encodeMessageV2` re-export is convenient for tests that want to assemble
// raw wire strings without importing the protocol module separately.
export { encodeMessageV2 };
