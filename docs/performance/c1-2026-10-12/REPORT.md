# C1 Native UI Thread Offload — Partial Acceptance

Date: 2026-10-12  
Baseline commit: `48530f7` (M6)  
Test host: Windows 11, two 1920×1080 displays at 100% scale; the app window was moved to secondary `\\.\DISPLAY2` at x=-1920.  
Status: implementation and worker-path checks pass; end-to-end performance acceptance is **incomplete**. Keep C1 `[ ]`.

## Implementation

- GSMTC queries and controls, system audio COM calls, cache file operations, and synchronous image/color helpers now run through bounded blocking workers. Admission is capped at four native and two background tasks; full pools return errors rather than accumulating unbounded work.
- Play/pause retains its confirmation refresh and result/error behavior. The 150 ms confirmation interval now yields asynchronously, and its follow-up snapshot runs on a native worker.
- COM `Apartment::enter()` remains inside the service operations executed on native workers. COM objects are not moved across threads.
- `animate_window_bounds` was checked as a possible remaining blocker. Its spring loop already runs on a detached thread and returns immediately; no 16 ms sleep loop runs on the command/UI thread.

## Verification and observed behavior

- `cargo check --manifest-path src-tauri/Cargo.toml --tests`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: **63 passed, 0 failed, 1 ignored** (the ignored case requires the Windows audio service).
- `node --check scripts/benchmark-c1.mjs`, PowerShell script parsing, and `git diff --check`: passed.
- In C1 Release round r11, the window stayed responsive on `DISPLAY2`; play/pause and restoration completed in 184/171 ms (including the intended async confirmation delay), seek and restoration in 8.9/11.3 ms, and all 32 volume calls completed. The volume returned to 33%, and playback returned to paused, matching its initial state. Sixteen expand/collapse transitions completed.
- C1 r11 WebView RAF had 1,066 samples, p95 5.1 ms, max 5.3 ms, with no interval above 50 ms. This is JavaScript callback cadence, not physical presentation timing.
- Process-tree thread samples peaked at 241 total / 70 in the largest process in C1 r7, versus 245 / 70 in the partial M6 r3 run. These short samples do not establish a thread-count reduction.
- The M6 r3 root process became unresponsive while synchronous media/volume commands were in flight; the matched C1 process remained responsive and its commands completed. The M6 run did not complete the same command sequence, so command latency and thread peaks are not a complete matched comparison.

## App-attributed presentation trace

After the user authorized an elevated capture session, PresentMon 2.6.0 captured the WebView2 GPU process swap chain. The statistics below use only the timestamp window covering the 16 expand/collapse actions; idle 1 Hz presentations outside that window are excluded. The matching CSV slices in `raw/` retain `TimeInMs`, `MsBetweenDisplayChange`, `MsCPUBusy`, and `MsGPUTime`; complete captures remain in the local `dist/performance/c1-2026-10-12/` directory.

| Run | Active intervals | p50 | p95 | p99 | Maximum | >16.7 ms | >50 ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| M6 r3, partial run | 829 | 5.00 ms | 7.39 ms | 7.47 ms | 30.03 ms | 4 | 0 |
| C1 r7 | 604 | 5.00 ms | 7.00 ms | 20.00 ms | 630.01 ms | 11 | 2 |
| C1 r11, paused/33% matched state | 590 | 5.00 ms | 5.06 ms | 25.01 ms | 615.00 ms | 13 | 1 |

Both C1 runs recorded a 615–630 ms display-change interval during the 15th collapsed-to-expanded transition. The corresponding PresentMon rows report roughly 614–634 ms of CPU busy time and under 1 ms of GPU time. WebView RAF and native command completion remained regular. This confirms an app-process presentation stall in those captures, but does not identify its cause. The earlier M6 trace did not contain such an interval during its captured interaction window; however, its run was incomplete and cannot serve as a fully matched pass/fail comparison.

## Decision

The worker implementation removes the observed synchronous media wait from the command path, preserves return/error behavior, and passed the available Release control workload. The actual presentation trace has a severe, repeated tail outlier during the same interaction, while the M6 baseline could not complete the matching workload. The evidence therefore does not support checking C1 or claiming a presentation improvement. This upload records the implementation and incomplete acceptance as a partial C1 change; it is not an acceptance commit. Keep M3–M6 unchanged, leave C1 `[ ]`, and do not infer the cause of the long intervals from these captures.

## Artifacts

- C1 Release executable SHA-256: `4C7E83C9C0D2CD5D4BACB7874D97BF353E94E366EBF12B3C1D19A2697D268C0F`
- M6 Release executable SHA-256: `EA35037231D1C48F516C66AA762D601DA5E083862F515FA262403C16582C2456`
- Thread and timeline samples are retained in `raw/`; full interaction JSON and progress logs remain under `dist/performance/c1-2026-10-12/`.

