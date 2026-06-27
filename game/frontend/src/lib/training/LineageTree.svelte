<script lang="ts">
  // Panel 3 — Lineage Tree.
  //
  // Polls `read_rater_index` at 1 Hz. Each accepted rater is a node;
  // parent_id wires the tree. We hand-roll the SVG layout (no D3) — the
  // accepted set is small (tens of nodes for a long run) and a simple
  // tidy-tree algorithm keeps the dependency surface minimal.
  //
  // Clicking a node emits the rater_id to the shared "selected rater"
  // context, which the Network Inspector panel reads on the right.

  import { onMount, getContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createPollingStore } from "$lib/training/polling";
  import type { RaterIndex, IndexEntry } from "$lib/training/types";

  const getRunDir = getContext<() => string>("training:getRunDir");
  const getSelected = getContext<() => string | null>("training:getSelectedRaterId");
  const setSelected = getContext<(id: string | null) => void>("training:setSelectedRaterId");

  let runDir = $state<string>("");
  let index: RaterIndex | null = $state(null);
  let pollErr: string | null = $state(null);

  type IdxStore = ReturnType<typeof createPollingStore<RaterIndex>>;
  let store: IdxStore | null = $state(null);

  onMount(() => {
    runDir = getRunDir();
    if (!runDir) return;
    store = createPollingStore<RaterIndex>({
      invokeCmd: "read_rater_index",
      args: { runDir },
      intervalMs: 1000,
    });
  });

  $effect(() => {
    if (!store) return;
    return store.subscribe((v) => {
      index = v.data;
      pollErr = v.error;
    });
  });

  // --- Layout ---------------------------------------------------------------
  // IndexEntry doesn't carry parent_id (that lives on PopulationMember in the
  // status snapshot). For this panel we render a flat acceptance-order
  // ladder, which is honest about the data we actually have. Wiring real
  // parent edges would require cross-referencing the status snapshot — we
  // can add that later by widening the context API to expose both stores.
  type Node = { id: string; entry: IndexEntry; x: number; y: number };

  const layout = $derived.by(() => {
    const entries = index?.entries ?? [];
    if (entries.length === 0) return { nodes: [] as Node[], width: 0, height: 0 };
    const ROW = 64;
    const PAD = 24;
    const W = 200;
    const nodes: Node[] = entries.map((e, i) => ({
      id: e.id,
      entry: e,
      x: PAD,
      y: PAD + i * ROW,
    }));
    return {
      nodes,
      width: PAD * 2 + W,
      height: PAD * 2 + entries.length * ROW,
    };
  });

  const selected = $derived<string | null>(getSelected());
  function pick(id: string): void {
    setSelected(id);
  }
</script>

<div class="lineage">
  {#if pollErr}<p class="error">Index poll error: {pollErr}</p>{/if}

  {#if !index}
    <div class="empty">Waiting for the trainer to publish a rater index.</div>
  {:else if layout.nodes.length === 0}
    <div class="empty">No accepted raters yet. The first acceptance lands when a candidate clears both gauntlet tiers.</div>
  {:else}
    <svg viewBox={`0 0 ${layout.width} ${layout.height}`} preserveAspectRatio="xMinYMin meet">
      {#each layout.nodes as n (n.id)}
        <g
          class="node"
          class:selected={selected === n.id}
          transform={`translate(${n.x}, ${n.y})`}
          role="button"
          tabindex="0"
          onclick={() => pick(n.id)}
          onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") pick(n.id); }}
        >
          <rect x="0" y="0" width="160" height="44" rx="6" />
          <text x="80" y="18" text-anchor="middle" class="title">{n.entry.id}</text>
          <text x="80" y="34" text-anchor="middle" class="sub">{n.entry.stem.split("/").pop() ?? n.entry.stem}</text>
        </g>
      {/each}
    </svg>

    {#if index.tracks && (index.tracks.fast || index.tracks.slow || index.tracks.overall)}
      <footer class="tracks">
        <span>Tracks:</span>
        {#if index.tracks.fast}<span>fast → <code>{index.tracks.fast}</code></span>{/if}
        {#if index.tracks.slow}<span>slow → <code>{index.tracks.slow}</code></span>{/if}
        {#if index.tracks.overall}<span>overall → <code>{index.tracks.overall}</code></span>{/if}
      </footer>
    {/if}
  {/if}
</div>

<style>
  .lineage {
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
  svg {
    width: 100%;
    height: auto;
    max-height: 540px;
    background: var(--paper-bg);
    border: 1px dashed var(--paper-line);
    border-radius: 6px;
  }
  .node {
    cursor: pointer;
  }
  .node rect {
    fill: white;
    stroke: var(--paper-line-strong);
    stroke-width: 1.5;
  }
  .node:hover rect {
    stroke: var(--p1, #2b4a8a);
  }
  .node.selected rect {
    fill: rgba(43, 74, 138, 0.12);
    stroke: var(--p1, #2b4a8a);
    stroke-width: 2.5;
  }
  .node text {
    font-family: inherit;
    pointer-events: none;
  }
  .node text.title {
    font-size: 13px;
    font-weight: 600;
  }
  .node text.sub {
    font-size: 11px;
    fill: var(--paper-ink-soft);
  }
  .tracks {
    display: flex;
    flex-wrap: wrap;
    gap: 1em;
    font-size: 0.92em;
    color: var(--paper-ink-soft);
  }
  .tracks code {
    font-family: inherit;
    background: var(--paper-bg);
    padding: 0.05em 0.35em;
    border-radius: 3px;
  }
</style>
