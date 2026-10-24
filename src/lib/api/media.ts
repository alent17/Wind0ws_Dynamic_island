import { invoke } from "@tauri-apps/api/core";
import type { MediaSessionInfo, MediaState, NeteaseSong, ResolvedCover } from "./types";

export const mediaApi = {
  neteaseSongLike: (title: string, artist: string, expected?: NeteaseLikeState, liked?: boolean) => invoke<NeteaseLikeState>("netease_song_like", { title, artist, trackId: expected?.trackId, playId: expected?.playId, liked }),
  setupNeteaseConnection: (restart = false, bind = false) => invoke<{ connected: boolean; configured: boolean; executablePath: string; shortcutPath: string; boundSessionId: string | null; bindingFailed: boolean }>("setup_netease_connection", { restart, bind: restart || bind }),
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
  toggleShuffle: () => invoke<void>("toggle_shuffle"),
  cycleRepeat: () => invoke<void>("cycle_repeat"),
  getNeteasePlaybackMode: () => invoke<NeteasePlaybackMode | null>("get_netease_playback_mode"),
  cycleNeteasePlaybackMode: () => invoke<NeteasePlaybackMode>("cycle_netease_playback_mode"),

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
