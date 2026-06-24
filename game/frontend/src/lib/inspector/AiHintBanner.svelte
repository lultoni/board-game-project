<script lang="ts">
  import { decodeAction, ActionKind } from "$lib/engine/action";
  import { skillById } from "$lib/engine/skills";
  import type { AiHint } from "$lib/state/inspector-store.svelte";

  interface Props {
    hint: AiHint;
    onApply: () => void;
    onDismiss: () => void;
  }
  let { hint, onApply, onDismiss }: Props = $props();

  function fmtSquare(sq: number): string {
    const file = String.fromCharCode("a".charCodeAt(0) + (sq % 8));
    const rank = Math.floor(sq / 8) + 1;
    return `${file}${rank}`;
  }

  const description = $derived.by(() => {
    if (hint.best === 0) return "no move found";
    const d = decodeAction(hint.best);
    if (d.kind === ActionKind.EndPhase) return "End phase";
    if (d.kind === ActionKind.EndTurn) return "End turn";
    if (d.kind === ActionKind.Move) {
      return `Move ${fmtSquare(d.src)}→${fmtSquare(d.target)}`;
    }
    if (d.kind === ActionKind.Skill) {
      const info = skillById(d.skillId);
      const name = info?.key ?? `skill${d.skillId}`;
      return `${name} ${fmtSquare(d.src)}→${fmtSquare(d.target)}`;
    }
    return "?";
  });
</script>

<div class="banner">
  <div class="copy">
    <strong>AI suggests:</strong> {description}
    <small>depth {hint.depth} · score {hint.score}</small>
  </div>
  <div class="actions">
    {#if hint.best !== 0}
      <button class="primary" type="button" onclick={onApply}>Apply</button>
    {/if}
    <button type="button" onclick={onDismiss}>Dismiss</button>
  </div>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    padding: 0.6rem 0.9rem;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 6px;
    background: #fff8dc;
    margin-bottom: 0.8rem;
  }
  .copy { flex: 1; display: flex; flex-direction: column; gap: 0.15rem; }
  .copy small { color: var(--paper-ink-soft); }
  .actions { display: flex; gap: 0.4rem; }
  .primary {
    background: var(--accent, #5a7cd6);
    color: #fff;
    border-color: var(--accent, #5a7cd6);
    font-weight: 600;
  }
</style>
