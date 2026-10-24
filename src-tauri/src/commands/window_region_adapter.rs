//! Feature-gated, immutable static-mask lab. Never installs on main/product UI.
use super::region_prototype::{raster_hotspot, Frame, Mask};
use super::{ISLAND_INTERACTION_REGION, ISLAND_INTERACTION_REGION_ACCEPTED_REVISION};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use windows052::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::*,
    System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::*,
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

pub(super) const LABEL: &str = "a5-region-static-lab";
static NEXT: AtomicU64 = AtomicU64::new(1);
static OWNER: Mutex<Option<Owner>> = Mutex::new(None);

// Actual initialization policy, also exercised without constructing a WebView.
#[derive(Debug)]
struct LoadGate {
    ticket: Ticket,
    document_url: String,
    receipt: String,
    hwnd: Option<isize>,
    armed: bool,
    navigation: bool,
    bootstrap_navigation: bool,
    loaded: bool,
    configured: bool,
    queued: bool,
    started: u32,
    finished: u32,
    stage: &'static str,
    failure: Option<String>,
    failure_stage: Option<&'static str>,
}
impl LoadGate {
    fn new(ticket: Ticket) -> Self {
        let receipt = format!(
            "a5-static-{}-{}-{}:loaded",
            std::process::id(),
            ticket.generation,
            ticket.revision
        );
        // No external resources, capabilities, product UI or untrusted interpolation.
        let html = format!("<!doctype html><html><head><meta charset=utf-8></head><body><script>addEventListener('load',()=>{{document.title='{receipt}'}},{{once:true}})</script></body></html>");
        let encoded: String = html.bytes().map(|b| format!("%{b:02X}")).collect();
        Self {
            ticket,
            document_url: format!("data:text/html;charset=utf-8,{encoded}"),
            receipt,
            hwnd: None,
            armed: false,
            navigation: false,
            bootstrap_navigation: false,
            loaded: false,
            configured: false,
            queued: false,
            started: 0,
            finished: 0,
            stage: "constructing",
            failure: None,
            failure_stage: None,
        }
    }
    fn fail(&mut self, reason: &str) {
        if self.failure.is_none() {
            self.failure_stage = Some(self.stage);
            self.stage = "failed";
            self.failure = Some(reason.into());
        }
    }
    fn navigation(&mut self, url: &str) -> bool {
        if self.failure.is_some() {
            return false;
        }
        if !self.navigation && !self.bootstrap_navigation && url == "about:blank" {
            self.bootstrap_navigation = true;
            return true;
        }
        if self.armed && !self.navigation && url == self.document_url {
            self.navigation = true;
            self.stage = "awaiting-document-receipt";
            return true;
        }
        self.fail("unexpected navigation/reload; no replay");
        false
    }
    fn receipt(&mut self, ticket: Ticket, hwnd: isize, title: &str, url: &str) -> bool {
        if self.failure.is_some()
            || ticket != self.ticket
            || self.hwnd != Some(hwnd)
            || !self.navigation
            || url != self.document_url
            || title != self.receipt
        {
            return false;
        }
        self.loaded = true;
        if !self.queued {
            self.stage = "document-loaded";
        }
        true
    }
    fn claim(&mut self, ticket: Ticket, hwnd: isize) -> bool {
        if self.failure.is_some()
            || ticket != self.ticket
            || self.hwnd != Some(hwnd)
            || !self.loaded
            || !self.configured
            || self.queued
        {
            return false;
        }
        self.queued = true;
        self.stage = "dispatching";
        true
    }
    fn snapshot(&self) -> Value {
        json!({"loaded":self.loaded,"configured":self.configured,
        "queued":self.queued,"armed":self.armed,"navigationAccepted":self.navigation,"bootstrapNavigation":self.bootstrap_navigation,
        "hwnd":self.hwnd,"documentUrl":self.document_url,"documentReceipt":self.receipt,
        "pageStarted":self.started,"pageFinished":self.finished,"stage":self.stage,"failure":self.failure,"failureStage":self.failure_stage})
    }
}
// UI getters may dispatch and wait. Never hold gate/OWNER across a provider;
// callbacks on that UI thread need the same gate to make progress.
fn configure_document(
    gate: &Mutex<LoadGate>,
    handle_provider: impl FnOnce() -> Result<isize, String>,
) -> Result<String, String> {
    let handle = handle_provider()?;
    if handle == 0 {
        return Err("null fixture HWND".into());
    }
    let mut g = gate.lock().map_err(|e| e.to_string())?;
    if g.failure.is_some() {
        return Err("initialization already invalidated".into());
    }
    g.hwnd = Some(handle);
    g.configured = true;
    g.armed = true;
    g.stage = "navigation-requested";
    Ok(g.document_url.clone())
}

fn schedule(
    window: &WebviewWindow,
    ticket: Ticket,
    source: Arc<super::IslandInteractionRegion>,
    gate: Arc<Mutex<LoadGate>>,
) {
    let h = match hwnd(window) {
        Ok(h) => h.0,
        Err(e) => {
            invalidate(ticket, &e);
            return;
        }
    };
    if !gate.lock().map(|mut g| g.claim(ticket, h)).unwrap_or(false) {
        return;
    }
    let target = window.clone();
    let applying_gate = gate.clone();
    if let Err(e) = window.run_on_main_thread(move || {
        if let Ok(mut g) = applying_gate.lock() {
            if g.failure.is_some() {
                return;
            }
            g.stage = "applying";
        }
        match apply_static(&target, ticket, &source) {
            Ok(()) => {
                if let Ok(mut g) = applying_gate.lock() {
                    if g.failure.is_none() {
                        g.stage = "applied";
                    }
                }
            }
            Err(e) => invalidate(ticket, &format!("native apply failed: {e}")),
        }
    }) {
        invalidate(ticket, &format!("UI enqueue failed: {e}"));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Enqueued,
    Applying,
    Applied,
    Invalid,
    Closed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ticket {
    generation: u64,
    revision: u64,
}
struct Owner {
    ticket: Ticket,
    phase: Phase,
    native_applied: bool,
    error: Option<String>,
    frame: Option<Frame>,
    init: Option<Arc<Mutex<LoadGate>>>,
}
impl Owner {
    fn require_current(&self, ticket: Ticket) -> Result<(), String> {
        if self.ticket == ticket {
            Ok(())
        } else {
            Err("stale window generation/revision".into())
        }
    }
    fn begin(&mut self, ticket: Ticket) -> Result<(), String> {
        self.require_current(ticket)?;
        if self.phase != Phase::Enqueued {
            return Err("stale or unsupported work".into());
        }
        self.phase = Phase::Applying;
        Ok(())
    }
    fn finish(&mut self, ticket: Ticket, result: Result<(), String>) -> Result<(), String> {
        if self.ticket != ticket {
            return Err("completion belongs to another window generation".into());
        }
        // Even a completion invalidated reentrantly must retain the successful
        // OS transfer as history; ready remains false in that case.
        if result.is_ok() && self.phase != Phase::Closed {
            self.native_applied = true;
        }
        if self.phase != Phase::Applying {
            self.error = Some("native transaction completed after invalidation; not ready".into());
            return Err("native transaction completed after invalidation; not ready".into());
        }
        match result {
            Ok(()) => {
                self.native_applied = true;
                self.phase = Phase::Applied;
                Ok(())
            }
            Err(e) => {
                self.phase = Phase::Invalid;
                self.error = Some(e.clone());
                Err(e)
            }
        }
    }
}

// Local to the dedicated host, not a global input hook. No user-data allocation
// is kept by native code: the subclass ID is the monotonic window generation.
fn may_reveal(phase: Option<Phase>) -> bool {
    phase == Some(Phase::Applied)
}
fn can_reset_absent(phase: Phase) -> bool {
    phase == Phase::Invalid || phase == Phase::Closed
}
unsafe extern "system" fn lifecycle(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    _: usize,
) -> LRESULT {
    let ticket = OWNER.lock().ok().and_then(|g| {
        g.as_ref()
            .filter(|o| o.ticket.generation == id as u64)
            .map(|o| o.ticket)
    });
    if ticket.is_none() {
        if message == WM_WINDOWPOSCHANGING && lparam.0 != 0 && !may_reveal(None) {
            // Missing/poisoned ownership never grants reveal. Do not force a
            // hide here: an existing press may still need its natural release.
            let pos = &mut *(lparam.0 as *mut WINDOWPOS);
            pos.flags &= !SWP_SHOWWINDOW;
        }
        if message == WM_NCDESTROY {
            if !RemoveWindowSubclass(window, Some(lifecycle), id).as_bool() {
                tracing::error!("[region-lab] orphan subclass removal unconfirmed");
            }
        }
        return DefSubclassProc(window, message, wparam, lparam);
    }
    if let Some(ticket) = ticket {
        match message {
            WM_SHOWWINDOW if wparam.0 == 0 => invalidate(
                ticket,
                "native hide invalidates immutable lab; restore unsupported",
            ),
            WM_ENTERSIZEMOVE | WM_CANCELMODE | WM_CAPTURECHANGED | WM_DPICHANGED => invalidate(
                ticket,
                "host drag/capture/cancel/DPI change unsupported; old shape retained for release",
            ),
            WM_WINDOWPOSCHANGING if lparam.0 != 0 => {
                let pos = &mut *(lparam.0 as *mut WINDOWPOS);
                let phase = OWNER
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().filter(|o| o.ticket == ticket).map(|o| o.phase));
                if pos.flags.contains(SWP_SHOWWINDOW) && !may_reveal(phase) {
                    pos.flags = (pos.flags & !SWP_SHOWWINDOW) | SWP_HIDEWINDOW;
                }
                if pos.flags.contains(SWP_HIDEWINDOW) && busy(window) {
                    pos.flags &= !SWP_HIDEWINDOW;
                    invalidate(
                        ticket,
                        "native hide deferred: input gate busy/unavailable; release preserved",
                    );
                }
            }
            WM_NCDESTROY => {
                if !RemoveWindowSubclass(window, Some(lifecycle), id).as_bool() {
                    tracing::error!("[region-lab] subclass removal unconfirmed during destruction");
                }
                if let Ok(mut guard) = OWNER.lock() {
                    if let Some(o) = guard.as_mut().filter(|o| o.ticket == ticket) {
                        o.phase = Phase::Closed;
                        o.native_applied = false;
                    }
                }
            }
            _ => {}
        }
    }
    DefSubclassProc(window, message, wparam, lparam)
}

// This is the actual ownership policy, shared with fault tests. A successful
// install transfers the destination. No delete/query/reuse follows that transfer.
trait Api {
    fn create(&mut self, l: i32, t: i32, r: i32, b: i32) -> Result<isize, String>;
    fn union(&mut self, dst: isize, row: isize) -> Result<(), String>;
    fn install(&mut self, region: isize) -> Result<(), String>;
    fn delete(&mut self, region: isize) -> Result<(), String>;
}
fn release(api: &mut impl Api, raw: isize, cause: String) -> String {
    match api.delete(raw) {
        Ok(()) => cause,
        Err(e) => format!("{cause}; cleanup of owned HRGN {raw} unconfirmed: {e}"),
    }
}
fn transfer(api: &mut impl Api, mask: &Mask) -> Result<(), String> {
    if mask.0.is_empty() {
        return Err("empty mask refused; not a readiness fallback".into());
    }
    let dst = api.create(0, 0, 0, 0)?;
    for s in &mask.0 {
        let row = match api.create(s.left, s.y, s.right, s.y + 1) {
            Ok(row) => row,
            Err(e) => return Err(release(api, dst, e)),
        };
        let combined = api.union(dst, row);
        let deleted = api.delete(row);
        if let Err(e) = combined {
            let e = match deleted {
                Ok(()) => e,
                Err(d) => format!("{e}; owned row HRGN {row} cleanup unconfirmed: {d}"),
            };
            return Err(release(api, dst, e));
        }
        if let Err(e) = deleted {
            return Err(release(
                api,
                dst,
                format!("owned row HRGN {row} cleanup unconfirmed: {e}"),
            ));
        }
    }
    match api.install(dst) {
        Ok(()) => Ok(()),
        Err(e) => Err(release(api, dst, e)),
    }
}
struct Win32 {
    hwnd: HWND,
}
impl Api for Win32 {
    fn create(&mut self, l: i32, t: i32, r: i32, b: i32) -> Result<isize, String> {
        let h = unsafe { CreateRectRgn(l, t, r, b) };
        if h.0 == 0 {
            Err("CreateRectRgn failed".into())
        } else {
            Ok(h.0)
        }
    }
    fn union(&mut self, dst: isize, row: isize) -> Result<(), String> {
        if unsafe { CombineRgn(HRGN(dst), HRGN(dst), HRGN(row), RGN_OR) }.0 == ERROR {
            Err("CombineRgn failed".into())
        } else {
            Ok(())
        }
    }
    fn install(&mut self, region: isize) -> Result<(), String> {
        if unsafe { GetWindowThreadProcessId(self.hwnd, None) } != unsafe { GetCurrentThreadId() } {
            return Err("SetWindowRgn requires this adapter's owning UI thread".into());
        }
        if busy(self.hwnd) {
            return Err("capture/buttons active; immutable experiment refuses apply".into());
        }
        if unsafe { SetWindowRgn(self.hwnd, HRGN(region), true) } == 0 {
            Err("SetWindowRgn failed; temporary HRGN remains owned".into())
        } else {
            Ok(())
        }
    }
    fn delete(&mut self, region: isize) -> Result<(), String> {
        if unsafe { DeleteObject(HRGN(region)) }.as_bool() {
            Ok(())
        } else {
            Err("DeleteObject failed".into())
        }
    }
}
fn input_blocked(own_foreground: bool, buttons: bool, captures: &[Result<bool, ()>]) -> bool {
    !own_foreground || buttons || captures.is_empty() || captures.iter().any(|v| *v != Ok(false))
}
fn busy(root: HWND) -> bool {
    struct Threads {
        ids: Vec<u32>,
        overflow: bool,
    }
    unsafe extern "system" fn child(window: HWND, data: LPARAM) -> BOOL {
        let threads = &mut *(data.0 as *mut Threads);
        let id = GetWindowThreadProcessId(window, None);
        if !threads.ids.contains(&id) {
            threads.ids.push(id);
        }
        if threads.ids.len() > 128 {
            threads.overflow = true;
            return BOOL(0);
        }
        BOOL(1)
    }
    unsafe {
        let mut foreground_pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut foreground_pid));
        let mut threads = Threads {
            ids: vec![
                GetCurrentThreadId(),
                GetWindowThreadProcessId(root, None),
                0,
            ],
            overflow: false,
        };
        let _ = EnumChildWindows(
            root,
            Some(child),
            LPARAM(&mut threads as *mut Threads as isize),
        );
        if threads.overflow {
            return true;
        }
        let captures: Vec<_> = threads
            .ids
            .into_iter()
            .map(|id| {
                let mut info = GUITHREADINFO {
                    cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                    ..Default::default()
                };
                GetGUIThreadInfo(id, &mut info)
                    .map(|_| info.hwndCapture.0 != 0 || info.hwndMoveSize.0 != 0)
                    .map_err(|_| ())
            })
            .collect();
        let buttons = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
            .iter()
            .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0);
        input_blocked(foreground_pid == GetCurrentProcessId(), buttons, &captures)
    }
}
fn hwnd(window: &WebviewWindow) -> Result<HWND, String> {
    window
        .hwnd()
        .map(|h| HWND(h.0 as _))
        .map_err(|e| e.to_string())
}
fn frame(window: &WebviewWindow) -> Result<Frame, String> {
    let h = hwnd(window)?;
    let mut rect = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        GetWindowRect(h, &mut rect).map_err(|e| e.to_string())?;
        if !ClientToScreen(h, &mut origin).as_bool() {
            return Err("ClientToScreen failed".into());
        }
    }
    Ok(Frame {
        width: (rect.right - rect.left)
            .try_into()
            .map_err(|_| "invalid width")?,
        height: (rect.bottom - rect.top)
            .try_into()
            .map_err(|_| "invalid height")?,
        scale: window.scale_factor().map_err(|e| e.to_string())?,
        client_offset: (origin.x - rect.left, origin.y - rect.top),
    })
}
fn invalidate(ticket: Ticket, reason: &str) {
    if let Ok(mut guard) = OWNER.lock() {
        if let Some(owner) = guard
            .as_mut()
            .filter(|o| o.ticket == ticket && o.phase != Phase::Closed)
        {
            if let Some(gate) = &owner.init {
                if let Ok(mut g) = gate.lock() {
                    g.fail(reason);
                }
            }
            owner.phase = Phase::Invalid;
            owner.error = Some(reason.into());
        }
    }
    tracing::warn!("[region-lab] invalidated: {reason}; no automatic restore/replay");
}
fn status() -> Result<Value, String> {
    let guard = OWNER.lock().map_err(|e| e.to_string())?;
    let mut result = match guard.as_ref() {
        Some(o) => json!({"mode":"static-mask-lab", "generation":o.ticket.generation,
            "desiredRevision":o.ticket.revision, "phase":format!("{:?}",o.phase),
            "nativeApplied":o.native_applied,"nativeReady":o.phase==Phase::Applied,
            "visualVerified":false,"productCompatible":false,"error":o.error,
            "initialization":o.init.as_ref().and_then(|g|g.lock().ok().map(|g|g.snapshot()))}),
        None => json!({"phase":"absent", "nativeReady":false}),
    };
    result["sourceAcceptedRevision"] =
        json!(ISLAND_INTERACTION_REGION_ACCEPTED_REVISION.load(Ordering::Acquire));
    Ok(result)
}

pub(super) async fn open(app: AppHandle, revision: u64, mode: String) -> Result<Value, String> {
    if mode != "static-mask-lab" {
        return Err("only explicit static-mask-lab supported; no product/hover/animation".into());
    }
    let region = {
        let r = ISLAND_INTERACTION_REGION
            .get()
            .ok_or("no accepted source geometry")?
            .lock()
            .map_err(|e| e.to_string())?;
        if revision == 0
            || revision != ISLAND_INTERACTION_REGION_ACCEPTED_REVISION.load(Ordering::Acquire)
        {
            return Err("source revision mismatch".into());
        }
        r.clone()
    };
    let ticket = Ticket {
        generation: NEXT.fetch_add(1, Ordering::AcqRel),
        revision,
    };
    let gate = Arc::new(Mutex::new(LoadGate::new(ticket)));
    {
        let mut guard = OWNER.lock().map_err(|e| e.to_string())?;
        if guard.as_ref().is_some_and(|o| o.phase != Phase::Closed) {
            return Err("close old experiment before opening; no live mutation".into());
        }
        *guard = Some(Owner {
            ticket,
            phase: Phase::Enqueued,
            native_applied: false,
            error: None,
            frame: None,
            init: Some(gate.clone()),
        });
    }
    let handle = app.clone();
    // Same 10 s budget as the coordinator; failure becomes observable even
    // when no document event arrives. This never fabricates readiness.
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let pending = OWNER
            .lock()
            .ok()
            .and_then(|g| {
                g.as_ref()
                    .filter(|o| o.ticket == ticket)
                    .map(|o| matches!(o.phase, Phase::Enqueued | Phase::Applying))
            })
            .unwrap_or(false);
        if pending {
            invalidate(ticket,"initialization 10 s deadline: inspect initialization stage/loaded/configured/queued");
        }
    });
    let source = Arc::new(region);
    let built = tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let main = handle.get_webview_window("main").ok_or("main unavailable")?;
        if main.is_visible().map_err(|e|e.to_string())? {
            return Err("hide the source main before isolated lab creation; overlapping legacy target is unsupported".into());
        }
        let size = main.outer_size().map_err(|e| e.to_string())?;
        let position = main.outer_position().map_err(|e| e.to_string())?;
        // A blank, non-interactive fixture: no capabilities, product code, drag
        // handlers, animation or navigation. It deliberately is not a UI clone.
        let callback_gate=gate.clone(); let callback_source=source.clone();
        let navigation_gate=gate.clone(); let page_gate=gate.clone();
        let w = WebviewWindowBuilder::new(&handle, LABEL, WebviewUrl::External("about:blank".parse().map_err(|_| "invalid fixture URL")?))
            .title("A5 static region mechanism experiment").visible(false).decorations(false)
            .resizable(false).shadow(false).focused(false).focusable(false).skip_taskbar(true)
            .transparent(true).always_on_top(true)
            .on_navigation(move |url| {
                let accepted=navigation_gate.lock().map(|mut g|g.navigation(url.as_str())).unwrap_or(false);
                if !accepted { invalidate(ticket,"unexpected navigation/reload; no replay"); } accepted
            })
            .on_page_load(move |_window,payload| {
                // Observations only: wry reads current URI, so Finished alone
                // cannot distinguish a delayed bootstrap completion.
                if let Ok(mut g)=page_gate.lock() {
                    if payload.event()==tauri::webview::PageLoadEvent::Started {g.started=g.started.saturating_add(1);}
                    else {g.finished=g.finished.saturating_add(1);}
                }
            })
            .on_document_title_changed(move |window,title| {
                let accepted=hwnd(&window).ok().and_then(|h|window.url().ok().map(|u|(h,u)))
                    .map(|(h,u)|callback_gate.lock().map(|mut g|g.receipt(ticket,h.0,&title,u.as_str())).unwrap_or(false)).unwrap_or(false);
                if accepted { schedule(&window,ticket,callback_source.clone(),callback_gate.clone()); }
            }).build().map_err(|e| e.to_string())?;
        // Still hidden: placement changes can never expose a full-window target.
        w.set_size(size).map_err(|e| e.to_string())?;
        w.set_position(position).map_err(|e| e.to_string())?;
        let observed = w.clone();
        w.on_window_event(move |event| match event {
            WindowEvent::Destroyed => {
                if let Ok(mut guard) = OWNER.lock() { if let Some(o) = guard.as_mut().filter(|o| o.ticket == ticket) { o.phase=Phase::Closed; o.native_applied=false; } }
            }
            WindowEvent::CloseRequested { api, .. } => {
                if hwnd(&observed).map(busy).unwrap_or(true) { api.prevent_close(); invalidate(ticket, "close deferred: input gate busy/unavailable; requires foreground Isle and no held input; explicitly retry"); }
            }
            WindowEvent::Moved(_) | WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                let current = frame(&observed);
                let changed = OWNER.lock().map(|g| g.as_ref().filter(|o| o.ticket == ticket)
                    .is_some_and(|o| o.frame.is_some() && current.as_ref().ok() != o.frame.as_ref())).unwrap_or(true);
                if changed { invalidate(ticket, "position/frame/DPI lifecycle changed; close and reopen, restore unsupported"); }
            }
            _ => {}
        });
        let document_url=configure_document(&gate,||hwnd(&w).map(|h|h.0))?;
        // Explicit full-document navigation occurs after construction/placement
        // and after all callbacks are installed; bootstrap completion is ignored.
        w.navigate(document_url.parse().map_err(|_|"invalid fixture document URL")?).map_err(|e|format!("fixture navigate failed: {e}"))?;
        schedule(&w,ticket,source,gate);
        Ok(())
    }).await.unwrap_or_else(|e| Err(format!("window construction task failed: {e}")));
    if let Err(e) = built {
        invalidate(ticket, &e);
        return Err(e);
    }
    // This acknowledges creation/enqueue only. Status tracks native completion.
    Ok(
        json!({"generation":ticket.generation,"desiredRevision":revision,"enqueued":true,
        "ack":"creation/enqueue only; query status for completion","mode":"static-mask-lab","query":"experimental_region_control/status"}),
    )
}

fn apply_static(
    window: &WebviewWindow,
    ticket: Ticket,
    region: &super::IslandInteractionRegion,
) -> Result<(), String> {
    if window
        .app_handle()
        .get_webview_window("main")
        .ok_or("source main absent")?
        .is_visible()
        .map_err(|e| e.to_string())?
    {
        return Err("source main became visible; overlapping legacy target unsupported".into());
    }
    let h = hwnd(window)?;
    if unsafe { GetWindowThreadProcessId(h, None) } != unsafe { GetCurrentThreadId() } {
        return Err("not HWND UI thread".into());
    }
    if window.is_visible().map_err(|e| e.to_string())? {
        return Err("live apply/restore unsupported; window must be hidden".into());
    }
    if busy(h) {
        return Err("capture/drag/button-held state unsupported; no input discarded".into());
    }
    {
        OWNER
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("owner absent")?
            .begin(ticket)?;
    }
    let f = frame(window)?;
    if !unsafe { SetWindowSubclass(h, Some(lifecycle), ticket.generation as usize, 0) }.as_bool() {
        return Err(
            "host lifecycle subclass installation failed; experiment remains hidden".into(),
        );
    }
    // Tao and native state stay consistent: only its setter runs, on its own UI
    // thread. There is no raw SetWindowLong override and no lab cursor monitor.
    window
        .set_ignore_cursor_events(false)
        .map_err(|e| e.to_string())?;
    let style = unsafe { GetWindowLongPtrW(h, GWL_EXSTYLE) } as u32;
    if style & (WS_EX_TRANSPARENT.0 | WS_EX_LAYOUTRTL.0) != 0 {
        return Err("transparent/RTL state refused after tao setter".into());
    }
    let mask = raster_hotspot(region, f).map_err(str::to_string)?;
    // Geometry preview has no product shadow, text, cover or controls. Native
    // ready does not assert a browser paint/presentation or visual compatibility.
    let spans: Vec<_> = mask
        .0
        .iter()
        .map(|s| json!([s.left, s.y, s.right - s.left, 1]))
        .collect();
    let script = format!("document.documentElement.style.cssText='margin:0;background:transparent;overflow:hidden;user-select:none;pointer-events:none';document.body.style.cssText='margin:0';const c=document.createElement('canvas');c.width={};c.height={};c.style.cssText='width:100vw;height:100vh;pointer-events:none';document.body.replaceChildren(c);const ctx=c.getContext('2d');ctx.fillStyle='#15151c';for(const r of {})ctx.fillRect(...r);", f.width, f.height, serde_json::to_string(&spans).map_err(|e| e.to_string())?);
    if f.client_offset != (0, 0) {
        return Err(
            "nonzero client-to-window offset unsupported by static preview; no ready".into(),
        );
    }
    window.eval(script).map_err(|e| e.to_string())?;
    {
        let mut guard = OWNER.lock().map_err(|e| e.to_string())?;
        let o = guard.as_mut().ok_or("owner absent")?;
        if o.ticket != ticket || o.phase != Phase::Applying {
            return Err("stale before native transfer".into());
        }
        o.frame = Some(f);
    }
    let result = transfer(&mut Win32 { hwnd: h }, &mask);
    // No owner lock was held across SetWindowRgn's synchronous window messages.
    {
        OWNER
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .ok_or("owner absent")?
            .finish(ticket, result)?;
    }
    if frame(window)? != f {
        invalidate(ticket, "frame changed during apply");
        return Err("stale physical frame".into());
    }
    window.show().map_err(|e| e.to_string())?;
    tracing::warn!(
        "[region-lab] native region applied, static geometry only; visual/OS routing UNVERIFIED"
    );
    Ok(())
}

pub(super) async fn control(app: AppHandle, action: String) -> Result<Value, String> {
    if action != "close" && action != "hide" && action != "status" {
        return Err(
            "only status/hide/close; restore, drag, animation and live mutation unsupported".into(),
        );
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let target = match app.get_webview_window(LABEL) {
        Some(w) => w,
        None if action == "status" => return status(),
        None if action == "close" => {
            {
                let mut guard = OWNER.lock().map_err(|e| e.to_string())?;
                if let Some(o) = guard.as_mut() {
                    if !can_reset_absent(o.phase) {
                        return Err(
                            "creation/apply still pending; cannot reset absent window".into()
                        );
                    }
                    o.phase = Phase::Closed;
                    o.native_applied = false;
                }
            }
            return status();
        }
        None => return Err("experiment absent".into()),
    };
    let expected = OWNER
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .ok_or("owner absent")?
        .ticket;
    let w = target.clone();
    target
        .run_on_main_thread(move || {
            let result = (|| -> Result<Value, String> {
                { OWNER.lock().map_err(|e|e.to_string())?.as_ref().ok_or("owner absent")?.require_current(expected)?; }
                if action=="status" {
                    let (phase, original_frame)={ let g=OWNER.lock().map_err(|e|e.to_string())?;let o=g.as_ref().ok_or("owner absent")?;(o.phase,o.frame) };
                    if phase==Phase::Applied {
                        let current=frame(&w);
                        let style=hwnd(&w).map(|h|unsafe{GetWindowLongPtrW(h,GWL_EXSTYLE)} as u32);
                        let source_visible=w.app_handle().get_webview_window("main").map(|m|m.is_visible().unwrap_or(true)).unwrap_or(true);
                        if source_visible || !w.is_visible().unwrap_or(false) || current.ok()!=original_frame
                            || style.map(|s|s & (WS_EX_TRANSPARENT.0|WS_EX_LAYOUTRTL.0)!=0).unwrap_or(true)
                            || hwnd(&w).map(busy).unwrap_or(true) {
                            invalidate(expected,"static supported-state contract no longer met; no ready, close/reopen required");
                        }
                    }
                    return status();
                }
                if hwnd(&w).map(busy).unwrap_or(true) {
                    return Err(
                        "buttons/capture held; release preserved; explicitly retry close/hide"
                            .into(),
                    );
                }
                invalidate(expected, "explicit hide/close; no restore");
                if action == "close" {
                    w.close().map_err(|e| e.to_string())?;
                } else {
                    w.hide().map_err(|e| e.to_string())?;
                }
                status()
            })();
            let _ = tx.send(result);
        })
        .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::super::region_prototype::Span;
    use super::*;
    #[derive(Default)]
    struct Fake {
        next: isize,
        events: Vec<String>,
        fail: Option<&'static str>,
        transferred: Vec<isize>,
    }
    impl Api for Fake {
        fn create(&mut self, _: i32, _: i32, _: i32, _: i32) -> Result<isize, String> {
            self.next += 1;
            self.events.push(format!("create{}", self.next));
            if self.fail
                == Some(if self.next == 1 {
                    "create-dst"
                } else {
                    "create-row"
                })
            {
                Err("create failed".into())
            } else {
                Ok(self.next)
            }
        }
        fn union(&mut self, _: isize, _: isize) -> Result<(), String> {
            self.events.push("union".into());
            if self.fail == Some("union") {
                Err("union failed".into())
            } else {
                Ok(())
            }
        }
        fn install(&mut self, h: isize) -> Result<(), String> {
            self.events.push(format!("install{h}"));
            if self.fail == Some("install") {
                Err("install failed".into())
            } else {
                self.transferred.push(h);
                Ok(())
            }
        }
        fn delete(&mut self, h: isize) -> Result<(), String> {
            assert!(!self.transferred.contains(&h));
            self.events.push(format!("delete{h}"));
            if self.fail == Some("delete") {
                Err("delete failed".into())
            } else {
                Ok(())
            }
        }
    }
    fn mask() -> Mask {
        Mask(vec![Span {
            y: 0,
            left: 2,
            right: 5,
        }])
    }
    #[test]
    fn successful_transfer_never_deletes_system_owned_region() {
        let mut a = Fake::default();
        transfer(&mut a, &mask()).unwrap();
        assert_eq!(
            a.events,
            vec!["create1", "create2", "union", "delete2", "install1"]
        );
        assert_eq!(a.transferred, vec![1]);
    }
    #[test]
    fn configuration_provider_can_run_callbacks_without_the_gate_lock() {
        let t = Ticket {
            generation: 11,
            revision: 2,
        };
        let gate = Mutex::new(LoadGate::new(t));
        let url = configure_document(&gate, || {
            let mut g = gate
                .try_lock()
                .expect("UI provider must acquire the actual gate");
            assert!(g.navigation("about:blank"));
            assert!(!g.configured);
            Ok(123)
        })
        .unwrap();
        let g = gate.lock().unwrap();
        assert_eq!(g.hwnd, Some(123));
        assert!(g.configured && g.armed);
        assert_eq!(g.document_url, url);
        drop(g);
        let failed = Mutex::new(LoadGate::new(t));
        assert!(configure_document(&failed, || {
            assert!(failed.try_lock().is_ok());
            Err("provider failed".into())
        })
        .is_err());
        assert!(!failed.lock().unwrap().configured);
        let invalid = Mutex::new(LoadGate::new(t));
        assert!(configure_document(&invalid, || {
            invalid.lock().unwrap().fail("callback invalidated");
            Ok(123)
        })
        .is_err());
        assert!(!invalid.lock().unwrap().configured);
    }
    #[test]
    fn fixture_protocol_orders_identity_once_and_failure() {
        let t = Ticket {
            generation: 41,
            revision: 7,
        };
        for load_first in [true, false] {
            let mut g = LoadGate::new(t);
            let url = g.document_url.clone();
            let receipt = g.receipt.clone();
            assert_eq!(url.parse::<tauri::Url>().unwrap().as_str(), url);
            assert!(g.navigation("about:blank"));
            assert!(!g.claim(t, 123));
            assert!(!g.receipt(t, 123, &receipt, &url));
            g.hwnd = Some(123);
            g.armed = true;
            assert!(g.navigation(&url));
            if !load_first {
                g.configured = true;
            }
            assert!(!g.receipt(
                Ticket {
                    generation: 40,
                    ..t
                },
                123,
                &receipt,
                &url
            ));
            assert!(!g.receipt(t, 124, &receipt, &url));
            assert!(!g.receipt(t, 123, &receipt, "about:blank"));
            assert!(g.receipt(t, 123, &receipt, &url));
            assert_eq!(g.claim(t, 123), !load_first);
            g.configured = true;
            assert_eq!(g.claim(t, 123), load_first);
            assert!(g.receipt(t, 123, &receipt, &url));
            assert!(!g.claim(t, 123));
            assert!(!g.navigation(&url));
            assert!(g.failure.is_some());
        }
        let mut late = LoadGate::new(t);
        late.hwnd = Some(123);
        late.armed = true;
        late.configured = true;
        assert!(late.navigation("about:blank"));
        assert!(!late.loaded);
        assert!(!late.claim(t, 123));
        let url = late.document_url.clone();
        let receipt = late.receipt.clone();
        assert!(late.navigation(&url));
        assert!(late.receipt(t, 123, &receipt, &url));
        assert!(late.claim(t, 123));
        assert!(!late.navigation("about:blank"));
        for reason in ["dispatch failed", "apply failed", "deadline"] {
            let mut g = LoadGate::new(t);
            g.hwnd = Some(123);
            g.configured = true;
            g.loaded = true;
            g.fail(reason);
            assert!(!g.claim(t, 123));
            assert_eq!(g.snapshot()["failure"], reason);
        }
    }
    #[test]
    fn uncertain_capture_foreground_and_held_input_all_refuse_mutation() {
        assert!(!input_blocked(true, false, &[Ok(false), Ok(false)]));
        assert!(input_blocked(false, false, &[Ok(false)]));
        assert!(input_blocked(true, true, &[Ok(false)]));
        assert!(input_blocked(true, false, &[Ok(true)]));
        assert!(input_blocked(true, false, &[Err(())]));
        assert!(input_blocked(true, false, &[]));
    }
    #[test]
    fn native_show_policy_rejects_pending_failed_hidden_destroyed_owners() {
        for phase in [
            None,
            Some(Phase::Enqueued),
            Some(Phase::Applying),
            Some(Phase::Invalid),
            Some(Phase::Closed),
        ] {
            assert!(!may_reveal(phase));
        }
        assert!(may_reveal(Some(Phase::Applied)));
    }
    #[test]
    fn destroyed_window_cannot_retain_applied_state_from_late_completion() {
        let t = Ticket {
            generation: 1,
            revision: 1,
        };
        let mut owner = Owner {
            ticket: t,
            phase: Phase::Closed,
            native_applied: false,
            error: None,
            frame: None,
            init: None,
        };
        assert!(owner.finish(t, Ok(())).is_err());
        assert!(!owner.native_applied);
        assert_eq!(owner.phase, Phase::Closed);
    }
    #[test]
    fn failed_creation_can_reset_but_pending_native_work_cannot_be_reused() {
        assert!(can_reset_absent(Phase::Invalid));
        assert!(can_reset_absent(Phase::Closed));
        for p in [Phase::Enqueued, Phase::Applying, Phase::Applied] {
            assert!(!can_reset_absent(p));
        }
    }
    #[test]
    fn install_failure_deletes_still_owned_destination() {
        let mut a = Fake {
            fail: Some("install"),
            ..Default::default()
        };
        assert!(transfer(&mut a, &mask()).is_err());
        assert_eq!(a.events.last().unwrap(), "delete1");
        assert!(a.transferred.is_empty());
    }
    #[test]
    fn row_allocation_failure_cleans_destination() {
        let mut a = Fake {
            fail: Some("create-row"),
            ..Default::default()
        };
        assert!(transfer(&mut a, &mask()).is_err());
        assert_eq!(a.events, vec!["create1", "create2", "delete1"]);
    }
    #[test]
    fn union_failure_cleans_both_owned_handles() {
        let mut a = Fake {
            fail: Some("union"),
            ..Default::default()
        };
        assert!(transfer(&mut a, &mask()).is_err());
        assert_eq!(&a.events[a.events.len() - 2..], &["delete2", "delete1"]);
    }
    #[test]
    fn delete_failure_is_not_reported_as_applied() {
        let mut a = Fake {
            fail: Some("delete"),
            ..Default::default()
        };
        let e = transfer(&mut a, &mask()).unwrap_err();
        assert!(e.contains("unconfirmed"));
        assert!(a.transferred.is_empty());
    }
    #[test]
    fn initial_allocation_and_empty_mask_fail_without_install() {
        let mut a = Fake {
            fail: Some("create-dst"),
            ..Default::default()
        };
        assert!(transfer(&mut a, &mask()).is_err());
        assert_eq!(a.events, vec!["create1"]);
        a.events.clear();
        assert!(transfer(&mut a, &Mask(vec![])).is_err());
        assert!(a.events.is_empty());
    }
    #[test]
    fn queue_ack_is_not_apply_and_stale_work_cannot_become_ready() {
        let t = Ticket {
            generation: 1,
            revision: 4,
        };
        let mut o = Owner {
            ticket: t,
            phase: Phase::Enqueued,
            native_applied: false,
            error: None,
            frame: None,
            init: None,
        };
        assert!(!o.native_applied);
        assert!(o.begin(Ticket { generation: 2, ..t }).is_err());
        o.begin(t).unwrap();
        o.phase = Phase::Invalid;
        assert!(o.finish(t, Ok(())).is_err());
        assert!(o.native_applied);
        assert_eq!(o.phase, Phase::Invalid);
    }
    #[test]
    fn backend_failure_is_explicit_and_not_ready() {
        let t = Ticket {
            generation: 1,
            revision: 4,
        };
        let mut o = Owner {
            ticket: t,
            phase: Phase::Enqueued,
            native_applied: false,
            error: None,
            frame: None,
            init: None,
        };
        o.begin(t).unwrap();
        assert!(o.finish(t, Err("native error".into())).is_err());
        assert_eq!(o.phase, Phase::Invalid);
        assert!(!o.native_applied);
        assert!(o.error.is_some());
    }
}
