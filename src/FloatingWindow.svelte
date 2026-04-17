<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    X,
    Pin,
    Minimize2,
  } from "lucide-svelte";

  interface MediaState {
    title: string;
    artist: string;
    album_art: string;
    is_playing: boolean;
    position_ms: number;
    duration_ms: number;
    source?: string;
  }

  interface WindowSize {
    width: number;
    height: number;
  }

  const PLACEHOLDER_TITLE = "等待播放...";
  const PLACEHOLDER_ARTIST = "未知艺术家";
  const FETCH_TIMEOUT = 8000;
  const MIN_COVER_SIZE_KB = 50;

  let mediaState = $state<MediaState>({
    title: PLACEHOLDER_TITLE,
    artist: PLACEHOLDER_ARTIST,
    album_art: "",
    is_playing: false,
    position_ms: 0,
    duration_ms: 0,
  });

  let currentTrackKey = "";
  let displayCover = $state("");
  let previousCover = $state("");
  let isHovered = $state(false);
  let slideDirection = $state<"left" | "right" | "">("");
  let isAnimating = $state(false); // 动画进行中标志
  let animationTimeoutId: ReturnType<typeof setTimeout> | null = null; // 动画定时器ID
  let bgColor = $state("rgb(40, 50, 60)");
  let bgGradient = $state(
    "radial-gradient(circle at 50% 50%, rgb(40, 50, 60), rgb(30, 40, 50))",
  );
  let windowSize = $state<WindowSize>({ width: 0, height: 0 });

  // MV 播放相关
  let isMVPlaybackEnabled = $state(false); // MV 播放功能是否启用
  let mvUrl = $state(""); // MV 视频链接
  let isPlayingMV = $state(false); // 是否正在播放 MV

  // 置顶状态
  let isAlwaysOnTop = $state(false);

  // 锁定悬浮窗（禁止移动）
  let isFloatingWindowLocked = $state(false); // 初始值，会在 onMount 中从设置加载

  // 专辑封面设置
  let enableHDCover = $state(true); // 高清封面获取
  let enablePixelArt = $state(false); // 像素化封面

  let unlisten: () => void;
  let unlistenResize: () => void;
  let progressInterval: ReturnType<typeof setInterval> | null = null;
  let savePositionTimeout: ReturnType<typeof setTimeout> | null = null;

  async function fetchHighResCover(
    title: string,
    artist: string,
    fallbackCover: string,
  ) {
    if (!title || title === PLACEHOLDER_TITLE) return fallbackCover;

    if (fallbackCover && fallbackCover.startsWith("data:image")) {
      try {
        const sizeInBytes = Math.round((fallbackCover.length * 3) / 4);
        if (sizeInBytes > MIN_COVER_SIZE_KB * 1024) {
          return fallbackCover;
        }
      } catch {
        // 忽略错误，继续获取网络高清图
      }
    }

    const fetchWithTimeout = async (
      url: string,
      timeout = FETCH_TIMEOUT,
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

    interface CoverSource {
      name: string;
      fetch: () => Promise<string | null>;
    }

    // 先尝试获取专辑封面
    const albumSources: CoverSource[] = [
      {
        name: "iTunes",
        fetch: async () => {
          try {
            const query = encodeURIComponent(`${title} ${artist}`);
            const res = await fetchWithTimeout(
              `https://itunes.apple.com/search?term=${query}&limit=1&media=music`,
            );

            // 检查响应状态
            if (!res.ok) {
              console.warn(`iTunes API 返回错误状态：${res.status}`);
              return null;
            }

            const contentType = res.headers.get("content-type");
            if (!contentType || !contentType.includes("application/json")) {
              console.warn("iTunes API 返回非 JSON 内容");
              return null;
            }

            const data = await res.json();
            if (data.results?.length > 0) {
              // 将 iTunes 图片改为 600x600
              return data.results[0].artworkUrl100.replace(
                "100x100bb.jpg",
                "600x600bb.jpg",
              );
            }
            return null;
          } catch (error) {
            console.warn("iTunes 封面获取失败:", error);
            return null;
          }
        },
      },
      {
        name: "Spotify",
        fetch: async () => {
          try {
            const query = encodeURIComponent(`${artist} ${title}`);
            const res = await fetchWithTimeout(
              `https://open.spotify.com/search/${query}`,
            );
            if (!res.ok) {
              console.warn(`Spotify 返回错误状态：${res.status}`);
              return null;
            }
            const html = await res.text();
            const imgMatch = html.match(/"images":\[{"url":"([^"]+)"}/);
            if (imgMatch?.[1]) {
              return imgMatch[1].replace("640x640", "600x600");
            }
            const ogMatch = html.match(
              /<meta property="og:image" content="([^"]+)"/,
            );
            return ogMatch?.[1]?.replace("640x640", "600x600") || null;
          } catch (error) {
            console.warn("Spotify 封面获取失败:", error);
            return null;
          }
        },
      },
      {
        name: "Apple Music",
        fetch: async () => {
          try {
            const query = encodeURIComponent(`${title} ${artist}`);
            const res = await fetchWithTimeout(
              `https://music.apple.com/search?term=${query}`,
            );
            if (!res.ok) {
              console.warn(`Apple Music 返回错误状态：${res.status}`);
              return null;
            }
            const html = await res.text();
            const match = html.match(/"artworkUrl100":"([^"]+)"/);
            return (
              match?.[1]?.replace("100x100bb.jpg", "600x600bb.jpg") || null
            );
          } catch (error) {
            console.warn("Apple Music 封面获取失败:", error);
            return null;
          }
        },
      },
      {
        name: "Last.fm",
        fetch: async () => {
          try {
            const artistQuery = encodeURIComponent(artist);
            const trackQuery = encodeURIComponent(title);
            const res = await fetchWithTimeout(
              `https://www.last.fm/music/${artistQuery}/_/${trackQuery}`,
            );
            if (!res.ok) {
              console.warn(`Last.fm 返回错误状态：${res.status}`);
              return null;
            }
            const html = await res.text();
            const match = html.match(
              /<meta property="og:image" content="([^"]+)"/,
            );
            return match?.[1] || null;
          } catch (error) {
            console.warn("Last.fm 封面获取失败:", error);
            return null;
          }
        },
      },
      {
        name: "MusicBrainz",
        fetch: async () => {
          try {
            const query = encodeURIComponent(
              `artist:${artist} recording:${title}`,
            );
            const res = await fetchWithTimeout(
              `https://musicbrainz.org/ws/2/recording/?query=${query}&fmt=json&limit=1`,
            );
            if (!res.ok) {
              console.warn(`MusicBrainz 返回错误状态：${res.status}`);
              return null;
            }
            const data = await res.json();
            if (data.recordings?.length > 0) {
              const recording = data.recordings[0];
              const releases = recording.releases;
              if (releases?.length > 0) {
                const release = releases[0];
                if (release["cover-art-archive"]?.count > 0) {
                  return `https://coverartarchive.org/release/${release.id}/front`;
                }
              }
            }
            return null;
          } catch (error) {
            console.warn("MusicBrainz 封面获取失败:", error);
            return null;
          }
        },
      },
    ];

    // 如果专辑封面失败，尝试获取歌手图片
    const artistSources: CoverSource[] = [
      {
        name: "iTunes Artist",
        fetch: async () => {
          const query = encodeURIComponent(artist);
          const res = await fetchWithTimeout(
            `https://itunes.apple.com/search?term=${query}&limit=1&entity=musicArtist`,
          );
          const data = await res.json();
          if (data.results?.length > 0) {
            // 获取歌手图片并转为 600x600
            return (
              data.results[0].artistArtworkUrl100?.replace(
                "100x100bb.jpg",
                "600x600bb.jpg",
              ) || null
            );
          }
          return null;
        },
      },
      {
        name: "Spotify Artist",
        fetch: async () => {
          const query = encodeURIComponent(artist);
          const res = await fetchWithTimeout(
            `https://open.spotify.com/search/${query}`,
          );
          const html = await res.text();
          // 尝试获取歌手图片
          const imgMatch = html.match(/"images":\[{"url":"([^"]+)"}/);
          if (imgMatch?.[1]) {
            return imgMatch[1].replace("640x640", "600x600");
          }
          return null;
        },
      },
      {
        name: "Last.fm Artist",
        fetch: async () => {
          const query = encodeURIComponent(artist);
          const res = await fetchWithTimeout(
            `https://www.last.fm/search/artists?q=${query}`,
          );
          const html = await res.text();
          const match = html.match(/<img class="avatar" src="([^"]+)"/);
          return match?.[1] || null;
        },
      },
    ];

    // 先尝试获取专辑封面
    for (const source of albumSources) {
      try {
        const result = await source.fetch();
        if (result) {
          console.log(`✅ 高清图获取成功，来源：${source.name}`);

          // 下载并缓存图片
          try {
            const cachedPath = await invoke<string>("download_and_cache", {
              url: result,
              contentType: "image/jpeg",
            });
            const safeUrl = convertFileSrc(cachedPath);
            return safeUrl;
          } catch (cacheError) {
            console.warn("[封面] 缓存失败，使用原始链接:", cacheError);
            // 缓存失败，返回原始链接
            return result;
          }
        }
      } catch (error: any) {
        if (error.name === "AbortError") {
          console.warn(`⏱️ ${source.name} 超时`);
        } else {
          console.warn(`❌ ${source.name} 失败:`, error.message);
        }
      }
    }

    // 专辑封面失败，尝试获取歌手图片
    console.log("⚠️ 专辑封面获取失败，尝试获取歌手图片...");
    for (const source of artistSources) {
      try {
        const result = await source.fetch();
        if (result) {
          console.log(`✅ 歌手图获取成功，来源：${source.name}`);

          // 下载并缓存图片
          try {
            const cachedPath = await invoke<string>("download_and_cache", {
              url: result,
              contentType: "image/jpeg",
            });
            const safeUrl = convertFileSrc(cachedPath);
            return safeUrl;
          } catch (cacheError) {
            console.warn("[封面] 缓存失败，使用原始链接:", cacheError);
            return result;
          }
        }
      } catch (error: any) {
        if (error.name === "AbortError") {
          console.warn(`⏱️ ${source.name} 超时`);
        } else {
          console.warn(`❌ ${source.name} 失败:`, error.message);
        }
      }
    }

    console.log("⚠️ 所有图片来源都失败，使用备用图片");
    return fallbackCover;
  }

  async function extractColors(imgSrc: string) {
    const DEFAULT_COLOR = { r: 60, g: 80, b: 100 };
    console.log("[颜色提取] 开始:", imgSrc.substring(0, 50));

    // 保存当前颜色
    const currentColor = parseColor(bgColor);

    try {
      const [r, g, b] = await invoke<[number, number, number]>(
        "extract_dominant_color",
        { imagePath: imgSrc },
      );

      if (
        r === DEFAULT_COLOR.r &&
        g === DEFAULT_COLOR.g &&
        b === DEFAULT_COLOR.b
      ) {
        console.log("[颜色提取] 使用默认颜色");
        animateColorTransition(currentColor, { r: 40, g: 50, b: 60 });
        return;
      }

      // 避免黑色：确保 RGB 值不低于最小亮度
      const MIN_BRIGHTNESS = 30; // 最小亮度值
      const adjustedR = Math.max(r, MIN_BRIGHTNESS);
      const adjustedG = Math.max(g, MIN_BRIGHTNESS);
      const adjustedB = Math.max(b, MIN_BRIGHTNESS);

      // 检查整体亮度，如果太暗则提升
      const brightness = (adjustedR + adjustedG + adjustedB) / 3;
      if (brightness < 50) {
        const scale = 50 / brightness;
        const finalR = Math.min(Math.round(adjustedR * scale), 255);
        const finalG = Math.min(Math.round(adjustedG * scale), 255);
        const finalB = Math.min(Math.round(adjustedB * scale), 255);

        const targetColor = {
          r: Math.min(finalR + 12, 255),
          g: Math.min(finalG + 12, 255),
          b: Math.min(finalB + 12, 255),
        };
        animateColorTransition(currentColor, targetColor);
      } else {
        const targetColor = {
          r: Math.min(adjustedR + 12, 255),
          g: Math.min(adjustedG + 12, 255),
          b: Math.min(adjustedB + 12, 255),
        };
        animateColorTransition(currentColor, targetColor);
      }

      console.log("[颜色提取] 成功");
    } catch (error) {
      console.error("[颜色提取] 失败:", error);
      animateColorTransition(currentColor, { r: 40, g: 50, b: 60 });
    }
  }

  // 解析颜色字符串为 RGB 对象
  function parseColor(colorStr: string): { r: number; g: number; b: number } {
    const match = colorStr.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/);
    if (match) {
      return {
        r: parseInt(match[1]),
        g: parseInt(match[2]),
        b: parseInt(match[3]),
      };
    }
    return { r: 40, g: 50, b: 60 }; // 默认颜色
  }

  // 颜色渐变动画
  function animateColorTransition(
    from: { r: number; g: number; b: number },
    to: { r: number; g: number; b: number },
  ) {
    const startTime = Date.now();
    const duration = 300;

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const eased = easeInOutCubic(progress);

      // 插值计算新颜色
      const currentR = Math.round(from.r + (to.r - from.r) * eased);
      const currentG = Math.round(from.g + (to.g - from.g) * eased);
      const currentB = Math.round(from.b + (to.b - from.b) * eased);

      bgColor = `rgb(${currentR}, ${currentG}, ${currentB})`;
      bgGradient = `radial-gradient(circle at 50% 50%, rgb(${Math.min(currentR + 8, 255)}, ${Math.min(currentG + 8, 255)}, ${Math.min(currentB + 8, 255)}), rgb(${Math.max(currentR - 15, 0)}, ${Math.max(currentG - 15, 0)}, ${Math.max(currentB - 15, 0)}))`;

      if (progress < 1) {
        requestAnimationFrame(animate);
      }
    };

    requestAnimationFrame(animate);
  }

  // 封面切换函数（无动画）
  function transitionCover(
    newCover: string,
    direction: "left" | "right" = "left",
  ) {
    if (animationTimeoutId) {
      clearTimeout(animationTimeoutId);
      animationTimeoutId = null;
    }

    // 保存当前颜色作为起始颜色
    const oldBgColor = bgColor;
    const oldBgGradient = bgGradient;

    displayCover = newCover;
    if (newCover) extractColors(newCover);
    previousCover = "";
    slideDirection = "";
    isAnimating = false;

    // 背景颜色渐变动画（200ms）
    const startTime = Date.now();
    const duration = 200;

    const animateColor = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const easeProgress = easeInOutCubic(progress);

      // 如果需要继续动画
      if (progress < 1) {
        requestAnimationFrame(animateColor);
      }
    };

    // 启动动画
    animateColor();
  }

  // 缓动函数
  function easeInOutCubic(t: number): number {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }

  // 检测 MV 文件大小
  async function getMVFileSize(url: string): Promise<number> {
    try {
      const res = await fetch(url, { method: "HEAD" });
      const contentLength = res.headers.get("content-length");
      if (contentLength) {
        return parseInt(contentLength, 10);
      }
      return 0;
    } catch {
      return 0;
    }
  }

  // 检测 MV 分辨率
  function getMVResolution(
    url: string,
  ): Promise<{ width: number; height: number } | null> {
    return new Promise((resolve) => {
      const video = document.createElement("video");
      video.preload = "metadata";
      video.muted = true;

      video.onloadedmetadata = () => {
        const resolution = {
          width: video.videoWidth,
          height: video.videoHeight,
        };
        resolve(resolution);
      };

      video.onerror = () => {
        resolve(null);
      };

      video.src = url;
    });
  }

  // 从 Apple Music 获取 MV 链接（使用本地缓存）
  async function fetchMVFromAppleMusic(title: string, artist: string) {
    if (!isMVPlaybackEnabled) return null; // 功能未启用，直接返回

    try {
      const query = encodeURIComponent(`${title} ${artist}`);
      const res = await fetch(
        `https://itunes.apple.com/search?term=${query}&limit=1&media=musicVideo`,
        {
          headers: {
            Referer: "https://music.apple.com",
            "User-Agent":
              "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
          },
        },
      );
      const data = await res.json();

      if (data.results?.length > 0) {
        const mvData = data.results[0];
        const previewUrl = mvData.previewUrl; // Apple Music 提供的 MV 预览链接

        // 先检查缓存
        try {
          const cachedPath = await invoke<string | null>("get_cached_media", {
            url: previewUrl,
          });
          if (cachedPath) {
            console.log("[MV] 使用缓存文件:", cachedPath);
            // 使用 convertFileSrc 转换为可访问的 URL
            return convertFileSrc(cachedPath);
          }
        } catch (cacheError) {
          console.warn("[MV] 检查缓存失败:", cacheError);
        }

        // 检测文件大小和分辨率
        const fileSize = await getMVFileSize(previewUrl);
        const fileSizeMB = (fileSize / (1024 * 1024)).toFixed(2);
        const resolution = await getMVResolution(previewUrl);

        console.log("[MV] 找到 MV:", mvData.trackName);
        console.log("[MV] URL:", previewUrl);
        console.log("[MV] 文件大小:", fileSizeMB, "MB");
        if (resolution) {
          console.log(
            "[MV] 分辨率:",
            `${resolution.width}x${resolution.height}`,
          );
        }

        // 下载并缓存 MV
        try {
          console.log("[MV] 下载并缓存...");
          const cachedPath = await invoke<string>("download_and_cache", {
            url: previewUrl,
            contentType: "video/mp4",
          });
          const safeUrl = convertFileSrc(cachedPath);
          return safeUrl;
        } catch (cacheError) {
          console.warn("[MV] 缓存失败，使用原始链接:", cacheError);
          // 缓存失败，返回原始链接
          return previewUrl;
        }
      }
      return null;
    } catch (error) {
      console.error("[MV] 获取失败:", error);
      return null;
    }
  }

  onMount(async () => {
    // 读取设置
    try {
      const settings = await invoke<any>("get_settings");
      isMVPlaybackEnabled = settings.enable_mv_playback ?? false;
      console.log(
        "[MV 播放] 功能状态:",
        isMVPlaybackEnabled ? "已启用" : "已禁用",
      );

      // 加载置顶设置
      isAlwaysOnTop = settings.always_on_top ?? true; // 默认置顶
      console.log("[置顶] 初始状态:", isAlwaysOnTop ? "已置顶" : "未置顶");

      // 加载锁定悬浮窗设置
      isFloatingWindowLocked = settings.lock_floating_window ?? false;
      console.log(
        "[锁定悬浮窗] 初始状态:",
        isFloatingWindowLocked ? "锁定" : "解锁",
      );

      // 加载专辑封面设置
      enableHDCover = settings.enable_hd_cover ?? true;
      enablePixelArt = settings.enable_pixel_art ?? false;
      console.log(
        "[专辑封面] 高清获取:",
        enableHDCover ? "开启" : "关闭",
        "| 像素化:",
        enablePixelArt ? "开启" : "关闭",
      );

      // 设置窗口是否可调整大小
      await invoke("set_floating_window_resizable", {
        resizable: !isFloatingWindowLocked,
      });
    } catch (error) {
      console.error("[设置] 读取失败:", error);
      isMVPlaybackEnabled = false;
      isAlwaysOnTop = false;
    }

    // 初始化事件监听器管理器
    eventListeners = [];

    // 监听 MV 播放设置变化事件
    const unlistenMVChange = await listen(
      "mv-playback-changed",
      (event: any) => {
        isMVPlaybackEnabled = event.payload.enable;
        console.log(
          "[MV 播放] 设置已更新:",
          isMVPlaybackEnabled ? "已启用" : "已禁用",
        );
        // 如果关闭了 MV 播放，停止当前播放
        if (!isMVPlaybackEnabled) {
          isPlayingMV = false;
          mvUrl = "";
          console.log("[MV 播放] 已停止");
        }
      },
    );
    eventListeners.push(unlistenMVChange);

    // 监听锁定悬浮窗设置变化事件
    const unlistenLockChange = await listen(
      "lock-floating-window-changed",
      (event: any) => {
        isFloatingWindowLocked = event.payload.lock;
        console.log(
          "[锁定悬浮窗] 设置已更新:",
          isFloatingWindowLocked ? "锁定" : "解锁",
        );

        // 同时设置窗口是否可调整大小
        invoke("set_floating_window_resizable", {
          resizable: !isFloatingWindowLocked,
        }).catch((err) => {
          console.error("[锁定] 设置窗口可调整大小失败:", err);
        });
      },
    );
    eventListeners.push(unlistenLockChange);

    // 监听高清封面获取设置变化事件
    const unlistenHDCoverChange = await listen(
      "hd-cover-changed",
      (event: any) => {
        enableHDCover = event.payload.enableHDCover;
        console.log("[高清封面] 设置已更新:", enableHDCover ? "开启" : "关闭");
      },
    );
    eventListeners.push(unlistenHDCoverChange);

    // 监听像素化封面设置变化事件
    const unlistenPixelArtChange = await listen(
      "pixel-art-changed",
      (event: any) => {
        enablePixelArt = event.payload.enablePixelArt;
        console.log(
          "[像素化封面] 设置已更新:",
          enablePixelArt ? "开启" : "关闭",
        );
      },
    );
    eventListeners.push(unlistenPixelArtChange);

    const appWindow = getCurrentWindow();
    const size = await appWindow.innerSize();
    windowSize = { width: size.width, height: size.height };

    // 监听窗口大小变化
    unlistenResize = await appWindow.onResized(({ payload }) => {
      // 锁定时忽略大小变化
      if (isFloatingWindowLocked) {
        console.log("[缩放] 悬浮窗已锁定，禁止缩放");
        return;
      }

      windowSize = { width: payload.width, height: payload.height };

      // 防抖保存位置和大小
      if (savePositionTimeout) clearTimeout(savePositionTimeout);
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
      if (savePositionTimeout) clearTimeout(savePositionTimeout);
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

    // 监听媒体更新事件（带防抖）
    let mediaUpdateTimeout: ReturnType<typeof setTimeout> | null = null;
    unlisten = await listen("media-update", (event: any) => {
      // 防抖处理，避免频繁更新
      if (mediaUpdateTimeout) {
        clearTimeout(mediaUpdateTimeout);
      }

      mediaUpdateTimeout = setTimeout(() => {
        const payload = event.payload;
        const newTrackKey = `${payload.title}-${payload.artist}`;

        // 检查是否是空状态（播放器关闭或无媒体）
        const isEmptyState =
          !payload.title ||
          payload.title === "" ||
          payload.title === "等待播放...";

        if (isEmptyState) {
          // 播放器退出，重置为等待状态
          currentTrackKey = "";
          mediaState = {
            title: PLACEHOLDER_TITLE,
            artist: PLACEHOLDER_ARTIST,
            album_art: "",
            is_playing: false,
            position_ms: 0,
            duration_ms: 0,
          };
          displayCover = "";
          isPlayingMV = false;
          mvUrl = "";
          console.log("[媒体更新] 播放器已退出，重置状态");
        } else if (newTrackKey !== currentTrackKey) {
          currentTrackKey = newTrackKey;

          // 使用 SMTC 提供的图片作为基础
          const smtcCover =
            payload.album_art || payload.thumbnail || payload.cover_url || "";

          mediaState = { ...mediaState, ...payload, album_art: smtcCover };

          // 判断是否是音乐播放器
          const isMusicPlayer =
            payload.source &&
            (payload.source === "netease" ||
              payload.source === "qqmusic" ||
              payload.source === "spotify" ||
              payload.source === "apple_music" ||
              payload.source === "local");

          // 网页播放时只获取歌手图片，音乐播放器获取专辑封面
          if (isMusicPlayer) {
            // 停止当前播放的 MV
            isPlayingMV = false;
            mvUrl = "";

            // 根据设置决定是否获取高清图
            if (enableHDCover) {
              // 获取专辑封面高清图
              fetchHighResCover(payload.title, payload.artist, smtcCover)
                .then((hdCover) => {
                  const img = new Image();
                  if (
                    hdCover.startsWith("http") &&
                    !hdCover.includes("asset.localhost")
                  )
                    img.crossOrigin = "Anonymous";
                  img.onload = () => {
                    if (currentTrackKey === newTrackKey) {
                      transitionCover(hdCover, "left");
                    }
                  };
                  img.onerror = () => {
                    if (currentTrackKey === newTrackKey) {
                      transitionCover(smtcCover, "left");
                    }
                  };
                  img.src = hdCover;
                })
                .catch(() => {
                  if (currentTrackKey === newTrackKey) {
                    transitionCover(smtcCover, "left");
                  }
                });
            } else {
              // 不获取高清图，直接使用 SMTC 图片
              if (currentTrackKey === newTrackKey) {
                transitionCover(smtcCover, "left");
              }
            }

            // 如果启用了 MV 播放，尝试获取 MV（在专辑封面加载完成后）
            if (isMVPlaybackEnabled) {
              fetchMVFromAppleMusic(payload.title, payload.artist).then(
                (mvLink) => {
                  if (mvLink && currentTrackKey === newTrackKey) {
                    setTimeout(() => {
                      if (currentTrackKey === newTrackKey) {
                        mvUrl = mvLink;
                        isPlayingMV = true;
                        console.log("[MV] 切换到新 MV:", mvLink);
                      }
                    }, 450);
                  }
                },
              );
            }
          } else {
            // 网页或其他来源，直接使用 SMTC 图片
            if (currentTrackKey === newTrackKey) {
              transitionCover(smtcCover, "left");
            }
          }
        } else {
          // 播放状态变化
          const wasPlaying = mediaState.is_playing;
          const isPlaying = payload.is_playing;

          console.log(
            "[播放状态] 变化:",
            wasPlaying ? "播放中" : "已暂停",
            "→",
            isPlaying ? "播放中" : "已暂停",
          );

          mediaState.is_playing = isPlaying;
          mediaState.position_ms = payload.position_ms;
          mediaState.duration_ms = payload.duration_ms;

          // 根据播放状态控制 MV
          if (isPlayingMV && mvUrl) {
            const videoElement = document.querySelector(
              ".mv-player",
            ) as HTMLVideoElement;
            if (videoElement) {
              if (!isPlaying && wasPlaying) {
                // 歌曲暂停，MV 也暂停
                videoElement.pause();
                console.log(
                  "[MV] 暂停 (时间：" +
                    videoElement.currentTime.toFixed(2) +
                    "s, paused=" +
                    videoElement.paused +
                    ")",
                );
              } else if (isPlaying && !wasPlaying) {
                // 歌曲从暂停恢复播放，MV 也恢复播放
                videoElement.play().catch((err) => {
                  console.error("[MV] 恢复播放失败:", err);
                });
                console.log(
                  "[MV] 恢复播放 (时间：" +
                    videoElement.currentTime.toFixed(2) +
                    "s, paused=" +
                    videoElement.paused +
                    ")",
                );
              }
            } else {
              console.warn("[MV] 未找到视频元素");
            }
          }
        }
      }, 50); // 50ms 防抖
    });

    // 进度更新使用 requestAnimationFrame 代替 setInterval
    let lastPosition = 0;
    const updateProgress = () => {
      if (
        mediaState.is_playing &&
        mediaState.duration_ms > 0 &&
        mediaState.position_ms < mediaState.duration_ms
      ) {
        const now = Date.now();
        if (now - lastPosition >= 1000) {
          mediaState.position_ms += 1000;
          lastPosition = now;
        }
      }
      requestAnimationFrame(updateProgress);
    };
    requestAnimationFrame(updateProgress);
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

    // 清理所有事件监听器
    if (eventListeners) {
      eventListeners.forEach((unlisten: () => void) => unlisten());
      eventListeners.length = 0;
    }

    // 清理临时 Canvas
    tempCanvasCache = null;
    newCanvasRef = null;
    oldCanvasRef = null;

    if (savePositionTimeout) clearTimeout(savePositionTimeout);
    if (progressInterval) clearInterval(progressInterval);
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

  // 缓存 Canvas 元素引用，避免重复查询
  let newCanvasRef = $state<HTMLCanvasElement | null>(null);
  let oldCanvasRef = $state<HTMLCanvasElement | null>(null);

  // 事件监听器管理器
  let eventListeners = $state<(() => void)[]>([]);

  // 设置 Canvas 元素引用 - 监听 displayCover 变化确保 Canvas 元素已创建
  $effect(() => {
    if (displayCover) {
      requestAnimationFrame(() => {
        const newCanvas = document.querySelector(
          ".album-art-new",
        ) as HTMLCanvasElement;
        const oldCanvas = document.querySelector(
          ".album-art-old",
        ) as HTMLCanvasElement;

        if (newCanvas && !newCanvasRef) {
          newCanvasRef = newCanvas;
        }
        if (oldCanvas && !oldCanvasRef) {
          oldCanvasRef = oldCanvas;
        }
      });
    }
  });

  // 监听 displayCover 和 enablePixelArt 变化，渲染到 Canvas
  $effect(() => {
    if (displayCover) {
      const renderFunction = enablePixelArt
        ? renderImageToCanvas
        : renderImageToCanvasNormal;

      const newCanvas =
        newCanvasRef ||
        (document.querySelector(".album-art-new") as HTMLCanvasElement);

      if (newCanvas) {
        if (!newCanvasRef) newCanvasRef = newCanvas;
        renderFunction(newCanvas, displayCover);
      }

      if (previousCover) {
        const oldCanvas =
          oldCanvasRef ||
          (document.querySelector(".album-art-old") as HTMLCanvasElement);

        if (oldCanvas) {
          if (!oldCanvasRef) oldCanvasRef = oldCanvas;
          renderFunction(oldCanvas, previousCover);
        }
      }
    }
  });

  // 缓存处理后的图片
  let processedImageCache = $state<{ [key: string]: string }>({}); // 缓存处理后的图片 base64
  let processingQueue = $state<Set<string>>(new Set()); // 正在处理的图片队列

  // 使用后端 API 处理图片（支持像素化）
  async function processImageBackend(
    imageUrl: string,
    enablePixelArt: boolean,
  ): Promise<string> {
    // 如果已经在处理队列中，等待
    if (processingQueue.has(imageUrl)) {
      return new Promise((resolve) => {
        const checkInterval = setInterval(() => {
          if (!processingQueue.has(imageUrl) && processedImageCache[imageUrl]) {
            clearInterval(checkInterval);
            resolve(processedImageCache[imageUrl]);
          }
        }, 50);
      });
    }

    // 如果已有缓存，直接返回
    if (processedImageCache[imageUrl]) {
      return processedImageCache[imageUrl];
    }

    processingQueue.add(imageUrl);

    try {
      // 调用后端 API 处理图片
      const processedBase64 = await invoke<string>("process_image", {
        imagePath: imageUrl,
        enablePixelArt: enablePixelArt,
      });

      // 缓存结果
      processedImageCache[imageUrl] = processedBase64;
      console.log(
        "[图片处理] 后端处理完成:",
        enablePixelArt ? "像素化" : "原图",
      );
      return processedBase64;
    } catch (error) {
      console.error("[图片处理] 后端处理失败:", error);
      // 如果是空图片错误，使用默认占位图
      if (
        error &&
        typeof error === "string" &&
        error.includes("图片数据为空")
      ) {
        console.log("[图片处理] 图片数据为空，使用原图");
        return imageUrl;
      }
      // 失败时返回原图
      return imageUrl;
    } finally {
      processingQueue.delete(imageUrl);
    }
  }

  // 渲染图片到 Canvas（简化版，直接使用后端处理后的 base64）
  async function renderImageToCanvas(
    canvas: HTMLCanvasElement,
    imageUrl: string,
  ) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    try {
      // 使用后端 API 处理图片
      const processedUrl = await processImageBackend(imageUrl, enablePixelArt);

      // 加载处理后的图片
      const img = new Image();
      img.onload = () => {
        // 设置 Canvas 尺寸
        canvas.width = img.width;
        canvas.height = img.height;

        // 绘制图片
        ctx.drawImage(img, 0, 0);
        console.log("[Canvas] 图片已绘制");
      };
      img.onerror = () => {
        console.error("[Canvas] 图片加载失败，使用原图");
        renderImageToCanvasNormal(canvas, imageUrl);
      };
      img.src = processedUrl;
    } catch (error) {
      console.error("[图片处理] 处理错误:", error);
      renderImageToCanvasNormal(canvas, imageUrl);
    }
  }

  // 智能图片质量分析函数
  function analyzeImageQuality(img: HTMLImageElement): number {
    // 基于图片尺寸、清晰度和内容复杂度评估质量
    const width = img.width;
    const height = img.height;

    // 基础质量评分（0-1）
    let qualityScore = Math.min(1, width / 500); // 基于宽度评分

    // 如果图片太小，降低评分
    if (width < 100 || height < 100) {
      qualityScore *= 0.5;
    }

    // 如果图片太大但可能是低质量放大，适当调整
    if (width > 800 && qualityScore > 0.8) {
      qualityScore = 0.8 + (qualityScore - 0.8) * 0.5;
    }

    return Math.max(0.1, Math.min(1, qualityScore));
  }

  // 计算最优像素大小
  function calculateOptimalPixelSize(
    img: HTMLImageElement,
    qualityScore: number,
  ): number {
    // 统一使用固定的6px像素大小
    return 12;
  }

  // 高级像素化渲染（使用调色板量化 + Floyd-Steinberg 抖动）
  function renderCachedImage(
    canvas: HTMLCanvasElement,
    img: HTMLImageElement,
    pixelated: boolean,
  ) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // 设置 Canvas 尺寸与图片一致
    canvas.width = img.width;
    canvas.height = img.height;

    if (pixelated) {
      // 智能像素化算法：根据图片质量和内容决定像素化程度
      const imgQuality = analyzeImageQuality(img);
      const pixelSize = calculateOptimalPixelSize(img, imgQuality);

      const scaledWidth = Math.ceil(img.width / pixelSize);
      const scaledHeight = Math.ceil(img.height / pixelSize);

      // 重用或创建临时 Canvas
      if (
        !tempCanvasCache ||
        tempCanvasCache.width !== scaledWidth ||
        tempCanvasCache.height !== scaledHeight
      ) {
        tempCanvasCache = document.createElement("canvas");
        tempCanvasCache.width = scaledWidth;
        tempCanvasCache.height = scaledHeight;
      }

      const tempCtx = tempCanvasCache.getContext("2d");
      if (!tempCtx) return;

      // 关闭图像平滑处理
      tempCtx.imageSmoothingEnabled = false;
      tempCtx.imageSmoothingQuality = "low";

      // 缩小图片
      tempCtx.drawImage(img, 0, 0, scaledWidth, scaledHeight);

      // 尝试获取像素数据（可能会因为跨域而失败）
      let imageData: ImageData;
      try {
        imageData = tempCtx.getImageData(0, 0, scaledWidth, scaledHeight);
      } catch (e) {
        // 跨域污染，使用简单像素化方案
        console.warn("[像素化] 跨域图片污染，使用简单方案:", e);
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(
          tempCanvasCache,
          0,
          0,
          scaledWidth,
          scaledHeight,
          0,
          0,
          canvas.width,
          canvas.height,
        );
        return;
      }

      const data = imageData.data;

      // 提取全局调色板（使用 NeuQuant 简化版）
      const palette = extractOptimalPalette(data, 32); // 32 种颜色

      // 应用调色板量化 + Floyd-Steinberg 抖动
      applyPaletteWithDithering(data, scaledWidth, scaledHeight, palette);

      // 将处理后的数据放回临时 Canvas
      tempCtx.putImageData(imageData, 0, 0);

      // 清除主 Canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.imageSmoothingEnabled = false;

      // 将处理好的像素画放大绘制回主 Canvas
      ctx.drawImage(
        tempCanvasCache,
        0,
        0,
        scaledWidth,
        scaledHeight,
        0,
        0,
        canvas.width,
        canvas.height,
      );
    } else {
      // 正常渲染高清图
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0);
    }
  }

  // 提取最优全局调色板（简化版 NeuQuant 算法）
  function extractOptimalPalette(
    data: Uint8ClampedArray,
    numColors: number,
  ): Uint8ClampedArray {
    // 统计颜色出现频率
    const colorCount = new Map<string, number>();
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a < 128) continue; // 跳过透明像素

      // 量化颜色以减少键数量
      const qr = Math.round(r / 8) * 8;
      const qg = Math.round(g / 8) * 8;
      const qb = Math.round(b / 8) * 8;
      const key = `${qr},${qg},${qb}`;

      colorCount.set(key, (colorCount.get(key) || 0) + 1);
    }

    // 按频率排序
    const sortedColors = Array.from(colorCount.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, numColors * 2); // 取前 2 倍数量用于后续优化

    // 使用简单的 k-means 聚类思想优化调色板
    const palette = new Uint8ClampedArray(numColors * 4);
    for (let i = 0; i < numColors; i++) {
      if (i < sortedColors.length) {
        const [r, g, b] = sortedColors[i][0].split(",").map(Number);
        palette[i * 4] = r;
        palette[i * 4 + 1] = g;
        palette[i * 4 + 2] = b;
        palette[i * 4 + 3] = 255;
      } else {
        // 填充剩余颜色
        palette[i * 4] = 0;
        palette[i * 4 + 1] = 0;
        palette[i * 4 + 2] = 0;
        palette[i * 4 + 3] = 255;
      }
    }

    return palette;
  }

  // 应用调色板量化 + Floyd-Steinberg 误差扩散抖动
  function applyPaletteWithDithering(
    data: Uint8ClampedArray,
    width: number,
    height: number,
    palette: Uint8ClampedArray,
  ) {
    // 创建误差缓冲区
    const errors = new Float32Array(data.length);

    // 查找最接近的调色板颜色
    function findClosestColor(
      r: number,
      g: number,
      b: number,
    ): [number, number, number] {
      let minDist = Infinity;
      let closestIdx = 0;

      for (let i = 0; i < palette.length / 4; i++) {
        const pr = palette[i * 4];
        const pg = palette[i * 4 + 1];
        const pb = palette[i * 4 + 2];

        // 使用加权欧几里得距离（考虑人眼对绿色更敏感）
        const dist = 2 * (r - pr) ** 2 + 4 * (g - pg) ** 2 + 3 * (b - pb) ** 2;

        if (dist < minDist) {
          minDist = dist;
          closestIdx = i;
        }
      }

      return [
        palette[closestIdx * 4],
        palette[closestIdx * 4 + 1],
        palette[closestIdx * 4 + 2],
      ];
    }

    // 逐像素处理
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = (y * width + x) * 4;

        // 加上累积的误差
        const r = Math.max(0, Math.min(255, data[i] + errors[i]));
        const g = Math.max(0, Math.min(255, data[i + 1] + errors[i + 1]));
        const b = Math.max(0, Math.min(255, data[i + 2] + errors[i + 2]));

        // 找到最接近的调色板颜色
        const [qr, qg, qb] = findClosestColor(r, g, b);

        // 更新像素
        data[i] = qr;
        data[i + 1] = qg;
        data[i + 2] = qb;
        // alpha 通道保持不变

        // 计算量化误差
        const errR = r - qr;
        const errG = g - qg;
        const errB = b - qb;

        // Floyd-Steinberg 误差扩散
        // 右：7/16
        if (x + 1 < width) {
          const ni = (y * width + (x + 1)) * 4;
          errors[ni] += errR * (7 / 16);
          errors[ni + 1] += errG * (7 / 16);
          errors[ni + 2] += errB * (7 / 16);
        }

        // 左下：3/16
        if (x > 0 && y + 1 < height) {
          const ni = ((y + 1) * width + (x - 1)) * 4;
          errors[ni] += errR * (3 / 16);
          errors[ni + 1] += errG * (3 / 16);
          errors[ni + 2] += errB * (3 / 16);
        }

        // 正下：5/16
        if (y + 1 < height) {
          const ni = ((y + 1) * width + x) * 4;
          errors[ni] += errR * (5 / 16);
          errors[ni + 1] += errG * (5 / 16);
          errors[ni + 2] += errB * (5 / 16);
        }

        // 右下：1/16
        if (x + 1 < width && y + 1 < height) {
          const ni = ((y + 1) * width + (x + 1)) * 4;
          errors[ni] += errR * (1 / 16);
          errors[ni + 1] += errG * (1 / 16);
          errors[ni + 2] += errB * (1 / 16);
        }
      }
    }
  }

  // 渲染图片到 Canvas（正常高清，不像素化）
  function renderImageToCanvasNormal(
    canvas: HTMLCanvasElement,
    imageUrl: string,
  ) {
    const img = new Image();
    if (imageUrl.startsWith("http") && !imageUrl.includes("asset.localhost")) {
      img.crossOrigin = "Anonymous";
    }

    img.onload = () => {
      renderCachedImage(canvas, img, false);
    };

    img.onerror = () => {
      console.error("[Canvas] 图片加载失败:", imageUrl);
    };

    img.src = imageUrl;
  }

  async function togglePlay(e: MouseEvent) {
    e.stopPropagation();
    await invoke("control_media", { action: "play_pause" });
    // 不手动更新状态，等待后端的 media-update 事件同步
  }

  async function toggleAlwaysOnTop(e: MouseEvent) {
    e.stopPropagation();
    console.log("[置顶] 按钮被点击，当前状态:", isAlwaysOnTop);
    isAlwaysOnTop = !isAlwaysOnTop;
    const appWindow = getCurrentWindow();
    await appWindow.setAlwaysOnTop(isAlwaysOnTop);
    console.log("[置顶] 设置为:", isAlwaysOnTop);

    // 保存设置
    try {
      await invoke("save_settings", {
        settings: { always_on_top: isAlwaysOnTop },
      });
    } catch (error) {
      console.error("[置顶] 保存设置失败:", error);
    }
  }

  function closeWindow(e: MouseEvent) {
    e.stopPropagation();
    getCurrentWindow().close();
  }

  // 用于拖拽的标题栏区域 - 排除关闭按钮和置顶按钮
  function handleDragBarMousedown(e: MouseEvent) {
    // 如果悬浮窗已锁定，禁止拖拽
    if (isFloatingWindowLocked) {
      console.log("[拖拽] 悬浮窗已锁定，禁止拖拽");
      return;
    }

    const target = e.target as HTMLElement;
    if (
      target.closest(".close-btn-topbar") ||
      target.closest(".pin-btn-topbar")
    ) {
      return; // 如果点击的是关闭按钮或置顶按钮，不拖拽
    }
    getCurrentWindow().startDragging();
  }
</script>

<div
  class="player"
  class:hovered={isHovered}
  class:locked={isFloatingWindowLocked}
  class:pixelated={enablePixelArt}
  role="region"
  aria-label="音乐播放器"
  style:--bg={bgColor}
  style:--bg-gradient={bgGradient}
>
  <div class="bg-solid" style:background={bgGradient}></div>

  <!-- 可拖拽的顶部栏 - 鼠标悬停时滑下（锁定时固定显示） -->
  {#if !isFloatingWindowLocked || isFloatingWindowLocked}
    <div
      class="drag-bar"
      class:locked={isFloatingWindowLocked}
      onmousedown={handleDragBarMousedown}
      role="button"
      aria-label="拖动窗口"
      tabindex="0"
    >
      <button
        class="pin-btn-topbar"
        onclick={toggleAlwaysOnTop}
        aria-label={isAlwaysOnTop ? "取消置顶" : "置顶"}
        class:pinned={isAlwaysOnTop}
      >
        <Pin size={16} strokeWidth={2} />
      </button>
      <div class="drag-handle">
        <div class="drag-dots">
          <div class="drag-dot"></div>
          <div class="drag-dot"></div>
          <div class="drag-dot"></div>
        </div>
      </div>
      <button class="close-btn-topbar" onclick={closeWindow} aria-label="关闭">
        <X size={16} strokeWidth={2} />
      </button>
    </div>
  {/if}

  <div class="album-stage">
    {#if displayCover}
      <div class="album-wrapper">
        <!-- MV 视频播放 -->
        {#if isPlayingMV && mvUrl}
          <video
            class="mv-player"
            src={mvUrl}
            autoplay
            muted
            loop
            playsinline
            preload="auto"
            disablepictureinpicture
            poster=""
            onloadeddata={(e) => {
              const video = e.target as HTMLVideoElement;
              // 确保视频已缓冲足够再播放
              video.play().catch(console.error);
            }}
            onwaiting={() => {
              console.log("[MV] 缓冲中...");
            }}
            onplaying={() => {
              console.log("[MV] 播放中...");
            }}
            onstalled={() => {
              console.log("[MV] 网络卡顿");
            }}
          ></video>
        {/if}
        <!-- 旧图（如果有） -->
        {#if previousCover}
          <canvas
            class="album-art album-art-old"
            class:slide-out={slideDirection}
            draggable="false"
          ></canvas>
        {/if}
        <!-- 新图 -->
        <canvas
          class="album-art album-art-new"
          class:slide-in={slideDirection}
          draggable="false"
        ></canvas>
      </div>
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
    border: 3px solid #000; /* 缩小边框 */
    box-sizing: border-box;
    box-shadow:
      0 12px 48px rgba(0, 0, 0, 0.5),
      0 0 0 0.5px rgba(255, 255, 255, 0.05);

    /* 原有的 clip-path 保留 */
    clip-path: inset(0 round 5px);

    /* 新增：修复 Webview 绝对定位子元素圆角溢出的核心代码 */
    isolation: isolate;
    -webkit-mask-image: -webkit-radial-gradient(white, black);
  }

  .bg-solid {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 60px; /* 填充到歌曲信息层上方 */
    z-index: 1;
    background: var(--bg-gradient);
    transition:
      background 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      top 0.3s cubic-bezier(0.4, 0, 0.2, 1);

    /* 新增：与父元素一致的圆角 */
    border-radius: 5px;
  }

  .player.hovered .bg-solid {
    top: 25px; /* 顶部栏出现时，填充色向下移动 */
  }

  /* 锁定时，即使 hovered 也不下滑 */
  .player.locked .bg-solid {
    top: 0 !important;
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
    justify-content: space-between; /* 两端对齐 */
    padding: 0 4px; /* 左右留一点空间 */
    box-sizing: border-box;
    visibility: hidden; /* 完全隐藏 */
    transform: translateY(-100%);
    transition:
      visibility 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    background: #000000; /* 纯黑色 */
    pointer-events: auto; /* 确保可以接收鼠标事件 */
  }

  .player.hovered .drag-bar {
    visibility: visible;
    transform: translateY(0);
  }

  /* 锁定状态下固定显示顶部栏 */
  .player.locked .drag-bar {
    visibility: visible !important;
    transform: translateY(0) !important;
    opacity: 0.8;
  }

  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1px 12px 7px 12px; /* 上内边距更小，让点更靠上 */
    flex: 1; /* 占据中间空间 */
  }

  .drag-dots {
    display: flex;
    gap: 3px;
    padding: 0;
  }

  .drag-dot {
    width: 3px;
    height: 3px;
    background: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    transition: background 0.2s ease;
  }

  .drag-bar:hover .drag-dot {
    background: rgba(255, 255, 255, 0.8);
  }

  /* 顶部栏置顶按钮 */
  .pin-btn-topbar {
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
    flex-shrink: 0; /* 不被压缩 */
  }

  .pin-btn-topbar:hover {
    color: #fff;
    transform: scale(1.1);
    background: transparent;
  }

  .pin-btn-topbar:active {
    transform: scale(0.9);
    background: transparent;
  }

  .pin-btn-topbar.pinned {
    color: #fff;
    transform: rotate(45deg);
  }

  .pin-btn-topbar.pinned:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  /* 顶部栏关闭按钮 */
  .close-btn-topbar {
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
    flex-shrink: 0; /* 不被压缩 */
  }

  .close-btn-topbar:hover {
    color: #fff;
    transform: scale(1.1);
    background: transparent;
  }

  .close-btn-topbar:active {
    transform: scale(0.9);
    background: transparent;
  }

  /* ==================== 专辑封面 ==================== */
  .album-stage {
    position: absolute;
    top: 25px; /* 留出顶部栏的空间（25px） */
    bottom: 60px; /* 与填充色底部对齐 */
    left: 0;
    right: 0;
    z-index: 3;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px; /* 减小内边距，让图片更大 */
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
      0 8px 32px rgba(0, 0, 0, 0.3),
      0 2px 12px rgba(0, 0, 0, 0.15);
    border-radius: 10px;
    overflow: hidden;
    min-width: 50px;
    min-height: 50px;
    /* 3D 变换容器 */
    transform-style: preserve-3d;
    transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  /* MV 播放器 */
  .mv-player {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 10;
    border-radius: 10px;
  }

  .album-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    aspect-ratio: 1 / 1;
    object-fit: cover;
    display: block;
    border-radius: 10px;
    pointer-events: none; /* 让鼠标事件穿透，不阻挡按钮点击 */
    /* 优化的像素化效果 */
    image-rendering: -webkit-optimize-contrast;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
    image-rendering: pixelated;
    -ms-interpolation-mode: nearest-neighbor;
  }

  /* 像素字体定义 */
  @font-face {
    font-family: "Fusion Pixel Latin";
    src: url("/fonts/fusion-pixel-12px-monospaced-latin.ttf") format("truetype");
    font-weight: normal;
    font-style: normal;
    font-display: swap;
  }

  @font-face {
    font-family: "Fusion Pixel Japanese";
    src: url("/fonts/fusion-pixel-12px-monospaced-ja.ttf") format("truetype");
    font-weight: normal;
    font-style: normal;
    font-display: swap;
  }

  @font-face {
    font-family: "Fusion Pixel Korean";
    src: url("/fonts/fusion-pixel-12px-monospaced-ko.ttf") format("truetype");
    font-weight: normal;
    font-style: normal;
    font-display: swap;
  }

  @font-face {
    font-family: "Fusion Pixel Simplified Chinese";
    src: url("/fonts/fusion-pixel-12px-monospaced-zh_hans.ttf")
      format("truetype");
    font-weight: normal;
    font-style: normal;
    font-display: swap;
  }

  @font-face {
    font-family: "Fusion Pixel Traditional Chinese";
    src: url("/fonts/fusion-pixel-12px-monospaced-zh_hant.ttf")
      format("truetype");
    font-weight: normal;
    font-style: normal;
    font-display: swap;
  }

  /* 通用像素字体栈 */
  :global(.pixel-font) {
    font-family: "Fusion Pixel Simplified Chinese",
      "Fusion Pixel Traditional Chinese", "Fusion Pixel Japanese",
      "Fusion Pixel Korean", "Fusion Pixel Latin", "Courier New",
      "Lucida Console", Monaco, monospace;
    font-weight: bold;
    letter-spacing: 0;
    -webkit-font-smoothing: none;
    -moz-osx-font-smoothing: grayscale;
    font-smooth: never;
    text-rendering: optimizeSpeed;
  }

  /* 旧图在上层，新图在下层 */
  .album-art-old {
    z-index: 2;
    will-change: transform, opacity;
    backface-visibility: hidden;
  }

  .album-art-new {
    z-index: 1;
    will-change: transform, opacity;
    backface-visibility: hidden;
  }

  /* 旧图向左滑出动画 */
  .album-art-old.slide-out {
    animation: slide-out-left 0.4s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  /* 新图从右滑入动画 */
  .album-art-new.slide-in {
    animation: slide-in-from-right 0.4s cubic-bezier(0.4, 0, 0.2, 1);
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
    transition: all 0.3s ease;
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
    transition: all 0.3s ease;
  }

  .player.pixelated .track-title,
  .player.pixelated .track-artist {
    font-family: "Fusion Pixel Simplified Chinese",
      "Fusion Pixel Traditional Chinese", "Fusion Pixel Japanese",
      "Fusion Pixel Korean", "Fusion Pixel Latin", "Courier New",
      "Lucida Console", Monaco, monospace;
    font-weight: bold;
    letter-spacing: 0;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    -webkit-font-smoothing: none;
    -moz-osx-font-smoothing: grayscale;
    font-smooth: never;
    text-rendering: optimizeSpeed;
    /* 优化GPU加速 */
    transform: translateZ(0);
    backface-visibility: hidden;
    perspective: 1000px;
  }

  /* 像素化圆角效果 - 仅移除专辑封面和控制按钮的圆角，保留悬浮窗整体圆角 */
  .player.pixelated .album-art {
    border-radius: 0 !important;
  }

  .player.pixelated .control-mask {
    border-radius: 0 !important;
  }

  .player.pixelated .control-btn {
    border-radius: 0 !important;
  }

  .player.pixelated .close-btn-topbar,
  .player.pixelated .pin-btn-topbar {
    border-radius: 0 !important;
  }

  .player.pixelated .top-bar {
    border-radius: 0 !important;
  }

  .player.pixelated .album-placeholder {
    border-radius: 0 !important;
  }

  .player.pixelated .progress-bar {
    border-radius: 0 !important;
  }

  .player.pixelated .progress-fill {
    border-radius: 0 !important;
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
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0),
      rgba(0, 0, 0, 0.6)
    );
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

  /* 专辑图片淡入动画 */
  @keyframes fade-enter {
    0% {
      opacity: 0;
    }
    100% {
      opacity: 1;
    }
  }

  /* 向左滑出（下一首） */
  @keyframes slide-out-left {
    0% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateX(-100%) scale(0.9);
      opacity: 0;
    }
  }

  /* 从右滑入（下一首） */
  @keyframes slide-in-from-right {
    0% {
      transform: translateX(100%) scale(0.9);
      opacity: 0;
    }
    100% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
  }

  /* 向右滑出（上一首） */
  @keyframes slide-out-right {
    0% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateX(100%) scale(0.9);
      opacity: 0;
    }
  }

  /* 从左滑入（上一首） */
  @keyframes slide-in-from-left {
    0% {
      transform: translateX(-100%) scale(0.9);
      opacity: 0;
    }
    100% {
      transform: translateX(0) scale(1);
      opacity: 1;
    }
  }

  /* 默认进入动画 */
  @keyframes slide-enter {
    0% {
      transform: scale(0.95);
      opacity: 0;
    }
    100% {
      transform: scale(1);
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
