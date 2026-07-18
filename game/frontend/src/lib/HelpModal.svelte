<script lang="ts">
  // In-game help / reference surface (ns-35 part A). Opened from the Help
  // button in the root layout, next to the Settings gear. Three tabs:
  //   - Skills:   every skill's name / category / cost / range / effect,
  //               read from the SKILLS registry + existing i18n keys.
  //   - Rules:    the Stack M "Simple Overview" + Goal (help.rules.*).
  //   - Controls: how-to-play tips (help.controls.*).
  // Structure and styling mirror SettingsModal.svelte (native <dialog>).
  // Deliberately does NOT reuse SkillInfoCard (coupled to the wheel's
  // SliceKind prop) - the skill list here is rendered inline. Component
  // unification is ns-35 part B, deferred.

  import { SKILLS, CATEGORY_COLOR, type SkillCategory } from "$lib/engine";
  import { t } from "$lib/state/i18n";
  import { sfx } from "$lib/audio/sfx";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  let dialogEl: HTMLDialogElement | null = $state(null);
  let tab = $state<"skills" | "rules" | "controls">("skills");

  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
    }
  });

  function onDialogCancel(ev: Event): void {
    ev.preventDefault();
    onClose();
  }

  const TABS: ReadonlyArray<{ id: "skills" | "rules" | "controls"; key: string }> = [
    { id: "skills", key: "help.tabSkills" },
    { id: "rules", key: "help.tabRules" },
    { id: "controls", key: "help.tabControls" },
  ];

  // Skills in stable id order.
  const skillList = Object.values(SKILLS).sort((a, b) => a.id - b.id);

  function categoryLabel(cat: SkillCategory): string {
    switch (cat) {
      case "strike": return t("wheel.categoryStrike");
      case "shield": return t("wheel.categoryShield");
      case "move":   return t("wheel.categoryMove");
      case "mystic": return t("wheel.categoryMystic");
    }
  }

  // Rules tab: goal first, then the 11 Simple-Overview bullets. Each entry
  // resolves a term + body i18n key so the German prose stays reviewable
  // line-by-line.
  const RULE_KEYS: ReadonlyArray<string> = [
    "goal", "rounds", "move", "moveAttack", "health", "armor",
    "skillPhase", "money", "path", "strikeMove", "bodyguard", "drafting", "progression",
  ];

  const CONTROL_KEYS: ReadonlyArray<string> = [
    "select", "wheel", "end", "undo", "sandbox",
  ];
</script>

<dialog bind:this={dialogEl} oncancel={onDialogCancel}>
  <div class="header">
    <h2>{t("help.title")}</h2>
    <button class="close" onclick={() => { sfx.play("click"); onClose(); }} aria-label={t("help.title")}>✕</button>
  </div>

  <div class="tabs">
    <div class="segmented">
      {#each TABS as tb}
        <button
          class:active={tab === tb.id}
          onclick={() => { sfx.play("click"); tab = tb.id; }}
        >{t(tb.key)}</button>
      {/each}
    </div>
  </div>

  {#if tab === "skills"}
    <section>
      <ul class="skill-list">
        {#each skillList as s}
          <li class="skill-row" style:--accent={CATEGORY_COLOR[s.category]}>
            <div class="skill-head">
              <span class="skill-name">{t(`skills.${s.key}.name`)}</span>
              <span class="chip">{categoryLabel(s.category)}</span>
            </div>
            <div class="skill-stats">
              <span class="stat">{t("wheel.cost", { n: s.cost })}</span>
              {#if s.defaultRange > 0}
                <span class="stat">{t("wheel.range", { n: s.defaultRange })}</span>
              {/if}
            </div>
            <p class="skill-desc">{t(`skills.${s.key}.desc`)}</p>
          </li>
        {/each}
      </ul>
    </section>
  {:else if tab === "rules"}
    <section>
      <dl class="rules">
        {#each RULE_KEYS as k}
          <div class="rule">
            <dt>{t(`help.rules.${k}Term`)}</dt>
            <dd>{t(`help.rules.${k}Body`)}</dd>
          </div>
        {/each}
      </dl>
    </section>
  {:else}
    <section>
      <dl class="rules">
        {#each CONTROL_KEYS as k}
          <div class="rule">
            <dt>{t(`help.controls.${k}Term`)}</dt>
            <dd>{t(`help.controls.${k}Body`)}</dd>
          </div>
        {/each}
      </dl>
    </section>
  {/if}
</dialog>

<style>
  dialog {
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 10px;
    padding: 0;
    background: var(--paper-bg);
    color: inherit;
    width: min(480px, 94vw);
    max-height: min(85vh, 700px);
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.4);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.9rem 1.1rem 0.6rem;
    position: sticky;
    top: 0;
    background: var(--paper-bg);
    border-bottom: 1px solid var(--paper-line);
    z-index: 1;
  }
  .header h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .close {
    background: none;
    border: none;
    padding: 0.2em 0.4em;
    font-size: 1rem;
    cursor: pointer;
    color: var(--paper-ink-soft);
    line-height: 1;
  }
  .close:hover {
    color: var(--paper-ink);
  }

  .tabs {
    padding: 0.7rem 1.1rem 0.2rem;
    position: sticky;
    top: 2.9rem;
    background: var(--paper-bg);
    z-index: 1;
  }

  section {
    padding: 0.5rem 1.1rem 0.9rem;
  }

  .segmented {
    display: flex;
    gap: 0;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    overflow: hidden;
  }
  .segmented button {
    flex: 1;
    border: none;
    border-left: 1px solid var(--paper-line-strong);
    border-radius: 0;
    padding: 0.3em 0.7em;
    font: inherit;
    font-size: 0.9em;
    background: transparent;
    cursor: pointer;
    transition: background 100ms;
  }
  .segmented button:first-child {
    border-left: none;
  }
  .segmented button.active {
    background: var(--accent);
    color: #fff;
  }

  /* Skills tab */
  .skill-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }
  .skill-row {
    --accent: #5a4a3a;
    padding: 0.5em 0.7em;
    border: 1px solid var(--paper-line);
    border-left: 4px solid var(--accent);
    border-radius: 6px;
    background: var(--paper-square-light, #ece2c8);
  }
  .skill-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.6em;
  }
  .skill-name {
    font-weight: 700;
    color: var(--accent);
  }
  .chip {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--paper-ink-soft);
  }
  .skill-stats {
    display: flex;
    gap: 0.5em;
    margin: 0.3em 0;
    font-size: 0.75rem;
    color: var(--paper-ink-soft);
  }
  .stat {
    padding: 0.05em 0.45em;
    border: 1px solid var(--paper-line);
    border-radius: 3px;
    background: var(--paper-bg);
  }
  .skill-desc {
    margin: 0;
    font-size: 0.88rem;
    line-height: 1.35;
    color: var(--paper-ink);
  }

  /* Rules + Controls tabs */
  .rules {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .rule dt {
    font-weight: 700;
    font-size: 0.92rem;
    color: var(--paper-ink);
  }
  .rule dd {
    margin: 0.15rem 0 0;
    font-size: 0.88rem;
    line-height: 1.4;
    color: var(--paper-ink-soft);
  }
</style>
