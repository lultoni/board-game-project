<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getEngine, ActionKind, decodeAction, encodeBodyguardChoice } from "$lib/engine";
  import { decodeMailbox } from "$lib/engine/mailbox";
  import { buildEngineConfigJson } from "$lib/engine/config";
  import {
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateSnapshot,
  } from "$lib/engine/snapshot-validator";
  import { PRE_MADE_LOADOUTS } from "$lib/state/draft";
  import { t } from "$lib/state/i18n";
  import {
    match,
    modeFromSeats,
    resetMatchState,
    startTelemetrySession,
    recordPly,
    finalizeTelemetrySession,
    abandonTelemetrySession,
    networkLostTelemetrySession,
  } from "$lib/state/match-store.svelte";
  import { settings } from "$lib/state/settings.svelte";
  import {
    moveTargetsFor,
    movableSources,
    actableSources,
    findActionByKind,
    approachChoicesFor,
  } from "$lib/state/move-targets";
  import { skillTargetsFor, skillIsCastable, hasFocusModeChoice, hasRetargetVariants, hasSelfAndRetargetChoice, variantIsSelfCast, allyMoverCandidates, allyMoverDestinations, rawForAllyMove, type SkillVariant } from "$lib/state/skill-targets";
  import {
    isSelfCast,
    SKILLS,
    MODIFIER_FOCUS,
    MODIFIER_CHARGE,
  } from "$lib/engine/skills";
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import SkillInfoCard from "$lib/board/SkillInfoCard.svelte";
  import ConnectivityPill from "$lib/multiplayer/ConnectivityPill.svelte";
  import GraceBanner from "$lib/multiplayer/GraceBanner.svelte";
  import {
    mpState,
    onRawData as mpOnRawData,
    sendRaw as mpSendRaw,
    disconnect as mpDisconnect,
  } from "$lib/multiplayer.svelte";
  import { decodeMessageV2, encodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { createMpEngine, type MpEngineHandle, type Role, type SubmitResult } from "$lib/multiplayer-engine";
  import { sfx } from "$lib/audio/sfx";
  import { getTelemetryStore } from "$lib/storage";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";

  const mode = $derived(match.mode === "multiplayer" ? "multiplayer" : modeFromSeats(match.side));

  let bootError = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  /** True while a `stepAi` call is in flight. Drives the "AI is thinking…" overlay. */
  let aiThinking = $state(false);

  /** Role-aware ply renderer. Owns the effects/SFX pipeline, pieceIds,
   *  shakingSquares, effectQueue, and the deferred-skill-refresh state. Both
   *  /match/ and /replay/ create one of these. */
  let renderer = $state<PlyRenderer | null>(null);
  /** AIvAI playback control. When true, the AI loop auto-chains turns. */
  let aiAutoPlay = $state(true);
  /** Transient toast for export / sandbox feedback. Cleared by a timer. */
  let toast = $state<string>("");
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
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
  const currentSeatIsLocal = $derived.by(() => {
    if (match.mode !== "multiplayer") return true;
    if (!match.position) return false;
    const toMove = match.position.toMove; // 0 = P1, 1 = P2
    // Seat-by-localSeat, NOT by role: post-handoff the role flips but the
    // peer's board seat stays the same. See match-store.svelte.ts/localSeat.
    const seat = match.localSeat ?? (match.multiplayerRole === "host" ? 0 : 1);
    return toMove === seat;
  });

  // Track which squares used their Move action this phase. Stored as the
  // attacker's final square (target for plain Move, approach_sq for
  // Move-Attack). Cleared whenever phase or to-move flips.
  let usedThisPhase = $state<Set<number>>(new Set());
  let lastPhaseKey = $state<string>(""); // `${toMove}:${phase}` — phase boundary detector

  // Live drag state for the parent — Board owns the pointer mechanics and
  // pushes updates here so we can render path trail + hover ring.
  let dragSrc = $state<number | null>(null);
  let dragTrail = $state<number[]>([]);
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
  const interactive = $derived(
    ready
    && !busy
    && match.position?.gameResult === 0
    && !currentSeatIsAi
    && currentSeatIsLocal
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

  /** AI scheduler. Whenever it's an AI seat's turn and the loop is allowed
   *  to run, queue a `runAiStep()`. For HvAI this fires automatically on
   *  every AI ply. For AIvAI it chains turn-after-turn while `aiAutoPlay`
   *  is true; pausing freezes the loop after the in-flight call returns.
   *  Anchored on `phaseKey()` rather than `position` directly so a stable
   *  side+phase pair doesn't re-trigger when other position fields change. */
  let aiScheduled = false;
  $effect(() => {
    if (!ready) return;
    if (busy) return;
    if (match.mode === "sandbox") return;
    if (!currentSeatIsAi) return;
    // For AIvAI, gate on the play/pause toggle. For HvAI, always run.
    if (match.mode === "aivai" && !aiAutoPlay) return;
    if (aiScheduled) return;
    aiScheduled = true;
    // Defer just long enough for the UI to paint state + "thinking" pill
    // before we block on the engine. For AIvAI, honour the user-configured
    // step delay so a spectator can actually watch the game.
    const delay = match.mode === "aivai"
      ? Math.max(16, settings.aivaiStepDelayMs)
      : 30;
    setTimeout(() => {
      aiScheduled = false;
      void runAiStep();
    }, delay);
  });

  onMount(async () => {
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
      // Preserve multiplayer mode + role/code through the reset — the lobby
      // set these before navigating here and the reset would otherwise drop
      // mode back to "idle".
      const wasMultiplayer = match.mode === "multiplayer";
      const mpRole = match.multiplayerRole;
      const mpCode = match.multiplayerCode;
      // L8 — pre-made loadout path. Snapshot BEFORE resetMatchState() because
      // the reset clears `preMadeLoadoutId` (so stale ids from a prior match
      // can't leak in via direct navigation). `/setup/` writes the field on
      // commit; we read it here once and consume.
      const preMadeId = match.preMadeLoadoutId;
      resetMatchState();
      match.side = sideAtBoot;
      if (wasMultiplayer) {
        match.multiplayerRole = mpRole;
        match.multiplayerCode = mpCode;
      }
      if (preMadeId) {
        // Both sides play the same curated loadout — mirror match.
        const loadout = PRE_MADE_LOADOUTS[preMadeId];
        const configJson = buildEngineConfigJson(sideAtBoot);
        await eng.createEngineWithLoadouts(configJson, loadout, loadout);
        // Consume — re-entering /match/ later (e.g. a snapshot restore from
        // the inspector) should NOT re-create from loadouts.
        match.preMadeLoadoutId = null;
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
          multiplayerCode: mpCode,
          multiplayerRole: mpRole,
        });
      }
      // In multiplayer, subscribe to action messages from the peer and
      // apply them locally (with fromWire: true so we don't echo back).
      // Also handle the resume handshake: hosts validate incoming requests
      // against their MatchLog; joiners restore from the host's snapshot.
      const role: Role =
        wasMultiplayer && mpRole === "host"
          ? "host"
          : wasMultiplayer && mpRole === "joiner"
            ? "joiner"
            : "solo";
      mpEngine = createMpEngine(
        {
          role,
          phase: "play",
          matchId: match.telemetryMatchId,
          code: mpCode,
        },
        {
          eng,
          send: (m: WireMessageV2) => mpSendRaw(encodeMessageV2(m)),
          subscribe: (cb) => mpOnRawData((raw) => {
            const decoded = decodeMessageV2(raw);
            if (decoded) cb(decoded);
          }),
          onApplied: async (raw, _phase) => {
            // Skip when this raw was just applied via the local submit path —
            // applyRaw already snapshotted pre-state and rendered effects;
            // re-rendering here would double-flash and double-bump telemetry.
            if (raw === pendingLocalRaw) return;
            if (!renderer) return;
            // Drain any deferred Skill refresh from a prior remote-applied
            // skill — its setTimeout would otherwise fire after we render
            // this new action and clobber the post-state.
            renderer.drainPendingSkillRefresh();
            // The wrapper has already called tryApply, so the engine itself
            // is post-state. But `match.position` is the route's reactive
            // mirror — refresh() hasn't been called yet for this raw, so
            // match.position is still the PRE-apply snapshot. Capture from
            // it before refreshing, then run the full effect pipeline so the
            // non-acting peer plays sounds, spawns damage/death effects, and
            // updates usedThisPhase (greying-out parity).
            const pre = renderer.snapshotPreState(raw);
            await renderer.renderApplied(raw, pre);
            match.lastApplied = raw;
            afterApplied();
            // Host records the ply for telemetry. Joiner writes nothing
            // per the authoritative-host model.
            if (role === "host" || role === "solo") {
              await recordPly(eng!);
            }
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
      // Re-announce session on every PeerJS open while we're mounted.
      mpConnectedUnsub = $effect.root(() => {
        $effect(() => {
          if (mpState.status === "connected") mpEngine?.notifyConnectionOpen();
          else if (mpState.status === "disconnected") mpEngine?.notifyConnectionLost();
        });
      });
      // Subscribe to route-layer wire side-channel messages. The legacy
      // `bodyguard-prompt` variant is deprecated (the engine now owns the
      // bodyguard handoff via pending_bodyguard + STM flip) and is silently
      // ignored. Kept here as a one-release deprecation window so old hosts
      // talking to new joiners don't trip an unknown-message warning.
      mpRouteWireUnsub = mpOnRawData((raw) => {
        const decoded = decodeMessageV2(raw);
        if (!decoded) return;
        if (decoded.kind === "bodyguard-prompt") {
          // No-op — engine state drives the defender's chooser now.
        }
      });
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
  /** Disposer for the route-layer raw-data subscription (currently used for
   *  the `bodyguard-prompt` side message, which the engine wrapper ignores). */
  let mpRouteWireUnsub: (() => void) | null = null;
  /** The raw u32 currently being applied via the local submit path. Used to
   *  short-circuit the wrapper's onApplied callback so we don't double-render
   *  on host-side (host's submitAction applies AND fires onApplied). */
  let pendingLocalRaw: number | null = null;

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
        await renderer.applyAndRender(raw, async () => {
          await eng!.tryApply(raw);
        });
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

      // Snapshot pre-state for effect rendering BEFORE handing off to the
      // wrapper. On host/solo, submitAction will mutate the engine
      // synchronously; on joiner, the engine doesn't move until the host's
      // committed envelope lands, but we still want pre-state captured from
      // the moment of click.
      renderer.drainPendingSkillRefresh();
      const pre = renderer.snapshotPreState(raw);
      pendingLocalRaw = raw;
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
        pendingLocalRaw = null;
        return;
      }
      // For solo + host: submitAction has already advanced the engine.
      // For joiner: engine moved when the host's committed landed via the
      // wrapper's onApplied. Render here either way — the engine state is
      // authoritative regardless of who originated the action.
      await recordPly(eng);
      await renderer.renderApplied(raw, pre);
      match.lastApplied = raw;
      afterApplied();
      match.selection = null;
      pendingApproach = null;
      pendingDirection = null;
      focusModePref = "activation";
      focusAllyChosen = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      pendingLocalRaw = null;
      busy = false;
    }
  }

  /** Phase-boundary bookkeeping that used to live inside renderApplied. */
  function afterApplied(): void {
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

  /** Run one AI step for the side-to-move, then render the result. The engine
   *  applies the action atomically inside stepAi, so we snapshot pre-state
   *  from the current `match.position` BEFORE the call. */
  async function runAiStep(): Promise<void> {
    if (!eng || !renderer || busy) return;
    if (match.mode === "sandbox") return;
    if (match.mode === "multiplayer") return;
    if (!match.position) return;
    if (match.position.gameResult !== 0) return;
    busy = true;
    aiThinking = true;
    try {
      // Drain any deferred Skill refresh before snapshotting pre-state — see
      // applyRaw for rationale.
      renderer.drainPendingSkillRefresh();
      const result = await eng.stepAi();
      const raw = result.appliedAction;
      if (raw === 0) {
        // AI returned no move (terminal or error). Refresh and bail.
        await refresh();
        return;
      }
      // Persist AI ply telemetry. Sandbox is gated above (early return).
      await recordPly(eng);
      // The engine has advanced, but `match.position` (the renderer's
      // positionSink) is still the PRE-step snapshot — refresh() hasn't run
      // yet. snapshotPreState reads from there, so this is safe.
      const pre = renderer.snapshotPreState(raw);
      await renderer.renderApplied(raw, pre);
      match.lastApplied = raw;
      afterApplied();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      aiThinking = false;
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
        applyRaw(encodeBodyguardChoice(0));
        return;
      }
      const k = pendingBodyguard.eligible.indexOf(sq);
      if (k >= 0) {
        applyRaw(encodeBodyguardChoice(k + 1));
        return;
      }
      // Click anywhere else: ignore. Defender must pick (decline or redirect).
      return;
    }

    if (pendingApproach) {
      if (pendingApproach.approaches.includes(sq)) {
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
    try {
      // Capture pre-sandbox snapshot BEFORE flipping mode — otherwise an
      // in-flight AI scheduler tick could mutate state between capture and
      // mode-flip.
      const snap = await eng.snapshotJson();
      clearAllPickers();
      // Pause telemetry while in sandbox. The session is "abandoned" from
      // storage's perspective, but the per-ply records up to this point
      // remain on disk. Exiting sandbox does NOT resume the old session —
      // the user would have already seen analysis-mode entry as a fork.
      await abandonTelemetrySession(eng);
      match.trueSnapshotJson = snap;
      match.sandboxMovesApplied = 0;
      match.mode = "sandbox";
    } finally {
      busy = false;
    }
  }

  async function exitSandbox(): Promise<void> {
    if (!eng || busy || aiThinking) return;
    if (match.mode !== "sandbox" || !match.trueSnapshotJson) return;
    if (match.sandboxMovesApplied > 0) {
      const msg = t("sandbox.confirmDiscard", { n: match.sandboxMovesApplied });
      if (!window.confirm(msg)) return;
    }
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
      match.mode = modeFromSeats(match.side);
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

  // Page-hide handler: fires when the tab is hidden, navigated away, or closed.
  // Unlike `beforeunload`, this is the last reliable hook for fire-and-forget
  // work in modern browsers — but async tasks still get cut. We kick off the
  // teardown writes here so they have the best chance of landing before the
  // page is discarded. onDestroy keeps the same logic for client-side nav
  // (where awaits resolve normally).
  function pageHideHandler(): void {
    if (match.telemetryMatchId && !match.telemetryFinalised) {
      if (match.mode === "multiplayer") {
        void networkLostTelemetrySession(eng ?? undefined);
      } else {
        void abandonTelemetrySession(eng ?? undefined);
      }
    }
  }

  onMount(() => {
    if (typeof window !== "undefined") {
      window.addEventListener("beforeunload", beforeUnloadGuard);
      window.addEventListener("pagehide", pageHideHandler);
    }
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("beforeunload", beforeUnloadGuard);
      window.removeEventListener("pagehide", pageHideHandler);
    }
    // If the user leaves /match/ before a natural end, mark the session
    // abandoned. Per-ply records on disk remain — replay still works.
    // In multiplayer mode the row is marked `mid-match-network-lost` so the
    // lobby's recent-sessions card list can pick it up for resume.
    // (On tab-close paths, pagehide already kicked these off — the helpers
    // are idempotent via the `telemetryMatchId` null-out at entry.)
    if (match.telemetryMatchId && !match.telemetryFinalised) {
      if (match.mode === "multiplayer") {
        void networkLostTelemetrySession(eng ?? undefined);
      } else {
        void abandonTelemetrySession(eng ?? undefined);
      }
    }
    if (mpEngine) {
      mpEngine.dispose();
      mpEngine = null;
    }
    if (mpConnectedUnsub) {
      mpConnectedUnsub();
      mpConnectedUnsub = null;
    }
    if (mpRouteWireUnsub) {
      mpRouteWireUnsub();
      mpRouteWireUnsub = null;
    }
    // Leaving /match/ before a natural end means we're going back to the
    // lobby (or home). Tear down the PeerJS connection so the OTHER peer
    // sees us drop immediately — otherwise the joiner-side `mpState` keeps
    // pinging the host from a stale page, and the host's heartbeat never
    // ages out (we observed the host pill staying "live" indefinitely while
    // the joiner sat on the home screen). Skip when telemetry has finalised
    // (natural game-end) — in that case both peers leave together and
    // resume isn't needed. Mirror's /draft/'s onDestroy teardown.
    if (match.mode === "multiplayer" && !match.telemetryFinalised) {
      mpDisconnect();
    }
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

<main class:sandbox-mode={match.mode === "sandbox"}>
  <header>
    <p class="back"><a href="../">← back</a></p>
    <h1>{t("match.title", { mode })}</h1>
    {#if match.mode === "multiplayer"}
      <ConnectivityPill />
    {/if}
  </header>

  {#if match.mode === "multiplayer"}
    <GraceBanner {eng} {mpEngine} />
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
    <section class="board-wrap">
      <div class="board-stack">
        <Board
          position={match.position}
          pieceIds={renderer?.pieceIds ?? new Map()}
          selection={match.selection}
          moveTargets={armedSkill ? armedSkillTargets : moveTargets.squares}
          {selectable}
          draggable={movable}
          usedSquares={usedThisPhase}
          shakingSquares={renderer?.shakingSquares ?? new Set()}
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
          {dragLanding}
          onApproachChoice={(ap) => {
            if (pendingApproach) {
              const target = pendingApproach.target;
              pendingApproach = null;
              commitMoveTargetApproach(target, ap);
            }
          }}
        />
        {#if renderer}
          <EffectsLayer viewBox={800} wheelPad={60} bind:queue={renderer.effectQueue} />
        {/if}
        {#if aiThinking}
          <div class="thinking" role="status" aria-live="polite">
            <span class="spinner" aria-hidden="true"></span>
            <span class="thinking-label">{t("controls.aiThinking")}</span>
          </div>
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
      {#if pendingApproach}
        <p class="hint">choose the path the attacker takes — click a highlighted square, or press Esc to cancel</p>
      {/if}
      {#if pendingBodyguard}
        <p class="hint">bodyguard: defender may redirect the hit — click the red defender to take the hit, or a blue guard to intercept</p>
      {/if}
      {#if armedNeedsAllyPick && focusAllyChosen === null}
        <p class="hint">pick an adjacent ally to channel onto, then choose where they move</p>
      {/if}
      {#if armedNeedsAllyPick && focusAllyChosen !== null}
        <p class="hint">choose the destination for the chosen ally — click another ally to switch</p>
      {/if}
      {#if pendingDirection}
        <p class="hint">choose a push direction — click an arrow, or press Esc to cancel</p>
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
            >Range (+1 tile)</button>
            <button
              type="button"
              role="radio"
              aria-checked={focusModePref === "effect"}
              class:active={focusModePref === "effect"}
              onclick={() => (focusModePref = "effect")}
            >Effect (push 2)</button>
          </div>
        </div>
      {/if}
      {#if armedHasRetargetChoice && !pendingDirection}
        <div class="focus-mode">
          <span class="focus-mode-label">Focus channels onto:</span>
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
    </section>

    <aside class="hud">
      <div>
        <span class="label">round</span>
        <span class="value">{match.position?.roundNumber ?? "–"}</span>
      </div>
      <div>
        <span class="label">to move</span>
        <span class="value">{match.position?.toMove === 0 ? "P1" : "P2"}</span>
      </div>
      <div>
        <span class="label">phase</span>
        <span class="value">{inMovePhase ? "move" : "skill"}</span>
      </div>
      <div>
        <span class="label">actions</span>
        <span class="value">{match.position?.actionsRemaining ?? "–"}</span>
      </div>
      <div>
        <span class="label">P1 $</span>
        <span class="value">{match.position?.p1Money ?? "–"}</span>
      </div>
      <div>
        <span class="label">P2 $</span>
        <span class="value">{match.position?.p2Money ?? "–"}</span>
      </div>
      <div>
        <span class="label">legal</span>
        <span class="value">{match.legal.length}</span>
      </div>
      <div class="actions">
        {#if match.mode === "aivai"}
          <button
            type="button"
            disabled={busy || match.position?.gameResult !== 0}
            onclick={() => (aiAutoPlay = !aiAutoPlay)}
          >{aiAutoPlay ? t("controls.pause") : t("controls.play")}</button>
          <button
            type="button"
            disabled={busy || aiAutoPlay || match.position?.gameResult !== 0}
            onclick={() => void runAiStep()}
          >{t("controls.step")}</button>
        {:else}
          <button
            type="button"
            disabled={!interactive || endPhaseAction === null}
            onclick={endPhase}
          >{t("controls.endPhase")}</button>
        {/if}
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
            disabled={busy || aiThinking}
            onclick={() => void (match.mode === "sandbox" ? exitSandbox() : enterSandbox())}
          >{match.mode === "sandbox" ? t("controls.exitSandbox") : t("controls.sandbox")}</button>
        </div>
      </div>
    </aside>

    {#if toast}
      <div class="toast" role="status" aria-live="polite">{toast}</div>
    {/if}

    {#if match.position?.gameResult !== 0}
      <p class="result">
        {match.position?.gameResult === 1
          ? t("result.p1Wins")
          : match.position?.gameResult === 2
            ? t("result.p2Wins")
            : t("result.draw")}
      </p>
    {/if}
  {/if}
</main>

<style>
  main {
    max-width: 960px;
    margin: 0 auto;
    padding: 0.6rem 1rem 2rem;
    position: relative;
  }
  main.sandbox-mode {
    box-shadow:
      inset 0 0 0 4px rgba(56, 178, 255, 0.85),
      inset 0 0 24px 8px rgba(56, 178, 255, 0.30);
    animation: sandbox-pulse 2.4s ease-in-out infinite;
    border-radius: 6px;
  }
  @keyframes sandbox-pulse {
    0%, 100% {
      box-shadow:
        inset 0 0 0 4px rgba(56, 178, 255, 0.85),
        inset 0 0 24px 8px rgba(56, 178, 255, 0.25);
    }
    50% {
      box-shadow:
        inset 0 0 0 4px rgba(56, 178, 255, 1.00),
        inset 0 0 32px 12px rgba(56, 178, 255, 0.45);
    }
  }
  .export-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-top: 0.6rem;
    padding-top: 0.6rem;
    border-top: 1px solid rgba(127, 127, 127, 0.25);
  }
  .export-group button { flex: 1 1 auto; }
  .sandbox-toggle { flex-basis: 100% !important; }
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
  header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.8rem;
  }
  header h1 {
    font-size: 1.4rem;
    margin: 0;
  }
  .back a { text-decoration: none; }
  .board-wrap {
    max-width: 720px;
    margin: 0 auto;
  }
  .board-stack {
    position: relative;
    /* No padding/margin hack here — the wheel's spillover area is now
       baked into the SVG's own viewBox (see WHEEL_PAD in Board.svelte),
       so the SVG element's hit-box naturally contains it without
       shadowing sibling controls (e.g. HUD buttons below). */
  }
  .info-anchor {
    position: absolute;
    top: 0;
    left: calc(100% + 1rem);
    z-index: 5;
    pointer-events: none;
  }
  @media (max-width: 980px) {
    .info-anchor {
      position: static;
      margin-top: 0.6rem;
    }
  }
  .hint {
    margin: 0.4rem 0 0;
    text-align: center;
    font-size: 0.9rem;
    color: var(--paper-ink-soft, #6a6055);
  }
  .focus-mode {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    margin: 0.5rem 0 0;
    font-size: 0.85rem;
  }
  .focus-mode-label {
    color: var(--paper-ink-soft, #6a6055);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.72rem;
  }
  .focus-mode-toggle {
    display: inline-flex;
    border: 1.5px solid #8a4abd;
    border-radius: 5px;
    overflow: hidden;
  }
  .focus-mode-toggle button {
    font: inherit;
    padding: 0.25em 0.65em;
    border: none;
    background: var(--paper-bg, #f3ecd9);
    color: inherit;
    cursor: pointer;
    border-right: 1px solid #8a4abd;
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
  .hud {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.6rem 1.4rem;
    margin: 1rem auto 0;
    max-width: 720px;
    padding: 0.6em 0.9em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
  }
  .hud .label {
    display: block;
    font-size: 0.72rem;
    color: var(--paper-ink-soft, #6a6055);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .hud .value {
    font-weight: 600;
    font-size: 1rem;
  }
  .hud .actions {
    margin-left: auto;
  }
  .hud button {
    font: inherit;
    padding: 0.4em 0.9em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 5px;
    background: var(--paper-bg, #f3ecd9);
    color: inherit;
    cursor: pointer;
  }
  .hud button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .hud button:not(:disabled):hover {
    background: var(--paper-square-light, #ece2c8);
  }
  .result {
    max-width: 720px;
    margin: 1rem auto 0;
    padding: 0.6em 0.9em;
    border: 1.5px solid var(--accent, #c79b3a);
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
    font-weight: 600;
    text-align: center;
  }
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
  .err > span {
    flex: 1 1 auto;
  }
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
  .err-dismiss:hover {
    background: rgba(169, 75, 59, 0.12);
  }
  .thinking {
    position: absolute;
    top: 0.6rem;
    right: 0.6rem;
    z-index: 6;
    display: inline-flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.4em 0.7em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 999px;
    background: var(--paper-bg, #f3ecd9);
    font-size: 0.85rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.08);
    pointer-events: none;
  }
  .thinking-label {
    color: var(--paper-ink, #1c1a17);
  }
  .spinner {
    width: 0.9em;
    height: 0.9em;
    border: 2px solid var(--paper-line, #c7b894);
    border-top-color: var(--paper-ink, #1c1a17);
    border-radius: 50%;
    animation: spinner-rot 0.9s linear infinite;
  }
  @keyframes spinner-rot {
    to { transform: rotate(1turn); }
  }
  @media (prefers-reduced-motion: reduce) {
    .spinner { animation-duration: 2.4s; }
  }
</style>
