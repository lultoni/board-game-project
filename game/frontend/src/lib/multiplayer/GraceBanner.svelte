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
  import { GRACE_MS } from "$lib/multiplayer-protocol";
  import { t } from "$lib/state/i18n";
  import { match, claimWinByOpponentForfeit } from "$lib/state/match-store.svelte";
  import type { EngineClient } from "$lib/engine/types";

  // Engine handle passed from /match/ so we can finalise on claim. Kept
  // optional so the component still renders during boot before eng resolves
  // — the button stays disabled until eng is wired.
  let { eng }: { eng: EngineClient | null } = $props();

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
  // ever dialled — while still showing the banner on every real drop,
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

  // mm:ss countdown for the not-yet-ready state.
  const remainingLabel = $derived.by(() => {
    const total = Math.ceil(remainingMs / 1000);
    const mm = Math.floor(total / 60).toString().padStart(2, "0");
    const ss = (total % 60).toString().padStart(2, "0");
    return `${mm}:${ss}`;
  });

  let busy = $state(false);
  async function onClaim(): Promise<void> {
    if (!canClaim || !eng || busy) return;
    busy = true;
    try {
      await claimWinByOpponentForfeit(eng);
    } finally {
      busy = false;
    }
  }
</script>

{#if visible}
  <div class="grace" role="status" aria-live="polite">
    <p class="msg">
      {match.multiplayerRole === "host"
        ? t("multiplayer.graceBannerHost")
        : t("multiplayer.graceBannerJoiner")}
    </p>
    <div class="actions">
      {#if canClaim}
        <button class="primary" type="button" disabled={!eng || busy} onclick={onClaim}>
          {t("multiplayer.claimWinNow")}
        </button>
      {:else}
        <span class="countdown">{t("multiplayer.claimWinIn", { time: remainingLabel })}</span>
      {/if}
    </div>
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
</style>
