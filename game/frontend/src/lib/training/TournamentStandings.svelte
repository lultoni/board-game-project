<script lang="ts">
  // Panel 2 — Tournament Standings.
  //
  // Reads the population + active match from the shared status store
  // (populated globally by the route shell at 1 Hz). Renders a sortable
  // table of every rater the orchestrator knows about, highlighting the
  // two rows that are currently playing. Footer shows generation/round/
  // ETA mirrored from the same snapshot.

  import { getContext } from "svelte";
  import type { Readable } from "svelte/store";
  import type { PollState } from "$lib/training/polling";
  import type { StatusSnapshot, PopulationMember } from "$lib/training/types";

  type StatusStore = Readable<PollState<StatusSnapshot>>;
  const getStore = getContext<() => StatusStore | null>("training:getStatusStore");

  let snap: StatusSnapshot | null = $state(null);

  $effect(() => {
    const store = getStore();
    if (!store) return;
    return store.subscribe((v) => (snap = v.data));
  });

  const population = $derived<PopulationMember[]>(snap?.population ?? []);
  const active = $derived(snap?.active_match ?? null);

  const sorted = $derived.by(() => {
    const copy = [...population];
    // Stable order: alive first, then by win-rate descending, then by lineage.
    copy.sort((a, b) => {
      if (a.alive !== b.alive) return a.alive ? -1 : 1;
      const wrA = winRate(a);
      const wrB = winRate(b);
      if (wrA !== wrB) return wrB - wrA;
      return a.lineage - b.lineage;
    });
    return copy;
  });

  function winRate(p: PopulationMember): number {
    const games = p.wins + p.losses + p.draws;
    if (games === 0) return 0;
    // Wins + half of draws, common chess convention.
    return (p.wins + 0.5 * p.draws) / games;
  }

  function activeRow(id: string): "challenger" | "defender" | null {
    if (!active) return null;
    if (active.challenger === id) return "challenger";
    if (active.defender === id) return "defender";
    return null;
  }

  function fmtEta(s: number | null | undefined): string {
    if (s === null || s === undefined || !Number.isFinite(s)) return "—";
    const total = Math.max(0, Math.round(s));
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const sec = total % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${sec}s`;
    return `${sec}s`;
  }
</script>

<div class="standings">
  {#if !snap}
    <div class="empty">Waiting for the trainer to publish a status snapshot.</div>
  {:else if population.length === 0}
    <div class="empty">No raters in the population yet. The first generation seeds the field.</div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Rater</th>
          <th>Parent</th>
          <th>Lin</th>
          <th>Gen</th>
          <th class="num">W</th>
          <th class="num">L</th>
          <th class="num">D</th>
          <th class="num">WR%</th>
          <th>State</th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as p (p.rater_id)}
          {@const role = activeRow(p.rater_id)}
          <tr class:active-c={role === "challenger"} class:active-d={role === "defender"}>
            <td><code>{p.rater_id}</code></td>
            <td>{p.parent_id ?? "—"}</td>
            <td class="num">{p.lineage}</td>
            <td class="num">{p.generation}</td>
            <td class="num">{p.wins}</td>
            <td class="num">{p.losses}</td>
            <td class="num">{p.draws}</td>
            <td class="num">{(winRate(p) * 100).toFixed(1)}</td>
            <td>{p.alive ? "alive" : "eliminated"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <footer>
      <span>Generation <b>{snap.generation}</b></span>
      <span>Round <b>{snap.round}</b></span>
      <span>ETA <b>{fmtEta(snap.eta_seconds)}</b></span>
      {#if active}
        <span class="activeMeta">
          Active: <code>{active.challenger}</code> vs <code>{active.defender}</code>
          · game {active.game_index + 1}/{active.games_total}
          · ply {active.ply}
          · {active.bracket}
        </span>
      {/if}
    </footer>
  {/if}
</div>

<style>
  .standings {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .empty {
    color: var(--paper-ink-soft);
    font-style: italic;
    padding: 1.5rem 0;
    text-align: center;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.95em;
  }
  th, td {
    border-bottom: 1px solid var(--paper-line);
    padding: 0.35em 0.6em;
    text-align: left;
  }
  th {
    color: var(--paper-ink-soft);
    font-weight: 600;
    font-size: 0.88em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  td code, .activeMeta code {
    font-family: inherit;
    background: var(--paper-bg);
    padding: 0.05em 0.35em;
    border-radius: 3px;
  }
  td.num, th.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  tr.active-c {
    background: rgba(43, 74, 138, 0.08);
  }
  tr.active-d {
    background: rgba(161, 58, 42, 0.08);
  }
  footer {
    display: flex;
    flex-wrap: wrap;
    gap: 1.2em;
    color: var(--paper-ink-soft);
    font-size: 0.92em;
    border-top: 1px solid var(--paper-line);
    padding-top: 0.5em;
  }
  footer b {
    color: var(--paper-ink);
    font-variant-numeric: tabular-nums;
  }
  .activeMeta {
    flex-basis: 100%;
  }
</style>
