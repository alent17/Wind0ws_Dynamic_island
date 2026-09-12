import { invoke } from "@tauri-apps/api/core";
import type { AudioDeviceInfo, SystemAudioState } from "./types";

export const audioApi = {
  getState: () => invoke<SystemAudioState>("get_system_audio_state"),
  listDevices: () => invoke<AudioDeviceInfo[]>("list_audio_output_devices"),
  setVolume: (volumePercent: number) =>
    invoke<void>("set_system_volume", { volumePercent: Math.round(volumePercent) }),
  setDefaultDevice: (deviceId: string) =>
    invoke<void>("set_default_audio_output", { deviceId }),
};
