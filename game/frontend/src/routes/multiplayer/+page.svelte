<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import {
    host as mpHost,
    join as mpJoin,
    disconnect as mpDisconnect,
    mpState,
    onData,
    sendData,
  } from "$lib/multiplayer.svelte";
  import { isValidCode } from "$lib/multiplayer-protocol";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";

  type LobbyView = "choose" | "hosting" | "joining" | "joined";

  let view = $state<LobbyView>("choose");
  let codeInput = $state("");
  let codeError = $state<string | null>(null);
  let busy = $state(false);

  let unsub: (() => void) | null = null;

  // Shareable URL derived from the host's current code. Built once per code
  // change so we don't construct a new string per keystroke.
  const shareUrl = $derived(
    mpState.code && typeof window !== "undefined"
      ? `${window.location.origin}${window.location.pathname.replace(/[^/]*$/, "")}?join=${mpState.code}`
      : null
  );

  let urlCopied = $state(false);
  let codeCopied = $state(false);
  async function copyText(s: string, which: "url" | "code"): Promise<void> {
    try {
      await navigator.clipboard.writeText(s);
      if (which === "url") {
        urlCopied = true;
        setTimeout(() => (urlCopied = false), 1200);
      } else {
        codeCopied = true;
        setTimeout(() => (codeCopied = false), 1200);
      }
    } catch { /* clipboard blocked — silent */ }
  }

  async function startHost(): Promise<void> {
    busy = true;
    codeError = null;
    try {
      resetMatchState();
      view = "hosting";
      const code = await mpHost();
      match.multiplayerRole = "host";
      match.multiplayerCode = code;
    } catch (e) {
      codeError = (e as Error)?.message ?? String(e);
      view = "choose";
    } finally {
      busy = false;
    }
  }

  async function startJoin(): Promise<void> {
    const code = codeInput.trim();
    if (!isValidCode(code)) {
      codeError = t("multiplayer.invalidCode");
      return;
    }
    busy = true;
    codeError = null;
    try {
      resetMatchState();
      view = "joining";
      await mpJoin(code);
      match.multiplayerRole = "joiner";
      match.multiplayerCode = code;
      view = "joined";
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      codeError = /peer-unavailable/i.test(msg)
        ? t("multiplayer.noSuchSession")
        : t("multiplayer.connectionError", { msg });
      view = "choose";
    } finally {
      busy = false;
    }
  }

  function cancel(): void {
    mpDisconnect();
    match.multiplayerRole = null;
    match.multiplayerCode = null;
    view = "choose";
  }

  // Host: after the local setup/draft completes, the draft route sends the
  // snapshot to the joiner and navigates the host to /match/. Nothing to do
  // here — the goto from the host side happens AFTER this route is unmounted.
  // We only navigate the HOST from the lobby when the joiner connects, so
  // they can pick seats. But seats are forced human in multiplayer — so we
  // just send them to /setup/ once `connected` is true.
  $effect(() => {
    if (view !== "hosting") return;
    if (mpState.status !== "connected") return;
    if (busy) return;
    // Joiner connected. Lock seats + advance the host to /setup/.
    match.side = { p1: "human", p2: "human" };
    match.mode = "multiplayer";
    match.multiplayerRole = "host";
    busy = true;
    void goto("../setup/");
  });

  // Joiner: wait for the host's snapshot, then forward to /match/ directly.
  function handleJoinerMessages(): void {
    if (unsub) unsub();
    unsub = onData((msg) => {
      if (msg.kind === "snapshot") {
        match.side = { p1: "human", p2: "human" };
        match.pendingSnapshotJson = msg.snapshotJson;
        match.mode = "multiplayer";
        match.multiplayerRole = "joiner";
        sendData({ kind: "ready" });
        // The match route consumes pendingSnapshotJson on mount.
        void goto("../match/");
      } else if (msg.kind === "error" && msg.reason === "session-full") {
        codeError = t("multiplayer.sessionFull");
        mpDisconnect();
        match.multiplayerRole = null;
        match.multiplayerCode = null;
        view = "choose";
      }
    });
  }

  onMount(async () => {
    // Auto-attempt connect from `?join=XXXXXX` query param.
    const params = typeof window !== "undefined"
      ? new URLSearchParams(window.location.search)
      : null;
    const auto = params?.get("join")?.trim() ?? "";
    if (auto && isValidCode(auto)) {
      codeInput = auto;
      handleJoinerMessages();
      await startJoin();
    } else {
      handleJoinerMessages();
    }
  });

  onDestroy(() => {
    if (unsub) {
      unsub();
      unsub = null;
    }
    // Don't disconnect on unmount — the host/joiner navigates onward and the
    // connection must persist for the match.
  });
</script>

<main>
  <header>
    <p class="back"><a href="../">{t("multiplayer.back")}</a></p>
    <h1>{t("multiplayer.title")}</h1>
  </header>

  {#if view === "choose"}
    <section class="cards">
      <article class="card host">
        <h2>{t("multiplayer.hostTitle")}</h2>
        <p class="hint">{t("multiplayer.hostHint")}</p>
        <button class="primary" type="button" disabled={busy} onclick={startHost}>
          {t("multiplayer.hostButton")}
        </button>
      </article>

      <article class="card join">
        <h2>{t("multiplayer.joinTitle")}</h2>
        <p class="hint">{t("multiplayer.joinHint")}</p>
        <label class="codefield">
          <span>{t("multiplayer.enterCode")}</span>
          <input
            type="text"
            inputmode="numeric"
            pattern="[0-9]*"
            maxlength="6"
            bind:value={codeInput}
            disabled={busy}
            placeholder="123456"
          />
        </label>
        <button class="primary" type="button" disabled={busy} onclick={startJoin}>
          {t("multiplayer.joinButton")}
        </button>
      </article>
    </section>

    {#if codeError}
      <p class="err">{codeError}</p>
    {/if}
  {:else if view === "hosting"}
    <section class="status">
      {#if mpState.code}
        <div class="code-display">
          <span class="code-label">{t("multiplayer.codeLabel")}</span>
          <div class="code-row">
            <span class="code-big">{mpState.code}</span>
            <button type="button" class="ghost" onclick={() => copyText(mpState.code!, "code")}>
              {codeCopied ? t("multiplayer.copied") : t("multiplayer.copy")}
            </button>
          </div>
          {#if shareUrl}
            <div class="url-row">
              <span class="url-label">{t("multiplayer.urlLabel")}</span>
              <code class="url">{shareUrl}</code>
              <button type="button" class="ghost" onclick={() => copyText(shareUrl, "url")}>
                {urlCopied ? t("multiplayer.copied") : t("multiplayer.copy")}
              </button>
            </div>
          {/if}
        </div>
        <p class="waiting">
          {#if mpState.status === "connecting"}
            {t("multiplayer.connecting")}
          {:else}
            {t("multiplayer.waitingForOpponent")}
          {/if}
        </p>
      {:else}
        <p class="waiting">{t("multiplayer.connecting")}</p>
      {/if}
      <button class="ghost" type="button" onclick={cancel}>{t("multiplayer.back")}</button>
    </section>
  {:else if view === "joining"}
    <section class="status">
      <p class="waiting">{t("multiplayer.connecting")}</p>
      <button class="ghost" type="button" onclick={cancel}>{t("multiplayer.back")}</button>
    </section>
  {:else if view === "joined"}
    <section class="status">
      <p class="connected">{t("multiplayer.connected")}</p>
      <p class="waiting">{t("multiplayer.waitingForHost")}</p>
      <button class="ghost" type="button" onclick={cancel}>{t("multiplayer.back")}</button>
    </section>
  {/if}
</main>

<style>
  main {
    max-width: 760px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.5rem;
    margin-bottom: 1.5rem;
  }
  header h1 {
    margin: 0.2rem 0 0;
    font-size: 1.8rem;
  }
  .back a {
    color: var(--paper-ink-soft);
    text-decoration: none;
  }
  .cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  @media (max-width: 640px) {
    .cards { grid-template-columns: 1fr; }
  }
  .card {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 1rem 1.2rem;
    background: var(--paper-bg);
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .card h2 { margin: 0; font-size: 1.2rem; }
  .hint { color: var(--paper-ink-soft); margin: 0; font-size: 0.92rem; line-height: 1.4; }
  .codefield { display: flex; flex-direction: column; gap: 0.3rem; }
  .codefield span { font-size: 0.85rem; color: var(--paper-ink-soft); }
  .codefield input {
    font: inherit;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.15em;
    font-size: 1.2rem;
    padding: 0.4em 0.6em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
  }
  .primary {
    padding: 0.55em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    color: inherit;
    border-radius: 5px;
    cursor: pointer;
    font: inherit;
    font-weight: 600;
  }
  .primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .ghost {
    padding: 0.35em 0.7em;
    border: 1.5px solid var(--paper-line-strong);
    background: transparent;
    color: inherit;
    border-radius: 5px;
    cursor: pointer;
    font: inherit;
  }
  .status {
    border: 1.5px dashed var(--paper-line-strong);
    border-radius: 8px;
    padding: 1.2rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
  }
  .code-display {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .code-label, .url-label {
    font-size: 0.85rem;
    color: var(--paper-ink-soft);
  }
  .code-row {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    justify-content: center;
  }
  .code-big {
    font-size: 2.4rem;
    letter-spacing: 0.18em;
    font-variant-numeric: tabular-nums;
    font-weight: 700;
  }
  .url-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 0.5rem;
    align-items: center;
    text-align: left;
  }
  .url {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0.3em 0.5em;
    border: 1px solid var(--paper-line);
    border-radius: 4px;
    background: var(--paper-bg);
    font-size: 0.85rem;
  }
  .waiting { color: var(--paper-ink-soft); margin: 0; }
  .connected { color: #2a6b3a; font-weight: 600; margin: 0; }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
    margin-top: 1rem;
  }
</style>
