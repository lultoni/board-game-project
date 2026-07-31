<script lang="ts">
  interface LogEntry {
    index: number;   // 1-based ply number
    notation: string;
    isP1: boolean;
  }

  interface Props {
    entries: LogEntry[];
    busy?: boolean;
    matchLogAvailable?: boolean;
    selectedPly?: number | null;
    /** The live-head ply index captured when preview was entered — the "you
     *  left the present here" marker. Pinned; does not track the live head. */
    leftAtPly?: number | null;
    onCopyFen?: () => void;
    onCopyLog?: () => void;
    onDownloadLog?: () => void;
    onSelectPly?: (index: number | null) => void;
  }

  let {
    entries = [],
    busy = false,
    matchLogAvailable = false,
    selectedPly = null,
    leftAtPly = null,
    onCopyFen,
    onCopyLog,
    onDownloadLog,
    onSelectPly,
  }: Props = $props();

  // Auto-scroll to bottom when entries change — BUT only when viewing the
  // present. While parked on a past ply (time-travel preview), new live moves
  // keep appending; yanking the list to the bottom would scroll the inspected
  // ply out from under the user, so we hold position until they return to live.
  let listEl = $state<HTMLElement | null>(null);
  $effect(() => {
    void entries.length;
    if (selectedPly !== null) return;
    if (listEl) listEl.scrollTop = listEl.scrollHeight;
  });

  // True when the preview is behind the live head — the game has moved on while
  // the player inspects the past. Drives the "jump to latest" catch-up control.
  const behindLive = $derived(selectedPly !== null && selectedPly < entries.length);
</script>

<div class="action-log">
  <div class="log-header">
    <span class="log-title">Action Log</span>
    <div class="log-actions">
      <button type="button" disabled={busy} onclick={onCopyFen} title="Copy FEN">FEN</button>
      <button type="button" disabled={busy || !matchLogAvailable} onclick={onCopyLog} title="Copy log">Copy</button>
      <button type="button" disabled={busy || !matchLogAvailable} onclick={onDownloadLog} title="Download log">↓</button>
    </div>
  </div>
  {#if behindLive}
    <button
      type="button"
      class="jump-latest"
      title="The game moved on while you were looking — return to the live position"
      onclick={() => onSelectPly?.(null)}
    >
      ⏭ Jump to latest ({entries.length})
    </button>
  {/if}
  <div class="log-list" bind:this={listEl}>
    {#if entries.length === 0}
      <span class="log-empty">No actions yet</span>
    {:else}
      {#each entries as entry (entry.index)}
        <button
          type="button"
          class="log-entry"
          class:p1={entry.isP1}
          class:p2={!entry.isP1}
          class:selected={selectedPly === entry.index}
          class:present-marker={leftAtPly !== null && entry.index === leftAtPly}
          onclick={() => onSelectPly?.(selectedPly === entry.index ? null : entry.index)}
        >
          <span class="log-idx">{entry.index}.</span>
          <span class="log-notation">{entry.notation}</span>
          {#if selectedPly === entry.index}
            <span class="tag tag-viewing">viewing</span>
          {:else if leftAtPly !== null && entry.index === leftAtPly}
            <span class="tag tag-present">left here</span>
          {/if}
        </button>
      {/each}
      {#if selectedPly !== null}
        <button
          type="button"
          class="log-entry log-present"
          onclick={() => onSelectPly?.(null)}
        >
          <span class="log-idx">▶</span>
          <span class="log-notation">Back to present</span>
        </button>
      {/if}
    {/if}
  </div>
</div>

<style>
  .action-log {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    border-radius: 5px;
    background: var(--paper-bg, #f3ecd9);
    overflow: hidden;
    min-height: 0;
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.3rem 0.5rem;
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    flex-shrink: 0;
  }

  .log-title {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--paper-ink-soft, #6a6055);
    font-weight: 600;
  }

  .log-actions {
    display: flex;
    gap: 0.25rem;
  }

  .log-actions button {
    font: inherit;
    font-size: 0.7rem;
    padding: 0.15em 0.5em;
    border: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    border-radius: 3px;
    background: transparent;
    color: var(--paper-ink-soft, #6a6055);
    cursor: pointer;
    transition: border-color 80ms, color 80ms;
  }

  .log-actions button:hover:not(:disabled) {
    border-color: var(--paper-line-strong);
    color: var(--paper-ink);
  }

  .log-actions button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .jump-latest {
    flex-shrink: 0;
    width: 100%;
    font: inherit;
    font-size: 0.74rem;
    font-weight: 600;
    padding: 0.3em 0.5em;
    border: none;
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    background: color-mix(in srgb, var(--accent, #c79b3a) 22%, var(--paper-bg, #f3ecd9));
    color: var(--paper-ink, #3a2f1f);
    cursor: pointer;
    text-align: center;
    transition: background 80ms;
  }
  .jump-latest:hover {
    background: color-mix(in srgb, var(--accent, #c79b3a) 34%, var(--paper-bg, #f3ecd9));
  }

  .log-list {
    overflow-y: auto;
    flex: 1;
    padding: 0.3rem 0.4rem;
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 220px;
  }

  .log-empty {
    font-size: 0.75rem;
    color: var(--paper-ink-soft, #6a6055);
    font-style: italic;
    padding: 0.3rem 0;
  }

  .log-entry {
    display: flex;
    gap: 0.4rem;
    align-items: baseline;
    padding: 0.15em 0.3em;
    border-radius: 3px;
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    font: inherit;
    transition: background 80ms;
  }
  .log-entry:hover { background: var(--paper-square-light, #ece2c8); }
  .log-entry.p1 { background: rgba(75, 107, 138, 0.07); }
  .log-entry.p2 { background: rgba(169, 75, 59, 0.07); }
  /* The ply you are currently inspecting: strong accent fill + left bar + caret. */
  .log-entry.selected {
    background: color-mix(in srgb, var(--accent, #c79b3a) 28%, var(--paper-bg));
    font-weight: 700;
    box-shadow: inset 3px 0 0 0 var(--accent, #c79b3a);
  }
  /* The live head you jumped away from ("left here"): dashed left bar so it
     reads as "the present is over here" while you look at the past. */
  .log-entry.present-marker:not(.selected) {
    box-shadow: inset 3px 0 0 0 color-mix(in srgb, var(--accent, #c79b3a) 55%, transparent);
  }
  .log-entry.log-present {
    color: var(--accent, #c79b3a);
    font-style: italic;
    margin-top: 2px;
    border-top: 1px solid var(--paper-line);
  }

  .tag {
    margin-left: auto;
    flex-shrink: 0;
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.05em 0.4em;
    border-radius: 3px;
    line-height: 1.4;
  }
  .tag-viewing {
    background: var(--accent, #c79b3a);
    color: var(--paper-bg, #f3ecd9);
  }
  .tag-present {
    background: transparent;
    color: var(--accent, #c79b3a);
    border: 1px solid color-mix(in srgb, var(--accent, #c79b3a) 60%, transparent);
  }

  .log-idx {
    color: var(--paper-ink-soft, #6a6055);
    font-size: 0.68rem;
    min-width: 2ch;
    text-align: right;
    flex-shrink: 0;
  }

  .log-notation {
    font-weight: 500;
    color: var(--paper-ink, #3a2f1f);
    word-break: break-all;
  }
</style>
