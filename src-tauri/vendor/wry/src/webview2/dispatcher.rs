// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The dispatcher lifetime belongs to a WebView, not a reusable HWND value.
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use windows::{
  core::s,
  Win32::{Foundation::*, Graphics::Gdi::*, UI::{Shell::*, WindowsAndMessaging::*}},
};

const SUBCLASS_ID: usize = (WM_USER + 0x66) as usize;
static EXEC_MSG_ID: Lazy<u32> = Lazy::new(|| unsafe { RegisterWindowMessageA(s!("Wry::ExecMsg")) });
type Callback = Box<Box<dyn FnOnce()>>;

pub(super) struct Dispatcher {
  hwnd: isize,
  alive: Mutex<bool>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use windows::core::w;

  struct Window(HWND);
  impl Window {
    unsafe fn new() -> Self {
      Self(CreateWindowExW(
        WINDOW_EX_STYLE::default(), w!("STATIC"), w!("Wry dispatch regression"),
        WS_OVERLAPPED, 0, 0, 0, 0, Some(HWND_MESSAGE), None, None, None,
      ).unwrap())
    }
  }
  impl Drop for Window {
    fn drop(&mut self) { let _ = unsafe { DestroyWindow(self.0) }; }
  }
  struct Dropped(Arc<AtomicUsize>);
  impl Drop for Dropped {
    fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
  }
  unsafe fn pump(hwnd: HWND) {
    let mut message = MSG::default();
    while PeekMessageW(&mut message, Some(hwnd), *EXEC_MSG_ID, *EXEC_MSG_ID, PM_REMOVE).as_bool() {
      DispatchMessageW(&message);
    }
  }

  #[test]
  fn live_window_executes_callback_once() { unsafe {
    let window = Window::new();
    let dispatcher = Dispatcher::new(window.0).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    dispatcher.dispatch(move || { observed.fetch_add(1, Ordering::SeqCst); });
    pump(window.0);
    pump(window.0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }}

  #[test]
  fn destroy_drops_queued_and_late_callbacks_without_executing() { unsafe {
    let window = Window::new();
    let dispatcher = Dispatcher::new(window.0).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let queued = Dropped(drops.clone());
    dispatcher.dispatch(move || { let _guard = queued; panic!("closed callback executed"); });
    drop(window);
    assert!(!dispatcher.is_alive());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let late = Dropped(drops.clone());
    dispatcher.dispatch(move || { let _guard = late; panic!("late callback executed"); });
    assert_eq!(drops.load(Ordering::SeqCst), 2);
  }}

  #[test]
  fn close_is_idempotent_and_prevents_worker_posts() { unsafe {
    let window = Window::new();
    let dispatcher = Dispatcher::new(window.0).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let worker_dispatcher = dispatcher.clone();
    let worker_drops = drops.clone();
    let worker = std::thread::spawn(move || {
      for _ in 0..100 {
        let guard = Dropped(worker_drops.clone());
        worker_dispatcher.dispatch(move || { let _guard = guard; panic!("closed callback executed"); });
      }
    });
    dispatcher.close();
    worker.join().unwrap();
    dispatcher.close();
    pump(window.0);
    assert_eq!(drops.load(Ordering::SeqCst), 100);
  }}

  #[test]
  fn failed_post_reclaims_callback() { unsafe {
    let window = Window::new();
    let hwnd = window.0.0 as isize;
    drop(window);
    let dispatcher = Dispatcher { hwnd, alive: Mutex::new(true) };
    let drops = Arc::new(AtomicUsize::new(0));
    let guard = Dropped(drops.clone());
    dispatcher.dispatch(move || { drop(guard); });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
  }}

  #[test]
  fn old_response_does_not_reach_replacement_window() { unsafe {
    let old_window = Window::new();
    let old = Dispatcher::new(old_window.0).unwrap();
    drop(old_window);
    let window = Window::new();
    let new = Dispatcher::new(window.0).unwrap();
    old.dispatch(|| panic!("old callback executed in replacement window"));
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    new.dispatch(move || { observed.fetch_add(1, Ordering::SeqCst); });
    pump(window.0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }}
}

impl Dispatcher {
  pub(super) unsafe fn new(hwnd: HWND) -> crate::Result<Arc<Self>> {
    let dispatcher = Arc::new(Self { hwnd: hwnd.0 as isize, alive: Mutex::new(true) });
    let data = Box::into_raw(Box::new(dispatcher.clone()));
    if !SetWindowSubclass(hwnd, Some(Self::window_proc), SUBCLASS_ID, data as usize).as_bool() {
      drop(Box::from_raw(data));
      return Err(windows::core::Error::from_win32().into());
    }
    Ok(dispatcher)
  }

  pub(super) fn is_alive(&self) -> bool {
    *self.alive.lock().unwrap_or_else(|e| e.into_inner())
  }

  pub(super) unsafe fn dispatch(&self, function: impl FnOnce() + 'static) {
    // Serialize posting with WM_DESTROY/close, so no new message can enter
    // the queue after close has drained it. Do not drop callbacks under the lock.
    let boxed: Callback = Box::new(Box::new(function));
    let alive = self.alive.lock().unwrap_or_else(|e| e.into_inner());
    if !*alive {
      drop(alive);
      return;
    }
    let raw = Box::into_raw(boxed);
    let result = PostMessageW(Some(HWND(self.hwnd as _)), *EXEC_MSG_ID, WPARAM(raw as usize), LPARAM(0));
    drop(alive);
    if let Err(error) = result {
      // PostMessage did not transfer ownership to the UI thread.
      drop(Box::from_raw(raw));
      // A native destroy may race the WM_DESTROY notification. Other errors
      // (including a full queue) must remain visible for diagnosis.
      if error.code() != windows::core::HRESULT::from_win32(ERROR_INVALID_WINDOW_HANDLE.0) {
        #[cfg(feature = "tracing")]
        tracing::error!("WebView2 callback dispatch failed: {error}");
        #[cfg(debug_assertions)]
        eprintln!("WebView2 callback dispatch failed: {error}");
      }
    }
  }

  /// Must run on the window's UI thread, before destroying its HWND.
  pub(super) unsafe fn close(&self) {
    let mut alive = self.alive.lock().unwrap_or_else(|e| e.into_inner());
    if !*alive {
      return;
    }
    *alive = false;
    drop(alive);
    let mut message = MSG::default();
    while PeekMessageW(&mut message, Some(HWND(self.hwnd as _)), *EXEC_MSG_ID, *EXEC_MSG_ID, PM_REMOVE).as_bool() {
      drop(Box::from_raw(message.wParam.0 as *mut Box<dyn FnOnce()>));
    }
  }

  unsafe extern "system" fn window_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
    subclass_id: usize, data: usize,
  ) -> LRESULT {
    let dispatcher = &*(data as *const Arc<Self>);
    if msg == *EXEC_MSG_ID {
      let function: Callback = Box::from_raw(wparam.0 as *mut _);
      if dispatcher.is_alive() {
        function();
        let _ = RedrawWindow(Some(hwnd), None, None, RDW_INTERNALPAINT);
      }
      return LRESULT(0);
    }
    if msg == WM_DESTROY || msg == WM_NCDESTROY {
      dispatcher.close();
    }
    if msg == WM_NCDESTROY {
      let _ = RemoveWindowSubclass(hwnd, Some(Self::window_proc), subclass_id);
      drop(Box::from_raw(data as *mut Arc<Self>));
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
  }
}
