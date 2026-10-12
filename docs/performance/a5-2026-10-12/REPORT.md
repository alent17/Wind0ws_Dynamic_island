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

| Scenario | Build | Start→settle-confirmation P50 | PresentMon P95 in the previous broad window | >50 ms, inside broad window | >50 ms, crossing its boundary |
|---|---|---:|---:|---:|---:|
| Animation only | A5-off | 348.0 ms | 5.05 ms | 0 / 48 | 47 |
| Animation only | Candidate | 522.4 ms | 5.03 ms | 7 / 48 | 43 |
| Combined | A5-off | 348.1 ms | 5.04 ms | 4 / 48 | 42 |
| Combined | Candidate | 521.0 ms | 5.03 ms | 5 / 48 | 42 |

The duration column is the probe's first-style-change to settle-confirmation span, not animation duration or physical display latency; it includes at least the 100 ms stability wait and polling delay. It must not be compared directly to the configured CSS timings (420 ms expand, 360 ms collapse, and 240 ms hover).

The table's “fully inside” and “crossing” labels reproduce the previous, too-wide interval ending at the settle-confirmation snapshot; do not interpret them as animation classifications. Rechecking the previously “fully inside” >50 ms rows against `confirmedEnd - 100 ms` gives:

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

The next useful capture must log the first style mutation, every/last style mutation or a trustworthy transition-finished event, and the separate 100 ms confirmation time. Recalculate the interval/action counts and rates per active animation second before deciding whether remaining long intervals merit deeper attribution. Keep C1's existing 95/55 ms gaps unresolved and do not use this A5 comparison to close C1.
