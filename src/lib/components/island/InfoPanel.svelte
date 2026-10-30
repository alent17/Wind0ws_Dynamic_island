<script lang="ts">
  import WidgetClock from "$lib/components/time/WidgetClock.svelte";
  import WidgetWeather from "$lib/components/island/WidgetWeather.svelte";
  import ComicClock from "$lib/components/time/ComicClock.svelte";
  import WeatherIcon from "$lib/components/media/WeatherIcon.svelte";
  import { locale, translate } from "$lib/i18n";
  import type { WeatherForecastDay } from "$lib/api/types";
  let { compact=false, widget=false, page, time, timeZone="system", city="", temperature=null, code=null, forecast=[], updatedAt=null, loading=false, failed=false, reduceMotion=false, onSettings, onExpandClock, onExpandWeather } = $props<{
    compact?:boolean;widget?:boolean;page:"clock"|"weather";time:string;timeZone?:string;city?:string;temperature?:number|null;code?:number|null;
    forecast?:WeatherForecastDay[];updatedAt?:number|null;loading?:boolean;failed?:boolean;reduceMotion?:boolean;onSettings?:()=>void;onExpandClock?:()=>void;onExpandWeather?:()=>void;
  }>();
  const t = (key: Parameters<typeof translate>[0]) => translate(key,{},$locale);
  const options = $derived(timeZone === "system" ? {} : {timeZone});
  const date = $derived.by(() => { time; return new Intl.DateTimeFormat($locale,{...options,year:"numeric",month:"long",day:"numeric",weekday:"short"}).format(new Date()); });
  const updated = $derived(updatedAt ? new Intl.DateTimeFormat($locale,{hour:"2-digit",minute:"2-digit"}).format(new Date(updatedAt < 1e12 ? updatedAt*1000 : updatedAt)) : "");
  const expand = () => page === "clock" ? onExpandClock : onExpandWeather;
  function clockExpandGesture(node:HTMLElement) {
    let gesture:{id:number;x:number;y:number}|null=null;
    const wheel=(event:WheelEvent)=>{if(!expand()||event.ctrlKey||event.deltaY<=0||Math.abs(event.deltaX)>Math.abs(event.deltaY))return;event.preventDefault();event.stopPropagation();expand()?.();};
    const down=(event:PointerEvent)=>{if(!expand()||!event.isPrimary||event.button!==0||(event.target as HTMLElement).closest("button,input,[role=slider]"))return;gesture={id:event.pointerId,x:event.clientX,y:event.clientY};node.setPointerCapture(event.pointerId);};
    const move=(event:PointerEvent)=>{if(!gesture||gesture.id!==event.pointerId)return;const dx=event.clientX-gesture.x,dy=event.clientY-gesture.y;if(dy>28&&dy>Math.abs(dx)){gesture=null;expand()?.();}else if(Math.abs(dx)>28||dy< -28)gesture=null;};
    const end=(event:PointerEvent)=>{if(gesture?.id===event.pointerId)gesture=null;};
    node.addEventListener("wheel",wheel,{passive:false});node.addEventListener("pointerdown",down);node.addEventListener("pointermove",move);node.addEventListener("pointerup",end);node.addEventListener("pointercancel",end);
    return {destroy(){node.removeEventListener("wheel",wheel);node.removeEventListener("pointerdown",down);node.removeEventListener("pointermove",move);node.removeEventListener("pointerup",end);node.removeEventListener("pointercancel",end);}};
  }
</script>
<!-- svelte-ignore a11y_no_noninteractive_tabindex: The panel becomes interactive only when expansion is available. -->
<div class="info-panel" class:compact class:weather={page === "weather"} class:widget-panel={widget} data-stop-toggle={expand() ? "" : undefined} use:clockExpandGesture role={expand() ? "button" : undefined} tabindex={expand() ? 0 : undefined} aria-label={expand() ? t(page) : undefined} onkeydown={(event)=>{if(event.target === event.currentTarget && expand() && (event.key === "Enter" || event.key === " " || event.key === "ArrowDown")){event.preventDefault();event.stopPropagation();expand()?.();}}}>
  {#if page === "clock"}
    {#if widget}<WidgetClock {timeZone} {reduceMotion} />{:else}<ComicClock {compact} {time} {date} {reduceMotion} />{/if}
  {:else if widget}
    <WidgetWeather {city} {temperature} {code} {forecast} {updatedAt} {loading} {failed} {onSettings} />
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
</div>
<style>
  .info-panel{width:100%;min-width:0;color:#f3f6fa;display:flex;flex-direction:column;justify-content:center;gap:12px;font-family:var(--app-font,system-ui)}
  .widget-panel{height:100%}
  .info-panel[role="button"]:focus-visible{outline:2px solid #002fa7;outline-offset:3px;border-radius:8px}
  .current-weather{display:flex;align-items:center;justify-content:space-between;gap:12px}.current-weather>div{min-width:0;display:flex;align-items:baseline;gap:12px}.current-weather strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:14px}.current-weather span{font-size:30px;font-weight:700}
  .forecast{display:flex;justify-content:space-between;gap:16px}.forecast>div{display:flex;flex-direction:column;align-items:center;gap:5px;font-size:13px}.forecast small,.updated,p{color:rgba(255,255,255,.5);font-size:13px}p{margin:0;line-height:1.5}button{align-self:flex-start;border:0;border-radius:12px;background:transparent;color:#fff;padding:10px;cursor:pointer}button:hover,button:focus-visible{background:rgba(255,255,255,.1)}
  .compact .current-weather{flex-direction:column-reverse;align-items:flex-start;gap:6px}
  .compact .current-weather>div{flex-direction:column;gap:4px}
  .compact .current-weather span{font-size:28px}
  .compact .forecast{gap:5px}.compact .forecast>div{font-size:13px}
  .compact{gap:8px}.compact .updated{font-size:13px}
</style>
