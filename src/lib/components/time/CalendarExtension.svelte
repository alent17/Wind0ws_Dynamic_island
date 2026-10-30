<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { ArrowLeft, Bell, Check, ChevronDown, ChevronLeft, ChevronRight, Plus, Trash2 } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { CLOCK_TIME_ZONES, clockTimeZoneOption, clockTimeZoneCity } from "$lib/timeZones";
  import { calendarDayDifference, calendarMonthDays, calendarMonthStart, calendarToday } from "$lib/calendarDates";

  let { timeZone = "system", reduceMotion = false, onBack, onTodoHeight } = $props<{timeZone?:string;reduceMotion?:boolean;onBack:()=>void;onTodoHeight:(height:number)=>void}>();
  type Todo = { id: number; date: number; endDate?: number; text: string; done: boolean; reminder?: { time: string; timeZone: string; daily?: boolean } };
  let todos = $state<Todo[]>(JSON.parse(localStorage.getItem("isle-calendar-todos") || "[]"));
  let todoText = $state("");
  let remind = $state(false);
  let reminderTime = $state("09:00");
  let reminderDaily = $state(false);
  let reminderPicker = $state<"hour" | "minute" | "zone" | null>(null);
  const visibleTodos = $derived(todos.filter(todo => todo.date <= rangeEnd && (todo.endDate ?? (todo.reminder?.daily ? Infinity : todo.date)) >= rangeStart));
  let reminderZone = $state(untrack(() => timeZone === "system" ? Intl.DateTimeFormat().resolvedOptions().timeZone : timeZone));
  let todoHeight = $state(0);
  $effect(() => onTodoHeight(todoHeight));
  function saveTodos() { localStorage.setItem("isle-calendar-todos", JSON.stringify(todos)); }
  function addTodo() {
    const text = todoText.trim();
    if (!text) return;
    todos = [...todos, { id: Date.now(), date: rangeStart, ...(rangeEnd !== rangeStart ? {endDate: rangeEnd} : {}), text, done: false, ...(remind ? { reminder: { time: reminderTime, timeZone: reminderZone, daily: reminderDaily } } : {}) }];
    todoText = "";
    saveTodos();
  }
  let now = $state(Date.now());
  let selected = $state(untrack(() => calendarToday(Date.now(), timeZone)));
  let rangeAnchor = $state(selected);
  let selecting = false;
  const rangeStart = $derived(Math.min(selected, rangeAnchor));
  const rangeEnd = $derived(Math.max(selected, rangeAnchor));
  function startRange(event: PointerEvent, day: number) {
    if (event.button !== 0) return;
    event.preventDefault(); event.stopPropagation();
    selecting = true; rangeAnchor = day; selected = day;
  }
  function extendRange(day: number) { if (selecting) selected = day; }
  function selectDay(day: number) { rangeAnchor = day; selected = day; }
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
  function showToday() { selectDay(today); month = calendarMonthStart(today); }
  $effect(() => { const zone = timeZone; untrack(() => { selectDay(calendarToday(Date.now(), zone)); month = calendarMonthStart(selected); }); });
  onMount(() => {const timer = setInterval(() => now = Date.now(), 30_000); return () => clearInterval(timer);});
</script>

<svelte:window onpointerup={() => selecting = false} onpointercancel={() => selecting = false}/>
<section class="calendar-extension" class:reduce-motion={reduceMotion} aria-label={t("calendar")}>
  {#if openPicker}<button type="button" class="picker-backdrop" aria-label={t("close")} tabindex="-1" onclick={() => closePicker()}></button>{/if}
  <header>
    <button type="button" class="icon-button" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button>
    <div><span>{t("date")}</span><strong>{t("calendar")}</strong></div>
    <button type="button" class="today-button" onclick={showToday}>{t("calendarToday")}</button>
  </header>
  <div class="month-heading" data-stop-toggle>
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
  <div class="month-grid" data-stop-toggle aria-label={monthLabel}>
    {#each weekdays as weekday}<span class="weekday">{weekday}</span>{/each}
    {#each cells as day}
      {#if day !== null}
        <button type="button" class="day" class:today={day === today} class:selected={day >= rangeStart && day <= rangeEnd} class:range-start={rangeStart !== rangeEnd && day === rangeStart} class:range-end={rangeStart !== rangeEnd && day === rangeEnd} class:range-middle={rangeStart !== rangeEnd && day > rangeStart && day < rangeEnd} aria-current={day === today ? "date" : undefined} aria-pressed={day >= rangeStart && day <= rangeEnd} aria-label={new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",weekday:"long",timeZone:"UTC"}).format(day)} onpointerdown={(event) => startRange(event, day)} onpointerenter={() => extendRange(day)} onclick={(event) => {if (event.detail === 0) selectDay(day)}}>{new Date(day).getUTCDate()}{#if todos.some(todo => (day >= todo.date && day <= (todo.endDate ?? (todo.reminder?.daily ? Infinity : todo.date))) && !todo.done)}<i class="todo-dot"></i>{/if}</button>
      {:else}<span></span>{/if}
    {/each}
  </div>
  <div class="date-detail" aria-live="polite">
    <span>{rangeStart === rangeEnd ? selectedLabel : `${new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",timeZone:"UTC"}).format(rangeStart)} — ${new Intl.DateTimeFormat($locale, {year:"numeric",month:"long",day:"numeric",timeZone:"UTC"}).format(rangeEnd)}`}</span><strong>{rangeStart === rangeEnd ? differenceLabel : `${calendarDayDifference(rangeEnd, rangeStart) + 1} 天`}</strong>
  </div>
  <div class="todos" bind:offsetHeight={todoHeight} data-stop-toggle>
    <div class="todo-heading"><strong>待办事项</strong><span>{visibleTodos.filter(todo => !todo.done).length} 项待完成</span></div>
    <form onsubmit={(event) => { event.preventDefault(); addTodo(); }}>
      <div class="todo-entry"><input bind:value={todoText} placeholder={rangeStart === rangeEnd ? "添加当天待办" : "添加所选日期待办"} aria-label="添加当天待办"/><button type="submit" aria-label="添加待办"><Plus size={20}/><span>添加</span></button></div>
      <button type="button" class="reminder-toggle" class:active={remind} aria-pressed={remind} onclick={() => remind = !remind}><Bell size={17}/><span>提醒</span><span class="toggle-indicator" aria-hidden="true"></span></button>
      {#if remind}
        <div class="reminder-picker-area">
        <div class="reminder-options">
          <div class="reminder-field"><span>时间</span><div class="time-buttons"><button type="button" aria-label="选择小时" onclick={() => reminderPicker = reminderPicker === "hour" ? null : "hour"}>{reminderTime.slice(0,2)}</button><b>:</b><button type="button" aria-label="选择分钟" onclick={() => reminderPicker = reminderPicker === "minute" ? null : "minute"}>{reminderTime.slice(3)}</button></div></div>
          <div class="reminder-field"><span>时区</span><button type="button" class="zone-trigger" aria-label="提醒时区" onclick={() => reminderPicker = reminderPicker === "zone" ? null : "zone"}><span>{clockTimeZoneCity(reminderZone, $locale)}</span><ChevronDown size={18}/></button></div>
        </div>
        {#if reminderPicker}<div class="reminder-menu" onwheel={(event) => event.stopPropagation()} data-stop-toggle>
          {#if reminderPicker === "zone"}
            {#each CLOCK_TIME_ZONES as zone}<button type="button" class:active={reminderZone === zone.value} onclick={() => {reminderZone = zone.value; reminderPicker = null;}}>{clockTimeZoneOption(zone.value, $locale).label}{#if reminderZone === zone.value}<Check size={18}/>{/if}</button>{/each}
          {:else}
            {#each Array.from({length:reminderPicker === "hour" ? 24 : 60}, (_, i) => String(i).padStart(2,"0")) as value}<button type="button" class:active={value === (reminderPicker === "hour" ? reminderTime.slice(0,2) : reminderTime.slice(3))} onclick={() => {reminderTime = reminderPicker === "hour" ? `${value}:${reminderTime.slice(3)}` : `${reminderTime.slice(0,2)}:${value}`; reminderPicker = null;}}>{value}</button>{/each}
          {/if}
        </div>{/if}
        </div>
        <button type="button" class="reminder-toggle" class:active={reminderDaily} aria-pressed={reminderDaily} onclick={() => reminderDaily = !reminderDaily}><span>每天重复</span><span class="toggle-indicator" aria-hidden="true"></span></button>
      {/if}
    </form>
    <div class="todo-list">{#each visibleTodos as todo (todo.id)}<div class="todo-item"><button type="button" class="todo-complete" class:done={todo.done} aria-pressed={todo.done} aria-label={`${todo.done ? "取消完成" : "完成"} ${todo.text}`} onclick={() => { todo.done = !todo.done; saveTodos(); }}><span class="todo-content"><strong>{todo.text}</strong>{#if todo.reminder}<small><Bell size={16}/>{todo.reminder.time} · {clockTimeZoneCity(todo.reminder.timeZone, $locale)}{todo.reminder.daily ? " · 每天" : ""}</small>{/if}</span>{#if todo.done}<Check size={22}/>{/if}</button><button type="button" class="todo-delete" aria-label={`删除 ${todo.text}`} onclick={() => { todos = todos.filter(item => item.id !== todo.id); saveTodos(); }}><Trash2 size={20}/></button></div>{/each}</div>
  </div>
</section>

<style>
  .calendar-extension{position:relative;display:flex;flex-direction:column;box-sizing:border-box;width:100%;height:100%;padding:calc(30px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px)) calc(48px + var(--extension-block-inset,0px));color:#fff;font-family:var(--app-font,system-ui);font-size:max(16px,calc(12px / var(--calendar-scale,1)));user-select:none}
  header{display:flex;align-items:center;gap:14px;flex:none;margin-bottom:18px}header>div{display:flex;flex:1;min-width:0;flex-direction:column;gap:3px}header span{font-size:max(15px,calc(12px / var(--calendar-scale,1)));color:#ffffff80}header strong{font-size:max(28px,calc(18px / var(--calendar-scale,1)));line-height:1.1}
  button{font:inherit;border:0;color:#fff;background:transparent;cursor:pointer;border-radius:12px}button:hover,button:focus-visible{background:#ffffff1a;outline:none}.icon-button{display:grid;place-items:center;width:42px;height:42px;flex:none;padding:0}.today-button{padding:8px 12px;font-size:max(18px,calc(12px / var(--calendar-scale,1)));color:#91caff}
  .month-heading{display:flex;justify-content:space-between;align-items:center;gap:8px;min-height:42px}.month-pickers{position:relative;z-index:6;display:flex;min-width:0;align-items:center;gap:4px;flex:1}.picker{position:relative;min-width:0}.year-picker{flex:none}.year-picker .picker-trigger{width:max-content}.month-picker{flex:1;max-width:200px}.picker-trigger{display:flex;align-items:center;justify-content:space-between;gap:4px;width:100%;height:40px;padding:0 7px 0 8px;font-size:max(23px,calc(12px / var(--calendar-scale,1)));font-weight:700;text-align:left}.picker-trigger span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.picker-trigger :global(svg){flex:none;color:#ffffff99;transition:transform 180ms cubic-bezier(.23,1,.32,1)}.picker-trigger.open{background:#ffffff1a}.picker-trigger.open :global(svg){transform:rotate(180deg)}.month-actions{display:flex;flex:none;gap:2px}
  .picker-backdrop{position:absolute;inset:0;z-index:5;width:100%;height:100%;padding:0;border-radius:0;background:transparent!important;cursor:default}.picker-menu{position:absolute;top:44px;left:0;z-index:7;width:max(100%,160px);max-width:210px;max-height:270px;box-sizing:border-box;overflow-y:auto;overscroll-behavior:contain;padding:6px;border:1px solid #ffffff24;border-radius:12px;background:#171719;box-shadow:0 8px 22px #0009;scrollbar-width:thin;scrollbar-color:#ffffff40 transparent;animation:picker-in 150ms cubic-bezier(.23,1,.32,1)}.picker-menu::-webkit-scrollbar{width:4px}.picker-menu::-webkit-scrollbar-thumb{border-radius:99px;background:#ffffff40}.picker-menu button{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;min-height:38px;padding:7px 10px;border-radius:8px;color:#ffffffbf;font-size:max(20px,calc(12px / var(--calendar-scale,1)));font-weight:500;text-align:left}.picker-menu button.active{background:#002fa724;color:#fff}.picker-menu button :global(svg){flex:none;color:#91caff}.reduce-motion .picker-menu{animation:none}.reduce-motion .picker-trigger :global(svg){transition:none}@keyframes picker-in{from{opacity:.35;transform:translateY(-4px)}to{opacity:1;transform:translateY(0)}}@media (prefers-reduced-motion:reduce){.picker-menu{animation:none}.picker-trigger :global(svg){transition:none}}
  .month-grid{container-type:inline-size;touch-action:none;display:grid;grid-template-columns:repeat(7,minmax(0,1fr));grid-template-rows:repeat(7,minmax(48px,1fr));flex:1 0 336px;min-height:336px;align-items:center;justify-items:center;font-variant-numeric:tabular-nums}.weekday{font-size:max(15px,calc(12px / var(--calendar-scale,1)));color:#ffffff80;white-space:nowrap}.day{display:grid;place-items:center;width:48px;height:48px;max-width:100%;max-height:100%;padding:0;border-radius:9px;font-size:max(25px,calc(12px / var(--calendar-scale,1)));font-weight:500}.day.selected:not(.today){background:#002fa7;box-shadow:inset 0 0 0 1px #ffffff70;color:#fff;font-weight:700}.day.today{background:#002fa7;color:#fff;font-weight:700}
  .day{position:relative;isolation:isolate}.day.range-start::before,.day.range-end::before,.day.range-middle::before{content:"";position:absolute;z-index:-1;top:0;bottom:0;background:#002fa7}.day.range-start::before{left:50%;right:calc((100% - 100cqw / 7) / 2)}.day.range-end::before{left:calc((100% - 100cqw / 7) / 2);right:50%}.day.range-middle::before{left:calc((100% - 100cqw / 7) / 2);right:calc((100% - 100cqw / 7) / 2)}.day.range-middle,.day.range-start,.day.range-end{box-shadow:none!important}.todo-dot{position:absolute;bottom:2px;width:5px;height:5px;border-radius:50%;background:#ff9f43}
  .todos{flex:none;padding-top:20px}.todo-heading{display:flex;align-items:baseline;justify-content:space-between;gap:12px;margin-bottom:12px}.todo-heading strong{font-size:max(24px,calc(16px / var(--calendar-scale,1)));font-weight:700}.todo-heading span{font-size:max(15px,calc(12px / var(--calendar-scale,1)));color:#ffffff80}
  .todos form{display:flex;flex-direction:column;gap:10px}.todo-entry{display:flex;gap:8px}.todo-entry input{flex:1;min-width:0;height:52px;box-sizing:border-box;padding:0 14px;border:1px solid transparent;border-radius:12px;background:#ffffff18;color:#fff;font:500 max(21px,calc(12px / var(--calendar-scale,1))) var(--app-font,system-ui);outline:none}.todo-entry input::placeholder{color:#ffffff70}.todo-entry input:focus{border-color:#ffffff80}.todo-entry button{display:flex;align-items:center;justify-content:center;gap:5px;min-width:88px;padding:0 12px;color:#91caff;font-size:max(20px,calc(12px / var(--calendar-scale,1)));font-weight:700}
  .reminder-toggle{align-self:flex-start;display:flex;align-items:center;gap:8px;min-height:44px;padding:0 10px;color:#ffffffa6;font-size:max(17px,calc(12px / var(--calendar-scale,1)))}.reminder-toggle.active{color:#91caff}.toggle-indicator{position:relative;width:28px;height:16px;margin-left:4px;border-radius:99px;background:#ffffff33}.toggle-indicator::after{content:"";position:absolute;top:3px;left:3px;width:10px;height:10px;border-radius:50%;background:#fff;transition:transform 180ms}.reminder-toggle.active .toggle-indicator{background:#002fa7}.reminder-toggle.active .toggle-indicator::after{transform:translateX(12px)}
  .reminder-picker-area{position:relative}.reminder-options{display:grid;grid-template-columns:minmax(120px,1fr) minmax(0,2fr);gap:10px}.reminder-field{display:flex;flex-direction:column;gap:6px;min-width:0;color:#ffffff80;font-size:max(15px,calc(12px / var(--calendar-scale,1)))}.time-buttons,.zone-trigger{display:flex;align-items:center;height:48px;box-sizing:border-box;border:1px solid #ffffff1a;border-radius:10px;background:#ffffff12;color:#fff}.time-buttons button{flex:1;height:100%;padding:0;font-size:max(19px,calc(12px / var(--calendar-scale,1)));font-variant-numeric:tabular-nums}.time-buttons b{color:#ffffff80}.zone-trigger{justify-content:space-between;gap:8px;padding:0 10px;font-size:max(19px,calc(12px / var(--calendar-scale,1)));text-align:left}.zone-trigger span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.zone-trigger :global(svg){flex:none}.reminder-menu{position:absolute;z-index:8;left:0;right:0;bottom:calc(100% + 8px);display:flex;flex-direction:column;max-height:210px;overflow-y:auto;overscroll-behavior:contain;padding:6px;border:1px solid #ffffff24;border-radius:12px;background:#171719;scrollbar-width:thin;scrollbar-color:#ffffff40 transparent}.reminder-menu button{display:flex;align-items:center;justify-content:space-between;min-height:42px;padding:8px 12px;color:#ffffffbf;font-size:max(19px,calc(12px / var(--calendar-scale,1)));text-align:left}.reminder-menu button.active{background:#002fa724;color:#fff}.reminder-menu button :global(svg){color:#91caff}
  .todo-list{overflow-wrap:anywhere}.todo-item{display:flex;align-items:stretch;min-height:64px;border-bottom:1px solid #ffffff14}.todo-complete{display:flex;flex:1;align-items:center;gap:12px;min-width:0;padding:10px 12px;text-align:left}.todo-content{flex:1;min-width:0}.todo-content strong{font-size:max(23px,calc(15px / var(--calendar-scale,1)));font-weight:600}.todo-content small{display:flex;align-items:center;gap:6px;margin-top:5px;color:#ffffff99;font-size:max(15px,calc(12px / var(--calendar-scale,1)))}.todo-complete.done .todo-content{text-decoration:line-through;opacity:.55}.todo-complete :global(svg){color:#91caff}.todo-delete{display:grid;place-items:center;align-self:center;flex:none;width:44px;height:44px;padding:0;color:#ffffff80}.todo-delete:hover{color:#ff453a}

  .date-detail{display:flex;flex:none;flex-direction:column;gap:7px;padding:18px 4px 4px;border-top:1px solid #ffffff24}.date-detail span{font-size:max(15px,calc(12px / var(--calendar-scale,1)));color:#ffffff99}.date-detail strong{font-size:max(27px,calc(17px / var(--calendar-scale,1)));color:#fff;font-variant-numeric:tabular-nums}
</style>
