# Cursor monitor DPI cache

## Change

The cursor monitor previously called `WebviewWindow::scale_factor()` for each poll. Locked dependencies are Tauri 2.10.3, tauri-runtime 2.10.1, tauri-runtime-wry 2.10.1, and tao 0.34.6. Tauri delegates to the runtime dispatcher; Wry's `window_getter!` sends `WindowMessage::ScaleFactor` and synchronously waits for the event-loop response. Tao's Windows backend then reads `WindowState.scale_factor` under its state mutex. This is a synchronous event-loop round trip from the cursor worker, not a direct cached getter at the Tauri boundary.

The cursor worker now keeps a cache per window label. The existing builder-level `on_window_event` callback updates it on `ScaleFactorChanged`. Each monitor generation queries once at startup and refreshes once per second as a missed-event fallback; the existing 33 ms active, 50 ms near, and 200 ms idle cursor intervals remain unchanged. Cursor physical-to-logical conversion, rounded-region hit testing, and `set_ignore_cursor_events` decisions are unchanged.

Each event advances a revision. A query only writes if no newer event arrived while that synchronous query was in flight, so an event received before or during initialization cannot be overwritten by a stale query result. Only finite, positive scale factors are accepted. An invalid event clears the cached value and prompts an immediate query; a failed/invalid query clears only the state for the revision it observed. Until a valid factor is available, the monitor follows its existing fail-open input behavior and retries at the existing 100 ms retry cadence. A late query failure cannot clear a newer event value.

The existing single global window-event handler receives scale changes; watchdog restarts do not register listeners. Destroying a window removes that label's cache and invalidates the active monitor generation. A new window with the same label therefore gets a new empty cache. Before starting a worker, code also checks that its HWND still matches the currently registered window with that label, preventing a delayed watchdog restart from attaching to a destroyed same-label instance.

## Validation

- `cargo check --offline --locked --lib` — passed.
- `cargo check --offline --locked --example c2-media-probe --features c2-media-probe` — passed.
- `cargo test --offline --locked --lib commands::window::interaction_region_tests` — 13 passed, 0 failed.
- `git diff --check` — passed; the worktree emitted only existing Git line-ending notices.

The added pure-logic tests cover event-versus-initial-query ordering, rejection of zero/negative/NaN/infinite factors, query failure clearing only the revision it observed, label isolation, and a new empty cache after same-label destruction/recreation. No product process was launched or controlled. No Release or probe was rebuilt or run, and no settings were changed.

## Remaining runtime check

Mixed-DPI product validation is still required: move the actual window between displays with different scaling, change Windows display scaling, verify the native `ScaleFactorChanged` update and fallback behavior, and confirm transparent interaction-region coordinates and cursor passthrough remain consistent. Also verify window destruction/recreation with the same label and confirm the 33/50/200 ms cursor policy remains intact. No CPU improvement is claimed; this change removes repeated synchronous getter calls from the cursor hot loop by source-level inspection, but no runtime measurement isolated their cost.
