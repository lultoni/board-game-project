<script lang="ts">
  import { onMount } from "svelte";
  import { getEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import { getTelemetryStore } from "$lib/storage";
  import { sfx } from "$lib/audio/sfx";

  let engineVersion = $state<string>(t("app.loading"));
  let backend = $state<"wasm" | "tauri" | "unknown">("unknown");
  let bootError = $state<string | null>(null);
  let resumeCount = $state(0);

  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;

  onMount(async () => {
    try {
      const eng = await getEngine();
      backend = eng.constructor.name === "TauriClient" ? "tauri" : "wasm";
      engineVersion = await eng.version();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
    // Surface a banner when there are resumable multiplayer sessions. Best-
    // effort: any storage error is swallowed (the banner just doesn't show).
    try {
      const rows = await getTelemetryStore().listMatches({
        mode: "multiplayer",
        status: "mid-match-network-lost",
      });
      const cutoff = Date.now() - RECENT_WINDOW_MS;
      resumeCount = rows.filter((r) =>
        r.startedAtUnixMs >= cutoff
        && !!r.multiplayerCode
        && !!r.multiplayerRole
      ).length;
    } catch {
      resumeCount = 0;
    }
  });
</script>

<main>
  <header>
    <h1>{t("app.title")}</h1>
    <small>
      {t("app.engineVersion", { version: engineVersion })}
      &middot; {backend}
    </small>
  </header>

  {#if bootError}
    <p class="err">boot error: {bootError}</p>
  {/if}

  {#if resumeCount > 0}
    <a class="resume-banner" href="./multiplayer/" onclick={() => sfx.play("click")}>
      {resumeCount === 1
        ? t("menu.resumeBannerOne")
        : t("menu.resumeBannerMany", { n: resumeCount })}
    </a>
  {/if}

  <section class="menu">
    <h2>{t("menu.newMatch")}</h2>
    <ul>
      <li><a href="./setup/" onclick={() => sfx.play("click")}>{t("menu.localPlay")}</a></li>
      <li><a href="./multiplayer/" onclick={() => sfx.play("click")}>{t("menu.openMultiplayer")}</a></li>
      <li><a href="./inspector/" onclick={() => sfx.play("click")}>{t("menu.openInspector")}</a></li>
      <li><a href="./replay/" onclick={() => sfx.play("click")}>{t("menu.openReplay")}</a></li>
      <li><a href="./library/" onclick={() => sfx.play("click")}>{t("menu.openLibrary")}</a></li>
      {#if backend === "tauri"}
        <li><a href="./training/" onclick={() => sfx.play("click")}>Training Observatory</a></li>
      {/if}
    </ul>
  </section>
</main>

<style>
  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 1rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.4rem;
  }
  h1 {
    font-size: 2.4rem;
    letter-spacing: 0.02em;
  }
  small {
    color: var(--paper-ink-soft);
  }
  .resume-banner {
    display: block;
    margin: 0 0 1rem;
    padding: 0.6em 0.9em;
    border: 1.5px solid #8a6a1f;
    border-left: 4px solid #8a6a1f;
    border-radius: 6px;
    background: var(--paper-bg);
    color: inherit;
    text-decoration: none;
    font-weight: 600;
  }
  .resume-banner:hover {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }
  .menu {
    margin-top: 1.5rem;
  }
  .menu ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .menu li {
    margin: 0.4rem 0;
  }
  .menu a {
    display: block;
    padding: 0.6em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
    color: inherit;
    text-decoration: none;
    transition: transform 80ms ease, box-shadow 80ms ease;
  }
  .menu a:hover {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
  }
</style>
