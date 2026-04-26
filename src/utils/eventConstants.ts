// 统一事件名称常量，避免硬编码字符串
export const Events = {
  // 媒体相关
  MEDIA_UPDATE: "media-update",
  AUDIO_SPECTRUM: "audio-spectrum",

  // 设置相关
  SETTINGS_UPDATED: "settings-updated",
  SETTINGS_CHANGED: "settings-changed",
  THEME_CHANGED: "theme-changed",
  CORNER_RADIUS_CHANGED: "corner-radius-changed",

  // 悬浮窗相关
  FLOATING_WINDOW_CLOSED: "floating-window-closed",
  LOCK_FLOATING_WINDOW_CHANGED: "lock-floating-window-changed",
  ALWAYS_ON_TOP_CHANGED: "always-on-top-changed",

  // 封面效果相关
  HD_COVER_CHANGED: "hd-cover-changed",
  PIXEL_ART_CHANGED: "pixel-art-changed",
  HALFTONE_CHANGED: "halftone-changed",

  // MV 相关
  MV_PLAYBACK_CHANGED: "mv-playback-changed",
} as const;

// 事件优先级
export const EventPriority = {
  HIGH: 0,    // 立即处理：用户交互、状态变更
  NORMAL: 1,  // 正常处理：设置更新
  LOW: 2,     // 可节流：频谱、进度
} as const;

// 需要节流的事件配置
export const ThrottledEvents = {
  [Events.AUDIO_SPECTRUM]: { interval: 50, priority: EventPriority.LOW },
  [Events.MEDIA_UPDATE]: { interval: 500, priority: EventPriority.NORMAL },
} as const;
