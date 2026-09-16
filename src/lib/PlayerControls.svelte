<script lang="ts">
  import { SkipBack, Pause, Play, SkipForward, Shuffle, Repeat, Repeat1 } from "lucide-svelte";
  import { mediaApi } from "$lib/api/media";
  import type { MediaCapabilities } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let { playing = false, capabilities, large = false, live = true, shuffleActive = false, repeatMode = "none", onPlayPause } = $props<{
    playing?: boolean;
    capabilities?: MediaCapabilities;
    large?: boolean;
    live?: boolean;
    shuffleActive?: boolean;
    repeatMode?: "none" | "track" | "list";
    onPlayPause?: () => void;
  }>();
  const enabled = (key: keyof MediaCapabilities) => capabilities?.[key] !== false;
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  const act = (action: "prev" | "play_pause" | "next") => {
    if (action === "play_pause" && onPlayPause) onPlayPause();
    if (live) mediaApi.controlMedia(action).catch(() => {});
  };
</script>

<div class:large class="controls" aria-label={t("playbackControls")}>
  {#if capabilities?.shuffle}<button class:active={shuffleActive} aria-label={shuffleActive ? t("shuffleOff") : t("shuffleOn")} onclick={(event)=>{event.stopPropagation();if(live)mediaApi.toggleShuffle().catch(()=>{})}}><Shuffle size={large ? 18 : 15}/></button>{/if}
  <button disabled={!enabled("previous")} aria-label={t("previous")} onclick={(event) => { event.stopPropagation(); act("prev"); }}><SkipBack size={large ? 21 : 17} fill="currentColor" /></button>
  <button class="primary" disabled={!enabled("playPause")} aria-label={playing ? t("pause") : t("play")} onclick={(event) => { event.stopPropagation(); act("play_pause"); }}>
    {#if playing}<Pause size={large ? 25 : 20} fill="currentColor" />{:else}<Play size={large ? 25 : 20} fill="currentColor" />{/if}
  </button>
  <button disabled={!enabled("next")} aria-label={t("next")} onclick={(event) => { event.stopPropagation(); act("next"); }}><SkipForward size={large ? 21 : 17} fill="currentColor" /></button>
  {#if capabilities?.repeat}<button class:active={repeatMode !== "none"} aria-label={t("repeatMode",{mode:repeatMode})} onclick={(event)=>{event.stopPropagation();if(live)mediaApi.cycleRepeat().catch(()=>{})}}>{#if repeatMode === "track"}<Repeat1 size={large ? 18 : 15}/>{:else}<Repeat size={large ? 18 : 15}/>{/if}</button>{/if}
</div>

<style>
  .controls { display:flex; align-items:center; justify-content:center; gap:18px; }
  button { width:32px; height:32px; display:grid; place-items:center; border:0; border-radius:50%; color:#fff; background:transparent; cursor:pointer; transition:transform 100ms ease-out, background 160ms ease; }
  button:active:not(:disabled) { transform:scale(.88); }
  button:focus-visible { outline:2px solid #fff; outline-offset:2px; }
  button:disabled { opacity:.25; cursor:default; }
  button.active { color:#72df8b; }
  .primary { width:36px; height:36px; background:#fff; color:#050505; }
  .large { gap:24px; }
  .large button { width:40px; height:40px; }
  .large .primary { width:48px; height:48px; }
</style>
