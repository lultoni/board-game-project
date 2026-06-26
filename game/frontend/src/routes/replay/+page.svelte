<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/state/i18n";
  import { settings } from "$lib/state/settings.svelte";
  import {
    getEngine,
    decodeAction,
    isBodyguardChoice,
    isDraftTurn,
    formatAction,
    SNAPSHOT_BUDGETS,
    SnapshotValidationError,
    validateMatchLog,
    type EngineClient,
  } from "$lib/engine";
  import { consumePendingMatchLog } from "$lib/storage/library-handoff";
  import { snapshotJsonFromMatchLog } from "$lib/multiplayer-resume";
  import Board from "$lib/board/Board.svelte";
  import EffectsLayer from "$lib/board/EffectsLayer.svelte";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";

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

  const actionLabel = $derived(
    currentPly > 0 && currentPly <= plies.length
      ? formatAction(plies[currentPly - 1])
      : null,
  );

  // Replay's lastApplied hint for the board overlay. The renderer also
  // tracks this for Move/Skill but we suppress it for BodyguardChoice
  // (defender doesn't move) and DraftTurn (no board squares).
  const lastAppliedDisplay = $derived.by<{ src: number; target: number } | null>(() => {
    if (currentPly === 0) return null;
    const raw = plies[currentPly - 1];
    if (isDraftTurn(raw) || isBodyguardChoice(raw)) return null;
    const d = decodeAction(raw);
    return { src: d.src, target: d.target };
  });

  onMount(async () => {
    eng = await getEngine();
    renderer = createPlyRenderer(eng, { sfxEnabled: false });
    const pending = consumePendingMatchLog();
    if (pending !== null) {
      await loadFromJson(pending);
    }
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
      await renderer.applyAndRender(raw, async () => {
        await eng!.tryApply(raw);
      });
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

  function togglePlay(): void {
    if (plies.length === 0) return;
    if (currentPly >= plies.length) return;
    playing = !playing;
  }

  function restart(): void {
    void jumpTo(0);
  }

  // Auto-play loop. Each tick schedules a single step; the next iteration
  // is triggered when currentPly changes (which retriggers this $effect).
  $effect(() => {
    if (!playing) return;
    if (busy) return;
    if (currentPly >= plies.length) return;
    const delay = Math.max(16, settings.aivaiStepDelayMs);
    const id = setTimeout(() => void stepForward(), delay);
    return () => clearTimeout(id);
  });

  function onScrubChange(ev: Event): void {
    const v = Number((ev.currentTarget as HTMLInputElement).value);
    void jumpTo(v);
  }

  function onPasteLoad(): void {
    if (!pastedRaw.trim()) return;
    void loadFromJson(pastedRaw);
  }
</script>

<main>
  <header>
    <p><a href="../">{t("replay.back")}</a></p>
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
      <div class="board-wrap">
        <Board
          position={renderer.position}
          pieceIds={renderer.pieceIds}
          shakingSquares={renderer.shakingSquares}
          lastApplied={lastAppliedDisplay}
          interactive={false}
        />
        <EffectsLayer viewBox={800} wheelPad={60} bind:queue={renderer.effectQueue} />
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
      </div>

      <div class="controls">
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
          onclick={() => void stepForward()}
        >
          {t("controls.step")}
        </button>
        <button
          type="button"
          disabled={busy || currentPly === 0}
          onclick={restart}
        >
          {t("replay.restart")}
        </button>
      </div>

      <div class="scrub">
        <input
          type="range"
          min="0"
          max={plies.length}
          value={currentPly}
          disabled={busy}
          onchange={onScrubChange}
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
  .board-wrap {
    max-width: 640px;
    align-self: center;
    width: 100%;
    position: relative;
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
