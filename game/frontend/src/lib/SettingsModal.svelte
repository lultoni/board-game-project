<script lang="ts">
  import { page } from "$app/stores";
  import { settings, type AnimationSpeed } from "$lib/state/settings.svelte";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  let dialogEl: HTMLDialogElement | null = $state(null);

  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
    }
  });

  function onDialogCancel(ev: Event): void {
    ev.preventDefault();
    onClose();
  }

  const route = $derived($page.url.pathname);
  const isMatch = $derived(route.startsWith("/match"));
  const isReplay = $derived(route.startsWith("/replay"));

  const ANIMATION_LABELS: Record<AnimationSpeed, string> = {
    off: "Off",
    normal: "Normal",
    fast: "Fast",
  };

  function clampInt(v: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, Math.round(v)));
  }
</script>

<dialog bind:this={dialogEl} oncancel={onDialogCancel}>
  <div class="header">
    <h2>Settings</h2>
    <button class="close" onclick={onClose} aria-label="Close settings">✕</button>
  </div>

  <!-- Contextual section -->
  {#if isMatch || isReplay}
    <section>
      <h3>{isReplay ? "Replay" : "Match"}</h3>
      {#if isReplay}
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
        <label class="row">
          <span>Wait for animation</span>
          <input type="checkbox" bind:checked={settings.replayRespectAnimation} />
        </label>
      {/if}
      {#if isMatch}
        <label class="row">
          <span>Show AI depth</span>
          <input type="checkbox" bind:checked={settings.showAiDepth} />
        </label>
      {/if}
    </section>
    <div class="divider"></div>
  {/if}

  <!-- Global -->
  <section>
    <h3>Visual</h3>
    <label class="row">
      <span>Animation speed</span>
      <div class="segmented">
        {#each (["off", "normal", "fast"] as AnimationSpeed[]) as speed}
          <button
            class:active={settings.animationSpeed === speed}
            onclick={() => { settings.animationSpeed = speed; }}
          >{ANIMATION_LABELS[speed]}</button>
        {/each}
      </div>
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
        class="slider"
      />
    </label>
  </section>

  <div class="divider"></div>

  <section>
    <h3>AI — Player 1</h3>
    <label class="row">
      <span>Think time (ms)</span>
      <input
        type="number"
        min="100"
        max="30000"
        step="100"
        value={settings.p1ThinkTimeMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p1ThinkTimeMs = clampInt(v, 100, 30000);
        }}
      />
    </label>
    <label class="row">
      <span>Max depth</span>
      <input
        type="number"
        min="1"
        max="20"
        step="1"
        value={settings.p1MaxDepth}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p1MaxDepth = clampInt(v, 1, 20);
        }}
      />
    </label>
  </section>

  <section>
    <h3>AI — Player 2</h3>
    <label class="row">
      <span>Think time (ms)</span>
      <input
        type="number"
        min="100"
        max="30000"
        step="100"
        value={settings.p2ThinkTimeMs}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p2ThinkTimeMs = clampInt(v, 100, 30000);
        }}
      />
    </label>
    <label class="row">
      <span>Max depth</span>
      <input
        type="number"
        min="1"
        max="20"
        step="1"
        value={settings.p2MaxDepth}
        oninput={(e) => {
          const v = parseInt((e.target as HTMLInputElement).value, 10);
          if (!isNaN(v)) settings.p2MaxDepth = clampInt(v, 1, 20);
        }}
      />
    </label>
  </section>
</dialog>

<style>
  dialog {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 10px;
    padding: 0;
    background: var(--paper-bg);
    color: inherit;
    width: min(480px, 94vw);
    max-height: min(85vh, 700px);
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.4);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.9rem 1.1rem 0.6rem;
    position: sticky;
    top: 0;
    background: var(--paper-bg);
    border-bottom: 1px solid var(--paper-line);
    z-index: 1;
  }
  .header h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .close {
    background: none;
    border: none;
    padding: 0.2em 0.4em;
    font-size: 1rem;
    cursor: pointer;
    color: var(--paper-ink-soft);
    line-height: 1;
  }
  .close:hover {
    color: var(--paper-ink);
  }

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
