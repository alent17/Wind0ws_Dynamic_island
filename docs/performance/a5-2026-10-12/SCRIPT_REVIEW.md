# Measurement review and replay

The new analyzer, capture wrapper, and mutation recorder were reviewed against the complete local raw inputs. The old animation-only classification counts reproduce unchanged. A separate verifier reads raw CSV/QPC anchors and transition JSON without importing the analyzer; all four build/scenario checks pass for row counts, classifications, active overlap totals/maxima and click-dispatch differences. See `raw-recompute-audit.json`.

Corrections made during review:

- Replace preparation-time `clickAt` with a page-side click-dispatch stamp for new measurements. Earlier captures explicitly have no click-response measurement.
- Stamp confirmation on the page, so its wall/performance time pair comes from the same clock.
- Record exact active/confirmation overlap per long interval and retain numeric timestamps alongside ISO strings.
- Verify local EXE/DLL and CSV hashes; retain matrix/bounds hashes and clock-anchor details.
- Reject ambiguous main-swapchain selection; preserve command envelopes and identify which transitions overlap them.
- Accept portable input directories for replay. A replay without binaries explicitly uses only the recorded binary hashes.

One-round combined results are in `combined-paired-analysis.json`, with raw inputs retained locally in `dist/performance/a5-boundary-2026-10-12/`. Each side has 16 transitions; both have zero fully-contained active `>50 ms` intervals, but long cross-boundary intervals overlap active windows by up to 37.6/38.2 ms. Command envelopes overlap baseline transitions 1–4 and candidate 1–3, so this is limited concurrent coverage. Command envelopes include CDP/harness round-trip time; they are not native worker execution traces. Programmatic click-delay P50 is 0.4 ms on both sides, not physical input-to-display latency. A5/C1 remain unchecked.

The probe requested an 8 ms sleep. Actual QPC anchor spacing in the combined captures had P50 15.53/15.59 ms and maximum 35.52/32.18 ms. Mapping uses actual QPC values, not the requested sleep duration. UTC/mutation timestamps have millisecond resolution and the probe reads UTC before QPC; values near boundaries have that measurement uncertainty.

The review ZIP contains the scripts, both analysis JSONs and every CSV/status/bounds/transition input needed for recomputation. It excludes saved application binaries. From its extracted root, use Node.js with no added packages:

```powershell
node scripts/analyze-a5-animation-capture.mjs animation-only pair-r1 raw --metadata-only-binaries
node scripts/analyze-a5-animation-capture.mjs combined combined-r1 raw --metadata-only-binaries
node verify.mjs raw
```

This rewrites the package's analysis JSONs and audit result from the bundled raw files. CSV hashes are verified; binary verification is explicitly labeled metadata-only. Live capture additionally needs the saved Release binaries, PresentMon, Playwright, and local WebView2 CDP access. Full native interaction, IPC, resource-cost and other-player checks remain pending. The application animation parameters were not changed; Spring migration remains deferred.
