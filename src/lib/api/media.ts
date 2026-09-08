import { invoke } from "@tauri-apps/api/core";
import type { MediaState, NeteaseSong } from "./types";

export const mediaApi = {
  async getMediaInfo(): Promise<MediaState> {
    return invoke<MediaState>("get_media_info_cmd");
  },

  async controlMedia(action: "play_pause" | "next" | "prev"): Promise<void> {
    return invoke("control_media", { action });
  },

  seekMedia: (positionMs: number) => invoke<void>("seek_media", { positionMs: Math.round(positionMs) }),
  toggleShuffle: () => invoke<void>("toggle_shuffle"),
  cycleRepeat: () => invoke<void>("cycle_repeat"),

  async getNeteaseSongInfo(songName: string, artist: string): Promise<NeteaseSong | null> {
    return invoke<NeteaseSong | null>("get_netease_song_info_cmd", { songName, artist });
  },

  async getNeteaseDuration(): Promise<number | null> {
    return invoke<number | null>("get_netease_duration_cmd");
  },

  async getNeteaseMvUrl(mvId: number): Promise<string | null> {
    return invoke<string | null>("get_netease_mv_url_cmd", { mvId });
  },

  async extractDominantColor(imagePath: string): Promise<[number, number, number]> {
    return invoke<[number, number, number]>("extract_dominant_color", { imagePath });
  },

};
