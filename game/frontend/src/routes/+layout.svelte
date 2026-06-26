<script lang="ts">
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import { applyMasterVolume } from "$lib/audio/sfx";
  import { resetEngine } from "$lib/engine";
  import MpErrorBanner from "$lib/multiplayer/MpErrorBanner.svelte";
  let { children } = $props();

  initSettingsPersistence();
  $effect(() => {
    void settings.audioVolume;
    applyMasterVolume();
  });

  // Vite HMR hook: when this module is replaced, the live Worker handle is
  // orphaned (a fresh +layout.svelte module instance can't reach the old
  // closure). Terminate the engine worker explicitly so the next getEngine()
  // re-spawns instead of dialling into a worker whose `postMessage` replies
  // are routed to a discarded onmessage handler.
  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      resetEngine();
    });
  }
</script>

<MpErrorBanner />
{@render children?.()}
