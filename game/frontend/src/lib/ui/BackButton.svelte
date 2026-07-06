<script lang="ts">
  // Shared back-button used by every top-level route. Keeps the visual
  // language consistent (boxed link, subtle border) and centralises the
  // sfx-on-click behaviour. Routes that need custom teardown (e.g. leaving
  // a multiplayer lobby) pass their own `onclick`; the click SFX is played
  // by the component so callers don't have to remember.

  import { t } from "$lib/state/i18n";
  import { sfx } from "$lib/audio/sfx";

  interface Props {
    /** Destination. Defaults to the parent route (`../`) which is correct
     *  for every top-level page. */
    href?: string;
    /** Optional extra work to run on click (e.g. leave a lobby). The SFX
     *  fires unconditionally before this runs. */
    onclick?: (ev: MouseEvent) => void;
    /** Override the label. Defaults to a generic "← back". */
    label?: string;
  }

  let { href = "../", onclick, label }: Props = $props();

  function handleClick(ev: MouseEvent): void {
    sfx.play("click");
    onclick?.(ev);
  }
</script>

<a class="back" {href} onclick={handleClick}>
  {label ?? t("app.back")}
</a>

<style>
  .back {
    display: inline-block;
    color: inherit;
    text-decoration: none;
    padding: 0.2em 0.6em;
    border: 1.5px solid var(--paper-line-strong);
    border-radius: 5px;
    font: inherit;
    line-height: 1.4;
    background: var(--paper-bg);
  }
  .back:hover {
    background: color-mix(in srgb, var(--paper-line) 40%, var(--paper-bg));
  }
</style>
