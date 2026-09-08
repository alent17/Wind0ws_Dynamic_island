<script lang="ts">
  import { spring } from "svelte/motion";
  import { untrack } from "svelte";
  import { Music2, AudioLines, PanelsTopLeft } from "lucide-svelte";
  import MediaProgress from "$lib/MediaProgress.svelte";
  import PlayerControls from "$lib/PlayerControls.svelte";
  import type { MediaState } from "$lib/api/types";

  export type IslandMode = "compact" | "hover" | "expanded" | "hidden";
  let { media, mode = "compact", position = 0, interactive = true, liveControls = true, onToggle, onFloating, onPreviewPlayPause } = $props<{ media: MediaState; mode?: IslandMode; position?: number; interactive?: boolean; liveControls?: boolean; onToggle?: () => void; onFloating?: () => void; onPreviewPlayPause?: () => void; }>();
  const geometryFor = (value: IslandMode) => value === "expanded" ? { width:371,height:156,radius:44 } : value === "hover" ? { width:136,height:39,radius:20 } : value === "hidden" ? { width:96,height:8,radius:6 } : { width:126,height:37,radius:19 };
  const initialGeometry = untrack(() => geometryFor(mode));
  const geometry = spring(initialGeometry, { stiffness: .24, damping: .78, precision: .05 });
  let size = $state(initialGeometry);
  const unsubscribe = geometry.subscribe((value) => size = value);
  $effect(() => {
    geometry.set(geometryFor(mode));
  });
  $effect(() => () => unsubscribe());
  function activate(event: KeyboardEvent | MouseEvent) {
    if (!interactive || event.target !== event.currentTarget || (event instanceof KeyboardEvent && event.key !== "Enter" && event.key !== " ")) return;
    event.preventDefault(); onToggle?.();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
<section class="island" class:expanded={mode === "expanded"} class:hidden={mode === "hidden"} style={`width:${size.width}px;height:${size.height}px;border-radius:${size.radius}px`} role="button" tabindex={interactive ? 0 : -1} aria-label={mode === "expanded" ? "收起灵动岛" : "展开灵动岛"} onclick={activate} onkeydown={activate}>
  {#if mode === "expanded"}
    <div class="expanded-layout">
      <div class="now-playing">
        <div class="art large-art">{#if media.albumArt}<img src={media.albumArt} alt="" />{:else}<Music2 size={23} />{/if}</div>
        <div class="metadata"><strong title={media.title}>{media.title || "未在播放"}</strong><span title={media.artist}>{media.artist || "打开音乐播放器开始"}</span></div>
        <span class="activity"><AudioLines size={22} aria-label={media.isPlaying ? "正在播放" : "已暂停"} /></span>
      </div>
      <MediaProgress {position} duration={media.durationMs} seekable={liveControls && media.capabilities?.seek} />
      <div class="footer">
        <span class="source">{media.sourceDisplay || "Isle"}</span>
        <PlayerControls playing={media.isPlaying} capabilities={media.capabilities} shuffleActive={media.shuffleActive} repeatMode={media.repeatMode} live={liveControls} onPlayPause={onPreviewPlayPause} />
        <button class="floating" aria-label="打开悬浮播放器" onclick={(event) => { event.stopPropagation(); onFloating?.(); }}><PanelsTopLeft size={16}/></button>
      </div>
    </div>
  {:else if mode !== "hidden"}
    <div class="compact-layout">
      <div class="art">{#if media.albumArt}<img src={media.albumArt} alt="" />{:else}<Music2 size={14} />{/if}</div>
      <div class="mini-bars" class:paused={!media.isPlaying} aria-hidden="true">{#each [12,20,15,24,17] as height, i}<i style={`--h:${height}px;--i:${i}`}></i>{/each}</div>
    </div>
  {/if}
</section>

<style>
  .island{position:relative;box-sizing:border-box;overflow:hidden;flex:none;color:#fff;background:#000;box-shadow:0 10px 30px rgba(0,0,0,.18);cursor:pointer;outline:none;transform:translateZ(0);will-change:width,height,border-radius}.island:active{filter:brightness(.88)}.island:focus-visible{outline:2px solid #77e591;outline-offset:3px}
  .compact-layout{height:100%;display:flex;align-items:center;justify-content:space-between;padding:4px 9px 4px 4px}.art{width:29px;height:29px;display:grid;place-items:center;flex:none;overflow:hidden;border-radius:50%;color:rgba(255,255,255,.68);background:linear-gradient(145deg,#303136,#111)}.art img{width:100%;height:100%;object-fit:cover}.large-art{width:48px;height:48px;border-radius:13px}
  .mini-bars{height:22px;display:flex;align-items:center;gap:2px;color:#72df8b}.mini-bars i{width:3px;height:var(--h);max-height:20px;border-radius:3px;background:currentColor;transform:scaleY(.45);animation:pulse .82s calc(var(--i)*-110ms) ease-in-out infinite alternate;transform-origin:center}.mini-bars.paused i{animation-play-state:paused;transform:scaleY(.25);opacity:.55}
  .expanded-layout{width:371px;height:156px;box-sizing:border-box;padding:15px 17px 13px;display:flex;flex-direction:column;gap:10px;cursor:default}.now-playing{display:grid;grid-template-columns:48px minmax(0,1fr) 28px;align-items:center;gap:11px}.metadata{min-width:0;display:flex;flex-direction:column;gap:4px}.metadata strong,.metadata span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.metadata strong{font-size:14px;line-height:1.1;letter-spacing:-.015em}.metadata span{color:rgba(255,255,255,.56);font-size:11px}.activity{color:#72df8b}
  .footer{display:grid;grid-template-columns:1fr auto 1fr;align-items:center;margin-top:-5px}.source{color:rgba(255,255,255,.42);font-size:9px;letter-spacing:.04em}.floating{justify-self:end;width:30px;height:30px;display:grid;place-items:center;border:0;border-radius:50%;color:#fff;background:rgba(255,255,255,.1);cursor:pointer}.floating:active{transform:scale(.9)}@keyframes pulse{to{transform:scaleY(1)}}@media(prefers-reduced-motion:reduce){.island{transition:opacity 160ms ease;will-change:auto}.mini-bars i{animation:none}}
</style>
