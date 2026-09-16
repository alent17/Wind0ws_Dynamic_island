import { invoke } from "@tauri-apps/api/core";
import type { IdleSnapshot, WeatherLocationCandidate } from "./types";

export const idleApi = {
  getSnapshot: () => invoke<IdleSnapshot>("get_idle_snapshot"),
  searchLocations: (query: string, language = "zh") => invoke<WeatherLocationCandidate[]>("search_weather_locations", { query, language }),
};
