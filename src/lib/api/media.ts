import { Channel, invoke } from "@tauri-apps/api/core";
import type { MediaSessionInfo, MediaState, NeteaseSong, ResolvedCover } from "./types";

const commandBasis = (media?: MediaState) => media ? {
  nativeEpoch: media.nativeEpoch ?? "", nativeTrackGeneration: media.nativeTrackGeneration ?? 0, sourceDisplay: media.sourceDisplay,
} : undefined;

export type NeteaseSetupStage = "waiting" | "detecting" | "configuring" | "waitingPlayer" | "binding" | "saving";
export interface NeteaseConnection {
  connected: boolean; configured: boolean; executablePath: string; shortcutPath: string;
  boundSessionId: string | null; bindingFailed: boolean; launchEntries: number; restartRequired?: boolean; pendingStartMenuEntries?: number;
}

export const mediaApi = {
  neteaseSongLike: (title: string, artist: string, expected?: NeteaseLikeState, liked?: boolean) => invoke<NeteaseLikeState>("netease_song_like", { title, artist, trackId: expected?.trackId, playId: expected?.playId, liked }),
  async setupNeteaseConnection(restart = false, bind = false, configureOnly = false, onProgress?: (stage: NeteaseSetupStage) => void, repairStartMenu = false): Promise<NeteaseConnection> {
    const progress = onProgress ? new Channel<NeteaseSetupStage>(onProgress) : undefined;
    try {
      return await invoke<NeteaseConnection>("setup_netease_connection", { restart, bind: restart || bind, configureOnly, progress, ...(repairStartMenu ? { repairStartMenu: true } : {}) });
    } finally {
      // Rust closes the channel on completion. Ignore any progress still in
      // flight so it cannot overwrite the final result in the UI.
      if (progress) progress.onmessage = () => {};
    }
  },
  async getMediaInfo(): Promise<MediaState> {
    return invoke<MediaState>("get_media_info_cmd");
  },
  listMediaSessions: () => invoke<MediaSessionInfo[]>("list_media_sessions"),

  async controlMedia(action: "play_pause" | "next" | "prev"): Promise<void> {
    return invoke("control_media", { action });
  },

  seekMedia: (
    positionMs: number,
    basis: { nativeEpoch: string; nativeTrackGeneration: number; title: string; artist: string },
  ) => invoke<{ trackId: string; positionMs: number; durationMs: number } | null>("seek_media", {
    positionMs: Math.round(positionMs),
    expectedNativeEpoch: basis.nativeEpoch,
    expectedTrackGeneration: basis.nativeTrackGeneration,
    expectedTitle: basis.title,
    expectedArtist: basis.artist,
  }),
  toggleShuffle: (media?: MediaState) => invoke<void>("toggle_shuffle", { expected: commandBasis(media) }),
  cycleRepeat: (media?: MediaState) => invoke<void>("cycle_repeat", { expected: commandBasis(media) }),
  getNeteasePlaybackMode: () => invoke<NeteasePlaybackMode | null>("get_netease_playback_mode"),
  cycleNeteasePlaybackMode: (media?: MediaState) => invoke<NeteasePlaybackMode>("cycle_netease_playback_mode", { expected: commandBasis(media) }),

  async getNeteaseSongInfo(songName: string, artist: string): Promise<NeteaseSong | null> {
    return invoke<NeteaseSong | null>("get_netease_song_info_cmd", { songName, artist });
  },

  resolveHdCover: (title: string, artist: string, source: string) =>
    invoke<ResolvedCover | null>("resolve_hd_cover", { title, artist, source }),

  async getNeteaseMvUrl(mvId: number): Promise<string | null> {
    return invoke<string | null>("get_netease_mv_url_cmd", { mvId });
  },

  async extractDominantColor(imagePath: string): Promise<[number, number, number]> {
    return invoke<[number, number, number]>("extract_dominant_color", { imagePath });
  },

};

export type NeteasePlaybackMode = "sequential" | "repeat_list" | "repeat_one" | "shuffle";
export interface NeteaseLikeState { trackId: string; playId: string; title: string; artist: string; liked: boolean }
