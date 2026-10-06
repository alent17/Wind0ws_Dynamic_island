<script lang="ts">
  import { onMount } from "svelte";
  let { text, reduceMotion = false } = $props<{text:string;reduceMotion?:boolean}>();
  let viewport: HTMLDivElement;
  let content: HTMLSpanElement;
  let overflow = $state(false);
  let mounted = $state(false);
  let revision = $state(0);
  onMount(() => {
    mounted = true;
    const observer = new ResizeObserver(() => revision++);
    observer.observe(viewport); observer.observe(content);
    return () => { observer.disconnect(); mounted = false; };
  });
  $effect(() => {
    text; revision;
    if (!mounted) return;
    const distance = Math.max(0, content.scrollWidth - viewport.clientWidth);
    overflow = distance > 1;
    if (!overflow || reduceMotion) return;
    const travel = distance / 24 * 1000;
    const duration = travel * 2 + 2400;
    const pause = 1200 / duration, end = (1200 + travel) / duration;
    const animation = content.animate([
      {transform:"translateX(0)",offset:0},
      {transform:"translateX(0)",offset:pause},
      {transform:`translateX(${-distance}px)`,offset:end},
      {transform:`translateX(${-distance}px)`,offset:end+pause},
      {transform:"translateX(0)",offset:1},
    ],{duration,iterations:Infinity,easing:"linear"});
    return () => animation.cancel();
  });
</script>
<div class="marquee-title" class:overflow bind:this={viewport} title={text} aria-label={text}>
  <span bind:this={content} aria-hidden="true">{text}</span>
</div>
<style>
  .marquee-title{min-width:0;width:100%;overflow:hidden;font:700 13px/1.2 var(--app-font,system-ui);margin-bottom:4px;white-space:nowrap}
  .marquee-title span{display:block;width:max-content;white-space:nowrap}
  .overflow{mask-image:linear-gradient(90deg,transparent,#000 10px,#000 calc(100% - 12px),transparent)}
</style>
