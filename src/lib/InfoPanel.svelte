<script lang="ts">
  import ComicClock from "$lib/ComicClock.svelte";
  import WeatherIcon from "$lib/WeatherIcon.svelte";
  import { locale, translate } from "$lib/i18n";
  import type { WeatherForecastDay } from "$lib/api/types";
  let { page, time, timeZone="system", city="", temperature=null, code=null, forecast=[], updatedAt=null, loading=false, failed=false, reduceMotion=false, onSettings } = $props<{
    page:"clock"|"weather";time:string;timeZone?:string;city?:string;temperature?:number|null;code?:number|null;
    forecast?:WeatherForecastDay[];updatedAt?:number|null;loading?:boolean;failed?:boolean;reduceMotion?:boolean;onSettings?:()=>void;
  }>();
  const t = (key: Parameters<typeof translate>[0]) => translate(key,{},$locale);
  const options = $derived(timeZone === "system" ? {} : {timeZone});
  const date = $derived.by(() => { time; return new Intl.DateTimeFormat($locale,{...options,year:"numeric",month:"long",day:"numeric",weekday:"short"}).format(new Date()); });
  const updated = $derived(updatedAt ? new Intl.DateTimeFormat($locale,{hour:"2-digit",minute:"2-digit"}).format(new Date(updatedAt < 1e12 ? updatedAt*1000 : updatedAt)) : "");
</script>
<section class="info-panel" class:weather={page === "weather"}>
  {#if page === "clock"}
    <ComicClock {time} {date} {reduceMotion} />
  {:else if !city}
    <p>{t("weatherChooseCity")}</p><button onclick={onSettings}>{t("settingsTool")}</button>
  {:else}
    <div class="current-weather"><div><strong title={city}>{city}</strong><span>{temperature === null ? "--" : Math.round(temperature)}°</span></div><WeatherIcon {code} size={36}/></div>
    {#if loading}<p role="status">{t("weatherLoading")}</p>{:else if failed || temperature === null}<p role="status">{t("weatherUnavailable")}</p>{/if}
    <div class="forecast" aria-label={t("weatherForecast")}>
      {#each forecast.slice(1,4) as day}<div><span>{day.date.slice(5)}</span><WeatherIcon code={day.weatherCode} size={19}/><strong>{Math.round(day.temperatureMax)}°</strong><small>{Math.round(day.temperatureMin)}°</small></div>{/each}
    </div>
    {#if updated}<small class="updated">{t("weatherUpdated")} {updated}</small>{/if}
  {/if}
</section>
<style>
  .info-panel{width:100%;min-width:0;color:#f3f6fa;display:flex;flex-direction:column;justify-content:center;gap:12px;font-family:var(--app-font,system-ui)}
  .current-weather{display:flex;align-items:center;justify-content:space-between;gap:12px}.current-weather>div{min-width:0;display:flex;align-items:baseline;gap:12px}.current-weather strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.current-weather span{font-size:30px;font-weight:700}
  .forecast{display:flex;justify-content:space-between;gap:16px}.forecast>div{display:flex;flex-direction:column;align-items:center;gap:5px;font-size:11px}.forecast small,.updated,p{color:rgba(255,255,255,.5);font-size:10px}p{margin:0;line-height:1.5}button{align-self:flex-start;border:0;border-radius:12px;background:transparent;color:#fff;padding:10px;cursor:pointer}button:hover,button:focus-visible{background:rgba(255,255,255,.1)}
</style>
