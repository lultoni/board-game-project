<script lang="ts">
  import { settings, type AnimationSpeed } from "$lib/state/settings.svelte";
  import { sfx } from "$lib/audio/sfx";
  import Modal from "$lib/ui/Modal.svelte";

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
</script>

<Modal {open} {onClose} title="Settings">
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
  </section>
</Modal>

<style>
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
