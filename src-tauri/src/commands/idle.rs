use crate::error::AppResult;
use crate::models::{IdleSnapshot, WeatherLocation, WeatherLocationCandidate, WeatherSnapshot};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_idle_snapshot(app: AppHandle) -> AppResult<IdleSnapshot> {
    crate::services::get_idle_snapshot(&app).await
}

#[tauri::command]
pub async fn get_system_snapshot() -> AppResult<IdleSnapshot> {
    crate::services::idle::get_system_snapshot().await
}

#[tauri::command]
pub async fn get_weather_snapshot(app: AppHandle) -> AppResult<WeatherSnapshot> {
    crate::services::get_weather_snapshot(&app).await
}

#[tauri::command]
pub async fn search_weather_locations(
    query: String,
    language: String,
) -> AppResult<Vec<WeatherLocationCandidate>> {
    crate::services::search_weather_locations(&query, &language).await
}

#[tauri::command]
pub async fn get_weather_for_location(location: WeatherLocation) -> AppResult<WeatherSnapshot> {
    crate::services::idle::get_weather_for_location(&location).await
}

#[tauri::command]
pub async fn get_weather_details(location: WeatherLocation) -> AppResult<serde_json::Value> {
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,apparent_temperature,relative_humidity_2m,surface_pressure,cloud_cover,precipitation,rain,snowfall,wind_speed_10m,wind_direction_10m,wind_gusts_10m&hourly=temperature_2m,weather_code,precipitation_probability,visibility,uv_index&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,sunrise,sunset,sunshine_duration,uv_index_max&forecast_days=16&timezone=auto", location.latitude, location.longitude);
    let value = reqwest::Client::new().get(url).send().await.map_err(|e| crate::error::AppError::network(format!("Weather request failed: {}", e)))?.json().await.map_err(|e| crate::error::AppError::parse(format!("Weather response failed: {}", e)))?;
    Ok(value)
}
