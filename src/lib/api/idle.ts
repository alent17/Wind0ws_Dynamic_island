import { invoke } from "@tauri-apps/api/core";
import type { IdleSnapshot, WeatherLocationCandidate, WeatherSnapshot } from "./types";

let snapshot: { key: string; at: number; value: IdleSnapshot } | undefined;
let pending: { key: string; request: Promise<IdleSnapshot> } | undefined;
const MAX_WEATHER_PENDING_KEYS = 4;
let weatherSnapshot: { key: string; at: number; value: WeatherSnapshot } | undefined;
const weatherPending = new Map<string, Promise<WeatherSnapshot>>();

function getWeatherSnapshot(key: string): Promise<WeatherSnapshot> {
  if (weatherSnapshot?.key === key && Date.now() - weatherSnapshot.at < 30_000) {
    return Promise.resolve(weatherSnapshot.value);
  }
  const existing = weatherPending.get(key);
  if (existing) return existing;
  if (weatherPending.size >= MAX_WEATHER_PENDING_KEYS) {
    return Promise.reject(new Error("Too many weather locations are pending"));
  }

  const request = invoke<WeatherSnapshot>("get_weather_snapshot").then(value => {
    if (weatherPending.get(key) === request) weatherSnapshot = {key, at: Date.now(), value};
    return value;
  }).finally(() => {
    if (weatherPending.get(key) === request) weatherPending.delete(key);
  });
  weatherPending.set(key, request);
  return request;
}

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
  getWeatherSnapshot,
  searchLocations: (query: string, language = "zh") => invoke<WeatherLocationCandidate[]>("search_weather_locations", { query, language }),
};
