# C2 work-area notification

## Change

The frontend already had a 5-minute work-area polling fallback and a shared focus, `pageshow`, and visibility recovery path with a 350 ms debounce. This change adds a native signal so display topology and Windows work-area changes can enter that same recovery path promptly.

The main HWND gets a dedicated `SetWindowSubclass` listener. It accepts `WM_DISPLAYCHANGE` and `WM_SETTINGCHANGE` only when `wParam == SPI_SETWORKAREA`. The subclass is separate from the experimental interaction-region subclass. It does not enumerate displays, resize the window, or invoke WebView code from the window procedure. A single atomic pending flag coalesces bursts before an async Tauri event emits `isle-work-area-changed`.

Installation is requested after the main window is created and again when the main page finishes loading. HWND registry and generation checks make installation idempotent and reject stale queued installs after destruction. `WM_NCDESTROY` uses a one-shot cleanup guard, removes the subclass when possible, and retains callback state through `DefSubclassProc`; the boxed state is released only after the default procedure returns. Any in-flight async notification holds its own `Arc` reference. The frontend subscribes using its existing `scheduleResumeRecovery` function and unregisters the listener when the component is destroyed. If event registration fails, the 5-minute fallback remains active.

## Validation

- `cargo check --offline --locked --lib` — passed.
- `cargo check --offline --locked --example c2-media-probe --features c2-media-probe` — passed.
- `cargo test --offline --locked --lib commands::window::interaction_region_tests` — 15 passed, 0 failed. Includes message-filter and destroy/reinstall registry checks.
- `npm run build` — passed with Vite 6.4.1; 3,868 modules transformed.
- `git diff --check` — passed (only existing LF/CRLF working-copy warnings from Git remain).
- The existing Release EXE/DLL hashes are checked separately in the handoff; no Release or probe rebuild/run was done for this change.

## Runtime boundary

No native display/work-area change was triggered or observed. Multi-monitor movement, actual message delivery, layout recovery, window teardown/recreation under live Windows messaging, and CPU/power impact remain unverified. The five-minute poll is still the recovery path if native registration or event delivery fails. A `RemoveWindowSubclass` failure is logged; callback state remains alive through default destruction handling, then relies on the terminal `WM_NCDESTROY` lifecycle. C2 remains open.
