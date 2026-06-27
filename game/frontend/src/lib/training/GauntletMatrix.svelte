<script lang="ts">
  // Panel 5 — Gauntlet Matrix.
  //
  // Polls `read_gauntlet_matrix` at 1 Hz (low cadence — entries change
  // only when a series finishes, which is rare relative to per-ply
  // updates). Cells are tinted by win-rate (challenger row vs. defender
  // col, in the dropdown-selected bracket). The diagonal is blank.

  import { onMount, getContext } from "svelte";
  import { createPollingStore } from "$lib/training/polling";
  import type { GauntletMatrix, MatrixEntry } from "$lib/training/types";

  const getRunDir = getContext<() => string>("training:getRunDir");

  let runDir = $state<string>("");
  let matrix: GauntletMatrix | null = $state(null);
  let pollErr: string | null = $state(null);
  let bracket = $state<string>("fast");

  type MatStore = ReturnType<typeof createPollingStore<GauntletMatrix>>;
  let store: MatStore | null = $state(null);

  onMount(() => {
    runDir = getRunDir();
    if (!runDir) return;
    store = createPollingStore<GauntletMatrix>({
      invokeCmd: "read_gauntlet_matrix",
      args: { runDir },
      intervalMs: 1000,
    });
  });

  $effect(() => {
    if (!store) return;
    return store.subscribe((v) => {
      matrix = v.data;
      pollErr = v.error;
    });
  });

  // Build the axis list: the union of challenger + defender ids that have
  // any entry in any bracket. Sorted for stable rendering.
  const axis = $derived.by(() => {
    const set = new Set<string>();
    for (const e of matrix?.entries ?? []) {
      set.add(e.challenger);
      set.add(e.defender);
    }
    return Array.from(set).sort();
  });

  // Available brackets, derived from the data.
  const brackets = $derived.by(() => {
    const set = new Set<string>();
    for (const e of matrix?.entries ?? []) set.add(e.bracket);
    return Array.from(set).sort();
  });

  // Auto-pick the first available bracket if the current selection has no
  // data yet.
  $effect(() => {
    if (brackets.length === 0) return;
    if (!brackets.includes(bracket)) bracket = brackets[0];
  });

  // (row, col) → entry for the active bracket.
  const cells = $derived.by(() => {
    const map = new Map<string, MatrixEntry>();
    for (const e of matrix?.entries ?? []) {
      if (e.bracket !== bracket) continue;
      map.set(`${e.challenger}::${e.defender}`, e);
    }
    return map;
  });

  function cellWinRate(challenger: string, defender: string): number | null {
    if (challenger === defender) return null;
    const e = cells.get(`${challenger}::${defender}`);
    if (!e) return null;
    const g = e.result.games_played;
    if (g === 0) return null;
    return (e.result.candidate_wins + 0.5 * e.result.indecisive) / g;
  }

  function cellGames(challenger: string, defender: string): number {
    const e = cells.get(`${challenger}::${defender}`);
    return e?.result.games_played ?? 0;
  }

  // Win-rate → CSS background. Centered at 0.5 (neutral grey), red below,
  // blue above. Matches P1/P2 colour vocabulary.
  function cellBg(wr: number | null): string {
    if (wr === null) return "transparent";
    if (wr >= 0.5) {
      const a = Math.min(1, (wr - 0.5) * 2);
      return `rgba(43, 74, 138, ${a.toFixed(2)})`;
    } else {
      const a = Math.min(1, (0.5 - wr) * 2);
      return `rgba(161, 58, 42, ${a.toFixed(2)})`;
    }
  }
</script>

<div class="matrix">
  {#if pollErr}<p class="error">Matrix poll error: {pollErr}</p>{/if}

  <header>
    <label>
      <span class="lbl">Bracket</span>
      <select bind:value={bracket}>
        {#each brackets as b}
          <option value={b}>{b}</option>
        {/each}
        {#if brackets.length === 0}
          <option value="fast" disabled>(no data)</option>
        {/if}
      </select>
    </label>
    <span class="legend">
      Cells are challenger (row) vs. defender (col). Blue = challenger
      wins; red = defender wins; intensity scales with win rate.
    </span>
  </header>

  {#if !matrix}
    <div class="empty">Waiting for the trainer to publish a gauntlet matrix.</div>
  {:else if axis.length === 0}
    <div class="empty">No gauntlet series have been recorded yet.</div>
  {:else}
    <div class="tableWrap">
      <table>
        <thead>
          <tr>
            <th></th>
            {#each axis as col}
              <th class="col"><code>{col}</code></th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each axis as row}
            <tr>
              <th class="row"><code>{row}</code></th>
              {#each axis as col}
                {@const wr = cellWinRate(row, col)}
                {@const games = cellGames(row, col)}
                <td
                  class:diagonal={row === col}
                  style:background={cellBg(wr)}
                  title={row === col ? "self" : wr === null ? "no data" : `${(wr * 100).toFixed(1)}% over ${games} games`}
                >
                  {#if row === col}
                    <span class="dash">—</span>
                  {:else if wr === null}
                    <span class="dot">·</span>
                  {:else}
                    <span class="wr">{(wr * 100).toFixed(0)}</span>
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .matrix {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
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
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.8em;
    border-bottom: 1px solid var(--paper-line);
    padding-bottom: 0.4em;
  }
  header label {
    display: flex;
    align-items: center;
    gap: 0.4em;
  }
  header select {
    font: inherit;
    padding: 0.2em 0.4em;
    border: 1px solid var(--paper-line);
    border-radius: 4px;
    background: white;
  }
  .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .legend {
    color: var(--paper-ink-soft);
    font-size: 0.88em;
    flex: 1;
  }
  .tableWrap {
    overflow: auto;
  }
  table {
    border-collapse: collapse;
    font-size: 0.88em;
  }
  th, td {
    border: 1px solid var(--paper-line);
    text-align: center;
    padding: 0.25em 0.4em;
    min-width: 2.6em;
  }
  th.col, th.row {
    font-weight: 600;
    color: var(--paper-ink-soft);
  }
  th.col code, th.row code {
    font-family: inherit;
    font-size: 0.88em;
  }
  td.diagonal {
    background: var(--paper-bg) !important;
    color: var(--paper-ink-soft);
  }
  td .wr {
    font-variant-numeric: tabular-nums;
    color: white;
    text-shadow: 0 0 2px rgba(0, 0, 0, 0.45);
    font-weight: 600;
  }
  td .dot {
    color: var(--paper-ink-soft);
  }
  td .dash {
    color: var(--paper-ink-soft);
  }
</style>
