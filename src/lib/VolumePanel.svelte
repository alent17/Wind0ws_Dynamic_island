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

  function handleVolumeInput(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    void onVolume?.(Number(input.value));
  }
</script>

<div class="panel-volume" data-stop-toggle>
  {#if muted || displayVolume === 0}
    <VolumeX size={22} aria-hidden="true" />
  {:else}
    <Volume2 size={22} aria-hidden="true" />
  {/if}

  <input
    type="range"
    min="0"
    max="100"
    step="1"
    value={displayVolume}
    aria-label={t("volume")}
    aria-valuetext={`${displayVolume}%`}
    oninput={handleVolumeInput}
  />

  <strong>{displayVolume}%</strong>
</div>

<style>
  .panel-volume{width:100%;display:grid;grid-template-columns:24px minmax(0,1fr) 34px;align-items:center;gap:10px;color:rgba(255,255,255,.78)}
  .panel-volume input{width:100%;accent-color:#f59a23;cursor:ew-resize}
  .panel-volume input:focus-visible{outline:2px solid #fff;outline-offset:4px}
  .panel-volume strong{font-size:10px;font-weight:600;text-align:right;font-variant-numeric:tabular-nums;color:rgba(255,255,255,.84)}
</style>
