<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "$lib/state/i18n";
  import { settings } from "$lib/state/settings.svelte";
  import { sfx } from "$lib/audio/sfx";
  import {
    getEngine,
    decodeAction,
    isBodyguardChoice,
    isDraftTurn,
    formatAction,
    ActionKind,
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateMatchLog,
    type EngineClient,
    type EvalBreakdown,
  } from "$lib/engine";
  import { consumePendingMatchLog } from "$lib/storage/library-handoff";
  import { snapshotJsonFromMatchLog } from "$lib/multiplayer-resume";
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";
  import PlayerPanel from "$lib/match/PlayerPanel.svelte";
  import ProgressionPanel from "$lib/match/ProgressionPanel.svelte";
  import EvalBreakdownPanel from "$lib/eval/EvalBreakdownPanel.svelte";
  import BackButton from "$lib/ui/BackButton.svelte";

  // Empty snapshot used to reset the engine before replaying from ply 0.
  // We keep the original log's start_fen + config and only zero `actions`.
  let baseSnapshotJson: string | null = null;

  let eng: EngineClient | null = null;
  let renderer: PlyRenderer | null = $state(null);
  let pastedRaw = $state("");
  let loadError = $state<string | null>(null);
  let plies = $state<number[]>([]);
  let currentPly = $state(0);
  let playing = $state(false);
  let busy = $state(false);
  let loaded = $state(false);
  let heuristicEvalBreakdown = $state<EvalBreakdown | null>(null);
  // Snapshot of the breakdown at the end of the previous round, so the panel
  // can show round-over-round change per component.
  let prevRoundBreakdown = $state<EvalBreakdown | null>(null);
  let lastRoundSeen = $state<number | null>(null);

  const actionLabel = $derived(
    currentPly > 0 && currentPly <= plies.length
      ? formatAction(plies[currentPly - 1])
      : null,
  );

  // Replay's lastApplied hint for the board overlay. The renderer also
  // tracks this for Move/Skill but we suppress it for BodyguardChoice
  // (defender doesn't move), DraftTurn (no board squares), and EndPhase /
  // EndTurn (both encode src=target=0 → corner; no actual ply happened).
  const lastAppliedDisplay = $derived.by<{ src: number; target: number } | null>(() => {
    if (currentPly === 0) return null;
    const raw = plies[currentPly - 1];
    if (isDraftTurn(raw) || isBodyguardChoice(raw)) return null;
    const d = decodeAction(raw);
    if (d.kind === ActionKind.EndPhase || d.kind === ActionKind.EndTurn) return null;
    return { src: d.src, target: d.target };
  });

  onMount(async () => {
    eng = await getEngine();
    renderer = createPlyRenderer(eng, { sfxEnabled: true });
    const pending = consumePendingMatchLog();
    if (pending !== null) {
      await loadFromJson(pending);
    }
  });

  onDestroy(() => {
    // Cancel timers, empty effectQueue, clear checkpoints. Same shape as
    // inspector/match — replay had none until now, so long-lived replay
    // sessions leaked renderer state on route exit.
    renderer?.dispose();
    renderer = null;
  });

  async function loadFromJson(json: string): Promise<void> {
    if (!eng || !renderer) return;
    busy = true;
    loadError = null;
    playing = false;
    try {
      try {
        validateMatchLog(json, {
          maxActions: SNAPSHOT_BUDGETS.PASTE_MAX_ACTIONS,
          maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
          requireConfig: true,
          source: "library-handoff",
        });
      } catch (e) {
        if (e instanceof SnapshotValidationError) {
          loadError = t("replay.invalid");
          return;
        }
        throw e;
      }
      const log = JSON.parse(json) as { plies?: Array<{ action: { raw: number } }> };
      const rawPlies: number[] = (log.plies ?? []).map((p) => p.action.raw >>> 0);

      const fullSnap = snapshotJsonFromMatchLog(json);
      if (fullSnap === null) {
        loadError = t("replay.invalid");
        return;
      }
      const parsed = JSON.parse(fullSnap);
      parsed.actions = [];
      const startSnap = JSON.stringify(parsed);

      baseSnapshotJson = startSnap;
      plies = rawPlies;
      currentPly = 0;
      await eng.restoreFromSnapshot(startSnap);
      await renderer.resyncFromEngine();
      loaded = true;
    } finally {
      busy = false;
    }
  }

  async function stepForward(): Promise<void> {
    if (!eng || !renderer || busy) return;
    if (currentPly >= plies.length) {
      playing = false;
      return;
    }
    busy = true;
    try {
      const raw = plies[currentPly];
      const base = baseSnapshotJson;
      await renderer.applyAndRender(
        raw,
        async () => {
          await eng!.tryApply(raw);
        },
        base !== null ? { plyHint: currentPly, plyHintBase: base } : undefined,
      );
      currentPly++;
      if (currentPly >= plies.length) playing = false;
    } finally {
      busy = false;
    }
  }

  async function jumpTo(target: number): Promise<void> {
    if (!eng || !renderer || baseSnapshotJson === null || busy) return;
    const clamped = Math.max(0, Math.min(plies.length, target | 0));
    if (clamped === currentPly) return;
    busy = true;
    playing = false;
    try {
      await renderer.fastForwardTo(baseSnapshotJson, plies, clamped);
      currentPly = clamped;
    } finally {
      busy = false;
    }
  }

  // Live-scrub coalescing: while a jumpTo is in flight, the user may drag
  // further. We remember the latest requested ply and re-fire jumpTo after the
  // current one settles. Without this, oninput-driven scrubs get dropped
  // whenever busy=true, and the slider feels unresponsive during drag.
  let pendingScrubTarget: number | null = null;
  async function scrubTo(target: number): Promise<void> {
    pendingScrubTarget = target;
    if (busy) return;
    while (pendingScrubTarget !== null) {
      const next = pendingScrubTarget;
      pendingScrubTarget = null;
      if (next === currentPly) continue;
      await jumpTo(next);
    }
  }

  function togglePlay(): void {
    if (plies.length === 0) return;
    if (currentPly >= plies.length) return;
    sfx.play("click");
    playing = !playing;
  }

  function restart(): void {
    sfx.play("click");
    void jumpTo(0);
  }

  function stepBackward(): void {
    if (busy || currentPly === 0) return;
    sfx.play("click");
    playing = false;
    void jumpTo(currentPly - 1);
  }

  // Poll heuristic eval breakdown on each ply change so the analysis panel
  // stays in sync with the current-position display. Gated on the setting;
  // no work performed when the panel is hidden.
  $effect(() => {
    void currentPly;
    void loaded;
    if (!settings.showEvalPanel || !eng || !loaded) {
      heuristicEvalBreakdown = null;
      prevRoundBreakdown = null;
      lastRoundSeen = null;
      return;
    }
    const e = eng;
    const priorBreakdown = heuristicEvalBreakdown;
    const priorRound = lastRoundSeen;
    void e.heuristicEval().then((v) => {
      const curRound = renderer?.position?.roundNumber ?? null;
      // When the round advances, freeze the last-seen breakdown as the "previous"
      // reference so the panel can display the round-over-round change.
      if (curRound !== null && priorRound !== null && curRound !== priorRound && priorBreakdown !== null) {
        prevRoundBreakdown = priorBreakdown;
      }
      lastRoundSeen = curRound;
      heuristicEvalBreakdown = v;
    }).catch(() => {});
  });

  // Auto-play loop. Each tick schedules a single step; the next iteration
  // is triggered when currentPly changes (which retriggers this $effect).
  //
  // With `respectAnimation` on, the step is gated on MAX(user-delay,
  // animation-done) — cinematic viewers get the full walk + lunge, "fast"
  // users with short delay values keep their pacing. With it off, only the
  // user delay matters and the next ply can interrupt an in-flight animation.
  $effect(() => {
    if (!playing) return;
    if (busy) return;
    if (currentPly >= plies.length) return;
    if (!renderer) return;
    const r = renderer;
    const delay = Math.max(16, settings.replayStepDelayMs);
    let cancelled = false;
    const sleep = new Promise<void>((r) => setTimeout(r, delay));
    const gate = settings.respectAnimation
      ? Promise.all([sleep, r.animationDone()]).then(() => undefined)
      : sleep;
    void gate.then(() => { if (!cancelled) void stepForward(); });
    return () => { cancelled = true; };
  });

  // Loop-on-end: when replay finishes and loopOnEnd is set, restart from ply 0.
  $effect(() => {
    if (!playing && loaded && currentPly >= plies.length && settings.replayLoopOnEnd && plies.length > 0) {
      void jumpTo(0).then(() => { playing = true; });
    }
  });

  function onScrubInput(ev: Event): void {
    const v = Number((ev.currentTarget as HTMLInputElement).value);
    void scrubTo(v);
  }

  function onPasteLoad(): void {
    if (!pastedRaw.trim()) return;
    sfx.play("click");
    void loadFromJson(pastedRaw);
  }

  let fenCopyState = $state<"idle" | "copied" | "failed">("idle");
  let fenCopyTimer: ReturnType<typeof setTimeout> | null = null;
  async function copyFen(): Promise<void> {
    if (!eng) return;
    sfx.play("click");
    try {
      const fen = await eng.positionFen();
      await navigator.clipboard.writeText(fen);
      fenCopyState = "copied";
    } catch {
      fenCopyState = "failed";
    }
    if (fenCopyTimer !== null) clearTimeout(fenCopyTimer);
    fenCopyTimer = setTimeout(() => { fenCopyState = "idle"; }, 1200);
  }
</script>

<main>
  <header>
    <BackButton />
    <h1>{t("replay.title")}</h1>
  </header>

  {#if !loaded}
    <section class="paste">
      <p class="hint">{t("replay.noLog")}</p>
      <textarea
        bind:value={pastedRaw}
        placeholder={t("replay.pastePlaceholder")}
        rows="8"
      ></textarea>
      <div class="paste-actions">
        <button type="button" disabled={busy || !pastedRaw.trim()} onclick={onPasteLoad}>
          {t("replay.load")}
        </button>
      </div>
      {#if loadError}
        <p class="error">{loadError}</p>
      {/if}
    </section>
  {:else if renderer}
    <section class="viewer">
      <div class="game-area">
        <div class="board-column">
          <PlayerPanel
            player="p2"
            position={renderer.position}
            aiThinking={false}
            aiLastDepth={0}
            aiLastScore={0}
            aiMaxDepth={0}
            isAiSeat={false}
          />

          <div class="board-stack">
            <Board
              position={renderer.position}
              pieceIds={renderer.pieceIds}
              shakingSquares={renderer.shakingSquares}
              lungeSquares={renderer.lungeSquares}
              pieceMotion={renderer.pieceMotion}
              lastApplied={lastAppliedDisplay}
              interactive={false}
            />
            <EffectsLayer viewBox={800} wheelPad={60} queue={renderer.effectQueue} />
          </div>

          <PlayerPanel
            player="p1"
            position={renderer.position}
            aiThinking={false}
            aiLastDepth={0}
            aiLastScore={0}
            aiMaxDepth={0}
            isAiSeat={false}
          />
        </div>

        <div class="right-column">
          <aside class="right-panel">
            <div class="status-block">
              <div class="stat-row">
                <span class="stat-label">Round</span>
                <span class="stat-value">{renderer.position?.roundNumber ?? "–"}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">Phase</span>
                <span class="phase-pill" class:move={renderer.position?.currentPhase === 0} class:skill={renderer.position?.currentPhase !== 0}>
                  {renderer.position?.currentPhase === 0 ? "Move" : "Skill"}
                </span>
              </div>
              <div class="stat-row">
                <span class="stat-label">Actions</span>
                <span class="stat-value">{renderer.position?.actionsRemaining ?? "–"}</span>
              </div>
            </div>

            <div class="panel-divider"></div>

            {#if renderer.position}
              <ProgressionPanel roundNumber={renderer.position.roundNumber} />
            {/if}
          </aside>

          {#if settings.showEvalPanel}
            <div class="eval-below">
              <EvalBreakdownPanel breakdown={heuristicEvalBreakdown} prevBreakdown={prevRoundBreakdown} />
            </div>
          {/if}
        </div>
      </div>

      <div class="meta">
        <div class="ply-counter">
          {t("replay.plyCounter", { current: currentPly, total: plies.length })}
        </div>
        <div class="action-label">
          {#if currentPly === 0}
            <em>{t("replay.atStart")}</em>
          {:else if currentPly >= plies.length}
            <em>{t("replay.atEnd")}</em>
            <span class="last-action">— {actionLabel}</span>
          {:else}
            <span class="last-action">{actionLabel}</span>
          {/if}
        </div>
        <button
          type="button"
          class="copy-fen"
          onclick={() => void copyFen()}
          disabled={busy}
          title="Copy FEN of current position"
        >
          {fenCopyState === "copied" ? "✓ Copied" : fenCopyState === "failed" ? "✗ Failed" : "Copy FEN"}
        </button>
      </div>

      <div class="controls">
        <button
          type="button"
          disabled={busy || currentPly === 0}
          onclick={restart}
        >
          {t("replay.restart")}
        </button>
        <button
          type="button"
          disabled={busy || playing || currentPly === 0}
          onclick={stepBackward}
          aria-label={t("replay.stepBack")}
        >
          {t("replay.stepBack")}
        </button>
        <button
          type="button"
          disabled={busy || plies.length === 0 || currentPly >= plies.length}
          onclick={togglePlay}
        >
          {playing ? t("controls.pause") : t("controls.play")}
        </button>
        <button
          type="button"
          disabled={busy || playing || currentPly >= plies.length}
          onclick={() => { sfx.play("click"); void stepForward(); }}
        >
          {t("replay.stepForward")}
        </button>
      </div>

      <div class="scrub">
        <input
          type="range"
          min="0"
          max={plies.length}
          value={currentPly}
          oninput={(ev) => { sfx.play("tick"); onScrubInput(ev); }}
        />
        <label class="jump">
          <span>{t("replay.jumpToPly")}</span>
          <input
            type="number"
            min="0"
            max={plies.length}
            value={currentPly}
            disabled={busy}
            onchange={(ev) => void jumpTo(Number((ev.currentTarget as HTMLInputElement).value))}
          />
        </label>
      </div>
    </section>
  {/if}
</main>

<style>
  main {
    max-width: 960px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
  }
  header h1 {
    margin: 0.2rem 0 0;
    font-size: 1.8rem;
  }
  .paste {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .paste textarea {
    width: 100%;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.85rem;
    padding: 0.5rem;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
    resize: vertical;
  }
  .paste-actions {
    display: flex;
    gap: 0.5rem;
  }
  .paste-actions button {
    padding: 0.4em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 5px;
    cursor: pointer;
  }
  .paste-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .hint {
    color: var(--paper-ink-soft);
  }
  .error {
    color: #a94b3b;
  }
  .viewer {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    align-items: stretch;
  }
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
    /* Replay reserves more vertical space than /match/ because meta, controls,
       and scrubber all sit below the board (match view puts them in a right
       column). ~170px matches match, +170px for the below-board block. */
    width: min(calc(100vw - 240px - 2rem), calc(100dvh - 340px));
    min-width: 280px;
  }
  .board-stack {
    position: relative;
    width: 100%;
  }
  .right-column {
    flex: 0 0 200px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .eval-below {
    display: flex;
    /* Break out of the 200px right-column so component names fit. Anchor to
       the column's right edge and grow leftward into the gap next to the
       board. Capped by viewport so it never eats the board on narrow windows. */
    width: min(360px, calc(100vw - 2rem));
    align-self: flex-end;
    margin-right: 0;
  }
  .eval-below :global(.eval-panel) {
    flex: 1 1 auto;
    width: 100%;
  }
  .right-panel {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.6rem 0.7rem;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
    min-height: 0;
  }
  .panel-divider {
    height: 1px;
    background: var(--paper-line);
    margin: 0.1rem 0;
  }
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
    color: var(--paper-ink-soft);
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
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.15em 0.55em;
    border-radius: 999px;
    border: 1.5px solid currentColor;
  }
  .phase-pill.move {
    color: var(--p1);
  }
  .phase-pill.skill {
    color: var(--p2);
  }
  @media (max-width: 820px) {
    .game-area {
      flex-direction: column;
      align-items: stretch;
    }
    .board-column {
      width: 100%;
    }
    .right-column {
      flex: 1 1 auto;
    }
  }
  .meta {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 0.4rem 0.6rem;
    border: 1.5px solid var(--paper-line);
    border-radius: 6px;
    background: var(--paper-bg);
  }
  .ply-counter {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
  .action-label {
    color: var(--paper-ink-soft);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.9rem;
  }
  .last-action {
    color: var(--paper-ink);
  }
  .copy-fen {
    padding: 0.3em 0.7em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    font-family: inherit;
    color: inherit;
  }
  .copy-fen:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .controls {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
  }
  .controls button {
    padding: 0.4em 1em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 5px;
    cursor: pointer;
    font-size: 0.95rem;
  }
  .controls button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .scrub {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .scrub input[type="range"] {
    flex: 1;
    min-width: 200px;
  }
  .jump {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.9rem;
    color: var(--paper-ink-soft);
  }
  .jump input[type="number"] {
    width: 5em;
    padding: 0.2em 0.4em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    background: var(--paper-bg);
    color: inherit;
  }
</style>
