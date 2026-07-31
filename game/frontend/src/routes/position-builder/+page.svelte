<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { getEngine, type EngineClient, type PositionView } from "$lib/engine";
  import { SKILLS } from "$lib/engine/skills";
  import { t } from "$lib/state/i18n";
  import { buildEngineConfigJson, match, modeFromSeats } from "$lib/state/match-store.svelte";
  import { createPlyRenderer, type PlyRenderer } from "$lib/board/ply-renderer.svelte";
  import { sfx } from "$lib/audio/sfx";
  import Board from "$lib/board/Board.svelte";
  import { consumePendingPositionFen } from "$lib/storage/position-handoff";
  import {
    parseBoardSection,
    parseToken,
    buildToken,
    mutateBoardToStaticFen,
  } from "$lib/state/position-fen";

  let eng = $state<EngineClient | null>(null);
  let renderer = $state<PlyRenderer | null>(null);
  let position = $state<PositionView | null>(null);
  let error = $state<string | null>(null);
  let launching = $state(false);
  // Live FEN of the current position, refreshed after every load/edit so the
  // user always sees the canonical string without having to hit "Copy".
  let liveFen = $state("");
  let fenCopied = $state(false);

  async function refreshLiveFen(): Promise<void> {
    if (!eng) return;
    try { liveFen = await eng.positionFen(); } catch { /* noop */ }
  }

  // Side config for launching a game from this position.
  let p1Kind = $state<"human" | "ai">("human");
  let p2Kind = $state<"human" | "ai">("human");

  // --- Piece editing state ---
  interface PieceEdit {
    square: number;
    char: string;      // 'K'|'C'|'G'|'k'|'c'|'g'
    hp: number;        // 1-2
    armor: number;     // 0-2
    s1: number;
    s2: number;
  }
  let editingPiece = $state<PieceEdit | null>(null);
  // All occupied squares (for selectable/draggable)
  const allPieceSquares = $derived.by((): Set<number> => {
    if (!position) return new Set();
    const out = new Set<number>();
    const [p1bb, p2bb] = position.bitboards;
    const occ = p1bb | p2bb;
    for (let sq = 0; sq < 64; sq++) {
      if ((occ >> BigInt(sq)) & 1n) out.add(sq);
    }
    return out;
  });

  onMount(async () => {
    try {
      eng = await getEngine();
      renderer = createPlyRenderer(eng, {
        onStateUpdate: (pos) => { position = pos; },
        sfxEnabled: false,
      });
      const pendingFen = consumePendingPositionFen();
      if (pendingFen) {
        await loadFen(pendingFen);
      } else {
        await eng.createEngine();
        await renderer.resyncFromEngine();
        await refreshLiveFen();
      }
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  });

  onDestroy(() => { renderer = null; });

  async function loadFen(fen: string): Promise<void> {
    if (!eng || !renderer) return;
    try {
      const configJson = buildEngineConfigJson({ p1: p1Kind, p2: p2Kind });
      const configObj = JSON.parse(configJson);
      const snap = JSON.stringify({ start_fen: fen, actions: [], config: configObj });
      await eng.restoreFromSnapshot(snap);
      await renderer.resyncFromEngine();
      await refreshLiveFen();
      error = null;
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  async function copyFen(): Promise<void> {
    if (!eng) return;
    try {
      const fen = liveFen || (await eng.positionFen());
      await navigator.clipboard.writeText(fen);
      fenCopied = true;
      setTimeout(() => (fenCopied = false), 1200);
    } catch { /* noop */ }
  }

  // --- FEN board manipulation ---
  // Pure parse/encode/mutate helpers live in $lib/state/position-fen (unit-
  // tested there; the engine is Tauri-only so the string logic can't be
  // exercised in a browser test).

  /** Apply a board mutation and reload the engine as a clean static position.
   *  `mutateBoardToStaticFen` zeroes `moved_this_phase` and drops turn-scoped
   *  trailer fields, so moving/removing any piece can't strand a moved bit and
   *  trigger BadDecimal{field:"moved_this_phase"} (which broke drag + edit). */
  async function applyBoardMutation(mutateFn: (squares: string[]) => void): Promise<void> {
    if (!eng || !renderer) return;
    try {
      const fen = await eng.positionFen();
      await loadFen(mutateBoardToStaticFen(fen, mutateFn));
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  // --- Board interaction handlers ---

  function handleSquareClick(square: number): void {
    if (!position) return;
    const [p1bb, p2bb] = position.bitboards;
    const occ = p1bb | p2bb;
    if (!((occ >> BigInt(square)) & 1n)) {
      // Empty square — dismiss any open editor
      editingPiece = null;
      return;
    }
    // Open edit menu for this piece
    void openEditMenu(square);
  }

  async function openEditMenu(square: number): Promise<void> {
    if (!eng) return;
    const fen = await eng.positionFen();
    const parts = fen.split(" ");
    const squares = parseBoardSection(parts[0]);
    const token = squares[square];
    if (!token) return;
    const info = parseToken(token);
    editingPiece = { square, ...info };
  }

  async function handlePieceDrop(src: number, _path: number[], _x: number, _y: number, tgt: number): Promise<void> {
    if (src === tgt) return;
    editingPiece = null;
    await applyBoardMutation((squares) => {
      squares[tgt] = squares[src];
      squares[src] = "";
    });
  }

  /** Skill options for the dropdowns: id 0 = empty slot, then every skill by
   *  name. Sorted by id so the list is stable. */
  const SKILL_OPTIONS = [
    { id: 0, name: "— none —" },
    ...Object.values(SKILLS)
      .sort((a, b) => a.id - b.id)
      .map((s) => ({ id: s.id, name: t(`skills.${s.key}.name`) })),
  ];

  /** Auto-apply the current editingPiece values to the board WITHOUT closing
   *  the panel (user tweaks HP/armor/skills and sees the board update live;
   *  the panel stays open until they click away or Cancel). */
  async function applyEdit(): Promise<void> {
    if (!editingPiece) return;
    const { square, char, hp, armor, s1, s2 } = editingPiece;
    await applyBoardMutation((squares) => {
      squares[square] = buildToken(char, hp, armor, s1, s2);
    });
    // Deliberately keep editingPiece open.
  }

  async function removePiece(): Promise<void> {
    if (!editingPiece) return;
    const sq = editingPiece.square;
    editingPiece = null;
    await applyBoardMutation((squares) => { squares[sq] = ""; });
  }

  async function launchGame(): Promise<void> {
    if (!eng || !renderer || launching) return;
    launching = true;
    editingPiece = null;
    sfx.play("wheelOpen");
    try {
      const configJson = buildEngineConfigJson({ p1: p1Kind, p2: p2Kind });
      const configObj = JSON.parse(configJson);
      const fen = await eng.positionFen();
      const freshSnap = JSON.stringify({ start_fen: fen, actions: [], config: configObj });
      await eng.restoreFromSnapshot(freshSnap);
      const snap = await eng.snapshotJson();
      match.pendingSnapshotJson = snap;
      match.side = { p1: p1Kind, p2: p2Kind };
      match.mode = modeFromSeats({ p1: p1Kind, p2: p2Kind });
      match.draftMode = "preMade";
      match.sideLoadouts = null;
      await goto("../match/");
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      launching = false;
    }
  }

  const PIECE_LABELS: Record<string, string> = {
    K: "P1 King", C: "P1 Champion", G: "P1 Guard",
    k: "P2 King", c: "P2 Champion", g: "P2 Guard",
  };
</script>

<main>
  <header>
    <h1>Position Builder</h1>
  </header>

  {#if error}
    <p class="err">{error}</p>
  {/if}

  <div class="layout">
    <div class="board-col">
      {#if renderer && position}
        <div class="board-wrap">
          <Board
            position={position}
            pieceIds={renderer.pieceIds}
            interactive={true}
            selectable={allPieceSquares}
            draggable={allPieceSquares}
            clickPieceOnTap={true}
            onSquareClick={(sq) => handleSquareClick(sq)}
            onPieceDrop={(src, path, x, y) => {
              const tgt = path[path.length - 1] ?? src;
              void handlePieceDrop(src, path, x, y, tgt);
            }}
          />
        </div>
        <p class="board-hint">Click a piece to edit · Drag to reposition</p>
      {:else}
        <div class="board-placeholder">Loading…</div>
      {/if}
    </div>

    <aside class="controls">
      {#if editingPiece}
        <section class="control-group edit-panel">
          <h2>{PIECE_LABELS[editingPiece.char] ?? editingPiece.char}</h2>

          <div class="edit-field">
            <span class="edit-label">HP</span>
            <div class="segmented">
              {#each [1, 2] as v}
                <button
                  type="button"
                  class:active={editingPiece.hp === v}
                  onclick={() => { editingPiece.hp = v; void applyEdit(); }}
                >{v}</button>
              {/each}
            </div>
          </div>

          <div class="edit-field">
            <span class="edit-label">Armor</span>
            <div class="segmented">
              {#each [0, 1, 2] as v}
                <button
                  type="button"
                  class:active={editingPiece.armor === v}
                  onclick={() => { editingPiece.armor = v; void applyEdit(); }}
                >{v}</button>
              {/each}
            </div>
          </div>

          {#if "CcKk".includes(editingPiece.char)}
            <div class="edit-field">
              <span class="edit-label">Skill 1</span>
              <select bind:value={editingPiece.s1} onchange={() => void applyEdit()}>
                {#each SKILL_OPTIONS as opt}
                  <option value={opt.id}>{opt.name}</option>
                {/each}
              </select>
            </div>
            <div class="edit-field">
              <span class="edit-label">Skill 2</span>
              <select bind:value={editingPiece.s2} onchange={() => void applyEdit()}>
                {#each SKILL_OPTIONS as opt}
                  <option value={opt.id}>{opt.name}</option>
                {/each}
              </select>
            </div>
          {/if}

          <div class="edit-btns">
            <button type="button" class="btn-danger" onclick={() => void removePiece()}>Remove</button>
            <button type="button" onclick={() => { editingPiece = null; }}>Close</button>
          </div>
        </section>
      {/if}

      <section class="control-group">
        <h2>Current FEN</h2>
        <textarea
          class="fen-area"
          rows="4"
          bind:value={liveFen}
          spellcheck="false"
          placeholder="FEN string — edit and Load, or paste your own…"
        ></textarea>
        <div class="fen-btns">
          <button type="button" onclick={() => void loadFen(liveFen.trim())}>Load</button>
          <button type="button" class="secondary-btn" onclick={() => void copyFen()}>{fenCopied ? "Copied ✓" : "Copy"}</button>
        </div>
      </section>

      <section class="control-group">
        <h2>Launch game</h2>
        <div class="seat-row">
          <label>P1:
            <select bind:value={p1Kind}>
              <option value="human">Human</option>
              <option value="ai">AI</option>
            </select>
          </label>
          <label>P2:
            <select bind:value={p2Kind}>
              <option value="human">Human</option>
              <option value="ai">AI</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          class="launch-btn"
          disabled={launching || !position}
          onclick={() => void launchGame()}
        >{launching ? "Launching…" : "▶ Start Game from Position"}</button>
      </section>
    </aside>
  </div>
</main>

<style>
  main {
    max-width: 1100px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
  }
  header {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
  }
  h1 { font-size: 1.8rem; margin: 0; }
  .layout {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 1.5rem;
    align-items: start;
  }
  @media (max-width: 700px) { .layout { grid-template-columns: 1fr; } }
  .board-col { display: flex; flex-direction: column; gap: 0.5rem; }
  .board-wrap {
    width: 100%;
    aspect-ratio: 1;
    max-width: 560px;
  }
  .board-hint {
    font-size: 0.8rem;
    color: var(--paper-ink-soft);
    margin: 0;
    font-style: italic;
  }
  .board-placeholder {
    width: 100%;
    aspect-ratio: 1;
    background: var(--paper-square-light);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--paper-ink-soft);
    font-style: italic;
  }
  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .control-group {
    padding: 0.8em 1em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    background: var(--paper-bg);
  }
  .control-group h2 {
    margin: 0 0 0.6em;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--paper-ink-soft);
  }
  .edit-panel { border-color: var(--accent, #c79b3a); }
  .edit-panel h2 { color: var(--accent, #c79b3a); }
  .edit-field {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-bottom: 0.5rem;
  }
  .edit-label {
    flex: 0 0 3.2rem;
    font-size: 0.8rem;
    color: var(--paper-ink-soft);
  }
  .edit-field select {
    flex: 1;
    padding: 0.3em 0.4em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    background: var(--paper-bg);
    font: inherit;
    font-size: 0.85rem;
  }
  .segmented {
    display: flex;
    flex: 1;
    gap: 0;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    overflow: hidden;
  }
  .segmented button {
    flex: 1;
    padding: 0.3em 0;
    border: none;
    border-right: 1.5px solid var(--paper-line-strong);
    background: var(--paper-bg);
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .segmented button:last-child { border-right: none; }
  .segmented button.active {
    background: var(--accent, #c79b3a);
    color: #fff;
    font-weight: 600;
  }
  .edit-btns {
    display: flex;
    gap: 0.4rem;
    margin-top: 0.5rem;
    flex-wrap: wrap;
  }
  .edit-btns button { flex: 1; font-size: 0.82rem; padding: 0.35em 0.5em; }
  .btn-danger {
    border-color: #a94b3b;
    color: #a94b3b;
  }
  .fen-area {
    width: 100%;
    box-sizing: border-box;
    padding: 0.4em 0.5em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.72rem;
    line-height: 1.35;
    resize: vertical;
    margin-bottom: 0.4rem;
    word-break: break-all;
  }
  .fen-btns { display: flex; gap: 0.4rem; margin-bottom: 0.6rem; }
  .fen-btns button { flex: 1; }
  .seat-row { display: flex; gap: 1rem; margin-bottom: 0.6rem; }
  .seat-row label { font-size: 0.9rem; display: flex; align-items: center; gap: 0.3rem; }
  .seat-row select {
    padding: 0.25em 0.4em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 4px;
    background: var(--paper-bg);
    font: inherit;
  }
  button {
    padding: 0.45em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    cursor: pointer;
    font: inherit;
    font-size: 0.88rem;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .secondary-btn { font-size: 0.82rem; width: 100%; }
  .launch-btn {
    width: 100%;
    background: var(--accent, #c79b3a);
    border-color: var(--accent, #c79b3a);
    color: #fff;
    font-weight: 600;
    font-size: 0.95rem;
    padding: 0.55em 0.9em;
  }
  .launch-btn:not(:disabled):hover { opacity: 0.9; }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
    margin-bottom: 1rem;
  }
</style>
