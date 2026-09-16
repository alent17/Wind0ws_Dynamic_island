use crate::error::AppResult;
use crate::models::{IdleSnapshot, WeatherLocationCandidate};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_idle_snapshot(app: AppHandle) -> AppResult<IdleSnapshot> {
    crate::services::get_idle_snapshot(&app).await
}

#[tauri::command]
pub async fn search_weather_locations(
    query: String,
    language: String,
) -> AppResult<Vec<WeatherLocationCandidate>> {
    crate::services::search_weather_locations(&query, &language).await
}
