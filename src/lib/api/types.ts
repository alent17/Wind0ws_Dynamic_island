import { normalizeWidgets, type IslandWidgets } from "$lib/widgetLayout";

export type AppAudioSession = { processId: number; deviceId: string; name: string; icon?: string | null; volumePercent: number; muted: boolean; systemSounds: boolean };
export type IslandStyle = "floating" | "edge";
export type IslandEdge = "top" | "right" | "bottom" | "left";
export type SpectrumMode = "realtime" | "random";
export type NeteaseProgressMode = "real" | "estimate";
export type AppLanguage = "system" | "zh-CN" | "en" | "ja";

export interface AppSettings {
  islandWidgets: IslandWidgets;
  islandWidgetsEnabled: boolean;
  neteaseProgressMode: NeteaseProgressMode;
  neteaseConnectionMode: "standard" | "enhanced";
  neteaseAutoConnect: boolean;
  neteaseExecutablePath: string | null;
  islandStyle: IslandStyle;
  islandEdge: IslandEdge;
  islandEdgePosition: number;
  collapsedEdgeShoulderRadius: number;
  expandedEdgeShoulderRadius: number;
  floatingExpandedLength: number;
  edgeExpandedLength: number;
  floatingCompactLength: number;
  edgeCompactLength: number;
  language: AppLanguage;
  fontId: FontId;
  captureHideOnScreenshot: boolean;
  captureHideOnRecording: boolean;
  captureHideOnFullscreen: boolean;
  captureHideOnScreenShare: boolean;
  showSpectrum: boolean;
  spectrumMode: SpectrumMode;
  enableAnimations: boolean;
  windowOpacity: number;
  alwaysOnTop: boolean;
  keepIslandExpanded: boolean;
  floatingWindowAlwaysOnTop: boolean;
  wheelVolumeControl: boolean;
  volumeProtectionEnabled: boolean;
  volumeProtectionLimit: number;
  reduceAnimations: boolean;
  showDebugInfo: boolean;
  logLevel: string;
  monitorIndex: number;
  playerWeights: Record<string, number>;
  playerOrderIds: string[];
  playerSourceOverrides: Record<string, string>;
  selectedPlayerIds: string[] | null;
  idleContentEnabled: boolean;
  idleRotationSeconds: number;
  idleItems: IdleContentItem[];
  weatherLocation: WeatherLocation | null;
  floatingWindowX: number | null;
  floatingWindowY: number | null;
  floatingWindowWidth: number | null;
  floatingWindowHeight: number | null;
  showSettingsTool: boolean;
  showFloatingTool: boolean;
  showVolumeTool: boolean;
  showTimerTool: boolean;
  showHideTool: boolean;
  showClockTool: boolean;
  showWeatherTool: boolean;
  showCustomFunctionPanel: boolean;
  enableMvPlayback: boolean;
  lockFloatingWindow: boolean;
  enableHdCover: boolean;
  floatingCircularAlbum: boolean;
  enablePixelArt: boolean;
  enableHalftone: boolean;
  cacheDirectory: string | null;
  autoStart: boolean;
  hideSettingsButton: boolean;
  hideMonitorSelector: boolean;
  hideFloatingWindow: boolean;
  floatingExpandedCornerRadius: number;
  edgeExpandedCornerRadius: number;
  alwaysShowTopBar: boolean;
  clockTimeZone: string;
}

export type FontId = "misans";
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
  weatherForecast: WeatherForecastDay[];
}
export interface WeatherSnapshot {
  weatherTemperature: number | null;
  weatherCode: number | null;
  weatherUpdatedAt: number | null;
  weatherForecast: WeatherForecastDay[];
}
export interface WeatherForecastDay {
  date: string;
  weatherCode: number;
  temperatureMax: number;
  temperatureMin: number;
}
export interface WeatherLocationCandidate extends WeatherLocation { country: string; admin1: string; admin2: string; }

export type AppPreferences = AppSettings;

export interface MediaFeature { source: "smtc" | "cdp" | "estimate" | "unavailable"; readable: boolean; controllable: boolean; }

export interface MediaCapabilities {
  previous: boolean;
  playPause: boolean;
  next: boolean;
  seek: boolean;
  shuffle: boolean;
  repeat: boolean;
}

export interface MediaState {
  artworkResolved?: boolean;
  /** Additive actor-owned native snapshot order; absent on legacy producers. */
  nativeEpoch?: string;
  /** Integer <= Number.MAX_SAFE_INTEGER, monotonic within nativeEpoch. */
  nativeRevision?: number;
  /** Actor-owned generation incremented when the selected track identity changes. */
  nativeTrackGeneration?: number;
  title: string;
  artist: string;
  albumArt: string;
  isPlaying: boolean;
  positionMs: number;
  durationMs: number;
  progressKnown?: boolean;
  neteaseProgressAvailable?: boolean;
  neteaseFeatures?: { progress: MediaFeature; like: MediaFeature; mode: MediaFeature };
  lastUpdatedTimestamp: number;
  source: string;
  sourceDisplay: string;
  capabilities?: MediaCapabilities;
  shuffleActive?: boolean;
  repeatMode?: "none" | "track" | "list";
  neteasePlaybackMode?: "sequential" | "repeat_list" | "repeat_one" | "shuffle";
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

export interface AudioDeviceInfo {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface SystemAudioState {
  volumePercent: number;
  muted: boolean;
  deviceId: string;
  deviceName: string;
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
  islandWidgets: { left: [], right: [] },
  islandWidgetsEnabled: true,
  neteaseProgressMode: "real",
  neteaseConnectionMode: "enhanced",
  neteaseAutoConnect: false,
  neteaseExecutablePath: null,
  islandStyle: "floating",
  islandEdge: "top",
  islandEdgePosition: 50,
  collapsedEdgeShoulderRadius: 8,
  expandedEdgeShoulderRadius: 32,
  floatingExpandedLength: 600,
  edgeExpandedLength: 600,
  floatingCompactLength: 80,
  edgeCompactLength: 80,
  language: "system",
  fontId: "misans",
  captureHideOnScreenshot: true,
  captureHideOnRecording: true,
  captureHideOnFullscreen: true,
  captureHideOnScreenShare: true,
  showSpectrum: true,
  spectrumMode: "realtime",
  enableAnimations: true,
  windowOpacity: 255,
  alwaysOnTop: true,
  keepIslandExpanded: false,
  floatingWindowAlwaysOnTop: true,
  wheelVolumeControl: false,
  volumeProtectionEnabled: false,
  volumeProtectionLimit: 80,
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
  playerOrderIds: [],
  playerSourceOverrides: {},
  selectedPlayerIds: null,
  idleContentEnabled: true,
  idleRotationSeconds: 5,
  idleItems: ["clock", "weather"].map((kind) => ({
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
  showSettingsTool: true,
  showFloatingTool: true,
  showVolumeTool: true,
  showTimerTool: true,
  showHideTool: true,
  showClockTool: true,
  showWeatherTool: true,
  showCustomFunctionPanel: true,
  enableMvPlayback: true,
  lockFloatingWindow: false,
  enableHdCover: true,
  floatingCircularAlbum: false,
  enablePixelArt: false,
  enableHalftone: false,
  cacheDirectory: null,
  autoStart: false,
  hideSettingsButton: false,
  hideMonitorSelector: false,
  hideFloatingWindow: false,
  floatingExpandedCornerRadius: 45,
  edgeExpandedCornerRadius: 45,
  alwaysShowTopBar: true,
  clockTimeZone: "system",
};

export const DEFAULT_PREFERENCES = DEFAULT_SETTINGS;

export function normalizeAppSettings(value: Partial<AppSettings>): AppSettings {
  const legacy = value as Partial<AppSettings> & { compactLength?: number; expandedCornerRadius?: number };
  const legacyCompactLength = Number(legacy.compactLength ?? 80);
  const legacyExpandedRadius = Number(legacy.expandedCornerRadius ?? 45);
  const merged = { ...DEFAULT_SETTINGS, ...value } as AppSettings;
  if (merged.neteaseConnectionMode !== "standard" && merged.neteaseConnectionMode !== "enhanced") merged.neteaseConnectionMode = DEFAULT_SETTINGS.neteaseConnectionMode;
  merged.islandWidgets = normalizeWidgets(value.islandWidgets);
  merged.playerSourceOverrides = Object.fromEntries(Object.entries(value.playerSourceOverrides ?? {}).filter(([id, source]) => id.trim().length > 0 && id.length <= 1024 && source === "netease"));
  merged.volumeProtectionLimit = Math.min(100, Math.max(40, Number.isFinite(Number(value.volumeProtectionLimit)) ? Math.round(Number(value.volumeProtectionLimit)) : 80));
  for (const key of ["floatingExpandedLength", "edgeExpandedLength"] as const) {
    const length = Number(value[key] ?? 600);
    merged[key] = Math.min(900, Math.max(480, Number.isFinite(length) ? Math.round(length) : 600));
  }
  merged.floatingCompactLength = Math.min(300, Math.max(80, Number(value.floatingCompactLength ?? legacyCompactLength)));
  merged.edgeCompactLength = Math.min(300, Math.max(80, Number(value.edgeCompactLength ?? legacyCompactLength)));
  merged.floatingExpandedCornerRadius = Math.min(80, Math.max(0, Number(value.floatingExpandedCornerRadius ?? legacyExpandedRadius)));
  merged.edgeExpandedCornerRadius = Math.min(80, Math.max(0, Number(value.edgeExpandedCornerRadius ?? legacyExpandedRadius)));
  if (merged.neteaseProgressMode !== "real" && merged.neteaseProgressMode !== "estimate") {
    merged.neteaseProgressMode = DEFAULT_SETTINGS.neteaseProgressMode;
  }
  return merged;
}

export const PLAYER_NAMES: Record<string, string> = {
  netease: "网易云音乐",
  spotify: "Spotify",
  bilibili: "Bilibili",
  qqmusic: "QQ 音乐",
  apple: "Apple Music",
  generic: "其他播放器",
};
