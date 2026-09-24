<script lang="ts">
  import RollingNumber from "$lib/RollingNumber.svelte";

  let { time = "00:00", date = "", greeting = "", compact = false, reduceMotion = false } = $props<{
    time?: string; date?: string; greeting?: string; compact?: boolean; reduceMotion?: boolean;
  }>();
</script>

<div class="clock-face" class:compact>
  {#if date}<div class="caption"><span>{date}</span></div>{/if}
  <time aria-label={time} class="digits"><RollingNumber value={time} ariaHidden {reduceMotion} /></time>
  {#if greeting}<span class="greeting">{greeting}</span>{/if}
</div>

<style>
  .clock-face{display:flex;flex-direction:column;align-items:flex-start;justify-content:center;min-width:0;user-select:none}
  .digits{display:flex;align-items:baseline;gap:1px;color:#fff;font-family:var(--app-font);font-size:52px;font-weight:700;line-height:.98;letter-spacing:-.04em;font-variant-numeric:tabular-nums;white-space:nowrap}
  .caption{max-width:100%;overflow:hidden;color:rgba(255,255,255,.5);font-size:9px;font-weight:600;line-height:1.2;letter-spacing:.015em;white-space:nowrap}
  .caption span{display:block;overflow:hidden;text-overflow:ellipsis}
  .greeting{margin-top:3px;color:rgba(255,255,255,.42);font-size:9px;line-height:1.2}
  .compact .digits{font-size:31px;letter-spacing:-.06em}.compact .caption{font-size:8px}.compact .greeting{margin-top:3px;font-size:8px}
</style>
