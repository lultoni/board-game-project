<script lang="ts">
  // Training Observatory route shell.
  //
  // Wires the top-bar (run-dir picker, Start/Stop, phase indicator) and a
  // tabbed layout for panels 1/2/3/5 on the left + Panel 4 (Inspector)
  // pinned on the right. Polls `read_training_status` at 1 Hz globally and
  // shares the resulting store with child panels via Svelte context.
  //
  // Panels arrive in subsequent commits (C3..C7); this shell renders
  // placeholder cards until they land so the layout itself can be reviewed.

  import { onMount, setContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
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
  // Set to true the moment we issue start_training_run, cleared once the
  // status file confirms the run is actually progressing. Decouples button
  // state from the in-flight IPC, which returns instantly.
  let runRequested = $state<boolean>(false);

  let selectedRaterId = $state<string | null>(null);

  // Status store is created once `runDir` is known. We expose the store via
  // context under a fixed key; panels grab it on mount.
  type StatusStore = ReturnType<typeof createPollingStore<StatusSnapshot>>;
  type StatusValue = { data: StatusSnapshot | null; error: string | null; lastUpdated: number | null };
  let statusStore: StatusStore | null = $state(null);
  let statusValue: StatusValue | null = $state(null);

  // Subscribe to whatever the current `statusStore` is. When the store
  // reference changes (after a fresh run-dir is applied), Svelte's $effect
  // re-runs and we resubscribe.
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
    runDir = runDirInput.trim();
    statusStore = createPollingStore<StatusSnapshot>({
      invokeCmd: "read_training_status",
      args: { runDir },
      intervalMs: 1000,
    });
  }

  async function start(): Promise<void> {
    if (!runDir) return;
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

  // The status file is considered "live" if it was written within the last
  // few seconds — the orchestrator throttles writes to ~1 Hz, so anything
  // older than ~5s means either nobody's running or the run died.
  const STALE_MS = 5000;
  const statusFresh = $derived.by(() => {
    const ts = statusValue?.data?.written_at_ms;
    if (!ts) return false;
    return Date.now() - ts < STALE_MS;
  });

  // The phase we display: only honour the file's `phase` field when the
  // status is fresh AND we're not in pre-start state. Otherwise show "idle"
  // (the orchestrator's idle-phase indicator does not need to round-trip via
  // disk to be true).
  const phase = $derived<TrainingPhase>(
    statusFresh ? (statusValue?.data?.phase ?? "idle") : "idle",
  );
  const isRunning = $derived(runRequested || (statusFresh && phase !== "idle"));

  const generation = $derived(statusValue?.data?.generation ?? null);
  const round = $derived(statusValue?.data?.round ?? null);
  const etaSeconds = $derived(statusValue?.data?.eta_seconds ?? null);
  const populationCount = $derived(statusValue?.data?.population?.length ?? 0);
  const activeMatch = $derived(statusValue?.data?.active_match ?? null);

  // Once a fresh, non-idle snapshot lands, the run is observably progressing
  // and we can clear the "we just clicked start" flag — from that moment on,
  // button state is driven by `statusFresh && phase !== idle`.
  $effect(() => {
    if (runRequested && statusFresh && phase !== "idle") {
      runRequested = false;
    }
  });

  // Human-readable activity sub-line — what is the orchestrator doing right
  // now? Honours phase + active_match + population so the long, quiet
  // training phase of generation 1 isn't invisible.
  const activityLine = $derived.by(() => {
    if (!isRunning) return "Not running.";
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
    return "";
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
</script>

<main>
  <header>
    <p class="back"><a href="../">← Back</a></p>
    <h1>Training Observatory</h1>
  </header>

  <section class="topbar">
    <label class="runDir">
      <span class="lbl">Run directory</span>
      <input type="text" bind:value={runDirInput} spellcheck="false" />
      <button onclick={applyRunDir}>Apply</button>
    </label>

    <div class="phase">
      <span class="lbl">Phase</span>
      <span class="phaseBadge" data-phase={phase}>
        {phase}
      </span>
    </div>

    <div class="genRound">
      <span class="lbl">Generation</span>
      <span class="val">{generation ?? "—"}</span>
      <span class="lbl">Round</span>
      <span class="val">{round ?? "—"}</span>
      <span class="lbl">ETA</span>
      <span class="val">{fmtEta(etaSeconds)}</span>
    </div>

    <div class="presets">
      <span class="lbl">Preset</span>
      <div class="presetButtons" role="radiogroup" aria-label="Run preset">
        <button
          role="radio"
          aria-checked={preset === "smoke"}
          class:active={preset === "smoke"}
          disabled={isRunning}
          onclick={() => (preset = "smoke")}
          title="2 gen × 4 lineage, depth-2 corpus, seconds"
        >Smoke</button>
        <button
          role="radio"
          aria-checked={preset === "medium"}
          class:active={preset === "medium"}
          disabled={isRunning}
          onclick={() => (preset = "medium")}
          title="5 gen × 4 lineage, depth-4 corpus"
        >Medium</button>
        <button
          role="radio"
          aria-checked={preset === "long"}
          class:active={preset === "long"}
          disabled={isRunning}
          onclick={() => (preset = "long")}
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
      >
        {#each backends as b}
          <option value={b.id}>{b.label}{b.is_default ? " (default)" : ""}</option>
        {/each}
      </select>
    </div>

    <div class="controls">
      <button onclick={start} disabled={starting || isRunning || !runDir}>
        {starting ? "Starting…" : isRunning ? "Running…" : "Start"}
      </button>
      <button onclick={stop} disabled={stopping || !isRunning}>
        {stopping ? "Stopping…" : "Stop"}
      </button>
    </div>
  </section>

  <p class="activity" class:idle={!isRunning}>{activityLine}</p>

  {#if startError}
    <p class="error">{startError}</p>
  {/if}

  <section class="workspace">
    <div class="leftPane">
      <div class="tabs" role="tablist">
        <button role="tab" aria-selected={activeTab === "live"}
          class:active={activeTab === "live"}
          onclick={() => (activeTab = "live")}>Live Match</button>
        <button role="tab" aria-selected={activeTab === "standings"}
          class:active={activeTab === "standings"}
          onclick={() => (activeTab = "standings")}>Standings</button>
        <button role="tab" aria-selected={activeTab === "lineage"}
          class:active={activeTab === "lineage"}
          onclick={() => (activeTab = "lineage")}>Lineage</button>
        <button role="tab" aria-selected={activeTab === "matrix"}
          class:active={activeTab === "matrix"}
          onclick={() => (activeTab = "matrix")}>Matrix</button>
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
  .topbar {
    display: grid;
    grid-template-columns: 2fr 1fr 2fr auto auto auto;
    gap: 1rem;
    align-items: center;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.6em 0.9em;
    background: var(--paper-bg);
    margin-bottom: 0.8rem;
  }
  .backend {
    display: grid;
    grid-template-columns: auto auto;
    gap: 0.4em;
    align-items: center;
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
  .presets {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4em;
    align-items: center;
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
  .runDir {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 0.5em;
    align-items: center;
  }
  .runDir input {
    font: inherit;
    padding: 0.3em 0.5em;
    border: 1px solid var(--paper-line);
    border-radius: 4px;
    background: white;
  }
  .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .val {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
  .phase {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.5em;
    align-items: center;
  }
  .phaseBadge {
    padding: 0.15em 0.55em;
    border-radius: 4px;
    border: 1px solid var(--paper-line);
    text-transform: capitalize;
    font-weight: 600;
  }
  .phaseBadge[data-phase="training"] {
    background: #fff3cd;
    border-color: #d4b94a;
  }
  .phaseBadge[data-phase="gauntlet"] {
    background: #d1e7dd;
    border-color: #5bb088;
  }
  .phaseBadge[data-phase="bookkeeping"] {
    background: #cfe2ff;
    border-color: #6c8fbf;
  }
  .phaseBadge[data-phase="idle"] {
    background: var(--paper-bg);
  }
  .genRound {
    display: grid;
    grid-template-columns: repeat(6, auto);
    gap: 0.4em 0.7em;
    align-items: baseline;
  }
  .controls {
    display: flex;
    gap: 0.4em;
  }
  .controls button,
  .runDir button {
    padding: 0.4em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    background: var(--paper-bg);
    font: inherit;
    cursor: pointer;
  }
  .controls button:hover:not(:disabled),
  .runDir button:hover:not(:disabled) {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
  }
  .controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    border: 1.5px solid var(--p2, #a13a2a);
    border-radius: 6px;
    padding: 0.5em 0.9em;
    color: var(--p2, #a13a2a);
    background: #fff5f3;
    margin-bottom: 0.8rem;
  }
  .activity {
    margin: -0.4rem 0 0.8rem;
    font-size: 0.95em;
    color: var(--paper-ink-soft);
  }
  .activity.idle {
    font-style: italic;
  }
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
</style>
