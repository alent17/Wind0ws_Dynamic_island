<script lang="ts">
  import { Cloud, CloudFog, CloudLightning, CloudRain, CloudSnow, CloudSun, Sun } from "lucide-svelte";

  let {
    code = null,
    size = 16,
    strokeWidth = 1.8,
  } = $props<{
    code?: number | null;
    size?: number;
    strokeWidth?: number;
  }>();
</script>

{#if code === 0}
  <Sun {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null && code <= 3}
  <CloudSun {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null && code <= 48}
  <CloudFog {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null && code <= 67}
  <CloudRain {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null && code <= 77}
  <CloudSnow {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null && code <= 86}
  <CloudRain {size} {strokeWidth} aria-hidden="true" />
{:else if code !== null}
  <CloudLightning {size} {strokeWidth} aria-hidden="true" />
{:else}
  <Cloud {size} {strokeWidth} aria-hidden="true" />
{/if}
