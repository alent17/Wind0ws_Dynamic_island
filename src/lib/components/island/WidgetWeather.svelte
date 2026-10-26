<script lang="ts">
  import WeatherIcon from "$lib/components/media/WeatherIcon.svelte";
  import { locale, translate } from "$lib/i18n";
  import type { WeatherForecastDay } from "$lib/api/types";

  let { city = "", temperature = null, code = null, forecast = [], updatedAt = null, loading = false, failed = false, onSettings } = $props<{
    city?: string; temperature?: number | null; code?: number | null;
    forecast?: WeatherForecastDay[]; updatedAt?: number | null;
    loading?: boolean; failed?: boolean; onSettings?: () => void;
  }>();
  const t = (key: Parameters<typeof translate>[0]) => translate(key, {}, $locale);
  const location = $derived(city.split(/\s*[·•]\s*/));
  const updated = $derived(updatedAt ? new Intl.DateTimeFormat($locale, { hour: "2-digit", minute: "2-digit" }).format(new Date(updatedAt < 1e12 ? updatedAt * 1000 : updatedAt)) : "");
</script>

<div class="widget-weather">
  {#if !city}
    <div class="weather-empty"><WeatherIcon size={40} /><p>{t("weatherChooseCity")}</p><button onclick={onSettings}>{t("settingsTool")}</button></div>
  {:else}
    <div class="weather-hero">
      <div class="weather-location" title={city}><strong>{location[0]}</strong>{#if location.length > 1}<span>{location.slice(1).join(" · ")}</span>{/if}</div>
      <span class="weather-temperature">{temperature === null ? "--" : Math.round(temperature)}<small>°</small></span>
      <div class="weather-symbol"><WeatherIcon {code} size={48} strokeWidth={1.6} /></div>
    </div>
    {#if forecast.length > 1}
      <div class="weather-days" aria-label={t("weatherForecast")}>
        {#each forecast.slice(1, 4) as day}
          <div class="weather-day"><span class="weather-day-date">{day.date.slice(5).replace("-", "/")}</span><WeatherIcon code={day.weatherCode} size={26} /><div class="weather-range"><strong>{Math.round(day.temperatureMax)}°</strong><span>{Math.round(day.temperatureMin)}°</span></div></div>
        {/each}
      </div>
    {/if}
    <div class="weather-status" role="status">
      {#if loading}{t("weatherLoading")}{:else if failed || temperature === null}{t("weatherUnavailable")}{:else if updated}{t("weatherUpdated")} {updated}{/if}
    </div>
  {/if}
</div>

<style>
  .widget-weather{container-type:inline-size;display:flex;flex-direction:column;gap:12px;width:100%;height:100%;min-width:0;padding:4px 0;box-sizing:border-box;color:#f3f6fa;user-select:none}
  .weather-hero{display:flex;align-items:center;gap:16px;flex:1;min-height:0}
  .weather-location{display:flex;flex-direction:column;gap:6px;flex:1;min-width:0}
  .weather-location strong{font-size:clamp(20px,6cqw,28px);font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;line-height:1.2}
  .weather-location span{font-size:14px;color:#ffffff80;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .weather-temperature{flex:none;font-size:clamp(52px,18cqw,80px);font-weight:700;line-height:1;letter-spacing:-.045em;font-variant-numeric:tabular-nums}
  .weather-temperature small{font-size:.65em;vertical-align:top;line-height:1.25}
  .weather-symbol{display:flex;align-items:center;flex:none;color:#ffffffd9}.weather-symbol :global(svg){width:clamp(32px,11cqw,48px);height:auto}
  .weather-days{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));padding-top:12px;border-top:1px solid #ffffff1a}
  .weather-day{display:flex;flex-direction:column;align-items:center;gap:7px;min-width:0}.weather-day+.weather-day{border-left:1px solid #ffffff14}
  .weather-day-date{font-size:16px;color:#ffffffa6;font-variant-numeric:tabular-nums}
  .weather-range{display:flex;align-items:baseline;justify-content:center;gap:10px;font-size:18px;font-variant-numeric:tabular-nums;line-height:1.2}.weather-range strong{font-weight:600}.weather-range span{color:#ffffff73}
  .weather-status{min-height:17px;font-size:13px;line-height:17px;color:#ffffff66;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
  .weather-empty{display:flex;flex:1;flex-direction:column;align-items:center;justify-content:center;gap:14px;color:#ffffff99;font-size:16px}.weather-empty p{margin:0;text-align:center}.weather-empty button{border:1px solid #ffffff30;border-radius:12px;background:#ffffff0d;color:#fff;padding:10px 16px;font:inherit;cursor:pointer}.weather-empty button:hover{background:#ffffff1a}
</style>
