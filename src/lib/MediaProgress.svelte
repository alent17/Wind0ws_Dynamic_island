<script lang="ts">
  import { formatTime } from "$lib/mediaClock";
  import { locale, translate } from "$lib/i18n";
  import RollingNumber from "$lib/RollingNumber.svelte";

  let { position = 0, duration = 0, light = false, showTimes = true, seekable = false, reduceMotion = false, onSeek } = $props<{
    position?: number;
    duration?: number;
    light?: boolean;
    showTimes?: boolean;
    seekable?: boolean;
    reduceMotion?: boolean;
    onSeek?: (positionMs: number) => void | Promise<void>;
  }>();

  let dragging = $state(false);
  let draftPosition = $state(0);
  let displayedPosition = $derived(dragging ? draftPosition : position);
  let ratio = $derived(duration ? Math.min(1, Math.max(0, displayedPosition / duration)) : 0);

  function preview(event: Event) {
    dragging = true;
    draftPosition = Number((event.currentTarget as HTMLInputElement).value);
  }

  function commit(event: Event) {
    const next = Number((event.currentTarget as HTMLInputElement).value);
    draftPosition = next;
    dragging = false;
    void onSeek?.(next);
  }

  function cancel() { dragging = false; }
</script>

<div class:light class:with-times={showTimes} class="progress-block" aria-label={translate("playbackProgress",{percent:Math.round(ratio*100)},$locale)}>
  {#if showTimes}<span class="time elapsed"><RollingNumber value={formatTime(displayedPosition)} {reduceMotion} /></span>{/if}
  <div class="track"><span style={`width:${ratio * 100}%`}></span>{#if seekable && duration > 0}<input type="range" min="0" max={duration} step="1000" value={displayedPosition} aria-label={translate("seekProgress",{},$locale)} onclick={(event)=>event.stopPropagation()} onpointerdown={(event)=>event.stopPropagation()} onpointercancel={cancel} onblur={cancel} oninput={preview} onchange={commit}/>{/if}</div>
  {#if showTimes}<span class="time remaining">-<RollingNumber value={formatTime(Math.max(0, duration - displayedPosition))} {reduceMotion} /></span>{/if}
</div>

<style>
  .progress-block { width: 100%; color: rgba(255,255,255,.55); font-variant-numeric: tabular-nums; }
  .progress-block.with-times { display: grid; grid-template-columns: max-content minmax(0,1fr) max-content; align-items: center; column-gap: 10px; }
  .time { display:inline-flex; align-items:center; height:1em; font-size:11px; font-weight:600; line-height:1; white-space:nowrap; }
  .track { position:relative; height: 6px; border-radius: 999px; background: rgba(255,255,255,.19); }
  .track span { display:block; height:100%; border-radius:inherit; background:#fff; }
  .track input { position:absolute; inset:-8px 0; width:100%; height:20px; margin:0; opacity:0; cursor:pointer; }
  .track:has(input:focus-visible) { outline:2px solid #fff; outline-offset:3px; }
  .light { color:rgba(17,17,19,.5); }
  .light .track { background:rgba(17,17,19,.12); }
  .light .track span { background:#111113; }
</style>
