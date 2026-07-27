<script lang="ts">
  // Banner shown in /match/ during multiplayer when the connectivity pill goes
  // 🔴 (disconnected). Renders a 5-minute countdown anchored to
  // `mpState.lastPongAt + GRACE_MS`. Once the countdown hits 0, the
  // "Claim win" button activates and clicking it finalises the match as an
  // opponent forfeit.
  //
  // The component is a no-op outside multiplayer; the caller should still
  // gate the mount on `match.mode === "multiplayer"` so it never appears in
  // local play.

  import { onDestroy } from "svelte";
  import { mpState, pillState } from "$lib/multiplayer.svelte";
  import { GRACE_MS, TAKEOVER_MS } from "$lib/multiplayer-protocol-v2";
  import { t } from "$lib/state/i18n";
  import type { EngineClient } from "$lib/engine";
  import type { MpEngineHandle } from "$lib/multiplayer-engine";

  // Banner is presentational: the route supplies the `role`/`code` identity
  // and the two action policies (`onClaim`, `onTakeOver`). The banner owns
  // ONLY the connectivity-derived rendering (pill state, countdowns) - that
  // is intrinsic to its purpose and is sourced from `mpState`.
  let {
    eng,
    mpEngine = null,
    role,
    code,
    onClaim,
    onTakeOver,
  }: {
    eng: EngineClient | null;
    mpEngine?: MpEngineHandle | null;
    role: "host" | "joiner" | null;
    code: string | null;
    onClaim: (eng: EngineClient) => Promise<void>;
    onTakeOver: (args: { eng: EngineClient; mpEngine: MpEngineHandle; code: string }) => Promise<{ ok: boolean }>;
  } = $props();

  // Coarse 500ms ticker so the countdown text updates without leaning on the
  // mpState now-timer (which is private to multiplayer.svelte.ts).
  let nowTick = $state(Date.now());
  const timer = setInterval(() => (nowTick = Date.now()), 500);
  onDestroy(() => clearInterval(timer));

  const pill = $derived(pillState());
  // Only show the banner once a peer has actually been present at some point
  // this session (peerEverPaired latches on first inbound traffic and is
  // cleared only by disconnect()). This suppresses the spurious banner shown
  // during the host's resume-rehost window, where `mpState.status ===
  // "hosting"` makes the pill report "disconnected" before the joiner has
  // ever dialled - while still showing the banner on every real drop,
  // including drops where lastPongAt has been reset to null.
  const visible = $derived(
    (pill === "disconnected" || pill === "forfeit") && mpState.peerEverPaired
  );

  const deadline = $derived(
    mpState.disconnectedSince !== null
      ? mpState.disconnectedSince + GRACE_MS
      : null
  );
  const remainingMs = $derived(
    deadline === null ? GRACE_MS : Math.max(0, deadline - nowTick)
  );
  const canClaim = $derived(remainingMs === 0);

  // Takeover countdown - runs in parallel to the claim-win countdown but
  // unlocks earlier (30s vs 5min). Only the joiner sees the takeover CTA;
  // host has the claim-win path instead.
  const takeoverDeadline = $derived(
    mpState.disconnectedSince !== null
      ? mpState.disconnectedSince + TAKEOVER_MS
      : null
  );
  const takeoverEligibleMs = $derived(
    takeoverDeadline === null ? TAKEOVER_MS : Math.max(0, takeoverDeadline - nowTick)
  );
  const canTakeOver = $derived(
    takeoverEligibleMs === 0 && role === "joiner"
  );
  const takeoverLabel = $derived.by(() => {
    const total = Math.ceil(takeoverEligibleMs / 1000);
    const mm = Math.floor(total / 60).toString().padStart(2, "0");
    const ss = (total % 60).toString().padStart(2, "0");
    return `${mm}:${ss}`;
  });

  // mm:ss countdown for the not-yet-ready state.
  const remainingLabel = $derived.by(() => {
    const total = Math.ceil(remainingMs / 1000);
    const mm = Math.floor(total / 60).toString().padStart(2, "0");
    const ss = (total % 60).toString().padStart(2, "0");
    return `${mm}:${ss}`;
  });

  // Joiner-side auto-redial telemetry. When a peer drop fires the banner, the
  // transport is already retrying in the background (bounded ladder up to ~52s,
  // then indefinite ~30s long-tail). Surface "next attempt in Xs" so the player
  // doesn't think the client is dead.
  const redialSecondsLeft = $derived(
    mpState.redial.nextAttemptAt === null
      ? null
      : Math.max(0, Math.ceil((mpState.redial.nextAttemptAt - nowTick) / 1000))
  );
  const redialLabel = $derived.by(() => {
    if (redialSecondsLeft === null) return null;
    const secs = `${redialSecondsLeft}s`;
    if (mpState.redial.mode === "ladder") {
      return t("multiplayer.redialLadder", {
        attempt: String(mpState.redial.attempt),
        time: secs,
      });
    }
    if (mpState.redial.mode === "longtail") {
      return t("multiplayer.redialLongtail", { time: secs });
    }
    return null;
  });

  let busy = $state(false);
  async function handleClaim(): Promise<void> {
    if (!canClaim || !eng || busy) return;
    busy = true;
    try {
      await onClaim(eng);
    } finally {
      busy = false;
    }
  }

  let busyTakeover = $state(false);
  let takeoverError = $state<string | null>(null);
  async function handleTakeOver(): Promise<void> {
    if (!canTakeOver || !eng || !mpEngine || busyTakeover) return;
    if (!code) return;
    busyTakeover = true;
    takeoverError = null;
    try {
      const r = await onTakeOver({ eng, mpEngine, code });
      if (!r.ok) {
        takeoverError = t("multiplayer.takeOverFailed");
      }
    } catch {
      takeoverError = t("multiplayer.takeOverFailed");
    } finally {
      busyTakeover = false;
    }
  }
</script>

{#if visible}
  <div class="grace" role="status" aria-live="polite">
    <p class="msg">
      {role === "host"
        ? t("multiplayer.graceBannerHost")
        : t("multiplayer.graceBannerJoiner")}
    </p>
    {#if redialLabel}
      <p class="redial">{redialLabel}</p>
    {/if}
    <div class="actions">
      {#if canClaim}
        <button class="primary" type="button" disabled={!eng || busy} onclick={handleClaim}>
          {t("multiplayer.claimWinNow")}
        </button>
      {:else}
        <span class="countdown">{t("multiplayer.claimWinIn", { time: remainingLabel })}</span>
      {/if}
      {#if role === "joiner"}
        {#if canTakeOver}
          <button
            class="secondary"
            type="button"
            disabled={!eng || !mpEngine || busyTakeover}
            onclick={handleTakeOver}
          >
            {busyTakeover
              ? t("multiplayer.takeOverInProgress")
              : t("multiplayer.takeOverNow")}
          </button>
        {:else}
          <span class="countdown">{t("multiplayer.takeOverIn", { time: takeoverLabel })}</span>
        {/if}
      {/if}
    </div>
    {#if takeoverError}
      <p class="error" role="alert">{takeoverError}</p>
    {/if}
  </div>
{/if}

<style>
  .grace {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.8rem;
    flex-wrap: wrap;
    padding: 0.55em 0.9em;
    margin: 0 0 0.7rem;
    border: 1.5px solid #a94b3b;
    border-left: 4px solid #a94b3b;
    border-radius: 6px;
    background: var(--paper-bg);
  }
  .msg {
    margin: 0;
    font-weight: 600;
    color: #a94b3b;
  }
  .redial {
    margin: 0;
    font-size: 0.88em;
    color: var(--paper-ink-soft);
    font-variant-numeric: tabular-nums;
    flex-basis: 100%;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .countdown {
    font-variant-numeric: tabular-nums;
    color: var(--paper-ink-soft);
  }
  .primary {
    padding: 0.4em 0.85em;
    border: 1.5px solid currentColor;
    background: transparent;
    color: inherit;
    border-radius: 5px;
    cursor: pointer;
    font: inherit;
    font-weight: 600;
  }
  .primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .secondary {
    padding: 0.4em 0.85em;
    border: 1.5px solid var(--paper-ink-soft);
    background: transparent;
    color: var(--paper-ink);
    border-radius: 5px;
    cursor: pointer;
    font: inherit;
  }
  .secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    margin: 0.4rem 0 0;
    color: #a94b3b;
    font-size: 0.9em;
    flex-basis: 100%;
  }
</style>
