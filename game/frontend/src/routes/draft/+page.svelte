<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import {
    getEngine,
    encodeDraftTurn,
    decodeMailbox,
    SKILLS,
    SKILL_COUNT,
    CATEGORY_COLOR,
    skillColor,
    type DraftStateView,
    type PositionView,
    type EngineClient,
  } from "$lib/engine";
  import { buildEngineConfigJson } from "$lib/state/match-store.svelte";
  import SkillGlyphDefs from "$lib/board/SkillGlyphDefs.svelte";
  import { t } from "$lib/state/i18n";
  import {
    match,
    modeFromSeats,
    resetMatchState,
    startTelemetrySession,
    recordPly,
    abandonTelemetrySession,
    networkLostTelemetrySession,
    abandonTelemetrySessionSync,
    networkLostTelemetrySessionSync,
  } from "$lib/state/match-store.svelte";
  import {
    mpState,
    onRawData as mpOnRawData,
    sendRaw as mpSendRaw,
    disconnect as mpDisconnect,
  } from "$lib/multiplayer.svelte";
  import {
    decodeMessageV2,
    encodeMessageV2,
    type WireMessageV2,
  } from "$lib/multiplayer-protocol-v2";
  import {
    createMpEngine,
    type MpEngineHandle,
    type Role,
    type SubmitResult,
  } from "$lib/multiplayer-engine";
  import ConnectivityPill from "$lib/multiplayer/ConnectivityPill.svelte";
  import GraceBanner from "$lib/multiplayer/GraceBanner.svelte";
  import { getTelemetryStore } from "$lib/storage";
  import {
    squareName,
    STACK_M_LOADOUT_SQUARES,
  } from "$lib/state/draft";
  import { settings } from "$lib/state/settings.svelte";

  // === Boot / engine handle ==================================================

  const mode = $derived(modeFromSeats(match.side));
  const isMultiplayer = $derived(match.mode === "multiplayer");

  let eng = $state<EngineClient | null>(null);
  let bootError = $state<string | null>(null);
  let booted = $state(false);
  let busy = $state(false);
  let starting = $state(false);

  let position = $state<PositionView | null>(null);
  let draftState = $state<DraftStateView | null>(null);

  const P1_SQUARES = STACK_M_LOADOUT_SQUARES.p1;
  const P2_SQUARES = STACK_M_LOADOUT_SQUARES.p2;
  const allSkillIds = Array.from({ length: SKILL_COUNT }, (_, i) => i + 1);

  const sideToMove = $derived(draftState?.sideToMove ?? 0);
  const isP1Turn = $derived(sideToMove === 0);
  const sideSquares = $derived(isP1Turn ? P1_SQUARES : P2_SQUARES);
  const currentSeat = $derived(isP1Turn ? match.side.p1 : match.side.p2);
  const currentSeatIsAi = $derived(currentSeat === "ai");

  // L7c: human draft commits flow through the wrapper's `submitAction`. AI
  // draft seats are solo-only (multiplayer setup forces both seats human) and
  // bypass the wrapper. `localCanDraft` gates the picker on the wire being
  // open and the seat being ours.
  let mpEngine = $state<MpEngineHandle | null>(null);
  let mpPaused = $state(false);
  let mpConnectedUnsub: (() => void) | null = null;
  let pendingLocalRaw: number | null = null;
  const localCanDraft = $derived.by(() => {
    if (currentSeatIsAi) return false;
    if (!isMultiplayer) return true;
    if (mpState.status !== "connected") return false;
    // Seat-by-localSeat, NOT by role: after a leader handoff the new-host's
    // role flips from "joiner" → "host" but they still occupy seat 1 (P2),
    // and the displaced peer who rejoins still occupies seat 0 (P1). Mapping
    // off role would swap the players' identities mid-game.
    const seat = match.localSeat ?? (match.multiplayerRole === "host" ? 0 : 1);
    if (seat === 0 &&  isP1Turn) return true;
    if (seat === 1 && !isP1Turn) return true;
    return false;
  });

  const draftComplete = $derived((draftState?.turnNo ?? 0) >= 12);

  // === Picker state ==========================================================
  //
  // The two picks of the in-progress turn live on the board itself as
  // "tentative" slots — there is no separate staging area. A pick is
  // (skillId, sq, slot); empty pick = skillId === 0.

  interface PendingPick { skillId: number; sq: number; slot: number; }
  function emptyPick(): PendingPick { return { skillId: 0, sq: -1, slot: -1 }; }

  let pick1 = $state<PendingPick>(emptyPick());
  let pick2 = $state<PendingPick>(emptyPick());

  function clearPicks(): void {
    pick1 = emptyPick();
    pick2 = emptyPick();
  }

  function tentativeAt(sq: number, slot: number): PendingPick | null {
    if (pick1.sq === sq && pick1.slot === slot) return pick1;
    if (pick2.sq === sq && pick2.slot === slot) return pick2;
    return null;
  }

  /** Empty in the engine AND not already held by a tentative pick. */
  function slotIsOpen(sq: number, slot: number): boolean {
    if (!position) return false;
    const entry = decodeMailbox(position.mailbox[sq]);
    const committed = slot === 0 ? entry.skill1 : entry.skill2;
    if (committed !== 0) return false;
    if (tentativeAt(sq, slot)) return false;
    return true;
  }

  /** Catalogue chip → slot drop legality. Same-piece-same-skill is blocked
   *  against committed skills AND against the other tentative pick. */
  function canDropSkillOn(skillId: number, sq: number, slot: number): boolean {
    if (!localCanDraft) return false;
    if (!position) return false;
    if (!sideSquares.includes(sq)) return false;
    if (!slotIsOpen(sq, slot)) return false;
    const entry = decodeMailbox(position.mailbox[sq]);
    const otherCommitted = slot === 0 ? entry.skill2 : entry.skill1;
    if (otherCommitted === skillId) return false;
    // Other tentative pick must not occupy the SAME slot or place the
    // same skill on the same piece.
    const other = (pick1.sq === sq && pick1.slot === slot) ? pick2
                : (pick2.sq === sq && pick2.slot === slot) ? pick1
                : (pick1.skillId === 0 ? pick2 : pick1);
    if (other.skillId !== 0 && other.sq === sq && other.skillId === skillId) return false;
    return true;
  }

  /** Place a skill onto a slot. If both picks are filled, replace the older
   *  one (pick1). Returns true on success. */
  function placePick(skillId: number, sq: number, slot: number): boolean {
    if (!canDropSkillOn(skillId, sq, slot)) return false;
    if (pick1.skillId === 0) {
      pick1 = { skillId, sq, slot };
      return true;
    }
    if (pick2.skillId === 0) {
      pick2 = { skillId, sq, slot };
      return true;
    }
    // Replace the older (pick1).
    pick1 = pick2;
    pick2 = { skillId, sq, slot };
    return true;
  }

  /** Move a tentative pick from one slot to another. */
  function movePick(fromSq: number, fromSlot: number, toSq: number, toSlot: number): boolean {
    const which = (pick1.sq === fromSq && pick1.slot === fromSlot) ? 1
                : (pick2.sq === fromSq && pick2.slot === fromSlot) ? 2
                : 0;
    if (which === 0) return false;
    const p = which === 1 ? pick1 : pick2;
    // Temporarily clear so the destination passes the open check.
    if (which === 1) pick1 = emptyPick();
    else pick2 = emptyPick();
    if (!canDropSkillOn(p.skillId, toSq, toSlot)) {
      // restore
      if (which === 1) pick1 = p; else pick2 = p;
      return false;
    }
    const moved = { skillId: p.skillId, sq: toSq, slot: toSlot };
    if (which === 1) pick1 = moved; else pick2 = moved;
    return true;
  }

  function clearPickAt(sq: number, slot: number): void {
    if (pick1.sq === sq && pick1.slot === slot) pick1 = emptyPick();
    else if (pick2.sq === sq && pick2.slot === slot) pick2 = emptyPick();
  }

  const commitReady = $derived.by(() => {
    if (!localCanDraft) return false;
    if (pick1.skillId === 0 || pick2.skillId === 0) return false;
    if (pick1.sq === pick2.sq && pick1.slot === pick2.slot) return false;
    if (pick1.sq === pick2.sq && pick1.skillId === pick2.skillId) return false;
    return true;
  });

  // === Drag and drop ========================================================
  //
  // dragPayload is the source of truth (HTML5 dataTransfer is opaque during
  // dragover so we can't gate hover styling off it). Two payload kinds:
  //   - { kind: "skill", id }                  — from the catalogue
  //   - { kind: "pick",  sq, slot }            — from an existing tentative slot
  // Stored as a module-level `$state` so individual slot components can ask
  // "would this current drag be legal on me?" for hover feedback.

  type DragPayload =
    | { kind: "skill"; id: number }
    | { kind: "pick"; sq: number; slot: number };

  let dragPayload = $state<DragPayload | null>(null);

  function dragStartSkill(ev: DragEvent, id: number): void {
    if (!localCanDraft) { ev.preventDefault(); return; }
    dragPayload = { kind: "skill", id };
    ev.dataTransfer?.setData("text/plain", `skill:${id}`);
    if (ev.dataTransfer) ev.dataTransfer.effectAllowed = "copy";
  }

  function dragStartPick(ev: DragEvent, sq: number, slot: number): void {
    if (!localCanDraft) { ev.preventDefault(); return; }
    dragPayload = { kind: "pick", sq, slot };
    ev.dataTransfer?.setData("text/plain", `pick:${sq}:${slot}`);
    if (ev.dataTransfer) ev.dataTransfer.effectAllowed = "move";
  }

  function dragEnd(): void { dragPayload = null; }

  function dropOnSlot(ev: DragEvent, sq: number, slot: number): void {
    ev.preventDefault();
    const p = dragPayload;
    dragPayload = null;
    if (!p) return;
    if (p.kind === "skill") placePick(p.id, sq, slot);
    else movePick(p.sq, p.slot, sq, slot);
  }

  function dropOnTrash(ev: DragEvent): void {
    ev.preventDefault();
    const p = dragPayload;
    dragPayload = null;
    if (!p || p.kind !== "pick") return;
    clearPickAt(p.sq, p.slot);
  }

  function dragOverIfLegal(ev: DragEvent, sq: number, slot: number): void {
    const p = dragPayload;
    if (!p) return;
    if (p.kind === "skill") {
      if (canDropSkillOn(p.id, sq, slot)) {
        ev.preventDefault();
        if (ev.dataTransfer) ev.dataTransfer.dropEffect = "copy";
      }
    } else if (p.kind === "pick") {
      if (p.sq === sq && p.slot === slot) return; // same slot — no-op
      if (canDropSkillOn(skillIdOfPick(p), sq, slot)) {
        ev.preventDefault();
        if (ev.dataTransfer) ev.dataTransfer.dropEffect = "move";
      }
    }
  }

  function skillIdOfPick(p: { sq: number; slot: number }): number {
    if (pick1.sq === p.sq && pick1.slot === p.slot) return pick1.skillId;
    if (pick2.sq === p.sq && pick2.slot === p.slot) return pick2.skillId;
    return 0;
  }

  /** For hover styling: would the in-flight drag land on this slot? */
  function isDragHoverTarget(sq: number, slot: number): boolean {
    const p = dragPayload;
    if (!p) return false;
    if (p.kind === "skill") return canDropSkillOn(p.id, sq, slot);
    if (p.kind === "pick") {
      if (p.sq === sq && p.slot === slot) return false;
      return canDropSkillOn(skillIdOfPick(p), sq, slot);
    }
    return false;
  }

  // === Click fallback (and explicit removal) =================================

  function clickSlot(sq: number, slot: number): void {
    if (!localCanDraft) return;
    if (!position) return;
    // If the slot has a tentative pick, clicking clears it.
    if (tentativeAt(sq, slot)) {
      clearPickAt(sq, slot);
      return;
    }
    // Otherwise: ignored — placement requires drag-and-drop.
  }

  // === Commit ================================================================

  async function commitTurn(): Promise<void> {
    if (!eng || !commitReady || busy) return;
    busy = true;
    try {
      const raw = encodeDraftTurn(
        pick1.skillId, pick1.sq, pick1.slot,
        pick2.skillId, pick2.sq, pick2.slot,
      );
      // Funnel local commits through the wrapper. In multiplayer the wrapper
      // sends `intent` (joiner) or `committed` (host) and only resolves
      // accepted=true once the action has been applied to our engine. In solo
      // (`role: "solo"`) it just forwards to eng.tryApply and fires onApplied.
      pendingLocalRaw = raw;
      let result: SubmitResult;
      try {
        result = mpEngine
          ? await mpEngine.submitAction(raw)
          : { accepted: true };
        if (!mpEngine) {
          // Defensive fallback if wrapper failed to instantiate. Should not
          // happen — `booted` only flips true after bootMpEngine.
          await eng.tryApply(raw);
        }
      } finally {
        pendingLocalRaw = null;
      }
      if (!result.accepted) {
        if (result.reason && result.reason !== "illegal") {
          bootError = `pick refused: ${result.reason}`;
        }
        return;
      }
      // Telemetry: wrapper's onApplied early-returns on local commits (via the
      // pendingLocalRaw guard), so we own the recordPly call here. recordPly
      // is a no-op for joiners (Step 2 guard) and the IDB checkpoint runs
      // inside it — replaces the old persistMatchLogCheckpoint call.
      await recordPly(eng);
      clearPicks();
      await refresh();
      // Navigation to /match/ on draft completion is wrapper-driven: the
      // mpEngine detects engine phase crossing (Draft → Move) and fires
      // onPhaseChange("play") for solo, host, and joiner alike.
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // === AI scheduling ========================================================

  /** Single owned timer handle replaces the prior `aiScheduled` boolean. The
   *  handle is set synchronously inside the $effect and cleared inside the
   *  callback or by `cancelAiTimer()` on teardown — no microtask window for
   *  a re-entrant $effect run to schedule a duplicate. */
  let aiTimer: ReturnType<typeof setTimeout> | null = null;
  function cancelAiTimer(): void {
    if (aiTimer !== null) {
      clearTimeout(aiTimer);
      aiTimer = null;
    }
  }
  $effect(() => {
    if (!booted) return cancelAiTimer();
    if (draftComplete) return cancelAiTimer();
    if (!currentSeatIsAi) return cancelAiTimer();
    if (busy || aiTimer !== null) return;
    const delay = mode === "aivai"
      ? Math.max(16, settings.aivaiStepDelayMs)
      : 200;
    aiTimer = setTimeout(() => {
      aiTimer = null;
      if (!booted || busy || draftComplete || !currentSeatIsAi) return;
      void runAiDraftStep();
    }, delay);
  });

  async function runAiDraftStep(): Promise<void> {
    if (!eng || busy) return;
    if (!currentSeatIsAi) return;
    if (draftComplete) return;
    busy = true;
    try {
      clearPicks();
      const r = await eng.stepAi();
      if (r.appliedAction === 0) {
        // AI returned no draft action. The scheduler is keyed off
        // `currentSeatIsAi` which is derived from `match.position`; since
        // nothing changed it won't re-fire on its own — the route just
        // stalls silently. Surface the wedge so the user sees why drafting
        // halted instead of staring at a frozen screen.
        bootError = "AI returned no draft pick — drafting paused";
        return;
      }
      await refresh();
      // AI applies bypass the wrapper (no submitAction call), so the wrapper's
      // automatic phase-change detection doesn't fire. Solo-only path — drive
      // the transition manually.
      if ((draftState?.turnNo ?? 0) >= 12) await finishAndForward();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // === Boot + refresh =======================================================

  async function refresh(): Promise<void> {
    if (!eng) return;
    position = await eng.positionView();
    draftState = await eng.draftState();
  }

  // True while navigating forward into /match/. Used by the teardown paths to
  // decide whether to tear down the PeerJS connection (back-button → home)
  // or keep it alive (draft → match handoff). The peer must persist across
  // the route change so the post-draft snapshot exchange / phase-change
  // envelope can land on /match/.
  let navigatingForward = false;

  // Mirror /match/'s host-side peer-drop detection so the host gets a
  // `mid-match-network-lost` row written if the joiner closes their tab
  // during the draft phase. Without this, a draft-time disconnect leaves no
  // recent-sessions card. See /match/+page.svelte for the identical pattern.
  // Idempotent via `hasMarkedNetworkLost`. Joiners never reach this code
  // path because joiner has no `telemetryMatchId` (Step 2 guard).
  let hasMarkedNetworkLost = false;
  $effect(() => {
    if (match.mode !== "multiplayer") return;
    if (!match.telemetryMatchId) return;
    if (mpState.status === "connected") {
      hasMarkedNetworkLost = false;
      return;
    }
    if (mpState.status !== "disconnected") return;
    if (hasMarkedNetworkLost) return;
    const id = match.telemetryMatchId;
    hasMarkedNetworkLost = true;
    void (async () => {
      try {
        let partial: string | undefined;
        if (eng) {
          try { partial = (await eng.matchLogJson()) ?? undefined; } catch { /* noop */ }
        }
        const store = getTelemetryStore();
        await store.markNetworkLost(id, partial);
      } catch {
        // Swallow — telemetry must never block gameplay.
      }
    })();
  });

  // Page-hide handler: fires on tab hide / close. Any async work started here
  // may be discarded before the IDB transaction commits — we use sync-entry
  // telemetry variants and accept the loss. `onDestroy` keeps the full async
  // path for client-side nav (where awaits resolve normally).
  function pageHideHandler(): void {
    if (match.telemetryMatchId && !match.telemetryFinalised) {
      if (match.mode === "multiplayer") {
        networkLostTelemetrySessionSync();
      } else {
        abandonTelemetrySessionSync();
      }
    }
  }

  /** Instantiate the role-aware wrapper for the booted engine. Subscribes to
   *  V2 wire envelopes (host: validate intents + broadcast committed; joiner:
   *  audit committed actions; solo: just forward submitAction → tryApply) and
   *  bridges mpState → notifyConnectionOpen/Lost via a $effect.root. */
  function bootMpEngine(role: Role, mpCode: string | null): void {
    if (!eng) return;
    const isMp = role !== "solo";
    const send = isMp
      ? ((m: WireMessageV2) => mpSendRaw(encodeMessageV2(m)))
      : (() => {});
    const subscribe = isMp
      ? ((cb: (m: WireMessageV2) => void) => mpOnRawData((raw) => {
          const m = decodeMessageV2(raw);
          if (m) cb(m);
        }))
      : (() => () => {});

    mpEngine = createMpEngine(
      {
        role,
        phase: "draft",
        matchId: match.telemetryMatchId,
        code: mpCode,
      },
      {
        eng,
        send,
        subscribe,
        onApplied: async (raw, _phase) => {
          // Local commits handled in commitTurn — guard via pendingLocalRaw.
          if (raw === pendingLocalRaw) return;
          // Remote-driven apply: the wrapper has already moved the engine;
          // we refresh and let the wrapper's automatic phase-change detection
          // drive navigation. Joiner's recordPly is a Step 2 no-op (joinerRole
          // guard inside the helper).
          await recordPly(eng!);
          clearPicks();
          await refresh();
        },
        onSnapshotApplied: async (_phase) => {
          await refresh();
        },
        onPhaseChange: async (to) => {
          if (to !== "play") return;
          // Stash the snapshot so /match/'s onMount restores into the same
          // engine state. The wrapper already applied the phase-change
          // snapshot to our local engine; we re-serialize so /match/'s
          // existing `pending` branch can re-anchor without clobbering.
          // See plan §Q9a (a).
          try {
            match.pendingSnapshotJson = await eng!.snapshotJson();
          } catch {
            // Best-effort; /match/ falls back to a fresh engine and the
            // host's next committed will re-sync via audit-mismatch.
          }
          match.mode = match.mode === "multiplayer"
            ? "multiplayer"
            : modeFromSeats(match.side);
          navigatingForward = true;
          await goto("../match/");
        },
        onCheatDetected: () => {
          bootError = "anti-cheat: opponent's engine disagreed";
        },
        onResyncFailed: ({ reason, attempts }) => {
          mpState.lastError = `lost sync with host (${reason}, ${attempts} attempts) — try Rejoin`;
        },
        onPausedChange: (p) => { mpPaused = p; },
        onHostCommitted: async () => { /* recordPly fires via onApplied */ },
      },
    );

    if (isMp && mpState.status === "connected") {
      mpEngine.notifyConnectionOpen();
    }
    mpConnectedUnsub = $effect.root(() => {
      $effect(() => {
        if (!mpEngine) return;
        if (match.mode !== "multiplayer") return;
        if (mpState.status === "connected") mpEngine.notifyConnectionOpen();
        else if (mpState.status === "disconnected" || mpState.status === "error") {
          mpEngine.notifyConnectionLost();
        }
      });
    });
  }

  onMount(async () => {
    try {
      // Use multiplayerRole (not match.mode) as the multiplayer indicator —
      // resetMatchState() doesn't clear multiplayerRole, but it does flip mode
      // to "idle". Reading mode here means a reset between routes silently
      // turns off the multiplayer branch below.
      const wasMultiplayer = match.multiplayerRole !== null;
      const mpRole = match.multiplayerRole;
      const mpCode = match.multiplayerCode;
      // Capture an in-flight telemetry id before resetMatchState clears it.
      // Rejoin from the lobby sets this; we want the same row to keep
      // accumulating plies, not a fresh one.
      const carriedTelemetryId = match.telemetryMatchId;
      if (match.pendingSnapshotJson) {
        const e = await getEngine();
        const newCfg = JSON.parse(buildEngineConfigJson(match.side));
        const parsed = JSON.parse(match.pendingSnapshotJson);
        parsed.config = newCfg;
        const newSnap = JSON.stringify(parsed);
        await e.restoreFromSnapshot(newSnap);
        // Probe phase: if the snapshot reflects a still-mid-draft state,
        // KEEP the engine and fall through to the normal draft mount path
        // (skipping createEngineWithDraft). This is the resume path from
        // the multiplayer lobby's Rejoin flow. Otherwise the snapshot
        // represents a finished draft (Phase F's symmetric handoff) — push
        // it forward to /match/.
        const probeView = await e.positionView();
        // Phase::Draft = 2 (see wrapper_api.rs:84).
        if (probeView.currentPhase !== 2) {
          match.pendingSnapshotJson = newSnap;
          match.mode = wasMultiplayer ? "multiplayer" : modeFromSeats(match.side);
          await goto("../match/");
          return;
        }
        // Mid-draft resume. The wrapper subscribes to V2 envelopes itself
        // (via onRawData); no V1 hookup needed here.
        resetMatchState();
        if (carriedTelemetryId) match.telemetryMatchId = carriedTelemetryId;
        if (wasMultiplayer) match.mode = "multiplayer";
        eng = e;
        await refresh();
        booted = true;
        // Telemetry: skip startTelemetrySession when a carrier id is present;
        // the existing row in IDB continues to accumulate plies. Start a
        // fresh session only if we don't have one (defensive — shouldn't
        // happen for the rejoin flow which sets it).
        if (!match.telemetryMatchId) {
          await startTelemetrySession(match.mode, {
            multiplayerCode: mpCode,
            multiplayerRole: mpRole,
          });
        }
        const roleResume: Role = !wasMultiplayer
          ? "solo"
          : mpRole === "host" ? "host" : "joiner";
        bootMpEngine(roleResume, mpCode);
        if (typeof window !== "undefined") {
          window.addEventListener("pagehide", pageHideHandler);
        }
        return;
      }
      // Stale-entry guard: a page reload mid-draft drops the in-memory engine
      // handle AND the match carrier ($state lives in memory). The user lands
      // back at `mode === "idle"` with no pending snapshot — the previous
      // draft is irretrievably gone. Rather than silently restarting in a
      // default HvH setup, bounce back to /setup/ so the user explicitly
      // re-picks seats and draft mode. Multiplayer is exempt: re-entry mid-
      // draft is the host's job to recover via the snapshot handshake.
      if (!wasMultiplayer && match.mode === "idle") {
        bootError = "Draft session lost (page was reloaded). Redirecting to setup…";
        await goto("../setup/");
        return;
      }
      resetMatchState();
      if (carriedTelemetryId) match.telemetryMatchId = carriedTelemetryId;
      if (wasMultiplayer) match.mode = "multiplayer";
      // multiplayerRole/Code survive resetMatchState (owned by the lobby).
      eng = await getEngine();
      const configJson = buildEngineConfigJson(match.side);
      await eng.createEngineWithDraft(configJson);
      await refresh();
      booted = true;
      // Start a telemetry session so draft plies are recorded into the same
      // MatchLog the resume / replay paths consume. Mirrors /match/'s pattern;
      // a no-op for sandbox/replay (the draft route doesn't reach those modes).
      // Skip if a carrier id is already set (rejoin flow).
      if (!match.telemetryMatchId) {
        await startTelemetrySession(match.mode, {
          multiplayerCode: mpCode,
          multiplayerRole: mpRole,
        });
      }
      // Instantiate the wrapper. In multiplayer this also fires
      // notifyConnectionOpen on the host (sending session-hello). Both peers
      // construct identical empty draft engines from identical configs, so
      // seq=0 is correct — first commit advances both lockstep. See plan §Q13.
      const roleFresh: Role = !wasMultiplayer
        ? "solo"
        : mpRole === "host" ? "host" : "joiner";
      bootMpEngine(roleFresh, mpCode);
      if (typeof window !== "undefined") {
        window.addEventListener("pagehide", pageHideHandler);
      }
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
  });

  onDestroy(() => {
    cancelAiTimer();
    if (typeof window !== "undefined") {
      window.removeEventListener("pagehide", pageHideHandler);
    }
    if (mpConnectedUnsub) {
      mpConnectedUnsub();
      mpConnectedUnsub = null;
    }
    mpEngine?.dispose();
    mpEngine = null;
    // Forward-nav into /match/ keeps the engine/peer alive. Any other exit
    // (back-button to /, route change, hot reload) finalises telemetry and,
    // for back-nav, tears down the PeerJS connection so the peer's pill
    // flips immediately rather than waiting for the heartbeat to time out.
    if (!navigatingForward) {
      if (match.telemetryMatchId && !match.telemetryFinalised) {
        if (match.mode === "multiplayer") {
          void networkLostTelemetrySession(eng ?? undefined);
        } else {
          void abandonTelemetrySession(eng ?? undefined);
        }
      }
      if (match.mode === "multiplayer") {
        mpDisconnect();
      }
    }
  });

  // === Finish ================================================================

  async function finishAndForward(): Promise<void> {
    if (!eng) return;
    starting = true;
    try {
      // Solo-only fallback for the AI-driven draft path (AI bypasses the
      // wrapper, so wrapper-driven phase-change auto-broadcast doesn't fire).
      // Human commits + multiplayer routes navigate via the wrapper's
      // onPhaseChange callback above.
      const snap = await eng.snapshotJson();
      match.pendingSnapshotJson = snap;
      match.mode = match.mode === "multiplayer"
        ? "multiplayer"
        : modeFromSeats(match.side);
      navigatingForward = true;
      await goto("../match/");
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
      starting = false;
      navigatingForward = false;
    }
  }

  // === Display helpers =======================================================

  function skillName(id: number): string {
    if (id === 0) return "—";
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.name`) : `?${id}`;
  }

  function skillDesc(id: number): string {
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.desc`) : "";
  }

  function categoryLabel(id: number): string {
    const c = SKILLS[id]?.category;
    if (!c) return "";
    if (c === "strike") return t("wheel.categoryStrike");
    if (c === "shield") return t("wheel.categoryShield");
    if (c === "move")   return t("wheel.categoryMove");
    return t("wheel.categoryMystic");
  }

  function pieceLabel(_sq: number, isKing: boolean, championIdx: number): string {
    return isKing ? t("draft.king") : t("draft.champion", { n: championIdx });
  }

  function slotCommittedSkill(sq: number, slot: number): number {
    if (!position) return 0;
    const entry = decodeMailbox(position.mailbox[sq]);
    return slot === 0 ? entry.skill1 : entry.skill2;
  }
</script>

<SkillGlyphDefs />

<main>
  <header>
    <p class="back"><a href="../">← back</a></p>
    <h1>{t("draft.title")}</h1>
    <small class="mode-tag">{mode}</small>
    {#if isMultiplayer}
      <ConnectivityPill />
    {/if}
  </header>

  {#if isMultiplayer}
    <GraceBanner {eng} {mpEngine} />
  {/if}

  {#if bootError}
    <p class="err">boot error: {bootError}</p>
  {:else if !booted || !position || !draftState}
    <p>{t("app.loading")}</p>
  {:else if draftComplete && starting}
    <p>{t("draft.starting")}</p>
  {:else}
    <section class="status">
      <div class="status-cell">
        <span class="status-label">{t("draft.turn")}</span>
        <span class="status-value">{Math.min(draftState.turnNo + 1, 12)} / 12</span>
      </div>
      <div class="status-cell">
        <span class="status-label">{t("draft.toPick")}</span>
        <span class="status-value" class:p1={isP1Turn} class:p2={!isP1Turn}>
          {isP1Turn ? t("setup.p1Label") : t("setup.p2Label")}
        </span>
      </div>
      <div class="status-cell">
        <span class="status-label">{t("draft.seat")}</span>
        <span class="status-value">
          {currentSeatIsAi ? t("setup.ai") : t("setup.human")}
        </span>
      </div>
      <div class="status-cell commit-cell">
        <button
          type="button"
          class="primary"
          disabled={!commitReady || busy}
          onclick={commitTurn}
        >{t("draft.commitTurn")}</button>
      </div>
    </section>

    {#if !localCanDraft}
      <p class="waiting">
        {#if currentSeatIsAi}
          {t("draft.aiDrafting")}
        {:else if isMultiplayer}
          {t("draft.waitingForPeer", { n: isP1Turn ? 1 : 2 })}
        {:else}
          {t("draft.waitingForPlayer", { n: isP1Turn ? 1 : 2 })}
        {/if}
      </p>
    {/if}

    <div class="layout">
      <!-- Skill catalogue: draggable source tiles -->
      <section class="picker">
        <h2>{t("draft.catalogue")}</h2>
        <ul class="skills" class:disabled={!localCanDraft}>
          {#each allSkillIds as id (id)}
            {@const color = skillColor(id)}
            <li>
              <button
                type="button"
                class="skill-chip"
                style:--cat={color}
                draggable={localCanDraft}
                disabled={!localCanDraft}
                ondragstart={(ev) => dragStartSkill(ev, id)}
                ondragend={dragEnd}
                title={`${skillName(id)} — ${categoryLabel(id)}\n${skillDesc(id)}`}
              >
                <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
                  <use href="#skill-glyph-{id}" />
                </svg>
                <span class="chip-name">{skillName(id)}</span>
                <span class="chip-cat">{categoryLabel(id)}</span>
              </button>
            </li>
          {/each}
        </ul>

        {#if localCanDraft}
          <div
            class="trash"
            class:armed={dragPayload?.kind === "pick"}
            ondragover={(ev) => { if (dragPayload?.kind === "pick") ev.preventDefault(); }}
            ondrop={dropOnTrash}
            role="region"
            aria-label="drop here to remove tentative pick"
          >
            {t("draft.removeHint")}
          </div>
        {/if}
      </section>

      <!-- Pieces (P1 above P2, active side highlighted). Slots are drop
           targets and visually communicate empty / tentative / committed. -->
      <section class="pieces-col">
        {#each [["p1", P1_SQUARES] as const, ["p2", P2_SQUARES] as const] as [side, squares]}
          {@const isActive = (side === "p1") === isP1Turn}
          <section class="side" class:p1={side === "p1"} class:p2={side === "p2"} class:active={isActive}>
            <h2>{side === "p1" ? t("setup.p1Label") : t("setup.p2Label")}</h2>
            <ul class="pieces">
              {#each squares as sq, i (sq)}
                {@const isKing = i === 0}
                <li class:king={isKing}>
                  <div class="piece-id">
                    <span class="pname">{pieceLabel(sq, isKing, i)}</span>
                    <span class="psq">{squareName(sq)}</span>
                  </div>
                  <div class="slots">
                    {#each [0, 1] as slot}
                      {@const committed = slotCommittedSkill(sq, slot)}
                      {@const tent = tentativeAt(sq, slot)}
                      {@const showId = committed !== 0 ? committed : (tent?.skillId ?? 0)}
                      {@const color = showId === 0 ? "transparent" : skillColor(showId)}
                      {@const hover = isDragHoverTarget(sq, slot)}
                      <button
                        type="button"
                        class="slot"
                        class:committed={committed !== 0}
                        class:tentative={tent !== null}
                        class:empty={showId === 0}
                        class:hover-ok={hover}
                        class:active-side={isActive}
                        style:--cat={color}
                        draggable={tent !== null && localCanDraft}
                        ondragstart={tent !== null ? (ev) => dragStartPick(ev, sq, slot) : undefined}
                        ondragend={dragEnd}
                        ondragover={(ev) => dragOverIfLegal(ev, sq, slot)}
                        ondrop={(ev) => dropOnSlot(ev, sq, slot)}
                        onclick={() => clickSlot(sq, slot)}
                        aria-label={`${pieceLabel(sq, isKing, i)} slot ${slot + 1}`}
                        title={showId === 0 ? "" : `${skillName(showId)} — ${categoryLabel(showId)}`}
                      >
                        {#if showId === 0}
                          <span class="slot-empty">slot {slot + 1}</span>
                        {:else}
                          <svg class="slot-glyph" viewBox="0 0 24 24" aria-hidden="true">
                            <use href="#skill-glyph-{showId}" />
                          </svg>
                          <span class="slot-name">{skillName(showId)}</span>
                        {/if}
                      </button>
                    {/each}
                  </div>
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </section>
    </div>
  {/if}
</main>

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0.6rem 1rem 2rem;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.4rem;
  }
  header h1 { font-size: 1.6rem; margin: 0; }
  .back a { text-decoration: none; }
  .mode-tag {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    padding: 0.05em 0.5em;
    color: var(--paper-ink-soft);
    font-size: 0.85rem;
  }
  .status {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    margin: 0.5rem 0 0.8rem;
    padding: 0.5em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
  }
  .status-cell { display: flex; flex-direction: column; gap: 0.1em; }
  .commit-cell { margin-left: auto; }
  .status-label {
    font-size: 0.72rem;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .status-value { font-weight: 600; }
  .status-value.p1 { color: var(--p1, #2b4a8a); }
  .status-value.p2 { color: var(--p2, #a13a2a); }
  .waiting {
    margin: 0.4rem 0 0.8rem;
    padding: 0.5em 0.9em;
    border: 1.5px dashed var(--paper-line-strong);
    border-radius: 6px;
    color: var(--paper-ink-soft);
    font-style: italic;
    text-align: center;
  }
  .layout {
    display: grid;
    grid-template-columns: minmax(280px, 360px) 1fr;
    gap: 1.2rem;
    align-items: start;
  }
  @media (max-width: 820px) {
    .layout { grid-template-columns: 1fr; }
  }
  .picker, .pieces-col > .side {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.8em 1em;
    background: var(--paper-bg);
  }
  .picker h2, .side h2 {
    margin: 0 0 0.5em;
    font-size: 1.05rem;
  }

  /* Catalogue chips */
  .skills {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4em;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .skill-chip {
    --cat: #888;
    display: grid;
    grid-template-rows: auto auto auto;
    align-items: center;
    justify-items: center;
    gap: 0.1em;
    width: 100%;
    padding: 0.45em 0.35em 0.35em;
    font: inherit;
    background: var(--paper-bg);
    border: 1.5px solid var(--cat);
    border-radius: 6px;
    cursor: grab;
    transition: transform 0.08s ease, box-shadow 0.08s ease, background 0.12s ease;
  }
  .skill-chip:hover:not(:disabled) {
    background: color-mix(in srgb, var(--cat) 12%, var(--paper-bg));
    transform: translateY(-1px);
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.08);
  }
  .skill-chip:active:not(:disabled) { cursor: grabbing; }
  .skill-chip:disabled { opacity: 0.4; cursor: not-allowed; }
  .skill-chip .glyph {
    width: 32px;
    height: 32px;
    color: var(--cat);
    stroke-width: 2.4;
  }
  .skill-chip .glyph :global(use) { stroke-width: 2.4; }
  .skill-chip .chip-name {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--paper-ink);
  }
  .skill-chip .chip-cat {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--cat);
  }
  .skills.disabled .skill-chip { opacity: 0.35; cursor: not-allowed; }

  .trash {
    margin-top: 0.8em;
    padding: 0.6em 0.8em;
    border: 1.5px dashed var(--paper-line);
    border-radius: 6px;
    text-align: center;
    color: var(--paper-ink-soft);
    font-size: 0.85rem;
    font-style: italic;
    transition: background 0.12s ease, border-color 0.12s ease, color 0.12s ease;
  }
  .trash.armed {
    border-color: #a94b3b;
    border-style: solid;
    color: #a94b3b;
    background: rgba(169, 75, 59, 0.06);
    font-style: normal;
    font-weight: 600;
  }

  /* Sides */
  .pieces-col { display: grid; gap: 1rem; }
  .side.p1 { border-top: 4px solid var(--p1, #2b4a8a); }
  .side.p2 { border-top: 4px solid var(--p2, #a13a2a); }
  .side:not(.active) { opacity: 0.55; }
  .pieces {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 0.4em;
  }
  .pieces li {
    display: grid;
    grid-template-columns: 9em 1fr;
    gap: 0.6em;
    align-items: center;
    padding: 0.25em 0.3em;
    border-bottom: 1px dashed var(--paper-line);
  }
  .pieces li.king { font-weight: 600; }
  .piece-id { display: flex; flex-direction: column; gap: 0.05em; }
  .pname { font-size: 0.92rem; }
  .psq { color: var(--paper-ink-soft); font-size: 0.78rem; }
  .slots {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.4em;
  }

  /* Slot states.
     - empty:    paper background, dashed light outline, only highlights when
                 the active side is drafting.
     - tentative: dashed accent border in the category color, light tint,
                 grabbable; clicking removes it.
     - committed: solid category background, white glyph, no border accent.
     - hover-ok:  pulsing accent ring when a legal drop is in flight. */
  .slot {
    --cat: transparent;
    position: relative;
    display: grid;
    grid-template-columns: 28px 1fr;
    align-items: center;
    gap: 0.45em;
    font: inherit;
    padding: 0.32em 0.5em;
    border: 1.5px dashed var(--paper-line);
    border-radius: 6px;
    background: var(--paper-bg);
    text-align: left;
    cursor: default;
    min-height: 38px;
    transition: background 0.12s ease, border-color 0.12s ease, box-shadow 0.12s ease;
  }
  .slot.empty .slot-empty {
    font-size: 0.78rem;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    grid-column: 1 / -1;
    text-align: center;
  }
  .slot.empty.active-side {
    border-color: var(--paper-line-strong);
  }
  .slot.tentative {
    border: 2px dashed var(--cat);
    background: color-mix(in srgb, var(--cat) 10%, var(--paper-bg));
    cursor: grab;
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--cat) 25%, transparent);
  }
  .slot.tentative:active { cursor: grabbing; }
  .slot.committed {
    border: 1.5px solid var(--cat);
    background: var(--cat);
    color: #fefcf3;
    cursor: default;
  }
  .slot.committed .slot-name { color: #fefcf3; }
  .slot.committed .slot-glyph { color: #fefcf3; }
  .slot.hover-ok {
    border-color: var(--paper-ink, #1c1a17);
    box-shadow: 0 0 0 3px rgba(199, 155, 58, 0.4);
  }
  .slot-glyph {
    width: 24px;
    height: 24px;
    stroke-width: 2.4;
  }
  .slot.tentative .slot-glyph { color: var(--cat); }
  .slot-name {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .slot.tentative .slot-name { color: var(--cat); }

  .primary {
    padding: 0.55em 1.2em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--accent, #c79b3a);
    color: #fff;
    border-color: var(--accent, #c79b3a);
    font-weight: 600;
    cursor: pointer;
  }
  .primary:hover:not(:disabled) {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
    transform: translateY(-1px);
  }
  .primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
  }
</style>
