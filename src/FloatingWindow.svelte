<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fade } from "svelte/transition";
  import { Play, Pause, SkipBack, SkipForward, X } from "lucide-svelte";

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
  let previousCover = $state("");
  let nextCover = $state("");
  let isHovered = $state(false);
  let isFlipping = $state(false);
  let flipKey = $state(0);
  let bgColor = $state("rgb(40, 50, 60)");
  let bgGradient = $state(
    "radial-gradient(circle at 50% 50%, rgb(40, 50, 60), rgb(30, 40, 50))",
  );
  let windowSize = $state({ width: 0, height: 0 });

  let unlisten: () => void;
  let unlistenResize: () => void;
  let progressInterval: ReturnType<typeof setInterval>;
  let savePositionTimeout: ReturnType<typeof setTimeout>;

  async function fetchHighResCover(
    title: string,
    artist: string,
    fallbackCover: string,
  ) {
    if (!title || title === "等待播放...") return fallbackCover;

    // 如果是本地高清图（data:image 且较大），直接使用
    if (fallbackCover && fallbackCover.startsWith("data:image")) {
      // 检查 data:image 的大小
      try {
        const sizeInBytes = Math.round((fallbackCover.length * 3) / 4);
        if (sizeInBytes > 50000) {
          // 大于 50KB 认为是高清图
          return fallbackCover;
        }
      } catch (e) {
        // 忽略错误，继续获取网络高清图
      }
    }

    // 创建带超时的 fetch
    const fetchWithTimeout = async (
      url: string,
      timeout = 5000,
      options: RequestInit = {},
    ) => {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      try {
        const res = await fetch(url, {
          ...options,
          signal: controller.signal,
        });
        clearTimeout(timeoutId);
        return res;
      } catch (error) {
        clearTimeout(timeoutId);
        throw error;
      }
    };

    // 尝试多个图片来源
    const sources = [
      // 1. iTunes (最可靠)
      async () => {
        const query = encodeURIComponent(`${title} ${artist}`);
        const res = await fetchWithTimeout(
          `https://itunes.apple.com/search?term=${query}&limit=1&media=music`,
          8000,
        );
        const data = await res.json();
        if (data.results && data.results.length > 0) {
          return data.results[0].artworkUrl100.replace(
            "100x100bb.jpg",
            "600x600bb.jpg",
          );
        }
        return null;
      },
      // 2. Spotify (通过 open.spotify.com)
      async () => {
        const query = encodeURIComponent(`${artist} ${title}`);
        const res = await fetchWithTimeout(
          `https://open.spotify.com/search/${query}`,
          8000,
        );
        const html = await res.text();
        // 尝试从 HTML 中提取 Spotify 图片
        const match = html.match(/"images":\[{"url":"([^"]+)"}/);
        if (match && match[1]) {
          return match[1];
        }
        // 尝试 og:image
        const ogMatch = html.match(
          /<meta property="og:image" content="([^"]+)"/,
        );
        if (ogMatch && ogMatch[1]) {
          return ogMatch[1];
        }
        return null;
      },
      // 3. Apple Music (备用)
      async () => {
        const query = encodeURIComponent(`${title} ${artist}`);
        const res = await fetchWithTimeout(
          `https://music.apple.com/search?term=${query}`,
          8000,
        );
        const html = await res.text();
        // 尝试从 HTML 中提取图片 URL
        const match = html.match(/"artworkUrl100":"([^"]+)"/);
        if (match && match[1]) {
          return match[1].replace("100x100bb.jpg", "600x600bb.jpg");
        }
        return null;
      },
      // 3. Apple Music (备用)
      async () => {
        const query = encodeURIComponent(`${title} ${artist}`);
        const res = await fetchWithTimeout(
          `https://music.apple.com/search?term=${query}`,
          8000,
        );
        const html = await res.text();
        // 尝试从 HTML 中提取图片 URL
        const match = html.match(/"artworkUrl100":"([^"]+)"/);
        if (match && match[1]) {
          return match[1].replace("100x100bb.jpg", "600x600bb.jpg");
        }
        return null;
      },
      // 4. Last.fm (无需 API key 的备用方法)
      async () => {
        const artistQuery = encodeURIComponent(artist);
        const trackQuery = encodeURIComponent(title);
        const res = await fetchWithTimeout(
          `https://www.last.fm/music/${artistQuery}/_/${trackQuery}`,
          8000,
        );
        const html = await res.text();
        // 尝试从页面提取专辑封面
        const match = html.match(/<meta property="og:image" content="([^"]+)"/);
        if (match && match[1]) {
          return match[1];
        }
        return null;
      },
      // 5. MusicBrainz (开源音乐数据库)
      async () => {
        const query = encodeURIComponent(`artist:${artist} recording:${title}`);
        const res = await fetchWithTimeout(
          `https://musicbrainz.org/ws/2/recording/?query=${query}&fmt=json&limit=1`,
          8000,
        );
        const data = await res.json();
        if (data.recordings && data.recordings.length > 0) {
          const recording = data.recordings[0];
          if ((releases = recording.releases && releases.length > 0)) {
            const release = releases[0];
            if (
              release["cover-art-archive"] &&
              release["cover-art-archive"].count > 0
            ) {
              return `https://coverartarchive.org/release/${release.id}/front`;
            }
          }
        }
        return null;
      },
    ];

    // 依次尝试每个来源
    for (const source of sources) {
      try {
        const result = await source();
        if (result) {
          console.log(
            `✅ 高清图获取成功，来源：${sources.indexOf(source) + 1}`,
          );
          return result;
        }
      } catch (error: any) {
        if (error.name === "AbortError") {
          console.warn(`⏱️ 图片来源 ${sources.indexOf(source) + 1} 超时`);
        } else {
          console.warn(
            `❌ 图片来源 ${sources.indexOf(source) + 1} 失败:`,
            error.message,
          );
        }
      }
    }

    console.log("⚠️ 所有图片来源都失败，使用备用图片");
    return fallbackCover;
  }

  function extractColors(imgSrc: string) {
    const img = new Image();
    if (imgSrc.startsWith("http")) img.crossOrigin = "Anonymous";
    img.onload = () => {
      try {
        const canvas = document.createElement("canvas");
        const size = 80;
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return;
        ctx.drawImage(img, 0, 0, size, size);
        const data = ctx.getImageData(0, 0, size, size).data;

        const buckets: Record<
          string,
          { r: number; g: number; b: number; count: number }
        > = {};

        for (let i = 0; i < data.length; i += 4) {
          const r = data[i],
            g = data[i + 1],
            b = data[i + 2];
          const lum = r * 0.299 + g * 0.587 + b * 0.114;
          if (lum < 30 || lum > 235) continue;

          const qr = Math.round(r / 12) * 12;
          const qg = Math.round(g / 12) * 12;
          const qb = Math.round(b / 12) * 12;
          const key = `${qr},${qg},${qb}`;
          if (!buckets[key]) buckets[key] = { r: qr, g: qg, b: qb, count: 0 };
          buckets[key].count++;
        }

        function toSaturation(r: number, g: number, b: number): number {
          const rn = r / 255,
            gn = g / 255,
            bn = b / 255;
          const max = Math.max(rn, gn, bn);
          const min = Math.min(rn, gn, bn);
          const l = (max + min) / 2;
          if (max === min) return 0;
          return l > 0.5
            ? (max - min) / (2 - max - min)
            : (max - min) / (max + min);
        }

        const sorted = Object.values(buckets)
          .filter((c) => c.count >= 2)
          .sort((a, b) => {
            const satA = toSaturation(a.r, a.g, a.b);
            const satB = toSaturation(b.r, b.g, b.b);
            const scoreA = satA * Math.log2(a.count + 1);
            const scoreB = satB * Math.log2(b.count + 1);
            return scoreB - scoreA;
          });

        if (sorted.length > 0) {
          const best = sorted[0];
          const rr = Math.min(best.r + 12, 255);
          const gg = Math.min(best.g + 12, 255);
          const bb = Math.min(best.b + 12, 255);

          const rr2 = Math.max(best.r - 20, 0);
          const gg2 = Math.max(best.g - 20, 0);
          const bb2 = Math.max(best.b - 20, 0);

          const newGradient = `radial-gradient(circle at 50% 50%, rgb(${rr}, ${gg}, ${bb}), rgb(${rr2}, ${gg2}, ${rr2}))`;

          // 使用颜色插值实现平滑过渡
          const steps = 10;
          let currentStep = 0;
          const interval = setInterval(() => {
            currentStep++;
            const t = currentStep / steps;
            const easeT = t * t * (3 - 2 * t); // smoothstep
            const currentR = Math.round(40 + (rr - 40) * easeT);
            const currentG = Math.round(50 + (gg - 50) * easeT);
            const currentB = Math.round(60 + (bb - 60) * easeT);
            const currentR2 = Math.round(30 + (rr2 - 30) * easeT);
            const currentG2 = Math.round(40 + (gg2 - 40) * easeT);
            const currentB2 = Math.round(50 + (bb2 - 50) * easeT);
            bgGradient = `radial-gradient(circle at 50% 50%, rgb(${currentR}, ${currentG}, ${currentB}), rgb(${currentR2}, ${currentG2}, ${currentB2}))`;
            if (currentStep >= steps) {
              clearInterval(interval);
            }
          }, 80);
        } else {
          bgColor = "rgb(40, 50, 60)";
          bgGradient =
            "radial-gradient(circle at 50% 50%, rgb(40, 50, 60), rgb(30, 40, 50))";
        }
      } catch {
        bgColor = "rgb(40, 50, 60)";
        bgGradient =
          "radial-gradient(circle at 50% 50%, rgb(40, 50, 60), rgb(30, 40, 50))";
      }
    };
    img.src = imgSrc;
  }

  onMount(async () => {
    const appWindow = getCurrentWindow();
    const size = await appWindow.innerSize();
    windowSize = { width: size.width, height: size.height };

    // 监听窗口大小变化
    unlistenResize = await appWindow.onResized(({ payload }) => {
      windowSize = { width: payload.width, height: payload.height };

      // 防抖保存位置和大小
      clearTimeout(savePositionTimeout);
      savePositionTimeout = setTimeout(async () => {
        try {
          const position = await appWindow.outerPosition();
          await invoke("save_floating_window_position", {
            x: Math.round(position.x),
            y: Math.round(position.y),
            width: payload.width,
            height: payload.height,
          });
          console.log("[悬浮窗] 位置和大小已保存");
        } catch (error) {
          console.error("[悬浮窗] 保存位置失败:", error);
        }
      }, 500); // 500ms 防抖
    });

    // 监听窗口位置变化
    const unlistenMoved = await appWindow.onMoved(({ payload }) => {
      // 防抖保存位置和大小
      clearTimeout(savePositionTimeout);
      savePositionTimeout = setTimeout(async () => {
        try {
          await invoke("save_floating_window_position", {
            x: Math.round(payload.x),
            y: Math.round(payload.y),
            width: windowSize.width,
            height: windowSize.height,
          });
          console.log("[悬浮窗] 位置已保存");
        } catch (error) {
          console.error("[悬浮窗] 保存位置失败:", error);
        }
      }, 500); // 500ms 防抖
    });

    // 添加全局鼠标事件监听
    let handleMouseMove = (e: MouseEvent) => {
      const player = document.querySelector(".player");
      if (player) {
        const rect = player.getBoundingClientRect();
        // 添加 5px 的容差，避免边界抖动
        // 排除底部区域，但包括顶部区域（让顶部栏可以触发）
        const isInside =
          e.clientX >= rect.left + 5 &&
          e.clientX <= rect.right - 5 &&
          e.clientY >= rect.top && // 包括顶部，让顶部栏可以触发
          e.clientY <= rect.bottom - 80 - 5; // 减去底部 80px 和 5px 容差

        // 使用 requestAnimationFrame 来避免频繁更新
        requestAnimationFrame(() => {
          if (isInside && !isHovered) {
            isHovered = true;
          } else if (!isInside && isHovered) {
            isHovered = false;
          }
        });
      }
    };

    // 保存引用以便清理
    (window as any).__handleMouseMove = handleMouseMove;
    window.addEventListener("mousemove", handleMouseMove);

    // 保存移动监听器引用
    (window as any).__unlistenMoved = unlistenMoved;

    unlisten = await listen("media-update", (event: any) => {
      if (event.payload) {
        const payload = event.payload;
        const newTrackKey = `${payload.title}-${payload.artist}`;
        if (newTrackKey !== currentTrackKey) {
          currentTrackKey = newTrackKey;

          // 使用 SMTC 提供的图片作为基础
          const smtcCover =
            payload.album_art || payload.thumbnail || payload.cover_url || "";

          console.log("📻 SMTC 传递的图片:", {
            album_art: payload.album_art,
            thumbnail: payload.thumbnail,
            cover_url: payload.cover_url,
            using: smtcCover ? smtcCover.substring(0, 80) + "..." : "无",
          });

          mediaState = { ...mediaState, ...payload, album_art: smtcCover };

          // 始终尝试获取 iTunes 高清图
          fetchHighResCover(payload.title, payload.artist, smtcCover).then(
            (hdCover) => {
              const img = new Image();
              if (hdCover.startsWith("http")) img.crossOrigin = "Anonymous";
              img.onload = () => {
                if (currentTrackKey === newTrackKey) {
                  displayCover = hdCover;
                  flipKey += 1; // 触发 key 更新，带动画
                  extractColors(hdCover);
                }
              };
              img.onerror = () => {
                if (currentTrackKey === newTrackKey) {
                  // 高清图加载失败，使用 SMTC 图片
                  displayCover = smtcCover;
                  flipKey += 1;
                  if (smtcCover) extractColors(smtcCover);
                }
              };
              img.src = hdCover;
            },
          );
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
    if (unlistenResize) unlistenResize();
    if ((window as any).__unlistenMoved) {
      (window as any).__unlistenMoved();
      delete (window as any).__unlistenMoved;
    }
    if ((window as any).__handleMouseMove) {
      window.removeEventListener(
        "mousemove",
        (window as any).__handleMouseMove,
      );
      delete (window as any).__handleMouseMove;
    }
    clearTimeout(savePositionTimeout);
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

  let showControls = $derived(
    isHovered && windowSize.width > 100 && windowSize.height > 100,
  );

  async function togglePlay(e: MouseEvent) {
    e.stopPropagation();
    await invoke("control_media", { action: "play_pause" });
    mediaState.is_playing = !mediaState.is_playing;
  }

  function closeWindow(e: MouseEvent) {
    e.stopPropagation();
    getCurrentWindow().close();
  }

  // 用于拖拽的标题栏区域
  function handleTitlebarMousedown() {
    getCurrentWindow().startDragging();
  }
</script>

<div
  class="player"
  class:hovered={isHovered}
  role="region"
  aria-label="音乐播放器"
  style="--bg: {bgColor}; --bg-gradient: {bgGradient};"
>
  <div class="bg-solid"></div>

  <!-- 可拖拽的顶部栏 - 鼠标悬停时滑下 -->
  <div class="drag-bar" onmousedown={handleTitlebarMousedown}>
    <div class="drag-handle">
      <div class="drag-dots">
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
        <div class="drag-dot"></div>
      </div>
    </div>
    <button class="close-btn-topbar" onclick={closeWindow} aria-label="关闭">
      <X size={16} strokeWidth={2} />
    </button>
  </div>

  <div class="album-stage">
    {#if displayCover}
      {#key flipKey}
        <div class="album-wrapper">
          <img
            src={displayCover}
            alt="专辑封面"
            class="album-art flip-enter"
            draggable="false"
          />
        </div>
      {/key}
    {:else}
      <div class="album-placeholder">
        <Play size={40} strokeWidth={1} color="rgba(255,255,255,0.15)" />
      </div>
    {/if}
  </div>

  <!-- 歌曲信息层 - 贴在渐变背景上 -->
  <div class="track-info-layer">
    <div class="track-title" title={mediaState.title}>
      {mediaState.title}
    </div>
    <div class="track-artist" title={mediaState.artist}>
      {mediaState.artist}
    </div>
    <!-- 右下角拖拽识别 -->
    <div class="resize-handle"></div>
  </div>

  {#if mediaState.source !== "netease"}
    <div class="progress-layer">
      <div class="progress-container">
        <div class="progress-row">
          <span class="time">{formatTime(mediaState.position_ms)}</span>
          <div class="progress-track">
            <div class="progress-fill" style="width: {progressPercent}%"></div>
          </div>
          <span class="time">{formatTime(mediaState.duration_ms)}</span>
        </div>
      </div>
    </div>
  {/if}

  <!-- 控制按钮遮罩层 -->
  <div class="controls-overlay" class:visible={showControls}>
    <div class="controls">
      <button
        class="ctrl-btn"
        onclick={(e) => {
          e.stopPropagation();
          invoke("control_media", { action: "prev" });
        }}
        aria-label="上一首"
      >
        <SkipBack size={18} fill="currentColor" />
      </button>

      <button
        class="play-btn"
        onclick={togglePlay}
        aria-label={mediaState.is_playing ? "暂停" : "播放"}
      >
        {#if mediaState.is_playing}
          <Pause size={24} fill="black" color="black" />
        {:else}
          <Play size={24} fill="black" color="black" style="margin-left:2px" />
        {/if}
      </button>

      <button
        class="ctrl-btn"
        onclick={(e) => {
          e.stopPropagation();
          invoke("control_media", { action: "next" });
        }}
        aria-label="下一首"
      >
        <SkipForward size={18} fill="currentColor" />
      </button>
    </div>
  </div>
</div>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      system-ui,
      sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  .player {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    border-radius: 5px;
    background: #000;
    user-select: none;
    -webkit-user-select: none;
    border: 5px solid #000;
    box-sizing: border-box;
    box-shadow:
      0 12px 48px rgba(0, 0, 0, 0.5),
      0 0 0 0.5px rgba(255, 255, 255, 0.05);
  }

  .bg-solid {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 60px; /* 填充到歌曲信息层上方 */
    z-index: 1;
    background: var(--bg-gradient);
    border-radius: 10px;
    transition: background 1.2s cubic-bezier(0.4, 0, 0.2, 1);
    /* 从上往下加深的阴影效果 */
    box-shadow: inset 0 -20px 80px rgba(0, 0, 0, 0.4);
  }

  /* 可拖拽的顶部栏 - 鼠标悬停时滑下 */
  .drag-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 25px; /* 调整高度为 25px */
    z-index: 300; /* 最高层级，确保不被遮罩层盖住 */
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    visibility: hidden; /* 完全隐藏 */
    transform: translateY(-10px);
    transition:
      opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      visibility 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    background: #000000; /* 纯黑色 */
    pointer-events: auto; /* 确保可以接收鼠标事件 */
  }

  /* 使用伪元素创建内凹圆角缺口 */
  .drag-bar::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 12px;
    background-color: #000000;
    /* 创建底部内凹圆角：使用两个圆形遮罩在底部切出缺口 */
    -webkit-mask-image: radial-gradient(
        circle at 12px 12px,
        transparent 12px,
        black 12px
      ),
      radial-gradient(
        circle at calc(100% - 12px) 12px,
        transparent 12px,
        black 12px
      );
    mask-image: radial-gradient(
        circle at 12px 12px,
        transparent 12px,
        black 12px
      ),
      radial-gradient(
        circle at calc(100% - 12px) 12px,
        transparent 12px,
        black 12px
      );
    -webkit-mask-position:
      left bottom,
      right bottom;
    mask-position:
      left bottom,
      right bottom;
    -webkit-mask-size: 24px 24px;
    mask-size: 24px 24px;
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
  }

  .player.hovered .drag-bar {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 16px;
  }

  .drag-dots {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: repeat(2, 1fr);
    gap: 1px;
    width: 18px;
    height: 8px;
  }

  .drag-dot {
    width: 3px;
    height: 3px;
    background: rgba(255, 255, 255, 0.5);
    border-radius: 50%;
  }

  /* 顶部栏关闭按钮 */
  .close-btn-topbar {
    position: absolute;
    right: 0px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    outline: none;
    padding: 6px;
    cursor: pointer;
    color: #dfdfdf;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
    z-index: 10;
  }

  .close-btn-topbar:hover {
    color: #fff;
    transform: translateY(-50%) scale(1.1);
    background: transparent;
  }

  .close-btn-topbar:active {
    transform: translateY(-50%) scale(0.9);
    background: transparent;
  }

  /* ==================== 专辑封面 ==================== */
  .album-stage {
    position: absolute;
    top: 25px; /* 留出顶部栏的空间（20px） */
    bottom: 60px; /* 与填充色底部对齐 */
    left: 0;
    right: 0;
    z-index: 3;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px; /* 减小内边距，让图片更大 */
    border-radius: 5px;
    perspective: 1200px; /* 3D 透视效果 */
  }

  .album-wrapper {
    width: 100%;
    height: 100%;
    aspect-ratio: 1 / 1;
    max-width: min(calc(100% - 24px), calc(100vh - 100px - 24px), 600px);
    max-height: min(calc(100% - 24px), calc(100vh - 100px - 24px), 600px);
    position: relative;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.5),
      0 2px 12px rgba(0, 0, 0, 0.3);
    border-radius: 5px;
    overflow: hidden;
    min-width: 50px;
    min-height: 50px;
    /* 3D 变换容器 */
    transform-style: preserve-3d;
    transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .album-wrapper.flipping {
    animation: pulse-glow 0.6s ease-out;
  }

  /* 3D 翻转容器 */
  .flipper {
    position: relative;
    width: 100%;
    height: 100%;
    transform-style: preserve-3d;
  }

  /* 翻转面 */
  .flip-face {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    overflow: hidden;
    border-radius: 5px;
  }

  /* 前面（旧封面） */
  .flip-face-front {
    transform: rotateY(0deg);
    animation: flip-out-front 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  /* 后面（新封面） */
  .flip-face-back {
    transform: rotateY(180deg);
    animation: flip-in-back 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  .album-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    aspect-ratio: 1 / 1;
    object-fit: cover;
    display: block;
    border-radius: 5px;
  }

  /* 翻转进入动画 */
  .album-art.flip-enter {
    animation: flip-enter 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    transform-origin: center;
    will-change: transform, opacity;
  }

  .album-placeholder {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    background: rgba(255, 255, 255, 0.03);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 5px;
  }

  /* ==================== 歌曲信息层 ==================== */
  .track-info-layer {
    position: absolute;
    bottom: 8px; /* 往下移动，更靠近底部 */
    left: 0;
    right: 0;
    z-index: 5;
    padding: 0 5px; /* 减小左右内边距，让文字更靠左 */
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    pointer-events: auto;
  }

  .track-title {
    color: #dfdfdf; /* 调整字体颜色 */
    font-size: 20px; /* 调整字体大小 */
    font-weight: 600;
    letter-spacing: 0.01em;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family:
      "SF Pro Display",
      -apple-system,
      BlinkMacSystemFont,
      sans-serif;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .track-artist {
    color: rgba(255, 255, 255, 0.7);
    font-size: 12px; /* 调整字体大小 */
    font-weight: 500;
    letter-spacing: 0.02em;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  /* ==================== 进度条 ==================== */
  .progress-layer {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 251;
    display: flex;
    flex-direction: column;
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .player.hovered .progress-layer {
    opacity: 1;
  }

  .progress-container {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 0 16px; /* 与歌曲信息对齐 */
    box-sizing: border-box;
    margin-bottom: 56px; /* 调整到歌曲信息上方 */
  }

  .progress-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .time {
    font-size: 10px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.65);
    min-width: 28px;
    text-align: center;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.03em;
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }

  .progress-track {
    flex: 1;
    height: 4px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 5px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #fff;
    border-radius: 5px;
    transition: width 1s linear;
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.4);
  }

  /* 控制按钮遮罩层 */
  .controls-overlay {
    position: absolute;
    top: 25px; /* 从顶部栏下方开始 */
    left: 0;
    right: 0;
    bottom: 60px; /* 到歌曲信息层上方结束 */
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0), rgba(0, 0, 0, 0.95));
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 250; /* 低于顶部栏的 z-index: 300 */
    pointer-events: auto;
    opacity: 0;
    visibility: hidden;
    transition:
      opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      visibility 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .controls-overlay.visible {
    opacity: 1;
    visibility: visible;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  /* 右下角拖拽识别 */
  .resize-handle {
    position: absolute;
    right: 2px;
    bottom: -2px;
    width: 12px;
    height: 12px;
    cursor: se-resize;
    pointer-events: auto;
  }

  .resize-handle::before {
    content: "";
    position: absolute;
    right: 0;
    bottom: 0;
    width: 6px;
    height: 1px;
    background: rgba(255, 255, 255, 0.35);
    transform: rotate(-45deg);
    transform-origin: right bottom;
  }

  .resize-handle::after {
    content: "";
    position: absolute;
    right: 0;
    bottom: 0;
    width: 12px;
    height: 1px;
    background: rgba(255, 255, 255, 0.35);
    transform: rotate(-45deg);
    transform-origin: right bottom;
    margin-right: 0px;
    margin-bottom: 4px;
  }

  .ctrl-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
  }

  .ctrl-btn:hover {
    color: #fff;
    transform: scale(1.12);
    background: rgba(255, 255, 255, 0.1);
  }

  .close-btn-bar {
    background: none;
    border: none;
    padding: 6px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
    flex-shrink: 0;
    margin-left: 12px;
  }

  .close-btn-bar:hover {
    color: #fff;
    transform: scale(1.1);
    background: rgba(255, 255, 255, 0.15);
  }

  .ctrl-btn:active {
    transform: scale(0.9);
  }

  .play-btn {
    width: 56px;
    height: 56px;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #fff;
    color: #000;
    transition:
      background 0.2s ease,
      transform 0.2s ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .play-btn:hover {
    background: #f0f0f0;
    transform: scale(1.05);
  }

  .play-btn:active {
    transform: scale(0.95);
  }

  .close-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition:
      color 0.15s ease,
      transform 0.15s ease,
      background 0.15s ease;
    margin-left: 2px;
  }

  .close-btn:hover {
    color: #fff;
    transform: scale(1.12);
    background: rgba(226, 33, 52, 0.85);
  }

  .close-btn:active {
    transform: scale(0.9);
  }

  /* ==================== 3D 翻转动画关键帧 ==================== */
  @keyframes flip-out-front {
    0% {
      transform: rotateY(0deg);
      opacity: 1;
      filter: brightness(1);
    }
    50% {
      transform: rotateY(-90deg);
      opacity: 0.5;
      filter: brightness(0.8);
    }
    100% {
      transform: rotateY(-180deg);
      opacity: 0;
      filter: brightness(0.5);
    }
  }

  @keyframes flip-in-back {
    0% {
      transform: rotateY(180deg);
      opacity: 0;
      filter: brightness(0.5);
    }
    50% {
      transform: rotateY(90deg);
      opacity: 0.5;
      filter: brightness(0.8);
    }
    100% {
      transform: rotateY(0deg);
      opacity: 1;
      filter: brightness(1);
    }
  }

  @keyframes flip-enter {
    0% {
      transform: perspective(1000px) rotateY(-180deg) scale(0.75);
      opacity: 0;
    }
    100% {
      transform: perspective(1000px) rotateY(0deg) scale(1);
      opacity: 1;
    }
  }

  @keyframes pulse-glow {
    0% {
      box-shadow:
        0 4px 20px rgba(0, 0, 0, 0.35),
        0 1px 6px rgba(0, 0, 0, 0.2);
    }
    50% {
      box-shadow:
        0 8px 32px rgba(0, 0, 0, 0.5),
        0 2px 12px rgba(255, 255, 255, 0.1),
        0 0 24px rgba(255, 255, 255, 0.15);
    }
    100% {
      box-shadow:
        0 4px 20px rgba(0, 0, 0, 0.35),
        0 1px 6px rgba(0, 0, 0, 0.2);
    }
  }
</style>
