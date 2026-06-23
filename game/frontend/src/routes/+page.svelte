<script lang="ts">
  import { onMount } from "svelte";
  import { getEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";

  let engineVersion = $state<string>(t("app.loading"));
  let backend = $state<"wasm" | "tauri" | "unknown">("unknown");
  let bootError = $state<string | null>(null);

  onMount(async () => {
    try {
      const eng = await getEngine();
      backend = eng.constructor.name === "TauriClient" ? "tauri" : "wasm";
      engineVersion = await eng.version();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
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

  <section class="menu">
    <h2>{t("menu.newMatch")}</h2>
    <ul>
      <li><a href="./draft/?mode=hvh">{t("menu.modeHvh")}</a></li>
      <li><a href="./draft/?mode=hvai">{t("menu.modeHvai")}</a></li>
      <li><a href="./draft/?mode=aivai">{t("menu.modeAivai")}</a></li>
      <li><a href="./replay/">{t("menu.openReplay")}</a></li>
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
