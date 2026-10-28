<script lang="ts">
  import { onMount } from "svelte";
  import { Heart } from "lucide-svelte";
  import { listen } from "@tauri-apps/api/event";
  import type { NeteaseLikeState } from "$lib/api/media";
  import { neteaseLikeState, acceptNeteaseLikeState } from "$lib/neteaseLikeState";
  import { locale, translate } from "$lib/i18n";

  let { title, artist, cacheKey = "", live = true, compact = false, available = false } = $props<{ title: string; artist: string; cacheKey?: string; live?: boolean; compact?: boolean; available?: boolean }>();
  const entry = $derived(neteaseLikeState(cacheKey, title, artist));
  const likeState = $derived($entry.value);
  const busy = $derived($entry.busy);
  const failed = $derived($entry.failed);
  const t = (key: "likeSong" | "unlikeSong" | "likeSongFailed" | "likeSongUnavailable") => translate(key, {}, $locale);
  const failureText = $derived([t("likeSongFailed"), $entry.error].filter(Boolean).join("："));

  onMount(() => {
    if (!live) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<NeteaseLikeState>("netease-like-changed", event => acceptNeteaseLikeState(event.payload)).then(stop => {
      if (disposed) stop(); else unlisten = stop;
    }).catch(() => {});

    return () => { disposed = true; unlisten?.(); };
  });
  $effect(() => { if (!live) return; if (available) void entry.load(); else entry.suspend(); });
</script>

<span class="like-wrap">
  <button type="button" class="like-control" class:compact class:liked={available && likeState?.liked === true}
    aria-label={!available || !likeState ? t("likeSongUnavailable") : likeState.liked ? t("unlikeSong") : t("likeSong")}
    aria-pressed={available && likeState ? likeState.liked : undefined} aria-busy={busy}
    title={!available ? t("likeSongUnavailable") : failed ? failureText : !likeState ? t("likeSongUnavailable") : likeState.liked ? t("unlikeSong") : t("likeSong")}
    disabled={!live || !available || !likeState || busy}
    onclick={(event) => { event.stopPropagation(); if (live && available && likeState) void entry.toggle(); }}>
    <Heart size={compact ? 20 : 36} strokeWidth={2} fill={available && likeState?.liked ? "currentColor" : "none"} />
  </button>
</span>

<style>
  .like-wrap{position:relative;display:inline-flex;width:48px;height:48px;flex:none;align-items:center;justify-content:center}
  .like-control{width:48px;height:48px;display:grid;place-items:center;padding:0;border:0;border-radius:9px;background:transparent;color:rgba(232,238,248,.82);cursor:pointer}
  .like-control.compact{border-radius:12px}
  .like-control.liked{color:#ff4058}
  .like-control:hover:not(:disabled){background:rgba(255,255,255,.1)}
  .like-control:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .like-control:disabled{cursor:default}
</style>
