<script lang="ts">
  import type { EvalBreakdown } from "$lib/engine";
  import { aiSearch } from "$lib/state/ai-search.svelte";

  // Reads the current heuristic breakdown and prior-round snapshot from the
  // ai-search store. Previously these came in as props from /match/, which
  // meant every unrelated re-render on the route (depth-tick during a search,
  // hover-square updates) also fired this component's $derived chain. Reading
  // straight from the store isolates re-renders to the fields we actually use.
  const breakdown = $derived<EvalBreakdown | null>(aiSearch.heuristicEvalBreakdown);
  const prevBreakdown = $derived<EvalBreakdown | null>(aiSearch.prevRoundBreakdown);

  const SATURATION = 1000;

  interface Row {
    label: string;
    p1: number;
    p2: number;
    /** Prior-round Δ (p1 - p2) for this row, if available. */
    prevDelta: number | null;
    emphasis?: boolean;
  }

  function rowVsPrev(cur: EvalBreakdown | null, prev: EvalBreakdown | null): Row[] {
    if (cur === null) return [];
    const dPrev = (a1: number, a2: number, b1: number, b2: number): number =>
      (a1 - a2) - (b1 - b2);
    // E9 shown as raw range (0..=4); its weighted contribution to total is
    // (p1 - p2) * 500 on the Rust side. The Δ column here reflects that weight
    // so the row's contribution is legible against the other rows.
    const REACH_WEIGHT = 500;
    const totalCurP1 = cur.material_p1 + cur.hp_p1 + cur.armor_p1 + cur.skills_p1 + cur.money_p1 + cur.mobility_p1 + cur.threat_p1 + cur.skill_act_p1 + cur.offensive_range_p1 * REACH_WEIGHT;
    const totalCurP2 = cur.material_p2 + cur.hp_p2 + cur.armor_p2 + cur.skills_p2 + cur.money_p2 + cur.mobility_p2 + cur.threat_p2 + cur.skill_act_p2 + cur.offensive_range_p2 * REACH_WEIGHT;
    const totalPrev = prev === null ? null :
      (prev.material_p1 + prev.hp_p1 + prev.armor_p1 + prev.skills_p1 + prev.money_p1 + prev.mobility_p1 + prev.threat_p1 + prev.skill_act_p1 + prev.offensive_range_p1 * REACH_WEIGHT)
      - (prev.material_p2 + prev.hp_p2 + prev.armor_p2 + prev.skills_p2 + prev.money_p2 + prev.mobility_p2 + prev.threat_p2 + prev.skill_act_p2 + prev.offensive_range_p2 * REACH_WEIGHT);
    return [
      { label: "Total",     p1: totalCurP1, p2: totalCurP2,
                            prevDelta: totalPrev === null ? null : (totalCurP1 - totalCurP2) - totalPrev,
                            emphasis: true },
      { label: "Material",  p1: cur.material_p1,   p2: cur.material_p2,
                            prevDelta: prev ? dPrev(cur.material_p1, cur.material_p2, prev.material_p1, prev.material_p2) : null },
      { label: "HP",        p1: cur.hp_p1,         p2: cur.hp_p2,
                            prevDelta: prev ? dPrev(cur.hp_p1, cur.hp_p2, prev.hp_p1, prev.hp_p2) : null },
      { label: "Armor",     p1: cur.armor_p1,      p2: cur.armor_p2,
                            prevDelta: prev ? dPrev(cur.armor_p1, cur.armor_p2, prev.armor_p1, prev.armor_p2) : null },
      { label: "Skills",    p1: cur.skills_p1,     p2: cur.skills_p2,
                            prevDelta: prev ? dPrev(cur.skills_p1, cur.skills_p2, prev.skills_p1, prev.skills_p2) : null },
      // E9 renders as a differential-only row: whichever side has higher raw
      // reach shows the weighted advantage, the other shows 0. The eval term
      // itself is `(p1 - p2) * WEIGHT` so per-side magnitudes aren't
      // meaningful in isolation - showing both sides at raw*WEIGHT was
      // misleading (implied additive contribution). Reach flag has no
      // per-side "score", only a differential.
      { label: "Off reach", p1: Math.max(0, (cur.offensive_range_p1 - cur.offensive_range_p2)) * REACH_WEIGHT,
                            p2: Math.max(0, (cur.offensive_range_p2 - cur.offensive_range_p1)) * REACH_WEIGHT,
                            prevDelta: prev
                              ? (Math.max(0, cur.offensive_range_p1 - cur.offensive_range_p2)
                                 - Math.max(0, cur.offensive_range_p2 - cur.offensive_range_p1)) * REACH_WEIGHT
                                - (Math.max(0, prev.offensive_range_p1 - prev.offensive_range_p2)
                                   - Math.max(0, prev.offensive_range_p2 - prev.offensive_range_p1)) * REACH_WEIGHT
                              : null },
      { label: "Money",     p1: cur.money_p1,      p2: cur.money_p2,
                            prevDelta: prev ? dPrev(cur.money_p1, cur.money_p2, prev.money_p1, prev.money_p2) : null },
      { label: "Reach",     p1: cur.mobility_p1,   p2: cur.mobility_p2,
                            prevDelta: prev ? dPrev(cur.mobility_p1, cur.mobility_p2, prev.mobility_p1, prev.mobility_p2) : null },
      { label: "Threat",    p1: cur.threat_p1,     p2: cur.threat_p2,
                            prevDelta: prev ? dPrev(cur.threat_p1, cur.threat_p2, prev.threat_p1, prev.threat_p2) : null },
      { label: "Skill act", p1: cur.skill_act_p1,  p2: cur.skill_act_p2,
                            prevDelta: prev ? dPrev(cur.skill_act_p1, cur.skill_act_p2, prev.skill_act_p1, prev.skill_act_p2) : null },
    ];
  }

  const rows = $derived<Row[]>(rowVsPrev(breakdown, prevBreakdown));

  const showVsPrev = $derived(prevBreakdown !== null);

  // Fill fraction on [-1, 1]. Positive = P1 fills downward (P1 sits at
  // bottom of board), negative = P2 fills upward.
  const fillFrac = $derived(
    breakdown === null
      ? 0
      : Math.max(-1, Math.min(1, breakdown.total / SATURATION))
  );

  const fillPct = $derived(Math.abs(fillFrac) * 50);

  function fmtSigned(n: number): string {
    if (n === 0) return "0";
    return n > 0 ? `+${n}` : `${n}`;
  }
</script>

<aside class="eval-panel" aria-label="Heuristic eval breakdown">
  <div class="bar-column">
    <div class="bar-track">
      <div class="bar-center-rule"></div>
      {#if breakdown !== null}
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
    {#if breakdown === null}
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
        {@const delta = row.p1 - row.p2}
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
            class:pos={delta > 0}
            class:neg={delta < 0}
          >{fmtSigned(delta)}</span>
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
