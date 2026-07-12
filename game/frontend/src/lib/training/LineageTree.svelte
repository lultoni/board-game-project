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
  // IndexEntry carries `parent_id` (added alongside this panel). We build a
  // children-of map and walk the forest from each root in acceptance order
  // to lay out a tidy tree. Roots stack vertically; each subtree extends
  // rightward column-by-column. Depth-first traversal assigns row numbers
  // by post-order, which keeps siblings adjacent.
  type Node = {
    id: string;
    entry: IndexEntry;
    children: Node[];
    depth: number;
    row: number;
    x: number;
    y: number;
  };

  const layout = $derived.by(() => {
    const entries = index?.entries ?? [];
    if (entries.length === 0) return { nodes: [] as Node[], edges: [] as { x1: number; y1: number; x2: number; y2: number }[], width: 0, height: 0 };

    const byId = new Map<string, Node>();
    for (const e of entries) {
      byId.set(e.id, {
        id: e.id,
        entry: e,
        children: [],
        depth: 0,
        row: 0,
        x: 0,
        y: 0,
      });
    }
    const roots: Node[] = [];
    for (const e of entries) {
      const node = byId.get(e.id)!;
      const pid = e.parent_id ?? null;
      if (pid && byId.has(pid)) {
        byId.get(pid)!.children.push(node);
      } else {
        roots.push(node);
      }
    }

    // Post-order row assignment: every leaf takes the next row; every
    // internal node sits at the midpoint of its children's rows.
    let nextRow = 0;
    const assign = (n: Node, depth: number): void => {
      n.depth = depth;
      if (n.children.length === 0) {
        n.row = nextRow++;
      } else {
        for (const c of n.children) assign(c, depth + 1);
        const first = n.children[0].row;
        const last = n.children[n.children.length - 1].row;
        n.row = (first + last) / 2;
      }
    };
    for (const r of roots) assign(r, 0);

    const ROW = 56;
    const COL = 200;
    const PAD = 24;
    const NODE_W = 170;
    const NODE_H = 44;

    const nodes: Node[] = [];
    const visit = (n: Node): void => {
      n.x = PAD + n.depth * COL;
      n.y = PAD + n.row * ROW;
      nodes.push(n);
      for (const c of n.children) visit(c);
    };
    for (const r of roots) visit(r);

    const edges = nodes.flatMap((n) =>
      n.children.map((c) => ({
        x1: n.x + NODE_W,
        y1: n.y + NODE_H / 2,
        x2: c.x,
        y2: c.y + NODE_H / 2,
      })),
    );

    const maxX = Math.max(...nodes.map((n) => n.x + NODE_W), 0);
    const maxY = Math.max(...nodes.map((n) => n.y + NODE_H), 0);
    return { nodes, edges, width: maxX + PAD, height: maxY + PAD };
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
      {#each layout.edges as e}
        <path
          class="edge"
          d={`M ${e.x1} ${e.y1} C ${(e.x1 + e.x2) / 2} ${e.y1}, ${(e.x1 + e.x2) / 2} ${e.y2}, ${e.x2} ${e.y2}`}
          fill="none"
        />
      {/each}
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
          <rect x="0" y="0" width="170" height="44" rx="6" />
          <text x="85" y="18" text-anchor="middle" class="title">{n.entry.id}</text>
          <text x="85" y="34" text-anchor="middle" class="sub">
            {#if n.entry.bracket_results && Object.keys(n.entry.bracket_results).length > 0}
              {@const br = Object.values(n.entry.bracket_results)[0]}
              {@const g = br.games_played}
              {@const wr = g > 0 ? ((br.candidate_wins + 0.5 * br.indecisive) / g * 100).toFixed(0) : "—"}
              {wr}% · {g}g · {n.entry.accepted_at.slice(0, 10)}
            {:else}
              {n.entry.accepted_at.slice(0, 10)}
            {/if}
          </text>
        </g>
      {/each}
    </svg>

    {#if index.tracks && index.tracks.champion}
      <footer class="tracks">
        <span>Champion:</span>
        <span><code>{index.tracks.champion}</code></span>
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
  .edge {
    stroke: var(--paper-ink-soft);
    stroke-width: 1.5;
    opacity: 0.6;
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
