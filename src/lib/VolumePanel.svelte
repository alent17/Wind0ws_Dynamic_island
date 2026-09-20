<script lang="ts">
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { AudioDeviceInfo } from "$lib/api/types";

  let {
    volume = 50,
    muted = false,
    deviceId = "",
    devices = [],
    switchingDevice = false,
    onVolume,
    onDevice,
  } = $props<{
    volume?: number;
    muted?: boolean;
    deviceId?: string;
    devices?: AudioDeviceInfo[];
    switchingDevice?: boolean;
    onVolume?: (volumePercent: number) => void | Promise<void>;
    onDevice?: (deviceId: string) => void | Promise<void>;
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

  function handleDeviceChange(event: Event) {
    const nextDeviceId = (event.currentTarget as HTMLSelectElement).value;
    if (nextDeviceId && nextDeviceId !== deviceId) void onDevice?.(nextDeviceId);
  }
</script>

<div class="panel-volume" data-stop-toggle>
  <div class="device-row">
    <label for="audio-output-device">{t("audioOutput")}</label>
    <select
      id="audio-output-device"
      value={deviceId}
      disabled={switchingDevice || devices.length === 0}
      aria-label={t("audioOutput")}
      aria-busy={switchingDevice}
      onchange={handleDeviceChange}
    >
      {#if devices.length === 0}
        <option value="">{t("noAudioDevices")}</option>
      {:else}
        {#each devices as device}
          <option value={device.id}>{device.name}</option>
        {/each}
      {/if}
    </select>
  </div>

  <span class="volume-label">{muted || draftVolume === 0 ? t("muted") : t("volume")}</span>
  <strong><span>{draftVolume}</span><small>%</small></strong>

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
</div>

<style>
  .panel-volume{width:100%;display:grid;grid-template-columns:minmax(0,1fr) auto;grid-template-rows:26px 24px 18px;align-items:center;column-gap:16px;row-gap:4px;color:#fff;user-select:none}
  .device-row{grid-column:1/-1;display:grid;grid-template-columns:auto minmax(0,1fr);align-items:center;gap:9px;min-width:0}
  .device-row label{color:rgba(255,255,255,.46);font-size:9px;font-weight:600;line-height:1;white-space:nowrap}
  .device-row select{min-width:0;width:100%;height:26px;padding:0 25px 0 9px;overflow:hidden;border:1px solid rgba(255,255,255,.1);border-radius:8px;color:rgba(255,255,255,.82);background-color:rgba(255,255,255,.07);font:600 9px/1 var(--app-font);text-overflow:ellipsis;white-space:nowrap;cursor:pointer;color-scheme:dark}
  .device-row select:hover:not(:disabled){border-color:rgba(255,255,255,.18);background-color:rgba(255,255,255,.1)}
  .device-row select:disabled{opacity:.48;cursor:default}
  .device-row select:focus-visible{outline:2px solid rgba(255,255,255,.9);outline-offset:2px}
  .volume-label{align-self:end;color:rgba(255,255,255,.58);font-size:10px;font-weight:600;line-height:1;letter-spacing:.01em}
  .panel-volume strong{align-self:end;display:flex;align-items:baseline;justify-content:flex-end;min-width:48px;color:#fff;font-size:23px;font-weight:700;line-height:.78;letter-spacing:-.035em;font-variant-numeric:tabular-nums}
  .panel-volume strong small{margin-left:2px;color:rgba(255,255,255,.46);font-size:9px;font-weight:650;letter-spacing:0}
  .volume-slider-shell{grid-column:1/-1;display:flex;align-items:center;min-width:0;height:18px;padding:0 2px}
  .volume-slider{--volume-progress:0%;display:block;width:100%;height:5px;margin:0;appearance:none;-webkit-appearance:none;writing-mode:horizontal-tb;direction:ltr;border:0;border-radius:999px;background:linear-gradient(90deg,#f59a23 0%,#f59a23 var(--volume-progress),rgba(255,255,255,.15) var(--volume-progress),rgba(255,255,255,.15) 100%);cursor:ew-resize}
  .volume-slider::-webkit-slider-runnable-track{height:5px;border-radius:999px;background:transparent}
  .volume-slider::-webkit-slider-thumb{width:14px;height:14px;margin-top:-4.5px;appearance:none;-webkit-appearance:none;border:3px solid #fff;border-radius:50%;background:#f59a23;box-shadow:0 2px 7px rgba(0,0,0,.45);transition:transform 150ms cubic-bezier(.23,1,.32,1),box-shadow 150ms ease}
  .volume-slider::-moz-range-track{height:5px;border-radius:999px;background:rgba(255,255,255,.15)}
  .volume-slider::-moz-range-progress{height:5px;border-radius:999px;background:#f59a23}
  .volume-slider::-moz-range-thumb{width:8px;height:8px;border:3px solid #fff;border-radius:50%;background:#f59a23;box-shadow:0 2px 7px rgba(0,0,0,.45);transition:transform 150ms cubic-bezier(.23,1,.32,1),box-shadow 150ms ease}
  .volume-slider:hover::-webkit-slider-thumb{transform:scale(1.12);box-shadow:0 3px 9px rgba(0,0,0,.52)}
  .volume-slider:hover::-moz-range-thumb{transform:scale(1.12);box-shadow:0 3px 9px rgba(0,0,0,.52)}
  .volume-slider:active::-webkit-slider-thumb{transform:scale(.92)}
  .volume-slider:active::-moz-range-thumb{transform:scale(.92)}
  .volume-slider:focus-visible{outline:2px solid rgba(255,255,255,.9);outline-offset:6px}
  @media (prefers-reduced-motion:reduce){.volume-slider,.volume-slider::-webkit-slider-thumb,.volume-slider::-moz-range-thumb{transition:none}}
</style>
