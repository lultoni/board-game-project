<script lang="ts">
  import type { EvalReport, TermEntry } from "$lib/engine";
  import { aiSearch } from "$lib/state/ai-search.svelte";

  // Reads the current eval report and prior-round snapshot from the ai-search
  // store. Reading straight from the store isolates re-renders to the fields we
  // actually use (not every depth-tick / hover update on the route).
  const report = $derived<EvalReport | null>(aiSearch.evalReport);
  const prevReport = $derived<EvalReport | null>(aiSearch.prevRoundReport);

  const SATURATION = 1000;

  interface Row {
    label: string;
    /** Owner-signed P1 magnitude for display (positive side). */
    p1: number;
    p2: number;
    /** This term's P1-POV signed contribution to total. */
    signed: number;
    /** Prior-round Δ (this term's signed) for the vs-prev column, if available. */
    prevDelta: number | null;
    emphasis?: boolean;
  }

  /** All active terms of a report, aggregate then side-level, in wire order. */
  function allTerms(r: EvalReport): TermEntry[] {
    return [...r.terms, ...r.side_terms];
  }

  function buildRows(cur: EvalReport | null, prev: EvalReport | null): Row[] {
    if (cur === null) return [];
    const prevByName = new Map<string, number>();
    if (prev !== null) {
      for (const t of allTerms(prev)) prevByName.set(t.name, t.signed);
    }
    const rows: Row[] = [
      {
        label: "Total",
        p1: Math.max(0, cur.total),
        p2: Math.max(0, -cur.total),
        signed: cur.total,
        prevDelta: prev === null ? null : cur.total - prev.total,
        emphasis: true,
      },
    ];
    for (const t of allTerms(cur)) {
      const prevSigned = prevByName.get(t.name);
      rows.push({
        label: t.label,
        p1: t.p1,
        p2: t.p2,
        signed: t.signed,
        prevDelta: prevSigned === undefined ? null : t.signed - prevSigned,
      });
    }
    return rows;
  }

  const rows = $derived<Row[]>(buildRows(report, prevReport));

  const showVsPrev = $derived(prevReport !== null);

  // Fill fraction on [-1, 1]. Positive = P1 fills downward (P1 sits at
  // bottom of board), negative = P2 fills upward.
  const fillFrac = $derived(
    report === null
      ? 0
      : Math.max(-1, Math.min(1, report.total / SATURATION))
  );

  const fillPct = $derived(Math.abs(fillFrac) * 50);

  function fmtSigned(n: number): string {
    if (n === 0) return "0";
    return n > 0 ? `+${n}` : `${n}`;
  }
</script>

<aside class="eval-panel" aria-label="Eval breakdown">
  <div class="bar-column">
    <div class="bar-track">
      <div class="bar-center-rule"></div>
      {#if report !== null}
        {#if fillFrac > 0}
          <div class="bar-fill bar-fill--p1" style="height: {fillPct}%"></div>
        {:else if fillFrac < 0}
          <div class="bar-fill bar-fill--p2" style="height: {fillPct}%"></div>
        {/if}
      {/if}
    </div>
    <div class="bar-legend">
      <span class="legend-p2">P2</span>
      <span class="legend-p1">P1</span>
    </div>
  </div>

  <div class="rows-column">
    {#if report === null}
      <div class="empty">No position loaded</div>
    {:else}
      <div class="header" class:with-prev={showVsPrev}>
        <span class="col-label">Component</span>
        <span class="col-p1">P1</span>
        <span class="col-p2">P2</span>
        <span class="col-delta">Δ</span>
        {#if showVsPrev}
          <span class="col-vs" title="Change vs previous round">vs prev</span>
        {/if}
      </div>
      {#each rows as row, i (row.label)}
        <div
          class="row"
          class:row--emphasis={row.emphasis}
          class:with-prev={showVsPrev}
        >
          <span class="col-label">{row.label}</span>
          <span class="col-p1 num">{fmtSigned(row.p1)}</span>
          <span class="col-p2 num">{fmtSigned(row.p2)}</span>
          <span
            class="col-delta num"
            class:pos={row.signed > 0}
            class:neg={row.signed < 0}
          >{fmtSigned(row.signed)}</span>
          {#if showVsPrev}
            <span
              class="col-vs num"
              class:pos={row.prevDelta !== null && row.prevDelta > 0}
              class:neg={row.prevDelta !== null && row.prevDelta < 0}
            >{row.prevDelta === null ? "-" : fmtSigned(row.prevDelta)}</span>
          {/if}
        </div>
        {#if i === 0}
          <div class="row-divider"></div>
        {/if}
      {/each}
    {/if}
  </div>
</aside>

<style>
  .eval-panel {
    display: flex;
    flex-direction: row;
    gap: 0.7rem;
    padding: 0.7rem 0.85rem;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
    min-height: 0;
    font-family: inherit;
    color: var(--paper-ink, #3a2f1f);
    /* Container-query enabled so type + spacing scale with the actual room
       the parent gives us, not the viewport width. */
    container-type: inline-size;
  }

  .bar-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    width: clamp(18px, 6cqw, 34px);
    flex: 0 0 auto;
  }

  .bar-track {
    position: relative;
    flex: 1 1 auto;
    width: 100%;
    min-height: 200px;
    border: 1px solid var(--paper-line, rgba(58,47,31,0.35));
    border-radius: 3px;
    background: var(--paper-bg-soft, #ece3cc);
    overflow: hidden;
  }
  .bar-center-rule {
    position: absolute;
    left: -1px;
    right: -1px;
    top: 50%;
    height: 1px;
    background: var(--paper-line-strong, #8a7a4e);
    transform: translateY(-0.5px);
    pointer-events: none;
    z-index: 1;
  }
  .bar-fill {
    position: absolute;
    left: 0;
    right: 0;
    transition: height 180ms ease-out;
  }
  /* P1 sits at the bottom of the board, so P1 winning fills downward from the
     centre rule. P2 winning fills upward. */
  .bar-fill--p1 {
    top: 50%;
    background: #3a5a7a; /* warm ink-blue */
  }
  .bar-fill--p2 {
    bottom: 50%;
    background: #a03a2a; /* warm ink-red */
  }

  .bar-legend {
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: clamp(0.55rem, 2.2cqw, 0.75rem);
    color: var(--paper-ink-soft, #6a6055);
    gap: 0.15rem;
  }
  .legend-p1 { color: #3a5a7a; }
  .legend-p2 { color: #a03a2a; }

  .rows-column {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .header,
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 3.6rem 3.6rem 4rem;
    gap: 0.4rem;
    align-items: baseline;
    font-size: clamp(0.72rem, 2.8cqw, 0.95rem);
  }
  .header.with-prev,
  .row.with-prev {
    grid-template-columns: minmax(0, 1fr) 3.2rem 3.2rem 3.6rem 3.6rem;
  }
  .col-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .col-vs {
    text-align: right;
    font-weight: 600;
    color: var(--paper-ink-soft, #6a6055);
  }
  .col-vs.pos { color: #3a7a3a; }
  .col-vs.neg { color: #a03030; }
  .header {
    font-size: clamp(0.6rem, 2.2cqw, 0.78rem);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--paper-ink-soft, #6a6055);
    padding-bottom: 0.25rem;
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    margin-bottom: 0.2rem;
  }
  .row {
    padding: 0.15rem 0;
  }
  .row--emphasis {
    font-size: clamp(0.85rem, 3.4cqw, 1.15rem);
    font-weight: 700;
  }

  .col-label { color: var(--paper-ink, #3a2f1f); }
  .col-p1, .col-p2 { color: var(--paper-ink-soft, #6a6055); text-align: right; }
  .col-delta { text-align: right; font-weight: 600; }

  .num {
    font-variant-numeric: tabular-nums;
  }

  .col-delta.pos { color: #3a7a3a; }
  .col-delta.neg { color: #a03030; }

  .row-divider {
    height: 1px;
    background: var(--paper-line, rgba(58,47,31,0.2));
    margin: 0.2rem 0;
  }

  .empty {
    padding: 0.5rem 0.1rem;
    font-size: 0.8rem;
    font-style: italic;
    color: var(--paper-ink-soft, #6a6055);
    text-align: center;
  }
</style>
