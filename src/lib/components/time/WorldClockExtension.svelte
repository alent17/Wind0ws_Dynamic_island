<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { ArrowLeft } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { clockTimeZoneCity } from "$lib/timeZones";
  import { resolveWorldClockZone, WORLD_CLOCK_CITIES } from "$lib/worldClockCities";

  const selectedCityKey = "isle.world-clock.selected-city";
  let { timeZone = "system", onBack } = $props<{timeZone?:string;onBack:()=>void}>();
  let selectedZone = $state(untrack(() => resolveWorldClockZone(timeZone)));
  let selectedId = $state<string|null>(untrack(() => WORLD_CLOCK_CITIES.find(city => city.timeZone === selectedZone)?.id ?? null));
  let timestamp = $state(Date.now());
  let timeOffset = $state(0);
  let drag: {pointerId:number; y:number; offset:number} | null = null;
  const t = (key:TranslationKey) => translate(key, {}, $locale);
  const referenceZone = $derived(resolveWorldClockZone(timeZone));
  const referenceCity = $derived(clockTimeZoneCity(timeZone, $locale));
  const displayTimestamp = $derived(timestamp + timeOffset);
  const selectedCity = $derived(WORLD_CLOCK_CITIES.find(city => city.id === selectedId));
  const cityName = $derived(selectedCity?.names[$locale] ?? clockTimeZoneCity(selectedZone, $locale));
  function formatted(zone:string) {
    return {
      time: new Intl.DateTimeFormat($locale, {timeZone:zone,hour:"2-digit",minute:"2-digit",second:"2-digit",hourCycle:"h23"}).format(displayTimestamp),
      date: new Intl.DateTimeFormat($locale, {timeZone:zone,year:"numeric",month:"long",day:"numeric",weekday:"long"}).format(displayTimestamp),
      offset: new Intl.DateTimeFormat("en", {timeZone:zone,timeZoneName:"shortOffset"}).formatToParts(displayTimestamp).find(part => part.type === "timeZoneName")?.value ?? "GMT",
    };
  }
  const selectedTime = $derived(formatted(selectedZone));
  const referenceTime = $derived(formatted(referenceZone));
  const majorCities = new Set(["Asia/Shanghai", "Asia/Tokyo", "Europe/London", "Europe/Paris", "America/Los_Angeles", "America/New_York", "Asia/Kolkata", "Europe/Moscow", "shanghai"]);

  function startDrag(event:PointerEvent) {
    if (event.button !== 0) return;
    event.stopPropagation();
    drag = {pointerId:event.pointerId,y:event.clientY,offset:timeOffset};
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }
  function moveDrag(event:PointerEvent) {
    if (!drag || drag.pointerId !== event.pointerId) return;
    event.stopPropagation();
    timeOffset = drag.offset + Math.round(drag.y - event.clientY) * 60 * 1000;
  }
  function endDrag(event:PointerEvent) { if (drag?.pointerId === event.pointerId) drag = null; }
  function adjustKey(event:KeyboardEvent) {
    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
    event.preventDefault(); event.stopPropagation();
    timeOffset += (event.key === "ArrowUp" ? 1 : -1) * 60 * 1000;
  }
  function adjustWheel(event:WheelEvent) {
    if (event.ctrlKey || event.deltaY === 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
    event.preventDefault(); event.stopPropagation();
    const minutes = Math.min(30, Math.max(1, Math.round(Math.abs(event.deltaY) / 24)));
    timeOffset += (event.deltaY < 0 ? 1 : -1) * minutes * 60 * 1000;
  }

  function chooseCity(id:string, zone:string) {
    selectedId = id; selectedZone = zone;
    try { localStorage.setItem(selectedCityKey, id); } catch { /* Storage may be unavailable in a preview. */ }
  }
  $effect(() => {const zone = timeZone; untrack(() => {
    let savedId: string | null = null;
    try { savedId = localStorage.getItem(selectedCityKey); } catch { /* Fall back to the configured time zone. */ }
    const saved = WORLD_CLOCK_CITIES.find(city => city.id === savedId);
    selectedZone = saved?.timeZone ?? resolveWorldClockZone(zone);
    selectedId = saved?.id ?? WORLD_CLOCK_CITIES.find(city => city.timeZone === selectedZone)?.id ?? null;
  });});
  onMount(() => {
    let timer:ReturnType<typeof setTimeout>;
    const update = () => {timestamp = Date.now();timer = setTimeout(update, 1000 - timestamp % 1000);};
    update();
    return () => clearTimeout(timer);
  });
</script>

<section class="world-clock-extension" aria-label={t("worldClockTitle")}>
  <header>
    <button type="button" class="back" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button>
    <div><span>{t("clock")}</span><strong>{t("worldClockTitle")}</strong></div>
  </header>
  <div class="clock-display">
    {#each [{name:cityName, value:selectedTime, side:"selected"}, {name:referenceCity, value:referenceTime, side:"reference"}] as clock (clock.side)}
      <div class="clock-column" data-clock-side={clock.side}>
        <div class="clock-city"><strong>{clock.name}</strong><span>{clock.value.offset}</span></div>
        <button type="button" class="clock-digits" aria-label={`${clock.name} ${clock.value.time} · ${t("worldClockAdjust")}`} title={t("worldClockAdjust")} onpointerdown={startDrag} onpointermove={moveDrag} onpointerup={endDrag} onpointercancel={endDrag} onkeydown={adjustKey} onwheel={adjustWheel}>{clock.value.time}</button>
        <span class="clock-date">{clock.value.date}</span>
      </div>
    {/each}
    {#if timeOffset !== 0}<button type="button" class="reset-time" onclick={() => {timeOffset = 0; timestamp = Date.now();}}>{t("worldClockNow")}</button>{/if}
  </div>
  <div class="city-list" data-stop-toggle role="group" aria-label={t("worldClockCities")} onwheel={(event) => event.stopPropagation()}>
    {#each WORLD_CLOCK_CITIES as city,index (city.id)}
      <button type="button" class="city-chip" class:prominent={index % 7 === 0 || index % 11 === 0} class:major={majorCities.has(city.id)} class:selected={city.id === selectedId} data-city={city.id} aria-pressed={city.id === selectedId} onclick={() => chooseCity(city.id,city.timeZone)}>{city.names[$locale]}</button>
    {/each}
  </div>
</section>

<style>
  .world-clock-extension{display:flex;flex-direction:column;box-sizing:border-box;width:100%;height:100%;padding:calc(30px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px)) calc(24px + var(--extension-block-inset,0px));color:#fff;font-family:var(--app-font,system-ui);user-select:none}
  header{display:flex;align-items:center;gap:14px;flex:none;margin-bottom:16px}header>div{display:flex;flex-direction:column;gap:3px}header span{font-size:14px;color:#ffffff80}header strong{font-size:24px;line-height:1.1}.back{display:grid;place-items:center;flex:none;width:42px;height:42px;padding:0;border:0;border-radius:12px;color:#fff;background:transparent;cursor:pointer}.back:hover,.back:focus-visible{outline:none;background:#ffffff1a}
  .clock-display{position:relative;display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:36px;flex:none;padding:2px 4px 18px;border-bottom:1px solid #ffffff24}.clock-column{min-width:0;display:flex;flex-direction:column;align-items:flex-start;gap:7px}.clock-city{width:100%;display:flex;align-items:baseline;justify-content:space-between;gap:12px}.clock-city strong{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:22px}.clock-city span{flex:none;padding:4px 8px;border-radius:7px;background:#002fa7;color:#fff;font-size:15px;font-variant-numeric:tabular-nums}.clock-digits{padding:0;border:0;color:#fff;background:transparent;font-size:58px;font-weight:700;line-height:1;letter-spacing:-.04em;font-variant-numeric:tabular-nums;white-space:nowrap;cursor:default;touch-action:none}.clock-digits:focus-visible{outline:2px solid #002fa7;outline-offset:4px}.clock-date{font-size:15px;color:#ffffffa6}.reset-time{position:absolute;right:4px;bottom:12px;border:0;background:transparent;color:#91caff;cursor:pointer}
  .city-list{display:flex;flex:1;min-height:0;align-content:safe center;align-items:center;justify-content:center;flex-wrap:wrap;gap:5px 7px;overflow-y:auto;overscroll-behavior:contain;padding:17px 2px 4px;scrollbar-width:thin;scrollbar-color:#ffffff40 transparent}.city-list::-webkit-scrollbar{width:4px}.city-list::-webkit-scrollbar-thumb{border-radius:99px;background:#ffffff40}.city-chip{flex:none;max-width:100%;min-height:38px;padding:5px 8px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;border:0;border-radius:8px;color:#ffffffd9;background:transparent;font:500 21px/1.2 var(--app-font,system-ui);cursor:pointer}.city-chip.prominent{font-size:26px;font-weight:700}.city-chip.major{background:#fff;color:#000;font-weight:700}.city-chip:hover,.city-chip:focus-visible{outline:none;background:#ffffff1a}.city-chip.selected{background:#002fa7;color:#fff;font-weight:700}
</style>
