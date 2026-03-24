<script lang="ts">
  import { onMount } from "svelte";
  import type { Window } from "@tauri-apps/api/window";
  import { getCurrentWindow } from "@tauri-apps/api/window"; // Tauri Window API，用于处理无边框窗口的拖拽和关闭

  let appWindow: Window | undefined;
  let isHovered = false; // 用于控制悬停 overlay 的显示

  // 模拟当前播放数据，界面 identical 需要
  const song = {
    title: "スキズキ",
    artist: "sucola",
    // 这里的专辑图片路径需要替换为你项目中的实际路径
    album_art_url: "/path/to/album_art.png",
  };

  // 可以在这里通过 invoke 获取后端数据
  /*
  invoke("get_current_song_data").then(data => {
    song = data;
  });
  */

  onMount(async () => {
    try {
      // 获取当前 PIP 窗口的 Tauri Window 对象
      appWindow = getCurrentWindow();
    } catch (e) {
      console.warn("Tauri Window API not available (not running in Tauri)", e);
    }
  });

  // 处理窗口关闭
  function closeWindow() {
    if (appWindow) {
      appWindow.close();
    }
  }

  // 播放器控件模拟函数
  function playPause() {
    console.log("Play/Pause");
  }
  function nextTrack() {
    console.log("Next Track");
  }
  function prevTrack() {
    console.log("Prev Track");
  }

  // 键盘事件处理
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      playPause();
    }
  }
</script>

<main
  class="pip-player"
  on:mouseenter={() => (isHovered = true)}
  // 鼠标移入，显示悬停 overlay
  on:mouseleave={() => (isHovered = false)}
  on:keydown={handleKeydown}
  tabindex="0"
  role="application"
  aria-label="迷你播放器"
>
  <header class="pip-titlebar" data-tauri-drag-region>
    <div class="left-section">
      <span class="pip-description">正在播放 - Mini Player</span>
    </div>
    <div class="right-section no-drag">
      <button
        class="pip-control-btn close-btn"
        on:click={closeWindow}
        aria-label="关闭迷你播放器"
      >
        <svg viewBox="0 0 24 24"
          ><path
            fill="currentColor"
            d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
          /></svg
        >
      </button>
    </div>
  </header>

  <img src={song.album_art_url} alt="Album Art" class="album-art" />

  <div class="pip-metadata-normal" class:hidden={isHovered}>
    <h1 class="pip-song-title">{song.title}</h1>
    <p class="pip-artist-name">{song.artist}</p>
  </div>

  <div
    class="pip-overlay"
    class:visible={isHovered}
    on:click|stopPropagation
    role="region"
    aria-label="播放器控制"
  >
    <div class="pip-controls-container">
      <div class="pip-controls no-drag">
        <button class="pip-btn volume-btn" aria-label="音量设置"
          ><svg viewBox="0 0 24 24"
            ><path
              fill="currentColor"
              d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"
            /></svg
          ></button
        >
        <button class="pip-btn shuffle-btn" aria-label="随机播放"
          ><svg viewBox="0 0 24 24"
            ><path
              fill="currentColor"
              d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.45 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"
            /></svg
          ></button
        >
        <button
          class="pip-btn prev-btn"
          on:click={prevTrack}
          aria-label="上一首"
          ><svg viewBox="0 0 24 24"
            ><path fill="currentColor" d="M6 6h2v12H6zm3.5 6l8.5 6V6z" /></svg
          ></button
        >
        <button
          class="pip-btn play-pause-btn"
          on:click={playPause}
          aria-label="播放或暂停"
          ><svg viewBox="0 0 24 24"
            ><path
              fill="currentColor"
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9V8h2v8zm4 0h-2V8h2v8z"
            /></svg
          ></button
        >
        <button
          class="pip-btn next-btn"
          on:click={nextTrack}
          aria-label="下一首"
          ><svg viewBox="0 0 24 24"
            ><path fill="currentColor" d="M16.5 12l-8.5 6V6z" /></svg
          ></button
        >
        <button class="pip-btn repeat-btn" aria-label="循环播放"
          ><svg viewBox="0 0 24 24"
            ><path
              fill="currentColor"
              d="M7 7h10v3l4-4-4-4v3H5v6h2V7zm10 10H7v-3l-4 4 4 4v-3h12v-6h-2v4z"
            /></svg
          ></button
        >
      </div>

      <div class="pip-progress no-drag">
        <span class="current-time">0:34</span>
        <div class="pip-progress-bar-bg">
          <div class="pip-progress-bar-fill" style="width: 18%"></div>
        </div>
        <span class="total-time">3:15</span>
      </div>

      <div class="pip-metadata-hover">
        <h1 class="pip-song-title-hover">{song.title}</h1>
        <p class="pip-artist-name-hover">{song.artist}</p>
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
    background-color: transparent; /* 重要：允许 Svelte 内容透明 */
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji",
      "Segoe UI Symbol"; /* 尝试匹配截图 */
    color: white;
  }

  /* data-tauri-drag-region 的通用样式，确保可拖拽 */
  [data-tauri-drag-region] {
    cursor: grab;
  }
  .no-drag {
    -webkit-app-region: no-drag; /* 核心：在 drag region 中定义不可拖拽区域 */
    cursor: auto;
  }

  .pip-player {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    /*Turn 2截图看起来比较小，可能是在桌面上的实际大小 300x300 */
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }

  /* 自定义顶部描述栏。完全 replica Turn 2。 */
  .pip-titlebar {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 24px; /* 截图看起来很窄，匹配截图 */
    background-color: #2e2e2e; /* 尝试匹配截图 Turn 2 的深色顶部栏 */
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 8px;
    box-sizing: border-box;
    z-index: 100; /* 在所有内容之上 */
    border-top-left-radius: 4px; /* 截图似乎有一点点圆角 */
    border-top-right-radius: 4px;
    font-size: 11px; /* 顶部描述文字很小 */
    color: rgba(255, 255, 255, 0.7); /* 文字颜色较浅 */
  }
  .pip-control-btn {
    background: none;
    border: none;
    color: inherit;
    padding: 0;
    cursor: pointer;
    width: 14px; /* 图标很小 */
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .pip-control-btn svg {
    width: 100%;
    height: 100%;
  }
  .pip-control-btn.close-btn:hover {
    color: white;
  }

  /* 专辑图片背景 */
  .album-art {
    position: absolute;
    top: 24px; /* 避开顶部栏 */
    left: 0;
    width: 100%;
    height: calc(100% - 24px); /* 占据剩余空间 */
    object-fit: cover;
    z-index: 1; /* 内容底层 */
  }

  /* 正常状态下的底部信息。完全 replica Turn 2。 */
  .pip-metadata-normal {
    position: absolute;
    bottom: 12px;
    left: 12px;
    z-index: 10;
    text-align: left;
    transition:
      opacity 0.2s ease,
      visibility 0.2s ease;
  }
  .pip-song-title {
    margin: 0;
    font-size: 16px; /* 匹配截图大标题 */
    font-weight: 700;
  }
  .pip-artist-name {
    margin: 2px 0 0 0;
    font-size: 12px; /* 匹配截图小歌手名 */
    color: rgba(255, 255, 255, 0.7);
  }
  .pip-metadata-normal.hidden {
    opacity: 0;
    visibility: hidden;
  }

  /* 悬停时的Overlay。完全 replica Turn 2 样式和排版。 */
  .pip-overlay {
    position: absolute;
    top: 24px; /* 避开顶部栏 */
    left: 0;
    width: 100%;
    height: calc(100% - 24px); /* 避开顶部栏 */
    background-color: rgba(
      0,
      0,
      0,
      0.65
    ); /* 截图看起来是较深的半透明黑色遮罩 */
    z-index: 50;
    display: flex;
    align-items: flex-end; /* 内容靠下对其 */
    justify-content: center;
    padding: 12px;
    box-sizing: border-box;
    opacity: 0;
    visibility: hidden;
    transition:
      opacity 0.2s ease,
      visibility 0.2s ease;
  }
  .pip-overlay.visible {
    opacity: 1;
    visibility: visible;
  }

  .pip-controls-container {
    width: 100%;
    text-align: center;
  }

  /* 播放器控件排版。完全 replica Turn 2。 */
  .pip-controls {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 10px; /* 图标间距 */
    margin-bottom: 10px;
    -webkit-app-region: no-drag; /* 控件本身不可拖拽 */
  }
  .pip-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.8); /* 正常颜色较深 */
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px; /* 图标尺寸较小，匹配截图 */
    height: 16px;
    transition: color 0.1s ease;
  }
  .pip-btn svg {
    width: 100%;
    height: 100%;
  }
  .pip-btn:hover {
    color: white; /* 悬停颜色变白 */
  }
  .pip-btn.play-pause-btn {
    width: 20px; /* 播放按钮比其他的大一点，匹配截图 */
    height: 20px;
    color: white; /* 正常颜色就是白色的，匹配截图 */
  }

  /* 进度条排版。完全 replica Turn 2。 */
  .pip-progress {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 8px; /* 文字和条的间距 */
    font-size: 10px; /* 进度时间文字非常小 */
    color: rgba(255, 255, 255, 0.6);
    margin-bottom: 10px;
    width: 90%; /* 截图看起来进度条占中间大部分区域 */
    margin-left: auto;
    margin-right: auto;
  }
  .pip-progress-bar-bg {
    flex: 1; /* 条占据剩余空间 */
    height: 3px; /* 进度条很细 */
    background-color: rgba(255, 255, 255, 0.2); /* 进度条背景颜色 */
    border-radius: 1.5px;
    position: relative;
    overflow: hidden;
  }
  .pip-progress-bar-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background-color: white; /* 进度条填充颜色 */
    border-radius: 1.5px;
    /* 进度由 style="width: 18%" 控制 */
  }

  /* 悬停时的歌曲信息。完全 replica Turn 2 排版。 */
  .pip-metadata-hover {
    text-align: left; /* 歌曲信息在左侧，匹配截图 */
    padding-left: 2px; /* 增加截图所见的左侧间距 */
  }
  .pip-song-title-hover {
    margin: 0;
    font-size: 16px; /* 匹配截图大标题 */
    font-weight: 700;
  }
  .pip-artist-name-hover {
    margin: 2px 0 0 0;
    font-size: 12px; /* 匹配截图小歌手名 */
    color: rgba(255, 255, 255, 0.7);
  }
</style>
