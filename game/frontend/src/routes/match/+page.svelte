<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { getEngine, ActionKind, decodeAction } from "$lib/engine";
  import { decodeMailbox } from "$lib/engine/mailbox";
  import { t } from "$lib/state/i18n";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import {
    moveTargetsFor,
    movableSources,
    findActionByKind,
    approachChoicesFor,
  } from "$lib/state/move-targets";
  import { bodyguardGuardsFor } from "$lib/state/bodyguard";
  import { skillTargetsFor, skillIsCastable } from "$lib/state/skill-targets";
  import {
    isSelfCast,
    SKILLS,
    MODIFIER_FOCUS,
    MODIFIER_CHARGE,
  } from "$lib/engine/skills";

  // Focus / Charge are themselves skills (ids 14 / 15). Casting them stages
  // the corresponding modifier in `position.pendingModifiers`; we read the
  // bitfield to drive the wheel's "active" glow.
  const FOCUS_SKILL_ID = 14;
  const CHARGE_SKILL_ID = 15;
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import type { Effect } from "$lib/viz/effects";

  const mode = $derived($page.url.searchParams.get("mode") ?? "hvh");

  let bootError = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  let lastAppliedPair = $state<{ src: number; target: number } | null>(null);

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

  let eng: Awaited<ReturnType<typeof getEngine>> | null = null;

  const moveTargets = $derived(moveTargetsFor(match.legal, match.selection));
  const selectable = $derived(movableSources(match.legal));
  const endPhaseAction = $derived(findActionByKind(match.legal, ActionKind.EndPhase));
  const inMovePhase = $derived(match.position?.currentPhase === 0);
  const interactive = $derived(ready && !busy && match.position?.gameResult === 0);

  // Wheel state. Open whenever a piece is selected (and the player isn't
  // mid-drag — we don't want the wheel popping up just from press-down).
  const wheelOpen = $derived.by(() => {
    if (!interactive) return null;
    if (match.selection === null) return null;
    if (dragSrc !== null) return null;
    if (pendingApproach !== null) return null;
    if (pendingBodyguard !== null) return null;
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

  // Per-slice legality on the wheel. A skill slice is enabled iff at least
  // one Skill action with that (caster, skill_id) is in `legal`. Modifier
  // slices are enabled iff an EndPhase or any Skill action exists (i.e.
  // we're in the Skill Phase with at least one castable skill). End-Phase
  // is enabled iff `endPhaseAction !== null`.
  const wheelLegality = $derived.by(() => {
    if (!wheelOpen) {
      return {
        skill1Legal: false,
        skill2Legal: false,
        focusLegal: false,
        chargeLegal: false,
        endPhaseLegal: false,
      };
    }
    const src = wheelOpen.square;
    const skill1Legal = wheelOpen.skill1 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill1);
    const skill2Legal = wheelOpen.skill2 > 0 && skillIsCastable(match.legal, src, wheelOpen.skill2);
    // Modifiers are always togglable while in Skill Phase with at least one
    // castable skill; toggling itself doesn't consume an action.
    const anyCastable = skill1Legal || skill2Legal;
    return {
      skill1Legal,
      skill2Legal,
      focusLegal: anyCastable,
      chargeLegal: anyCastable,
      endPhaseLegal: endPhaseAction !== null,
    };
  });

  // Target set for the currently-armed skill. Empty when no skill armed.
  const armedSkillTargets = $derived.by(() => {
    if (!armedSkill) return new Set<number>();
    return skillTargetsFor(match.legal, armedSkill.square, armedSkill.skillId).squares;
  });

  onMount(async () => {
    try {
      eng = await getEngine();
      const pending = match.pendingSnapshotJson;
      resetMatchState();
      if (pending) {
        await eng.restoreFromSnapshot(pending);
      } else {
        await eng.createEngine();
      }
      await refresh();
      reconcilePieceIds();
      lastPhaseKey = phaseKey();
      match.mode = mode as typeof match.mode;
      ready = true;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
  });

  function phaseKey(): string {
    return `${match.position?.toMove ?? -1}:${match.position?.currentPhase ?? -1}`;
  }

  async function refresh() {
    if (!eng) return;
    match.position = await eng.positionView();
    match.legal = await eng.legalActions();
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
  }

  async function applyRaw(raw: number) {
    if (!eng || busy) return;
    busy = true;
    try {
      const decoded = decodeAction(raw);
      // Snapshot pre-state on the target so we can compute damage on attacks.
      const preMailbox = match.position?.mailbox;
      const preTarget = preMailbox ? decodeMailbox(preMailbox[decoded.target]) : null;
      const preBodyguard: { sq: number; entry: ReturnType<typeof decodeMailbox> }[] = [];
      // Bodyguard candidates: adjacent friendly Guards of the defender. We
      // capture all friendly-Guard mailbox entries adjacent to the target so
      // we can detect whichever one absorbed the hit afterwards.
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

      await eng.tryApply(raw);

      // Transfer piece ids along the move BEFORE refresh, so the new
      // bitboards from refresh see a piece with a stable identity at the
      // destination square. For plain Move: src → target. For Move-Attack
      // (no kill): src → approach_sq. For Move-Attack with kill: the attacker
      // advances all the way to target, and the defender's id is dropped.
      // We detect kill by comparing pre/post mailbox at target.
      let killed = false;
      if (decoded.kind === ActionKind.Move && decoded.hasAux && preTarget) {
        // Defender died iff their pre-state HP+armor was 1 AND post is empty
        // AND no bodyguard absorbed (we re-check by reading post target).
        // Easier: after refresh below, check if mailbox[target] now holds
        // the attacker. But we want to transfer ids BEFORE refresh, so use
        // the pre-target totals plus the rule: kill happens when defender's
        // hp+armor was 1 and Bodyguard didn't intercept.
        // The clean signal: read mailbox[target] post-apply. Engine state
        // is already updated by tryApply; the cached `match.position` is
        // stale until refresh, but `eng.positionView()` is what we need.
        // We'll defer the kill check to AFTER refresh, then fix up ids.
        killed = false; // placeholder; overwritten after refresh
      }

      if (decoded.kind === ActionKind.Move) {
        const approach = decoded.hasAux ? decoded.auxSq : decoded.target;
        const srcId = pieceIds.get(decoded.src);
        if (srcId !== undefined) {
          pieceIds.delete(decoded.src);
          pieceIds.set(approach, srcId);
        }
      }

      await refresh();

      // Now detect kill by reading the refreshed mailbox at target.
      if (decoded.kind === ActionKind.Move && decoded.hasAux && preTarget && match.position) {
        const postTarget = decodeMailbox(match.position.mailbox[decoded.target]);
        // A kill means the defender vacated AND the tile is now occupied by
        // the attacker (post-hop). If postTarget is empty, no kill-advance
        // happened (defender survived). If postTarget is non-empty and the
        // attacker's approach tile is now empty, the attacker walked the
        // final hop into the defender's square.
        const approach = decoded.auxSq;
        if (!postTarget.empty && approach !== decoded.target) {
          const postApproach = decodeMailbox(match.position.mailbox[approach]);
          if (postApproach.empty) {
            killed = true;
            // Move the attacker's id from approach → target.
            const aid = pieceIds.get(approach);
            if (aid !== undefined) {
              pieceIds.delete(approach);
              pieceIds.set(decoded.target, aid);
            }
          }
        } else if (!postTarget.empty && approach === decoded.target) {
          // Speed-1 attack that killed: attacker is at src→target in one hop.
          // pieceIds was set to approach (=target), nothing to fix.
          killed = true;
        }
      }

      reconcilePieceIds();

      // Effects.
      if (decoded.kind === ActionKind.Move) {
        const path = walkedPath(decoded, killed);
        if (path.length >= 2) {
          effectQueue.push({ kind: "dust", path, startedAt: performance.now() });
        }
        // Move-Attack damage detection.
        if (decoded.hasAux && preTarget && match.position) {
          const postTarget = decodeMailbox(match.position.mailbox[decoded.target]);
          const before = preTarget.hp + preTarget.armor;
          // On kill, the attacker now occupies target — so postTarget reads
          // the attacker's stats, not the defender's. Treat that as "after = 0".
          const after = killed ? 0 : postTarget.hp + postTarget.armor;
          if (after < before) {
            pushDamageEffect(decoded.target, before, after);
          } else {
            // Defender unhurt: a Bodyguard likely ate the hit. Find which
            // adjacent friendly Guard lost HP or armor.
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
        // Mark attacker's final square as used this phase.
        const finalSq = decoded.hasAux
          ? (killed ? decoded.target : decoded.auxSq)
          : decoded.target;
        usedThisPhase = new Set([...usedThisPhase, finalSq]);
      }

      lastAppliedPair =
        decoded.kind === ActionKind.Move || decoded.kind === ActionKind.Skill
          ? { src: decoded.src, target: decoded.target }
          : null;
      match.lastApplied = raw;
      match.selection = null;
      pendingApproach = null;

      // Phase / side flip → clear used-this-phase set.
      const k = phaseKey();
      if (k !== lastPhaseKey) {
        usedThisPhase = new Set();
        lastPhaseKey = k;
      }
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
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
      return;
    }

    if (selectable.has(sq)) {
      match.selection = sq;
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
    const src = wheelOpen.square;

    if (slice.kind === "skill") {
      // Self-cast skills fire immediately.
      if (isSelfCast(slice.skillId)) {
        const raw = rawForSelfCast(src, slice.skillId);
        if (raw !== null) {
          armedSkill = null;
          applyRaw(raw);
        }
        return;
      }
      // Otherwise arm it (or disarm if already armed with the same skill).
      if (armedSkill && armedSkill.skillId === slice.skillId) {
        armedSkill = null;
      } else {
        armedSkill = { square: src, skillId: slice.skillId };
      }
      return;
    }

    if (slice.kind === "modifier") {
      // Focus / Charge are themselves skills (ids 14 / 15). Cast as a
      // self-cast skill action; the engine stages the modifier on the
      // position. The wheel's "active" glow then lights up via the
      // derived `focusActive` / `chargeActive` reading pendingModifiers.
      const skillId = slice.modifier === "focus" ? FOCUS_SKILL_ID : CHARGE_SKILL_ID;
      const raw = rawForSelfCast(src, skillId);
      if (raw !== null) {
        armedSkill = null;
        applyRaw(raw);
      }
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
  // one variant per legal (caster, skill, target); modifier state lives in
  // `position.pendingModifiers`, not on the action. Just return the first
  // matching variant.
  function rawForArmedTarget(src: number, skillId: number, target: number): number | null {
    for (let i = 0; i < match.legal.length; i++) {
      const raw = match.legal[i];
      const a = decodeAction(raw);
      if (a.kind !== ActionKind.Skill) continue;
      if (a.src !== src) continue;
      if (a.skillId !== skillId) continue;
      if (a.target !== target) continue;
      return raw;
    }
    return null;
  }

  function handleKeyDown(ev: KeyboardEvent) {
    if (ev.key === "Escape") {
      if (pendingApproach) {
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
</script>

<svelte:window onkeydown={handleKeyDown} />

<main>
  <header>
    <p class="back"><a href="../">← back</a></p>
    <h1>{t("match.title", { mode })}</h1>
  </header>

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
          usedSquares={usedThisPhase}
          {shakingSquares}
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
        <EffectsLayer viewBox={800} bind:queue={effectQueue} />
      </div>
      {#if pendingApproach}
        <p class="hint">choose the path the attacker takes — click a highlighted square, or press Esc to cancel</p>
      {/if}
      {#if pendingBodyguard}
        <p class="hint">bodyguard: defender may redirect the hit — click the red defender to take the hit, or a blue guard to intercept</p>
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
        <button
          type="button"
          disabled={!interactive || endPhaseAction === null}
          onclick={endPhase}
        >{t("controls.endPhase")}</button>
      </div>
    </aside>

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
  }
  .hint {
    margin: 0.4rem 0 0;
    text-align: center;
    font-size: 0.9rem;
    color: var(--paper-ink-soft, #6a6055);
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
</style>
