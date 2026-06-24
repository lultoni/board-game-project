<script lang="ts">
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import { applyMasterVolume } from "$lib/audio/sfx";
  let { children } = $props();

  initSettingsPersistence();
  $effect(() => {
    void settings.audioVolume;
    applyMasterVolume();
  });
</script>

{@render children?.()}
