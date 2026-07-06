<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import BackButton from "$lib/ui/BackButton.svelte";
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";
  import {
    getEngine,
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateMatchLog,
    validateSnapshot,
    decodeAction,
    ActionKind,
    formatAction,
    formatSquare,
    runAiCall,
    AiCallError,
  } from "$lib/engine";
  import { buildEngineConfigJson, applyEvaluatorSettings } from "$lib/state/match-store.svelte";
  import { match } from "$lib/state/match-store.svelte";
  import {
    moveTargetsFor,
    actableSources,
    rawForTargetApproach,
    approachChoicesFor,
    findActionByKind,
    EMPTY_MOVE_TARGETS,
    type MoveTargetSet,
  } from "$lib/state/move-targets";
  import {
    addChild,
    buildSnapshotForNode,
    dfs,
    findChildByEdge,
    inspector,
    initTree,
    loadTree,
    markPoi,
    poiNodes,
    resetInspector,
    selectNode,
    serializeTree,
    unmarkPoi,
    type InspectorNode,
  } from "$lib/state/inspector-store.svelte";
  import MoveListItem from "$lib/inspector/MoveListItem.svelte";
  import AiHintBanner from "$lib/inspector/AiHintBanner.svelte";
  import PoiLabelDialog from "$lib/inspector/PoiLabelDialog.svelte";
  import { consumePendingMatchLog } from "$lib/storage/library-handoff";

  // ---------------------------------------------------------------------------
  // Local UI state.
  // ---------------------------------------------------------------------------

  let bootError = $state<string | null>(null);
  let busy = $state(false);
  let status = $state(""); // status text near the picker
  let pasteMatchLog = $state("");
  let pasteFen = $state("");
  let pasteTree = $state("");

  // AI state.
  let aiBusy = $state(false);
  let aiContinuous = $state(false);
  let aiCancelRequested = false;

  // PlyRenderer drives Board/EffectsLayer for this route. Stage 6a — replaces
  // the prior inline pieceIds bookkeeping + manual restoreFromSnapshot path.
  // `sfxEnabled: false` because inspector is an analysis tool, not a player
  // surface (matches replay's convention).
  let renderer = $state<PlyRenderer | null>(null);

  // Selection + approach chooser state (for move phase).
  let selection = $state<number | null>(null);
  let approachChoices = $state<number[] | null>(null);
  let approachContext = $state<{ target: number } | null>(null);

  // POI label dialog state (replaces window.prompt in handleMarkPoi).
  let poiDialogOpen = $state(false);
  let poiDialogTargetId = $state<string | null>(null);

  const tree = $derived(inspector.tree);
  const currentNode = $derived.by<InspectorNode | null>(() => {
    if (!tree) return null;
    return tree.nodes[tree.currentId] ?? null;
  });

  // ---------------------------------------------------------------------------
  // Engine sync — whenever currentId changes we restore the engine to that node.
  // ---------------------------------------------------------------------------

  let lastSyncedNodeId: string | null = null;

  $effect(() => {
    if (!tree) return;
    const cid = tree.currentId;
    if (cid === lastSyncedNodeId) return;
    void syncEngineToNode(cid);
  });

  async function syncEngineToNode(nodeId: string): Promise<void> {
    if (!tree || !renderer) return;
    const node = tree.nodes[nodeId];
    if (!node) return;
    busy = true;
    selection = null;
    approachChoices = null;
    approachContext = null;
    try {
      const eng = await getEngine();
      // Root snapshot for this tree — actions=[], config from configJson.
      // The renderer's fastForwardTo restores from here and replays
      // node.actions silently (with checkpoint caching from Stage 6c), then
      // runs the full effect pipeline for the landing ply.
      const cfgObj = JSON.parse(tree.configJson);
      const baseSnap = JSON.stringify({
        start_fen: tree.startFen,
        actions: [],
        config: cfgObj,
      });
      await renderer.fastForwardTo(baseSnap, node.actions, node.actions.length);
      const pv = await eng.positionView();
      const la = await eng.legalActions();
      inspector.position = pv;
      inspector.legal = la;
      if (!node.fen) node.fen = await eng.positionFen();
      lastSyncedNodeId = nodeId;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Tree construction from the four entry points.
  // ---------------------------------------------------------------------------

  function defaultConfigJson(): string {
    try {
      return buildEngineConfigJson(match.side);
    } catch {
      return buildEngineConfigJson({ p1: "human", p2: "human" });
    }
  }

  async function entryFromFen(fen: string): Promise<void> {
    busy = true;
    bootError = null;
    try {
      const eng = await getEngine();
      const configJson = defaultConfigJson();
      const cfgObj = JSON.parse(configJson);
      const snap = JSON.stringify({ start_fen: fen, actions: [], config: cfgObj });
      await eng.restoreFromSnapshot(snap);
      await applyEvaluatorSettings(eng);
      const rootFen = await eng.positionFen();
      const t = initTree({ startFen: fen, configJson, rootFen });
      inspector.tree = t;
      lastSyncedNodeId = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  async function entryFromMatchLog(json: string): Promise<void> {
    busy = true;
    bootError = null;
    try {
      // Bundle envelopes are not engine MatchLogs — unwrap first if present,
      // then validate the inner log via the shared trust gate. The bundle
      // shape itself is structurally trivial (schema string + logs array);
      // the size cap on the outer JSON is enforced by the inner validator
      // after we've taken the substring back through JSON.stringify.
      if (json.length > SNAPSHOT_BUDGETS.MAX_JSON_BYTES) {
        throw new Error(`input too large: ${json.length} bytes`);
      }
      let parsed: unknown;
      try {
        parsed = JSON.parse(json);
      } catch (e) {
        throw new Error(`malformed JSON: ${(e as Error)?.message ?? String(e)}`);
      }
      let logJson = json;
      let log: any = parsed;
      if (
        log && typeof log === "object" &&
        typeof log.schema === "string" && log.schema.startsWith("boardgame-bundle") &&
        Array.isArray(log.logs)
      ) {
        if (log.logs.length === 0) {
          throw new Error("bundle contained no matches");
        }
        if (log.logs.length > 1) {
          bootError = `bundle contains ${log.logs.length} matches — loading the first; export individually to inspect others.`;
        }
        log = log.logs[0];
        logJson = JSON.stringify(log);
      }
      try {
        validateMatchLog(logJson, {
          maxActions: SNAPSHOT_BUDGETS.PASTE_MAX_ACTIONS,
          maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
          // Inspector paste tolerates missing config (falls back to defaults).
          requireConfig: false,
          source: "joiner-paste",
        });
      } catch (e) {
        if (e instanceof SnapshotValidationError) {
          throw new Error(`invalid MatchLog (${e.reason})`);
        }
        throw e;
      }
      if (typeof log.start_fen !== "string") {
        throw new Error("not a MatchLog — expected start_fen at root (or a bundle envelope with logs[].start_fen)");
      }
      const startFen: string = log.start_fen;
      let configObj = log.config;
      if (!configObj || typeof configObj !== "object") {
        configObj = JSON.parse(defaultConfigJson());
      }
      const configJson = JSON.stringify(configObj);
      const plies: Array<{ action: { raw: number }; notes?: string | null }> = log.plies ?? [];

      const eng = await getEngine();
      await eng.restoreFromSnapshot(
        JSON.stringify({ start_fen: startFen, actions: [], config: configObj }),
      );
      await applyEvaluatorSettings(eng);
      const rootFen = await eng.positionFen();
      const t = initTree({ startFen, configJson, rootFen });

      let curId = t.rootId;
      for (const ply of plies) {
        const raw = (ply.action?.raw ?? 0) >>> 0;
        try {
          const r = await eng.tryApply(raw);
          if (r.appliedAction === 0) break;
          const fen = await eng.positionFen();
          const childId = addChild(t, curId, raw, fen);
          if (ply.notes) t.nodes[childId].label = String(ply.notes).slice(0, 80);
          curId = childId;
        } catch {
          break;
        }
      }
      t.currentId = curId;
      inspector.tree = t;
      lastSyncedNodeId = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  async function entryFromSnapshotJson(snapshotJson: string): Promise<void> {
    busy = true;
    bootError = null;
    try {
      try {
        validateSnapshot(snapshotJson, {
          maxActions: SNAPSHOT_BUDGETS.PASTE_MAX_ACTIONS,
          maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
          requireConfig: false,
          source: "library-handoff",
        });
      } catch (e) {
        if (e instanceof SnapshotValidationError) {
          throw new Error(`invalid snapshot (${e.reason})`);
        }
        throw e;
      }
      const snap = JSON.parse(snapshotJson);
      const startFen: string = snap.start_fen;
      let configObj = snap.config;
      if (!configObj || typeof configObj !== "object") {
        configObj = JSON.parse(defaultConfigJson());
      }
      const configJson = JSON.stringify(configObj);
      const actions: number[] = (snap.actions ?? []).map((a: number) => a >>> 0);

      const eng = await getEngine();
      await eng.restoreFromSnapshot(
        JSON.stringify({ start_fen: startFen, actions: [], config: configObj }),
      );
      await applyEvaluatorSettings(eng);
      const rootFen = await eng.positionFen();
      const t = initTree({ startFen, configJson, rootFen });

      let curId = t.rootId;
      for (const raw of actions) {
        const r = await eng.tryApply(raw);
        if (r.appliedAction === 0) break;
        const fen = await eng.positionFen();
        curId = addChild(t, curId, raw, fen);
      }
      t.currentId = curId;
      inspector.tree = t;
      lastSyncedNodeId = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  async function entryFromTreeJson(json: string): Promise<void> {
    busy = true;
    bootError = null;
    try {
      const t = loadTree(json);
      if (typeof t.configJson !== "string") {
        throw new Error("tree JSON missing configJson");
      }
      inspector.tree = t;
      lastSyncedNodeId = null;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Mount: consume any handoff snapshot.
  // ---------------------------------------------------------------------------

  onMount(async () => {
    resetInspector();
    const eng = await getEngine();
    renderer = createPlyRenderer(eng, { sfxEnabled: false });
    if (match.pendingSnapshotJson) {
      const snap = match.pendingSnapshotJson;
      match.pendingSnapshotJson = null;
      await entryFromSnapshotJson(snap);
      return;
    }
    const pendingLog = consumePendingMatchLog();
    if (pendingLog) {
      await entryFromMatchLog(pendingLog);
    }
  });

  onDestroy(() => {
    aiCancelRequested = true;
    renderer?.dispose();
    renderer = null;
  });

  // ---------------------------------------------------------------------------
  // Action application — branches if the chosen action differs from the
  // existing child; otherwise just selects the existing child.
  // ---------------------------------------------------------------------------

  async function applyActionToCurrent(raw: number): Promise<void> {
    if (!tree || !currentNode || !renderer) return;
    const existing = findChildByEdge(tree, currentNode.id, raw);
    if (existing !== null) {
      selectNode(tree, existing);
      return;
    }
    busy = true;
    selection = null;
    approachChoices = null;
    approachContext = null;
    try {
      const eng = await getEngine();
      let rejected = false;
      await renderer.applyAndRender(raw, async () => {
        const r = await eng.tryApply(raw);
        if (r.appliedAction === 0) rejected = true;
      });
      if (rejected) {
        status = "engine rejected action";
        return;
      }
      const fen = await eng.positionFen();
      const newId = addChild(tree, currentNode.id, raw, fen);
      selectNode(tree, newId);
      const pv = await eng.positionView();
      const la = await eng.legalActions();
      inspector.position = pv;
      inspector.legal = la;
      lastSyncedNodeId = newId;
      status = "";
    } catch (e) {
      status = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Move-phase board interaction (click-to-move).
  // ---------------------------------------------------------------------------

  const inMovePhase = $derived(inspector.position?.currentPhase === 0);
  const selectable = $derived(actableSources(inspector.legal));
  const moveTargets = $derived<MoveTargetSet>(
    selection !== null && inMovePhase
      ? moveTargetsFor(inspector.legal, selection)
      : EMPTY_MOVE_TARGETS,
  );

  function handleSquareClick(sq: number): void {
    if (busy) return;
    // Approach chooser is open.
    if (approachChoices && approachContext) {
      if (approachChoices.includes(sq)) {
        const target = approachContext.target;
        const raw = rawForTargetApproach(moveTargets, target, sq);
        approachChoices = null;
        approachContext = null;
        if (raw !== null) void applyActionToCurrent(raw);
        return;
      }
      // Clicking off cancels the chooser.
      approachChoices = null;
      approachContext = null;
      return;
    }
    if (selection !== null && inMovePhase && moveTargets.squares.has(sq)) {
      // Picking a move target.
      const approaches = approachChoicesFor(moveTargets, sq);
      if (approaches.length > 1) {
        approachChoices = approaches;
        approachContext = { target: sq };
        return;
      }
      const approach = approaches[0] ?? sq;
      const raw = rawForTargetApproach(moveTargets, sq, approach);
      if (raw !== null) {
        void applyActionToCurrent(raw);
      }
      selection = null;
      return;
    }
    // Click to select / re-select.
    if (selectable.has(sq)) {
      selection = sq;
    } else {
      selection = null;
    }
  }

  function handleApproachChoice(approach: number): void {
    if (!approachContext) return;
    const target = approachContext.target;
    const raw = rawForTargetApproach(moveTargets, target, approach);
    approachChoices = null;
    approachContext = null;
    if (raw !== null) void applyActionToCurrent(raw);
  }

  const endPhaseAction = $derived(findActionByKind(inspector.legal, ActionKind.EndPhase));
  function endPhase(): void {
    if (endPhaseAction === null) return;
    void applyActionToCurrent(endPhaseAction);
  }

  // ---------------------------------------------------------------------------
  // Ask AI — single-shot, or continuous loop until cancelled.
  // ---------------------------------------------------------------------------

  async function askAiOnce(): Promise<void> {
    if (!currentNode) return;
    aiBusy = true;
    inspector.lastAiHint = null;
    try {
      const eng = await getEngine();
      const r = await eng.requestAiMoveForced();
      if (!currentNode) return;
      inspector.lastAiHint = {
        best: r.appliedAction >>> 0,
        score: r.score,
        depth: r.depth,
        forNodeId: currentNode.id,
      };
    } catch (e) {
      status = `AI: ${(e as Error)?.message ?? String(e)}`;
    } finally {
      aiBusy = false;
    }
  }

  async function askAiContinuous(): Promise<void> {
    if (!currentNode) return;
    aiContinuous = true;
    aiCancelRequested = false;
    aiBusy = true;
    const nodeId = currentNode.id;
    try {
      const eng = await getEngine();
      // Iterative deepening: each call extends the depth by 1. The engine's
      // shared TT means earlier plies are mostly TT hits on the next pass.
      let depth = 1;
      while (!aiCancelRequested && depth <= 64) {
        const r = await runAiCall(
          () => eng.requestAiMoveAtDepth(depth),
          { cancelled: () => aiCancelRequested },
        );
        if (!tree || tree.currentId !== nodeId) break;
        inspector.lastAiHint = {
          best: r.appliedAction >>> 0,
          score: r.score,
          depth: r.depth,
          forNodeId: nodeId,
        };
        // If we hit a forced mate at this depth, deeper search won't help.
        if (Math.abs(r.score) > 29000) break;
        depth += 1;
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    } catch (e) {
      // Cancellation is user-driven (stopAiSearch); don't surface it as an error.
      if (!(e instanceof AiCallError && e.reason === "cancelled")) {
        status = `AI: ${(e as Error)?.message ?? String(e)}`;
      }
    } finally {
      aiBusy = false;
      aiContinuous = false;
    }
  }

  function stopAiSearch(): void {
    aiCancelRequested = true;
  }

  async function applyAiHint(): Promise<void> {
    const h = inspector.lastAiHint;
    if (!h || h.best === 0) return;
    inspector.lastAiHint = null;
    await applyActionToCurrent(h.best);
  }

  function dismissHint(): void {
    inspector.lastAiHint = null;
  }

  // ---------------------------------------------------------------------------
  // POI handlers.
  // ---------------------------------------------------------------------------

  function handleMarkPoi(id: string): void {
    if (!tree) return;
    poiDialogTargetId = id;
    poiDialogOpen = true;
  }
  function handleUnmarkPoi(id: string): void {
    if (!tree) return;
    unmarkPoi(tree, id);
  }
  function handlePoiSave(label: string): void {
    if (!tree || poiDialogTargetId === null) {
      poiDialogOpen = false;
      poiDialogTargetId = null;
      return;
    }
    markPoi(tree, poiDialogTargetId, label);
    poiDialogOpen = false;
    poiDialogTargetId = null;
  }
  function handlePoiCancel(): void {
    poiDialogOpen = false;
    poiDialogTargetId = null;
  }

  // ---------------------------------------------------------------------------
  // Tree export / play handoff.
  // ---------------------------------------------------------------------------

  async function copyTreeJson(): Promise<void> {
    if (!tree) return;
    const s = serializeTree(tree);
    try {
      await navigator.clipboard.writeText(s);
      status = "tree JSON copied";
    } catch {
      pasteTree = s;
      status = "clipboard blocked — copy from textarea below";
    }
  }

  async function playThisPosition(): Promise<void> {
    if (!tree || !currentNode) return;
    const snap = buildSnapshotForNode(tree, currentNode, defaultConfigJson());
    match.pendingSnapshotJson = snap;
    await goto("../setup/");
  }

  function backToEntry(): void {
    aiCancelRequested = true;
    resetInspector();
  }

  // ---------------------------------------------------------------------------
  // Skill-phase action picker (fallback for non-move actions while the full
  // wheel UI hasn't been extracted from /match/+page.svelte).
  // ---------------------------------------------------------------------------

  interface ActionRow {
    raw: number;
    label: string;
  }

  const groupedActions = $derived.by(() => {
    const out = new Map<string, ActionRow[]>();
    const legal = inspector.legal;
    for (let i = 0; i < legal.length; i++) {
      const raw = legal[i] >>> 0;
      const d = decodeAction(raw);
      let key: string;
      if (d.kind === ActionKind.EndPhase || d.kind === ActionKind.EndTurn) {
        key = "—";
      } else {
        key = formatSquare(d.src);
      }
      const row: ActionRow = { raw, label: formatAction(raw) };
      if (!out.has(key)) out.set(key, []);
      out.get(key)!.push(row);
    }
    // Sort keys: "—" (end actions) first, then by square.
    return new Map(
      [...out.entries()].sort(([a], [b]) => {
        if (a === b) return 0;
        if (a === "—") return -1;
        if (b === "—") return 1;
        return a.localeCompare(b);
      }),
    );
  });

  const phaseLabel = $derived.by(() => {
    const p = inspector.position;
    if (!p) return "";
    const phase = p.currentPhase === 0 ? "Move" : "Skill";
    const who = p.toMove === 0 ? "P1" : "P2";
    return `${who} · ${phase} · ${p.actionsRemaining} actions left · round ${p.roundNumber}`;
  });

  const lastAppliedPair = $derived.by(() => {
    if (!currentNode || currentNode.edgeAction === null) return null;
    const d = decodeAction(currentNode.edgeAction);
    return { src: d.src, target: d.target };
  });

  const aiHintHighlight = $derived.by(() => {
    const h = inspector.lastAiHint;
    if (!h || h.best === 0 || !currentNode || h.forNodeId !== currentNode.id) return null;
    const d = decodeAction(h.best);
    return { src: d.src, target: d.target };
  });

  // Show the AI hint pair on the board (preferred), else last applied.
  const boardLastApplied = $derived(aiHintHighlight ?? lastAppliedPair);
</script>

<main>
  <header>
    <BackButton />
    <h1>Inspector</h1>
    {#if tree}
      <button class="ghost" type="button" onclick={backToEntry}>Discard tree</button>
    {/if}
  </header>

  {#if bootError}
    <div class="err">
      <strong>error:</strong> {bootError}
      <button type="button" onclick={() => (bootError = null)}>dismiss</button>
    </div>
  {/if}

  {#if !tree}
    <section class="entry">
      <p class="lede">
        Open the inspector with a saved game, a position, or a previously-exported tree.
        You can also fork a position out of a live match (use the "Open in inspector" button in the match HUD).
      </p>

      <div class="entry-grid">
        <div class="card">
          <h3>Paste a match log</h3>
          <p>JSON exported by an AIvAI run or finished match.</p>
          <textarea bind:value={pasteMatchLog} rows="4" placeholder="MatchLog JSON…"></textarea>
          <button class="primary" type="button" disabled={busy || pasteMatchLog.trim() === ""} onclick={() => entryFromMatchLog(pasteMatchLog)}>Open log</button>
        </div>

        <div class="card">
          <h3>Paste a FEN</h3>
          <p>Start with a custom position, then sandbox from there.</p>
          <textarea bind:value={pasteFen} rows="2" placeholder="…"></textarea>
          <button class="primary" type="button" disabled={busy || pasteFen.trim() === ""} onclick={() => entryFromFen(pasteFen.trim())}>Open FEN</button>
        </div>

        <div class="card">
          <h3>Restore an inspector tree</h3>
          <p>JSON produced by "Copy tree JSON" in a previous session.</p>
          <textarea bind:value={pasteTree} rows="4" placeholder="Inspector tree JSON…"></textarea>
          <button class="primary" type="button" disabled={busy || pasteTree.trim() === ""} onclick={() => entryFromTreeJson(pasteTree)}>Load tree</button>
        </div>

        <div class="card">
          <h3>Fresh draft</h3>
          <p>Pick seats &amp; loadouts as if starting a new game — then inspect.</p>
          <button class="primary" type="button" onclick={() => goto("../setup/")}>Open setup →</button>
        </div>
      </div>
    </section>
  {:else}
    <section class="view">
      <aside class="tree-panel">
        <h3>Tree</h3>
        {#if poiNodes(tree).length > 0}
          <div class="pois">
            <h4>Points of interest</h4>
            <ul>
              {#each poiNodes(tree) as n (n.id)}
                <li>
                  <button type="button" class:selected={tree.currentId === n.id} onclick={() => selectNode(tree, n.id)}>
                    ★ {n.label} <small>· ply {n.ply}</small>
                  </button>
                </li>
              {/each}
            </ul>
          </div>
        {/if}
        <div class="rows">
          {#each dfs(tree) as n (n.id)}
            <MoveListItem
              node={n}
              selected={tree.currentId === n.id}
              depth={n.ply}
              onSelect={(id) => selectNode(tree, id)}
              onMarkPoi={handleMarkPoi}
              onUnmarkPoi={handleUnmarkPoi}
            />
          {/each}
        </div>
        <div class="tree-actions">
          <button type="button" onclick={copyTreeJson}>Copy tree JSON</button>
          <button class="primary" type="button" onclick={playThisPosition}>Play this position →</button>
        </div>
      </aside>

      <div class="board-pane">
        <div class="hud">
          <span class="phase">{phaseLabel}</span>
          {#if !aiContinuous}
            <button type="button" disabled={aiBusy || busy} onclick={askAiOnce}>{aiBusy ? "Thinking…" : "Ask AI (once)"}</button>
            <button type="button" disabled={aiBusy || busy} onclick={askAiContinuous}>Search continuously</button>
          {:else}
            <button class="primary" type="button" onclick={stopAiSearch}>Stop search</button>
          {/if}
          {#if inMovePhase}
            <button type="button" disabled={endPhaseAction === null || busy} onclick={endPhase}>End phase</button>
          {/if}
        </div>

        {#if inspector.lastAiHint && currentNode && inspector.lastAiHint.forNodeId === currentNode.id}
          <AiHintBanner hint={inspector.lastAiHint} onApply={applyAiHint} onDismiss={dismissHint} />
        {/if}

        <div class="board-wrap">
          <Board
            position={inspector.position}
            pieceIds={renderer?.pieceIds ?? new Map()}
            shakingSquares={renderer?.shakingSquares ?? new Set()}
            interactive={!busy && !aiContinuous}
            {selection}
            moveTargets={moveTargets.squares}
            {selectable}
            approachChoices={approachChoices ?? undefined}
            lastApplied={boardLastApplied}
            onSquareClick={(sq) => handleSquareClick(sq)}
            onApproachChoice={handleApproachChoice}
          />
          {#if renderer}
            <EffectsLayer viewBox={800} wheelPad={60} queue={renderer.effectQueue} />
          {/if}
        </div>

        {#if !inMovePhase}
          <section class="picker">
            <h3>Legal actions</h3>
            <p class="muted">All legal actions at this node — click any to apply it.</p>
            {#if groupedActions.size === 0}
              <p class="muted">No legal actions. Game may be over.</p>
            {:else}
              <div class="picker-groups">
                {#each [...groupedActions.entries()] as [src, rows] (src)}
                  <div class="group">
                    <header>
                      <strong>{src}</strong>
                      <small>{rows.length}</small>
                    </header>
                    <div class="acts">
                      {#each rows as r (r.raw)}
                        <button
                          type="button"
                          disabled={busy}
                          class:ai-suggested={inspector.lastAiHint && inspector.lastAiHint.best === r.raw && currentNode && inspector.lastAiHint.forNodeId === currentNode.id}
                          onclick={() => applyActionToCurrent(r.raw)}
                        >{r.label}</button>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {:else}
          <section class="picker">
            <h3>Legal actions <small class="muted">— or click the board</small></h3>
            {#if groupedActions.size === 0}
              <p class="muted">No legal actions.</p>
            {:else}
              <div class="picker-groups">
                {#each [...groupedActions.entries()] as [src, rows] (src)}
                  <div class="group">
                    <header>
                      <strong>{src}</strong>
                      <small>{rows.length}</small>
                    </header>
                    <div class="acts">
                      {#each rows as r (r.raw)}
                        <button
                          type="button"
                          disabled={busy}
                          class:ai-suggested={inspector.lastAiHint && inspector.lastAiHint.best === r.raw && currentNode && inspector.lastAiHint.forNodeId === currentNode.id}
                          onclick={() => applyActionToCurrent(r.raw)}
                        >{r.label}</button>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {/if}

        {#if status}<p class="muted status">{status}</p>{/if}
      </div>
    </section>
  {/if}
</main>

<PoiLabelDialog
  open={poiDialogOpen}
  initial=""
  onSave={handlePoiSave}
  onCancel={handlePoiCancel}
/>

<style>
  main { max-width: 1400px; margin: 0 auto; padding: 0.6rem 1rem 2rem; }
  header {
    display: flex; align-items: baseline; gap: 1rem;
    margin-bottom: 0.6rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.4rem;
  }
  header h1 { margin: 0; font-size: 1.5rem; flex: 1; }
  .ghost { background: transparent; }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
    margin-bottom: 0.6rem;
    display: flex; align-items: center; gap: 0.6rem;
  }
  .err button { margin-left: auto; font-size: 0.85rem; }

  .lede { color: var(--paper-ink-soft); border-left: 3px solid var(--paper-line-strong); padding-left: 0.7rem; }
  .entry-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
  @media (max-width: 760px) { .entry-grid { grid-template-columns: 1fr; } }
  .card { border: 1.5px solid var(--paper-line-strong); border-radius: 8px; padding: 0.8rem 1rem; background: var(--paper-bg); }
  .card h3 { margin: 0 0 0.4rem; font-size: 1.05rem; }
  .card p { color: var(--paper-ink-soft); margin: 0 0 0.5rem; font-size: 0.9rem; }
  .card textarea { width: 100%; box-sizing: border-box; font: inherit; font-family: ui-monospace, monospace; font-size: 0.78rem; margin-bottom: 0.5rem; }
  .primary { background: var(--accent, #5a7cd6); color: #fff; border-color: var(--accent, #5a7cd6); font-weight: 600; }
  .primary:disabled { opacity: 0.45; cursor: not-allowed; }

  .view {
    display: grid;
    grid-template-columns: 320px 1fr;
    gap: 1rem;
    align-items: start;
  }
  @media (max-width: 900px) { .view { grid-template-columns: 1fr; } }
  .tree-panel {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.6rem 0.5rem 0.6rem 0.7rem;
    background: var(--paper-bg);
    max-height: 80vh;
    overflow: auto;
    display: flex; flex-direction: column; gap: 0.5rem;
  }
  .tree-panel h3 { margin: 0 0 0.3rem; font-size: 1rem; }
  .pois { border-bottom: 1px dashed var(--paper-line); padding-bottom: 0.4rem; }
  .pois h4 { margin: 0 0 0.25rem; font-size: 0.85rem; color: var(--paper-ink-soft); }
  .pois ul { list-style: none; padding: 0; margin: 0; }
  .pois li button {
    width: 100%; text-align: left; background: transparent;
    border: 1px solid transparent; padding: 0.2rem 0.4rem; font: inherit; cursor: pointer; border-radius: 4px;
  }
  .pois li button:hover { background: rgba(0,0,0,0.04); }
  .pois li button.selected { background: var(--accent, #5a7cd6); color: #fff; }
  .pois small { color: var(--paper-ink-soft); }
  .pois li button.selected small { color: rgba(255,255,255,0.85); }
  .rows { display: flex; flex-direction: column; gap: 0; }
  .tree-actions { display: flex; gap: 0.4rem; flex-wrap: wrap; }

  .board-pane { display: flex; flex-direction: column; gap: 0.6rem; }
  .hud { display: flex; align-items: center; gap: 0.6rem; flex-wrap: wrap; }
  .phase { flex: 1; color: var(--paper-ink-soft); }
  .board-wrap {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    overflow: hidden;
    background: var(--paper-bg);
    position: relative;
  }
  .picker {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.6rem 0.8rem;
    background: var(--paper-bg);
  }
  .picker h3 { margin: 0 0 0.4rem; font-size: 1rem; }
  .picker-groups { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.4rem; }
  .group { border: 1px dashed var(--paper-line); border-radius: 6px; padding: 0.3rem 0.4rem; }
  .group header { display: flex; justify-content: space-between; margin-bottom: 0.2rem; padding: 0; border: 0; }
  .group header small { color: var(--paper-ink-soft); }
  .acts { display: flex; flex-direction: column; gap: 0.2rem; }
  .acts button {
    width: 100%; text-align: left; font: inherit; font-size: 0.85rem;
    padding: 0.2rem 0.5rem; border-radius: 4px;
    background: transparent; border: 1px solid var(--paper-line);
    cursor: pointer;
  }
  .acts button:hover:not(:disabled) { background: rgba(0,0,0,0.04); }
  .acts button:disabled { opacity: 0.5; cursor: not-allowed; }
  .acts button.ai-suggested {
    background: rgba(90, 124, 214, 0.15);
    border-color: var(--accent, #5a7cd6);
    font-weight: 600;
  }
  .acts button.ai-suggested::before { content: "★ "; color: var(--accent, #5a7cd6); }
  .muted { color: var(--paper-ink-soft); font-size: 0.9rem; }
  .status { margin: 0; }
</style>
