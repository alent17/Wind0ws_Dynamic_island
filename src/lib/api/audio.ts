import { invoke } from "@tauri-apps/api/core";
import type { AppAudioSession, AudioDeviceInfo, SystemAudioState } from "./types";

export const audioApi = {
  getState: () => invoke<SystemAudioState>("get_system_audio_state"),
  listDevices: () => invoke<AudioDeviceInfo[]>("list_audio_output_devices"),
  listAppSessions: () => invoke<AppAudioSession[]>("list_app_audio_sessions"),
  setAppVolume: (session: AppAudioSession, volumePercent: number | null, muted: boolean) => invoke<void>("set_app_audio_volume", {
    deviceId: session.deviceId, processId: session.processId, systemSounds: session.systemSounds, volumePercent: volumePercent === null ? null : Math.round(volumePercent), muted,
  }),
  setVolume: (volumePercent: number) =>
    invoke<void>("set_system_volume", { volumePercent: Math.round(volumePercent) }),
  setDefaultDevice: (deviceId: string) =>
    invoke<void>("set_default_audio_output", { deviceId }),
};
