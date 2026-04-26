export interface AppSettings {
  islandTheme: string;
  autoHide: boolean;
  showSpectrum: boolean;
  enableAnimations: boolean;
  windowOpacity: number;
  alwaysOnTop: boolean;
  hardwareAcceleration: boolean;
  reduceAnimations: boolean;
  showDebugInfo: boolean;
  logLevel: string;
  monitorIndex: number;
  playerWeights: Record<string, number>;
  floatingWindowX: number | null;
  floatingWindowY: number | null;
  floatingWindowWidth: number | null;
  floatingWindowHeight: number | null;
  enableMvPlayback: boolean;
  lockFloatingWindow: boolean;
  enableHdCover: boolean;
  enablePixelArt: boolean;
  cacheDirectory: string | null;
  autoStart: boolean;
  hideSettingsButton: boolean;
  hideMonitorSelector: boolean;
  hideFloatingWindow: boolean;
  expandedCornerRadius: number;
}

export interface MediaState {
  title: string;
  artist: string;
  albumArt: string;
  isPlaying: boolean;
  positionMs: number;
  durationMs: number;
  lastUpdatedTimestamp: number;
  source: string;
  sourceDisplay: string;
}

export interface NeteaseSong {
  duration?: number;
  albumPic?: string;
  mvId?: number;
  mvUrl?: string;
}

export interface CacheStats {
  totalSizeMb: number;
  totalFiles: number;
  mvCount: number;
  coverCount: number;
  cacheDirectory: string;
}

export interface MonitorInfo {
  index: number;
  name: string;
  width: number;
  height: number;
  isPrimary: boolean;
}

export interface AppError {
  code: number;
  message: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  islandTheme: "original",
  autoHide: true,
  showSpectrum: true,
  enableAnimations: true,
  windowOpacity: 255,
  alwaysOnTop: true,
  hardwareAcceleration: true,
  reduceAnimations: false,
  showDebugInfo: false,
  logLevel: "Info",
  monitorIndex: 0,
  playerWeights: {
    netease: 50,
    spotify: 50,
    bilibili: 50,
    qqmusic: 50,
    apple: 50,
    generic: 10,
  },
  floatingWindowX: null,
  floatingWindowY: null,
  floatingWindowWidth: null,
  floatingWindowHeight: null,
  enableMvPlayback: false,
  lockFloatingWindow: false,
  enableHdCover: true,
  enablePixelArt: false,
  cacheDirectory: null,
  autoStart: false,
  hideSettingsButton: false,
  hideMonitorSelector: false,
  hideFloatingWindow: false,
  expandedCornerRadius: 16,
};

export const PLAYER_NAMES: Record<string, string> = {
  netease: "网易云音乐",
  spotify: "Spotify",
  bilibili: "Bilibili",
  qqmusic: "QQ 音乐",
  apple: "Apple Music",
  generic: "其他播放器",
};
