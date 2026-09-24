<script lang="ts">
  import { onMount } from "svelte";
  import { rollingDigitDelta } from "$lib/rollingNumber";

  let { value = "0", reduceMotion = false } = $props<{ value?: string; reduceMotion?: boolean }>();

  let digit: number | undefined;
  let position = $state(10);
  let ready = $state(false);
  let recentering = $state(false);
  let systemReduceMotion = $state(false);
  const motionDisabled = $derived(reduceMotion || systemReduceMotion);

  onMount(() => {
    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updatePreference = () => systemReduceMotion = preference.matches;
    updatePreference();
    preference.addEventListener("change", updatePreference);
    return () => preference.removeEventListener("change", updatePreference);
  });

  $effect(() => {
    const next = Number(value) || 0;
    if (digit === undefined) {
      digit = next;
      position = 10 + next;
      ready = true;
      return;
    }
    if (motionDisabled) {
      digit = next;
      position = 10 + next;
      recentering = false;
      return;
    }
    const step = rollingDigitDelta(digit, next);
    digit = next;
    if (step !== 0) position += step;
  });

  function recenter(event: TransitionEvent) {
    if (event.propertyName !== "transform" || recentering || digit === undefined) return;
    const center = 10 + digit;
    if (position === center) return;
    recentering = true;
    position = center;
    requestAnimationFrame(() => requestAnimationFrame(() => recentering = false));
  }
</script>

<span class="digit-window" class:reduce-motion={motionDisabled} aria-hidden="true">
  <span class="digit-strip" class:ready class:recenter={recentering} style={`transform:translateY(-${position}em)`} ontransitionend={recenter}>
    {#each Array.from({ length: 30 }, (_, index) => index % 10) as number}
      <span>{number}</span>
    {/each}
  </span>
</span>

<style>
  .digit-window{position:relative;display:inline-block;flex:none;width:.62em;height:1em;overflow:hidden;vertical-align:baseline;line-height:1em}
  .digit-strip{position:absolute;top:0;left:0;display:flex;width:100%;height:30em;flex-direction:column;opacity:0;transition:transform 280ms cubic-bezier(.22,1,.36,1),opacity 120ms ease;}
  .digit-strip.ready{opacity:1}
  .digit-strip>span{display:block;flex:0 0 1em;height:1em;line-height:1em;text-align:center}
  .digit-strip.recenter{transition:none}
  .reduce-motion .digit-strip,.reduce-motion .digit-strip.recenter{transition:none}
  @media (prefers-reduced-motion:reduce){.digit-strip,.digit-strip.recenter{transition:none}}
</style>
