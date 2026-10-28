<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { ArrowLeft, Check, ChevronDown, ChevronLeft, ChevronRight } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { calendarDayDifference, calendarMonthDays, calendarMonthStart, calendarToday } from "$lib/calendarDates";

  let { timeZone = "system", reduceMotion = false, onBack } = $props<{timeZone?:string;reduceMotion?:boolean;onBack:()=>void}>();
  let now = $state(Date.now());
  let selected = $state(untrack(() => calendarToday(Date.now(), timeZone)));
  let month = $state(untrack(() => calendarMonthStart(selected)));
  const t = (key:TranslationKey, values:Record<string,string|number> = {}) => translate(key, values, $locale);
  const today = $derived(calendarToday(now, timeZone));
  const cells = $derived(calendarMonthDays(month));
  const difference = $derived(calendarDayDifference(selected, today));
  const monthLabel = $derived(new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",timeZone:"UTC"}).format(month));
  const monthYear = $derived(new Date(month).getUTCFullYear());
  const monthIndex = $derived(new Date(month).getUTCMonth());
  const years = $derived.by(() => {
    const standard = Array.from({length:201}, (_, index) => 1900 + index);
    return monthYear < 1900 ? [monthYear, ...standard] : monthYear > 2100 ? [...standard, monthYear] : standard;
  });
  const monthNames = $derived(Array.from({length:12}, (_, index) => new Intl.DateTimeFormat($locale, {month:"long",timeZone:"UTC"}).format(Date.UTC(2024,index,1))));
  const selectedLabel = $derived(new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",weekday:"long",timeZone:"UTC"}).format(selected));
  const weekdays = $derived(Array.from({length:7}, (_, index) => new Intl.DateTimeFormat($locale, {weekday:"short",timeZone:"UTC"}).format(Date.UTC(2024,0,1+index))));
  const differenceLabel = $derived(difference === 0 ? t("calendarIsToday") : difference > 0 ? t("calendarDaysAhead",{days:difference}) : t("calendarDaysAgo",{days:-difference}));
  let openPicker = $state<"year"|"month"|null>(null);
  let pickerRoot: HTMLDivElement;
  let pickerMenu = $state<HTMLDivElement>();
  function closePicker(restoreFocus = false) {
    const previous = openPicker;
    openPicker = null;
    if (restoreFocus && previous) void tick().then(() => pickerRoot?.querySelector<HTMLButtonElement>(`[data-picker-trigger="${previous}"]`)?.focus({preventScroll:true}));
  }
  async function togglePicker(kind:"year"|"month") {
    if (openPicker === kind) {closePicker(); return;}
    openPicker = kind;
    await tick();
    if (openPicker !== kind || !pickerMenu) return;
    const selectedOption = pickerMenu.querySelector<HTMLButtonElement>('[aria-selected="true"]');
    if (selectedOption) {
      pickerMenu.scrollTop = selectedOption.offsetTop - (pickerMenu.clientHeight - selectedOption.offsetHeight) / 2;
      selectedOption.focus({preventScroll:true});
    }
  }
  function choosePickerValue(kind:"year"|"month", value:number) {
    month = kind === "year" ? Date.UTC(value, monthIndex, 1) : Date.UTC(monthYear, value, 1);
    closePicker(true);
  }
  function pickerKeydown(event:KeyboardEvent) {
    if (event.key === "Escape" && openPicker) {event.preventDefault();event.stopPropagation();closePicker(true);return;}
    if (!pickerMenu || !["ArrowDown","ArrowUp","Home","End"].includes(event.key)) return;
    event.preventDefault();event.stopPropagation();
    const options = [...pickerMenu.querySelectorAll<HTMLButtonElement>('[role="option"]')];
    const index = options.indexOf(document.activeElement as HTMLButtonElement);
    const next = event.key === "Home" ? 0 : event.key === "End" ? options.length - 1 : Math.max(0, Math.min(options.length - 1, index + (event.key === "ArrowDown" ? 1 : -1)));
    options[next]?.focus();
  }
  function pickerTriggerKeydown(event:KeyboardEvent, kind:"year"|"month") {
    if (!openPicker && (event.key === "ArrowDown" || event.key === "ArrowUp")) {event.preventDefault();event.stopPropagation();void togglePicker(kind);return;}
    pickerKeydown(event);
  }
  function showToday() { selected = today; month = calendarMonthStart(today); }
  $effect(() => { const zone = timeZone; untrack(() => { selected = calendarToday(Date.now(), zone); month = calendarMonthStart(selected); }); });
  onMount(() => {const timer = setInterval(() => now = Date.now(), 30_000); return () => clearInterval(timer);});
</script>

<section class="calendar-extension" class:reduce-motion={reduceMotion} aria-label={t("calendar")}>
  {#if openPicker}<button type="button" class="picker-backdrop" aria-label={t("close")} tabindex="-1" onclick={() => closePicker()}></button>{/if}
  <header>
    <button type="button" class="icon-button" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button>
    <div><span>{t("date")}</span><strong>{t("calendar")}</strong></div>
    <button type="button" class="today-button" onclick={showToday}>{t("calendarToday")}</button>
  </header>
  <div class="month-heading">
    <div class="month-pickers" bind:this={pickerRoot}>
      <div class="picker year-picker">
        <button type="button" class="picker-trigger" class:open={openPicker === "year"} data-picker-trigger="year" aria-label={t("calendarYear")} aria-haspopup="listbox" aria-expanded={openPicker === "year"} onclick={() => void togglePicker("year")} onkeydown={(event) => pickerTriggerKeydown(event,"year")}><span>{monthYear}{#if $locale === "zh-CN" || $locale === "ja"}年{/if}</span><ChevronDown size={16} aria-hidden="true"/></button>
        {#if openPicker === "year"}<div class="picker-menu" role="listbox" tabindex="-1" aria-label={t("calendarYear")} bind:this={pickerMenu} onwheel={(event) => event.stopPropagation()} onkeydown={pickerKeydown}>{#each years as year}<button type="button" role="option" data-year={year} aria-selected={year === monthYear} class:active={year === monthYear} onclick={() => choosePickerValue("year",year)}><span>{year}{#if $locale === "zh-CN" || $locale === "ja"}年{/if}</span>{#if year === monthYear}<Check size={17}/>{/if}</button>{/each}</div>{/if}
      </div>
      <div class="picker month-picker">
        <button type="button" class="picker-trigger" class:open={openPicker === "month"} data-picker-trigger="month" aria-label={t("calendarMonth")} aria-haspopup="listbox" aria-expanded={openPicker === "month"} onclick={() => void togglePicker("month")} onkeydown={(event) => pickerTriggerKeydown(event,"month")}><span>{monthNames[monthIndex]}</span><ChevronDown size={16} aria-hidden="true"/></button>
        {#if openPicker === "month"}<div class="picker-menu" role="listbox" tabindex="-1" aria-label={t("calendarMonth")} bind:this={pickerMenu} onwheel={(event) => event.stopPropagation()} onkeydown={pickerKeydown}>{#each monthNames as name,index}<button type="button" role="option" data-month={index} aria-selected={index === monthIndex} class:active={index === monthIndex} onclick={() => choosePickerValue("month",index)}><span>{name}</span>{#if index === monthIndex}<Check size={17}/>{/if}</button>{/each}</div>{/if}
      </div>
    </div>
    <div class="month-actions">
      <button type="button" class="icon-button" aria-label={t("calendarPreviousMonth")} onclick={() => month = calendarMonthStart(month,-1)}><ChevronLeft size={22}/></button>
      <button type="button" class="icon-button" aria-label={t("calendarNextMonth")} onclick={() => month = calendarMonthStart(month,1)}><ChevronRight size={22}/></button>
    </div>
  </div>
  <div class="month-grid" aria-label={monthLabel}>
    {#each weekdays as weekday}<span class="weekday">{weekday}</span>{/each}
    {#each cells as day}
      {#if day !== null}
        <button type="button" class="day" class:today={day === today} class:selected={day === selected} aria-current={day === today ? "date" : undefined} aria-pressed={day === selected} aria-label={new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",weekday:"long",timeZone:"UTC"}).format(day)} onclick={() => selected = day}>{new Date(day).getUTCDate()}</button>
      {:else}<span></span>{/if}
    {/each}
  </div>
  <div class="date-detail" aria-live="polite">
    <span>{selectedLabel}</span><strong>{differenceLabel}</strong>
  </div>
</section>

<style>
  .calendar-extension{position:relative;display:flex;flex-direction:column;box-sizing:border-box;width:100%;height:100%;padding:calc(30px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px)) calc(24px + var(--extension-block-inset,0px));color:#fff;font-family:var(--app-font,system-ui);user-select:none}
  header{display:flex;align-items:center;gap:14px;flex:none;margin-bottom:18px}header>div{display:flex;flex:1;min-width:0;flex-direction:column;gap:3px}header span{font-size:16px;color:#ffffff80}header strong{font-size:28px;line-height:1.1}
  button{font:inherit;border:0;color:#fff;background:transparent;cursor:pointer;border-radius:12px}button:hover,button:focus-visible{background:#ffffff1a;outline:none}.icon-button{display:grid;place-items:center;width:42px;height:42px;flex:none;padding:0}.today-button{padding:8px 12px;font-size:18px;color:#91caff}
  .month-heading{display:flex;justify-content:space-between;align-items:center;gap:8px;min-height:42px}.month-pickers{position:relative;z-index:6;display:flex;min-width:0;align-items:center;gap:4px;flex:1}.picker{position:relative;min-width:0}.year-picker{flex:0 1 125px}.month-picker{flex:1;max-width:200px}.picker-trigger{display:flex;align-items:center;justify-content:space-between;gap:4px;width:100%;height:40px;padding:0 7px 0 8px;font-size:23px;font-weight:700;text-align:left}.picker-trigger span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.picker-trigger :global(svg){flex:none;color:#ffffff99;transition:transform 180ms cubic-bezier(.23,1,.32,1)}.picker-trigger.open{background:#ffffff1a}.picker-trigger.open :global(svg){transform:rotate(180deg)}.month-actions{display:flex;flex:none;gap:2px}
  .picker-backdrop{position:absolute;inset:0;z-index:5;width:100%;height:100%;padding:0;border-radius:0;background:transparent!important;cursor:default}.picker-menu{position:absolute;top:44px;left:0;z-index:7;width:max(100%,160px);max-width:210px;max-height:270px;box-sizing:border-box;overflow-y:auto;overscroll-behavior:contain;padding:6px;border:1px solid #ffffff24;border-radius:12px;background:#171719;box-shadow:0 8px 22px #0009;scrollbar-width:thin;scrollbar-color:#ffffff40 transparent;animation:picker-in 150ms cubic-bezier(.23,1,.32,1)}.picker-menu::-webkit-scrollbar{width:4px}.picker-menu::-webkit-scrollbar-thumb{border-radius:99px;background:#ffffff40}.picker-menu button{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;min-height:38px;padding:7px 10px;border-radius:8px;color:#ffffffbf;font-size:20px;font-weight:500;text-align:left}.picker-menu button.active{background:#4191eb24;color:#fff}.picker-menu button :global(svg){flex:none;color:#91caff}.reduce-motion .picker-menu{animation:none}.reduce-motion .picker-trigger :global(svg){transition:none}@keyframes picker-in{from{opacity:.35;transform:translateY(-4px)}to{opacity:1;transform:translateY(0)}}@media (prefers-reduced-motion:reduce){.picker-menu{animation:none}.picker-trigger :global(svg){transition:none}}
  .month-grid{display:grid;grid-template-columns:repeat(7,minmax(0,1fr));grid-template-rows:repeat(7,minmax(0,1fr));flex:1;min-height:0;align-items:center;justify-items:center;font-variant-numeric:tabular-nums}.weekday{font-size:19px;color:#ffffff80;white-space:nowrap}.day{display:grid;place-items:center;width:48px;height:48px;max-width:100%;max-height:100%;padding:0;border-radius:9px;font-size:25px;font-weight:500}.day.selected:not(.today){background:#193a8e;box-shadow:inset 0 0 0 1px #ffffff70;color:#fff;font-weight:700}.day.today{background:#193a8e;color:#fff;font-weight:700}
  .date-detail{display:flex;flex:none;flex-direction:column;gap:7px;padding:18px 4px 4px;border-top:1px solid #ffffff24}.date-detail span{font-size:19px;color:#ffffff99}.date-detail strong{font-size:27px;color:#fff;font-variant-numeric:tabular-nums}
</style>
