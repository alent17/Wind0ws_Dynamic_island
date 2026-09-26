<script lang="ts">
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { AudioDeviceInfo } from "$lib/api/types";
  import { Check, ChevronDown, LoaderCircle } from "lucide-svelte";
  import RollingNumber from "$lib/RollingNumber.svelte";

  let {
    volume = 50,
    muted = false,
    deviceId = "",
    devices = [],
    switchingDevice = false,
    reduceMotion = false,
    onVolume,
    onDevice,
  } = $props<{
    volume?: number;
    muted?: boolean;
    deviceId?: string;
    devices?: AudioDeviceInfo[];
    switchingDevice?: boolean;
    reduceMotion?: boolean;
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

  let rulerElement: HTMLDivElement;
  let pointerId: number | null = null;
  let startX = 0;
  let startVolume = 0;
  let moved = false;
  const rulerValues = Array.from({ length: 101 }, (_, value) => value);

  function chooseVolume(value: number) {
    draftVolume = Math.max(0, Math.min(100, Math.round(value)));
    void onVolume?.(draftVolume);
  }

  function startDrag(event: PointerEvent) {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    deviceMenuOpen = false;
    pointerId = event.pointerId;
    startX = event.clientX;
    startVolume = draftVolume;
    dragging = true;
    moved = false;
    rulerElement.setPointerCapture(event.pointerId);
  }

  function moveDrag(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;
    if (Math.abs(event.clientX - startX) >= 3) moved = true;
    if (moved) chooseVolume(startVolume + (startX - event.clientX) / 10);
  }

  function endDrag(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;
    if (!moved && event.type === "pointerup") {
      const rect = rulerElement.getBoundingClientRect();
      chooseVolume(startVolume + (event.clientX - rect.left - rect.width / 2) / 10);
    }
    pointerId = null;
    if (rulerElement.hasPointerCapture(event.pointerId)) rulerElement.releasePointerCapture(event.pointerId);
    dragging = false;
  }

  function rulerKeydown(event: KeyboardEvent) {
    const changes: Record<string, number> = { ArrowLeft: -1, ArrowDown: -1, ArrowRight: 1, ArrowUp: 1, PageDown: -10, PageUp: 10 };
    if (event.key in changes || event.key === "Home" || event.key === "End") {
      event.preventDefault();
      chooseVolume(event.key === "Home" ? 0 : event.key === "End" ? 100 : draftVolume + changes[event.key]);
    }
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

<div class="panel-volume" class:reduce-motion={reduceMotion}>
  <div class="volume-ruler" class:dragging bind:this={rulerElement}
    role="slider" tabindex="0" aria-label={t("volume")}
    aria-valuemin={0} aria-valuemax={100} aria-valuenow={draftVolume} aria-valuetext={`${draftVolume}%`}
    onpointerdown={startDrag} onpointermove={moveDrag} onpointerup={endDrag}
    onpointercancel={endDrag} onlostpointercapture={endDrag} onkeydown={rulerKeydown}>
    <div class="ruler-track" style={`--active-index:${draftVolume}`} aria-hidden="true">
      {#each rulerValues as value}
        <span class="ruler-mark" class:major={value % 5 === 0} class:prior={value < draftVolume} class:selected={value === draftVolume}>
          <span class="tick-label">{value}</span><span class="tick"></span>
        </span>
      {/each}
    </div>
  </div>
  <div class="volume-footer">
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

    <div class="volume-readout">
      <strong><RollingNumber value={String(draftVolume)} {reduceMotion} /><small>%</small></strong>
      <span class="volume-label">{draftVolume === 0 ? t("muted") : t("volume")}</span>
    </div>
  </div>
</div>

<style>
  .panel-volume{position:relative;width:100%;height:100%;display:flex;flex-direction:column;color:#eaf4ff;user-select:none;font-family:var(--app-font, system-ui)}
  .volume-ruler{position:relative;flex:0 0 44px;height:44px;margin:0 -6px;overflow:hidden;touch-action:none;cursor:grab;mask-image:linear-gradient(90deg,transparent,#000 10%,#000 90%,transparent)}
  .volume-ruler.dragging{cursor:grabbing}
  .ruler-track{position:absolute;top:2px;left:50%;display:flex;width:max-content;height:42px;transform:translateX(calc(-5px - var(--active-index)*10px));transition:transform 180ms cubic-bezier(.22,1,.36,1)}
  .dragging .ruler-track,.reduce-motion .ruler-track{transition:none}
  .ruler-mark{display:flex;flex-direction:column;align-items:center;width:10px;height:41px}
  .tick-label{height:15px;color:rgba(110,178,255,.78);font:500 8px/1 var(--app-font, system-ui)}
  .ruler-mark:not(.major) .tick-label{opacity:0}
  .tick{display:block;width:3px;height:24px;border-radius:999px;background:rgba(130,153,185,.3);transform:scaleY(.667);transform-origin:center bottom}
  .major .tick{transform:scaleY(.852);background:rgba(105,170,255,.7)}
  .prior .tick{background:#eaf4ff}.prior .tick-label,.selected .tick-label{color:#b9ddff}
  .selected .tick{transform:scaleY(1);background:#eaf4ff;box-shadow:0 0 9px rgba(234,244,255,.65)}
  .volume-ruler:focus-visible{outline:2px solid #91caff;outline-offset:-2px;border-radius:5px}
  .volume-footer{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:5px;min-width:0}
  .volume-readout{display:flex;flex-direction:column;align-items:flex-end;flex:none;min-width:54px;gap:3px}
  .device-row{position:relative;z-index:3;flex:1;display:grid;grid-template-columns:minmax(0,1fr);align-items:center;gap:9px;min-width:0}
  .device-label{display:none;color:rgba(255,255,255,.46);font-size:9px;font-weight:600;line-height:1;white-space:nowrap}
  .device-trigger{min-width:0;width:100%;height:26px;display:grid;grid-template-columns:minmax(0,1fr) 14px;align-items:center;gap:5px;padding:0 7px 0 9px;overflow:hidden;border:1px solid rgba(255,255,255,.1);border-radius:8px;color:rgba(255,255,255,.82);background:rgba(255,255,255,.07);font:600 9px/1 var(--app-font, system-ui);text-align:left;cursor:pointer;transition:border-color 140ms ease,background 140ms ease}
  .device-trigger>span,.device-menu button>span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .device-trigger:hover:not(:disabled),.device-trigger.open{border-color:rgba(105,170,255,.32);background:rgba(255,255,255,.1)}
  .device-trigger:disabled{opacity:.48;cursor:default}
  .device-trigger:focus-visible,.device-menu button:focus-visible{outline:2px solid rgba(255,255,255,.9);outline-offset:2px}
  .device-trigger :global(svg){justify-self:end;flex:none;color:rgba(255,255,255,.48)}
  .device-trigger.open :global(.device-chevron){transform:rotate(180deg)}
  .device-trigger :global(.device-chevron){transition:transform 180ms cubic-bezier(.23,1,.32,1)}
  .device-trigger :global(.device-spinner){animation:device-spin 700ms linear infinite}
  .device-menu{position:absolute;bottom:30px;right:0;z-index:8;width:100%;max-height:78px;padding:4px;overflow-x:hidden;overflow-y:auto;border:1px solid rgba(255,255,255,.12);border-radius:10px;background:#171719;box-shadow:0 10px 24px rgba(0,0,0,.5);animation:device-menu-in 150ms cubic-bezier(.23,1,.32,1)}
  .device-menu button{width:100%;height:28px;display:grid;grid-template-columns:minmax(0,1fr) 14px;align-items:center;gap:6px;padding:0 7px;border:0;border-radius:7px;color:rgba(255,255,255,.68);background:transparent;font:600 9px/1 var(--app-font, system-ui);text-align:left;cursor:pointer}
  .device-menu button:hover,.device-menu button:focus-visible{color:#fff;background:rgba(255,255,255,.08)}
  .device-menu button.active{color:#fff;background:rgba(65,145,235,.14)}
  .device-menu button :global(svg){justify-self:end;color:#91caff}
  .device-menu::-webkit-scrollbar{width:4px}.device-menu::-webkit-scrollbar-thumb{border-radius:999px;background:rgba(255,255,255,.2)}
  .volume-label{color:rgba(138,190,245,.58);font:500 8px/1 var(--app-font, system-ui);white-space:nowrap}
  .volume-readout strong{display:flex;align-items:baseline;color:#eaf4ff;font:700 24px/.9 var(--app-font, system-ui);letter-spacing:-.035em;font-variant-numeric:tabular-nums}
  .volume-readout strong small{margin-left:2px;font-size:10px;color:rgba(138,190,245,.58)}
  @keyframes device-menu-in{from{opacity:.45;transform:translateY(-4px) scale(.985)}to{opacity:1;transform:translateY(0) scale(1)}}
  @keyframes device-spin{to{transform:rotate(360deg)}}
  @media (prefers-reduced-motion:reduce){.ruler-track,.device-trigger{transition:none}.device-trigger :global(.device-chevron){transition:none}.device-menu,.device-trigger :global(.device-spinner){animation:none}}
</style>
