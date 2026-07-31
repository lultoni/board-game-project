<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { getEngine } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import { getTelemetryStore } from "$lib/storage";
  import { match } from "$lib/state/match-store.svelte";
  import { seatsFromMode, matchModeLabel } from "$lib/state/match-store.svelte";
  import { sfx } from "$lib/audio/sfx";
  import type { MatchMeta } from "$lib/storage";

  let engineVersion = $state<string>(t("app.loading"));
  let bootError = $state<string | null>(null);
  let resumeCount = $state(0);
  let localResumable = $state<MatchMeta[]>([]);
  let importState = $state<"idle" | "loading" | "done" | "error">("idle");
  let importTimer: ReturnType<typeof setTimeout> | null = null;

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
    await loadLocalResumable();
  });

  async function loadLocalResumable(): Promise<void> {
    try {
      const store = getTelemetryStore();
      // Show only "in-progress" local games as active continue-banners. The
      // ✕ marks a game "abandoned" (it drops off the banner but stays in the
      // library and is still resumable there). MP resumes go through the lobby.
      const rows = await store.listMatches({ status: "in-progress" });
      const locals = rows.filter((r) => r.mode !== "multiplayer");
      // Keep only ones that have a saved snapshot, newest first.
      const withSnap: MatchMeta[] = [];
      for (const r of locals) {
        const snap = await store.getResumeSnapshot(r.matchId);
        if (snap) withSnap.push(r);
      }
      withSnap.sort((a, b) => b.startedAtUnixMs - a.startedAtUnixMs);
      localResumable = withSnap.slice(0, 3); // show at most 3
    } catch {
      localResumable = [];
    }
  }

  async function resumeLocalGame(meta: MatchMeta): Promise<void> {
    sfx.play("click");
    match.resumeMatchId = meta.matchId;
    // Recover seats from the stored mode (aivai/hvh/hvai) so the match page
    // boots the right seat kinds; the restored snapshot carries the position.
    match.side = seatsFromMode(meta.mode);
    await goto("./match/");
  }

  /** Dismiss a local resume entry from the hub. Marks the match "abandoned"
   *  (was in-progress) so it drops off the continue-banner but stays in the
   *  library and remains resumable there — non-destructive, unlike a delete. */
  async function dismissLocalGame(matchId: string, ev: MouseEvent): Promise<void> {
    ev.stopPropagation();
    ev.preventDefault();
    sfx.play("click");
    try {
      await getTelemetryStore().markAbandoned(matchId);
    } catch { /* best effort */ }
    await loadLocalResumable();
  }

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

  {#if localResumable.length > 0}
    <div class="local-resume-section">
      {#each localResumable as meta (meta.matchId)}
        <div class="resume-local-row">
          <button
            type="button"
            class="resume-banner resume-local"
            onclick={() => void resumeLocalGame(meta)}
          >
            Continue {matchModeLabel(meta.mode)}
            <span class="resume-local-time">
              {new Date(meta.startedAtUnixMs).toLocaleDateString()}
            </span>
          </button>
          <button
            type="button"
            class="resume-dismiss"
            title="Stop offering to continue (keeps it in your library)"
            aria-label="Dismiss this continue prompt"
            onclick={(ev) => void dismissLocalGame(meta.matchId, ev)}
          >✕</button>
        </div>
      {/each}
    </div>
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
    <a class="tile" href="./position-builder/" onclick={() => sfx.play("click")}>Position Builder</a>
    <label class="tile tile-import" title="Import games from a bundle file" class:busy={importState === "loading"}>
      {#if importState === "loading"}
        Importing…
      {:else if importState === "done"}
        Imported ✓
      {:else if importState === "error"}
        Import failed
      {:else}
        Import Games
      {/if}
      <input
        type="file"
        accept=".json"
        style="display:none"
        disabled={importState === "loading"}
        onchange={async (e) => {
          const file = (e.currentTarget as HTMLInputElement).files?.[0];
          if (!file) return;
          (e.currentTarget as HTMLInputElement).value = "";
          importState = "loading";
          try {
            const text = await file.text();
            const bundle = JSON.parse(text);
            const store = getTelemetryStore();
            const matches = Array.isArray(bundle) ? bundle : bundle.matches ?? [];
            for (const m of matches) {
              if (m.matchId && m.matchLogJson) {
                await store.finalizeMatch(m.matchId, m.matchLogJson, m.endReason ?? "abandoned", m.resultByte ?? 3, m.totalPlies ?? 0, m.totalWallMs ?? 0).catch(() => {});
              }
            }
            sfx.play("click");
            importState = "done";
          } catch {
            importState = "error";
          }
          if (importTimer !== null) clearTimeout(importTimer);
          importTimer = setTimeout(() => { importState = "idle"; }, 2500);
        }}
      />
    </label>
  </section>

  <!-- Dev section: training observatory, hidden from regular players -->
  <details class="dev-section">
    <summary>Dev tools</summary>
    <a class="tile" href="./training/" onclick={() => sfx.play("click")}>Training Observatory</a>
    <a class="tile" href="./inspector/" onclick={() => sfx.play("click")}>{t("menu.openInspector")}</a>
    <a class="tile" href="./replay/" onclick={() => sfx.play("click")}>{t("menu.openReplay")}</a>
  </details>
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

  .local-resume-section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
  }
  .resume-local-row {
    display: flex;
    align-items: stretch;
    gap: 0.4rem;
  }
  .resume-local-row .resume-local { flex: 1; }
  .resume-dismiss {
    flex: 0 0 auto;
    padding: 0 0.7em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    background: var(--paper-bg);
    color: var(--paper-ink-soft);
    font: inherit;
    font-size: 0.9rem;
    cursor: pointer;
    transition: color 120ms ease, border-color 120ms ease;
  }
  .resume-dismiss:hover {
    color: #a94b3b;
    border-color: #a94b3b;
  }
  .resume-local {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    text-align: left;
    font: inherit;
    cursor: pointer;
  }
  .resume-local-time {
    font-size: 0.82rem;
    font-weight: 400;
    color: var(--paper-ink-soft);
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
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
    margin-top: 1rem;
  }
  @media (max-width: 520px) {
    .secondary { grid-template-columns: 1fr 1fr; }
  }
  .tile-import {
    cursor: pointer;
    transition: color 120ms, border-color 120ms;
  }
  .tile-import.busy {
    opacity: 0.6;
    cursor: wait;
  }
  .dev-section {
    margin-top: 1.5rem;
    border-top: 1px solid var(--paper-line);
    padding-top: 0.6rem;
  }
  .dev-section summary {
    font-size: 0.75rem;
    color: var(--paper-ink-soft);
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.5rem;
  }
  .dev-section .tile {
    display: inline-block;
    margin: 0.2rem;
    font-size: 0.82rem;
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
