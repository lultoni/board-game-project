<script lang="ts">
  // Custom loadout manager route.
  //
  // Left column: list of saved loadouts (with rename/delete + a 12-skill preview
  // strip). Right column: editor pane — LoadoutBoard on top, SkillPicker in
  // "click" mode below for whichever piece is currently selected.
  //
  // Persistence goes through `getTelemetryStore()` (IDB v3 loadouts store).
  // Save is guarded by findDuplicate (see 8h) so the same skill tuple can't be
  // saved twice under different names.

  import { onMount } from "svelte";
  import { t } from "$lib/state/i18n";
  import { sfx } from "$lib/audio/sfx";
  import { getTelemetryStore, newMatchId } from "$lib/storage";
  import type { SavedLoadout } from "$lib/storage/types";
  import type { Owner, SideLoadout } from "$lib/engine";
  import LoadoutBoard from "$lib/board/LoadoutBoard.svelte";
  import SkillPicker from "$lib/board/SkillPicker.svelte";
  import SkillGlyphDefs from "$lib/board/SkillGlyphDefs.svelte";
  import { findDuplicate, loadoutKey } from "$lib/storage/loadout-dedupe";
  import {
    encodeJson,
    encodeShareCode,
    parseImport,
  } from "$lib/storage/loadout-codec";

  const EMPTY: SideLoadout = [
    [0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0],
  ] as const;

  // --- state ---------------------------------------------------------------

  let rows = $state<SavedLoadout[]>([]);
  let loading = $state(true);

  /** ID of the currently-selected saved loadout, or `"__new__"` for the
   *  new-loadout draft, or `null` when nothing is being edited. */
  let editingId = $state<string | null>(null);

  /** Working copy of the loadout being edited. Committed to IDB on Save. */
  let draftLoadout = $state<SideLoadout>(cloneLoadout(EMPTY));
  let draftName = $state("");
  let orientation = $state<Owner>("p1");

  /** Which piece (0..5) has focus in the editor. Null = no piece selected. */
  let selectedPieceIdx = $state<number | null>(null);
  /** Which of the two slots on the selected piece the picker will write to. */
  let editingSlot = $state<0 | 1>(0);

  let confirmDeleteId = $state<string | null>(null);
  let renameTargetId = $state<string | null>(null);
  let renameDraft = $state("");

  let copiedFlashId = $state<string | null>(null);

  let importText = $state("");
  let importReport = $state<string | null>(null);

  function cloneLoadout(l: SideLoadout): SideLoadout {
    return l.map((p) => [p[0], p[1]] as [number, number]) as unknown as SideLoadout;
  }

  // Mutable-in-place view of a cloned loadout so we can write slot values
  // without fighting the readonly-tuple types. Callers always work on a
  // fresh clone and then hand the result back to `$state` reactivity.
  type MutableLoadout = [
    [number, number], [number, number], [number, number],
    [number, number], [number, number], [number, number],
  ];
  function mutClone(l: SideLoadout): MutableLoadout {
    return l.map((p) => [p[0], p[1]] as [number, number]) as MutableLoadout;
  }

  // --- lifecycle -----------------------------------------------------------

  async function refresh(): Promise<void> {
    rows = await getTelemetryStore().listLoadouts();
  }

  onMount(async () => {
    try {
      await refresh();
    } finally {
      loading = false;
    }
  });

  // --- editor actions ------------------------------------------------------

  function startNew(): void {
    sfx.play("click");
    editingId = "__new__";
    draftLoadout = cloneLoadout(EMPTY);
    draftName = "";
    selectedPieceIdx = 0;
    editingSlot = 0;
  }

  function selectRow(row: SavedLoadout): void {
    sfx.play("click");
    editingId = row.id;
    draftLoadout = cloneLoadout(row.loadout);
    draftName = row.name;
    selectedPieceIdx = 0;
    editingSlot = 0;
  }

  function onPieceClick(idx: number): void {
    sfx.play("tick");
    if (selectedPieceIdx === idx) {
      // Toggle slot when clicking the already-selected piece so a single
      // piece click can flip between its two slots without touching a radio.
      editingSlot = editingSlot === 0 ? 1 : 0;
      return;
    }
    selectedPieceIdx = idx;
    editingSlot = 0;
  }

  function pickSkill(skillId: number): void {
    if (selectedPieceIdx === null) return;
    const idx = selectedPieceIdx;
    const next = mutClone(draftLoadout);
    next[idx][editingSlot] = skillId;
    draftLoadout = next as unknown as SideLoadout;
    sfx.play("draftPick");
    // Advance the slot cursor so a click-click flow fills both slots
    // without an extra tap on the radio.
    if (editingSlot === 0) editingSlot = 1;
  }

  function clearSlot(slotIdx: 0 | 1): void {
    if (selectedPieceIdx === null) return;
    const idx = selectedPieceIdx;
    const next = mutClone(draftLoadout);
    next[idx][slotIdx] = 0;
    draftLoadout = next as unknown as SideLoadout;
    sfx.play("click");
  }

  // --- save guards ---------------------------------------------------------

  const isComplete = $derived(
    draftLoadout.every((pair) => pair[0] !== 0 && pair[1] !== 0),
  );

  /** Rows to compare against when checking for duplicates. When editing an
   *  existing row, exclude its own ID so saving a no-op edit works. */
  const otherRows = $derived(
    editingId === "__new__" || editingId === null
      ? rows
      : rows.filter((r) => r.id !== editingId),
  );

  const duplicate = $derived(findDuplicate(draftLoadout, otherRows));

  const saveDisabled = $derived(
    editingId === null
    || draftName.trim().length === 0
    || !isComplete
    || duplicate !== null,
  );

  const saveDisabledReason = $derived.by(() => {
    if (editingId === null) return "";
    if (draftName.trim().length === 0) return t("loadouts.cannotSaveEmptyName");
    if (!isComplete) return t("loadouts.cannotSaveIncomplete");
    if (duplicate) return t("loadouts.cannotSaveDuplicate", { name: duplicate.name });
    return "";
  });

  async function save(): Promise<void> {
    if (saveDisabled) return;
    const name = draftName.trim();
    if (editingId === "__new__") {
      const row: SavedLoadout = {
        id: newMatchId(),
        name,
        loadout: cloneLoadout(draftLoadout),
        createdAt: Date.now(),
      };
      await getTelemetryStore().saveLoadout(row);
      editingId = row.id;
    } else if (editingId) {
      // Overwrite path: same ID, refreshed skills + name + createdAt kept.
      const existing = rows.find((r) => r.id === editingId);
      const row: SavedLoadout = {
        id: editingId,
        name,
        loadout: cloneLoadout(draftLoadout),
        createdAt: existing?.createdAt ?? Date.now(),
      };
      await getTelemetryStore().saveLoadout(row);
    }
    sfx.play("phaseEnd");
    await refresh();
  }

  // --- list actions --------------------------------------------------------

  async function requestDelete(id: string): Promise<void> {
    if (confirmDeleteId === id) {
      await getTelemetryStore().deleteLoadout(id);
      confirmDeleteId = null;
      if (editingId === id) {
        editingId = null;
      }
      sfx.play("death");
      await refresh();
    } else {
      confirmDeleteId = id;
      sfx.play("click");
    }
  }

  function cancelDelete(): void {
    confirmDeleteId = null;
    sfx.play("tick");
  }

  function startRename(row: SavedLoadout): void {
    renameTargetId = row.id;
    renameDraft = row.name;
    sfx.play("click");
  }

  async function commitRename(): Promise<void> {
    const id = renameTargetId;
    const name = renameDraft.trim();
    if (!id || name.length === 0) {
      renameTargetId = null;
      return;
    }
    await getTelemetryStore().updateLoadoutName(id, name);
    renameTargetId = null;
    if (editingId === id) draftName = name;
    sfx.play("phaseEnd");
    await refresh();
  }

  function cancelRename(): void {
    renameTargetId = null;
    renameDraft = "";
  }

  // --- share / export ------------------------------------------------------

  async function copyShareCode(row: SavedLoadout): Promise<void> {
    const code = encodeShareCode(row.loadout, row.name);
    try {
      await navigator.clipboard.writeText(code);
      copiedFlashId = row.id;
      setTimeout(() => {
        if (copiedFlashId === row.id) copiedFlashId = null;
      }, 1400);
      sfx.play("phaseEnd");
    } catch {
      // Fallback: put the code into the import textarea so the user can grab
      // it manually. Rare (only if clipboard perms denied).
      importText = code;
    }
  }

  function downloadJson(row: SavedLoadout): void {
    const json = encodeJson({ name: row.name, loadout: row.loadout });
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    const slug = row.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "") || "loadout";
    a.href = url;
    a.download = `loadout-${slug}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    sfx.play("click");
  }

  // --- import --------------------------------------------------------------

  async function runImport(): Promise<void> {
    const chunks = importText
      .split(/\r?\n/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    if (chunks.length === 0) {
      importReport = null;
      return;
    }
    let imported = 0;
    const skipped: string[] = [];
    const errors: string[] = [];
    // Snapshot existing rows once; dedupe against both existing IDB rows and
    // rows we've just imported in this batch (so pasting the same code twice
    // doesn't save it twice).
    let poolKey = new Map<string, string>();
    for (const r of rows) poolKey.set(loadoutKey(r.loadout), r.name);
    for (const raw of chunks) {
      const r = parseImport(raw);
      if ("error" in r) {
        errors.push(r.error);
        continue;
      }
      const key = loadoutKey(r.loadout);
      const dup = poolKey.get(key);
      if (dup) {
        skipped.push(r.name || dup);
        continue;
      }
      const row: SavedLoadout = {
        id: newMatchId(),
        name: r.name || "imported",
        loadout: r.loadout,
        createdAt: Date.now(),
      };
      await getTelemetryStore().saveLoadout(row);
      poolKey.set(key, row.name);
      imported += 1;
    }
    const bits: string[] = [];
    bits.push(
      imported === 1
        ? t("loadouts.importSuccessOne")
        : t("loadouts.importSuccess", { n: imported }),
    );
    if (skipped.length) {
      bits.push(t("loadouts.importSkipped", { names: skipped.join(", ") }));
    }
    if (errors.length) {
      bits.push(t("loadouts.importErrors", { details: errors.join("; ") }));
    }
    importReport = bits.join(" ");
    importText = "";
    sfx.play("phaseEnd");
    await refresh();
  }

  // --- helpers -------------------------------------------------------------

  function pieceLabel(idx: number): string {
    return idx === 0 ? t("draft.king") : t("draft.champion", { n: idx });
  }

  // Preview strip: 12 skill IDs as-is. Empty slot renders as an empty circle.
  function previewIds(l: SideLoadout): number[] {
    const out: number[] = [];
    for (const pair of l) {
      out.push(pair[0], pair[1]);
    }
    return out;
  }
</script>

<svelte:head>
  <title>{t("loadouts.title")}</title>
</svelte:head>

<SkillGlyphDefs />

<main class="loadouts">
  <header>
    <a class="back" href="./" onclick={() => sfx.play("click")}>{t("loadouts.back")}</a>
    <h1>{t("loadouts.title")}</h1>
  </header>

  <div class="cols">
    <!-- Left: saved list ------------------------------------------------- -->
    <section class="list">
      <div class="list-head">
        <h2>{t("loadouts.listHeading")}</h2>
        <button type="button" class="primary" onclick={startNew}>
          {t("loadouts.new")}
        </button>
      </div>

      {#if loading}
        <p class="muted">…</p>
      {:else if rows.length === 0}
        <p class="muted">{t("loadouts.empty")}</p>
      {:else}
        <ul>
          {#each rows as row (row.id)}
            <li class:selected={editingId === row.id}>
              {#if renameTargetId === row.id}
                <div class="rename-row">
                  <input
                    class="name-input"
                    type="text"
                    bind:value={renameDraft}
                    onkeydown={(e) => {
                      if (e.key === "Enter") commitRename();
                      else if (e.key === "Escape") cancelRename();
                    }}
                    aria-label={t("loadouts.name")}
                  />
                  <button type="button" onclick={commitRename}>
                    {t("loadouts.save")}
                  </button>
                  <button type="button" onclick={cancelRename}>
                    {t("loadouts.cancelDelete")}
                  </button>
                </div>
              {:else}
                <button
                  type="button"
                  class="row-body"
                  onclick={() => selectRow(row)}
                >
                  <div class="row-name">{row.name}</div>
                  <div class="preview" aria-hidden="true">
                    {#each previewIds(row.loadout) as id, i (i)}
                      {#if id === 0}
                        <span class="glyph-empty"></span>
                      {:else}
                        <svg class="glyph-mini" viewBox="0 0 24 24">
                          <use href="#skill-glyph-{id}" />
                        </svg>
                      {/if}
                    {/each}
                  </div>
                </button>
                <div class="row-actions">
                  <button type="button" onclick={() => startRename(row)}>
                    {t("loadouts.rename")}
                  </button>
                  <button
                    type="button"
                    onclick={() => copyShareCode(row)}
                    title={t("loadouts.copyShareCode")}
                  >
                    {copiedFlashId === row.id
                      ? t("loadouts.copied")
                      : t("loadouts.copyShareCode")}
                  </button>
                  <button
                    type="button"
                    onclick={() => downloadJson(row)}
                    title={t("loadouts.downloadJson")}
                  >
                    {t("loadouts.downloadJson")}
                  </button>
                  {#if confirmDeleteId === row.id}
                    <button type="button" class="danger" onclick={() => requestDelete(row.id)}>
                      {t("loadouts.confirmDelete")}
                    </button>
                    <button type="button" onclick={cancelDelete}>
                      {t("loadouts.cancelDelete")}
                    </button>
                  {:else}
                    <button type="button" onclick={() => requestDelete(row.id)}>
                      {t("loadouts.delete")}
                    </button>
                  {/if}
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}

      <!-- Import ------------------------------------------------------- -->
      <section class="import-block">
        <h3>{t("loadouts.import")}</h3>
        <textarea
          bind:value={importText}
          placeholder={t("loadouts.importPlaceholder")}
          rows="4"
        ></textarea>
        <button
          type="button"
          class="primary"
          onclick={runImport}
          disabled={importText.trim().length === 0}
        >
          {t("loadouts.importButton")}
        </button>
        {#if importReport}
          <p class="import-report">{importReport}</p>
        {/if}
      </section>
    </section>

    <!-- Right: editor --------------------------------------------------- -->
    <section class="editor">
      <h2>{t("loadouts.editorHeading")}</h2>

      {#if editingId === null}
        <p class="muted">{t("loadouts.editorEmpty")}</p>
      {:else}
        <div class="editor-controls">
          <label class="name-field">
            <span>{t("loadouts.name")}</span>
            <input
              type="text"
              bind:value={draftName}
              placeholder={t("loadouts.namePlaceholder")}
              maxlength="63"
            />
          </label>

          <div class="orientation">
            <span class="orientation-label">{t("loadouts.orientation")}</span>
            <label>
              <input
                type="radio"
                name="orientation"
                value="p1"
                bind:group={orientation}
              />
              {t("loadouts.orientationP1")}
            </label>
            <label>
              <input
                type="radio"
                name="orientation"
                value="p2"
                bind:group={orientation}
              />
              {t("loadouts.orientationP2")}
            </label>
          </div>
        </div>

        <div class="editor-body">
          <div class="board-column">
            <LoadoutBoard
              side={orientation}
              loadout={draftLoadout}
              interactive={true}
              selectedPieceIdx={selectedPieceIdx}
              onPieceClick={onPieceClick}
            />
            {#if duplicate}
              <p class="dup-warning">
                {t("loadouts.duplicateOf", { name: duplicate.name })}
              </p>
            {/if}
            <div class="save-row">
              <button
                type="button"
                class="primary"
                disabled={saveDisabled}
                title={saveDisabledReason}
                onclick={save}
              >
                {editingId === "__new__" ? t("loadouts.saveNew") : t("loadouts.save")}
              </button>
              {#if saveDisabled && saveDisabledReason}
                <span class="save-hint">{saveDisabledReason}</span>
              {/if}
            </div>
          </div>

          <div class="picker-column">
            {#if selectedPieceIdx !== null}
              <div class="picker-head">
                <span>
                  {t("loadouts.pieceEditor", { piece: pieceLabel(selectedPieceIdx) })}
                </span>
                <div class="slot-radios">
                  <label>
                    <input
                      type="radio"
                      name="slot"
                      value={0}
                      checked={editingSlot === 0}
                      onchange={() => (editingSlot = 0)}
                    />
                    {t("loadouts.slot", { n: 1 })}
                    {#if draftLoadout[selectedPieceIdx][0] !== 0}
                      <button
                        type="button"
                        class="slot-clear"
                        onclick={() => clearSlot(0)}
                        title={t("loadouts.clearSlot")}
                      >×</button>
                    {/if}
                  </label>
                  <label>
                    <input
                      type="radio"
                      name="slot"
                      value={1}
                      checked={editingSlot === 1}
                      onchange={() => (editingSlot = 1)}
                    />
                    {t("loadouts.slot", { n: 2 })}
                    {#if draftLoadout[selectedPieceIdx][1] !== 0}
                      <button
                        type="button"
                        class="slot-clear"
                        onclick={() => clearSlot(1)}
                        title={t("loadouts.clearSlot")}
                      >×</button>
                    {/if}
                  </label>
                </div>
              </div>
              <SkillPicker
                interaction="click"
                disabledIds={[draftLoadout[selectedPieceIdx][editingSlot === 0 ? 1 : 0]].filter((v) => v !== 0)}
                onPick={pickSkill}
              />
            {/if}
          </div>
        </div>
      {/if}
    </section>
  </div>
</main>

<style>
  .loadouts {
    max-width: 1180px;
    margin: 0 auto;
    padding: 1rem 1.5rem 2rem;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 1rem;
    border-bottom: 1.5px solid var(--paper-line);
    padding-bottom: 0.4rem;
  }
  header h1 {
    font-size: 1.8rem;
    margin: 0;
  }
  .back {
    color: inherit;
    text-decoration: none;
    padding: 0.2em 0.5em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
  }
  .back:hover { background: var(--paper-line); }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1.15fr;
    gap: 1.2rem;
  }
  @media (max-width: 900px) {
    .cols { grid-template-columns: 1fr; }
  }
  section { min-width: 0; }
  h2 { font-size: 1.15rem; margin: 0 0 0.6rem; }
  h3 { font-size: 1rem; margin: 0.8rem 0 0.4rem; }
  .muted { color: var(--paper-ink-soft, #6a604a); }
  .list-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .list ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .list li {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    padding: 0.5em 0.6em;
    background: var(--paper-bg);
  }
  .list li.selected {
    outline: 2px solid var(--accent, #c94);
    outline-offset: 0px;
  }
  .row-body {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .row-name {
    font-weight: 600;
    margin-bottom: 0.25em;
  }
  .preview {
    display: flex;
    flex-wrap: wrap;
    gap: 0.15em;
  }
  .glyph-mini {
    width: 18px;
    height: 18px;
    color: var(--paper-ink);
    stroke-width: 2.4;
  }
  .glyph-mini :global(use) { stroke-width: 2.4; }
  .glyph-empty {
    display: inline-block;
    width: 18px;
    height: 18px;
    border: 1.2px dashed var(--paper-line-strong);
    border-radius: 50%;
    opacity: 0.4;
  }
  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3em;
    margin-top: 0.4em;
  }
  .row-actions button,
  .rename-row button,
  .list-head button,
  .save-row button,
  .import-block button {
    padding: 0.2em 0.55em;
    border: 1.2px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  button.primary {
    border-color: var(--accent, #c94);
    background: color-mix(in srgb, var(--accent, #c94) 15%, var(--paper-bg));
    font-weight: 600;
  }
  button.danger {
    border-color: #a94b3b;
    color: #a94b3b;
    font-weight: 600;
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--paper-line) 40%, var(--paper-bg));
  }
  .rename-row {
    display: flex;
    gap: 0.3em;
    align-items: center;
  }
  .name-input {
    flex: 1;
    padding: 0.25em 0.4em;
    font: inherit;
  }
  .import-block {
    margin-top: 1.2em;
    border-top: 1.5px dashed var(--paper-line-strong);
    padding-top: 0.6em;
  }
  .import-block textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.4em;
    font: inherit;
    font-family: ui-monospace, "SFMono-Regular", Menlo, monospace;
    font-size: 0.85rem;
    border: 1.2px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
    margin-bottom: 0.4em;
    resize: vertical;
  }
  .import-report {
    margin: 0.4em 0 0;
    font-size: 0.9em;
    color: var(--paper-ink-soft, #6a604a);
  }
  /* Editor -------------------------------------------------------------- */
  .editor {
    border-left: 1.5px solid var(--paper-line);
    padding-left: 1.2rem;
  }
  @media (max-width: 900px) {
    .editor { border-left: none; padding-left: 0; }
  }
  .editor-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: center;
    margin-bottom: 0.8rem;
  }
  .name-field {
    display: flex;
    flex-direction: column;
    gap: 0.2em;
    flex: 1 1 220px;
  }
  .name-field input {
    padding: 0.3em 0.5em;
    font: inherit;
    border: 1.2px solid var(--paper-line-strong);
    border-radius: 5px;
    background: var(--paper-bg);
    color: inherit;
  }
  .orientation {
    display: flex;
    gap: 0.5em;
    align-items: center;
  }
  .orientation-label {
    font-size: 0.9em;
    color: var(--paper-ink-soft, #6a604a);
  }
  .editor-body {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 1rem;
    align-items: start;
  }
  @media (max-width: 700px) {
    .editor-body { grid-template-columns: 1fr; }
  }
  .board-column {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
  }
  .save-row {
    display: flex;
    gap: 0.5em;
    align-items: center;
    flex-wrap: wrap;
  }
  .save-hint {
    font-size: 0.85em;
    color: var(--paper-ink-soft, #6a604a);
  }
  .dup-warning {
    margin: 0.3em 0 0;
    font-size: 0.9em;
    color: #a94b3b;
  }
  .picker-column {
    min-width: 0;
  }
  .picker-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5em;
    margin-bottom: 0.5em;
    font-size: 0.9em;
  }
  .slot-radios {
    display: flex;
    gap: 0.7em;
  }
  .slot-radios label {
    display: inline-flex;
    align-items: center;
    gap: 0.2em;
  }
  .slot-clear {
    padding: 0 0.35em;
    border: 1px solid var(--paper-line-strong);
    background: var(--paper-bg);
    border-radius: 4px;
    cursor: pointer;
    font: inherit;
    line-height: 1;
    color: inherit;
  }
  .slot-clear:hover {
    background: color-mix(in srgb, var(--paper-line) 40%, var(--paper-bg));
  }
</style>
