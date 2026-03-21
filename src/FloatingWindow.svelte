<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window"; // 引入原生窗口控制 API
  import { fade } from "svelte/transition";
  import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Shuffle,
    Repeat,
    Volume2,
    ExternalLink,
  } from "lucide-svelte";

  let mediaState = $state({
    title: "等待播放...",
    artist: "未知艺术家",
    album_art: "",
    is_playing: false,
    position_ms: 0,
    duration_ms: 0,
  });

  let currentTrackKey = "";
  let displayCover = $state("");

  let unlisten: () => void;
  let progressInterval: ReturnType<typeof setInterval>;

  // ========== 终极拖拽修复 ==========
  function startDrag(e: MouseEvent) {
    // 如果点击的是按钮或者按钮内部的图标，则不触发拖拽
    if ((e.target as HTMLElement).closest("button")) {
      return;
    }
    // 只有鼠标左键按下时才触发拖拽
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  }

  async function fetchHighResCover(
    title: string,
    artist: string,
    fallbackCover: string,
  ) {
    if (!title || title === "等待播放...") return fallbackCover;
    try {
      const query = encodeURIComponent(`${title} ${artist}`);
      const res = await fetch(
        `https://itunes.apple.com/search?term=${query}&limit=1&media=music`,
      );
      const data = await res.json();
      if (data.results && data.results.length > 0) {
        return data.results[0].artworkUrl100.replace(
          "100x100bb.jpg",
          "600x600bb.jpg",
        );
      }
    } catch (error) {
      console.warn("高清图获取失败", error);
    }
    return fallbackCover;
  }

  onMount(async () => {
    unlisten = await listen("media-update", (event: any) => {
      if (event.payload) {
        const payload = event.payload;
        const newTrackKey = `${payload.title}-${payload.artist}`;

        if (newTrackKey !== currentTrackKey) {
          currentTrackKey = newTrackKey;
          mediaState = { ...mediaState, ...payload, album_art: displayCover };

          fetchHighResCover(
            payload.title,
            payload.artist,
            payload.album_art,
          ).then((hdCover) => {
            const img = new Image();
            if (hdCover.startsWith("http")) img.crossOrigin = "Anonymous";
            img.onload = () => {
              if (currentTrackKey === newTrackKey) displayCover = hdCover;
            };
            img.onerror = () => {
              if (currentTrackKey === newTrackKey)
                displayCover = payload.album_art;
            };
            img.src = hdCover;
          });
        } else {
          mediaState.is_playing = payload.is_playing;
          mediaState.position_ms = payload.position_ms;
          mediaState.duration_ms = payload.duration_ms;
        }
      }
    });

    progressInterval = setInterval(() => {
      if (
        mediaState.is_playing &&
        mediaState.duration_ms > 0 &&
        mediaState.position_ms < mediaState.duration_ms
      ) {
        mediaState.position_ms += 1000;
      }
    }, 1000);
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    clearInterval(progressInterval);
  });

  function formatTime(ms: number): string {
    if (!ms || ms <= 0) return "0:00";
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  let progressPercent = $derived(
    mediaState.duration_ms > 0
      ? (mediaState.position_ms / mediaState.duration_ms) * 100
      : 0,
  );

  async function togglePlay(e: MouseEvent) {
    e.stopPropagation();
    await invoke("control_media", { action: "play_pause" });
    mediaState.is_playing = !mediaState.is_playing;
  }
</script>

<main class="spotify-mini-player" onmousedown={startDrag}>
  {#if displayCover}
    {#key displayCover}
      <img
        src={displayCover}
        alt="cover"
        class="bg-image"
        draggable="false"
        transition:fade={{ duration: 450 }}
      />
    {/key}
  {:else}
    <div class="bg-placeholder"></div>
  {/if}

  <div class="bottom-gradient"></div>

  <div class="info-section">
    <div class="title" title={mediaState.title}>{mediaState.title}</div>
    <div class="artist" title={mediaState.artist}>{mediaState.artist}</div>
  </div>

  <div class="hover-overlay">
    <div class="controls-container">
      <div class="buttons-row">
        <button class="icon-btn secondary" title="Volume"
          ><Volume2 size={18} strokeWidth={2} /></button
        >
        <button class="icon-btn secondary" title="Shuffle"
          ><Shuffle size={18} strokeWidth={2} /></button
        >
        <button
          class="icon-btn primary"
          title="Previous"
          onclick={(e) => {
            e.stopPropagation();
            invoke("control_media", { action: "prev" });
          }}><SkipBack size={24} fill="currentColor" /></button
        >

        <button class="play-pause-btn" onclick={togglePlay}>
          {#if mediaState.is_playing}
            <Pause size={24} fill="currentColor" color="black" />
          {:else}
            <Play
              size={24}
              fill="currentColor"
              color="black"
              style="margin-left: 3px;"
            />
          {/if}
        </button>

        <button
          class="icon-btn primary"
          title="Next"
          onclick={(e) => {
            e.stopPropagation();
            invoke("control_media", { action: "next" });
          }}><SkipForward size={24} fill="currentColor" /></button
        >
        <button class="icon-btn secondary" title="Repeat"
          ><Repeat size={18} strokeWidth={2} /></button
        >
        <button class="icon-btn secondary" title="Open in Spotify"
          ><ExternalLink size={18} strokeWidth={2} /></button
        >
      </div>

      <div class="progress-row">
        <span class="time">{formatTime(mediaState.position_ms)}</span>
        <div class="progress-bar-bg">
          <div
            class="progress-bar-fill"
            style="width: {progressPercent}%"
          ></div>
        </div>
        <span class="time">{formatTime(mediaState.duration_ms)}</span>
      </div>
    </div>
  </div>
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent;
    font-family: "Circular Sp", "Helvetica Neue", Arial, sans-serif;
  }

  .spotify-mini-player {
    position: relative;
    container-type: inline-size;
    width: 100vw;
    height: 100vh;
    border-radius: 12px;
    overflow: hidden;
    background-color: #121212;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    user-select: none;
    -webkit-user-select: none;

    /* 让 OS 层面识别这个区域为拖拽区 */
    -webkit-app-region: drag;
    cursor: grab;
  }

  .spotify-mini-player:active {
    cursor: grabbing;
  }

  .bg-image,
  .bg-placeholder {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 1;
    pointer-events: none;
  }
  .bg-placeholder {
    background: linear-gradient(135deg, #333, #121);
  }

  .bottom-gradient {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 50%;
    background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.85) 0%,
      rgba(0, 0, 0, 0) 100%
    );
    z-index: 2;
    pointer-events: none;
  }

  .info-section {
    position: absolute;
    bottom: 16px;
    left: 16px;
    right: 16px;
    z-index: 3;
    pointer-events: none;
    display: flex;
    flex-direction: column;
  }

  .title {
    color: #fff;
    font-weight: 800;
    letter-spacing: -0.5px;
    font-size: clamp(16px, 6.5cqi, 28px);
    margin-bottom: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artist {
    color: rgba(255, 255, 255, 0.7);
    font-weight: 400;
    font-size: clamp(12px, 4cqi, 16px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hover-overlay {
    position: absolute;
    inset: 0;
    z-index: 4;
    background-color: rgba(0, 0, 0, 0.5);
    opacity: 0;
    transition: opacity 0.3s cubic-bezier(0.3, 0, 0.4, 1);
  }
  .spotify-mini-player:hover .hover-overlay {
    opacity: 1;
  }

  .controls-container {
    position: absolute;
    bottom: 72px;
    width: 100%;
    padding: 0 16px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .buttons-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0 4px;
    box-sizing: border-box;
  }

  button {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;

    /* 核心：把按钮排除在拖拽区域外，恢复其点击响应 */
    -webkit-app-region: no-drag;
  }

  .icon-btn.primary {
    color: #fff;
  }
  .icon-btn.secondary {
    color: rgba(255, 255, 255, 0.6);
  }
  .icon-btn:hover {
    color: #fff;
    transform: scale(1.1);
  }
  .icon-btn:active {
    transform: scale(0.95);
  }

  .play-pause-btn {
    width: 48px;
    height: 48px;
    background-color: #fff;
    border-radius: 50%;
    color: #000;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
  .play-pause-btn:hover {
    transform: scale(1.05);
    background-color: #f0f0f0;
  }
  .play-pause-btn:active {
    transform: scale(0.96);
  }

  .progress-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    /* 让进度条区域也不触发拖拽，方便以后做点击跳转进度 */
    -webkit-app-region: no-drag;
  }

  .time {
    font-size: 11px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    min-width: 32px;
  }

  .progress-bar-bg {
    flex: 1;
    height: 4px;
    background-color: rgba(255, 255, 255, 0.25);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background-color: #fff;
    border-radius: 2px;
    transition: width 1s linear;
  }
</style>
