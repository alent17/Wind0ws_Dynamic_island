<script lang="ts">
  import { onMount } from "svelte";
  import { CalendarDays } from "lucide-svelte";
  import { locale, translate } from "$lib/i18n";
  import { calendarSevenDays, calendarToday } from "$lib/calendarDates";

  let { timeZone = "system", onExpand } = $props<{ timeZone?: string; onExpand?: () => void }>();
  let now = $state(Date.now());
  const t = (key: Parameters<typeof translate>[0]) => translate(key, {}, $locale);
  const today = $derived(calendarToday(now, timeZone));
  const days = $derived(calendarSevenDays(today));
  const month = $derived(new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",timeZone:"UTC"}).format(today));
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
  <div class="calendar-heading"><span><CalendarDays size={18}/>{t("calendar")}</span><span>{month}</span></div>
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
  .calendar-heading{display:flex;justify-content:space-between;align-items:center;gap:8px;font-size:16px;color:#ffffff99}.calendar-heading span:first-child{display:flex;align-items:center;gap:6px}.calendar-heading span:last-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .seven-days{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));align-items:center;flex:1;min-height:0;gap:4px}.date-cell{display:flex;min-width:0;flex-direction:column;align-items:center;justify-content:center;gap:6px}.weekday{font-size:15px;color:#ffffff99;white-space:nowrap}.date-cell strong{display:grid;place-items:center;width:46px;height:46px;max-width:100%;border-radius:10px;font-size:24px;line-height:1;font-variant-numeric:tabular-nums}.date-cell strong.today{background:#193a8e;color:#fff}
  .calendar-widget:focus-visible{outline:2px solid #91caff;outline-offset:3px;border-radius:8px}
</style>
