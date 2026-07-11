<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import Settings from "$lib/SettingsModal.svelte";
  import Help from "$lib/HelpModal.svelte";
  import { applyMasterVolume, sfx } from "$lib/audio/sfx";
  import { resetEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import MpErrorBanner from "$lib/multiplayer/MpErrorBanner.svelte";
  let { children } = $props();

  let settingsOpen = $state(false);
  let helpOpen = $state(false);

  initSettingsPersistence();
  $effect(() => {
    void settings.audioVolume;
    applyMasterVolume();
  });

  onMount(() => {
    const onVisibility = () => { if (!document.hidden) sfx.unlock(); };
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  });

  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      resetEngine();
    });
  }
</script>

<MpErrorBanner />
<button class="help-btn" onclick={() => { helpOpen = true; }} aria-label={t("app.help")}>{t("app.help")}</button>
<button class="gear-btn" onclick={() => { settingsOpen = true; }} aria-label={t("app.settings")}>{t("app.settings")}</button>
<Help open={helpOpen} onClose={() => { helpOpen = false; }} />
<Settings open={settingsOpen} onClose={() => { settingsOpen = false; }} />
{@render children?.()}
