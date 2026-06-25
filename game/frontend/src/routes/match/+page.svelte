<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getEngine, ActionKind, decodeAction } from "$lib/engine";
  import { decodeMailbox } from "$lib/engine/mailbox";
  import { buildEngineConfigJson } from "$lib/engine/config";
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
  import { bodyguardGuardsFor } from "$lib/state/bodyguard";
  import { skillTargetsFor, skillIsCastable, hasFocusModeChoice, hasRetargetVariants, type SkillVariant } from "$lib/state/skill-targets";
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
  } from "$lib/multiplayer.svelte";
  import { decodeMessageV2, encodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { createMpEngine, type MpEngineHandle, type Role, type SubmitResult } from "$lib/multiplayer-engine";
  import type { Effect } from "$lib/viz/effects";
  import { sfx } from "$lib/audio/sfx";
  import { getTelemetryStore } from "$lib/storage";

  const mode = $derived(match.mode === "multiplayer" ? "multiplayer" : modeFromSeats(match.side));

  let bootError = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  /** True while a `stepAi` call is in flight. Drives the "AI is thinking…" overlay. */
  let aiThinking = $state(false);
  /** AIvAI playback control. When true, the AI loop auto-chains turns. */
  let aiAutoPlay = $state(true);
  let lastAppliedPair = $state<{ src: number; target: number } | null>(null);
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
    if (match.multiplayerRole === "host") return toMove === 0;
    if (match.multiplayerRole === "joiner") return toMove === 1;
    return false;
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

  // Bodyguard chooser state. After the attacker commits a Move-Attack on a
  // Champion/King with eligible adjacent Guards, we PAUSE before applying.
  // The defender's seat then clicks one of: the defender (take the hit) or
  // an eligible Guard square (redirect). Each click maps to a pre-decoded
  // raw action; we just call applyRaw with the chosen one.
  let pendingBodyguard = $state<{
    /** Defender square (Champion/King). */
    target: number;
    /** Action raw for choice_idx = 0 (defender takes hit). */
    defenderRaw: number;
    /** Eligible Guard squares (in canonical ascending order), each paired
     *  with the raw action for the redirect variant pointing at it. */
    redirects: { guardSq: number; raw: number }[];
  } | null>(null);

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

  // Stable per-piece identity. The engine has no notion of piece IDs; we
  // derive them from bitboards + mailbox. Without a stable key the
  // `{#each pieces}` block remounts the SVG element on every move, which
  // kills the CSS slide transition. We carry a Map<square, id> ourselves
  // and rewrite it on every applied action: src → dest (and clear the
  // defender's id on kill). On every refresh we reconcile against the
  // engine state: any square the engine has a piece on but we don't, we
  // assign a fresh id (covers boot, snapshot restore, edge cases).
  let pieceIds = $state<Map<number, number>>(new Map());
  let nextPieceId = 1;

  function reconcilePieceIds(): void {
    if (!match.position) return;
    const occupied = new Set<number>();
    const p1 = match.position.bitboards[0];
    const p2 = match.position.bitboards[1];
    const both = p1 | p2;
    for (let sq = 0; sq < 64; sq++) {
      if (((both >> BigInt(sq)) & 1n) === 1n) occupied.add(sq);
    }
    // Drop ids for vacated squares we didn't already transfer.
    for (const sq of pieceIds.keys()) {
      if (!occupied.has(sq)) pieceIds.delete(sq);
    }
    // Assign ids to newly seen occupied squares.
    for (const sq of occupied) {
      if (!pieceIds.has(sq)) pieceIds.set(sq, nextPieceId++);
    }
    pieceIds = new Map(pieceIds); // trigger reactivity
  }

  // Squares with pieces that are mid-hit-shake. Added on damage, cleared
  // after the CSS shake animation completes (~320ms).
  let shakingSquares = $state<Set<number>>(new Set());

  function triggerShake(sq: number) {
    shakingSquares = new Set([...shakingSquares, sq]);
    setTimeout(() => {
      shakingSquares = new Set([...shakingSquares].filter((s) => s !== sq));
    }, 340);
  }

  // Canvas effects queue; bound into EffectsLayer.
  let effectQueue: Effect[] = $state([]);

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
  const interactive = $derived(ready && !busy && match.position?.gameResult === 0 && !currentSeatIsAi && currentSeatIsLocal);

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

  // Target set for the currently-armed skill. Filtered by focusModePref when
  // both interpretations exist; otherwise unfiltered.
  const armedSkillTargets = $derived.by(() => {
    if (!armedSkill) return new Set<number>();
    const ts = skillTargetsFor(match.legal, armedSkill.square, armedSkill.skillId);
    if (!armedHasFocusModeChoice) return ts.squares;
    // Filter by focus-mode preference. `focusMode=true` → effect-buff variant.
    const wantEffect = focusModePref === "effect";
    const filtered = new Set<number>();
    for (const [tgt, vs] of ts.variantsByTarget) {
      if (vs.some((v) => v.focusMode === wantEffect)) filtered.add(tgt);
    }
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
        await eng.restoreFromSnapshot(pending);
      } else {
        await eng.createEngine();
      }
      await refresh();
      reconcilePieceIds();
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
            // Remote-driven apply: pre-state was the engine's state BEFORE
            // the wrapper called tryApply. We don't have that snapshot any
            // more (engine already moved), so we render against the current
            // mailbox — visual fidelity for remote ply is best-effort. The
            // important thing is that match.position / match.legal refresh.
            await refresh();
            reconcilePieceIds();
            lastAppliedPair = null;
            match.lastApplied = raw;
            const k = phaseKey();
            if (k !== lastPhaseKey) {
              usedThisPhase = new Set();
              lastPhaseKey = k;
            }
            // Host records the ply for telemetry. Joiner writes nothing
            // per the authoritative-host model.
            if (role === "host" || role === "solo") {
              await recordPly(eng!);
            }
          },
          onSnapshotApplied: async () => {
            await refresh();
            reconcilePieceIds();
            lastPhaseKey = phaseKey();
          },
          onPhaseChange: async () => { /* no-op in /match/ */ },
          onCheatDetected: () => {
            bootError = "anti-cheat: opponent's engine disagreed";
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
    match.position = await eng.positionView();
    match.legal = await eng.legalActions();
  }

  /** Fetch fresh position+legal from the engine WITHOUT assigning to
   *  `match.position` / `match.legal` yet. Lets the caller stage UI changes
   *  (impact effects on the pre-state board) before flipping the rendered
   *  state to the new one. */
  async function fetchFreshState() {
    if (!eng) return null;
    const pos = await eng.positionView();
    const legal = await eng.legalActions();
    return { pos, legal };
  }

  // Build the walked-square path for dust. For a plain Move (no approach),
  // we use src→target. For a Move-Attack that didn't kill, src→approach (attacker
  // stops on approach_sq). For a Move-Attack that killed the defender, the
  // attacker advances into the now-empty target tile: src→approach→target
  // (or just src→target when approach == src, i.e. a speed-1 adjacent attack).
  function walkedPath(
    decoded: ReturnType<typeof decodeAction>,
    killed: boolean,
  ): number[] {
    if (decoded.kind !== ActionKind.Move) return [];
    const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
    if (decoded.hasAux && killed) {
      // Attacker walked all the way to the defender's tile.
      if (approach === decoded.src) return [decoded.src, decoded.target];
      return [decoded.src, approach, decoded.target];
    }
    if (approach === decoded.src) return [decoded.src];
    return [decoded.src, approach];
  }

  function pushDamageEffect(targetSq: number, before: number, after: number) {
    const dmg = before - after;
    if (dmg <= 0) return;
    const now = performance.now();
    effectQueue.push({ kind: "impact", at: targetSq, startedAt: now });
    effectQueue.push({ kind: "damageNumber", at: targetSq, amount: dmg, startedAt: now + 80 });
    triggerShake(targetSq);
    sfx.play("damage");
  }

  /** Chebyshev distance between two squares (max of rank/file deltas). */
  function chebyshev(a: number, b: number): number {
    const dx = Math.abs((a & 7) - (b & 7));
    const dy = Math.abs(((a >> 3) & 7) - ((b >> 3) & 7));
    return Math.max(dx, dy);
  }

  /** Pre→post diff summary for a skill action. */
  interface SkillDiff {
    /** Squares where a piece remained at the same square (delta in stats). */
    stayed: number[];
    /** Vacated→arrived pairings (a piece relocated). Includes Chebyshev dist. */
    moves: { from: number; to: number; dist: number }[];
    /** Unpaired vacated squares (a piece died here). */
    deaths: number[];
  }

  /** Pair vacated squares with arrived squares by nearest Chebyshev. */
  function diffSkillMailbox(pre: Uint16Array, post: Uint16Array): SkillDiff {
    const stayed: number[] = [];
    const vacated: number[] = [];
    const arrived: number[] = [];
    for (let sq = 0; sq < 64; sq++) {
      const a = decodeMailbox(pre[sq]);
      const b = decodeMailbox(post[sq]);
      if (a.empty && b.empty) continue;
      if (!a.empty && !b.empty) { stayed.push(sq); continue; }
      if (!a.empty && b.empty) { vacated.push(sq); continue; }
      if (a.empty && !b.empty) { arrived.push(sq); continue; }
    }
    const moves: { from: number; to: number; dist: number }[] = [];
    const usedV = new Set<number>();
    for (const dst of arrived) {
      let bestV = -1;
      let bestD = Infinity;
      for (const v of vacated) {
        if (usedV.has(v)) continue;
        const d = chebyshev(v, dst);
        if (d < bestD) { bestD = d; bestV = v; }
      }
      if (bestV >= 0) {
        usedV.add(bestV);
        moves.push({ from: bestV, to: dst, dist: bestD });
      }
    }
    const deaths: number[] = vacated.filter((v) => !usedV.has(v));
    return { stayed, moves, deaths };
  }

  /** Emit the impact-class effects (damage, heal, armor) for stayed pieces
   *  AND for relocated pieces (damage delta computed across the pairing).
   *  Returns whether any such event fired. */
  function emitImpactEvents(pre: Uint16Array, post: Uint16Array, diff: SkillDiff): boolean {
    const now = performance.now();
    let fired = false;
    const visit = (preSq: number, postSq: number) => {
      const a = decodeMailbox(pre[preSq]);
      const b = decodeMailbox(post[postSq]);
      if (a.empty || b.empty) return;
      const hpDelta = b.hp - a.hp;
      const arDelta = b.armor - a.armor;
      // Render damage/heal on the piece's POST-MOVE square so the number
      // travels with the relocated piece.
      const renderSq = postSq;
      if (hpDelta < 0) {
        pushDamageEffect(renderSq, a.hp + a.armor, b.hp + b.armor);
        fired = true;
      } else if (hpDelta > 0) {
        effectQueue.push({ kind: "heal", at: renderSq, amount: hpDelta, startedAt: now });
        sfx.play("heal");
        fired = true;
      }
      if (arDelta > 0) {
        effectQueue.push({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now + 40 });
        sfx.play("armor");
        fired = true;
      } else if (arDelta < 0 && hpDelta === 0) {
        effectQueue.push({ kind: "armor", at: renderSq, amount: arDelta, startedAt: now });
        sfx.play("armorBreak");
        fired = true;
      }
    };
    for (const sq of diff.stayed) visit(sq, sq);
    for (const m of diff.moves) visit(m.from, m.to);
    return fired;
  }

  /** Emit dust + move-SFX for each relocation and death-flash for each
   *  unpaired vacated square. Transfers piece ids along each move. */
  function emitRelocationAndDeathEvents(pre: Uint16Array, diff: SkillDiff) {
    const now = performance.now();
    for (const m of diff.moves) {
      const path = straightPath(m.from, m.to);
      if (path.length >= 2) {
        effectQueue.push({ kind: "dust", path, startedAt: now });
      }
      const id = pieceIds.get(m.from);
      if (id !== undefined) {
        pieceIds.delete(m.from);
        pieceIds.set(m.to, id);
      }
      sfx.play("move", { tiles: m.dist });
    }
    for (const v of diff.deaths) {
      const a = decodeMailbox(pre[v]);
      const dmg = a.hp + a.armor;
      effectQueue.push({ kind: "impact", at: v, startedAt: now });
      if (dmg > 0) {
        effectQueue.push({ kind: "damageNumber", at: v, amount: dmg, startedAt: now + 80 });
      }
      triggerShake(v);
      pieceIds.delete(v);
      sfx.play("death");
    }
  }

  /** True iff diff contains any relocations or deaths. */
  function hasRelocationOrDeath(diff: SkillDiff): boolean {
    return diff.moves.length > 0 || diff.deaths.length > 0;
  }

  /** Straight-line path of squares from `from` to `to` along a queen-ray
   *  direction. Returns [from, …, to] inclusive. If they're not on a queen
   *  ray (shouldn't happen for skill relocations), returns [from, to]. */
  function straightPath(from: number, to: number): number[] {
    const fF = from & 7, fR = (from >> 3) & 7;
    const tF = to & 7, tR = (to >> 3) & 7;
    const dF = Math.sign(tF - fF);
    const dR = Math.sign(tR - fR);
    const steps = Math.max(Math.abs(tF - fF), Math.abs(tR - fR));
    if (steps === 0) return [from];
    // Sanity: only emit a ray-walk if Chebyshev matches both axes.
    const okF = dF === 0 || Math.abs(tF - fF) === steps;
    const okR = dR === 0 || Math.abs(tR - fR) === steps;
    if (!okF || !okR) return [from, to];
    const out: number[] = [];
    for (let i = 0; i <= steps; i++) {
      const f = fF + dF * i;
      const r = fR + dR * i;
      out.push((r << 3) | f);
    }
    return out;
  }

  type BodyguardSnapshot = { sq: number; entry: ReturnType<typeof decodeMailbox> }[];

  /** Snapshot the pre-state slice we need to render `raw`'s effects. */
  function snapshotPreState(raw: number): {
    preFull: Uint16Array | null;
    preTarget: ReturnType<typeof decodeMailbox> | null;
    preBodyguard: BodyguardSnapshot;
  } {
    const decoded = decodeAction(raw);
    const preMailbox = match.position?.mailbox;
    const preFull: Uint16Array | null = preMailbox ? new Uint16Array(preMailbox) : null;
    const preTarget = preMailbox ? decodeMailbox(preMailbox[decoded.target]) : null;
    const preBodyguard: BodyguardSnapshot = [];
    if (preMailbox && decoded.kind === ActionKind.Move && decoded.hasAux) {
      const tFile = decoded.target & 7;
      const tRank = (decoded.target >> 3) & 7;
      for (let df = -1; df <= 1; df++) {
        for (let dr = -1; dr <= 1; dr++) {
          if (df === 0 && dr === 0) continue;
          const nf = tFile + df, nr = tRank + dr;
          if (nf < 0 || nf > 7 || nr < 0 || nr > 7) continue;
          const sq = (nr << 3) | nf;
          const ent = decodeMailbox(preMailbox[sq]);
          if (!ent.empty) preBodyguard.push({ sq, entry: ent });
        }
      }
    }
    return { preFull, preTarget, preBodyguard };
  }

  /** Render effects for an action that the engine has ALREADY applied.
   *  Caller is responsible for: snapshotting pre-state, calling tryApply
   *  (or stepAi), and clearing UI affordances (selection, pending choosers)
   *  after this completes. */
  async function renderApplied(
    raw: number,
    preFull: Uint16Array | null,
    preTarget: ReturnType<typeof decodeMailbox> | null,
    preBodyguard: BodyguardSnapshot,
  ): Promise<void> {
    const decoded = decodeAction(raw);
    if (decoded.kind === ActionKind.Skill) {
      sfx.play("skillFire");
    } else if (decoded.kind === ActionKind.EndPhase) {
      sfx.play("phaseEnd");
    }

    // Transfer piece ids along the move BEFORE refresh, so the new
    // bitboards see a piece with a stable identity at the destination.
    if (decoded.kind === ActionKind.Move) {
      const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
      const srcId = pieceIds.get(decoded.src);
      if (srcId !== undefined) {
        pieceIds.delete(decoded.src);
        pieceIds.set(approach, srcId);
      }
    }

    // For Move / EndPhase we refresh immediately. For Skill we DEFER the
    // state flip — see the skill-effects block below.
    if (decoded.kind !== ActionKind.Skill) {
      await refresh();
    }

    let killed = false;
    if (decoded.kind === ActionKind.Move && decoded.hasAux && preTarget && match.position) {
      const postTarget = decodeMailbox(match.position.mailbox[decoded.target]);
      const approach = decoded.auxSq;
      if (!postTarget.empty && approach !== decoded.target) {
        const postApproach = decodeMailbox(match.position.mailbox[approach]);
        if (postApproach.empty) {
          killed = true;
          const aid = pieceIds.get(approach);
          if (aid !== undefined) {
            pieceIds.delete(approach);
            pieceIds.set(decoded.target, aid);
          }
        }
      } else if (!postTarget.empty && approach === decoded.target) {
        killed = true;
      }
    }

    if (decoded.kind !== ActionKind.Skill) {
      reconcilePieceIds();
    }

    if (decoded.kind === ActionKind.Move) {
      const path = walkedPath(decoded, killed);
      if (path.length >= 2) {
        effectQueue.push({ kind: "dust", path, startedAt: performance.now() });
      }
      const finalAttackerSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      const tiles = chebyshev(decoded.src, finalAttackerSq);
      sfx.play(decoded.hasAux ? "attack" : "move", { tiles });
      if (killed) sfx.play("death");
      if (decoded.hasAux && preTarget && match.position) {
        const postTarget = decodeMailbox(match.position.mailbox[decoded.target]);
        const before = preTarget.hp + preTarget.armor;
        const after = killed ? 0 : postTarget.hp + postTarget.armor;
        if (after < before) {
          pushDamageEffect(decoded.target, before, after);
        } else {
          for (const bg of preBodyguard) {
            const post = decodeMailbox(match.position.mailbox[bg.sq]);
            const bgBefore = bg.entry.hp + bg.entry.armor;
            const bgAfter = post.hp + post.armor;
            if (bgAfter < bgBefore) {
              pushDamageEffect(bg.sq, bgBefore, bgAfter);
              break;
            }
          }
        }
      }
      const finalSq = decoded.hasAux
        ? (killed ? decoded.target : decoded.auxSq)
        : decoded.target;
      usedThisPhase = new Set([...usedThisPhase, finalSq]);
    }

    const RELOC_DELAY_MS = 260;
    if (decoded.kind === ActionKind.Skill && preFull) {
      const fresh = await fetchFreshState();
      if (!fresh) return;
      const newMailbox = fresh.pos.mailbox;
      const diff = diffSkillMailbox(preFull, newMailbox);
      const hasReloc = hasRelocationOrDeath(diff);
      const impactFired = emitImpactEvents(preFull, newMailbox, diff);
      if (hasReloc && impactFired) {
        setTimeout(() => {
          match.position = fresh.pos;
          match.legal = fresh.legal;
          emitRelocationAndDeathEvents(preFull, diff);
          reconcilePieceIds();
        }, RELOC_DELAY_MS);
      } else {
        match.position = fresh.pos;
        match.legal = fresh.legal;
        if (hasReloc) emitRelocationAndDeathEvents(preFull, diff);
        reconcilePieceIds();
      }
    }

    lastAppliedPair =
      decoded.kind === ActionKind.Move || decoded.kind === ActionKind.Skill
        ? { src: decoded.src, target: decoded.target }
        : null;
    match.lastApplied = raw;

    const k = phaseKey();
    if (k !== lastPhaseKey) {
      usedThisPhase = new Set();
      lastPhaseKey = k;
    }
  }

  async function applyRaw(raw: number) {
    if (!eng || busy) return;
    busy = true;
    try {
      // Sandbox bypasses the wrapper — the user is exploring locally and
      // sandbox moves must NOT echo to a peer or get logged as match plies.
      if (match.mode === "sandbox") {
        const { preFull, preTarget, preBodyguard } = snapshotPreState(raw);
        await eng.tryApply(raw);
        match.sandboxMovesApplied += 1;
        await renderApplied(raw, preFull, preTarget, preBodyguard);
        match.selection = null;
        pendingApproach = null;
        pendingDirection = null;
        focusModePref = "activation";
        return;
      }

      // Snapshot pre-state for effect rendering BEFORE handing off to the
      // wrapper. On host/solo, submitAction will mutate the engine
      // synchronously; on joiner, the engine doesn't move until the host's
      // committed envelope lands, but we still want pre-state captured from
      // the moment of click.
      const { preFull, preTarget, preBodyguard } = snapshotPreState(raw);
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
      // For solo + host: engine has already moved; render and record.
      // For joiner: engine moved when the host's committed landed via the
      // wrapper's onApplied. Render here either way — the engine state is
      // authoritative regardless of who originated the action.
      await recordPly(eng);
      await renderApplied(raw, preFull, preTarget, preBodyguard);
      match.selection = null;
      pendingApproach = null;
      pendingDirection = null;
      focusModePref = "activation";
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      pendingLocalRaw = null;
      busy = false;
    }
  }

  /** Run one AI step for the side-to-move, then render the result. The engine
   *  applies the action atomically inside stepAi, so we snapshot pre-state
   *  from the current `match.position` BEFORE the call. */
  async function runAiStep(): Promise<void> {
    if (!eng || busy) return;
    if (match.mode === "sandbox") return;
    if (match.mode === "multiplayer") return;
    if (!match.position) return;
    if (match.position.gameResult !== 0) return;
    busy = true;
    aiThinking = true;
    try {
      // Snapshot from the pre-call position (we don't know the action yet,
      // but the mailbox + adjacency snapshot is action-agnostic).
      const preMailbox = match.position.mailbox;
      const preFull = new Uint16Array(preMailbox);
      const result = await eng.stepAi();
      const raw = result.appliedAction;
      if (raw === 0) {
        // AI returned no move (terminal or error). Refresh and bail.
        await refresh();
        return;
      }
      // Persist AI ply telemetry. Sandbox is gated above (early return).
      await recordPly(eng);
      const decoded = decodeAction(raw);
      const preTarget = decodeMailbox(preFull[decoded.target]);
      const preBodyguard: BodyguardSnapshot = [];
      if (decoded.kind === ActionKind.Move && decoded.hasAux) {
        const tFile = decoded.target & 7;
        const tRank = (decoded.target >> 3) & 7;
        for (let df = -1; df <= 1; df++) {
          for (let dr = -1; dr <= 1; dr++) {
            if (df === 0 && dr === 0) continue;
            const nf = tFile + df, nr = tRank + dr;
            if (nf < 0 || nf > 7 || nr < 0 || nr > 7) continue;
            const sq = (nr << 3) | nf;
            const ent = decodeMailbox(preFull[sq]);
            if (!ent.empty) preBodyguard.push({ sq, entry: ent });
          }
        }
      }
      await renderApplied(raw, preFull, preTarget, preBodyguard);
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      aiThinking = false;
      busy = false;
    }
  }

  /** After the attacker has decided (target, approach) — either apply directly
   *  or stage a Bodyguard chooser for the defender's seat. */
  function commitMoveTargetApproach(target: number, approach: number) {
    const perTarget = moveTargets.byTarget.get(target);
    if (!perTarget) return;
    const variants = perTarget.get(approach);
    if (!variants) return;
    if (variants.redirects.length === 0) {
      applyRaw(variants.defenderRaw);
      return;
    }
    // Bodyguard variants exist. Recompute eligible Guard squares from the
    // current position so we know which square each redirect's choice_idx
    // points at. The k-th redirect (choiceIdx = k) maps to the k-th Guard
    // in canonical ascending order — same ordering the engine uses.
    const pos = match.position;
    if (!pos) return;
    const guards = bodyguardGuardsFor(pos, target, approach);
    const redirects: { guardSq: number; raw: number }[] = [];
    for (const r of variants.redirects) {
      const idx = r.choiceIdx - 1;
      if (idx < 0 || idx >= guards.length) continue;
      redirects.push({ guardSq: guards[idx], raw: r.raw });
    }
    if (redirects.length === 0) {
      // No mappable redirects (shouldn't happen): apply defender variant.
      applyRaw(variants.defenderRaw);
      return;
    }
    pendingBodyguard = {
      target,
      defenderRaw: variants.defenderRaw,
      redirects,
    };
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

    // Bodyguard chooser is active: clicks select defender or a Guard.
    if (pendingBodyguard) {
      if (sq === pendingBodyguard.target) {
        const raw = pendingBodyguard.defenderRaw;
        pendingBodyguard = null;
        applyRaw(raw);
        return;
      }
      const hit = pendingBodyguard.redirects.find((r) => r.guardSq === sq);
      if (hit) {
        const raw = hit.raw;
        pendingBodyguard = null;
        applyRaw(raw);
        return;
      }
      // Click anywhere else: ignore. Attacker already committed; the choice
      // is binding. (We could cancel here, but Stack M says "the defender
      // chooses whether to intercept" — there's no opt-out from making a
      // choice, just from intercepting.)
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
          applyRaw(raw);
          return;
        }
      }
      armedSkill = null;
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
      } else {
        armedSkill = { square: src, skillId: slice.skillId };
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
    const ts = skillTargetsFor(match.legal, src, skillId);
    const variants = ts.variantsByTarget.get(target);
    if (!variants || variants.length === 0) return null;
    if (hasFocusModeChoice(match.legal, src, skillId)) {
      const wantEffect = focusModePref === "effect";
      const v = variants.find((x) => x.focusMode === wantEffect);
      if (v) return v.raw;
    }
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
   *  half-armed action doesn't bleed across the mode boundary. */
  function clearAllPickers(): void {
    match.selection = null;
    pendingApproach = null;
    pendingBodyguard = null;
    armedSkill = null;
    pendingDirection = null;
    focusModePref = "activation";
  }

  /** Pull engine state into the reactive `match` store after a snapshot
   *  restore. Mirrors the shape of inspector/+page.svelte:syncEngineToNode. */
  async function syncFromEngine(): Promise<void> {
    if (!eng) return;
    const pv = await eng.positionView();
    const la = await eng.legalActions();
    match.position = pv;
    match.legal = la;
    reconcilePieceIds();
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
    <GraceBanner {eng} />
  {/if}

  {#if bootError}
    <p class="err">boot error: {bootError}</p>
  {:else if !ready}
    <p>{t("app.loading")}</p>
  {:else}
    <section class="board-wrap">
      <div class="board-stack">
        <Board
          position={match.position}
          {pieceIds}
          selection={match.selection}
          moveTargets={armedSkill ? armedSkillTargets : moveTargets.squares}
          {selectable}
          draggable={movable}
          usedSquares={usedThisPhase}
          {shakingSquares}
          effectsActive={effectQueue.length > 0}
          approachChoices={pendingApproach?.approaches ?? []}
          bodyguardChoice={pendingBodyguard ? {
            defender: pendingBodyguard.target,
            guards: pendingBodyguard.redirects.map((r) => r.guardSq),
          } : null}
          lastApplied={lastAppliedPair}
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
        <EffectsLayer viewBox={800} wheelPad={60} bind:queue={effectQueue} />
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
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
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
