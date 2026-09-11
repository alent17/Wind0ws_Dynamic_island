<script lang="ts">
  import { formatTime } from "$lib/mediaClock";

  let { position = 0, duration = 0, light = false, showTimes = true, seekable = false, onSeek } = $props<{
    position?: number;
    duration?: number;
    light?: boolean;
    showTimes?: boolean;
    seekable?: boolean;
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
</script>

<div class:light class="progress-block" aria-label={`播放进度 ${Math.round(ratio * 100)}%`}>
  <div class="track"><span style={`transform:scaleX(${ratio})`}></span>{#if seekable && duration > 0}<input type="range" min="0" max={duration} value={displayedPosition} aria-label="调整播放进度" onclick={(event)=>event.stopPropagation()} oninput={preview} onchange={commit}/>{/if}</div>
  {#if showTimes}
    <div class="times"><span>{formatTime(displayedPosition)}</span><span>-{formatTime(Math.max(0, duration - displayedPosition))}</span></div>
  {/if}
</div>

<style>
  .progress-block { width: 100%; color: rgba(255,255,255,.55); font-variant-numeric: tabular-nums; }
  .track { position:relative; height: 4px; border-radius: 999px; background: rgba(255,255,255,.19); }
  .track span { display:block; width:100%; height:100%; border-radius:inherit; background:#fff; transform-origin:left center; will-change:transform; }
  .track input { position:absolute; inset:-8px 0; width:100%; height:20px; margin:0; opacity:0; cursor:pointer; }
  .track:has(input:focus-visible) { outline:2px solid #fff; outline-offset:3px; }
  .times { display:flex; justify-content:space-between; margin-top:5px; font-size:9px; line-height:1; }
  .light { color:rgba(17,17,19,.5); }
  .light .track { background:rgba(17,17,19,.12); }
  .light .track span { background:#111113; }
</style>
