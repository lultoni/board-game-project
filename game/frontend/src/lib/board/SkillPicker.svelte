<script lang="ts">
  // Shared skill catalogue chip grid.
  //
  // Two consumers:
  //   - Draft (`interaction: "drag"`): chips are draggable; the draft page's
  //     existing drop-target logic (dragPayload / dropOnSlot) reads dataTransfer
  //     and knows what to do. The picker itself doesn't own the drag state — it
  //     just fires onDragStart/onDragEnd callbacks so the parent can track its
  //     dragPayload the way it did before extraction.
  //   - Loadout editor (`interaction: "click"`): chips emit onPick(id) on click.
  //     The editor tracks "which slot is being edited" externally.
  //
  // `disabledIds` grays out and blocks interaction on individual chips. In the
  // loadout editor this is used to prevent same-skill-both-slots (the second
  // slot's currently-selected skill is greyed in the picker while editing the
  // first slot, and vice versa).
  //
  // The `disabled` prop is a blanket lock (e.g. draft: not your turn) —
  // everything is uninteractive but still visible.

  import { SKILLS, SKILL_COUNT, skillColor } from "$lib/engine";
  import { t } from "$lib/state/i18n";

  interface Props {
    /** Skill IDs to render, in order. Defaults to 1..SKILL_COUNT. */
    skills?: number[];
    /** Interaction mode. Draft = drag source. Loadout editor = click emits pick. */
    interaction: "click" | "drag";
    /** Blanket disable — everything visible but uninteractive. */
    disabled?: boolean;
    /** Individual skill IDs to gray out and block. */
    disabledIds?: number[];
    /** Click-mode: fired when a chip is activated. */
    onPick?: (id: number) => void;
    /** Drag-mode: parent tracks its own dragPayload/dropzones. */
    onDragStart?: (ev: DragEvent, id: number) => void;
    onDragEnd?: () => void;
  }

  let {
    skills,
    interaction,
    disabled = false,
    disabledIds = [],
    onPick,
    onDragStart,
    onDragEnd,
  }: Props = $props();

  const ids = $derived(skills ?? Array.from({ length: SKILL_COUNT }, (_, i) => i + 1));
  const disabledSet = $derived(new Set(disabledIds));

  function skillName(id: number): string {
    if (id === 0) return "—";
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.name`) : `?${id}`;
  }

  function skillDesc(id: number): string {
    const info = SKILLS[id];
    return info ? t(`skills.${info.key}.desc`) : "";
  }

  function categoryLabel(id: number): string {
    const c = SKILLS[id]?.category;
    if (!c) return "";
    if (c === "strike") return t("wheel.categoryStrike");
    if (c === "shield") return t("wheel.categoryShield");
    if (c === "move") return t("wheel.categoryMove");
    return t("wheel.categoryMystic");
  }

  function isDisabled(id: number): boolean {
    return disabled || disabledSet.has(id);
  }
</script>

<ul class="skills" class:disabled>
  {#each ids as id (id)}
    {@const color = skillColor(id)}
    {@const chipDisabled = isDisabled(id)}
    <li>
      <button
        type="button"
        class="skill-chip"
        style:--cat={color}
        class:click-mode={interaction === "click"}
        draggable={interaction === "drag" && !chipDisabled}
        disabled={chipDisabled}
        onclick={interaction === "click" && !chipDisabled ? () => onPick?.(id) : undefined}
        ondragstart={interaction === "drag" && !chipDisabled
          ? (ev) => onDragStart?.(ev, id)
          : undefined}
        ondragend={interaction === "drag" ? () => onDragEnd?.() : undefined}
        title={`${skillName(id)} — ${categoryLabel(id)}\n${skillDesc(id)}`}
      >
        <svg class="glyph" viewBox="0 0 24 24" aria-hidden="true">
          <use href="#skill-glyph-{id}" />
        </svg>
        <span class="chip-name">{skillName(id)}</span>
        <span class="chip-cat">{categoryLabel(id)}</span>
      </button>
    </li>
  {/each}
</ul>

<style>
  .skills {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4em;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .skill-chip {
    --cat: #888;
    display: grid;
    grid-template-rows: auto auto auto;
    align-items: center;
    justify-items: center;
    gap: 0.1em;
    width: 100%;
    padding: 0.45em 0.35em 0.35em;
    font: inherit;
    background: var(--paper-bg);
    border: 1.5px solid var(--cat);
    border-radius: 6px;
    cursor: grab;
    transition: transform 0.08s ease, box-shadow 0.08s ease, background 0.12s ease;
  }
  .skill-chip.click-mode { cursor: pointer; }
  .skill-chip:hover:not(:disabled) {
    background: color-mix(in srgb, var(--cat) 12%, var(--paper-bg));
    transform: translateY(-1px);
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.08);
  }
  .skill-chip:active:not(:disabled) { cursor: grabbing; }
  .skill-chip.click-mode:active:not(:disabled) { cursor: pointer; }
  .skill-chip:disabled { opacity: 0.4; cursor: not-allowed; }
  .skill-chip .glyph {
    width: 32px;
    height: 32px;
    color: var(--cat);
    stroke-width: 2.4;
  }
  .skill-chip .glyph :global(use) { stroke-width: 2.4; }
  .skill-chip .chip-name {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--paper-ink);
  }
  .skill-chip .chip-cat {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--cat);
  }
  .skills.disabled .skill-chip { opacity: 0.35; cursor: not-allowed; }
</style>
