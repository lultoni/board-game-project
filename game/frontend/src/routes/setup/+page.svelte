<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/state/i18n";
  import { sfx } from "$lib/audio/sfx";
  import {
    match,
    modeFromSeats,
    multiplayerRole,
    type SeatKind,
    type DraftMode,
    type PreMadeLoadoutId,
  } from "$lib/state/match-store.svelte";
  import { settings, type EvaluatorChoice, type EvaluatorSource } from "$lib/state/settings.svelte";
  import { isPreMadeLoadoutReady } from "$lib/state/draft";
  import {
    disconnect as mpDisconnect,
    mpState,
    onRawData as mpOnRawData,
    sendRaw as mpSendRaw,
    claimRouteOwnership,
  } from "$lib/multiplayer.svelte";
  import { decodeMessageV2, encodeMessageV2, type WireMessageV2 } from "$lib/multiplayer-protocol-v2";
  import { tearDownMultiplayerOnLeave } from "$lib/multiplayer/route-lifecycle";
  import MultiplayerStatusStrip from "$lib/multiplayer/MultiplayerStatusStrip.svelte";
  import { getEngine } from "$lib/engine";
  import { getTelemetryStore } from "$lib/storage";

  // /setup/ is reached two ways:
  //  1. Local-play entry from the main menu (no MP state should be live).
  //  2. Host-side handoff from the lobby once a joiner connects (MP state IS
  //     live — `mpState.status === "connected"` + multiplayerRole === "host").
  //
  // We can't key the multiplayer UI off `match.mode` because, post-L7c, the
  // lobby Back link (a plain href, not the cancel button) leaves `match.mode`
  // and `multiplayerRole` set without a live PeerJS connection — and the
  // /draft/ route then boots in MP mode and reports "disconnected".
  //
  // Treat the live `mpState.status` as the source of truth: if there's no
  // active host/join connection, scrub any residual MP state on mount so
  // local play starts from a clean slate. The lobby's own teardown (cancel,
  // disconnect-on-exit) still runs first when the user uses those paths;
  // this onMount is the safety net for the Back-link case.
  onMount(() => {
    const status = mpState.status;
    const liveMp =
      status === "connected"
      || status === "connecting"
      || status === "hosting"
      || status === "joining";
    if (!liveMp && multiplayerRole() !== null) {
      mpDisconnect();
      match.localSeat = null;
      if (match.mode === "multiplayer") match.mode = "idle";
    }
  });

  // MP joiner: wait for the host's `game-config` and follow the same route
  // the host picks. The host generates + owns the draft-mode decision here on
  // /setup/; the joiner has no picker (see template gate below). Any other
  // v2 envelope (session-hello, committed, snapshot, …) is left in the
  // wrapper's rawInbox and drained by the destination route's mpEngine.
  //
  // navigatingForward gates the onDestroy teardown: when we intentionally
  // hand off to /draft/ or /match/, the next route inherits the live MP
  // connection and we skip the soft teardown.
  let mpRawUnsub: (() => void) | null = null;
  let navigatingForward = false;
  /** Route-ownership token; gates onDestroy's teardown against stale unmounts
   *  that fire after a newer route has claimed ownership. */
  let ownershipToken = 0;
  onMount(() => {
    ownershipToken = claimRouteOwnership();
    if (multiplayerRole() !== "joiner") return;
    // Record this code as recently-joined so the lobby's list shows it if
    // the peer drops before a match row exists (setup-phase drop).
    const code = mpState.code;
    if (code) {
      void getTelemetryStore().recordJoinedCode({ code }).catch(() => { /* telemetry never blocks */ });
    }
    mpRawUnsub = mpOnRawData((raw) => {
      const msg = decodeMessageV2(raw);
      if (!msg || msg.kind !== "game-config") return;
      match.draftMode = msg.mode;
      match.preMadeLoadoutId = msg.mode === "preMade"
        ? (msg.preMadeId as PreMadeLoadoutId | null)
        : null;
      match.side = { p1: "human", p2: "human" };
      navigatingForward = true;
      if (msg.mode === "preMade") {
        void goto("../match/");
      } else {
        void goto("../draft/");
      }
    });
  });

  onDestroy(() => {
    mpRawUnsub?.();
    mpRawUnsub = null;
    // /setup/ never finalises telemetry (no game has been played). The
    // decision is just "did we hand off forward?" — if not, keep state so
    // the peer can rejoin.
    tearDownMultiplayerOnLeave({
      navigatingForward,
      telemetryFinalised: false,
      ownershipToken,
    });
  });

  const isMultiplayer = $derived(multiplayerRole() !== null);
  const isMpHost = $derived(multiplayerRole() === "host");
  const isMpJoiner = $derived(multiplayerRole() === "joiner");

  let p1: SeatKind = $state(match.side.p1);
  let p2: SeatKind = $state(match.side.p2);

  // L8 — draft mode + pre-made loadout selection. Custom is the default;
  // pre-made picks one of the three curated loadouts (all play as mirror
  // matches — both sides use the same loadout).
  let draftMode: DraftMode = $state(match.draftMode);
  let preMadeId: PreMadeLoadoutId = $state(match.preMadeLoadoutId ?? "firstGame");

  const hasAi = $derived(p1 === "ai" || p2 === "ai");
  const isAivAi = $derived(p1 === "ai" && p2 === "ai");

  const preMadeReady = $derived(isPreMadeLoadoutReady(preMadeId));

  const PRE_MADE_OPTIONS: { id: PreMadeLoadoutId; labelKey: string }[] = [
    { id: "firstGame",  labelKey: "setup.preMadeLoadouts.firstGame" },
    { id: "secondGame", labelKey: "setup.preMadeLoadouts.secondGame" },
    { id: "thirdGame",  labelKey: "setup.preMadeLoadouts.thirdGame" },
  ];

  async function start(): Promise<void> {
    sfx.play("wheelOpen");
    if (isMultiplayer) {
      // Seats are forced human in multiplayer; nothing to copy back.
      match.side = { p1: "human", p2: "human" };
    } else {
      match.side = { p1, p2 };
      // Write the local mode so /draft/'s stale-entry guard (which bounces
      // mode === "idle" back to /setup/ as a reloaded-mid-draft recovery)
      // doesn't misfire on the normal fresh-entry path. /draft/ + /match/
      // re-derive mode from seats on mount; this is just the handoff value.
      match.mode = modeFromSeats({ p1, p2 });
    }
    match.draftMode = draftMode;
    match.preMadeLoadoutId = draftMode === "preMade" ? preMadeId : null;
    // MP host coordinates navigation with the joiner via `game-config`. The
    // matchId slot here is a nav-only placeholder — the real authoritative
    // matchId is anchored later by the wrapper's `session-hello` from
    // /draft/ or /match/. We reuse the mp code so the decoder's non-empty
    // check passes without pretending an IDB row exists.
    if (isMpHost) {
      const msg: WireMessageV2 = {
        kind: "game-config",
        mode: draftMode,
        preMadeId: draftMode === "preMade" ? preMadeId : null,
        matchId: mpState.code ?? "pending",
      };
      mpSendRaw(encodeMessageV2(msg));
    }
    if (draftMode === "preMade") {
      // Skip the /draft/ route entirely — /match/ reads preMadeLoadoutId and
      // builds the engine with both sides preloaded.
      navigatingForward = true;
      await goto("../match/");
    } else {
      navigatingForward = true;
      await goto("../draft/");
    }
  }

  // Per-seat evaluator picker. Raters get listed lazily: `default_run_dir`
  // resolves the repo-relative active run, then `list_available_raters` walks
  // both that dir and `raters/blessed/`.
  interface RaterListing { source: EvaluatorSource; id: string; acceptedAt: number; parentId: string | null; }
  let availableRaters = $state<RaterListing[]>([]);
  let raterLoadError = $state<string | null>(null);
  onMount(async () => {
    try {
      const runDir = await invoke<string>("default_run_dir");
      const raw = await invoke<Array<{ source: string; id: string; accepted_at: number; parent_id: string | null }>>(
        "list_available_raters",
        { runDir },
      );
      availableRaters = raw.map((r) => ({
        source: r.source as EvaluatorSource,
        id: r.id,
        acceptedAt: r.accepted_at,
        parentId: r.parent_id,
      }));
    } catch (e) {
      raterLoadError = String(e);
    }
  });

  const ratersBySource = $derived<Record<EvaluatorSource, RaterListing[]>>({
    heuristic: [],
    run: availableRaters.filter((r) => r.source === "run"),
    blessed: availableRaters.filter((r) => r.source === "blessed"),
  });

  function pickEval(seat: "p1" | "p2", choice: EvaluatorChoice) {
    if (seat === "p1") settings.p1Evaluator = choice;
    else                settings.p2Evaluator = choice;
  }
  function onSourceChange(seat: "p1" | "p2", source: EvaluatorSource) {
    if (source === "heuristic") {
      pickEval(seat, { source: "heuristic", id: null });
    } else {
      const first = ratersBySource[source][0]?.id ?? null;
      pickEval(seat, { source, id: first });
    }
  }
</script>

<main>
  <header>
    <p class="back"><a href="../" onclick={() => sfx.play("click")}>{t("setup.back")}</a></p>
    <h1>{t("setup.title")}</h1>
  </header>

  {#if isMultiplayer}
    <MultiplayerStatusStrip
      waitingReason={isMpJoiner ? t("multiplayer.waitingForHostConfig") : null}
    />
    <p class="banner">{t("multiplayer.sessionFor", { n: (match.localSeat ?? 0) + 1 })}</p>
  {:else}
    <section class="seats">
      {#each [{ id: "p1", label: t("setup.p1Label") }, { id: "p2", label: t("setup.p2Label") }] as seat}
        <fieldset class="seat" class:p1={seat.id === "p1"} class:p2={seat.id === "p2"}>
          <legend>{seat.label}</legend>
          <label>
            <input
              type="radio"
              name={seat.id}
              value="human"
              checked={(seat.id === "p1" ? p1 : p2) === "human"}
              onchange={() => { sfx.play("click"); (seat.id === "p1" ? (p1 = "human") : (p2 = "human")); }}
            />
            {t("setup.human")}
          </label>
          <label>
            <input
              type="radio"
              name={seat.id}
              value="ai"
              checked={(seat.id === "p1" ? p1 : p2) === "ai"}
              onchange={() => { sfx.play("click"); (seat.id === "p1" ? (p1 = "ai") : (p2 = "ai")); }}
            />
            {t("setup.ai")}
          </label>
        </fieldset>
      {/each}
    </section>
  {/if}

  {#if hasAi && !isMultiplayer}
    <section class="ai">
      <h2>{t("setup.aiHeader")}</h2>
      <div class="grid">
        {#if p1 === "ai"}
          <label class="row">
            <span class="rowLabel">P1 · {t("setup.thinkTime")}</span>
            <input
              type="range"
              min="0"
              max="5000"
              step="50"
              bind:value={settings.p1ThinkTimeMs}
              oninput={() => sfx.play("tick")}
            />
            <output>{settings.p1ThinkTimeMs}</output>
          </label>
          <label class="row">
            <span class="rowLabel">P1 · {t("setup.maxDepth")}</span>
            <input
              type="range"
              min="1"
              max="12"
              step="1"
              bind:value={settings.p1MaxDepth}
              oninput={() => sfx.play("tick")}
            />
            <output>{settings.p1MaxDepth}</output>
          </label>
          <label class="row">
              <span class="rowLabel">P1 · Evaluator</span>
              <select
                value={settings.p1Evaluator.source}
                onchange={(e) => {
                  sfx.play("tick");
                  onSourceChange("p1", (e.currentTarget as HTMLSelectElement).value as EvaluatorSource);
                }}
              >
                <option value="heuristic">Heuristic</option>
                <option value="run" disabled={ratersBySource.run.length === 0}>Run</option>
                <option value="blessed" disabled={ratersBySource.blessed.length === 0}>Blessed</option>
              </select>
              {#if settings.p1Evaluator.source !== "heuristic"}
                <select
                  value={settings.p1Evaluator.id ?? ""}
                  onchange={(e) => {
                    sfx.play("tick");
                    pickEval("p1", {
                      source: settings.p1Evaluator.source,
                      id: (e.currentTarget as HTMLSelectElement).value || null,
                    });
                  }}
                >
                  {#each ratersBySource[settings.p1Evaluator.source] as r}
                    <option value={r.id}>{r.id}</option>
                  {/each}
                </select>
              {/if}
            </label>
        {/if}
        {#if p2 === "ai"}
          <label class="row">
            <span class="rowLabel">P2 · {t("setup.thinkTime")}</span>
            <input
              type="range"
              min="0"
              max="5000"
              step="50"
              bind:value={settings.p2ThinkTimeMs}
              oninput={() => sfx.play("tick")}
            />
            <output>{settings.p2ThinkTimeMs}</output>
          </label>
          <label class="row">
            <span class="rowLabel">P2 · {t("setup.maxDepth")}</span>
            <input
              type="range"
              min="1"
              max="12"
              step="1"
              bind:value={settings.p2MaxDepth}
              oninput={() => sfx.play("tick")}
            />
            <output>{settings.p2MaxDepth}</output>
          </label>
          <label class="row">
              <span class="rowLabel">P2 · Evaluator</span>
              <select
                value={settings.p2Evaluator.source}
                onchange={(e) => {
                  sfx.play("tick");
                  onSourceChange("p2", (e.currentTarget as HTMLSelectElement).value as EvaluatorSource);
                }}
              >
                <option value="heuristic">Heuristic</option>
                <option value="run" disabled={ratersBySource.run.length === 0}>Run</option>
                <option value="blessed" disabled={ratersBySource.blessed.length === 0}>Blessed</option>
              </select>
              {#if settings.p2Evaluator.source !== "heuristic"}
                <select
                  value={settings.p2Evaluator.id ?? ""}
                  onchange={(e) => {
                    sfx.play("tick");
                    pickEval("p2", {
                      source: settings.p2Evaluator.source,
                      id: (e.currentTarget as HTMLSelectElement).value || null,
                    });
                  }}
                >
                  {#each ratersBySource[settings.p2Evaluator.source] as r}
                    <option value={r.id}>{r.id}</option>
                  {/each}
                </select>
              {/if}
            </label>
        {/if}
        {#if isAivAi}
          <label class="row">
            <span class="rowLabel">{t("setup.aivaiDelay")}</span>
            <input
              type="range"
              min="0"
              max="2000"
              step="50"
              bind:value={settings.aivaiStepDelayMs}
              oninput={() => sfx.play("tick")}
            />
            <output>{settings.aivaiStepDelayMs}</output>
          </label>
        {/if}
      </div>
    </section>
  {/if}

  {#if !isMultiplayer || multiplayerRole() === "host"}
    <section class="draftMode">
      <h2>{t("setup.draftMode.header")}</h2>
      <div class="modes">
        <label>
          <input
            type="radio"
            name="draftMode"
            value="custom"
            checked={draftMode === "custom"}
            onchange={() => { sfx.play("click"); draftMode = "custom"; }}
          />
          <span class="modeLabel">{t("setup.draftMode.custom")}</span>
          <span class="modeHint">{t("setup.draftMode.customHint")}</span>
        </label>
        <label>
          <input
            type="radio"
            name="draftMode"
            value="preMade"
            checked={draftMode === "preMade"}
            onchange={() => { sfx.play("click"); draftMode = "preMade"; }}
          />
          <span class="modeLabel">{t("setup.draftMode.preMade")}</span>
          <span class="modeHint">{t("setup.draftMode.preMadeHint")}</span>
        </label>
      </div>

      {#if draftMode === "preMade"}
        <fieldset class="preMadePicker">
          <legend>{t("setup.preMadeLoadouts.header")}</legend>
          {#each PRE_MADE_OPTIONS as opt}
            {@const ready = isPreMadeLoadoutReady(opt.id)}
            <label class:disabled={!ready}>
              <input
                type="radio"
                name="preMadeId"
                value={opt.id}
                checked={preMadeId === opt.id}
                disabled={!ready}
                onchange={() => { sfx.play("click"); preMadeId = opt.id; }}
              />
              <span>{t(opt.labelKey)}</span>
              {#if !ready}
                <span class="placeholderTag">{t("setup.preMadeLoadouts.placeholder")}</span>
              {/if}
            </label>
          {/each}
          {#if !preMadeReady}
            <p class="warn">{t("setup.preMadeLoadouts.notReadyWarning")}</p>
          {/if}
        </fieldset>
      {/if}
    </section>
  {/if}

  {#if !isMpJoiner}
    <div class="actions">
      <button
        class="primary"
        onclick={start}
        disabled={draftMode === "preMade" && !preMadeReady}
      >{t("setup.continue")}</button>
    </div>
  {/if}
</main>

<style>
  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    margin-bottom: 1rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.4rem;
  }
  .back a {
    color: var(--paper-ink-soft);
    text-decoration: none;
  }
  h1 {
    font-size: 2rem;
    margin: 0.2em 0 0;
  }
  .seats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin: 1rem 0;
  }
  .banner {
    border: 1.5px solid var(--paper-line-strong);
    border-left: 4px solid var(--p1, #2b4a8a);
    border-radius: 6px;
    padding: 0.7em 1em;
    margin: 1rem 0;
    background: var(--paper-bg);
    font-weight: 600;
  }
  .seat {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.6em 0.9em;
    background: var(--paper-bg);
  }
  .seat legend {
    padding: 0 0.3em;
    font-weight: 600;
  }
  .seat.p1 legend { color: var(--p1, #2b4a8a); }
  .seat.p2 legend { color: var(--p2, #a13a2a); }
  .seat label {
    display: block;
    padding: 0.25em 0.1em;
    cursor: pointer;
  }
  .ai {
    margin-top: 1rem;
    border: 1.5px dashed var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.7em 0.9em;
    background: var(--paper-bg);
  }
  .ai h2 {
    font-size: 1.1rem;
    margin: 0 0 0.5em;
  }
  .grid {
    display: grid;
    gap: 0.5em;
  }
  .row {
    display: grid;
    grid-template-columns: 14em 1fr 4em;
    align-items: center;
    gap: 0.6em;
  }
  .rowLabel {
    color: var(--paper-ink-soft);
  }
  .row output {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .actions {
    margin-top: 1.5rem;
    display: flex;
    justify-content: flex-end;
  }
  button.primary {
    padding: 0.7em 1.2em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  button.primary:hover:not(:disabled) {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }
  button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .draftMode {
    margin-top: 1rem;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.7em 0.9em;
    background: var(--paper-bg);
  }
  .draftMode h2 {
    font-size: 1.1rem;
    margin: 0 0 0.5em;
  }
  .modes {
    display: grid;
    gap: 0.4em;
  }
  .modes > label {
    display: grid;
    grid-template-columns: auto auto 1fr;
    align-items: baseline;
    gap: 0.5em;
    padding: 0.3em 0.2em;
    cursor: pointer;
  }
  .modeLabel {
    font-weight: 600;
  }
  .modeHint {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .preMadePicker {
    margin-top: 0.7em;
    border: 1.5px dashed var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.5em 0.8em;
    background: var(--paper-bg);
  }
  .preMadePicker legend {
    padding: 0 0.3em;
    font-weight: 600;
  }
  .preMadePicker label {
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.2em 0;
    cursor: pointer;
  }
  .preMadePicker label.disabled {
    color: var(--paper-ink-soft);
    cursor: not-allowed;
  }
  .placeholderTag {
    font-size: 0.85em;
    border: 1px solid var(--paper-line);
    border-radius: 3px;
    padding: 0 0.4em;
    color: var(--paper-ink-soft);
  }
  .warn {
    margin: 0.4em 0 0;
    font-size: 0.9em;
    color: var(--p2, #a13a2a);
  }
</style>
