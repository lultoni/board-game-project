<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { sfx } from "$lib/audio/sfx";
  import { createPollingStore } from "$lib/training/polling";
  import type { StatusSnapshot, TrainingPhase, BackendInfo } from "$lib/training/types";
  import LiveMatchView from "$lib/training/LiveMatchView.svelte";
  import TournamentStandings from "$lib/training/TournamentStandings.svelte";
  import LineageTree from "$lib/training/LineageTree.svelte";
  import NetworkInspector from "$lib/training/NetworkInspector.svelte";
  import GauntletMatrix from "$lib/training/GauntletMatrix.svelte";

  type Tab = "live" | "standings" | "lineage" | "matrix";
  type Preset = "smoke" | "medium" | "long";

  let runDir = $state<string>("");
  let runDirInput = $state<string>("");
  let activeTab = $state<Tab>("live");
  let preset = $state<Preset>("smoke");
  let backends = $state<BackendInfo[]>([]);
  let backend = $state<string>("cpu");
  let starting = $state<boolean>(false);
  let stopping = $state<boolean>(false);
  let startError = $state<string | null>(null);
  let runRequested = $state<boolean>(false);

  let selectedRaterId = $state<string | null>(null);

  type StatusStore = ReturnType<typeof createPollingStore<StatusSnapshot>>;
  type StatusValue = { data: StatusSnapshot | null; error: string | null; lastUpdated: number | null };
  let statusStore: StatusStore | null = $state(null);
  let statusValue: StatusValue | null = $state(null);

  $effect(() => {
    if (!statusStore) {
      statusValue = null;
      return;
    }
    const unsub = statusStore.subscribe((v) => (statusValue = v));
    return unsub;
  });

  setContext("training:getRunDir", () => runDir);
  setContext("training:getSelectedRaterId", () => selectedRaterId);
  setContext("training:setSelectedRaterId", (id: string | null) => {
    selectedRaterId = id;
  });
  setContext("training:getStatusStore", () => statusStore);

  onMount(async () => {
    try {
      const def = await invoke<string>("default_run_dir");
      runDir = def;
      runDirInput = def;
      statusStore = createPollingStore<StatusSnapshot>({
        invokeCmd: "read_training_status",
        args: { runDir: def },
        intervalMs: 1000,
      });
      const list = await invoke<BackendInfo[]>("list_backends");
      backends = list;
      const persisted = typeof localStorage !== "undefined"
        ? localStorage.getItem("training:backend")
        : null;
      const fallback = list.find((b) => b.is_default)?.id ?? list[0]?.id ?? "cpu";
      backend = persisted && list.some((b) => b.id === persisted) ? persisted : fallback;
    } catch (e: unknown) {
      startError = e instanceof Error ? e.message : String(e);
    }
  });

  function applyRunDir(): void {
    if (!runDirInput.trim()) return;
    sfx.play("click");
    runDir = runDirInput.trim();
    statusStore = createPollingStore<StatusSnapshot>({
      invokeCmd: "read_training_status",
      args: { runDir },
      intervalMs: 1000,
    });
  }

  async function start(): Promise<void> {
    if (!runDir) return;
    sfx.play("click");
    starting = true;
    startError = null;
    runRequested = true;
    try {
      if (typeof localStorage !== "undefined") {
        localStorage.setItem("training:backend", backend);
      }
      await invoke("start_training_run", { runDir, preset, backend });
    } catch (e: unknown) {
      startError = e instanceof Error ? e.message : String(e);
      runRequested = false;
    } finally {
      starting = false;
    }
  }

  async function stop(): Promise<void> {
    sfx.play("click");
    stopping = true;
    runRequested = false;
    try {
      await invoke("stop_training_run");
    } catch (e: unknown) {
      startError = e instanceof Error ? e.message : String(e);
    } finally {
      stopping = false;
    }
  }

  const STALE_MS = 30_000;
  const statusFresh = $derived.by(() => {
    const ts = statusValue?.data?.written_at_ms;
    if (!ts) return false;
    return Date.now() - ts < STALE_MS;
  });

  const phase = $derived<TrainingPhase>(
    statusFresh ? (statusValue?.data?.phase ?? "idle") : "idle",
  );
  const isRunning = $derived(runRequested || (statusFresh && phase !== "idle"));

  const generation = $derived(statusValue?.data?.generation ?? null);
  const round = $derived(statusValue?.data?.round ?? null);
  const etaSeconds = $derived(statusValue?.data?.eta_seconds ?? null);
  const populationCount = $derived(statusValue?.data?.population?.length ?? 0);
  const activeMatch = $derived(statusValue?.data?.active_match ?? null);

  $effect(() => {
    if (runRequested && statusFresh && phase !== "idle") {
      runRequested = false;
    }
  });

  const activityLine = $derived.by(() => {
    if (!isRunning) return null;
    if (!statusFresh && runRequested) return "Starting up…";
    if (phase === "training") {
      const gen = generation ?? "?";
      const pop = populationCount > 0
        ? ` — ${populationCount} candidates ready`
        : " — no candidates yet";
      return `Training generation ${gen}${pop}`;
    }
    if (phase === "gauntlet") {
      const gen = generation ?? "?";
      const r = round ?? "?";
      if (activeMatch) {
        const a = activeMatch;
        return `Gauntlet gen ${gen}, round ${r} — ${a.challenger} vs ${a.defender} (${a.bracket}, game ${a.game_index + 1}/${a.games_total})`;
      }
      return `Gauntlet gen ${gen}, round ${r} — preparing next match`;
    }
    if (phase === "bookkeeping") {
      return `Bookkeeping generation ${generation ?? "?"} — saving accepted raters`;
    }
    return null;
  });

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

  const PHASE_LABELS: Record<TrainingPhase, string> = {
    idle: "Idle",
    training: "Training",
    gauntlet: "Gauntlet",
    bookkeeping: "Bookkeeping",
  };
</script>

<main>
  <header>
    <p class="back"><a href="../" onclick={() => sfx.play("click")}>← Back</a></p>
    <h1>Training Observatory</h1>
  </header>

  <div class="controls-bar">
    <label class="runDir-row">
      <span class="lbl">Run directory</span>
      <input type="text" bind:value={runDirInput} spellcheck="false" />
      <button onclick={applyRunDir}>Apply</button>
    </label>

    <div class="controls-row">
      <div class="presets">
        <span class="lbl">Preset</span>
        <div class="presetButtons" role="radiogroup" aria-label="Run preset">
          <button
            role="radio"
            aria-checked={preset === "smoke"}
            class:active={preset === "smoke"}
            disabled={isRunning}
            onclick={() => { sfx.play("click"); preset = "smoke"; }}
            title="2 gen × 4 lineage, depth-2 corpus, seconds"
          >Smoke</button>
          <button
            role="radio"
            aria-checked={preset === "medium"}
            class:active={preset === "medium"}
            disabled={isRunning}
            onclick={() => { sfx.play("click"); preset = "medium"; }}
            title="5 gen × 4 lineage, depth-4 corpus"
          >Medium</button>
          <button
            role="radio"
            aria-checked={preset === "long"}
            class:active={preset === "long"}
            disabled={isRunning}
            onclick={() => { sfx.play("click"); preset = "long"; }}
            title="10 gen × 8 lineage, depth-6 corpus (GPU)"
          >Long</button>
        </div>
      </div>

      <div class="backend">
        <span class="lbl">Backend</span>
        <select
          bind:value={backend}
          disabled={isRunning || backends.length === 0}
          title="GPU = training on GPU; inference (search-time evaluator) is always CPU."
          onchange={() => sfx.play("tick")}
        >
          {#each backends as b}
            <option value={b.id}>{b.label}{b.is_default ? " (default)" : ""}</option>
          {/each}
        </select>
      </div>

      <div class="run-controls">
        <button
          class="btn-start"
          onclick={start}
          disabled={starting || isRunning || !runDir}
        >
          {starting ? "Starting…" : isRunning ? "Running…" : "▶ Start"}
        </button>
        <button
          class="btn-stop"
          onclick={stop}
          disabled={stopping || !isRunning}
        >
          {stopping ? "Stopping…" : "■ Stop"}
        </button>
      </div>
    </div>
  </div>

  <div class="status-strip" data-phase={phase}>
    <span class="phase-dot" aria-hidden="true">●</span>
    <span class="phase-label">{PHASE_LABELS[phase]}</span>
    {#if isRunning}
      <span class="divider" aria-hidden="true">·</span>
      <span class="stat-group">
        <span class="lbl">Gen</span>
        <span class="val">{generation ?? "—"}</span>
        <span class="lbl">Round</span>
        <span class="val">{round ?? "—"}</span>
        <span class="lbl">ETA</span>
        <span class="val">{fmtEta(etaSeconds)}</span>
      </span>
    {/if}
    {#if activityLine}
      <span class="divider" aria-hidden="true">·</span>
      <span class="activity">{activityLine}</span>
    {/if}
  </div>

  {#if startError}
    <p class="error">{startError}</p>
  {/if}

  <section class="workspace">
    <div class="leftPane">
      <div class="tabs" role="tablist">
        <button role="tab" aria-selected={activeTab === "live"}
          class:active={activeTab === "live"}
          onclick={() => { sfx.play("click"); activeTab = "live"; }}>Live Match</button>
        <button role="tab" aria-selected={activeTab === "standings"}
          class:active={activeTab === "standings"}
          onclick={() => { sfx.play("click"); activeTab = "standings"; }}>Standings</button>
        <button role="tab" aria-selected={activeTab === "lineage"}
          class:active={activeTab === "lineage"}
          onclick={() => { sfx.play("click"); activeTab = "lineage"; }}>Lineage</button>
        <button role="tab" aria-selected={activeTab === "matrix"}
          class:active={activeTab === "matrix"}
          onclick={() => { sfx.play("click"); activeTab = "matrix"; }}>Matrix</button>
      </div>

      <div class="panel" role="tabpanel">
        {#if activeTab === "live"}
          <LiveMatchView />
        {:else if activeTab === "standings"}
          <TournamentStandings />
        {:else if activeTab === "lineage"}
          <LineageTree />
        {:else if activeTab === "matrix"}
          <GauntletMatrix />
        {/if}
      </div>
    </div>

    <aside class="rightPane">
      <h2>Network Inspector</h2>
      <div class="panel">
        <NetworkInspector />
      </div>
    </aside>
  </section>
</main>

<style>
  main {
    max-width: 1400px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    margin-bottom: 1rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.4rem;
  }
  .back a {
    color: var(--paper-ink-soft);
    text-decoration: none;
  }
  h1 {
    font-size: 2rem;
    margin: 0.2em 0 0;
  }

  /* ── Controls bar ───────────────────────────────────── */
  .controls-bar {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.7em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
    margin-bottom: 0.6rem;
  }
  .runDir-row {
    display: flex;
    align-items: center;
    gap: 0.5em;
  }
  .runDir-row input {
    flex: 1;
    font: inherit;
    padding: 0.3em 0.5em;
    border: 1px solid var(--paper-line);
    border-radius: 4px;
    background: white;
    min-width: 0;
  }
  .controls-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .presets {
    display: flex;
    align-items: center;
    gap: 0.4em;
  }
  .presetButtons {
    display: flex;
    gap: 0.2em;
  }
  .presetButtons button {
    padding: 0.3em 0.7em;
    border: 1.5px solid var(--paper-line);
    border-radius: 4px;
    background: var(--paper-bg);
    font: inherit;
    cursor: pointer;
  }
  .presetButtons button.active {
    background: white;
    border-color: var(--paper-line-strong);
    font-weight: 600;
  }
  .presetButtons button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .backend {
    display: flex;
    align-items: center;
    gap: 0.4em;
  }
  .backend select {
    font: inherit;
    padding: 0.3em 0.5em;
    border: 1.5px solid var(--paper-line);
    border-radius: 4px;
    background: var(--paper-bg);
  }
  .backend select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .run-controls {
    display: flex;
    gap: 0.4em;
    margin-left: auto;
  }
  .run-controls button {
    padding: 0.4em 1em;
    border-radius: 5px;
    font: inherit;
    cursor: pointer;
    border: 1.5px solid var(--paper-line-strong);
    transition: box-shadow 80ms ease, transform 80ms ease;
  }
  .run-controls button:hover:not(:disabled) {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    transform: translateY(-1px);
  }
  .run-controls button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .btn-start {
    background: var(--paper-ink, #1a1a1a);
    color: var(--paper-bg, #fff);
    border-color: var(--paper-ink, #1a1a1a);
    font-weight: 600;
  }
  .btn-stop {
    background: var(--paper-bg);
    color: inherit;
  }
  .runDir-row button {
    padding: 0.3em 0.8em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    background: var(--paper-bg);
    font: inherit;
    cursor: pointer;
  }
  .runDir-row button:hover {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
  }

  /* ── Status strip ───────────────────────────────────── */
  .status-strip {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5em 0.9em;
    border-radius: 6px;
    border: 1.5px solid var(--paper-line);
    background: var(--paper-bg);
    margin-bottom: 0.8rem;
    font-size: 0.95em;
    flex-wrap: wrap;
  }
  .phase-dot {
    font-size: 0.7em;
    line-height: 1;
  }
  .phase-label {
    font-weight: 700;
    font-size: 0.9em;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .divider {
    color: var(--paper-line-strong);
  }
  .stat-group {
    display: flex;
    gap: 0.4em;
    align-items: baseline;
  }
  .stat-group .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.88em;
  }
  .stat-group .val {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
  .activity {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }

  /* Phase colours applied to dot + label */
  .status-strip[data-phase="idle"] .phase-dot,
  .status-strip[data-phase="idle"] .phase-label { color: var(--paper-ink-soft); }

  .status-strip[data-phase="training"] .phase-dot,
  .status-strip[data-phase="training"] .phase-label { color: #b07a10; }
  .status-strip[data-phase="training"] { border-color: #d4b94a; background: #fffbf0; }

  .status-strip[data-phase="gauntlet"] .phase-dot,
  .status-strip[data-phase="gauntlet"] .phase-label { color: #2a7a52; }
  .status-strip[data-phase="gauntlet"] { border-color: #5bb088; background: #f0faf5; }

  .status-strip[data-phase="bookkeeping"] .phase-dot,
  .status-strip[data-phase="bookkeeping"] .phase-label { color: #2a5a9a; }
  .status-strip[data-phase="bookkeeping"] { border-color: #6c8fbf; background: #f0f5ff; }

  /* ── Errors ─────────────────────────────────────────── */
  .error {
    border: 1.5px solid var(--p2, #a13a2a);
    border-radius: 6px;
    padding: 0.5em 0.9em;
    color: var(--p2, #a13a2a);
    background: #fff5f3;
    margin-bottom: 0.8rem;
  }

  /* ── Workspace ──────────────────────────────────────── */
  .workspace {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
    gap: 1rem;
  }
  .leftPane,
  .rightPane {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .tabs {
    display: flex;
    gap: 0.25em;
  }
  .tabs button {
    padding: 0.4em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-bottom-color: transparent;
    border-radius: 6px 6px 0 0;
    background: var(--paper-bg);
    font: inherit;
    cursor: pointer;
  }
  .tabs button.active {
    background: white;
    border-bottom-color: white;
    position: relative;
    top: 1px;
    font-weight: 600;
  }
  .panel {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    padding: 1rem;
    background: white;
    min-height: 320px;
  }
  .rightPane h2 {
    margin: 0;
    font-size: 1.05rem;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ── Misc ───────────────────────────────────────────── */
  .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
    white-space: nowrap;
  }
</style>
