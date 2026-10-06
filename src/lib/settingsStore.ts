import { writable, get } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Events } from "../utils/eventConstants";
import { settingsApi } from "$lib/api/settings";
import { DEFAULT_SETTINGS, type AppSettings } from "$lib/api/types";

export const settingsStore = writable<AppSettings>(DEFAULT_SETTINGS);

let generation = 0;
let eventRevision = 0;
let unlistenSettings: UnlistenFn | undefined;

/**
 * Own the settings snapshot for this webview. Subscribe before fetching the
 * initial snapshot so a concurrent update cannot be overwritten by stale data.
 */
export async function connectSettingsStore(
  normalize: (settings: Partial<AppSettings>) => AppSettings,
): Promise<{ settings: AppSettings; disconnect: () => void }> {
  const epoch = ++generation;

  try {
    const unlisten = await listen<Partial<AppSettings>>(Events.SETTINGS_UPDATED, ({ payload }) => {
      if (epoch !== generation || !payload) return;
      eventRevision += 1;
      publish(normalize(payload));
    });

    if (epoch !== generation) {
      unlisten();
      return { settings: get(settingsStore), disconnect: () => {} };
    }
    unlistenSettings?.();
    unlistenSettings = unlisten;

    const revisionAtRequest = eventRevision;
    await refreshSettingsStore(normalize, () => epoch === generation && revisionAtRequest === eventRevision);

    return {
      settings: get(settingsStore),
      disconnect: () => {
        if (epoch !== generation) return;
        generation += 1;
        unlistenSettings?.();
        unlistenSettings = undefined;
      },
    };
  } catch (error) {
    console.error("[设置] 注册同步失败:", error);
    return { settings: get(settingsStore), disconnect: () => {} };
  }
}

export async function refreshSettingsStore(
  normalize: (settings: Partial<AppSettings>) => AppSettings,
  shouldPublish: () => boolean = () => true,
): Promise<AppSettings> {
  try {
    const next = normalize(await settingsApi.getSettings());
    if (shouldPublish()) publish(next);
  } catch (error) {
    console.error("[设置] 读取失败:", error);
  }
  return get(settingsStore);
}

function publish(next: AppSettings): void {
  const current = get(settingsStore);
  if (JSON.stringify(current) === JSON.stringify(next)) return;
  settingsStore.set(next);
}
