<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import "../app.css";
  import { initSettingsPersistence, settings } from "$lib/state/settings.svelte";
  import Settings from "$lib/SettingsModal.svelte";
  import Help from "$lib/HelpModal.svelte";
  import { applyMasterVolume, sfx } from "$lib/audio/sfx";
  import { resetEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import { backNav } from "$lib/state/back-nav.svelte";
  import MpErrorBanner from "$lib/multiplayer/MpErrorBanner.svelte";
  let { children } = $props();

  let settingsOpen = $state(false);
  let helpOpen = $state(false);

  // Global back button. It lives here (not per-route) so it always sits in the
  // top-left, mirroring the Settings/Help pills. Hidden on the hub ("/") — there
  // is nowhere to go back to. Routes register a context-specific destination or
  // teardown via `setBackNav`; otherwise it defaults to the hub.
  const isHub = $derived(page.url.pathname === "/");
  const backHref = $derived(backNav.current?.href ?? "/");
  const backLabel = $derived(backNav.current?.label ?? t("app.back"));

  function onBackClick(ev: MouseEvent): void {
    ev.preventDefault();
    sfx.play("click");
    // Route-registered teardown (leave lobby, abandon game, etc.) runs first,
    // then the layout performs the navigation to the resolved destination.
    backNav.current?.onclick?.(ev);
    void goto(backHref);
  }

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
{#if !isHub}
  <a class="back-btn" href={backHref} onclick={onBackClick}>{backLabel}</a>
{/if}
<button class="help-btn" onclick={() => { helpOpen = !helpOpen; }} aria-label={t("app.help")}>{t("app.help")}</button>
<button class="gear-btn" onclick={() => { settingsOpen = !settingsOpen; }} aria-label={t("app.settings")}>{t("app.settings")}</button>
<Help open={helpOpen} onClose={() => { helpOpen = false; }} />
<Settings open={settingsOpen} onClose={() => { settingsOpen = false; }} />
<div class="app-body" class:has-back={!isHub}>
  {@render children?.()}
</div>

<style>
  /* Override: drawers are fixed side panels, no backdrop. This is the only
     style needed here — the drawer CSS lives in HelpModal/SettingsModal. */
</style>
