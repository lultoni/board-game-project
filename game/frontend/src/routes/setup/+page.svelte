<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import { sfx } from "$lib/audio/sfx";
  import BackButton from "$lib/ui/BackButton.svelte";
  import {
    match,
    modeFromSeats,
    multiplayerRole,
    type SeatKind,
    type DraftMode,
    type LoadoutRef,
    type PreMadeLoadoutId,
  } from "$lib/state/match-store.svelte";
  import { settings } from "$lib/state/settings.svelte";
  import { isPreMadeLoadoutReady } from "$lib/state/draft";
  import type { SavedLoadout } from "$lib/storage/types";
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
  //     live - `mpState.status === "connected"` + multiplayerRole === "host").
  //
  // We can't key the multiplayer UI off `match.mode` because, post-L7c, the
  // lobby Back link (a plain href, not the cancel button) leaves `match.mode`
  // and `multiplayerRole` set without a live relay connection - and the
  // /draft/ route then boots in MP mode and reports "disconnected".
  //
  // Treat the live `mpState.status` as the source of truth: if there's no
  // active host/join connection, scrub any residual MP state on mount so
  // local play starts from a clean slate. The lobby's own teardown (cancel,
  // disconnect-on-exit) still runs first when the user uses those paths;
  // this onMount is the safety net for the Back-link case.
  onMount(() => {
    console.log(`[mp] /setup/ mounted (mode=${match.mode}, role=${multiplayerRole()}, localSeat=${match.localSeat}, status=${mpState.status})`);
    const status = mpState.status;
    const liveMp =
      status === "connected"
      || status === "connecting"
      || status === "hosting"
      || status === "joining";
    if (!liveMp && multiplayerRole() !== null) {
      mpDisconnect();
      console.log(`[mp] seat write: ${match.localSeat} → null (source: setup.safetyNet)`);
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
      if (msg.mode === "preMade" && msg.preMadeId) {
        // Multiplayer keeps the shared-picker rule: both sides play the same
        // pre-made loadout. Custom loadouts are disabled in MP for fairness.
        const ref: LoadoutRef = { kind: "preMade", id: msg.preMadeId as PreMadeLoadoutId };
        match.sideLoadouts = { p1: ref, p2: ref };
      } else {
        match.sideLoadouts = null;
      }
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
    // decision is just "did we hand off forward?" - if not, keep state so
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

  // L8/Task 8 - draft mode + per-side loadout selection. Custom is the
  // default; pre-made picks one of the three curated loadouts (or a saved
  // custom loadout, local play only).
  //
  // For local play, P1 and P2 can independently pick any pre-made or any
  // saved custom loadout. For multiplayer, the picker is a single shared
  // control that only offers pre-mades - both sides get the same ref. The
  // fairness story for MP custom loadouts hasn't been designed yet
  // (deferred question), so custom picks are disallowed in MP.
  let draftMode: DraftMode = $state(match.draftMode);
  // Initialize each side's ref from whatever match state currently holds, or
  // default to firstGame. Saved rows are loaded async in a separate onMount.
  const initialRef = (side: "p1" | "p2"): LoadoutRef => {
    const cur = match.sideLoadouts?.[side];
    if (cur) return cur;
    return { kind: "preMade", id: "firstGame" };
  };
  let p1Ref = $state<LoadoutRef>(initialRef("p1"));
  let p2Ref = $state<LoadoutRef>(initialRef("p2"));
  // MP shared-picker mirror: the host's single pick that both sides receive.
  let mpSharedPreMadeId = $state<PreMadeLoadoutId>(
    (match.sideLoadouts?.p1.kind === "preMade" ? match.sideLoadouts.p1.id : null) ?? "firstGame",
  );

  // Saved custom loadouts, loaded async on mount. Empty until the IDB
  // read completes; the picker treats an empty list as "no customs yet"
  // and only shows the pre-made section.
  let savedLoadouts = $state<SavedLoadout[]>([]);
  onMount(async () => {
    try {
      const { getTelemetryStore } = await import("$lib/storage");
      savedLoadouts = await getTelemetryStore().listLoadouts();
    } catch {
      savedLoadouts = [];
    }
  });

  const hasAi = $derived(p1 === "ai" || p2 === "ai");
  const isAivAi = $derived(p1 === "ai" && p2 === "ai");

  // --- Trained-rater picker (ns-50) --------------------------------------
  // Each AI seat can use the built-in Heuristic (default) or a trained rater.
  // The choice is stored in settings.{p1,p2}Evaluator ({ source, id }); the
  // match route already installs it via set_ai_evaluator at game start. We
  // just populate the dropdown here from the active run dir + blessed set.
  interface RaterListing {
    source: "run" | "blessed";
    id: string;
    acceptedAt: string;
    parentId: string | null;
    isChampion: boolean;
  }
  let raters = $state<RaterListing[]>([]);
  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      raters = await invoke<RaterListing[]>("list_available_raters", { runDir: null });
    } catch {
      raters = []; // no trainer / no raters yet - picker shows Heuristic only.
    }
  });
  const champion = $derived(raters.find((r) => r.isChampion) ?? null);

  /** Encode a seat's evaluator choice as a single select value. */
  function evalKey(source: string, id: string | null): string {
    return source === "heuristic" ? "heuristic" : `${source}:${id}`;
  }
  function currentEvalKey(seat: "p1" | "p2"): string {
    const c = seat === "p1" ? settings.p1Evaluator : settings.p2Evaluator;
    return evalKey(c.source, c.id);
  }
  /** Apply a select value back to the per-seat evaluator setting. */
  function setEval(seat: "p1" | "p2", value: string): void {
    let choice: { source: "heuristic" | "run" | "blessed"; id: string | null };
    if (value === "heuristic") {
      choice = { source: "heuristic", id: null };
    } else if (value === "champion") {
      choice = champion
        ? { source: champion.source, id: champion.id }
        : { source: "heuristic", id: null };
    } else {
      const [source, id] = value.split(":", 2);
      choice = { source: source as "run" | "blessed", id: id ?? null };
    }
    if (seat === "p1") settings.p1Evaluator = choice;
    else settings.p2Evaluator = choice;
  }

  /** Returns true when the given ref points at something valid and complete.
   *  For pre-mades this is `isPreMadeLoadoutReady`; for custom refs it
   *  checks that the id still exists in `savedLoadouts`. */
  function isRefReady(ref: LoadoutRef): boolean {
    if (ref.kind === "preMade") return isPreMadeLoadoutReady(ref.id);
    return savedLoadouts.some((r) => r.id === ref.id);
  }

  const p1Ready = $derived(isRefReady(p1Ref));
  const p2Ready = $derived(isRefReady(p2Ref));
  const mpSharedReady = $derived(isPreMadeLoadoutReady(mpSharedPreMadeId));
  const bothReady = $derived(
    isMultiplayer ? mpSharedReady : (p1Ready && p2Ready),
  );

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
    if (draftMode === "preMade") {
      if (isMultiplayer) {
        // MP: mirror the host's single pre-made pick onto both sides.
        const ref: LoadoutRef = { kind: "preMade", id: mpSharedPreMadeId };
        match.sideLoadouts = { p1: ref, p2: ref };
      } else {
        match.sideLoadouts = { p1: p1Ref, p2: p2Ref };
      }
    } else {
      match.sideLoadouts = null;
    }
    // MP host coordinates navigation with the joiner via `game-config`. The
    // matchId slot here is a nav-only placeholder - the real authoritative
    // matchId is anchored later by the wrapper's `session-hello` from
    // /draft/ or /match/. We reuse the mp code so the decoder's non-empty
    // check passes without pretending an IDB row exists.
    if (isMpHost) {
      const msg: WireMessageV2 = {
        kind: "game-config",
        mode: draftMode,
        preMadeId: draftMode === "preMade" ? mpSharedPreMadeId : null,
        matchId: mpState.code ?? "pending",
      };
      mpSendRaw(encodeMessageV2(msg));
    }
    if (draftMode === "preMade") {
      // Skip the /draft/ route entirely - /match/ reads sideLoadouts and
      // builds the engine with both sides preloaded.
      navigatingForward = true;
      await goto("../match/");
    } else {
      navigatingForward = true;
      await goto("../draft/");
    }
  }
</script>

<main>
  <header>
    <BackButton />
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
            <span class="rowLabel">P1 · {t("setup.evaluator")}</span>
            <select
              value={currentEvalKey("p1")}
              onchange={(e) => { sfx.play("click"); setEval("p1", e.currentTarget.value); }}
            >
              <option value="heuristic">{t("setup.evaluatorHeuristic")}</option>
              {#if champion}
                <option value="champion">{t("setup.evaluatorChampion")} ({champion.id})</option>
              {/if}
              {#each raters as r}
                <option value={evalKey(r.source, r.id)}>{r.id} ({r.source})</option>
              {/each}
            </select>
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
            <span class="rowLabel">P2 · {t("setup.evaluator")}</span>
            <select
              value={currentEvalKey("p2")}
              onchange={(e) => { sfx.play("click"); setEval("p2", e.currentTarget.value); }}
            >
              <option value="heuristic">{t("setup.evaluatorHeuristic")}</option>
              {#if champion}
                <option value="champion">{t("setup.evaluatorChampion")} ({champion.id})</option>
              {/if}
              {#each raters as r}
                <option value={evalKey(r.source, r.id)}>{r.id} ({r.source})</option>
              {/each}
            </select>
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
      {#if raters.length > 0}
        <p class="raterNote">{t("setup.evaluatorNote")}</p>
      {/if}
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
        {#if isMultiplayer}
          <!-- MP: single shared picker; both sides play the same pre-made. -->
          <fieldset class="preMadePicker">
            <legend>{t("setup.preMadeLoadouts.header")}</legend>
            {#each PRE_MADE_OPTIONS as opt}
              {@const ready = isPreMadeLoadoutReady(opt.id)}
              <label class:disabled={!ready}>
                <input
                  type="radio"
                  name="mpSharedPreMadeId"
                  value={opt.id}
                  checked={mpSharedPreMadeId === opt.id}
                  disabled={!ready}
                  onchange={() => { sfx.play("click"); mpSharedPreMadeId = opt.id; }}
                />
                <span>{t(opt.labelKey)}</span>
                {#if !ready}
                  <span class="placeholderTag">{t("setup.preMadeLoadouts.placeholder")}</span>
                {/if}
              </label>
            {/each}
            {#if !mpSharedReady}
              <p class="warn">{t("setup.preMadeLoadouts.notReadyWarning")}</p>
            {/if}
          </fieldset>
        {:else}
          <!-- Local play: per-side pickers. Each side lists pre-mades then a
               divider then any saved custom loadouts. Passing a `ref` shape
               keeps the union type honest at the write site. -->
          <div class="sidePickers">
            {#each [{ side: "p1" as const, label: t("setup.p1Label") }, { side: "p2" as const, label: t("setup.p2Label") }] as col}
              {@const currentRef = col.side === "p1" ? p1Ref : p2Ref}
              <fieldset class="preMadePicker side" class:p1={col.side === "p1"} class:p2={col.side === "p2"}>
                <legend>{col.label}</legend>
                {#each PRE_MADE_OPTIONS as opt}
                  {@const ready = isPreMadeLoadoutReady(opt.id)}
                  {@const isChecked = currentRef.kind === "preMade" && currentRef.id === opt.id}
                  <label class:disabled={!ready}>
                    <input
                      type="radio"
                      name={`loadout-${col.side}`}
                      checked={isChecked}
                      disabled={!ready}
                      onchange={() => {
                        sfx.play("click");
                        const ref: LoadoutRef = { kind: "preMade", id: opt.id };
                        if (col.side === "p1") p1Ref = ref; else p2Ref = ref;
                      }}
                    />
                    <span>{t(opt.labelKey)}</span>
                    {#if !ready}
                      <span class="placeholderTag">{t("setup.preMadeLoadouts.placeholder")}</span>
                    {/if}
                  </label>
                {/each}
                {#if savedLoadouts.length > 0}
                  <div class="divider">- {t("loadouts.listHeading")} -</div>
                  {#each savedLoadouts as row (row.id)}
                    {@const isChecked = currentRef.kind === "custom" && currentRef.id === row.id}
                    <label>
                      <input
                        type="radio"
                        name={`loadout-${col.side}`}
                        checked={isChecked}
                        onchange={() => {
                          sfx.play("click");
                          const ref: LoadoutRef = { kind: "custom", id: row.id };
                          if (col.side === "p1") p1Ref = ref; else p2Ref = ref;
                        }}
                      />
                      <span>{row.name}</span>
                    </label>
                  {/each}
                {/if}
              </fieldset>
            {/each}
          </div>
          {#if !bothReady}
            <p class="warn">{t("setup.preMadeLoadouts.notReadyWarning")}</p>
          {/if}
        {/if}
      {/if}
    </section>
  {/if}

  {#if !isMpJoiner}
    <div class="actions">
      <button
        class="primary"
        onclick={start}
        disabled={draftMode === "preMade" && !bothReady}
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
  .raterNote {
    font-size: 0.82em;
    font-style: italic;
    color: var(--paper-ink-soft);
    margin: 0.6em 0 0;
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
  .sidePickers {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.7em;
    margin-top: 0.7em;
  }
  @media (max-width: 640px) {
    .sidePickers { grid-template-columns: 1fr; }
  }
  .preMadePicker.side.p1 legend { color: var(--p1, #2b4a8a); }
  .preMadePicker.side.p2 legend { color: var(--p2, #a13a2a); }
  .divider {
    margin: 0.4em 0 0.2em;
    font-size: 0.85em;
    color: var(--paper-ink-soft);
    text-align: center;
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
