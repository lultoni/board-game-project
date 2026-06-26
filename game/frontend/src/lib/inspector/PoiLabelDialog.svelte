<script lang="ts">
  // Tiny POI label dialog. Replaces the prior `window.prompt` at the inspector
  // route. Native <dialog> gives us focus trap, ESC-to-close, and backdrop
  // styling without aria scaffolding. Targets evergreen browsers and Tauri
  // webview, both of which support <dialog> (Safari ≥ 15.4).

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

  let dialogEl: HTMLDialogElement | null = $state(null);
  let value = $state("");
  let inputEl: HTMLInputElement | null = $state(null);

  // When `open` flips true, sync `value` to the latest initial and showModal().
  // When it flips false, close the dialog. We track via $effect rather than
  // bind-with-attribute because <dialog>'s `open` attribute is non-modal —
  // we want the modal (showModal) variant.
  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) {
      value = initial;
      dialogEl.showModal();
      // Defer focus until after the dialog is in the DOM.
      queueMicrotask(() => inputEl?.focus());
    } else if (!open && dialogEl.open) {
      dialogEl.close();
    }
  });

  function handleSave(): void {
    onSave(value.trim());
  }

  function handleCancel(): void {
    onCancel();
  }

  function onDialogCancel(ev: Event): void {
    // Native ESC fires `cancel`. Forward to onCancel so parent state flips.
    ev.preventDefault();
    onCancel();
  }

  function onKeyDown(ev: KeyboardEvent): void {
    if (ev.key === "Enter") {
      ev.preventDefault();
      handleSave();
    }
  }
</script>

<dialog bind:this={dialogEl} oncancel={onDialogCancel}>
  <form method="dialog" onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
    <h3>Label this point of interest</h3>
    <input
      type="text"
      bind:value
      bind:this={inputEl}
      placeholder="e.g. blunder, key decision…"
      maxlength="80"
      onkeydown={onKeyDown}
    />
    <div class="actions">
      <button type="button" onclick={handleCancel}>Cancel</button>
      <button type="submit" class="primary">Save</button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.9rem 1.1rem;
    background: var(--paper-bg);
    color: inherit;
    min-width: min(360px, 90vw);
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.15);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.35);
  }
  h3 {
    margin: 0 0 0.6rem;
    font-size: 1.05rem;
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
