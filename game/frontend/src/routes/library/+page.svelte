<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import { getTelemetryStore } from "$lib/storage";
  import { sfx } from "$lib/audio/sfx";
  import { setPendingMatchLog } from "$lib/storage/library-handoff";
  import type { MatchMeta } from "$lib/storage/types";
  import type { MatchMode } from "$lib/state/match-store.svelte";

  type FilterMode = "all" | MatchMode;
  type FilterResult = "all" | "p1win" | "p2win" | "draw" | "in-progress" | "abandoned";

  let loading = $state(true);
  let rows = $state<MatchMeta[]>([]);
  let selected = $state<Set<string>>(new Set());
  let filterMode = $state<FilterMode>("all");
  let filterResult = $state<FilterResult>("all");
  let confirmDelete = $state<string | null>(null);
  let busy = $state(false);
  /** Transient banner shown when an export skipped one or more matches due
   *  to a missing/corrupt log. Cleared on the next export attempt. */
  let exportSkipNotice = $state<string | null>(null);

  const filtered = $derived.by<MatchMeta[]>(() => {
    return rows.filter((m) => {
      if (filterMode !== "all" && m.mode !== filterMode) return false;
      if (filterResult === "all") return true;
      if (filterResult === "in-progress") {
        return m.status === "in-progress" || m.status === "mid-match-network-lost";
      }
      if (filterResult === "abandoned") return m.status === "abandoned";
      if (m.status !== "ended") return false;
      if (filterResult === "p1win") return m.resultByte === 0;
      if (filterResult === "p2win") return m.resultByte === 1;
      if (filterResult === "draw")  return m.resultByte === 2;
      return true;
    });
  });

  async function refresh(): Promise<void> {
    rows = await getTelemetryStore().listMatches();
  }

  onMount(async () => {
    try {
      await refresh();
    } finally {
      loading = false;
    }
  });

  function formatDate(ms: number): string {
    return new Date(ms).toLocaleString();
  }

  function modeLabel(mode: MatchMode): string {
    switch (mode) {
      case "hvh": return t("library.modeHvh");
      case "hvai": return t("library.modeHvai");
      case "aivai": return t("library.modeAivai");
      case "multiplayer": return t("library.modeMultiplayer");
      default: return mode;
    }
  }

  /** Returns [label, css-class] for the result chip. */
  function resultChip(m: MatchMeta): { label: string; cls: string } {
    if (m.status === "in-progress") return { label: t("library.resultInProgress"), cls: "pending" };
    if (m.status === "abandoned")   return { label: t("library.resultAbandoned"),  cls: "abandoned" };
    if (m.status === "mid-match-network-lost") return { label: t("library.resultNetworkLost"), cls: "abandoned" };
    // ended:
    if (m.resultByte === 0) return { label: t("library.resultP1Win"), cls: "p1win" };
    if (m.resultByte === 1) return { label: t("library.resultP2Win"), cls: "p2win" };
    if (m.resultByte === 2) return { label: t("library.resultDraw"),  cls: "draw" };
    return { label: "?", cls: "pending" };
  }

  function countLabel(n: number): string {
    return t(n === 1 ? "library.count" : "library.countPlural", { n });
  }

  /** True when the row has a stored MatchLog the inspector/export can use.
   *  Both finalised and partial-log-on-abandon paths set `endedAtUnixMs`. */
  function hasLog(m: MatchMeta): boolean {
    return m.endedAtUnixMs !== undefined;
  }

  function download(filename: string, json: string): void {
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function bundleFilename(): string {
    const d = new Date();
    const pad = (n: number) => String(n).padStart(2, "0");
    const stamp = `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}`;
    return `boardgame-bundle-${stamp}.json`;
  }

  function toggleSelect(id: string): void {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id); else next.add(id);
    selected = next;
  }

  async function openInInspector(matchId: string): Promise<void> {
    sfx.play("click");
    busy = true;
    try {
      const m = await getTelemetryStore().getMatch(matchId);
      if (!m) return;
      setPendingMatchLog(m.matchLogJson);
      await goto("../inspector/");
    } finally {
      busy = false;
    }
  }

  async function openInReplay(matchId: string): Promise<void> {
    sfx.play("click");
    busy = true;
    try {
      const m = await getTelemetryStore().getMatch(matchId);
      if (!m) return;
      setPendingMatchLog(m.matchLogJson);
      await goto("../replay/");
    } finally {
      busy = false;
    }
  }

  async function exportSingle(matchId: string): Promise<void> {
    sfx.play("click");
    busy = true;
    exportSkipNotice = null;
    try {
      const { bundle, skipped } = await getTelemetryStore().bundleMatches([matchId]);
      download(`boardgame-match-${matchId}.json`, bundle);
      if (skipped.length > 0) {
        exportSkipNotice = `Could not export this match - its stored log is missing or corrupt.`;
      }
    } finally {
      busy = false;
    }
  }

  async function sendBundle(): Promise<void> {
    if (selected.size === 0) return;
    busy = true;
    exportSkipNotice = null;
    try {
      const ids = [...selected];
      const { bundle, skipped } = await getTelemetryStore().bundleMatches(ids);
      download(bundleFilename(), bundle);
      if (skipped.length > 0) {
        exportSkipNotice = `Exported ${ids.length - skipped.length} of ${ids.length} matches - ${skipped.length} skipped due to missing or corrupt logs.`;
      }
      selected = new Set();
    } finally {
      busy = false;
    }
  }

  async function deleteRow(matchId: string): Promise<void> {
    sfx.play("click");
    busy = true;
    try {
      await getTelemetryStore().deleteMatch(matchId);
      rows = rows.filter((r) => r.matchId !== matchId);
      if (selected.has(matchId)) {
        const next = new Set(selected);
        next.delete(matchId);
        selected = next;
      }
      confirmDelete = null;
    } finally {
      busy = false;
    }
  }
</script>

<main>
  <header>
    <p><a href="../" onclick={() => sfx.play("click")}>{t("library.back")}</a></p>
    <h1>{t("library.title")}</h1>
    <small>{countLabel(filtered.length)}</small>
  </header>

  {#if exportSkipNotice}
    <p class="export-skip-notice" role="status">
      {exportSkipNotice}
      <button type="button" onclick={() => (exportSkipNotice = null)} aria-label="dismiss">x</button>
    </p>
  {/if}

  <section class="filters">
    <label>
      <span>{t("library.filterMode")}</span>
      <select bind:value={filterMode} onchange={() => sfx.play("tick")}>
        <option value="all">{t("library.modeAll")}</option>
        <option value="hvh">{t("library.modeHvh")}</option>
        <option value="hvai">{t("library.modeHvai")}</option>
        <option value="aivai">{t("library.modeAivai")}</option>
        <option value="multiplayer">{t("library.modeMultiplayer")}</option>
      </select>
    </label>
    <label>
      <span>{t("library.filterResult")}</span>
      <select bind:value={filterResult} onchange={() => sfx.play("tick")}>
        <option value="all">{t("library.resultAll")}</option>
        <option value="p1win">{t("library.resultP1Win")}</option>
        <option value="p2win">{t("library.resultP2Win")}</option>
        <option value="draw">{t("library.resultDraw")}</option>
        <option value="in-progress">{t("library.resultInProgress")}</option>
        <option value="abandoned">{t("library.resultAbandoned")}</option>
      </select>
    </label>
    <button class="bundle" type="button" disabled={busy || selected.size === 0} onclick={sendBundle}>
      {selected.size > 0
        ? t("library.sendBundle", { n: selected.size })
        : t("library.sendBundleNone")}
    </button>
  </section>

  {#if loading}
    <p class="hint">{t("library.loading")}</p>
  {:else if filtered.length === 0}
    <p class="hint empty">{t("library.empty")}</p>
  {:else}
    <ul class="rows">
      {#each filtered as m (m.matchId)}
        {@const chip = resultChip(m)}
        <li class="row">
          <label class="sel">
            <input
              type="checkbox"
              checked={selected.has(m.matchId)}
              onchange={() => toggleSelect(m.matchId)}
            />
          </label>
          <div class="date">{formatDate(m.startedAtUnixMs)}</div>
          <div class="mode">{modeLabel(m.mode)}</div>
          <div class={`chip ${chip.cls}`}>{chip.label}</div>
          <div class="plies">
            {m.status === "ended" && m.totalPlies !== undefined
              ? t("library.plies", { n: m.totalPlies })
              : t("library.ongoing")}
          </div>
          <div class="actions">
            <button
              type="button"
              disabled={busy || !hasLog(m)}
              onclick={() => openInReplay(m.matchId)}
            >{t("library.actionReplay")}</button>
            <button
              type="button"
              disabled={busy || !hasLog(m)}
              onclick={() => openInInspector(m.matchId)}
            >{t("library.actionInspector")}</button>
            <button
              type="button"
              disabled={busy || !hasLog(m)}
              onclick={() => exportSingle(m.matchId)}
            >{t("library.actionExport")}</button>
            {#if confirmDelete === m.matchId}
              <button
                class="danger"
                type="button"
                disabled={busy}
                onclick={() => deleteRow(m.matchId)}
              >{t("library.actionConfirmDelete")}</button>
              <button
                type="button"
                disabled={busy}
                onclick={() => (confirmDelete = null)}
              >{t("library.actionCancelDelete")}</button>
            {:else}
              <button
                type="button"
                disabled={busy}
                onclick={() => (confirmDelete = m.matchId)}
              >{t("library.actionDelete")}</button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  main {
    max-width: 960px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  .export-skip-notice {
    margin: 0 0 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--paper-warn-bg, #fff4d6);
    border: 1px solid var(--paper-warn-border, #d8b65a);
    border-radius: 4px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
  }
  .export-skip-notice button {
    background: transparent;
    border: 0;
    cursor: pointer;
    font-size: 1.1rem;
    line-height: 1;
    padding: 0 0.25rem;
  }
  header {
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
  }
  header h1 {
    margin: 0.2rem 0 0;
    font-size: 1.8rem;
  }
  header small {
    color: var(--paper-ink-soft);
  }
  .filters {
    display: flex;
    gap: 0.8rem;
    align-items: end;
    flex-wrap: wrap;
    margin-bottom: 1rem;
  }
  .filters label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.9rem;
    color: var(--paper-ink-soft);
  }
  .filters select {
    padding: 0.35em 0.5em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 5px;
    color: inherit;
  }
  .bundle {
    margin-left: auto;
    padding: 0.5em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 5px;
    cursor: pointer;
  }
  .bundle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .hint {
    color: var(--paper-ink-soft);
    padding: 1rem 0;
  }
  .empty {
    border: 1.5px dashed var(--paper-line-strong);
    padding: 1rem;
    border-radius: 6px;
    text-align: center;
  }
  .rows {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .row {
    display: grid;
    grid-template-columns: auto 12em 5em 8em 6em 1fr;
    gap: 0.6rem;
    align-items: center;
    padding: 0.5rem 0.7rem;
    border: 1.5px solid var(--paper-line);
    border-radius: 6px;
    background: var(--paper-bg);
  }
  .sel {
    display: flex;
    align-items: center;
  }
  .date {
    font-variant-numeric: tabular-nums;
    color: var(--paper-ink-soft);
    font-size: 0.9rem;
  }
  .mode {
    font-weight: 600;
  }
  .chip {
    display: inline-block;
    padding: 0.1em 0.55em;
    border-radius: 999px;
    font-size: 0.85rem;
    border: 1.5px solid currentColor;
    text-align: center;
  }
  .chip.p1win   { color: #2a6b3a; }
  .chip.p2win   { color: #2a4d7a; }
  .chip.draw    { color: #6b6b2a; }
  .chip.pending { color: var(--paper-ink-soft); }
  .chip.abandoned { color: #a94b3b; }
  .plies {
    color: var(--paper-ink-soft);
    font-size: 0.9rem;
  }
  .actions {
    display: flex;
    gap: 0.35rem;
    justify-content: flex-end;
    flex-wrap: wrap;
  }
  .actions button {
    padding: 0.3em 0.6em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 5px;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .actions button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .actions button.danger {
    color: #a94b3b;
    border-color: #a94b3b;
  }
  @media (max-width: 760px) {
    .row {
      grid-template-columns: auto 1fr;
    }
    .row .date,
    .row .mode,
    .row .chip,
    .row .plies {
      grid-column: 2;
    }
    .row .actions {
      grid-column: 1 / span 2;
      justify-content: flex-start;
    }
  }
</style>
