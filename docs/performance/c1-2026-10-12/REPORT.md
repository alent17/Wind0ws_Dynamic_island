# C1 Native UI Thread Offload — Partial Acceptance

Date: 2026-10-12
Baseline commit: `48530f7` (M6)
Test host: Windows 11, two 1920×1080 displays at 100% scale; the app window was moved to secondary `\\.\DISPLAY2` at x=-1920.
Status updated 2026-10-14: implementation checks and the later media-capability/error and resource-measurement subtasks pass; end-to-end C1/A5 performance acceptance remains **incomplete**. Keep C1 and A5 `[ ]`. The historical 95/55 ms activity attribution is withdrawn; see the [raw-input review](../c1-review-2026-10-14/REPORT.md).

## Historical snapshot follow-up — 2026-10-12 (attribution superseded 2026-10-14)

The saved r4 matrix records animationStart and animationEnd snapshots. The end was described as a target-style-stable-for-100-ms confirmation, but the raw matrix/progress files do not save the final mutation or lastChange value. Consequently the reported 343–374 ms spans are start-to-confirmation envelopes, not precise active style durations. The old analysis additionally omitted MsUntilDisplayed and inferred a rounded 16-hour offset from the first CPUStartDateTime and coordinator launch UTC. Neither frame QPC nor the date-time conversion provenance/residual error was saved. Native bounds QPC/UTC cannot alone align these date-only frame rows. Current activity attribution is unconfirmed; the table below is retained solely as historical analyzer output, superseded by the 2026-10-14 review.

M6 and C1 Release were each run through animation-only, controls-only, and combined scenarios. Initial and final state was paused at 33% volume. `QueryDisplayConfig` reported both active targets (33024 and 33025) at 200 Hz. The main HWND sampler observed `\\.\DISPLAY2` throughout each capture; the window was 372×289 px and moved within that monitor during setup. PresentMon targeted only each build's WebView2 GPU PID and recorded one SwapChain per capture, `VidPnSourceId=0`, and `LayerIndex=0`. Both capture logs contain start and stop markers and both CSVs are non-empty. The full-resolution QPC/UTC HWND samples and PresentMon CSVs, plus the compact joined analysis, are retained in `raw/` and `matrix-analysis-user-r4.json`; full captures remain in the local `dist/performance/c1-2026-10-12/` directory.

| Scenario | Build | Display intervals | p50 | p95 | Maximum | >50 ms |
|---|---|---:|---:|---:|---:|---:|
| Animation only | M6 | 789 | 5.00 ms | 7.75 ms | 115.04 ms | 15 |
| Animation only | C1 | 791 | 5.00 ms | 7.07 ms | 120.00 ms | 15 |
| Controls only | M6 | 1 | 859.98 ms | 859.98 ms | 859.98 ms | 1 |
| Controls only | C1 | 35 | 5.00 ms | 10.01 ms | 270.00 ms | 1 |
| Combined | M6 | 792 | 5.00 ms | 7.29 ms | 120.02 ms | 16 |
| Combined | C1 | 786 | 5.00 ms | 7.05 ms | 137.98 ms | 16 |

The two source intervals are established: C1 CSV line 669 records 95.0015 ms, PID [omitted]/swapchain0x21E4A377F30, historically assigned to animation-only transition13; line1687 records54.9938 ms, the same PID/swapchain, assigned to combined transition14. The legacy inside-animation labels are reproducible only for its saved start-to-confirmation envelope and inferred offset. They do not establish that either interval was fully inside actual animation.

Under the explicitly unproven exact-16-hour mapping, adding the row's MsUntilDisplayed yields intervals 05:18:57.261037600–05:18:57.356039100Z and05:19:05.355903500–05:19:05.410897300Z. If the described >=100-ms-stability contract also held, both lie in the approximate confirmation tail; their starts are only2.0376/1.9035 ms after confirmation-minus100ms, so even a few milliseconds of residual clock error change that classification. This sensitivity is not a reconstructed last mutation or a proven clock mapping. The original 95/55 ms display interval values remain, while activity overlap/full containment and thread-blocking cause are unknown. Earlier 615–630 ms broad-window values likewise do not establish continuous-animation stalls; historical r4 comparisons cannot close that uncertainty with this clock/boundary evidence.

The controls-only rows also demonstrate the metric's boundary sensitivity. M6's sole 859.98 ms interval started before the controls window and ended during it, with no animation active. C1's 270.00 ms interval fell inside the controls-only window, also with no animation active; the visible surface was otherwise static. These samples do not establish an animation stall. Media and volume commands completed successfully in both builds, with maximum measured call durations of 17.54 ms for M6 and 17.61 ms for C1 in controls-only, and 18.49/17.84 ms in the combined run. Both builds ended paused at 33% volume.

The historical r4 session was NetEase. Later batch02 independently accepted real Apple Music play/pause, advancing/stable timeline, seek=false pre-dispatch rejection/read-only UI, no-session errors and separately labeled UI error injection. Apple seek=false is a genuinely unsupported capability, not a product requirement to implement seek for Apple. Effective seek on a declared-supported session remains N/A. See the [media report](../c1-media-2026-10-13/REPORT.md). Later batch03 independently accepted the resource readiness/clock gates and one diagnostic pair of the original saved A5 builds; it did not validate the new media-fixed Release's performance. See the [resource report](../c1-resources-2026-10-14/REPORT.md).

The historical WPR attempt failed with0xc5585011; no relevant thread wait stacks are saved. Low GPU time or MsCPUBusy cannot identify a blocking thread or cause. The failed VidPnSourceId/display crosswalk remains unresolved; sampled HWND placement on DISPLAY2 is distinct evidence. No new capture/elevation was performed in this review. Remaining acceptance is listed in the historical-review report; stacks are needed only if a newly authorized, correctly aligned run confirms a genuine active stall.

## Implementation

- GSMTC queries and controls, system audio COM calls, cache file operations, and synchronous image/color helpers now run through bounded blocking workers. Admission is capped at four native and two background tasks; full pools return errors rather than accumulating unbounded work.
- Play/pause retains its confirmation refresh; the later batch02 fixes additionally propagate rejected WinRT boolean results. The 150 ms confirmation interval now yields asynchronously, and its follow-up snapshot runs on a native worker.
- COM `Apartment::enter()` remains inside the service operations executed on native workers. COM objects are not moved across threads.
- `animate_window_bounds` was checked as a possible remaining blocker. Its spring loop already runs on a detached thread and returns immediately; no 16 ms sleep loop runs on the command/UI thread.

## Verification and observed behavior

- `cargo check --manifest-path src-tauri/Cargo.toml --tests`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: **63 passed, 0 failed, 1 ignored** (the ignored case requires the Windows audio service).
- `node --check scripts/benchmark-c1.mjs`, PowerShell script parsing, and `git diff --check`: passed.
- In C1 Release round r11, the window stayed responsive on `DISPLAY2`; play/pause and restoration completed in 184/171 ms (including the intended async confirmation delay), seek requests and restoration requests returned in 8.9/11.3 ms (not proof of effective seek), and all 32 volume calls completed. The volume returned to 33%, and playback returned to paused, matching its initial state. Sixteen expand/collapse transitions completed.
- C1 r11 WebView RAF had 1,066 samples, p95 5.1 ms, max 5.3 ms, with no interval above 50 ms. This is JavaScript callback cadence, not physical presentation timing.
- Process-tree thread samples peaked at 241 total / 70 in the largest process in C1 r7, versus 245 / 70 in the partial M6 r3 run. These short samples do not establish a thread-count reduction.
- The M6 r3 root process became unresponsive while synchronous media/volume commands were in flight; the matched C1 process remained responsive and its commands completed. The M6 run did not complete the same command sequence, so command latency and thread peaks are not a complete matched comparison.

## Initial app-attributed presentation trace (unaligned animation bounds)

After the user authorized an elevated capture session, PresentMon 2.6.0 captured the WebView2 GPU process swap chain. The statistics below use only the timestamp window covering the 16 expand/collapse actions; idle 1 Hz presentations outside that window are excluded. The matching CSV slices in `raw/` retain `TimeInMs`, `MsBetweenDisplayChange`, `MsCPUBusy`, and `MsGPUTime`; complete captures remain in the local `dist/performance/c1-2026-10-12/` directory.

| Run | Active intervals | p50 | p95 | p99 | Maximum | >16.7 ms | >50 ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| M6 r3, partial run | 829 | 5.00 ms | 7.39 ms | 7.47 ms | 30.03 ms | 4 | 0 |
| C1 r7 | 604 | 5.00 ms | 7.00 ms | 20.00 ms | 630.01 ms | 11 | 2 |
| C1 r11, paused/33% matched state | 590 | 5.00 ms | 5.06 ms | 25.01 ms | 615.00 ms | 13 | 1 |

Both C1 runs recorded a 615–630 ms display-change interval inside the broad click-to-fixed-wait window for the 15th collapsed-to-expanded action. The corresponding PresentMon rows report roughly 614–634 ms of CPU busy time and under 1 ms of GPU time. The old harness did not timestamp actual style-animation start/end, so these rows do not establish that the whole interval occurred during continuous animation. The M6 trace was incomplete and cannot serve as a matched comparison. The 2026-10-14 raw-input review supersedes the old animation attribution; the earlier raw values remain for history.

## Decision

The worker changes remove synchronous media work from the command path. Media error/capability correctness and resource measurement readiness have passed their separately scoped later acceptance. The historical matrix preserves real display interval values, but cannot prove two fully-inside active-animation gaps: precise last mutations and an unambiguous frame-clock mapping are absent. C1/A5 therefore remain `[ ]`; no presentation improvement or root cause is established. A matched performance run of the current media-fixed implementation and the remaining native interaction/cost checks are still required. This historical review is documentation/measurement evidence only, not a product or acceptance commit.

## Artifacts

- C1 Release executable SHA-256: `4C7E83C9C0D2CD5D4BACB7874D97BF353E94E366EBF12B3C1D19A2697D268C0F`
- M6 Release executable SHA-256: `EA35037231D1C48F516C66AA762D601DA5E083862F515FA262403C16582C2456`
- Thread and timeline samples are retained in `raw/`; full interaction JSON and progress logs remain under `dist/performance/c1-2026-10-12/`.
- The aligned matrix is summarized in [matrix-analysis-user-r4.json](matrix-analysis-user-r4.json); raw PresentMon and native bounds CSVs are in `raw/`.


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
