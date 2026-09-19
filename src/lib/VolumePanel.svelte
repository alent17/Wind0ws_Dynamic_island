<script lang="ts">
  import { Volume2, VolumeX } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let {
    volume = 50,
    muted = false,
    onVolume,
  } = $props<{
    volume?: number;
    muted?: boolean;
    onVolume?: (volumePercent: number) => void | Promise<void>;
  }>();

  const t = (key: TranslationKey) => translate(key, {}, $locale);
  const displayVolume = $derived(muted ? 0 : Math.max(0, Math.min(100, Math.round(Number(volume) || 0))));
  let draftVolume = $state(0);
  let dragging = $state(false);

  $effect(() => {
    if (!dragging) draftVolume = displayVolume;
  });

  function handleVolumeInput(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    dragging = true;
    draftVolume = Math.max(0, Math.min(100, Number(input.value) || 0));
    void onVolume?.(Number(input.value));
  }

  function finishVolumeInput() {
    dragging = false;
    draftVolume = displayVolume;
  }
</script>

<div class="panel-volume" data-stop-toggle>
  <span class="volume-icon" aria-hidden="true">
    {#if muted || draftVolume === 0}
      <VolumeX size={21} />
    {:else}
      <Volume2 size={21} />
    {/if}
  </span>

  <div class="volume-slider-shell">
    <input
      class="volume-slider"
      type="range"
      min="0"
      max="100"
      step="1"
      value={draftVolume}
      style={`--volume-progress:${draftVolume}%`}
      aria-label={t("volume")}
      aria-valuenow={draftVolume}
      aria-valuetext={`${draftVolume}%`}
      oninput={handleVolumeInput}
      onchange={finishVolumeInput}
      onblur={finishVolumeInput}
    />
  </div>

  <strong>{draftVolume}%</strong>
</div>

<style>
  .panel-volume{width:100%;display:grid;grid-template-columns:30px minmax(0,1fr) 42px;align-items:center;gap:11px;color:rgba(255,255,255,.78)}
  .volume-icon{display:grid;place-items:center;width:30px;height:30px;border-radius:9px;color:rgba(255,255,255,.72);background:rgba(255,255,255,.07)}
  .volume-slider-shell{display:flex;align-items:center;min-width:0;height:30px}
  .volume-slider{--volume-progress:0%;display:block;width:100%;height:14px;margin:0;appearance:none;-webkit-appearance:none;writing-mode:horizontal-tb;direction:ltr;border:1px solid rgba(255,255,255,.25);border-radius:999px;background:linear-gradient(90deg,#a9a9ab 0%,#a9a9ab var(--volume-progress),#39393d var(--volume-progress),#39393d 100%);box-shadow:inset 0 1px 2px rgba(0,0,0,.4);cursor:ew-resize}
  .volume-slider::-webkit-slider-runnable-track{height:12px;border-radius:999px;background:transparent}
  .volume-slider::-webkit-slider-thumb{width:20px;height:20px;margin-top:-5px;appearance:none;-webkit-appearance:none;border:4px solid #737376;border-radius:50%;background:#d6d6d8;box-shadow:0 2px 5px rgba(0,0,0,.38)}
  .volume-slider::-moz-range-track{height:12px;border-radius:999px;background:transparent}
  .volume-slider::-moz-range-progress{height:12px;border-radius:999px;background:#a9a9ab}
  .volume-slider::-moz-range-thumb{width:12px;height:12px;border:4px solid #737376;border-radius:50%;background:#d6d6d8;box-shadow:0 2px 5px rgba(0,0,0,.38)}
  .volume-slider:focus-visible{outline:2px solid #fff;outline-offset:5px}
  .panel-volume strong{font-size:11px;font-weight:650;text-align:right;font-variant-numeric:tabular-nums;color:rgba(255,255,255,.88)}
</style>
