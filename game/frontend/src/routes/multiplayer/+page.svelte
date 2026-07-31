<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import { setBackNav, clearBackNav } from "$lib/state/back-nav.svelte";
  import { sfx } from "$lib/audio/sfx";
  import {
    host as mpHost,
    join as mpJoin,
    joinKeepState as mpJoinKeepState,
    disconnect as mpDisconnect,
    mpState,
    onRawData as mpOnRawData,
    onConnected as mpOnConnected,
    probeHost,
    isWebRtcSupported,
  } from "$lib/multiplayer.svelte";
  import { isValidCode } from "$lib/multiplayer-protocol-v2";
  import { decodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { snapshotJsonFromMatchLog, logIsMidDraftCheap } from "$lib/multiplayer-resume";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import { getTelemetryStore, type MatchMeta, type JoinedCodeEntry } from "$lib/storage";

  type LobbyView = "choose" | "hosting" | "joining";

  let view = $state<LobbyView>("choose");
  let codeInput = $state("");
  let codeError = $state<string | null>(null);
  let busy = $state(false);
  // Latches once the host's $effect routes onward, so the same effect can't
  // re-fire after a rejoin/host that already navigated.
  let hostNavigated = $state(false);

  // Network-lost sessions from the last 24h. Both host and joiner peers now
  // own their own `matches` row; each machine sees its own local rows here.
  // `recentJoinedCodes` remains for setup-phase drops where no `matches`
  // row exists yet.
  const RECENT_WINDOW_MS = 24 * 60 * 60 * 1000;
  let recentLostSessions = $state<MatchMeta[]>([]);
  let recentJoinedCodes = $state<JoinedCodeEntry[]>([]);
  let recentError = $state<string | null>(null);
  // Per-code liveness from a one-shot relay probe. Keyed by 6-digit code.
  let recentLiveness = $state<Record<string, "probing" | "live" | "dead">>({});
  let livenessRefreshTimer: ReturnType<typeof setInterval> | null = null;
  const PROBE_LIMIT = 5;
  const LIVENESS_REFRESH_MS = 5_000;

  function refreshLiveness(): void {
    // Both matches-row cards and joined-code cards represent "you have a
    // session under this code, click to resume". Probe both uniformly.
    // Skip codes we're currently hosting from THIS machine - the probe would
    // report `live=true` off our own relay session, which is not useful info
    // to the user.
    const own = mpState.code;
    const codes = new Set<string>();
    for (const m of recentLostSessions.slice(0, PROBE_LIMIT)) {
      if (m.multiplayerCode && m.multiplayerCode !== own) codes.add(m.multiplayerCode);
    }
    for (const e of recentJoinedCodes.slice(0, PROBE_LIMIT)) {
      if (e.code !== own) codes.add(e.code);
    }
    for (const code of codes) {
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
        .filter((r) => r.startedAtUnixMs >= cutoff && !!r.multiplayerCode)
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
      // Filter out codes for which we already own a matches row - the row is
      // the source of truth (has phase + status), the joined-code entry is
      // for pre-match sessions only.
      const rows = await store.listMatches({ mode: "multiplayer" });
      const codesWithRow = new Set(rows.map((r) => r.multiplayerCode).filter((c): c is string => !!c));
      recentJoinedCodes = all.filter((e) =>
        e.lastJoinedAtUnixMs >= cutoff && !codesWithRow.has(e.code),
      );
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
    sfx.play("click");
    try {
      await navigator.clipboard.writeText(s);
      if (which === "url") {
        urlCopied = true;
        setTimeout(() => (urlCopied = false), 1200);
      } else {
        codeCopied = true;
        setTimeout(() => (codeCopied = false), 1200);
      }
    } catch { /* clipboard blocked - silent */ }
  }

  async function startHost(): Promise<void> {
    if (!isWebRtcSupported()) return;
    sfx.play("click");
    busy = true;
    codeError = null;
    try {
      resetMatchState();
      hostNavigated = false;
      console.log(`[mp] lobby view: ${view} → hosting (source: startHost)`);
      view = "hosting";
      await mpHost();
      console.log(`[mp] seat write: ${match.localSeat} → 0 (source: startHost)`);
      match.localSeat = 0;
    } catch (e) {
      codeError = (e as Error)?.message ?? String(e);
      console.log(`[mp] lobby view: ${view} → choose (source: startHost.catch)`);
      view = "choose";
    } finally {
      busy = false;
    }
  }

  // Look up this peer's own matches row for a code, regardless of role, and
  // derive the resume route + snapshot from its matchLogJson. Used by both
  // the promoted-host branch and the fresh-joiner path so the routing logic
  // is identical no matter which role the relay ends up assigning.
  async function resumeFromOwnRow(code: string): Promise<{ route: string; matchId: string | null }> {
    let route = "../setup/";
    let matchId: string | null = null;
    try {
      const store = getTelemetryStore();
      const rows = await store.listMatches({ mode: "multiplayer" });
      const own = rows
        .filter((r) => r.multiplayerCode === code && r.status !== "ended")
        .sort((a, b) => b.startedAtUnixMs - a.startedAtUnixMs)[0] ?? null;
      if (own) {
        matchId = own.matchId;
        match.telemetryMatchId = own.matchId;
        const persisted = await store.getMatch(own.matchId);
        const matchLogJson = persisted?.matchLogJson ?? null;
        if (matchLogJson) {
          const snap = snapshotJsonFromMatchLog(matchLogJson);
          if (snap) match.pendingSnapshotJson = snap;
          route = logIsMidDraftCheap(matchLogJson) ? "../draft/" : "../match/";
        }
      }
    } catch { /* IDB failure - fall back to /setup/ */ }
    return { route, matchId };
  }

  async function startJoin(opts: { keepState?: boolean; pinSeat?: 0 | 1 } = {}): Promise<void> {
    if (!isWebRtcSupported()) return;
    const code = codeInput.trim();
    if (!isValidCode(code)) {
      codeError = t("multiplayer.invalidCode");
      return;
    }
    sfx.play("click");
    busy = true;
    codeError = null;
    try {
      resetMatchState();
      // Restore any explicitly pinned seat AFTER reset. Rejoining an ex-host
      // whose ex-joiner has since promoted, or an ex-joiner whose ex-host is
      // gone, requires the seat to survive the relay's role assignment -
      // otherwise identities swap. Seat is a game-identity, role is a
      // network concept; they must not be inferred from each other on rejoin.
      if (opts.pinSeat !== undefined) {
        console.log(`[mp] seat write: ${match.localSeat} → ${opts.pinSeat} (source: startJoin.pinSeat)`);
        match.localSeat = opts.pinSeat;
      }
      console.log(`[mp] lobby view: ${view} → joining (source: startJoin)`);
      view = "joining";
      if (opts.keepState) {
        await mpJoinKeepState(code);
      } else {
        await mpJoin(code);
      }
      match.mode = "multiplayer";
      match.side = { p1: "human", p2: "human" };

      // The relay may have promoted us to host (host slot was empty when we
      // joined). If we do have a matches row, flip its multiplayerRole so the
      // authoritative record reflects the new state. Do NOT overwrite
      // match.localSeat - a pinned seat from rejoinFromRow must survive
      // promotion (an ex-joiner promoted to host still plays seat 1).
      if (mpState.role === "host") {
        if (match.localSeat === null) {
          console.log(`[mp] seat write: null → 0 (source: startJoin.promotedHost)`);
          match.localSeat = 0;
        }
        const { route, matchId } = await resumeFromOwnRow(code);
        if (matchId) {
          try {
            await getTelemetryStore().updateMultiplayerRole(matchId, "host");
          } catch { /* best-effort - the row is still usable */ }
        }
        hostNavigated = true;
        void goto(route);
        return;
      }

      // Fresh-joiner default seat (only if no pin was applied above).
      if (match.localSeat === null) {
        console.log(`[mp] seat write: null → 1 (source: startJoin.freshJoiner)`);
        match.localSeat = 1;
      }
      // Prefer our own matches row for phase derivation (has plies +
      // matchLogJson). If none exists yet (setup-phase drop), fall through
      // to `joined_codes` + /setup/ so the code shows up in the lobby.
      const { route: joinerRoute, matchId: joinerMatchId } = await resumeFromOwnRow(code);
      if (!joinerMatchId) {
        try {
          await getTelemetryStore().recordJoinedCode({ code });
        } catch { /* telemetry never blocks gameplay */ }
      }
      void goto(joinerRoute);
      return;
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      codeError = /peer-unavailable/i.test(msg)
        ? t("multiplayer.noSuchSession")
        : t("multiplayer.connectionError", { msg });
      console.log(`[mp] lobby view: ${view} → choose (source: startJoin.catch)`);
      view = "choose";
    } finally {
      busy = false;
    }
  }

  // NOTE: No "Take over as host" affordance lives in the lobby. The takeover
  // CTA is intentionally only surfaced from inside an active match via
  // <GraceBanner /> - the operation requires a live mpEngine handle and the
  // engine state currently in memory, neither of which the lobby has. A user
  // who has navigated back to the lobby has effectively abandoned the session
  // and should Rejoin (which restores from IDB) rather than take over.

  // Unified rejoin handler for a `matches` row (both host- and joiner-role
  // rows land here - the split into two handlers was a legacy of L7c when
  // only hosts persisted a row). The row's `multiplayerRole` pins the peer's
  // board seat (host → seat 0, joiner → seat 1) so identities can't swap
  // even if the relay assigns a different role on rebind - e.g. an ex-joiner
  // whose ex-host is gone gets promoted to host role but must stay seat 1.
  //
  // Everything else - WS bind, role promotion, IDB role flip, snapshot
  // hydration, phase routing - is already handled by startJoin({keepState:
  // true}) + resumeFromOwnRow(code). No probeHost, no hostWithCode retry
  // ladder: bindJoiner is role-agnostic and the relay decides whether we
  // attach as joiner or get promoted to host.
  async function rejoinFromRow(meta: MatchMeta): Promise<void> {
    const code = meta.multiplayerCode!;
    // Pin seat from the row's role. Passed through startJoin so it survives
    // the `resetMatchState()` at the top of that function. Even if the relay
    // re-issues us the OTHER role on rebind (promoted joiner, or displaced
    // host), we keep the same board seat - role is a network concept, seat
    // is a game-identity concept.
    const seat: 0 | 1 = meta.multiplayerRole === "joiner" ? 1 : 0;
    // Best-effort: clear the network-lost status now so a subsequent takeover
    // can't accumulate a second row for the same code (two lobby cards for
    // one game). startJoin's resumeFromOwnRow re-selects the newest row and
    // updateMultiplayerRole flips its role in place - the same matchId
    // continues to own the log.
    try {
      await getTelemetryStore().dismissNetworkLost(meta.matchId);
    } catch { /* lobby cleanup is best-effort */ }
    codeInput = code;
    await startJoin({ keepState: true, pinSeat: seat });
  }

  // Rejoin from a `joined_codes` entry - a setup-phase drop where no
  // matches row exists yet. Defaults to seat 1 (joiners are player 2 at
  // setup); if we get promoted to host on rebind, startJoin's promoted-host
  // branch handles the role flip but keeps seat 1.
  async function rejoinFromJoinedCode(entry: JoinedCodeEntry): Promise<void> {
    codeInput = entry.code;
    await startJoin({ keepState: true, pinSeat: 1 });
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
    console.log(`[mp] seat write: ${match.localSeat} → null (source: cancel)`);
    match.localSeat = null;
    console.log(`[mp] lobby view: ${view} → choose (source: cancel)`);
    view = "choose";
  }

  /** Leave the lobby cleanly. Same teardown as cancel() (so MP state can't
   *  bleed into a subsequent local-play session), then navigate home. Without
   *  this, the lobby's Back link was a plain `<a href="../">` and left
   *  `multiplayerRole` / `match.mode = "multiplayer"` set if the user had
   *  hosted or joined - /setup/ then forced HvH and /draft/ booted in MP
   *  mode and reported "disconnected". */
  function leaveLobby(): void {
    cancel();
    // Reset mode too - cancel() leaves match.mode alone because it's still
    // owned by the lobby flow until the user actually leaves. The global back
    // button performs the navigation to the hub after this teardown runs.
    if (match.mode === "multiplayer") match.mode = "idle";
  }

  // Host: once a joiner connects (transport onOpen fires with `peer-connected`
  // from the relay), advance to /setup/. Direct callback, not a $effect -
  // navigation is a protocol event, not a UI concern. See PROTOCOL_TRACE.md
  // Part 2 §2. Registered in onMount and disposed in onDestroy.
  let unsubConnected: (() => void) | null = null;
  function handleHostConnected(): void {
    console.log(`[mp] handleHostConnected fired (view=${view}, role=${mpState.role}, localSeat=${match.localSeat}) → navigating=${view === "hosting" && !hostNavigated}`);
    if (view !== "hosting") return;
    if (hostNavigated) return;
    match.side = { p1: "human", p2: "human" };
    match.mode = "multiplayer";
    if (match.localSeat === null) {
      console.log(`[mp] seat write: null → 0 (source: handleHostConnected)`);
      match.localSeat = 0;
    }
    hostNavigated = true;
    void goto("../setup/");
  }

  // Joiner: raw-message subscription. session-hello is no longer used for
  // navigation (host and joiner navigate together on transport.onOpen - see
  // startJoin's post-await goto). This subscription now only surfaces the
  // relay's session-full kick so the lobby can show the right error state.
  // Anything else (session-hello, committed, snapshot, …) is left buffered
  // in the transport's raw inbox for the destination route's wrapper.
  let unsubRaw: (() => void) | null = null;
  function handleLobbyRawPeek(raw: string): void {
    const msg: WireMessageV2 | null = decodeMessageV2(raw);
    if (!msg) return;
    if (msg.kind === "error" && msg.reason === "session-full") {
      codeError = t("multiplayer.sessionFull");
      mpDisconnect();
      console.log(`[mp] seat write: ${match.localSeat} → null (source: session-full)`);
      match.localSeat = null;
      console.log(`[mp] lobby view: ${view} → choose (source: session-full)`);
      view = "choose";
      return;
    }
  }

  onMount(async () => {
    // Leaving the lobby must tear down the relay connection first; register it
    // as the global back button's click handler (it navigates to the hub).
    setBackNav({ onclick: () => leaveLobby() });
    // Load both recent lists in parallel.
    await Promise.all([loadRecentLost(), loadRecentJoinedCodes()]);

    refreshLiveness();
    livenessRefreshTimer = setInterval(refreshLiveness, LIVENESS_REFRESH_MS);

    unsubRaw = mpOnRawData(handleLobbyRawPeek);
    unsubConnected = mpOnConnected(handleHostConnected);

    // Auto-attempt connect from `?join=XXXXXX` query param.
    const params = typeof window !== "undefined"
      ? new URLSearchParams(window.location.search)
      : null;
    const auto = params?.get("join")?.trim() ?? "";
    if (auto && isValidCode(auto)) {
      // Strip the ?join= param immediately so back-navigation doesn't
      // re-trigger startJoin (which would disconnect the active session).
      history.replaceState(null, "", window.location.pathname);
      codeInput = auto;
      await startJoin();
    }
  });

  onDestroy(() => {
    if (unsubRaw) {
      unsubRaw();
      unsubRaw = null;
    }
    if (unsubConnected) {
      unsubConnected();
      unsubConnected = null;
    }
    if (livenessRefreshTimer) {
      clearInterval(livenessRefreshTimer);
      livenessRefreshTimer = null;
    }
    // Don't disconnect on unmount - the host/joiner navigates onward and the
    // connection must persist for the match.
    clearBackNav();
  });
</script>

<main>
  <header>
    <h1>{t("multiplayer.title")}</h1>
  </header>

  {#if view === "choose"}
    {#if recentLostSessions.length > 0 || recentJoinedCodes.length > 0}
      <section class="recent">
        <h2>{t("multiplayer.recentSessionsTitle")}</h2>
        <ul class="recent-list">
          {#each recentLostSessions as meta (meta.matchId)}
            {@const liveness = meta.multiplayerCode === mpState.code
              ? "self"
              : (recentLiveness[meta.multiplayerCode!] ?? "probing")}
            <li class="recent-card">
              <div class="recent-meta">
                <span class="liveness" data-state={liveness} aria-hidden="true">
                  {liveness === "live" ? "🟢" : liveness === "dead" ? "⚫" : "·"}
                </span>
                <span class="recent-time">
                  {t("multiplayer.startedAt", { time: formatStartedAt(meta.startedAtUnixMs) })}
                </span>
                <span class="recent-role">
                  {t("multiplayer.youWerePlayer", { n: meta.multiplayerRole === "joiner" ? 2 : 1 })}
                </span>
                <span class="recent-code">{meta.multiplayerCode}</span>
              </div>
              <div class="recent-actions">
                <button class="primary" type="button" disabled={busy}
                  onclick={() => rejoinFromRow(meta)}>
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
            {@const liveness = entry.code === mpState.code
              ? "self"
              : (recentLiveness[entry.code] ?? "probing")}
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
                  onclick={() => rejoinFromJoinedCode(entry)}>
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
        <button class="primary" type="button" disabled={busy} onclick={() => startJoin()}>
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
  .liveness[data-state="probing"],
  .liveness[data-state="self"] {
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
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
    margin-top: 1rem;
  }
</style>
