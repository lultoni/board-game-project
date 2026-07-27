<script lang="ts">
  import { sfx } from "$lib/audio/sfx";

  interface Props {
    open: boolean;
    onClose: () => void;
    title: string;
    /** Override default width. Defaults to min(480px, 94vw). */
    width?: string;
    /** Override default max-height. Defaults to min(85vh, 700px). */
    maxHeight?: string;
    children: import("svelte").Snippet;
  }

  let { open, onClose, title, width, maxHeight, children }: Props = $props();

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
</script>

<dialog
  bind:this={dialogEl}
  oncancel={onDialogCancel}
  style:--modal-width={width ?? "min(480px, 94vw)"}
  style:--modal-max-height={maxHeight ?? "min(85vh, 700px)"}
>
  <div class="modal-header">
    <h2>{title}</h2>
    <button class="close" onclick={() => { sfx.play("click"); onClose(); }} aria-label="Close">✕</button>
  </div>
  {@render children()}
</dialog>

<style>
  dialog {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 10px;
    padding: 0;
    background: var(--paper-bg);
    color: inherit;
    width: var(--modal-width);
    max-height: var(--modal-max-height);
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.4);
  }

  .modal-header {
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
  .modal-header h2 {
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
</style>
