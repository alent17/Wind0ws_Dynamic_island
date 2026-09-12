use crate::error::AppResult;
use crate::models::{AudioDeviceInfo, SystemAudioState};
use crate::services;

#[tauri::command]
pub fn get_system_audio_state() -> AppResult<SystemAudioState> {
    services::get_system_audio_state()
}

#[tauri::command]
pub fn list_audio_output_devices() -> AppResult<Vec<AudioDeviceInfo>> {
    services::list_audio_output_devices()
}

#[tauri::command]
pub fn set_system_volume(volume_percent: u8) -> AppResult<()> {
    services::set_system_volume(volume_percent)
}

#[tauri::command]
pub fn set_default_audio_output(device_id: String) -> AppResult<()> {
    services::set_default_audio_output(&device_id)
}
