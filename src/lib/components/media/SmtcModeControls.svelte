<script lang="ts">
  import { onDestroy } from "svelte";
  import { Repeat, Repeat1, Shuffle } from "lucide-svelte";
  import { mediaApi } from "$lib/api/media";
  import type { MediaState } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  let { media, compact = false, live = true } = $props<{ media: MediaState; compact?: boolean; live?: boolean }>();
  let busy = $state(false);
  let error = $state("");
  let disposed = false;
  onDestroy(() => { disposed = true; });
  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  async function changeMode(kind: "shuffle" | "repeat") {
    if (!live || busy) return;
    busy = true;
    error = "";
    try {
      if (kind === "shuffle") await mediaApi.toggleShuffle();
      else await mediaApi.cycleRepeat();
    } catch {
      if (!disposed) error = t("mediaModeFailed");
    } finally {
      if (!disposed) busy = false;
    }
  }
</script>

<span class="smtc-modes" class:compact>
  {#if media.capabilities?.shuffle}
    <button type="button" class:active={media.shuffleActive} disabled={!live || busy || media.shuffleActive === undefined} aria-pressed={media.shuffleActive ?? false} aria-label={t(media.shuffleActive ? "shuffleOff" : "shuffleOn")} onclick={(event) => { event.stopPropagation(); void changeMode("shuffle"); }}><Shuffle size={compact ? 16 : 34}/></button>
  {/if}
  {#if media.capabilities?.repeat}
    <button type="button" class:active={media.repeatMode !== undefined && media.repeatMode !== "none"} disabled={!live || busy || media.repeatMode === undefined} aria-label={t("repeatMode", { mode: t(media.repeatMode === "track" ? "modeRepeatOne" : media.repeatMode === "list" ? "modeRepeatList" : "modeSequential") })} onclick={(event) => { event.stopPropagation(); void changeMode("repeat"); }}>
      {#if media.repeatMode === "track"}<Repeat1 size={compact ? 16 : 34}/>{:else}<Repeat size={compact ? 16 : 34}/>{/if}
    </button>
  {/if}
  {#if error}<span class="mode-error" role="alert">{error}</span>{/if}
</span>

<style>
  .smtc-modes{position:relative;display:inline-flex;align-items:center;gap:8px;flex:none}
  button{width:48px;height:48px;display:grid;place-items:center;padding:0;border:0;border-radius:9px;background:transparent;color:rgba(255,255,255,.6);cursor:pointer}
  button.active{color:#fff;background:rgba(255,255,255,.08)}
  button:hover:not(:disabled){color:#fff;background:rgba(255,255,255,.1)}
  button:disabled{opacity:.4;cursor:default}
  button:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .compact button{width:28px;height:28px}
  .mode-error{position:absolute;top:100%;left:0;width:max-content;max-width:220px;padding:4px 7px;border-radius:6px;background:#201012;color:#ffb4ab;font-size:11px;z-index:5}
</style>
