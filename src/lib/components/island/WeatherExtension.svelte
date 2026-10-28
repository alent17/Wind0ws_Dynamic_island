<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowLeft, Search, X, RefreshCw } from "lucide-svelte";
  import { idleApi } from "$lib/api/idle";
  import type { WeatherLocation, WeatherLocationCandidate, WeatherSnapshot } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import WeatherIcon from "$lib/components/media/WeatherIcon.svelte";

  let { onBack } = $props<{onBack:()=>void}>();
  const storageKey = "isle.weather-extension.locations";
  let locations = $state<WeatherLocation[]>([]);
  let snapshots = $state<Record<string, WeatherSnapshot>>({});
  let failed = $state<Record<string, boolean>>({});
  let query = $state("");
  let results = $state<WeatherLocationCandidate[]>([]);
  let searching = $state(false);
  let searchError = $state(false);
  let searched = $state(false);
  let loading = $state<Record<string, boolean>>({});
  let searchRevision = 0;
  let mounted = false;
  const t = (key:TranslationKey) => translate(key, {}, $locale);
  const key = (location:WeatherLocation) => `${location.latitude},${location.longitude}`;
  const label = (item:WeatherLocationCandidate) => [...new Set([item.name,item.admin2,item.admin1,item.country].filter(Boolean))].join(" · ");
  function persist() { try {localStorage.setItem(storageKey,JSON.stringify(locations));} catch { /* Storage can be unavailable in preview. */ } }
  async function refresh(location:WeatherLocation) {
    const id=key(location);
    loading = {...loading,[id]:true}; failed = {...failed,[id]:false};
    try {const value=await idleApi.getWeatherForLocation(location);if(mounted && locations.some(item=>key(item)===id)) snapshots={...snapshots,[id]:value};}
    catch {if(mounted && locations.some(item=>key(item)===id)) failed={...failed,[id]:true};}
    finally {if(mounted) loading={...loading,[id]:false};}
  }
  async function search() {
    const text=query.trim(), revision=++searchRevision;
    results=[]; searchError=false; searched=Boolean(text);
    if (!text) {searching=false;return;}
    searching=true;
    try {const found=await idleApi.searchLocations(text,$locale === "zh-CN" ? "zh" : $locale);if(mounted && revision===searchRevision) results=found;}
    catch {if(mounted && revision===searchRevision) searchError=true;}
    finally {if(mounted && revision===searchRevision) searching=false;}
  }
  function add(item:WeatherLocationCandidate) {
    if (locations.length>=3 || locations.some(location=>key(location)===key(item))) return;
    const location={name:label(item),latitude:item.latitude,longitude:item.longitude};
    locations=[...locations,location];persist();query="";results=[];searched=false;searchRevision++;void refresh(location);
  }
  function remove(location:WeatherLocation) {locations=locations.filter(item=>key(item)!==key(location));persist();}
  onMount(() => {
    mounted=true;
    try {const data=JSON.parse(localStorage.getItem(storageKey) ?? "[]");if(Array.isArray(data)) {const seen=new Set<string>();locations=data.filter((item:WeatherLocation)=>{if(!item || typeof item.name!=="string" || !Number.isFinite(item.latitude) || !Number.isFinite(item.longitude) || Math.abs(item.latitude)>90 || Math.abs(item.longitude)>180 || seen.has(key(item))) return false;seen.add(key(item));return true;}).slice(0,3);}} catch {locations=[];}
    for(const location of locations) void refresh(location);
    return () => {mounted=false;searchRevision++;};
  });
</script>
<section class="weather-extension" aria-label={t("weatherPlacesTitle")}>
  <header><button class="icon-button" type="button" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button><div><span>{t("weather")}</span><strong>{t("weatherPlacesTitle")}</strong></div><span class="count">{locations.length}/3</span></header>
  <div class="search-area" data-stop-toggle>
    <div class="search"><Search size={21}/><input aria-label={t("searchCity")} placeholder={t("cityExample")} bind:value={query} onkeydown={(event)=>{event.stopPropagation();if(event.key==="Enter"){event.preventDefault();void search();}}}/><button type="button" onclick={() => void search()} disabled={searching || !query.trim() || locations.length>=3}>{t("searchCity")}</button></div>
    {#if locations.length>=3}<p class="message">{t("weatherPlacesLimit")}</p>{:else if searching}<p class="message">{t("searching")}</p>{:else if searchError}<p class="message error">{t("weatherFailed")}</p>{:else if searched && !results.length}<p class="message">{t("noCity")}</p>{/if}
    {#if results.length && locations.length<3}<div class="results" role="group" aria-label={t("searchCity")} onwheel={(event)=>event.stopPropagation()}>{#each results as item}<button type="button" disabled={locations.some(location=>key(location)===key(item))} onclick={()=>add(item)}><strong>{item.name}</strong><span>{label(item)}</span></button>{/each}</div>{/if}
  </div>
  <div class="places" data-stop-toggle onwheel={(event)=>event.stopPropagation()} role="group" aria-label={t("weatherPlacesTitle")}>
    {#if !locations.length}<div class="empty">{t("weatherPlacesEmpty")}</div>{/if}
    {#each locations as location (key(location))}
      {@const snapshot = snapshots[key(location)]}
      <article class="place">
        <div class="place-top"><strong title={location.name}>{location.name}</strong><div class="actions"><button type="button" class="icon-button refresh" aria-label={t("weatherRefresh")} disabled={loading[key(location)]} onclick={()=>void refresh(location)}><RefreshCw size={18}/></button><button type="button" class="icon-button remove" aria-label={`${t("weatherRemove")} ${location.name}`} onclick={()=>remove(location)}><X size={20}/></button></div></div>
        {#if loading[key(location)] && !snapshot}<p class="state">{t("weatherLoading")}</p>{:else if failed[key(location)] && !snapshot}<p class="state error">{t("weatherUnavailable")}</p>{:else if snapshot}
          <div class="conditions"><WeatherIcon code={snapshot.weatherCode} size={36}/><strong>{snapshot.weatherTemperature === null ? "--" : Math.round(snapshot.weatherTemperature)}°</strong><div class="daily">{#each snapshot.weatherForecast.slice(0,3) as day}<span>{day.date.slice(5)} <b>{Math.round(day.temperatureMax)}°</b> / {Math.round(day.temperatureMin)}°</span>{/each}</div></div>
          {#if failed[key(location)]}<span class="state error">{t("weatherUnavailable")}</span>{/if}
        {/if}
      </article>
    {/each}
  </div>
</section>
<style>
  .weather-extension{display:flex;flex-direction:column;box-sizing:border-box;width:100%;height:100%;padding:calc(26px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px)) calc(24px + var(--extension-block-inset,0px));color:#fff;font-family:var(--app-font,system-ui)}
  header{display:flex;align-items:center;gap:14px;flex:none;margin-bottom:16px}header div{display:flex;flex:1;flex-direction:column;gap:3px}header div span{font-size:14px;color:#ffffff80}header strong{font-size:24px}.count{color:#ffffff80;font-size:15px}.icon-button{display:grid;place-items:center;flex:none;width:40px;height:40px;border:0;border-radius:12px;background:transparent;color:#fff;cursor:pointer}.remove{color:#ff6b6b}.icon-button:disabled{opacity:.4}.search-area{position:relative;z-index:3;flex:none}.search{display:flex;align-items:center;gap:10px;padding:0 12px;border:1px solid #ffffff38;border-radius:12px;color:#ffffff99}.search input{flex:1;min-width:0;height:44px;border:0;outline:0;background:transparent;color:#fff;font:16px var(--app-font,system-ui)}.search button{border:0;background:transparent;color:#fff;font-size:15px;cursor:pointer}.search button:disabled{opacity:.35;cursor:default}.message{margin:6px 4px;font-size:13px;color:#ffffff99}.error{color:#ffb4ab}.results{position:absolute;top:50px;left:0;right:0;max-height:215px;overflow-y:auto;overscroll-behavior:contain;padding:6px;border:1px solid #ffffff30;border-radius:12px;background:#171719;box-shadow:0 8px 22px #000a}.results button{display:flex;flex-direction:column;gap:3px;width:100%;padding:9px 10px;border:0;border-radius:8px;background:transparent;color:#fff;text-align:left;cursor:pointer}.results button span{color:#ffffff99;font-size:12px}.results button:disabled{opacity:.35;cursor:default}
  .places{display:flex;flex-direction:column;flex:1;min-height:0;overflow-y:auto;overscroll-behavior:contain;margin-top:10px}.empty{margin:auto;text-align:center;color:#ffffff80;font-size:16px}.place{padding:8px 4px 12px;border-bottom:1px solid #ffffff24}.place-top{display:flex;align-items:center;gap:8px}.place-top>strong{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:18px}.actions{display:flex}.conditions{display:flex;align-items:center;gap:12px}.conditions>strong{font-size:36px;line-height:1}.daily{display:flex;flex:1;justify-content:flex-end;gap:12px;color:#ffffff99;font-size:13px;white-space:nowrap}.daily b{color:#fff}.state{margin:5px 0;font-size:13px;color:#ffffff99}@media(max-width:500px){.daily{gap:5px;font-size:11px}}
</style>
