# Matched Release Benchmark C

This kit injects a shared, opt-in adapter into isolated source copies. It does
not edit the application's working-tree source. See the measured protocol and
results in `docs/performance/m3-release-c-2026-10-09/`.

The corpus is 100 fixed real-player IDs plus one warmup song. A locally backed-up
NetEase queue supplies IDs and durations. The player preparation helper uses the
existing project's NetEase control adapter and requires a player launched with
local port 9223; formal measurements disconnect that interface and use SMTC.
Queue preparation changes the active playlist. Run it only with the user's
authorization and restore the backed-up playlist afterward.

1. Prepare a fresh source pair with `python scripts/benchmark-c/prepare-sources.py <new-directory>`.
2. Link the existing `node_modules`, build the production frontend and native
   Release in each copy, and record binary/source hashes. Both sides retain M2.
3. Prepare the corpus/queue/settings JSON under the dated local performance
   directory. `freeze-covers.mjs` verifies and freezes artwork bytes; an unavailable
   HD source must be recorded explicitly. Preserve the full original queue and
   player state locally; do not publish the unrelated playlist backup.
4. Run `serve-covers.mjs`, then `prewarm-player.mjs` outside formal measurements.
5. Set `ISLE_BENCH_PILOT=0` and a new `ISLE_BENCH_BATCH` directory. Run `run.mjs`
   with label/variant pairs `before-r1 before`, `after-r1 after`, `after-r2 after`,
   `before-r2 before`, `before-r3 before`, `after-r3 after`, sequentially.
6. Run `analyze.py <performance-root> <batch>` only after all six finish. It checks
   input identities, completion, DPI/window dimensions and unchanged renderer PIDs.
7. Run `diagnose.mjs <before|after> <continuous|reopen>` in four separate fresh
   diagnostic directories; `quality.mjs` compares the actual displayed image
   sources from continuous diagnostics. These passes are excluded from formal
   CPU/memory measurements. `plot.py <root> <batch> <output>` exports the curves.
8. Restore with `prepare-player.mjs restore` and stop the owned replay/native
   processes. Report regressions and limits before deciding whether M3 is accepted.

The scripts currently target the dated test directory and Windows environment
used in this report. They are a reproducible experiment kit, not an unattended
cross-platform test runner. Raw binaries, artwork, profiles, heaps and private
playlist backups remain under ignored `dist/performance`.

The plot exporter requires Python and matplotlib (`python -m pip install
matplotlib`); Node tools use the repository's installed Playwright and sharp.
Cached asset images may taint a Canvas. The opt-in screenshot check keeps browser
security enabled and compares the full interior against a browser-rendered source
reference with RMS tolerance 8 RGB levels (or three exact probes within 2 levels).
This accounts for compositor scaling differences; it is a diagnostic similarity
check, not proof that every displayed pixel is identical.

Buffer probes count known lexical allocations; they exclude decoder scratch and
WebView decoded-image/GPU allocations. Paint timestamps are receipt of draw-complete
events, not compositor presentation. Quality lists decoded display-source pixels
at a common raster size and actual CSS-sized Canvas screenshots separately. A five-minute recovery
curve alone cannot establish long-term GPU stability or prove a memory leak.
