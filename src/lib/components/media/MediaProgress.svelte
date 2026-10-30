<script lang="ts">
  import { formatTime } from "$lib/mediaClock";
  import { locale, translate } from "$lib/i18n";

  let { position = 0, duration = 0, light = false, showTimes = true, seekable = false, seekBusy = false, progressKnown = true, estimated = false, seekError = "", onSeek } = $props<{
    position?: number;
    duration?: number;
    light?: boolean;
    showTimes?: boolean;
    seekable?: boolean;
    seekBusy?: boolean;
    progressKnown?: boolean;
    estimated?: boolean;
    seekError?: string;
    onSeek?: (positionMs: number) => void | Promise<void>;
  }>();

  let dragging = $state(false);
  let draftPosition = $state(0);
  let timelineKnown = $derived(progressKnown && Number.isFinite(duration) && duration > 0);
  let displayedPosition = $derived(timelineKnown ? dragging ? draftPosition : position : 0);
  let ratio = $derived(timelineKnown ? Math.min(1, Math.max(0, displayedPosition / duration)) : 0);

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

<div title={estimated ? translate("neteaseProgressEstimate", {}, $locale) : undefined} class:light class:with-times={showTimes} class="progress-block" style={`--time-width:${Math.max(5, formatTime(duration).length + 1 + (estimated ? 2 : 0))}ch`} aria-label={translate("playbackProgress",{percent:Math.round(ratio*100)},$locale)}>
  {#if showTimes}<span class="time elapsed">{timelineKnown ? `${estimated ? "≈ " : ""}${formatTime(displayedPosition)}` : "--:--"}</span>{/if}
  <div class="track-controls">
    <div class="track"><span style={`width:${ratio * 100}%`}></span>{#if seekable && timelineKnown}<input type="range" min="0" max={duration} step="1000" value={displayedPosition} aria-label={translate("seekProgress",{},$locale)} onclick={(event)=>event.stopPropagation()} onpointerdown={(event)=>event.stopPropagation()} onpointercancel={cancel} onblur={cancel} oninput={preview} onchange={commit}/>{/if}</div>
  </div>
  {#if showTimes}<span class="time remaining">{timelineKnown ? `-${formatTime(Math.max(0, duration - displayedPosition))}` : "--:--"}</span>{/if}
  {#if seekError}<p class="seek-error" role="alert">{seekError}</p>{/if}
</div>

<style>
  .progress-block { width: 100%; color: #fff; font-variant-numeric: tabular-nums; }
  .progress-block.with-times { display: grid; grid-template-columns: var(--time-width) minmax(0,1fr) var(--time-width); align-items: center; column-gap: 10px; }
  .time { display:inline-flex; align-items:center; height:1em; font-size:13px; font-weight:600; line-height:1; white-space:nowrap; }
  .remaining { justify-content: flex-end; }
  .track-controls{display:flex;align-items:center;min-width:0;gap:6px}
  .track { position:relative;flex:1;min-width:0;height: 6px; border-radius: 999px; background: rgba(255,255,255,.19); }
  .track span { display:block; height:100%; border-radius:inherit; background:#fff; }
  .track input { position:absolute; inset:-8px 0; width:100%; height:20px; margin:0; opacity:0; cursor:pointer; }
  .track:has(input:focus-visible) { outline:2px solid #fff; outline-offset:3px; }
  .light { color:rgba(17,17,19,.5); }
  .light .track { background:rgba(17,17,19,.12); }
  .light .track span { background:#111113; }
  .seek-error { grid-column: 1 / -1; margin: 5px 0 0; color: #ffb4ab; font-size: 13px; line-height: 1.3; overflow-wrap: anywhere; }
  .light .seek-error { color: #b3261e; }
</style>
