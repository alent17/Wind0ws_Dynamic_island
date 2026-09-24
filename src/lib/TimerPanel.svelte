<script lang="ts">
  import { Check, X } from "lucide-svelte";
  import RollingNumber from "$lib/RollingNumber.svelte";
  import PlayPauseIcon from "$lib/PlayPauseIcon.svelte";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    status = "idle",
    remainingMs = 0,
    finished = false,
    reduceMotion = false,
    onStart,
    onPause,
    onResume,
    onReset,
    onFinishedDismiss,
  } = $props<{
    status?: CountdownStatus;
    remainingMs?: number;
    finished?: boolean;
    reduceMotion?: boolean;
    onStart?: (durationMs: number) => void;
    onPause?: () => void;
    onResume?: () => void;
    onReset?: () => void;
    onFinishedDismiss?: () => void;
  }>();

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  const minuteStepPx = 10;
  const maxMinutes = 24 * 60;
  const active = $derived((status === "running" || status === "paused") && remainingMs > 0);
  let selectedMinutes = $state(20);
  let rulerElement = $state<HTMLDivElement>();
  let dragging = $state(false);
  let dragPointerId: number | null = null;
  let dragStartX = 0;
  let dragStartMinutes = 20;
  let suppressClick = false;
  let centerMinutes = $derived(active ? Math.max(1, Math.ceil(remainingMs / 60_000)) : selectedMinutes);
  let rulerValues = $derived.by(() => {
    const start = Math.max(1, centerMinutes - 15);
    return Array.from({ length: 31 }, (_, index) => start + index);
  });
  let activeIndex = $derived(Math.max(0, rulerValues.findIndex((value) => value === centerMinutes)));
  let previewMs = $derived(active ? remainingMs : selectedMinutes * 60_000);
  let mainLabel = $derived(status === "running" ? t("pauseTimer") : status === "paused" ? t("resumeTimer") : t("startTimer"));

  function chooseMinutes(minutes: number) {
    if (active) return;
    if (suppressClick) {
      suppressClick = false;
      return;
    }
    selectedMinutes = Math.max(1, Math.min(maxMinutes, Math.round(minutes)));
  }

  function updateFromDrag(clientX: number) {
    const deltaMinutes = Math.round((dragStartX - clientX) / minuteStepPx);
    selectedMinutes = Math.max(1, Math.min(maxMinutes, dragStartMinutes + deltaMinutes));
  }

  function handlePointerDown(event: PointerEvent) {
    if (active || (event.pointerType === "mouse" && event.button !== 0)) return;
    dragPointerId = event.pointerId;
    dragStartX = event.clientX;
    dragStartMinutes = selectedMinutes;
    dragging = false;
    suppressClick = false;
    rulerElement?.setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: PointerEvent) {
    if (active || dragPointerId !== event.pointerId) return;
    const distance = event.clientX - dragStartX;
    if (Math.abs(distance) >= 3) dragging = true;
    if (!dragging) return;
    event.preventDefault();
    updateFromDrag(event.clientX);
  }

  function endPointer(event: PointerEvent) {
    if (dragPointerId !== event.pointerId) return;
    if (dragging) suppressClick = true;
    if (rulerElement?.hasPointerCapture(event.pointerId)) rulerElement.releasePointerCapture(event.pointerId);
    dragPointerId = null;
    dragging = false;
  }

  function handleRulerKeydown(event: KeyboardEvent) {
    if (active) return;
    if (event.key === "ArrowLeft" || event.key === "ArrowRight" || event.key === "Home" || event.key === "End") {
      event.preventDefault();
      if (event.key === "Home") selectedMinutes = 1;
      else if (event.key === "End") selectedMinutes = maxMinutes;
      else {
        const direction = event.key === "ArrowLeft" ? 1 : -1;
        const step = event.shiftKey ? 5 : 1;
        selectedMinutes = Math.max(1, Math.min(maxMinutes, selectedMinutes + direction * step));
      }
    }
  }

  function handleMainAction() {
    if (status === "running" && remainingMs > 0) {
      onPause?.();
      return;
    }
    if (status === "paused" && remainingMs > 0) {
      onResume?.();
      return;
    }
    onStart?.(selectedMinutes * 60_000);
  }
</script>

<div class="timer-panel" class:active={active} class:complete={finished}>
  {#if finished}
    <div class="timer-complete-card" role="status" aria-live="assertive">
      <span class="timer-complete-mark"><Check size={17} strokeWidth={2.4} /></span>
      <div class="timer-complete-copy">
        <strong>{t("timerComplete")}</strong>
        <span>{t("timerCompleteHint")}</span>
      </div>
      <button type="button" class="timer-complete-dismiss" aria-label={t("dismissNotice")} onclick={(event) => { event.stopPropagation(); onFinishedDismiss?.(); }}>
        <X size={14} strokeWidth={2} />
      </button>
    </div>
  {:else}
  {#if !active}
    <div
      bind:this={rulerElement}
      class="timer-ruler"
      class:dragging
      data-stop-toggle
      role="slider"
      tabindex="0"
      aria-label={t("selectDuration")}
      aria-valuemin="1"
      aria-valuemax={maxMinutes}
      aria-valuenow={centerMinutes}
      aria-valuetext={`${centerMinutes} ${t("minutesShort")}`}
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={endPointer}
      onpointercancel={endPointer}
      onkeydown={handleRulerKeydown}
    >
      <div class="ruler-track" style={`--active-index:${activeIndex}`}>
        {#each rulerValues as minutes}
          <button
            class:major={minutes % 5 === 0}
            class:prior={minutes < centerMinutes}
            class:selected={minutes === centerMinutes}
            type="button"
            disabled={active}
            aria-label={`${minutes} ${t("minutesShort")}`}
            onclick={(event) => { event.stopPropagation(); chooseMinutes(minutes); }}
          >
            <span class="tick-label">{minutes % 5 === 0 ? minutes : ""}</span>
            <i class="tick"></i>
          </button>
        {/each}
      </div>
      <span class="ruler-fade left" aria-hidden="true"></span>
      <span class="ruler-fade right" aria-hidden="true"></span>
    </div>
  {/if}

  <div class="timer-footer">
    <div class="timer-actions" class:timer-active={active}>
      <button class="timer-main-button" type="button" aria-label={mainLabel} onclick={(event) => { event.stopPropagation(); handleMainAction(); }}>
      {#if active}<PlayPauseIcon playing={status === "running"} size={20} {reduceMotion} />{:else}<span>{mainLabel}</span>{/if}
      </button>
      {#if active}
        <button class="timer-reset" type="button" aria-label={t("cancelTimer")} onclick={(event) => { event.stopPropagation(); onReset?.(); }}>
          <X size={19} strokeWidth={2.2} />
        </button>
      {/if}
    </div>

    <div class="timer-readout" aria-live="polite">
      <strong><RollingNumber value={formatCountdown(previewMs)} {reduceMotion} /></strong>
      <small>{active ? (status === "paused" ? t("timerPaused") : t("timerRunning")) : t("timerReady")}</small>
    </div>
  </div>
  {/if}

</div>

<style>
  .timer-panel {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    min-height: 0;
    box-sizing: border-box;
    color: #fff;
    font-family: var(--app-font);
    font-synthesis: none;
    font-variant-numeric: tabular-nums;
  }

  .timer-panel.active .timer-footer { flex: 1; margin-top: 0; }
  .timer-panel.complete{justify-content:center}
  .timer-complete-card{position:relative;display:grid;grid-template-columns:34px minmax(0,1fr) auto;align-items:center;gap:9px;width:100%;min-height:70px;padding:10px 11px;border:1px solid rgba(255,255,255,.085);border-radius:13px;background:linear-gradient(110deg,rgba(255,255,255,.045),rgba(255,255,255,.015));box-shadow:inset 0 1px 0 rgba(255,255,255,.035)}
  .timer-complete-mark{display:grid;place-items:center;width:30px;height:30px;border-radius:50%;color:#8ac6ff;background:rgba(67,155,255,.14);box-shadow:inset 0 0 0 1px rgba(92,174,255,.17)}
  .timer-complete-copy{display:flex;min-width:0;flex-direction:column;gap:5px}
  .timer-complete-copy strong{overflow:hidden;color:#f4f7fb;font-size:13px;font-weight:700;line-height:1.1;text-overflow:ellipsis;white-space:nowrap}
  .timer-complete-copy span{display:-webkit-box;overflow:hidden;color:rgba(229,235,245,.52);font-size:10px;line-height:1.2;text-overflow:ellipsis;white-space:normal;line-clamp:2;-webkit-box-orient:vertical;-webkit-line-clamp:2}
  .timer-complete-dismiss{display:grid;place-items:center;width:27px;height:27px;padding:0;border:1px solid transparent;border-radius:50%;color:rgba(241,246,255,.78);background:transparent;cursor:pointer;transition:color 140ms ease,background 140ms ease,border-color 140ms ease}
  .timer-complete-dismiss:hover{color:#fff;border-color:rgba(255,255,255,.2);background:rgba(255,255,255,.1)}
  .timer-complete-dismiss:focus-visible{color:#fff;border-color:rgba(255,255,255,.2);background:rgba(255,255,255,.1)}
  .timer-complete-dismiss:active{transform:scale(.96)}

  .timer-ruler {
    position: relative;
    flex: 0 0 44px;
    height: 44px;
    margin: 0 -6px;
    overflow: hidden;
    touch-action: none;
    user-select: none;
    cursor: grab;
    mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
    -webkit-mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
  }

  .timer-ruler.dragging { cursor: grabbing; }
  .timer-ruler.dragging .ruler-track { transition: none; }

  .ruler-track {
    position: absolute;
    top: 2px;
    left: 0;
    display: flex;
    align-items: flex-start;
    width: max-content;
    height: 42px;
    transform: translateX(calc(136px - var(--active-index) * 10px));
    transition: transform 260ms cubic-bezier(.22, 1, .36, 1);
  }

  .ruler-track button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    width: 10px;
    height: 41px;
    padding: 0;
    border: 0;
    color: transparent;
    background: transparent;
    cursor: inherit;
    text-transform: none;
  }

  .ruler-track button:disabled { cursor: default; }
  .tick-label { height: 15px; color: rgba(110, 178, 255, .78); font: 500 8px/1 var(--app-font); }
  .ruler-track button:not(.major) .tick-label { opacity: 0; }
  .tick { display: block; width: 3px; height: 24px; border-radius: 999px; background: rgba(130, 153, 185, .3); transform:scaleY(.667); transform-origin:center bottom; transition: transform 180ms ease, background 180ms ease, box-shadow 180ms ease; }
  .ruler-track button.major .tick { transform:scaleY(.852); background: rgba(105, 170, 255, .7); }
  .ruler-track button.prior .tick { background: #eaf4ff; }
  .ruler-track button.prior .tick-label { color: #b9ddff; }
  .ruler-track button.selected .tick { transform:scaleY(1); background: #eaf4ff; box-shadow: 0 0 9px rgba(234, 244, 255, .65); }
  .ruler-track button.selected .tick-label { color: #b9ddff; }

  .ruler-fade { position: absolute; top: 0; bottom: 0; z-index: 2; width: 42px; pointer-events: none; }
  .ruler-fade.left { left: 0; background: linear-gradient(90deg, rgba(0,0,0,.96), transparent); }
  .ruler-fade.right { right: 0; background: linear-gradient(270deg, rgba(0,0,0,.96), transparent); }

  .timer-footer { display: flex; align-items: center; justify-content: space-between; gap: 8px; min-height: 0; margin-top: 5px; }
  .timer-actions { display: flex; align-items: center; gap: 5px; min-width: 0; }
  .timer-actions.timer-active { gap: 12px; }
  .timer-main-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 28px;
    border: 0;
    border-radius: 17px;
    color: #91caff;
    background: rgba(65, 145, 235, .14);
    font: 700 11px/1 var(--app-font);
    letter-spacing: .01em;
    text-transform: none;
    white-space: nowrap;
    cursor: pointer;
    transition: width 180ms cubic-bezier(.22,1,.36,1), height 180ms cubic-bezier(.22,1,.36,1), background 150ms ease, color 150ms ease, border-radius 180ms cubic-bezier(.22,1,.36,1), box-shadow 150ms ease, transform 150ms cubic-bezier(.23,1,.32,1);
  }

  .timer-main-button { padding: 0 10px; font-size: 10px; }
  .timer-main-button:hover { color: #d2eaff; background: rgba(65, 145, 235, .22); }
  .timer-main-button:active { transform: scale(.95); }
  .timer-actions.timer-active .timer-main-button { flex: none; width: 48px; height: 48px; padding: 0; border-radius: 12px; color: rgba(255,255,255,.9); background: transparent; box-shadow: none; }

  .timer-readout { display: flex; flex: 1; flex-direction: column; align-items: flex-end; min-width: 0; line-height: 1; }
  .timer-readout strong { display: block; box-sizing: border-box; width: 100%; max-width: 100%; padding-right: 6px; color: #eaf4ff; font: 700 clamp(20px, 6vw, 24px)/.9 var(--app-font); letter-spacing: -.035em; text-align: right; white-space: nowrap; }
  .timer-readout small { margin-top: 3px; color: rgba(138, 190, 245, .58); font: 500 8px/1 var(--app-font); letter-spacing: .04em; }
  .timer-reset { flex: none; display: grid; place-items: center; width: 48px; height: 48px; padding: 0; border: 0; border-radius: 12px; color: rgba(255,255,255,.9); background: transparent; cursor: pointer; transition: color 150ms ease, background 150ms ease, transform 150ms cubic-bezier(.23,1,.32,1); }
  .timer-reset:active { transform: scale(.95); }

  @media (hover: hover) and (pointer: fine) {
    .timer-actions.timer-active .timer-main-button:hover { color: #fff; background: rgba(255,255,255,.1); }
    .timer-reset:hover { color: #fff; background: rgba(255,255,255,.1); }
  }

  .timer-actions.timer-active .timer-main-button:focus-visible { color: #fff; background: rgba(255,255,255,.1); }
  .timer-reset:focus-visible { color: #fff; background: rgba(255,255,255,.1); }

  button:focus-visible { outline: 2px solid #fff; outline-offset: 2px; }

  @media (prefers-reduced-motion: reduce) {
    .ruler-track, .tick, .timer-main-button, .timer-reset { transition: none; }
  }
</style>
