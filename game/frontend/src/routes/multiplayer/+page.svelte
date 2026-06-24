<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import {
    host as mpHost,
    hostWithCode as mpHostWithCode,
    join as mpJoin,
    disconnect as mpDisconnect,
    mpState,
    onData,
    sendData,
    probeHost,
  } from "$lib/multiplayer.svelte";
  import { isValidCode } from "$lib/multiplayer-protocol";
  import { extractResumeStateFromLog } from "$lib/multiplayer-resume";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import { getTelemetryStore, type MatchMeta } from "$lib/storage";

  type LobbyView = "choose" | "hosting" | "joining" | "joined";

  let view = $state<LobbyView>("choose");
  let codeInput = $state("");
  let codeError = $state<string | null>(null);
  let busy = $state(false);

  let unsub: (() => void) | null = null;

  // Network-lost multiplayer sessions from the last 24h, surfaced as cards
  // above the Host/Join panel so the user can reclaim a session after a
  // dropped tab or network blip. Loaded on mount and refreshed after
  // Dismiss.
  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;
  let recentLostSessions = $state<MatchMeta[]>([]);
  let recentError = $state<string | null>(null);
  // Per-card liveness from a one-shot PeerJS probe. Keyed by matchId.
  let recentLiveness = $state<Record<string, "probing" | "live" | "dead">>({});

  async function loadRecentLost(): Promise<void> {
    try {
      const store = getTelemetryStore();
      const rows = await store.listMatches({
        mode: "multiplayer",
        status: "mid-match-network-lost",
      });
      const cutoff = Date.now() - RECENT_WINDOW_MS;
      recentLostSessions = rows
        .filter((r) =>
          r.startedAtUnixMs >= cutoff
          && !!r.multiplayerCode
          && !!r.multiplayerRole
        )
        .sort((a, b) => b.startedAtUnixMs - a.startedAtUnixMs);
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
    }
  }

  function formatStartedAt(unixMs: number): string {
    const d = new Date(unixMs);
    // Locale-aware short time; falls back to ISO if Intl is unavailable.
    try {
      return d.toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return d.toISOString();
    }
  }

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

  // Rejoin a recent network-lost session by re-dialling the saved code in
  // whichever role we held last time. For the host we attempt to claim the
  // same PeerJS ID via hostWithCode; on collision the broker rejects and we
  // surface "code taken". For the joiner we dial the host like any normal
  // join — if the host hasn't come back yet, the dial fails with
  // peer-unavailable, which the existing UI maps to "no such session".
  async function rejoinSession(meta: MatchMeta): Promise<void> {
    const code = meta.multiplayerCode!;
    const role = meta.multiplayerRole!;
    busy = true;
    codeError = null;
    try {
      resetMatchState();
      if (role === "host") {
        view = "hosting";
        await mpHostWithCode(code);
        match.multiplayerRole = "host";
        match.multiplayerCode = code;
        // $effect below picks up "connected" and forwards to /setup/.
      } else {
        // Stage a resume request so the joiner sends it the moment its
        // DataConnection opens, instead of waiting for a fresh snapshot.
        // The host validates the (plyCount, zobrist) pair against its
        // MatchLog and replies accept or reject.
        const store = getTelemetryStore();
        let plyCount = 0;
        let zobrist = "0";
        try {
          const persisted = await store.getMatch(meta.matchId);
          if (persisted?.matchLogJson) {
            const r = extractResumeStateFromLog(persisted.matchLogJson);
            plyCount = r.plyCount;
            zobrist = r.zobrist;
          }
        } catch { /* fall through with 0/"0" sentinels */ }
        mpState.pendingResume = { code, plyCount, zobrist };
        codeInput = code;
        view = "joining";
        await mpJoin(code);
        match.multiplayerRole = "joiner";
        match.multiplayerCode = code;
        // We stay on "joining" — the host's resume-accept will deliver a
        // snapshot. We route the joiner forward to /match/ here so the
        // existing onData subscription there can pick up the snapshot;
        // until accept arrives the joiner sees the in-match boot screen.
        match.mode = "multiplayer";
        match.side = { p1: "human", p2: "human" };
        void goto("../match/");
      }
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      if (role === "host" && /taken|unavailable-id/i.test(msg)) {
        codeError = t("multiplayer.rejoinFailedCodeTaken");
      } else if (/peer-unavailable/i.test(msg)) {
        codeError = t("multiplayer.noSuchSession");
      } else {
        codeError = t("multiplayer.connectionError", { msg });
      }
      view = "choose";
    } finally {
      busy = false;
    }
  }

  async function dismissSession(meta: MatchMeta): Promise<void> {
    try {
      const store = getTelemetryStore();
      await store.dismissNetworkLost(meta.matchId);
      // Drop any liveness entry tied to the dismissed match so the
      // recentLiveness map doesn't grow unboundedly across lobby mounts.
      if (meta.matchId in recentLiveness) {
        const next = { ...recentLiveness };
        delete next[meta.matchId];
        recentLiveness = next;
      }
      await loadRecentLost();
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
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
    // If we got bounced back from /match/ with a resume failure, surface it
    // as a code error and clear the sticky flag.
    if (mpState.resumeFailed) {
      codeError = mpState.resumeFailed === "zobrist-mismatch"
        ? t("multiplayer.resumeFailedZobrist")
        : mpState.resumeFailed === "host-not-in-match"
        ? t("multiplayer.resumeFailedHost")
        : t("multiplayer.resumeFailedNoSession");
      mpState.resumeFailed = null;
    }

    // Load recent network-lost rows so we can offer Rejoin. This is best-
    // effort: if IDB isn't available (private browsing in some Safari
    // modes) the lobby still works without recent-sessions UI.
    await loadRecentLost();

    // Liveness probes — one fire-and-forget PeerJS dial per card to see if
    // the host PeerJS ID is still listening. Visual hint only; the Rejoin
    // button works either way (and a stale dead probe could be wrong if
    // the host reconnects between probe and click).
    // Capped at the 5 most-recent cards so a long history doesn't burn
    // ~40 broker sockets every lobby mount.
    const PROBE_LIMIT = 5;
    for (const meta of recentLostSessions.slice(0, PROBE_LIMIT)) {
      if (!meta.multiplayerCode) continue;
      const id = meta.matchId;
      recentLiveness[id] = "probing";
      probeHost(meta.multiplayerCode).then((alive) => {
        recentLiveness[id] = alive ? "live" : "dead";
      });
    }

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
    {#if recentLostSessions.length > 0}
      <section class="recent">
        <h2>{t("multiplayer.recentSessionsTitle")}</h2>
        <ul class="recent-list">
          {#each recentLostSessions as meta (meta.matchId)}
            {@const playerNo = meta.multiplayerRole === "host" ? 1 : 2}
            {@const liveness = recentLiveness[meta.matchId] ?? "probing"}
            <li class="recent-card">
              <div class="recent-meta">
                <span class="liveness" data-state={liveness} aria-hidden="true">
                  {liveness === "live" ? "🟢" : liveness === "dead" ? "⚫" : "·"}
                </span>
                <span class="recent-time">
                  {t("multiplayer.startedAt", { time: formatStartedAt(meta.startedAtUnixMs) })}
                </span>
                <span class="recent-role">
                  {t("multiplayer.youWerePlayer", { n: playerNo })}
                </span>
                <span class="recent-code">{meta.multiplayerCode}</span>
              </div>
              <div class="recent-actions">
                <button class="primary" type="button" disabled={busy}
                  onclick={() => rejoinSession(meta)}>
                  {t("multiplayer.rejoin")}
                </button>
                <button class="ghost" type="button" disabled={busy}
                  onclick={() => dismissSession(meta)}>
                  {t("multiplayer.dismiss")}
                </button>
              </div>
            </li>
          {/each}
        </ul>
        {#if recentError}
          <p class="err">{recentError}</p>
        {/if}
      </section>
    {/if}

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
  .recent {
    margin-bottom: 1.2rem;
    border: 1.5px solid var(--paper-line-strong);
    border-left: 4px solid #8a6a1f;
    border-radius: 8px;
    padding: 0.7rem 1rem 0.9rem;
    background: var(--paper-bg);
  }
  .recent h2 {
    margin: 0 0 0.5rem;
    font-size: 1.05rem;
  }
  .recent-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .recent-card {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.5rem 0.6rem;
    border: 1px dashed var(--paper-line);
    border-radius: 6px;
  }
  .recent-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.6rem;
  }
  .recent-time { color: var(--paper-ink-soft); font-size: 0.88rem; }
  .recent-role { font-size: 0.9rem; }
  .liveness {
    font-size: 0.85rem;
    line-height: 1;
  }
  .liveness[data-state="probing"] {
    color: var(--paper-ink-soft);
    opacity: 0.5;
  }
  .recent-code {
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.1em;
    font-weight: 600;
  }
  .recent-actions {
    display: flex;
    gap: 0.4rem;
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
