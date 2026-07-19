<script lang="ts">
  // HUD banner that surfaces the most recent multiplayer error
  // (`mpState.lastError`) so the user sees the failure instead of having it
  // buried in a console.warn. Dismiss clears the value so a stale message
  // doesn't linger forever - the next failure will repopulate it.
  //
  // Anchored top-right by default; routes can override via CSS variable
  // `--mp-error-anchor-top` if they want it lower.

  import { mpState } from "$lib/multiplayer.svelte";

  function dismiss(): void {
    mpState.lastError = null;
  }
</script>

{#if mpState.lastError}
  <div class="banner" role="status" aria-live="polite">
    <span class="icon" aria-hidden="true">⚠</span>
    <span class="msg">{mpState.lastError}</span>
    <button type="button" class="dismiss" onclick={dismiss} aria-label="dismiss">
      x
    </button>
  </div>
{/if}

<style>
  .banner {
    position: fixed;
    top: var(--mp-error-anchor-top, 0.6rem);
    right: 0.6rem;
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.45em 0.5em 0.45em 0.7em;
    max-width: min(420px, calc(100vw - 1.2rem));
    border: 1.5px solid var(--paper-bad, #b54a3a);
    border-left-width: 4px;
    border-radius: 5px;
    background: var(--paper-bad-bg, #f7e3df);
    color: var(--paper-ink, #1c1a17);
    font-size: 0.84rem;
    line-height: 1.3;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
    animation: slide-in 160ms ease-out;
  }
  @keyframes slide-in {
    from { opacity: 0; transform: translateY(-6px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .icon {
    color: var(--paper-bad, #b54a3a);
    font-weight: 700;
  }
  .msg {
    flex: 1 1 auto;
    word-break: break-word;
  }
  .dismiss {
    flex: 0 0 auto;
    width: 1.5em;
    height: 1.5em;
    padding: 0;
    border: none;
    border-radius: 3px;
    background: transparent;
    color: var(--paper-ink-soft, #6a6055);
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
  }
  .dismiss:hover {
    background: rgba(0, 0, 0, 0.06);
    color: var(--paper-ink, #1c1a17);
  }
</style>
