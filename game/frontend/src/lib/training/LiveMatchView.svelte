<script lang="ts">
  // Panel 1 - Live Match View.
  //
  // Subscribes to per-ply updates via the `live.sub` sentinel, then polls
  // `read_training_live` at 4 Hz. Every fresh FEN gets fed to
  // `fen_to_position_view` so the existing Board.svelte renderer draws
  // the position unchanged. Three eval bars sit beside the board -
  // challenger NN, defender NN, heuristic - using the centipawn-scale
  // numbers the trainer writes into live.json.
  //
  // Cleanup matters: on unmount, unsubscribe + the polling store stops on
  // last-unsub so the trainer can throttle its per-ply writes back down.

  import { onMount, onDestroy, getContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createPollingStore } from "$lib/training/polling";
  import { normalisePositionView, type PositionViewDto } from "$lib/engine/tauri-client";
  import type { PositionView } from "$lib/engine";
  import type { LivePosition } from "$lib/training/types";
  import Board from "$lib/board/Board.svelte";
  import EvalBar from "./EvalBar.svelte";

  const getRunDir = getContext<() => string>("training:getRunDir");

  let runDir = $state<string>("");
  let live: LivePosition | null = $state(null);
  let livePollErr: string | null = $state(null);
  let positionView: PositionView | null = $state(null);
  let parseErr: string | null = $state(null);

  type LiveStore = ReturnType<typeof createPollingStore<LivePosition>>;
  let store: LiveStore | null = $state(null);

  onMount(async () => {
    runDir = getRunDir();
    if (!runDir) return;
    try {
      await invoke("subscribe_training_live", { runDir });
    } catch (e: unknown) {
      livePollErr = e instanceof Error ? e.message : String(e);
      return;
    }
    store = createPollingStore<LivePosition>({
      invokeCmd: "read_training_live",
      args: { runDir },
      intervalMs: 250,
    });
  });

  onDestroy(() => {
    if (runDir) {
      void invoke("unsubscribe_training_live", { runDir }).catch(() => {});
    }
  });

  $effect(() => {
    if (!store) return;
    return store.subscribe((v) => {
      live = v.data;
      livePollErr = v.error;
    });
  });

  // Dedupe on FEN - ETA-only or eval-only updates would otherwise trigger
  // a needless re-parse.
  let lastFen: string | null = $state(null);
  $effect(() => {
    const l = live;
    if (!l) {
      positionView = null;
      lastFen = null;
      return;
    }
    if (l.fen === lastFen) return;
    const fen = l.fen;
    lastFen = fen;
    void (async () => {
      try {
        const dto = await invoke<PositionViewDto>("fen_to_position_view", { fen });
        positionView = normalisePositionView(dto);
        parseErr = null;
      } catch (e: unknown) {
        parseErr = e instanceof Error ? e.message : String(e);
        positionView = null;
      }
    })();
  });
</script>

<div class="liveMatch">
  {#if livePollErr}
    <p class="error">Live poll error: {livePollErr}</p>
  {/if}
  {#if parseErr}
    <p class="error">FEN parse error: {parseErr}</p>
  {/if}

  {#if !live}
    <div class="empty">
      Waiting for the trainer to publish a live position. Start a run from
      the top bar - the gauntlet phase produces self-play matches that
      stream here.
    </div>
  {:else}
    <header class="meta">
      <div class="raters">
        <span class="rater p1">{live.challenger}</span>
        <span class="vs">vs</span>
        <span class="rater p2">{live.defender}</span>
      </div>
      <div class="stats">
        <span>Game {live.game_index + 1}/{live.games_total}</span>
        <span>Ply {live.ply}</span>
        <span>Last: <code>{live.last_action}</code></span>
      </div>
    </header>

    <div class="boardArea">
      <div class="boardWrap">
        <Board position={positionView} interactive={false} viewBox={640} />
      </div>
      <div class="bars">
        <EvalBar label={live.challenger} value={live.evals.challenger_nn} color="p1" />
        <EvalBar label={live.defender} value={live.evals.defender_nn} color="p2" />
        <EvalBar label="Heuristic" value={live.evals.heuristic} />
      </div>
    </div>
  {/if}
</div>

<style>
  .liveMatch {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .error {
    color: var(--p2, #a13a2a);
    font-size: 0.92em;
    margin: 0;
  }
  .empty {
    color: var(--paper-ink-soft);
    font-style: italic;
    padding: 1.5rem 0;
    text-align: center;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 0.3em;
    border-bottom: 1px solid var(--paper-line);
    padding-bottom: 0.5em;
  }
  .raters {
    display: flex;
    gap: 0.6em;
    align-items: baseline;
    font-weight: 600;
  }
  .rater.p1 { color: var(--p1, #2b4a8a); }
  .rater.p2 { color: var(--p2, #a13a2a); }
  .vs { color: var(--paper-ink-soft); }
  .stats {
    display: flex;
    gap: 1.2em;
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .stats code {
    font-family: inherit;
    background: var(--paper-bg);
    padding: 0.05em 0.35em;
    border-radius: 3px;
  }
  .boardArea {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, auto);
    gap: 0.8rem;
    align-items: start;
  }
  .boardWrap {
    display: flex;
    justify-content: center;
  }
  .bars {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    min-width: 0;
  }
</style>
