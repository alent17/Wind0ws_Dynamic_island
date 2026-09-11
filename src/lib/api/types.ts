export type IslandStyle = "floating" | "edge";
export type IslandEdge = "top" | "right" | "bottom" | "left";
export type SpectrumMode = "realtime" | "random";

export interface AppSettings {
  islandTheme: string;
  islandStyle: IslandStyle;
  islandEdge: IslandEdge;
  islandEdgePosition: number;
  edgeShoulderRadius: number;
  compactLength: number;
  fontId: FontId;
  autoHide: boolean;
  showSpectrum: boolean;
  spectrumMode: SpectrumMode;
  enableAnimations: boolean;
  windowOpacity: number;
  alwaysOnTop: boolean;
  hardwareAcceleration: boolean;
  reduceAnimations: boolean;
  showDebugInfo: boolean;
  logLevel: string;
  monitorIndex: number;
  playerWeights: Record<string, number>;
  selectedPlayerIds: string[] | null;
  idleContentEnabled: boolean;
  idleRotationSeconds: number;
  idleItems: IdleContentItem[];
  weatherLocation: WeatherLocation | null;
  floatingWindowX: number | null;
  floatingWindowY: number | null;
  floatingWindowWidth: number | null;
  floatingWindowHeight: number | null;
  enableMvPlayback: boolean;
  lockFloatingWindow: boolean;
  enableHdCover: boolean;
  enablePixelArt: boolean;
  enableHalftone: boolean;
  cacheDirectory: string | null;
  autoStart: boolean;
  hideSettingsButton: boolean;
  hideMonitorSelector: boolean;
  hideFloatingWindow: boolean;
  expandedCornerRadius: number;
  alwaysShowTopBar: boolean;
}

export type FontId = "system" | "misans" | "source-han-serif-cn-bold" | "alibaba-puhuiti-heavy";
export type IdleContentKind = "clock" | "date" | "weather" | "network" | "cpu" | "memory" | "battery" | "custom";
export interface IdleContentItem { id: string; kind: IdleContentKind; enabled: boolean; text: string; }
export interface WeatherLocation { name: string; latitude: number; longitude: number; }
export interface MediaSessionInfo { id: string; displayName: string; source: string; isPlaying: boolean; }
export interface IdleSnapshot {
  cpuPercent: number;
  memoryPercent: number;
  uploadBytesPerSecond: number;
  downloadBytesPerSecond: number;
  batteryPercent: number | null;
  batteryCharging: boolean | null;
  weatherTemperature: number | null;
  weatherCode: number | null;
  weatherUpdatedAt: number | null;
}
export interface WeatherLocationCandidate extends WeatherLocation { country: string; admin1: string; admin2: string; }

export type AppPreferences = AppSettings;

export interface MediaCapabilities {
  previous: boolean;
  playPause: boolean;
  next: boolean;
  seek: boolean;
  shuffle: boolean;
  repeat: boolean;
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
  capabilities?: MediaCapabilities;
  shuffleActive?: boolean;
  repeatMode?: "none" | "track" | "list";
}

export interface NeteaseSong {
  duration?: number;
  albumPic?: string;
  mvId?: number;
  mvUrl?: string;
}

export interface ResolvedCover {
  url: string;
  provider: "netease" | "apple";
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
  x: number;
  y: number;
  workX: number;
  workY: number;
  workWidth: number;
  workHeight: number;
  scaleFactor: number;
}

export interface AppError {
  code: number;
  message: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  islandTheme: "original",
  islandStyle: "floating",
  islandEdge: "top",
  islandEdgePosition: 50,
  edgeShoulderRadius: 8,
  compactLength: 80,
  fontId: "system",
  autoHide: true,
  showSpectrum: true,
  spectrumMode: "realtime",
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
  selectedPlayerIds: null,
  idleContentEnabled: true,
  idleRotationSeconds: 5,
  idleItems: ["clock", "date", "weather", "network", "cpu", "memory", "battery"].map((kind) => ({
    id: kind,
    kind: kind as IdleContentKind,
    enabled: true,
    text: "",
  })),
  weatherLocation: null,
  floatingWindowX: null,
  floatingWindowY: null,
  floatingWindowWidth: null,
  floatingWindowHeight: null,
  enableMvPlayback: true,
  lockFloatingWindow: false,
  enableHdCover: true,
  enablePixelArt: false,
  enableHalftone: false,
  cacheDirectory: null,
  autoStart: false,
  hideSettingsButton: false,
  hideMonitorSelector: false,
  hideFloatingWindow: false,
  expandedCornerRadius: 45,
  alwaysShowTopBar: true,
};

export const DEFAULT_PREFERENCES = DEFAULT_SETTINGS;

export const PLAYER_NAMES: Record<string, string> = {
  netease: "网易云音乐",
  spotify: "Spotify",
  bilibili: "Bilibili",
  qqmusic: "QQ 音乐",
  apple: "Apple Music",
  generic: "其他播放器",
};
