import { invoke } from "@tauri-apps/api/core";
import type { IdleSnapshot, WeatherLocationCandidate } from "./types";

let snapshot: { key: string; at: number; value: IdleSnapshot } | undefined;
let pending: { key: string; request: Promise<IdleSnapshot> } | undefined;
export const idleApi = {
  getSnapshot: (key = "") => {
    if (snapshot?.key === key && Date.now() - snapshot.at < 30_000) return Promise.resolve(snapshot.value);
    if (pending?.key === key) return pending.request;
    const request = invoke<IdleSnapshot>("get_idle_snapshot").then(value => {
      if (pending?.request === request) snapshot = {key, at: Date.now(), value};
      return value;
    }).finally(() => { if (pending?.request === request) pending = undefined; });
    pending = {key, request};
    return request;
  },
  searchLocations: (query: string, language = "zh") => invoke<WeatherLocationCandidate[]>("search_weather_locations", { query, language }),
};
