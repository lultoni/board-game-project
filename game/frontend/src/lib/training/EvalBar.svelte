<script lang="ts">
  // Small horizontal centipawn bar from -BAR_CAP to +BAR_CAP. P1 is positive
  // (right). The zero tick marks the center for orientation. Used by
  // LiveMatchView to show challenger NN, defender NN, and heuristic scores
  // side-by-side.

  interface Props {
    label: string;
    value: number | null | undefined;
    barCap?: number;
  }
  let { label, value, barCap = 3000 }: Props = $props();

  const fraction = $derived.by(() => {
    if (value === null || value === undefined || !Number.isFinite(value)) return 0.5;
    const clamped = Math.max(-barCap, Math.min(barCap, value));
    return (clamped + barCap) / (2 * barCap);
  });
  const display = $derived.by(() => {
    if (value === null || value === undefined || !Number.isFinite(value)) return "—";
    return value >= 0 ? `+${value}` : `${value}`;
  });
</script>

<div class="evalBar" title={`${label}: ${display}`}>
  <span class="lbl">{label}</span>
  <div class="track">
    <div class="fill" style:width={`${fraction * 100}%`}></div>
    <div class="zero"></div>
  </div>
  <span class="val">{display}</span>
</div>

<style>
  .evalBar {
    display: grid;
    grid-template-columns: 8em 1fr 4em;
    gap: 0.5em;
    align-items: center;
  }
  .lbl {
    color: var(--paper-ink-soft);
    font-size: 0.92em;
  }
  .val {
    font-variant-numeric: tabular-nums;
    text-align: right;
    font-size: 0.92em;
  }
  .track {
    position: relative;
    height: 14px;
    background: var(--paper-bg);
    border: 1px solid var(--paper-line);
    border-radius: 3px;
    overflow: hidden;
  }
  .fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: linear-gradient(
      to right,
      var(--p2, #a13a2a) 0%,
      var(--p2, #a13a2a) 50%,
      var(--p1, #2b4a8a) 50%,
      var(--p1, #2b4a8a) 100%
    );
    background-size: 200% 100%;
    background-position: right;
    /* No fill animation — the polling cadence is already 4 Hz; an extra
       transition just lags the cell vs. the underlying number. */
  }
  .zero {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 1px;
    background: var(--paper-ink-soft);
    opacity: 0.6;
  }
</style>
