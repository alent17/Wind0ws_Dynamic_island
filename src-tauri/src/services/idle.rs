use crate::error::{AppError, AppResult};
use crate::models::{IdleSnapshot, WeatherLocation, WeatherLocationCandidate};
use crate::state::AppState;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Networks, System};
use tauri::{AppHandle, Manager};

struct SystemCollector {
    system: System,
    networks: Networks,
    last_refresh: Instant,
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self {
            // Only CPU and memory are read below. new_all() also enumerates
            // every process and disk on first use, causing a visible pause.
            system: System::new(),
            networks: Networks::new_with_refreshed_list(),
            last_refresh: Instant::now(),
        }
    }
}

#[derive(Clone, Default)]
struct WeatherCache {
    location: Option<(f64, f64)>,
    temperature: Option<f32>,
    code: Option<u16>,
    updated_at: Option<u64>,
    fetched: Option<Instant>,
}

static SYSTEM_COLLECTOR: OnceLock<Mutex<SystemCollector>> = OnceLock::new();
static WEATHER_CACHE: OnceLock<Mutex<WeatherCache>> = OnceLock::new();

fn battery_status() -> (Option<u8>, Option<bool>) {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() || status.BatteryLifePercent == 255 {
        return (None, None);
    }
    (
        Some(status.BatteryLifePercent),
        Some(status.ACLineStatus == 1),
    )
}

fn system_snapshot() -> AppResult<IdleSnapshot> {
    let mut collector = SYSTEM_COLLECTOR
        .get_or_init(|| Mutex::new(SystemCollector::default()))
        .lock()
        .map_err(|_| AppError::lock("Failed to lock idle system collector"))?;
    let elapsed = collector.last_refresh.elapsed().as_secs_f64().max(0.1);
    collector.system.refresh_cpu();
    collector.system.refresh_memory();
    collector.networks.refresh();
    collector.last_refresh = Instant::now();
    let total = collector.system.total_memory();
    let used = collector.system.used_memory();
    let (download, upload) =
        collector
            .networks
            .iter()
            .fold((0u64, 0u64), |(down, up), (_, data)| {
                (
                    down.saturating_add(data.received()),
                    up.saturating_add(data.transmitted()),
                )
            });
    let (battery_percent, battery_charging) = battery_status();
    Ok(IdleSnapshot {
        cpu_percent: collector.system.global_cpu_info().cpu_usage(),
        memory_percent: if total > 0 {
            used as f32 / total as f32 * 100.0
        } else {
            0.0
        },
        upload_bytes_per_second: (upload as f64 / elapsed) as u64,
        download_bytes_per_second: (download as f64 / elapsed) as u64,
        battery_percent,
        battery_charging,
        ..IdleSnapshot::default()
    })
}

#[derive(Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}
#[derive(Deserialize)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    #[serde(default)]
    country: String,
    #[serde(default)]
    admin1: String,
    #[serde(default)]
    admin2: String,
    #[serde(default)]
    feature_code: String,
    #[serde(default)]
    population: u64,
}

fn is_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
}

fn geocoding_rank(item: &GeocodingResult) -> (u8, u64) {
    let administrative = matches!(
        item.feature_code.as_str(),
        "PPLC" | "PPLA" | "PPLA2" | "PPLA3" | "PPLA4"
    );
    (u8::from(administrative), item.population)
}

async fn fetch_geocoding_results(query: &str, language: &str) -> AppResult<Vec<GeocodingResult>> {
    let language = match language {
        "en" => "en",
        "ja" => "ja",
        _ => "zh",
    };
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=8&language={}&format=json",
        urlencoding::encode(query),
        language
    );
    let response: GeocodingResponse = reqwest::get(url)
        .await
        .map_err(|e| AppError::network(format!("Weather location request failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::parse(format!("Weather location response failed: {}", e)))?;
    Ok(response.results.unwrap_or_default())
}

fn dedupe_weather_candidates(
    candidates: impl IntoIterator<Item = WeatherLocationCandidate>,
) -> Vec<WeatherLocationCandidate> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| {
            let key = format!(
                "{}|{}|{}|{}|{:.4}|{:.4}",
                candidate.name.trim().to_lowercase(),
                candidate.admin1.trim().to_lowercase(),
                candidate.admin2.trim().to_lowercase(),
                candidate.country.trim().to_lowercase(),
                candidate.latitude,
                candidate.longitude
            );
            seen.insert(key)
        })
        .collect()
}

pub async fn search_weather_locations(
    query: &str,
    language: &str,
) -> AppResult<Vec<WeatherLocationCandidate>> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }

    let mut results = fetch_geocoding_results(query, language).await?;
    // GeoNames often indexes Chinese prefecture-level cities with the 市 suffix only.
    // Searching both forms makes queries such as “江门” find “江门市” instead of
    // returning only small same-name settlements in other provinces.
    if is_cjk(query) && !query.ends_with('市') {
        results.extend(fetch_geocoding_results(&format!("{query}市"), language).await?);
    }
    results.sort_by(|left, right| geocoding_rank(right).cmp(&geocoding_rank(left)));

    Ok(dedupe_weather_candidates(
        results
            .into_iter()
            .map(|item| WeatherLocationCandidate {
                name: item.name,
                latitude: item.latitude,
                longitude: item.longitude,
                country: item.country,
                admin1: item.admin1,
                admin2: item.admin2,
            })
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize)]
struct ForecastResponse {
    current: Option<CurrentWeather>,
}
#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f32,
    weather_code: u16,
}

async fn weather_for(location: &WeatherLocation) -> AppResult<WeatherCache> {
    {
        let cache = WEATHER_CACHE
            .get_or_init(|| Mutex::new(WeatherCache::default()))
            .lock()
            .map_err(|_| AppError::lock("Failed to lock weather cache"))?;
        if cache_matches_location(&cache, location)
            && cache
                .fetched
                .map(|at| at.elapsed() < Duration::from_secs(1800))
                .unwrap_or(false)
        {
            return Ok(cache.clone());
        }
    }
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&timezone=auto", location.latitude, location.longitude);
    let response: ForecastResponse = reqwest::get(url)
        .await
        .map_err(|e| AppError::network(format!("Weather request failed: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::parse(format!("Weather response failed: {}", e)))?;
    let current = response
        .current
        .ok_or_else(|| AppError::parse("Weather response has no current conditions"))?;
    let next = WeatherCache {
        location: Some((location.latitude, location.longitude)),
        temperature: Some(current.temperature_2m),
        code: Some(current.weather_code),
        updated_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
        fetched: Some(Instant::now()),
    };
    *WEATHER_CACHE
        .get_or_init(|| Mutex::new(WeatherCache::default()))
        .lock()
        .map_err(|_| AppError::lock("Failed to lock weather cache"))? = next.clone();
    Ok(next)
}

pub async fn get_idle_snapshot(app: &AppHandle) -> AppResult<IdleSnapshot> {
    let mut snapshot = system_snapshot()?;
    let location = app
        .state::<AppState>()
        .settings
        .lock()
        .ok()
        .and_then(|settings| settings.weather_location.clone());
    if let Some(location) = location {
        let weather = match weather_for(&location).await {
            Ok(value) => value,
            Err(_) => {
                let cached = WEATHER_CACHE
                    .get_or_init(|| Mutex::new(WeatherCache::default()))
                    .lock()
                    .map_err(|_| AppError::lock("Failed to lock weather cache"))?
                    .clone();
                if cache_matches_location(&cached, &location) {
                    cached
                } else {
                    WeatherCache::default()
                }
            }
        };
        snapshot.weather_temperature = weather.temperature;
        snapshot.weather_code = weather.code;
        snapshot.weather_updated_at = weather.updated_at;
    }
    Ok(snapshot)
}

fn cache_matches_location(cache: &WeatherCache, location: &WeatherLocation) -> bool {
    cache.location == Some((location.latitude, location.longitude))
}

#[cfg(test)]
mod tests {
    use super::{
        cache_matches_location, dedupe_weather_candidates, geocoding_rank, GeocodingResult,
        WeatherCache,
    };
    use crate::models::{WeatherLocation, WeatherLocationCandidate};

    fn city(name: &str, admin1: &str, latitude: f64) -> WeatherLocationCandidate {
        WeatherLocationCandidate {
            name: name.to_string(),
            latitude,
            longitude: 121.47,
            country: "中国".to_string(),
            admin1: admin1.to_string(),
            admin2: String::new(),
        }
    }

    #[test]
    fn removes_exact_duplicates_but_keeps_same_name_in_another_region() {
        let result = dedupe_weather_candidates(vec![
            city("上海", "上海", 31.23),
            city("上海", "上海", 31.23),
            city("上海", "云南", 25.04),
        ]);
        assert_eq!(result.len(), 2);
        assert_ne!(result[0].latitude, result[1].latitude);
    }

    #[test]
    fn ranks_prefecture_cities_ahead_of_same_name_settlements() {
        let city = GeocodingResult {
            name: "江门市".to_string(),
            latitude: 22.58,
            longitude: 113.08,
            country: "中国".to_string(),
            admin1: "广东".to_string(),
            admin2: "江门市".to_string(),
            feature_code: "PPLA2".to_string(),
            population: 1_795_459,
        };
        let village = GeocodingResult {
            name: "江门".to_string(),
            latitude: 25.29,
            longitude: 109.98,
            country: "中国".to_string(),
            admin1: "广西".to_string(),
            admin2: "桂林市".to_string(),
            feature_code: "PPL".to_string(),
            population: 0,
        };
        assert!(geocoding_rank(&city) > geocoding_rank(&village));
    }

    #[test]
    fn cached_weather_isolated_by_exact_coordinates() {
        let cache = WeatherCache {
            location: Some((22.58, 113.08)),
            ..WeatherCache::default()
        };
        let jiangmen = WeatherLocation {
            name: "江门市 · 广东 · 中国".to_string(),
            latitude: 22.58,
            longitude: 113.08,
        };
        let same_name_elsewhere = WeatherLocation {
            name: "江门 · 广西 · 中国".to_string(),
            latitude: 25.29,
            longitude: 109.98,
        };
        assert!(cache_matches_location(&cache, &jiangmen));
        assert!(!cache_matches_location(&cache, &same_name_elsewhere));
    }
}
