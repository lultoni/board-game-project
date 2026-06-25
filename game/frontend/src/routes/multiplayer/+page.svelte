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
    onRawData as mpOnRawData,
    probeHost,
  } from "$lib/multiplayer.svelte";
  import { isValidCode } from "$lib/multiplayer-protocol";
  import { decodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { snapshotJsonFromMatchLog, logIsMidDraftCheap } from "$lib/multiplayer-resume";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import { getTelemetryStore, type MatchMeta, type JoinedCodeEntry } from "$lib/storage";

  type LobbyView = "choose" | "hosting" | "joining" | "joined";

  let view = $state<LobbyView>("choose");
  let codeInput = $state("");
  let codeError = $state<string | null>(null);
  let busy = $state(false);
  // Latches once the host's $effect routes onward, so the same effect can't
  // re-fire after a rejoin/host that already navigated.
  let hostNavigated = $state(false);

  // Network-lost host rows from the last 24h. The joiner's recent list is
  // separate (joinedCodes below) because in the L7c authoritative-host model
  // joiners don't own a `matches` row.
  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;
  let recentLostSessions = $state<MatchMeta[]>([]);
  let recentJoinedCodes = $state<JoinedCodeEntry[]>([]);
  let recentError = $state<string | null>(null);
  // Per-code liveness from a one-shot PeerJS probe. Keyed by 6-digit code.
  let recentLiveness = $state<Record<string, "probing" | "live" | "dead">>({});
  let livenessRefreshTimer: ReturnType<typeof setInterval> | null = null;
  const PROBE_LIMIT = 5;
  const LIVENESS_REFRESH_MS = 5_000;

  function refreshLiveness(): void {
    // Re-probe joined-code cards (joiner-only by construction). Host-role
    // network-lost cards probe their OWN code, which always resolves dead
    // from the same machine — skip those entirely.
    for (const entry of recentJoinedCodes.slice(0, PROBE_LIMIT)) {
      const code = entry.code;
      if (!(code in recentLiveness)) recentLiveness[code] = "probing";
      probeHost(code).then((alive) => {
        recentLiveness[code] = alive ? "live" : "dead";
      });
    }
  }

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
          && r.multiplayerRole === "host"
        )
        .sort((a, b) => b.startedAtUnixMs - a.startedAtUnixMs);
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
    }
  }

  async function loadRecentJoinedCodes(): Promise<void> {
    try {
      const store = getTelemetryStore();
      const all = await store.listJoinedCodes();
      const cutoff = Date.now() - RECENT_WINDOW_MS;
      recentJoinedCodes = all.filter((e) => e.lastJoinedAtUnixMs >= cutoff);
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
    }
  }

  function formatStartedAt(unixMs: number): string {
    const d = new Date(unixMs);
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

  // Shareable URL derived from the host's current code.
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
      hostNavigated = false;
      view = "hosting";
      const code = await mpHost();
      match.multiplayerRole = "host";
      match.multiplayerCode = code;
      match.localSeat = 0;
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
      match.mode = "multiplayer";
      match.side = { p1: "human", p2: "human" };
      // Fresh-joiner default seat. rejoinHost's probe-fall-through path
      // overrides this BEFORE calling startJoin when the rejoining peer
      // was originally the host (seat 0).
      if (match.localSeat === null) match.localSeat = 1;
      // Remember this code so it shows up under "Resume a recent session"
      // next time the user opens the lobby.
      try {
        const store = getTelemetryStore();
        await store.recordJoinedCode({ code });
      } catch { /* telemetry never blocks gameplay */ }
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

  // NOTE: No "Take over as host" affordance lives in the lobby. The takeover
  // CTA is intentionally only surfaced from inside an active match via
  // <GraceBanner /> — the operation requires a live mpEngine handle and the
  // engine state currently in memory, neither of which the lobby has. A user
  // who has navigated back to the lobby has effectively abandoned the session
  // and should Rejoin (which restores from IDB) rather than take over.

  // Host-side Rejoin. Reclaims the same 6-digit code, restores the engine
  // from the persisted MatchLog via a snapshot stuffed into
  // match.pendingSnapshotJson, and navigates immediately (without waiting
  // for the joiner to dial back in). The wrapper in /match/ or /draft/
  // sends `session-hello` once the joiner reconnects; the joiner's wrapper
  // then asks for a snapshot if its mirror seq lags.
  async function rejoinHost(meta: MatchMeta): Promise<void> {
    const code = meta.multiplayerCode!;
    busy = true;
    codeError = null;
    try {
      // Displaced-host check: if another peer already holds this code, the
      // joiner has promoted themselves to host during our absence. Falling
      // through to `hostWithCode` here would conflict with the new authority
      // (broker rejects the re-claim; both sides briefly think they're host).
      // Probe first; if the code is taken, rejoin as joiner — the new host's
      // session-hello + snapshot will re-anchor us on the next bind.
      const codeTaken = await probeHost(code).catch(() => false);
      if (codeTaken) {
        // This peer was originally the host (seat 0), even though the new
        // authority forces them into the "joiner" role for this connection.
        // Pin localSeat now so startJoin's fresh-joiner default doesn't put
        // them on seat 1 — that would swap player identities mid-game.
        match.localSeat = 0;
        // The old matches row stays "mid-match-network-lost" forever otherwise,
        // and after a subsequent takeover we'd accumulate a second row for the
        // same code — two lobby cards offering to resume the same session.
        // Marking it abandoned now keeps the resume list clean. Best-effort.
        try {
          const store = getTelemetryStore();
          await store.dismissNetworkLost(meta.matchId);
        } catch { /* lobby cleanup is best-effort */ }
        codeInput = code;
        await startJoin();
        return;
      }

      const store = getTelemetryStore();
      let matchLogJson: string | null = null;
      try {
        const persisted = await store.getMatch(meta.matchId);
        matchLogJson = persisted?.matchLogJson ?? null;
      } catch { /* fall through with null */ }
      const midDraft = matchLogJson ? logIsMidDraftCheap(matchLogJson) : false;
      const targetRoute = midDraft ? "../draft/" : "../match/";

      resetMatchState();
      hostNavigated = false;
      match.telemetryMatchId = meta.matchId;
      if (matchLogJson) {
        const snap = snapshotJsonFromMatchLog(matchLogJson);
        if (snap) match.pendingSnapshotJson = snap;
      }
      view = "hosting";
      await mpHostWithCode(code);
      match.multiplayerRole = "host";
      match.multiplayerCode = code;
      match.mode = "multiplayer";
      match.side = { p1: "human", p2: "human" };
      match.localSeat = 0;
      hostNavigated = true;
      void goto(targetRoute);
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      codeError = /taken|unavailable-id/i.test(msg)
        ? t("multiplayer.rejoinFailedCodeTaken")
        : t("multiplayer.connectionError", { msg });
      view = "choose";
    } finally {
      busy = false;
    }
  }

  // Joiner-side Rejoin. Same as startJoin, but driven from a card click and
  // refreshes the joined-codes list afterwards.
  async function rejoinJoiner(entry: JoinedCodeEntry): Promise<void> {
    codeInput = entry.code;
    await startJoin();
    await loadRecentJoinedCodes();
  }

  async function dismissSession(meta: MatchMeta): Promise<void> {
    try {
      const store = getTelemetryStore();
      await store.dismissNetworkLost(meta.matchId);
      await loadRecentLost();
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
    }
  }

  async function dismissJoinedCode(entry: JoinedCodeEntry): Promise<void> {
    try {
      const store = getTelemetryStore();
      await store.forgetJoinedCode(entry.code);
      // Drop any liveness entry tied to the dismissed code so the
      // recentLiveness map doesn't grow unboundedly across lobby mounts.
      if (entry.code in recentLiveness) {
        const next = { ...recentLiveness };
        delete next[entry.code];
        recentLiveness = next;
      }
      await loadRecentJoinedCodes();
    } catch (e) {
      recentError = (e as Error)?.message ?? String(e);
    }
  }

  function cancel(): void {
    mpDisconnect();
    match.multiplayerRole = null;
    match.multiplayerCode = null;
    match.localSeat = null;
    view = "choose";
  }

  // Host: once a joiner connects, advance to /setup/ so the host can pick
  // draft mode (custom vs preMade). The wrapper in /setup/'s destination
  // route (/draft/ or /match/) will send `session-hello` and the joiner's
  // V2 peek subscription below will follow.
  $effect(() => {
    if (view !== "hosting") return;
    if (busy) return;
    if (hostNavigated) return;
    if (mpState.status !== "connected") return;
    match.side = { p1: "human", p2: "human" };
    match.mode = "multiplayer";
    match.multiplayerRole = "host";
    if (match.localSeat === null) match.localSeat = 0;
    hostNavigated = true;
    busy = true;
    void goto("../setup/");
  });

  // Joiner: subscribe to V2 raw and route on `session-hello`. The wrapper
  // will own this subscription once we land in /draft/ or /match/, but the
  // lobby needs first dibs to know WHICH route to navigate to. The
  // multiplayer transport's raw inbox buffers any messages that arrive
  // between this unsubscribe and the destination route's wrapper mounting.
  let unsubRaw: (() => void) | null = null;
  function handleSessionHelloPeek(raw: string): void {
    const msg: WireMessageV2 | null = decodeMessageV2(raw);
    if (!msg) return;
    if (msg.kind === "session-hello") {
      match.side = { p1: "human", p2: "human" };
      match.mode = "multiplayer";
      match.multiplayerRole = "joiner";
      if (match.localSeat === null) match.localSeat = 1;
      match.telemetryMatchId = msg.matchId;
      void goto(msg.phase === "draft" ? "../draft/" : "../match/");
      return;
    }
    if (msg.kind === "error" && msg.reason === "session-full") {
      codeError = t("multiplayer.sessionFull");
      mpDisconnect();
      match.multiplayerRole = null;
      match.multiplayerCode = null;
      match.localSeat = null;
      view = "choose";
      return;
    }
    // Anything else (committed, snapshot, …) is for the wrapper. Buffered by
    // the transport's rawInbox once we unsubscribe, replayed when the
    // destination route's wrapper subscribes.
  }

  onMount(async () => {
    // Load both recent lists in parallel.
    await Promise.all([loadRecentLost(), loadRecentJoinedCodes()]);

    refreshLiveness();
    livenessRefreshTimer = setInterval(refreshLiveness, LIVENESS_REFRESH_MS);

    unsubRaw = mpOnRawData(handleSessionHelloPeek);

    // Auto-attempt connect from `?join=XXXXXX` query param.
    const params = typeof window !== "undefined"
      ? new URLSearchParams(window.location.search)
      : null;
    const auto = params?.get("join")?.trim() ?? "";
    if (auto && isValidCode(auto)) {
      codeInput = auto;
      await startJoin();
    }
  });

  onDestroy(() => {
    if (unsubRaw) {
      unsubRaw();
      unsubRaw = null;
    }
    if (livenessRefreshTimer) {
      clearInterval(livenessRefreshTimer);
      livenessRefreshTimer = null;
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
    {#if recentLostSessions.length > 0 || recentJoinedCodes.length > 0}
      <section class="recent">
        <h2>{t("multiplayer.recentSessionsTitle")}</h2>
        <ul class="recent-list">
          {#each recentLostSessions as meta (meta.matchId)}
            <li class="recent-card">
              <div class="recent-meta">
                <span class="recent-time">
                  {t("multiplayer.startedAt", { time: formatStartedAt(meta.startedAtUnixMs) })}
                </span>
                <span class="recent-role">
                  {t("multiplayer.youWerePlayer", { n: 1 })}
                </span>
                <span class="recent-code">{meta.multiplayerCode}</span>
              </div>
              <div class="recent-actions">
                <button class="primary" type="button" disabled={busy}
                  onclick={() => rejoinHost(meta)}>
                  {t("multiplayer.rejoin")}
                </button>
                <button class="ghost" type="button" disabled={busy}
                  onclick={() => dismissSession(meta)}>
                  {t("multiplayer.dismiss")}
                </button>
              </div>
            </li>
          {/each}
          {#each recentJoinedCodes as entry (entry.code)}
            {@const liveness = recentLiveness[entry.code] ?? "probing"}
            <li class="recent-card">
              <div class="recent-meta">
                <span class="liveness" data-state={liveness} aria-hidden="true">
                  {liveness === "live" ? "🟢" : liveness === "dead" ? "⚫" : "·"}
                </span>
                <span class="recent-time">
                  {t("multiplayer.startedAt", { time: formatStartedAt(entry.lastJoinedAtUnixMs) })}
                </span>
                <span class="recent-role">
                  {t("multiplayer.youWerePlayer", { n: 2 })}
                </span>
                <span class="recent-code">{entry.code}</span>
              </div>
              <div class="recent-actions">
                <button class="primary" type="button" disabled={busy}
                  onclick={() => rejoinJoiner(entry)}>
                  {t("multiplayer.rejoin")}
                </button>
                <button class="ghost" type="button" disabled={busy}
                  onclick={() => dismissJoinedCode(entry)}>
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
