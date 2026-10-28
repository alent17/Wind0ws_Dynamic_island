# Local Wry 0.54.4 patch

Source: crates.io `wry` 0.54.4, checksum
`e5a8135d8676225e5744de000d4dff5a082501bf7db6a1c1495034f8c314edbc`.
Original MIT and Apache-2.0 licenses are retained.

The WebView2 asynchronous responder previously posted raw boxed callbacks to
an HWND even after the window was destroyed. This produced repeated
`PostMessage failed ... 0x80070578` messages and leaked callbacks on failure.

`src/webview2/dispatcher.rs` now owns dispatch lifetime per WebView. Posting
is serialized with shutdown. WebView drop and native window destruction stop
new posts and release queued callbacks on the UI thread. Failed posts reclaim
their callback; errors other than invalid window handles remain visible.
Each responder retains its original dispatcher, so recreating a window cannot
route an old response into a reused HWND.

The root Cargo manifest uses `[patch.crates-io]` to apply this copy in clean
builds. Revisit this patch when upgrading Wry.

Run the native regression tests from `src-tauri`:

```powershell
cargo test --offline --manifest-path vendor/wry/dispatch-tests/Cargo.toml --target-dir target/dispatch-tests -- --test-threads=1
```

The test harness compiles the production dispatcher directly, creates hidden
message-only windows, and checks normal delivery, queued callback cleanup,
late responses, concurrent posts during shutdown, and failed-post cleanup.
