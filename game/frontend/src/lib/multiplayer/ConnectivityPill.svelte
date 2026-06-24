<script lang="ts">
  import { mpState, pillState } from "$lib/multiplayer.svelte";
  import { t } from "$lib/state/i18n";

  const pill = $derived(pillState());
  const dot = $derived(
    pill === "live" ? "🟢"
      : pill === "unstable" ? "🟡"
      : pill === "disconnected" ? "🔴"
      : "⚫"
  );
  const label = $derived(
    pill === "live" ? t("multiplayer.pillLive")
      : pill === "unstable" ? t("multiplayer.pillUnstable")
      : pill === "disconnected" ? t("multiplayer.pillDisconnected")
      : t("multiplayer.pillForfeit")
  );

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;
  async function copyCode(): Promise<void> {
    if (!mpState.code) return;
    try {
      await navigator.clipboard.writeText(mpState.code);
      copied = true;
      if (copyTimer) clearTimeout(copyTimer);
      copyTimer = setTimeout(() => (copied = false), 1200);
    } catch { /* clipboard blocked — silent */ }
  }
</script>

<div class="pill" data-state={pill} title={label}>
  <span class="dot" aria-hidden="true">{dot}</span>
  <span class="lbl">{label}</span>
  {#if pill !== "live" && pill !== "unstable" && mpState.code}
    <span class="sep">·</span>
    <span class="code">{mpState.code}</span>
    <button type="button" class="copy" onclick={copyCode}>
      {copied ? t("multiplayer.copied") : t("multiplayer.copy")}
    </button>
  {/if}
</div>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35em;
    padding: 0.15em 0.55em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 999px;
    background: var(--paper-bg);
    font-size: 0.85rem;
    line-height: 1.4;
    color: var(--paper-ink-soft);
  }
  .pill[data-state="live"]         { color: #2a6b3a; border-color: #2a6b3a; }
  .pill[data-state="unstable"]     { color: #8a6a1f; border-color: #8a6a1f; }
  .pill[data-state="disconnected"] { color: #a94b3b; border-color: #a94b3b; }
  .pill[data-state="forfeit"]      { color: #4a4a4a; border-color: #4a4a4a; }
  .dot { font-size: 0.9em; }
  .sep { opacity: 0.5; }
  .code {
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.05em;
    font-weight: 600;
  }
  .copy {
    font: inherit;
    font-size: 0.8rem;
    padding: 0.05em 0.45em;
    border: 1px solid currentColor;
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
</style>
