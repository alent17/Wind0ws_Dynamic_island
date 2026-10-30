<script lang="ts">
  import { onMount } from "svelte";
  import { locale, translate } from "$lib/i18n";
  import { calendarSevenDays, calendarToday } from "$lib/calendarDates";

  let { timeZone = "system", onExpand } = $props<{ timeZone?: string; onExpand?: () => void }>();
  let now = $state(Date.now());
  const t = (key: Parameters<typeof translate>[0]) => translate(key, {}, $locale);
  const today = $derived(calendarToday(now, timeZone));
  const days = $derived(calendarSevenDays(today));
  const monthParts = $derived(new Intl.DateTimeFormat($locale, {year:"numeric",month:($locale === "zh-CN" || $locale === "ja") ? "numeric" : "long",timeZone:"UTC"}).formatToParts(today));
  const weekday = (day:number) => new Intl.DateTimeFormat($locale, {weekday:"short",timeZone:"UTC"}).format(day);
  const fullDate = (day:number) => new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",weekday:"long",timeZone:"UTC"}).format(day);

  onMount(() => {
    const timer = setInterval(() => now = Date.now(), 30_000);
    return () => clearInterval(timer);
  });
  function expandGesture(node: HTMLElement) {
    let gesture: {id:number;x:number;y:number} | null = null;
    const wheel = (event: WheelEvent) => {
      if (!onExpand || event.ctrlKey || event.deltaY <= 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
      event.preventDefault(); event.stopPropagation(); onExpand();
    };
    const down = (event: PointerEvent) => {
      if (!onExpand || !event.isPrimary || event.button !== 0) return;
      gesture = {id:event.pointerId,x:event.clientX,y:event.clientY};
      node.setPointerCapture(event.pointerId);
    };
    const move = (event: PointerEvent) => {
      if (!gesture || gesture.id !== event.pointerId) return;
      const dx = event.clientX - gesture.x, dy = event.clientY - gesture.y;
      if (dy > 28 && dy > Math.abs(dx)) { gesture = null; onExpand?.(); }
      else if (Math.abs(dx) > 28 || dy < -28) gesture = null;
    };
    const end = (event:PointerEvent) => { if (gesture?.id === event.pointerId) gesture = null; };
    node.addEventListener("wheel",wheel,{passive:false}); node.addEventListener("pointerdown",down);
    node.addEventListener("pointermove",move); node.addEventListener("pointerup",end); node.addEventListener("pointercancel",end);
    return {destroy() {node.removeEventListener("wheel",wheel);node.removeEventListener("pointerdown",down);node.removeEventListener("pointermove",move);node.removeEventListener("pointerup",end);node.removeEventListener("pointercancel",end);}};
  }
</script>

<div class="calendar-widget" data-stop-toggle use:expandGesture role="button" tabindex={onExpand ? 0 : -1} aria-label={t("calendar")} onkeydown={(event) => {if (onExpand && (event.key === "Enter" || event.key === " " || event.key === "ArrowDown")) {event.preventDefault();event.stopPropagation();onExpand();}}}>
  <div class="calendar-heading">{#if $locale === "zh-CN" || $locale === "ja"}<span class="year-number">{new Date(today).getUTCFullYear()}</span><span>年</span><span class="month-number">{new Date(today).getUTCMonth() + 1}</span><span>月</span>{:else}{#each monthParts as part}<span class:year-number={part.type === "year"} class:month-number={part.type === "month"}>{part.value}</span>{/each}{/if}</div>
  <div class="seven-days" aria-label={t("calendarSevenDays")}>
    {#each days as day}
      <div class="date-cell" title={fullDate(day)} aria-label={fullDate(day)} aria-current={day === today ? "date" : undefined}>
        <span class="weekday">{weekday(day)}</span><strong class:today={day === today}>{new Date(day).getUTCDate()}</strong>
      </div>
    {/each}
  </div>
</div>

<style>
  .calendar-widget{display:flex;flex-direction:column;justify-content:space-between;box-sizing:border-box;width:100%;height:100%;min-height:0;gap:8px;padding:4px 0;color:#fff;user-select:none}
  .calendar-heading{display:flex;align-items:baseline;min-width:0;padding-left:max(0px,calc((100% - 24px) / 14 - 14px));font-size:calc(13px / var(--content-scale,1));font-weight:700;color:#fff;line-height:1.25}.calendar-heading span{white-space:pre}.calendar-heading .year-number{font-size:calc(20px / var(--content-scale,1))}.calendar-heading .month-number{font-size:calc(17px / var(--content-scale,1))}

  .seven-days{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));align-items:center;flex:1;min-height:0;gap:4px}.date-cell{display:flex;min-width:0;flex-direction:column;align-items:center;justify-content:center;gap:6px}.weekday{font-size:15px;color:#ffffff99;white-space:nowrap}.date-cell strong{display:grid;place-items:center;width:46px;height:46px;max-width:100%;border-radius:10px;font-size:24px;line-height:1;font-variant-numeric:tabular-nums}.date-cell strong.today{background:#002fa7;color:#fff}
  .calendar-widget:focus-visible{outline:2px solid #002fa7;outline-offset:3px;border-radius:8px}
</style>
