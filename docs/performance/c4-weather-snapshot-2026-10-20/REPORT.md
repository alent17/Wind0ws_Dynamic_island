# C4 first slice: weather-only snapshots

## Scope

The App and Studio weather panels previously called `get_idle_snapshot`, which initialized/refreshed the process-wide sysinfo collector for CPU, memory, network and battery before fetching weather. No current App or Studio consumer displayed those system fields. This slice separates weather reads while preserving the old full-snapshot API for any other or future consumer.

## Implementation

- Added Rust `WeatherSnapshot` with only temperature, weather code, update timestamp and forecast fields, plus the `get_weather_snapshot` Tauri command.
- The new service reads the configured location, calls the existing `weather_for` path, and retains its single-flight request, 30-minute weather TTL, coordinate match and same-location stale-cache fallback. It returns an empty weather payload if no location is configured. It does not call `system_snapshot`, `spawn_blocking`, or initialize/read `SYSTEM_COLLECTOR`.
- Kept `get_idle_snapshot` and `IdleSnapshot` as the complete system-plus-weather snapshot. The existing `idleApi.getSnapshot` cache/pending state remains independent and unchanged.
- Added a separate frontend weather cache: one last value keyed by the requested location, 30-second TTL, and a pending-request map keyed by location. Up to four distinct keys can be in flight; same-key calls share a promise, rejected calls are removed so they can retry, and overflow rejects instead of growing state without bound.
- App and Studio now store only `WeatherSnapshot`; they no longer carry placeholder CPU/memory/network/battery values. App's C3 `pageVisible && !isHidden` visibility gate and 30-second refresh remain. Studio verifies the location key again before applying a response after a city change.

## Validation

- `svelte-check`: 0 errors, 0 warnings.
- New `src/lib/api/idle.test.ts`: 4/4 tests passed for separate legacy/weather commands and caches, location keying, same-key single-flight across other key requests, retry after failure, and the four-key pending bound.
- Rust `services::idle::tests`: 5/5 passed, including exact-coordinate fallback and weather-only projection; `cargo check --offline --locked --lib` passed; existing `c2-media-probe` feature example check passed offline/locked.
- Existing lifecycle Playwright suite: 7/7 passed. The controlled-clock App case now asserts that weather polling uses `get_weather_snapshot`, hidden time triggers no request, an in-TTL restore reuses the cache, an expired-cache restore makes one new request, and `get_idle_snapshot` remains at zero calls.
- Existing lifecycle fixture production build passed (3,854 modules); main frontend production build passed (3,868 modules). One requested UI detector run for `src/App.svelte` returned no findings.
- No live weather/network request was used in tests. No Release build was performed.

## Limits and runtime identity

This is structural evidence that the weather command path does not enter the existing system collector. It is not a native call-count, CPU, battery or power measurement. Weather network refresh/error handling is covered through existing pure cache behavior and mocks, not a live provider.


## Second slice: Studio and TimerWindow clocks

- Studio's preview clock now refreshes immediately on mount, then schedules the next refresh at the next wall-minute boundary from `Date.now()`. Each callback recalculates the next boundary, so delayed callbacks do not accumulate drift. Cleanup clears the pending timeout. The existing `$derived(formatClock(settings.clockTimeZone, $locale, previewNow))` remains intact, so timezone and locale changes recompute the display immediately without waiting for the next minute. Studio's weather and session polling are unchanged.
- TimerWindow contains no continuous progress ring or other 250 ms visual consumer: the timer wheel and status use the formatted displayed second. Its refresh now uses one timeout to the next countdown display-second boundary, based on absolute remaining time (`5,500 ms` schedules about `500 ms`; `5,000 ms` schedules `1,000 ms`) and recalculates from `Date.now()` after each callback. Existing absolute countdown state, completion handling, formatted wheel transition and CSS are unchanged. Paused, idle, hidden and zero-remaining states retain no display timeout; state and visibility changes synchronize immediately, and effect cleanup cancels the pending timeout.
- The existing lifecycle fixture now instruments timeout scheduling and can mount Studio and TimerWindow. Two targeted Playwright tests passed with a paused controlled clock: Studio's `00:00:00.250` start scheduled `59,750 ms`, refreshed at the boundary to `60,000 ms`, and canceled on unmount; TimerWindow verified the 5.5-second half-second boundary, pause cancellation, hidden deadline passage with immediate visibility resync, and unmount cancellation without late callbacks.
- Validation: lifecycle fixture production build passed (3,866 modules); targeted Playwright tests passed (2/2); `svelte-check --tsconfig ./tsconfig.json` passed with 0 errors and 0 warnings; frontend production build passed (3,868 modules). No Release build or native UI automation was run.

The second slice is code and controlled-clock evidence only; it makes no native CPU/power claim. C2 and C3 remain open. C4 remains `[ ]` because remaining C4 work is not fully implemented or accepted. No settings, registry values or running product process were modified, and no commit or push was made.

## Third slice: gate spectrum cover-color extraction

- App's shared `IslandSurface` renders compact `Spectrum` in compact/hover modes, and expanded `Spectrum` on the music page; both require `showSpectrum`. When it is off, the compact playing dot still uses `spectrumTopColor`, so that visible consumer also requires current cover colors while playing. The effect now starts only for a visible compact/hover Spectrum, a visible expanded music Spectrum, or a visible playing dot with `showSpectrum` off. Page-hidden and main-island-hidden states gate all consumers. An effect cleanup aborts the in-flight `loadImage`; the post-await result check rechecks actual consumers and the current cover before publishing colors.
- Studio passes `appearanceDraft.showSpectrum` to its preview `IslandSurface`. Its effect requires the preview to be ready and a cover URL, then samples for compact/hover Spectrum, expanded music Spectrum, or a playing dot in compact/hover mode while Spectrum is off. Expanded non-music pages and hidden preview mode have no color consumer. The existing `loadImage` AbortSignal path cancels superseded/hidden work, and result publication checks the current cover and current preview conditions. Re-entering a color-consuming preview state reruns the effect using the current sample cover.
- Both `main.ts` and `studio.ts` mount content through the existing `VisibleWindow`. That wrapper listens for `visibilitychange`, native visibility and focus changes, then unmounts its child while hidden and mounts it again on restore. The Studio minute clock and spectrum effect therefore refresh/cancel through normal component mount and cleanup; no separate Studio visibility listener was added.
- There is no shared result cache in `spectrumColors.ts`; its existing extraction path already accepts `AbortSignal`, so no second cache or image-loading mechanism was added. Spectrum palette generation, FFT subscriptions, and FloatingWindow's separate album-color behavior were not changed. With Spectrum off, colors still refresh when the compact playing dot is actually visible and playing; they are skipped on expanded non-music pages, where neither the Spectrum nor compact dot is visible.
- The lifecycle fixture now distinguishes the existing cover-display `Image` objects from extraction work by recording the image loader's 5-second timeout, plus sampling-canvas and `drawImage` creation. Two spectrum-focused Playwright tests passed, covering playing-dot color refresh with Spectrum off, a true no-consumer expanded weather state, hidden/cancelled stale requests, and current-cover recovery. A final combined targeted run including both second-slice clock cases passed 4/4. Fixture build passed (3,867 modules), `svelte-check` passed with 0 errors/0 warnings, and frontend production build passed (3,868 modules).

This is controlled-clock and mocked-image evidence only. No native UI, CPU/power or live artwork-provider measurement was performed. No Release build, user setting/registry change or product-process operation was performed; no commit or push was made. C4 remains `[ ]` because other listed C4 work remains.

## Passive dynamic process-tree sample (2026-10-21; uncontrolled state)

The existing `scripts/measure-native.ps1` was used read-only against the already-running candidate PID [omitted] after verifying its absolute EXE path and SHA-256. The command requested a 10-second warmup and 60-second sample with GPU counters enabled. It produced 42 samples over 60.971 seconds (timestamps 2026-10-21 12:00:39–12:01:40 UTC); the slower sample count reflects the script's per-sample process and GPU counter collection. The dynamic tree had the same seven process identities in every sample. GPU engine counters were available, but must be read per engine and not summed as total GPU utilization.

Across this one sample, process-tree machine CPU was median 1.2527% (range 0–5.2715%), equivalent to 15.0325% of one logical core at the median on this 12-core machine. Summed CPU time by role was app 1.703125 s, browser 1.343750 s, GPU process 5.609375 s, renderer 3.765625 s, utility 0.015625 s, crashpad 0 s. GPU 3D engine utilization had median 0% and maximum 1%; this is not whole-GPU utilization. Process-tree private bytes ranged from 265.26 MiB at the first sample to 279.41 MiB at the last; GPU-process committed memory ranged from 19.81 MiB to 41.17 MiB. These endpoint changes are a single uncontrolled observation and do not establish a leak, a recovery trend, or a change caused by C4.

The active page, visibility/capture state, weather/clock-only display condition, music state and foreground workload were not reliably observed. No collector-call, IPC or frontend effect counters were available in this production build. Therefore this sample cannot establish that hidden system sampling was zero, nor show before/after performance improvement. There was no comparable pre-C4 sample under the same conditions.

The sampling wrapper mistakenly treated `$LASTEXITCODE` as the exit result of a PowerShell `.ps1` invocation and stopped before its planned immediate post-run identity/settings/Run check. The report files were independently read and hashed on 2026-10-22; a fresh CIM check then found zero Isle processes. The settings mode was still `real` and its current file hash matched the pre-sample hash; the current Run-value hash also matched the pre-sample hash. Because these checks happened the next day, they cannot substitute for the missing immediate post-run comparison or prove that no intermediate changes occurred. The sample data remains intact: JSON SHA-256 `6977379EB3371E7D507CB5CA7FFC1A60B524C8244A7170523C62CDDC60527205`; JSONL SHA-256 `244D04AF7E06DBA4678C9226242E9CAFD5023E8A30248FEAF928FCDDFB57E8E4`.

C4 remains `[ ]`. The controlled native clock/weather-only case, actual collector/IPC/effect counts, native hide/restore/capture/OS timezone behavior and matched before/after CPU/GPU comparison are still outstanding. No settings or playback state was changed, no UI automation was used, and no commit or push was made.

## Fourth slice: gate Studio media-session polling by Players visibility

- Studio now observes `.players-section` with `IntersectionObserver`. The existing settings-store initialization is allowed to finish first (including its existing catch-and-continue fallback); once the Players section intersects the viewport, Studio requests the current media-session list immediately and starts the existing 10-second fallback interval. Leaving the viewport stops the interval and invalidates the current request. Re-entry requests a fresh current list immediately. If `IntersectionObserver` is unavailable, the previous always-visible polling behavior remains.
- `refreshSessions` serializes list calls. A timer tick or manual refresh during an in-flight call queues at most one follow-up; after the in-flight call settles, it runs only if the section is still visible and Studio is mounted. Request generations prevent a response arriving after the section leaves or Studio unmounts from replacing the visible list. Existing selected/offline player ordering, selection, persistence, refresh button and loading UI remain intact.
- The lifecycle fixture can hold and resolve media-session IPC calls and records active intervals/observer cleanup. A controlled-clock Playwright test verifies no offscreen request, immediate fetch on entry, the 10-second refresh without overlapping a held request, stopping on leave, stale response rejection, immediate fetch on re-entry, manual refresh and timer/observer cleanup at unmount. The focused test passed (1/1).
- Validation for this slice: fixture production build passed (3,867 modules); `svelte-check --tsconfig ./tsconfig.json` passed with 0 errors and 0 warnings; the combined Studio/Timer clock, session-list and spectrum-color Playwright suite passed (5/5); frontend production build passed (3,868 modules).

No native UI, CPU/power or Release validation was performed. No settings, registry values or running product process were modified; no commit or push was made. C4 remains `[ ]` because other listed C4 work remains.

## Fifth slice: split App minute clock, media progress and countdown deadline

- `clockNow` has two remaining consumers: interpolated media position (`projectedPosition`) and the absolute countdown (`getRemainingMs`). Clock text now has a separate timeout aligned to the next `Date.now()` minute boundary. Its effect tracks timezone and locale, formats immediately when either changes, and re-arms from current wall time after each callback. Visibility/focus recovery forces an immediate text refresh so a changed system timezone is picked up even when the clock zone remains `system`. The existing minute key still prevents a second formatter construction within the same minute; no global formatter cache was added.
- The high-resolution clock scheduler now runs at 250 ms only when the document is visible, the main island is not capture-hidden, the expanded music page is active, and media is playing. Pausing, leaving that page, capture hiding or document hiding cancels that tick; visibility recovery synchronizes `clockNow` immediately so progress catches up from the media snapshot's timestamp.
- A running countdown continues to refresh at its next displayed-second boundary while visible. While document-hidden or capture-hidden, the clock scheduler installs only one timeout for the absolute `endsAt` deadline. That wake updates `clockNow`, allowing the existing completion effect and `completeCountdown` path to finish once. The existing 10-second completion notice timeout remains unchanged. With neither visible playing media nor a running countdown, `clockNow` has no recurring timer.
- Lifecycle fixture instrumentation counts hour/minute `Intl.DateTimeFormat` constructions and completion-card insertions. Controlled-clock tests cover the minute boundary and no repeat formatting within a minute, immediate timezone reformat, visible playing progress, no 250 ms tick when paused/capture-hidden/document-hidden, current progress immediately after recovery, hidden countdown completion at its absolute deadline, one completion notice and unmount cleanup. Both new App tests pass as part of the combined 7/7 Studio/Timer/App lifecycle suite.
- Validation: `svelte-check --tsconfig ./tsconfig.json` passed with 0 errors and 0 warnings; fixture production build passed (3,867 modules); frontend production build passed (3,868 modules); `git diff --check` passed.

### C4 implementation status and remaining acceptance

The five source slices now cover separate weather snapshots, minute-aligned Studio/TimerWindow display updates, visible color-consumer gating, visible-region Studio session polling, and App minute/media/countdown scheduling. C4 remains `[ ]` because validation is still browser-fixture and structural evidence.

Remaining acceptance work:

- Run a matched native App scenario with only clock/weather shown and record actual system collector/IPC calls, frontend effect activity, and CPU/GPU before and after. Existing Rust/weather tests and browser fixtures establish the code paths, not native call counts or a measured performance delta.
- Verify the same behavior in the native WebView under real focus loss, document/native window hide/restore, capture hiding and OS timezone/locale changes. Current restore tests use controlled browser visibility/time and mock providers.
- Record native Studio Players intersection polling and provider responses when the native settings/session APIs are delayed or unavailable; current test holds mocked IPC and confirms request lifetime/order.

No native UI, CPU/GPU measurement or Release build was performed. No user settings, registry values or product processes were changed; no commit or push was made. C4 remains `[ ]` pending the acceptance work above.


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
