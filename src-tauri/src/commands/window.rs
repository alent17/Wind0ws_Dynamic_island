use crate::error::{AppError, AppResult};
use crate::event_bus::EVENT_BUS;
use crate::services::write_settings_file;
use crate::state::AppState;
use serde::Deserialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex, OnceLock,
};
use tauri::{AppHandle, Manager};

const DEFAULT_FLOATING_WINDOW_WIDTH: u32 = 260;
const DEFAULT_FLOATING_WINDOW_HEIGHT: u32 = 360;
const FLOATING_WINDOW_TRANSITION_MS: u64 = 260;
static FLOATING_WINDOW_MOTION_REVISION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionCornerRadii {
    top_left: f64,
    top_right: f64,
    bottom_right: f64,
    bottom_left: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
pub struct InteractionPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionExtraRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct IslandInteractionRegion {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: InteractionCornerRadii,
    polygon: Vec<InteractionPoint>,
    extra_rects: Vec<InteractionExtraRect>,
    enabled: bool,
}

static ISLAND_INTERACTION_REGION: OnceLock<Mutex<IslandInteractionRegion>> = OnceLock::new();
static ISLAND_INTERACTION_REGION_REVISION: AtomicU64 = AtomicU64::new(0);
static ISLAND_CURSOR_MONITOR_STARTED: AtomicBool = AtomicBool::new(false);
static ISLAND_CURSOR_WATCHDOG_STARTED: AtomicBool = AtomicBool::new(false);
static ISLAND_CURSOR_MONITOR_GENERATION: AtomicU64 = AtomicU64::new(0);
static ISLAND_CURSOR_MONITOR_HEARTBEAT: AtomicU64 = AtomicU64::new(0);
static WINDOW_MOTION_REVISION: AtomicU64 = AtomicU64::new(0);

const CURSOR_MONITOR_ACTIVE_INTERVAL_MS: u64 = 33;
const CURSOR_MONITOR_NEAR_INTERVAL_MS: u64 = 50;
const CURSOR_MONITOR_IDLE_INTERVAL_MS: u64 = 200;
const CURSOR_MONITOR_NEAR_DISTANCE_PX: i64 = 240;
const CURSOR_MONITOR_RETRY_MS: u64 = 100;
const CURSOR_MONITOR_WATCHDOG_INTERVAL_MS: u64 = 1_000;
const CURSOR_MONITOR_STALE_MS: u64 = 3_000;

#[derive(Default)]
struct WindowMotionState {
    handle: isize,
    current: [f64; 4],
    target: [f64; 4],
    velocity: [f64; 4],
    lower: [f64; 4],
    upper: [f64; 4],
    running: bool,
}

static WINDOW_MOTION_STATE: OnceLock<Mutex<WindowMotionState>> = OnceLock::new();

fn floating_window_animations_enabled(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    state
        .settings
        .lock()
        .map(|settings| settings.enable_animations && !settings.reduce_animations)
        .unwrap_or(true)
}

fn monitor_left_edge_for_position(
    window: &tauri::WebviewWindow,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> i32 {
    let Ok(monitors) = window.available_monitors() else {
        return 0;
    };
    if monitors.is_empty() {
        return 0;
    }

    let center_x = x as i64 + width as i64 / 2;
    let center_y = y as i64 + height as i64 / 2;
    let nearest = monitors
        .iter()
        .find(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            center_x >= position.x as i64
                && center_x < position.x as i64 + size.width as i64
                && center_y >= position.y as i64
                && center_y < position.y as i64 + size.height as i64
        })
        .or_else(|| {
            monitors.iter().min_by_key(|monitor| {
                let position = monitor.position();
                let size = monitor.size();
                let dx = if center_x < position.x as i64 {
                    position.x as i64 - center_x
                } else if center_x >= position.x as i64 + size.width as i64 {
                    center_x - (position.x as i64 + size.width as i64 - 1)
                } else {
                    0
                };
                let dy = if center_y < position.y as i64 {
                    position.y as i64 - center_y
                } else if center_y >= position.y as i64 + size.height as i64 {
                    center_y - (position.y as i64 + size.height as i64 - 1)
                } else {
                    0
                };
                dx * dx + dy * dy
            })
        });

    nearest.map(|monitor| monitor.position().x).unwrap_or(0)
}

fn cubic_bezier_component(t: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - t;
    3.0 * inverse * inverse * t * first + 3.0 * inverse * t * t * second + t * t * t
}

fn floating_window_ease_out(progress: f64) -> f64 {
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..16 {
        let midpoint = (low + high) / 2.0;
        if cubic_bezier_component(midpoint, 0.23, 0.32) < progress {
            low = midpoint;
        } else {
            high = midpoint;
        }
    }
    cubic_bezier_component((low + high) / 2.0, 1.0, 1.0)
}

async fn animate_floating_window_position(
    window: tauri::WebviewWindow,
    target_x: i32,
    target_y: i32,
    animate: bool,
) -> AppResult<bool> {
    let start = window
        .outer_position()
        .map_err(|error| AppError::window(error.to_string()))?;
    let revision = FLOATING_WINDOW_MOTION_REVISION.fetch_add(1, Ordering::AcqRel) + 1;

    if !animate || (start.x == target_x && start.y == target_y) {
        window
            .set_position(tauri::PhysicalPosition::new(target_x, target_y))
            .map_err(|error| AppError::window(error.to_string()))?;
        return Ok(true);
    }

    tauri::async_runtime::spawn_blocking(move || -> AppResult<bool> {
        let started_at = std::time::Instant::now();
        let start_x = start.x as f64;
        let start_y = start.y as f64;
        loop {
            if FLOATING_WINDOW_MOTION_REVISION.load(Ordering::Acquire) != revision {
                return Ok(false);
            }

            let progress = (started_at.elapsed().as_millis() as f64
                / FLOATING_WINDOW_TRANSITION_MS as f64)
                .clamp(0.0, 1.0);
            let eased = floating_window_ease_out(progress);
            let x = (start_x + (target_x as f64 - start_x) * eased).round() as i32;
            let y = (start_y + (target_y as f64 - start_y) * eased).round() as i32;
            window
                .set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|error| AppError::window(error.to_string()))?;

            if progress >= 1.0 {
                return Ok(true);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    })
    .await
    .map_err(|error| AppError::window(format!("Floating window animation failed: {error}")))?
}

fn normalized_interaction_region(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: InteractionCornerRadii,
    polygon: Vec<InteractionPoint>,
    extra_rects: Vec<InteractionExtraRect>,
) -> Result<IslandInteractionRegion, &'static str> {
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || !radii.top_left.is_finite()
        || !radii.top_right.is_finite()
        || !radii.bottom_right.is_finite()
        || !radii.bottom_left.is_finite()
        || width <= 0.0
        || height <= 0.0
        || polygon
            .iter()
            .any(|point| !point.x.is_finite() || !point.y.is_finite())
        || extra_rects.iter().any(|rect| {
            !rect.x.is_finite()
                || !rect.y.is_finite()
                || !rect.width.is_finite()
                || !rect.height.is_finite()
                || !rect.radius.is_finite()
                || rect.width <= 0.0
                || rect.height <= 0.0
        })
    {
        return Err("Invalid island interaction region");
    }

    let max_radius = width.min(height) / 2.0;
    let clamp = |radius: f64| radius.max(0.0).min(max_radius);
    Ok(IslandInteractionRegion {
        x,
        y,
        width,
        height,
        radii: InteractionCornerRadii {
            top_left: clamp(radii.top_left),
            top_right: clamp(radii.top_right),
            bottom_right: clamp(radii.bottom_right),
            bottom_left: clamp(radii.bottom_left),
        },
        polygon,
        extra_rects,
        enabled: true,
    })
}

fn point_in_polygon(x: f64, y: f64, points: &[InteractionPoint]) -> bool {
    if points.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = points.len() - 1;
    for current in 0..points.len() {
        let a = points[current];
        let b = points[previous];
        if ((a.y > y) != (b.y > y)) && x < (b.x - a.x) * (y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn point_in_rect(x: f64, y: f64, rect: &InteractionExtraRect) -> bool {
    if x < rect.x || y < rect.y || x > rect.x + rect.width || y > rect.y + rect.height {
        return false;
    }
    let radius = rect.radius.max(0.0).min(rect.width.min(rect.height) / 2.0);
    if radius <= 0.0 {
        return true;
    }
    let corners = [
        (rect.x + radius, rect.y + radius, x, y),
        (rect.x + rect.width - radius, rect.y + radius, x, y),
        (
            rect.x + rect.width - radius,
            rect.y + rect.height - radius,
            x,
            y,
        ),
        (rect.x + radius, rect.y + rect.height - radius, x, y),
    ];
    for (corner_x, corner_y, point_x, point_y) in corners {
        let in_corner_x = (point_x < rect.x + radius && corner_x == rect.x + radius)
            || (point_x > rect.x + rect.width - radius && corner_x == rect.x + rect.width - radius);
        let in_corner_y = (point_y < rect.y + radius && corner_y == rect.y + radius)
            || (point_y > rect.y + rect.height - radius
                && corner_y == rect.y + rect.height - radius);
        if in_corner_x && in_corner_y {
            let dx = point_x - corner_x;
            let dy = point_y - corner_y;
            return dx * dx + dy * dy <= radius * radius;
        }
    }
    true
}

fn point_in_rounded_rect(x: f64, y: f64, region: &IslandInteractionRegion) -> bool {
    if !region.enabled {
        return false;
    }
    let local_x = x - region.x;
    let local_y = y - region.y;
    if local_x >= 0.0 && local_y >= 0.0 && local_x <= region.width && local_y <= region.height {
        if region.polygon.len() >= 3 {
            if point_in_polygon(local_x, local_y, &region.polygon) {
                return true;
            }
        } else {
            let corners = [
                (region.radii.top_left, local_x, local_y),
                (region.radii.top_right, region.width - local_x, local_y),
                (
                    region.radii.bottom_right,
                    region.width - local_x,
                    region.height - local_y,
                ),
                (region.radii.bottom_left, local_x, region.height - local_y),
            ];
            let mut inside_base = true;
            for (radius, corner_x, corner_y) in corners {
                if radius > 0.0 && corner_x < radius && corner_y < radius {
                    let dx = corner_x - radius;
                    let dy = corner_y - radius;
                    inside_base = dx * dx + dy * dy <= radius * radius;
                    break;
                }
            }
            if inside_base {
                return true;
            }
        }
    }
    if region
        .extra_rects
        .iter()
        .any(|rect| point_in_rect(x, y, rect))
    {
        return true;
    }
    false
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

fn cursor_monitor_is_stale(started: bool, heartbeat: u64, now: u64) -> bool {
    started && now.saturating_sub(heartbeat) >= CURSOR_MONITOR_STALE_MS
}

fn monitor_owns_generation(owned: u64, current: u64) -> bool {
    owned == current
}

fn interaction_revision_is_current(candidate: u64, current: u64) -> bool {
    candidate >= current
}

fn cursor_monitor_interval(
    cursor_x: i32,
    cursor_y: i32,
    rect: &windows::Win32::Foundation::RECT,
    inside: bool,
) -> u64 {
    if inside {
        return CURSOR_MONITOR_ACTIVE_INTERVAL_MS;
    }

    let dx = if cursor_x < rect.left {
        i64::from(rect.left) - i64::from(cursor_x)
    } else if cursor_x > rect.right {
        i64::from(cursor_x) - i64::from(rect.right)
    } else {
        0
    };
    let dy = if cursor_y < rect.top {
        i64::from(rect.top) - i64::from(cursor_y)
    } else if cursor_y > rect.bottom {
        i64::from(cursor_y) - i64::from(rect.bottom)
    } else {
        0
    };
    if dx.saturating_mul(dx) + dy.saturating_mul(dy)
        <= CURSOR_MONITOR_NEAR_DISTANCE_PX * CURSOR_MONITOR_NEAR_DISTANCE_PX
    {
        CURSOR_MONITOR_NEAR_INTERVAL_MS
    } else {
        CURSOR_MONITOR_IDLE_INTERVAL_MS
    }
}

fn spawn_cursor_monitor(window: tauri::WebviewWindow, generation: u64) -> AppResult<()> {
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect};

    let raw_handle = window
        .hwnd()
        .map_err(|error| AppError::window(error.to_string()))?;
    let handle = HWND(raw_handle.0 as _);
    std::thread::spawn(move || {
        let mut last_ignored = None;
        while monitor_owns_generation(
            generation,
            ISLAND_CURSOR_MONITOR_GENERATION.load(Ordering::Acquire),
        ) {
            ISLAND_CURSOR_MONITOR_HEARTBEAT.store(unix_millis(), Ordering::Release);
            if window
                .app_handle()
                .get_webview_window(window.label())
                .is_none()
            {
                break;
            }

            let mut cursor = POINT::default();
            let mut window_rect = RECT::default();
            if unsafe { GetCursorPos(&mut cursor) }.is_err()
                || unsafe { GetWindowRect(handle, &mut window_rect) }.is_err()
            {
                let _ = window.set_ignore_cursor_events(false);
                last_ignored = Some(false);
                std::thread::sleep(std::time::Duration::from_millis(CURSOR_MONITOR_RETRY_MS));
                continue;
            }

            let scale = match window.scale_factor() {
                Ok(scale) => scale.max(f64::EPSILON),
                Err(_) => {
                    let _ = window.set_ignore_cursor_events(false);
                    last_ignored = Some(false);
                    std::thread::sleep(std::time::Duration::from_millis(CURSOR_MONITOR_RETRY_MS));
                    continue;
                }
            };
            let local_x = (cursor.x - window_rect.left) as f64 / scale;
            let local_y = (cursor.y - window_rect.top) as f64 / scale;
            let inside = match ISLAND_INTERACTION_REGION
                .get()
                .and_then(|region| region.lock().ok())
            {
                Some(region) => point_in_rounded_rect(local_x, local_y, &region),
                None => {
                    let _ = window.set_ignore_cursor_events(false);
                    last_ignored = Some(false);
                    std::thread::sleep(std::time::Duration::from_millis(CURSOR_MONITOR_RETRY_MS));
                    continue;
                }
            };
            let ignored = !inside;

            if last_ignored != Some(ignored) {
                if window.set_ignore_cursor_events(ignored).is_err() {
                    let _ = window.set_ignore_cursor_events(false);
                    last_ignored = Some(false);
                    std::thread::sleep(std::time::Duration::from_millis(CURSOR_MONITOR_RETRY_MS));
                    continue;
                }
                last_ignored = Some(ignored);
            }
            let interval = cursor_monitor_interval(cursor.x, cursor.y, &window_rect, inside);
            std::thread::sleep(std::time::Duration::from_millis(interval));
        }

        let _ = window.set_ignore_cursor_events(false);
        if monitor_owns_generation(
            generation,
            ISLAND_CURSOR_MONITOR_GENERATION.load(Ordering::Acquire),
        ) {
            ISLAND_CURSOR_MONITOR_STARTED.store(false, Ordering::Release);
        }
    });
    Ok(())
}

fn ensure_cursor_watchdog(window: &tauri::WebviewWindow) {
    if ISLAND_CURSOR_WATCHDOG_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = window.app_handle().clone();
    let label = window.label().to_string();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(
            CURSOR_MONITOR_WATCHDOG_INTERVAL_MS,
        ));
        let Some(window) = app.get_webview_window(&label) else {
            ISLAND_CURSOR_WATCHDOG_STARTED.store(false, Ordering::Release);
            break;
        };
        let started = ISLAND_CURSOR_MONITOR_STARTED.load(Ordering::Acquire);
        let heartbeat = ISLAND_CURSOR_MONITOR_HEARTBEAT.load(Ordering::Acquire);
        if cursor_monitor_is_stale(started, heartbeat, unix_millis()) {
            ISLAND_CURSOR_MONITOR_GENERATION.fetch_add(1, Ordering::AcqRel);
            let _ = window.set_ignore_cursor_events(false);
            ISLAND_CURSOR_MONITOR_STARTED.store(false, Ordering::Release);
            let _ = install_island_cursor_passthrough(&window);
        } else if !started {
            let _ = install_island_cursor_passthrough(&window);
        }
    });
}

pub fn install_island_cursor_passthrough(window: &tauri::WebviewWindow) -> AppResult<()> {
    let initial_radii = InteractionCornerRadii {
        top_left: 14.0,
        top_right: 14.0,
        bottom_right: 14.0,
        bottom_left: 14.0,
    };
    let initial = normalized_interaction_region(
        110.0,
        22.0,
        80.0,
        28.0,
        initial_radii,
        Vec::new(),
        Vec::new(),
    )
    .map_err(AppError::window)?;
    ISLAND_INTERACTION_REGION.get_or_init(|| Mutex::new(initial));

    ensure_cursor_watchdog(window);

    if ISLAND_CURSOR_MONITOR_STARTED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    let generation = ISLAND_CURSOR_MONITOR_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
    ISLAND_CURSOR_MONITOR_HEARTBEAT.store(unix_millis(), Ordering::Release);
    if let Err(error) = spawn_cursor_monitor(window.clone(), generation) {
        ISLAND_CURSOR_MONITOR_STARTED.store(false, Ordering::Release);
        let _ = window.set_ignore_cursor_events(false);
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub fn set_island_interaction_region(
    revision: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: InteractionCornerRadii,
    polygon: Vec<InteractionPoint>,
    extra_rects: Vec<InteractionExtraRect>,
) -> AppResult<()> {
    let next = normalized_interaction_region(x, y, width, height, radii, polygon, extra_rects)
        .map_err(AppError::window)?;
    ISLAND_INTERACTION_REGION_REVISION.fetch_max(revision, Ordering::AcqRel);
    let mut region = ISLAND_INTERACTION_REGION
        .get_or_init(|| Mutex::new(IslandInteractionRegion::default()))
        .lock()
        .map_err(|_| AppError::lock("Failed to lock island interaction region"))?;
    if !interaction_revision_is_current(
        revision,
        ISLAND_INTERACTION_REGION_REVISION.load(Ordering::Acquire),
    ) {
        return Ok(());
    }
    if *region == next {
        return Ok(());
    }
    *region = next;
    Ok(())
}

#[cfg(test)]
mod interaction_region_tests {
    use super::{
        cursor_monitor_is_stale, interaction_revision_is_current, monitor_owns_generation,
        normalized_interaction_region, point_in_rounded_rect, InteractionCornerRadii,
        InteractionExtraRect, InteractionPoint,
    };

    fn radii(value: f64) -> InteractionCornerRadii {
        InteractionCornerRadii {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    #[test]
    fn accepts_the_pill_and_rejects_its_transparent_corners() {
        let region = normalized_interaction_region(
            110.0,
            22.0,
            80.0,
            28.0,
            radii(14.0),
            Vec::new(),
            Vec::new(),
        )
        .expect("valid region");
        assert!(point_in_rounded_rect(150.0, 36.0, &region));
        assert!(point_in_rounded_rect(111.0, 36.0, &region));
        assert!(!point_in_rounded_rect(110.0, 22.0, &region));
        assert!(!point_in_rounded_rect(109.0, 36.0, &region));
    }

    #[test]
    fn clamps_corner_radius_to_half_the_short_edge() {
        let region = normalized_interaction_region(
            0.0,
            0.0,
            80.0,
            20.0,
            radii(99.0),
            Vec::new(),
            Vec::new(),
        )
        .expect("valid region");
        assert_eq!(region.radii.top_left, 10.0);
        assert_eq!(region.radii.bottom_right, 10.0);
    }

    #[test]
    fn supports_square_attached_corners_and_round_inner_corners() {
        let region = normalized_interaction_region(
            0.0,
            0.0,
            80.0,
            28.0,
            InteractionCornerRadii {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 14.0,
                bottom_left: 14.0,
            },
            Vec::new(),
            Vec::new(),
        )
        .expect("valid region");
        assert!(point_in_rounded_rect(0.0, 0.0, &region));
        assert!(point_in_rounded_rect(80.0, 0.0, &region));
        assert!(!point_in_rounded_rect(0.0, 28.0, &region));
    }

    #[test]
    fn rejects_invalid_dimensions() {
        assert!(normalized_interaction_region(
            0.0,
            0.0,
            0.0,
            20.0,
            radii(4.0),
            Vec::new(),
            Vec::new()
        )
        .is_err());
        assert!(normalized_interaction_region(
            0.0,
            0.0,
            20.0,
            -1.0,
            radii(4.0),
            Vec::new(),
            Vec::new()
        )
        .is_err());
        assert!(normalized_interaction_region(
            f64::NAN,
            0.0,
            20.0,
            20.0,
            radii(4.0),
            Vec::new(),
            Vec::new()
        )
        .is_err());
    }

    #[test]
    fn polygon_rejects_the_transparent_shoulder_cutout() {
        let polygon = vec![
            InteractionPoint { x: 0.0, y: 0.0 },
            InteractionPoint { x: 80.0, y: 0.0 },
            InteractionPoint { x: 70.0, y: 8.0 },
            InteractionPoint { x: 70.0, y: 28.0 },
            InteractionPoint { x: 10.0, y: 28.0 },
            InteractionPoint { x: 10.0, y: 8.0 },
        ];
        let region =
            normalized_interaction_region(0.0, 0.0, 80.0, 28.0, radii(0.0), polygon, Vec::new())
                .expect("valid polygon region");
        assert!(point_in_rounded_rect(40.0, 14.0, &region));
        assert!(point_in_rounded_rect(2.0, 1.0, &region));
        assert!(!point_in_rounded_rect(2.0, 12.0, &region));
    }

    #[test]
    fn accepts_an_extra_feature_rect_outside_the_island_bounds() {
        let region = normalized_interaction_region(
            0.0,
            0.0,
            80.0,
            28.0,
            radii(14.0),
            Vec::new(),
            vec![InteractionExtraRect {
                x: 88.0,
                y: -51.0,
                width: 38.0,
                height: 130.0,
                radius: 19.0,
            }],
        )
        .expect("valid rail region");
        assert!(point_in_rounded_rect(100.0, 0.0, &region));
        assert!(!point_in_rounded_rect(130.0, 0.0, &region));
    }

    #[test]
    fn rejects_out_of_order_interaction_regions() {
        assert!(interaction_revision_is_current(42, 42));
        assert!(interaction_revision_is_current(43, 42));
        assert!(!interaction_revision_is_current(41, 42));
    }

    #[test]
    fn cursor_monitor_generation_has_a_single_owner() {
        assert!(monitor_owns_generation(7, 7));
        assert!(!monitor_owns_generation(7, 8));
    }

    #[test]
    fn watchdog_only_restarts_a_started_stale_monitor() {
        assert!(!cursor_monitor_is_stale(false, 1_000, 9_000));
        assert!(!cursor_monitor_is_stale(true, 7_001, 10_000));
        assert!(cursor_monitor_is_stale(true, 7_000, 10_000));
    }
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window
            .set_focus()
            .map_err(|e| AppError::window(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_studio_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("studio-window") {
        // Studio is pre-created hidden in tauri.conf.json. Reuse that WebView
        // so the first click only has to show and focus an already initialized
        // page instead of building a heavy Svelte/Canvas tree synchronously.
        if window.is_minimized().unwrap_or(false) {
            window
                .unminimize()
                .map_err(|e| AppError::window(e.to_string()))?;
        }
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window
            .set_focus()
            .map_err(|e| AppError::window(e.to_string()))?;
        return Ok(());
    }

    // Keep a fallback for older configs/dev profiles that do not have the
    // pre-created window yet.
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "studio-window",
        tauri::WebviewUrl::App("studio.html".into()),
    )
    .title("Isle Studio")
    .inner_size(1000.0, 750.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .decorations(true)
    .transparent(false)
    .build()
    .map_err(|e| AppError::window(format!("创建设置窗口失败: {}", e)))?;

    window.show().map_err(|e| AppError::window(e.to_string()))?;
    window
        .set_focus()
        .map_err(|e| AppError::window(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn toggle_studio_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("studio-window") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| AppError::window(e.to_string()))?;
            return Ok(());
        }

        return show_studio_window(app);
    }

    show_studio_window(app)
}

#[tauri::command]
pub async fn toggle_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        if window.is_visible().unwrap_or(false) {
            return close_floating_window(app).await;
        }
    }
    open_floating_window(app).await
}

#[tauri::command]
pub async fn open_floating_window(app: AppHandle) -> AppResult<()> {
    let animate = floating_window_animations_enabled(&app);
    let state = app.state::<AppState>();
    let (saved_position, always_on_top) = {
        let settings = state
            .settings
            .lock()
            .map_err(|_| AppError::lock("Failed to lock settings"))?;
        (
            (
                settings.floating_window_x,
                settings.floating_window_y,
                settings.floating_window_width,
                settings.floating_window_height,
            ),
            settings.floating_window_always_on_top,
        )
    };

    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_shadow(false)
            .map_err(|e| AppError::window(e.to_string()))?;
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| AppError::window(e.to_string()))?;

        if window.is_visible().unwrap_or(false) {
            window
                .set_focus()
                .map_err(|e| AppError::window(e.to_string()))?;
            return Ok(());
        }

        let target = window
            .outer_position()
            .map_err(|e| AppError::window(e.to_string()))?;
        let size = window
            .outer_size()
            .map_err(|e| AppError::window(e.to_string()))?;
        let start_x =
            monitor_left_edge_for_position(&window, target.x, target.y, size.width, size.height)
                - size.width as i32;
        if animate {
            window
                .set_position(tauri::PhysicalPosition::new(start_x, target.y))
                .map_err(|e| AppError::window(e.to_string()))?;
        }
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window
            .set_focus()
            .map_err(|e| AppError::window(e.to_string()))?;
        let _ = animate_floating_window_position(window, target.x, target.y, animate).await?;
        return Ok(());
    }

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app,
        "floating_player",
        tauri::WebviewUrl::App("index.html?window=floating".into()),
    )
    .title("Mini Player")
    .min_inner_size(200.0, 200.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(always_on_top);

    if let (Some(x), Some(y), Some(w), Some(h)) = saved_position {
        builder = builder.inner_size(w.max(200) as f64, h.max(200) as f64);
        builder = builder.position(x as f64, y as f64);
    } else {
        builder = builder.inner_size(
            DEFAULT_FLOATING_WINDOW_WIDTH as f64,
            DEFAULT_FLOATING_WINDOW_HEIGHT as f64,
        );
        builder = builder.center();
    }

    let window = builder
        .visible(false)
        .build()
        .map_err(|e| AppError::window(e.to_string()))?;

    let target = window
        .outer_position()
        .map_err(|e| AppError::window(e.to_string()))?;
    let size = window
        .outer_size()
        .map_err(|e| AppError::window(e.to_string()))?;
    let start_x =
        monitor_left_edge_for_position(&window, target.x, target.y, size.width, size.height)
            - size.width as i32;
    if animate {
        window
            .set_position(tauri::PhysicalPosition::new(start_x, target.y))
            .map_err(|e| AppError::window(e.to_string()))?;
    }
    window.show().map_err(|e| AppError::window(e.to_string()))?;
    window
        .set_focus()
        .map_err(|e| AppError::window(e.to_string()))?;
    let _ = animate_floating_window_position(window, target.x, target.y, animate).await?;

    Ok(())
}

#[tauri::command]
pub fn open_timer_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("timer_window") {
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        window
            .set_focus()
            .map_err(|e| AppError::window(e.to_string()))?;
        return Ok(());
    }

    Err(AppError::window("倒计时窗口尚未初始化"))
}

#[tauri::command]
pub fn toggle_timer_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("timer_window") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| AppError::window(e.to_string()))?;
        } else {
            window.show().map_err(|e| AppError::window(e.to_string()))?;
            window
                .set_focus()
                .map_err(|e| AppError::window(e.to_string()))?;
        }
        return Ok(());
    }

    Err(AppError::window("倒计时窗口尚未初始化"))
}

#[tauri::command]
pub async fn close_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        if window.is_visible().unwrap_or(false) {
            let animate = floating_window_animations_enabled(&app);
            let position = window
                .outer_position()
                .map_err(|e| AppError::window(e.to_string()))?;
            let size = window
                .outer_size()
                .map_err(|e| AppError::window(e.to_string()))?;
            let target_x = monitor_left_edge_for_position(
                &window,
                position.x,
                position.y,
                size.width,
                size.height,
            ) - size.width as i32;
            let completed =
                animate_floating_window_position(window.clone(), target_x, position.y, animate)
                    .await?;
            if !completed {
                return Ok(());
            }
        }
        window
            .close()
            .map_err(|e| AppError::window(e.to_string()))?;
        let _ = EVENT_BUS.emit(crate::event_bus::EVENT_FLOATING_WINDOW_CLOSED, ());
    }
    Ok(())
}

#[tauri::command]
pub fn reset_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_size(tauri::PhysicalSize::new(
                DEFAULT_FLOATING_WINDOW_WIDTH,
                DEFAULT_FLOATING_WINDOW_HEIGHT,
            ))
            .map_err(|e| AppError::window(e.to_string()))?;
        window
            .center()
            .map_err(|e| AppError::window(e.to_string()))?;
        window.show().map_err(|e| AppError::window(e.to_string()))?;
        return Ok(());
    }
    Err(AppError::window("悬浮播放器尚未打开"))
}

#[tauri::command]
pub fn sync_window_bounds(
    app: AppHandle,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
) -> AppResult<()> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOZORDER,
    };

    if let Some(window) = app.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                let _ = SetWindowPos(
                    HWND(hwnd.0 as _),
                    HWND_TOP,
                    x,
                    y,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn animate_window_bounds(
    app: AppHandle,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    animate: bool,
) -> AppResult<()> {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowRect, SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOZORDER,
    };

    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let raw = window
        .hwnd()
        .map_err(|error| AppError::window(error.to_string()))?;
    let hwnd = HWND(raw.0 as _);
    if !animate {
        WINDOW_MOTION_REVISION.fetch_add(1, Ordering::AcqRel);
        if let Ok(mut state) = WINDOW_MOTION_STATE
            .get_or_init(|| Mutex::new(WindowMotionState::default()))
            .lock()
        {
            state.running = false;
            state.velocity = [0.0; 4];
        }
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOP,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        return Ok(());
    }

    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        return Ok(());
    }
    let start = (
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
    );
    let handle = hwnd.0 as isize;
    let target = [x as f64, y as f64, width as f64, height as f64];
    let motion = WINDOW_MOTION_STATE.get_or_init(|| Mutex::new(WindowMotionState::default()));
    let mut state = motion
        .lock()
        .map_err(|_| AppError::lock("Failed to lock window motion state"))?;
    state.target = target;
    if state.running && state.handle == handle {
        for index in 0..4 {
            state.lower[index] = state.current[index].min(target[index]);
            state.upper[index] = state.current[index].max(target[index]);
        }
        // Retarget the live spring and preserve its velocity.
        return Ok(());
    }
    state.handle = handle;
    state.current = [
        start.0 as f64,
        start.1 as f64,
        start.2 as f64,
        start.3 as f64,
    ];
    for index in 0..4 {
        state.lower[index] = state.current[index].min(target[index]);
        state.upper[index] = state.current[index].max(target[index]);
    }
    state.velocity = [0.0; 4];
    state.running = true;
    drop(state);

    let revision = WINDOW_MOTION_REVISION.fetch_add(1, Ordering::AcqRel) + 1;
    // Send the raw handle as an integer; HWND itself is deliberately not Send.
    std::thread::spawn(move || {
        let mut previous_tick = std::time::Instant::now();
        loop {
            if WINDOW_MOTION_REVISION.load(Ordering::Acquire) != revision {
                return;
            }
            let now = std::time::Instant::now();
            let dt = now
                .duration_since(previous_tick)
                .as_secs_f64()
                .clamp(0.001, 0.016);
            previous_tick = now;
            let values = {
                let Ok(mut state) = WINDOW_MOTION_STATE
                    .get_or_init(|| Mutex::new(WindowMotionState::default()))
                    .lock()
                else {
                    return;
                };
                if !state.running || state.handle != handle {
                    return;
                }
                let mut settled = true;
                // Approved interaction spring: mass 1, stiffness 100, damping 10.
                for index in 0..4 {
                    let displacement = state.current[index] - state.target[index];
                    let acceleration = -100.0 * displacement - 10.0 * state.velocity[index];
                    state.velocity[index] += acceleration * dt;
                    state.current[index] += state.velocity[index] * dt;
                    let clamped =
                        state.current[index].clamp(state.lower[index], state.upper[index]);
                    if clamped != state.current[index] {
                        state.current[index] = clamped;
                        state.velocity[index] = 0.0;
                    }
                    if displacement.abs() >= 0.35 || state.velocity[index].abs() >= 1.0 {
                        settled = false;
                    } else {
                        state.current[index] = state.target[index];
                        state.velocity[index] = 0.0;
                    }
                }
                let values = state.current;
                if settled {
                    state.running = false;
                }
                (values, settled)
            };
            unsafe {
                let _ = SetWindowPos(
                    HWND(handle as _),
                    HWND_TOP,
                    values.0[0].round() as i32,
                    values.0[1].round() as i32,
                    values.0[2].round() as i32,
                    values.0[3].round() as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            if values.1 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });
    Ok(())
}

#[tauri::command]
pub fn set_floating_window_resizable(app: AppHandle, resizable: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("floating_player") {
        window
            .set_resizable(resizable)
            .map_err(|e| AppError::window(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_application(name: String) -> AppResult<()> {
    use std::process::Command;

    let protocol = match name.as_str() {
        "NeteaseCloudMusic" => "orpheus://",
        "Spotify" => "spotify:",
        "Bilibili" => "bilibili://",
        "QQMusic" => "qqmusic://",
        "AppleMusic" => "https://music.apple.com",
        _ => "",
    };

    if !protocol.is_empty() {
        let output = Command::new("cmd")
            .args(["/C", "start", "", protocol])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Ok(());
            }
        }
    }

    let common_paths: Vec<&str> = match name.as_str() {
        "Spotify" => vec![
            r"%APPDATA%\Spotify\Spotify.exe",
            r"%LOCALAPPDATA%\Spotify\Spotify.exe",
        ],
        "NeteaseCloudMusic" => vec![
            r"%LOCALAPPDATA%\Netease\CloudMusic\cloudmusic.exe",
            r"%PROGRAMFILES%\Netease\CloudMusic\cloudmusic.exe",
            r"%PROGRAMFILES(X86)%\Netease\CloudMusic\cloudmusic.exe",
        ],
        "QQMusic" => vec![
            r"%PROGRAMFILES%\Tencent\QQMusic\QQMusic.exe",
            r"%PROGRAMFILES(X86)%\Tencent\QQMusic\QQMusic.exe",
        ],
        "Bilibili" => vec![
            r"%LOCALAPPDATA%\Programs\bilibili\bilibili.exe",
            r"%PROGRAMFILES%\Bilibili\bilibili.exe",
        ],
        "AppleMusic" => vec![
            r"%LOCALAPPDATA%\Microsoft\WindowsApps\AppleMusic.exe",
            r"%PROGRAMFILES%\WindowsApps\AppleMusic.exe",
        ],
        _ => vec![],
    };

    for path_template in &common_paths {
        let expanded = path_template
            .replace("%APPDATA%", &std::env::var("APPDATA").unwrap_or_default())
            .replace(
                "%LOCALAPPDATA%",
                &std::env::var("LOCALAPPDATA").unwrap_or_default(),
            )
            .replace(
                "%PROGRAMFILES%",
                &std::env::var("ProgramFiles").unwrap_or_default(),
            )
            .replace(
                "%PROGRAMFILES(X86)%",
                &std::env::var("ProgramFiles(x86)").unwrap_or_default(),
            );

        if std::path::Path::new(&expanded).exists() {
            let output = Command::new("cmd")
                .args(["/C", "start", "", &expanded])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(());
                }
            }
        }
    }

    Err(AppError::business(4001, format!("无法打开应用: {}", name)))
}

#[tauri::command]
pub fn check_fullscreen_app(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
) -> AppResult<bool> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GWL_STYLE, WS_CAPTION,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok(false);
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return Ok(false);
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width <= 0 || height <= 0 {
            return Ok(false);
        }

        let is_on_target_monitor = rect.left <= monitor_x + monitor_width
            && rect.right >= monitor_x
            && rect.top <= monitor_y + monitor_height
            && rect.bottom >= monitor_y;

        if !is_on_target_monitor {
            return Ok(false);
        }

        let covers_screen = width >= monitor_width - 2 && height >= monitor_height - 2;

        if !covers_screen {
            return Ok(false);
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let no_caption = (style & WS_CAPTION.0 as isize) == 0;

        Ok(no_caption)
    }
}

#[tauri::command]
pub async fn get_available_monitors(app: AppHandle) -> AppResult<Vec<String>> {
    let window = app
        .get_webview_window("main")
        .or_else(|| app.get_webview_window("floating"))
        .ok_or_else(|| AppError::window("Failed to get window"))?;

    let monitors = window
        .available_monitors()
        .map_err(|e| AppError::window(format!("Failed to get monitors: {}", e)))?;

    let monitor_names: Vec<String> = monitors
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            m.name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("显示器 {}", idx + 1))
        })
        .collect();

    Ok(monitor_names)
}

#[tauri::command]
pub async fn get_current_monitor_index(state: tauri::State<'_, AppState>) -> AppResult<u32> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    Ok(settings.monitor_index)
}

#[tauri::command]
pub fn set_current_monitor_index(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    index: u32,
) -> AppResult<()> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| AppError::lock("Failed to lock settings"))?;
    settings.monitor_index = index;

    write_settings_file(&app, &settings)?;

    let _ = EVENT_BUS.emit(crate::event_bus::EVENT_SETTINGS_CHANGED, "monitor_index");

    Ok(())
}
