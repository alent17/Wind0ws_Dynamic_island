// Exercise the exact production dispatcher without Wry's unrelated example dependencies.
#![cfg(windows)]
#![allow(dead_code)]
pub type Result<T> = windows::core::Result<T>;
#[path = "../src/webview2/dispatcher.rs"]
mod dispatcher;
