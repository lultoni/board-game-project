<script lang="ts">
  // Diagnostic hover card for a single board square. Shows the piece's
  // eval components (material, hp, armor, skills, mobility, exposure,
  // coverage) plus the raw intermediates that feed them (attackers,
  // adjacent guards, mobility raw count, empty-ring shielded/total,
  // skill availabilities). A side-context footer surfaces per-side
  // money (with cap) and tempo terms plus the game total so any square's
  // popup carries the full "why is the AI reading this position this way"
  // picture.

  import type { EvalBreakdownBySquare, SquareBreakdown } from "$lib/engine";
  import { SKILLS } from "$lib/engine";

  interface Props {
    /** The full by-square breakdown. Card slices out `data.squares[sq]`. */
    data: EvalBreakdownBySquare | null;
    /** Square under the cursor (0..63). `null` hides the card. */
    sq: number | null;
    /** Viewport-space cursor position; card sits near the cursor with
     *  viewport-edge clamping. */
    clientX: number;
    clientY: number;
  }

  const { data, sq, clientX, clientY }: Props = $props();

  const SKILL_AVAIL_MAX = 256;

  const entry = $derived<SquareBreakdown | null>(
    data !== null && sq !== null ? data.squares[sq] : null,
  );

  function fmtSigned(n: number): string {
    if (n === 0) return "0";
    return n > 0 ? `+${n}` : `${n}`;
  }

  function fileRankLabel(sq: number): string {
    const file = "abcdefgh"[sq & 7];
    const rank = ((sq >> 3) & 7) + 1;
    return `${file}${rank}`;
  }

  function pieceKindName(k: number): string {
    switch (k) {
      case 1: return "Guard";
      case 2: return "Champion";
      case 3: return "King";
      default: return "";
    }
  }

  function ownerLabel(isP1: boolean): string {
    return isP1 ? "P1" : "P2";
  }

  function skillName(id: number): string {
    if (id === 0) return "—";
    const s = SKILKlookup(id);
    return s ? s.key : `#${id}`;
  }

  // Alias for lookup (kept explicit so build tools don't tree-shake it).
  function SKILKlookup(id: number) {
    return SKILLS[id];
  }

  function availPct(fp: number): number {
    return Math.round((fp * 100) / SKILL_AVAIL_MAX);
  }

  function mobilityLabel(kind: number): string {
    switch (kind) {
      case 1: return "reachable (BFS-2)";
      case 2: return "reachable squares";
      case 3: return "adjacent escape squares";
      default: return "raw";
    }
  }

  // Position the card near the cursor but clamp so it stays fully on-screen.
  const OFFSET = 16;
  const CARD_W = 280;
  const CARD_H_EST = 460;

  const style = $derived.by(() => {
    if (typeof window === "undefined") return "";
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    let left = clientX + OFFSET;
    let top  = clientY + OFFSET;
    if (left + CARD_W > vw) left = clientX - OFFSET - CARD_W;
    if (top  + CARD_H_EST > vh) top = Math.max(4, vh - CARD_H_EST - 4);
    if (left < 4) left = 4;
    if (top  < 4) top  = 4;
    return `left: ${left}px; top: ${top}px;`;
  });

  interface Row {
    label: string;
    value: number;
    signed: boolean;
    isMagnitude?: boolean; // exposure is subtracted from the piece total; render with a "-" prefix visually
  }

  const componentRows = $derived<Row[]>(
    entry === null || !entry.occupied ? [] : [
      { label: "Material", value: entry.material,      signed: true },
      { label: "HP",       value: entry.hp_term,       signed: true },
      { label: "Armor",    value: entry.armor_term,    signed: true },
      { label: "Skills",   value: entry.skills_term,   signed: true },
      { label: "Reach",    value: entry.mobility_term, signed: true },
      { label: "Exposure", value: entry.exposure_term, signed: true, isMagnitude: true },
      { label: "Coverage", value: entry.coverage_term, signed: true },
    ]
  );

  const largestIdx = $derived.by<number>(() => {
    const rows = componentRows;
    if (rows.length === 0) return -1;
    let best = 0;
    let bestMag = Math.abs(rows[0].value);
    for (let i = 1; i < rows.length; i++) {
      const m = Math.abs(rows[i].value);
      if (m > bestMag) { bestMag = m; best = i; }
    }
    return bestMag > 0 ? best : -1;
  });

  const unshielded = $derived(
    entry === null ? 0 : Math.max(0, entry.n_attackers - entry.n_adj_guards),
  );
</script>

{#if data !== null && entry !== null}
  <div class="sq-card" style={style} role="tooltip">
    {#if entry.occupied}
      {@const pk = pieceKindName(entry.piece_kind)}
      {@const owner = ownerLabel(entry.is_p1)}
      <header class:p1={entry.is_p1} class:p2={!entry.is_p1}>
        <h3>{owner} {pk} @ {fileRankLabel(entry.sq)}</h3>
        <span class="hp-armor">HP {entry.hp} · AR {entry.armor}</span>
      </header>

      {#if entry.piece_kind === 2 || entry.piece_kind === 3}
        <ul class="skills">
          {#each [{ id: entry.skill1_id, fp: entry.skill1_avail_fp }, { id: entry.skill2_id, fp: entry.skill2_avail_fp }] as slot}
            {@const pct = availPct(slot.fp)}
            {@const dim = pct === 0}
            {@const name = skillName(slot.id)}
            {@const cost = SKILKlookup(slot.id)?.cost ?? 0}
            <li class:dim>
              <span class="skill-name">{name}</span>
              <span class="skill-cost">{cost}g</span>
              <span class="skill-avail">{pct}%</span>
            </li>
          {/each}
        </ul>
      {/if}

      <table class="components">
        <thead>
          <tr><th>Component</th><th class="num">Value</th></tr>
        </thead>
        <tbody>
          {#each componentRows as row, i}
            <tr class:largest={i === largestIdx}>
              <td>{row.label}</td>
              <td class="num" class:pos={row.value > 0} class:neg={row.value < 0}>
                {#if row.isMagnitude && row.value > 0}
                  −{row.value}
                {:else}
                  {fmtSigned(row.value)}
                {/if}
              </td>
            </tr>
          {/each}
          <tr class="total-row">
            <td>Piece total</td>
            <td class="num" class:pos={entry.piece_total > 0} class:neg={entry.piece_total < 0}>{fmtSigned(entry.piece_total)}</td>
          </tr>
        </tbody>
      </table>

      <dl class="intermediates">
        <dt>Reach raw</dt>
        <dd>{entry.mobility_raw} <span class="mut">({mobilityLabel(entry.piece_kind)})</span></dd>

        <dt>Exposure</dt>
        <dd>
          {entry.n_attackers} attacker{entry.n_attackers === 1 ? "" : "s"},
          {entry.n_adj_guards} adj guard{entry.n_adj_guards === 1 ? "" : "s"}
          → <b>{unshielded}</b> unshielded
        </dd>

        {#if entry.piece_kind === 3}
          <dt>Coverage</dt>
          <dd>
            {#if entry.empty_ring_total === 0}
              <span class="mut">fully surrounded</span>
            {:else}
              {entry.empty_ring_shielded}/{entry.empty_ring_total} shielded
            {/if}
          </dd>
        {/if}
      </dl>
    {:else}
      <header class="empty-header">
        <h3>Empty @ {fileRankLabel(entry.sq)}</h3>
      </header>
    {/if}

    <footer class="side-ctx">
      <div class="side-row">
        <span class="side-tag p1">P1</span>
        <span>money {data.p1_money}/{data.p1_money_cap} → {fmtSigned(data.p1_money_term)}</span>
        <span>tempo {fmtSigned(data.p1_tempo_term)}</span>
      </div>
      <div class="side-row">
        <span class="side-tag p2">P2</span>
        <span>money {data.p2_money}/{data.p2_money_cap} → {fmtSigned(data.p2_money_term)}</span>
        <span>tempo {fmtSigned(data.p2_tempo_term)}</span>
      </div>
      <div class="total-row-footer">
        Total <b class:pos={data.total > 0} class:neg={data.total < 0}>{fmtSigned(data.total)}</b>
        {#if data.terminal}<span class="mut">(terminal)</span>{/if}
      </div>
    </footer>
  </div>
{/if}

<style>
  .sq-card {
    position: fixed;
    z-index: 4000;
    width: 280px;
    padding: 0.5em 0.7em 0.55em;
    border: 1.5px solid var(--paper-line-strong, #8a7a4e);
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
    box-shadow: 0 6px 14px rgba(0, 0, 0, 0.14);
    font-size: 0.82rem;
    line-height: 1.35;
    color: var(--paper-ink, #3a2f1f);
    font-family: inherit;
    pointer-events: none;
    animation: card-fade 120ms ease-out;
  }
  @keyframes card-fade {
    from { opacity: 0; transform: translateY(-3px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5em;
    padding-bottom: 0.25em;
    margin-bottom: 0.3em;
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.2));
  }
  header h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
  }
  header.p1 h3 { color: #3a5a7a; }
  header.p2 h3 { color: #a03a2a; }
  .empty-header h3 {
    color: var(--paper-ink-soft, #6a6055);
    font-style: italic;
    font-weight: 500;
  }
  .hp-armor {
    font-size: 0.72rem;
    color: var(--paper-ink-soft, #6a6055);
    font-variant-numeric: tabular-nums;
  }

  .skills {
    list-style: none;
    padding: 0;
    margin: 0 0 0.35em;
    display: flex;
    flex-direction: column;
    gap: 0.15em;
    font-size: 0.78rem;
  }
  .skills li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 2.6em 3.2em;
    gap: 0.4em;
    padding: 0.05em 0.4em;
    border: 1px solid var(--paper-line, rgba(58,47,31,0.25));
    border-radius: 3px;
    background: var(--paper-square-light, #ece2c8);
    align-items: baseline;
  }
  .skills li.dim {
    opacity: 0.45;
  }
  .skill-name { text-transform: capitalize; font-weight: 600; }
  .skill-cost { text-align: right; color: var(--paper-ink-soft, #6a6055); }
  .skill-avail { text-align: right; font-variant-numeric: tabular-nums; font-weight: 600; }

  .components {
    width: 100%;
    border-collapse: collapse;
    margin: 0.25em 0 0.35em;
    font-size: 0.8rem;
  }
  .components th,
  .components td {
    padding: 0.1em 0.15em;
    text-align: left;
  }
  .components thead th {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--paper-ink-soft, #6a6055);
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.2));
    font-weight: 500;
  }
  .components tbody tr.largest td {
    background: rgba(199, 155, 58, 0.14);
    font-weight: 600;
  }
  .components .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .components .pos { color: #3a7a3a; }
  .components .neg { color: #a03030; }
  .components tr.total-row td {
    border-top: 1px solid var(--paper-line, rgba(58,47,31,0.25));
    font-weight: 700;
    padding-top: 0.2em;
  }

  .intermediates {
    display: grid;
    grid-template-columns: 6em 1fr;
    gap: 0.15em 0.4em;
    margin: 0.3em 0 0;
    padding-top: 0.25em;
    border-top: 1px dashed var(--paper-line, rgba(58,47,31,0.25));
    font-size: 0.76rem;
  }
  .intermediates dt {
    color: var(--paper-ink-soft, #6a6055);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    font-size: 0.7rem;
    align-self: baseline;
  }
  .intermediates dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
  .mut { color: var(--paper-ink-soft, #6a6055); font-style: italic; }

  .side-ctx {
    margin-top: 0.5em;
    padding-top: 0.35em;
    border-top: 1px solid var(--paper-line-strong, #8a7a4e);
    font-size: 0.74rem;
  }
  .side-row {
    display: flex;
    gap: 0.5em;
    align-items: baseline;
    font-variant-numeric: tabular-nums;
  }
  .side-tag {
    font-weight: 700;
    min-width: 1.5em;
  }
  .side-tag.p1 { color: #3a5a7a; }
  .side-tag.p2 { color: #a03a2a; }
  .total-row-footer {
    margin-top: 0.25em;
    padding-top: 0.2em;
    border-top: 1px dashed var(--paper-line, rgba(58,47,31,0.25));
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .total-row-footer .pos { color: #3a7a3a; }
  .total-row-footer .neg { color: #a03030; }
</style>
