<script lang="ts">
  import { goto } from "$app/navigation";
  import { t } from "$lib/state/i18n";
  import { match, type SeatKind, type DraftMode, type PreMadeLoadoutId } from "$lib/state/match-store.svelte";
  import { settings } from "$lib/state/settings.svelte";
  import { isPreMadeLoadoutReady } from "$lib/state/draft";

  const isMultiplayer = $derived(match.mode === "multiplayer");

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
    if (isMultiplayer) {
      // Seats are forced human in multiplayer; nothing to copy back.
      match.side = { p1: "human", p2: "human" };
    } else {
      match.side = { p1, p2 };
    }
    match.draftMode = draftMode;
    match.preMadeLoadoutId = draftMode === "preMade" ? preMadeId : null;
    if (draftMode === "preMade") {
      // Skip the /draft/ route entirely — /match/ reads preMadeLoadoutId and
      // builds the engine with both sides preloaded.
      await goto("../match/");
    } else {
      await goto("../draft/");
    }
  }
</script>

<main>
  <header>
    <p class="back"><a href="../">{t("setup.back")}</a></p>
    <h1>{t("setup.title")}</h1>
  </header>

  {#if isMultiplayer}
    <p class="banner">{t("multiplayer.sessionFor", { n: 1 })}</p>
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
              onchange={() => (seat.id === "p1" ? (p1 = "human") : (p2 = "human"))}
            />
            {t("setup.human")}
          </label>
          <label>
            <input
              type="radio"
              name={seat.id}
              value="ai"
              checked={(seat.id === "p1" ? p1 : p2) === "ai"}
              onchange={() => (seat.id === "p1" ? (p1 = "ai") : (p2 = "ai"))}
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
            />
            <output>{settings.p1MaxDepth}</output>
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
            />
            <output>{settings.p2MaxDepth}</output>
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
            />
            <output>{settings.aivaiStepDelayMs}</output>
          </label>
        {/if}
      </div>
    </section>
  {/if}

  {#if !isMultiplayer}
    <section class="draftMode">
      <h2>{t("setup.draftMode.header")}</h2>
      <div class="modes">
        <label>
          <input
            type="radio"
            name="draftMode"
            value="custom"
            checked={draftMode === "custom"}
            onchange={() => (draftMode = "custom")}
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
            onchange={() => (draftMode = "preMade")}
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
                onchange={() => (preMadeId = opt.id)}
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

  <div class="actions">
    <button
      class="primary"
      onclick={start}
      disabled={draftMode === "preMade" && !preMadeReady && !isMultiplayer}
    >{t("setup.continue")}</button>
  </div>
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
