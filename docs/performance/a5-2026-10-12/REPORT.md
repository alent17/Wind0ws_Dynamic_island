# A5 Windows Release validation — 2026-10-12

## Result

The Windows Playwright regression suite passed **10/10 tests** when launched from a normal, non-administrator PowerShell. The earlier failure in the restricted execution environment was a local-port `EACCES` while Playwright connected to Vite; Vite itself reported ready and Chrome launched. The successful ordinary-shell run establishes that the suite can run under the user's normal Windows token, but does not identify the specific policy responsible for sandbox socket denial.

The matched Release comparison is complete as a data collection, but **A5 is not accepted**. The previous analysis treated the 100 ms settle-confirmation time as the animation endpoint. Its 7/5 candidate intervals therefore are not confirmed animation stalls. The raw PresentMon rows can be reclassified against a conservative confirmation-tail boundary, but the harness did not save the timestamp of the last style mutation, so the actual animation endpoint remains unknown. A5 and C1 remain `[ ]`.

## Builds and conditions

Both binaries were built locally with the same C1 working-tree implementation and build configuration; the A5-off baseline was made by temporarily reversing only the A5 frontend/fixture diff, then restoring the candidate. No C1 source changed between builds.

| Build | EXE SHA-256 | `isle_lib.dll` SHA-256 |
|---|---|---|
| A5-off baseline | `00AE8D2837E18405515BCCB28EF1BA2479E68CCD2403D67B6822EB3E8B2FA336` | `A245133252EAFF80D0141B892AC4D1A5CA019C26484CEA6BE11A5F7896AC0208` |
| A5 candidate | `0F1D57468F384642D5C6A11530DBF8446AEB64DC6155A4B949B207BF2A1A9E4B` | `754349D12F1CF194A920A8FCB00149AEE7B7A5F1132B3ADE8192DDD29E912FAC` |

Three corrected Release rounds were captured per build, with animation-only, controls-only, and combined matrices in each round. The display/HWND sampler identified `\\.\DISPLAY2`, scale factor 1.0; both active display paths reported 200 Hz. The active media session was paused NetEase Cloud Music, system volume began and ended at 33%, and media and volume commands succeeded. PresentMon captures had complete start/stop markers and were non-elevated. Each animation/combined matrix exercised 16 expand/collapse transitions per round (48 per build and mode).

The first attempted A/B set is retained but excluded: the harness used media command labels that did not map to the native `control_media` command. The harness mapping was corrected before the six reported rounds; this changed only the benchmark script, not application code.

## Results

`MsBetweenDisplayChange` is the time a prior frame remained displayed. The benchmark records the first inline-style mutation as the start, then polls until the target style has remained unchanged for 100 ms. `animationEnd` is the later poll snapshot, not the last style mutation or confirmed animation endpoint. The previous analyzer used that snapshot as the end and therefore included the stability-confirmation tail in the purported active interval.

| Scenario | Build | Median of three round start→settle-confirmation P50s | Previous PM P95* | >50 ms, inside broad window* | >50 ms, crossing its boundary* |
|---|---|---:|---:|---:|---:|
| Animation only | A5-off | 348.0 ms | 5.05 ms | 0 / 48 | 47 |
| Animation only | Candidate | 528.0 ms | 5.03 ms | 7 / 48 | 43 |
| Combined | A5-off | 348.5 ms | 5.04 ms | 4 / 48 | 42 |
| Combined | Candidate | 521.1 ms | 5.03 ms | 5 / 48 | 42 |

The duration column is the median of the three per-round medians from first-style-change to settle-confirmation, not animation duration or physical display latency; it includes at least the 100 ms stability wait and polling delay. The earlier `522.4 ms` animation-only candidate figure was round 2's median; the three-round median is **528.0 ms**. It must not be compared directly to configured CSS timings (420 ms expand, 360 ms collapse, and 240 ms hover). `*` PresentMon time alignment and window-edge inclusion were incorrect in this earlier analysis; these columns are preserved only as superseded historical output and are not used for acceptance.

The table's “fully inside” and “crossing” labels reproduce the previous, too-wide interval ending at the settle-confirmation snapshot and using the Present start as display time; do not interpret them as animation classifications. Rechecking the previously “fully inside” >50 ms rows against `confirmedEnd - 100 ms`, without the corrected display-time alignment, gives:

| Scenario | Build | Before the 100 ms tail | Fully in the 100 ms confirmation tail* | Overlaps that tail |
|---|---|---:|---:|---:|
| Animation only | A5-off | 0 | 0 | 0 |
| Animation only | Candidate | 0 | 0 | 7 |
| Combined | A5-off | 0 | 3 | 1 |
| Combined | Candidate | 0 | 1 | 4 |

“Before the tail” means the PresentMon interval ends no later than 100 ms before the confirmation snapshot; it does not prove CSS was still animating then. `*` The 100 ms boundary is estimated from the confirmation snapshot; the harness did not persist the precise last-mutation timestamp or the poll-check timestamp. Rows in the tail-overlap column cannot be classified without those timestamps. Rows fully in the confirmation tail are consistent with a period after styles stopped changing, but do not independently establish the exact rendered-animation endpoint. Thus the old 7/5 candidate counts do **not** establish seven/five animation stalls; the maximum supported statement is that those rows intersect the confirmation tail. The new counts are interval counts, not distinct failed actions. Active-window PresentMon P95 is similar between builds but cannot answer this endpoint question. No ETW wait-stack capture or direct IPC-per-transition count was collected.

Media command timing was similar at the median but had long tails in both builds; the six runs are insufficient for a stable latency distribution. HWND samples stayed on DISPLAY2, although they recorded two distinct rectangles and do not directly count `SetWindowPos` calls or interaction-region IPC. The runs do not cover the full four-edge/floating, focus, manual-hide, artwork, and reduced-motion matrix, and no subjective native hand-feel sign-off was recorded. No performance or visual-quality benefit is claimed.

## Artifacts and next validation

Uploaded analyzer and joined results: [`analysis.json`](analysis.json) and [`analyze.mjs`](analyze.mjs). The analyzer consumes the original PresentMon CSV, status JSON, bounds CSV, and per-mode JSON files in `dist/performance/c1-2026-10-12/`; these source captures remain local and are not included with the two uploaded artifacts. The joined JSON keeps summaries and >50 ms rows, but not every PresentMon row or each mutation timestamp, so it is insufficient to independently reconstruct an exact animation endpoint. The baseline and candidate executable/DLL copies and isolated media caches are in `dist/performance/a5-2026-10-12/{baseline,candidate}/`.

## Corrected one-round PresentMon boundary capture

The analyzer and bounded capture harness are in `scripts/analyze-a5-animation-capture.mjs` and `scripts/run-a5-animation-capture.ps1`. The capture used the saved binaries above without rebuilding them, with 16 animation-only transitions per build. It recorded first style mutation, last style mutation, and the separate 100 ms settle-confirmation time. The complete PresentMon CSV, status metadata, HWND/monitor/QPC samples, and transition JSON are retained under `dist/performance/a5-boundary-2026-10-12/{baseline,candidate}/`; the joined result is `dist/performance/a5-boundary-2026-10-12/paired-analysis.json`.

The first diagnostic attempt exposed a clock mismatch: PresentMon's timezone-free `TimeInDateTime` string was 16 hours ahead of the same-run UTC stamps; applying the captured UTC+8 timezone still left an 8-hour mismatch. The wall-clock conversion cause remains unresolved. That attempt was discarded. The retained capture uses `--qpc_time`; each PresentMon start QPC is mapped to UTC using the window probe's paired `Stopwatch.GetTimestamp()`/UTC samples (10 MHz QPC, requested 8 ms sampling; actual QPC anchor spacing is recorded). `MsUntilDisplayed` is then added to the mapped Present start, and `MsBetweenDisplayChange` defines the preceding display interval. The analyzer selects intervals by intersection (`intervalStart < windowEnd && displayedAt > windowStart`), so an interval ending just outside the window is retained. This uses the PresentMon v2.6.0 metric semantics and avoids ambiguous wall-clock conversion. [PresentMon v2.6.0 metrics source](https://github.com/GameTechDev/PresentMon/blob/v2.6.0/IntelPresentMon/CommonUtilities/mc/MetricsCalculatorAnimation.cpp), [QPC output option](https://github.com/GameTechDev/PresentMon/blob/main/PresentMon/CommandLine.cpp), [CSV writer](https://github.com/GameTechDev/PresentMon/blob/main/PresentMon/CsvOutput.cpp).

The HWND sampler identified `\\.\DISPLAY2` in both runs. Swapchain mapping is a unique temporal correlation, not a direct HWND field in PresentMon: `0x2764A1A3230` matched all 16 baseline style-mutation intervals, and `0x24B251BB1F0` matched all 16 candidate intervals; each was the only swapchain with presents across all 16 transitions (`VidPnSourceId=0`, `LayerIndex=0`). The raw PresentMon capture targeted the WebView2 GPU process, so its process ID is recorded alongside each row. Baseline and candidate CSV SHA-256 values are `94F4C44CE6C22D530A2C6FC0E0A4C84BC144C4BA653AD092AFC5032B37196E96` and `2784CA24D58DE536A30F21C28F92AB6522209E1692660D1C97A94F1A6091B263` respectively. The capture status confirms PresentMon start/stop markers; its process exit code was not exposed by `Start-Process`, so completion is evidenced by those markers and non-empty hashed output.

| Build | Main swapchain intervals intersecting the transition span | `>50 ms` total | Fully within first→last style-mutation interval | Fully within last-mutation→confirmation tail | Crosses a transition/tail boundary |
|---|---:|---:|---:|---:|---:|
| A5-off baseline | 780 | 17 | 0 | 1 | 16 |
| A5 candidate | 1,272 | 17 | 0 | 0 | 17 |

Among fully-contained active style-mutation intervals, the worst `MsBetweenDisplayChange` was 10.05 ms baseline and 15.02 ms candidate; neither run had a `>50 ms` interval wholly inside that measured span. The long intervals in this single pair all straddle a transition boundary or the preceding confirmation tail. The baseline had one 75.02 ms interval fully inside the confirmation tail. This narrows the old 7/5 claims: they are not established animation-internal stalls. The one-round result also does not prove absence of intermittent stalls or a performance improvement; its sample count and duration are diagnostic only. Some PresentMon rows lacked `MsUntilDisplayed` (39 baseline, 25 candidate) or a display interval (2 each) and cannot be assigned to a display-time interval.

The original three-round `528.0 ms` candidate median remains a first-style-mutation-to-settle-confirmation span, not animation duration. The `522.4 ms` figure remains the second round alone. This capture does not close the separate C1 95/55 ms findings, include ETW wait stacks, or satisfy the full A5 Release/visual/interaction acceptance. C1 and A5 remain `[ ]`.

## Script review and one-round combined follow-up

The analyzer now accepts mode, capture suffix, and input directory, verifies raw CSV and local saved binary hashes, records input hashes/clock anchors, and rejects ambiguous main-swapchain selection. For each long interval it records the exact overlap length with each first-to-last mutation window, rather than dismissing the whole interval because it crosses a boundary. Numeric millisecond timestamps are retained alongside the readable ISO timestamps so sub-millisecond arithmetic can be reproduced. The first-at-or-after QPC anchor is used for conversion; UTC/mutation wall-clock stamps have millisecond resolution and the probe reads UTC then QPC, so boundary and overlap values remain approximate at that resolution.

The matrix script's old `clickAt` was stamped before preparation/CDP dispatch; it was unsuitable for measuring click response. New captures stamp immediately before `element.click()` within the page, and subtract browser `performance.now()` from the first mutation. Confirmation is also stamped on the page, instead of pairing a Node UTC timestamp with browser performance time. This is a programmatic DOM click, not physical input latency. The earlier animation-only captures have no dispatch stamp, so their click-delay result is explicitly unavailable; the earlier preparation timestamp is retained separately. Their classification counts reproduce the previous report unchanged.

The same saved baseline and candidate binaries were captured once more with `combined` mode, 16 transitions each, on DISPLAY2. The media command burst and continuous volume sequence run concurrently with the beginning of the animation sequence. There are 5 media/query and 6 volume calls on each side; calls returned success, final state was paused/33%. NetEase returned position/duration of zero, so command success does not establish a functional seek check or fulfill the other-player acceptance.

| Combined diagnostic | baseline | candidate |
|---|---:|---:|
| Fully within mutation window, `>50 ms` | 0 | 0 |
| Confirmation-tail, `>50 ms` | 1 | 0 |
| Boundary-crossing, `>50 ms` | 14 | 17 |
| Long-interval overlap with active windows, maximum | 37.6 ms | 38.2 ms |
| Sum of long-interval overlap with active windows | 151.1 ms | 192.8 ms |
| Programmatic click→first mutation P50 / maximum | 0.4 / 1.2 ms | 0.4 / 1.1 ms |
| Actual command-envelope overlap with transitions | 1–4 | 1–3 |

The longest active overlap on both sides occurs at the first transition, inside a roughly 130 ms interval that begins before the style mutation and ends about 38 ms after it. This is a remaining onset/presentation observation, not proof of a 130 ms animation-internal stall or of its cause. Cross-boundary overlap must not be discarded as entirely idle. The summed overlap is display-interval intersection time, not measured thread-blocked time. Different animation durations mean the same single command burst overlaps a different number of transitions, so this is limited diagnostic coverage rather than a matched all-16-transition concurrent stress test.

The combined main swapchains were baseline `0x2761A5EC000` (GPU PID [omitted]) and candidate `0x249250799C0` (GPU PID [omitted]), each correlated with all 16 transitions. Full raw CSV hashes are `08DF2ECB0F3FCBBDF016C7E14A35312DE4CF55E1EAFBAB01D6CA4268A64AA234` and `8E99B185D6BA193BF90D9554E924B1A8DDCAAB80B5316D0FF3D00DACB1479C80`. `combined-paired-analysis.json` retains command times and per-transition overlaps. On the old animation-only captures, long intervals overlap active windows by up to 39.2/35.8 ms respectively; their zero fully-contained count likewise does not establish zero onset delay.

The review package includes the measurement scripts, both joined JSONs, and complete CSV/status/bounds/transition inputs. Replay can verify the raw CSV and recompute the classifications without the saved EXE/DLL; that mode explicitly labels binary hashes as capture metadata only. The local analysis separately verified the actual saved binaries. C1's old 95/55 ms classifications still need correction with their own source inputs. Native interaction, IPC/resize counts, CPU/GPU/resource costs, other-player checks, and full Release acceptance remain pending. Animation parameters remain unchanged and Spring migration is deferred; C1/A5 stay `[ ]`.


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.
