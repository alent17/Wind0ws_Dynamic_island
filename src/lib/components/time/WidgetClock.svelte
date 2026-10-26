<script lang="ts">
  import { onMount } from "svelte";
  import RollingNumber from "$lib/components/time/RollingNumber.svelte";
import { locale, translate } from "$lib/i18n";
import { clockTimeZoneCity } from "$lib/timeZones";

let { timeZone = "system", reduceMotion = false } = $props<{
  timeZone?: string; reduceMotion?: boolean;
  }>();
  let timestamp = $state(Date.now());
  onMount(() => {
    let timer: ReturnType<typeof setTimeout>;
    const tick = () => {
      timestamp = Date.now();
      timer = setTimeout(tick, 1000 - timestamp % 1000);
    };
    tick();
  return () => clearTimeout(timer);
  });
  const formatter = $derived(new Intl.DateTimeFormat($locale, {
    ...(timeZone === "system" ? {} : { timeZone }),
    hour: "2-digit", minute: "2-digit", second: "2-digit", hourCycle: "h23",
  }));
  const time = $derived(formatter.format(new Date(timestamp)));
  const calendarMinute = $derived(Math.floor(timestamp / 60_000));
  const calendar = $derived.by(() => {
    const now = new Date(calendarMinute * 60_000);
    const options = timeZone === "system" ? {} : { timeZone };
    const zone = new Intl.DateTimeFormat($locale, { ...options, timeZoneName: "shortOffset" })
      .formatToParts(now).find(part => part.type === "timeZoneName")?.value ?? "";
    return {
      date: new Intl.DateTimeFormat($locale, { ...options, year: "numeric", month: "long", day: "numeric" }).format(now),
      weekday: new Intl.DateTimeFormat($locale, { ...options, weekday: "long" }).format(now),
      zone,
    };
  });
</script>

<div class="widget-clock">
  <div class="clock-heading"><span>{translate("clock", {}, $locale)}</span><span class="timezone-label">{clockTimeZoneCity(timeZone, $locale)} · {calendar.zone}</span></div>
  <time class="widget-time" aria-label={time}><RollingNumber value={time} ariaHidden {reduceMotion} /></time>
  <div class="clock-calendar"><span>{calendar.date}</span><span>{calendar.weekday}</span></div>
</div>

<style>
  .widget-clock{position:relative;container-type:size;display:flex;flex-direction:column;justify-content:space-between;align-items:stretch;width:100%;height:100%;min-width:0;gap:14px;padding:8px 0;box-sizing:border-box;user-select:none}
  .clock-heading,.clock-calendar{display:flex;align-items:baseline;justify-content:space-between;gap:16px;line-height:1.35}
  .clock-heading{color:#ffffff80;font-size:16px;font-weight:500;letter-spacing:.04em}
  .timezone-label{max-width:58%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;text-align:right}
  .widget-time{display:flex;align-items:center;justify-content:center;flex:1;min-height:0;color:#fff;font-family:var(--app-font,system-ui);font-size:max(48px,min(20cqw,calc(100cqh - 80px),144px));font-weight:700;line-height:1;font-variant-numeric:tabular-nums;white-space:nowrap}
  .clock-calendar{font-size:clamp(14px,4.4cqw,20px);font-weight:500;color:#ffffffb3}
  .clock-calendar span:first-child{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .clock-calendar span:last-child{flex:none;color:#fff}
</style>
