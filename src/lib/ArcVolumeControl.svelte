<script lang="ts">
  import { Volume1, Volume2, VolumeX } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    volume = 50,
    muted = false,
    onVolume,
    onOpen,
  } = $props<{
    volume?: number;
    muted?: boolean;
    onVolume?: (volumePercent: number) => void | Promise<void>;
    onOpen?: () => void | Promise<void>;
  }>();

  let dragging = $state(false);

  const t = (key: TranslationKey) => translate(key, {}, $locale);
  const clampedVolume = $derived(
    Math.max(0, Math.min(100, Number(volume) || 0)),
  );
  const displayVolume = $derived(muted ? 0 : clampedVolume);
  const progress = $derived(displayVolume / 100);
  const knobX = $derived(5 + 38 * progress);
  const knobY = $derived(26 - Math.sin(progress * Math.PI) * 15);

  function volumeFromPointer(event: PointerEvent) {
    const element = event.currentTarget as HTMLElement;
    const rect = element.getBoundingClientRect();
    const x = Math.max(5, Math.min(43, event.clientX - rect.left));

    return Math.round(((x - 5) / 38) * 100);
  }

  function handlePointerDown(event: PointerEvent) {
    event.stopPropagation();
    dragging = true;

    const element = event.currentTarget as HTMLElement;
    element.setPointerCapture(event.pointerId);

    void onOpen?.();
    void onVolume?.(volumeFromPointer(event));
  }

  function handlePointerMove(event: PointerEvent) {
    if (!dragging) return;

    event.stopPropagation();
    void onVolume?.(volumeFromPointer(event));
  }

  function finishPointer(event: PointerEvent) {
    if (!dragging) return;

    dragging = false;
    const element = event.currentTarget as HTMLElement;

    if (element.hasPointerCapture(event.pointerId)) {
      element.releasePointerCapture(event.pointerId);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    let next = displayVolume;

    if (event.key === "ArrowRight" || event.key === "ArrowUp") {
      next += event.shiftKey ? 10 : 5;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowDown") {
      next -= event.shiftKey ? 10 : 5;
    } else if (event.key === "Home") {
      next = 0;
    } else if (event.key === "End") {
      next = 100;
    } else {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    void onOpen?.();
    void onVolume?.(Math.max(0, Math.min(100, next)));
  }
</script>

<div
  class="arc-volume"
  class:dragging
  data-stop-toggle
  role="slider"
  tabindex="0"
  aria-label={t("volume")}
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={displayVolume}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={finishPointer}
  onpointercancel={finishPointer}
  onkeydown={handleKeyDown}
  onclick={(event) => event.stopPropagation()}
>
  <svg class="arc" viewBox="0 0 48 34" aria-hidden="true">
    <path class="arc-track" d="M 5 26 Q 24 1 43 26" pathLength="100" />
    <path
      class="arc-progress"
      d="M 5 26 Q 24 1 43 26"
      pathLength="100"
      style={`stroke-dasharray:${displayVolume} 100`}
    />
  </svg>

  <span
    class="volume-knob"
    style={`left:${knobX}px;top:${knobY}px`}
  >
    {#if displayVolume === 0}
      <VolumeX size={11} strokeWidth={2.2} />
    {:else if displayVolume < 50}
      <Volume1 size={11} strokeWidth={2.2} />
    {:else}
      <Volume2 size={11} strokeWidth={2.2} />
    {/if}
  </span>

  <span class="volume-value">{displayVolume}%</span>
</div>

<style>
  .arc-volume {
    position: relative;
    width: 48px;
    height: 38px;
    flex: none;
    touch-action: none;
    user-select: none;
    cursor: ew-resize;
    outline: none;
    opacity: 0.82;
    transition:
      opacity 160ms ease,
      transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .arc-volume:hover,
  .arc-volume:focus-visible,
  .arc-volume.dragging {
    opacity: 1;
  }

  .arc-volume:focus-visible {
    outline: 2px solid rgba(255, 255, 255, 0.82);
    outline-offset: 3px;
  }

  .arc {
    position: absolute;
    left: 0;
    top: 0;
    width: 48px;
    height: 34px;
    overflow: visible;
    pointer-events: none;
  }

  .arc-track,
  .arc-progress {
    fill: none;
    stroke-width: 2.6;
    stroke-linecap: round;
  }

  .arc-track {
    stroke: rgba(255, 255, 255, 0.13);
  }

  .arc-progress {
    stroke: rgba(255, 255, 255, 0.86);
    transition: stroke-dasharray 80ms linear;
  }

  .volume-knob {
    position: absolute;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    color: #050505;
    background: linear-gradient(145deg, #ffffff, #d8d8d8);
    box-shadow:
      0 2px 7px rgba(0, 0, 0, 0.55),
      inset 0 1px 1px rgba(255, 255, 255, 0.9);
    transform: translate(-50%, -50%) scale(1);
    pointer-events: none;
    transition:
      left 75ms linear,
      top 75ms linear,
      transform 140ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .dragging .volume-knob {
    transform: translate(-50%, -50%) scale(1.12);
    transition: transform 120ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .volume-value {
    position: absolute;
    left: 50%;
    bottom: -1px;
    color: rgba(255, 255, 255, 0.48);
    font: 600 8px/1 var(--app-font);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    transform: translateX(-50%);
    pointer-events: none;
    opacity: 0;
    transition:
      opacity 130ms ease,
      transform 160ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .arc-volume:hover .volume-value,
  .arc-volume:focus-visible .volume-value,
  .arc-volume.dragging .volume-value {
    opacity: 1;
    transform: translateX(-50%) translateY(1px);
  }

  @media (prefers-reduced-motion: reduce) {
    .arc-volume,
    .volume-knob,
    .volume-value,
    .arc-progress {
      transition: none;
    }
  }
</style>
