<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { getEngine } from "$lib/engine";
  import { rewriteFenWithLoadouts } from "$lib/engine/fen";
  import { SKILLS } from "$lib/engine/skills";
  import { t } from "$lib/state/i18n";
  import { match, resetMatchState } from "$lib/state/match-store.svelte";
  import {
    presetLoadout,
    mergeLoadouts,
    squareName,
    STACK_M_LOADOUT_SQUARES,
    type Loadout,
    type LoadoutMap,
    type PresetName,
  } from "$lib/state/draft";

  const mode = $derived($page.url.searchParams.get("mode") ?? "hvh");

  // King is index 0 in STACK_M_LOADOUT_SQUARES — labelled separately.
  const P1_SQUARES = STACK_M_LOADOUT_SQUARES.p1;
  const P2_SQUARES = STACK_M_LOADOUT_SQUARES.p2;

  let bootError = $state<string | null>(null);
  let starting = $state(false);
  let baseSnapshotJson = $state<string | null>(null);
  let baseFen = $state<string | null>(null);

  // Editable per-side maps. Initialised to all-empty loadouts; each row is
  // [s1, s2]. Using plain objects (not Map) keeps Svelte reactivity simple.
  let p1: Record<number, Loadout> = $state(
    Object.fromEntries(P1_SQUARES.map((sq) => [sq, [0, 0] as Loadout])),
  );
  let p2: Record<number, Loadout> = $state(
    Object.fromEntries(P2_SQUARES.map((sq) => [sq, [0, 0] as Loadout])),
  );

  const allSkillIds = Object.keys(SKILLS).map(Number).sort((a, b) => a - b);

  // Validation: every slot must have a non-zero skill id.
  const p1Filled = $derived(
    P1_SQUARES.every((sq) => p1[sq][0] > 0 && p1[sq][1] > 0),
  );
  const p2Filled = $derived(
    P2_SQUARES.every((sq) => p2[sq][0] > 0 && p2[sq][1] > 0),
  );
  const ready = $derived(!!baseSnapshotJson && p1Filled && p2Filled);

  onMount(async () => {
    try {
      resetMatchState();
      const eng = await getEngine();
      await eng.createEngine();
      baseFen = await eng.positionFen();
      baseSnapshotJson = await eng.snapshotJson();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
  });

  function applyPreset(side: "p1" | "p2", preset: PresetName): void {
    const squares = side === "p1" ? P1_SQUARES : P2_SQUARES;
    const seed = preset === "random" ? Date.now() & 0xffff : 1;
    const generated = presetLoadout(squares, preset, seed);
    const target = side === "p1" ? p1 : p2;
    for (const sq of squares) {
      const lo = generated.get(sq) ?? ([0, 0] as Loadout);
      target[sq] = [lo[0], lo[1]];
    }
  }

  function clear(side: "p1" | "p2"): void {
    const squares = side === "p1" ? P1_SQUARES : P2_SQUARES;
    const target = side === "p1" ? p1 : p2;
    for (const sq of squares) target[sq] = [0, 0];
  }

  function mirror(): void {
    // Copy P1's loadouts onto P2's pieces (King → King, Champion-i → Champion-i).
    for (let i = 0; i < P1_SQUARES.length; i++) {
      const src = p1[P1_SQUARES[i]];
      p2[P2_SQUARES[i]] = [src[0], src[1]];
    }
  }

  async function start(): Promise<void> {
    if (!ready || !baseSnapshotJson || !baseFen) return;
    starting = true;
    try {
      const merged: LoadoutMap = mergeLoadouts(
        new Map(P1_SQUARES.map((sq) => [sq, p1[sq]])),
        new Map(P2_SQUARES.map((sq) => [sq, p2[sq]])),
      );
      const newFen = rewriteFenWithLoadouts(baseFen, merged);
      const parsed = JSON.parse(baseSnapshotJson);
      parsed.start_fen = newFen;
      parsed.actions = [];
      const newSnap = JSON.stringify(parsed);

      // Verify by restoring on the live engine before navigating; if the
      // engine rejects the snapshot we'd rather surface the error here than
      // on the match route.
      const eng = await getEngine();
      await eng.restoreFromSnapshot(newSnap);

      match.pendingSnapshotJson = newSnap;
      match.mode = mode as typeof match.mode;
      await goto(`../match/?mode=${encodeURIComponent(mode)}`);
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
      starting = false;
    }
  }

  function skillLabel(id: number): string {
    if (id === 0) return "—";
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.name`) : `?${id}`;
  }
</script>

<main>
  <header>
    <p class="back"><a href="../">← back</a></p>
    <h1>{t("draft.title")}</h1>
    <small class="mode-tag">{mode}</small>
  </header>

  <p class="note">{t("draft.note")}</p>

  {#if bootError}
    <p class="err">boot error: {bootError}</p>
  {:else if !baseSnapshotJson}
    <p>{t("app.loading")}</p>
  {:else}
    <div class="cols">
      {#each [["p1", P1_SQUARES, p1] as const, ["p2", P2_SQUARES, p2] as const] as [side, squares, store]}
        <section class="col" class:p1={side === "p1"} class:p2={side === "p2"}>
          <h2>{side === "p1" ? "Player 1" : "Player 2"}</h2>
          <div class="presets">
            <button onclick={() => applyPreset(side, "aggro")}>{t("draft.preset.aggro")}</button>
            <button onclick={() => applyPreset(side, "defense")}>{t("draft.preset.defense")}</button>
            <button onclick={() => applyPreset(side, "combo")}>{t("draft.preset.combo")}</button>
            <button onclick={() => applyPreset(side, "random")}>{t("draft.preset.random")}</button>
            <button class="ghost" onclick={() => clear(side)}>{t("draft.clear")}</button>
          </div>
          <ul class="pieces">
            {#each squares as sq, i (sq)}
              {@const isKing = i === 0}
              <li class:king={isKing}>
                <span class="ptype">{isKing ? "King" : `Champion ${i}`}</span>
                <span class="psq">{squareName(sq)}</span>
                <select
                  bind:value={store[sq][0]}
                  aria-label="slot 1 for {squareName(sq)}"
                >
                  <option value={0}>—</option>
                  {#each allSkillIds as id}
                    <option value={id}>{skillLabel(id)}</option>
                  {/each}
                </select>
                <select
                  bind:value={store[sq][1]}
                  aria-label="slot 2 for {squareName(sq)}"
                >
                  <option value={0}>—</option>
                  {#each allSkillIds as id}
                    <option value={id}>{skillLabel(id)}</option>
                  {/each}
                </select>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>

    <div class="bottom">
      <button class="ghost" onclick={mirror}>{t("draft.mirror")}</button>
      <button
        class="primary"
        disabled={!ready || starting}
        onclick={start}
      >{starting ? t("app.loading") : t("draft.start")}</button>
    </div>
  {/if}
</main>

<style>
  main {
    max-width: 1100px;
    margin: 0 auto;
    padding: 0.6rem 1rem 2rem;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.4rem;
  }
  header h1 { font-size: 1.6rem; margin: 0; }
  .back a { text-decoration: none; }
  .mode-tag {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    padding: 0.05em 0.5em;
    color: var(--paper-ink-soft);
    font-size: 0.85rem;
  }
  .note {
    color: var(--paper-ink-soft);
    border-left: 3px solid var(--paper-line-strong);
    padding-left: 0.7rem;
    margin: 0 0 1rem;
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }
  @media (max-width: 720px) {
    .cols { grid-template-columns: 1fr; }
  }
  .col {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.8rem 1rem;
    background: var(--paper-bg);
  }
  .col.p1 { border-top: 4px solid var(--p1); }
  .col.p2 { border-top: 4px solid var(--p2); }
  .col h2 { margin: 0 0 0.5rem; font-size: 1.15rem; }
  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.6rem;
  }
  .presets button { font-size: 0.85rem; padding: 0.25em 0.65em; }
  .ghost { background: transparent; }
  .pieces {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 0.35rem;
  }
  .pieces li {
    display: grid;
    grid-template-columns: 7em 3em 1fr 1fr;
    gap: 0.4rem;
    align-items: center;
    padding: 0.2rem 0.3rem;
    border-bottom: 1px dashed var(--paper-line);
  }
  .pieces li.king { font-weight: 600; }
  .ptype { font-size: 0.9rem; }
  .psq { color: var(--paper-ink-soft); font-size: 0.85rem; }
  select { font: inherit; padding: 0.15em 0.3em; }
  .bottom {
    display: flex;
    justify-content: space-between;
    margin-top: 1.2rem;
  }
  .primary {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
    font-weight: 600;
    padding: 0.5em 1.4em;
  }
  .primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
  }
</style>
