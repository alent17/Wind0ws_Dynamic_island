import { invoke } from "@tauri-apps/api/core";
import type { CacheStats } from "./types";

export const cacheApi = {
  async clearCache(): Promise<void> {
    return invoke("clear_cache");
  },

  async getCacheStats(): Promise<CacheStats> {
    return invoke<CacheStats>("get_cache_stats");
  },

  async getCacheDirectory(): Promise<string> {
    return invoke<string>("get_cache_directory");
  },

  async setCacheDirectory(newPath: string): Promise<void> {
    return invoke("set_cache_directory", { newPath });
  },

  async pickCacheDirectory(): Promise<string | null> {
    return invoke<string | null>("pick_cache_directory");
  },

  async getCachedMedia(url: string): Promise<string | null> {
    return invoke<string | null>("get_cached_media", { url });
  },

  async downloadAndCache(url: string, contentType: string): Promise<string> {
    return invoke<string>("download_and_cache", { url, contentType });
  },
};
