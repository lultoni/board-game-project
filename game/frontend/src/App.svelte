<script lang="ts">
  import Board from "./lib/Board.svelte";
  import { engine } from "./lib/engine";

  let version = $state("loading…");

  $effect(() => {
    engine.version().then((v) => (version = v));
  });
</script>

<main>
  <header>
    <h1>(GAME NAME)</h1>
    <small>engine v{version}</small>
  </header>
  <Board />
</main>

<style>
  main {
    font-family: "Caveat", "Georgia", serif;
    max-width: 720px;
    margin: 0 auto;
    padding: 1rem;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 1rem;
  }
  h1 {
    margin: 0;
  }
</style>
