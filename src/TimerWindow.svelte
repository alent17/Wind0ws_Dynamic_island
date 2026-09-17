<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { emit, listen } from "@tauri-apps/api/event";
  import { Minus, Pause, Play, Plus, RotateCcw, X } from "lucide-svelte";
  import { Events } from "./utils/eventConstants";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  type TimerSnapshot = {
    status: CountdownStatus;
    durationMs: number;
    remainingMs: number;
    label: string;
  };

  const presets = [5, 10, 25, 60] as const;
  const maxCustomMinutes = 24 * 60;
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);

  let snapshot = $state<TimerSnapshot>({ status: "idle", durationMs: 0, remainingMs: 0, label: "倒计时" });
  let syncedAt = $state(Date.now());
  let now = $state(Date.now());
  let selectedIndex = $state(3);
  let customSelected = $state(false);
  let customDurationMinutes = $state(30);
  let customDurationOpen = $state(false);
  let customHours = $state(0);
  let customMinutes = $state(30);
  let customError = $state("");
  let displayedTime = $state("00:00");
  let lastWheelTime = "00:00";
  let wheelKey = $state(0);
  let wheelDirection = $state<"down" | "up">("down");
  let dispose: (() => void) | undefined;

  let remainingMs = $derived(snapshot.status === "running"
    ? Math.max(0, snapshot.remainingMs - (now - syncedAt))
    : snapshot.remainingMs);
  let active = $derived((snapshot.status === "running" || snapshot.status === "paused") && remainingMs > 0);
  let paused = $derived(snapshot.status === "paused" && remainingMs > 0);
  let selectedMinutes = $derived(customSelected ? customDurationMinutes : presets[selectedIndex]);
  let selectedLabel = $derived(formatDurationLabel(selectedMinutes));
  let statusLabel = $derived(snapshot.status === "running"
    ? t("timerRunning")
    : snapshot.status === "paused"
      ? t("timerPaused")
      : t("timerReady"));

  const localRemainingAt = (time: number) => snapshot.status === "running"
    ? Math.max(0, snapshot.remainingMs - (time - syncedAt))
    : snapshot.remainingMs;

  const wheelPrevious = $derived(formatCountdown(Math.max(0, remainingMs + 1_000)));
  const wheelNext = $derived(formatCountdown(Math.max(0, remainingMs - 1_000)));

  function formatDurationLabel(minutes: number) {
    const safeMinutes = Math.max(1, Math.round(minutes));
    const hours = Math.floor(safeMinutes / 60);
    const rest = safeMinutes % 60;
    if ($locale === "zh-CN") {
      return hours > 0 ? `${hours}小时${rest > 0 ? ` ${rest}分` : ""}` : `${safeMinutes}分钟`;
    }
    if ($locale === "ja") {
      return hours > 0 ? `${hours}時間${rest > 0 ? ` ${rest}分` : ""}` : `${safeMinutes}分`;
    }
    return hours > 0 ? `${hours}h${rest > 0 ? ` ${rest}m` : ""}` : `${safeMinutes} min`;
  }

  function syncSelection(durationMs: number) {
    const minutes = Math.max(1, Math.round(durationMs / 60_000));
    const presetIndex = presets.findIndex((value) => value === minutes);
    if (presetIndex >= 0) {
      selectedIndex = presetIndex;
      customSelected = false;
      return;
    }
    customDurationMinutes = Math.min(maxCustomMinutes, minutes);
    customSelected = true;
  }

  function send(action: string, durationMs?: number) {
    void emit(Events.TIMER_ACTION, { action, ...(durationMs === undefined ? {} : { durationMs }) });
  }

  function startSelected() {
    send("start", selectedMinutes * 60_000);
  }

  function choosePreset(index: number) {
    if (active) return;
    selectedIndex = index;
    customSelected = false;
    customDurationOpen = false;
    customError = "";
  }

  function openCustomDuration() {
    if (active) return;
    const base = customSelected ? customDurationMinutes : presets[selectedIndex];
    customHours = Math.floor(base / 60);
    customMinutes = base % 60;
    customError = "";
    customDurationOpen = !customDurationOpen;
  }

  function submitCustom(event: SubmitEvent) {
    event.preventDefault();
    const hours = Number.isFinite(customHours) ? Math.max(0, Math.floor(customHours)) : 0;
    const minutes = Number.isFinite(customMinutes) ? Math.max(0, Math.floor(customMinutes)) : 0;
    const total = hours * 60 + minutes;
    if (total < 1 || total > maxCustomMinutes || minutes > 59) {
      customError = t("durationInvalid");
      return;
    }
    customDurationMinutes = total;
    customSelected = true;
    customDurationOpen = false;
    customError = "";
    send("start", total * 60_000);
  }

  function handleAction() {
    if (snapshot.status === "running" && remainingMs > 0) {
      send("pause");
      return;
    }
    if (snapshot.status === "paused" && remainingMs > 0) {
      send("resume");
      return;
    }
    startSelected();
  }

  function startDragging(event: MouseEvent) {
    if ((event.target as HTMLElement).closest("button, input, form, label")) return;
    void getCurrentWindow().startDragging();
  }

  function close() {
    // Keep the pre-created window alive so reopening it never has to build a
    // new WebView2 instance on the main UI thread.
    void getCurrentWindow().hide();
  }

  function minimize() {
    void getCurrentWindow().minimize();
  }

  onMount(() => {
    let disposed = false;
    void (async () => {
      const unlisten = await listen<TimerSnapshot>(Events.TIMER_STATE_CHANGED, (event) => {
        const value = event.payload;
        if (!value) return;
        snapshot = { ...snapshot, ...value };
        syncedAt = Date.now();
        if (value.durationMs > 0) syncSelection(value.durationMs);
      });
      if (disposed) {
        unlisten();
        return;
      }
      dispose = unlisten;
      await emit(Events.TIMER_REQUEST_STATE).catch(() => undefined);
    })();

    const interval = window.setInterval(() => {
      const tickNow = Date.now();
      now = tickNow;
      const nextRemaining = localRemainingAt(tickNow);
      const nextTime = formatCountdown(nextRemaining);
      if (nextTime !== lastWheelTime) {
        wheelDirection = nextRemaining <= localRemainingAt(tickNow - 250) ? "down" : "up";
        lastWheelTime = nextTime;
        displayedTime = nextTime;
        wheelKey += 1;
      }
    }, 250);

    return () => {
      disposed = true;
      window.clearInterval(interval);
      dispose?.();
    };
  });
</script>

<svelte:head><title>{t("timer")}</title></svelte:head>
<svelte:window onmousedown={startDragging} />

<main class="timer-window">
  <section class="timer-shell" aria-label={t("timer")}>
    <div class="window-actions" aria-label={t("windowActions")}>
      <button type="button" class="window-button" aria-label={t("minimize")} title={t("minimize")} onclick={minimize}><Minus size={14} strokeWidth={2.2} /></button>
      <button type="button" class="window-button close-button" aria-label={t("close")} title={t("close")} onclick={close}><X size={14} strokeWidth={2.2} /></button>
    </div>

    <div class="timer-content">
      <div class="topline">
        <div class="status-label" class:active>
          <span class="state-dot" aria-hidden="true"></span>
          <span>{statusLabel}</span>
        </div>
        <button
          type="button"
          class="action-button"
          class:paused
          aria-label={snapshot.status === "running" ? t("pauseTimer") : snapshot.status === "paused" ? t("resumeTimer") : t("startTimer")}
          title={snapshot.status === "running" ? t("pauseTimer") : snapshot.status === "paused" ? t("resumeTimer") : t("startTimer")}
          onclick={handleAction}
        >
          {#if snapshot.status === "running" && remainingMs > 0}
            <Pause size={18} fill="currentColor" strokeWidth={2.4} />
          {:else}
            <Play size={19} fill="currentColor" strokeWidth={2.4} />
          {/if}
        </button>
      </div>

      <div class="time-wheel" aria-live="polite" aria-label={displayedTime}>
        {#key wheelKey}
          <div class="wheel-stack" class:roll-down={wheelDirection === "down"} class:roll-up={wheelDirection === "up"}>
            <span class="wheel-value previous">{wheelPrevious}</span>
            <strong class="wheel-value current">{displayedTime}</strong>
            <span class="wheel-value next">{wheelNext}</span>
          </div>
        {/key}
        <div class="wheel-fade top" aria-hidden="true"></div>
        <div class="wheel-fade bottom" aria-hidden="true"></div>
      </div>

      <div class="timer-copy">
        <span class="copy-label">{t("duration")}</span>
        <span class="copy-separator" aria-hidden="true">·</span>
        <span>{active ? formatDurationLabel(Math.ceil((snapshot.durationMs || remainingMs) / 60_000)) : t("selectDuration")}</span>
      </div>

      <section class="duration-section" aria-label={t("duration")}>
        <div class="section-heading">
          <div>
            <span class="section-title">{t("duration")}</span>
            <span class="section-hint">{t("presetHint")}</span>
          </div>
          {#if active}
            <span class="status-chip" class:paused>{paused ? t("timerPaused") : t("timerRunning")}</span>
          {/if}
        </div>

        <div class="preset-control" class:disabled={active}>
          <div class="dial-segments">
            {#each presets as minutes, index}
              <button
                type="button"
                class:selected={!customSelected && selectedIndex === index}
                aria-pressed={!customSelected && selectedIndex === index}
                aria-label={formatDurationLabel(minutes)}
                disabled={active}
                onclick={() => choosePreset(index)}
              >
                <strong>{minutes}</strong>
                <span>{t("minutesShort")}</span>
              </button>
            {/each}
          </div>
          {#if !customSelected}
            <div class="selection-indicator" style={`--selected-index:${selectedIndex}`} aria-hidden="true">
              <span class="stripe-layer"></span>
              <span class="indicator-sheen"></span>
            </div>
          {/if}
        </div>

        <button
          type="button"
          class="custom-toggle"
          class:selected={customSelected}
          class:open={customDurationOpen}
          disabled={active}
          aria-expanded={customDurationOpen}
          onclick={openCustomDuration}
        >
          <span class="custom-icon" aria-hidden="true"><Plus size={14} strokeWidth={2.4} /></span>
          <span class="custom-toggle-copy">
            <strong>{t("customDuration")}</strong>
            <small>{customSelected ? selectedLabel : t("customHint")}</small>
          </span>
          <span class="custom-chevron" aria-hidden="true">⌄</span>
        </button>

        {#if customDurationOpen}
          <form class="custom-form" onsubmit={submitCustom}>
            <div class="custom-input">
              <label for="custom-hours">{t("hours")}</label>
              <input id="custom-hours" type="number" min="0" max="24" step="1" bind:value={customHours} inputmode="numeric" />
            </div>
            <span class="time-separator" aria-hidden="true">:</span>
            <div class="custom-input">
              <label for="custom-minutes">{t("minutes")}</label>
              <input id="custom-minutes" type="number" min="0" max="59" step="1" bind:value={customMinutes} inputmode="numeric" />
            </div>
            <button type="submit" class="custom-submit">{t("applyDuration")}</button>
            {#if customError}<p class="custom-error" role="alert">{customError}</p>{/if}
          </form>
        {/if}
      </section>

      <div class="footer-actions" class:visible={active}>
        {#if active}
          <span class="dial-caption">{paused ? t("timerPaused") : t("timerRunning")}</span>
          <button type="button" class="quiet-action" onclick={() => send("adjust", 60_000)}><Plus size={14} />{t("addMinute")}</button>
          <button type="button" class="quiet-action" onclick={() => send("adjust", 300_000)}><Plus size={14} />{t("addFiveMinutes")}</button>
          <button type="button" class="reset-action" aria-label={t("cancelTimer")} onclick={() => send("reset")}><RotateCcw size={14} /></button>
        {:else}
          <span class="dial-caption">{t("selectedDuration", { value: selectedLabel })}</span>
        {/if}
      </div>
    </div>
  </section>
</main>

<style>
  :global(html), :global(body), :global(#app) {
    width: 100%;
    height: 100%;
    margin: 0;
    overflow: hidden;
    background: transparent;
  }

  :global(body) {
    font-family: var(--app-font, "Segoe UI", sans-serif);
    -webkit-font-smoothing: antialiased;
    text-rendering: optimizeLegibility;
  }

  .timer-window {
    position: relative;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    width: 100%;
    height: 100%;
    padding: 22px 24px 18px 31px;
    color: #fff;
    background: transparent;
  }

  .timer-shell {
    position: relative;
    width: 100%;
    height: auto;
    aspect-ratio: 1 / 1;
    flex: none;
    overflow: hidden;
    border-radius: 42px;
    background: #141414;
    box-shadow: 0 18px 55px rgba(0, 0, 0, .38), inset 0 1px 0 rgba(255, 255, 255, .035);
    isolation: isolate;
    user-select: none;
    -webkit-user-select: none;
    font-variant-numeric: tabular-nums;
  }

  .timer-shell::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    background: radial-gradient(circle at 50% -20%, rgba(255,255,255,.045), transparent 42%);
  }

  .timer-content {
    position: relative;
    width: 100%;
    height: 100%;
    padding: 30px 33px 26px;
  }

  .topline {
    position: relative;
    z-index: 4;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
  }

  .status-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: #7d858d;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: .02em;
  }

  .state-dot {
    width: 7px;
    height: 7px;
    flex: none;
    border-radius: 50%;
    background: #59616a;
    box-shadow: 0 0 0 4px rgba(89, 97, 106, .09);
  }

  .status-label.active .state-dot {
    background: #2c8bfe;
    box-shadow: 0 0 0 4px rgba(44, 139, 254, .12), 0 0 12px rgba(44, 139, 254, .35);
    animation: status-pulse 1.8s ease-in-out infinite;
  }

  .action-button {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    flex: none;
    padding: 0;
    border: 0;
    border-radius: 50%;
    color: #fff;
    background: #2c8bfe;
    box-shadow: 0 6px 20px rgba(44, 139, 254, .28);
    cursor: pointer;
    transition: transform 160ms cubic-bezier(.23, 1, .32, 1), background 160ms ease, box-shadow 160ms ease;
  }

  .action-button:hover {
    background: #4398ff;
    box-shadow: 0 7px 25px rgba(44, 139, 254, .38);
    transform: translateY(-1px) scale(1.03);
  }

  .action-button:active { transform: scale(.94); }

  .action-button.paused {
    background: #47647f;
    box-shadow: 0 6px 20px rgba(71, 100, 127, .18);
  }

  .window-actions {
    position: absolute;
    top: 10px;
    right: 12px;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 6px;
    opacity: 0;
    pointer-events: none;
    transform: translateY(-3px);
    transition: opacity 160ms ease, transform 180ms cubic-bezier(.23, 1, .32, 1);
  }

  .timer-window:hover .window-actions, .window-actions:focus-within {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }

  .window-button {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    color: rgba(255,255,255,.55);
    background: transparent;
    cursor: pointer;
    transition: color 140ms ease, background 140ms ease, transform 140ms cubic-bezier(.23, 1, .32, 1);
  }

  .window-button:hover { color: #fff; background: rgba(255,255,255,.1); }
  .window-button:active { transform: scale(.9); }
  .window-button.close-button:hover { color: #fff; background: rgba(255, 89, 89, .22); }

  .time-wheel {
    position: absolute;
    top: 13px;
    right: 33px;
    left: 33px;
    height: 131px;
    overflow: hidden;
    isolation: isolate;
  }

  .wheel-stack {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
  }

  .wheel-value {
    position: absolute;
    right: 0;
    left: 0;
    display: block;
    text-align: left;
    white-space: nowrap;
    line-height: 1;
    letter-spacing: -.075em;
    font-weight: 700;
    transform-origin: left center;
  }

  .wheel-value.previous {
    top: 0;
    color: rgba(255,255,255,.2);
    font-size: clamp(48px, 12vw, 76px);
    opacity: .52;
    transform: translateY(0) scale(.82);
    filter: blur(.15px);
  }

  .wheel-value.current {
    top: 50%;
    color: #fff;
    font-size: clamp(65px, 16vw, 94px);
    transform: translateY(-50%) scale(1);
  }

  .wheel-value.next {
    bottom: 0;
    color: rgba(255,255,255,.18);
    font-size: clamp(48px, 12vw, 76px);
    opacity: .42;
    transform: translateY(0) scale(.82);
    filter: blur(.2px);
  }

  .wheel-fade {
    position: absolute;
    right: 0;
    left: 0;
    z-index: 2;
    height: 33px;
    pointer-events: none;
  }

  .wheel-fade.top { top: 0; background: linear-gradient(#141414, transparent); }
  .wheel-fade.bottom { bottom: 0; background: linear-gradient(transparent, #141414); }

  .wheel-stack.roll-down .previous { animation: previous-roll-down 220ms cubic-bezier(.23, 1, .32, 1); }
  .wheel-stack.roll-down .current { animation: current-roll-down 220ms cubic-bezier(.23, 1, .32, 1); }
  .wheel-stack.roll-down .next { animation: next-roll-down 220ms cubic-bezier(.23, 1, .32, 1); }
  .wheel-stack.roll-up .previous { animation: previous-roll-up 220ms cubic-bezier(.23, 1, .32, 1); }
  .wheel-stack.roll-up .current { animation: current-roll-up 220ms cubic-bezier(.23, 1, .32, 1); }
  .wheel-stack.roll-up .next { animation: next-roll-up 220ms cubic-bezier(.23, 1, .32, 1); }

  .timer-copy {
    position: absolute;
    top: 121px;
    right: 33px;
    left: 33px;
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 0;
    min-height: 21px;
    color: #858585;
    font-size: 12px;
    line-height: 1.5;
  }

  .copy-label { color: #b7bec6; font-weight: 700; }
  .copy-separator { color: #4f565d; }

  .duration-section {
    position: absolute;
    right: 33px;
    bottom: 39px;
    left: 33px;
  }

  .section-heading {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 9px;
  }

  .section-heading > div { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .section-title { color: #f4f7fa; font-size: 13px; font-weight: 700; }
  .section-hint { color: #69727c; font-size: 10px; }

  .status-chip {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    border: 1px solid rgba(44,139,254,.25);
    border-radius: 999px;
    color: #8fc6ff;
    background: rgba(44,139,254,.1);
    font-size: 10px;
    font-weight: 700;
    white-space: nowrap;
  }

  .status-chip.paused { border-color: rgba(125,151,177,.26); color: #aabaca; background: rgba(125,151,177,.1); }

  .preset-control {
    position: relative;
    height: 74px;
    overflow: hidden;
    border: 1px solid rgba(255,255,255,.055);
    border-radius: 17px;
    background: #0f1e2c;
    box-shadow: inset 0 0 0 1px rgba(255,255,255,.025), 0 7px 20px rgba(0,0,0,.18);
  }

  .preset-control.disabled { opacity: .72; }

  .dial-segments {
    position: absolute;
    inset: 4px;
    z-index: 1;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
  }

  .dial-segments button {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 0;
    gap: 4px;
    padding: 0;
    border: 0;
    border-radius: 12px;
    color: #8ea5bc;
    background: #16395c;
    cursor: pointer;
    transition: background 160ms ease, color 160ms ease, transform 160ms cubic-bezier(.23, 1, .32, 1);
  }

  .dial-segments button:hover:not(:disabled) { color: #e3f1ff; background: #1c486f; }
  .dial-segments button:active:not(:disabled) { transform: scale(.97); }
  .dial-segments button.selected { color: #fff; background: rgba(22,57,92,.32); }
  .dial-segments button:disabled { cursor: not-allowed; }
  .dial-segments strong { font-size: 18px; line-height: 1; letter-spacing: -.03em; }
  .dial-segments span { color: currentColor; font-size: 10px; font-weight: 600; opacity: .72; }

  .selection-indicator {
    position: absolute;
    top: 4px;
    bottom: 4px;
    left: 4px;
    z-index: 0;
    width: calc((100% - 20px) / 4);
    overflow: hidden;
    border: 2px solid #2c8bfe;
    border-radius: 12px;
    background: rgba(14, 49, 82, .34);
    box-shadow: 0 0 0 1px rgba(44,139,254,.18), 0 0 12px rgba(44,139,254,.62), inset 0 0 16px rgba(44,139,254,.16);
    pointer-events: none;
    transform: translateX(calc(var(--selected-index) * (100% + 4px)));
    transition: transform 280ms cubic-bezier(.23, 1, .32, 1), box-shadow 180ms ease;
  }

  .stripe-layer {
    position: absolute;
    inset: 0;
    opacity: .62;
    background: repeating-linear-gradient(135deg, rgba(56,157,255,.72) 0 3px, rgba(44,139,254,.18) 3px 7px, transparent 7px 11px);
    animation: stripe-flow 1.8s linear infinite;
  }

  .indicator-sheen {
    position: absolute;
    inset: 0;
    background: linear-gradient(100deg, transparent 15%, rgba(255,255,255,.18) 48%, transparent 78%);
    opacity: .52;
  }

  .custom-toggle {
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 39px;
    margin-top: 8px;
    padding: 6px 10px 6px 8px;
    border: 1px solid rgba(255,255,255,.08);
    border-radius: 11px;
    color: #aab3bd;
    background: rgba(255,255,255,.035);
    text-align: left;
    cursor: pointer;
    transition: border-color 160ms ease, background 160ms ease, color 160ms ease, transform 160ms cubic-bezier(.23, 1, .32, 1);
  }

  .custom-toggle:hover:not(:disabled), .custom-toggle.selected { border-color: rgba(44,139,254,.42); color: #e8f4ff; background: rgba(44,139,254,.1); }
  .custom-toggle:active:not(:disabled) { transform: scale(.985); }
  .custom-toggle:disabled { cursor: not-allowed; opacity: .52; }
  .custom-icon { display: grid; place-items: center; width: 24px; height: 24px; margin-right: 8px; border-radius: 8px; color: #80bcff; background: rgba(44,139,254,.14); }
  .custom-toggle-copy { display: flex; flex: 1; flex-direction: column; gap: 2px; min-width: 0; }
  .custom-toggle-copy strong { font-size: 11px; line-height: 1.1; }
  .custom-toggle-copy small { overflow: hidden; color: #707b87; font-size: 9px; line-height: 1.1; text-overflow: ellipsis; white-space: nowrap; }
  .custom-toggle.selected .custom-toggle-copy small { color: #8fc6ff; }
  .custom-chevron { color: #66717c; font-size: 18px; line-height: .7; transform: translateY(-2px); transition: transform 180ms cubic-bezier(.23, 1, .32, 1); }
  .custom-toggle.open .custom-chevron { transform: translateY(1px) rotate(180deg); }

  .custom-form {
    position: relative;
    display: grid;
    grid-template-columns: 1fr auto 1fr auto;
    align-items: end;
    gap: 6px;
    margin-top: 7px;
    padding: 8px;
    border: 1px solid rgba(44,139,254,.25);
    border-radius: 12px;
    background: #182533;
    animation: form-enter 180ms cubic-bezier(.23, 1, .32, 1) both;
  }

  .custom-input { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
  .custom-input label { color: #7e8c9a; font-size: 9px; }
  .custom-input input { width: 100%; height: 28px; padding: 0 7px; border: 1px solid rgba(255,255,255,.1); border-radius: 7px; color: #fff; background: rgba(0,0,0,.2); font-size: 12px; text-align: center; outline: none; }
  .custom-input input:focus { border-color: #2c8bfe; box-shadow: 0 0 0 2px rgba(44,139,254,.14); }
  .time-separator { padding-bottom: 7px; color: #74808c; font-weight: 700; }
  .custom-submit { height: 28px; padding: 0 10px; border: 0; border-radius: 7px; color: #fff; background: #2c8bfe; font-size: 10px; font-weight: 700; cursor: pointer; transition: background 140ms ease, transform 140ms cubic-bezier(.23, 1, .32, 1); }
  .custom-submit:hover { background: #4398ff; }
  .custom-submit:active { transform: scale(.96); }
  .custom-error { grid-column: 1 / -1; margin: 0; color: #ff9a9a; font-size: 9px; line-height: 1.2; }

  .footer-actions {
    position: absolute;
    right: 33px;
    bottom: 10px;
    left: 33px;
    display: flex;
    align-items: center;
    gap: 7px;
    min-height: 28px;
  }

  .footer-actions.visible { justify-content: flex-end; }
  .footer-actions:not(.visible) { justify-content: flex-start; }

  .quiet-action, .reset-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 28px;
    padding: 0 9px;
    border: 1px solid rgba(255,255,255,.09);
    border-radius: 9px;
    color: #84919f;
    background: rgba(255,255,255,.035);
    font: 500 10px/1 var(--app-font, "Segoe UI", sans-serif);
    cursor: pointer;
    transition: color 140ms ease, background 140ms ease, transform 140ms cubic-bezier(.23, 1, .32, 1);
  }

  .reset-action { width: 28px; padding: 0; }
  .quiet-action:hover, .reset-action:hover { color: #fff; background: rgba(255,255,255,.1); }
  .quiet-action:active, .reset-action:active { transform: scale(.95); }
  .dial-caption { overflow: hidden; min-width: 0; margin-right: auto; color: #59636e; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }

  button:focus-visible, input:focus-visible { outline: 2px solid #fff; outline-offset: 3px; }

  @keyframes stripe-flow {
    from { transform: translateX(-16px); }
    to { transform: translateX(16px); }
  }

  @keyframes status-pulse {
    0%, 100% { opacity: .7; }
    50% { opacity: 1; }
  }

  @keyframes form-enter {
    from { opacity: 0; transform: translateY(-5px) scale(.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  @keyframes previous-roll-down {
    from { opacity: .28; transform: translateY(-16px) scale(.86); }
    to { opacity: .52; transform: translateY(0) scale(.82); }
  }

  @keyframes current-roll-down {
    from { opacity: .55; transform: translateY(calc(-50% + 18px)) scale(.94); }
    to { opacity: 1; transform: translateY(-50%) scale(1); }
  }

  @keyframes next-roll-down {
    from { opacity: .72; transform: translateY(14px) scale(.86); }
    to { opacity: .42; transform: translateY(0) scale(.82); }
  }

  @keyframes previous-roll-up {
    from { opacity: .72; transform: translateY(14px) scale(.86); }
    to { opacity: .52; transform: translateY(0) scale(.82); }
  }

  @keyframes current-roll-up {
    from { opacity: .55; transform: translateY(calc(-50% - 18px)) scale(.94); }
    to { opacity: 1; transform: translateY(-50%) scale(1); }
  }

  @keyframes next-roll-up {
    from { opacity: .28; transform: translateY(-14px) scale(.86); }
    to { opacity: .42; transform: translateY(0) scale(.82); }
  }

  @media (max-width: 430px) {
    .timer-window { padding-right: 17px; padding-left: 24px; }
    .timer-shell { border-radius: 34px; }
    .timer-content { padding-right: 25px; padding-left: 25px; }
    .duration-section, .footer-actions, .timer-copy { right: 25px; left: 25px; }
    .wheel-value { letter-spacing: -.065em; }
  }

  @media (max-height: 430px) {
    .timer-window { padding-top: 16px; padding-bottom: 10px; }
    .timer-content { padding-top: 24px; padding-bottom: 17px; }
    .time-wheel { top: 22px; height: 108px; }
    .timer-copy { top: 103px; }
    .duration-section { bottom: 31px; }
    .preset-control { height: 66px; }
    .footer-actions { bottom: 8px; }
  }

  @media (prefers-reduced-motion: reduce) {
    .status-label.active .state-dot, .stripe-layer { animation: none; }
    .window-actions, .window-button, .action-button, .selection-indicator, .custom-toggle, .custom-chevron, .custom-form { transition: none; animation: none; }
    .wheel-stack.roll-down .previous, .wheel-stack.roll-down .current, .wheel-stack.roll-down .next,
    .wheel-stack.roll-up .previous, .wheel-stack.roll-up .current, .wheel-stack.roll-up .next { animation: none; }
  }
</style>
