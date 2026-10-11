# M4 artwork work coalescing and cancellation — acceptance

Date: 2026-10-12
Status: **Accepted; M4 is checked.**

## Change and matched Release comparison

M4 adds process-wide single-flight work keyed by cover identity and processing parameters. Owners are tracked per WebView and slot; superseded work is canceled when its last consumer releases it, while other windows sharing that work continue. Color/pixel image reads now occur inside the shared job, so duplicate consumers do not separately load and hash the same input before joining. Frontend generations reject late results and allow failed requests to retry.

The final instrumented Release was built from the same M2/M3 source and measurement adapter as candidate B, with only M4 implementation differences. Three 100-track continuous-window rounds per side used the frozen 101-image corpus, DPI 1, 200×395 window, identical settings, empty per-run app/profile caches, and a five-minute paused recovery period. The final-source candidate binary SHA-256 was `B910690F365E0D724EA30B098FD5664AF38038C21DF7E0288518A4740DD20E84`. The final analyzer verified all full-length corpus identities and no candidate HD dimension misses. Formal performance rounds had no DevTools or artwork trace.

## Three-round medians

| Metric | Before | Final M4 | Change |
|---|---:|---:|---:|
| Process-tree Private Commit peak | 637.80 MiB | 639.81 MiB | +0.32% |
| Renderer Private Commit peak | 371.04 MiB | 384.86 MiB | +3.72% |
| GPU committed peak | 81.08 MiB | 81.83 MiB | +0.93% |
| Machine CPU average during track changes | 2.72% | 2.46% | −9.47% |
| Machine CPU p95 | 5.40% | 4.72% | −12.71% |
| First-paint median | 957.5 ms | 714.5 ms | −25.38% |
| First-paint p95 | 1,644 ms | 1,279 ms | −22.20% |
| HD-paint median | 2,301 ms | 1,960.5 ms | −14.80% |
| HD-paint p95 | 3,378 ms | 3,077 ms | −8.91% |
| Recovery last-minute process-tree Private Commit | 532.31 MiB | 520.47 MiB | −2.22% |
| Recovery last-minute Renderer Private Commit | 320.03 MiB | 310.70 MiB | −2.92% |
| Native codec decode calls per round, median | 605 | 527 | −12.9% |
| Download-input observations per round, median | 403 | 315 | −21.8% |
| Download-buffered peak | 196,608 bytes | 131,072 bytes | −33.33% |
| Encoded-buffer peak | 5,273 bytes | 0 bytes | −100% |

Processing counts, CPU, and cover latency improved. Peak tree Commit was effectively flat and peak Renderer Commit rose by 13.81 MiB; this is a small but real peak-memory regression. Five-minute recovery Commit moved down. GPU peak was effectively flat, so these rounds do not establish long-term GPU or memory stability. Paint latency uses the draw-complete event stream between track markers, not compositor presentation. This avoids undercounting when the saved per-track snapshot was sampled before the matching HD paint. The runner now uses the native marker's UTC timestamp for event ordering and a monotonic timer for minimum run duration.

## Functional acceptance

A final-candidate Release diagnostic changed 100 distinct tracks continuously and closed/reopened the floating window after tracks 25, 50, 75, and 100. All 100 states had one connected Canvas, a loaded image, and Canvas pixels matching the displayed source. The five heap snapshots reported zero Detached Canvas nodes; no page errors or blank/mismatched covers were recorded. The first diagnostic attempt stopped at track 6 because the image did not settle within the acceptance timeout, despite the native cover resolver reporting a result. Its report did not include enough URL/loading detail to identify a cause. The same final Release passed a fresh full 100-track/four-reopen run, including that track; the first failure remains recorded as an intermittent diagnostic outcome, not erased or attributed to a confirmed code defect.

Task and owner tests cover same-track replacement with a new cover identity/revision, explicit clear with a delayed stale result, stale pixel cancellation, shared work across two windows when one owner cancels, final-owner cancellation, old-revision cancellation, and failure followed by retry. Formal Release runs also measured the color/pixel input-read move in the final candidate. The diagnostic trace itself does not establish that a cancellation occurred during the 100-track pass because each request completed before ownership was superseded.

Verification on the final working tree: 84 frontend tests passed; Svelte check found 0 errors and 0 warnings; 56 Rust tests passed and 1 Windows-audio-service stress test was ignored. The full matching Release build succeeded. The original NetEase queue and playback mode were restored and playback was paused after testing.

## Acceptance boundary

M4 is accepted and marked `[x]`. M3 remains `[x]`; M5 and later phases were not changed. This acceptance does not claim an overall peak-memory reduction, a completed B0 baseline, long-duration GPU/memory stability, no-session recovery, or real cross-screen Windows DPI validation. The initial track-6 diagnostic timeout is recorded above and the passing rerun used the same final Release binary.
