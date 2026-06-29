<script lang="ts">
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import Settings from "$lib/SettingsModal.svelte";
  import { applyMasterVolume } from "$lib/audio/sfx";
  import { resetEngine } from "$lib/engine";
  import MpErrorBanner from "$lib/multiplayer/MpErrorBanner.svelte";
  let { children } = $props();

  let settingsOpen = $state(false);

  initSettingsPersistence();
  $effect(() => {
    void settings.audioVolume;
    applyMasterVolume();
  });

  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      resetEngine();
    });
  }
</script>

<MpErrorBanner />
<button class="gear-btn" onclick={() => { settingsOpen = true; }} aria-label="Open settings">⚙</button>
<Settings open={settingsOpen} onClose={() => { settingsOpen = false; }} />
{@render children?.()}
