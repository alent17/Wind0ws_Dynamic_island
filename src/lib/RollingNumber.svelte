<script lang="ts">
  import RollingDigit from "$lib/RollingDigit.svelte";

  let { value = "", class: className = "", ariaHidden = false, reduceMotion = false } = $props<{ value?: string; class?: string; ariaHidden?: boolean; reduceMotion?: boolean }>();
  const characters = $derived(value.split(""));
</script>

<span class={`rolling-number ${className}`} aria-hidden={ariaHidden}>
  <span class="rolling-number-sr">{value}</span>
  <span class="rolling-number-visual" aria-hidden="true">
    {#each characters as character, index (`${index}-${/^\d$/.test(character) ? "digit" : character}`)}
      {#if /^\d$/.test(character)}
        <RollingDigit value={character} {reduceMotion} />
      {:else}
        <span class="static-character" class:colon={character === ":"}>{character}</span>
      {/if}
    {/each}
  </span>
</span>

<style>
  .rolling-number{display:inline-flex;align-items:center;min-width:0;height:1em;line-height:1em;letter-spacing:0;white-space:pre;vertical-align:baseline;font-variant-numeric:tabular-nums}
  .rolling-number-visual{display:inline-flex;align-items:stretch;gap:.015em;height:1em;white-space:pre}
  .static-character{display:block;height:1em;line-height:1em;text-align:center;white-space:pre}
  .static-character.colon{width:.24em;flex:none}
  .rolling-number-sr{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
</style>
