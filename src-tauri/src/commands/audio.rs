use crate::error::{AppError, AppResult};
use crate::models::{AppAudioSession, AudioDeviceInfo, SystemAudioState};
use crate::services;

#[tauri::command]
pub async fn get_system_audio_state() -> AppResult<SystemAudioState> {
    super::native_task::run_native(services::get_system_audio_state)
        .await
        .map_err(|error| AppError::business(3010, format!("读取系统音频后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn list_audio_output_devices() -> AppResult<Vec<AudioDeviceInfo>> {
    super::native_task::run_native(services::list_audio_output_devices)
        .await
        .map_err(|error| AppError::business(3010, format!("枚举音频设备后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn set_system_volume(volume_percent: u8) -> AppResult<()> {
    super::native_task::run_native(move || services::set_system_volume(volume_percent))
        .await
        .map_err(|error| AppError::business(3010, format!("设置音量后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn set_default_audio_output(device_id: String) -> AppResult<()> {
    super::native_task::run_native(move || services::set_default_audio_output(&device_id))
        .await
        .map_err(|error| AppError::business(3010, format!("切换音频设备后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn list_app_audio_sessions() -> AppResult<Vec<AppAudioSession>> {
    super::native_task::run_native(services::list_app_audio_sessions).await
        .map_err(|error| AppError::business(3010, format!("读取应用音量后台任务失败：{error}")))?
}

#[tauri::command]
pub async fn set_app_audio_volume(device_id: String, process_id: u32, system_sounds: bool, volume_percent: Option<u8>, muted: bool) -> AppResult<()> {
    super::native_task::run_native(move || services::set_app_audio_volume(&device_id, process_id, system_sounds, volume_percent, muted)).await
        .map_err(|error| AppError::business(3010, format!("设置应用音量后台任务失败：{error}")))?
}
