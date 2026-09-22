<script lang="ts">
  let { time = "00:00", date = "", greeting = "", compact = false } = $props<{
    time?: string; date?: string; greeting?: string; compact?: boolean;
  }>();
  const parts = $derived(time.split(":"));
</script>

<div class="comic-clock" class:compact>
  <time aria-label={time} class="digits">
    <span class="hours">{parts[0]}</span><span class="colon" aria-hidden="true">:</span><span class="minutes">{parts[1] ?? "00"}</span>
  </time>
  <div class="caption"><span>{date}</span></div>
  {#if greeting}<span class="greeting">{greeting}</span>{/if}
</div>

<style>
  .comic-clock{position:relative;display:flex;flex-direction:column;align-items:flex-start;justify-content:center;min-width:0;padding:5px 8px 8px 4px;isolation:isolate;color:#fff;user-select:none}
  .digits{display:flex;align-items:baseline;gap:2px;font-family:"Alibaba PuHuiTi Heavy","MiSans",sans-serif;font-weight:900;font-size:52px;line-height:1.08;letter-spacing:-.03em;font-variant-numeric:tabular-nums;white-space:nowrap;transform:rotate(-4deg) skewX(-7deg);transform-origin:left center;filter:drop-shadow(3px 4px 0 #f28b31)}
  .hours,.minutes{-webkit-text-stroke:1px #fff;paint-order:stroke fill;text-shadow:2px 2px 0 #000}
  .minutes{color:#f7bd76;-webkit-text-stroke:1px #f7bd76;transform:translateY(-2px)}
  .colon{font-size:.7em;align-self:center;color:#fff;margin:0 1px}
  .caption{position:relative;margin-top:9px;padding:4px 7px;background:#fff;color:#171311;font-size:10px;font-weight:700;line-height:1.2;transform:rotate(-2deg);white-space:nowrap;max-width:100%;box-sizing:border-box}
  .caption span{display:block;overflow:hidden;text-overflow:ellipsis}
  .greeting{margin-top:7px;font-size:10px;color:#c9c1b8;line-height:1.2}
  .compact .digits{font-size:33px;gap:0}.compact .caption{font-size:9px;padding:3px 4px}.compact .greeting{font-size:9px}
</style>
