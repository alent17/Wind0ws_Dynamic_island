//! One-shot adapter for the real Compact component in an independent host.
//! Never installed on main; intentionally no live animation or polling fallback.
use super::region_prototype::{install_probe_mask, raster_hotspot, Frame};
use super::{normalized_interaction_region, InteractionCornerRadii, InteractionPoint};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use windows052::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    UI::WindowsAndMessaging::{GetClientRect, GetWindowRect},
};

static CLAIMED: AtomicBool = AtomicBool::new(false);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    mode: String,
    settled: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radii: InteractionCornerRadii,
    polygon: Vec<InteractionPoint>,
}

fn validate(label: &str, value: Value) -> Result<Request, String> {
    if label != "compact-native-probe" {
        return Err("independent probe host required".into());
    }
    let request: Request = serde_json::from_value(value).map_err(|e| e.to_string())?;
    if request.mode != "compact" || !request.settled {
        return Err("only immutable settled Compact is supported".into());
    }
    Ok(request)
}

/// Caller must execute on the window UI thread, once, before revealing the host.
pub fn apply_compact_probe_region(
    window: &tauri::WebviewWindow,
    value: Value,
) -> Result<Value, String> {
    let r = validate(window.label(), value)?;
    let region =
        normalized_interaction_region(r.x, r.y, r.width, r.height, r.radii, r.polygon, vec![])?;
    if CLAIMED.swap(true, Ordering::AcqRel) {
        return Err("probe already claimed; no live mask mutation".into());
    }
    let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0 as isize);
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let mut outer = RECT::default();
    let mut client = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        GetWindowRect(hwnd, &mut outer).map_err(|e| e.to_string())?;
        GetClientRect(hwnd, &mut client).map_err(|e| e.to_string())?;
        if !ClientToScreen(hwnd, &mut origin).as_bool() {
            return Err("ClientToScreen failed".into());
        }
    }
    if origin.x != outer.left
        || origin.y != outer.top
        || outer.right - outer.left != client.right
        || outer.bottom - outer.top != client.bottom
    {
        return Err("probe requires zero client offset and borderless host".into());
    }
    let frame = Frame {
        width: (outer.right - outer.left) as u32,
        height: (outer.bottom - outer.top) as u32,
        scale,
        client_offset: (0, 0),
    };
    let mask = raster_hotspot(&region, frame).map_err(str::to_string)?;
    install_probe_mask(hwnd, &mask).map_err(str::to_string)?;
    Ok(
        json!({"nativeApplied":true,"dpi":scale,"spans":mask.0.len(),
        "x":r.x,"y":r.y,"width":r.width,"height":r.height,"hwnd":hwnd.0}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data(mode: &str, settled: bool) -> Value {
        json!({"mode":mode,"settled":settled,
        "x":1,"y":2,"width":80,"height":28,"radii":{"topLeft":14,"topRight":14,"bottomLeft":14,"bottomRight":14},"polygon":[]})
    }
    #[test]
    fn rejects_product_host() {
        assert!(validate("main", data("compact", true)).is_err());
    }
    #[test]
    fn rejects_motion_and_other_modes() {
        assert!(validate("compact-native-probe", data("compact", false)).is_err());
        for mode in ["hover", "expanded", "hidden"] {
            assert!(validate("compact-native-probe", data(mode, true)).is_err());
        }
    }
    #[test]
    fn accepts_only_frozen_compact_schema() {
        assert!(validate("compact-native-probe", data("compact", true)).is_ok());
        let mut value = data("compact", true);
        value["extraRects"] = json!([]);
        assert!(validate("compact-native-probe", value).is_err());
    }
}
