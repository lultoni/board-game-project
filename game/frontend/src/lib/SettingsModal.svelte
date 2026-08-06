<script lang="ts">
  import { settings, type AnimationSpeed } from "$lib/state/settings.svelte";
  import { sfx } from "$lib/audio/sfx";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  const ANIMATION_LABELS: Record<AnimationSpeed, string> = {
    off: "Off",
    fast: "Fast",
    normal: "Normal",
    cinematic: "Cinematic",
  };

  const LOCALE_LABELS: Record<"en" | "de", string> = {
    en: "English",
    de: "Deutsch",
  };

  function clampInt(v: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, Math.round(v)));
  }

  // UI-eval evaluator picker: which evaluator drives the eval bar / breakdown
  // panel / replay / inspector (independent of the AI seats). Populated from the
  // unified `list_evaluators` command (builtins + trained raters); the list is
  // loaded once the drawer is first opened.
  interface EvaluatorListing {
    source: "builtin" | "run" | "blessed";
    id: string;
    label: string;
    isChampion: boolean;
  }
  let evaluators = $state<EvaluatorListing[]>([]);
  let evaluatorsLoaded = $state(false);
  async function loadEvaluators(): Promise<void> {
    if (evaluatorsLoaded) return;
    evaluatorsLoaded = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      evaluators = await invoke<EvaluatorListing[]>("list_evaluators", { runDir: null });
    } catch {
      evaluators = [
        { source: "builtin", id: "custom", label: "Custom (default)", isChampion: false },
        { source: "builtin", id: "heuristic", label: "Heuristic", isChampion: false },
      ];
    }
  }
  // Lazy-load when the drawer opens.
  $effect(() => { if (open) void loadEvaluators(); });

  function uiEvalKey(source: string, id: string | null): string {
    if (source === "heuristic") return "builtin:heuristic";
    return `${source}:${id}`;
  }
  const currentUiEvalKey = $derived(uiEvalKey(settings.uiEvaluator.source, settings.uiEvaluator.id));
  function setUiEval(value: string): void {
    const [source, id] = value.split(":", 2);
    settings.uiEvaluator = { source: source as "builtin" | "run" | "blessed", id: id ?? null };
  }

  // Per-AI-seat evaluator pick (same list as the UI-eval pick + the setup
  // screen). Stored in settings.p{1,2}Evaluator; applied on the next engine
  // boot via applyEvaluatorSettings (setup / match / draft / inspector all call
  // it). Changing it mid-match takes effect when the engine is next created.
  function seatEvalKey(seat: "p1" | "p2"): string {
    const c = seat === "p1" ? settings.p1Evaluator : settings.p2Evaluator;
    return uiEvalKey(c.source, c.id);
  }
  function setSeatEval(seat: "p1" | "p2", value: string): void {
    const [source, id] = value.split(":", 2);
    const choice = { source: source as "builtin" | "run" | "blessed", id: id ?? null };
    if (seat === "p1") settings.p1Evaluator = choice;
    else settings.p2Evaluator = choice;
  }
</script>

{#if open}
<aside class="side-drawer" aria-label="Settings">
  <div class="drawer-header">
    <span class="drawer-title">Settings</span>
    <button class="drawer-close" onclick={onClose} aria-label="Close">✕</button>
  </div>
  <div class="drawer-body">
  <section>
    <h3>Visual</h3>
    <label class="row">
      <span>Animation speed</span>
      <div class="segmented">
        {#each (["off", "fast", "normal", "cinematic"] as AnimationSpeed[]) as speed}
          <button
            class:active={settings.animationSpeed === speed}
            onclick={() => { sfx.play("click"); settings.animationSpeed = speed; }}
          >{ANIMATION_LABELS[speed]}</button>
        {/each}
      </div>
    </label>
    <label class="row">
      <span>Wait for animations before next action</span>
      <input type="checkbox" bind:checked={settings.respectAnimation} />
    </label>
    <label class="row">
      <span>Show legal targets</span>
      <input type="checkbox" bind:checked={settings.showLegalTargets} />
    </label>
    <label class="row">
      <span>Show projectile path</span>
      <input type="checkbox" bind:checked={settings.showProjectilePath} />
    </label>
    <label class="row">
      <span>Show illegal owner</span>
      <input type="checkbox" bind:checked={settings.showIllegalOwner} />
    </label>
    <label class="row">
      <span>Show blocked path</span>
      <input type="checkbox" bind:checked={settings.showBlockedByFriendly} />
    </label>
    <label class="row">
      <span>Show bodyguard cover</span>
      <input type="checkbox" bind:checked={settings.showBodyguardCover} />
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>Sound</h3>
    <label class="row">
      <span>Volume</span>
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        bind:value={settings.audioVolume}
        oninput={() => sfx.play("tick")}
        class="slider"
      />
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>Language</h3>
    <label class="row">
      <span>Language</span>
      <div class="segmented">
        {#each (["en", "de"] as const) as loc}
          <button
            class:active={settings.locale === loc}
            onclick={() => { sfx.play("click"); settings.locale = loc; }}
          >{LOCALE_LABELS[loc]}</button>
        {/each}
      </div>
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>Match</h3>
    <label class="row">
      <span>Show AI depth</span>
      <input type="checkbox" bind:checked={settings.showAiDepth} />
    </label>
    <label class="row">
      <span>Show AI think-progress bar</span>
      <input type="checkbox" bind:checked={settings.showThinkProgressBar} />
    </label>
    <label class="row">
      <span>Show heuristic eval</span>
      <input type="checkbox" bind:checked={settings.showHeuristicEval} />
    </label>
    <label class="row">
      <span>Show eval breakdown panel</span>
      <input type="checkbox" bind:checked={settings.showEvalPanel} />
    </label>
    <label class="row">
      <span>Eval panel evaluator</span>
      <select
        value={currentUiEvalKey}
        onchange={(e) => { sfx.play("click"); setUiEval((e.currentTarget as HTMLSelectElement).value); }}
      >
        {#each evaluators as ev}
          <option value={uiEvalKey(ev.source, ev.id)}>{ev.label}</option>
        {/each}
      </select>
    </label>
    <label class="row">
      <span>AIvAI step delay (ms)</span>
      <input
        type="number"
        min="0"
        max="5000"
        step="50"
        value={settings.aivaiStepDelayMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.aivaiStepDelayMs = clampInt(v, 0, 5000);
        }}
      />
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>Replay</h3>
    <label class="row">
      <span>Step delay (ms)</span>
      <input
        type="number"
        min="0"
        max="5000"
        step="50"
        value={settings.replayStepDelayMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.replayStepDelayMs = clampInt(v, 0, 5000);
        }}
      />
    </label>
    <label class="row">
      <span>Loop on end</span>
      <input type="checkbox" bind:checked={settings.replayLoopOnEnd} />
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>AI - Player 1</h3>
    <label class="row">
      <span>Think time (ms)</span>
      <input
        type="number"
        min="0"
        max="30000"
        step="100"
        value={settings.p1ThinkTimeMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p1ThinkTimeMs = clampInt(v, 0, 30000);
        }}
      />
    </label>
    <p class="hint">0 = no time limit; search runs to Max depth.</p>
    <label class="row">
      <span>Max depth</span>
      <select
        value={settings.p1MaxDepth}
        onchange={(e) => {
          settings.p1MaxDepth = parseInt((e.target as HTMLSelectElement).value, 10);
        }}
      >
        {#each [1,2,3,4,5,6,7,8,9,10,12,15,20] as d}
          <option value={d}>{d}</option>
        {/each}
        <option value={0}>∞</option>
      </select>
    </label>
    <label class="row">
      <span>Evaluator</span>
      <select
        value={seatEvalKey("p1")}
        onchange={(e) => { sfx.play("click"); setSeatEval("p1", (e.currentTarget as HTMLSelectElement).value); }}
      >
        {#each evaluators as ev}
          <option value={uiEvalKey(ev.source, ev.id)}>{ev.label}</option>
        {/each}
      </select>
    </label>
    <p class="hint">Applied when the match engine is next created.</p>
  </section>

  <section>
    <h3>AI - Player 2</h3>
    <label class="row">
      <span>Think time (ms)</span>
      <input
        type="number"
        min="0"
        max="30000"
        step="100"
        value={settings.p2ThinkTimeMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p2ThinkTimeMs = clampInt(v, 0, 30000);
        }}
      />
    </label>
    <p class="hint">0 = no time limit; search runs to Max depth.</p>
    <label class="row">
      <span>Max depth</span>
      <select
        value={settings.p2MaxDepth}
        onchange={(e) => {
          settings.p2MaxDepth = parseInt((e.target as HTMLSelectElement).value, 10);
        }}
      >
        {#each [1,2,3,4,5,6,7,8,9,10,12,15,20] as d}
          <option value={d}>{d}</option>
        {/each}
        <option value={0}>∞</option>
      </select>
    </label>
    <label class="row">
      <span>Evaluator</span>
      <select
        value={seatEvalKey("p2")}
        onchange={(e) => { sfx.play("click"); setSeatEval("p2", (e.currentTarget as HTMLSelectElement).value); }}
      >
        {#each evaluators as ev}
          <option value={uiEvalKey(ev.source, ev.id)}>{ev.label}</option>
        {/each}
      </select>
    </label>
    <p class="hint">Applied when the match engine is next created.</p>
  </section>
  </div>
</aside>
{/if}

<style>
  .side-drawer {
    position: fixed;
    top: 0;
    right: 0;
    height: 100dvh;
    width: min(340px, 90vw);
    background: var(--paper-bg, #f3ecd9);
    border-left: 1.5px solid var(--paper-line-strong);
    box-shadow: -4px 0 16px rgba(0,0,0,0.12);
    z-index: 200;
    display: flex;
    flex-direction: column;
    animation: drawer-in 180ms ease-out both;
  }
  @keyframes drawer-in {
    from { transform: translateX(100%); opacity: 0; }
    to   { transform: translateX(0);    opacity: 1; }
  }
  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid var(--paper-line, rgba(58,47,31,0.15));
    flex-shrink: 0;
  }
  .drawer-title { font-weight: 700; font-size: 1rem; }
  .drawer-close {
    background: none; border: none; font-size: 1.1rem; cursor: pointer;
    color: var(--paper-ink-soft); padding: 0.2em 0.4em; border-radius: 4px;
  }
  .drawer-close:hover { background: var(--paper-square-light); }
  .drawer-body { flex: 1; overflow-y: auto; padding: 0 0 1rem; }
  section {
    padding: 0.7rem 1.1rem 0.4rem;
  }
  section h3 {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--paper-ink-soft);
    margin: 0 0 0.5rem;
  }

  .divider {
    height: 1px;
    background: var(--paper-line);
    margin: 0 1.1rem;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.8rem;
    padding: 0.3rem 0;
    font-size: 0.95rem;
    cursor: default;
  }
  .row span {
    flex: 1;
  }
  .row input[type="checkbox"] {
    width: 1.1em;
    height: 1.1em;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .row input[type="number"] {
    width: 5.5em;
    padding: 0.25em 0.45em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
    font: inherit;
    text-align: right;
  }
  .hint {
    margin: -0.15rem 0 0.35rem;
    font-size: 0.78rem;
    opacity: 0.65;
    font-style: italic;
  }
  .slider {
    flex: 1;
    max-width: 160px;
    accent-color: var(--accent);
  }

  .segmented {
    display: flex;
    gap: 0;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    overflow: hidden;
  }
  .segmented button {
    border: none;
    border-left: 1px solid var(--paper-line-strong);
    border-radius: 0;
    padding: 0.25em 0.7em;
    font-size: 0.9em;
    background: transparent;
    cursor: pointer;
    transition: background 100ms;
  }
  .segmented button:first-child {
    border-left: none;
  }
  .segmented button.active {
    background: var(--accent);
    color: #fff;
  }
</style>
