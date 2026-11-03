<script lang="ts">
  import { Timer, X, Play, Pause, Volume2, VolumeX } from "lucide-svelte";
  import RollingNumber from "$lib/components/time/RollingNumber.svelte";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    compact = false,
    soundEnabled = true,
    showCompact = true,
    onSoundChange,
    onShowCompactChange,
    status = "idle",
    remainingMs = 0,
    durationMs = 0,
    finished = false,
    reduceMotion = false,
    onStart,
    onPause,
    onResume,
    onReset,
    onFinishedDismiss,
  } = $props<{
    compact?: boolean;
    soundEnabled?: boolean;
    showCompact?: boolean;
    onSoundChange?: (value: boolean) => void;
    onShowCompactChange?: (value: boolean) => void;
    status?: CountdownStatus;
    remainingMs?: number;
    durationMs?: number;
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
  let activeActionLabel = $derived(status === "running" ? t("timerPauseAction") : t("timerResumeAction"));

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

<div class="timer-panel" class:compact class:active={active} class:complete={finished}>
  {#if finished}
    <div class="timer-finished-layout" role="status" aria-live="assertive">
      <button type="button" class="timer-finished-dismiss" aria-label={t("dismissNotice")} onclick={(event) => { event.stopPropagation(); onFinishedDismiss?.(); }}><X size={28}/></button>
      <div class="timer-finished-info"><span><Timer size={18}/>{t("timerComplete")}</span><strong>00:00</strong></div>
    </div>
  {:else}
  {#if active}
  <div class="timer-active-layout" class:paused={status === "paused"}>
    <div class="timer-active-controls">
      <button class="timer-main-button" type="button" aria-label={mainLabel} onclick={(event) => { event.stopPropagation(); handleMainAction(); }}>
        {#if status === "running"}<Pause size={34} fill="currentColor"/>{:else}<Play size={34} fill="currentColor"/>{/if}
      </button>
      <button class="timer-reset" type="button" aria-label={t("cancelTimer")} onclick={(event) => { event.stopPropagation(); onReset?.(); }}>
        <X size={34}/>
      </button>
    </div>
    <div class="timer-active-info">
      <div class="timer-readout" aria-live="polite">
        <strong><RollingNumber value={formatCountdown(remainingMs)} {reduceMotion} /></strong>
      </div>
    </div>
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
        {#each rulerValues as minutes, index}
          <button
            style={`--tick-color:${index <= activeIndex ? `color-mix(in srgb, #ff9f43 ${activeIndex === 0 ? 100 : 35 + 65 * index / activeIndex}%, black)` : "#777"}`}
            class:major={minutes % 5 === 0}
            class:prior={minutes < centerMinutes}
            class:selected={minutes === centerMinutes}
            type="button"
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
    <div class="timer-actions">
      <button class="timer-main-button" type="button" aria-label={mainLabel} onclick={(event) => { event.stopPropagation(); handleMainAction(); }}>
        <Play size={34} fill="currentColor"/>
      </button>
      <button class="timer-option" class:enabled={soundEnabled} type="button" aria-label="倒计时响铃" aria-pressed={soundEnabled} onclick={(event)=>{event.stopPropagation();onSoundChange?.(!soundEnabled)}}>{#if soundEnabled}<Volume2 size={28} fill="currentColor"/>{:else}<VolumeX size={28}/>{/if}</button>
      <button class="timer-option" class:enabled={showCompact} type="button" aria-label="缩小时显示倒计时" aria-pressed={showCompact} onclick={(event)=>{event.stopPropagation();onShowCompactChange?.(!showCompact)}}><svg width="28" height="28" viewBox="0 0 28 28" aria-hidden="true"><rect x="2" y="7" width="24" height="14" rx="7" fill={showCompact ? "currentColor" : "none"} stroke="currentColor" stroke-width="2"/><path d="M9 14h3m4 0h3" stroke={showCompact ? "#171717" : "currentColor"} stroke-width="2" stroke-linecap="round"/></svg></button>
    </div>
    <div class="timer-readout" aria-live="polite">
      <strong><RollingNumber value={formatCountdown(previewMs)} {reduceMotion} /></strong>
    </div>
  </div>
  {/if}
  {/if}

</div>

<style>
.timer-finished-layout{display:flex;align-items:center;justify-content:space-between;gap:16px;width:100%;min-width:0}.timer-finished-dismiss{display:grid;place-items:center;width:54px;height:54px;flex:none;border:1px solid #ff6b6b33;border-radius:12px;background:#ff6b6b0e;color:#ff7777;cursor:pointer}.timer-finished-info{display:flex;min-width:0;flex-direction:column;align-items:flex-end;gap:6px}.timer-finished-info span{display:flex;align-items:center;gap:6px;color:#ff9f43;font-size:13px;font-weight:600}.timer-finished-info strong{font-size:clamp(44px,10cqw,70px);font-weight:700;line-height:.92;letter-spacing:-.035em;font-variant-numeric:tabular-nums}

.timer-actions{gap:8px}.timer-actions .timer-main-button{display:grid;place-items:center;width:54px;height:54px;padding:0;border-radius:14px}.timer-option{display:grid;place-items:center;width:54px;height:54px;border:0;border-radius:14px;background:#ffffff0d;color:#ffffff66;cursor:pointer}.timer-option.enabled{color:#ff9f43;background:#ff9f4320}.timer-active-controls .timer-main-button,.timer-active-controls .timer-reset{min-width:68px !important;width:68px !important;height:68px !important;padding:0 !important;display:grid;place-items:center}

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

  .timer-panel.active { justify-content: center; }
  .timer-active-layout { display: flex; align-items: center; gap: 9px; width: 100%; min-width: 0; }
  .timer-active-controls { display: flex; flex: none; align-items: center; gap: 6px; }
  .timer-active-info { display: flex; flex: 1; flex-direction: column; justify-content: center; min-width: 0; gap: 5px; }
  .timer-active-info .timer-readout { align-items: flex-end; }
  .timer-active-info .timer-readout strong { padding: 0; font-size: clamp(44px, 10cqw, 70px); line-height: .92; }
  .timer-active-info .timer-readout small { margin-top: 4px; font-size: 13px; }
  .timer-panel.complete{justify-content:center}
  .timer-complete-card{display:flex;flex-direction:column;justify-content:center;gap:12px;width:100%;min-width:0;padding:2px 0}
  .timer-complete-heading{display:flex;align-items:center;justify-content:space-between;gap:8px;min-width:0}
  .timer-complete-label{display:flex;align-items:center;gap:8px;min-width:0;color:#fff}
  .timer-complete-label :global(svg){flex:none;color:#ff9f43}
  .timer-complete-label strong{overflow:hidden;font-size:17px;line-height:1.2;text-overflow:ellipsis;white-space:nowrap}
  .timer-complete-time{color:#f4f7fb;font-size:clamp(38px,12cqw,64px);font-weight:700;line-height:.95;letter-spacing:-.04em;font-variant-numeric:tabular-nums}
  .timer-complete-bottom{display:flex;align-items:center;justify-content:space-between;gap:12px;color:rgba(229,235,245,.6);font-size:14px}
  .timer-complete-accent{width:34px;height:4px;flex:none;border-radius:4px;background:#ff9f43}
  .timer-complete-dismiss{display:grid;place-items:center;flex:none;width:38px;height:38px;padding:0;border:0;border-radius:12px;color:#ff6b6b;background:transparent;cursor:pointer;transition:color 140ms ease,background 140ms ease}
  .timer-complete-dismiss:hover,.timer-complete-dismiss:focus-visible{color:#ff8585;background:rgba(255,107,107,.14)}
  .timer-complete-dismiss:active{transform:scale(.96)}

  .timer-footer { display: flex; align-items: center; justify-content: space-between; gap: 8px; min-height: 0; margin-top: 5px; }
  .timer-actions { display: flex; align-items: center; gap: 5px; min-width: 0; }

  .timer-ruler {
    position: relative;
    flex: 0 0 56px;
    height: 56px;
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
    height: 54px;
    transform: translateX(calc(136px - var(--active-index) * 10px));
    transition: transform 260ms cubic-bezier(.22, 1, .36, 1);
  }

  .ruler-track button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    width: 10px;
    height: 53px;
    padding: 0;
    border: 0;
    color: transparent;
    background: transparent;
    cursor: inherit;
    text-transform: none;
  }

  .ruler-track button:disabled { cursor: default; }
  .tick-label { height: 19px; color: #fff; font: 600 11px/1 var(--app-font); }
  .ruler-track button:not(.major) .tick-label { opacity: 0; }
  .tick { display: block; width: 3px; height: 32px; border-radius: 999px; background: var(--tick-color); transform:scaleY(.667); transform-origin:center bottom; transition: transform 180ms ease, background 180ms ease, box-shadow 180ms ease; }
  .ruler-track button.major .tick { transform:scaleY(.852); background: var(--tick-color); }
  .ruler-track button.prior .tick { background: var(--tick-color); }
  .ruler-track button .tick-label { color: var(--tick-color); }
  .ruler-track button.selected .tick { transform:scaleY(1); background: var(--tick-color); box-shadow: 0 0 9px rgba(255, 159, 67, .65); }
  .ruler-track button.selected .tick-label { color: var(--tick-color); }

  .ruler-fade { position: absolute; top: 0; bottom: 0; z-index: 2; width: 42px; pointer-events: none; }
  .ruler-fade.left { left: 0; background: linear-gradient(90deg, rgba(0,0,0,.96), transparent); }
  .ruler-fade.right { right: 0; background: linear-gradient(270deg, rgba(0,0,0,.96), transparent); }

  .timer-main-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 28px;
    border: 0;
    border-radius: 12px;
    color: #ff9f43;
    background: rgba(255, 159, 67, .14);
    font: 700 11px/1 var(--app-font);
    letter-spacing: .01em;
    text-transform: none;
    white-space: nowrap;
    cursor: pointer;
    transition: width 180ms cubic-bezier(.22,1,.36,1), height 180ms cubic-bezier(.22,1,.36,1), background 150ms ease, color 150ms ease, border-radius 180ms cubic-bezier(.22,1,.36,1), box-shadow 150ms ease, transform 150ms cubic-bezier(.23,1,.32,1);
  }

  .timer-main-button { padding: 0 10px; font-size: 13px; }
  .timer-main-button:hover { color: #ff9f43; background: rgba(255, 159, 67, .22); }
  .timer-main-button:active { transform: scale(.95); }
  .timer-active-controls .timer-main-button { flex: none; width: auto; min-width: 78px; height: 44px; padding: 0 12px; border: 1px solid rgba(255,255,255,.12); border-radius: 12px; color: rgba(255,255,255,.9); background: rgba(255,255,255,.055); box-shadow: none; font-size: 20px; font-weight: 700; line-height: 20px; letter-spacing: .04em; }

  .timer-readout { display: flex; flex: 1; flex-direction: column; align-items: flex-end; min-width: 0; line-height: 1; }
  .timer-readout strong { display: block; box-sizing: border-box; width: 100%; max-width: 100%; padding-right: 6px; color: #fff4e9; font: 700 clamp(20px, 6vw, 24px)/.9 var(--app-font); letter-spacing: -.035em; text-align: right; white-space: nowrap; }
  .timer-readout small { margin-top: 3px; color: rgba(255, 190, 125, .58); font: 500 8px/1 var(--app-font); letter-spacing: .04em; }
  .timer-reset { flex: none; display: grid; place-items: center; width: 30px; height: 38px; padding: 0; border: 0; border-radius: 10px; color: #ff6b6b; background: transparent; cursor: pointer; transition: color 150ms ease, background 150ms ease, transform 150ms cubic-bezier(.23,1,.32,1); }
  .timer-active-controls .timer-reset { width: auto; min-width: 76px; height: 44px; padding: 0 12px; border: 1px solid rgba(255,107,107,.2); border-radius: 12px; color: #ff7777; background: rgba(255,107,107,.055); font: 700 20px/20px var(--app-font); letter-spacing: .04em; }
  .timer-reset:active { transform: scale(.95); }

  @media (hover: hover) and (pointer: fine) {
    .timer-active-controls .timer-main-button:hover { color: #fff; background: rgba(255,255,255,.1); }
    .timer-reset:hover { color: #ff8585; background: rgba(255,107,107,.14); }
  }

  .timer-active-controls .timer-main-button:focus-visible { color: #fff; background: rgba(255,255,255,.1); }
  .timer-reset:focus-visible { color: #ff8585; background: rgba(255,107,107,.14); }

  button:focus-visible { outline: 2px solid #fff; outline-offset: 2px; }

  @media (prefers-reduced-motion: reduce) {
    .ruler-track, .tick, .timer-main-button, .timer-reset { transition: none; }
  }
  .compact{justify-content:center}
  .compact .timer-footer{flex-direction:column-reverse;align-items:stretch;gap:12px}
  .compact .timer-readout{align-items:center;text-align:center}
  .compact .timer-readout strong{font-size:30px;text-align:center;padding-right:0}
  .compact .ruler-track{left:50%;transform:translateX(calc(-5px - var(--active-index) * 10px))}
  .compact .timer-actions{justify-content:center;gap:8px}
  .compact .timer-main-button{min-width:0;font-size:13px;padding:8px 10px}
  .compact .timer-complete-card{gap:7px}
  .compact .timer-complete-label strong{font-size:14px}
  .compact .timer-complete-time{font-size:36px}
  .compact .timer-complete-bottom{font-size:14px}
</style>
