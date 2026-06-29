<script lang="ts">
  // Panel 4 — Network Inspector (pinned right pane).
  //
  // Picks up two inputs from the route shell:
  //   - selectedRaterId — set by clicking a node in the Lineage panel
  //   - latest FEN — sourced from the live status polling stream, but we
  //     re-poll `read_training_live` directly here at 0.5 Hz so the
  //     Inspector stays responsive even when the Live panel isn't mounted.
  //
  // On every (rater_id, fen) pair we invoke `inspect_rater` and show the
  // forward output + per-layer weight stats. Layer-level introspection is
  // server-side only — the panel never sees raw weights.

  import { onMount, getContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createPollingStore } from "$lib/training/polling";
  import type { RaterInspection, LivePosition } from "$lib/training/types";

  const getRunDir = getContext<() => string>("training:getRunDir");
  const getSelected = getContext<() => string | null>("training:getSelectedRaterId");

  let runDir = $state<string>("");
  let live: LivePosition | null = $state(null);
  let inspection: RaterInspection | null = $state(null);
  let inspectErr: string | null = $state(null);
  let loading = $state(false);

  type LiveStore = ReturnType<typeof createPollingStore<LivePosition>>;
  let liveStore: LiveStore | null = $state(null);

  onMount(() => {
    runDir = getRunDir();
    if (!runDir) return;
    liveStore = createPollingStore<LivePosition>({
      invokeCmd: "read_training_live",
      args: { runDir },
      intervalMs: 2000,
    });
  });

  $effect(() => {
    if (!liveStore) return;
    return liveStore.subscribe((v) => (live = v.data));
  });

  // Re-inspect whenever the (rater, fen) pair changes.
  let lastKey = $state<string>("");
  $effect(() => {
    const id = getSelected();
    const fen = live?.fen ?? "";
    if (!id || !fen || !runDir) {
      inspection = null;
      return;
    }
    const key = `${id}::${fen}`;
    if (key === lastKey) return;
    lastKey = key;
    loading = true;
    void (async () => {
      try {
        const r = await invoke<RaterInspection>("inspect_rater", {
          runDir,
          raterId: id,
          fen,
        });
        inspection = r;
        inspectErr = null;
      } catch (e: unknown) {
        inspectErr = e instanceof Error ? e.message : String(e);
        inspection = null;
      } finally {
        loading = false;
      }
    })();
  });

  const selected = $derived(getSelected());
</script>

<div class="inspector">
  {#if !selected}
    <p class="hint">No rater selected. Switch to the <strong>Lineage</strong> tab and click a node to inspect it here.</p>
  {:else if !live}
    <p class="hint">
      Rater <code>{selected}</code> selected — waiting for a live match position to evaluate against.
    </p>
  {:else}
    <header>
      <span class="lbl">Rater</span>
      <code class="raterId">{selected}</code>
    </header>

    {#if loading}
      <p class="hint">Loading…</p>
    {/if}
    {#if inspectErr}
      <p class="error">{inspectErr}</p>
    {/if}

    {#if inspection}
      <section class="forwardOut">
        <span class="lbl">Forward output</span>
        <span class="val">{inspection.forwardOutput.toFixed(4)}</span>
        <span class="hint">
          (unit-scale; ×{inspection.evalScale.toFixed(0)} →
          {Math.round(inspection.forwardOutput * inspection.evalScale)} cp)
        </span>
      </section>

      {#if inspection.weightStats.length > 0}
        <section class="weights">
          <h3>Per-layer weight stats</h3>
          <table>
            <thead>
              <tr>
                <th>Layer</th>
                <th class="num">Mean</th>
                <th class="num">Std</th>
                <th class="num">Min</th>
                <th class="num">Max</th>
                <th class="num">NaN</th>
              </tr>
            </thead>
            <tbody>
              {#each inspection.weightStats as ws (ws.layer)}
                <tr>
                  <td>{ws.layer}</td>
                  <td class="num">{ws.mean.toFixed(4)}</td>
                  <td class="num">{ws.std.toFixed(4)}</td>
                  <td class="num">{ws.min.toFixed(4)}</td>
                  <td class="num">{ws.max.toFixed(4)}</td>
                  <td class="num" class:warn={ws.nanCount > 0}>{ws.nanCount}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </section>
      {:else}
        <p class="hint">No weight stats available for this rater.</p>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
    font-size: 0.95em;
  }
  .hint {
    color: var(--paper-ink-soft);
    font-style: italic;
    margin: 0;
  }
  .error {
    color: var(--p2, #a13a2a);
    margin: 0;
  }
  header {
    display: flex;
    gap: 0.5em;
    align-items: baseline;
    border-bottom: 1px solid var(--paper-line);
    padding-bottom: 0.4em;
  }
  .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .raterId, code {
    font-family: inherit;
    background: var(--paper-bg);
    padding: 0.05em 0.35em;
    border-radius: 3px;
    font-weight: 600;
  }
  .forwardOut {
    display: flex;
    gap: 0.5em;
    align-items: baseline;
  }
  .forwardOut .val {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    font-size: 1.15em;
  }
  .weights h3 {
    margin: 0 0 0.4em;
    font-size: 1em;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9em;
  }
  th, td {
    border-bottom: 1px solid var(--paper-line);
    padding: 0.3em 0.5em;
    text-align: left;
  }
  th {
    color: var(--paper-ink-soft);
    font-size: 0.88em;
    font-weight: 600;
  }
  td.num, th.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  td.warn {
    color: var(--p2, #a13a2a);
    font-weight: 600;
  }
</style>
