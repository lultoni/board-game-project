<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    getEngine,
    ActionKind,
    decodeAction,
    encodeBodyguardChoice,
    decodeMailbox,
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateSnapshot,
    isSelfCast,
    SKILLS,
    MODIFIER_FOCUS,
    MODIFIER_CHARGE,
    runAiCall,
    type EvalBreakdown,
  } from "$lib/engine";
  import { resolveLoadout } from "$lib/state/draft";
  import { t } from "$lib/state/i18n";
  import BackButton from "$lib/ui/BackButton.svelte";
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
  } from "$lib/state/match-store.svelte";
  import { settings, slideDurationMs } from "$lib/state/settings.svelte";
  import {
    moveTargetsFor,
    movableSources,
    actableSources,
    findActionByKind,
    approachChoicesFor,
  } from "$lib/state/move-targets";
  import { skillTargetsFor, skillIsCastable, hasFocusModeChoice, hasRetargetVariants, hasSelfAndRetargetChoice, variantIsSelfCast, allyMoverCandidates, allyMoverDestinations, rawForAllyMove, type SkillVariant } from "$lib/state/skill-targets";
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
  import PlayerPanel from "$lib/match/PlayerPanel.svelte";
  import ProgressionPanel from "$lib/match/ProgressionPanel.svelte";
  import EvalBreakdownPanel from "$lib/eval/EvalBreakdownPanel.svelte";

  const mode = $derived(match.mode === "multiplayer" ? "multiplayer" : modeFromSeats(match.side));

  let bootError = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  /** True while a `stepAi` call is in flight. Drives the "AI is thinking…" overlay. */
  let aiThinking = $state(false);
  /** Search depth reached by the last completed AI move. 0 = none yet. */
  let aiLastDepth = $state(0);
  /** Score (centipawns, P1 POV) from the last completed or in-progress AI depth iteration. */
  let aiLastScore = $state(0);
  /** Timestamp of last depth-update UI flush — used to throttle reactive writes to 100ms. */
  let lastDepthUpdateMs = 0;
  /** `Date.now()` when the current AI search started. null when idle. Drives
   *  the time-based progress bar in PlayerPanel. */
  let aiSearchStartedAt = $state<number | null>(null);
  /** Monotonic ply counter — incremented after every successful apply. Used
   *  to time the AI thinking indicator's post-search linger: the indicator
   *  stays visible for one opponent turn after the search finished. */
  let plyCount = $state(0);
  /** `plyCount` snapshot captured when the last AI search finished. null
   *  when no search has ever run this match, or after the linger has hidden. */
  let aiFinishedAtPly = $state<number | null>(null);
  /** Static heuristic eval of the current board (P1 POV). null = not yet polled. */
  let heuristicEvalScore = $state<number | null>(null);
  /** Full eval breakdown for the analysis panel. null = not yet polled. */
  let heuristicEvalBreakdown = $state<EvalBreakdown | null>(null);
  // Snapshot of the breakdown at the end of the previous round, so the panel
  // can show round-over-round change per component.
  let prevRoundBreakdown = $state<EvalBreakdown | null>(null);
  let lastRoundSeen = $state<number | null>(null);

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
  /** Transient toast for export / sandbox feedback. Cleared by a timer. */
  let toast = $state<string>("");
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  /** Confirmation dialog state for sandbox discard. */
  let sandboxConfirmMsg = $state<string | null>(null);
  let sandboxConfirmResolve: ((ok: boolean) => void) | null = null;
  let sandboxUndoStack = $state<string[]>([]);
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
   *  multiplayer this is always true — both seats are local. */
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
  // AI thinking indicator targets the seat that is currently thinking.
  const p1Thinking = $derived(aiThinking && match.position?.toMove === 0);
  const p2Thinking = $derived(aiThinking && match.position?.toMove === 1);

  // Track which squares used their Move action this phase. Stored as the
  // attacker's final square (target for plain Move, approach_sq for
  // Move-Attack). Cleared whenever phase or to-move flips.
  let usedThisPhase = $state<Set<number>>(new Set());
  let lastPhaseKey = $state<string>(""); // `${toMove}:${phase}` — phase boundary detector

  // Live drag state for the parent — Board owns the pointer mechanics and
  // pushes updates here so we can render path trail + hover ring.
  let dragSrc = $state<number | null>(null);
  let dragTrail = $state<number[]>([]);
  /** Square currently under the pointer (drag or hover). Used for click-mode
   *  landing preview: shows where the attacker would land before clicking. */
  let hoverSq = $state<number | null>(null);
  let dragHover = $state<number | null>(null);
  /** Live cursor position in SVG coords (viewBox = 800), used to pick the
   * sub-tile approach for multi-path Move-Attacks. (0,0) when idle. */
  let cursorX = $state<number>(0);
  let cursorY = $state<number>(0);

  // Approach-square chooser state: when the user clicks a Move-Attack target
  // with multiple approach paths, we surface a chooser.
  let pendingApproach = $state<{ target: number; approaches: number[] } | null>(null);

  // Bodyguard chooser state. The engine owns this — `Position.pending_bodyguard`
  // is set on the attacker's tentative Move-Attack and cleared on the
  // defender's `BodyguardChoice`. All four play modes converge here because
  // the engine flips STM to the defender as part of the tentative apply.
  // `legalActions` is then restricted to BodyguardChoice variants, so any
  // click that maps to one is automatically valid on the host.
  const pendingBodyguard = $derived(match.position?.pendingBodyguard ?? null);

  // Armed skill: when the player clicks a skill slice on the wheel, the
  // skill is "armed" and the next click on a valid target tile fires it.
  // Self-cast skills fire immediately on slice click and never enter armed
  // state. Cleared on any non-target click, on selection change, or after
  // firing.
  let armedSkill = $state<{ square: number; skillId: number } | null>(null);
  /** When the armed skill needs a direction (Shove), clicking a target opens
   *  this picker instead of firing. Cleared on cancel / pick / disarm. */
  let pendingDirection = $state<{ target: number; variants: SkillVariant[] } | null>(null);
  /** Focus-mode preference. Only consulted when Focus is staged AND the
   *  armed skill is Blast or Shove (the two skills with two distinct Focus
   *  interpretations). "activation" = +1 to range (broader target set);
   *  "effect" = base range, but the effect itself is boosted (Blast pushes 2,
   *  Shove pushes 2). Player toggles via SkillInfoCard while armed. */
  let focusModePref = $state<"activation" | "effect">("activation");
  /** Focus-retarget preference. Only consulted when Focus is staged on a
   *  skill that has both a self-cast branch and an ally-retarget branch
   *  (Shield, Dash, Retreat). "self" = caster channels the skill;
   *  "ally" = adjacent ally is the recipient/mover. Player toggles via the
   *  Self / Ally picker that mirrors the focus-mode (Range/Effect) toggle. */
  let focusRetargetPref = $state<"self" | "ally">("self");
  /** Two-stage Focus-retarget picker: in "ally" mode for movement skills
   *  (Dash/Retreat — where the ally's destination differs from the ally
   *  itself), the player first clicks WHICH adjacent ally moves, then clicks
   *  the destination. Null = no ally picked yet (squares show the ally
   *  candidates); set = ally chosen (squares show that ally's destinations).
   *  Reset on arm change, mode switch back to self, or fire. Not used for
   *  Shield retarget — there, target == aux_sq, so a single click suffices. */
  let focusAllyChosen = $state<number | null>(null);
  /** Focus / Charge are derived from the engine's pendingModifiers bitfield.
   *  Casting Focus / Charge (skills 14 / 15) stages the modifier; the wheel
   *  reads these flags to render the slice as "active". */
  const focusActive = $derived(
    (match.position?.pendingModifiers ?? 0) & MODIFIER_FOCUS ? true : false,
  );
  const chargeActive = $derived(
    (match.position?.pendingModifiers ?? 0) & MODIFIER_CHARGE ? true : false,
  );
  /** Slice currently hovered on the wheel. Drives the range overlay. */
  let hoveredSlice = $state<import("$lib/board/SkillWheel.svelte").SliceKind | null>(null);

  // Compute whether the currently hovered drag square is a legal drop.
  const dragHoverLegal = $derived.by(() => {
    if (dragSrc === null || dragHover === null) return false;
    const targets = moveTargetsFor(match.legal, dragSrc);
    return targets.squares.has(dragHover);
  });

  // Where the attacker would actually land if the player released the drag
  // right now. For plain Move this equals the hovered square; for Move-Attack
  // it equals the approach_sq (penultimate tile of the path). On multi-
  // approach attacks we infer the approach from the most recent crossing
  // in the drag trail; if none matches we leave it null and the parent
  // shows no landing marker (chooser will open on drop).
  // Live drag landing-marker square. For a plain Move (no approach choice)
  // it equals the approach_sq (penultimate tile of the path). On multi-
  // approach attacks we pick the approach by where the cursor sits within
  // the hovered target tile: the candidate approach whose direction from
  // the target best matches the cursor's offset from the target center wins.
  // This makes diagonal 1-tile attacks easy ("aim where you're coming from")
  // and lets click-attacks pick an intermediate without a chooser.
  const SQUARE_SIZE = 100; // board viewBox is 800, 8 tiles.
  function pickApproachByCursor<T>(
    target: number,
    cx: number,
    cy: number,
    candidates: Map<number, T>,
  ): number | null {
    if (candidates.size === 0) return null;
    if (candidates.size === 1) {
      return candidates.keys().next().value as number;
    }
    const tgtFile = target & 7;
    const tgtRank = (target >> 3) & 7;
    const tgtCX = tgtFile * SQUARE_SIZE + SQUARE_SIZE / 2;
    const tgtCY = (7 - tgtRank) * SQUARE_SIZE + SQUARE_SIZE / 2;
    let offX = cx - tgtCX;
    let offY = cy - tgtCY;
    // Cursor exactly at center → fall back to `approach == src` if available
    // (the obvious "direct attack" default), else first candidate.
    const offLen2 = offX * offX + offY * offY;
    if (offLen2 < 4) {
      if (candidates.has(target)) return target; // shouldn't happen — approach != target
      // Default to direct (approach == src) if present.
      // src isn't known here, but the caller can supply it via the map key.
      // We just pick the first one as a fallback.
      return candidates.keys().next().value as number;
    }
    let best: number | null = null;
    let bestScore = -Infinity;
    for (const ap of candidates.keys()) {
      const apFile = ap & 7;
      const apRank = (ap >> 3) & 7;
      const apCX = apFile * SQUARE_SIZE + SQUARE_SIZE / 2;
      const apCY = (7 - apRank) * SQUARE_SIZE + SQUARE_SIZE / 2;
      let dirX = apCX - tgtCX;
      let dirY = apCY - tgtCY;
      const dirLen2 = dirX * dirX + dirY * dirY;
      if (dirLen2 === 0) continue; // approach == target shouldn't occur for multi
      // Normalised dot product = cosine of the angle between cursor-offset
      // and approach-direction. Higher = better aligned.
      const score = (offX * dirX + offY * dirY) / Math.sqrt(dirLen2);
      if (score > bestScore) {
        bestScore = score;
        best = ap;
      }
    }
    return best;
  }

  const dragLanding = $derived.by(() => {
    if (dragSrc === null || dragHover === null) return null;
    const targets = moveTargetsFor(match.legal, dragSrc);
    if (!targets.squares.has(dragHover)) return null;
    const approaches = targets.byTarget.get(dragHover);
    if (!approaches || approaches.size === 0) return null;
    return pickApproachByCursor(dragHover, cursorX, cursorY, approaches);
  });

  // Click-mode landing preview: when a piece is selected (no drag) and the
  // cursor is over a legal target, show where the attacker would land — same
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

  function handlePressStart(src: number) {
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
  // Standard interactivity: it's the local seat's turn and we're not busy.
  // Bodyguard no longer needs a special-case override — when the engine sets
  // `pending_bodyguard` it also flips STM to the defender, so the defender's
  // seat naturally becomes `currentSeatIsLocal`.
  // In sandbox mode: always interactive regardless of whose turn it is,
  // so the user can freely move pieces for both sides.
  const interactive = $derived(
    ready
    && !busy
    && match.position?.gameResult === 0
    && (match.mode === "sandbox" || (!currentSeatIsAi && currentSeatIsLocal))
  );

  // Wheel state. Open whenever a piece is selected in the Skill Phase
  // (and the player isn't mid-drag — we don't want the wheel popping up
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
    // Hide the wheel once a skill is armed — the player is now choosing a
    // target, so the wheel chrome would just obscure the board.
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
  // skills (ids 14/15) — if the piece has one equipped they show up as the
  // corresponding skill sector; they have NO dedicated sector of their own.
  const wheelLegality = $derived.by(() => {
    if (!wheelOpen) {
      return {
        skill1Legal: false,
        skill2Legal: false,
        endPhaseLegal: false,
      };
    }
    const src = wheelOpen.square;
    const skill1Legal = wheelOpen.skill1 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill1);
    const skill2Legal = wheelOpen.skill2 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill2);
    return {
      skill1Legal,
      skill2Legal,
      endPhaseLegal: endPhaseAction !== null,
    };
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
  // Shield retarget has target == ally, so a single click suffices — this is
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
      // there's no "local" side — use gameEnd for draws, victory for any win.
      if (result === 3) {
        sfx.play("gameEnd");
      } else if (match.mode === "aivai" || match.mode === "sandbox") {
        sfx.play("gameEnd");
      } else {
        const localSeat = match.localSeat ?? (match.side.p1 === "human" ? 0 : 1);
        if (match.localSeat === null && !__loggedSeatFallback526) {
          __loggedSeatFallback526 = true;
          // WARNING: this fallback uses match.side.p1 === "human" instead of role — likely wrong in HvH-MP.
          console.warn(`[mp] seat fallback used at match:526 [suspect] (localSeat=null, side.p1=${match.side.p1}, role=${multiplayerRole()}) → seat=${localSeat}`);
        }
        const localWon = (result === 1 && localSeat === 0) || (result === 2 && localSeat === 1);
        sfx.play(localWon ? "victory" : "defeat");
      }
    }
    lastGameResult = result;
  });

  /** AI scheduler. Whenever it's an AI seat's turn and the loop is allowed
   *  to run, queue a `runAiStep()`. For HvAI this fires automatically on
   *  every AI ply. For AIvAI it chains turn-after-turn while `aiAutoPlay`
   *  is true; pausing freezes the loop after the in-flight call returns.
   *  Anchored on `phaseKey()` rather than `position` directly so a stable
   *  side+phase pair doesn't re-trigger when other position fields change.
   *
   *  Owns a single timer handle (`aiTimer`) rather than a boolean latch. The
   *  handle is set synchronously when the timer is scheduled and only cleared
   *  inside the timer callback or by teardown — no microtask window where a
   *  re-entrant $effect run could schedule a duplicate. */
  $effect(() => {
    if (!ready) return;
    if (match.mode === "sandbox") return;
    if (!currentSeatIsAi) return;
    // For AIvAI, gate on the play/pause toggle. For HvAI, always run.
    if (match.mode === "aivai" && !aiAutoPlay) return;
    if (busy) return;
    // Visible cooldown (so a spectator can watch AIvAI step-by-step, or HvAI
    // has a beat to repaint the board). `runAiStep` runs the search in
    // parallel with this delay — the cooldown is a floor, not a sequential
    // wait. For AIvAI we honour the user-configured step delay; HvAI is a
    // small fixed beat.
    const delay = match.mode === "aivai"
      ? Math.max(16, settings.aivaiStepDelayMs)
      : 30;
    void runAiStep(delay);
  });

  onMount(async () => {
    ownershipToken = claimRouteOwnership();
    console.log(`[mp] /match/ mounted (mode=${match.mode}, role=${mpState.role}, localSeat=${match.localSeat}, status=${mpState.status})`);
    try {
      eng = await getEngine();
      renderer = createPlyRenderer(eng, {
        positionSink: match,
        sfxEnabled: true,
        onMoveLanding: (finalSq) => {
          usedThisPhase = new Set([...usedThisPhase, finalSq]);
        },
      });
      const pending = match.pendingSnapshotJson;
      // Snapshot side before reset so it survives the reset (which clears
      // mode/position/legal but preserves side by design).
      const sideAtBoot = { p1: match.side.p1, p2: match.side.p2 };
      // Preserve multiplayer mode through the reset — the lobby set this
      // before navigating here and the reset would otherwise drop mode back
      // to "idle". MP role/code now live in mpState (read via the
      // `multiplayerRole` / `multiplayerCode` $derived constants) so they
      // automatically survive the reset.
      const wasMultiplayer = match.mode === "multiplayer";
      // Task 8 — per-side loadout path (either both pre-made / mirror match,
      // or per-side custom+preMade mixes for local play). Snapshot BEFORE
      // resetMatchState() because the reset clears `sideLoadouts` (so stale
      // ids from a prior match can't leak in via direct navigation).
      // `/setup/` writes the field on commit; we read it here once and
      // consume via `resolveLoadout()`.
      const sideLoadouts = match.sideLoadouts;
      resetMatchState();
      match.side = sideAtBoot;
      if (sideLoadouts) {
        const p1Loadout = await resolveLoadout(sideLoadouts.p1);
        const p2Loadout = await resolveLoadout(sideLoadouts.p2);
        if (p1Loadout && p2Loadout) {
          const configJson = buildEngineConfigJson(sideAtBoot);
          await eng.createEngineWithLoadouts(configJson, p1Loadout, p2Loadout);
        } else {
          // Custom row was deleted between the /setup/ pick and here. Fall
          // back to a blank engine so the route doesn't deadlock.
          console.warn("resolveLoadout returned null; falling back to fresh engine");
          await eng.createEngine();
        }
        // Consume — re-entering /match/ later (e.g. a snapshot restore from
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
      match.mode = wasMultiplayer ? "multiplayer" : modeFromSeats(match.side);
      await refreshMatchLogAvailable();
      // Start the telemetry session for non-analysis modes. No-op for
      // sandbox; sandbox enters via /match/ but flips mode immediately,
      // so we'd never reach this with mode === "sandbox" on boot.
      // Skip if a session is already active — the carrier survives the
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
          send: (m: WireMessageV2) => mpSendRaw(encodeMessageV2(m)),
          subscribe: (cb) => mpOnRawData((raw) => {
            const decoded = decodeMessageV2(raw);
            if (decoded) cb(decoded);
          }),
          onApplied: async (raw, _phase, meta) => {
            if (!renderer) return;
            // Drain any deferred Skill refresh from a prior remote-applied
            // skill — its setTimeout would otherwise fire after we render
            // this new action and clobber the post-state.
            renderer.drainPendingSkillRefresh();
            // The wrapper captured `prePositionView` before tryApply, so we
            // diff against an explicit pre-state value — no reliance on the
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
            mpState.lastError = `lost sync with host (${reason}, ${attempts} attempts) — try Rejoin`;
          },
          onPausedChange: (p) => {
            mpPaused = p;
          },
          onHostCommitted: async () => { /* recordPly fires via onApplied */ },
        },
      );
      // Re-announce session on every transport-open while we're mounted.
      // Direct callbacks — no $effect. Protocol sequencing shouldn't be
      // scheduled through Svelte's reactive graph; effects fire on the
      // next microtask, and network state machines don't tolerate that
      // window. See PROTOCOL_TRACE.md Part 2 §6.
      const unsubOpen = mpOnConnected(() => mpEngine?.notifyConnectionOpen());
      const unsubClose = mpOnDisconnected(() => mpEngine?.notifyConnectionLost());
      mpConnectedUnsub = () => { unsubOpen(); unsubClose(); };
      // If the transport is already open by the time /match/ mounts (the
      // usual case — pairing happened in the lobby), the onConnected event
      // has already fired and we missed it. Fire once synchronously so the
      // engine emits its `session-hello`.
      if (mpState.status === "connected") mpEngine.notifyConnectionOpen();
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
  // `match.telemetryMatchId` stays set — the match might recover, in which
  // case `recordPly`/`finalizeMatch` continue working. `markNetworkLost`
  // itself only flips the row if status is "in-progress", so the natural
  // finalize path will still overwrite it to "ended" if the match completes.
  //
  // Idempotent via `hasMarkedNetworkLost` — repeat disconnections during the
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
        // Swallow — telemetry must never block gameplay.
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
      // Sandbox bypasses the wrapper — the user is exploring locally and
      // sandbox moves must NOT echo to a peer or get logged as match plies.
      if (match.mode === "sandbox") {
        const snap = await eng!.snapshotJson();
        await renderer.applyAndRender(raw, async () => {
          await eng!.tryApply(raw);
        });
        sandboxUndoStack = [...sandboxUndoStack.slice(-49), snap];
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
      // submitAction sends `intent` and waits for the host's `committed` —
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
    // A successful apply means whatever transient error the user was looking
    // at (move-refused, illegal-target, etc.) is no longer relevant.
    // Anti-cheat / engine-boot errors set bootError too, but those branches
    // never reach here — afterApplied only fires after a committed action.
    if (bootError !== null) bootError = null;
    const k = phaseKey();
    if (k !== lastPhaseKey) {
      usedThisPhase = new Set();
      lastPhaseKey = k;
    }
  }

  // Poll the heuristic eval whenever the position advances (including the
  // initial one after engine boot, which afterApplied() never sees) or the
  // relevant settings toggle on. Reads through the same guards.
  $effect(() => {
    void match.position;
    void settings.showHeuristicEval;
    void settings.showEvalPanel;
    if (!(settings.showHeuristicEval || settings.showEvalPanel)) return;
    if (!eng || match.mode === "multiplayer" || !match.position) return;
    const e = eng;
    const priorBreakdown = heuristicEvalBreakdown;
    const priorRound = lastRoundSeen;
    void e.heuristicEval().then((v) => {
      heuristicEvalScore = v.total;
      const curRound = match.position?.roundNumber ?? null;
      // On round transition, freeze the last-seen breakdown as the "previous"
      // reference so the panel can display the round-over-round change.
      if (curRound !== null && priorRound !== null && curRound !== priorRound && priorBreakdown !== null) {
        prevRoundBreakdown = priorBreakdown;
      }
      lastRoundSeen = curRound;
      heuristicEvalBreakdown = v;
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
    busy = true;
    aiThinking = true;
    // Do NOT reset aiLastDepth / aiLastScore / aiFinishedAtPly here. The prior
    // values stay visible until the streaming depth callback overwrites them
    // (typically within a few frames) or the search completes. The `thinking`
    // spinner already visually takes over from the linger badge, so there's no
    // risk of confusion — and this avoids the "d0 +0" flash the user reported
    // when quick shallow depths report before the deeper ones catch up.
    aiSearchStartedAt = Date.now();
    try {
      // Drain any deferred Skill refresh before snapshotting pre-state — see
      // applyRaw for rationale.
      renderer.drainPendingSkillRefresh();
      const delayP = minDelayMs > 0
        ? new Promise<void>((r) => setTimeout(r, minDelayMs))
        : Promise.resolve();
      const [result] = await Promise.all([runAiCall(() => eng!.stepAi((d, s) => {
        const now = Date.now();
        if (now - lastDepthUpdateMs >= 100) {
          lastDepthUpdateMs = now;
          aiLastDepth = d;
          aiLastScore = s;
        }
      })), delayP]);
      const raw = result.appliedAction;
      aiLastDepth = result.depth;
      if (raw === 0) {
        // AI returned no move. Two cases:
        //   - match.position.gameResult !== 0 → terminal (mate/stalemate),
        //     legitimate no-op; just refresh.
        //   - gameResult === 0 → engine returned no action on a live position.
        //     That's a wedge — the AivAI scheduler would re-fire forever.
        //     Disable auto-play and surface a toast so the user sees the
        //     stall instead of an apparent freeze. Don't try to recover here.
        await refresh();
        if (match.position && match.position.gameResult === 0) {
          aiAutoPlay = false;
          showToast("AI returned no move — pausing");
        }
        return;
      }
      // Persist AI ply telemetry. Sandbox is gated above (early return).
      await recordPly(eng);
      // The engine has advanced, but `match.position` (the renderer's
      // positionSink) is still the PRE-step snapshot — refresh() hasn't run
      // yet. snapshotPreState reads from there, so this is safe.
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
      aiThinking = false;
      aiSearchStartedAt = null;
      // Capture the ply the search finished on so PlayerPanel can render the
      // greyed-out linger for exactly one opponent turn (until `plyCount`
      // advances past this snapshot). afterApplied already bumped plyCount
      // on the applied AI ply, so `plyCount` at this point == "AI just moved".
      aiFinishedAtPly = plyCount;
      busy = false;
    }
  }

  /** After the attacker has decided (target, approach), apply the tentative
   *  Move-Attack. If the engine has eligible Bodyguard Guards it will set
   *  `pending_bodyguard` + flip STM to the defender, who then resolves via a
   *  `BodyguardChoice` ply — handled in `handleSquareClick`. All four play
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
    sfx.unlock();

    // Bodyguard chooser is active (engine has pending_bodyguard set): clicks
    // select defender (decline, idx=0) or an eligible Guard (idx=k+1). The
    // legal-action set is restricted to BodyguardChoice variants at this
    // point — submitting any other raw would fail anti-cheat in MP.
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
        // Shove (11) needs a push-direction pick before firing. Open the
        // direction picker on the target tile and let the player choose.
        if (armedSkill.skillId === 11) {
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
    if (match.selection !== src) {
      match.selection = src;
    }
    const targets = moveTargetsFor(match.legal, src);
    const dropSq = path[path.length - 1];
    const candidates = targets.byTarget.get(dropSq);
    if (!candidates || candidates.size === 0) {
      // Dropped on illegal tile (or back on src) — soft drop thud.
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

  // Find the raw u32 for a self-cast skill (target == src). Used when a
  // self-cast skill slice on the wheel is clicked: we don't need a target
  // click, just look up and fire. Returns null if no such action.
  function rawForSelfCast(src: number, skillId: number): number | null {
    for (let i = 0; i < match.legal.length; i++) {
      const raw = match.legal[i];
      const a = decodeAction(raw);
      if (a.kind !== ActionKind.Skill) continue;
      if (a.src !== src) continue;
      if (a.skillId !== skillId) continue;
      if (a.target !== src) continue;
      return raw;
    }
    return null;
  }

  function handleWheelSliceClick(slice: import("$lib/board/SkillWheel.svelte").SliceKind) {
    if (!interactive) return;
    if (!wheelOpen) return;
    sfx.unlock();
    sfx.play("click");
    const src = wheelOpen.square;

    if (slice.kind === "skill") {
      // Self-cast skills normally fire immediately — BUT when Focus is staged
      // and the engine emitted retarget variants (Shield → adjacent ally,
      // Dash/Retreat → adjacent ally), we arm instead so the player can pick
      // a recipient (or click self to take the self-cast).
      if (isSelfCast(slice.skillId)) {
        const retargetable = hasRetargetVariants(match.legal, src, slice.skillId);
        if (!retargetable) {
          const raw = rawForSelfCast(src, slice.skillId);
          if (raw !== null) {
            armedSkill = null;
            applyRaw(raw);
          }
          return;
        }
        // Fall through to arm — the target picker will surface src + ally
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

    // `modifierBadge` is hover-only — clicking it is a no-op. Focus / Charge
    // are cast as regular skills via the piece's skill slot, not from the
    // wheel directly.
    if (slice.kind === "modifierBadge") {
      return;
    }

    if (slice.kind === "endphase") {
      if (endPhaseAction !== null) {
        armedSkill = null;
        match.selection = null;
        applyRaw(endPhaseAction);
      }
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
  // (Shove) shouldn't go through this path — they hit the DirectionPicker
  // first — but we fall through to "any matching" for safety.
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
   *  when the (src, skill) has a Focus-mode choice. */
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
   *  isn't reset here — snapshot restore / engine swap handles it. */
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
    if (!eng || busy || aiThinking) return;
    if (match.mode === "sandbox") return;
    busy = true;
    sfx.play("sandboxEnter");
    try {
      // Capture pre-sandbox snapshot BEFORE flipping mode — otherwise an
      // in-flight AI scheduler tick could mutate state between capture and
      // mode-flip.
      const snap = await eng.snapshotJson();
      clearAllPickers();
      // Sandbox is a purely local fork — the underlying match (including MP
      // transport and telemetry) continues to exist. We do NOT abandon the
      // telemetry session on entry: the exploratory plies never touch the
      // engine's true state (they roll back on exit), so nothing spurious is
      // logged, and keeping the telemetry row live means a subsequent
      // leave-during-sandbox still fires markNetworkLost / markAbandoned and
      // surfaces the "you left a game" card in the lobby.
      match.trueSnapshotJson = snap;
      match.sandboxMovesApplied = 0;
      sandboxUndoStack = [];
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
      const snap = sandboxUndoStack[sandboxUndoStack.length - 1];
      sandboxUndoStack = sandboxUndoStack.slice(0, -1);
      await eng.restoreFromSnapshot(snap);
      match.sandboxMovesApplied = Math.max(0, match.sandboxMovesApplied - 1);
      clearAllPickers();
      await syncFromEngine();
    } finally {
      busy = false;
    }
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
      // Restore the mode we entered sandbox from. `modeFromSeats()` can't
      // round-trip "multiplayer" (both seats are "human" in MP too), so we
      // rely on the stashed value; fall back to seat-derivation only if
      // preSandboxMode was somehow lost.
      match.mode = match.preSandboxMode ?? modeFromSeats(match.side);
      match.preSandboxMode = null;
      clearAllPickers();
      await syncFromEngine();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
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
      void (async () => {
        try {
          let partial: string | undefined;
          if (engRef) {
            try { partial = (await engRef.matchLogJson()) ?? undefined; } catch { /* engine bad state */ }
          }
          await getTelemetryStore().markAbandoned(id, partial);
        } catch {
          // Swallow — telemetry must never block navigation.
        }
      })();
    }
    if (mpEngine) {
      mpEngine.dispose();
      mpEngine = null;
    }
    if (mpConnectedUnsub) {
      mpConnectedUnsub();
      mpConnectedUnsub = null;
    }
    // Leaving /match/ before a natural end means we're going back to the
    // lobby (or home). Soft-tear the transport so the peer sees the drop but
    // our carrier state (code, peerEverPaired, disconnectedSince) survives —
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
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

<main>
  <header>
    <BackButton />
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
      <button type="button" class="err-dismiss" onclick={() => (bootError = null)} aria-label="dismiss">×</button>
    </div>
  {/if}

  {#if !ready}
    <p>{t("app.loading")}</p>
  {:else}
    <div class="game-area">
      <!-- Left column: P2 panel + board + P1 panel -->
      <div class="board-column">
        <PlayerPanel
          player="p2"
          position={match.position}
          aiThinking={p2Thinking}
          aiLastDepth={aiLastDepth}
          aiLastScore={aiLastScore}
          aiMaxDepth={settings.p2MaxDepth}
          isAiSeat={p2IsAi}
          aiSearchStartedAt={p2Thinking ? aiSearchStartedAt : null}
          aiThinkBudgetMs={settings.p2ThinkTimeMs}
          aiFinishedAtPly={aiFinishedAtPly}
          plyCount={plyCount}
        />

        <div class="board-stack" class:sandbox-mode={match.mode === "sandbox"}>
          <Board
            position={match.position}
            pieceIds={renderer?.pieceIds ?? new Map()}
            selection={match.selection}
            moveTargets={armedSkill ? armedSkillTargets : moveTargets.squares}
            {selectable}
            draggable={movable}
            usedSquares={usedThisPhase}
            shakingSquares={renderer?.shakingSquares ?? new Set()}
            lungeSquares={renderer?.lungeSquares ?? new Map()}
            pieceMotion={renderer?.pieceMotion ?? new Map()}
            toMove={match.position?.gameResult === 0 ? (match.position?.toMove ?? null) : null}
            effectsActive={(renderer?.effectQueue.length ?? 0) > 0}
            approachChoices={pendingApproach?.approaches ?? []}
            bodyguardChoice={pendingBodyguard ? {
              defender: pendingBodyguard.targetSq,
              guards: pendingBodyguard.eligible.slice(),
            } : null}
            lastApplied={renderer?.lastApplied ?? null}
            {interactive}
            wheelOpen={wheelOpen}
            armedSkillId={armedSkill?.skillId ?? null}
            {focusActive}
            {chargeActive}
            {wheelLegality}
            onWheelSliceClick={handleWheelSliceClick}
            onWheelSliceHover={handleWheelSliceHover}
            directionPicker={pendingDirection}
            onDirectionPick={handleDirectionPick}
            onDirectionCancel={handleDirectionCancel}
            onSquareClick={handleSquareClick}
            onPieceDrop={handlePieceDrop}
            onPressStart={handlePressStart}
            onDragMove={handleDragMove}
            {dragTrail}
            {dragHover}
            {dragHoverLegal}
            dragLanding={effectiveLanding}
            onApproachChoice={(ap) => {
              if (pendingApproach) {
                const target = pendingApproach.target;
                pendingApproach = null;
                commitMoveTargetApproach(target, ap);
              }
            }}
          />
          {#if renderer}
            <EffectsLayer viewBox={800} wheelPad={60} queue={renderer.effectQueue} />
          {/if}
          {#if hoveredSlice && wheelOpen}
            <div class="info-anchor">
              <SkillInfoCard
                slice={hoveredSlice}
                {focusActive}
                {chargeActive}
                armed={hoveredSlice.kind === "skill"
                  && armedSkill?.skillId === hoveredSlice.skillId}
              />
            </div>
          {/if}
        </div>

        <PlayerPanel
          player="p1"
          position={match.position}
          aiThinking={p1Thinking}
          aiLastDepth={aiLastDepth}
          aiLastScore={aiLastScore}
          aiMaxDepth={settings.p1MaxDepth}
          isAiSeat={p1IsAi}
          aiSearchStartedAt={p1Thinking ? aiSearchStartedAt : null}
          aiThinkBudgetMs={settings.p1ThinkTimeMs}
          aiFinishedAtPly={aiFinishedAtPly}
          plyCount={plyCount}
        />
      </div>

      <!-- Right column: status + controls + export + progression -->
      <div class="right-column">
      <aside class="right-panel">
        <!-- Status -->
        <div class="status-block">
          <div class="stat-row">
            <span class="stat-label">Round</span>
            <span class="stat-value">{match.position?.roundNumber ?? "–"}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Phase</span>
            <span class="phase-pill" class:move={inMovePhase} class:skill={!inMovePhase}>
              {inMovePhase ? "Move" : "Skill"}
            </span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Actions</span>
            <span class="stat-value">{match.position?.actionsRemaining ?? "–"}</span>
          </div>
        </div>

        <div class="panel-divider"></div>

        <!-- Primary action -->
        <div class="primary-actions">
          {#if match.position?.gameResult !== 0}
            <p class="result">
              {match.position?.gameResult === 1
                ? t("result.p1Wins")
                : match.position?.gameResult === 2
                  ? t("result.p2Wins")
                  : t("result.draw")}
            </p>
          {:else if match.mode === "aivai"}
            <button
              type="button"
              class="btn-primary"
              disabled={match.position?.gameResult !== 0}
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
              disabled={busy || aiAutoPlay || match.position?.gameResult !== 0}
              onclick={() => void runAiStep()}
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

        <!-- Contextual hints + skill toggles -->
        {#if pendingApproach}
          <p class="hint">Choose the path the attacker takes — click a highlighted square, or press Esc to cancel</p>
        {/if}
        {#if pendingBodyguard}
          <p class="hint">Bodyguard: click the red defender to take the hit, or a blue guard to intercept</p>
        {/if}
        {#if armedNeedsAllyPick && focusAllyChosen === null}
          <p class="hint">Pick an adjacent ally to channel onto, then choose where they move</p>
        {/if}
        {#if armedNeedsAllyPick && focusAllyChosen !== null}
          <p class="hint">Choose the destination for the chosen ally — click another ally to switch</p>
        {/if}
        {#if pendingDirection}
          <p class="hint">Choose a push direction — click an arrow, or press Esc to cancel</p>
        {/if}
        {#if armedHasFocusModeChoice && !pendingDirection}
          <div class="focus-mode">
            <span class="focus-mode-label">Focus boosts:</span>
            <div class="focus-mode-toggle" role="radiogroup" aria-label="focus mode">
              <button
                type="button"
                role="radio"
                aria-checked={focusModePref === "activation"}
                class:active={focusModePref === "activation"}
                onclick={() => (focusModePref = "activation")}
              >Range (+1)</button>
              <button
                type="button"
                role="radio"
                aria-checked={focusModePref === "effect"}
                class:active={focusModePref === "effect"}
                onclick={() => (focusModePref = "effect")}
              >Effect</button>
            </div>
          </div>
        {/if}
        {#if armedHasRetargetChoice && !pendingDirection}
          <div class="focus-mode">
            <span class="focus-mode-label">Focus onto:</span>
            <div class="focus-mode-toggle" role="radiogroup" aria-label="focus recipient">
              <button
                type="button"
                role="radio"
                aria-checked={focusRetargetPref === "self"}
                class:active={focusRetargetPref === "self"}
                onclick={() => { focusRetargetPref = "self"; focusAllyChosen = null; }}
              >Self</button>
              <button
                type="button"
                role="radio"
                aria-checked={focusRetargetPref === "ally"}
                class:active={focusRetargetPref === "ally"}
                onclick={() => { focusRetargetPref = "ally"; focusAllyChosen = null; }}
              >Ally</button>
            </div>
          </div>
        {/if}

        <div class="panel-divider"></div>

        <!-- Export + sandbox -->
        <div class="export-group">
          <button
            type="button"
            disabled={busy}
            onclick={() => void copyFen()}
          >{t("controls.copyFen")}</button>
          <button
            type="button"
            disabled={busy || !matchLogAvailable}
            onclick={() => void copyMatchLog()}
          >{t("controls.copyMatchLog")}</button>
          <button
            type="button"
            disabled={busy || !matchLogAvailable}
            onclick={() => void downloadMatchLog()}
          >{t("controls.downloadMatchLog")}</button>
          <button
            type="button"
            class="sandbox-toggle"
            disabled={busy || (match.mode !== "sandbox" && aiThinking)}
            onclick={() => void (match.mode === "sandbox" ? exitSandbox() : enterSandbox())}
          >{match.mode === "sandbox" ? t("controls.exitSandbox") : t("controls.sandbox")}</button>
          {#if match.mode === "sandbox"}
            <button
              type="button"
              disabled={busy || sandboxUndoStack.length === 0}
              onclick={() => void undoSandbox()}
            >{t("controls.undo")}</button>
          {/if}
        </div>
        {#if settings.showHeuristicEval && heuristicEvalScore !== null && match.mode !== "multiplayer"}
          <div class="eval-bar-row">
            <span class="eval-label">Eval</span>
            <span class="eval-score" class:positive={heuristicEvalScore > 0} class:negative={heuristicEvalScore < 0}>
              {heuristicEvalScore > 0 ? '+' : ''}{heuristicEvalScore}
            </span>
          </div>
        {/if}
      </aside>

      <!-- Progression panel: income + skill actions over upcoming rounds -->
      {#if match.position}
        <ProgressionPanel roundNumber={match.position.roundNumber} />
      {/if}
      </div>

      {#if settings.showEvalPanel && match.mode !== "multiplayer"}
        <div class="eval-column">
          <EvalBreakdownPanel breakdown={heuristicEvalBreakdown} prevBreakdown={prevRoundBreakdown} />
        </div>
      {/if}
    </div>
  {/if}
</main>

{#if toast}
  <div class="toast" role="status" aria-live="polite">{toast}</div>
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
    top: 0;
    left: calc(100% + 0.6rem);
    z-index: 5;
    pointer-events: none;
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
  .phase-pill {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 0.1em 0.5em;
    border-radius: 3px;
  }
  .phase-pill.move {
    background: rgba(75, 107, 138, 0.15);
    color: var(--p1, #4b6b8a);
  }
  .phase-pill.skill {
    background: rgba(138, 74, 189, 0.15);
    color: #7a3aad;
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

  /* Hints */
  .hint {
    margin: 0;
    font-size: 0.8rem;
    color: var(--paper-ink-soft, #6a6055);
    line-height: 1.4;
  }

  /* Focus mode toggles */
  .focus-mode {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.82rem;
  }
  .focus-mode-label {
    color: var(--paper-ink-soft, #6a6055);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.68rem;
  }
  .focus-mode-toggle {
    display: inline-flex;
    border: 1.5px solid #8a4abd;
    border-radius: 4px;
    overflow: hidden;
    width: 100%;
  }
  .focus-mode-toggle button {
    font: inherit;
    flex: 1;
    padding: 0.22em 0.4em;
    border: none;
    background: var(--paper-bg, #f3ecd9);
    color: inherit;
    cursor: pointer;
    border-right: 1px solid #8a4abd;
    font-size: 0.82rem;
  }
  .focus-mode-toggle button:last-child { border-right: none; }
  .focus-mode-toggle button.active {
    background: #8a4abd;
    color: #f8f1de;
    font-weight: 600;
  }
  .focus-mode-toggle button:not(.active):hover {
    background: rgba(138, 74, 189, 0.15);
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
