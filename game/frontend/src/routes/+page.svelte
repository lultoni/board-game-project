<script lang="ts">
  import { onMount } from "svelte";
  import { getEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import { getTelemetryStore } from "$lib/storage";
  import { sfx } from "$lib/audio/sfx";

  let engineVersion = $state<string>(t("app.loading"));
  let bootError = $state<string | null>(null);
  let resumeCount = $state(0);

  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;

  onMount(async () => {
    try {
      const eng = await getEngine();
      engineVersion = await eng.version();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
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
    <small>{t("app.engineVersion", { version: engineVersion })}</small>
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

  <section class="primary">
    <a class="card card-play" href="./setup/" onclick={() => sfx.play("click")}>
      <span class="card-title">{t("menu.localPlay")}</span>
      <span class="card-sub">{t("menu.localPlaySubtitle")}</span>
    </a>
    <a class="card card-play" href="./multiplayer/" onclick={() => sfx.play("click")}>
      <span class="card-title">{t("menu.openMultiplayer")}</span>
      <span class="card-sub">{t("menu.multiplayerSubtitle")}</span>
    </a>
  </section>

  <section class="primary primary-wide">
    <a class="card card-library" href="./library/" onclick={() => sfx.play("click")}>
      <span class="card-title">{t("menu.openLibrary")}</span>
      <span class="card-sub">{t("menu.librarySubtitle")}</span>
    </a>
  </section>

  <section class="secondary">
    <a class="tile" href="./loadouts/" onclick={() => sfx.play("click")}>{t("menu.openLoadouts")}</a>
    <a class="tile" href="./inspector/" onclick={() => sfx.play("click")}>{t("menu.openInspector")}</a>
    <a class="tile" href="./replay/" onclick={() => sfx.play("click")}>{t("menu.openReplay")}</a>
    <a class="tile" href="./training/" onclick={() => sfx.play("click")}>Training Observatory</a>
  </section>
</main>

<style>
  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    text-align: center;
    margin-bottom: 1.5rem;
    padding-bottom: 0.8rem;
    border-bottom: 1.5px solid var(--paper-line);
  }
  h1 {
    font-size: 2.6rem;
    letter-spacing: 0.02em;
    margin: 0 0 0.2em;
  }
  header small {
    color: var(--paper-ink-soft);
  }

  .resume-banner {
    display: block;
    margin: 0 0 1.2rem;
    padding: 0.7em 0.9em;
    border: 1.5px solid #8a6a1f;
    border-left: 4px solid #8a6a1f;
    border-radius: 6px;
    background: var(--paper-bg);
    color: inherit;
    text-decoration: none;
    font-weight: 600;
    transition: transform 80ms ease, box-shadow 80ms ease;
  }
  .resume-banner:hover {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }

  .primary {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.8rem;
    margin-bottom: 0.8rem;
  }
  .primary-wide {
    grid-template-columns: 1fr;
  }
  @media (max-width: 520px) {
    .primary {
      grid-template-columns: 1fr;
    }
  }

  .card {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 1.1em 1em;
    min-height: 5.2em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    background: var(--paper-bg);
    color: inherit;
    text-decoration: none;
    transition: transform 80ms ease, box-shadow 80ms ease;
  }
  .card:hover {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }
  .card-title {
    font-size: 1.25rem;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .card-sub {
    color: var(--paper-ink-soft);
    font-size: 0.9rem;
    margin-top: 0.15em;
  }
  .card-library {
    background: transparent;
    border-color: var(--paper-line);
    min-height: 4.4em;
    padding: 0.9em 1em;
  }
  .card-library .card-title {
    font-size: 1.1rem;
  }

  .secondary {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
    margin-top: 1rem;
  }
  @media (max-width: 520px) {
    .secondary {
      grid-template-columns: 1fr 1fr;
    }
  }
  .tile {
    display: block;
    text-align: center;
    padding: 0.6em 0.5em;
    border: 1.5px solid var(--paper-line);
    border-radius: 6px;
    background: transparent;
    color: var(--paper-ink-soft);
    text-decoration: none;
    font-size: 0.92rem;
    transition: transform 80ms ease, box-shadow 80ms ease, color 80ms ease, border-color 80ms ease;
  }
  .tile:hover {
    color: var(--paper-ink);
    border-color: var(--paper-line-strong);
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
