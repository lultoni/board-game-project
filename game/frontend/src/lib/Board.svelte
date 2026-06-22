<script lang="ts">
  // 8×8 board rendered as CSS Grid. Per ADR-005 Layer 6: no canvas; HTML divs.
  // Pieces will be round (paper-aesthetic) and styled via scoped CSS.

  // Square indexing matches core_engine: i = rank * 8 + file, rank 0 = bottom.
  const squares = Array.from({ length: 64 }, (_, i) => i);

  function onPointerDown(_sq: number) {
    // TODO: wire to engine.legalActions(sq) → highlight valid targets.
  }
</script>

<div class="board" role="grid" aria-label="game board">
  {#each squares as sq (sq)}
    {@const rank = Math.floor(sq / 8)}
    {@const file = sq % 8}
    {@const dark = (rank + file) % 2 === 1}
    <div
      class="sq"
      class:dark
      role="gridcell"
      tabindex="0"
      onpointerdown={() => onPointerDown(sq)}
    ></div>
  {/each}
</div>

<style>
  .board {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    aspect-ratio: 1 / 1;
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    border: 2px solid #3a2f1f;
    /* Paper-aesthetic placeholder — swap for real texture asset later. */
    background: #f4ecd8;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  .sq {
    border: 1px solid rgba(58, 47, 31, 0.15);
    touch-action: manipulation;
  }
  .sq.dark {
    background: rgba(58, 47, 31, 0.08);
  }
  .sq:focus {
    outline: 2px solid #c79b3a;
    outline-offset: -2px;
  }
</style>
