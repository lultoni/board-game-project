<script lang="ts">
  // Tiny POI label dialog. Replaces the prior `window.prompt` at the inspector
  // route. Native <dialog> gives us focus trap, ESC-to-close, and backdrop
  // styling without aria scaffolding.

  import Modal from "$lib/ui/Modal.svelte";

  let {
    open,
    initial = "",
    onSave,
    onCancel,
  }: {
    open: boolean;
    initial?: string;
    onSave: (label: string) => void;
    onCancel: () => void;
  } = $props();

  let value = $state("");
  let inputEl: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (open) {
      value = initial;
      queueMicrotask(() => inputEl?.focus());
    }
  });

  function handleSave(): void {
    onSave(value.trim());
  }

  function onKeyDown(ev: KeyboardEvent): void {
    if (ev.key === "Enter") {
      ev.preventDefault();
      handleSave();
    }
  }
</script>

<Modal {open} onClose={onCancel} title="Label this point of interest" width="min(360px, 90vw)" maxHeight="none">
  <form method="dialog" class="body" onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
    <input
      type="text"
      bind:value
      bind:this={inputEl}
      placeholder="e.g. blunder, key decision…"
      maxlength="80"
      onkeydown={onKeyDown}
    />
    <div class="actions">
      <button type="button" onclick={onCancel}>Cancel</button>
      <button type="submit" class="primary">Save</button>
    </div>
  </form>
</Modal>

<style>
  .body {
    padding: 0.9rem 1.1rem;
  }
  input[type="text"] {
    width: 100%;
    box-sizing: border-box;
    padding: 0.4em 0.55em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
    font: inherit;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.8rem;
  }
  .actions button {
    padding: 0.35em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: transparent;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
  .primary {
    background: var(--accent, #5a7cd6);
    color: #fff;
    border-color: var(--accent, #5a7cd6);
    font-weight: 600;
  }
</style>
