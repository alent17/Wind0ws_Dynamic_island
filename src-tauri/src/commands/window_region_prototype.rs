//! Offline window-shape proposal. No HWND, GUI, hook or input calls.
//! Headless GDI region construction only; never calls SetWindowRgn.
//! Raster is shared with the feature-gated static lab adapter. The old model
//! and its GDI helper remain offline reference code, not the installed policy.
use super::{point_in_rounded_rect, IslandInteractionRegion};

const MAX_PIXELS: u64 = 1_000_000;
const MAX_SEGMENTS: usize = 16_384;
const MAX_VERTICES: usize = 256;
const MAX_EXTRA_RECTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Frame {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale: f64,
    // SetWindowRgn uses window coordinates; source geometry is client-local DIP.
    pub(super) client_offset: (i32, i32),
}

#[derive(Debug, PartialEq)]
pub(super) struct Span {
    pub(super) y: i32,
    pub(super) left: i32,
    pub(super) right: i32, // exclusive
}

#[derive(Debug)]
pub(super) struct Mask(pub(super) Vec<Span>);

struct OwnedRegion(windows052::Win32::Graphics::Gdi::HRGN);

impl Drop for OwnedRegion {
    fn drop(&mut self) {
        // This object never transfers ownership to a window in this prototype.
        if !unsafe { windows052::Win32::Graphics::Gdi::DeleteObject(self.0) }.as_bool() {
            eprintln!("prototype GDI region cleanup failed; resource release unconfirmed");
        }
    }
}

fn gdi_region(mask: &Mask) -> Result<OwnedRegion, &'static str> {
    use windows052::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, ERROR, RGN_OR};
    let create = |left, top, right, bottom| {
        let raw = unsafe { CreateRectRgn(left, top, right, bottom) };
        if raw.0 == 0 {
            Err("CreateRectRgn failed")
        } else {
            Ok(OwnedRegion(raw))
        }
    };
    let result = create(0, 0, 0, 0)?;
    for span in &mask.0 {
        let row = create(span.left, span.y, span.right, span.y + 1)?;
        if unsafe { CombineRgn(result.0, result.0, row.0, RGN_OR) }.0 == ERROR {
            return Err("CombineRgn failed");
        }
        // row is deleted here; CombineRgn copies into result.
    }
    Ok(result)
}

#[cfg(feature = "compact-native-probe")]
pub(super) fn install_probe_mask(
    hwnd: windows052::Win32::Foundation::HWND,
    mask: &Mask,
) -> Result<(), &'static str> {
    if mask.0.is_empty() {
        return Err("empty probe mask");
    }
    let owned = gdi_region(mask)?;
    if unsafe { windows052::Win32::Graphics::Gdi::SetWindowRgn(hwnd, owned.0, true) } == 0 {
        return Err("SetWindowRgn failed");
    }
    std::mem::forget(owned); // successful SetWindowRgn transfers HRGN ownership
    Ok(())
}

pub(super) fn raster(region: &IslandInteractionRegion, frame: Frame) -> Result<Mask, &'static str> {
    raster_sample(region, frame, 0.5)
}

// Match the existing integer GetCursorPos hotspot contract; centre sampling
// can admit a border pixel whose integer hotspot is outside the DIP geometry.
pub(super) fn raster_hotspot(
    region: &IslandInteractionRegion,
    frame: Frame,
) -> Result<Mask, &'static str> {
    raster_sample(region, frame, 0.0)
}

fn raster_sample(
    region: &IslandInteractionRegion,
    frame: Frame,
    sample_offset: f64,
) -> Result<Mask, &'static str> {
    if !frame.scale.is_finite()
        || frame.scale <= 0.0
        || frame.width == 0
        || frame.height == 0
        || u64::from(frame.width) * u64::from(frame.height) > MAX_PIXELS
        || region.polygon.len() > MAX_VERTICES
        || region.extra_rects.len() > MAX_EXTRA_RECTS
    {
        return Err("invalid or over-budget physical frame");
    }
    let mut spans = Vec::new();
    for y in 0..frame.height as i32 {
        let mut start = None;
        for x in 0..=frame.width as i32 {
            // The caller chooses the explicit sample contract, not a margin.
            let inside = x < frame.width as i32
                && point_in_rounded_rect(
                    (f64::from(x) + sample_offset - f64::from(frame.client_offset.0)) / frame.scale,
                    (f64::from(y) + sample_offset - f64::from(frame.client_offset.1)) / frame.scale,
                    region,
                );
            match (start, inside) {
                (None, true) => start = Some(x),
                (Some(left), false) => {
                    if spans.len() == MAX_SEGMENTS {
                        return Err("over-budget region complexity");
                    }
                    spans.push(Span { y, left, right: x });
                    start = None;
                }
                _ => {}
            }
        }
    }
    Ok(Mask(spans))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ticket {
    generation: u64,
    epoch: u64,
    revision: u64,
}

trait Backend {
    // A future UI-thread adapter must return native apply success, not enqueue.
    // Failure must leave the previous mask intact, or explicitly report unknown.
    fn install(&mut self, mask: &Mask) -> Result<(), &'static str>;
}

struct Draft {
    ticket: Ticket,
    mask: Mask,
}

struct Session {
    current: Ticket,
    alive: bool,
    ready: bool,
    // Last successful apply is history, never reset just because a new one failed.
    last_applied: Option<Ticket>,
}

impl Session {
    fn new(generation: u64) -> Self {
        Self {
            current: Ticket {
                generation,
                epoch: 0,
                revision: 0,
            },
            alive: true,
            ready: false,
            last_applied: None,
        }
    }

    // Geometry, hide/restore, DPI, client offset, drag/capture changes require
    // invalidation on the owning UI thread *before* accepting new work.
    // In a real host ready=false must gate visibility/input; this model cannot.
    fn invalidate(&mut self) {
        self.current.epoch += 1;
        self.ready = false;
    }

    fn prepare(
        &mut self,
        revision: u64,
        region: &IslandInteractionRegion,
        frame: Frame,
    ) -> Result<Draft, &'static str> {
        if !self.alive || revision <= self.current.revision {
            return Err("destroyed, stale or duplicate revision");
        }
        self.current.revision = revision;
        self.ready = false;
        Ok(Draft {
            ticket: self.current,
            mask: raster(region, frame)?,
        })
    }

    fn commit(&mut self, draft: Draft, backend: &mut impl Backend) -> Result<(), &'static str> {
        if !self.alive || self.current != draft.ticket {
            return Err("stale window generation/epoch/revision");
        }
        self.ready = false;
        backend.install(&draft.mask)?;
        self.last_applied = Some(draft.ticket);
        self.ready = true;
        Ok(())
    }

    fn destroy(&mut self) {
        self.alive = false;
        self.invalidate();
        // A real host must detach owned callbacks and cancel pending work.
        // No handle ownership is simulated here.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::window::{
        normalized_interaction_region, InteractionCornerRadii, InteractionExtraRect,
        InteractionPoint,
    };

    fn shape() -> IslandInteractionRegion {
        normalized_interaction_region(
            2.0,
            2.0,
            8.0,
            8.0,
            InteractionCornerRadii {
                top_left: 2.0,
                top_right: 2.0,
                bottom_left: 2.0,
                bottom_right: 2.0,
            },
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
    }
    fn frame(scale: f64) -> Frame {
        Frame {
            width: 40,
            height: 40,
            scale,
            client_offset: (0, 0),
        }
    }
    fn hit(mask: &Mask, x: i32, y: i32) -> bool {
        mask.0
            .iter()
            .any(|s| s.y == y && x >= s.left && x < s.right)
    }
    #[derive(Default)]
    struct Fake {
        calls: usize,
        fail: bool,
        spans: usize,
    }
    impl Backend for Fake {
        fn install(&mut self, mask: &Mask) -> Result<(), &'static str> {
            self.calls += 1;
            if self.fail {
                return Err("native install failed");
            }
            self.spans = mask.0.len();
            Ok(())
        }
    }

    #[test]
    fn rounded_holes_and_physical_edges_at_three_dpis() {
        for scale in [1.0, 1.5, 2.0] {
            let mask = raster(&shape(), frame(scale)).unwrap();
            assert!(hit(&mask, (6.0 * scale) as i32, (6.0 * scale) as i32));
            assert!(!hit(&mask, (2.0 * scale) as i32, (2.0 * scale) as i32));
            assert!(!hit(&mask, (10.0 * scale) as i32, (6.0 * scale) as i32));
        }
    }
    #[test]
    fn polygon_replaces_base_and_extra_rects_union_without_filling_holes() {
        let mut r = shape();
        r.polygon = vec![
            InteractionPoint { x: 0.0, y: 0.0 },
            InteractionPoint { x: 8.0, y: 0.0 },
            InteractionPoint { x: 0.0, y: 8.0 },
        ];
        r.extra_rects = vec![InteractionExtraRect {
            x: 12.0,
            y: 2.0,
            width: 4.0,
            height: 4.0,
            radius: 1.0,
        }];
        let mask = raster(&r, frame(1.0)).unwrap();
        assert!(hit(&mask, 3, 3));
        assert!(!hit(&mask, 8, 8));
        assert!(!hit(&mask, 11, 3));
        assert!(hit(&mask, 13, 3));
        assert!(!hit(&mask, 16, 3));
    }
    #[test]
    fn client_to_window_offset_is_not_screen_origin() {
        let f = Frame {
            client_offset: (3, 4),
            ..frame(1.0)
        };
        let mask = raster(&shape(), f).unwrap();
        assert!(hit(&mask, 9, 10));
        assert!(!hit(&mask, 3, 4));
        // No screen origin is baked into a local mask: a secondary monitor
        // at -1920 can use exactly this mask after moving the parent.
    }
    #[test]
    fn disabled_region_is_empty_not_unrestricted() {
        let mut r = shape();
        r.enabled = false;
        assert!(raster(&r, frame(1.0)).unwrap().0.is_empty());
    }
    #[test]
    fn integer_hotspot_does_not_admit_fractional_edge_pixel_from_centre_sampling() {
        let mut r = shape();
        r.x = 2.25;
        r.y = 2.25;
        r.radii = InteractionCornerRadii::default();
        assert!(hit(&raster(&r, frame(1.0)).unwrap(), 2, 6));
        let mask = raster_hotspot(&r, frame(1.0)).unwrap();
        assert!(!hit(&mask, 2, 6));
        assert!(hit(&mask, 3, 6));
    }
    #[test]
    fn integer_hotspot_mask_matches_existing_hit_predicate_at_three_dpis() {
        let mut polygon = shape();
        polygon.polygon = vec![
            InteractionPoint { x: 0.0, y: 0.0 },
            InteractionPoint { x: 8.0, y: 0.0 },
            InteractionPoint { x: 0.0, y: 8.0 },
        ];
        polygon.extra_rects = vec![InteractionExtraRect {
            x: 12.25,
            y: 2.25,
            width: 4.0,
            height: 4.0,
            radius: 1.0,
        }];
        for r in [shape(), polygon] {
            for scale in [1.0, 1.5, 2.0] {
                let mask = raster_hotspot(&r, frame(scale)).unwrap();
                for y in 0..40 {
                    for x in 0..40 {
                        assert_eq!(
                            hit(&mask, x, y),
                            point_in_rounded_rect(f64::from(x) / scale, f64::from(y) / scale, &r)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn headless_gdi_region_matches_rounded_mask_at_three_dpis() {
        use windows052::Win32::Graphics::Gdi::PtInRegion;
        for scale in [1.0, 1.5, 2.0] {
            let mask = raster(&shape(), frame(scale)).unwrap();
            let region = gdi_region(&mask).unwrap();
            for y in 0..40 {
                for x in 0..40 {
                    assert_eq!(
                        unsafe { PtInRegion(region.0, x, y) }.as_bool(),
                        hit(&mask, x, y)
                    );
                }
            }
            assert!(!unsafe { PtInRegion(region.0, -1, -1) }.as_bool());
        }
    }

    #[test]
    fn headless_gdi_empty_region_contains_no_points() {
        use windows052::Win32::Graphics::Gdi::PtInRegion;
        let region = gdi_region(&Mask(Vec::new())).unwrap();
        assert!(!unsafe { PtInRegion(region.0, 0, 0) }.as_bool());
        assert!(!unsafe { PtInRegion(region.0, 10, 10) }.as_bool());
    }
    #[test]
    fn malformed_scale_and_work_budget_fail_without_installing() {
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(raster(&shape(), frame(scale)).is_err());
        }
        assert!(raster(
            &shape(),
            Frame {
                width: u32::MAX,
                ..frame(1.0)
            }
        )
        .is_err());
        assert!(raster(
            &shape(),
            Frame {
                height: 0,
                ..frame(1.0)
            }
        )
        .is_err());
    }

    #[test]
    fn excessive_geometry_is_rejected_instead_of_truncated() {
        let mut r = shape();
        r.polygon = vec![InteractionPoint { x: 0.0, y: 0.0 }; MAX_VERTICES + 1];
        assert!(raster(&r, frame(1.0)).is_err());
        r.polygon.clear();
        r.extra_rects = vec![
            InteractionExtraRect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
                radius: 0.0
            };
            MAX_EXTRA_RECTS + 1
        ];
        assert!(raster(&r, frame(1.0)).is_err());
    }
    #[test]
    fn stale_revision_cannot_override_newer_geometry() {
        let mut s = Session::new(1);
        let mut b = Fake::default();
        let old = s.prepare(1, &shape(), frame(1.0)).unwrap();
        let new = s.prepare(2, &shape(), frame(2.0)).unwrap();
        assert!(s.commit(old, &mut b).is_err());
        assert_eq!(b.calls, 0);
        s.commit(new, &mut b).unwrap();
        assert!(s.ready);
        assert!(s.prepare(2, &shape(), frame(1.0)).is_err());
    }
    #[test]
    fn dpi_hide_restore_drag_changes_invalidate_queued_work() {
        for _event in [
            "dpi",
            "hide",
            "restore",
            "client-offset",
            "capture-start",
            "capture-cancel",
        ] {
            let mut s = Session::new(1);
            let mut b = Fake::default();
            let old = s.prepare(1, &shape(), frame(1.0)).unwrap();
            s.invalidate();
            assert!(s.commit(old, &mut b).is_err());
            assert_eq!(b.calls, 0);
            let new = s.prepare(2, &shape(), frame(1.5)).unwrap();
            s.commit(new, &mut b).unwrap();
            assert!(s.ready);
        }
    }
    #[test]
    fn native_failure_preserves_last_success_and_is_not_reported_ready() {
        let mut s = Session::new(1);
        let mut b = Fake::default();
        let first = s.prepare(1, &shape(), frame(1.0)).unwrap();
        s.commit(first, &mut b).unwrap();
        let original = s.last_applied;
        b.fail = true;
        let next = s.prepare(2, &shape(), frame(2.0)).unwrap();
        assert_eq!(s.commit(next, &mut b), Err("native install failed"));
        assert!(!s.ready);
        assert_eq!(s.last_applied, original);
        b.fail = false;
        let retry = s.prepare(3, &shape(), frame(2.0)).unwrap();
        s.commit(retry, &mut b).unwrap();
        assert!(s.ready);
    }
    #[test]
    fn destroyed_or_reused_window_never_receives_old_work() {
        let mut a = Session::new(1);
        let mut b = Fake::default();
        let old = a.prepare(1, &shape(), frame(1.0)).unwrap();
        let old_for_destroyed = a.prepare(2, &shape(), frame(1.0)).unwrap();
        a.destroy();
        assert!(!a.ready);
        assert!(a.commit(old_for_destroyed, &mut b).is_err());
        assert!(a.prepare(2, &shape(), frame(1.0)).is_err());
        let mut replacement = Session::new(2);
        assert!(replacement.commit(old, &mut b).is_err());
        assert_eq!(b.calls, 0);
    }
    #[test]
    fn raster_failure_cannot_acknowledge_new_geometry() {
        let mut s = Session::new(1);
        let mut b = Fake::default();
        let draft = s.prepare(1, &shape(), frame(1.0)).unwrap();
        s.commit(draft, &mut b).unwrap();
        assert!(s.prepare(2, &shape(), frame(f64::NAN)).is_err());
        assert!(!s.ready);
        assert_eq!(s.last_applied.unwrap().revision, 1);
        assert_eq!(b.calls, 1);
    }
}
