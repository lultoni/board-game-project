<script lang="ts">
  // Uniform "what are we waiting on?" strip that mounts on /setup/, /draft/,
  // and /match/. Reads mpState + pillState() + route-supplied waitingReason
  // and paused flag. Rendering priority (highest first):
  //   1. forfeit  → suppressed (GraceBanner owns that state)
  //   2. disconnected + redialing  → "Reconnecting (attempt N, next try in Xs)"
  //   3. disconnected + not redialing  → "Connection lost. Retrying…"
  //   4. paused (host only)  → "Waiting for opponent to return."
  //   5. unstable  → "Connection slow." (non-blocking)
  //   6. connected + waitingReason  → route-supplied ("waiting for Player 2…")
  //   7. connected + no reason  → hidden.

  import { onDestroy } from "svelte";
  import { mpState, pillState } from "$lib/multiplayer.svelte";
  import { t } from "$lib/state/i18n";

  let {
    waitingReason = null,
    paused = false,
  }: {
    /** Route-specific "we're waiting on the peer" hint, rendered only when
     *  the connection is healthy. Null when the current player has agency. */
    waitingReason?: string | null;
    /** Host-only: true while the wrapper is pausing local actions because
     *  the joiner is disconnected. Ignored on joiner and in /setup/. */
    paused?: boolean;
  } = $props();

  let nowTick = $state(Date.now());
  const timer = setInterval(() => (nowTick = Date.now()), 500);
  onDestroy(() => clearInterval(timer));

  const pill = $derived(pillState());

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

  type Strip =
    | { kind: "disconnected"; text: string; tone: "warn" }
    | { kind: "paused"; text: string; tone: "warn" }
    | { kind: "unstable"; text: string; tone: "info" }
    | { kind: "waiting"; text: string; tone: "info" }
    | null;

  const strip = $derived.by<Strip>(() => {
    // 1. Forfeit - hand off to GraceBanner. Strip stays hidden so the two
    //    banners don't stack redundantly.
    if (pill === "forfeit") return null;

    // 2/3. Disconnected. Prefer redial telemetry; fall back to brief text.
    if (pill === "disconnected") {
      if (redialLabel) {
        return { kind: "disconnected", text: redialLabel, tone: "warn" };
      }
      return {
        kind: "disconnected",
        text: t("multiplayer.connectionLostBrief"),
        tone: "warn",
      };
    }

    // 4. Paused (host-only). Only meaningful when the connection is otherwise
    //    up - a full disconnect already renders state 2/3.
    if (paused) {
      return { kind: "paused", text: t("multiplayer.paused"), tone: "warn" };
    }

    // 5. Unstable. Non-blocking hint.
    if (pill === "unstable") {
      return { kind: "unstable", text: t("multiplayer.unstable"), tone: "info" };
    }

    // 6. Healthy connection + route says we're waiting on the peer.
    if (pill === "live" && waitingReason) {
      return { kind: "waiting", text: waitingReason, tone: "info" };
    }

    // 7. Nothing to say.
    return null;
  });
</script>

{#if strip}
  <div class="strip" data-tone={strip.tone} role="status" aria-live="polite">
    <span class="dot" data-tone={strip.tone} aria-hidden="true"></span>
    <span class="text">{strip.text}</span>
  </div>
{/if}

<style>
  .strip {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.4em 0.75em;
    margin: 0 0 0.6rem;
    border-radius: 5px;
    border: 1px solid var(--paper-ink-soft);
    background: var(--paper-bg);
    font-size: 0.92em;
  }
  .strip[data-tone="warn"] {
    border-color: #a94b3b;
    color: #a94b3b;
  }
  .strip[data-tone="info"] {
    color: var(--paper-ink-soft);
  }
  .dot {
    width: 0.55em;
    height: 0.55em;
    border-radius: 50%;
    background: currentColor;
    flex-shrink: 0;
  }
  .text {
    font-variant-numeric: tabular-nums;
  }
</style>
