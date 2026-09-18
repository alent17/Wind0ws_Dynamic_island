<script lang="ts">
  import { Pause, Play, Plus, RotateCcw } from "lucide-svelte";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    status = "idle",
    remainingMs = 0,
    onStart,
    onPause,
    onResume,
    onAdjust,
    onReset,
  } = $props<{
    status?: CountdownStatus;
    remainingMs?: number;
    onStart?: (durationMs: number) => void;
    onPause?: () => void;
    onResume?: () => void;
    onAdjust?: (deltaMs: number) => void;
    onReset?: () => void;
  }>();

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  const active = $derived((status === "running" || status === "paused") && remainingMs > 0);
  let selectedMinutes = $state(20);
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
    selectedMinutes = Math.max(1, Math.min(24 * 60, minutes));
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

<div class="timer-panel" data-stop-toggle>
  <div class="timer-ruler" aria-label={t("selectDuration")}>
    <div class="ruler-track" style={`--active-index:${activeIndex}`}>
      {#each rulerValues as minutes}
        <button
          class:major={minutes % 5 === 0}
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
    <span class="ruler-pointer" aria-hidden="true"></span>
    <span class="ruler-fade left" aria-hidden="true"></span>
    <span class="ruler-fade right" aria-hidden="true"></span>
  </div>

  <div class="timer-footer">
    <div class="timer-actions">
      <button class="timer-main-button" type="button" aria-label={mainLabel} onclick={(event) => { event.stopPropagation(); handleMainAction(); }}>
        {#if active}{#if status === "running"}<Pause size={13} fill="currentColor" />{:else}<Play size={13} fill="currentColor" />{/if}{/if}
        <span>{mainLabel}</span>
      </button>
      {#if active}
        <button class="timer-mini-button" type="button" aria-label={t("addMinute")} onclick={(event) => { event.stopPropagation(); onAdjust?.(60_000); }}><Plus size={13} />1m</button>
        <button class="timer-mini-button" type="button" aria-label={t("addFiveMinutes")} onclick={(event) => { event.stopPropagation(); onAdjust?.(300_000); }}><Plus size={13} />5m</button>
      {/if}
    </div>

    <div class="timer-readout" aria-live="polite">
      <strong>{formatCountdown(previewMs)}</strong>
      <small>{active ? (status === "paused" ? t("timerPaused") : t("timerRunning")) : t("timerReady")}</small>
    </div>
  </div>

  {#if active}
    <button class="timer-reset" type="button" aria-label={t("cancelTimer")} onclick={(event) => { event.stopPropagation(); onReset?.(); }}>
      <RotateCcw size={13} />
    </button>
  {/if}
</div>

<style>
  .timer-panel {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 124px;
    color: #fff;
    font-variant-numeric: tabular-nums;
  }

  .timer-ruler {
    position: relative;
    height: 69px;
    margin: 0 -10px;
    overflow: hidden;
    mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
    -webkit-mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
  }

  .ruler-track {
    position: absolute;
    top: 2px;
    left: 0;
    display: flex;
    align-items: flex-start;
    width: max-content;
    height: 66px;
    transform: translateX(calc(122px - var(--active-index) * 12px));
    transition: transform 260ms cubic-bezier(.22, 1, .36, 1);
  }

  .ruler-track button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    width: 12px;
    height: 63px;
    padding: 0;
    border: 0;
    color: transparent;
    background: transparent;
    cursor: pointer;
    text-transform: none;
  }

  .ruler-track button:disabled { cursor: default; }
  .tick-label { height: 23px; color: rgba(242, 139, 49, .74); font: 600 10px/1 var(--app-font, "Segoe UI", sans-serif); }
  .ruler-track button:not(.major) .tick-label { opacity: 0; }
  .tick { display: block; width: 4px; height: 25px; border-radius: 999px; background: rgba(178, 89, 31, .34); transition: height 180ms ease, background 180ms ease, box-shadow 180ms ease; }
  .ruler-track button.major .tick { height: 31px; background: rgba(234, 119, 38, .72); }
  .ruler-track button.selected .tick { height: 35px; background: #ffbc55; box-shadow: 0 0 9px rgba(255, 154, 48, .86); }
  .ruler-track button.selected .tick-label { color: #ffd18a; }

  .ruler-pointer {
    position: absolute;
    bottom: 0;
    left: 50%;
    width: 0;
    height: 0;
    border-right: 6px solid transparent;
    border-bottom: 9px solid #f28b31;
    border-left: 6px solid transparent;
    transform: translateX(-50%);
    filter: drop-shadow(0 0 5px rgba(242, 139, 49, .55));
  }

  .ruler-fade { position: absolute; top: 0; bottom: 0; z-index: 2; width: 42px; pointer-events: none; }
  .ruler-fade.left { left: 0; background: linear-gradient(90deg, rgba(0,0,0,.96), transparent); }
  .ruler-fade.right { right: 0; background: linear-gradient(270deg, rgba(0,0,0,.96), transparent); }

  .timer-footer { display: flex; align-items: flex-end; justify-content: space-between; gap: 10px; margin-top: auto; }
  .timer-actions { display: flex; align-items: center; gap: 5px; min-width: 0; }
  .timer-main-button, .timer-mini-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 32px;
    border: 0;
    border-radius: 17px;
    color: #f5a052;
    background: rgba(242, 139, 49, .12);
    font: 650 11px/1 var(--app-font, "Segoe UI", sans-serif);
    letter-spacing: .01em;
    text-transform: none;
    white-space: nowrap;
    cursor: pointer;
    transition: background 150ms ease, color 150ms ease, transform 150ms cubic-bezier(.23,1,.32,1);
  }

  .timer-main-button { padding: 0 13px; }
  .timer-mini-button { height: 27px; padding: 0 7px; border: 1px solid rgba(242,139,49,.18); color: rgba(255,196,124,.8); background: rgba(242,139,49,.07); font-size: 9px; }
  .timer-main-button:hover, .timer-mini-button:hover { color: #ffd096; background: rgba(242, 139, 49, .22); }
  .timer-main-button:active, .timer-mini-button:active { transform: scale(.95); }

  .timer-readout { display: flex; flex-direction: column; align-items: flex-end; min-width: 0; }
  .timer-readout strong { color: #f28b31; font: 300 clamp(36px, 11vw, 50px)/.84 var(--app-font, "Segoe UI", sans-serif); letter-spacing: -.075em; white-space: nowrap; }
  .timer-readout small { margin-top: 6px; color: rgba(242, 139, 49, .5); font: 600 9px/1 var(--app-font, "Segoe UI", sans-serif); letter-spacing: .06em; }
  .timer-reset { position: absolute; top: -3px; right: -4px; display: grid; place-items: center; width: 24px; height: 24px; padding: 0; border: 0; border-radius: 50%; color: rgba(255,255,255,.45); background: transparent; cursor: pointer; }
  .timer-reset:hover { color: #fff; background: rgba(255,255,255,.1); }

  button:focus-visible { outline: 2px solid #fff; outline-offset: 2px; }

  @media (prefers-reduced-motion: reduce) {
    .ruler-track, .tick, .timer-main-button, .timer-mini-button { transition: none; }
  }
</style>
