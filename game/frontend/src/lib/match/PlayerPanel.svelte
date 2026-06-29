<script lang="ts">
  import type { PositionView } from "$lib/engine";
  import { settings } from "$lib/state/settings.svelte";

  interface Props {
    player: "p1" | "p2";
    position: PositionView | null;
    /** True when the AI is currently thinking for this player's seat. */
    aiThinking: boolean;
    /** Depth reached by the last completed AI search (0 = none yet). */
    aiLastDepth: number;
    /** Whether this seat is controlled by an AI (drives indicator visibility). */
    isAiSeat: boolean;
  }

  let { player, position, aiThinking, aiLastDepth, isAiSeat }: Props = $props();

  function popcount(bb: bigint): number {
    let n = 0;
    let b = bb;
    while (b !== 0n) { b &= b - 1n; n++; }
    return n;
  }

  // bitboards layout: [p1, p2, kings, champions, guards]
  // "opp" = the side whose pieces this player has captured.
  const oppIdx = $derived(player === "p1" ? 1 : 0);

  // Remaining pieces of each type belonging to the opponent.
  const oppKingsAlive = $derived(!position ? 1 : popcount(position.bitboards[2] & position.bitboards[oppIdx]));
  const oppGuardsAlive = $derived(!position ? 0 : popcount(position.bitboards[4] & position.bitboards[oppIdx]));
  const oppChampsAlive = $derived(!position ? 5 : popcount(position.bitboards[oppIdx]) - oppKingsAlive - oppGuardsAlive);

  // Track maximum ever-seen counts so we know starting army size.
  // maxGuards stays at 0 for Stack M (no guards in default armies).
  let maxGuardsSeen = $state(0);
  $effect(() => { if (oppGuardsAlive > maxGuardsSeen) maxGuardsSeen = oppGuardsAlive; });

  // Captures = starting count minus alive. Kings always start at 1. Champs default 5.
  const capturedKings = $derived(Math.max(0, 1 - oppKingsAlive));
  const capturedChamps = $derived(!position ? 0 : Math.max(0, 5 - oppChampsAlive));
  const capturedGuards = $derived(Math.max(0, maxGuardsSeen - oppGuardsAlive));

  const money = $derived(
    position ? (player === "p1" ? position.p1Money : position.p2Money) : 0,
  );

  const color = $derived(player === "p1" ? "var(--p1, #4b6b8a)" : "var(--p2, #a94b3b)");
  const label = $derived(player === "p1" ? "Player 1" : "Player 2");
</script>

<div class="player-panel" class:p1={player === "p1"} class:p2={player === "p2"}>
  <div class="identity">
    <span class="colour-dot" style:background={color}></span>
    <span class="name">{label}</span>
    {#if isAiSeat && aiThinking}
      <span class="thinking-badge">
        <span class="spinner" aria-hidden="true"></span>
        <span class="thinking-text">thinking</span>
        {#if settings.showAiDepth && aiLastDepth > 0}
          <span class="depth">d{aiLastDepth}</span>
        {/if}
      </span>
    {/if}
  </div>

  <div class="stats">
    <div class="captures">
      <!-- King pip (circle) — 1 slot -->
      <span
        class="cap-pip king"
        class:taken={capturedKings > 0}
        style:--pip-color={color}
        title="King"
      ></span>
      <!-- Champion pips (diamonds) — 5 slots for Stack M default -->
      {#each { length: 5 } as _, i}
        <span
          class="cap-pip champ"
          class:taken={i < capturedChamps}
          style:--pip-color={color}
          title="Champion"
        ></span>
      {/each}
      <!-- Guard pips (squares) — only shown when guards present in army -->
      {#each { length: maxGuardsSeen } as _, i}
        <span
          class="cap-pip guard"
          class:taken={i < capturedGuards}
          style:--pip-color={color}
          title="Guard"
        ></span>
      {/each}
    </div>
    <span class="money">${money}</span>
  </div>
</div>

<style>
  .player-panel {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.8rem;
    padding: 0.35rem 0.5rem;
    border-radius: 5px;
    background: var(--paper-bg, #f3ecd9);
    border: 1px solid var(--paper-line, rgba(58,47,31,0.15));
  }

  .identity {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .colour-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .name {
    font-size: 0.85rem;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .thinking-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.75rem;
    color: var(--paper-ink-soft, #6a6055);
    padding: 0.1em 0.5em;
    border-radius: 3px;
    background: var(--paper-square-light, #ece2c8);
  }

  .thinking-text {
    font-style: italic;
  }

  .depth {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--paper-ink, #3a2f1f);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 0.7rem;
  }

  .captures {
    display: flex;
    gap: 3px;
    align-items: center;
  }

  .cap-pip {
    display: inline-block;
    width: 9px;
    height: 9px;
    border: 1.5px solid var(--pip-color, currentColor);
    transition: background 200ms, opacity 200ms;
    opacity: 0.25;
    flex-shrink: 0;
  }
  .cap-pip.taken {
    background: var(--pip-color, currentColor);
    opacity: 1;
  }

  /* King: full circle */
  .cap-pip.king {
    border-radius: 50%;
  }
  /* Champion: rotated square (diamond) */
  .cap-pip.champ {
    border-radius: 1px;
    transform: rotate(45deg);
    width: 7px;
    height: 7px;
  }
  /* Guard: small square with rounded corners */
  .cap-pip.guard {
    border-radius: 2px;
  }

  .money {
    font-size: 0.85rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--paper-ink-soft, #6a6055);
    min-width: 2.5ch;
    text-align: right;
  }

  /* Spinner */
  .spinner {
    display: inline-block;
    width: 0.7em;
    height: 0.7em;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    opacity: 0.7;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
