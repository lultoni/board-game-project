<script lang="ts">
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import { applyMasterVolume } from "$lib/audio/sfx";
  import MpErrorBanner from "$lib/multiplayer/MpErrorBanner.svelte";
  let { children } = $props();

  initSettingsPersistence();
  $effect(() => {
    void settings.audioVolume;
    applyMasterVolume();
  });
</script>

<MpErrorBanner />
{@render children?.()}
