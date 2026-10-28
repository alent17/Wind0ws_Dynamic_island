<script lang="ts">
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { AudioDeviceInfo } from "$lib/api/types";
  import { Check, ChevronDown, LoaderCircle } from "lucide-svelte";
  import RollingNumber from "$lib/components/time/RollingNumber.svelte";

  let {
    compact = false,
    volume = 50,
    muted = false,
    deviceId = "",
    devices = [],
    switchingDevice = false,
    reduceMotion = false,
    onVolume,
    onDevice,
    onExpand,
  } = $props<{
    compact?: boolean;
    volume?: number;
    muted?: boolean;
    deviceId?: string;
    devices?: AudioDeviceInfo[];
    switchingDevice?: boolean;
    reduceMotion?: boolean;
    onVolume?: (volumePercent: number) => void | Promise<void>;
    onDevice?: (deviceId: string) => void | Promise<void>;
    onExpand?: () => void;
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
  let moved = false;
  const rulerValues = Array.from({ length: 21 }, (_, index) => index * 5);

  // The menu must keep its own scrolling; a downward gesture elsewhere on
  // the card opens its extension before the island's volume wheel sees it.
  function extensionGesture(node: HTMLElement) {
    let gesture: { id:number; x:number; y:number } | null = null;
    const wheel = (event: WheelEvent) => {
      if ((event.target as HTMLElement).closest(".device-menu")) { event.stopPropagation(); return; }
      if (deviceMenuOpen) { event.stopPropagation(); return; }
      if (!onExpand || event.ctrlKey || event.deltaY <= 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
      event.preventDefault(); event.stopPropagation(); onExpand();
    };
    const down = (event: PointerEvent) => {
      if (!onExpand || deviceMenuOpen || !event.isPrimary || event.button !== 0 || (event.target as HTMLElement).closest("button,input,[role=slider]")) return;
      gesture = { id:event.pointerId, x:event.clientX, y:event.clientY };
      node.setPointerCapture(event.pointerId);
    };
    const move = (event: PointerEvent) => {
      if (!gesture || gesture.id !== event.pointerId) return;
      const dx = event.clientX - gesture.x, dy = event.clientY - gesture.y;
      if (dy > 28 && dy > Math.abs(dx)) { gesture = null; onExpand?.(); }
      else if (Math.abs(dx) > 28 || dy < -28) gesture = null;
    };
    const end = (event: PointerEvent) => { if (gesture?.id === event.pointerId) gesture = null; };
    node.addEventListener("wheel", wheel, {passive:false});
    node.addEventListener("pointerdown", down); node.addEventListener("pointermove", move);
    node.addEventListener("pointerup", end); node.addEventListener("pointercancel", end);
    return { destroy() { node.removeEventListener("wheel", wheel); node.removeEventListener("pointerdown", down); node.removeEventListener("pointermove", move); node.removeEventListener("pointerup", end); node.removeEventListener("pointercancel", end); } };
  }

  function chooseVolume(value: number) {
    draftVolume = Math.max(0, Math.min(100, Math.round(value)));
    void onVolume?.(draftVolume);
  }

  function startDrag(event: PointerEvent) {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    deviceMenuOpen = false;
    pointerId = event.pointerId;
    startX = event.clientX;
    dragging = true;
    moved = false;
    rulerElement.setPointerCapture(event.pointerId);
  }

  function moveDrag(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;
    if (Math.abs(event.clientX - startX) >= 3) moved = true;
    if (moved) chooseVolume(volumeAtPointer(event.clientX));
  }

  function volumeAtPointer(clientX: number) {
    const rect = rulerElement.getBoundingClientRect();
    return rect.width > 0 ? (clientX - rect.left) / rect.width * 100 : draftVolume;
  }

  function endDrag(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;
    if (event.type === "pointerup") {
      chooseVolume(volumeAtPointer(event.clientX));
    }
    pointerId = null;
    if (rulerElement.hasPointerCapture(event.pointerId)) rulerElement.releasePointerCapture(event.pointerId);
    dragging = false;
  }

  function rulerKeydown(event: KeyboardEvent) {
    if (event.shiftKey && event.key === "ArrowDown" && onExpand) { event.preventDefault(); event.stopPropagation(); onExpand(); return; }
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

<div class="panel-volume" class:compact class:reduce-motion={reduceMotion} data-stop-toggle use:extensionGesture>
  <div class="volume-heading">
    <div class="volume-title"><span class="volume-label">{draftVolume === 0 ? t("muted") : t("volume")}</span></div>
    <div class="volume-readout">
      <strong><RollingNumber value={String(draftVolume)} {reduceMotion} /></strong>
    </div>
  </div>
  <div class="volume-ruler" class:dragging bind:this={rulerElement}
    role="slider" tabindex="0" aria-label={t("volume")}
    aria-valuemin={0} aria-valuemax={100} aria-valuenow={draftVolume} aria-valuetext={`${draftVolume}%`} aria-keyshortcuts={onExpand ? "Shift+ArrowDown" : undefined}
    onpointerdown={startDrag} onpointermove={moveDrag} onpointerup={endDrag}
    onpointercancel={endDrag} onlostpointercapture={endDrag} onkeydown={rulerKeydown}>
    <div class="volume-track" aria-hidden="true">
      <span class="volume-fill" style={`width:${draftVolume}%`}></span>
      {#each rulerValues as value}
        <span class="volume-mark" class:passed={value <= draftVolume} style={`left:${value}%`}>
          <span class="volume-tick"></span>
        </span>
      {/each}
    </div>
  </div>
  <div class="volume-footer">
  <div class="device-row">
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
  </div>
</div>

<style>
  .panel-volume{position:relative;width:100%;height:100%;display:flex;flex-direction:column;justify-content:center;gap:11px;color:#eaf4ff;user-select:none;font-family:var(--app-font, system-ui)}
  .volume-heading{display:flex;align-items:flex-end;justify-content:space-between;gap:12px;min-width:0}
  .volume-title{display:flex;align-items:baseline;gap:8px;min-width:0}
  .volume-ruler{position:relative;flex:0 0 72px;height:72px;overflow:visible;touch-action:none;cursor:pointer}
  .volume-ruler.dragging{cursor:grabbing}
  .volume-track{position:absolute;inset:8px 0;overflow:hidden;border-radius:8px;background:transparent}
  .volume-fill{position:absolute;top:8px;bottom:8px;left:0;max-width:100%;border-radius:2px;background:#193a8e;transition:width 100ms linear}
  .volume-mark{position:absolute;top:0;bottom:0;display:flex;width:3px;align-items:center;justify-content:center;transform:translateX(-50%);pointer-events:none}
  .volume-tick{display:block;width:3px;height:52px;border-radius:1px;background:#fff;transition:opacity 120ms ease,transform 120ms ease}
  .volume-mark.passed .volume-tick{opacity:0;transform:scaleY(.8)}
  .volume-ruler:focus-visible{outline:2px solid #91caff;outline-offset:4px;border-radius:5px}
  .volume-footer{display:flex;align-items:center;gap:12px;min-width:0}
  .volume-readout{display:flex;flex-direction:column;align-items:flex-end;flex:none;min-width:54px;gap:3px}
  .device-row{position:relative;z-index:3;flex:1;display:grid;grid-template-columns:minmax(0,1fr);align-items:center;gap:9px;min-width:0}
  .device-trigger{min-width:0;width:100%;height:38px;display:grid;grid-template-columns:minmax(0,1fr) 18px;align-items:center;gap:8px;padding:0 10px 0 12px;overflow:hidden;border:1px solid rgba(255,255,255,.14);border-radius:9px;color:rgba(255,255,255,.88);background:rgba(255,255,255,.055);font:600 14px/1 var(--app-font, system-ui);text-align:left;cursor:pointer;transition:border-color 140ms ease,background 140ms ease}
  .device-trigger>span,.device-menu button>span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .device-trigger:hover:not(:disabled),.device-trigger.open{border-color:rgba(105,170,255,.32);background:rgba(255,255,255,.1)}
  .device-trigger:disabled{opacity:.48;cursor:default}
  .device-trigger:focus-visible,.device-menu button:focus-visible{outline:2px solid rgba(255,255,255,.9);outline-offset:2px}
  .device-trigger :global(svg){justify-self:end;flex:none;width:16px;height:16px;color:rgba(255,255,255,.58)}
  .device-trigger.open :global(.device-chevron){transform:rotate(180deg)}
  .device-trigger :global(.device-chevron){transition:transform 180ms cubic-bezier(.23,1,.32,1)}
  .device-trigger :global(.device-spinner){animation:device-spin 700ms linear infinite}
  .device-menu{position:absolute;bottom:44px;right:0;z-index:8;width:100%;max-height:180px;padding:6px;overflow-x:hidden;overflow-y:auto;border:1px solid rgba(255,255,255,.12);border-radius:10px;background:#171719;box-shadow:0 10px 24px rgba(0,0,0,.5);animation:device-menu-in 150ms cubic-bezier(.23,1,.32,1)}
  .device-menu button{width:100%;min-height:36px;display:grid;grid-template-columns:minmax(0,1fr) 18px;align-items:center;gap:8px;padding:6px 10px;border:0;border-radius:7px;color:rgba(255,255,255,.74);background:transparent;font:600 14px/1.2 var(--app-font, system-ui);text-align:left;cursor:pointer}
  .device-menu button:hover,.device-menu button:focus-visible{color:#fff;background:rgba(255,255,255,.08)}
  .device-menu button.active{color:#fff;background:rgba(65,145,235,.14)}
  .device-menu button :global(svg){justify-self:end;color:#91caff}
  .device-menu::-webkit-scrollbar{width:4px}.device-menu::-webkit-scrollbar-thumb{border-radius:999px;background:rgba(255,255,255,.2)}
  .volume-label{color:rgba(222,231,248,.68);font:500 16px/1 var(--app-font, system-ui);white-space:nowrap}
  .volume-readout strong{display:flex;align-items:baseline;color:#eaf4ff;font:700 32px/.9 var(--app-font, system-ui);letter-spacing:-.035em;font-variant-numeric:tabular-nums}
  @keyframes device-menu-in{from{opacity:.45;transform:translateY(-4px) scale(.985)}to{opacity:1;transform:translateY(0) scale(1)}}
  @keyframes device-spin{to{transform:rotate(360deg)}}
  @media (prefers-reduced-motion:reduce){.volume-track,.device-trigger{transition:none}.device-trigger :global(.device-chevron){transition:none}.device-menu,.device-trigger :global(.device-spinner){animation:none}}
  .panel-volume.compact{gap:8px}
  .panel-volume.compact .volume-readout strong{font-size:28px}
  .panel-volume.compact .device-row{min-width:0;width:100%}
  .panel-volume.compact .device-trigger{max-width:100%;width:100%;box-sizing:border-box;font-size:12px}
  .panel-volume.compact .device-menu{max-height:180px;overflow-y:auto;min-width:0;width:100%;box-sizing:border-box}
</style>
