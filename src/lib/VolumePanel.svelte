<script lang="ts">
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { AudioDeviceInfo } from "$lib/api/types";
  import { Check, ChevronDown, LoaderCircle } from "lucide-svelte";

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
  const selectedDevice = $derived(devices.find((device: AudioDeviceInfo) => device.id === deviceId) ?? devices[0]);
  let draftVolume = $state(0);
  let dragging = $state(false);
  let deviceMenuOpen = $state(false);

  $effect(() => {
    if (!dragging) draftVolume = displayVolume;
  });

  $effect(() => {
    if (switchingDevice || devices.length === 0) deviceMenuOpen = false;
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

  function selectDevice(nextDeviceId: string) {
    deviceMenuOpen = false;
    if (nextDeviceId && nextDeviceId !== deviceId) void onDevice?.(nextDeviceId);
  }

  function handleDeviceKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      deviceMenuOpen = false;
    }
  }

</script>

<div class="panel-volume" data-stop-toggle>
  <div class="device-row">
    <span class="device-label">{t("audioOutput")}</span>
    <button
      class="device-trigger"
      class:open={deviceMenuOpen}
      type="button"
      disabled={switchingDevice || devices.length === 0}
      aria-label={t("audioOutput")}
      aria-busy={switchingDevice}
      aria-haspopup="listbox"
      aria-expanded={deviceMenuOpen}
      onclick={() => deviceMenuOpen = !deviceMenuOpen}
      onkeydown={handleDeviceKeydown}
    >
      <span>{selectedDevice?.name ?? t("noAudioDevices")}</span>
      {#if switchingDevice}
        <LoaderCircle class="device-spinner" size={12} strokeWidth={2.1} aria-hidden="true" />
      {:else}
        <ChevronDown class="device-chevron" size={12} strokeWidth={2.2} aria-hidden="true" />
      {/if}
    </button>

    {#if deviceMenuOpen}
      <div class="device-menu" role="listbox" aria-label={t("audioOutput")}>
        {#each devices as device}
          <button
            class:active={device.id === deviceId}
            type="button"
            role="option"
            aria-selected={device.id === deviceId}
            title={device.name}
            onclick={() => selectDevice(device.id)}
            onkeydown={handleDeviceKeydown}
          >
            <span>{device.name}</span>
            {#if device.id === deviceId}<Check size={12} strokeWidth={2.4} aria-hidden="true" />{/if}
          </button>
        {/each}
      </div>
    {/if}
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
      onfocus={() => deviceMenuOpen = false}
      oninput={handleVolumeInput}
      onchange={finishVolumeInput}
      onblur={finishVolumeInput}
    />
  </div>
</div>

<style>
  .panel-volume{position:relative;width:100%;display:grid;grid-template-columns:minmax(0,1fr) auto;grid-template-rows:26px 24px 18px;align-items:center;column-gap:16px;row-gap:4px;color:#fff;user-select:none}
  .device-row{position:relative;z-index:3;grid-column:1/-1;display:grid;grid-template-columns:auto minmax(0,1fr);align-items:center;gap:9px;min-width:0}
  .device-label{color:rgba(255,255,255,.46);font-size:9px;font-weight:600;line-height:1;white-space:nowrap}
  .device-trigger{min-width:0;width:100%;height:26px;display:grid;grid-template-columns:minmax(0,1fr) 14px;align-items:center;gap:5px;padding:0 7px 0 9px;overflow:hidden;border:1px solid rgba(255,255,255,.1);border-radius:8px;color:rgba(255,255,255,.82);background:rgba(255,255,255,.07);font:600 9px/1 var(--app-font);text-align:left;cursor:pointer;transition:border-color 140ms ease,background 140ms ease}
  .device-trigger>span,.device-menu button>span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .device-trigger:hover:not(:disabled),.device-trigger.open{border-color:rgba(245,154,35,.32);background:rgba(255,255,255,.1)}
  .device-trigger:disabled{opacity:.48;cursor:default}
  .device-trigger:focus-visible,.device-menu button:focus-visible{outline:2px solid rgba(255,255,255,.9);outline-offset:2px}
  .device-trigger :global(svg){justify-self:end;flex:none;color:rgba(255,255,255,.48)}
  .device-trigger.open :global(.device-chevron){transform:rotate(180deg)}
  .device-trigger :global(.device-chevron){transition:transform 180ms cubic-bezier(.23,1,.32,1)}
  .device-trigger :global(.device-spinner){animation:device-spin 700ms linear infinite}
  .device-menu{position:absolute;top:30px;right:0;z-index:8;width:calc(100% - 43px);max-height:78px;padding:4px;overflow-x:hidden;overflow-y:auto;border:1px solid rgba(255,255,255,.12);border-radius:10px;background:#171719;box-shadow:0 10px 24px rgba(0,0,0,.5);animation:device-menu-in 150ms cubic-bezier(.23,1,.32,1)}
  .device-menu button{width:100%;height:28px;display:grid;grid-template-columns:minmax(0,1fr) 14px;align-items:center;gap:6px;padding:0 7px;border:0;border-radius:7px;color:rgba(255,255,255,.68);background:transparent;font:600 9px/1 var(--app-font);text-align:left;cursor:pointer}
  .device-menu button:hover,.device-menu button:focus-visible{color:#fff;background:rgba(255,255,255,.08)}
  .device-menu button.active{color:#fff;background:rgba(245,154,35,.12)}
  .device-menu button :global(svg){justify-self:end;color:#f59a23}
  .device-menu::-webkit-scrollbar{width:4px}.device-menu::-webkit-scrollbar-thumb{border-radius:999px;background:rgba(255,255,255,.2)}
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
  @keyframes device-menu-in{from{opacity:.45;transform:translateY(-4px) scale(.985)}to{opacity:1;transform:translateY(0) scale(1)}}
  @keyframes device-spin{to{transform:rotate(360deg)}}
  @media (prefers-reduced-motion:reduce){.volume-slider,.volume-slider::-webkit-slider-thumb,.volume-slider::-moz-range-thumb,.device-trigger{transition:none}.device-trigger :global(.device-chevron){transition:none}.device-menu,.device-trigger :global(.device-spinner){animation:none}}
</style>
