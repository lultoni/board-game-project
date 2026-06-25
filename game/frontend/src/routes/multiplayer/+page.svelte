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
  import { extractResumeStateFromLog, snapshotJsonFromMatchLog, logIsMidDraft } from "$lib/multiplayer-resume";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import { getTelemetryStore, type MatchMeta } from "$lib/storage";

  type LobbyView = "choose" | "hosting" | "joining" | "joined";

  let view = $state<LobbyView>("choose");
  let codeInput = $state("");
  let codeError = $state<string | null>(null);
  let busy = $state(false);

  let unsub: (() => void) | null = null;
  // True while the host's `rejoinSession` is awaiting the joiner — flips the
  // "connected" $effect from "advance host to /setup/ for a fresh game" to
  // "advance host straight to /match/ for an in-progress resume". Set
  // before `mpHostWithCode`, read by the $effect, cleared after navigation.
  let resumingHost = $state(false);
  // Resume target for the host: "../draft/" if the saved log is mid-draft,
  // "../match/" otherwise. Set alongside `resumingHost` in rejoinSession;
  // read by the host $effect when advancing.
  let resumingHostTarget = $state<"../draft/" | "../match/">("../match/");

  // Network-lost multiplayer sessions from the last 24h, surfaced as cards
  // above the Host/Join panel so the user can reclaim a session after a
  // dropped tab or network blip. Loaded on mount and refreshed after
  // Dismiss.
  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;
  let recentLostSessions = $state<MatchMeta[]>([]);
  let recentError = $state<string | null>(null);
  // Per-card liveness from a one-shot PeerJS probe. Keyed by matchId.
  let recentLiveness = $state<Record<string, "probing" | "live" | "dead">>({});
  // Interval handle for periodic liveness re-probes while the lobby is
  // mounted. PeerJS broker state can flip (host rehosts, ID slot expires)
  // after the initial one-shot probe — without refresh the dot can stay
  // stale until the user dismisses + re-enters the lobby. Cleared in
  // onDestroy.
  let livenessRefreshTimer: ReturnType<typeof setInterval> | null = null;
  // Cap how many cards we re-probe per cycle to avoid burning broker
  // sockets on long histories.
  const PROBE_LIMIT = 5;
  const LIVENESS_REFRESH_MS = 5_000;

  function refreshLiveness(): void {
    // Re-probe each recent-sessions card. Idempotent — if a probe is still
    // in flight we just kick off another one and the latter resolution
    // wins (last write to recentLiveness[id]). Acceptable: probes finish
    // in <= 2s and the refresh interval is 5s.
    //
    // Host-role cards are skipped: the saved `multiplayerCode` is the
    // host's OWN PeerJS ID, so probing it from the same machine that
    // wasn't actively hosting just returns "unavailable" — the dot would
    // always show black and mean nothing. We hide the dot in the template
    // and skip the probe so we don't burn broker sockets on it.
    for (const meta of recentLostSessions.slice(0, PROBE_LIMIT)) {
      if (!meta.multiplayerCode) continue;
      if (meta.multiplayerRole === "host") continue;
      const id = meta.matchId;
      // Only show "probing" on the very first probe — subsequent refreshes
      // keep the last-known state visible so the dot doesn't flicker.
      if (!(id in recentLiveness)) recentLiveness[id] = "probing";
      probeHost(meta.multiplayerCode).then((alive) => {
        recentLiveness[id] = alive ? "live" : "dead";
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
  //
  // Phase-aware: when the saved MatchLog ends mid-draft (Phase::Draft after
  // replay), we route to /draft/ instead of /match/. The /draft/ route knows
  // how to restore an engine from a snapshot without forwarding to /match/.
  async function rejoinSession(meta: MatchMeta): Promise<void> {
    const code = meta.multiplayerCode!;
    const role = meta.multiplayerRole!;
    busy = true;
    codeError = null;
    try {
      // Load the saved MatchLog once for both branches — used for the
      // snapshot reconstruction and the phase probe.
      const store = getTelemetryStore();
      let matchLogJson: string | null = null;
      try {
        const persisted = await store.getMatch(meta.matchId);
        matchLogJson = persisted?.matchLogJson ?? null;
      } catch { /* fall through with null */ }
      const midDraft = matchLogJson ? await logIsMidDraft(matchLogJson) : false;
      const targetRoute = midDraft ? "../draft/" : "../match/";

      resetMatchState();
      // Carry the existing telemetry id forward so /draft/ and /match/ skip
      // their startTelemetrySession calls. Without this we'd create a new
      // IDB row on resume — the original row would stay in `mid-match-
      // network-lost` and the new one in `in-progress`, producing duplicate
      // recent-sessions cards next time the user drops.
      match.telemetryMatchId = meta.matchId;

      if (role === "host") {
        // Rebuild the host's saved engine state from its own MatchLog. The
        // engine doesn't expose a log-replay API, but Snapshot ({ start_fen,
        // actions, config }) is replayable via `restoreFromSnapshot` and we
        // can construct one from the persisted log. Stuff it into
        // `pendingSnapshotJson` so /match/ or /draft/ restores from it.
        if (matchLogJson) {
          const snap = snapshotJsonFromMatchLog(matchLogJson);
          if (snap) match.pendingSnapshotJson = snap;
        }
        resumingHost = true;
        resumingHostTarget = targetRoute;
        view = "hosting";
        await mpHostWithCode(code);
        match.multiplayerRole = "host";
        match.multiplayerCode = code;
        match.mode = "multiplayer";
        match.side = { p1: "human", p2: "human" };
        // $effect below sees `resumingHost` and routes immediately to
        // `resumingHostTarget` once `mpState.code` is set — without waiting
        // for the joiner. We need to be on the play route with mpUnsub
        // registered before the joiner's resume-request arrives.
      } else {
        // Stage a resume request so the joiner sends it the moment its
        // DataConnection opens, instead of waiting for a fresh snapshot.
        // The host validates the (plyCount, zobrist) pair against its
        // MatchLog and replies accept or reject.
        let plyCount = 0;
        let zobrist = "0";
        if (matchLogJson) {
          const r = extractResumeStateFromLog(matchLogJson);
          plyCount = r.plyCount;
          zobrist = r.zobrist;
        }
        mpState.pendingResume = { code, plyCount, zobrist };
        // For mid-draft resume, also stage the joiner's own snapshot so
        // /draft/ can restore the engine before the host's snapshot arrives.
        // This avoids a flicker where /draft/ would otherwise call
        // createEngineWithDraft and show an empty board until the host
        // responds. (For play-phase resume, /match/ waits for the host's
        // resume-accept and ignores any local snapshot in the meantime.)
        if (midDraft && matchLogJson) {
          const snap = snapshotJsonFromMatchLog(matchLogJson);
          if (snap) match.pendingSnapshotJson = snap;
        }
        codeInput = code;
        view = "joining";
        await mpJoin(code);
        match.multiplayerRole = "joiner";
        match.multiplayerCode = code;
        match.mode = "multiplayer";
        match.side = { p1: "human", p2: "human" };
        void goto(targetRoute);
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
  //
  // Exception: host resume. `rejoinSession` for the host stages a snapshot
  // and sets `resumingHost = true` + `resumingHostTarget`. In that case we
  // advance as soon as the broker accepts our PeerJS ID (`mpState.code` is
  // set), WITHOUT waiting for the joiner. The host needs to be on the play
  // route with mpUnsub registered before the joiner's resume-request lands;
  // otherwise the request sits in the lobby inbox forever.
  $effect(() => {
    if (view !== "hosting") return;
    if (busy) return;
    if (resumingHost && mpState.code) {
      const target = resumingHostTarget;
      resumingHost = false;
      busy = true;
      match.side = { p1: "human", p2: "human" };
      match.mode = "multiplayer";
      match.multiplayerRole = "host";
      void goto(target);
      return;
    }
    if (mpState.status !== "connected") return;
    match.side = { p1: "human", p2: "human" };
    match.mode = "multiplayer";
    match.multiplayerRole = "host";
    busy = true;
    void goto("../setup/");
  });

  // Joiner: wait for the host's signal, then forward to /draft/ (custom) or
  // /match/ (preMade / resume snapshot). The order of message kinds matters:
  //   - `draft-mode` arrives first at fresh-game start (post-/setup/);
  //   - `snapshot` only fires for resume flows or the legacy preMade fallback.
  function handleJoinerMessages(): void {
    if (unsub) unsub();
    unsub = onData((msg) => {
      if (msg.kind === "draft-mode") {
        if (msg.mode === "preMade" && !msg.loadoutId) {
          // Defence-in-depth: the wire decoder rejects this shape, but a peer
          // could still ship a malformed envelope. Reject loudly rather than
          // routing the joiner to /match/ with a null loadoutId (which would
          // crash the boot path).
          codeError = t("multiplayer.connectionError", { msg: "invalid draft-mode" });
          mpDisconnect();
          match.multiplayerRole = null;
          match.multiplayerCode = null;
          view = "choose";
          return;
        }
        match.side = { p1: "human", p2: "human" };
        match.mode = "multiplayer";
        match.multiplayerRole = "joiner";
        match.draftMode = msg.mode;
        match.preMadeLoadoutId = msg.mode === "preMade" ? msg.loadoutId! : null;
        if (msg.mode === "preMade") {
          // /match/ reads preMadeLoadoutId and builds both sides locally.
          // No snapshot exchange needed — engine setup is deterministic.
          void goto("../match/");
        } else {
          // /draft/ co-drafts via the draft-turn wire protocol.
          void goto("../draft/");
        }
      } else if (msg.kind === "snapshot") {
        // Legacy / fallback path: host shipped a pre-built snapshot (used for
        // resume handshakes and pre-Phase-F builds). Drop straight into /match/.
        match.side = { p1: "human", p2: "human" };
        match.pendingSnapshotJson = msg.snapshotJson;
        match.mode = "multiplayer";
        match.multiplayerRole = "joiner";
        sendData({ kind: "ready" });
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
    // ~40 broker sockets every lobby mount. Re-probed every
    // LIVENESS_REFRESH_MS so a rehosted/expired ID slot gets noticed
    // without forcing the user to dismiss + return.
    refreshLiveness();
    livenessRefreshTimer = setInterval(refreshLiveness, LIVENESS_REFRESH_MS);

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
    {#if recentLostSessions.length > 0}
      <section class="recent">
        <h2>{t("multiplayer.recentSessionsTitle")}</h2>
        <ul class="recent-list">
          {#each recentLostSessions as meta (meta.matchId)}
            {@const playerNo = meta.multiplayerRole === "host" ? 1 : 2}
            {@const liveness = recentLiveness[meta.matchId] ?? "probing"}
            {@const showDot = meta.multiplayerRole === "joiner"}
            <li class="recent-card">
              <div class="recent-meta">
                {#if showDot}
                  <span class="liveness" data-state={liveness} aria-hidden="true">
                    {liveness === "live" ? "🟢" : liveness === "dead" ? "⚫" : "·"}
                  </span>
                {/if}
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
