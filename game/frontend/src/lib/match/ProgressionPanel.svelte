<script lang="ts">
  interface Props {
    /** Current round number (1-based). */
    roundNumber: number;
    /** How many rounds ahead to show (default 5). */
    lookahead?: number;
    /** How many past rounds to show above current (default 2). */
    lookback?: number;
  }

  let { roundNumber, lookahead = 5, lookback = 2 }: Props = $props();

  function incomeForRound(r: number): number {
    if (r <= 1) return 0;
    return 2 + Math.floor(r / 5);
  }

  function skillActionsForRound(r: number): number {
    return 2 + Math.floor((r - 1) / 10);
  }

  // Past rounds (dimmed history) + current + future
  const rows = $derived.by(() => {
    const result = [];
    const start = Math.max(1, roundNumber - lookback);
    for (let r = start; r <= roundNumber + lookahead; r++) {
      const income = incomeForRound(r);
      const actions = skillActionsForRound(r);
      const prevIncome = incomeForRound(r - 1);
      const prevActions = skillActionsForRound(r - 1);
      result.push({
        round: r,
        income,
        actions,
        incomeUp: income > prevIncome,
        actionsUp: actions > prevActions,
        isCurrent: r === roundNumber,
        isPast: r < roundNumber,
      });
    }
    return result;
  });
</script>

<div class="progression-panel">
  <div class="panel-header">Rounds</div>
  <div class="grid">
    <span class="col-head">Rnd</span>
    <span class="col-head">Income</span>
    <span class="col-head">Skill act.</span>
    {#each rows as row}
      <span class="cell round" class:current={row.isCurrent} class:past={row.isPast}>{row.round}</span>
      <span class="cell" class:current={row.isCurrent} class:past={row.isPast} class:bump={row.incomeUp && !row.isPast}>
        {row.round <= 1 ? "-" : `$${row.income}`}
        {#if row.incomeUp && !row.isPast}<span class="up-arrow" aria-label="increase">↑</span>{/if}
      </span>
      <span class="cell" class:current={row.isCurrent} class:past={row.isPast} class:bump={row.actionsUp && !row.isPast}>
        {row.actions}
        {#if row.actionsUp && !row.isPast}<span class="up-arrow" aria-label="increase">↑</span>{/if}
      </span>
    {/each}
  </div>
  <p class="footnote">Per player, per turn</p>
</div>

<style>
  .progression-panel {
    padding: 0.45rem 0.55rem;
    border: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    border-radius: 5px;
    background: var(--paper-bg, #f3ecd9);
  }

  .panel-header {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--paper-ink-soft, #6a6055);
    margin-bottom: 0.4rem;
  }

  .grid {
    display: grid;
    grid-template-columns: 2.2ch 1fr 1fr;
    row-gap: 2px;
    column-gap: 0.3rem;
    align-items: center;
  }

  .col-head {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--paper-ink-soft, #6a6055);
    padding-bottom: 2px;
  }

  .cell {
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
    padding: 1px 3px;
    border-radius: 3px;
    transition: background 150ms;
  }

  .cell.round {
    color: var(--paper-ink-soft, #6a6055);
  }

  .cell.past {
    opacity: 0.42;
    color: var(--paper-ink-soft, #6a6055);
  }

  .cell.current {
    font-weight: 700;
    color: var(--paper-ink, #3a2f1f);
    background: var(--paper-square-light, #ece2c8);
  }

  .cell.bump {
    color: #5a8a3a;
  }

  .cell.current.bump {
    background: rgba(90, 138, 58, 0.15);
    color: #3a6a1a;
  }

  .up-arrow {
    font-size: 0.65rem;
    margin-left: 1px;
    font-style: normal;
  }

  .footnote {
    margin: 0.35rem 0 0;
    font-size: 0.62rem;
    color: var(--paper-ink-soft, #6a6055);
    font-style: italic;
  }
</style>
