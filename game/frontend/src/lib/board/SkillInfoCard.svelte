<script lang="ts">
  // Info card shown while hovering a wheel slice. Renders the skill's name,
  // category, cost, range, and description from i18n. For modifier slices
  // (Focus / Charge) we surface that they're themselves skills (ids 14 / 15)
  // and whether they're currently staged. For the End-Phase slice we show
  // a short explanation.

  import { SKILLS, CATEGORY_COLOR, type SkillCategory } from "$lib/engine/skills";
  import { t } from "$lib/state/i18n";
  import type { SliceKind } from "$lib/board/SkillWheel.svelte";

  interface Props {
    slice: SliceKind;
    /** Whether the corresponding modifier is currently staged on the
     * position. Used for the "staged" badge on Focus / Charge cards. */
    focusActive: boolean;
    chargeActive: boolean;
    /** Whether the slice is currently armed (target-pending). Drives the
     * "armed — click a target" hint on a skill card. */
    armed: boolean;
  }

  let { slice, focusActive, chargeActive, armed }: Props = $props();

  // Resolve i18n + category for skill / modifier-badge / end-phase slices.
  const info = $derived.by(() => {
    if (slice.kind === "skill") {
      const s = SKILLS[slice.skillId];
      if (!s) return null;
      return {
        name: t(`skills.${s.key}.name`),
        desc: t(`skills.${s.key}.desc`),
        category: s.category as SkillCategory,
        cost: s.cost,
        range: s.defaultRange,
        staged: false,
      };
    }
    if (slice.kind === "modifierBadge") {
      // Hover-only marker shown when a modifier is currently active on
      // pendingModifiers. Cast was already done from the piece's skill
      // slot — this card explains what's about to apply.
      const id = slice.modifier === "focus" ? 14 : 15;
      const s = SKILLS[id];
      if (!s) return null;
      const stagedKey = slice.modifier === "focus"
        ? "wheel.focusStaged"
        : "wheel.chargeStaged";
      return {
        name: t(`skills.${s.key}.name`),
        desc: t(stagedKey),
        category: s.category as SkillCategory,
        cost: null,
        range: null,
        staged: true,
      };
    }
    // endphase
    return {
      name: t("wheel.endphase.name"),
      desc: t("wheel.endphase.desc"),
      category: null,
      cost: null,
      range: null,
      staged: false,
    };
  });

  const categoryLabel = $derived.by(() => {
    if (!info?.category) return "";
    switch (info.category) {
      case "strike": return t("wheel.categoryStrike");
      case "shield": return t("wheel.categoryShield");
      case "move":   return t("wheel.categoryMove");
      case "mystic": return t("wheel.categoryMystic");
    }
  });

  const accent = $derived(info?.category ? CATEGORY_COLOR[info.category] : "#5a4a3a");
</script>

{#if info}
  <article class="info-card" style:--accent={accent}>
    <header>
      <h3>{info.name}</h3>
      {#if info.category}
        <span class="category">{categoryLabel}</span>
      {/if}
    </header>
    {#if slice.kind !== "endphase"}
      <ul class="stats">
        {#if info.cost != null}
          <li>{t("wheel.cost", { n: info.cost })}</li>
        {/if}
        {#if info.range != null && info.range > 0}
          <li>{t("wheel.range", { n: info.range })}</li>
        {/if}
      </ul>
    {/if}
    <p class="desc">{info.desc}</p>
    {#if info.staged}
      <p class="badge staged">● {t("wheel.staged")}</p>
    {/if}
    {#if armed}
      <p class="badge armed">⌖ {t("wheel.armed")}</p>
    {/if}
  </article>
{/if}

<style>
  .info-card {
    --accent: #5a4a3a;
    min-width: 220px;
    max-width: 280px;
    padding: 0.55em 0.75em 0.6em;
    border: 1.5px solid var(--accent);
    border-left-width: 4px;
    border-radius: 6px;
    background: var(--paper-bg, #f3ecd9);
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.08);
    font-size: 0.88rem;
    line-height: 1.35;
    animation: card-fade 140ms ease-out;
  }
  @keyframes card-fade {
    from { opacity: 0; transform: translateY(-3px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.6em;
    margin-bottom: 0.25em;
  }
  header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 700;
    color: var(--accent);
  }
  .category {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--paper-ink-soft, #6a6055);
  }
  .stats {
    display: flex;
    gap: 0.7em;
    margin: 0 0 0.3em;
    padding: 0;
    list-style: none;
    font-size: 0.78rem;
    color: var(--paper-ink-soft, #6a6055);
  }
  .stats li {
    padding: 0.05em 0.45em;
    border: 1px solid var(--paper-line, #b0a47a);
    border-radius: 3px;
    background: var(--paper-square-light, #ece2c8);
  }
  .desc {
    margin: 0;
    color: var(--paper-ink, #1c1a17);
  }
  .badge {
    margin: 0.35em 0 0;
    padding: 0.1em 0.5em;
    border-radius: 3px;
    font-size: 0.78rem;
    font-weight: 600;
    display: inline-block;
  }
  .badge.staged {
    color: var(--accent);
    background: rgba(138, 74, 189, 0.12);
  }
  .badge.armed {
    color: var(--accent);
    background: rgba(204, 58, 42, 0.12);
    margin-left: 0.4em;
  }
</style>
