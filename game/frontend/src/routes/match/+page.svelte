<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    getEngine,
    ActionKind,
    decodeAction,
    encodeBodyguardChoice,
    decodeMailbox,
    bitsOf,
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateSnapshot,
    isSelfCast,
    SKILLS,
    skillById,
    focusSplitKind,
    MODIFIER_FOCUS,
    MODIFIER_CHARGE,
    runAiCall,
    plyEvalOf,
    startAivaiProducer,
    stopAivaiProducer,
    aivaiProducerLog,
    onAivaiProgress,
    producerRawsFromLog,
    producerMetaFromLog,
    snapshotActionCount,
    type SearchMetaLog,
    type ProducerPlyMeta,
  } from "$lib/engine";
  import { resolveLoadout, mirrorLoadout } from "$lib/state/draft";
  import { t } from "$lib/state/i18n";
  import {
    match,
    modeFromSeats,
    resetMatchState,
    buildEngineConfigJson,
    applyEvaluatorSettings,
    startTelemetrySession,
    recordPly,
    finalizeTelemetrySession,
    networkLostTelemetrySession,
    multiplayerRole,
    multiplayerCode,
    claimWinByOpponentForfeit,
    resignGame,
    agreeDrawGame,
  } from "$lib/state/match-store.svelte";
  import { settings, slideDurationMs } from "$lib/state/settings.svelte";
  import {
    moveTargetsFor,
    movableSources,
    actableSources,
    findActionByKind,
    approachChoicesFor,
    pickApproachByCursor,
  } from "$lib/state/move-targets";
  import { skillTargetsFor, skillVariantsFor, skillIsCastable, hasFocusModeChoice, hasRetargetVariants, hasSelfAndRetargetChoice, variantIsSelfCast, allyMoverCandidates, allyMoverDestinations, rawForAllyMove, rawForSelfCast, type SkillVariant } from "$lib/state/skill-targets";
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import SkillInfoCard from "$lib/board/SkillInfoCard.svelte";
  import ConnectivityPill from "$lib/multiplayer/ConnectivityPill.svelte";
  import GraceBanner from "$lib/multiplayer/GraceBanner.svelte";
  import MultiplayerStatusStrip from "$lib/multiplayer/MultiplayerStatusStrip.svelte";
  import { tearDownMultiplayerOnLeave } from "$lib/multiplayer/route-lifecycle";
  import { takeoverAsHost } from "$lib/multiplayer-handoff";
  import {
    mpState,
    onRawData as mpOnRawData,
    onConnected as mpOnConnected,
    onDisconnected as mpOnDisconnected,
    sendRaw as mpSendRaw,
    claimRouteOwnership,
    getRouteOwnershipToken,
  } from "$lib/multiplayer.svelte";
  import { decodeMessageV2, encodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { createMpEngine, type MpEngineHandle, type Role, type SubmitResult } from "$lib/multiplayer-engine";
  import { sfx } from "$lib/audio/sfx";
  import { getTelemetryStore } from "$lib/storage";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";
  import { TauriClient } from "$lib/engine/tauri-client";
  import { snapshotJsonFromMatchLog } from "$lib/multiplayer-resume";
  import type { PositionView, EngineClient } from "$lib/engine/types";
  import PlayerPanel from "$lib/match/PlayerPanel.svelte";
  import ProgressionPanel from "$lib/match/ProgressionPanel.svelte";
  import ActionLogPanel from "$lib/match/ActionLogPanel.svelte";
  import EvalBreakdownPanel from "$lib/eval/EvalBreakdownPanel.svelte";
  import SquareEvalCard from "$lib/eval/SquareEvalCard.svelte";
  import {
    aiSearch,
    beginSearch,
    updateDepth,
    endSearch,
    setFinalDepth,
    setHeuristic,
    setHeuristicBySquare,
    setPrevRoundBreakdown,
    setLastRoundSeen,
    setBackgroundEval,
    resetAiSearch,
  } from "$lib/state/ai-search.svelte";

  const mode = $derived(match.mode === "multiplayer" ? "multiplayer" : modeFromSeats(match.side));

  let bootError = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  /** Monotonic ply counter - incremented after every successful apply. Used
   *  to time the AI thinking indicator's post-search linger: the indicator
   *  stays visible for one opponent turn after the search finished. */
  let plyCount = $state(0);
  /** Board square currently under the cursor (for the eval hover card).
   *  null when the cursor is off-board. */
  let hoveredSq = $state<number | null>(null);
  let hoverX = $state(0);
  let hoverY = $state(0);

  /** Role-aware ply renderer. Owns the effects/SFX pipeline, pieceIds,
   *  shakingSquares, effectQueue, and the deferred-skill-refresh state. Both
   *  /match/ and /replay/ create one of these. */
  let renderer = $state<PlyRenderer | null>(null);
  /** Route-ownership token claimed at mount. Passed back to
   *  `tearDownMultiplayerOnLeave` so a stale onDestroy (from a route we
   *  navigated away from before a newer route claimed ownership) can't tear
   *  down the newer session. See route-lifecycle.ts for rationale. */
  let ownershipToken = 0;
  /** AIvAI playback control. When true, the AI loop auto-chains turns. */
  let aiAutoPlay = $state(true);
  /** When true, pause after the current move finishes instead of mid-animation. */
  let pendingPause = $state(false);
  // === AIvAI producer/view split (Change 6) ================================
  // For AIvAI the ENGINE plays the whole game to completion on a background
  // thread (the "producer") while the frontend is a "log player" that replays
  // the producer's log through the interactive view engine at display cadence.
  /** Raw actions the producer has computed so far (its match log). The ceiling
   *  the view advances toward; refreshed from the producer log on each
   *  `aivai-progress` event. */
  let producerRaws = $state<number[]>([]);
  /** Per-ply search readout (depth + P1-POV score) aligned index-for-index with
   *  `producerRaws`. Drives the AIvAI thinking/linger pills in both PlayerPanels
   *  — `advanceView` replays the depth/score into the `aiSearch` store so the
   *  spectator sees the same depth badge HvAI shows. Entries can be null for a
   *  ply that carried no `ai` meta (still shows a pill, just no depth). */
  let producerMetas = $state<(ProducerPlyMeta | null)[]>([]);
  /** How many plies the view engine has rendered. Never exceeds
   *  `producerRaws.length`. */
  let viewPly = $state(0);
  /** True once the producer thread has finished (game over / wedge). */
  let producerDone = $state(false);
  /** Unlisten handle for the `aivai-progress` subscription. */
  let aivaiProgressUnsub: (() => void) | null = null;
  /** True while awaiting the producer's final ply on leave (drives the
   *  "finishing current move…" state). */
  let leavingAivai = $state(false);
  /** Transient toast for export / sandbox feedback. Cleared by a timer. */
  let toast = $state<string>("");
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  /** Confirmation dialog state for sandbox discard. */
  let sandboxConfirmMsg = $state<string | null>(null);
  let sandboxConfirmResolve: ((ok: boolean) => void) | null = null;
  /** Resign / draw dialog state. */
  let showResignConfirm = $state(false);
  let showDrawOfferConfirm = $state(false);   // local HvH or HvAI: offer-draw confirm
  let showIncomingDrawOffer = $state(false);  // MP: peer offered draw, waiting for our response
  let sandboxUndoStack = $state<string[]>([]);
  let sandboxRedoStack = $state<string[]>([]);
  /** "Play My Moves" confirm dialog: list of notation strings to commit. */
  let playMyMovesConfirm = $state<string[] | null>(null);
  let playMyMovesPlaying = $state(false);
  /** Queue of staged sandbox plies waiting to be committed to the real line.
   *  Drained one at a time through the NORMAL apply path (applyRaw), so each
   *  ply respects the same seat/turn/bodyguard gating a live player would.
   *  Cleared on completion, bodyguard interruption, or any player action. */
  let playMyMovesQueue = $state<number[]>([]);
  /** Non-error notice shown when playback halts for a Bodyguard choice. */
  let playMyMovesNotice = $state<string | null>(null);
  function sandboxConfirm(msg: string): Promise<boolean> {
    sandboxConfirmMsg = msg;
    return new Promise((resolve) => {
      sandboxConfirmResolve = resolve;
    });
  }
  /** Whether `eng.matchLogJson()` currently returns a log (config.auto_log on).
   *  Refreshed lazily on toast-bar interaction; cheap to recompute. */
  let matchLogAvailable = $state(false);

  /** True iff the side currently to move is an AI seat. */
  const currentSeatIsAi = $derived.by(() => {
    if (!match.position) return false;
    if (match.position.gameResult !== 0) return false;
    const seat = match.position.toMove === 0 ? match.side.p1 : match.side.p2;
    return seat === "ai";
  });

  /** True iff (in multiplayer) the side currently to move is ours. Outside
   *  multiplayer this is always true - both seats are local. */
  let __loggedSeatFallback150 = false;
  let __loggedSeatFallback526 = false;
  const currentSeatIsLocal = $derived.by(() => {
    if (match.mode !== "multiplayer") return true;
    if (!match.position) return false;
    const toMove = match.position.toMove; // 0 = P1, 1 = P2
    // Seat-by-localSeat, NOT by role: post-handoff the role flips but the
    // peer's board seat stays the same. See match-store.svelte.ts/localSeat.
    const seat = match.localSeat ?? (multiplayerRole() === "host" ? 0 : 1);
    if (match.localSeat === null && !__loggedSeatFallback150) {
      __loggedSeatFallback150 = true;
      console.warn(`[mp] seat fallback used at match:150 (localSeat=null, role=${multiplayerRole()}) → seat=${seat}`);
    }
    return toMove === seat;
  });

  const p1IsAi = $derived(match.side.p1 === "ai");
  const p2IsAi = $derived(match.side.p2 === "ai");

  // Flip the board so the local human always sits at the bottom.
  // HvAI: flip when the human is P2 (seat 1). MP: flip when localSeat is 1.
  // HvH local: no flip (both players share the screen). AIvAI: no flip.
  const boardFlipped = $derived.by(() => {
    if (match.mode === "multiplayer") return match.localSeat === 1;
    if (match.mode === "hvai") return match.side.p1 === "ai"; // human is P2
    return false;
  });

  // Which squares used their Move action this phase — read straight from the
  // engine's authoritative `moved_this_phase` bitboard (projected into
  // PositionView). Deriving it (rather than tracking incrementally) means the
  // greyed-out rendering is correct on LOAD too: resume, snapshot restore, and
  // the time-travel preview all show the right already-moved set immediately,
  // not just for pieces that moved during this session.
  const movedSquares = $derived.by(() => {
    const bb = match.position?.movedThisPhase ?? 0n;
    return new Set<number>(bitsOf(bb));
  });
  /** Preview equivalent of `movedSquares`, read from the frozen preview view. */
  const previewMoved = $derived.by(() => {
    const bb = previewPosition?.movedThisPhase ?? 0n;
    return new Set<number>(bitsOf(bb));
  });
  let lastPhaseKey = $state<string>(""); // `${toMove}:${phase}` - phase boundary detector
  /** Baseline piece counts detected at match boot. Override for custom positions. */
  let baselinePieces = $state<{ kings: number; champs: number; guards: number }>({ kings: 1, champs: 5, guards: 6 });
  /** In-game action log entries for the ActionLogPanel. */
  let actionLogEntries = $state<Array<{ index: number; notation: string; isP1: boolean }>>([]);
  /** Time travel: index of the ply currently being previewed (1-based), or null = present. */
  let previewPly = $state<number | null>(null);
  /** Read-only time-travel preview (P3-E). The preview runs on a SEPARATE,
   *  isolated engine handle (its own `TauriClient` → its own registry entry) so
   *  the live game keeps running untouched — the AI keeps moving, MP peer moves
   *  keep landing, telemetry keeps recording — while the player inspects the
   *  past on a frozen board. Nothing here ever calls into the live `eng`. */
  let previewEng = $state<EngineClient | null>(null);
  let previewRenderer = $state<PlyRenderer | null>(null);
  let previewPosition = $state<PositionView | null>(null);
  /** The live-head ply index captured at the moment preview was ENTERED (the
   *  "you left the present here" marker). Pinned — it does NOT follow the live
   *  head as new moves append while you inspect the past. Null when at present. */
  let leftAtPly = $state<number | null>(null);
  /** True while a past ply is being previewed (board shows the frozen preview
   *  source instead of the live one). Live loops are NOT gated on this. */
  const previewing = $derived(previewPly !== null);

  // Live drag state for the parent - Board owns the pointer mechanics and
  // pushes updates here so we can render path trail + hover ring.
  let dragSrc = $state<number | null>(null);
  let dragTrail = $state<number[]>([]);
  /** Square currently under the pointer (drag or hover). */
  let hoverSq = $state<number | null>(null);
  let dragHover = $state<number | null>(null);
  /** Live cursor position in SVG coords (viewBox = 800), used to pick the
   *  sub-tile approach for multi-path Move-Attacks. (0,0) when idle. */
  let cursorX = $state<number>(0);
  let cursorY = $state<number>(0);

  // Approach-square chooser state.
  let pendingApproach = $state<{ target: number; approaches: number[] } | null>(null);

  // Bodyguard chooser state. Engine-owned via Position.pending_bodyguard.
  const pendingBodyguard = $derived(match.position?.pendingBodyguard ?? null);
  // Clear the "playback paused for Bodyguard" notice once the choice is resolved
  // (the engine drops pending_bodyguard). Playback is not auto-resumed — the
  // user drove the choice, so remaining staged plies stay dropped by design.
  $effect(() => {
    if (pendingBodyguard === null && playMyMovesNotice !== null) {
      playMyMovesNotice = null;
    }
  });

  // Armed skill state.
  let armedSkill = $state<{ square: number; skillId: number } | null>(null);
  let pendingDirection = $state<{ target: number; variants: SkillVariant[] } | null>(null);
  let focusModePref = $state<"activation" | "effect">("activation");
  let focusRetargetPref = $state<"self" | "ally">("self");
  let focusAllyChosen = $state<number | null>(null);

  const focusActive = $derived(
    (match.position?.pendingModifiers ?? 0) & MODIFIER_FOCUS ? true : false,
  );
  const chargeActive = $derived(
    (match.position?.pendingModifiers ?? 0) & MODIFIER_CHARGE ? true : false,
  );
  let hoveredSlice = $state<import("$lib/board/SkillWheel.svelte").SliceKind | null>(null);
  // Delayed visibility for the skill tooltip — shows after 1s of continuous hover.
  let hoveredSliceVisible = $state(false);
  let hoveredSliceTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    if (hoveredSlice === null) {
      if (hoveredSliceTimer) { clearTimeout(hoveredSliceTimer); hoveredSliceTimer = null; }
      hoveredSliceVisible = false;
    } else {
      if (!hoveredSliceVisible) {
        if (hoveredSliceTimer) clearTimeout(hoveredSliceTimer);
        hoveredSliceTimer = setTimeout(() => { hoveredSliceVisible = true; }, 1000);
      }
    }
  });

  const dragHoverLegal = $derived.by(() => {
    if (dragSrc === null || dragHover === null) return false;
    const targets = moveTargetsFor(match.legal, dragSrc);
    return targets.squares.has(dragHover);
  });

  const dragLanding = $derived.by(() => {
    if (dragSrc === null || dragHover === null) return null;
    const targets = moveTargetsFor(match.legal, dragSrc);
    if (!targets.squares.has(dragHover)) return null;
    const approaches = targets.byTarget.get(dragHover);
    if (!approaches || approaches.size === 0) return null;
    return pickApproachByCursor(dragHover, cursorX, cursorY, approaches);
  });

  // Click-mode landing preview: when a piece is selected (no drag) and the
  // cursor is over a legal target, show where the attacker would land - same
  // visual as drag-and-drop hover, but driven by mouse-over without press.
  const clickLanding = $derived.by(() => {
    if (dragSrc !== null) return null; // drag is active, dragLanding takes over
    if (match.selection === null || hoverSq === null) return null;
    if (pendingApproach !== null) return null; // chooser already open
    const targets = moveTargets;
    if (!targets.squares.has(hoverSq)) return null;
    const approaches = targets.byTarget.get(hoverSq);
    if (!approaches || approaches.size === 0) return null;
    if (approaches.size === 1) return approaches.keys().next().value as number;
    return pickApproachByCursor(hoverSq, cursorX, cursorY, approaches);
  });

  const effectiveLanding = $derived(dragSrc !== null ? dragLanding : clickLanding);

  // Money delta preview: show pending cost when a skill is hovered or armed.
  const pendingSkillCost = $derived.by((): number | null => {
    const skillId = hoveredSlice?.kind === "skill" ? hoveredSlice.skillId
      : armedSkill?.skillId ?? null;
    if (skillId === null) return null;
    const s = SKILLS[skillId];
    return s ? s.cost : null;
  });
  // (piece selected, no drag, no pending chooser), show available approach squares
  // so the player can see where they'll land before clicking.
  const hoverApproachChoices = $derived.by((): number[] => {
    if (dragSrc !== null) return [];              // dragging: board handles it
    if (match.selection === null) return [];
    if (pendingApproach !== null) return [];       // chooser already open
    if (hoverSq === null) return [];
    if (!moveTargets.squares.has(hoverSq)) return [];
    const approaches = moveTargets.byTarget.get(hoverSq);
    if (!approaches) return [];
    // Multiple approaches → always show chooser.
    // Single approach that differs from target → speed-2 intermediate square,
    // show it even without ambiguity so the player sees the path.
    const keys = [...approaches.keys()].sort((a, b) => a - b);
    if (approaches.size > 1 || (approaches.size === 1 && keys[0] !== hoverSq)) {
      return keys;
    }
    return [];
  });

  function handlePressStart(src: number) {
    // Player intervention cancels any in-flight Play-My-Moves drain.
    if (playMyMovesQueue.length > 0) playMyMovesQueue = [];
    sfx.unlock();
    sfx.play("pickup");
    match.selection = src;
    dragSrc = src;
    dragTrail = [src];
    dragHover = src;
  }

  function handleDragMove(
    _src: number,
    overSq: number | null,
    path: number[],
    x: number,
    y: number,
  ) {
    dragHover = overSq;
    hoverSq = overSq;
    dragTrail = path;
    cursorX = x;
    cursorY = y;
    if (overSq === null && path.length === 0) {
      // Release signal from Board on pointerup/cancel.
      dragSrc = null;
    }
  }

  // Stable per-piece identity, hit-shake set, and the effects queue all live
  // inside the PlyRenderer now (see imports). The renderer reconciles
  // pieceIds against the engine state on every applyAndRender and on
  // resyncFromEngine.

  let eng = $state<Awaited<ReturnType<typeof getEngine>> | null>(null);

  const moveTargets = $derived(moveTargetsFor(match.legal, match.selection));

  // P2-B: the drag trail's intermediate squares should only show when THIS
  // approach is genuinely speed-2 (the attacker lands one tile short of the
  // hovered square). A piece that CAN move speed-2 elsewhere but is being
  // dragged onto a speed-1 target must not show a trail — so gate on the live
  // hover/landing, not on whether any target is speed-2. When landing === hover
  // it's a plain move / speed-1 attack → no intermediate squares.
  const dragTrailShown = $derived(
    dragLanding !== null && dragHover !== null && dragLanding !== dragHover
      ? dragTrail
      : dragTrail.slice(0, 1),
  );

  // P2-C: the legal target square the cursor is over while a piece is SELECTED
  // and NOT dragging. Drives the click-mode hover ring + landing crosshair in
  // the Board (same preview as a drag). Null unless previewing a real move.
  const clickHoverTarget = $derived(
    match.selection !== null
      && !previewing
      && dragSrc === null
      && pendingApproach === null
      && hoverSq !== null
      && moveTargets.squares.has(hoverSq)
      ? hoverSq
      : null,
  );
  // `selectable` gates piece-pickup interactivity (draggable, click-to-select,
  // cursors). In Move Phase: pieces with at least one Move action. In Skill
  // Phase: pieces with at least one Skill action (movable=empty there anyway).
  // We always union both so the same primitive works across phases.
  const selectable = $derived(actableSources(match.legal));
  /** Pieces that own a *Move* action specifically. Used to render the Move-
   *  Phase "pickup hint" ring (drives the dotted outline on movable pieces),
   *  while `selectable` continues to drive selection / wheel-open. */
  const movable = $derived(movableSources(match.legal));
  const endPhaseAction = $derived(findActionByKind(match.legal, ActionKind.EndPhase));
  const inMovePhase = $derived(match.position?.currentPhase === 0);
  // Phase action-budgets derived from the ruleset so the *inactive* phase box
  // still shows its real count (greyed) rather than a dash. Move is always 2
  // actions; Skill grows with Progression: 2 + floor((round-1)/10).
  const movePhaseBudget = 2;
  const skillPhaseBudget = $derived(
    2 + Math.floor(((match.position?.roundNumber ?? 1) - 1) / 10),
  );
  // Standard interactivity: it's the local seat's turn and we're not busy.
  // Bodyguard no longer needs a special-case override - when the engine sets
  // `pending_bodyguard` it also flips STM to the defender, so the defender's
  // seat naturally becomes `currentSeatIsLocal`.
  // In sandbox mode: always interactive regardless of whose turn it is,
  // so the user can freely move pieces for both sides.
  const interactive = $derived(
    ready
    && !busy
    && !previewing
    && match.position?.gameResult === 0
    && !match.forcedResult
    && (match.mode === "sandbox" || (!currentSeatIsAi && currentSeatIsLocal))
  );

  // Wheel state. Open whenever a piece is selected in the Skill Phase
  // (and the player isn't mid-drag - we don't want the wheel popping up
  // just from press-down). In the Move Phase, selecting a piece highlights
  // move targets only; the wheel is strictly a Skill-Phase affordance.
  const wheelOpen = $derived.by(() => {
    if (!interactive) return null;
    if (match.selection === null) return null;
    if (inMovePhase) return null;
    if (dragSrc !== null) return null;
    if (pendingApproach !== null) return null;
    if (pendingBodyguard !== null) return null;
    if (pendingDirection !== null) return null;
    // Hide the wheel once a skill is armed - the player is now choosing a
    // target, so the wheel chrome would just obscure the board and could
    // intercept clicks meant for target tiles. This applies to focus-split
    // quarters too (picking a quarter arms + closes, exactly like a normal
    // skill half); the armed skill stays cancelable via ✕ Cancel / Escape.
    if (armedSkill !== null) return null;
    const pos = match.position;
    if (!pos) return null;
    const m = decodeMailbox(pos.mailbox[match.selection]);
    if (!m) return null;
    return {
      square: match.selection,
      skill1: m.skill1,
      skill2: m.skill2,
    };
  });

  // Per-sector legality on the wheel. A skill sector is enabled iff at least
  // one Skill action with that (caster, skill_id) is in `legal`. End-Phase
  // is enabled iff `endPhaseAction !== null`. Focus / Charge are themselves
  // skills (ids 14/15) - if the piece has one equipped they show up as the
  const wheelLegality = $derived.by(() => {
    if (!wheelOpen) {
      return { skill1Legal: false, skill2Legal: false };
    }
    const src = wheelOpen.square;
    const skill1Legal = wheelOpen.skill1 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill1);
    const skill2Legal = wheelOpen.skill2 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill2);
    return { skill1Legal, skill2Legal };
  });

  // Per-slot focus split descriptors for the wheel. A slot's half splits into
  // two quarters when Focus is staged AND the slot's skill is focus-eligible by
  // TYPE (focusSplitKind): Blast/Shove → activation(+rng)/effect(+eff); Shield/
  // Dash/Retreat → self/ally. Split is advertised on skill type (not gated on a
  // live legal target); each quarter's legality greys the unavailable side.
  const wheelSplits = $derived.by((): {
    split1: import("$lib/board/SkillWheel.svelte").SplitDesc | null;
    split2: import("$lib/board/SkillWheel.svelte").SplitDesc | null;
  } => {
    if (!wheelOpen || !focusActive) return { split1: null, split2: null };
    const src = wheelOpen.square;
    const build = (
      skillId: number,
    ): import("$lib/board/SkillWheel.svelte").SplitDesc | null => {
      if (skillId <= 0) return null;
      const kind = focusSplitKind(skillId);
      if (kind === null) return null;
      const variants = skillVariantsFor(match.legal, src, skillId);
      const armedHere = armedSkill?.skillId === skillId;
      if (kind === "focusMode") {
        const aLegal = variants.some((v) => !v.focusMode);
        const bLegal = variants.some((v) => v.focusMode);
        // Show the split only if at least one quarter is castable — otherwise
        // the skill isn't legal at all under Focus and its slice is disabled.
        if (!aLegal && !bLegal) return null;
        return {
          kind,
          aLegal,
          bLegal,
          armed: armedHere ? focusModePref : null,
        };
      }
      // retarget: self vs ally
      const aLegal = variants.some((v) => variantIsSelfCast(v, src));
      const bLegal = variants.some((v) => v.hasAux && v.auxSq !== src);
      if (!aLegal && !bLegal) return null;
      return {
        kind,
        aLegal,
        bLegal,
        armed: armedHere ? focusRetargetPref : null,
      };
    };
    return { split1: build(wheelOpen.skill1), split2: build(wheelOpen.skill2) };
  });

  // Whether the currently-armed skill has a Focus-mode choice for the player
  // to make. True only for Blast (10) and Shove (11) when Focus is staged
  // AND the engine emitted both `focus_mode=0` (activation-buff) and
  // `focus_mode=1` (effect-buff) variants. Other skills under Focus have a
  // single interpretation, so no toggle is shown.
  const armedHasFocusModeChoice = $derived.by(() => {
    if (!armedSkill) return false;
    return hasFocusModeChoice(match.legal, armedSkill.square, armedSkill.skillId);
  });

  // Whether the currently-armed skill has a Self vs Ally retarget choice.
  // True when Focus is staged AND the engine emitted both a self-cast branch
  // and at least one ally-retarget branch (Shield, Dash, Retreat).
  const armedHasRetargetChoice = $derived.by(() => {
    if (!armedSkill) return false;
    return hasSelfAndRetargetChoice(match.legal, armedSkill.square, armedSkill.skillId);
  });

  // True when we're in Ally-retarget mode for a movement skill (Dash/Retreat)
  // and need the player to pick WHICH ally moves before showing destinations.
  // Shield retarget has target == ally, so a single click suffices - this is
  // false there.
  const armedNeedsAllyPick = $derived.by(() => {
    if (!armedSkill) return false;
    if (!armedHasRetargetChoice) return false;
    if (focusRetargetPref !== "ally") return false;
    return allyMoverCandidates(match.legal, armedSkill.square, armedSkill.skillId).length > 0;
  });

  // Candidate ally squares in Ally-pick stage 1.
  const armedAllyCandidates = $derived.by(() => {
    if (!armedNeedsAllyPick) return new Set<number>();
    return new Set(allyMoverCandidates(match.legal, armedSkill!.square, armedSkill!.skillId));
  });

  // Target set for the currently-armed skill. Filtered by focusModePref when
  // both interpretations exist; further filtered by focusRetargetPref when a
  // Self/Ally choice exists. For Dash/Retreat retarget in Ally mode the flow
  // is two-stage: stage 1 surfaces ally candidates, stage 2 (after
  // `focusAllyChosen` is set) surfaces that ally's destinations.
  const armedSkillTargets = $derived.by(() => {
    if (!armedSkill) return new Set<number>();
    const src = armedSkill.square;
    if (armedNeedsAllyPick) {
      if (focusAllyChosen === null) return armedAllyCandidates;
      const focusMode = armedHasFocusModeChoice
        ? (focusModePref === "effect")
        : null;
      return allyMoverDestinations(match.legal, src, armedSkill.skillId, focusAllyChosen, focusMode);
    }
    const ts = skillTargetsFor(match.legal, src, armedSkill.skillId);
    const wantEffect = focusModePref === "effect";
    const wantSelf = focusRetargetPref === "self";
    const filtered = new Set<number>();
    for (const [tgt, vs] of ts.variantsByTarget) {
      const matches = vs.some((v) => {
        if (armedHasFocusModeChoice && v.focusMode !== wantEffect) return false;
        if (armedHasRetargetChoice && variantIsSelfCast(v, src) !== wantSelf) return false;
        return true;
      });
      if (matches) filtered.add(tgt);
    }
    if (!armedHasFocusModeChoice && !armedHasRetargetChoice) return ts.squares;
    return filtered;
  });

  // Wheel open/close SFX. Fires once each time the wheel transitions
  // from null → open (a piece is selected in skill phase). We avoid an
  // open sound during boot by gating on `ready`.
  let wheelWasOpen = false;
  $effect(() => {
    const open = wheelOpen !== null;
    if (open && !wheelWasOpen && ready) {
      sfx.play("wheelOpen");
    }
    wheelWasOpen = open;
  });

  // Game-result SFX. Fires once when the game ends (gameResult transitions
  // from 0 to a terminal value). P1 wins = 1, P2 wins = 2, draw = 3.
  let lastGameResult = $state(0);
  $effect(() => {
    const result = match.position?.gameResult ?? 0;
    if (result !== 0 && lastGameResult === 0 && ready) {
      // Determine win/lose from the local player's perspective. In sandbox/aivai
      // there's no "local" side - use gameEnd for draws, victory for any win.
      if (result === 3) {
        sfx.play("gameEnd");
      } else if (match.mode === "aivai" || match.mode === "sandbox") {
        sfx.play("gameEnd");
      } else {
        const localSeat = match.localSeat ?? (match.side.p1 === "human" ? 0 : 1);
        if (match.localSeat === null && !__loggedSeatFallback526) {
          __loggedSeatFallback526 = true;
          // WARNING: this fallback uses match.side.p1 === "human" instead of role - likely wrong in HvH-MP.
          console.warn(`[mp] seat fallback used at match:526 [suspect] (localSeat=null, side.p1=${match.side.p1}, role=${multiplayerRole()}) → seat=${localSeat}`);
        }
        const localWon = (result === 1 && localSeat === 0) || (result === 2 && localSeat === 1);
        sfx.play(localWon ? "victory" : "defeat");
      }
    }
    lastGameResult = result;
  });

  /** HvAI scheduler. When it's the AI seat's turn, queue a `runAiStep()` (a
   *  single blocking search + apply, rendered immediately). AIvAI does NOT use
   *  this path anymore — it's driven by the producer/view log-player below.
   *  Anchored on `phaseKey()` so a stable side+phase pair doesn't re-trigger
   *  when other position fields change. */
  $effect(() => {
    if (!ready) return;
    if (match.mode === "sandbox") return;
    if (match.mode === "aivai") return; // AIvAI is the producer/view loop, not stepAi
    if (playMyMovesQueue.length > 0) return; // Play-My-Moves owns the engine while draining
    if (!currentSeatIsAi) return;
    if (busy) return;
    // Small fixed beat so the board repaints between the human's move and the
    // AI reply. `runAiStep` runs the search in parallel with this delay - it's
    // a floor, not a sequential wait.
    void phaseKey();
    void runAiStep(30);
  });

  /** AIvAI paced view loop (Change 6). The producer thread races ahead and
   *  publishes its ply count via `aivai-progress` (→ `producerRaws`). This loop
   *  is the DISPLAY clock: while playing and there are un-rendered plies, it
   *  waits for the current animation to settle plus the user-configured step
   *  delay, then renders exactly one more ply. It never renders past
   *  `producerRaws.length`, so a fast producer can't outrun the animations.
   *  Re-fires whenever `viewPly`, `producerRaws`, or `aiAutoPlay` change. */
  $effect(() => {
    if (!ready) return;
    if (match.mode !== "aivai") return;
    if (!aiAutoPlay) return;
    if (busy) return;
    // Nothing new to show yet — the producer will bump the ceiling and re-fire.
    if (viewPly >= producerRaws.length) return;
    void viewPly;
    void producerRaws.length;
    void advanceView(Math.max(16, settings.aivaiStepDelayMs));
  });

  onMount(async () => {
    ownershipToken = claimRouteOwnership();
    // Wipe any AI-search transients left over from a previous /match/ session.
    // Route teardown does the same on exit, but resetting on entry too covers
    // navigation paths where a prior route bypassed onDestroy (e.g. hard
    // refresh mid-search).
    resetAiSearch();
    console.log(`[mp] /match/ mounted (mode=${match.mode}, role=${mpState.role}, localSeat=${match.localSeat}, status=${mpState.status})`);
    try {
      eng = await getEngine();
      renderer = createPlyRenderer(eng, {
        // The renderer owns its rendered state; we mirror each flip into the
        // match store here. The store is the explicit writer of its own
        // fields - the renderer only notifies.
        onStateUpdate: (pos, legal) => {
          match.position = pos;
          match.legal = legal;
        },
        // Live moves keep rendering while the player inspects history (the game
        // never pauses), but they render OFF-SCREEN — so mute their SFX while
        // previewing. Evaluated per-play, so audio returns the moment you jump
        // back to the present.
        sfxEnabled: () => !previewing,
      });
      // B3: after a human ply, the engine runs a time-bounded background eval
      // and emits `background-eval-ready`. Pick up the freshly-annotated
      // `background_eval` from the latest ply and surface it in the HUD. Local
      // + hotseat only — in multiplayer the authoritative log lives on the
      // host and eval display would be misleading for the joiner's mirror.
      backgroundEvalUnsub = await eng.onBackgroundEvalReady(async () => {
        if (!eng) return;
        try {
          const json = await eng.latestPlyJson();
          if (!json) return;
          const ply = JSON.parse(json) as { ai?: SearchMetaLog | null; background_eval?: SearchMetaLog | null };
          setBackgroundEval(plyEvalOf(ply));
        } catch {
          // A malformed/absent ply just leaves the prior eval in place.
        }
      });
      const pending = match.pendingSnapshotJson;
      // Snapshot side before reset so it survives the reset (which clears
      // mode/position/legal but preserves side by design).
      const sideAtBoot = { p1: match.side.p1, p2: match.side.p2 };
      // Preserve multiplayer mode through the reset - the lobby set this
      // before navigating here and the reset would otherwise drop mode back
      // to "idle". MP role/code now live in mpState (read via the
      // `multiplayerRole` / `multiplayerCode` $derived constants) so they
      // automatically survive the reset.
      const wasMultiplayer = match.mode === "multiplayer";
      // Task 8 - per-side loadout path (either both pre-made / mirror match,
      // or per-side custom+preMade mixes for local play). Snapshot BEFORE
      // resetMatchState() because the reset clears `sideLoadouts` (so stale
      // ids from a prior match can't leak in via direct navigation).
      // `/setup/` writes the field on commit; we read it here once and
      // consume via `resolveLoadout()`.
      const sideLoadouts = match.sideLoadouts;
      const resumeMatchId = match.resumeMatchId;
      resetMatchState();
      match.side = sideAtBoot;
      if (resumeMatchId) {
        match.resumeMatchId = null; // consume
        let resumed = false;
        try {
          const snap = await getTelemetryStore().getResumeSnapshot(resumeMatchId);
          if (snap) {
            validateSnapshot(snap, {
              maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
              maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
              requireConfig: true,
              source: "idb-resume",
            });
            await eng.restoreFromSnapshot(snap);
            // Re-use the existing telemetry row instead of starting a new one.
            match.telemetryMatchId = resumeMatchId;
            resumed = true;
          }
        } catch (e) {
          console.warn("[resume] failed to restore local game:", e);
        }
        if (!resumed) await eng.createEngine();
      } else if (sideLoadouts) {
        const p1Loadout = await resolveLoadout(sideLoadouts.p1);
        let p2Loadout = await resolveLoadout(sideLoadouts.p2);
        // Mirror match: when both sides use the SAME pre-made loadout, P2's
        // skills must be placed as a 180° rotation of P1's (b1 → g8), not
        // file-aligned. Point-mirror the P2 array so the engine's per-side
        // ascending placement yields a symmetric board. Per-side custom mixes
        // are authored independently and must NOT be mirrored.
        const isMirrorMatch =
          sideLoadouts.p1.kind === "preMade" &&
          sideLoadouts.p2.kind === "preMade" &&
          sideLoadouts.p1.id === sideLoadouts.p2.id;
        if (isMirrorMatch && p2Loadout) {
          p2Loadout = mirrorLoadout(p2Loadout);
        }
        if (p1Loadout && p2Loadout) {
          const configJson = buildEngineConfigJson(sideAtBoot);
          await eng.createEngineWithLoadouts(configJson, p1Loadout, p2Loadout);
        } else {
          // Custom row was deleted between the /setup/ pick and here. Fall
          // back to a blank engine so the route doesn't deadlock.
          console.warn("resolveLoadout returned null; falling back to fresh engine");
          await eng.createEngine();
        }
        // Consume - re-entering /match/ later (e.g. a snapshot restore from
        // the inspector) should NOT re-create from loadouts.
        match.sideLoadouts = null;
      } else if (pending) {
        try {
          validateSnapshot(pending, {
            maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
            maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
            requireConfig: true,
            source: "idb-resume",
          });
          await eng.restoreFromSnapshot(pending);
        } catch (e) {
          if (e instanceof SnapshotValidationError) {
            console.warn("idb-resume validation failed:", e.reason);
          } else {
            console.warn("idb-resume restore failed:", e);
          }
          // Fall back to a fresh engine so the route doesn't deadlock on a
          // corrupt persisted snapshot. The user lands on a clean board.
          await eng.createEngine();
        }
      } else {
        await eng.createEngine();
      }
      await applyEvaluatorSettings(eng);
      await renderer.resyncFromEngine();
      lastPhaseKey = phaseKey();
      // Detect non-standard piece counts for custom positions.
      if (match.position) {
        const pos = match.position;
        function popcount(bb: bigint): number {
          let n = 0; let b = bb;
          while (b !== 0n) { b &= b - 1n; n++; }
          return n;
        }
        const [p1bb, p2bb, kings, , guards] = pos.bitboards;
        const p1Kings = popcount(kings & p1bb);
        const p1Guards = popcount(guards & p1bb);
        const p1Champs = popcount(p1bb) - p1Kings - p1Guards;
        const p2Kings = popcount(kings & p2bb);
        const p2Guards = popcount(guards & p2bb);
        const p2Champs = popcount(p2bb) - p2Kings - p2Guards;
        // Box count per type = max(standard army, actual on either side). A
        // custom position with fewer pieces still shows the standard number of
        // boxes (with the extras pre-filled as "captured"); a position with
        // MORE pieces grows the row. Symmetric across sides so both panels use
        // the same box count.
        const STD = { kings: 1, champs: 5, guards: 6 };
        baselinePieces = {
          kings: Math.max(STD.kings, p1Kings, p2Kings),
          champs: Math.max(STD.champs, p1Champs, p2Champs),
          guards: Math.max(STD.guards, p1Guards, p2Guards),
        };
      }
      // Stamp the decision clock for the opening action window; afterApplied()
      // only sees transitions AFTER the first apply, so seed it here.
      match.turnStartedMs = Date.now();
      match.mode = wasMultiplayer ? "multiplayer" : modeFromSeats(match.side);
      await refreshMatchLogAvailable();
      // Start the telemetry session for non-analysis modes. No-op for
      // sandbox; sandbox enters via /match/ but flips mode immediately,
      // so we'd never reach this with mode === "sandbox" on boot.
      // Skip if a session is already active - the carrier survives the
      // /draft/ → /match/ goto, so the draft route's row continues into
      // play. Without this guard, the handoff produces two IDB rows for
      // one game and resume cards show duplicates.
      if (!match.telemetryMatchId) {
        await startTelemetrySession(match.mode, {
          multiplayerCode: multiplayerCode(),
          multiplayerRole: multiplayerRole(),
        });
      }
      // In multiplayer, subscribe to action messages from the peer and
      // apply them locally (with fromWire: true so we don't echo back).
      // Also handle the resume handshake: hosts validate incoming requests
      // against their MatchLog; joiners restore from the host's snapshot.
      // For solo we hard-pin "solo"; for MP we read live from mpState so a
      // joiner→host handoff (which mutates mpState.role) is reflected on the
      // very next decide-branch without re-creating the wrapper.
      const isMp = wasMultiplayer;
      const currentRole = multiplayerRole();
      const role: Role =
        wasMultiplayer && currentRole === "host"
          ? "host"
          : wasMultiplayer && currentRole === "joiner"
            ? "joiner"
            : "solo";
      mpEngine = createMpEngine(
        {
          phase: "play",
          matchId: match.telemetryMatchId,
        },
        {
          eng,
          getRole: () => (isMp ? ((mpState.role ?? "joiner") as Role) : "solo"),
          getCode: () => mpState.code,
          getTurnStartedMs: () => match.turnStartedMs,
          getBackgroundEval: () => settings.showHeuristicEval || settings.showEvalPanel,
          ensureLiveEngine: ensureLiveEngineOnTrueLine,
          send: (m: WireMessageV2) => mpSendRaw(encodeMessageV2(m)),
          subscribe: (cb) => mpOnRawData((raw) => {
            const decoded = decodeMessageV2(raw);
            if (decoded) cb(decoded);
          }),
          onApplied: async (raw, _phase, meta) => {
            if (!renderer) return;
            // Drain any deferred Skill refresh from a prior remote-applied
            // skill - its setTimeout would otherwise fire after we render
            // this new action and clobber the post-state.
            renderer.drainPendingSkillRefresh();
            // The wrapper captured `prePositionView` before tryApply, so we
            // diff against an explicit pre-state value - no reliance on the
            // reactive mirror's update ordering.
            const pre = renderer.snapshotPreState(raw, meta.prePositionView);
            await renderer.renderApplied(raw, pre);
            match.lastApplied = raw;
            afterApplied();
            // Both peers persist every applied ply into their own matches
            // row. Joiner's onApplied fires on mirror-apply, so this covers
            // remote plies too. The joiner's row is the ground truth when
            // the relay promotes it to host.
            await recordPly(eng!);
            // `meta.isLocalEcho` is observed but currently unused in /match/.
            // It is the future hook for local-only side effects (sounds,
            // haptic feedback) that should NOT fire for remote actions on
            // the non-originating peer.
            void meta.isLocalEcho;
          },
          onSnapshotApplied: async () => {
            if (!renderer) return;
            await renderer.resyncFromEngine();
            lastPhaseKey = phaseKey();
          },
          onPhaseChange: async () => { /* no-op in /match/ */ },
          onCheatDetected: () => {
            bootError = "anti-cheat: opponent's engine disagreed";
          },
          onResyncFailed: ({ reason, attempts }) => {
            mpState.lastError = `lost sync with host (${reason}, ${attempts} attempts) - try Rejoin`;
          },
          onPausedChange: (p) => {
            mpPaused = p;
          },
          onDrawOffer: () => {
            showIncomingDrawOffer = true;
          },
          onDrawResponse: (accepted) => {
            if (accepted && eng) {
              void agreeDrawGame(eng);
            } else {
              toast = "Draw offer declined.";
              if (toastTimer) clearTimeout(toastTimer);
              toastTimer = setTimeout(() => { toast = ""; }, 3000);
            }
          },
          onResign: (seat) => {
            // Peer resigned. resignGame(seat) sets the winner to the OTHER
            // seat and finalises our local telemetry/log the same way our own
            // resign does, so both peers converge on the same result.
            if (eng) void resignGame(eng, seat);
          },
          onHostCommitted: async () => { /* recordPly fires via onApplied */ },
        },
      );
      // Re-announce session on every transport-open while we're mounted.
      // Direct callbacks - no $effect. Protocol sequencing shouldn't be
      // scheduled through Svelte's reactive graph; effects fire on the
      // next microtask, and network state machines don't tolerate that
      // window. See PROTOCOL_TRACE.md Part 2 §6.
      const unsubOpen = mpOnConnected(() => mpEngine?.notifyConnectionOpen());
      const unsubClose = mpOnDisconnected(() => mpEngine?.notifyConnectionLost());
      mpConnectedUnsub = () => { unsubOpen(); unsubClose(); };
      // If the transport is already open by the time /match/ mounts (the
      // usual case - pairing happened in the lobby), the onConnected event
      // has already fired and we missed it. Fire once synchronously so the
      // engine emits its `session-hello`.
      if (mpState.status === "connected") mpEngine.notifyConnectionOpen();

      // AIvAI (Change 6): the engine plays the whole game to completion on a
      // background "producer" thread; this view engine becomes a log player.
      // Seed the producer from THIS engine's snapshot so producer + view share
      // an identical start_fen + config; re-install both seat evaluators
      // (from_snapshot resets them to heuristic engine-side). Gate on the mode
      // set above; HvAI / MP / sandbox never start a producer.
      if (match.mode === "aivai") {
        try {
          const viewSnapshotJson = await eng.snapshotJson();
          // The view engine may already have plies baked in from the pending
          // snapshot it booted from (crucially, the 12 DRAFT plies for a drafted
          // AIvAI match — /draft/ runs those, then forwards a Move-phase
          // snapshot here). `from_snapshot` REBUILDS the producer's log by
          // replaying every action in the snapshot, so the producer log's
          // leading plies ARE those already-applied ones. The view engine is
          // positioned AFTER them, so the log-player must START at that offset —
          // replaying a draft raw onto a Move-phase view engine is exactly the
          // "illegal action" we must avoid. `actions.length` in the snapshot is
          // that baseline ply count.
          viewPly = snapshotActionCount(viewSnapshotJson);
          await startAivaiProducer(
            eng,
            viewSnapshotJson,
            { source: settings.p1Evaluator.source, id: settings.p1Evaluator.id },
            { source: settings.p2Evaluator.source, id: settings.p2Evaluator.id },
          );
          // Progress events only raise the ply-count ceiling + refresh the raw
          // list; the paced view loop (below) decides WHEN to render the next
          // ply, so a fast producer never outruns the animation buffers.
          aivaiProgressUnsub = await onAivaiProgress(eng, async (_plies, done) => {
            try {
              const log = await aivaiProducerLog(eng!);
              producerRaws = producerRawsFromLog(log);
              producerMetas = producerMetaFromLog(log);
            } catch {
              // A transient read failure just leaves the prior ceiling; the
              // next event refreshes it.
            }
            if (done) producerDone = true;
          });
          // Initial pull: the producer starts emitting immediately, so a very
          // fast game could fire (and even finish) between `startAivaiProducer`
          // and the listener attaching above. Read the current log once now so
          // we pick up any plies already computed; every subsequent event
          // refreshes the full list, so no ceiling bump is ever lost.
          try {
            const log0 = await aivaiProducerLog(eng);
            producerRaws = producerRawsFromLog(log0);
            producerMetas = producerMetaFromLog(log0);
          } catch { /* first event will populate it */ }
        } catch (e) {
          console.warn("[aivai] producer start failed; falling back to idle:", e);
        }
      }
      ready = true;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
  });

  /** Role-aware engine wrapper. One funnel for both solo and multiplayer
   *  apply traffic. Owns intent/commit/snapshot/audit logic; the route just
   *  calls submitAction and reads match.position/match.legal as before. */
  let mpEngine = $state<MpEngineHandle | null>(null);
  /** True while the host has paused commits because the joiner dropped. The
   *  HUD surfaces this so the player isn't confused by a refused end-phase. */
  let mpPaused = $state(false);
  /** Disposer for the $effect.root that bridges mpState → wrapper lifecycle. */
  let mpConnectedUnsub: (() => void) | null = null;
  /** Unlisten for the `background-eval-ready` Tauri event (B3). */
  let backgroundEvalUnsub: (() => void) | null = null;

  // MP HUD: is it the peer's turn? Used by MultiplayerStatusStrip to render
  // "Waiting for Player N…" whenever the local player has no agency. Local +
  // hotseat games don't render the strip at all (gated on match.mode).
  const isPeerTurn = $derived.by(() => {
    if (match.mode !== "multiplayer") return false;
    if (!match.position) return false;
    if (match.position.gameResult !== 0) return false;
    if (match.localSeat === null) return false;
    return match.position.toMove !== match.localSeat;
  });
  const peerSeatNumber = $derived((match.localSeat ?? 0) === 0 ? 2 : 1);

  // === Host-side peer-drop detection ========================================
  //
  // The unload/route-change paths in this file write `mid-match-network-lost`
  // rows for the side that's leaving. But when the *other* peer leaves and we
  // stay mounted (e.g. the joiner closed their tab and we're still here as
  // host), we need to write our own row so the lobby's recent-sessions card
  // list picks us up too. Otherwise the host returns to /multiplayer/ later
  // and sees nothing.
  //
  // Unlike the unload paths, this writes the row directly via the store so
  // `match.telemetryMatchId` stays set - the match might recover, in which
  // case `recordPly`/`finalizeMatch` continue working. `markNetworkLost`
  // itself only flips the row if status is "in-progress", so the natural
  // finalize path will still overwrite it to "ended" if the match completes.
  //
  // Idempotent via `hasMarkedNetworkLost` - repeat disconnections during the
  // same mount don't double-write. Cleared on successful reconnect so a
  // flaky-then-recovered peer can re-trigger a new row if it drops again.
  let hasMarkedNetworkLost = false;
  $effect(() => {
    if (match.mode !== "multiplayer") return;
    if (!match.telemetryMatchId) return;
    if (match.telemetryFinalised) return;
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
          try { partial = (await eng.matchLogJson()) ?? undefined; } catch { /* engine bad state */ }
        }
        const store = getTelemetryStore();
        await store.markNetworkLost(id, partial);
      } catch {
        // Swallow - telemetry must never block gameplay.
      }
    })();
  });

  // === Resume handshake ====================================================
  //
  // L7c authoritative-host model: the legacy resume-request/resume-accept/
  // resume-reject mini-protocol is gone. Snapshot push (host → joiner) is
  // the single mechanism for getting a mirror in sync, driven by the wrapper
  // via session-hello + request-snapshot + snapshot envelopes.

  function phaseKey(): string {
    return `${match.position?.toMove ?? -1}:${match.position?.currentPhase ?? -1}`;
  }

  async function refresh() {
    if (!eng) return;
    renderer?.drainPendingSkillRefresh();
    match.position = await eng.positionView();
    match.legal = await eng.legalActions();
  }

  async function applyRaw(raw: number) {
    if (!eng || !renderer || busy) return;
    busy = true;
    try {
      // Sandbox bypasses the wrapper - the user is exploring locally and
      // sandbox moves must NOT echo to a peer or get logged as match plies.
      if (match.mode === "sandbox") {
        const snap = await eng!.snapshotJson();
        await renderer.applyAndRender(raw, async () => {
          await eng!.tryApply(raw);
        });
        sandboxUndoStack = [...sandboxUndoStack.slice(-49), snap];
        sandboxRedoStack = []; // new action clears redo history
        match.sandboxMovesApplied += 1;
        match.lastApplied = raw;
        afterApplied();
        match.selection = null;
        pendingApproach = null;
        pendingDirection = null;
        focusModePref = "activation";
        focusAllyChosen = null;
        return;
      }

      // Funnel through the wrapper. For solo/host, submitAction synchronously
      // calls onApplied(isLocalEcho=true) before returning, which handles the
      // pre-state snapshot, render, recordPly, and lastApplied. For joiner,
      // submitAction sends `intent` and waits for the host's `committed` -
      // the matching commit handler fires onApplied with isLocalEcho=true.
      // Either way, render + telemetry happens inside onApplied; applyRaw
      // only owns the post-success UI cleanup (selection, focus mode).
      renderer.drainPendingSkillRefresh();
      const result: SubmitResult = mpEngine
        ? await mpEngine.submitAction(raw)
        : { accepted: true };
      if (!result.accepted) {
        // Refused (illegal, paused, peer-lost, …). Keep the UI selection so
        // the user can try a different move; surface the reason via bootError
        // for now (a dedicated toast lives in L7c step 5's lobby polish).
        if (result.reason && result.reason !== "illegal") {
          bootError = `move refused: ${result.reason}`;
        }
        return;
      }
      // onApplied has already run by now for solo/host. For joiner, it ran
      // on the same microtask cycle as the intent's promise resolution.
      match.selection = null;
      pendingApproach = null;
      pendingDirection = null;
      focusModePref = "activation";
      focusAllyChosen = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  /** Phase-boundary bookkeeping that used to live inside renderApplied. */
  function afterApplied(): void {
    plyCount += 1;
    if (bootError !== null) bootError = null;
    const k = phaseKey();
    if (k !== lastPhaseKey) {
      // `movedSquares` is derived from the engine's moved_this_phase bitboard,
      // which the engine itself clears on the phase flip — no manual reset here.
      lastPhaseKey = k;
      match.turnStartedMs = Date.now();
      // Clear skill hover/armed state so money preview doesn't persist across phase.
      hoveredSlice = null;
      armedSkill = null;
    }
    // Append to action log (async notation fetch — fire-and-forget).
    if (match.lastApplied !== null && match.mode !== "sandbox") {
      const raw = match.lastApplied;
      const idx = actionLogEntries.length + 1;
      const isP1 = (match.position?.toMove ?? 0) === 1; // toMove already flipped after apply
      const localEng = eng;
      if (localEng) {
        localEng.actionToNotation(raw).then((n) => {
          actionLogEntries = [...actionLogEntries, { index: idx, notation: n, isP1 }];
        }).catch(() => {
          actionLogEntries = [...actionLogEntries, { index: idx, notation: String(raw), isP1 }];
        });
      }
    }
  }

  // Auto-end phase: when the only legal action is EndPhase, fire it without
  // requiring the player to click the button. Skipped in sandbox (exploratory),
  // when busy (mid-apply), and on AI turns (AI handles its own phase endings).
  $effect(() => {
    if (!ready) return;
    if (busy) return;
    if (match.mode === "sandbox") return;
    if (playMyMovesQueue.length > 0) return; // Play-My-Moves owns the engine while draining
    if (match.forcedResult) return;
    if (currentSeatIsAi) return;
    if (!currentSeatIsLocal) return;
    if (match.position?.gameResult !== 0) return;
    const legal = match.legal;
    if (legal.length === 1 && endPhaseAction !== null) {
      void applyRaw(endPhaseAction);
    }
  });

  // Poll the heuristic eval whenever the position advances (including the
  // initial one after engine boot, which afterApplied() never sees) or the
  // relevant settings toggle on. Reads through the same guards.
  $effect(() => {
    void match.position;
    void settings.showHeuristicEval;
    void settings.showEvalPanel;
    if (!(settings.showHeuristicEval || settings.showEvalPanel)) return;
    if (!eng || match.mode === "multiplayer" || !match.position) return;
    if (aiSearch.anyThinking) return;
    const e = eng;
    const priorBreakdown = aiSearch.heuristicEvalBreakdown;
    const priorRound = aiSearch.lastRoundSeen;
    void e.heuristicEval().then((v) => {
      const curRound = match.position?.roundNumber ?? null;
      // On round transition, freeze the last-seen breakdown as the "previous"
      // reference so the panel can display the round-over-round change.
      if (curRound !== null && priorRound !== null && curRound !== priorRound && priorBreakdown !== null) {
        setPrevRoundBreakdown(priorBreakdown);
      }
      setLastRoundSeen(curRound);
      setHeuristic(v);
    }).catch(() => {});
    void e.heuristicEvalBySquare().then((v) => {
      setHeuristicBySquare(v);
    }).catch(() => {});
  });

  /** Run one AI step for the side-to-move, then render the result. The engine
   *  applies the action atomically inside stepAi, so we snapshot pre-state
   *  from the current `match.position` BEFORE the call.
   *
   *  `minDelayMs` runs in parallel with the search so the visible cooldown
   *  is a floor, not a sequential wait. For HvAI this is a small "let the UI
   *  paint" beat; for AIvAI it's the user-configured spectator pacing. Either
   *  way the search starts immediately and we just wait for the slower of the
   *  two to finish before committing. */
  async function runAiStep(minDelayMs: number = 0): Promise<void> {
    if (!eng || !renderer || busy) return;
    if (match.mode === "sandbox") return;
    if (match.mode === "multiplayer") return;
    if (!match.position) return;
    if (match.position.gameResult !== 0) return;
    // Capture side up-front - match.position advances during renderApplied(),
    // so endSearch() at the end of this call needs the side that was to move
    // when the search STARTED, not the (now next) side.
    const side: "p1" | "p2" = match.position.toMove === 0 ? "p1" : "p2";
    busy = true;
    // Do NOT reset lastDepth / lastScore / finishedAtPly here. The prior
    // values stay visible until the streaming depth callback overwrites them
    // (typically within a few frames) or the search completes. The `thinking`
    // spinner already visually takes over from the linger badge, so there's no
    // risk of confusion - and this avoids the "d0 +0" flash the user reported
    // when quick shallow depths report before the deeper ones catch up.
    beginSearch(side);
    try {
      // Drain any deferred Skill refresh before snapshotting pre-state - see
      // applyRaw for rationale.
      renderer.drainPendingSkillRefresh();
      const delayP = minDelayMs > 0
        ? new Promise<void>((r) => setTimeout(r, minDelayMs))
        : Promise.resolve();
      const [result] = await Promise.all([runAiCall(() => eng!.stepAi((d, s) => {
        updateDepth(side, d, s);
      })), delayP]);
      const raw = result.appliedAction;
      setFinalDepth(side, result.depth);
      if (raw === 0) {
        // AI returned no move. Two cases:
        //   - match.position.gameResult !== 0 → terminal (mate/stalemate),
        //     legitimate no-op; just refresh.
        //   - gameResult === 0 → engine returned no action on a live position.
        //     That's a wedge - the AivAI scheduler would re-fire forever.
        //     Disable auto-play and surface a toast so the user sees the
        //     stall instead of an apparent freeze. Don't try to recover here.
        await refresh();
        if (match.position && match.position.gameResult === 0) {
          aiAutoPlay = false;
          showToast("AI returned no move - pausing");
        }
        return;
      }
      // Persist AI ply telemetry. Sandbox is gated above (early return).
      await recordPly(eng);
      // The engine has advanced, but the renderer's rendered state (mirrored
      // into `match.position` via onStateUpdate) is still the PRE-step
      // snapshot - renderApplied hasn't flipped it yet. snapshotPreState reads
      // the renderer's own state, so this is safe.
      const pre = renderer.snapshotPreState(raw);
      await renderer.renderApplied(raw, pre);
      // Wait for the piece animation to finish before the next AI step so the
      // board settles between plies. Gated on the `respectAnimation` setting:
      // when off, we still keep a tiny floor equal to one slide duration to
      // give the browser a paint frame, but multi-hop walks + kill lunges no
      // longer block AI cadence.
      if (settings.respectAnimation) {
        await renderer.animationDone();
      } else {
        const slideDur = slideDurationMs();
        if (slideDur > 0) {
          await new Promise<void>((r) => setTimeout(r, slideDur));
        }
      }
      match.lastApplied = raw;
      afterApplied();
      // Honour deferred pause: user pressed pause during a move, flip now that
      // the animation has finished so the board settles before going idle.
      if (pendingPause) {
        pendingPause = false;
        aiAutoPlay = false;
      }
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      // Capture the ply the search finished on so PlayerPanel can render the
      // greyed-out linger for exactly one opponent turn (until `plyCount`
      // advances past this snapshot). afterApplied already bumped plyCount
      // on the applied AI ply, so `plyCount` at this point == "AI just moved".
      endSearch(side, plyCount);
      busy = false;
    }
  }

  /** AIvAI log-player step (Change 6): render exactly ONE more ply from the
   *  producer's computed log through the VIEW engine, respecting the display
   *  cadence. This is the replay route's `stepForward` pattern applied to the
   *  producer log — the engine already computed the ply; we only apply it to
   *  the view engine and render it. Deliberately does NOT call `recordPly`:
   *  the producer's log is the authoritative one saved to the library.
   *
   *  `minDelayMs` runs in parallel with the (near-instant) view apply so the
   *  visible cadence is a floor. The animation gate runs AFTER the render so
   *  the next ply doesn't start until the board settles. */
  async function advanceView(minDelayMs: number = 0): Promise<void> {
    if (!eng || !renderer || busy) return;
    if (match.mode !== "aivai") return;
    if (viewPly >= producerRaws.length) return;
    const raw = producerRaws[viewPly];
    // Which seat computed this ply — captured BEFORE apply flips to-move, so the
    // depth/score pill and its post-render linger attribute to the right panel.
    const side: "p1" | "p2" = (match.position?.toMove ?? 0) === 0 ? "p1" : "p2";
    // Per-ply search readout the producer recorded when it chose this move.
    const meta = producerMetas[viewPly] ?? null;
    busy = true;
    // Drive the AIvAI thinking pill: HvAI does this via runAiStep's
    // beginSearch/updateDepth/endSearch; the producer/view split never did, so
    // both panels stayed blank (BUG-2). We replay the producer's recorded
    // depth/score into the same store the panels read. `beginSearch` shows the
    // spinner during this ply's (near-instant) view apply; the linger badge
    // then keeps the last depth visible until the next ply advances.
    beginSearch(side);
    if (meta) {
      // Score in the log is P1-POV; PlayerPanel shows it raw and HvAI feeds it
      // seat-relative, so flip sign for P2 to match that convention.
      const seatScore = meta.scoreCp === null ? 0 : (side === "p1" ? meta.scoreCp : -meta.scoreCp);
      updateDepth(side, meta.depth, seatScore);
      setFinalDepth(side, meta.depth);
    }
    try {
      renderer.drainPendingSkillRefresh();
      const delayP = minDelayMs > 0
        ? new Promise<void>((r) => setTimeout(r, minDelayMs))
        : Promise.resolve();
      // Apply the producer-computed raw to the view engine and render it. The
      // renderer reads post-state from the (view) engine, which now sits at the
      // post-ply position — exactly as replay's stepForward relies on.
      await Promise.all([
        renderer.applyAndRender(raw, async () => { await eng!.tryApply(raw); }),
        delayP,
      ]);
      viewPly += 1;
      // Settle the board before the loop re-fires for the next ply.
      if (settings.respectAnimation) {
        await renderer.animationDone();
      } else {
        const slideDur = slideDurationMs();
        if (slideDur > 0) await new Promise<void>((r) => setTimeout(r, slideDur));
      }
      match.lastApplied = raw;
      afterApplied();
      // Persist a resume snapshot from the VIEW engine so an interrupted AIvAI
      // game can be resumed from the library. advanceView deliberately skips
      // recordPly (the producer owns the authoritative log), so this is the
      // only place an AIvAI resume snapshot gets written. Non-fatal on failure.
      if (match.telemetryMatchId) {
        try {
          const snap = await eng.snapshotJson();
          if (snap) await getTelemetryStore().saveResumeSnapshot(match.telemetryMatchId, snap);
        } catch { /* resume just won't be available for this ply */ }
      }
      // Honour a deferred pause requested mid-render.
      if (pendingPause) {
        pendingPause = false;
        aiAutoPlay = false;
      }
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      // End this seat's search so the pill flips from spinner → linger badge.
      // afterApplied() already bumped plyCount for the applied ply, so this
      // records "AI just moved" — the linger lasts until viewPly advances past
      // it, exactly like runAiStep's HvAI path.
      endSearch(side, plyCount);
      busy = false;
    }
  }

  /** After the attacker has decided (target, approach), apply the tentative
   *  Move-Attack. If the engine has eligible Bodyguard Guards it will set
   *  `pending_bodyguard` + flip STM to the defender, who then resolves via a
   *  `BodyguardChoice` ply - handled in `handleSquareClick`. All four play
   *  modes (local HvH, HvAI, AivAI, online HvH) flow through this same path
   *  because the engine owns the STM transition. */
  function commitMoveTargetApproach(target: number, approach: number) {
    const perTarget = moveTargets.byTarget.get(target);
    if (!perTarget) return;
    const variants = perTarget.get(approach);
    if (!variants) return;
    applyRaw(variants.defenderRaw);
  }

  function tryCommitMoveTo(target: number, cx: number, cy: number) {
    const candidates = moveTargets.byTarget.get(target);
    if (!candidates || candidates.size === 0) return;
    if (candidates.size === 1) {
      const approach = candidates.keys().next().value as number;
      commitMoveTargetApproach(target, approach);
      return;
    }
    const ap = pickApproachByCursor(target, cx, cy, candidates);
    if (ap !== null) {
      commitMoveTargetApproach(target, ap);
      return;
    }
    pendingApproach = {
      target,
      approaches: approachChoicesFor(moveTargets, target),
    };
  }

  function handleSquareClick(sq: number, cx: number, cy: number) {
    if (!interactive) return;
    // Player intervention cancels any in-flight Play-My-Moves drain (a
    // bodyguard-choice click is itself the intended interaction, so clearing
    // here is correct — the drain has already stopped and left the chooser up).
    if (playMyMovesQueue.length > 0) playMyMovesQueue = [];
    sfx.unlock();

    // Bodyguard chooser is active (engine has pending_bodyguard set): clicks
    // select defender (decline, idx=0) or an eligible Guard (idx=k+1). The
    // legal-action set is restricted to BodyguardChoice variants at this
    // point - submitting any other raw would fail anti-cheat in MP.
    if (pendingBodyguard) {
      if (sq === pendingBodyguard.targetSq) {
        sfx.play("click");
        applyRaw(encodeBodyguardChoice(0));
        return;
      }
      const k = pendingBodyguard.eligible.indexOf(sq);
      if (k >= 0) {
        sfx.play("click");
        applyRaw(encodeBodyguardChoice(k + 1));
        return;
      }
      // Click anywhere else: ignore. Defender must pick (decline or redirect).
      return;
    }

    if (pendingApproach) {
      if (pendingApproach.approaches.includes(sq)) {
        sfx.play("click");
        commitMoveTargetApproach(pendingApproach.target, sq);
        pendingApproach = null;
      } else {
        pendingApproach = null;
      }
      return;
    }

    // Armed skill: clicking a legal target fires it; clicking elsewhere
    // disarms but keeps the selection.
    if (armedSkill && armedSkill.square === match.selection) {
      // Two-stage ally pick: stage 1 records the ally; stage 2 (handled by
      // the standard target branch below) fires for that ally.
      if (armedNeedsAllyPick && focusAllyChosen === null) {
        if (armedAllyCandidates.has(sq)) {
          focusAllyChosen = sq;
          return;
        }
        // Click elsewhere: cancel ally pick + disarm.
        armedSkill = null;
        focusAllyChosen = null;
        return;
      }
      if (armedSkillTargets.has(sq)) {
        // Direction-pick skills (Shove) need a push-direction choice before
        // firing. The `needsDirectionPick` flag comes from engine metadata, so
        // no skill id is hardcoded here.
        if (skillById(armedSkill.skillId)?.needsDirectionPick === true) {
          openDirectionPicker(armedSkill.square, armedSkill.skillId, sq);
          return;
        }
        const raw = rawForArmedTarget(armedSkill.square, armedSkill.skillId, sq);
        if (raw !== null) {
          armedSkill = null;
          focusAllyChosen = null;
          applyRaw(raw);
          return;
        }
      }
      // Allow re-picking the ally mover when in ally-stage-2.
      if (armedNeedsAllyPick && focusAllyChosen !== null && armedAllyCandidates.has(sq)) {
        focusAllyChosen = sq;
        return;
      }
      armedSkill = null;
      focusAllyChosen = null;
      return;
    }

    if (match.selection !== null) {
      if (moveTargets.squares.has(sq)) {
        tryCommitMoveTo(sq, cx, cy);
        return;
      }
    }

    if (match.selection === sq) {
      match.selection = null;
      sfx.play("drop");
      return;
    }

    if (selectable.has(sq)) {
      match.selection = sq;
      sfx.play("pickup");
      return;
    }

    match.selection = null;
  }

  // Map the drop to a concrete raw action:
  // 1) Find candidate Move actions whose target equals the drop square.
  // 2) Pick the approach via cursor sub-tile position (same primitive as
  //    `tryCommitMoveTo` and `dragLanding`).
  // 3) Route through `commitMoveTargetApproach` so Bodyguard variants get
  //    surfaced as a defender chooser before the action applies.
  function handlePieceDrop(src: number, path: number[], cx: number, cy: number) {
    if (!interactive) return;
    // When the dropped piece was already the selection, the `moveTargets`
    // $derived already holds moveTargetsFor(match.legal, src); reuse it. When
    // it wasn't, set the selection and compute directly - a $derived read in
    // the same synchronous tick would still see the stale pre-assignment value.
    let targets;
    if (match.selection === src) {
      targets = moveTargets;
    } else {
      match.selection = src;
      targets = moveTargetsFor(match.legal, src);
    }
    const dropSq = path[path.length - 1];
    const candidates = targets.byTarget.get(dropSq);
    if (!candidates || candidates.size === 0) {
      // Dropped on illegal tile (or back on src) - soft drop thud.
      sfx.play("drop");
      return;
    }
    if (candidates.size === 1) {
      const approach = candidates.keys().next().value as number;
      commitMoveTargetApproach(dropSq, approach);
      return;
    }
    const ap = pickApproachByCursor(dropSq, cx, cy, candidates);
    if (ap !== null) {
      commitMoveTargetApproach(dropSq, ap);
      return;
    }
    pendingApproach = {
      target: dropSq,
      approaches: [...candidates.keys()].sort((a, b) => a - b),
    };
  }

  function endPhase() {
    if (endPhaseAction !== null) applyRaw(endPhaseAction);
  }

  // --- Resign / Draw ----------------------------------------------------------

  /** Which seat (0=P1, 1=P2) is currently moving a piece locally.
   *  In HvAI the human could be either seat; in MP it's localSeat. */
  function localHumanSeat(): 0 | 1 {
    if (match.mode === "multiplayer") return match.localSeat ?? 0;
    // HvH: the side currently to move is the "active" human.
    return (match.position?.toMove ?? 0) as 0 | 1;
  }

  async function confirmResign(): Promise<void> {
    if (!eng) return;
    showResignConfirm = false;
    const seat = localHumanSeat();
    // In MP, tell the peer first so their game terminates too (resign is
    // unilateral — no ack needed). Then finalise locally.
    if (match.mode === "multiplayer") mpEngine?.sendResign(seat);
    await resignGame(eng, seat);
  }

  async function offerDraw(): Promise<void> {
    if (!eng) return;
    if (match.mode === "multiplayer") {
      // In MP: send the offer to the peer; wait for their draw-response callback.
      mpEngine?.sendDrawOffer();
      showDrawOfferConfirm = false;
      return;
    }
    if (match.mode === "hvai") {
      showDrawOfferConfirm = false;
      const aiSeat: 0 | 1 = match.side.p1 === "ai" ? 0 : 1;
      const aiAccepts = await eng.evaluateDrawOffer(aiSeat);
      if (aiAccepts) {
        await agreeDrawGame(eng);
      } else {
        toast = "The AI declines the draw offer.";
        if (toastTimer) clearTimeout(toastTimer);
        toastTimer = setTimeout(() => { toast = ""; }, 3000);
      }
      return;
    }
    // HvH: both players present — confirm dialog is sufficient, treat confirm as both agreeing.
    showDrawOfferConfirm = false;
    await agreeDrawGame(eng);
  }

  async function respondToDrawOffer(accepted: boolean): Promise<void> {
    showIncomingDrawOffer = false;
    mpEngine?.sendDrawResponse(accepted);
    if (accepted && eng) {
      await agreeDrawGame(eng);
    }
  }

  function handleWheelSliceClick(slice: import("$lib/board/SkillWheel.svelte").SliceKind) {
    if (!interactive) return;
    if (!wheelOpen) return;
    sfx.unlock();
    sfx.play("click");
    const src = wheelOpen.square;

    if (slice.kind === "skill") {
      // Self-cast skills normally fire immediately - BUT when Focus is staged
      // and the engine emitted retarget variants (Shield → adjacent ally,
      // Dash/Retreat → adjacent ally), we arm instead so the player can pick
      // a recipient (or click self to take the self-cast).
      if (isSelfCast(slice.skillId)) {
        const retargetable = hasRetargetVariants(match.legal, src, slice.skillId);
        if (!retargetable) {
          const raw = rawForSelfCast(match.legal, src, slice.skillId);
          if (raw !== null) {
            armedSkill = null;
            applyRaw(raw);
          }
          return;
        }
        // Fall through to arm - the target picker will surface src + ally
        // recipients as legal targets.
      }
      // Otherwise arm it (or disarm if already armed with the same skill).
      if (armedSkill && armedSkill.skillId === slice.skillId) {
        armedSkill = null;
        focusAllyChosen = null;
      } else {
        armedSkill = { square: src, skillId: slice.skillId };
        focusRetargetPref = "self";
        focusAllyChosen = null;
      }
      return;
    }

    // `modifierBadge` is hover-only - clicking it is a no-op. Focus / Charge
    // are cast as regular skills via the piece's skill slot, not from the
    // wheel directly.
    if (slice.kind === "modifierBadge") {
      return;
    }

    if (slice.kind === "focusBoost") {
      // A quarter of a split (focus-eligible) skill. Set the chosen variant AND
      // arm that slot's skill so the next click is the target (per user: "arm
      // the skill with that mode"). focusMode skills (Blast/Shove) set
      // focusModePref; retarget skills (Shield/Dash/Retreat) set focusRetargetPref.
      const skillId = slice.skillId;
      if (skillId <= 0) return;
      if (slice.variant === "activation" || slice.variant === "effect") {
        focusModePref = slice.variant;
      } else {
        focusRetargetPref = slice.variant; // "self" | "ally"
      }
      // Self-cast quarter of a retargetable self-skill with no ambiguity fires
      // immediately (mirrors the plain-skill self-cast path); otherwise arm.
      if (slice.variant === "self" && isSelfCast(skillId)
          && !hasRetargetVariants(match.legal, src, skillId)) {
        const raw = rawForSelfCast(match.legal, src, skillId);
        if (raw !== null) { armedSkill = null; applyRaw(raw); return; }
      }
      armedSkill = { square: src, skillId };
      focusAllyChosen = null;
      return;
    }
  }

  function handleWheelSliceHover(slice: import("$lib/board/SkillWheel.svelte").SliceKind | null) {
    hoveredSlice = slice;
  }

  // Pick the raw u32 for an armed skill firing at `target`. The engine emits
  // one variant per legal (caster, skill, target, choice_idx, focus_mode);
  // for skills with a Focus-mode choice we filter to the player's preference,
  // otherwise we just return the first matching variant. Direction skills
  // (Shove) shouldn't go through this path - they hit the DirectionPicker
  // first - but we fall through to "any matching" for safety.
  function rawForArmedTarget(src: number, skillId: number, target: number): number | null {
    // Two-stage ally pick: caller has already set focusAllyChosen, target is
    // the destination for that ally.
    if (armedNeedsAllyPick && focusAllyChosen !== null) {
      const focusMode = armedHasFocusModeChoice ? (focusModePref === "effect") : null;
      return rawForAllyMove(match.legal, src, skillId, focusAllyChosen, target, focusMode);
    }
    const ts = skillTargetsFor(match.legal, src, skillId);
    const variants = ts.variantsByTarget.get(target);
    if (!variants || variants.length === 0) return null;
    const hasFocusChoice = hasFocusModeChoice(match.legal, src, skillId);
    const hasRetargetChoice = hasSelfAndRetargetChoice(match.legal, src, skillId);
    const wantEffect = focusModePref === "effect";
    const wantSelf = focusRetargetPref === "self";
    const v = variants.find((x) => {
      if (hasFocusChoice && x.focusMode !== wantEffect) return false;
      if (hasRetargetChoice && variantIsSelfCast(x, src) !== wantSelf) return false;
      return true;
    });
    if (v) return v.raw;
    return variants[0].raw;
  }

  /** Build a variant list for the Shove (or generally direction-skill) target
   *  square and open the direction picker. Filters by focus-mode preference
   *  when the (src, skill) has a Focus-mode choice. Fires immediately when only
   *  one direction is legal; otherwise the player MUST pick the push direction
   *  explicitly via the arrow overlay — we never auto-resolve from the resting
   *  cursor position here, because this is a click-to-fire path (not a drag
   *  gesture), so the cursor does not express an intended direction. */
  function openDirectionPicker(src: number, skillId: number, target: number) {
    const ts = skillTargetsFor(match.legal, src, skillId);
    let variants = ts.variantsByTarget.get(target) ?? [];
    if (hasFocusModeChoice(match.legal, src, skillId)) {
      const wantEffect = focusModePref === "effect";
      variants = variants.filter((v) => v.focusMode === wantEffect);
    }
    if (hasSelfAndRetargetChoice(match.legal, src, skillId)) {
      const wantSelf = focusRetargetPref === "self";
      variants = variants.filter((v) => variantIsSelfCast(v, src) === wantSelf);
    }
    if (variants.length === 0) return;
    // Only one legal push direction — no choice to make, fire it.
    if (variants.length === 1) {
      applyRaw(variants[0].raw);
      return;
    }
    // Ambiguous: always surface the arrow overlay so the player chooses.
    pendingDirection = { target, variants };
  }

  function handleKeyDown(ev: KeyboardEvent) {
    if (ev.key === "Escape") {
      if (pendingDirection) {
        pendingDirection = null;
        ev.preventDefault();
      } else if (pendingApproach) {
        pendingApproach = null;
        ev.preventDefault();
      } else if (armedSkill) {
        armedSkill = null;
        ev.preventDefault();
      } else if (match.selection !== null) {
        match.selection = null;
        ev.preventDefault();
      }
    }
  }

  function handleDirectionPick(raw: number) {
    pendingDirection = null;
    armedSkill = null;
    applyRaw(raw);
  }

  function handleDirectionCancel() {
    pendingDirection = null;
    // Keep `armedSkill` so the player can pick a different target without
    // re-arming the skill.
  }

  // === Export / Sandbox ====================================================

  function showToast(msg: string): void {
    toast = msg;
    if (toastTimer !== null) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toast = ""; toastTimer = null; }, 2200);
  }

  async function refreshMatchLogAvailable(): Promise<void> {
    if (!eng) { matchLogAvailable = false; return; }
    try {
      const log = await eng.matchLogJson();
      matchLogAvailable = log !== null;
    } catch {
      matchLogAvailable = false;
    }
  }

  async function copyFen(): Promise<void> {
    if (!eng) return;
    sfx.play("click");
    try {
      const fen = await eng.positionFen();
      await navigator.clipboard.writeText(fen);
      showToast(t("toast.fenCopied"));
    } catch {
      showToast(t("toast.clipboardBlocked"));
    }
  }

  async function copyMatchLog(): Promise<void> {
    if (!eng) return;
    sfx.play("click");
    try {
      const log = await eng.matchLogJson();
      if (log === null) { showToast(t("toast.logUnavailable")); return; }
      await navigator.clipboard.writeText(log);
      showToast(t("toast.logCopied"));
    } catch {
      showToast(t("toast.clipboardBlocked"));
    }
  }

  async function downloadMatchLog(): Promise<void> {
    if (!eng) return;
    sfx.play("click");
    try {
      const log = await eng.matchLogJson();
      if (log === null) { showToast(t("toast.logUnavailable")); return; }
      const blob = new Blob([log], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      const now = new Date();
      const pad = (n: number) => String(n).padStart(2, "0");
      const stamp = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
      a.download = `match-${stamp}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showToast(t("toast.logDownloaded"));
    } catch {
      showToast(t("toast.clipboardBlocked"));
    }
  }

  /** Reset local picker / armed-skill state. Used on sandbox toggle so a
   *  half-armed action doesn't bleed across the mode boundary. Bodyguard
   *  chooser state is engine-owned now (Position.pending_bodyguard) so it
   *  isn't reset here - snapshot restore / engine swap handles it. */
  function clearAllPickers(): void {
    match.selection = null;
    pendingApproach = null;
    armedSkill = null;
    focusAllyChosen = null;
    pendingDirection = null;
    focusModePref = "activation";
  }

  /** Pull engine state into the reactive `match` store after a snapshot
   *  restore. Mirrors the shape of inspector/+page.svelte:syncEngineToNode. */
  async function syncFromEngine(): Promise<void> {
    if (!eng || !renderer) return;
    await renderer.resyncFromEngine();
    await refreshMatchLogAvailable();
  }

  async function enterSandbox(): Promise<void> {
    if (!eng || busy || aiSearch.anyThinking) return;
    if (match.mode === "sandbox") return;
    busy = true;
    sfx.play("sandboxEnter");
    try {
      // Capture pre-sandbox snapshot BEFORE flipping mode - otherwise an
      // in-flight AI scheduler tick could mutate state between capture and
      // mode-flip.
      const snap = await eng.snapshotJson();
      clearAllPickers();
      // Sandbox is a purely local fork - the underlying match (including MP
      // transport and telemetry) continues to exist. We do NOT abandon the
      // telemetry session on entry: the exploratory plies never touch the
      // engine's true state (they roll back on exit), so nothing spurious is
      // logged, and keeping the telemetry row live means a subsequent
      // leave-during-sandbox still fires markNetworkLost / markAbandoned and
      // surfaces the "you left a game" card in the lobby.
      match.trueSnapshotJson = snap;
      match.sandboxMovesApplied = 0;
      sandboxUndoStack = [];
    sandboxRedoStack = [];
      match.preSandboxMode = match.mode;
      match.mode = "sandbox";
    } finally {
      busy = false;
    }
  }

  async function undoSandbox(): Promise<void> {
    if (!eng || busy || sandboxUndoStack.length === 0) return;
    busy = true;
    try {
      const currentSnap = await eng.snapshotJson();
      const snap = sandboxUndoStack[sandboxUndoStack.length - 1];
      sandboxUndoStack = sandboxUndoStack.slice(0, -1);
      sandboxRedoStack = [...sandboxRedoStack, currentSnap];
      await eng.restoreFromSnapshot(snap);
      match.sandboxMovesApplied = Math.max(0, match.sandboxMovesApplied - 1);
      clearAllPickers();
      await syncFromEngine();
    } finally {
      busy = false;
    }
  }

  async function redoSandbox(): Promise<void> {
    if (!eng || busy || sandboxRedoStack.length === 0) return;
    busy = true;
    try {
      const currentSnap = await eng.snapshotJson();
      const snap = sandboxRedoStack[sandboxRedoStack.length - 1];
      sandboxRedoStack = sandboxRedoStack.slice(0, -1);
      sandboxUndoStack = [...sandboxUndoStack, currentSnap];
      await eng.restoreFromSnapshot(snap);
      match.sandboxMovesApplied += 1;
      clearAllPickers();
      await syncFromEngine();
    } finally {
      busy = false;
    }
  }

  /** "Play My Moves": collect notation for sandbox actions, show confirm dialog. */
  async function openPlayMyMoves(): Promise<void> {
    if (!eng || match.sandboxMovesApplied === 0) return;
    // Gather notation from the undo stack snapshots — use action log entries
    // added during sandbox (they're already there from afterApplied, but we
    // blocked sandbox mode earlier). Instead, just show the count as confirmation.
    const count = match.sandboxMovesApplied;
    playMyMovesConfirm = Array.from({ length: count }, (_, i) => `Move ${i + 1}`);
  }

  /** Tear down the preview sibling engine + renderer and return to the live
   *  (free-running) board. Idempotent. Never mutates the live engine — but it
   *  DOES resync the live RENDERER to the true current engine state, which
   *  clears the effect queue that accumulated (undrained) while its EffectsLayer
   *  was unmounted during preview. Without that, returning to present would
   *  replay a burst of stale hit/heal pulses from every move made while you were
   *  looking at the past. */
  async function teardownPreview(): Promise<void> {
    const wasPreviewing = previewPly !== null;
    previewPly = null;
    previewPosition = null;
    leftAtPly = null;
    const r = previewRenderer;
    const pe = previewEng;
    previewRenderer = null;
    previewEng = null;
    try { r?.dispose(); } catch { /* best effort */ }
    // dispose() drops the Rust registry handle so the preview engine doesn't leak.
    try { await pe?.dispose(); } catch { /* best effort */ }
    // Repaint the live board from the true present, dropping the stale effect
    // backlog. resyncFromEngine reads (never writes) the live engine.
    if (wasPreviewing && renderer) {
      try { await renderer.resyncFromEngine(); } catch { /* best effort */ }
    }
  }

  /** Enter/refresh/exit read-only time-travel preview (P3-E).
   *
   *  CORRECTNESS: this runs entirely on a SEPARATE engine handle. It never calls
   *  the live `eng` except for a read-only `matchLogJson()` to source the line.
   *  The live game therefore keeps running underneath — AI keeps moving, MP peer
   *  moves keep arriving, telemetry keeps recording — and the preview stays
   *  FROZEN on the chosen ply (we only re-read the log when the user explicitly
   *  clicks a ply, so a live move landing does not move the preview). The board
   *  simply renders the preview source while `previewing`, so new live moves
   *  advance off-screen. Deliberately NOT gated on `busy`. */
  async function selectPreviewPly(plyIndex: number | null): Promise<void> {
    if (!eng) return;
    if (match.mode === "sandbox") return;
    if (plyIndex === null) {
      await teardownPreview();
      return;
    }
    try {
      // Source the true line from the live engine's match log (read-only).
      const logJson = await eng.matchLogJson();
      if (!logJson) return;
      const fullSnap = snapshotJsonFromMatchLog(logJson);
      if (fullSnap === null) return;
      // Zero the actions so the base is the START position; fastForwardTo then
      // silently replays plies 0..target-1 on the preview engine (replay pattern).
      const parsed = JSON.parse(fullSnap);
      const rawPlies: number[] = Array.isArray(parsed.actions)
        ? parsed.actions.map((a: number) => a >>> 0)
        : [];
      parsed.actions = [];
      const startSnap = JSON.stringify(parsed);
      const target = Math.max(0, Math.min(rawPlies.length, plyIndex | 0));

      // Lazily create the isolated preview engine + silent renderer.
      if (!previewEng) {
        previewEng = new TauriClient();
        previewRenderer = createPlyRenderer(previewEng, {
          sfxEnabled: false,
          onStateUpdate: (pos) => { previewPosition = pos; },
        });
      }
      // Capture the live head at the moment we FIRST enter preview, pinned so
      // the "left here" marker stays put as new live moves append. Re-clicking a
      // different past ply while already previewing must NOT move it.
      if (previewPly === null) {
        leftAtPly = actionLogEntries.length;
      }
      await previewEng.restoreFromSnapshot(startSnap);
      await previewRenderer!.fastForwardTo(startSnap, rawPlies, target);
      previewPly = plyIndex;
    } catch {
      // A malformed log or engine hiccup: bail out of preview cleanly rather
      // than leaving a half-built frozen board.
      await teardownPreview();
    }
  }


  async function confirmPlayMyMoves(): Promise<void> {
    if (!eng || !match.trueSnapshotJson) return;
    playMyMovesConfirm = null;
    playMyMovesNotice = null;
    // Gather the staged sandbox actions from the sandbox match log, restore the
    // real line, then load them into a queue drained through the normal apply
    // path. Draining (not a blind loop) means each ply obeys the same
    // seat/turn/bodyguard gating a live player would — so we never force an
    // AI-owned ply into the engine (the old NotAiTurn bug).
    try {
      const trueSnap = match.trueSnapshotJson;
      const logJson = await eng.matchLogJson();
      if (!logJson) return;
      const log = JSON.parse(logJson) as { plies?: Array<{ action?: { raw?: number } }> };
      // The match-log ply's `action` is an ActionDecoded object; the raw u32 we
      // re-apply lives at `action.raw`.
      const raws = (log.plies ?? [])
        .slice(-(match.sandboxMovesApplied))
        .map((p) => p.action?.raw)
        .filter((a): a is number => typeof a === "number");
      // Load the queue BEFORE any mode flip / await below. Setting mode out of
      // sandbox re-enables the auto-end-phase and HvAI scheduler effects; both
      // are gated on `playMyMovesQueue.length > 0`, so the queue must already be
      // non-empty when those effects flush (during the awaits) or they would
      // race the drain — the auto-ender would fire an EndPhase and consume the
      // turn before a single staged ply is applied (the observed bug).
      playMyMovesQueue = raws;
      // Restore to true line + leave sandbox mode.
      await eng.restoreFromSnapshot(trueSnap);
      const restoreMode = match.preSandboxMode ?? modeFromSeats(match.side);
      match.trueSnapshotJson = null;
      match.preSandboxMode = null;
      match.mode = restoreMode;
      sandboxUndoStack = [];
      sandboxRedoStack = [];
      match.sandboxMovesApplied = 0;
      clearAllPickers();
      await syncFromEngine();
      // Kick the driver.
      void drainPlayMyMoves();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
      playMyMovesQueue = [];
    }
  }

  /** Drain `playMyMovesQueue` one ply at a time through the normal apply path.
   *  Stops (leaving the remaining queue cleared) when:
   *   - the queue is emptied (all committed),
   *   - the next ply's turn belongs to a non-local / AI seat — we hand off to
   *     the live flow (the AI scheduler resumes once the queue is empty) rather
   *     than force-applying the sandbox's guess for that seat,
   *   - a Bodyguard choice is pending — the decision belongs to the user,
   *   - the user intervened (queue cleared elsewhere) or an apply failed. */
  async function drainPlayMyMoves(): Promise<void> {
    if (playMyMovesPlaying) return; // already draining
    playMyMovesPlaying = true;
    const delayMs = Math.max(0, settings.aivaiStepDelayMs);
    let busyWaits = 0;
    try {
      while (playMyMovesQueue.length > 0) {
        // Wait out any transient in-flight apply rather than giving up (an
        // effect firing on the mode flip could momentarily hold `busy`). Bounded
        // so a stuck `busy` can't spin forever.
        if (busy) {
          if (busyWaits++ > 100) break;
          await new Promise<void>((r) => setTimeout(r, 20));
          continue;
        }
        busyWaits = 0;
        // A pending bodyguard means the previous ply triggered an intercept
        // choice; that decision is the user's. Stop and surface a notice.
        if (match.position?.pendingBodyguard != null) {
          playMyMovesNotice = "Playback paused: a Bodyguard choice is required. Resolve it to continue.";
          break;
        }
        // Only commit plies for the local human's own turn. When the turn has
        // handed to the AI (or, in MP, the peer), stop: the staged guess for
        // that seat is not authoritative, and force-applying it would desync /
        // error. Clearing the queue lets the AI scheduler take over naturally.
        const toMove = match.position?.toMove ?? 0;
        const seat = toMove === 0 ? match.side.p1 : match.side.p2;
        const isLocalHumanTurn =
          seat === "human" && (match.mode !== "multiplayer" || currentSeatIsLocal);
        if (!isLocalHumanTurn) break;

        const raw = playMyMovesQueue[0];
        await applyRaw(raw);
        // applyRaw refused / the queue was mutated by user intervention → stop.
        if (playMyMovesQueue[0] !== raw) break;
        playMyMovesQueue = playMyMovesQueue.slice(1);
        if (playMyMovesQueue.length > 0 && delayMs > 0) {
          await new Promise<void>((r) => setTimeout(r, delayMs));
        }
      }
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      playMyMovesQueue = [];
      playMyMovesPlaying = false;
    }
  }

  /** Restore the engine to the true authoritative line captured at sandbox
   *  entry and clear all sandbox carrier state. This is the shared core of
   *  leaving sandbox - it does NOT own `busy`, the confirm-discard dialog, or
   *  sfx; each caller layers those on:
   *    - `exitSandbox` (user-initiated): confirm dialog + busy + sfx, then this.
   *    - `ensureLiveEngine` (auto, ns-37): no dialog - an incoming opponent
   *      move can't wait on a modal and the exploration is discarded by design.
   *  No-op when not in sandbox (idempotent: flips `match.mode` out of sandbox),
   *  so a double-call (e.g. two wire messages back-to-back) is safe. */
  async function restoreTrueLineFromSandbox(): Promise<void> {
    if (!eng) return;
    if (match.mode !== "sandbox" || !match.trueSnapshotJson) return;
    validateSnapshot(match.trueSnapshotJson, {
      maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
      maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
      requireConfig: true,
      source: "sandbox-restore",
    });
    await eng.restoreFromSnapshot(match.trueSnapshotJson);
    match.trueSnapshotJson = null;
    match.sandboxMovesApplied = 0;
    sandboxUndoStack = [];
    sandboxRedoStack = [];
    // Restore the mode we entered sandbox from. `modeFromSeats()` can't
    // round-trip "multiplayer" (both seats are "human" in MP too), so we
    // rely on the stashed value; fall back to seat-derivation only if
    // preSandboxMode was somehow lost.
    match.mode = match.preSandboxMode ?? modeFromSeats(match.side);
    match.preSandboxMode = null;
    clearAllPickers();
    await syncFromEngine();
  }

  async function exitSandbox(): Promise<void> {
    if (!eng || busy) return;
    if (match.mode !== "sandbox" || !match.trueSnapshotJson) return;
    if (match.sandboxMovesApplied > 0) {
      const msg = t("sandbox.confirmDiscard", { n: match.sandboxMovesApplied });
      if (!await sandboxConfirm(msg)) return;
    }
    sfx.play("click");
    busy = true;
    try {
      await restoreTrueLineFromSandbox();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  /** Wire-path hook (ns-37): before the mp wrapper validates/applies an
   *  incoming opponent action on the shared engine, guarantee we're on the
   *  true authoritative line. If this peer is mid-sandbox, auto-exit it now -
   *  otherwise the wrapper would tryApply the opponent's real move against our
   *  sandbox-forked state and falsely cry "engine disagreed". Skips the
   *  confirm-discard dialog on purpose. Runs outside applyRaw's `busy` window
   *  (fired from the wire handler), so it does not touch `busy`. */
  async function ensureLiveEngineOnTrueLine(): Promise<void> {
    if (match.mode !== "sandbox") return;
    await restoreTrueLineFromSandbox();
  }

  // Watch for natural game-end and finalise the telemetry session exactly
  // once. The idempotency flag lives on the match carrier so re-renders,
  // HMR, and claim-win double-clicks all converge on a single finalise.
  $effect(() => {
    const gr = match.position?.gameResult ?? 0;
    if (gr === 0) return;
    if (match.telemetryFinalised) return;
    if (!match.telemetryMatchId) return;
    if (match.mode === "sandbox") return;
    match.telemetryFinalised = true;
    const resultByte: 0 | 1 | 2 | 3 = gr === 1 ? 0 : gr === 2 ? 1 : 2;
    const localEng = eng;
    if (!localEng) return;
    (async () => {
      if (match.mode === "aivai") {
        // AIvAI: the background producer already finalised ITS log (the
        // authoritative one). The view engine's log is a replay we never
        // persist, so we neither finaliseLog nor read the view engine here —
        // we persist the producer's log. By the time the view has played
        // through to the game-over ply, the producer has long finished, so its
        // published log is complete.
        let producerLog: string | null = null;
        try {
          producerLog = await aivaiProducerLog(localEng);
        } catch (e) {
          console.warn("[aivai] producer log read on finalise failed:", e);
        }
        await finalizeTelemetrySession(localEng, "checkmate", resultByte, producerLog);
        return;
      }
      try {
        await localEng.finaliseLog(resultByte);
      } catch (e) {
        // Already logged inside the helper; do not block.
        console.warn("[telemetry] eng.finaliseLog failed:", e);
      }
      await finalizeTelemetrySession(localEng, "checkmate", resultByte);
    })();
  });

  // Multiplayer: prompt before tab close while a match is in progress. The
  // browser ignores the returned string in modern Chromium/Firefox but still
  // shows its built-in confirmation dialog. The listener is gated on
  // `match.mode` so local matches don't prompt on close. We mount it once
  // and let the guard read the current `match` state at fire time.
  function beforeUnloadGuard(e: BeforeUnloadEvent): string | undefined {
    if (match.mode !== "multiplayer") return undefined;
    if (match.telemetryFinalised) return undefined;
    const msg = t("multiplayer.beforeUnloadPrompt");
    e.preventDefault();
    // Some browsers still read the returnValue assignment.
    (e as BeforeUnloadEvent & { returnValue: string }).returnValue = msg;
    return msg;
  }



  onMount(() => {
    if (typeof window !== "undefined") {
      window.addEventListener("beforeunload", beforeUnloadGuard);
    }
  });

  onDestroy(() => {
    // eslint-disable-next-line no-console
    console.log("[match] onDestroy", { mode: match.mode, finalised: match.telemetryFinalised, ownershipToken, currentToken: getRouteOwnershipToken(), stack: new Error().stack?.split("\n").slice(1,5).join(" | ") });
    if (typeof window !== "undefined") {
      window.removeEventListener("beforeunload", beforeUnloadGuard);
    }
    // Local matches (HvH / HvAI / AIvAI / sandbox): if the player leaves before
    // the game finishes naturally, transition the telemetry row from
    // in-progress → abandoned so the Library stops treating it as live. MP
    // uses markNetworkLost via a separate reactive effect. markAbandoned is
    // idempotent (only flips when status === "in-progress"), so a finalized
    // row is unaffected.
    // Gate off role rather than `match.mode` so that leaving from inside
    // sandbox during an MP match still routes to the MP teardown below.
    if (multiplayerRole() === null
        && match.telemetryMatchId
        && !match.telemetryFinalised) {
      const id = match.telemetryMatchId;
      const engRef = eng;
      const isAivai = match.mode === "aivai";
      // AIvAI: stop the background producer FIRST (abort + join the thread —
      // bounded by one ply's think-time), then persist ITS finalised log. This
      // is what guarantees the saved log length equals exactly what the
      // producer computed "behind closed doors", not what the view displayed.
      // Marking `leavingAivai` drives the "finishing current move…" HUD state.
      if (isAivai) leavingAivai = true;
      void (async () => {
        try {
          let partial: string | undefined;
          if (isAivai && engRef) {
            try {
              partial = (await stopAivaiProducer(engRef)) ?? undefined;
            } catch { /* producer stop failed; abandon without a log */ }
          } else if (engRef) {
            try { partial = (await engRef.matchLogJson()) ?? undefined; } catch { /* engine bad state */ }
          }
          await getTelemetryStore().markAbandoned(id, partial);
        } catch {
          // Swallow - telemetry must never block navigation.
        } finally {
          leavingAivai = false;
        }
      })();
    } else if (match.mode === "aivai" && eng) {
      // Finalised (natural end) or no telemetry row, but a producer may still
      // be alive (rare: user leaves on the exact game-over frame). Abort it so
      // no detached thread keeps running.
      const engRef = eng;
      void stopAivaiProducer(engRef).catch(() => { /* best-effort */ });
    }
    if (mpEngine) {
      mpEngine.dispose();
      mpEngine = null;
    }
    if (mpConnectedUnsub) {
      mpConnectedUnsub();
      mpConnectedUnsub = null;
    }
    if (backgroundEvalUnsub) {
      backgroundEvalUnsub();
      backgroundEvalUnsub = null;
    }
    if (aivaiProgressUnsub) {
      aivaiProgressUnsub();
      aivaiProgressUnsub = null;
    }
    // Leaving /match/ before a natural end means we're going back to the
    // lobby (or home). Soft-tear the transport so the peer sees the drop but
    // our carrier state (code, peerEverPaired, disconnectedSince) survives -
    // GraceBanner then stays visible when the user clicks Rejoin from the
    // lobby. On a natural end, hard disconnect together.
    // Gate off role rather than `match.mode` so leaving from inside sandbox
    // (mode === "sandbox") still tears down the MP session correctly.
    if (multiplayerRole() !== null) {
      tearDownMultiplayerOnLeave({
        navigatingForward: false,
        telemetryFinalised: match.telemetryFinalised ?? false,
        ownershipToken,
      });
    }
    // Renderer teardown last: cancels shake/deferred-skill timers and empties
    // effectQueue. inspector already does this - matching the pattern here so
    // long AIvAI sessions don't leak PlyRenderer instances across route churn.
    renderer?.dispose();
    renderer = null;
    // Preview sibling engine + renderer (P3-E time-travel): dispose so the
    // extra Rust registry handle doesn't outlive the route.
    void teardownPreview();
    // Clear AI-search transients so the next match doesn't inherit stale depth
    // / breakdown / linger state from a previous session.
    resetAiSearch();
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

<main>
  <header>
    <h1>{t("match.title", { mode })}</h1>
    {#if match.mode === "multiplayer"}
      <ConnectivityPill />
    {/if}
  </header>

  {#if match.mode === "multiplayer"}
    <MultiplayerStatusStrip
      waitingReason={isPeerTurn ? t("multiplayer.waitingForPeerMove", { n: peerSeatNumber }) : null}
      paused={mpPaused}
    />
    <GraceBanner
      {eng}
      {mpEngine}
      role={multiplayerRole()}
      code={multiplayerCode()}
      onClaim={claimWinByOpponentForfeit}
      onTakeOver={takeoverAsHost}
    />
  {/if}

  {#if bootError}
    <div class="err" role="alert">
      <span>{bootError}</span>
      <button type="button" class="err-dismiss" onclick={() => (bootError = null)} aria-label="dismiss">x</button>
    </div>
  {/if}

  {#if !ready}
    <p>{t("app.loading")}</p>
  {:else}
    <div class="game-area">
      <!-- Left column: P2 panel + board + P1 panel -->
      <div class="board-column" class:flipped={boardFlipped}>
        <PlayerPanel
          player="p2"
          position={match.position}
          aiMaxDepth={settings.p2MaxDepth}
          isAiSeat={p2IsAi}
          aiThinkBudgetMs={settings.p2ThinkTimeMs}
          isActive={match.position?.toMove === 1}
          roundNumber={match.position?.roundNumber ?? 1}
          {baselinePieces}
          pendingCost={match.position?.toMove === 1 ? pendingSkillCost : null}
        />

        <div class="board-stack" class:sandbox-mode={match.mode === "sandbox"}>
          <Board
            position={previewing ? previewPosition : match.position}
            pieceIds={previewing ? (previewRenderer?.pieceIds ?? new Map()) : (renderer?.pieceIds ?? new Map())}
            selection={previewing ? null : match.selection}
            moveTargets={previewing ? new Set() : (armedSkill ? armedSkillTargets : moveTargets.squares)}
            {selectable}
            draggable={movable}
            usedSquares={previewing ? previewMoved : movedSquares}
            shakingSquares={previewing ? (previewRenderer?.shakingSquares ?? new Set()) : (renderer?.shakingSquares ?? new Set())}
            pieceMotion={previewing ? (previewRenderer?.pieceMotion ?? new Map()) : (renderer?.pieceMotion ?? new Map())}
            toMove={previewing
              ? (previewPosition?.gameResult === 0 ? (previewPosition?.toMove ?? null) : null)
              : (match.position?.gameResult === 0 ? (match.position?.toMove ?? null) : null)}
            effectsActive={previewing ? false : (renderer?.effectQueue.length ?? 0) > 0}
            approachChoices={pendingApproach?.approaches ?? hoverApproachChoices}
            bodyguardChoice={pendingBodyguard ? {
              defender: pendingBodyguard.targetSq,
              guards: pendingBodyguard.eligible.slice(),
            } : null}
            lastApplied={previewing ? (previewRenderer?.lastApplied ?? null) : (renderer?.lastApplied ?? null)}
            {interactive}
            flipped={boardFlipped}
            wheelOpen={wheelOpen}
            armedSkillId={armedSkill?.skillId ?? null}
            {focusActive}
            {chargeActive}
            {wheelLegality}
            split1={wheelSplits.split1}
            split2={wheelSplits.split2}
            onWheelSliceClick={handleWheelSliceClick}
            onWheelSliceHover={handleWheelSliceHover}
            directionPicker={pendingDirection}
            onDirectionPick={handleDirectionPick}
            onDirectionCancel={handleDirectionCancel}
            onSquareClick={handleSquareClick}
            onPieceDrop={handlePieceDrop}
            onPressStart={handlePressStart}
            onDragMove={handleDragMove}
            dragTrail={dragTrailShown}
            {dragHover}
            {dragHoverLegal}
            dragLanding={effectiveLanding}
            clickHover={clickHoverTarget}
            onApproachChoice={(ap) => {
              if (pendingApproach) {
                const target = pendingApproach.target;
                pendingApproach = null;
                commitMoveTargetApproach(target, ap);
              }
            }}
            onSquareHover={(sq, x, y, sx, sy) => {
              hoveredSq = sq;
              hoverX = x;
              hoverY = y;
              // Click-mode hover (no drag): mirror the pointer into hoverSq +
              // the SVG cursor coords so clickLanding / hoverApproachChoices /
              // clickHoverTarget resolve the approach exactly like a drag would.
              // While dragging, handleDragMove owns these — don't stomp them.
              if (dragSrc === null) {
                hoverSq = sq;
                if (sq !== null) { cursorX = sx; cursorY = sy; }
              }
            }}
          />
          {#if previewing && previewRenderer}
            <EffectsLayer viewBox={800} wheelPad={60} queue={previewRenderer.effectQueue} flipped={boardFlipped} />
          {:else if renderer}
            <EffectsLayer viewBox={800} wheelPad={60} queue={renderer.effectQueue} flipped={boardFlipped} />
          {/if}
          {#if hoveredSlice && hoveredSliceVisible && wheelOpen}
            {@const wFile = wheelOpen.square & 7}
            {@const wRank = (wheelOpen.square >> 3) & 7}
            {@const colFrac = boardFlipped ? (7 - wFile + 0.5) / 8 : (wFile + 0.5) / 8}
            {@const rowFrac = boardFlipped ? (wRank + 0.5) / 8 : (7 - wRank + 0.5) / 8}
            {@const onLeftHalf = colFrac < 0.5}
            <!-- Anchor the tooltip beside the hovered piece: to its right when
                 the piece is on the left half of the board, to its left
                 otherwise, so the card never runs off-screen. Vertically
                 centred on the piece's row. -->
            <div
              class="info-anchor skill-info-fade"
              class:to-right={onLeftHalf}
              class:to-left={!onLeftHalf}
              style:left="{colFrac * 100}%"
              style:top="{rowFrac * 100}%"
            >
              <SkillInfoCard
                slice={hoveredSlice}
                {focusActive}
                {chargeActive}
                armed={hoveredSlice.kind === "skill"
                  && armedSkill?.skillId === hoveredSlice.skillId}
              />
            </div>
          {/if}
          <!-- Contextual cancel button: visible whenever a skill is mid-activation -->
          {#if armedSkill !== null && interactive}
            <button
              type="button"
              class="cancel-skill-btn"
              onclick={() => { armedSkill = null; pendingDirection = null; focusAllyChosen = null; }}
              aria-label="Cancel skill"
            >✕ Cancel</button>
          {/if}
        </div>

        <PlayerPanel
          player="p1"
          position={match.position}
          aiMaxDepth={settings.p1MaxDepth}
          isAiSeat={p1IsAi}
          aiThinkBudgetMs={settings.p1ThinkTimeMs}
          isActive={match.position?.toMove === 0}
          roundNumber={match.position?.roundNumber ?? 1}
          {baselinePieces}
          pendingCost={match.position?.toMove === 0 ? pendingSkillCost : null}
        />
      </div>

      <!-- Right column: status + controls + export + progression -->
      <div class="right-column">
      <aside class="right-panel">
        <!-- Status -->
        <div class="status-block">
          <div class="stat-row">
            <span class="stat-label">Round</span>
            <span class="stat-value">{match.position?.roundNumber ?? "-"}</span>
          </div>
          <!-- Phase indicator: two boxes, active one coloured -->
          <div class="phase-boxes">
            <div class="phase-box" class:active={inMovePhase} class:inactive={!inMovePhase}>
              <span class="phase-box-label">Move</span>
              <span class="phase-box-count" class:greyed={!inMovePhase}>
                {#if inMovePhase}
                  {match.position?.actionsRemaining ?? "-"}
                {:else}
                  {movePhaseBudget}
                {/if}
              </span>
            </div>
            <div class="phase-box" class:active={!inMovePhase} class:inactive={inMovePhase}>
              <span class="phase-box-label">Skill</span>
              <span class="phase-box-count" class:greyed={inMovePhase}>
                {inMovePhase ? skillPhaseBudget : (match.position?.actionsRemaining ?? "-")}
              </span>
            </div>
          </div>
        </div>

        <div class="panel-divider"></div>

        <!-- Primary action -->
        <div class="primary-actions">
          {#if match.position?.gameResult !== 0 || match.forcedResult}
            <p class="result">
              {#if match.forcedResult}
                {match.forcedResult.reason === "draw"
                  ? t("result.draw")
                  : match.forcedResult.resultByte === 0
                    ? t("result.p1Wins")
                    : t("result.p2Wins")}
                <span class="result-reason">
                  ({match.forcedResult.reason === "draw" ? "agreed draw" : "resignation"})
                </span>
              {:else}
                {match.position?.gameResult === 1
                  ? t("result.p1Wins")
                  : match.position?.gameResult === 2
                    ? t("result.p2Wins")
                    : t("result.draw")}
              {/if}
            </p>
          {:else if match.mode === "aivai"}
            <!-- AIvAI is a log player over the background producer (Change 6):
                 Play/Pause toggles the paced view loop; Step advances one ply
                 from the producer's already-computed log. "Playback complete"
                 is when the view has rendered every ply AND the producer has
                 finished the game. -->
            <button
              type="button"
              class="btn-primary"
              disabled={leavingAivai || (producerDone && viewPly >= producerRaws.length)}
              onclick={() => {
                if (busy && aiAutoPlay) {
                  pendingPause = !pendingPause;
                } else {
                  pendingPause = false;
                  aiAutoPlay = !aiAutoPlay;
                }
              }}
            >{pendingPause ? t("controls.pausing") : aiAutoPlay ? t("controls.pause") : t("controls.play")}</button>
            <button
              type="button"
              class="btn-secondary"
              disabled={busy || aiAutoPlay || leavingAivai || viewPly >= producerRaws.length}
              onclick={() => void advanceView()}
            >{t("controls.step")}</button>
          {:else}
            <button
              type="button"
              class="btn-primary"
              disabled={!interactive || endPhaseAction === null}
              onclick={endPhase}
            >{t("controls.endPhase")}</button>
          {/if}
        </div>

        <!-- Resign / Draw — shown during live human play only -->
        {#if interactive && !match.forcedResult && match.position?.gameResult === 0 && match.mode !== "aivai"}
          <div class="resign-draw-row">
            <button
              type="button"
              class="btn-danger-ghost"
              onclick={() => { showResignConfirm = true; }}
            >Resign</button>
            <button
              type="button"
              class="btn-ghost"
              onclick={() => { showDrawOfferConfirm = true; }}
            >Offer Draw</button>
          </div>
        {/if}

        <!-- Resign confirm dialog -->
        {#if showResignConfirm}
          <div class="inline-dialog">
            <p>Resign this game?</p>
            <div class="inline-dialog-btns">
              <button type="button" class="btn-danger" onclick={() => void confirmResign()}>Yes, resign</button>
              <button type="button" class="btn-ghost" onclick={() => { showResignConfirm = false; }}>Cancel</button>
            </div>
          </div>
        {/if}

        <!-- Draw offer confirm (local HvH / HvAI) -->
        {#if showDrawOfferConfirm}
          <div class="inline-dialog">
            <p>{match.mode === "hvh" ? "Both players agree to a draw?" : "Offer draw to AI?"}</p>
            <div class="inline-dialog-btns">
              <button type="button" class="btn-primary" onclick={() => void offerDraw()}>Confirm</button>
              <button type="button" class="btn-ghost" onclick={() => { showDrawOfferConfirm = false; }}>Cancel</button>
            </div>
          </div>
        {/if}

        <!-- Incoming draw offer (MP only) -->
        {#if showIncomingDrawOffer}
          <div class="inline-dialog">
            <p>Opponent offers a draw. Accept?</p>
            <div class="inline-dialog-btns">
              <button type="button" class="btn-primary" onclick={() => void respondToDrawOffer(true)}>Accept</button>
              <button type="button" class="btn-ghost" onclick={() => void respondToDrawOffer(false)}>Decline</button>
            </div>
          </div>
        {/if}

        <!-- Contextual hints + skill toggles -->
        {#if pendingApproach}
          <p class="hint">Choose the path the attacker takes - click a highlighted square, or press Esc to cancel</p>
        {/if}
        {#if pendingBodyguard}
          <p class="hint">Bodyguard: click the red defender to take the hit, or a blue guard to intercept</p>
        {/if}
        {#if armedNeedsAllyPick && focusAllyChosen === null}
          <p class="hint">Pick an adjacent ally to channel onto, then choose where they move</p>
        {/if}
        {#if armedNeedsAllyPick && focusAllyChosen !== null}
          <p class="hint">Choose the destination for the chosen ally - click another ally to switch</p>
        {/if}
        {#if pendingDirection}
          <p class="hint">Choose a push direction - click an arrow, or press Esc to cancel</p>
        {/if}

        <div class="panel-divider"></div>

        <!-- Sandbox / Play My Moves controls -->
        <div class="export-group">
          <button
            type="button"
            class="sandbox-toggle"
            disabled={busy || (match.mode !== "sandbox" && aiSearch.anyThinking)}
            onclick={() => void (match.mode === "sandbox" ? exitSandbox() : enterSandbox())}
          >{match.mode === "sandbox" ? t("controls.exitSandbox") : t("controls.sandbox")}</button>
          {#if match.mode === "sandbox"}
            <button
              type="button"
              disabled={busy || sandboxUndoStack.length === 0}
              onclick={() => void undoSandbox()}
            >{t("controls.undo")}</button>
            <button
              type="button"
              disabled={busy || sandboxRedoStack.length === 0}
              onclick={() => void redoSandbox()}
            >Redo</button>
            {#if match.sandboxMovesApplied > 0 && !playMyMovesPlaying}
              <button
                type="button"
                class="play-moves-btn"
                disabled={busy}
                onclick={() => void openPlayMyMoves()}
              >▶ Play moves</button>
            {/if}
            {#if playMyMovesPlaying}
              <button
                type="button"
                onclick={() => { playMyMovesQueue = []; }}
              >■ Stop</button>
            {/if}
          {/if}
        </div>

        {#if playMyMovesNotice !== null}
          <p class="play-moves-notice" role="status">{playMyMovesNotice}</p>
        {/if}

        <!-- Play My Moves confirm dialog -->
        {#if playMyMovesConfirm !== null}
          <div class="inline-dialog">
            <p>Commit {playMyMovesConfirm.length} sandbox move{playMyMovesConfirm.length === 1 ? '' : 's'} to the real game?</p>
            <div class="inline-dialog-btns">
              <button type="button" class="btn-primary" onclick={() => void confirmPlayMyMoves()}>Confirm</button>
              <button type="button" class="btn-ghost" onclick={() => { playMyMovesConfirm = null; }}>Cancel</button>
            </div>
          </div>
        {/if}
        {#if settings.showHeuristicEval && aiSearch.heuristicEvalBreakdown !== null && match.mode !== "multiplayer"}
          {@const evalScore = aiSearch.heuristicEvalBreakdown.total}
          <div class="eval-bar-row">
            <span class="eval-label">Eval</span>
            <span class="eval-score" class:positive={evalScore > 0} class:negative={evalScore < 0}>
              {evalScore > 0 ? '+' : ''}{evalScore}
            </span>
          </div>
          {#if aiSearch.backgroundEval}
            {@const be = aiSearch.backgroundEval}
            <div class="eval-bar-row engine-eval-row" title="Engine's time-bounded search read of the last move">
              <span class="eval-label">Engine</span>
              <span class="eval-score">
                {#if be.wasMate && be.mateIn !== null}
                  #{be.mateIn}
                {:else}
                  {(be.scoreCp ?? 0) > 0 ? '+' : ''}{be.scoreCp ?? 0}
                {/if}
                <span class="eval-depth">d{be.depth}</span>
              </span>
            </div>
          {/if}
        {/if}
      </aside>

      <!-- Progression panel: income + skill actions over rounds -->
      {#if match.position}
        <ProgressionPanel roundNumber={match.position.roundNumber} />
      {/if}

      <!-- Action log: move history + copy/download buttons -->
      <ActionLogPanel
        entries={actionLogEntries}
        {busy}
        {matchLogAvailable}
        selectedPly={previewPly}
        leftAtPly={leftAtPly}
        onCopyFen={() => void copyFen()}
        onCopyLog={() => void copyMatchLog()}
        onDownloadLog={() => void downloadMatchLog()}
        onSelectPly={(i) => void selectPreviewPly(i)}
      />
      </div>

      {#if settings.showEvalPanel && match.mode !== "multiplayer"}
        <div class="eval-column">
          <EvalBreakdownPanel />
        </div>
      {/if}
    </div>
  {/if}
</main>

{#if toast}
  <div class="toast" role="status" aria-live="polite">{toast}</div>
{/if}

{#if settings.showEvalPanel && match.mode !== "multiplayer" && hoveredSq !== null && aiSearch.heuristicEvalBySquare !== null}
  <SquareEvalCard
    data={aiSearch.heuristicEvalBySquare}
    sq={hoveredSq}
    clientX={hoverX}
    clientY={hoverY}
  />
{/if}

{#if sandboxConfirmMsg !== null}
  {@const openDialog = (el: HTMLDialogElement | null) => { if (el) el.showModal(); }}
  <dialog use:openDialog>
    <p>{sandboxConfirmMsg}</p>
    <div class="confirm-actions">
      <button type="button" onclick={() => { sandboxConfirmMsg = null; sandboxConfirmResolve?.(false); sandboxConfirmResolve = null; }}>Cancel</button>
      <button type="button" class="confirm-ok" onclick={() => { sandboxConfirmMsg = null; sandboxConfirmResolve?.(true); sandboxConfirmResolve = null; }}>Discard & exit</button>
    </div>
  </dialog>
{/if}

<style>
  main {
    padding: 0.5rem 0.8rem 1.5rem;
    position: relative;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.6rem;
  }
  header h1 {
    font-size: 1.2rem;
    margin: 0;
  }

  /* ── Game area: board column + right panel ──────────────────────────────── */
  .game-area {
    display: flex;
    gap: 0.8rem;
    align-items: flex-start;
  }

  .board-column {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    /* Board fills available viewport height.
       Deduct: main padding-top (8px) + header (~40px) + header margin (10px)
               + 2 player panels (~36px each) + gaps (8px) + bottom padding (24px) = ~162px.
       Use 170px to be safe; board is always a square so width = this height. */
    width: min(calc(100vw - 240px - 2rem), calc(100dvh - 170px));
    min-width: 280px;
  }
  /* Opponent-at-bottom view: swap the two PlayerPanels (P2 top / P1 bottom
     becomes P1 top / P2 bottom) so the local player's banner sits under the
     board, matching the 180°-rotated board. Pure visual reorder. */
  .board-column.flipped {
    flex-direction: column-reverse;
  }

  .board-stack {
    position: relative;
    width: 100%;
  }

  /* Sandbox: glow on the board only, not the whole page */
  .board-stack.sandbox-mode {
    box-shadow:
      0 0 0 3px rgba(56, 178, 255, 0.85),
      0 0 20px 6px rgba(56, 178, 255, 0.30);
    animation: sandbox-pulse 2.4s ease-in-out infinite;
    border-radius: 4px;
  }
  @keyframes sandbox-pulse {
    0%, 100% {
      box-shadow:
        0 0 0 3px rgba(56, 178, 255, 0.85),
        0 0 20px 6px rgba(56, 178, 255, 0.25);
    }
    50% {
      box-shadow:
        0 0 0 3px rgba(56, 178, 255, 1.00),
        0 0 28px 10px rgba(56, 178, 255, 0.45);
    }
  }

  .info-anchor {
    position: absolute;
    z-index: 5;
    pointer-events: none;
    /* left/top are set inline to the hovered piece's centre (% of board-stack).
       The transform places the card beside the piece, vertically centred, with
       a gap of ~9% of a tile (0.075 * 12.5%) so it clears the wheel ring. */
  }
  .info-anchor.to-right {
    /* piece on the left half → card to the right of the piece */
    transform: translate(calc(6.25% + 0.9rem), -50%);
  }
  .info-anchor.to-left {
    /* piece on the right half → card to the left of the piece */
    transform: translate(calc(-100% - 6.25% - 0.9rem), -50%);
  }
  .cancel-skill-btn {
    position: absolute;
    bottom: 0.5rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
    background: var(--paper-bg, #f3ecd9);
    border: 1.5px solid #c0392b;
    color: #c0392b;
    border-radius: 6px;
    padding: 0.3em 0.8em;
    font: inherit;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    pointer-events: auto;
    transition: background 80ms;
  }
  .cancel-skill-btn:hover { background: #c0392b0f; }
  .play-moves-btn {
    background: color-mix(in srgb, var(--accent, #c79b3a) 15%, var(--paper-bg));
    border-color: var(--accent, #c79b3a);
    color: var(--accent, #c79b3a);
    font-weight: 600;
  }
  .play-moves-notice {
    margin: 0.4rem 0 0;
    padding: 0.4rem 0.55rem;
    font-size: 0.78rem;
    line-height: 1.35;
    color: var(--paper-ink, #3a2f1f);
    background: color-mix(in srgb, var(--accent, #c79b3a) 12%, var(--paper-bg));
    border: 1px solid var(--accent, #c79b3a);
    border-radius: 5px;
  }
  .skill-info-fade {
    animation: skill-info-fadein 180ms ease-out both;
  }
  @keyframes skill-info-fadein {
    from { opacity: 0; transform: translateX(-4px); }
    to   { opacity: 1; transform: translateX(0); }
  }

  /* ── Right panel ─────────────────────────────────────────────────────────── */
  .right-column {
    flex: 0 0 200px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .eval-column {
    flex: 1 1 320px;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    min-height: 0;
    /* Match the board's height budget so the panel fills vertically as well.
       Same formula as .board-column's height cap so the fill bar's track can
       run the full board height on typical viewports. */
    max-height: calc(100dvh - 170px);
  }
  .eval-column :global(.eval-panel) {
    flex: 1 1 auto;
    height: 100%;
  }

  .right-panel {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.6rem 0.7rem;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
    min-height: 0;
  }

  .panel-divider {
    height: 1px;
    background: var(--paper-line, rgba(58,47,31,0.15));
    margin: 0.1rem 0;
  }

  /* Status block */
  .status-block {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .stat-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }
  .stat-label {
    font-size: 0.72rem;
    color: var(--paper-ink-soft, #6a6055);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .stat-value {
    font-weight: 600;
    font-size: 0.95rem;
    font-variant-numeric: tabular-nums;
  }
  /* Phase indicator boxes */
  .phase-boxes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.4rem;
    margin-top: 0.4rem;
  }
  .phase-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.1rem;
    padding: 0.35em 0.4em;
    border-radius: 6px;
    border: 1.5px solid var(--paper-line, rgba(58,47,31,0.15));
    background: var(--paper-bg);
    transition: border-color 120ms, background 120ms;
  }
  .phase-box.active {
    border-color: var(--accent, #c79b3a);
    background: color-mix(in srgb, var(--accent, #c79b3a) 10%, var(--paper-bg));
  }
  .phase-box-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--paper-ink-soft, #6a6055);
    font-weight: 600;
  }
  .phase-box.active .phase-box-label {
    color: var(--accent, #c79b3a);
  }
  .phase-box-count {
    font-size: 1.1rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--paper-ink, #3a2f1f);
  }
  .phase-box-count.greyed {
    color: var(--paper-ink-soft, #6a6055);
    opacity: 0.55;
  }

  /* Primary action buttons */
  .primary-actions {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .btn-primary {
    font: inherit;
    width: 100%;
    padding: 0.5em 0.8em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 5px;
    background: var(--paper-ink, #3a2f1f);
    color: var(--paper-bg, #f3ecd9);
    font-weight: 600;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .btn-primary:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  .btn-primary:not(:disabled):hover {
    background: #2a1f10;
  }
  .btn-secondary {
    font: inherit;
    width: 100%;
    padding: 0.4em 0.8em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 5px;
    background: var(--paper-bg, #f3ecd9);
    color: inherit;
    cursor: pointer;
    font-size: 0.88rem;
  }
  .btn-secondary:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  .btn-secondary:not(:disabled):hover {
    background: var(--paper-square-light, #ece2c8);
  }

  /* Result message */
  .result {
    font-weight: 600;
    font-size: 0.9rem;
    text-align: center;
    padding: 0.4em 0;
    color: var(--accent, #c79b3a);
  }
  .result-reason {
    font-weight: 400;
    font-size: 0.8rem;
    color: var(--paper-ink-soft, #6a6055);
  }

  /* Resign / Draw row */
  .resign-draw-row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .btn-ghost {
    flex: 1;
    padding: 0.4em 0.7em;
    border: 1.5px solid var(--paper-line, #ccc);
    border-radius: 6px;
    background: transparent;
    color: var(--paper-ink-soft, #6a6055);
    font: inherit;
    font-size: 0.82rem;
    cursor: pointer;
    transition: border-color 80ms, color 80ms;
  }
  .btn-ghost:hover { border-color: var(--paper-line-strong); color: var(--paper-ink); }
  .btn-danger-ghost {
    flex: 1;
    padding: 0.4em 0.7em;
    border: 1.5px solid #c0392b44;
    border-radius: 6px;
    background: transparent;
    color: #c0392b;
    font: inherit;
    font-size: 0.82rem;
    cursor: pointer;
    transition: border-color 80ms, background 80ms;
  }
  .btn-danger-ghost:hover { border-color: #c0392b; background: #c0392b0f; }
  .btn-danger {
    padding: 0.45em 0.9em;
    border: 1.5px solid #c0392b;
    border-radius: 6px;
    background: #c0392b;
    color: #fff;
    font: inherit;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }
  .btn-danger:hover { background: #a93226; }

  /* Inline confirm dialogs */
  .inline-dialog {
    margin-top: 0.6rem;
    padding: 0.7em 0.8em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    background: var(--paper-bg);
    font-size: 0.85rem;
  }
  .inline-dialog p { margin: 0 0 0.5em; font-weight: 600; }
  .inline-dialog-btns { display: flex; gap: 0.4rem; }
  .inline-dialog-btns button { flex: 1; }

  /* Hints */
  .hint {
    margin: 0;
    font-size: 0.8rem;
    color: var(--paper-ink-soft, #6a6055);
    line-height: 1.4;
  }

  /* Export group */
  .export-group {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .eval-bar-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.3em 0.5em;
    background: var(--paper-square-light, #ece2c8);
    border-radius: 4px;
    font-size: 0.82rem;
  }
  .eval-label { color: var(--paper-ink-soft, #6a6055); }
  .eval-score { font-weight: 700; font-variant-numeric: tabular-nums; }
  .eval-score.positive { color: #3a7a3a; }
  .eval-score.negative { color: #a03030; }
  .engine-eval-row {
    margin-top: 0.25em;
    background: var(--paper-square-dark, #e3d7b8);
  }
  .eval-depth {
    margin-left: 0.35em;
    font-weight: 400;
    font-size: 0.85em;
    color: var(--paper-ink-soft, #6a6055);
  }
  .export-group button {
    font: inherit;
    width: 100%;
    padding: 0.32em 0.6em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 4px;
    background: var(--paper-bg, #f3ecd9);
    color: inherit;
    cursor: pointer;
    font-size: 0.82rem;
    text-align: left;
  }
  .export-group button:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  .export-group button:not(:disabled):hover {
    background: var(--paper-square-light, #ece2c8);
  }
  .sandbox-toggle {
    border-color: rgba(56, 178, 255, 0.6) !important;
    color: rgb(20, 120, 180);
  }
  .sandbox-toggle:not(:disabled):hover {
    background: rgba(56, 178, 255, 0.08) !important;
  }

  /* Error banner */
  .err {
    display: flex;
    align-items: center;
    gap: 0.5em;
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.4em 0.6em;
    margin: 0 0 0.5em;
    border-radius: 6px;
  }
  .err > span { flex: 1 1 auto; }
  .err-dismiss {
    flex: 0 0 auto;
    width: 1.5em;
    height: 1.5em;
    padding: 0;
    border: none;
    border-radius: 3px;
    background: transparent;
    color: inherit;
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
  }
  .err-dismiss:hover { background: rgba(169, 75, 59, 0.12); }

  /* Toast */
  .toast {
    position: fixed;
    bottom: 1.2rem;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(20, 24, 32, 0.92);
    color: #fff;
    padding: 0.55rem 1rem;
    border-radius: 6px;
    font-size: 0.92rem;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
    z-index: 1000;
    pointer-events: none;
  }

  /* Sandbox confirm dialog */
  dialog[open] {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 1rem 1.2rem;
    background: var(--paper-bg);
    color: inherit;
    min-width: min(320px, 90vw);
    box-shadow: 0 6px 24px rgba(0,0,0,0.18);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.45);
  }
  dialog[open] p { margin: 0 0 0.9rem; }
  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .confirm-ok {
    background: var(--p2, #a94b3b);
    color: #fff;
    border-color: var(--p2, #a94b3b);
    font-weight: 600;
  }

  /* ── Narrow screen: stack vertically ────────────────────────────────────── */
  @media (max-width: 820px) {
    .game-area {
      flex-direction: column;
    }
    .board-column {
      width: 100%;
    }
    .right-column {
      flex: unset;
      width: 100%;
    }
    .eval-column {
      flex: unset;
      width: 100%;
    }
    .right-panel {
      flex: unset;
      width: 100%;
      flex-direction: row;
      flex-wrap: wrap;
      gap: 0.6rem 1.2rem;
    }
    .status-block {
      flex-direction: row;
      flex-wrap: wrap;
      gap: 0.4rem 1.2rem;
    }
    .primary-actions {
      flex-direction: row;
    }
    .btn-primary, .btn-secondary {
      width: auto;
    }
    .export-group {
      flex-direction: row;
      flex-wrap: wrap;
    }
    .export-group button {
      width: auto;
      flex: 1 1 auto;
    }
    .panel-divider {
      display: none;
    }
    .info-anchor {
      position: static;
      margin-top: 0.4rem;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .board-stack.sandbox-mode { animation: none; }
  }
</style>
