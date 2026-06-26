<script lang="ts">
  import { formatAction } from "$lib/engine";
  import type { InspectorNode } from "$lib/state/inspector-store.svelte";

  interface Props {
    node: InspectorNode;
    selected: boolean;
    depth: number;
    onSelect: (id: string) => void;
    onMarkPoi: (id: string) => void;
    onUnmarkPoi: (id: string) => void;
  }
  let { node, selected, depth, onSelect, onMarkPoi, onUnmarkPoi }: Props = $props();

  function fmtEdge(): string {
    if (node.edgeAction === null) return "[start]";
    return formatAction(node.edgeAction);
  }
</script>

<div
  class="row"
  class:selected
  style="padding-left: {0.4 + depth * 0.9}rem"
  role="button"
  tabindex="0"
  onclick={() => onSelect(node.id)}
  onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onSelect(node.id); } }}
>
  <span class="ply">{node.ply}.</span>
  <span class="edge">{fmtEdge()}</span>
  {#if node.label && node.parent !== null}
    <span class="star" title={node.label}>★ {node.label}</span>
  {/if}
  {#if node.parent !== null}
    {#if node.label}
      <button class="poi-btn" type="button" onclick={(e) => { e.stopPropagation(); onUnmarkPoi(node.id); }} title="Remove POI">✕</button>
    {:else}
      <button class="poi-btn" type="button" onclick={(e) => { e.stopPropagation(); onMarkPoi(node.id); }} title="Mark as POI">☆</button>
    {/if}
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    width: 100%;
    padding: 0.2rem 0.4rem;
    border: 0;
    background: transparent;
    text-align: left;
    font: inherit;
    cursor: pointer;
    border-radius: 4px;
  }
  .row:hover { background: rgba(0, 0, 0, 0.04); }
  .row.selected {
    background: var(--accent, #5a7cd6);
    color: #fff;
  }
  .row.selected:hover { background: var(--accent, #5a7cd6); }
  .ply { color: var(--paper-ink-soft); font-variant-numeric: tabular-nums; min-width: 2em; }
  .row.selected .ply { color: rgba(255, 255, 255, 0.85); }
  .edge { flex: 1; font-size: 0.9rem; }
  .star { font-size: 0.8rem; color: #cc8b2a; }
  .row.selected .star { color: #ffd070; }
  .poi-btn {
    background: transparent;
    border: 1px solid transparent;
    padding: 0 0.3em;
    font-size: 0.85em;
    cursor: pointer;
    color: inherit;
    opacity: 0.55;
  }
  .poi-btn:hover { opacity: 1; border-color: var(--paper-line); }
</style>
