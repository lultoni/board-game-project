<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { getEngine, encodeDraftTurn } from "$lib/engine";
  import { buildEngineConfigJson } from "$lib/engine/config";
  import { decodeMailbox } from "$lib/engine/mailbox";
  import { SKILLS, SKILL_COUNT } from "$lib/engine/skills";
  import { t } from "$lib/state/i18n";
  import {
    match,
    modeFromSeats,
    resetMatchState,
  } from "$lib/state/match-store.svelte";
  import { sendData } from "$lib/multiplayer.svelte";
  import {
    squareName,
    STACK_M_LOADOUT_SQUARES,
  } from "$lib/state/draft";
  import { settings } from "$lib/state/settings.svelte";
  import type { DraftStateView, PositionView, EngineClient } from "$lib/engine/types";

  // === Boot / engine handle ==================================================

  const mode = $derived(modeFromSeats(match.side));
  const isMultiplayer = $derived(match.mode === "multiplayer");

  let eng = $state<EngineClient | null>(null);
  let bootError = $state<string | null>(null);
  let booted = $state(false);
  let busy = $state(false);
  let starting = $state(false);

  // Live engine snapshots, refreshed after every applied DraftTurn.
  let position = $state<PositionView | null>(null);
  let draftState = $state<DraftStateView | null>(null);

  const P1_SQUARES = STACK_M_LOADOUT_SQUARES.p1;
  const P2_SQUARES = STACK_M_LOADOUT_SQUARES.p2;
  const allSkillIds = Array.from({ length: SKILL_COUNT }, (_, i) => i + 1);

  // Which side is currently drafting (0 = P1, 1 = P2).
  const sideToMove = $derived(draftState?.sideToMove ?? 0);
  const isP1Turn = $derived(sideToMove === 0);
  const sideSquares = $derived(isP1Turn ? P1_SQUARES : P2_SQUARES);
  const currentSeat = $derived(isP1Turn ? match.side.p1 : match.side.p2);
  const currentSeatIsAi = $derived(currentSeat === "ai");

  // In multiplayer, Phase E falls back to "host drafts both sides, ships
  // finished snapshot to joiner" — same protocol as today's makeshift draft.
  // The host therefore drafts every turn locally. Phase F replaces this with
  // peer-to-peer DraftTurn streaming and makes joiner-side picks possible.
  const localCanDraft = $derived.by(() => {
    if (currentSeatIsAi) return false;
    if (!isMultiplayer) return true;
    if (match.multiplayerRole === "host") return true;
    if (match.multiplayerRole === "joiner") return false;
    return false;
  });

  const draftComplete = $derived((draftState?.turnNo ?? 0) >= 12);

  // === Picker state ==========================================================
  //
  // A draft "turn" is two picks. The player drives the staging UI freely:
  // - Click a skill on the catalogue → fills the first empty staging slot.
  // - Click a piece+slot in the panel → assigns that piece+slot to the next
  //   staging pick that has a skill but no target (or, if both already have
  //   targets, replaces the most recently assigned target).
  // - "Commit" submits the DraftTurn to the engine.
  //
  // Staging state is wiped after every commit / on side flip / on AI turn.

  interface StagedPick {
    skillId: number;       // 1..15, or 0 = empty
    sq: number;            // 0..63, or -1 = unassigned
    slot: number;          // 0 (slot1) or 1 (slot2), or -1 = unassigned
  }
  function emptyPick(): StagedPick { return { skillId: 0, sq: -1, slot: -1 }; }

  let pick1 = $state<StagedPick>(emptyPick());
  let pick2 = $state<StagedPick>(emptyPick());

  function clearPicks(): void {
    pick1 = emptyPick();
    pick2 = emptyPick();
  }

  /** True iff (sq, slot) on this side is empty in the engine state AND
   *  isn't already claimed by a staged pick. */
  function isStageableTarget(sq: number, slot: number): boolean {
    if (!position) return false;
    // Own-side ownership check — we only let the active player stage onto
    // their own pieces. Engine would reject otherwise, but the UI shouldn't
    // tempt the user.
    if (!sideSquares.includes(sq)) return false;
    const entry = decodeMailbox(position.mailbox[sq]);
    if (slot === 0 && entry.skill1 !== 0) return false;
    if (slot === 1 && entry.skill2 !== 0) return false;
    if (pick1.sq === sq && pick1.slot === slot) return false;
    if (pick2.sq === sq && pick2.slot === slot) return false;
    return true;
  }

  /** True iff both staged picks have a skill AND a target AND would be
   *  accepted by `legal_draft_turns` (no same-piece-same-skill). */
  const commitReady = $derived.by(() => {
    if (!localCanDraft) return false;
    if (pick1.skillId === 0 || pick2.skillId === 0) return false;
    if (pick1.sq < 0 || pick1.slot < 0) return false;
    if (pick2.sq < 0 || pick2.slot < 0) return false;
    if (pick1.sq === pick2.sq && pick1.slot === pick2.slot) return false;
    // Same piece, same skill in different slots → illegal.
    if (pick1.sq === pick2.sq && pick1.skillId === pick2.skillId) return false;
    if (!position) return false;
    // Same-skill-on-same-piece check also reads the *other* slot the engine
    // already has filled. If pick1 hits piece P slot 0 with skill X, and
    // pick2's piece P already has skill X in slot 1 (from a prior turn),
    // that's illegal — and vice versa. The engine catches this too, but
    // the user shouldn't be allowed to stage it.
    for (const p of [pick1, pick2] as const) {
      const entry = decodeMailbox(position.mailbox[p.sq]);
      const otherSlotSkill = p.slot === 0 ? entry.skill2 : entry.skill1;
      if (otherSlotSkill === p.skillId) return false;
    }
    return true;
  });

  // === Skill / target click handlers ========================================

  function handleSkillClick(skillId: number): void {
    if (!localCanDraft) return;
    // Toggle: clicking an already-staged skill that has no target yet
    // removes it. Otherwise fill the next empty staging slot.
    if (pick1.skillId === skillId && pick1.sq < 0) {
      pick1 = emptyPick();
      return;
    }
    if (pick2.skillId === skillId && pick2.sq < 0) {
      pick2 = emptyPick();
      return;
    }
    if (pick1.skillId === 0) {
      pick1 = { ...pick1, skillId };
      return;
    }
    if (pick2.skillId === 0) {
      pick2 = { ...pick2, skillId };
      return;
    }
    // Both staging slots have skills — replace the one without a target, or
    // pick1 if both have targets.
    if (pick1.sq < 0) { pick1 = { ...pick1, skillId }; return; }
    if (pick2.sq < 0) { pick2 = { ...pick2, skillId }; return; }
    pick1 = { skillId, sq: -1, slot: -1 };
  }

  function handleTargetClick(sq: number, slot: number): void {
    if (!localCanDraft) return;
    if (!isStageableTarget(sq, slot)) return;
    // Route into the first staged pick that has a skill but no target.
    if (pick1.skillId !== 0 && pick1.sq < 0) {
      pick1 = { ...pick1, sq, slot };
      return;
    }
    if (pick2.skillId !== 0 && pick2.sq < 0) {
      pick2 = { ...pick2, sq, slot };
      return;
    }
    // Both picks have targets (or neither has a skill yet). Re-assign the
    // most recently-targeted pick — gives the user a way to course-correct.
    if (pick2.skillId !== 0) {
      pick2 = { ...pick2, sq, slot };
      return;
    }
    if (pick1.skillId !== 0) {
      pick1 = { ...pick1, sq, slot };
    }
  }

  function clearPick(which: 1 | 2): void {
    if (which === 1) pick1 = emptyPick();
    else pick2 = emptyPick();
  }

  // === Commit ===============================================================

  async function commitTurn(): Promise<void> {
    if (!eng || !commitReady || busy) return;
    busy = true;
    try {
      const raw = encodeDraftTurn(
        pick1.skillId, pick1.sq, pick1.slot,
        pick2.skillId, pick2.sq, pick2.slot,
      );
      await eng.tryApply(raw);
      clearPicks();
      await refresh();
      if ((draftState?.turnNo ?? 0) >= 12) await finishAndForward();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // === AI scheduling ========================================================
  //
  // Every refresh, if the side-to-draft is AI and the draft isn't complete,
  // queue a `stepAi` call. The engine's AI draft heuristic is the
  // fixed-preset path (see Phase C) — it picks a deterministic loadout per
  // piece. We honour the user's AIvAI step delay so the player can watch.

  let aiScheduled = false;
  $effect(() => {
    if (!booted) return;
    if (busy) return;
    if (draftComplete) return;
    if (!currentSeatIsAi) return;
    if (aiScheduled) return;
    aiScheduled = true;
    const delay = mode === "aivai"
      ? Math.max(16, settings.aivaiStepDelayMs)
      : 200;
    setTimeout(() => {
      aiScheduled = false;
      void runAiDraftStep();
    }, delay);
  });

  async function runAiDraftStep(): Promise<void> {
    if (!eng || busy) return;
    if (!currentSeatIsAi) return;
    if (draftComplete) return;
    busy = true;
    try {
      clearPicks();
      const r = await eng.stepAi();
      if (r.appliedAction === 0) return; // AI failed to find a move
      await refresh();
      if ((draftState?.turnNo ?? 0) >= 12) await finishAndForward();
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  // === Boot + refresh =======================================================

  async function refresh(): Promise<void> {
    if (!eng) return;
    position = await eng.positionView();
    draftState = await eng.draftState();
  }

  onMount(async () => {
    try {
      const wasMultiplayer = match.mode === "multiplayer";
      // Inspector handoff: a pre-built snapshot is staged → forward to /match/.
      if (match.pendingSnapshotJson) {
        const e = await getEngine();
        const newCfg = JSON.parse(buildEngineConfigJson(match.side));
        const parsed = JSON.parse(match.pendingSnapshotJson);
        parsed.config = newCfg;
        const newSnap = JSON.stringify(parsed);
        await e.restoreFromSnapshot(newSnap);
        match.pendingSnapshotJson = newSnap;
        match.mode = wasMultiplayer ? "multiplayer" : modeFromSeats(match.side);
        await goto("../match/");
        return;
      }
      resetMatchState();
      if (wasMultiplayer) match.mode = "multiplayer";
      eng = await getEngine();
      const configJson = buildEngineConfigJson(match.side);
      await eng.createEngineWithDraft(configJson);
      await refresh();
      booted = true;
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
    }
  });

  // === Finish ==============================================================

  async function finishAndForward(): Promise<void> {
    if (!eng) return;
    starting = true;
    try {
      const snap = await eng.snapshotJson();
      match.pendingSnapshotJson = snap;
      match.mode = match.mode === "multiplayer"
        ? "multiplayer"
        : modeFromSeats(match.side);
      // In multiplayer, ship the fully-drafted snapshot to the joiner — same
      // protocol as the makeshift draft. Phase F replaces this with per-turn
      // streaming, but for Phase E the host drafts alone and the joiner
      // receives the finished snapshot.
      if (match.mode === "multiplayer") {
        sendData({ kind: "snapshot", snapshotJson: snap });
      }
      await goto("../match/");
    } catch (e) {
      bootError = (e as Error)?.message ?? String(e);
      starting = false;
    }
  }

  // === Display helpers =====================================================

  function skillName(id: number): string {
    if (id === 0) return "—";
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.name`) : `?${id}`;
  }

  function skillDesc(id: number): string {
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.desc`) : "";
  }

  function pieceLabel(sq: number, isKing: boolean, championIdx: number): string {
    return isKing ? t("draft.king") : t("draft.champion", { n: championIdx });
  }

  function slotSkillId(sq: number, slot: number): number {
    if (!position) return 0;
    const entry = decodeMailbox(position.mailbox[sq]);
    return slot === 0 ? entry.skill1 : entry.skill2;
  }

  /** True iff (sq, slot) carries a pick we just staged this turn — render
   *  with a "staged" tint so the user can see what they're about to commit. */
  function isStagedTarget(sq: number, slot: number): "p1" | "p2" | null {
    if (pick1.sq === sq && pick1.slot === slot) return "p1";
    if (pick2.sq === sq && pick2.slot === slot) return "p2";
    return null;
  }

  /** For displaying the staged pick on a target slot when the engine slot is
   *  still empty — we want to show the *future* skill there. */
  function stagedSkillAt(sq: number, slot: number): number {
    if (pick1.sq === sq && pick1.slot === slot) return pick1.skillId;
    if (pick2.sq === sq && pick2.slot === slot) return pick2.skillId;
    return 0;
  }
</script>

<main>
  <header>
    <p class="back"><a href="../">← back</a></p>
    <h1>{t("draft.title")}</h1>
    <small class="mode-tag">{mode}</small>
  </header>

  {#if bootError}
    <p class="err">boot error: {bootError}</p>
  {:else if !booted || !position || !draftState}
    <p>{t("app.loading")}</p>
  {:else if draftComplete && starting}
    <p>{t("draft.starting")}</p>
  {:else}
    <section class="status">
      <div class="status-cell">
        <span class="status-label">{t("draft.turn")}</span>
        <span class="status-value">{Math.min(draftState.turnNo + 1, 12)} / 12</span>
      </div>
      <div class="status-cell">
        <span class="status-label">{t("draft.toPick")}</span>
        <span class="status-value" class:p1={isP1Turn} class:p2={!isP1Turn}>
          {isP1Turn ? t("setup.p1Label") : t("setup.p2Label")}
        </span>
      </div>
      <div class="status-cell">
        <span class="status-label">{t("draft.seat")}</span>
        <span class="status-value">
          {currentSeatIsAi ? t("setup.ai") : t("setup.human")}
        </span>
      </div>
    </section>

    {#if !localCanDraft}
      <p class="waiting">
        {#if currentSeatIsAi}
          {t("draft.aiDrafting")}
        {:else if isMultiplayer}
          {t("draft.waitingForPeer", { n: isP1Turn ? 1 : 2 })}
        {:else}
          {t("draft.waitingForPlayer", { n: isP1Turn ? 1 : 2 })}
        {/if}
      </p>
    {/if}

    <div class="layout">
      <!-- Left: skill catalogue + staged picks -->
      <section class="picker">
        <h2>{t("draft.catalogue")}</h2>
        <ul class="skills" class:disabled={!localCanDraft}>
          {#each allSkillIds as id (id)}
            {@const staged1 = pick1.skillId === id && pick1.sq < 0}
            {@const staged2 = pick2.skillId === id && pick2.sq < 0}
            <li>
              <button
                type="button"
                class:staged={staged1 || staged2}
                disabled={!localCanDraft}
                onclick={() => handleSkillClick(id)}
                title={skillDesc(id)}
              >
                <span class="skillName">{skillName(id)}</span>
              </button>
            </li>
          {/each}
        </ul>

        <h2>{t("draft.staging")}</h2>
        <ul class="staging">
          {#each [pick1, pick2] as p, i}
            <li class="stage">
              <span class="stageLabel">{t("draft.pickN", { n: i + 1 })}</span>
              <span class="stageSkill">{p.skillId === 0 ? "—" : skillName(p.skillId)}</span>
              <span class="stageArrow">→</span>
              <span class="stageTarget">
                {#if p.sq < 0}
                  <em>{t("draft.unassigned")}</em>
                {:else}
                  {squareName(p.sq)} · slot {p.slot + 1}
                {/if}
              </span>
              <button
                type="button"
                class="ghost"
                disabled={!localCanDraft || (p.skillId === 0 && p.sq < 0)}
                onclick={() => clearPick((i + 1) as 1 | 2)}
              >{t("draft.clearPick")}</button>
            </li>
          {/each}
        </ul>

        <div class="commit">
          <button
            type="button"
            class="primary"
            disabled={!commitReady || busy}
            onclick={commitTurn}
          >{t("draft.commitTurn")}</button>
        </div>
      </section>

      <!-- Right: pieces (P1 above P2, current side highlighted) -->
      <section class="pieces-col">
        {#each [["p1", P1_SQUARES] as const, ["p2", P2_SQUARES] as const] as [side, squares]}
          {@const isActive = (side === "p1") === isP1Turn}
          <section class="side" class:p1={side === "p1"} class:p2={side === "p2"} class:active={isActive}>
            <h2>{side === "p1" ? t("setup.p1Label") : t("setup.p2Label")}</h2>
            <ul class="pieces">
              {#each squares as sq, i (sq)}
                {@const isKing = i === 0}
                <li class:king={isKing}>
                  <span class="pname">{pieceLabel(sq, isKing, i)}</span>
                  <span class="psq">{squareName(sq)}</span>
                  {#each [0, 1] as slot}
                    {@const filled = slotSkillId(sq, slot)}
                    {@const stagedSide = isStagedTarget(sq, slot)}
                    {@const stagedFor = stagedSkillAt(sq, slot)}
                    {@const clickable = isActive && localCanDraft && filled === 0}
                    <button
                      type="button"
                      class="slot"
                      class:filled={filled !== 0}
                      class:staged-p1={stagedSide === "p1"}
                      class:staged-p2={stagedSide === "p2"}
                      class:empty-target={clickable}
                      disabled={!clickable}
                      onclick={() => handleTargetClick(sq, slot)}
                    >
                      {#if filled !== 0}
                        {skillName(filled)}
                      {:else if stagedFor !== 0}
                        <em>{skillName(stagedFor)}</em>
                      {:else}
                        —
                      {/if}
                    </button>
                  {/each}
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </section>
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
  .status {
    display: flex;
    gap: 1.5rem;
    margin: 0.5rem 0 0.8rem;
    padding: 0.5em 0.9em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--paper-bg);
  }
  .status-cell {
    display: flex;
    flex-direction: column;
    gap: 0.1em;
  }
  .status-label {
    font-size: 0.72rem;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .status-value { font-weight: 600; }
  .status-value.p1 { color: var(--p1, #2b4a8a); }
  .status-value.p2 { color: var(--p2, #a13a2a); }
  .waiting {
    margin: 0.4rem 0 0.8rem;
    padding: 0.5em 0.9em;
    border: 1.5px dashed var(--paper-line-strong);
    border-radius: 6px;
    color: var(--paper-ink-soft);
    font-style: italic;
    text-align: center;
  }
  .layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.2rem;
    align-items: start;
  }
  @media (max-width: 820px) {
    .layout { grid-template-columns: 1fr; }
  }
  .picker, .pieces-col > .side {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 8px;
    padding: 0.8em 1em;
    background: var(--paper-bg);
  }
  .picker h2, .side h2 {
    margin: 0 0 0.5em;
    font-size: 1.1rem;
  }
  .picker h2:not(:first-child) { margin-top: 0.9em; }
  .skills {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.35em;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .skills li { margin: 0; }
  .skills button {
    width: 100%;
    font: inherit;
    padding: 0.35em 0.5em;
    border: 1.5px solid var(--paper-line);
    border-radius: 5px;
    background: var(--paper-bg);
    cursor: pointer;
  }
  .skills button:hover:not(:disabled) {
    background: var(--paper-square-light, #ece2c8);
  }
  .skills button.staged {
    border-color: var(--accent, #c79b3a);
    background: rgba(199, 155, 58, 0.18);
    font-weight: 600;
  }
  .skills.disabled button {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .staging {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 0.3em;
  }
  .stage {
    display: grid;
    grid-template-columns: 4em 6em 1em 1fr auto;
    align-items: center;
    gap: 0.4em;
    padding: 0.3em 0.5em;
    border: 1px dashed var(--paper-line);
    border-radius: 5px;
  }
  .stageLabel {
    font-size: 0.78rem;
    color: var(--paper-ink-soft);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .stageSkill { font-weight: 600; }
  .stageArrow { color: var(--paper-ink-soft); text-align: center; }
  .stageTarget em {
    font-style: italic;
    color: var(--paper-ink-soft);
  }
  .stage .ghost {
    font: inherit;
    padding: 0.15em 0.5em;
    border: 1px solid var(--paper-line);
    border-radius: 4px;
    background: transparent;
    cursor: pointer;
    font-size: 0.85em;
  }
  .stage .ghost:disabled { opacity: 0.3; cursor: not-allowed; }
  .commit {
    display: flex;
    justify-content: flex-end;
    margin-top: 0.8em;
  }
  .primary {
    padding: 0.55em 1.2em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: var(--accent, #c79b3a);
    color: #fff;
    border-color: var(--accent, #c79b3a);
    font-weight: 600;
    cursor: pointer;
  }
  .primary:hover:not(:disabled) {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
    transform: translateY(-1px);
  }
  .primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .pieces-col {
    display: grid;
    gap: 1rem;
  }
  .side.p1 { border-top: 4px solid var(--p1, #2b4a8a); }
  .side.p2 { border-top: 4px solid var(--p2, #a13a2a); }
  .side:not(.active) { opacity: 0.55; }
  .pieces {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 0.35em;
  }
  .pieces li {
    display: grid;
    grid-template-columns: 7em 3em 1fr 1fr;
    gap: 0.4em;
    align-items: center;
    padding: 0.2em 0.3em;
    border-bottom: 1px dashed var(--paper-line);
  }
  .pieces li.king { font-weight: 600; }
  .pname { font-size: 0.9rem; }
  .psq { color: var(--paper-ink-soft); font-size: 0.85rem; }
  .slot {
    font: inherit;
    padding: 0.25em 0.45em;
    border: 1.5px solid var(--paper-line);
    border-radius: 4px;
    background: var(--paper-bg);
    text-align: center;
    cursor: pointer;
  }
  .slot:disabled { cursor: default; }
  .slot.filled {
    background: var(--paper-square-light, #ece2c8);
    cursor: default;
  }
  .slot.empty-target:hover {
    background: rgba(199, 155, 58, 0.15);
    border-color: var(--accent, #c79b3a);
  }
  .slot.staged-p1, .slot.staged-p2 {
    border-color: var(--accent, #c79b3a);
    border-style: dashed;
    background: rgba(199, 155, 58, 0.10);
    font-style: italic;
  }
  .err {
    color: #a94b3b;
    border: 1.5px dashed currentColor;
    padding: 0.5em 0.8em;
    border-radius: 6px;
  }
</style>
